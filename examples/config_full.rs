//! Full configuration: tui feature, explicit chart options, fixed scale and theme selection.

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
