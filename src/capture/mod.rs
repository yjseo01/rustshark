use std::fs::File;
use std::time::Duration;

use crossbeam_channel::Sender;
use pcap_file::pcap::{PcapPacket, PcapReader, PcapWriter};

use crate::parser::parse_packet;
use crate::types::ParsedPacket;

pub fn read_pcap_file(path: &str, tx: &Sender<ParsedPacket>) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { eprintln!("pcap 파일 열기 실패: {e}"); return; }
    };
    let mut reader = match PcapReader::new(file) {
        Ok(r) => r,
        Err(e) => { eprintln!("pcap 파일 파싱 실패: {e}"); return; }
    };

    while let Some(result) = reader.next_packet() {
        let pkt = match result {
            Ok(p) => p,
            Err(e) => { eprintln!("패킷 읽기 오류: {e}"); break; }
        };
        let ts = pkt.timestamp.as_secs() as f64
            + pkt.timestamp.subsec_nanos() as f64 / 1_000_000_000.0;
        let parsed = parse_packet(&pkt.data, ts);
        if tx.send(parsed).is_err() {
            break; // TUI 스레드 종료됨
        }
    }
}

pub fn save_pcap_file(path: &str, packets: &[ParsedPacket]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut writer = PcapWriter::new(file).map_err(|e| e.to_string())?;

    for pkt in packets {
        let secs = pkt.timestamp.max(0.0) as u64;
        let nanos = ((pkt.timestamp.max(0.0) - secs as f64) * 1_000_000_000.0) as u32;
        let ts = Duration::new(secs, nanos);
        let raw_len = pkt.raw.len() as u32;
        let pcap_pkt = PcapPacket::new(ts, raw_len, &pkt.raw);
        writer.write_packet(&pcap_pkt).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn start_capture(interface: &str, tx: &Sender<ParsedPacket>) {
    let cap = match pcap::Capture::from_device(interface) {
        Ok(c) => c,
        Err(e) => { eprintln!("캡처 디바이스 오류: {e}"); return; }
    };
    let mut cap = match cap.immediate_mode(true).open() {
        Ok(c) => c,
        Err(e) => { eprintln!("캡처 시작 실패: {e}"); return; }
    };

    loop {
        match cap.next_packet() {
            Ok(packet) => {
                let ts = packet.header.ts.tv_sec as f64
                    + packet.header.ts.tv_usec as f64 / 1_000_000.0;
                let parsed = parse_packet(packet.data, ts);
                if tx.send(parsed).is_err() {
                    break;
                }
            }
            Err(pcap::Error::TimeoutExpired) => continue,
            Err(e) => { eprintln!("캡처 오류: {e}"); break; }
        }
    }
}
