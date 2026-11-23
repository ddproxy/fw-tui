# Security scanning

Who this is for: a maintainer or reviewer who needs to know what each workflow checks and where its findings appear.

All checks run in GitHub Actions. The required status checks on pull requests are `test`, `Audit`, `CodeQL` and `dependency-review`.

| Workflow | Checks | Findings |
| --- | --- | --- |
| [CI](../.github/workflows/ci.yml) | Formatting, clippy with `-D warnings`, rustdoc with warnings denied, tests with default and all features, and docs checks; every cargo invocation uses `--locked`. | Checks tab on the commit or pull request. |
| [Audit](../.github/workflows/audit.yml) | `cargo deny check advisories bans licenses sources` on push, pull requests and daily. Licences and sources come from [deny.toml](../deny.toml). | Checks tab; full diagnostics in the workflow log. |
| [CodeQL](../.github/workflows/codeql.yml) | Rust static analysis with `build-mode: none` on main pushes, pull requests and weekly. | Security tab → Code scanning alerts. |
| [Scorecard](../.github/workflows/scorecard.yml) | OpenSSF Scorecard on main pushes and weekly; results are published. | Security tab → Code scanning alerts; the SARIF file is also an artifact kept for 5 days. |
| [Dependency Review](../.github/workflows/dependency-review.yml) | New dependency vulnerabilities at high severity or above, and licences outside `deny.toml`. | Checks tab on the pull request. |
| [Release](../.github/workflows/release.yml) | On `v*` tags, a CycloneDX JSON SBOM, SHA-256 checksums and a build provenance attestation for the source archive. | Release assets and GitHub attestations. |

## Local checks

`bash scripts/check_workflows.sh` validates workflow YAML, action pins, permissions, licence sync and Dependabot config before a pull request. `make docs-check` validates docs structure, links, quickstart proof, drift and the two configuration examples. `make check` adds formatting, clippy, rustdoc and tests.

## Verify a release

Replace `v0.1.0` with the tag you are verifying:

```sh
tag=v0.1.0
gh release download "$tag" --repo ddproxy/fw-tui \
  --pattern "fw-tui-${tag}-source.tar.gz" \
  --pattern "fw-tui-${tag}-sbom.json" \
  --pattern checksums.txt

gh attestation verify "fw-tui-${tag}-source.tar.gz" --repo ddproxy/fw-tui
sha256sum --check checksums.txt
```

`fw-tui-${tag}-sbom.json` is a CycloneDX SBOM; inspect it as JSON or in a CycloneDX viewer.

## Repository settings (owner steps)

These settings live in GitHub and cannot be set from files. The repository owner must:

- [ ] Enable secret scanning and push protection under Settings → Code security.
- [ ] Enable Dependabot alerts under Settings → Code security.
- [ ] Enable private vulnerability reporting under Settings → Code security.
- [ ] Protect `main` and require the status checks `test`, `Audit`, `CodeQL` and `dependency-review`; require branches to be up to date before merging.
- [ ] Leave code scanning default setup disabled; these workflows upload the code scanning results.
- [ ] Register the project for the OpenSSF Best Practices badge at <https://www.bestpractices.dev/>. Add the badge to the README only after registration.
- [ ] Publish the crate to crates.io and add a crates.io badge only after publication; `Cargo.toml` currently sets `publish = false`.
