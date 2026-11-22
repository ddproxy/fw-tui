# Changelog

All notable changes to this project are documented here.

## Unreleased

### Added

- Rustdoc examples for every public item, including modules, enums, variants, structs, fields,
  methods and functions. Doc tests run with default features and with all features.
- Regression tests for zero-sized areas, very narrow and very wide terminals, resize, wide
  Unicode and combining characters, ANSI/control-sequence injection, empty lists, and selection
  and scrolling bounds.
- Crate-level `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` attributes.
- This changelog.

### Fixed

- Untrusted strings are now escaped at the drawing boundary for panel titles, gauge labels and
  right-hand text, table headers, line-chart series names, footer chips and view-model text.
  Previously, data built through public fields could carry an escape sequence onto the screen.
- The sanitizer now neutralises additional invisible format characters, including soft hyphen,
  Arabic format marks, line and paragraph separators, and Unicode tag characters.
- Model rows and gauge labels now pad by display width, so wide Unicode and combining characters
  no longer shift the following columns.
- Table selection and scroll offset are clamped even when the column list is empty, the row list
  is empty, or the draw area has zero size.
- Removed dead placeholder dataset construction from the line chart; data is now built once.

### Changed

- No breaking public API changes: names and signatures are unchanged. Display-width helpers are
  shared through a new private module, and the public `text` helpers keep their existing paths.
