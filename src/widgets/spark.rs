//! One-row sparkline and filled bar panel with shared scaling.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::Sparkline;
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let mut terminal = Terminal::new(TestBackend::new(10, 1)).unwrap();
//! terminal
//!     .draw(|f| {
//!         Sparkline::new(vec![1.0, 3.0, 2.0]).render(
//!             f.area(),
//!             f.buffer_mut(),
//!             Theme::dark().accent,
//!         )
//!     })
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Sparkline as RSparkline;
use ratatui::widgets::Widget;

use crate::theme::Theme;

/// Vertical scale for [`Sparkline`] and [`BarPanel`].
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{Scale, Sparkline};
///
/// let spark = Sparkline::new(vec![1.0]).scale(Scale::Fixed(10.0));
/// assert!(matches!(spark.scale, Scale::Fixed(10.0)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scale {
    /// Scale to the largest finite positive value, with a floor of one.
    ///
    /// ```
    /// use fw_tui::widgets::Scale;
    ///
    /// assert_eq!(Scale::Auto, Scale::Auto);
    /// ```
    Auto,
    /// Use a fixed maximum; non-finite or non-positive values fall back to auto scaling.
    ///
    /// ```
    /// use fw_tui::widgets::Scale;
    ///
    /// assert_ne!(Scale::Fixed(5.0), Scale::Auto);
    /// ```
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
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{Scale, Sparkline};
///
/// let spark = Sparkline::new(vec![0.0, 1.0, 2.0]).scale(Scale::Auto);
/// assert_eq!(spark.values.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct Sparkline {
    /// Samples to draw; non-finite and negative samples count as empty.
    ///
    /// ```
    /// use fw_tui::widgets::Sparkline;
    ///
    /// let spark = Sparkline::new(vec![f64::NAN]);
    /// assert!(spark.values[0].is_nan());
    /// ```
    pub values: Vec<f64>,
    /// Scale used for all samples.
    ///
    /// ```
    /// use fw_tui::widgets::{Scale, Sparkline};
    ///
    /// let spark = Sparkline::new(vec![]);
    /// assert_eq!(spark.scale, Scale::Auto);
    /// ```
    pub scale: Scale,
}

impl Sparkline {
    /// Creates a sparkline with automatic scaling.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Sparkline;
    ///
    /// let spark = Sparkline::new(vec![1.0, 2.0]);
    /// assert_eq!(spark.values.len(), 2);
    /// ```
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            scale: Scale::Auto,
        }
    }

    /// Selects the scale, for example a shared [`Scale::Fixed`] across panels.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{Scale, Sparkline};
    ///
    /// let spark = Sparkline::new(vec![1.0]).scale(Scale::Fixed(4.0));
    /// assert_eq!(spark.scale, Scale::Fixed(4.0));
    /// ```
    pub fn scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    /// Draws the most recent `area.width` samples. A zero-sized area draws nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Sparkline;
    /// use ratatui::backend::TestBackend;
    /// use ratatui::style::Style;
    /// use ratatui::Terminal;
    ///
    /// let spark = Sparkline::new(vec![1.0, 2.0]);
    /// let mut terminal = Terminal::new(TestBackend::new(10, 1)).unwrap();
    /// terminal
    ///     .draw(|f| spark.render(f.area(), f.buffer_mut(), Style::default()))
    ///     .unwrap();
    /// ```
    pub fn render(&self, area: Rect, buf: &mut Buffer, style: Style) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        // ratatui sparkline takes u64 data; scale to max for shared-scale behavior.
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
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::BarPanel;
///
/// let panel = BarPanel::new(vec![1.0, 2.0, 3.0]);
/// assert_eq!(panel.values.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct BarPanel {
    /// Samples to draw; non-finite and negative samples are drawn empty.
    ///
    /// ```
    /// use fw_tui::widgets::BarPanel;
    ///
    /// let panel = BarPanel::new(vec![f64::INFINITY]);
    /// assert!(panel.values[0].is_infinite());
    /// ```
    pub values: Vec<f64>,
    /// Scale used for all bars.
    ///
    /// ```
    /// use fw_tui::widgets::{BarPanel, Scale};
    ///
    /// let panel = BarPanel::new(vec![]).scale(Scale::Fixed(10.0));
    /// assert_eq!(panel.scale, Scale::Fixed(10.0));
    /// ```
    pub scale: Scale,
}

impl BarPanel {
    /// Creates a bar panel with automatic scaling.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::BarPanel;
    ///
    /// let panel = BarPanel::new(vec![1.0]);
    /// assert_eq!(panel.values.len(), 1);
    /// ```
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            scale: Scale::Auto,
        }
    }

    /// Selects the scale.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{BarPanel, Scale};
    ///
    /// let panel = BarPanel::new(vec![1.0]).scale(Scale::Auto);
    /// assert_eq!(panel.scale, Scale::Auto);
    /// ```
    pub fn scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    /// Draws the most recent `area.width` samples from the bottom up.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::theme::Theme;
    /// use fw_tui::widgets::BarPanel;
    /// use ratatui::backend::TestBackend;
    /// use ratatui::Terminal;
    ///
    /// let theme = Theme::dark();
    /// let panel = BarPanel::new(vec![1.0, 2.0]);
    /// let mut terminal = Terminal::new(TestBackend::new(10, 3)).unwrap();
    /// terminal
    ///     .draw(|f| panel.render(f.area(), f.buffer_mut(), &theme))
    ///     .unwrap();
    /// ```
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
    use crate::test_support::{assert_no_control_chars, render_text};

    #[test]
    fn sparkline_and_barpanel_no_panic() {
        let theme = Theme::dark();
        let vals = vec![0.0, 1.0, f64::NAN, 5.0, f64::INFINITY, -1.0, 3.0];
        let text = render_text(10, 3, |f| {
            let area = f.area();
            let top = Rect::new(area.x, area.y, area.width, 1);
            let bot = Rect::new(area.x, area.y + 1, area.width, 2);
            Sparkline::new(vals.clone()).render(top, f.buffer_mut(), theme.accent);
            BarPanel::new(vals).render(bot, f.buffer_mut(), &theme);
        });
        assert_no_control_chars(&text);
    }

    #[test]
    fn shared_scale_fixed() {
        let s = Sparkline::new(vec![1.0, 2.0]).scale(Scale::Fixed(10.0));
        assert!(matches!(s.scale, Scale::Fixed(_)));
        assert_eq!(effective_max(&[1.0, 2.0], Scale::Fixed(10.0)), 10.0);
        assert_eq!(effective_max(&[1.0, 2.0], Scale::Fixed(0.0)), 2.0);
        assert_eq!(effective_max(&[1.0, 2.0], Scale::Fixed(f64::NAN)), 2.0);
        assert_eq!(effective_max(&[], Scale::Auto), 1.0);
        assert_eq!(effective_max(&[-1.0], Scale::Auto), 1.0);
    }

    #[test]
    fn empty_values_and_zero_areas_are_safe() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (3, 2), (200, 4)] {
            let text = render_text(w, h, |f| {
                Sparkline::new(vec![]).render(f.area(), f.buffer_mut(), theme.accent);
                BarPanel::new(vec![]).render(f.area(), f.buffer_mut(), &theme);
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn many_values_are_clipped_to_the_visible_width() {
        let theme = Theme::dark();
        let values: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let text = render_text(20, 4, |f| {
            BarPanel::new(values.clone()).render(f.area(), f.buffer_mut(), &theme)
        });
        assert_no_control_chars(&text);
        assert!(text.contains('█'));
    }

    #[test]
    fn resize_keeps_bars_drawable() {
        let theme = Theme::dark();
        let values = vec![1.0, 4.0, 2.0];
        for (w, h) in [(40, 6), (2, 1), (120, 3)] {
            let text = render_text(w, h, |f| {
                Sparkline::new(values.clone()).render(f.area(), f.buffer_mut(), theme.accent);
                BarPanel::new(values.clone()).render(f.area(), f.buffer_mut(), &theme);
            });
            assert_no_control_chars(&text);
        }
    }
}
