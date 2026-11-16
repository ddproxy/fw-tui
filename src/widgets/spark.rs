use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Sparkline as RSparkline;
use ratatui::widgets::Widget;

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scale {
    Auto,
    Fixed(f64),
}

fn effective_max(values: &[f64], scale: Scale) -> f64 {
    match scale {
        Scale::Fixed(m) if m.is_finite() && m > 0.0 => m,
        _ => values
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .fold(0.0f64, f64::max)
            .max(1.0),
    }
}

/// Compact one-row sparkline (8 levels).
#[derive(Debug, Clone)]
pub struct Sparkline {
    pub values: Vec<f64>,
    pub scale: Scale,
}

impl Sparkline {
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            scale: Scale::Auto,
        }
    }

    pub fn scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, style: Style) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        // ratatui sparkline takes u64 data; scale to max for shared-scale behavior
        let max = effective_max(&self.values, self.scale);
        let data: Vec<u64> = self
            .values
            .iter()
            .map(|v| {
                if !v.is_finite() || *v <= 0.0 {
                    0
                } else {
                    ((v / max) * 8.0).round() as u64
                }
            })
            .collect();
        let w = area.width as usize;
        let slice = if data.len() > w {
            &data[data.len() - w..]
        } else {
            &data[..]
        };
        RSparkline::default()
            .data(slice)
            .style(style)
            .render(area, buf);
    }
}

/// Filled bar chart of the last N values across the panel width.
#[derive(Debug, Clone)]
pub struct BarPanel {
    pub values: Vec<f64>,
    pub scale: Scale,
}

impl BarPanel {
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            scale: Scale::Auto,
        }
    }

    pub fn scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let w = area.width as usize;
        let h = area.height as usize;
        let vals: Vec<f64> = if self.values.len() > w {
            self.values[self.values.len() - w..].to_vec()
        } else {
            self.values.clone()
        };
        let max = effective_max(&vals, self.scale).max(1.0);
        for (i, v) in vals.iter().enumerate() {
            if i >= w {
                break;
            }
            let vv = if v.is_finite() && *v > 0.0 { *v } else { 0.0 };
            let filled = ((vv / max) * h as f64).round() as usize;
            let filled = filled.min(h);
            for row in 0..h {
                let y = area.y + (h.saturating_sub(1 + row)) as u16;
                let x = area.x + i as u16;
                if x >= area.x + area.width || y >= area.y + area.height {
                    continue;
                }
                let ch = if row < filled { '█' } else { ' ' };
                let cell = &mut buf[(x, y)];
                cell.set_char(ch);
                if ch != ' ' {
                    cell.set_style(theme.accent);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn sparkline_and_barpanel_no_panic() {
        let backend = TestBackend::new(10, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let vals = vec![0.0, 1.0, f64::NAN, 5.0, f64::INFINITY, -1.0, 3.0];
        terminal
            .draw(|f| {
                let area = f.area();
                let top = Rect::new(area.x, area.y, area.width, 1);
                let bot = Rect::new(area.x, area.y + 1, area.width, 2);
                Sparkline::new(vals.clone()).render(top, f.buffer_mut(), theme.accent);
                BarPanel::new(vals).render(bot, f.buffer_mut(), &theme);
            })
            .unwrap();
        let _ = buffer_text(terminal.backend().buffer());
    }

    #[test]
    fn shared_scale_fixed() {
        let s = Sparkline::new(vec![1.0, 2.0]).scale(Scale::Fixed(10.0));
        assert!(matches!(s.scale, Scale::Fixed(_)));
    }
}
