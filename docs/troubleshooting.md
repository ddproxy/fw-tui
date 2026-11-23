# Troubleshooting

Who this is for: a developer or operator who has a failing command and wants the exact message and fix.

| Symptom or exact message | Cause | Fix |
| --- | --- | --- |
| `error[E0433]: cannot find \`theme\` in \`fw_tui\`` | Widget modules are behind the optional feature. | Add `--features tui` to cargo commands or set `fw-tui = { path = "../fw-tui", features = ["tui"] }` in your manifest. |
| `error: cannot update the lock file ... because --locked was passed to prevent this` | Dependencies changed without a committed lockfile update. | Run `cargo generate-lockfile`, review the diff, commit `Cargo.lock`, then rerun with `--locked`. |
| `error: docs missing code token: --once` from `make docs-check` | A flag or key was added or removed in code without a docs update. | Update the docs table in `cli.md` and the drift list in `scripts/check_docs.sh`, then rerun the check. |
| `error: broken link: docs/foo.md -> bar.md` from `make docs-check` | A docs page was renamed or moved. | Fix the relative link and ensure `mkdocs.yml` lists the new page. |
| `error: <file>: sh block without # expect:` from `make docs-check` | A shell block was added to the quickstart without an assertion. | Add a `# expect:` line with the output the block must print, or move the block to a reference page. |
| `make docs-check` reports a proof block failure | The example output changed, or the build failed. | Read the reported output. Fix the code if the build failed, or update the expected line if the output legitimately changed. |
| The interactive gallery leaves the terminal in raw mode | The process was interrupted before its cleanup ran. | Run `reset` in the terminal. The library demo `run_demo` uses `TerminalGuard`, which restores state on drop and on panic. |
| `--once` prints a frame but no colour | Non-interactive mode always renders monochrome. | Nothing to fix. Interactive mode uses `Theme::auto()`, which selects monochrome when `NO_COLOR` is present. |

## Reading check output

`make docs-check` prints one `error:` line per failure and exits with status 1. It checks, in order: required files, relative links, mkdocs navigation, quickstart proof blocks, config examples, code token drift and a short style pass. `bash scripts/check_workflows.sh` prints `workflow checks passed` on success and `error:` lines on failure.

## When to regenerate the lockfile

Only when dependencies or features change. Do not regenerate to work around a docs or lint failure. All cargo invocations in CI use `--locked`; [operations](operations.md) has the upgrade and rollback steps.
