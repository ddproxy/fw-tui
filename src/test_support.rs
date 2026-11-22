//! Test-only helpers for rendering widgets into a text buffer.

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use crate::text::buffer_text;

/// Draw into a `TestBackend` of the requested size and return the visible text.
pub(crate) fn render_text(
    width: u16,
    height: u16,
    draw: impl FnOnce(&mut ratatui::Frame<'_>),
) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| draw(frame)).unwrap();
    buffer_text(terminal.backend().buffer())
}

/// Fails if `text` contains a control char other than the newlines inserted between rows.
pub(crate) fn assert_no_control_chars(text: &str) {
    for ch in text.chars() {
        assert!(
            !ch.is_control() || ch == '\n',
            "a control character reached the screen: {ch:?} in {text:?}"
        );
    }
}
