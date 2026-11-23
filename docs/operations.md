# Operations

Who this is for: a developer or release operator keeping a checkout healthy and moving between versions.

This crate is a library. Operating it means managing the source checkout, the lockfile and CI checks, not running a service.

## Upgrade

1. Read the [changelog](../CHANGELOG.md) and any `Requires:` line in the version you are moving to.
2. Fetch with `git pull --ff-only`.
3. Run `make check`. If a dependency moved, regenerate `Cargo.lock` with `cargo generate-lockfile`, review the diff and commit it.
4. Run `cargo test --all-features --locked`.

## Rollback

Check out the previous tag with `git checkout <tag>`, restore the matching `Cargo.lock`, then run `cargo test --locked`. There is no data migration to undo. Replace `<tag>` with a real tag such as `v0.1.0`; no credentials are involved.

## Backup and restore

Back up the source checkout and `Cargo.lock`. The crate has no runtime state, database, config directory, cache or log rotated on disk, so restoring those two files is sufficient.

## Logs and metrics

The library emits no logs and exposes no metrics endpoint. Observable signals are:

| Source | Signal |
| --- | --- |
| `cargo test --locked` | Unit and doc test counts, one `test result: ok.` line per test binary. |
| `make lint` | rustfmt, clippy and rustdoc exit status. |
| `make docs-check` | Docs structure, links, proof blocks, drift and example validation. |
| `bash scripts/check_workflows.sh` | `workflow checks passed` or `error:` lines. |
| CI | Required checks `test`, `Audit`, `CodeQL` and `dependency-review`. |

## Health checks

Run these before merging or tagging:

```sh
make help
make build
make test
make lint
make docs-check
bash scripts/check_workflows.sh
```

`make check` combines lint, tests and docs checks. See [AGENTS.md](../AGENTS.md) for the stable command list and exit codes.

## Verify a release

Requires `gh`, `sha256sum` and network access. Replace `v0.1.0` with the tag you are verifying.

```sh
tag=v0.1.0
gh release download "$tag" --repo ddproxy/fw-tui \
  --pattern "fw-tui-${tag}-source.tar.gz" \
  --pattern "fw-tui-${tag}-sbom.json" \
  --pattern checksums.txt

gh attestation verify "fw-tui-${tag}-source.tar.gz" --repo ddproxy/fw-tui
sha256sum --check checksums.txt
```

The SBOM is CycloneDX JSON. Full workflow details are in [security scanning](security-scanning.md).

## Common failures

Exact messages, causes and fixes are in [troubleshooting](troubleshooting.md). Terminal restoration and key handling are described in [CLI reference](cli.md).
