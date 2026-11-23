# Limits and non-goals

Who this is for: an integrator deciding whether this crate fits a dashboard, or a reviewer checking stated limits against code.

## Hard limits in code

| Area | Limit | Source |
| --- | --- | --- |
| Line chart | 16 series drawn, extras reported as `+N` in the legend | `widgets/line_chart.rs` |
| Line chart legend | Shown only with more than one series, height above 4 and width above the legend gutter plus 24 cells | `widgets/line_chart.rs` |
| Table header | 20 display cells | `widgets/table.rs` |
| Table cell | 40 display cells | `widgets/table.rs` |
| Status column | 2 cells when any row has a status marker | `widgets/table.rs` |
| Gauge label | 12 display cells | `widgets/gauge.rs` |
| Gauge right text | 8 display cells | `widgets/gauge.rs` |
| Frequency bar | Count clamped to 65535 for bar height | `widgets/frequency.rs` |
| Overlay | Three quarters of the area, at least 10 by 5 cells when the area allows | `widgets/overlay.rs` |
| Sparkline | Values scaled to 8 levels; the most recent area width samples are drawn | `widgets/spark.rs` |
| Model row | 16 cells for the name and 10 for the count, 27 cells total | `model.rs` |
| Demo key polling | 100 ms, only in `run::run_demo` | `run.rs` |

## Approximations

- Display width uses a local table for common wide and combining ranges. It is not full Unicode grapheme segmentation or a full width database. Some scripts measure imperfectly.
- The privacy mask uses a deterministic fold into an eight digit hexadecimal label. It collides in principle and is not cryptographic.
- `fmt_count` uses decimal k, M and G suffixes above 999; `fmt_bytes` uses binary K, M and G suffixes above 1023. They are display helpers, not unit conversions for billing.

## Non-goals

- No binary, daemon, installer, config file, plugin system or network client.
- No persistence, database, cache or log rotation. There is nothing to back up beyond source and lockfile.
- No authentication, authorisation or secret storage.
- No full screen editor, mouse handling or event framework. The demo loop is a sample, not a runtime.
- No 1.0 API promise and no crates.io publication at version 0.1.0.

## Behaviour under bad input

Text is escaped rather than passed through: unknown controls and invisible characters become `?`. Numbers are clamped rather than rejected. Empty lists and zero-sized areas draw nothing and return an empty visible range. This is fail-closed for text and fail-safe for layout; there is no access control in the crate, so there is no fail-open permission path. See [security model](security.md).
