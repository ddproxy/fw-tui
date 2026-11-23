//! Minimal configuration: model defaults, no terminal feature required.

use std::time::Instant;

use fw_tui::model::Dashboard;

fn main() {
    let mut dashboard = Dashboard::new(&["peer-a", "host-1"]);
    dashboard.tick();
    let view = dashboard.view(Instant::now());
    assert!(view.plain_text().contains("peer-a"));
    println!("{}", view.plain_text());
}
