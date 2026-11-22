//! Input handling and the demo event loop.
//!
//! Requires the `tui` feature. Key mapping is a pure function and is tested without a terminal;
//! only [`run_demo`] needs a tty.
//!
//! # Examples
//!
//! ```
//! use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
//! use fw_tui::run::{action_for, Action};
//!
//! let key = KeyEvent {
//!     code: KeyCode::Char('q'),
//!     modifiers: KeyModifiers::NONE,
//!     kind: KeyEventKind::Press,
//!     state: KeyEventState::NONE,
//! };
//! assert_eq!(action_for(key), Some(Action::Quit));
//! ```

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::model::Dashboard;
use crate::render;
use crate::terminal::TerminalGuard;

/// Longest a key press waits before the screen is redrawn anyway.
const POLL: Duration = Duration::from_millis(100);

/// What a recognised key press asks the demo to do.
///
/// # Examples
///
/// ```
/// use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
/// use fw_tui::run::{action_for, Action};
///
/// let key = KeyEvent {
///     code: KeyCode::Esc,
///     modifiers: KeyModifiers::NONE,
///     kind: KeyEventKind::Press,
///     state: KeyEventState::NONE,
/// };
/// assert_eq!(action_for(key), Some(Action::Quit));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Leave the demo and restore the terminal.
    ///
    /// ```
    /// use fw_tui::run::Action;
    ///
    /// assert_eq!(Action::Quit, Action::Quit);
    /// ```
    Quit,
    /// Freeze or resume the displayed rows.
    ///
    /// ```
    /// use fw_tui::run::Action;
    ///
    /// assert_ne!(Action::TogglePause, Action::Quit);
    /// ```
    TogglePause,
}

/// Maps a key press to an action; unknown keys and non-press events are ignored.
///
/// # Examples
///
/// ```
/// use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
/// use fw_tui::run::{action_for, Action};
///
/// let key = KeyEvent {
///     code: KeyCode::Char('p'),
///     modifiers: KeyModifiers::NONE,
///     kind: KeyEventKind::Press,
///     state: KeyEventState::NONE,
/// };
/// assert_eq!(action_for(key), Some(Action::TogglePause));
/// ```
pub fn action_for(event: KeyEvent) -> Option<Action> {
    // Some terminals also report key releases; acting on them would double every press.
    if event.kind != KeyEventKind::Press {
        return None;
    }
    match (event.code, event.modifiers) {
        (KeyCode::Char('q') | KeyCode::Esc, _) => Some(Action::Quit),
        (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::Quit)
        }
        (KeyCode::Char('p' | ' '), _) => Some(Action::TogglePause),
        _ => None,
    }
}

/// Runs the demo dashboard until the user quits or `stop` is set. The caller sets `stop` from a
/// signal handler; the guard restores the terminal on the way out either way.
///
/// This example compiles but is not executed during doc tests because it needs a terminal.
///
/// ```no_run
/// use std::sync::atomic::AtomicBool;
/// use std::time::Duration;
///
/// let stop = AtomicBool::new(false);
/// fw_tui::run::run_demo(&stop, Duration::from_millis(250)).unwrap();
/// ```
pub fn run_demo(stop: &AtomicBool, tick: Duration) -> io::Result<()> {
    let _guard = TerminalGuard::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut dashboard = Dashboard::new(&["peer-a", "peer-b", "host-1"]);
    let mut last_tick = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        let now = Instant::now();
        if now.duration_since(last_tick) >= tick {
            dashboard.tick();
            last_tick = now;
        }
        terminal.draw(|frame| render::draw(frame, &dashboard.view(now)))?;
        if event::poll(POLL.min(tick))? {
            if let Event::Key(key) = event::read()? {
                match action_for(key) {
                    Some(Action::Quit) => break,
                    Some(Action::TogglePause) => dashboard.toggle_pause(Instant::now()),
                    None => {}
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyEventState;

    use super::*;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn the_keys_map_to_their_actions() {
        let none = KeyModifiers::NONE;
        assert_eq!(
            action_for(press(KeyCode::Char('q'), none)),
            Some(Action::Quit)
        );
        assert_eq!(action_for(press(KeyCode::Esc, none)), Some(Action::Quit));
        assert_eq!(
            action_for(press(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
        assert_eq!(
            action_for(press(KeyCode::Char('p'), none)),
            Some(Action::TogglePause)
        );
        assert_eq!(
            action_for(press(KeyCode::Char(' '), none)),
            Some(Action::TogglePause)
        );
        assert_eq!(action_for(press(KeyCode::Char('x'), none)), None);
        assert_eq!(
            action_for(press(KeyCode::Char('c'), none)),
            None,
            "plain c is not quit"
        );
    }

    #[test]
    fn key_releases_and_repeats_do_nothing() {
        for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
            let mut event = press(KeyCode::Char('q'), KeyModifiers::NONE);
            event.kind = kind;
            assert_eq!(action_for(event), None, "{kind:?} must not be handled");
        }
    }

    #[test]
    fn unrelated_keys_and_navigation_are_ignored() {
        for code in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Enter,
            KeyCode::Backspace,
            KeyCode::Char('?'),
        ] {
            assert_eq!(action_for(press(code, KeyModifiers::NONE)), None);
        }
    }

    #[test]
    fn quit_keys_work_with_extra_modifiers() {
        assert_eq!(
            action_for(press(KeyCode::Char('q'), KeyModifiers::ALT)),
            Some(Action::Quit)
        );
        assert_eq!(
            action_for(press(KeyCode::Esc, KeyModifiers::SHIFT)),
            Some(Action::Quit)
        );
        assert_eq!(
            action_for(press(KeyCode::Char('p'), KeyModifiers::CONTROL)),
            Some(Action::TogglePause)
        );
    }
}
