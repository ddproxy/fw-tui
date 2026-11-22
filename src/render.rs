//! Lays a [`ViewModel`] out on a ratatui frame.
//!
//! Requires the `tui` feature. All text decisions are made in the model; this module only lays
//! them out and escapes them again at the terminal boundary.
//!
//! # Examples
//!
//! ```
//! use std::time::Instant;
//!
//! use fw_tui::model::Dashboard;
//! use fw_tui::render::draw;
//! use ratatui::backend::TestBackend;
//! use ratatui::Terminal;
//!
//! let view = Dashboard::new(&["peer-a"]).view(Instant::now());
//! let mut terminal = Terminal::new(TestBackend::new(40, 6)).unwrap();
//! terminal.draw(|frame| draw(frame, &view)).unwrap();
//! ```

use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::model::ViewModel;
use crate::sanitize::printable;

/// Lays out a view model. All text decisions were made in the model.
///
/// Even a directly constructed [`ViewModel`] is escaped here, so user-supplied strings can never
/// put terminal control sequences into the output.
///
/// # Examples
///
/// ```
/// use std::time::Instant;
///
/// use fw_tui::model::Dashboard;
/// use fw_tui::render::draw;
/// use ratatui::backend::TestBackend;
/// use ratatui::Terminal;
///
/// let view = Dashboard::new(&["peer-a"]).view(Instant::now());
/// let mut terminal = Terminal::new(TestBackend::new(40, 6)).unwrap();
/// terminal.draw(|frame| draw(frame, &view)).unwrap();
/// ```
pub fn draw(frame: &mut Frame, view: &ViewModel) {
    let title = printable(&view.title);
    let status = printable(&view.status);
    let footer = printable(&view.footer);
    let [body, footer_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let mut lines: Vec<Line> = view
        .rows
        .iter()
        .map(|row| Line::from(printable(row)))
        .collect();
    lines.push(Line::default());
    let status_style = if view.paused {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(ratatui::style::Color::Yellow)
    } else {
        Style::default()
    };
    lines.push(Line::from(status).style(status_style));
    let block = Block::bordered().title(title);
    frame.render_widget(Paragraph::new(Text::from(lines)).block(block), body);
    frame.render_widget(Paragraph::new(footer), footer_area);
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::Terminal;

    use super::*;
    use crate::model::{Dashboard, ViewModel};
    use crate::test_support::{assert_no_control_chars, render_text};

    fn screen(view: &ViewModel, width: u16, height: u16) -> String {
        render_text(width, height, |frame| draw(frame, view))
    }

    fn text_of(buffer: &Buffer) -> String {
        let area = buffer.area;
        (area.y..area.y + area.height)
            .map(|y| {
                (area.x..area.x + area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn a_live_screen_draws_rows_status_and_footer() {
        let mut dashboard = Dashboard::new(&["peer-a"]);
        dashboard.tick();
        let text = screen(&dashboard.view(Instant::now()), 50, 8);
        assert!(text.contains("fw-app top"));
        assert!(text.contains("peer-a"));
        assert!(text.contains("live, update 1"));
        assert!(text.contains("q quit"));
    }

    #[test]
    fn a_paused_screen_says_how_old_its_data_is() {
        let mut dashboard = Dashboard::new(&["host-1"]);
        let t0 = Instant::now();
        dashboard.toggle_pause(t0);
        let text = screen(&dashboard.view(t0 + Duration::from_secs(7)), 60, 8);
        assert!(text.contains("paused: showing data from 7s ago"), "{text}");
    }

    #[test]
    fn a_tiny_terminal_draws_without_panicking() {
        let view = Dashboard::new(&["peer-a"]).view(Instant::now());
        for (w, h) in [(0, 0), (1, 1), (2, 2), (12, 2)] {
            let text = screen(&view, w, h);
            assert_no_control_chars(&text);
        }
    }

    #[test]
    fn narrow_and_wide_terminals_both_draw() {
        let view = Dashboard::new(&["peer-a", "host-1"]).view(Instant::now());
        let narrow = screen(&view, 4, 3);
        assert_no_control_chars(&narrow);
        let wide = screen(&view, 240, 20);
        assert_no_control_chars(&wide);
        assert!(wide.contains("peer-a"));
        assert!(wide.contains("q quit"));
    }

    #[test]
    fn a_directly_built_view_cannot_inject_terminal_sequences() {
        let view = ViewModel {
            title: "t\u{1b}[2J".into(),
            rows: vec!["r\u{9b}31m".into(), "\u{202e}bad".into()],
            status: "s\u{7}".into(),
            footer: "f\u{1b}]8;;http://example.com\u{7}".into(),
            paused: false,
        };
        let text = screen(&view, 60, 8);
        assert_no_control_chars(&text);
        assert!(!text.contains("\u{1b}"));
        assert!(!text.contains("\u{202e}"));
        assert!(text.contains("t?[2J"));
    }

    #[test]
    fn resizing_keeps_the_same_view_drawable() {
        let view = Dashboard::new(&["peer-a"]).view(Instant::now());
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        for (w, h) in [(60, 10), (3, 2), (200, 6), (0, 0), (40, 8)] {
            terminal.backend_mut().resize(w, h);
            terminal.draw(|frame| draw(frame, &view)).unwrap();
            assert_no_control_chars(&text_of(terminal.backend().buffer()));
        }
    }
}
