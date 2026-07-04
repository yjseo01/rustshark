use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use ratatui::layout::Rect;

use crate::types::{ModbusPdu, ParsedPacket, ProtocolData};

pub fn render(frame: &mut Frame, area: Rect, packet: Option<&ParsedPacket>) {
    let items: Vec<ListItem> = match packet {
        None => vec![ListItem::new("패킷을 선택하세요")],
        Some(pkt) => build_detail_items(pkt),
    };

    let list = List::new(items)
        .block(Block::default().title("Packet Details").borders(Borders::ALL));

    frame.render_widget(list, area);
}

fn build_detail_items(pkt: &ParsedPacket) -> Vec<ListItem<'static>> {
    let mut items = Vec::new();

    let header_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let field_style  = Style::default();
    let value_style  = Style::default().fg(Color::Green);

    let push_header = |items: &mut Vec<ListItem<'static>>, text: &str| {
        items.push(ListItem::new(Line::from(Span::styled(text.to_string(), header_style))));
    };
    let push_field = |items: &mut Vec<ListItem<'static>>, key: &str, val: String| {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {key}: "), field_style),
            Span::styled(val, value_style),
        ])));
    };

    // Network layer
    push_header(&mut items, "► Network");
    push_field(&mut items, "Src IP",  pkt.src_ip.clone().unwrap_or_else(|| "-".into()));
    push_field(&mut items, "Dst IP",  pkt.dst_ip.clone().unwrap_or_else(|| "-".into()));
    if let Some(p) = pkt.src_port { push_field(&mut items, "Src Port", p.to_string()); }
    if let Some(p) = pkt.dst_port { push_field(&mut items, "Dst Port", p.to_string()); }

    // Protocol layer
    match &pkt.protocol {
        ProtocolData::Modbus(m) => {
            push_header(&mut items, "► Modbus TCP");
            push_field(&mut items, "Transaction ID", format!("{:#06x}", m.transaction_id));
            push_field(&mut items, "Unit ID",         m.unit_id.to_string());

            match &m.pdu {
                ModbusPdu::ReadHoldingRegistersReq { start_addr, quantity } => {
                    push_header(&mut items, "  ► Read Holding Registers Request");
                    push_field(&mut items, "    Start Address", format!("{start_addr} ({start_addr:#06x})"));
                    push_field(&mut items, "    Quantity",      quantity.to_string());
                }
                ModbusPdu::ReadHoldingRegistersRes { values } => {
                    push_header(&mut items, "  ► Read Holding Registers Response");
                    push_field(&mut items, "    Count", values.len().to_string());
                    for (i, v) in values.iter().enumerate() {
                        push_field(&mut items, &format!("    Reg[{i}]"), format!("{v} ({v:#06x})"));
                    }
                }
                ModbusPdu::WriteMultipleRegistersReq { start_addr, values } => {
                    push_header(&mut items, "  ► Write Multiple Registers Request");
                    push_field(&mut items, "    Start Address", format!("{start_addr} ({start_addr:#06x})"));
                    push_field(&mut items, "    Quantity",      values.len().to_string());
                    for (i, v) in values.iter().enumerate() {
                        push_field(&mut items, &format!("    Data[{i}]"), format!("{v} ({v:#06x})"));
                    }
                }
                ModbusPdu::WriteMultipleRegistersRes { start_addr, quantity } => {
                    push_header(&mut items, "  ► Write Multiple Registers Response");
                    push_field(&mut items, "    Start Address", format!("{start_addr} ({start_addr:#06x})"));
                    push_field(&mut items, "    Quantity",      quantity.to_string());
                }
                ModbusPdu::ExceptionResponse { function_code, exception_code } => {
                    push_header(&mut items, "  ► Exception Response");
                    push_field(&mut items, "    Function Code",   format!("{function_code:#04x}"));
                    push_field(&mut items, "    Exception Code",  format!("{exception_code:#04x} ({})", exception_name(*exception_code)));
                }
                ModbusPdu::Unknown(raw) => {
                    push_header(&mut items, "  ► Unknown PDU");
                    push_field(&mut items, "    Length", raw.len().to_string());
                }
            }
        }
        ProtocolData::Unknown(raw) => {
            push_header(&mut items, "► Unknown Protocol");
            push_field(&mut items, "Length", raw.len().to_string());
        }
    }

    items
}

fn exception_name(code: u8) -> &'static str {
    match code {
        0x01 => "Illegal Function",
        0x02 => "Illegal Data Address",
        0x03 => "Illegal Data Value",
        0x04 => "Server Device Failure",
        0x05 => "Acknowledge",
        0x06 => "Server Device Busy",
        0x08 => "Memory Parity Error",
        0x0a => "Gateway Path Unavailable",
        0x0b => "Gateway Target No Response",
        _ => "Unknown",
    }
}
