//! Rounded bordered panel with a title and optional right-aligned bottom title.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::Panel;
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let mut terminal = Terminal::new(TestBackend::new(20, 4)).unwrap();
//! terminal
//!     .draw(|f| Panel::new("Title").render(f, f.area(), &Theme::dark()))
//!     .unwrap();
//! ```

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Frame;

use crate::sanitize::printable;
use crate::theme::Theme;

/// A rounded bordered block with semantic border colours.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::Panel;
///
/// let panel = Panel::new("Title").right_title("hints").focused(true);
/// assert!(panel.focused);
/// ```
#[derive(Debug, Clone)]
pub struct Panel<'a> {
    /// Top title, escaped before it reaches the block.
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    ///
    /// let panel = Panel::new("Title");
    /// assert_eq!(panel.title, "Title");
    /// ```
    pub title: &'a str,
    /// Optional right-aligned title drawn at the bottom border.
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    ///
    /// let panel = Panel::new("Title").right_title("hint");
    /// assert_eq!(panel.right, Some("hint"));
    /// ```
    pub right: Option<&'a str>,
    /// Whether to use the focused border style.
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    ///
    /// let panel = Panel::new("Title").focused(true);
    /// assert!(panel.focused);
    /// ```
    pub focused: bool,
}

impl<'a> Panel<'a> {
    /// Creates a panel with the given title and no bottom title.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    ///
    /// let panel = Panel::new("Title");
    /// assert!(panel.right.is_none());
    /// ```
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            right: None,
            focused: false,
        }
    }

    /// Adds a right-aligned title on the bottom border.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    ///
    /// let panel = Panel::new("Title").right_title("hint");
    /// assert_eq!(panel.right, Some("hint"));
    /// ```
    pub fn right_title(mut self, right: &'a str) -> Self {
        self.right = Some(right);
        self
    }

    /// Selects the focused or unfocused border style.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    ///
    /// let panel = Panel::new("Title").focused(false);
    /// assert!(!panel.focused);
    /// ```
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Builds the underlying ratatui [`Block`], escaping both titles.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use fw_tui::widgets::Panel;
    ///
    /// let block = Panel::new("Title").right_title("hint").block(&Theme::dark());
    /// let _ = block;
    /// ```
    pub fn block(&self, theme: &Theme) -> Block<'a> {
        let border_style = if self.focused {
            theme.border_focused
        } else {
            theme.border
        };
        let mut b = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(printable(self.title));
        if let Some(r) = self.right {
            b = b.title_bottom(Line::from(printable(r)).right_aligned());
        }
        b
    }

    /// Interior area of a bordered rectangle, or a zero-sized rectangle when too small.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Panel;
    /// use ratatui::layout::Rect;
    ///
    /// assert_eq!(Panel::inner(Rect::new(0, 0, 10, 5)).width, 8);
    /// assert_eq!(Panel::inner(Rect::new(0, 0, 1, 1)).width, 0);
    /// ```
    pub fn inner(area: Rect) -> Rect {
        if area.width < 2 || area.height < 2 {
            Rect::new(area.x, area.y, 0, 0)
        } else {
            Rect::new(
                area.x.saturating_add(1),
                area.y.saturating_add(1),
                area.width.saturating_sub(2),
                area.height.saturating_sub(2),
            )
        }
    }

    /// Renders the panel into a frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use fw_tui::widgets::Panel;
    /// use ratatui::backend::TestBackend;
    /// use ratatui::Terminal;
    ///
    /// let mut terminal = Terminal::new(TestBackend::new(20, 4)).unwrap();
    /// terminal
    ///     .draw(|f| Panel::new("Title").render(f, f.area(), &Theme::dark()))
    ///     .unwrap();
    /// ```
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        frame.render_widget(self.block(theme), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_control_chars, render_text};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn panel_inner_tolerates_tiny() {
        assert_eq!(Panel::inner(Rect::new(0, 0, 0, 0)).width, 0);
        assert_eq!(Panel::inner(Rect::new(0, 0, 1, 1)).height, 0);
        assert_eq!(Panel::inner(Rect::new(0, 0, 1, 10)).width, 0);
        assert_eq!(Panel::inner(Rect::new(3, 4, 10, 5)).x, 4);
        assert_eq!(Panel::inner(Rect::new(3, 4, 10, 5)).y, 5);
        assert_eq!(Panel::inner(Rect::new(0, 0, 10, 5)).width, 8);
        assert_eq!(Panel::inner(Rect::new(0, 0, 10, 5)).height, 3);
    }

    #[test]
    fn panel_renders_title() {
        let theme = Theme::dark();
        let text = render_text(20, 4, |f| Panel::new("Title").render(f, f.area(), &theme));
        assert!(text.contains("Title"), "{text}");
    }

    #[test]
    fn panel_escapes_both_titles() {
        let theme = Theme::dark();
        let text = render_text(30, 5, |f| {
            Panel::new("T\u{1b}[2J")
                .right_title("r\u{9b}31m")
                .render(f, f.area(), &theme)
        });
        assert_no_control_chars(&text);
        assert!(!text.contains('\u{1b}'));
    }

    #[test]
    fn panel_survives_resize_and_wide_titles() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (2, 2), (12, 3), (200, 8)] {
            let text = render_text(w, h, |f| {
                Panel::new("中中\u{301}")
                    .right_title("hint")
                    .focused(true)
                    .render(f, f.area(), &theme)
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn panel_block_is_reusable() {
        let theme = Theme::dark();
        let panel = Panel::new("A").right_title("B");
        let block = panel.block(&theme);
        let mut terminal = Terminal::new(TestBackend::new(20, 4)).unwrap();
        terminal
            .draw(|f| f.render_widget(block.clone(), f.area()))
            .unwrap();
        let text = crate::text::buffer_text(terminal.backend().buffer());
        assert!(text.contains('A'));
        assert!(text.contains('B'));
    }
}
