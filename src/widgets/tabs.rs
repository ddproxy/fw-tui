use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::sanitize::printable;
use crate::theme::Theme;

/// Render a row of tab labels with one highlighted.
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
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn tabs_highlight_selected() {
        let backend = TestBackend::new(30, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| {
                render_tabs(
                    f.area(),
                    f.buffer_mut(),
                    &["one", "two", "three"],
                    1,
                    &theme,
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("two"));
    }

    #[test]
    fn tabs_narrow_no_panic() {
        let backend = TestBackend::new(3, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| render_tabs(f.area(), f.buffer_mut(), &["alpha", "beta"], 0, &theme))
            .unwrap();
        let _ = buffer_text(terminal.backend().buffer());
    }
}
