//! Selectable table with fixed/min/flex columns, status dots, expandable details and sortable
//! columns.
//!
//! Requires the `tui` feature.
//!
//! Sorting uses a stable permutation of row indices, so the caller's row vector is never
//! reordered. [`TableColumn`] describes which columns are sortable and how; [`TableViewState`]
//! holds the active [`TableSort`] and the selection.
//!
//! # Examples
//!
//! ```
//! use fw_tui::theme::Theme;
//! use fw_tui::widgets::{
//!     render_table, render_table_sorted, ColWidth, SortKind, TableColumn, TableRow,
//!     TableViewState,
//! };
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let cols = vec![
//!     TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text),
//!     TableColumn::sortable("Count", ColWidth::Fixed(8), SortKind::Number),
//! ];
//! let rows = vec![TableRow::new(vec!["peer-a".into(), "1,200".into()])];
//! let mut state = TableViewState::default();
//! state.cycle_sort(1);
//! let mut terminal = Terminal::new(TestBackend::new(30, 5)).unwrap();
//! terminal
//!     .draw(|f| {
//!         render_table_sorted(f.area(), f.buffer_mut(), &cols, &rows, &mut state, &Theme::dark());
//!     })
//!     .unwrap();
//! ```
//!
//! The original [`render_table`] entry point keeps working unchanged:
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

use std::cmp::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
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

/// How a sortable column compares two cell strings.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::SortKind;
///
/// assert_ne!(SortKind::Text, SortKind::Number);
/// ```
#[derive(Debug, Clone, Copy)]
pub enum SortKind {
    /// Case-insensitive natural order: `item2` sorts before `item10`.
    ///
    /// ```
    /// use fw_tui::widgets::SortKind;
    ///
    /// assert_eq!(SortKind::Text, SortKind::Text);
    /// ```
    Text,
    /// Numeric order; accepts separators, `K`/`M`/`G` units and `%`.
    ///
    /// Unparseable cells sort last in both directions.
    ///
    /// ```
    /// use fw_tui::widgets::SortKind;
    ///
    /// assert_eq!(SortKind::Number, SortKind::Number);
    /// ```
    Number,
    /// Numeric order for durations such as `90s`, `2m` or `1h12m`.
    ///
    /// Unparseable cells sort last in both directions.
    ///
    /// ```
    /// use fw_tui::widgets::SortKind;
    ///
    /// assert_eq!(SortKind::Duration, SortKind::Duration);
    /// ```
    Duration,
    /// Caller-supplied comparison of two cell strings.
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use fw_tui::widgets::SortKind;
    ///
    /// fn by_len(a: &str, b: &str) -> Ordering {
    ///     a.len().cmp(&b.len())
    /// }
    /// let kind = SortKind::Custom(by_len);
    /// assert!(matches!(kind, SortKind::Custom(_)));
    /// ```
    Custom(fn(&str, &str) -> Ordering),
}

impl PartialEq for SortKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text, Self::Text)
            | (Self::Number, Self::Number)
            | (Self::Duration, Self::Duration) => true,
            (Self::Custom(a), Self::Custom(b)) => std::ptr::fn_addr_eq(*a, *b),
            _ => false,
        }
    }
}

impl Eq for SortKind {}

/// Active sort: a column index and a direction.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::TableSort;
///
/// let sort = TableSort { column: 1, descending: true };
/// assert!(sort.descending);
/// assert_eq!(sort.column, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableSort {
    /// Zero-based index into the column slice.
    ///
    /// ```
    /// use fw_tui::widgets::TableSort;
    ///
    /// assert_eq!(TableSort::default().column, 0);
    /// ```
    pub column: usize,
    /// When true, larger values come first.
    ///
    /// ```
    /// use fw_tui::widgets::TableSort;
    ///
    /// assert!(!TableSort::default().descending);
    /// ```
    pub descending: bool,
}

/// Full description of one table column, including sort metadata.
///
/// Existing [`Column`] values convert with `TableColumn::from` (or `From<&Column>`), which keeps
/// their title, width and alignment and marks the column as unsortable.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{ColWidth, Column, SortKind, TableColumn};
///
/// let plain = TableColumn::from(&Column::new("Name", ColWidth::Flex(1)));
/// assert!(plain.sort.is_none());
/// let sortable = TableColumn::sortable("Count", ColWidth::Fixed(6), SortKind::Number);
/// assert_eq!(sortable.header, "Count");
/// assert_eq!(sortable.sort, Some(SortKind::Number));
/// ```
#[derive(Debug, Clone)]
pub struct TableColumn {
    /// Header text, escaped at construction and again when drawn.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, TableColumn};
    ///
    /// let column = TableColumn::new("N\u{1b}[31m", ColWidth::Fixed(4));
    /// assert!(!column.header.contains('\u{1b}'));
    /// ```
    pub header: String,
    /// Width rule for this column.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, TableColumn};
    ///
    /// let column = TableColumn::new("N", ColWidth::Min(4));
    /// assert_eq!(column.width, ColWidth::Min(4));
    /// ```
    pub width: ColWidth,
    /// Comparison used when this column is sorted; `None` means not sortable.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn};
    ///
    /// let column = TableColumn::sortable("N", ColWidth::Fixed(4), SortKind::Number);
    /// assert_eq!(column.sort, Some(SortKind::Number));
    /// ```
    pub sort: Option<SortKind>,
    /// Whether cell text is right-aligned.
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, TableColumn};
    ///
    /// let column = TableColumn::new("N", ColWidth::Fixed(4)).right();
    /// assert!(column.right);
    /// ```
    pub right: bool,
}

impl TableColumn {
    /// Creates an unsortable column with an escaped header.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, TableColumn};
    ///
    /// let column = TableColumn::new("Name", ColWidth::Flex(1));
    /// assert_eq!(column.header, "Name");
    /// assert_eq!(column.sort, None);
    /// ```
    pub fn new(header: impl Into<String>, width: ColWidth) -> Self {
        Self {
            header: printable(&header.into()),
            width,
            sort: None,
            right: false,
        }
    }

    /// Creates a sortable column with an escaped header.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn};
    ///
    /// let column = TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text);
    /// assert_eq!(column.sort, Some(SortKind::Text));
    /// ```
    pub fn sortable(header: impl Into<String>, width: ColWidth, sort: SortKind) -> Self {
        Self {
            sort: Some(sort),
            ..Self::new(header, width)
        }
    }

    /// Makes the column right-aligned.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn};
    ///
    /// let column = TableColumn::sortable("N", ColWidth::Fixed(4), SortKind::Number).right();
    /// assert!(column.right);
    /// ```
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

impl From<&Column> for TableColumn {
    fn from(column: &Column) -> Self {
        Self {
            header: column.title.clone(),
            width: column.width,
            sort: None,
            right: column.right,
        }
    }
}

impl From<Column> for TableColumn {
    fn from(column: Column) -> Self {
        Self::from(&column)
    }
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

    /// Converts this column into sortable [`TableColumn`] metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, Column, SortKind};
    ///
    /// let column = Column::new("Count", ColWidth::Fixed(6)).sortable(SortKind::Number);
    /// assert_eq!(column.sort, Some(SortKind::Number));
    /// ```
    pub fn sortable(&self, sort: SortKind) -> TableColumn {
        TableColumn {
            sort: Some(sort),
            ..TableColumn::from(self)
        }
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
    /// Optional stable identifier used to keep the selection on the same logical row when the
    /// data is refreshed or reordered.
    ///
    /// ```
    /// use fw_tui::widgets::TableRow;
    ///
    /// let row = TableRow::new(vec!["peer-a".into()]).with_key("peer-a");
    /// assert_eq!(row.key.as_deref(), Some("peer-a"));
    /// ```
    pub key: Option<String>,
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
            key: None,
        }
    }

    /// Sets the stable row key used to track selection across refreshes.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::TableRow;
    ///
    /// let row = TableRow::new(vec!["a".into()]).with_key("row-1");
    /// assert_eq!(row.key.as_deref(), Some("row-1"));
    /// ```
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(printable(&key.into()));
        self
    }
}

/// Mutable selection, scroll position and sort state for [`render_table`] and
/// [`render_table_sorted`].
///
/// `selected` is an index into the caller's row slice, not a position in the sorted view. The
/// optional [`TableRow::key`] lets the selection follow the same logical row when rows are
/// refreshed or reordered. All indices are clamped on every render.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::TableViewState;
///
/// let state = TableViewState::new(3, 1);
/// assert_eq!(state.selected, 3);
/// assert_eq!(state.offset, 1);
/// assert_eq!(state.sort, None);
/// ```
#[derive(Debug, Clone)]
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
    /// Position of the first visible row in the current (possibly sorted) view.
    ///
    /// ```
    /// use fw_tui::widgets::TableViewState;
    ///
    /// let state = TableViewState::new(5, 2);
    /// assert_eq!(state.offset, 2);
    /// ```
    pub offset: usize,
    /// Active sort, or `None` for the caller's row order.
    ///
    /// ```
    /// use fw_tui::widgets::{TableSort, TableViewState};
    ///
    /// let state = TableViewState::default().with_sort(TableSort::default());
    /// assert_eq!(state.sort, Some(TableSort::default()));
    /// ```
    pub sort: Option<TableSort>,
    /// Key of the row currently selected, remembered across renders.
    ///
    /// ```
    /// use fw_tui::widgets::TableViewState;
    ///
    /// assert_eq!(TableViewState::default().selected_key, None);
    /// ```
    pub selected_key: Option<String>,
    /// Selection index seen at the previous render, used to detect caller-driven moves.
    ///
    /// ```
    /// use fw_tui::widgets::TableViewState;
    ///
    /// assert_eq!(TableViewState::default().last_selected, usize::MAX);
    /// ```
    pub last_selected: usize,
}

impl Default for TableViewState {
    fn default() -> Self {
        Self {
            selected: 0,
            offset: 0,
            sort: None,
            selected_key: None,
            last_selected: usize::MAX,
        }
    }
}

impl TableViewState {
    /// Creates state for a selection and scroll offset, with no sort active.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::TableViewState;
    ///
    /// let state = TableViewState::new(2, 1);
    /// assert_eq!(state.selected, 2);
    /// assert_eq!(state.offset, 1);
    /// ```
    pub fn new(selected: usize, offset: usize) -> Self {
        Self {
            selected,
            offset,
            ..Self::default()
        }
    }

    /// Sets the active sort on this state.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{TableSort, TableViewState};
    ///
    /// let sort = TableSort { column: 1, descending: true };
    /// let state = TableViewState::new(0, 0).with_sort(sort);
    /// assert_eq!(state.sort, Some(sort));
    /// ```
    pub fn with_sort(mut self, sort: TableSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Cycles a column through ascending, descending and unsorted.
    ///
    /// Returns the new sort state. Selecting a different column starts at ascending.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{TableSort, TableViewState};
    ///
    /// let mut state = TableViewState::default();
    /// assert_eq!(state.cycle_sort(1), Some(TableSort { column: 1, descending: false }));
    /// assert_eq!(state.cycle_sort(1), Some(TableSort { column: 1, descending: true }));
    /// assert_eq!(state.cycle_sort(1), None);
    /// ```
    pub fn cycle_sort(&mut self, column: usize) -> Option<TableSort> {
        self.sort = match self.sort {
            Some(TableSort {
                column: current,
                descending: false,
            }) if current == column => Some(TableSort {
                column,
                descending: true,
            }),
            Some(TableSort {
                column: current,
                descending: true,
            }) if current == column => None,
            _ => Some(TableSort {
                column,
                descending: false,
            }),
        };
        self.sort
    }

    /// Sorts by the next sortable column, wrapping around and starting ascending.
    ///
    /// Returns `false` when no column is sortable.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn, TableViewState};
    ///
    /// let columns = [
    ///     TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text),
    ///     TableColumn::new("Note", ColWidth::Flex(1)),
    ///     TableColumn::sortable("Count", ColWidth::Fixed(6), SortKind::Number),
    /// ];
    /// let mut state = TableViewState::default();
    /// assert!(state.sort_next_column(&columns));
    /// assert_eq!(state.sort.unwrap().column, 0);
    /// assert!(state.sort_next_column(&columns));
    /// assert_eq!(state.sort.unwrap().column, 2);
    /// ```
    pub fn sort_next_column(&mut self, columns: &[TableColumn]) -> bool {
        self.move_sort_column(columns, true)
    }

    /// Sorts by the previous sortable column, wrapping around and starting ascending.
    ///
    /// Returns `false` when no column is sortable.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn, TableViewState};
    ///
    /// let columns = [
    ///     TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text),
    ///     TableColumn::new("Note", ColWidth::Flex(1)),
    ///     TableColumn::sortable("Count", ColWidth::Fixed(6), SortKind::Number),
    /// ];
    /// let mut state = TableViewState::default();
    /// assert!(state.sort_prev_column(&columns));
    /// assert_eq!(state.sort.unwrap().column, 2);
    /// ```
    pub fn sort_prev_column(&mut self, columns: &[TableColumn]) -> bool {
        self.move_sort_column(columns, false)
    }

    /// Reverses the direction of the active sort.
    ///
    /// Returns `false` when no sort is active.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{TableSort, TableViewState};
    ///
    /// let mut state = TableViewState::default().with_sort(TableSort::default());
    /// assert!(state.reverse_sort());
    /// assert!(state.sort.unwrap().descending);
    /// assert!(TableViewState::default().reverse_sort() == false);
    /// ```
    pub fn reverse_sort(&mut self) -> bool {
        match &mut self.sort {
            Some(sort) => {
                sort.descending = !sort.descending;
                true
            }
            None => false,
        }
    }

    /// Handles `s` (next column), `S` (previous column) and `r` (reverse) on key presses.
    ///
    /// Returns `true` when the key changed the sort. Releases and repeats are ignored, and the
    /// helper does not touch the terminal.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn, TableViewState};
    ///
    /// fn press(code: KeyCode) -> KeyEvent {
    ///     KeyEvent {
    ///         code,
    ///         modifiers: KeyModifiers::NONE,
    ///         kind: KeyEventKind::Press,
    ///         state: KeyEventState::NONE,
    ///     }
    /// }
    /// let columns = [TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text)];
    /// let mut state = TableViewState::default();
    /// assert!(state.handle_sort_key(press(KeyCode::Char('s')), &columns));
    /// assert!(state.sort.unwrap().column == 0);
    /// assert!(state.handle_sort_key(press(KeyCode::Char('r')), &columns));
    /// assert!(state.sort.unwrap().descending);
    /// ```
    pub fn handle_sort_key(&mut self, key: KeyEvent, columns: &[TableColumn]) -> bool {
        if key.kind != KeyEventKind::Press
            || key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return false;
        }
        let shifted = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Char('s') if !shifted => self.sort_next_column(columns),
            KeyCode::Char('S') => self.sort_prev_column(columns),
            KeyCode::Char('s') if shifted => self.sort_prev_column(columns),
            KeyCode::Char('r' | 'R') => self.reverse_sort(),
            _ => false,
        }
    }

    /// Returns the row indices in the order they will be drawn.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::widgets::{ColWidth, SortKind, TableColumn, TableRow, TableViewState};
    ///
    /// let columns = [TableColumn::sortable("N", ColWidth::Fixed(4), SortKind::Number)];
    /// let rows = [TableRow::new(vec!["10".into()]), TableRow::new(vec!["2".into()])];
    /// let state = TableViewState::new(0, 0).with_sort(
    ///     fw_tui::widgets::TableSort::default(),
    /// );
    /// assert_eq!(state.sorted_indices(&columns, &rows), vec![1, 0]);
    /// ```
    pub fn sorted_indices(&self, columns: &[TableColumn], rows: &[TableRow]) -> Vec<usize> {
        sort_permutation(columns, rows, self)
    }

    fn move_sort_column(&mut self, columns: &[TableColumn], forward: bool) -> bool {
        let sortables: Vec<usize> = columns
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.sort.map(|_| i))
            .collect();
        if sortables.is_empty() {
            return false;
        }
        let next = match self
            .sort
            .and_then(|s| sortables.iter().position(|&c| c == s.column))
        {
            Some(pos) if forward => sortables[(pos + 1) % sortables.len()],
            Some(pos) => sortables[(pos + sortables.len() - 1) % sortables.len()],
            None if forward => sortables[0],
            None => sortables[sortables.len() - 1],
        };
        self.sort = Some(TableSort {
            column: next,
            descending: false,
        });
        true
    }
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

/// Stable row-index permutation for the active sort.
///
/// The caller's rows are never reordered: the returned vector holds indices into `rows` in drawing
/// order. Without an active sort, or when the active column is not sortable, the result is
/// `0..rows.len()`.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{
///     sort_permutation, ColWidth, SortKind, TableColumn, TableRow, TableSort, TableViewState,
/// };
///
/// let columns = [TableColumn::sortable("Count", ColWidth::Fixed(4), SortKind::Number)];
/// let rows = [
///     TableRow::new(vec!["10".into()]),
///     TableRow::new(vec!["2".into()]),
///     TableRow::new(vec!["n/a".into()]),
/// ];
/// let state = TableViewState::default().with_sort(TableSort::default());
/// assert_eq!(sort_permutation(&columns, &rows, &state), vec![1, 0, 2]);
/// assert_eq!(rows[0].cells[0], "10");
/// ```
pub fn sort_permutation(
    columns: &[TableColumn],
    rows: &[TableRow],
    state: &TableViewState,
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    let Some(sort) = state.sort else {
        return order;
    };
    let Some(Some(kind)) = columns.get(sort.column).map(|c| c.sort) else {
        return order;
    };

    let mut cells = Vec::with_capacity(rows.len());
    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        let cell = row
            .cells
            .get(sort.column)
            .map(|value| printable(value))
            .unwrap_or_default();
        keys.push(match kind {
            SortKind::Text => SortKey::Text(text_key(&cell)),
            SortKind::Number => SortKey::Number(parse_number(&cell)),
            SortKind::Duration => SortKey::Duration(parse_duration(&cell)),
            SortKind::Custom(_) => SortKey::Custom,
        });
        cells.push(cell);
    }

    // The explicit index tie-break keeps equal cells in input order in both directions.
    order.sort_by(|&a, &b| {
        compare_keys(
            kind,
            &keys[a],
            &keys[b],
            &cells[a],
            &cells[b],
            sort.descending,
        )
        .then_with(|| a.cmp(&b))
    });
    order
}

#[derive(Debug, Clone)]
enum TextChunk {
    Text(String),
    Digits(String),
}

#[derive(Debug, Clone)]
enum SortKey {
    Text(Vec<TextChunk>),
    Number(Option<f64>),
    Duration(Option<f64>),
    Custom,
}

fn text_key(value: &str) -> Vec<TextChunk> {
    fn push(chunks: &mut Vec<TextChunk>, buffer: &mut String, numeric: bool) {
        let chunk = if numeric {
            TextChunk::Digits(std::mem::take(buffer))
        } else {
            TextChunk::Text(std::mem::take(buffer))
        };
        chunks.push(chunk);
    }

    let mut chunks = Vec::new();
    let mut buffer = String::new();
    let mut numeric = false;
    let mut started = false;
    for ch in value.trim().chars() {
        let is_digit = ch.is_ascii_digit();
        if started && is_digit != numeric {
            push(&mut chunks, &mut buffer, numeric);
        }
        if is_digit {
            buffer.push(ch);
        } else {
            buffer.extend(ch.to_lowercase());
        }
        numeric = is_digit;
        started = true;
    }
    if started {
        push(&mut chunks, &mut buffer, numeric);
    }
    chunks
}

fn compare_chunks(a: &[TextChunk], b: &[TextChunk]) -> Ordering {
    for (left, right) in a.iter().zip(b.iter()) {
        let comparison = match (left, right) {
            (TextChunk::Text(x), TextChunk::Text(y)) => x.cmp(y),
            (TextChunk::Digits(x), TextChunk::Digits(y)) => {
                let x = x.trim_start_matches('0');
                let y = y.trim_start_matches('0');
                let x = if x.is_empty() { "0" } else { x };
                let y = if y.is_empty() { "0" } else { y };
                x.len().cmp(&y.len()).then_with(|| x.cmp(y))
            }
            (TextChunk::Text(_), TextChunk::Digits(_)) => Ordering::Less,
            (TextChunk::Digits(_), TextChunk::Text(_)) => Ordering::Greater,
        };
        if comparison != Ordering::Equal {
            return comparison;
        }
    }
    a.len().cmp(&b.len())
}

fn compare_parsed(a: Option<f64>, b: Option<f64>, descending: bool) -> Ordering {
    match (a, b) {
        // Unparseable cells always go last, including in descending order.
        (Some(x), Some(y)) => {
            let comparison = x.partial_cmp(&y).unwrap_or(Ordering::Equal);
            if descending {
                comparison.reverse()
            } else {
                comparison
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_keys(
    kind: SortKind,
    a: &SortKey,
    b: &SortKey,
    a_cell: &str,
    b_cell: &str,
    descending: bool,
) -> Ordering {
    let directed = |comparison: Ordering| {
        if descending {
            comparison.reverse()
        } else {
            comparison
        }
    };
    match (kind, a, b) {
        (SortKind::Custom(compare), _, _) => directed(compare(a_cell, b_cell)),
        (SortKind::Text, SortKey::Text(x), SortKey::Text(y)) => directed(compare_chunks(x, y)),
        (SortKind::Number, SortKey::Number(x), SortKey::Number(y)) => {
            compare_parsed(*x, *y, descending)
        }
        (SortKind::Duration, SortKey::Duration(x), SortKey::Duration(y)) => {
            compare_parsed(*x, *y, descending)
        }
        _ => Ordering::Equal,
    }
}

fn parse_number(value: &str) -> Option<f64> {
    let mut text = value.trim().replace(',', "");
    let suffix = text.chars().last()?.to_ascii_lowercase();
    let factor = match suffix {
        'k' => 1_000.0,
        'm' => 1_000_000.0,
        'g' => 1_000_000_000.0,
        '%' => 1.0,
        _ => 1.0,
    };
    if matches!(suffix, 'k' | 'm' | 'g' | '%') {
        text.pop();
    }
    let parsed: f64 = text.trim().parse().ok()?;
    if parsed.is_finite() {
        Some(parsed * factor)
    } else {
        None
    }
}

fn parse_duration(value: &str) -> Option<f64> {
    let text = value.trim();
    if text.is_empty() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut total = 0.0;
    let mut found = false;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
            i += 1;
        }
        if start == i {
            return None;
        }
        let amount: f64 = text[start..i].parse().ok()?;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let unit = *bytes.get(i)?;
        i += 1;
        let seconds = match unit.to_ascii_lowercase() {
            b's' => 1.0,
            b'm' => 60.0,
            b'h' => 3_600.0,
            b'd' => 86_400.0,
            _ => return None,
        };
        total += amount * seconds;
        found = true;
    }
    if found && total.is_finite() {
        Some(total)
    } else {
        None
    }
}

/// Maps a header-cell click to a sortable column index.
///
/// `x` is a screen coordinate. Column widths and the one-cell spacing match the table renderer;
/// clicks in a gap, outside the area, or on an unsortable column return `None`.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{sort_column_at, ColWidth, SortKind, TableColumn};
/// use ratatui::layout::Rect;
///
/// let columns = [
///     TableColumn::sortable("Name", ColWidth::Fixed(6), SortKind::Text),
///     TableColumn::sortable("Count", ColWidth::Fixed(6), SortKind::Number),
/// ];
/// let area = Rect::new(4, 0, 13, 4);
/// assert_eq!(sort_column_at(area, 6, &columns), Some(0));
/// assert_eq!(sort_column_at(area, 12, &columns), Some(1));
/// assert_eq!(sort_column_at(area, 10, &columns), None); // gap between columns
/// ```
pub fn sort_column_at(area: Rect, x: u16, columns: &[TableColumn]) -> Option<usize> {
    sort_column_at_with_status(area, x, columns, false)
}

/// Like [`sort_column_at`], accounting for the status-dot column when one is drawn.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::{sort_column_at_with_status, ColWidth, SortKind, TableColumn};
/// use ratatui::layout::Rect;
///
/// let columns = [TableColumn::sortable("Name", ColWidth::Fixed(6), SortKind::Text)];
/// let area = Rect::new(0, 0, 12, 3);
/// assert_eq!(sort_column_at_with_status(area, 0, &columns, true), None);
/// assert_eq!(sort_column_at_with_status(area, 4, &columns, true), Some(0));
/// ```
pub fn sort_column_at_with_status(
    area: Rect,
    x: u16,
    columns: &[TableColumn],
    has_status: bool,
) -> Option<usize> {
    if area.width == 0 || columns.is_empty() {
        return None;
    }
    let x = x.checked_sub(area.x)?;
    if x >= area.width {
        return None;
    }
    let mut constraints = Vec::with_capacity(columns.len() + usize::from(has_status));
    if has_status {
        constraints.push(Constraint::Length(2));
    }
    constraints.extend(columns.iter().map(|c| constraint(c.width)));
    let rects = Layout::horizontal(constraints)
        .spacing(1)
        .split(Rect::new(0, 0, area.width, 1));
    let skip = usize::from(has_status);
    columns.iter().enumerate().find_map(|(index, column)| {
        column.sort?;
        rects.get(index + skip).and_then(|rect| {
            (x >= rect.x && x < rect.x.saturating_add(rect.width)).then_some(index)
        })
    })
}

fn constraint(width: ColWidth) -> Constraint {
    match width {
        ColWidth::Fixed(w) => Constraint::Length(w),
        ColWidth::Min(w) => Constraint::Min(w),
        ColWidth::Flex(w) => Constraint::Fill(w.max(1)),
    }
}

fn sort_indicator(state: &TableViewState, column: usize, theme: &Theme) -> &'static str {
    match state.sort {
        Some(TableSort {
            column: active,
            descending,
            ..
        }) if active == column => {
            if theme.prefers_ascii() {
                if descending {
                    "v"
                } else {
                    "^"
                }
            } else if descending {
                "▼"
            } else {
                "▲"
            }
        }
        _ => "",
    }
}

fn sync_selection(state: &mut TableViewState, rows: &[TableRow]) {
    if rows.is_empty() {
        state.selected = 0;
        state.offset = 0;
        state.selected_key = None;
        state.last_selected = usize::MAX;
        return;
    }
    if state.selected != state.last_selected {
        // The caller moved the selection directly: adopt the row now under it.
        state.selected = state.selected.min(rows.len() - 1);
        state.selected_key = rows[state.selected].key.clone();
    } else if let Some(key) = state.selected_key.clone() {
        // A refresh may have reordered the rows; find the same logical row again.
        match rows
            .iter()
            .position(|row| row.key.as_deref() == Some(key.as_str()))
        {
            Some(index) => state.selected = index,
            None => {
                state.selected = state.selected.min(rows.len() - 1);
                state.selected_key = rows[state.selected].key.clone();
            }
        }
    } else {
        state.selected = state.selected.min(rows.len() - 1);
    }
    state.last_selected = state.selected;
}

/// Render a table with header, selection, optional status dot column and expandable selected row.
/// Returns visible row range (data indices).
///
/// Empty column lists, empty row lists and zero-sized areas are safe; in those cases the returned
/// range is empty and the selection/scroll state is still clamped. Header text is escaped even
/// when [`Column`] was built directly. This entry point keeps the caller's row order; use
/// [`render_table_sorted`] to enable sorting.
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
    let columns: Vec<TableColumn> = columns.iter().map(TableColumn::from).collect();
    render_table_sorted(area, buf, &columns, rows, state, theme)
}

/// Renders a table with sortable columns and a header sort indicator.
///
/// The active [`TableSort`] produces a stable index permutation at render time; `rows` is never
/// reordered and `state.selected` stays an index into `rows`. With sorting active,
/// [`TableRenderInfo::visible_range`] is the smallest range of row indices covering the drawn rows,
/// so it may include rows hidden by the permutation.
///
/// # Examples
///
/// ```
/// use fw_tui::theme::Theme;
/// use fw_tui::widgets::{
///     render_table_sorted, ColWidth, SortKind, TableColumn, TableRow, TableSort, TableViewState,
/// };
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let columns = [
///     TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text),
///     TableColumn::sortable("Count", ColWidth::Fixed(8), SortKind::Number).right(),
/// ];
/// let rows = [
///     TableRow::new(vec!["peer-b".into(), "12".into()]),
///     TableRow::new(vec!["peer-a".into(), "1,200".into()]),
/// ];
/// let mut state = TableViewState::default().with_sort(TableSort {
///     column: 1,
///     descending: true,
/// });
/// let mut terminal = Terminal::new(TestBackend::new(30, 5)).unwrap();
/// terminal
///     .draw(|f| {
///         render_table_sorted(f.area(), f.buffer_mut(), &columns, &rows, &mut state, &Theme::dark());
///     })
///     .unwrap();
/// assert_eq!(state.selected, 0);
/// ```
pub fn render_table_sorted(
    area: Rect,
    buf: &mut Buffer,
    columns: &[TableColumn],
    rows: &[TableRow],
    state: &mut TableViewState,
    theme: &Theme,
) -> TableRenderInfo {
    sync_selection(state, rows);
    if area.width == 0 || area.height == 0 || columns.is_empty() || rows.is_empty() {
        state.offset = state.offset.min(state.selected);
        return TableRenderInfo {
            visible_range: 0..0,
        };
    }

    let order = sort_permutation(columns, rows, state);
    let mut positions = vec![0usize; rows.len()];
    for (position, &index) in order.iter().enumerate() {
        positions[index] = position;
    }
    let selected = state.selected;
    let display_selected = positions.get(selected).copied().unwrap_or(0);
    state.offset = state.offset.min(display_selected);

    let has_status = rows.iter().any(|r| r.status.is_some());
    let mut cols: Vec<TableColumn> = Vec::with_capacity(columns.len() + usize::from(has_status));
    if has_status {
        cols.push(TableColumn::new("", ColWidth::Fixed(2)));
    }
    cols.extend(columns.iter().cloned());
    let widths: Vec<Constraint> = cols.iter().map(|c| constraint(c.width)).collect();

    let inner_h = area.height.saturating_sub(2) as usize; // header inside the table area
    let detail_len = |position: usize| rows[order[position]].detail.len();

    // Compute a window that leaves room for the selected row's expanded detail.
    let mut offset = state.offset;
    loop {
        let (start, end) = window_range(
            order.len(),
            offset,
            display_selected,
            inner_h.max(1),
            &detail_len,
        );
        if display_selected < start {
            offset = display_selected;
            continue;
        }
        if display_selected >= end && end < order.len() {
            offset = offset.saturating_add(1);
            continue;
        }
        state.offset = offset;
        break;
    }
    let (start, end) = window_range(
        order.len(),
        state.offset,
        display_selected,
        inner_h.max(1),
        &detail_len,
    );

    // Build the header row (reversed bar style). Direct construction of TableColumn is public, so
    // escape here too rather than trusting the constructor alone.
    let skip = usize::from(has_status);
    let header_cells = cols.iter().enumerate().map(|(ci, col)| {
        let marker = if ci >= skip {
            sort_indicator(state, ci - skip, theme)
        } else {
            ""
        };
        let suffix = if marker.is_empty() {
            String::new()
        } else {
            format!(" {marker}")
        };
        let title = truncate_width(
            &printable(&col.header),
            20usize.saturating_sub(suffix.chars().count()),
        );
        Cell::from(Line::from(Span::styled(
            format!("{title}{suffix}"),
            theme.title,
        )))
    });
    let header = Row::new(header_cells).style(theme.header_bar).height(1);

    let mut table_rows: Vec<Row> = Vec::new();
    let mut detail_rows: Vec<(usize, Vec<String>)> = Vec::new();
    for (position, &index) in order.iter().enumerate().take(end).skip(start) {
        let row = &rows[index];
        let mut cells: Vec<Cell> = Vec::new();
        let mut values: Vec<String> = Vec::new();
        if has_status {
            values.push(String::new());
        }
        for cell in &row.cells {
            values.push(printable(cell));
        }
        if has_status {
            if let Some(st) = &row.status {
                cells.push(Cell::from(Line::from(st.clone())));
            } else {
                cells.push(Cell::from(" "));
            }
        }
        for (ci, col) in columns.iter().enumerate() {
            let value = values
                .get(ci + usize::from(has_status))
                .cloned()
                .unwrap_or_default();
            let text = truncate_width(&value, 40);
            let styled = if col.right {
                Line::from(text).right_aligned()
            } else {
                Line::from(text)
            };
            cells.push(Cell::from(styled));
        }
        let style = if position == display_selected {
            theme.selected_row
        } else {
            Style::default()
        };
        table_rows.push(Row::new(cells).style(style).height(1));
        if position == display_selected && !row.detail.is_empty() {
            detail_rows.push((index, row.detail.clone()));
        }
    }

    let table = RTable::new(table_rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .column_spacing(1);
    table.render(area, buf);

    // Draw expansion lines under the selected row inside the table area (best effort).
    if let Some((_, details)) = detail_rows.first() {
        let rel = display_selected.saturating_sub(start);
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

    let mut first = usize::MAX;
    let mut last = 0usize;
    for &index in &order[start..end] {
        first = first.min(index);
        last = last.max(index);
    }

    TableRenderInfo {
        visible_range: if start == end {
            0..0
        } else {
            first..last.saturating_add(1)
        },
    }
}

fn window_range(
    len: usize,
    offset: usize,
    selected: usize,
    capacity: usize,
    detail_len: &dyn Fn(usize) -> usize,
) -> (usize, usize) {
    if len == 0 || capacity == 0 {
        return (0, 0);
    }
    let mut start = offset.min(len - 1);
    let mut used = 0usize;
    let mut end = start;
    while end < len && used < capacity {
        used += 1; // row
        if end == selected {
            let extra = detail_len(end).min(capacity.saturating_sub(used));
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
    if selected >= end && end < len {
        // Shift start forward until the selected row fits.
        while start < selected {
            start += 1;
            let mut u = 0;
            let mut e = start;
            while e < len && u < capacity {
                u += 1;
                if e == selected {
                    u += detail_len(e).min(capacity.saturating_sub(u));
                }
                if u > capacity {
                    break;
                }
                e += 1;
            }
            if selected < e || e == len {
                end = e;
                break;
            }
            end = e;
        }
    }
    (start, end.min(len))
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
        let mut state = TableViewState::new(2, 0);
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
            key: None,
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
        let mut state = TableViewState::new(7, 5);
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
                    let mut state = TableViewState::new(selected, offset);
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
            let mut state = TableViewState::new(3, 0);
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
        let mut state = TableViewState::new(6, 4);
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

    use std::cmp::Ordering;
    use std::time::{Duration, Instant};

    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn one_column(kind: SortKind) -> Vec<TableColumn> {
        vec![TableColumn::sortable("Value", ColWidth::Fixed(12), kind)]
    }

    fn value_rows(values: &[&str]) -> Vec<TableRow> {
        values
            .iter()
            .map(|value| TableRow::new(vec![(*value).into()]))
            .collect()
    }

    fn sorted_values(
        columns: &[TableColumn],
        rows: &[TableRow],
        state: &TableViewState,
    ) -> Vec<String> {
        sort_permutation(columns, rows, state)
            .into_iter()
            .map(|index| rows[index].cells[0].clone())
            .collect()
    }

    #[test]
    fn text_uses_case_insensitive_natural_order() {
        let columns = one_column(SortKind::Text);
        let rows = value_rows(&["item10", "Item 2", "item1", "ITEM 02", "z", "A"]);
        let ascending = TableViewState::default().with_sort(TableSort::default());
        assert_eq!(
            sorted_values(&columns, &rows, &ascending),
            ["A", "item1", "item10", "Item 2", "ITEM 02", "z"]
        );
        let descending = TableViewState::default().with_sort(TableSort {
            column: 0,
            descending: true,
        });
        assert_eq!(
            sorted_values(&columns, &rows, &descending),
            ["z", "Item 2", "ITEM 02", "item10", "item1", "A"]
        );
    }

    #[test]
    fn numbers_accept_separators_units_and_percentages() {
        let columns = one_column(SortKind::Number);
        let rows = value_rows(&["1,200", "3K", "12%", "bad", "2M", "-5", "1.5k", "0"]);
        let ascending = TableViewState::default().with_sort(TableSort::default());
        assert_eq!(
            sorted_values(&columns, &rows, &ascending),
            ["-5", "0", "12%", "1,200", "1.5k", "3K", "2M", "bad"]
        );
        let descending = TableViewState::default().with_sort(TableSort {
            column: 0,
            descending: true,
        });
        assert_eq!(
            sorted_values(&columns, &rows, &descending),
            ["2M", "3K", "1.5k", "1,200", "12%", "0", "-5", "bad"]
        );
    }

    #[test]
    fn durations_compare_in_seconds() {
        let columns = one_column(SortKind::Duration);
        let rows = value_rows(&["1h12m", "2m", "90s", "1h", "bad", "4m00s"]);
        let state = TableViewState::default().with_sort(TableSort::default());
        assert_eq!(
            sorted_values(&columns, &rows, &state),
            ["90s", "2m", "4m00s", "1h", "1h12m", "bad"]
        );
    }

    #[test]
    fn custom_sort_uses_the_callback_and_stays_stable() {
        fn by_len(a: &str, b: &str) -> Ordering {
            a.len().cmp(&b.len())
        }
        let columns = one_column(SortKind::Custom(by_len));
        let rows = value_rows(&["bbb", "a", "cc", "dd"]);
        let ascending = TableViewState::default().with_sort(TableSort::default());
        assert_eq!(
            sorted_values(&columns, &rows, &ascending),
            ["a", "cc", "dd", "bbb"]
        );
        let descending = TableViewState::default().with_sort(TableSort {
            column: 0,
            descending: true,
        });
        assert_eq!(
            sorted_values(&columns, &rows, &descending),
            ["bbb", "cc", "dd", "a"]
        );
    }

    #[test]
    fn equal_values_keep_input_order_and_rows_are_not_mutated() {
        let columns = one_column(SortKind::Number);
        let rows = value_rows(&["1", "2", "1", "2", "1"]);
        let before: Vec<String> = rows.iter().map(|row| row.cells[0].clone()).collect();
        let mut state = TableViewState::default().with_sort(TableSort::default());
        assert_eq!(
            sort_permutation(&columns, &rows, &state),
            vec![0, 2, 4, 1, 3]
        );
        let theme = Theme::dark();
        render_text(20, 6, |f| {
            render_table_sorted(
                f.area(),
                f.buffer_mut(),
                &columns,
                &rows,
                &mut state,
                &theme,
            );
        });
        let after: Vec<String> = rows.iter().map(|row| row.cells[0].clone()).collect();
        assert_eq!(after, before);
    }

    #[test]
    fn selection_follows_its_key_across_resorts_and_refreshes() {
        let columns = vec![TableColumn::sortable(
            "Name",
            ColWidth::Fixed(8),
            SortKind::Text,
        )];
        let rows = vec![
            TableRow::new(vec!["b".into()]).with_key("b"),
            TableRow::new(vec!["a".into()]).with_key("a"),
            TableRow::new(vec!["c".into()]).with_key("c"),
        ];
        let theme = Theme::dark();
        let mut state = TableViewState::new(1, 0).with_sort(TableSort::default());
        let text = render_text(20, 8, |f| {
            render_table_sorted(
                f.area(),
                f.buffer_mut(),
                &columns,
                &rows,
                &mut state,
                &theme,
            );
        });
        assert_eq!(state.selected, 1);
        assert!(text.lines().nth(1).unwrap().contains('a'));

        state.cycle_sort(0);
        state.offset = 0;
        let text = render_text(20, 8, |f| {
            render_table_sorted(
                f.area(),
                f.buffer_mut(),
                &columns,
                &rows,
                &mut state,
                &theme,
            );
        });
        assert_eq!(state.selected, 1);
        assert!(text.lines().nth(3).unwrap().contains('a'));

        let refreshed = vec![rows[2].clone(), rows[1].clone(), rows[0].clone()];
        state.offset = 0;
        let text = render_text(20, 8, |f| {
            render_table_sorted(
                f.area(),
                f.buffer_mut(),
                &columns,
                &refreshed,
                &mut state,
                &theme,
            );
        });
        assert_eq!(state.selected, 1);
        assert!(text.lines().nth(3).unwrap().contains('a'));
    }

    #[test]
    fn header_clicks_map_to_sortable_columns() {
        let columns = [
            TableColumn::sortable("A", ColWidth::Fixed(6), SortKind::Text),
            TableColumn::new("B", ColWidth::Fixed(6)),
            TableColumn::sortable("C", ColWidth::Fixed(6), SortKind::Number),
        ];
        let area = Rect::new(5, 0, 20, 4);
        assert_eq!(sort_column_at(area, 5, &columns), Some(0));
        assert_eq!(sort_column_at(area, 11, &columns), None); // gap
        assert_eq!(sort_column_at(area, 12, &columns), None); // unsortable column
        assert_eq!(sort_column_at(area, 19, &columns), Some(2));
        assert_eq!(sort_column_at(area, 4, &columns), None);
        assert_eq!(sort_column_at(area, 25, &columns), None);
        assert_eq!(sort_column_at(Rect::new(0, 0, 0, 4), 0, &columns), None);
        assert_eq!(
            sort_column_at_with_status(Rect::new(0, 0, 25, 4), 0, &columns, true),
            None
        );
        assert_eq!(
            sort_column_at_with_status(Rect::new(0, 0, 25, 4), 4, &columns, true),
            Some(0)
        );
    }

    #[test]
    fn sort_keys_are_handled_without_a_terminal() {
        let columns = [
            TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text),
            TableColumn::sortable("Count", ColWidth::Fixed(6), SortKind::Number),
        ];
        let mut state = TableViewState::default();
        assert!(!state.handle_sort_key(press(KeyCode::Char('s'), KeyModifiers::NONE), &[]));
        assert!(state.handle_sort_key(press(KeyCode::Char('s'), KeyModifiers::NONE), &columns));
        assert_eq!(state.sort.unwrap().column, 0);
        assert!(state.handle_sort_key(press(KeyCode::Char('s'), KeyModifiers::NONE), &columns));
        assert_eq!(state.sort.unwrap().column, 1);
        assert!(state.handle_sort_key(press(KeyCode::Char('S'), KeyModifiers::NONE), &columns));
        assert_eq!(state.sort.unwrap().column, 0);
        assert!(state.handle_sort_key(press(KeyCode::Char('r'), KeyModifiers::NONE), &columns));
        assert!(state.sort.unwrap().descending);
        assert!(state.handle_sort_key(press(KeyCode::Char('R'), KeyModifiers::SHIFT), &columns));
        assert!(!state.sort.unwrap().descending);

        let mut released = press(KeyCode::Char('s'), KeyModifiers::NONE);
        released.kind = KeyEventKind::Release;
        assert!(!state.handle_sort_key(released, &columns));
        assert!(!state.handle_sort_key(press(KeyCode::Char('x'), KeyModifiers::NONE), &columns));
        assert!(!state.handle_sort_key(press(KeyCode::Char('s'), KeyModifiers::CONTROL), &columns));
    }

    #[test]
    fn sort_indicator_renders_without_changing_widths() {
        let columns = [
            TableColumn::sortable("Name", ColWidth::Fixed(10), SortKind::Text),
            TableColumn::sortable("Count", ColWidth::Fixed(8), SortKind::Number),
        ];
        let rows = value_rows(&["x"]);
        let dark = Theme::dark();
        let mut state = TableViewState::default();
        let plain = render_text(30, 5, |f| {
            render_table_sorted(f.area(), f.buffer_mut(), &columns, &rows, &mut state, &dark);
        });
        assert!(plain.contains("Name"));
        assert!(!plain.contains('▲'));

        state.cycle_sort(0);
        let ascending = render_text(30, 5, |f| {
            render_table_sorted(f.area(), f.buffer_mut(), &columns, &rows, &mut state, &dark);
        });
        assert!(ascending.contains("Name ▲"));
        state.cycle_sort(0);
        let descending = render_text(30, 5, |f| {
            render_table_sorted(f.area(), f.buffer_mut(), &columns, &rows, &mut state, &dark);
        });
        assert!(descending.contains("Name ▼"));

        let mono = Theme::monochrome();
        let ascii = render_text(30, 5, |f| {
            render_table_sorted(f.area(), f.buffer_mut(), &columns, &rows, &mut state, &mono);
        });
        assert!(ascii.contains("Name v"));

        let column_start = |text: &str| {
            let line = text.lines().next().unwrap();
            line.find("Count").map(|byte| line[..byte].chars().count())
        };
        assert_eq!(column_start(&plain), column_start(&ascending));
        assert_eq!(column_start(&plain), column_start(&descending));
        assert_eq!(column_start(&plain), column_start(&ascii));
    }

    #[test]
    fn ten_thousand_rows_sort_within_a_generous_bound() {
        let columns = vec![
            TableColumn::sortable("N", ColWidth::Fixed(8), SortKind::Number),
            TableColumn::sortable("T", ColWidth::Fixed(8), SortKind::Text),
        ];
        let rows: Vec<TableRow> = (0..10_000)
            .map(|i| {
                TableRow::new(vec![
                    format!("{}", (i * 37) % 10_007),
                    format!("item-{}", (i * 7_919) % 10_007),
                ])
            })
            .collect();
        let original: Vec<String> = rows.iter().map(|row| row.cells[0].clone()).collect();
        let mut state = TableViewState::default().with_sort(TableSort {
            column: 0,
            descending: false,
        });
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
        let started = Instant::now();
        terminal
            .draw(|f| {
                render_table_sorted(
                    f.area(),
                    f.buffer_mut(),
                    &columns,
                    &rows,
                    &mut state,
                    &Theme::dark(),
                );
            })
            .unwrap();
        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(
            rows.iter()
                .map(|row| row.cells[0].clone())
                .collect::<Vec<_>>(),
            original
        );
        assert_eq!(state.sorted_indices(&columns, &rows).len(), rows.len());
    }
}
