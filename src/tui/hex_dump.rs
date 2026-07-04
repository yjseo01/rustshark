use ratatui::{
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use ratatui::layout::Rect;

pub fn render(frame: &mut Frame, area: Rect, raw: &[u8]) {
    let content = format_hex_dump(raw);
    let para = Paragraph::new(content)
        .block(Block::default().title("Hex Dump").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(para, area);
}

fn format_hex_dump(raw: &[u8]) -> String {
    if raw.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(raw.len() * 4);

    for (i, chunk) in raw.chunks(16).enumerate() {
        let offset = i * 16;

        // hex part
        let mut hex = String::with_capacity(48);
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 { hex.push(' '); } // extra space between groups of 8
            hex.push_str(&format!("{byte:02x} "));
        }

        // ASCII part
        let ascii: String = chunk.iter().map(|&b| {
            if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
        }).collect();

        out.push_str(&format!("{offset:08x}  {hex:<49} {ascii}\n"));
    }

    out
}
