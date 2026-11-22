//! Private display-width helpers shared by the terminal-free model and the `tui` text module.

/// Approximate display width of a char using a small local table.
pub(crate) fn char_width(ch: char) -> usize {
    let u = ch as u32;
    if u == 0 {
        return 0;
    }
    // Combining marks and format/variation selectors treated as zero-width here.
    if matches!(u,
        0x0300..=0x036F | // Combining Diacritical Marks
        0x1AB0..=0x1AFF |
        0x1DC0..=0x1DFF |
        0x20D0..=0x20FF |
        0xFE20..=0xFE2F |
        0xFE00..=0xFE0F | // Variation Selectors
        0xE0100..=0xE01EF
    ) {
        return 0;
    }
    // CJK, fullwidth, and common emoji/pictographic ranges as width 2.
    if matches!(u,
        0x1100..=0x115F |
        0x2329..=0x232A |
        0x2E80..=0xA4CF |
        0xAC00..=0xD7A3 |
        0xF900..=0xFAFF |
        0xFE10..=0xFE19 |
        0xFE30..=0xFE6F |
        0xFF01..=0xFF60 |
        0xFFE0..=0xFFE6 |
        0x1F300..=0x1FAFF | // emoji/pictographs
        0x20000..=0x3FFFD
    ) {
        return 2;
    }
    1
}

/// Sum of the display widths of the chars in `text`.
pub(crate) fn display_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

/// Truncate to at most `max_width` display cells, never splitting a char.
pub(crate) fn truncate_width(text: &str, max_width: usize) -> String {
    let mut out = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let w = char_width(ch);
        if used.saturating_add(w) > max_width {
            if used == 0 && max_width == 0 {
                break;
            }
            // Zero-width marks may still be appended without exceeding the budget.
            if w == 0 {
                out.push(ch);
                continue;
            }
            break;
        }
        out.push(ch);
        used = used.saturating_add(w);
    }
    out
}

/// Pad or truncate to exactly `width` display cells, left-aligned.
pub(crate) fn pad_width(text: &str, width: usize) -> String {
    let t = truncate_width(text, width);
    let w = display_width(&t);
    if w >= width {
        t
    } else {
        let mut s = t;
        s.push_str(&" ".repeat(width - w));
        s
    }
}

/// Pad or truncate to exactly `width` display cells, right-aligned.
pub(crate) fn pad_start_width(text: &str, width: usize) -> String {
    let t = truncate_width(text, width);
    let w = display_width(&t);
    if w >= width {
        t
    } else {
        format!("{}{}", " ".repeat(width - w), t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_and_combining_marks_are_measured_in_cells() {
        assert_eq!(display_width("A中"), 3);
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(pad_width("中", 4), "中  ");
        assert_eq!(pad_start_width("7", 3), "  7");
        assert_eq!(truncate_width("A中B", 2), "A");
    }
}
