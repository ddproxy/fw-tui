//! One row of tab labels with one highlighted.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::render_tabs;
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let mut terminal = Terminal::new(TestBackend::new(30, 1)).unwrap();
//! terminal
//!     .draw(|f| {
//!         render_tabs(f.area(), f.buffer_mut(), &["one", "two"], 0, &Theme::dark())
//!     })
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::sanitize::printable;
use crate::theme::Theme;

/// Render a row of tab labels with one highlighted.
///
/// An out-of-range selection simply highlights nothing. Labels are escaped before drawing and
/// clipped by the area width.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::render_tabs;
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let mut terminal = Terminal::new(TestBackend::new(30, 1)).unwrap();
/// terminal
///     .draw(|f| {
///         render_tabs(
///             f.area(),
///             f.buffer_mut(),
///             &["Overview", "Table", "Charts"],
///             1,
///             &Theme::dark(),
///         )
///     })
///     .unwrap();
/// ```
pub fn render_tabs(area: Rect, buf: &mut Buffer, labels: &[&str], selected: usize, theme: &Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let mut spans = Vec::new();
    for (i, label) in labels.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("│", theme.dim));
        }
        let text = printable(label);
        let style = if i == selected {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            theme.label
        };
        spans.push(Span::styled(format!(" {text} "), style));
    }
    let line = Line::from(spans);
    Paragraph::new(line).render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_control_chars, render_text};

    #[test]
    fn tabs_highlight_selected() {
        let theme = Theme::dark();
        let text = render_text(30, 1, |f| {
            render_tabs(
                f.area(),
                f.buffer_mut(),
                &["one", "two", "three"],
                1,
                &theme,
            )
        });
        assert!(text.contains("two"));
        assert!(text.contains('│'));
    }

    #[test]
    fn tabs_narrow_no_panic() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (3, 1), (200, 2)] {
            let text = render_text(w, h, |f| {
                render_tabs(f.area(), f.buffer_mut(), &["alpha", "beta"], 0, &theme)
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn empty_labels_and_out_of_range_selection() {
        let theme = Theme::dark();
        let empty = render_text(20, 1, |f| {
            render_tabs(f.area(), f.buffer_mut(), &[], 0, &theme)
        });
        assert!(empty.trim().is_empty());

        let text = render_text(20, 1, |f| {
            render_tabs(f.area(), f.buffer_mut(), &["a", "b"], 99, &theme)
        });
        assert!(text.contains('a'));
        assert!(text.contains('b'));
    }

    #[test]
    fn labels_cannot_inject_terminal_sequences() {
        let theme = Theme::dark();
        let text = render_text(40, 1, |f| {
            render_tabs(
                f.area(),
                f.buffer_mut(),
                &["a\u{1b}[2J", "\u{202e}b", "c\u{9b}31m"],
                0,
                &theme,
            )
        });
        assert_no_control_chars(&text);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{202e}'));
    }

    #[test]
    fn wide_and_combining_labels_are_safe() {
        let theme = Theme::dark();
        for (w, h) in [(5, 1), (30, 1)] {
            let text = render_text(w, h, |f| {
                render_tabs(
                    f.area(),
                    f.buffer_mut(),
                    &["中中", "e\u{301}", "✓"],
                    1,
                    &theme,
                )
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn resize_keeps_tabs_drawable() {
        let theme = Theme::dark();
        for (w, h) in [(40, 1), (12, 1), (240, 3)] {
            let text = render_text(w, h, |f| {
                render_tabs(f.area(), f.buffer_mut(), &["one", "two"], 1, &theme)
            });
            assert_no_control_chars(&text);
            if w >= 12 {
                assert!(text.contains("two"));
            }
        }
    }
}
