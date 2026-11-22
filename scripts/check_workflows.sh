#!/usr/bin/env bash
# Validate workflow YAML, action pins, permissions, licence sync and Dependabot config.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/src"
cp "$root/scripts/check_workflows.rs" "$tmp/src/main.rs"
cat > "$tmp/Cargo.toml" <<'TOML'
[package]
name = "check-workflows"
version = "0.0.0"
edition = "2021"

[dependencies]
serde_yaml = "=0.9.34"
TOML

cargo run --quiet --manifest-path "$tmp/Cargo.toml" -- "$root"
