# Quickstart

Who this is for: a developer with a Rust checkout who wants one visible result in under ten minutes.

Each shell block below is executed by `make docs-check`. A line starting with `# expect:` is the assertion; the driver removes it before running the block.

## 1. Confirm the checkout

From the repository root, check that this is the crate you expect.

```sh
test -f Cargo.toml && grep -q '^name = "fw-tui"' Cargo.toml && echo 'PASS source checkout'
# expect: PASS source checkout
```

Expected output:

```text
PASS source checkout
```

## 2. Run the default-feature tests

The model and sanitizer have no terminal dependency, so this step works without a tty or a feature flag.

```sh
out="$(cargo test --locked 2>&1)"; printf '%s\n' "$out" | grep -F -m1 'test result: ok.'
# expect: test result: ok.
```

Expected output: a line beginning with `test result: ok.`. On the checked toolchain the run reports 13 unit tests and 21 doc tests. All-feature counts are 93 unit tests and 166 doc tests; see [CLI reference](cli.md).

## 3. Proof without a terminal

The gallery example has a `--once` mode that renders a fixed 100 by 30 cell frame as plain text. This is the proof step: it asserts the rendered title rather than relying on an interactive screen.

```sh
cargo run --locked --features tui --example gallery -- --once | grep -F 'fw-tui gallery' | sed 's/.*fw-tui gallery.*/PASS gallery renders/'
# expect: PASS gallery renders
```

Expected output:

```text
PASS gallery renders
```

The full frame also contains `Overview │ Table │ Charts`, `peer-a`, `*rtt` and `[privacy: 0]`. The frame is deterministic, so those strings are stable.

## Next steps

- Interactive view: `cargo run --locked --features tui --example gallery` in a terminal, then `q` to quit. Keys are listed in [CLI reference](cli.md).
- Deeper reading: [install](install.md), [configuration](configuration.md), [architecture](architecture.md).
- Verification: `make check` runs lint, tests and these proof blocks.
