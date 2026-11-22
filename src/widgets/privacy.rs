//! Deterministic privacy masks for screenshots.
//!
//! Requires the `tui` feature (re-exported from [`crate::widgets`]).
//!
//! # Examples
//!
//! ```
//! use fw_tui::widgets::mask;
//!
//! assert_eq!(mask("peer-a", 0), "peer-a");
//! assert_eq!(mask("peer-a", 1), "pe..-a");
//! ```

use crate::sanitize::printable;

/// Mask text for screenshots. Level 0 none, 1 abbreviate middle, 2 stable short token.
/// Deterministic fold, no hashing crates.
///
/// Escaping happens before masking, so a control sequence cannot survive at any level.
///
/// # Examples
///
/// ```
/// use fw_tui::widgets::mask;
///
/// let token = mask("host-1", 2);
/// assert!(token.starts_with("id-"));
/// assert_eq!(token, mask("host-1", 2));
/// ```
pub fn mask(text: &str, level: u8) -> String {
    let clean = printable(text);
    match level {
        0 => clean,
        1 => abbreviate_middle(&clean),
        _ => stable_token(&clean),
    }
}

fn abbreviate_middle(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= 4 {
        return "*".repeat(chars.len());
    }
    let head: String = chars.iter().take(2).collect();
    let tail: String = chars
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}..{tail}")
}

fn stable_token(text: &str) -> String {
    // Simple deterministic fold into hex token.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("id-{h:08x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_levels() {
        assert_eq!(mask("peer-a", 0), "peer-a");
        assert_eq!(mask("peer-a", 1), "pe..-a");
        let a = mask("peer-a", 2);
        let b = mask("peer-a", 2);
        let c = mask("peer-b", 2);
        assert_eq!(a, b, "deterministic");
        assert_ne!(a, c);
        assert!(a.starts_with("id-"));
    }

    #[test]
    fn mask_sanitizes() {
        let m = mask("a\u{1b}b", 0);
        assert!(!m.contains('\u{1b}'));
        assert_eq!(m, "a?b");
        for level in [0, 1, 2, 255] {
            let masked = mask("x\u{202e}y", level);
            assert!(!masked.contains('\u{202e}'));
            assert!(!masked.contains('\u{1b}'));
        }
    }

    #[test]
    fn mask_handles_empty_short_and_wide_text() {
        assert_eq!(mask("", 0), "");
        assert_eq!(mask("", 1), "");
        assert_eq!(mask("abc", 1), "***");
        assert_eq!(mask("中中中中中", 1), "中中..中中");
        assert_eq!(mask("e\u{301}fghi", 2), mask("e\u{301}fghi", 2));
    }

    #[test]
    fn token_level_never_stores_the_input() {
        let token = mask("secret-value", 2);
        assert!(!token.contains("secret"));
        assert!(token.starts_with("id-"));
        assert!(token.len() <= "id-".len() + 16);
    }
}
