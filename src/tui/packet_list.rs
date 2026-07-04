use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};
use ratatui::layout::Rect;
use ratatui::widgets::TableState;

use crate::types::{ModbusPdu, ParsedPacket, ProtocolData};

pub fn render(
    frame: &mut Frame,
    area: Rect,
    packets: &[ParsedPacket],
    filtered: &[usize],
    state: &mut TableState,
) {
    let rows: Vec<Row> = filtered.iter().enumerate().map(|(display_idx, &pkt_idx)| {
        let pkt = &packets[pkt_idx];
        let ts = format!("{:.6}", pkt.timestamp);
        let src = pkt.src_ip.as_deref().unwrap_or("-");
        let dst = pkt.dst_ip.as_deref().unwrap_or("-");
        let proto = protocol_name(&pkt.protocol);
        let info = protocol_summary(&pkt.protocol);

        let style = if proto == "Modbus" {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::new(format!("{}", display_idx + 1)),
            Cell::new(ts),
            Cell::new(src.to_string()),
            Cell::new(dst.to_string()),
            Cell::new(proto),
            Cell::new(info),
        ])
        .style(style)
    }).collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(14),
        Constraint::Length(17),
        Constraint::Length(17),
        Constraint::Length(10),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["No.", "Time", "Source", "Destination", "Protocol", "Info"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(Block::default().title("Packets").borders(Borders::ALL))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, state);
}

fn protocol_name(proto: &ProtocolData) -> &'static str {
    match proto {
        ProtocolData::Modbus(_) => "Modbus",
        ProtocolData::Unknown(_) => "Unknown",
    }
}

fn protocol_summary(proto: &ProtocolData) -> String {
    match proto {
        ProtocolData::Modbus(p) => match &p.pdu {
            ModbusPdu::ReadHoldingRegistersReq { start_addr, quantity } =>
                format!("Read Holding Regs Req | addr={start_addr} qty={quantity}"),
            ModbusPdu::ReadHoldingRegistersRes { values } =>
                format!("Read Holding Regs Res | {} regs", values.len()),
            ModbusPdu::WriteMultipleRegistersReq { start_addr, values } =>
                format!("Write Multiple Regs Req | addr={start_addr} qty={}", values.len()),
            ModbusPdu::WriteMultipleRegistersRes { start_addr, quantity } =>
                format!("Write Multiple Regs Res | addr={start_addr} qty={quantity}"),
            ModbusPdu::ExceptionResponse { function_code, exception_code } =>
                format!("Exception | FC={function_code:#04x} code={exception_code:#04x}"),
            ModbusPdu::Unknown(raw) =>
                format!("Modbus Unknown | {} bytes", raw.len()),
        },
        ProtocolData::Unknown(raw) => format!("{} bytes", raw.len()),
    }
}
