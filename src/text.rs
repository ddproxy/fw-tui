use ratatui::buffer::Buffer;

/// Render a buffer to plain text lines, one string per row, joined by newlines.
/// Used by tests and by a non-interactive `--once` mode.
pub fn buffer_text(buffer: &Buffer) -> String {
    let area = buffer.area;
    (area.y..area.y.saturating_add(area.height))
        .map(|y| {
            (area.x..area.x.saturating_add(area.width))
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Approximate display width of a char using a small local table.
/// East Asian Wide/Fullwidth and common emoji/pictograph ranges count as 2,
/// combining marks and variation selectors count as 0, others as 1.
/// Limits: not full Unicode width/grapheme segmentation; sufficient for terminal widgets.
pub fn char_width(ch: char) -> usize {
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

pub fn display_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

/// Truncate to at most `max_width` display cells, never splitting a char.
/// Appends nothing; callers may add ellipsis if desired.
pub fn truncate_width(text: &str, max_width: usize) -> String {
    let mut out = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let w = char_width(ch);
        if used.saturating_add(w) > max_width {
            if used == 0 && max_width == 0 {
                break;
            }
            // do not split; stop before this char
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

/// Pad or truncate to exactly `width` display cells.
pub fn pad_width(text: &str, width: usize) -> String {
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

pub fn fmt_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        let m = secs / 60;
        let s = secs % 60;
        format!("{m}m{s:02}s")
    } else {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        format!("{h}h{m:02}m")
    }
}

pub fn fmt_count(n: u64) -> String {
    if n < 1_000 {
        n.to_string()
    } else if n < 1_000_000 {
        format_scaled(n as f64, 1_000.0, "k")
    } else if n < 1_000_000_000 {
        format_scaled(n as f64, 1_000_000.0, "M")
    } else {
        format_scaled(n as f64, 1_000_000_000.0, "G")
    }
}

fn format_scaled(v: f64, base: f64, suffix: &str) -> String {
    let scaled = v / base;
    // keep one decimal like 1.2k, but avoid ugly .0 when exact enough? spec shows 1.2k, 3.4M
    if (scaled * 10.0).round() >= 100.0 && base < 1_000_000_000.0 {
        // e.g. 999k would be shown as 999k? but input < next base; 999.9 -> 999.9k not possible by range
    }
    let s = format!("{scaled:.1}");
    // trim trailing .0? spec examples have .1 style; keep .1
    format!("{s}{suffix}")
}

pub fn fmt_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    if n < 1024 {
        format!("{n}B")
    } else if (n as f64) < KB * KB {
        format!("{:.1}K", n as f64 / KB)
    } else if (n as f64) < KB * KB * KB {
        format!("{:.1}M", n as f64 / (KB * KB))
    } else {
        format!("{:.1}G", n as f64 / (KB * KB * KB))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;

    #[test]
    fn buffer_text_renders_lines() {
        let backend = TestBackend::new(5, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| f.render_widget(Paragraph::new("ab\ncd"), f.area()))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert_eq!(text.lines().count(), 2);
        assert!(text.contains("ab"));
    }

    #[test]
    fn truncate_and_pad_by_width() {
        assert_eq!(truncate_width("abc", 2), "ab");
        assert_eq!(truncate_width("café", 3), "caf");
        assert_eq!(pad_width("a", 3), "a  ");
        assert_eq!(pad_width("abcd", 2), "ab");
    }

    #[test]
    fn wide_char_not_split() {
        // U+4E2D '中' treated as width 2
        assert_eq!(truncate_width("A中B", 2), "A");
        assert_eq!(truncate_width("A中B", 3), "A中");
        assert_eq!(display_width("A中"), 3);
    }

    #[test]
    fn fmt_helpers() {
        assert_eq!(fmt_duration(45), "45s");
        assert_eq!(fmt_duration(240), "4m00s");
        assert_eq!(fmt_duration(15120), "4h12m");
        assert_eq!(fmt_count(999), "999");
        assert_eq!(fmt_count(1200), "1.2k");
        assert_eq!(fmt_count(3_400_000), "3.4M");
        assert_eq!(fmt_bytes(512), "512B");
        assert_eq!(fmt_bytes(2048), "2.0K");
    }
}
