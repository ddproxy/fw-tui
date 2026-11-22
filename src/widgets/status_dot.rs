//! Fixed one-cell status marker with a text fallback for `NO_COLOR`.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{status_dot, DotState};
//!
//! let span = status_dot(DotState::Good, &Theme::dark(), false);
//! assert_eq!(span.content.as_ref(), "●");
//! ```

use ratatui::style::Style;
use ratatui::text::Span;

use crate::theme::Theme;

/// Semantic state of a status dot.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::DotState;
///
/// assert_eq!(DotState::Good, DotState::Good);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotState {
    /// Healthy state.
    ///
    /// ```
    /// use fw_tui::widgets::DotState;
    ///
    /// assert_eq!(DotState::Good, DotState::Good);
    /// ```
    Good,
    /// Warning state.
    ///
    /// ```
    /// use fw_tui::widgets::DotState;
    ///
    /// assert_eq!(DotState::Warn, DotState::Warn);
    /// ```
    Warn,
    /// Error state.
    ///
    /// ```
    /// use fw_tui::widgets::DotState;
    ///
    /// assert_eq!(DotState::Bad, DotState::Bad);
    /// ```
    Bad,
    /// Inactive or unknown state.
    ///
    /// ```
    /// use fw_tui::widgets::DotState;
    ///
    /// assert_eq!(DotState::Dim, DotState::Dim);
    /// ```
    Dim,
    /// Informational state.
    ///
    /// ```
    /// use fw_tui::widgets::DotState;
    ///
    /// assert_eq!(DotState::Info, DotState::Info);
    /// ```
    Info,
}

impl DotState {
    fn style(self, theme: &Theme) -> Style {
        match self {
            DotState::Good => theme.good,
            DotState::Warn => theme.warn,
            DotState::Bad => theme.bad,
            DotState::Dim => theme.dim,
            DotState::Info => theme.accent,
        }
    }

    fn fallback(self) -> char {
        match self {
            DotState::Good => 'o',
            DotState::Warn => '!',
            DotState::Bad => 'x',
            DotState::Dim => '.',
            DotState::Info => 'i',
        }
    }
}

/// Fixed one-cell status marker.
///
/// With `no_color` set, a distinct ASCII fallback is used instead of the coloured dot.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::{status_dot, DotState};
///
/// let theme = Theme::dark();
/// assert_eq!(status_dot(DotState::Bad, &theme, true).content.as_ref(), "x");
/// assert_eq!(status_dot(DotState::Good, &theme, false).content.as_ref(), "●");
/// ```
pub fn status_dot(state: DotState, theme: &Theme, no_color: bool) -> Span<'static> {
    if no_color {
        Span::raw(state.fallback().to_string())
    } else {
        Span::styled("●", state.style(theme))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::render_text;
    use ratatui::widgets::Paragraph;

    #[test]
    fn dot_is_one_cell() {
        let theme = Theme::dark();
        let s = status_dot(DotState::Good, &theme, false);
        assert_eq!(s.content.chars().count(), 1);
        let s2 = status_dot(DotState::Bad, &theme, true);
        assert_eq!(s2.content.as_ref(), "x");
    }

    #[test]
    fn all_states_have_distinct_fallbacks_and_styles() {
        let theme = Theme::dark();
        let states = [
            DotState::Good,
            DotState::Warn,
            DotState::Bad,
            DotState::Dim,
            DotState::Info,
        ];
        let mut fallbacks = std::collections::BTreeSet::new();
        for state in states {
            let span = status_dot(state, &theme, true);
            assert!(fallbacks.insert(span.content.to_string()));
            let coloured = status_dot(state, &theme, false);
            assert_eq!(coloured.content.as_ref(), "●");
            assert_eq!(coloured.style, state.style(&theme));
        }
        assert_eq!(fallbacks.len(), states.len());
    }

    #[test]
    fn dot_renders() {
        let theme = Theme::dark();
        let text = render_text(2, 1, |f| {
            f.render_widget(
                Paragraph::new(status_dot(DotState::Warn, &theme, false)),
                f.area(),
            )
        });
        assert!(text.contains('●'));
        let text = render_text(2, 1, |f| {
            f.render_widget(
                Paragraph::new(status_dot(DotState::Warn, &theme, true)),
                f.area(),
            )
        });
        assert!(text.contains('!'));
    }

    #[test]
    fn zero_sized_and_resized_areas_are_safe() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (10, 2)] {
            let text = render_text(w, h, |f| {
                f.render_widget(
                    Paragraph::new(status_dot(DotState::Info, &theme, true)),
                    f.area(),
                )
            });
            assert!(text.chars().all(|c| c == 'i' || c == ' ' || c == '\n'));
        }
    }
}
