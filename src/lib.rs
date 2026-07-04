pub mod capture;
pub mod filter;
pub mod parser;
pub mod tui;
pub mod types;

use std::fs::File;

use pcap_file::pcap::PcapReader;

use crate::parser::parse_packet;
use crate::types::ParsedPacket;

/// pcap 파일을 읽어 ParsedPacket 벡터로 반환 (테스트/통합 테스트용)
pub fn capture_read_pcap_to_vec(path: &str) -> Vec<ParsedPacket> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("파일 열기 실패: {e}"); return vec![]; }
    };
    let mut reader = match PcapReader::new(file) {
        Ok(r) => r,
        Err(e) => { eprintln!("pcap 파싱 실패: {e}"); return vec![]; }
    };

    let mut packets = Vec::new();
    while let Some(result) = reader.next_packet() {
        match result {
            Ok(pkt) => {
                let ts = pkt.timestamp.as_secs() as f64
                    + pkt.timestamp.subsec_nanos() as f64 / 1_000_000_000.0;
                packets.push(parse_packet(&pkt.data, ts));
            }
            Err(e) => { eprintln!("패킷 읽기 오류: {e}"); break; }
        }
    }
    packets
}
