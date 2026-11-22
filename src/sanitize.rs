//! Escape untrusted text before it is drawn.
//!
//! # Examples
//!
//! ```
//! use fw_tui::sanitize::printable;
//!
//! assert_eq!(printable("peer\x1b[31m"), "peer?[31m");
//! ```

/// Makes a string from the network safe to draw: control characters, bidi overrides and
/// invisible format characters (zero-width marks, the byte-order mark, line/paragraph separators,
/// variation selectors and tag characters) become `?`. A peer name therefore cannot move the
/// cursor, recolour the screen, or reorder or hide the text around it.
///
/// The transformation is idempotent: escaping an already-safe string leaves ordinary characters,
/// including `?`, unchanged.
///
/// # Examples
///
/// ```
/// use fw_tui::sanitize::printable;
///
/// let escaped = printable("peer\u{202e}\u{1b}[2J");
/// assert!(!escaped.contains('\u{1b}'));
/// assert!(!escaped.contains('\u{202e}'));
/// assert_eq!(escaped, "peer??[2J");
/// ```
pub fn printable(text: &str) -> String {
    text.chars()
        .map(|c| {
            let invisible = matches!(
                c,
                '\u{00ad}'
                    | '\u{0600}'..='\u{0605}'
                    | '\u{061c}'
                    | '\u{200b}'..='\u{200f}'
                    | '\u{2028}'..='\u{202e}'
                    | '\u{2060}'..='\u{206f}'
                    | '\u{feff}'
                    | '\u{fff9}'..='\u{fffb}'
                    | '\u{e0001}'
                    | '\u{e0020}'..='\u{e007f}'
            );
            if c.is_control() || invisible {
                '?'
            } else {
                c
            }
        })
        .collect()
}

/// Replaces every control or invisible char in `text` with `?`.
///
/// Kept separate so the renderer can assert that no escape sequence survives without repeating
/// the character table in test code.
#[cfg(test)]
pub(crate) fn assert_printable(text: &str) {
    use std::fmt::Write as _;

    let escaped = printable(text);
    for ch in escaped.chars() {
        assert!(!ch.is_control(), "control character survived: {ch:?}");
    }
    let mut again = String::new();
    for ch in escaped.chars() {
        // Re-escaping must not change anything, including the question marks we inserted.
        let _ = write!(again, "{}", printable(&ch.to_string()));
    }
    assert_eq!(again, escaped, "escaping is not idempotent for {text:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_sequences_and_line_breaks_are_neutralised() {
        assert_eq!(printable("ok\u{1b}[2J\nnext"), "ok?[2J?next");
        assert_eq!(printable("bell\u{7}"), "bell?");
        assert_eq!(printable("c1\u{9b}31m"), "c1?31m");
        assert_printable("ok\u{1b}[2J\nnext\u{7}\u{9b}");
    }

    #[test]
    fn bidi_overrides_are_neutralised() {
        assert_eq!(printable("ab\u{202e}cd"), "ab?cd");
        assert_eq!(printable("x\u{2066}y"), "x?y");
    }

    #[test]
    fn zero_width_and_format_characters_are_neutralised() {
        assert_eq!(
            printable("a\u{200b}b\u{feff}c\u{2060}d\u{200f}e"),
            "a?b?c?d?e"
        );
        assert_eq!(printable("a\u{2028}b\u{ad}c"), "a?b?c");
        assert_eq!(printable("a\u{e0001}b"), "a?b");
        assert_printable("a\u{2028}b\u{ad}c\u{e0001}d");
    }

    #[test]
    fn ordinary_text_passes_through_unchanged() {
        assert_eq!(printable("peer-a 12 café ✓"), "peer-a 12 café ✓");
        assert_eq!(printable("?"), "?");
    }

    #[test]
    fn escaping_is_idempotent() {
        assert_printable("mixed\u{1b}[31m\u{202e}\u{200b}text");
    }
}
