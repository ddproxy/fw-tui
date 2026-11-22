//! Vertical frequency (category count) bar chart.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{render_frequency_chart, FrequencyBar};
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let bars = vec![FrequencyBar { label: "2xx".into(), count: 9 }];
//! let mut terminal = Terminal::new(TestBackend::new(20, 6)).unwrap();
//! terminal
//!     .draw(|f| render_frequency_chart(f.area(), f.buffer_mut(), &bars, &Theme::dark()))
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Bar, BarChart, BarGroup, Widget};

use crate::sanitize::printable;
use crate::theme::Theme;

/// One category in a [`render_frequency_chart`].
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::FrequencyBar;
///
/// let bar = FrequencyBar { label: "4xx".into(), count: 2 };
/// assert_eq!(bar.count, 2);
/// ```
#[derive(Debug, Clone)]
pub struct FrequencyBar {
    /// Category label, escaped before drawing.
    ///
    /// ```
    /// use fw_tui::widgets::FrequencyBar;
    ///
    /// let bar = FrequencyBar { label: "x".into(), count: 0 };
    /// assert_eq!(bar.label, "x");
    /// ```
    pub label: String,
    /// Count for the category; drawn values are clamped to the bar height.
    ///
    /// ```
    /// use fw_tui::widgets::FrequencyBar;
    ///
    /// let bar = FrequencyBar { label: "x".into(), count: u64::MAX };
    /// assert_eq!(bar.count, u64::MAX);
    /// ```
    pub count: u64,
}

/// Small vertical bar chart of counts per category.
///
/// An empty slice or a zero-sized area draws nothing.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::{render_frequency_chart, FrequencyBar};
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let bars = vec![FrequencyBar { label: "2xx".into(), count: 5 }];
/// let mut terminal = Terminal::new(TestBackend::new(30, 8)).unwrap();
/// terminal
///     .draw(|f| render_frequency_chart(f.area(), f.buffer_mut(), &bars, &Theme::dark()))
///     .unwrap();
/// ```
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
    use crate::test_support::{assert_no_control_chars, render_text};

    #[test]
    fn frequency_renders_counts() {
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
        let text = render_text(30, 8, |f| {
            render_frequency_chart(f.area(), f.buffer_mut(), &bars, &theme)
        });
        assert!(text.contains("2xx"));
        assert!(text.contains("5xx"));
    }

    #[test]
    fn frequency_tiny_no_panic() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (3, 2), (200, 3)] {
            let text = render_text(w, h, |f| {
                render_frequency_chart(
                    f.area(),
                    f.buffer_mut(),
                    &[FrequencyBar {
                        label: "x".into(),
                        count: u64::MAX,
                    }],
                    &theme,
                )
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn empty_and_wide_unicode_labels_are_safe() {
        let theme = Theme::dark();
        let empty = render_text(20, 5, |f| {
            render_frequency_chart(f.area(), f.buffer_mut(), &[], &theme)
        });
        assert!(empty.trim().is_empty());

        let bars = vec![
            FrequencyBar {
                label: "中中\u{301}".into(),
                count: u64::MAX,
            },
            FrequencyBar {
                label: "\u{1b}[2Jbad".into(),
                count: 3,
            },
        ];
        let text = render_text(30, 8, |f| {
            render_frequency_chart(f.area(), f.buffer_mut(), &bars, &theme)
        });
        assert_no_control_chars(&text);
    }

    #[test]
    fn resizing_keeps_rendering_without_panics() {
        let theme = Theme::dark();
        let bars = vec![FrequencyBar {
            label: "ok".into(),
            count: 12,
        }];
        for (w, h) in [(40, 8), (7, 2), (120, 4)] {
            let text = render_text(w, h, |f| {
                render_frequency_chart(f.area(), f.buffer_mut(), &bars, &theme)
            });
            assert_no_control_chars(&text);
        }
    }
}
