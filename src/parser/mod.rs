pub mod ethernet;
pub mod ip;
pub mod modbus;
pub mod tcp_udp;

use crate::types::{ParsedPacket, ProtocolData};
use ethernet::{ETHER_TYPE_IPV4, ETHER_TYPE_IPV6};
use ip::{IP_PROTO_TCP, IP_PROTO_UDP};

pub fn parse_packet(raw: &[u8], timestamp: f64) -> ParsedPacket {
    parse_inner(raw, timestamp).unwrap_or_else(|| ParsedPacket {
        timestamp,
        src_ip: None,
        dst_ip: None,
        src_port: None,
        dst_port: None,
        protocol: ProtocolData::Unknown(raw.to_vec()),
        raw: raw.to_vec(),
    })
}

fn parse_inner(raw: &[u8], timestamp: f64) -> Option<ParsedPacket> {
    let (payload, eth) = ethernet::parse_ethernet(raw).ok()?;

    let (transport_payload, src_ip, dst_ip, ip_proto) = match eth.ether_type {
        ETHER_TYPE_IPV4 => {
            let (rest, hdr) = ip::parse_ipv4(payload).ok()?;
            (rest, hdr.src.to_string(), hdr.dst.to_string(), hdr.protocol)
        }
        ETHER_TYPE_IPV6 => {
            let (rest, hdr) = ip::parse_ipv6(payload).ok()?;
            (rest, hdr.src.to_string(), hdr.dst.to_string(), hdr.next_header)
        }
        _ => return None,
    };

    let (app_payload, src_port, dst_port) = match ip_proto {
        IP_PROTO_TCP => {
            let (rest, hdr) = tcp_udp::parse_tcp(transport_payload).ok()?;
            (rest, hdr.src_port, hdr.dst_port)
        }
        IP_PROTO_UDP => {
            let (rest, hdr) = tcp_udp::parse_udp(transport_payload).ok()?;
            (rest, hdr.src_port, hdr.dst_port)
        }
        _ => return None,
    };

    let protocol = dispatch_protocol(src_port, dst_port, app_payload);

    Some(ParsedPacket {
        timestamp,
        src_ip: Some(src_ip),
        dst_ip: Some(dst_ip),
        src_port: Some(src_port),
        dst_port: Some(dst_port),
        protocol,
        raw: raw.to_vec(),
    })
}

fn dispatch_protocol(src_port: u16, dst_port: u16, payload: &[u8]) -> ProtocolData {
    if src_port == modbus::MODBUS_PORT || dst_port == modbus::MODBUS_PORT {
        match modbus::parse_modbus(payload) {
            Ok((_, pkt)) => ProtocolData::Modbus(pkt),
            Err(_) => ProtocolData::Unknown(payload.to_vec()),
        }
    } else {
        ProtocolData::Unknown(payload.to_vec())
    }
}
