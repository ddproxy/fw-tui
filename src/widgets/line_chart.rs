//! Multi-series line chart with axes, gaps, zoom aggregation and a boxed legend.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{render_line_chart, LineChartConfig, Series};
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let series = vec![Series::new("rtt", vec![Some(1.0), None, Some(3.0)])];
//! let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
//! terminal
//!     .draw(|f| {
//!         render_line_chart(
//!             f.area(),
//!             f.buffer_mut(),
//!             &series,
//!             &LineChartConfig::default(),
//!             &Theme::dark(),
//!         )
//!     })
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget};

use crate::sanitize::printable;
use crate::theme::Theme;

/// How samples inside a zoom bucket are combined.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{LineChartConfig, ZoomAgg};
///
/// let cfg = LineChartConfig { agg: ZoomAgg::Mean, ..Default::default() };
/// assert_eq!(cfg.agg, ZoomAgg::Mean);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomAgg {
    /// Keep the largest finite sample in the bucket.
    ///
    /// ```
    /// use fw_tui::widgets::ZoomAgg;
    ///
    /// assert_eq!(ZoomAgg::Max, ZoomAgg::Max);
    /// ```
    Max,
    /// Use the mean of the finite samples in the bucket.
    ///
    /// ```
    /// use fw_tui::widgets::ZoomAgg;
    ///
    /// assert_ne!(ZoomAgg::Mean, ZoomAgg::Max);
    /// ```
    Mean,
}

/// One named series; `None` points are drawn as gaps.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::Series;
///
/// let series = Series::new("rtt", vec![Some(1.0), None, Some(3.0)]);
/// assert_eq!(series.points.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct Series {
    /// Legend/dataset name, escaped before drawing.
    ///
    /// ```
    /// use fw_tui::widgets::Series;
    ///
    /// let series = Series::new("rtt", vec![]);
    /// assert_eq!(series.name, "rtt");
    /// ```
    pub name: String,
    /// Samples in order; missing, negative and non-finite values are ignored.
    ///
    /// ```
    /// use fw_tui::widgets::Series;
    ///
    /// let series = Series { name: "x".into(), points: vec![None, Some(f64::NAN)] };
    /// assert_eq!(series.points.len(), 2);
    /// ```
    pub points: Vec<Option<f64>>,
}

impl Series {
    /// Creates a series with an escaped name.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::Series;
    ///
    /// let series = Series::new("r\u{1b}[31m", vec![]);
    /// assert!(!series.name.contains('\u{1b}'));
    /// ```
    pub fn new(name: impl Into<String>, points: Vec<Option<f64>>) -> Self {
        Self {
            name: printable(&name.into()),
            points,
        }
    }
}

/// Chart options: zoom factor, aggregation and focused series index.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{LineChartConfig, ZoomAgg};
///
/// let cfg = LineChartConfig::default();
/// assert_eq!(cfg.zoom, 1);
/// assert_eq!(cfg.agg, ZoomAgg::Max);
/// assert_eq!(cfg.focused, 0);
/// ```
#[derive(Debug, Clone)]
pub struct LineChartConfig {
    /// Number of samples combined into one displayed point; values below one act as one.
    ///
    /// ```
    /// use fw_tui::widgets::LineChartConfig;
    ///
    /// let cfg = LineChartConfig { zoom: 4, ..Default::default() };
    /// assert_eq!(cfg.zoom, 4);
    /// ```
    pub zoom: usize,
    /// Reduction used for each zoom bucket.
    ///
    /// ```
    /// use fw_tui::widgets::{LineChartConfig, ZoomAgg};
    ///
    /// let cfg = LineChartConfig { agg: ZoomAgg::Max, ..Default::default() };
    /// assert_eq!(cfg.agg, ZoomAgg::Max);
    /// ```
    pub agg: ZoomAgg,
    /// Index of the highlighted series; out-of-range values clamp to the last one.
    ///
    /// ```
    /// use fw_tui::widgets::LineChartConfig;
    ///
    /// let cfg = LineChartConfig { focused: 99, ..Default::default() };
    /// assert_eq!(cfg.focused, 99);
    /// ```
    pub focused: usize,
}

impl Default for LineChartConfig {
    fn default() -> Self {
        Self {
            zoom: 1,
            agg: ZoomAgg::Max,
            focused: 0,
        }
    }
}

fn clean_point(v: Option<f64>) -> Option<f64> {
    match v {
        Some(x) if x.is_finite() => Some(x.max(0.0)),
        _ => None,
    }
}

fn aggregate(points: &[Option<f64>], zoom: usize, agg: ZoomAgg) -> Vec<Option<f64>> {
    let zoom = zoom.max(1);
    if zoom == 1 {
        return points.iter().map(|p| clean_point(*p)).collect();
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < points.len() {
        let end = (i + zoom).min(points.len());
        let chunk = &points[i..end];
        let agg_val = match agg {
            ZoomAgg::Max => chunk
                .iter()
                .filter_map(|p| clean_point(*p))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)),
            ZoomAgg::Mean => {
                let collected: Vec<f64> = chunk.iter().filter_map(|p| clean_point(*p)).collect();
                if collected.is_empty() {
                    None
                } else {
                    Some(collected.iter().sum::<f64>() / collected.len() as f64)
                }
            }
        };
        out.push(agg_val);
        i = end;
    }
    out
}

/// Render a multi-series line chart with y ticks, x labels, zoom factor and boxed legend.
///
/// Handles empty input, more than sixteen series (which are reported as `+N` in the legend),
/// hostile floats and zero-sized areas. Series names are escaped here as well as in
/// [`Series::new`], because the struct fields are public.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::{render_line_chart, LineChartConfig, Series};
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let series = vec![Series::new("a", vec![Some(1.0), Some(2.0)])];
/// let mut terminal = Terminal::new(TestBackend::new(50, 8)).unwrap();
/// terminal
///     .draw(|f| {
///         render_line_chart(
///             f.area(),
///             f.buffer_mut(),
///             &series,
///             &LineChartConfig::default(),
///             &Theme::dark(),
///         )
///     })
///     .unwrap();
/// ```
pub fn render_line_chart(
    area: Rect,
    buf: &mut Buffer,
    series: &[Series],
    cfg: &LineChartConfig,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        Paragraph::new("").render(area, buf);
        return;
    }
    let max_series = 16usize;
    let shown: Vec<Series> = series
        .iter()
        .take(max_series)
        .map(|s| Series {
            name: printable(&s.name),
            points: s.points.clone(),
        })
        .collect();
    let extra = series.len().saturating_sub(max_series);
    let zoom = cfg.zoom.max(1);

    // Dataset slices must outlive the Chart, so keep the owned data in `data_store`.
    let mut data_store: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut ds_meta: Vec<(String, Style)> = Vec::new();
    let mut max_y = 0.0f64;
    let mut sample_count = 0usize;

    for (idx, s) in shown.iter().enumerate() {
        let agg = aggregate(&s.points, zoom, cfg.agg);
        sample_count = sample_count.max(agg.len());
        let focused = idx == cfg.focused.min(shown.len().saturating_sub(1));
        let style = if shown.len() <= 1 {
            theme.accent
        } else if focused {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            theme.dim
        };
        // Chart has no gap marker, so split missing samples into separate datasets.
        let mut seg: Vec<(f64, f64)> = Vec::new();
        let mut segs: Vec<Vec<(f64, f64)>> = Vec::new();
        for (i, p) in agg.iter().enumerate() {
            if let Some(v) = p {
                max_y = max_y.max(*v);
                seg.push((i as f64, *v));
            } else if !seg.is_empty() {
                segs.push(seg);
                seg = Vec::new();
            }
        }
        if !seg.is_empty() {
            segs.push(seg);
        }
        if segs.is_empty() {
            // Idle series still draw a flat line.
            segs.push(vec![
                (0.0, 0.0),
                (sample_count.saturating_sub(1).max(1) as f64, 0.0),
            ]);
        }
        for (si, segv) in segs.into_iter().enumerate() {
            let name = if si == 0 {
                s.name.clone()
            } else {
                String::new()
            };
            ds_meta.push((name, style));
            data_store.push(segv);
        }
    }
    let datasets: Vec<Dataset> = ds_meta
        .iter()
        .zip(data_store.iter())
        .map(|((name, style), data)| {
            Dataset::default()
                .name(name.as_str())
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(*style)
                .data(data)
        })
        .collect();

    // Autoscale with a floor so idle data draws a flat line.
    let y_max = if max_y <= 0.0 { 1.0 } else { max_y };
    let x_max = sample_count.saturating_sub(1).max(1) as f64;
    let x_label_right = if zoom > 1 {
        format!("{} ({}x)", sample_count, zoom)
    } else {
        format!("{}", sample_count)
    };

    let legend_lines: Vec<String> = {
        let mut l: Vec<String> = shown
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mark = if i == cfg.focused.min(shown.len().saturating_sub(1)) {
                    "*"
                } else {
                    " "
                };
                format!("{mark}{}", s.name)
            })
            .collect();
        if extra > 0 {
            l.push(format!("+{extra}"));
        }
        l
    };

    let chart = Chart::new(datasets)
        .block(Block::default().borders(Borders::NONE))
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(vec![Span::raw("0"), Span::raw(x_label_right)])
                .style(theme.dim),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(vec![
                    Span::raw("0"),
                    Span::raw(format!("{:.1}", y_max / 2.0)),
                    Span::raw(format!("{:.1}", y_max)),
                ])
                .style(theme.dim),
        );
    // The legend gets its own gutter on the right, so it never sits on top of a line.
    let legend_width = legend_lines
        .iter()
        .map(|s| crate::text::display_width(s))
        .max()
        .unwrap_or(0)
        .min(12) as u16
        + 2;
    let show_legend = shown.len() > 1
        && !legend_lines.is_empty()
        && area.height > 4
        && area.width > legend_width.saturating_add(24);
    let chart_area = if show_legend {
        Rect::new(
            area.x,
            area.y,
            area.width.saturating_sub(legend_width + 1),
            area.height,
        )
    } else {
        area
    };
    chart.render(chart_area, buf);

    if show_legend {
        let lw = legend_width;
        let lh = (legend_lines.len() as u16 + 2).min(area.height.saturating_sub(1));
        let lx = area.x + area.width.saturating_sub(lw);
        let ly = area.y + 1;
        let lrect = Rect::new(lx, ly, lw, lh);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border);
        let inner = block.inner(lrect);
        block.render(lrect, buf);
        let lines: Vec<Line> = legend_lines
            .into_iter()
            .enumerate()
            .map(|(i, s)| {
                let focused = i == cfg.focused.min(shown.len().saturating_sub(1));
                let style = if focused { theme.accent } else { theme.dim };
                Line::from(Span::styled(
                    crate::text::truncate_width(&s, inner.width as usize),
                    style,
                ))
            })
            .collect();
        Paragraph::new(lines).render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_no_control_chars, render_text};

    #[test]
    fn line_chart_labels_legend_zoom() {
        let theme = Theme::dark();
        let series = vec![
            Series::new("a", vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)]),
            Series::new("b", vec![Some(2.0), Some(1.0), None, Some(2.0)]),
        ];
        let cfg = LineChartConfig {
            zoom: 2,
            agg: ZoomAgg::Max,
            focused: 0,
        };
        let text = render_text(60, 12, |f| {
            render_line_chart(f.area(), f.buffer_mut(), &series, &cfg, &theme)
        });
        assert!(text.contains("0"), "{text}");
        assert!(text.contains("2x") || text.contains("(2x)"), "{text}");
        assert!(text.contains("*a"), "{text}");
    }

    #[test]
    fn line_chart_tiny_no_panic() {
        let theme = Theme::dark();
        for (w, h) in [(0, 0), (1, 1), (2, 3), (200, 4)] {
            let text = render_text(w, h, |f| {
                render_line_chart(
                    f.area(),
                    f.buffer_mut(),
                    &[Series::new(
                        "a",
                        vec![None, Some(f64::NAN), Some(f64::INFINITY)],
                    )],
                    &LineChartConfig::default(),
                    &theme,
                )
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn empty_and_oversized_series_are_safe() {
        let theme = Theme::dark();
        let empty = render_text(30, 6, |f| {
            render_line_chart(
                f.area(),
                f.buffer_mut(),
                &[],
                &LineChartConfig::default(),
                &theme,
            )
        });
        assert_no_control_chars(&empty);

        let many: Vec<Series> = (0..20)
            .map(|i| Series::new(format!("s{i}"), vec![Some(i as f64)]))
            .collect();
        let text = render_text(70, 10, |f| {
            render_line_chart(
                f.area(),
                f.buffer_mut(),
                &many,
                &LineChartConfig {
                    focused: 99,
                    ..Default::default()
                },
                &theme,
            )
        });
        assert!(text.contains("+4") || text.contains("s"), "{text}");
    }

    #[test]
    fn direct_series_names_cannot_inject_ansi() {
        let theme = Theme::dark();
        let series = vec![
            Series {
                name: "a\u{1b}[2J".into(),
                points: vec![Some(1.0)],
            },
            Series {
                name: "b\u{202e}".into(),
                points: vec![Some(2.0)],
            },
        ];
        let text = render_text(70, 10, |f| {
            render_line_chart(
                f.area(),
                f.buffer_mut(),
                &series,
                &LineChartConfig::default(),
                &theme,
            )
        });
        assert_no_control_chars(&text);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{202e}'));
    }

    #[test]
    fn aggregation_modes_and_hostile_points() {
        let points = vec![Some(1.0), None, Some(f64::NAN), Some(5.0), Some(-3.0)];
        assert_eq!(aggregate(&points, 0, ZoomAgg::Max).len(), 5);
        let maxed = aggregate(&points, 2, ZoomAgg::Max);
        assert_eq!(maxed[0], Some(1.0));
        assert_eq!(maxed[1], Some(5.0));
        assert_eq!(maxed[2], Some(0.0));
        let mean = aggregate(&points, 3, ZoomAgg::Mean);
        assert_eq!(mean.len(), 2);
        assert!((mean[1].unwrap() - 2.5).abs() < 1e-9);
        assert_eq!(aggregate(&[None], 4, ZoomAgg::Mean)[0], None);
    }

    #[test]
    fn resizing_keeps_chart_drawable() {
        let theme = Theme::dark();
        let series = vec![
            Series::new("a", vec![Some(1.0), None, Some(4.0)]),
            Series::new("b", vec![Some(2.0), Some(3.0), None]),
        ];
        for (w, h) in [(80, 12), (12, 3), (240, 8), (40, 5)] {
            let text = render_text(w, h, |f| {
                render_line_chart(
                    f.area(),
                    f.buffer_mut(),
                    &series,
                    &LineChartConfig::default(),
                    &theme,
                )
            });
            assert_no_control_chars(&text);
        }
    }
}
