//! Draw every widget with fake data. `--once` prints plain text; otherwise runs a TUI and quits on `q`.

use std::io;

use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::{CrosstermBackend, TestBackend};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;

use fw_tui::text::buffer_text;
use fw_tui::theme::Theme;
use fw_tui::widgets::{
    mask, render_footer_bar, render_frequency_chart, render_header_bar, render_line_chart,
    render_overlay, render_table, render_tabs, status_dot, BarPanel, ColWidth, Column,
    CountdownGauge, DotState, FrequencyBar, LineChartConfig, Panel, Scale, Series, Sparkline,
    TableRow, TableViewState, ToggleChip, ZoomAgg,
};

fn render_gallery(
    frame: &mut ratatui::Frame,
    theme: &Theme,
    table_state: &mut TableViewState,
    show_overlay: bool,
) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    // Header bar inside panel
    Panel::new("Header").render(frame, chunks[0], theme);
    let header_inner = Panel::inner(chunks[0]);
    render_header_bar(
        header_inner,
        frame.buffer_mut(),
        &["fw-tui gallery".into(), "demo".into()],
        &["12:00:00".into(), "[?] help [q] quit".into()],
        theme,
    );

    render_tabs(
        chunks[1],
        frame.buffer_mut(),
        &["Overview", "Table", "Charts"],
        0,
        theme,
    );

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[2]);

    // Table
    Panel::new("Table").render(frame, mid[0], theme);
    let tinner = Panel::inner(mid[0]);
    let cols = vec![
        Column::new("Name", ColWidth::Flex(1)),
        Column::new("Count", ColWidth::Fixed(8)).right(),
    ];
    let mut rows = vec![
        TableRow::new(vec!["peer-a".into(), "120".into()]),
        TableRow::new(vec!["host-1".into(), "42".into()]),
        TableRow::new(vec!["node-x".into(), "7".into()]),
    ];
    rows[0].status = Some(status_dot(DotState::Good, theme, false));
    rows[1].status = Some(status_dot(DotState::Warn, theme, false));
    rows[2].status = Some(status_dot(DotState::Bad, theme, false));
    rows[1].detail = vec![format!("masked: {}", mask("host-1-secret", 1))];
    render_table(tinner, frame.buffer_mut(), &cols, &rows, table_state, theme);

    // Right side charts/gauges
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(4),
        ])
        .split(mid[1]);

    Panel::new("Gauge").render(frame, right[0], theme);
    CountdownGauge::from(15120, 21600)
        .to_gauge("cooldown", 16, theme.warn)
        .render(Panel::inner(right[0]), frame.buffer_mut());
    Panel::new("Spark/Bar").render(frame, right[1], theme);
    Sparkline::new(vec![1.0, 3.0, 2.0, 5.0, 4.0, 7.0, 3.0]).render(
        Panel::inner(right[1]),
        frame.buffer_mut(),
        theme.accent,
    );

    Panel::new("LineChart").render(frame, right[2], theme);
    let series = vec![
        Series::new(
            "rtt",
            vec![Some(1.0), Some(2.0), Some(1.5), None, Some(3.0), Some(2.5)],
        ),
        Series::new(
            "jitter",
            vec![
                Some(0.5),
                Some(0.7),
                Some(0.4),
                Some(0.6),
                Some(0.8),
                Some(0.5),
            ],
        ),
    ];
    render_line_chart(
        Panel::inner(right[2]),
        frame.buffer_mut(),
        &series,
        &LineChartConfig {
            zoom: 1,
            agg: ZoomAgg::Max,
            focused: 0,
        },
        theme,
    );

    Panel::new("Frequency").render(frame, right[3], theme);
    render_frequency_chart(
        Panel::inner(right[3]),
        frame.buffer_mut(),
        &[
            FrequencyBar {
                label: "2xx".into(),
                count: 9,
            },
            FrequencyBar {
                label: "4xx".into(),
                count: 2,
            },
            FrequencyBar {
                label: "5xx".into(),
                count: 1,
            },
        ],
        theme,
    );

    // bottom bar panel and footer
    Panel::new("Samples").render(frame, chunks[3], theme);
    BarPanel::new(vec![1.0, 2.0, 5.0, 3.0, 4.0, 6.0, 2.0, 1.0])
        .scale(Scale::Auto)
        .render(Panel::inner(chunks[3]), frame.buffer_mut(), theme);

    render_footer_bar(
        chunks[4],
        frame.buffer_mut(),
        "ready",
        &[
            ToggleChip::toggle("detail", true),
            ToggleChip::value("privacy", "0"),
        ],
        theme,
    );

    if show_overlay {
        render_overlay(
            area,
            frame.buffer_mut(),
            "Help",
            &["q quit".into(), "? help".into()],
            theme,
        );
    }
}

fn render_once() -> String {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::monochrome();
    let mut state = TableViewState::default();
    terminal
        .draw(|f| render_gallery(f, &theme, &mut state, false))
        .unwrap();
    buffer_text(terminal.backend().buffer())
}

fn run_tui() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let theme = Theme::auto();
    let mut state = TableViewState::default();
    let mut show_overlay = false;
    loop {
        terminal.draw(|f| render_gallery(f, &theme, &mut state, show_overlay))?;
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('?') => show_overlay = !show_overlay,
                    KeyCode::Down => state.selected = state.selected.saturating_add(1),
                    KeyCode::Up => state.selected = state.selected.saturating_sub(1),
                    _ => {}
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--once") {
        println!("{}", render_once());
        Ok(())
    } else {
        run_tui()
    }
}
