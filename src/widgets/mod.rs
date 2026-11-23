//! Data-agnostic terminal widgets.
//!
//! Requires the `tui` feature. Every widget takes plain strings and numbers, escapes untrusted
//! text, clamps hostile numbers and tolerates zero-sized areas.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{render_tabs, Panel};
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let theme = Theme::dark();
//! let mut terminal = Terminal::new(TestBackend::new(30, 3)).unwrap();
//! terminal
//!     .draw(|frame| {
//!         Panel::new("Tabs").render(frame, frame.area(), &theme);
//!         render_tabs(frame.area(), frame.buffer_mut(), &["one", "two"], 0, &theme);
//!     })
//!     .unwrap();
//! ```

pub mod bars;
pub mod frequency;
pub mod gauge;
pub mod line_chart;
pub mod overlay;
pub mod panel;
pub mod privacy;
pub mod spark;
pub mod status_dot;
pub mod table;
pub mod tabs;

pub use bars::{render_footer_bar, render_header_bar, ToggleChip};
pub use frequency::{render_frequency_chart, FrequencyBar};
pub use gauge::{CountdownGauge, Gauge};
pub use line_chart::{render_line_chart, LineChartConfig, Series, ZoomAgg};
pub use overlay::render_overlay;
pub use panel::Panel;
pub use privacy::mask;
pub use spark::{BarPanel, Scale, Sparkline};
pub use status_dot::{status_dot, DotState};
pub use table::{
    render_table, render_table_sorted, sort_column_at, sort_column_at_with_status,
    sort_permutation, ColWidth, Column, SortKind, TableColumn, TableRenderInfo, TableRow,
    TableSort, TableViewState,
};
pub use tabs::render_tabs;
