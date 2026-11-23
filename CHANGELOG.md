# Changelog

All notable changes to this project are documented here. Format follows Keep a Changelog.

## Unreleased

### Added

- Documentation set covering install, quickstart, configuration, CLI and keys, operations, security model, limits, troubleshooting and architecture.
- `AGENTS.md` with the stable command list, repo map, conventions and smoke test.
- `Makefile` with `help`, `build`, `test`, `lint`, `check`, `docs-check` and `install` targets.
- `scripts/check_docs.sh` to check docs files, relative links, quickstart proof blocks, config examples and code drift.
- `examples/config_minimal.rs` and `examples/config_full.rs` as validated minimal and full configuration examples.
- `llms.txt` as a machine-readable docs index.
- Rustdoc examples for every public item, including modules, enums, variants, structs, fields, methods and functions. Doc tests run with default features and with all features.
- Regression tests for zero-sized areas, very narrow and very wide terminals, resize, wide Unicode and combining characters, ANSI and control-sequence injection, empty lists, and selection and scrolling bounds.
- Sortable table columns via `TableColumn`, `SortKind` (case-insensitive natural text, numbers with
  separators/units/percentages, durations and custom comparisons) and `TableSort`.
- `render_table_sorted` draws a stable index permutation without reordering caller rows, reports
  visible row indices and shows a sort indicator; column widths are unchanged. ASCII indicators are
  used when the theme requests them.
- `TableViewState` sort helpers (`cycle_sort`, `sort_next_column`, `sort_prev_column`,
  `reverse_sort`, `handle_sort_key`) plus `sort_column_at` for header clicks.
- Optional `TableRow::key` keeps the selection on the same row across re-sorts and data refreshes;
  `sorted_indices` exposes the permutation for callers.
- Sortable table guide, gallery keys (`s`/`S`/`r`) and regression tests for ordering, stability,
  selection tracking, click mapping, indicators and 10k-row sorting.
- Rustdoc examples for every public item, including modules, enums, variants, structs, fields,
  methods and functions. Doc tests run with default features and with all features.
- Regression tests for zero-sized areas, very narrow and very wide terminals, resize, wide
  Unicode and combining characters, ANSI/control-sequence injection, empty lists, and selection
  and scrolling bounds.
- Crate-level `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` attributes.
- This changelog.

### Fixed

- Untrusted strings are now escaped at the drawing boundary for panel titles, gauge labels and right-hand text, table headers, line-chart series names, footer chips and view-model text. Previously, data built through public fields could carry an escape sequence onto the screen.
- The sanitizer now neutralises additional invisible format characters, including soft hyphen, Arabic format marks, line and paragraph separators, and Unicode tag characters.
- Model rows and gauge labels now pad by display width, so wide Unicode and combining characters no longer shift the following columns.
- Table selection and scroll offset are clamped even when the column list is empty, the row list is empty, or the draw area has zero size.
- Removed dead placeholder dataset construction from the line chart; data is now built once.

### Changed

- README and docs follow the shared template: proof first, explicit limits, security model, compatibility matrix and operating notes. No command, flag, key or default was changed.
- No breaking public API changes: names and signatures are unchanged. Display-width helpers are shared through a new private module, and the public `text` helpers keep their existing paths.

### Upgrade notes

Requires: none. No operator action is needed for this release.
- No breaking public API changes: names and signatures are unchanged. Display-width helpers are
  shared through a new private module, and the public `text` helpers keep their existing paths.
  Sorting extends the existing table types with additive fields; `TableViewState::new` and
  `..Default::default()` cover struct literals.
