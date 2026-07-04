/// modbus_test.pcap 파싱 통합 테스트
use rustshark::types::{ModbusPdu, ProtocolData};

#[test]
fn test_pcap_contains_5_packets() {
    let packets = load_pcap();
    assert_eq!(packets.len(), 5, "pcap 파일에 5개 패킷이 있어야 함");
}

#[test]
fn test_first_packet_is_modbus_read_req() {
    let packets = load_pcap();
    let pkt = &packets[0];
    assert!(
        matches!(&pkt.protocol, ProtocolData::Modbus(_)),
        "첫 번째 패킷은 Modbus여야 함"
    );
    if let ProtocolData::Modbus(m) = &pkt.protocol {
        assert!(
            matches!(m.pdu, ModbusPdu::ReadHoldingRegistersReq { start_addr: 0, quantity: 10 }),
            "FC 0x03 Request: start=0, qty=10이어야 함"
        );
    }
}

#[test]
fn test_second_packet_is_modbus_read_res() {
    let packets = load_pcap();
    if let ProtocolData::Modbus(m) = &packets[1].protocol {
        if let ModbusPdu::ReadHoldingRegistersRes { values } = &m.pdu {
            assert_eq!(values.len(), 10);
            assert_eq!(values[0], 1);
            assert_eq!(values[9], 10);
        } else {
            panic!("두 번째 패킷은 ReadHoldingRegistersRes여야 함");
        }
    } else {
        panic!("두 번째 패킷은 Modbus여야 함");
    }
}

#[test]
fn test_third_packet_is_write_multiple_req() {
    let packets = load_pcap();
    if let ProtocolData::Modbus(m) = &packets[2].protocol {
        if let ModbusPdu::WriteMultipleRegistersReq { start_addr, values } = &m.pdu {
            assert_eq!(*start_addr, 1);
            assert_eq!(values, &[10, 20, 30]);
        } else {
            panic!("세 번째 패킷은 WriteMultipleRegistersReq여야 함: {:?}", m.pdu);
        }
    } else {
        panic!("세 번째 패킷은 Modbus여야 함");
    }
}

#[test]
fn test_fifth_packet_is_exception() {
    let packets = load_pcap();
    if let ProtocolData::Modbus(m) = &packets[4].protocol {
        assert!(
            matches!(
                m.pdu,
                ModbusPdu::ExceptionResponse { function_code: 3, exception_code: 2 }
            ),
            "다섯 번째 패킷은 Exception (FC=3, code=2)여야 함: {:?}", m.pdu
        );
    } else {
        panic!("다섯 번째 패킷은 Modbus여야 함");
    }
}

#[test]
fn test_modbus_src_dst_ip() {
    let packets = load_pcap();
    let pkt = &packets[0];
    assert_eq!(pkt.src_ip.as_deref(), Some("127.0.0.1"));
    assert_eq!(pkt.dst_ip.as_deref(), Some("127.0.0.1"));
    assert_eq!(pkt.dst_port, Some(502));
}

fn load_pcap() -> Vec<rustshark::types::ParsedPacket> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/pcap_samples/modbus_test.pcap"
    );
    rustshark::capture_read_pcap_to_vec(path)
}
