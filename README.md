# fw-tui

[![CI](https://github.com/ddproxy/fw-tui/actions/workflows/ci.yml/badge.svg?query=branch%3Amain)](https://github.com/ddproxy/fw-tui/actions/workflows/ci.yml?query=branch%3Amain)
[![CodeQL](https://github.com/ddproxy/fw-tui/actions/workflows/codeql.yml/badge.svg)](https://github.com/ddproxy/fw-tui/actions/workflows/codeql.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/ddproxy/fw-tui/badge)](https://scorecard.dev/viewer/?uri=github.com/ddproxy/fw-tui)
[![Release](https://img.shields.io/github/v/release/ddproxy/fw-tui)](https://github.com/ddproxy/fw-tui/releases/latest)
[![License](https://img.shields.io/github/license/ddproxy/fw-tui)](LICENSE)

`fw-tui` is a small Rust library of reusable terminal dashboard widgets for ratatui-based TUIs.
It provides a pure model/sanitizer that builds without terminal dependencies and, behind the `tui`
feature, rendering widgets and terminal helpers. The crate forbids unsafe code.

## Features

- `tui` (off by default): enables ratatui/crossterm widgets, drawing helpers, terminal guard, and event loop.

## Usage

The model and sanitizer have no terminal dependencies:

```rust
use std::time::Instant;

use fw_tui::model::Dashboard;

let mut dashboard = Dashboard::new(&["peer-a", "host-1"]);
dashboard.tick();
let view = dashboard.view(Instant::now());
assert!(view.plain_text().contains("peer-a"));
```

With `tui` enabled, widgets render into a ratatui buffer or `Frame`:

```rust
use fw_tui::theme::Theme;
use fw_tui::widgets::{render_tabs, Panel};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

let mut terminal = Terminal::new(TestBackend::new(30, 3)).unwrap();
terminal.draw(|frame| {
    Panel::new("Tabs").render(frame, frame.area(), &Theme::dark());
    render_tabs(frame.area(), frame.buffer_mut(), &["one", "two"], 0, &Theme::dark());
}).unwrap();
```

## Widgets

Data-agnostic widgets take plain data (strings, numbers, slices), not application types:

- **Theme**: semantic styles (`title`, `label`, `dim`, `good`, `warn`, `bad`, `accent`, selected/header/border) with dark and `NO_COLOR`-friendly monochrome variants.
- **Panel**: bordered block with title and optional right-aligned bottom title.
- **Header/Footer bars**: header with left/right text, footer with toggle chips (`[■ detail]`, `[privacy: 0]`).
- **Tabs**: labeled row with selection and truncation.
- **Table**: columns (fixed/min/flex/right), reversed header, selection/scroll, optional status dot, expandable selected row details, visible-range reporting.
- **StatusDot**: fixed one-cell state marker with text fallback.
- **Gauge / CountdownGauge**: horizontal fill bar with right-aligned text, drain/countdown mode.
- **Sparkline / BarPanel**: compact 8-level sparkline and filled sample bar panel with auto/fixed/shared scaling.
- **LineChart**: multi-series line plot with y ticks, x labels and zoom factor, boxed legend, autoscale floor, gaps for missing points, zoom aggregation.
- **FrequencyChart**: vertical category bar chart.
- **Overlay**: centered help/settings box.
- **PrivacyMask**: deterministic masking helper (none/abbreviate/token).
- **Text helpers**: `buffer_text`, width-aware truncate/pad, `fmt_duration`, `fmt_count`, `fmt_bytes`.

All widgets sanitize untrusted strings (including ANSI escape sequences, bidi overrides and
invisible format characters), clamp hostile numbers, truncate by display width without splitting
chars, and tolerate tiny/zero areas without panicking. Escaping is applied at the drawing boundary
as well as in constructors, so publicly constructed data cannot inject terminal controls. Unknown
configuration keys remain errors; credentials are never logged or stored.

## Example

```sh
cargo run --features tui --example gallery -- --once
cargo run --features tui --example gallery
```

## Testing

```sh
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_TARGET_DIR=/srv/work/target-ft
cargo test --locked
cargo test --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --locked
cargo fmt --all -- --check
```

`cargo test` also runs the doc tests: every public item has a compiling example. `cargo llvm-cov`
can be used for coverage when the `llvm-tools-preview` rustup component is available.

## Documentation

- [Documentation index](docs/index.md)
- [Security scanning](docs/security-scanning.md)
- [Changelog](CHANGELOG.md)

## Security

Report vulnerabilities through [GitHub private vulnerability reporting](https://github.com/ddproxy/fw-tui/security/advisories/new). See [SECURITY.md](SECURITY.md) for supported versions and response times, and [Security scanning](docs/security-scanning.md) for workflow checks and finding locations.

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
