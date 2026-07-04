use crate::types::{ParsedPacket, ProtocolData};

/// key-value 디스플레이 필터 (예: `protocol=modbus`, `src_ip=192.168.1.10`)
#[derive(Debug, Default, Clone)]
pub struct DisplayFilter {
    pub protocol: Option<String>,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
}

impl DisplayFilter {
    pub fn parse(input: &str) -> Self {
        let mut filter = DisplayFilter::default();
        for part in input.split_whitespace() {
            if let Some((key, value)) = part.split_once('=') {
                match key.trim() {
                    "protocol" => filter.protocol = Some(value.trim().to_lowercase()),
                    "src_ip"   => filter.src_ip   = Some(value.trim().to_string()),
                    "dst_ip"   => filter.dst_ip   = Some(value.trim().to_string()),
                    "src_port" => filter.src_port  = value.trim().parse().ok(),
                    "dst_port" => filter.dst_port  = value.trim().parse().ok(),
                    _ => {}
                }
            }
        }
        filter
    }

    pub fn is_empty(&self) -> bool {
        self.protocol.is_none()
            && self.src_ip.is_none()
            && self.dst_ip.is_none()
            && self.src_port.is_none()
            && self.dst_port.is_none()
    }

    pub fn matches(&self, packet: &ParsedPacket) -> bool {
        if let Some(ref proto) = self.protocol {
            let packet_proto = match &packet.protocol {
                ProtocolData::Modbus(_) => "modbus",
                ProtocolData::Unknown(_) => "unknown",
            };
            if packet_proto != proto.as_str() {
                return false;
            }
        }
        if let Some(ref ip) = self.src_ip {
            if packet.src_ip.as_deref() != Some(ip.as_str()) {
                return false;
            }
        }
        if let Some(ref ip) = self.dst_ip {
            if packet.dst_ip.as_deref() != Some(ip.as_str()) {
                return false;
            }
        }
        if let Some(port) = self.src_port {
            if packet.src_port != Some(port) {
                return false;
            }
        }
        if let Some(port) = self.dst_port {
            if packet.dst_port != Some(port) {
                return false;
            }
        }
        true
    }

    pub fn display_str(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref p)    = self.protocol { parts.push(format!("protocol={p}")); }
        if let Some(ref ip)   = self.src_ip   { parts.push(format!("src_ip={ip}"));  }
        if let Some(ref ip)   = self.dst_ip   { parts.push(format!("dst_ip={ip}"));  }
        if let Some(port)     = self.src_port  { parts.push(format!("src_port={port}")); }
        if let Some(port)     = self.dst_port  { parts.push(format!("dst_port={port}")); }
        parts.join(" ")
    }
}
