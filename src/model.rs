//! State and view model for the dashboard. Nothing here touches a terminal, so the freeze-on-pause
//! behaviour and layout decisions are tested without a tty.
//!
//! # Examples
//!
//! ```
//! use std::time::Instant;
//!
//! use fw_tui::model::Dashboard;
//!
//! let mut dashboard = Dashboard::new(&["peer-a"]);
//! dashboard.tick();
//! dashboard.toggle_pause(Instant::now());
//! assert!(dashboard.paused());
//! assert!(dashboard.view(Instant::now()).paused);
//! ```

use std::time::{Duration, Instant};

use crate::sanitize::printable;

/// One named live value.
///
/// # Examples
///
/// ```
/// use fw_tui::model::Row;
///
/// let row = Row { name: "peer-a".into(), count: 7 };
/// assert_eq!(row.name, "peer-a");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Untrusted label; it is escaped when a [`ViewModel`] is built.
    ///
    /// ```
    /// use fw_tui::model::Row;
    ///
    /// let row = Row { name: "peer\u{1b}[31m".into(), count: 0 };
    /// assert!(row.name.contains('\u{1b}'));
    /// ```
    pub name: String,
    /// Current count for the row.
    ///
    /// ```
    /// use fw_tui::model::Row;
    ///
    /// let row = Row { name: "peer-a".into(), count: 12 };
    /// assert_eq!(row.count, 12);
    /// ```
    pub count: u64,
}

/// Live rows keep moving. Pausing keeps a copy of them and the moment it was taken, and the view
/// shows that copy with its age. A paused screen therefore never looks live.
///
/// # Examples
///
/// ```
/// use std::time::Instant;
///
/// use fw_tui::model::Dashboard;
///
/// let mut dashboard = Dashboard::new(&["peer-a", "host-1"]);
/// dashboard.tick();
/// let now = Instant::now();
/// dashboard.toggle_pause(now);
/// assert!(dashboard.view(now + std::time::Duration::from_secs(3)).paused);
/// ```
#[derive(Debug, Clone)]
pub struct Dashboard {
    live: Vec<Row>,
    frozen: Option<(Vec<Row>, Instant)>,
    seed: u64,
    ticks: u64,
}

impl Dashboard {
    /// Creates a dashboard whose rows start at zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let dashboard = Dashboard::new(&["peer-a"]);
    /// assert_eq!(dashboard.view(Instant::now()).rows.len(), 1);
    /// ```
    pub fn new(names: &[&str]) -> Self {
        Self {
            live: names
                .iter()
                .map(|name| Row {
                    name: (*name).to_owned(),
                    count: 0,
                })
                .collect(),
            frozen: None,
            seed: 0x9e37_79b9_7f4a_7c15,
            ticks: 0,
        }
    }

    /// Advances the live rows by a demo step. A real feed would apply its own updates here.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let mut dashboard = Dashboard::new(&["peer-a"]);
    /// dashboard.tick();
    /// assert!(dashboard.view(Instant::now()).status.contains("update 1"));
    /// ```
    pub fn tick(&mut self) {
        for row in &mut self.live {
            self.seed = self
                .seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            row.count += (self.seed >> 33) % 4;
        }
        self.ticks += 1;
    }

    /// Reports whether the dashboard currently shows a frozen copy.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let mut dashboard = Dashboard::new(&[]);
    /// assert!(!dashboard.paused());
    /// dashboard.toggle_pause(Instant::now());
    /// assert!(dashboard.paused());
    /// ```
    pub fn paused(&self) -> bool {
        self.frozen.is_some()
    }

    /// Freezes a copy of the live rows, or resumes when already paused.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let mut dashboard = Dashboard::new(&["peer-a"]);
    /// dashboard.toggle_pause(Instant::now());
    /// assert!(dashboard.paused());
    /// dashboard.toggle_pause(Instant::now());
    /// assert!(!dashboard.paused());
    /// ```
    pub fn toggle_pause(&mut self, now: Instant) {
        self.frozen = match self.frozen.take() {
            Some(_) => None,
            None => Some((self.live.clone(), now)),
        };
    }

    /// Builds the text the screen shows at `now`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let view = Dashboard::new(&["peer-a"]).view(Instant::now());
    /// assert_eq!(view.rows.len(), 1);
    /// ```
    pub fn view(&self, now: Instant) -> ViewModel {
        match &self.frozen {
            Some((rows, taken)) => ViewModel::build(
                rows,
                Some(now.saturating_duration_since(*taken)),
                self.ticks,
            ),
            None => ViewModel::build(&self.live, None, self.ticks),
        }
    }
}

/// What the screen shows, as plain text. The renderer only lays these lines out.
///
/// # Examples
///
/// ```
/// use std::time::Instant;
///
/// use fw_tui::model::{Dashboard, ViewModel};
///
/// let view: ViewModel = Dashboard::new(&["peer-a"]).view(Instant::now());
/// assert!(view.plain_text().contains("fw-app top"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewModel {
    /// Title shown in the bordered body.
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let view = Dashboard::new(&[]).view(Instant::now());
    /// assert_eq!(view.title, "fw-app top");
    /// ```
    pub title: String,
    /// Rendered, escaped row lines.
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let view = Dashboard::new(&["peer-a"]).view(Instant::now());
    /// assert_eq!(view.rows.len(), 1);
    /// ```
    pub rows: Vec<String>,
    /// Live or paused status line.
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let view = Dashboard::new(&[]).view(Instant::now());
    /// assert!(view.status.starts_with("live"));
    /// ```
    pub status: String,
    /// Key hints drawn in the footer.
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let view = Dashboard::new(&[]).view(Instant::now());
    /// assert!(view.footer.contains("q quit"));
    /// ```
    pub footer: String,
    /// Whether the rows are a frozen copy.
    ///
    /// ```
    /// use std::time::Instant;
    ///
    /// use fw_tui::model::Dashboard;
    ///
    /// let mut dashboard = Dashboard::new(&[]);
    /// dashboard.toggle_pause(Instant::now());
    /// assert!(dashboard.view(Instant::now()).paused);
    /// ```
    pub paused: bool,
}

impl ViewModel {
    fn build(rows: &[Row], paused_for: Option<Duration>, ticks: u64) -> Self {
        let rows = rows
            .iter()
            .map(|row| {
                let name = crate::width::pad_width(&printable(&row.name), 16);
                let count = crate::width::pad_start_width(&row.count.to_string(), 10);
                format!("{name} {count}")
            })
            .collect();
        let (status, paused) = match paused_for {
            Some(age) => (
                format!("paused: showing data from {}s ago", age.as_secs()),
                true,
            ),
            None => (format!("live, update {ticks}"), false),
        };
        Self {
            title: "fw-app top".to_owned(),
            rows,
            status,
            footer: "p pause/resume   q quit".to_owned(),
            paused,
        }
    }

    /// The whole screen as text, one line per row. Tests assert against this.
    ///
    /// Every field is escaped again here so a directly constructed view cannot smuggle terminal
    /// control sequences into plain-text output.
    ///
    /// # Examples
    ///
    /// ```
    /// use fw_tui::model::ViewModel;
    ///
    /// let view = ViewModel {
    ///     title: "top".into(),
    ///     rows: vec!["peer-a".into()],
    ///     status: "live".into(),
    ///     footer: "q quit".into(),
    ///     paused: false,
    /// };
    /// assert_eq!(view.plain_text(), "top\npeer-a\nlive\nq quit");
    /// ```
    pub fn plain_text(&self) -> String {
        let mut lines = vec![printable(&self.title)];
        lines.extend(self.rows.iter().map(|row| printable(row)));
        lines.push(printable(&self.status));
        lines.push(printable(&self.footer));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_paused_view_keeps_the_rows_from_the_moment_of_the_pause() {
        let mut dashboard = Dashboard::new(&["peer-a", "host-1"]);
        dashboard.tick();
        let t0 = Instant::now();
        dashboard.toggle_pause(t0);
        let frozen_text = dashboard.view(t0).plain_text();
        for _ in 0..5 {
            dashboard.tick();
        }
        let later = dashboard.view(t0 + Duration::from_secs(10));
        assert!(later.paused);
        assert_eq!(later.rows, dashboard.view(t0).rows, "the rows did not move");
        assert!(later.status.contains("10s ago"), "{}", later.status);
        assert!(frozen_text.contains("peer-a"));
    }

    #[test]
    fn resuming_shows_the_live_rows_again() {
        let mut dashboard = Dashboard::new(&["peer-a"]);
        let t0 = Instant::now();
        dashboard.toggle_pause(t0);
        dashboard.tick();
        dashboard.tick();
        let paused_rows = dashboard.view(t0).rows;
        dashboard.toggle_pause(t0);
        assert!(!dashboard.paused());
        let live = dashboard.view(t0);
        assert!(!live.paused);
        assert_ne!(live.rows, paused_rows, "the live count moved while paused");
        assert!(live.status.starts_with("live"));
    }

    #[test]
    fn names_from_the_network_are_sanitized_in_the_view() {
        let dashboard = Dashboard::new(&["peer\u{1b}[31m"]);
        let view = dashboard.view(Instant::now());
        assert!(!view.plain_text().contains('\u{1b}'));
        assert!(view.rows[0].starts_with("peer?"));
    }

    #[test]
    fn an_empty_dashboard_renders_its_chrome() {
        let view = Dashboard::new(&[]).view(Instant::now());
        assert!(view.rows.is_empty());
        assert!(view.plain_text().contains("q quit"));
    }

    #[test]
    fn wide_unicode_names_use_display_width_padding() {
        let view = Dashboard::new(&["peer-a", "中"]).view(Instant::now());
        assert_eq!(view.rows.len(), 2);
        // Both rows occupy the same number of display cells regardless of char count.
        let ascii_width = crate::width::display_width(&view.rows[0]);
        let wide_width = crate::width::display_width(&view.rows[1]);
        assert_eq!(ascii_width, wide_width);
        assert_eq!(ascii_width, 27);
    }

    #[test]
    fn a_directly_built_view_is_escaped_in_plain_text() {
        let view = ViewModel {
            title: "t\u{1b}[2J".into(),
            rows: vec!["r\u{202e}x".into()],
            status: "s\u{7}".into(),
            footer: "f".into(),
            paused: false,
        };
        let text = view.plain_text();
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{202e}'));
        assert!(!text.contains('\u{7}'));
        assert!(text.contains("t?[2J"));
    }

    #[test]
    fn huge_counts_stay_within_the_allotted_column() {
        let mut dashboard = Dashboard::new(&["peer-a"]);
        for _ in 0..64 {
            dashboard.tick();
        }
        let view = dashboard.view(Instant::now());
        assert_eq!(crate::width::display_width(&view.rows[0]), 27);
    }
}
