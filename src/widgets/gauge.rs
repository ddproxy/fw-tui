//! Horizontal fill gauges and countdown gauges.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::CountdownGauge;
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let gauge = CountdownGauge::from(30, 60).to_gauge("time", 12, Theme::dark().warn);
//! let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
//! terminal
//!     .draw(|f| gauge.render(f.area(), f.buffer_mut()))
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::sanitize::printable;
use crate::text::{fmt_duration, pad_width, truncate_width};

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
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::Gauge;
/// use ratatui::style::Style;
///
/// let gauge = Gauge::new("load", 0.5, 10, Style::default()).right_text("50%");
/// assert_eq!(gauge.right, "50%");
/// ```
#[derive(Debug, Clone)]
pub struct Gauge {
    /// Left-hand label, escaped and padded by display width when drawn.
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", 0.0, 10, Style::default());
    /// assert_eq!(gauge.label, "load");
    /// ```
    pub label: String,
    /// Right-hand text, escaped and clipped when drawn.
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", 0.0, 10, Style::default()).right_text("1s");
    /// assert_eq!(gauge.right, "1s");
    /// ```
    pub right: String,
    /// Fill fraction; NaN is drawn empty, infinities are drawn full.
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", f64::NAN, 10, Style::default());
    /// assert!(gauge.fraction.is_nan()); // clamped when rendered
    /// ```
    pub fraction: f64,
    /// Requested bar width in cells, clipped to the available area.
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", 0.0, 10, Style::default());
    /// assert_eq!(gauge.bar_width, 10);
    /// ```
    pub bar_width: u16,
    /// Style applied to the filled cells.
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", 0.0, 10, Style::default());
    /// assert_eq!(gauge.style, Style::default());
    /// ```
    pub style: Style,
}

impl Gauge {
    /// Creates a gauge with no right-hand text.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", 0.25, 8, Style::default());
    /// assert_eq!(gauge.label, "load");
    /// assert!(gauge.right.is_empty());
    /// ```
    pub fn new(label: impl Into<String>, fraction: f64, bar_width: u16, style: Style) -> Self {
        Self {
            label: printable(&label.into()),
            right: String::new(),
            fraction,
            bar_width,
            style,
        }
    }

    /// Adds escaped right-hand text.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = Gauge::new("load", 0.5, 8, Style::default()).right_text("\u{1b}[31m");
    /// assert!(!gauge.right.contains('\u{1b}'));
    /// ```
    pub fn right_text(mut self, text: impl Into<String>) -> Self {
        self.right = printable(&text.into());
        self
    }

    /// Draws the gauge. A zero-sized area draws nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Gauge;
    /// use ratatui::backend::TestBackend;
    /// use ratatui::style::Style;
    /// use ratatui::Terminal;
    ///
    /// let gauge = Gauge::new("load", 0.5, 10, Style::default());
    /// let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
    /// terminal.draw(|f| gauge.render(f.area(), f.buffer_mut())).unwrap();
    /// ```
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let frac = clamp01(self.fraction);
        let bar_w = self.bar_width.min(area.width.saturating_sub(2)) as usize;
        let filled = ((bar_w as f64) * frac).round() as usize;
        let filled = filled.min(bar_w);
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_w.saturating_sub(filled));
        // Escape and pad at the boundary: fields are public and may be built directly.
        let label = pad_width(&printable(&self.label), 12);
        let right = truncate_width(&printable(&self.right), 8);
        let mut spans = vec![
            Span::raw(format!("{label} ")),
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
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::CountdownGauge;
///
/// let gauge = CountdownGauge::from(0, 60);
/// assert_eq!(gauge.fraction(), 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct CountdownGauge {
    /// Seconds still remaining.
    ///
    /// ```
    /// use fw_tui::widgets::CountdownGauge;
    ///
    /// assert_eq!(CountdownGauge::from(9, 10).remaining_secs, 9);
    /// ```
    pub remaining_secs: u64,
    /// Seconds in a full gauge; values below one are treated as one.
    ///
    /// ```
    /// use fw_tui::widgets::CountdownGauge;
    ///
    /// assert_eq!(CountdownGauge::from(0, 0).full_secs, 0);
    /// ```
    pub full_secs: u64,
}

impl CountdownGauge {
    /// Creates a countdown gauge, clamping the displayed fraction to `0..=1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::CountdownGauge;
    ///
    /// let gauge = CountdownGauge::from(120, 60);
    /// assert_eq!(gauge.fraction(), 1.0);
    /// ```
    pub fn from(remaining_secs: u64, full_secs: u64) -> Self {
        Self {
            remaining_secs,
            full_secs,
        }
    }

    /// Fraction of the gauge that is still full.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::CountdownGauge;
    ///
    /// assert_eq!(CountdownGauge::from(30, 60).fraction(), 0.5);
    /// ```
    pub fn fraction(&self) -> f64 {
        let full = self.full_secs.max(1) as f64;
        let rem = self.remaining_secs.min(self.full_secs) as f64;
        clamp01(rem / full)
    }

    /// Converts the countdown into a drawable [`Gauge`] with a formatted right-hand time.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::CountdownGauge;
    /// use ratatui::style::Style;
    ///
    /// let gauge = CountdownGauge::from(90, 120).to_gauge("wait", 10, Style::default());
    /// assert_eq!(gauge.right, "1m30s");
    /// ```
    pub fn to_gauge(&self, label: impl Into<String>, bar_width: u16, style: Style) -> Gauge {
        Gauge::new(label, self.fraction(), bar_width, style)
            .right_text(fmt_duration(self.remaining_secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_control_chars, render_text};
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
        assert_eq!(CountdownGauge::from(500, 100).fraction(), 1.0);
        assert_eq!(CountdownGauge::from(1, 0).fraction(), 0.0);
    }

    #[test]
    fn hostile_numbers_clamped() {
        let g = Gauge::new("x", f64::NAN, 10, Style::default());
        assert_eq!(clamp01(g.fraction), 0.0);
        let g2 = Gauge::new("x", f64::INFINITY, 10, Style::default());
        assert_eq!(clamp01(g2.fraction), 1.0);
        assert_eq!(clamp01(f64::NEG_INFINITY), 0.0);
        assert_eq!(clamp01(0.25), 0.25);
    }

    #[test]
    fn gauge_tiny_area_no_panic() {
        for (w, h) in [(0, 0), (1, 1), (2, 1), (3, 2), (200, 2)] {
            let gauge = Gauge::new("longlabel", 0.5, 20, Style::default());
            let text = render_text(w, h, |f| gauge.render(f.area(), f.buffer_mut()));
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn directly_built_gauge_cannot_inject_ansi() {
        let gauge = Gauge {
            label: "l\u{1b}[2J".into(),
            right: "r\u{9b}31m".into(),
            fraction: 0.5,
            bar_width: 12,
            style: Style::default(),
        };
        let text = render_text(40, 1, |f| gauge.render(f.area(), f.buffer_mut()));
        assert_no_control_chars(&text);
        assert!(!text.contains('\u{1b}'));
    }

    #[test]
    fn wide_labels_use_display_width_alignment() {
        for label in ["ab", "中", "e\u{301}f"] {
            let gauge = Gauge::new(label, 1.0, 20, Style::default());
            let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
            terminal
                .draw(|f| gauge.render(f.area(), f.buffer_mut()))
                .unwrap();
            let buffer = terminal.backend().buffer();
            // Inspect cells, not chars: a wide char uses a blank second cell and a combining
            // mark shares a cell with its base character.
            let bar_start = (0..40).find(|&x| buffer[(x, 0)].symbol() == "█");
            assert_eq!(bar_start, Some(13), "{label:?}");
        }
    }

    #[test]
    fn countdown_to_gauge_sanitizes_and_formats() {
        let gauge = CountdownGauge::from(90, 120).to_gauge("w\u{1b}[31m", 10, Style::default());
        assert!(!gauge.label.contains('\u{1b}'));
        assert_eq!(gauge.right, "1m30s");
        assert!((gauge.fraction - 0.75).abs() < 1e-9);
    }
}
