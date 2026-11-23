# Sortable table columns

The table widget can sort without reordering the rows owned by the caller. Sorting is
stable and is applied to a permutation of row indices computed at render time.

## Columns and comparison

`TableColumn` carries the header, width, alignment and an optional `SortKind`. Existing `Column` values convert with `TableColumn::from`, so an unsorted
table can migrate one column at a time.

| `SortKind` | Order |
| --- | --- |
| `Text` | Case-insensitive natural order, so `item2` precedes `item10`. |
| `Number` | Parses decimals, thousands separators, `K`/`M`/`G` units and `%`; unparseable cells sort last. |
| `Duration` | Parses `s`, `m`, `h` and `d` components such as `90s` or `1h12m`; unparseable cells sort last. |
| `Custom(fn(&str, &str) -> Ordering)` | Uses the caller's comparison. |

```rust
use fw_tui::widgets::{
    render_table_sorted, ColWidth, SortKind, TableColumn, TableRow, TableSort, TableViewState,
};

let columns = vec![
    TableColumn::sortable("Name", ColWidth::Flex(1), SortKind::Text),
    TableColumn::sortable("Count", ColWidth::Fixed(8), SortKind::Number).right(),
];
let rows = vec![
    TableRow::new(vec!["peer-b".into(), "12".into()]).with_key("peer-b"),
    TableRow::new(vec!["peer-a".into(), "1,200".into()]).with_key("peer-a"),
];
let mut state = TableViewState::default().with_sort(TableSort {
    column: 1,
    descending: true,
});
```

Render with `render_table_sorted(area, buffer, &columns, &rows, &mut state, &theme)`. The original
`render_table` keeps its signature and ignores sorting metadata. Column widths and spacing are
unchanged; the sorted header gains a `▲`/`▼` marker, or `^`/`v` when the theme requests ASCII
(for example `Theme::monochrome()`).

## State and interaction

`TableViewState` holds `selected`, `offset`, the optional `sort`, and the bookkeeping used to
track a selected row by key. New state can be built with `TableViewState::new(selected, offset)`
or `Default`; struct literals can add `..Default::default()`.

- `cycle_sort(column)` steps ascending → descending → off.
- `sort_next_column(columns)` and `sort_prev_column(columns)` move between sortable columns.
- `handle_sort_key(key, columns)` maps `s`, `S` and `r` and returns whether the sort changed.
  It ignores key releases, repeats and other keys; it never touches the terminal.
- `sort_column_at(area, x, columns)` maps a header click to a sortable column index. Use
  `sort_column_at_with_status` when a status-dot column is drawn.

```rust
use crossterm::event::{Event, KeyCode};
use fw_tui::widgets::{TableColumn, TableViewState};

fn handle(event: Event, state: &mut TableViewState, columns: &[TableColumn]) -> bool {
    if let Event::Key(key) = event {
        if state.handle_sort_key(key, columns) {
            return false; // The sort changed; redraw on the next frame.
        }
        if key.code == KeyCode::Char('q') {
            return true;
        }
    }
    false
}
```

## Selection and visible rows

`selected` is an index into the caller's `rows`, never a position in the sorted view. If rows
carry a stable `key`, the selection follows that logical row across re-sorts and data refreshes
that reorder the input; without a key it follows the index. `TableRenderInfo::visible_range`
reports the smallest range of row indices covering the drawn rows: with sorting active it can be
a superset, because the permutation is not contiguous.

Call `TableViewState::sorted_indices(&columns, &rows)` when the application needs the same
permutation outside the renderer. Sorting 10,000 rows allocates keys once per cell and keeps the
input vector untouched; a regression test covers that bound.
