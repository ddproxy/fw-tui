use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

use crate::sanitize::printable;
use crate::theme::Theme;

/// Centered bordered overlay box for help/settings.
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
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn overlay_clears_and_shows_title() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| {
                Paragraph::new("BACKGROUND").render(f.area(), f.buffer_mut());
                render_overlay(
                    f.area(),
                    f.buffer_mut(),
                    "Help",
                    &["q quit".into(), "p pause".into()],
                    &theme,
                );
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Help"), "{text}");
        assert!(text.contains("q quit"), "{text}");
    }

    #[test]
    fn overlay_tiny_no_panic() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| render_overlay(f.area(), f.buffer_mut(), "H", &["x".into()], &theme))
            .unwrap();
        let _ = buffer_text(terminal.backend().buffer());
    }
}
