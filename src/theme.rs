use ratatui::style::{Color, Modifier, Style};

/// Semantic styles used by widgets. Applications refer to names, not raw colours.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub title: Style,
    pub label: Style,
    pub dim: Style,
    pub good: Style,
    pub warn: Style,
    pub bad: Style,
    pub accent: Style,
    pub selected_row: Style,
    pub header_bar: Style,
    pub border: Style,
    pub border_focused: Style,
}

impl Theme {
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
        let _ = Theme::dark();
        let _ = Theme::monochrome();
        let _ = Theme::auto();
    }
}
