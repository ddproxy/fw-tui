//! Text and buffer helpers used by the drawing code and by non-interactive output.
//!
//! Requires the `tui` feature because [`buffer_text`] reads a ratatui buffer; the pure width and
//! formatting helpers work on ordinary strings.
//!
//! # Examples
//!
//! ```
//! use fw_tui::text::{fmt_bytes, fmt_count, pad_width};
//!
//! assert_eq!(pad_width("中", 4), "中  ");
//! assert_eq!(fmt_count(1200), "1.2k");
//! assert_eq!(fmt_bytes(1024), "1.0K");
//! ```

use ratatui::buffer::Buffer;

/// Render a buffer to plain text lines, one string per row, joined by newlines.
/// Used by tests and by a non-interactive `--once` mode.
///
/// An empty buffer produces an empty string.
///
/// # Examples
///
/// ```
/// use fw_tui::text::buffer_text;
/// use ratatui::backend::TestBackend;
/// use ratatui::widgets::Paragraph;
/// use ratatui::Terminal;
///
/// let mut terminal = Terminal::new(TestBackend::new(5, 2)).unwrap();
/// terminal
///     .draw(|f| f.render_widget(Paragraph::new("ab\ncd"), f.area()))
///     .unwrap();
/// let text = buffer_text(terminal.backend().buffer());
/// assert_eq!(text, "ab   \ncd   ");
/// ```
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
///
/// # Examples
///
/// ```
/// use fw_tui::text::char_width;
///
/// assert_eq!(char_width('A'), 1);
/// assert_eq!(char_width('中'), 2);
/// assert_eq!(char_width('\u{301}'), 0);
/// assert_eq!(char_width('\0'), 0);
/// ```
pub fn char_width(ch: char) -> usize {
    crate::width::char_width(ch)
}

/// Display width of `text`, the sum of [`char_width`] over its chars.
///
/// # Examples
///
/// ```
/// use fw_tui::text::display_width;
///
/// assert_eq!(display_width("A中"), 3);
/// assert_eq!(display_width("e\u{301}"), 1);
/// assert_eq!(display_width(""), 0);
/// ```
pub fn display_width(text: &str) -> usize {
    crate::width::display_width(text)
}

/// Truncate to at most `max_width` display cells, never splitting a char.
/// Appends nothing; callers may add ellipsis if desired.
///
/// # Examples
///
/// ```
/// use fw_tui::text::truncate_width;
///
/// assert_eq!(truncate_width("A中B", 2), "A");
/// assert_eq!(truncate_width("A中B", 3), "A中");
/// assert_eq!(truncate_width("abc", 0), "");
/// ```
pub fn truncate_width(text: &str, max_width: usize) -> String {
    crate::width::truncate_width(text, max_width)
}

/// Pad or truncate to exactly `width` display cells.
///
/// Padding goes on the right, so this is left-aligned text.
///
/// # Examples
///
/// ```
/// use fw_tui::text::pad_width;
///
/// assert_eq!(pad_width("a", 3), "a  ");
/// assert_eq!(pad_width("中", 4), "中  ");
/// assert_eq!(pad_width("abcd", 2), "ab");
/// ```
pub fn pad_width(text: &str, width: usize) -> String {
    crate::width::pad_width(text, width)
}

/// Format seconds as `Ns`, `NmNNs` or `NhNNm`.
///
/// # Examples
///
/// ```
/// use fw_tui::text::fmt_duration;
///
/// assert_eq!(fmt_duration(45), "45s");
/// assert_eq!(fmt_duration(240), "4m00s");
/// assert_eq!(fmt_duration(15120), "4h12m");
/// ```
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

/// Format a count with one decimal and a `k`, `M` or `G` suffix above 999.
///
/// # Examples
///
/// ```
/// use fw_tui::text::fmt_count;
///
/// assert_eq!(fmt_count(999), "999");
/// assert_eq!(fmt_count(1200), "1.2k");
/// assert_eq!(fmt_count(3_400_000), "3.4M");
/// ```
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
    format!("{scaled:.1}{suffix}")
}

/// Format a byte count with binary `K`, `M` and `G` suffixes.
///
/// # Examples
///
/// ```
/// use fw_tui::text::fmt_bytes;
///
/// assert_eq!(fmt_bytes(512), "512B");
/// assert_eq!(fmt_bytes(2048), "2.0K");
/// assert!(fmt_bytes(u64::MAX).ends_with('G'));
/// ```
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
        assert_eq!(text, "ab   \ncd   ");
    }

    #[test]
    fn buffer_text_handles_zero_sized_areas() {
        assert_eq!(
            buffer_text(&Buffer::empty(ratatui::layout::Rect::new(0, 0, 0, 0))),
            ""
        );
        assert_eq!(
            buffer_text(&Buffer::empty(ratatui::layout::Rect::new(3, 2, 0, 4))),
            "\n\n\n"
        );
    }

    #[test]
    fn truncate_and_pad_by_width() {
        assert_eq!(truncate_width("abc", 2), "ab");
        assert_eq!(truncate_width("café", 3), "caf");
        assert_eq!(pad_width("a", 3), "a  ");
        assert_eq!(pad_width("abcd", 2), "ab");
        assert_eq!(display_width(&pad_width("a", 3)), 3);
    }

    #[test]
    fn wide_char_not_split() {
        assert_eq!(truncate_width("A中B", 2), "A");
        assert_eq!(truncate_width("A中B", 3), "A中");
        assert_eq!(display_width("A中"), 3);
    }

    #[test]
    fn combining_marks_do_not_consume_cells() {
        let text = "e\u{301}\u{301}中";
        assert_eq!(display_width(text), 3);
        assert_eq!(truncate_width(text, 3), text);
        assert_eq!(truncate_width(text, 2), "e\u{301}\u{301}");
        assert_eq!(display_width(&pad_width(text, 5)), 5);
    }

    #[test]
    fn very_long_and_empty_inputs_are_safe() {
        let long = "x".repeat(4096);
        assert_eq!(truncate_width(&long, 7).len(), 7);
        assert_eq!(pad_width("", 0), "");
        assert_eq!(pad_width("", 4), "    ");
        assert_eq!(display_width(""), 0);
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
        assert!(fmt_count(u64::MAX).ends_with('G'));
    }
}
