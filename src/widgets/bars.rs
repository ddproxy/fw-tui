use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::sanitize::printable;
use crate::text::{pad_width, truncate_width};
use crate::theme::Theme;

/// Header bar: left lines and right-aligned text inside an area.
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
        let r_w = crate::text::display_width(&r_trunc);
        let l_w = w.saturating_sub(r_w);
        let l_trunc = pad_width(&truncate_width(&l, l_w), l_w);
        lines
            .push(Line::from(vec![Span::raw(l_trunc), Span::raw(r_trunc)]).style(theme.header_bar));
    }
    Paragraph::new(lines).render(area, buf);
}

/// Footer bar: one line of left text and right-aligned toggle chips.
#[derive(Debug, Clone)]
pub struct ToggleChip {
    pub label: String,
    pub on: Option<bool>,
    pub value: Option<String>,
}

impl ToggleChip {
    pub fn toggle(label: impl Into<String>, on: bool) -> Self {
        Self {
            label: printable(&label.into()),
            on: Some(on),
            value: None,
        }
    }
    pub fn value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: printable(&label.into()),
            on: None,
            value: Some(printable(&value.into())),
        }
    }
    pub fn render_text(&self) -> String {
        match (self.on, &self.value) {
            (Some(true), _) => format!("[■ {}]", self.label),
            (Some(false), _) => format!("[□ {}]", self.label),
            (None, Some(v)) => format!("[{}: {}]", self.label, v),
            _ => format!("[{}]", self.label),
        }
    }
}

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
    let r_w = crate::text::display_width(&r_trunc);
    let l_w = w.saturating_sub(r_w);
    let l_trunc = pad_width(&truncate_width(&left, l_w), l_w);
    let line = Line::from(vec![
        Span::styled(l_trunc, Style::default()),
        Span::styled(r_trunc, theme.label),
    ]);
    Paragraph::new(line).render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

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
            let backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            let theme = Theme::dark();
            terminal
                .draw(|f| {
                    let a = f.area();
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
                })
                .unwrap();
            let _ = buffer_text(terminal.backend().buffer());
        }
    }
}
