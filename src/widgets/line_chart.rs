use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget};

use crate::sanitize::printable;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomAgg {
    Max,
    Mean,
}

#[derive(Debug, Clone)]
pub struct Series {
    pub name: String,
    pub points: Vec<Option<f64>>, // missing => gap
}

impl Series {
    pub fn new(name: impl Into<String>, points: Vec<Option<f64>>) -> Self {
        Self {
            name: printable(&name.into()),
            points,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineChartConfig {
    pub zoom: usize,
    pub agg: ZoomAgg,
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
    let shown: Vec<Series> = series.iter().take(max_series).cloned().collect();
    let extra = series.len().saturating_sub(max_series);
    let zoom = cfg.zoom.max(1);

    // Prepare data per series, splitting gaps into segments for Chart (Chart doesn't natively gap; use NaN breaks not supported)
    // ratatui Chart draws lines between points in dataset; to draw gaps we split into datasets per contiguous segment.
    let mut datasets: Vec<Dataset> = Vec::new();
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
        // split segments on None
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
            // idle flat line at 0
            segs.push(vec![(0.0, 0.0)]);
        }
        for segv in segs {
            // store in a leaked-like structure? Dataset holds &[ (f64,f64) ] by ref; need owned storage alive.
            // We'll collect owned and rebuild later by storing boxes - easier: use owned Vec and keep in parallel vec.
            datasets.push(
                Dataset::default()
                    .name(if focused || shown.len() == 1 {
                        s.name.clone()
                    } else {
                        String::new()
                    })
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(style)
                    .data(&[]),
            ); // placeholder, fix below
               // can't set data ref easily; workaround: render manually? but Chart is convenient. Instead build data store.
            let _ = segv; // handled below
        }
    }

    // Rebuild properly with owned storage
    let mut data_store: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut ds_meta: Vec<(String, Style)> = Vec::new();
    for (idx, s) in shown.iter().enumerate() {
        let agg = aggregate(&s.points, zoom, cfg.agg);
        let focused = idx == cfg.focused.min(shown.len().saturating_sub(1));
        let style = if shown.len() <= 1 {
            theme.accent
        } else if focused {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            theme.dim
        };
        let mut seg: Vec<(f64, f64)> = Vec::new();
        let mut segs: Vec<Vec<(f64, f64)>> = Vec::new();
        for (i, p) in agg.iter().enumerate() {
            if let Some(v) = p {
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

    // autoscale with floor so idle draws flat
    let y_max = if max_y <= 0.0 { 1.0 } else { max_y };
    let x_max = sample_count.saturating_sub(1).max(1) as f64;
    let x_label_right = if zoom > 1 {
        format!("{} ({}x)", sample_count, zoom)
    } else {
        format!("{}", sample_count)
    };

    // Legend box text on right (render chart then overlay legend manually area)
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
    use crate::text::buffer_text;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn line_chart_labels_legend_zoom() {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).unwrap();
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
        terminal
            .draw(|f| render_line_chart(f.area(), f.buffer_mut(), &series, &cfg, &theme))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("0"), "{text}");
        assert!(text.contains("(2x)") || text.contains("2x"), "{text}");
    }

    #[test]
    fn line_chart_tiny_no_panic() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let series = vec![Series::new(
            "a",
            vec![None, Some(f64::NAN), Some(f64::INFINITY)],
        )];
        terminal
            .draw(|f| {
                render_line_chart(
                    f.area(),
                    f.buffer_mut(),
                    &series,
                    &LineChartConfig::default(),
                    &theme,
                )
            })
            .unwrap();
        let _ = buffer_text(terminal.backend().buffer());
    }
}
