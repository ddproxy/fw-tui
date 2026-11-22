//! Terminal dashboard widgets with a terminal-free model.
//!
//! The [`model`] and [`sanitize`] modules always build. The drawing helpers, widgets, theme and
//! event loop are available behind the `tui` feature.
//!
//! # Examples
//!
//! ```
//! use std::time::Instant;
//!
//! use fw_tui::model::Dashboard;
//!
//! let mut dashboard = Dashboard::new(&["peer-a", "host-1"]);
//! dashboard.tick();
//! let view = dashboard.view(Instant::now());
//! assert!(view.plain_text().contains("peer-a"));
//! assert!(!view.paused);
//! ```
//!
//! Untrusted text is escaped before it reaches a terminal:
//!
//! ```
//! use fw_tui::sanitize::printable;
//!
//! let safe = printable("peer\u{1b}[2J");
//! assert!(!safe.contains('\u{1b}'));
//! assert_eq!(safe, "peer?[2J");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub(crate) mod width;

pub mod model;
pub mod sanitize;

#[cfg(feature = "tui")]
pub mod render;
#[cfg(feature = "tui")]
pub mod run;
#[cfg(feature = "tui")]
pub mod terminal;
#[cfg(feature = "tui")]
pub mod text;
#[cfg(feature = "tui")]
pub mod theme;
#[cfg(feature = "tui")]
pub mod widgets;

#[cfg(all(test, feature = "tui"))]
pub(crate) mod test_support;
