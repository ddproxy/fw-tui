# Install

Who this is for: a developer adding this crate to a Rust project or reviewing its source before use.

## One path

There is no binary, installer or crates.io release. The crate sets `publish = false` and is consumed from source:

`git clone https://github.com/ddproxy/fw-tui.git && cd fw-tui`

Verify before trusting it: read `Cargo.toml` and `src/`, then run `cargo test --locked`. Rust 1.99 or newer is required; edition is 2021.

## Manual alternative

Add a path dependency to your own `Cargo.toml` and point it at the checkout.

```toml
[dependencies]
fw-tui = { path = "../fw-tui" }
```

The model and sanitizer need no terminal library. To get widgets, theme, text helpers, the terminal guard and the demo loop, enable the optional feature:

```toml
[dependencies]
fw-tui = { path = "../fw-tui", features = ["tui"] }
```

The `tui` feature is off by default. It pulls ratatui 0.30 and crossterm 0.29; the base crate has no terminal dependency.

## What installs with it

Nothing outside the checkout. There is no global state, config directory, cache, service or shell integration. Removing the checkout removes the crate.

## Verify the install

From your project, `cargo check --locked` compiles the model. With the feature enabled, `cargo check --locked --features tui` also compiles the drawing modules. The repository's own verification is `make check`; see [operations](operations.md).
