//! Centered bordered overlay for help and settings.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::render_overlay;
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
//! terminal
//!     .draw(|f| {
//!         render_overlay(
//!             f.area(),
//!             f.buffer_mut(),
//!             "Help",
//!             &["q quit".into()],
//!             &Theme::dark(),
//!         )
//!     })
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

use crate::sanitize::printable;
use crate::theme::Theme;

/// Centered bordered overlay box for help/settings.
///
/// The box covers at least 10x5 cells when the area allows it, and shrinks to the area when it
/// does not. The title and every line are escaped before drawing.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::render_overlay;
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
/// terminal
///     .draw(|f| {
///         render_overlay(
///             f.area(),
///             f.buffer_mut(),
///             "Help",
///             &["q quit".into(), "p pause".into()],
///             &Theme::dark(),
///         )
///     })
///     .unwrap();
/// ```
pub fn render_overlay(area: Rect, buf: &mut Buffer, title: &str, lines: &[String], theme: &Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let w = area.width.saturating_mul(3) / 4;
    let h = area.height.saturating_mul(3) / 4;
    let w = w.max(10.min(area.width)).min(area.width);
    let h = h.max(5.min(area.height)).min(area.height);
    let popup = centered_rect(w, h, area);
    Clear.render(popup, buf);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(printable(title))
        .border_style(theme.border_focused)
        .style(theme.label);
    let inner = block.inner(popup);
    block.render(popup, buf);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let content: Vec<Line> = lines
        .iter()
        .map(|l| Line::from(Span::raw(printable(l))))
        .collect();
    Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .render(inner, buf);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_control_chars, render_text};

    #[test]
    fn overlay_clears_and_shows_title() {
        let theme = Theme::dark();
        let text = render_text(40, 10, |f| {
            Paragraph::new("BACKGROUND").render(f.area(), f.buffer_mut());
            render_overlay(
                f.area(),
                f.buffer_mut(),
                "Help",
                &["q quit".into(), "p pause".into()],
                &theme,
            );
        });
        assert!(text.contains("Help"), "{text}");
        assert!(text.contains("q quit"), "{text}");
        // The popup clears its own area; text outside the popup is untouched.
        assert!(
            text.lines().next().unwrap().contains("BACKGROUND"),
            "{text}"
        );
        assert!(
            !text.lines().nth(2).unwrap().contains("BACKGROUND"),
            "{text}"
        );
    }

    #[test]
    fn overlay_tiny_no_panic() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (2, 2), (5, 5)] {
            let text = render_text(w, h, |f| {
                render_overlay(f.area(), f.buffer_mut(), "H", &["x".into()], &theme)
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn empty_lines_and_wide_unicode_are_safe() {
        let theme = Theme::dark();
        let text = render_text(30, 8, |f| {
            render_overlay(f.area(), f.buffer_mut(), "中中", &[], &theme)
        });
        assert_no_control_chars(&text);
        assert!(text.contains('中'));
    }

    #[test]
    fn overlay_escapes_title_and_lines() {
        let theme = Theme::dark();
        let text = render_text(40, 10, |f| {
            render_overlay(
                f.area(),
                f.buffer_mut(),
                "H\u{1b}[2J",
                &["q\u{9b}31m".into(), "\u{202e}x".into()],
                &theme,
            )
        });
        assert_no_control_chars(&text);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{202e}'));
    }

    #[test]
    fn resize_keeps_overlay_drawable() {
        let theme = Theme::dark();
        for (w, h) in [(80, 20), (6, 3), (200, 8)] {
            let text = render_text(w, h, |f| {
                render_overlay(f.area(), f.buffer_mut(), "Help", &["q quit".into()], &theme)
            });
            assert_no_control_chars(&text);
        }
    }
}
