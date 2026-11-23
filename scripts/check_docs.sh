#!/usr/bin/env bash
# Check documentation structure, links, quickstart proof, config examples and code drift.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

errors=0
err() {
  printf 'error: %s\n' "$*" >&2
  errors=$((errors + 1))
}

required=(
  README.md AGENTS.md SECURITY.md CHANGELOG.md Makefile llms.txt
  docs/index.md docs/quickstart.md docs/install.md docs/configuration.md
  docs/operations.md docs/security.md docs/limits.md docs/troubleshooting.md
  docs/cli.md docs/architecture.md docs/security-scanning.md
  examples/config_minimal.rs examples/config_full.rs scripts/check_docs.sh
)
for file in "${required[@]}"; do
  [[ -f "$file" ]] || err "missing $file"
done

for file in README.md AGENTS.md SECURITY.md CHANGELOG.md docs/*.md; do
  [[ -f "$file" ]] || continue
  dir="$(dirname "$file")"
  while IFS= read -r target; do
    [[ -n "$target" ]] || continue
    target="${target%%#*}"
    case "$target" in
      http://* | https://* | mailto:*) continue ;;
    esac
    [[ -e "$dir/$target" ]] || err "broken link: $file -> $target"
  done < <(sed -nE 's/.*\]\(([^)]+)\).*/\1/p' "$file")
done

for page in docs/*.md; do
  grep -Fq "$(basename "$page")" mkdocs.yml || err "mkdocs.yml has no $(basename "$page")"
done

run_quickstart() {
  local file="$1" line in_sh=0 block="" want output
  local -a expected=()
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ $in_sh -eq 0 ]]; then
      if [[ "$line" == '```sh' ]]; then
        in_sh=1
        block=""
        expected=()
      fi
      continue
    fi
    if [[ "$line" == '```' ]]; then
      in_sh=0
      if [[ ${#expected[@]} -eq 0 ]]; then
        err "$file: sh block without # expect:"
      elif [[ -z "$block" ]]; then
        err "$file: empty sh block"
      elif ! output="$(bash -eu -c "$block" 2>&1)"; then
        err "$file: proof block failed: $output"
      else
        for want in "${expected[@]}"; do
          [[ "$output" == *"$want"* ]] || err "$file: expected '$want' in: $output"
        done
      fi
      continue
    fi
    if [[ "$line" == '# expect: '* ]]; then
      expected+=("${line#\# expect: }")
    else
      block+="$line"$'\n'
    fi
  done < "$file"
}
run_quickstart docs/quickstart.md

grep -Fq '# expect: PASS gallery renders' README.md ||
  err "README quickstart proof marker missing"

check_example() {
  local name="$1" output
  shift
  if ! output="$(cargo check --locked "$@" --example "$name" 2>&1)"; then
    err "example $name failed: $output"
  fi
}
check_example config_minimal
check_example config_full --features tui

docs_text="$(cat README.md docs/*.md AGENTS.md SECURITY.md CHANGELOG.md)"
code_text="$(cat Cargo.toml src/*.rs src/widgets/*.rs examples/*.rs)"
tokens=(tui NO_COLOR --once Esc 1.99 0.30 0.29 16 LineChartConfig ZoomAgg Scale ColWidth mask)
for token in "${tokens[@]}"; do
  grep -Fq -- "$token" <<<"$docs_text" || err "docs missing code token: $token"
  grep -Fq -- "$token" <<<"$code_text" || err "code missing documented token: $token"
done

style_files=(README.md AGENTS.md SECURITY.md CHANGELOG.md docs/*.md)
if grep -n '—' "${style_files[@]}"; then
  err "em dash found in docs"
fi
for filler in seamless robust cutting-edge effortless; do
  if grep -qi -- "$filler" "${style_files[@]}"; then
    err "filler word found in docs: $filler"
  fi
done

if [[ $errors -ne 0 ]]; then
  printf '%d docs check error(s)\n' "$errors" >&2
  exit 1
fi
printf 'docs checks passed\n'
