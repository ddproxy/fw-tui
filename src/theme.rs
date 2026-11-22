//! Semantic styles for widgets.
//!
//! Requires the `tui` feature. Applications refer to names such as [`Theme::good`] rather than raw
//! colours, so a monochrome variant can be swapped in without touching drawing code.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use ratatui::style::Color;
//!
//! assert_eq!(Theme::dark().good.fg, Some(Color::Green));
//! ```

use ratatui::style::{Color, Modifier, Style};

/// Semantic styles used by widgets. Applications refer to names, not raw colours.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
///
/// let theme = Theme::dark();
/// let _ = theme.good;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Style for panel and screen titles.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().title.fg, Some(Color::White));
    /// ```
    pub title: Style,
    /// Style for ordinary labels.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().label.fg, Some(Color::Gray));
    /// ```
    pub label: Style,
    /// Style for secondary or inactive information.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().dim.fg, Some(Color::DarkGray));
    /// ```
    pub dim: Style,
    /// Style for healthy states.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().good.fg, Some(Color::Green));
    /// ```
    pub good: Style,
    /// Style for warning states.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().warn.fg, Some(Color::Yellow));
    /// ```
    pub warn: Style,
    /// Style for error states.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().bad.fg, Some(Color::Red));
    /// ```
    pub bad: Style,
    /// Style for the focused series, tab or accent marks.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().accent.fg, Some(Color::Cyan));
    /// ```
    pub accent: Style,
    /// Style applied to the selected table row.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().selected_row.fg, Some(Color::White));
    /// ```
    pub selected_row: Style,
    /// Style for reversed header bars.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Modifier;
    ///
    /// assert!(Theme::dark().header_bar.add_modifier.contains(Modifier::REVERSED));
    /// ```
    pub header_bar: Style,
    /// Style for unfocused panel borders.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().border.fg, Some(Color::Gray));
    /// ```
    pub border: Style,
    /// Style for the focused panel border.
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().border_focused.fg, Some(Color::Cyan));
    /// ```
    pub border_focused: Style,
}

impl Theme {
    /// Colourful palette for dark terminals.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(Theme::dark().good.fg, Some(Color::Green));
    /// ```
    pub fn dark() -> Self {
        Self {
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            label: Style::default().fg(Color::Gray),
            dim: Style::default().fg(Color::DarkGray),
            good: Style::default().fg(Color::Green),
            warn: Style::default().fg(Color::Yellow),
            bad: Style::default().fg(Color::Red),
            accent: Style::default().fg(Color::Cyan),
            selected_row: Style::default().bg(Color::DarkGray).fg(Color::White),
            header_bar: Style::default().add_modifier(Modifier::REVERSED),
            border: Style::default().fg(Color::Gray),
            border_focused: Style::default().fg(Color::Cyan),
        }
    }

    /// Monochrome variant suitable when colour is undesirable.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::{Color, Modifier};
    ///
    /// let theme = Theme::monochrome();
    /// assert_eq!(theme.good.fg, None);
    /// assert!(theme.bad.add_modifier.contains(Modifier::UNDERLINED));
    /// ```
    pub fn monochrome() -> Self {
        let base = Style::default();
        Self {
            title: base.add_modifier(Modifier::BOLD),
            label: base,
            dim: base.add_modifier(Modifier::DIM),
            good: base.add_modifier(Modifier::BOLD),
            warn: base.add_modifier(Modifier::BOLD),
            bad: base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            accent: base.add_modifier(Modifier::BOLD),
            selected_row: base.add_modifier(Modifier::REVERSED),
            header_bar: base.add_modifier(Modifier::REVERSED),
            border: base,
            border_focused: base.add_modifier(Modifier::BOLD),
        }
    }

    /// Chooses monochrome when `NO_COLOR` is set, otherwise dark.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use ratatui::style::Modifier;
    ///
    /// let theme = Theme::auto();
    /// assert!(theme.title.add_modifier.contains(Modifier::BOLD));
    /// ```
    pub fn auto() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            Self::monochrome()
        } else {
            Self::dark()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provides_variants() {
        let dark = Theme::dark();
        let mono = Theme::monochrome();
        let auto = Theme::auto();
        assert_eq!(dark.good.fg, Some(Color::Green));
        assert_eq!(mono.good.fg, None);
        assert!(mono.bad.add_modifier.contains(Modifier::UNDERLINED));
        assert!(auto.title.add_modifier.contains(Modifier::BOLD));
        assert_eq!(
            auto.label,
            if std::env::var_os("NO_COLOR").is_some() {
                mono.label
            } else {
                dark.label
            }
        );
    }

    #[test]
    fn every_semantic_style_is_distinct_where_it_matters() {
        let dark = Theme::dark();
        assert_ne!(dark.good.fg, dark.bad.fg);
        assert_ne!(dark.border.fg, dark.border_focused.fg);
        assert_eq!(dark.selected_row.bg, Some(Color::DarkGray));
        assert!(dark.header_bar.add_modifier.contains(Modifier::REVERSED));
    }
}
