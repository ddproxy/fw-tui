//! Selectable table with fixed/min/flex columns, status dots and expandable details.
//!
//! Requires the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{render_table, ColWidth, Column, TableRow, TableViewState};
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let cols = vec![Column::new("Name", ColWidth::Flex(1))];
//! let rows = vec![TableRow::new(vec!["peer-a".into()])];
//! let mut state = TableViewState::default();
//! let mut terminal = Terminal::new(TestBackend::new(30, 5)).unwrap();
//! terminal
//!     .draw(|f| {
//!         render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &Theme::dark());
//!     })
//!     .unwrap();
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table as RTable, Widget};

use crate::sanitize::printable;
use crate::text::truncate_width;
use crate::theme::Theme;

/// How a table column claims horizontal space.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{ColWidth, Column};
///
/// let column = Column::new("N", ColWidth::Fixed(4));
/// assert_eq!(column.width, ColWidth::Fixed(4));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColWidth {
    /// Exactly this many cells.
    ///
    /// ```
    /// use fw_tui::widgets::ColWidth;
    ///
    /// assert_eq!(ColWidth::Fixed(4), ColWidth::Fixed(4));
    /// ```
    Fixed(u16),
    /// At least this many cells, sharing the rest with other columns.
    ///
    /// ```
    /// use fw_tui::widgets::ColWidth;
    ///
    /// assert_ne!(ColWidth::Min(4), ColWidth::Fixed(4));
    /// ```
    Min(u16),
    /// Flexible share of the remaining space.
    ///
    /// ```
    /// use fw_tui::widgets::ColWidth;
    ///
    /// assert_eq!(ColWidth::Flex(2), ColWidth::Flex(2));
    /// ```
    Flex(u16),
}

/// One table column.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{ColWidth, Column};
///
/// let column = Column::new("Count", ColWidth::Fixed(6)).right();
/// assert!(column.right);
/// ```
#[derive(Debug, Clone)]
pub struct Column {
    /// Header text, escaped at construction and again when drawn.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, Column};
    ///
    /// let column = Column::new("N\u{1b}[31m", ColWidth::Fixed(4));
    /// assert!(!column.title.contains('\u{1b}'));
    /// ```
    pub title: String,
    /// Width rule for this column.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, Column};
    ///
    /// let column = Column::new("N", ColWidth::Min(4));
    /// assert_eq!(column.width, ColWidth::Min(4));
    /// ```
    pub width: ColWidth,
    /// Whether cell text is right-aligned.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, Column};
    ///
    /// let column = Column::new("N", ColWidth::Fixed(4)).right();
    /// assert!(column.right);
    /// ```
    pub right: bool,
}

impl Column {
    /// Creates a left-aligned column with an escaped title.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, Column};
    ///
    /// let column = Column::new("Name", ColWidth::Flex(1));
    /// assert_eq!(column.title, "Name");
    /// assert!(!column.right);
    /// ```
    pub fn new(title: impl Into<String>, width: ColWidth) -> Self {
        Self {
            title: printable(&title.into()),
            width,
            right: false,
        }
    }

    /// Makes the column right-aligned.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, Column};
    ///
    /// let column = Column::new("N", ColWidth::Fixed(4)).right();
    /// assert!(column.right);
    /// ```
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

/// One data row, with an optional status dot and expanded detail lines.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::TableRow;
///
/// let mut row = TableRow::new(vec!["peer-a".into(), "12".into()]);
/// row.detail.push("more".into());
/// assert_eq!(row.cells.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct TableRow {
    /// Cell strings, escaped when drawn.
    ///
    /// ```
    /// use fw_tui::widgets::TableRow;
    ///
    /// let row = TableRow::new(vec!["x".into()]);
    /// assert_eq!(row.cells[0], "x");
    /// ```
    pub cells: Vec<String>,
    /// Optional one-cell marker; rendered as the first column when any row has one.
    ///
    /// ```
    /// use fw_tui::widgets::TableRow;
    ///
    /// let row = TableRow::new(vec![]);
    /// assert!(row.status.is_none());
    /// ```
    pub status: Option<Span<'static>>,
    /// Extra lines drawn under the row when it is selected.
    ///
    /// ```
    /// use fw_tui::widgets::TableRow;
    ///
    /// let mut row = TableRow::new(vec![]);
    /// row.detail.push("detail".into());
    /// assert_eq!(row.detail.len(), 1);
    /// ```
    pub detail: Vec<String>,
}

impl TableRow {
    /// Creates a row from its cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::TableRow;
    ///
    /// let row = TableRow::new(vec!["a".into(), "b".into()]);
    /// assert_eq!(row.cells.len(), 2);
    /// ```
    pub fn new(cells: Vec<String>) -> Self {
        Self {
            cells,
            status: None,
            detail: vec![],
        }
    }
}

/// Mutable selection and scroll position for [`render_table`].
///
/// Both fields are clamped on every render, so stale state after a resize or a shorter data set
/// cannot point outside the table.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::TableViewState;
///
/// let state = TableViewState { selected: 3, offset: 1 };
/// assert_eq!(state.selected, 3);
/// ```
#[derive(Debug, Clone, Default)]
pub struct TableViewState {
    /// Index of the selected data row.
    ///
    /// ```
    /// use fw_tui::widgets::TableViewState;
    ///
    /// let state = TableViewState::default();
    /// assert_eq!(state.selected, 0);
    /// ```
    pub selected: usize,
    /// Index of the first visible data row.
    ///
    /// ```
    /// use fw_tui::widgets::TableViewState;
    ///
    /// let state = TableViewState { selected: 5, offset: 2 };
    /// assert_eq!(state.offset, 2);
    /// ```
    pub offset: usize,
}

/// Which data rows were actually drawn.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::TableRenderInfo;
///
/// let info = TableRenderInfo { visible_range: 2..5 };
/// assert_eq!(info.visible_range, 2..5);
/// ```
#[derive(Debug, Clone)]
pub struct TableRenderInfo {
    /// Half-open range of visible data indices.
    ///
    /// ```
    /// use fw_tui::widgets::TableRenderInfo;
    ///
    /// let info = TableRenderInfo { visible_range: 0..0 };
    /// assert!(info.visible_range.is_empty());
    /// ```
    pub visible_range: std::ops::Range<usize>,
}

/// Render a table with header, selection, optional status dot column and expandable selected row.
/// Returns visible row range (data indices).
///
/// Empty column lists, empty row lists and zero-sized areas are safe; in those cases the returned
/// range is empty and the selection/scroll state is still clamped. Header text is escaped even
/// when [`Column`] was built directly.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::{render_table, ColWidth, Column, TableRow, TableViewState};
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let cols = vec![Column::new("Name", ColWidth::Flex(1))];
/// let rows = vec![TableRow::new(vec!["peer-a".into()])];
/// let mut state = TableViewState::default();
/// let mut terminal = Terminal::new(TestBackend::new(30, 5)).unwrap();
/// terminal
///     .draw(|f| {
///         render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &Theme::dark());
///     })
///     .unwrap();
/// assert_eq!(state.selected, 0);
/// ```
pub fn render_table(
    area: Rect,
    buf: &mut Buffer,
    columns: &[Column],
    rows: &[TableRow],
    state: &mut TableViewState,
    theme: &Theme,
) -> TableRenderInfo {
    // Clamp state even when there is nothing to draw: callers keep this state across resizes and
    // data changes, and stale indices must not survive a zero-sized render.
    let selected = if rows.is_empty() {
        0
    } else {
        state.selected.min(rows.len() - 1)
    };
    state.selected = selected;
    state.offset = state.offset.min(selected);

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

    let inner_h = area.height.saturating_sub(2) as usize; // header inside the table area

    // Compute a window that leaves room for the selected row's expanded detail.
    let mut offset = state.offset;
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

    // Build the header row (reversed bar style). Direct construction of Column is public, so
    // escape here too rather than trusting the constructor alone.
    let header_cells = cols.iter().map(|c| {
        Cell::from(Line::from(Span::styled(
            truncate_width(&printable(&c.title), 20),
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

    let table = RTable::new(table_rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .column_spacing(1);
    table.render(area, buf);

    // Draw expansion lines under the selected row inside the table area (best effort).
    if let Some((_, details)) = detail_rows.first() {
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
        // Shift start forward until the selected row fits.
        while start < selected {
            start += 1;
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
    use crate::test_support::{assert_no_control_chars, render_text};
    use crate::widgets::status_dot::{status_dot, DotState};

    #[test]
    fn table_renders_and_scrolls() {
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
        let mut info = None;
        let text = render_text(40, 8, |f| {
            info = Some(render_table(
                f.area(),
                f.buffer_mut(),
                &cols,
                &rows,
                &mut state,
                &theme,
            ));
        });
        assert!(text.contains("Name"));
        assert!(text.contains("row-2"));
        assert!(info.unwrap().visible_range.contains(&2));
    }

    #[test]
    fn table_tiny_no_panic() {
        let theme = Theme::dark();
        for (w, h) in [(1, 1), (0, 0), (10, 2), (200, 4)] {
            let cols = vec![Column::new("A", ColWidth::Flex(1))];
            let rows = vec![TableRow::new(vec!["x".into()])];
            let mut state = TableViewState::default();
            let text = render_text(w, h, |f| {
                render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn hostile_strings_and_numbers() {
        let theme = Theme::dark();
        let cols = vec![Column::new("A", ColWidth::Flex(1))];
        let rows = vec![TableRow::new(vec!["a\u{1b}[31m\u{202e}b".into()])];
        let mut state = TableViewState::default();
        let text = render_text(30, 6, |f| {
            render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
        });
        assert!(!text.contains('\u{1b}'));
        assert_no_control_chars(&text);
    }

    #[test]
    fn direct_column_and_row_fields_cannot_inject_ansi() {
        let theme = Theme::dark();
        let cols = vec![Column {
            title: "h\u{1b}[2J".into(),
            width: ColWidth::Flex(1),
            right: false,
        }];
        let rows = vec![TableRow {
            cells: vec!["c\u{9b}31m".into()],
            status: None,
            detail: vec!["d\u{1b}[31m".into()],
        }];
        let mut state = TableViewState::default();
        let text = render_text(30, 6, |f| {
            render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
        });
        assert_no_control_chars(&text);
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{9b}'));
    }

    #[test]
    fn empty_lists_produce_empty_visible_range() {
        let theme = Theme::dark();
        let mut state = TableViewState {
            selected: 7,
            offset: 5,
        };
        let mut info = None;
        let text = render_text(30, 5, |f| {
            info = Some(render_table(
                f.area(),
                f.buffer_mut(),
                &[],
                &[],
                &mut state,
                &theme,
            ));
        });
        assert!(text.trim().is_empty());
        assert_eq!(info.unwrap().visible_range, 0..0);
        assert_eq!(state.selected, 0);
        assert_eq!(state.offset, 0);

        let cols = vec![Column::new("A", ColWidth::Flex(1))];
        let mut info = None;
        render_text(30, 5, |f| {
            info = Some(render_table(
                f.area(),
                f.buffer_mut(),
                &cols,
                &[],
                &mut state,
                &theme,
            ));
        });
        assert_eq!(info.unwrap().visible_range, 0..0);
    }

    #[test]
    fn selection_and_offset_are_always_clamped() {
        let theme = Theme::dark();
        let cols = vec![Column::new("A", ColWidth::Flex(1))];
        let rows: Vec<TableRow> = (0..12)
            .map(|i| TableRow::new(vec![format!("row-{i}")]))
            .collect();
        for height in [3, 4, 5, 8, 20] {
            for selected in [0usize, 1, 5, 11, 99] {
                for offset in [0usize, 3, 11, 99] {
                    let mut state = TableViewState { selected, offset };
                    let mut info = None;
                    render_text(30, height, |f| {
                        info = Some(render_table(
                            f.area(),
                            f.buffer_mut(),
                            &cols,
                            &rows,
                            &mut state,
                            &theme,
                        ));
                    });
                    assert!(state.selected < rows.len());
                    assert!(state.offset <= state.selected);
                    let info = info.unwrap();
                    assert!(
                        info.visible_range.contains(&state.selected),
                        "selected {} not in {:?} (h={height}, offset={offset})",
                        state.selected,
                        info.visible_range
                    );
                }
            }
        }
    }

    #[test]
    fn expanded_detail_fits_within_short_tables() {
        let theme = Theme::dark();
        let cols = vec![Column::new("A", ColWidth::Flex(1))];
        let mut rows: Vec<TableRow> = (0..6)
            .map(|i| TableRow::new(vec![format!("r{i}")]))
            .collect();
        rows[3].detail = vec!["d1".into(), "d2".into(), "d3".into()];
        for height in [3, 4, 6, 10] {
            let mut state = TableViewState {
                selected: 3,
                offset: 0,
            };
            let mut info = None;
            let text = render_text(30, height, |f| {
                info = Some(render_table(
                    f.area(),
                    f.buffer_mut(),
                    &cols,
                    &rows,
                    &mut state,
                    &theme,
                ));
            });
            assert!(info.unwrap().visible_range.contains(&3));
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn wide_and_combining_cells_are_clipped_safely() {
        let theme = Theme::dark();
        let cols = vec![
            Column::new("中中", ColWidth::Flex(1)),
            Column::new("n", ColWidth::Fixed(4)).right(),
        ];
        let rows = vec![
            TableRow::new(vec!["中中中中".into(), "1".into()]),
            TableRow::new(vec!["e\u{301}\u{301}中".into(), "2".into()]),
        ];
        let mut state = TableViewState::default();
        for (w, h) in [(8, 4), (30, 5), (200, 6)] {
            let text = render_text(w, h, |f| {
                render_table(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &theme);
            });
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn resize_reuses_selection_without_panicking() {
        let theme = Theme::dark();
        let cols = vec![Column::new("A", ColWidth::Flex(1))];
        let rows: Vec<TableRow> = (0..10)
            .map(|i| TableRow::new(vec![format!("r{i}")]))
            .collect();
        let mut state = TableViewState {
            selected: 6,
            offset: 4,
        };
        for (w, h) in [(40, 8), (10, 3), (200, 12), (4, 2), (40, 8)] {
            let mut info = None;
            render_text(w, h, |f| {
                info = Some(render_table(
                    f.area(),
                    f.buffer_mut(),
                    &cols,
                    &rows,
                    &mut state,
                    &theme,
                ));
            });
            assert!(info.unwrap().visible_range.contains(&state.selected));
        }
    }
}
