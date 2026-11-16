use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Frame;

use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct Panel<'a> {
    pub title: &'a str,
    pub right: Option<&'a str>,
    pub focused: bool,
}

impl<'a> Panel<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            right: None,
            focused: false,
        }
    }

    pub fn right_title(mut self, right: &'a str) -> Self {
        self.right = Some(right);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

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
            .title(self.title);
        if let Some(r) = self.right {
            b = b.title_bottom(ratatui::text::Line::from(r).right_aligned());
        }
        b
    }

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

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        frame.render_widget(self.block(theme), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn panel_inner_tolerates_tiny() {
        assert_eq!(Panel::inner(Rect::new(0, 0, 0, 0)).width, 0);
        assert_eq!(Panel::inner(Rect::new(0, 0, 1, 1)).height, 0);
        assert_eq!(Panel::inner(Rect::new(0, 0, 10, 5)).width, 8);
    }

    #[test]
    fn panel_renders_title() {
        let backend = TestBackend::new(20, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        terminal
            .draw(|f| Panel::new("Title").render(f, f.area(), &theme))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Title"), "{text}");
    }
}
