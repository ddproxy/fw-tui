use ratatui::style::Style;
use ratatui::text::Span;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotState {
    Good,
    Warn,
    Bad,
    Dim,
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
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;

    #[test]
    fn dot_is_one_cell() {
        let theme = Theme::dark();
        let s = status_dot(DotState::Good, &theme, false);
        assert_eq!(s.content.chars().count(), 1);
        let s2 = status_dot(DotState::Bad, &theme, true);
        assert_eq!(s2.content.as_ref(), "x");
    }

    #[test]
    fn dot_renders() {
        let backend = TestBackend::new(2, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| {
                f.render_widget(
                    Paragraph::new(status_dot(DotState::Warn, &theme, false)),
                    f.area(),
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains('●') || text.contains('!'));
    }
}
