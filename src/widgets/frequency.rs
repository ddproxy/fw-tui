use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Bar, BarChart, BarGroup, Widget};

use crate::sanitize::printable;
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct FrequencyBar {
    pub label: String,
    pub count: u64,
}

/// Small vertical bar chart of counts per category.
pub fn render_frequency_chart(area: Rect, buf: &mut Buffer, bars: &[FrequencyBar], theme: &Theme) {
    if area.width == 0 || area.height == 0 || bars.is_empty() {
        return;
    }
    let safe: Vec<Bar> = bars
        .iter()
        .map(|b| {
            let label = printable(&b.label);
            let val = b.count.min(u64::from(u16::MAX));
            Bar::default()
                .value(val)
                .label(ratatui::text::Line::from(label))
                .text_value(b.count.to_string())
                .style(theme.accent)
                .value_style(theme.label)
        })
        .collect();
    BarChart::default()
        .data(BarGroup::default().bars(&safe))
        .bar_width(3)
        .bar_gap(1)
        .render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn frequency_renders_counts() {
        let backend = TestBackend::new(30, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let bars = vec![
            FrequencyBar {
                label: "2xx".into(),
                count: 5,
            },
            FrequencyBar {
                label: "4xx".into(),
                count: 2,
            },
            FrequencyBar {
                label: "5xx".into(),
                count: 1,
            },
        ];
        terminal
            .draw(|f| render_frequency_chart(f.area(), f.buffer_mut(), &bars, &theme))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("2xx"));
        assert!(text.contains("5xx"));
    }

    #[test]
    fn frequency_tiny_no_panic() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| {
                render_frequency_chart(
                    f.area(),
                    f.buffer_mut(),
                    &[FrequencyBar {
                        label: "x".into(),
                        count: u64::MAX,
                    }],
                    &theme,
                )
            })
            .unwrap();
        let _ = buffer_text(terminal.backend().buffer());
    }
}
