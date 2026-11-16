use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::sanitize::printable;
use crate::text::{fmt_duration, truncate_width};

fn clamp01(v: f64) -> f64 {
    if v.is_nan() || v < 0.0 {
        0.0
    } else if v > 1.0 || v.is_infinite() {
        1.0
    } else {
        v
    }
}

/// Horizontal bar with label on left and right-aligned text (e.g. countdown).
#[derive(Debug, Clone)]
pub struct Gauge {
    pub label: String,
    pub right: String,
    pub fraction: f64,
    pub bar_width: u16,
    pub style: Style,
}

impl Gauge {
    pub fn new(label: impl Into<String>, fraction: f64, bar_width: u16, style: Style) -> Self {
        Self {
            label: printable(&label.into()),
            right: String::new(),
            fraction,
            bar_width,
            style,
        }
    }

    pub fn right_text(mut self, text: impl Into<String>) -> Self {
        self.right = printable(&text.into());
        self
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let frac = clamp01(self.fraction);
        let bar_w = self.bar_width.min(area.width.saturating_sub(2)) as usize;
        let filled = ((bar_w as f64) * frac).round() as usize;
        let filled = filled.min(bar_w);
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_w.saturating_sub(filled));
        let label = truncate_width(&self.label, 12);
        let right = truncate_width(&self.right, 8);
        let mut spans = vec![
            Span::raw(format!("{label:<12} ")),
            Span::styled(bar, self.style),
        ];
        if !right.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::raw(right));
        }
        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}

/// Countdown gauge where fill drains as remaining approaches 0.
#[derive(Debug, Clone)]
pub struct CountdownGauge {
    pub remaining_secs: u64,
    pub full_secs: u64,
}

impl CountdownGauge {
    pub fn from(remaining_secs: u64, full_secs: u64) -> Self {
        Self {
            remaining_secs,
            full_secs,
        }
    }

    pub fn fraction(&self) -> f64 {
        let full = self.full_secs.max(1) as f64;
        let rem = self.remaining_secs.min(self.full_secs) as f64;
        clamp01(rem / full)
    }

    pub fn to_gauge(&self, label: impl Into<String>, bar_width: u16, style: Style) -> Gauge {
        Gauge::new(label, self.fraction(), bar_width, style)
            .right_text(fmt_duration(self.remaining_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn countdown_drain_levels() {
        let g0 = CountdownGauge::from(0, 100);
        let g50 = CountdownGauge::from(50, 100);
        let g100 = CountdownGauge::from(100, 100);
        assert_eq!(g0.fraction(), 0.0);
        assert!((g50.fraction() - 0.5).abs() < 1e-9);
        assert_eq!(g100.fraction(), 1.0);
    }

    #[test]
    fn hostile_numbers_clamped() {
        let g = Gauge::new("x", f64::NAN, 10, Style::default());
        assert_eq!(clamp01(g.fraction), 0.0);
        let g2 = Gauge::new("x", f64::INFINITY, 10, Style::default());
        assert_eq!(clamp01(g2.fraction), 1.0);
    }

    #[test]
    fn gauge_tiny_area_no_panic() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let gauge = Gauge::new("longlabel", 0.5, 20, Style::default());
        terminal
            .draw(|f| gauge.render(f.area(), f.buffer_mut()))
            .unwrap();
        let _ = buffer_text(terminal.backend().buffer());
    }
}
