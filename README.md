# fw-tui

`fw-tui` is a small Rust library of reusable terminal dashboard widgets for ratatui-based TUIs.
It provides a pure model/sanitizer that build without terminal dependencies and, behind the `tui`
feature, rendering widgets and terminal helpers.

## Features

- `tui` (off by default): enables ratatui/crossterm widgets, drawing helpers, terminal guard, and event loop.

## Widgets

Data-agnostic widgets take plain data (strings, numbers, slices), not application types:

- **Theme**: semantic styles (`title`, `label`, `dim`, `good`, `warn`, `bad`, `accent`, selected/header/border) with dark and `NO_COLOR`-friendly monochrome variants.
- **Panel**: bordered block with title and optional right-aligned hint/title.
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

All widgets sanitize untrusted strings, clamp hostile numbers, truncate by display width without splitting chars, and tolerate tiny/zero areas without panicking.

## Example

```sh
cargo run --features tui --example gallery -- --once
cargo run --features tui --example gallery
```

## Testing

```sh
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_TARGET_DIR=/srv/work/target-ft
cargo test --features tui
cargo test
```
