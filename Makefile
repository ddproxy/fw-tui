.DEFAULT_GOAL := help

CARGO_HOME ?= $(HOME)/.cargo
CARGO_TARGET_DIR ?= target
export CARGO_HOME CARGO_TARGET_DIR

.PHONY: help build test lint check docs-check workflow-check install

help:
	@printf '%s\n' \
	  'build          Build with default features and with all features' \
	  'test           Test with default features and with all features' \
	  'lint           rustfmt check, clippy -D warnings, rustdoc -D warnings' \
	  'docs-check     Docs files, links, proof, drift and config examples' \
	  'check          lint, test and docs-check' \
	  'workflow-check Workflow pins, permissions, licence sync' \
	  'install        Print the path dependency for this library'

build:
	cargo build --locked
	cargo build --locked --all-features

test:
	cargo test --locked
	cargo test --locked --all-features

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features --locked -- -D warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked

docs-check:
	bash scripts/check_docs.sh

check: lint test docs-check

workflow-check:
	bash scripts/check_workflows.sh

install:
	@printf '%s\n' 'This is a library. Add fw-tui = { path = "$(CURDIR)" } to your Cargo.toml.'
