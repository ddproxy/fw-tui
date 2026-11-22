//! Header and footer bars with right-aligned text and toggle chips.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{render_footer_bar, ToggleChip};
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let theme = Theme::dark();
//! let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
//! terminal
//!     .draw(|f| {
//!         render_footer_bar(
//!             f.area(),
//!             f.buffer_mut(),
//!             "ready",
//!             &[ToggleChip::toggle("detail", true)],
//!             &theme,
//!         )
//!     })
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::sanitize::printable;
use crate::text::{display_width, pad_width, truncate_width};
use crate::theme::Theme;

/// Header bar: left lines and right-aligned text inside an area.
///
/// Both lists are escaped and clipped by display width. Missing lines are drawn as blank lines.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::render_header_bar;
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let theme = Theme::dark();
/// let mut terminal = Terminal::new(TestBackend::new(30, 2)).unwrap();
/// terminal
///     .draw(|f| {
///         render_header_bar(
///             f.area(),
///             f.buffer_mut(),
///             &["left".into(), "second".into()],
///             &["right".into()],
///             &theme,
///         )
///     })
///     .unwrap();
/// ```
pub fn render_header_bar(
    area: Rect,
    buf: &mut Buffer,
    left: &[String],
    right: &[String],
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let lines_count = area.height as usize;
    let mut lines: Vec<Line> = Vec::new();
    for i in 0..lines_count {
        let l = left.get(i).map(|s| printable(s)).unwrap_or_default();
        let r = right.get(i).map(|s| printable(s)).unwrap_or_default();
        let w = area.width as usize;
        let r_trunc = truncate_width(&r, w);
        let r_w = display_width(&r_trunc);
        let l_w = w.saturating_sub(r_w);
        let l_trunc = pad_width(&truncate_width(&l, l_w), l_w);
        lines
            .push(Line::from(vec![Span::raw(l_trunc), Span::raw(r_trunc)]).style(theme.header_bar));
    }
    Paragraph::new(lines).render(area, buf);
}

/// Footer bar: one line of left text and right-aligned toggle chips.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::render_footer_bar;
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let theme = Theme::dark();
/// let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
/// terminal
///     .draw(|f| render_footer_bar(f.area(), f.buffer_mut(), "ready", &[], &theme))
///     .unwrap();
/// ```
pub fn render_footer_bar(
    area: Rect,
    buf: &mut Buffer,
    left: &str,
    chips: &[ToggleChip],
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let left = printable(left);
    let right = chips
        .iter()
        .map(|c| c.render_text())
        .collect::<Vec<_>>()
        .join(" ");
    let w = area.width as usize;
    let r_trunc = truncate_width(&right, w);
    let r_w = display_width(&r_trunc);
    let l_w = w.saturating_sub(r_w);
    let l_trunc = pad_width(&truncate_width(&left, l_w), l_w);
    let line = Line::from(vec![
        Span::styled(l_trunc, Style::default()),
        Span::styled(r_trunc, theme.label),
    ]);
    Paragraph::new(line).render(area, buf);
}

/// One chip in the footer: a boolean toggle or a named value.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::ToggleChip;
///
/// assert_eq!(ToggleChip::toggle("detail", true).render_text(), "[■ detail]");
/// ```
#[derive(Debug, Clone)]
pub struct ToggleChip {
    /// Chip label. Escaped at construction and again when rendered.
    ///
    /// ```
    /// use fw_tui::widgets::ToggleChip;
    ///
    /// let chip = ToggleChip { label: "detail".into(), on: Some(true), value: None };
    /// assert_eq!(chip.label, "detail");
    /// ```
    pub label: String,
    /// `Some(true)` for on, `Some(false)` for off, `None` for a value chip.
    ///
    /// ```
    /// use fw_tui::widgets::ToggleChip;
    ///
    /// let chip = ToggleChip { label: "d".into(), on: Some(false), value: None };
    /// assert_eq!(chip.on, Some(false));
    /// ```
    pub on: Option<bool>,
    /// Optional displayed value, used when `on` is `None`.
    ///
    /// ```
    /// use fw_tui::widgets::ToggleChip;
    ///
    /// let chip = ToggleChip { label: "privacy".into(), on: None, value: Some("0".into()) };
    /// assert_eq!(chip.value.as_deref(), Some("0"));
    /// ```
    pub value: Option<String>,
}

impl ToggleChip {
    /// Creates an on/off chip.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::ToggleChip;
    ///
    /// assert_eq!(ToggleChip::toggle("detail", false).render_text(), "[□ detail]");
    /// ```
    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            label: printable(&label.into()),
            on: Some(on),
            value: None,
        }
    }

    /// Creates a value chip.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::ToggleChip;
    ///
    /// assert_eq!(ToggleChip::value("privacy", "0").render_text(), "[privacy: 0]");
    /// ```
    pub fn value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: printable(&label.into()),
            on: None,
            value: Some(printable(&value.into())),
        }
    }

    /// Formats the chip, escaping the label and value even if the struct was built directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::ToggleChip;
    ///
    /// let chip = ToggleChip { label: "x\u{1b}[31m".into(), on: None, value: None };
    /// assert!(!chip.render_text().contains('\u{1b}'));
    /// ```
    pub fn render_text(&self) -> String {
        let label = printable(&self.label);
        match (self.on, &self.value) {
            (Some(true), _) => format!("[■ {label}]"),
            (Some(false), _) => format!("[□ {label}]"),
            (None, Some(v)) => format!("[{label}: {}]", printable(v)),
            (None, None) => format!("[{label}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_control_chars, render_text};

    #[test]
    fn footer_chips_text() {
        let on = ToggleChip::toggle("detail", true).render_text();
        let off = ToggleChip::toggle("detail", false).render_text();
        let val = ToggleChip::value("privacy", "0").render_text();
        assert_eq!(on, "[■ detail]");
        assert_eq!(off, "[□ detail]");
        assert_eq!(val, "[privacy: 0]");
    }

    #[test]
    fn bars_render_narrow_and_tiny() {
        for (w, h) in [(30u16, 2u16), (10, 1), (1, 1), (0, 0)] {
            let text = render_text(w, h, |f| {
                let a = f.area();
                let theme = Theme::dark();
                render_header_bar(
                    a,
                    f.buffer_mut(),
                    &["left".into()],
                    &["right".into()],
                    &theme,
                );
                render_footer_bar(
                    a,
                    f.buffer_mut(),
                    "L",
                    &[ToggleChip::toggle("d", true)],
                    &theme,
                );
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn empty_lists_and_long_right_text() {
        let theme = Theme::dark();
        let text = render_text(12, 2, |f| {
            render_header_bar(f.area(), f.buffer_mut(), &[], &[], &theme)
        });
        assert_eq!(text, "            \n            ");

        let text = render_text(12, 1, |f| {
            render_footer_bar(f.area(), f.buffer_mut(), "L", &[], &theme)
        });
        assert!(text.starts_with('L'));
    }

    #[test]
    fn right_text_stays_right_aligned_when_resized() {
        let theme = Theme::dark();
        for (w, h) in [(20, 1), (5, 1), (200, 1), (12, 3)] {
            let text = render_text(w, h, |f| {
                render_header_bar(
                    f.area(),
                    f.buffer_mut(),
                    &["L".into()],
                    &["R".into()],
                    &theme,
                )
            });
            let first = text.lines().next().unwrap();
            assert!(first.starts_with('L'), "{first:?}");
            assert!(first.trim_end().ends_with('R'), "{first:?}");
        }
    }

    #[test]
    fn directly_built_chips_cannot_inject_ansi() {
        let theme = Theme::dark();
        let chip = ToggleChip {
            label: "d\u{1b}[31m".into(),
            on: None,
            value: Some("0\u{9b}2J".into()),
        };
        assert!(!chip.render_text().contains('\u{1b}'));
        assert!(!chip.render_text().contains('\u{9b}'));
        let text = render_text(40, 1, |f| {
            render_footer_bar(f.area(), f.buffer_mut(), "L\u{1b}[2J", &[chip], &theme)
        });
        assert_no_control_chars(&text);
    }

    #[test]
    fn wide_and_combining_labels_are_clipped_by_cells() {
        let theme = Theme::dark();
        let text = render_text(8, 1, |f| {
            render_header_bar(
                f.area(),
                f.buffer_mut(),
                &["中中中".into(), "e\u{301}".into()],
                &["中".into()],
                &theme,
            )
        });
        assert_no_control_chars(&text);
        assert_eq!(text.chars().count(), 8);
    }
}
