# Configuration reference

Who this is for: an integrator choosing defaults and limits for a dashboard built with this crate.

There is no config file, environment loader or settings struct for the whole crate. Configuration is the Cargo feature, one environment variable read by the theme, and public fields on the option structs. Every default below comes from code and is exercised by a test or a checked example.

## Keys and defaults

| Key | Default | Meaning |
| --- | --- | --- |
| Cargo feature `tui` | off | Enables `render`, `run`, `terminal`, `text`, `theme` and `widgets`. |
| `NO_COLOR` | unset | If the variable is present, `Theme::auto()` returns `Theme::monochrome()`. Any value, including empty, counts as set. |
| `LineChartConfig.zoom` | `1` | Samples combined into one displayed point. Below one acts as one. |
| `LineChartConfig.agg` | `ZoomAgg::Max` | Bucket reduction. `ZoomAgg::Mean` averages finite non-negative samples. |
| `LineChartConfig.focused` | `0` | Highlighted series index. Out-of-range values clamp to the last shown series when drawn; the stored field is unchanged. |
| `Scale` | `Scale::Auto` | Scale for sparklines and bar panels. `Scale::Fixed(v)` is used only when `v` is finite and greater than zero. Auto scale has a floor of one. |
| `mask` level | caller-supplied | `0` returns escaped text, `1` abbreviates the middle, `2` gives a stable short label with an `id-` prefix. Level 2 and above behave alike. |
| `Gauge.fraction` | caller-supplied | Clamped at draw time: NaN or negative draws empty, above 1 or infinite draws full. |
| `Gauge.bar_width` | caller-supplied | Clipped to the area width minus two cells. |
| `CountdownGauge.full_secs` | caller-supplied | Values below one are treated as one for the fraction; remaining seconds clamp to full. |
| `ColWidth::Flex(n)` | caller-supplied | `n` below one is treated as one. `Fixed` and `Min` take the requested cell count. |
| `TableViewState` | `selected = 0`, `offset = 0` | Both are clamped on every render, including empty tables and zero-sized areas. |
| `Dashboard` names | empty | Rows start at count zero. `tick()` applies a demo step in the range 0 to 3 per row. |

## Defaults in the view model

`Dashboard::view` builds a view with title `fw-app top`, footer `p pause/resume   q quit`, and status `live, update N` or `paused: showing data from Ns ago`. Names pad to 16 display cells, counts to 10, for 27 cells per row. These strings are asserted by unit tests.

## Minimal example

Uses only the always-available modules and all defaults. The file [examples/config_minimal.rs](../examples/config_minimal.rs) is compiled by `cargo check --locked --example config_minimal`.

```rust
use std::time::Instant;

use fw_tui::model::Dashboard;

fn main() {
    let mut dashboard = Dashboard::new(&["peer-a", "host-1"]);
    dashboard.tick();
    let view = dashboard.view(Instant::now());
    assert!(view.plain_text().contains("peer-a"));
    println!("{}", view.plain_text());
}
```

## Full example

Enables `tui`, sets the chart options explicitly, uses a fixed scale, selects a privacy mask level and uses the theme that reads `NO_COLOR`. The file [examples/config_full.rs](../examples/config_full.rs) is compiled by `cargo check --locked --features tui --example config_full`.

```rust
use std::time::Instant;

use fw_tui::model::Dashboard;
use fw_tui::theme::Theme;
use fw_tui::widgets::{mask, render_line_chart, BarPanel, LineChartConfig, Scale, Series, ZoomAgg};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn main() {
    let theme = Theme::auto();
    let mut dashboard = Dashboard::new(&["peer-a"]);
    dashboard.tick();
    println!("{}", dashboard.view(Instant::now()).plain_text());

    let series = vec![Series::new("rtt", vec![Some(1.0), None, Some(3.0)])];
    let chart = LineChartConfig {
        zoom: 1,
        agg: ZoomAgg::Max,
        focused: 0,
    };
    let bars = BarPanel::new(vec![1.0, 2.0, 3.0]).scale(Scale::Fixed(4.0));
    let label = mask("peer-a", 2);
    assert!(label.starts_with("id-"));

    let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
    terminal
        .draw(|frame| {
            render_line_chart(frame.area(), frame.buffer_mut(), &series, &chart, &theme);
            bars.render(frame.area(), frame.buffer_mut(), &theme);
        })
        .unwrap();
}
```

## Validation

`make docs-check` compiles both examples with the real cargo validator and asserts that every documented key, feature, flag and version also exists in the source. [Limits](limits.md) lists the clamps applied when values are hostile.
