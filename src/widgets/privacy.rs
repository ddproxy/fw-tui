use crate::sanitize::printable;

/// Mask text for screenshots. Level 0 none, 1 abbreviate middle, 2 stable short token.
/// Deterministic fold, no hashing crates.
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
    }
}
