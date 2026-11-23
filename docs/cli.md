# CLI and key reference

Who this is for: a developer running the gallery example or wiring the demo loop into an application.

## There is no installed binary

The crate publishes no executable. The only runnable target is the `gallery` example, which is gated behind the `tui` feature. All commands below run from the repository root.

## Commands

| Command | Needs a tty | Output |
| --- | --- | --- |
| `cargo test --locked` | no | 13 unit tests and 21 doc tests with default features. |
| `cargo test --all-features --locked` | no | 93 unit tests and 166 doc tests. |
| `cargo run --locked --features tui --example gallery -- --once` | no | A 100 by 30 cell plain-text frame ending with the footer `ready` and `[privacy: 0]`. |
| `cargo run --locked --features tui --example gallery` | yes | Interactive dashboard until `q`. |
| `cargo check --locked --example config_minimal` | no | Compiles the minimal configuration example. |
| `cargo check --locked --features tui --example config_full` | no | Compiles the full configuration example. |

## Flags

| Flag | Applies to | Meaning |
| --- | --- | --- |
| `--once` | gallery example | Prints one frame with `TestBackend` and exits. No raw mode, no alternate screen. |

The example checks whether any argument equals `--once`. Other arguments are ignored and the interactive mode starts. There are no config-file flags, no verbosity flags and no global cargo flags defined by this crate.

## Keys in the interactive gallery

| Key | Action |
| --- | --- |
| `q` | Quit and restore the terminal. |
| `?` | Toggle the help overlay. |
| Down arrow | Move the table selection down. |
| Up arrow | Move the table selection up. |

The gallery does not implement pause. It handles only those four keys; other keys are ignored.

## Keys in the library demo loop

`run::run_demo(stop, tick)` is a library function, not a binary. It uses these recognised key presses:

| Key | Action |
| --- | --- |
| `q`, `Esc`, `Ctrl-C` | Quit. Extra modifiers on `q` and `Esc` are accepted. |
| `p`, Space | Toggle pause. The view then shows a frozen copy with its age. |
| Anything else | Ignored. Key releases and repeats are also ignored. |

It polls for input every 100 ms and redraws on each loop. `tick` is supplied by the caller.

## Environment

| Variable | Default | Effect |
| --- | --- | --- |
| `NO_COLOR` | unset | When present, `Theme::auto()` selects the monochrome palette. Any value, including empty, counts as set. Explicit `Theme::dark()` and `Theme::monochrome()` ignore it. |

The `--once` mode always uses `Theme::monochrome()` and ignores `NO_COLOR`.

## What is not enforced

There is no authentication, policy engine, rate limit or input validator at the CLI layer. The example feeds fixed data. In an application, the caller owns validation before data reaches the widgets; the crate only guarantees escaping and non-panicking layout.

## Exit status

Normal exit from the gallery returns status 0. Terminal setup or draw errors are returned from `main` as an I/O error and produce a non-zero status with the underlying message. Check scripts in this repository return 0 on success and 1 on failure, printing `error:` lines for each problem.
