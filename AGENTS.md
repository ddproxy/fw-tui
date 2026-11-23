# AGENTS.md

Who this is for: an AI agent or a human contributor working in this repository. Read this file before changing code or docs.

## Repo map

| Path | What lives there |
| --- | --- |
| `src/model.rs`, `src/sanitize.rs`, `src/width.rs` | Terminal-free model, escaping and private width helpers. |
| `src/render.rs`, `src/run.rs`, `src/terminal.rs`, `src/text.rs`, `src/theme.rs`, `src/widgets/` | Modules behind the optional `tui` feature. |
| `examples/gallery.rs` | Widget gallery; `--once` prints a plain-text frame, no flag starts the TUI. |
| `examples/config_minimal.rs`, `examples/config_full.rs` | Validated configuration examples used by the docs check. |
| `docs/` | Topic pages; `docs/index.md` is the index. |
| `scripts/check_docs.sh` | Docs structure, links, proof blocks, drift and example validation. |
| `scripts/check_workflows.rs`, `scripts/check_workflows.sh` | Workflow pin, permission and licence sync checks. |
| `.github/workflows/` | CI, audit, CodeQL, Scorecard, dependency review and release. |
| `Makefile` | Stable entry points listed by `make help`. |

## Commands

| Command | Result on success |
| --- | --- |
| `make help` | Lists targets. |
| `make build` | Builds default and all features. |
| `make test` | Tests default and all features, including doc tests. |
| `make lint` | rustfmt, clippy with `-D warnings`, rustdoc with `-D warnings`. |
| `make docs-check` | Prints `docs checks passed`. |
| `make check` | Lint, tests and docs checks. |
| `bash scripts/check_workflows.sh` | Prints `workflow checks passed`. |

Exit codes: all commands return 0 on success and 1 on failure. Check scripts print one `error:` line per problem. Cargo uses its own non-zero codes for compile or test failures.

## Conventions

- Rust edition 2021, MSRV 1.99. Unsafe code is forbidden and missing docs are denied at crate level.
- Clippy `all` is denied in `Cargo.toml`; CI adds `-D warnings`. Never introduce a warning.
- Every public item has a rustdoc example. Keep examples compiling with default and all features.
- The `tui` feature is off by default. Model changes must not pull in ratatui or crossterm.
- Escape untrusted text at constructors and at the drawing boundary. Clamp hostile numbers. Keep zero-sized-area tests passing.
- Keep each number identical across code, README and `docs/`: feature versions, clamps, key names and defaults.
- Do not write secret-shaped text into files. Describe placeholders in prose. The job controller scrubs key blocks, token assignments and bearer strings, which corrupts patches.
- Commit subjects are terse imperative phrases. Do not put personal names or job names in commits, docs or logs.
- Docs shell blocks that are executed must end with a `# expect:` line stating the output to assert. The driver removes that comment before running the block.

## Things not to touch without a reason

- `Cargo.lock`: regenerate only when dependencies or features change, and commit the result. CI uses `--locked`.
- `.github/workflows/` action pins: they are 40-character SHAs with version comments. `scripts/check_workflows.sh` enforces this.
- `deny.toml` licence list: keep it in sync with `allow-licenses` in `dependency-review.yml`.
- Badge URLs and the release workflow outputs: source archive, SBOM, checksums and attestation names are part of the verification commands.
- `publish = false` in `Cargo.toml` until a crates.io release is planned.

## Quickstart as a smoke test

Run these from the repository root. They are the same proof commands as [docs/quickstart.md](docs/quickstart.md) and are executed by `make docs-check`.

```sh
test -f Cargo.toml && grep -q '^name = "fw-tui"' Cargo.toml && echo 'PASS source checkout'
```

```sh
cargo test --locked
```

```sh
cargo run --locked --features tui --example gallery -- --once
```

Expected: `PASS source checkout`, a `test result: ok.` line for each test binary, and a plain-text frame containing `fw-tui gallery`, `peer-a`, `*rtt` and `[privacy: 0]`. Interactive mode needs a tty and quits on `q`; it is not part of the automated smoke test.
