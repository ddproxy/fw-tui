use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table as RTable, Widget};

use crate::sanitize::printable;
use crate::text::truncate_width;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColWidth {
    Fixed(u16),
    Min(u16),
    Flex(u16),
}

#[derive(Debug, Clone)]
pub struct Column {
    pub title: String,
    pub width: ColWidth,
    pub right: bool,
}

impl Column {
    pub fn new(title: impl Into<String>, width: ColWidth) -> Self {
        Self {
            title: printable(&title.into()),
            width,
            right: false,
        }
    }
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
    pub status: Option<Span<'static>>, // one-cell dot
    pub detail: Vec<String>,           // expanded lines under selected row
}

impl TableRow {
    pub fn new(cells: Vec<String>) -> Self {
        Self {
            cells,
            status: None,
            detail: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableViewState {
    pub selected: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct TableRenderInfo {
    pub visible_range: std::ops::Range<usize>,
}

/// Render a table with header, selection, optional status dot column and expandable selected row.
/// Returns visible row range (data indices).
pub fn render_table(
    area: Rect,
    buf: &mut Buffer,
    columns: &[Column],
    rows: &[TableRow],
    state: &mut TableViewState,
    theme: &Theme,
) -> TableRenderInfo {
    if area.width == 0 || area.height == 0 || columns.is_empty() {
        return TableRenderInfo {
            visible_range: 0..0,
        };
    }
    let has_status = rows.iter().any(|r| r.status.is_some());
    let mut cols: Vec<Column> = Vec::new();
    if has_status {
        cols.push(Column::new("", ColWidth::Fixed(2)));
    }
    cols.extend(columns.iter().cloned());
    let widths: Vec<Constraint> = cols
        .iter()
        .map(|c| match c.width {
            ColWidth::Fixed(w) => Constraint::Length(w),
            ColWidth::Min(w) => Constraint::Min(w),
            ColWidth::Flex(w) => Constraint::Fill(w.max(1)),
        })
        .collect();

    let inner_h = area.height.saturating_sub(2) as usize; // header inside bordered table area minus header row
    let selected = if rows.is_empty() {
        0
    } else {
        state.selected.min(rows.len() - 1)
    };
    state.selected = selected;

    // compute window with expansion
    let mut offset = state.offset.min(selected);
    // ensure selected visible considering expansion
    loop {
        let (start, end) = window_range(rows, offset, selected, inner_h.max(1));
        if selected < start {
            offset = selected;
            continue;
        }
        if selected >= end && end < rows.len() {
            offset = offset.saturating_add(1);
            continue;
        }
        state.offset = offset;
        break;
    }
    let (start, end) = window_range(rows, state.offset, selected, inner_h.max(1));

    // build header row (reversed bar style)
    let header_cells = cols.iter().map(|c| {
        Cell::from(Line::from(Span::styled(
            truncate_width(&c.title, 20),
            theme.title,
        )))
    });
    let header = Row::new(header_cells).style(theme.header_bar).height(1);

    let mut table_rows: Vec<Row> = Vec::new();
    let mut detail_rows: Vec<(usize, Vec<String>)> = Vec::new();
    for (i, row) in rows.iter().enumerate().take(end).skip(start) {
        let mut cells: Vec<Cell> = Vec::new();
        let mut values: Vec<String> = Vec::new();
        if has_status {
            values.push(String::new());
        }
        for c in &row.cells {
            values.push(printable(c));
        }
        if has_status {
            if let Some(st) = &row.status {
                cells.push(Cell::from(Line::from(st.clone())));
            } else {
                cells.push(Cell::from(" "));
            }
        }
        for (ci, col) in columns.iter().enumerate() {
            let val = values
                .get(ci + if has_status { 1 } else { 0 })
                .cloned()
                .unwrap_or_default();
            let t = truncate_width(&val, 40);
            let styled = if col.right {
                Line::from(t).right_aligned()
            } else {
                Line::from(t)
            };
            cells.push(Cell::from(styled));
        }
        let style = if i == selected {
            theme.selected_row
        } else {
            Style::default()
        };
        table_rows.push(Row::new(cells).style(style).height(1));
        if i == selected && !row.detail.is_empty() {
            detail_rows.push((i, row.detail.clone()));
        }
    }

    // Render table blockless inside area? use bordered block for clarity.
    let table = RTable::new(table_rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .column_spacing(1);
    table.render(area, buf);

    // Draw expansion lines under selected row inside table area (best effort)
    if let Some((_, details)) = detail_rows.first() {
        // find y of selected row: header at area.y, data starts at area.y+1 relative to table content without border
        let rel = selected.saturating_sub(start);
        let base_y = area
            .y
            .saturating_add(1)
            .saturating_add(rel as u16)
            .saturating_add(1);
        for (j, line) in details.iter().enumerate() {
            let y = base_y.saturating_add(j as u16);
            if y >= area.y.saturating_add(area.height) {
                break;
            }
            let text = truncate_width(&printable(line), area.width.saturating_sub(2) as usize);
            let rect = Rect::new(area.x.saturating_add(2), y, area.width.saturating_sub(2), 1);
            Paragraph::new(text).style(theme.dim).render(rect, buf);
        }
    }

    TableRenderInfo {
        visible_range: start..end,
    }
}

fn window_range(
    rows: &[TableRow],
    offset: usize,
    selected: usize,
    capacity: usize,
) -> (usize, usize) {
    if rows.is_empty() || capacity == 0 {
        return (0, 0);
    }
    let mut start = offset.min(rows.len().saturating_sub(1));
    let mut used = 0usize;
    let mut end = start;
    while end < rows.len() && used < capacity {
        used += 1; // row
        if end == selected {
            let extra = rows[end].detail.len().min(capacity.saturating_sub(used));
            used += extra;
        }
        if used > capacity && end == start {
            end += 1;
            break;
        }
        if used > capacity {
            break;
        }
        end += 1;
    }
    if selected >= end && end < rows.len() {
        // shift start forward until fits
        while start < selected {
            start += 1;
            let (_s, _e) = (start, end);
            let mut u = 0;
            let mut e = start;
            while e < rows.len() && u < capacity {
                u += 1;
                if e == selected {
                    u += rows[e].detail.len().min(capacity.saturating_sub(u));
                }
                if u > capacity {
                    break;
                }
                e += 1;
            }
            if selected < e || e == rows.len() {
                end = e;
                break;
            }
            end = e;
        }
    }
    (start, end.min(rows.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::buffer_text;
    use crate::widgets::status_dot::{status_dot, DotState};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn table_renders_and_scrolls() {
        let backend = TestBackend::new(40, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let cols = vec![
            Column::new("Name", ColWidth::Flex(1)),
            Column::new("N", ColWidth::Fixed(4)).right(),
        ];
        let mut rows = vec![];
        for i in 0..20 {
            rows.push(TableRow::new(vec![format!("row-{i}"), format!("{i}")]));
        }
        rows[2].detail = vec!["detail-a".into(), "detail-b".into()];
        rows[2].status = Some(status_dot(DotState::Good, &theme, false));
        let mut state = TableViewState {
            selected: 2,
            offset: 0,
        };
        terminal
            .draw(|f| {
                render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Name"));
        assert!(text.contains("row-2"));
    }

    #[test]
    fn table_tiny_no_panic() {
        for (w, h) in [(1, 1), (0, 0), (10, 2)] {
            let backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            let theme = Theme::dark();
            let cols = vec![Column::new("A", ColWidth::Flex(1))];
            let rows = vec![TableRow::new(vec!["x".into()])];
            let mut state = TableViewState::default();
            terminal
                .draw(|f| {
                    render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
                })
                .unwrap();
            let _ = buffer_text(terminal.backend().buffer());
        }
    }

    #[test]
    fn hostile_strings_and_numbers() {
        let backend = TestBackend::new(30, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let cols = vec![Column::new("A", ColWidth::Flex(1))];
        let rows = vec![TableRow::new(vec!["a\u{1b}[31m\u{202e}b".into()])];
        let mut state = TableViewState::default();
        terminal
            .draw(|f| {
                render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(!text.contains('\u{1b}'));
    }
}
