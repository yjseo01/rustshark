use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub filter_bar: Rect,
    pub packet_list: Rect,
    pub packet_detail: Rect,
    pub hex_dump: Rect,
}

pub fn split(area: Rect) -> AppLayout {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(35),
            Constraint::Percentage(25),
        ])
        .split(outer[1]);

    AppLayout {
        filter_bar: outer[0],
        packet_list: inner[0],
        packet_detail: inner[1],
        hex_dump: inner[2],
    }
}
