use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn mapping_as_set(value: Option<&Value>) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    if let Some(Value::Mapping(mapping)) = value {
        for (key, value) in mapping {
            let key = match key {
                Value::String(key) => key.clone(),
                other => format!("{other:?}"),
            };
            let value = match value {
                Value::String(value) => value.clone(),
                other => format!("{other:?}"),
            };
            set.insert(format!("{key}: {value}"));
        }
    }
    set
}

fn expected_permissions(workflow: &str, job: &str) -> Option<&'static [&'static str]> {
    match (workflow, job) {
        ("ci.yml", "test") => Some(&[]),
        ("audit.yml", "Audit") => Some(&["contents: read"]),
        ("codeql.yml", "CodeQL") => Some(&["contents: read", "security-events: write"]),
        ("scorecard.yml", "scorecard") => {
            Some(&["contents: read", "id-token: write", "security-events: write"])
        }
        ("dependency-review.yml", "dependency-review") => Some(&["contents: read"]),
        ("release.yml", "release") => {
            Some(&["attestations: write", "contents: write", "id-token: write"])
        }
        _ => None,
    }
}

fn check_uses(path: &Path, text: &str, errors: &mut Vec<String>) {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    for (line_number, line) in text.lines().enumerate() {
        let Some(position) = line.find("uses:") else {
            continue;
        };
        let rest = line[position + "uses:".len()..].trim();
        let (spec, comment) = match rest.split_once('#') {
            Some((spec, comment)) => (spec.trim(), Some(comment.trim())),
            None => (rest, None),
        };
        if spec.starts_with("./") {
            continue;
        }
        let Some((_, reference)) = spec.split_once('@') else {
            errors.push(format!("{name}:{}: {spec} is not pinned", line_number + 1));
            continue;
        };
        if reference.len() != 40 || !reference.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            errors.push(format!(
                "{name}:{}: {spec} is not pinned by a 40-character SHA",
                line_number + 1
            ));
        }
        if !comment.is_some_and(|comment| comment.starts_with('v') && comment.len() > 1) {
            errors.push(format!(
                "{name}:{}: {spec} has no trailing version comment",
                line_number + 1
            ));
        }
    }
}

fn deny_allow_list(root: &Path, errors: &mut Vec<String>) -> Option<BTreeSet<String>> {
    let text = fs::read_to_string(root.join("deny.toml")).ok()?;
    let mut in_licenses = false;
    let mut in_allow = false;
    let mut allowed = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_licenses = line == "[licenses]";
            in_allow = false;
        } else if in_licenses && line.starts_with("allow") && line.contains('[') {
            in_allow = true;
        } else if in_allow && line.starts_with(']') {
            in_allow = false;
        } else if in_allow {
            let license = line.trim_end_matches(',').trim().trim_matches('"');
            if !license.is_empty() {
                allowed.insert(license.to_owned());
            }
        }
    }
    for needle in [
        "yanked = \"deny\"",
        "unknown-registry = \"deny\"",
        "unknown-git = \"deny\"",
        "https://github.com/rust-lang/crates.io-index",
        "github = [\"ddproxy\"]",
    ] {
        if !text.contains(needle) {
            errors.push(format!("deny.toml is missing {needle}"));
        }
    }
    Some(allowed)
}

fn check_license_sync(
    docs: &BTreeMap<String, Value>,
    root: &Path,
    errors: &mut Vec<String>,
) {
    let Some(deny_allowed) = deny_allow_list(root, errors) else {
        return;
    };
    let Some(doc) = docs.get("dependency-review.yml") else {
        errors.push("dependency-review.yml is missing".to_owned());
        return;
    };
    let with = doc
        .get("jobs")
        .and_then(Value::as_mapping)
        .and_then(|jobs| jobs.get(&Value::String("dependency-review".into())))
        .and_then(|job| job.get("steps"))
        .and_then(Value::as_sequence)
        .and_then(|steps| steps.iter().find_map(|step| step.get("with")));
    let Some(with) = with else {
        errors.push("dependency-review.yml has no with block".to_owned());
        return;
    };
    let review_allowed: BTreeSet<String> = with
        .get("allow-licenses")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|license| !license.is_empty())
        .map(str::to_owned)
        .collect();
    if review_allowed != deny_allowed {
        errors.push(format!(
            "licence lists differ: deny.toml={deny_allowed:?}, dependency-review.yml={review_allowed:?}"
        ));
    }
    if with.get("license-configuration").and_then(Value::as_str) != Some("allow") {
        errors.push("dependency-review.yml must use license-configuration: allow".to_owned());
    }
    if with.get("fail-on-severity").and_then(Value::as_str) != Some("high") {
        errors.push("dependency-review.yml must use fail-on-severity: high".to_owned());
    }
}

fn check_dependabot(root: &Path, errors: &mut Vec<String>) {
    let Ok(text) = fs::read_to_string(root.join(".github/dependabot.yml")) else {
        errors.push(".github/dependabot.yml is missing".to_owned());
        return;
    };
    let Ok(doc) = serde_yaml::from_str::<Value>(&text) else {
        errors.push(".github/dependabot.yml is not valid YAML".to_owned());
        return;
    };
    let Some(updates) = doc.get("updates").and_then(Value::as_sequence) else {
        errors.push(".github/dependabot.yml has no updates list".to_owned());
        return;
    };
    for ecosystem in ["cargo", "github-actions"] {
        let Some(entry) = updates
            .iter()
            .find(|entry| entry.get("package-ecosystem").and_then(Value::as_str) == Some(ecosystem))
        else {
            errors.push(format!("dependabot.yml has no {ecosystem} update"));
            continue;
        };
        if entry
            .get("schedule")
            .and_then(|schedule| schedule.get("interval"))
            .and_then(Value::as_str)
            != Some("weekly")
        {
            errors.push(format!("dependabot.yml {ecosystem} schedule is not weekly"));
        }
        if entry.get("open-pull-requests-limit").and_then(Value::as_i64) != Some(5) {
            errors.push(format!("dependabot.yml {ecosystem} limit is not 5"));
        }
        let update_types: BTreeSet<&str> = entry
            .get("groups")
            .and_then(|groups| groups.get("minor-and-patch"))
            .and_then(|group| group.get("update-types"))
            .and_then(Value::as_sequence)
            .map(|types| types.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        for kind in ["minor", "patch"] {
            if !update_types.contains(kind) {
                errors.push(format!("dependabot.yml {ecosystem} does not group {kind} updates"));
            }
        }
    }
}

fn run(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let workflows_dir = root.join(".github/workflows");
    let Ok(entries) = fs::read_dir(&workflows_dir) else {
        errors.push(".github/workflows is missing".to_owned());
        return errors;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect();
    paths.sort();

    let mut docs = BTreeMap::new();
    let mut job_ids = BTreeSet::new();
    for path in &paths {
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let Ok(text) = fs::read_to_string(path) else {
            errors.push(format!("{name} is unreadable"));
            continue;
        };
        check_uses(path, &text, &mut errors);
        let Ok(doc) = serde_yaml::from_str::<Value>(&text) else {
            errors.push(format!("{name} is not valid YAML"));
            continue;
        };
        match doc.get("permissions") {
            Some(Value::Mapping(permissions)) if permissions.is_empty() => {}
            _ => errors.push(format!("{name} must declare top-level permissions: {{}}")),
        }
        let has_on = doc.as_mapping().is_some_and(|mapping| {
            mapping.contains_key(&Value::String("on".into()))
                || mapping.contains_key(&Value::Bool(true))
        });
        if !has_on {
            errors.push(format!("{name} has no on: triggers"));
        }
        let Some(jobs) = doc.get("jobs").and_then(Value::as_mapping) else {
            errors.push(format!("{name} has no jobs"));
            continue;
        };
        for (id, job) in jobs {
            let Some(id) = id.as_str() else {
                errors.push(format!("{name} has a non-string job id"));
                continue;
            };
            if job.get("name").and_then(Value::as_str) != Some(id) {
                errors.push(format!("{name} job {id} has no matching name"));
            }
            let expected = expected_permissions(&name, id).unwrap_or_default();
            let expected: BTreeSet<String> =
                expected.iter().map(|permission| (*permission).to_owned()).collect();
            let actual = mapping_as_set(job.get("permissions"));
            if expected.is_empty() && !actual.is_empty() {
                errors.push(format!("{name} job {id} grants unexpected permissions: {actual:?}"));
            } else if !expected.is_empty() && actual != expected {
                errors.push(format!(
                    "{name} job {id} permissions: expected {expected:?}, got {actual:?}"
                ));
            }
            if expected_permissions(&name, id).is_none() {
                errors.push(format!("{name} has unexpected job {id}"));
            }
            job_ids.insert(id.to_owned());
        }
        docs.insert(name, doc);
    }
    for required in ["test", "Audit", "CodeQL", "dependency-review"] {
        if !job_ids.contains(required) {
            errors.push(format!("missing required check name {required}"));
        }
    }
    check_license_sync(&docs, root, &mut errors);
    check_dependabot(root, &mut errors);
    errors
}

fn main() -> ExitCode {
    let root = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let errors = run(&root);
    if errors.is_empty() {
        println!("workflow checks passed");
        ExitCode::SUCCESS
    } else {
        for error in errors {
            eprintln!("error: {error}");
        }
        ExitCode::FAILURE
    }
}
