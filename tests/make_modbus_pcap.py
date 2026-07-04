#!/usr/bin/env python3
"""
Modbus TCP pcap 샘플 파일 생성기.
실제 네트워크 없이 테스트용 pcap을 만든다.
"""
import struct, os

OUT = os.path.join(os.path.dirname(__file__), "pcap_samples", "modbus_test.pcap")

# ── 프레임 빌더 ────────────────────────────────────────────────────────────────

def eth_ip_tcp(src_port: int, dst_port: int, payload: bytes,
               seq: int = 1, ack: int = 1) -> bytes:
    """Ethernet + IPv4 + TCP 프레임 (체크섬 0x0000, 파서는 검증 안 함)"""
    # Modbus 요청: client(random port) → server(502)
    # Modbus 응답: server(502) → client(random port)
    src_mac = b"\x00\x00\x00\x00\x00\x01"
    dst_mac = b"\x00\x00\x00\x00\x00\x02"
    src_ip  = b"\x7f\x00\x00\x01"   # 127.0.0.1
    dst_ip  = b"\x7f\x00\x00\x01"

    tcp = struct.pack(">HHIIHHHH",
        src_port, dst_port,
        seq, ack,
        0x5018,          # data offset=5, flags=PSH|ACK
        65535,           # window
        0x0000,          # checksum (skipped)
        0,               # urgent
    ) + payload

    ip_len = 20 + len(tcp)
    ip = struct.pack(">BBHHHBBH4s4s",
        0x45, 0,         # version+IHL, DSCP
        ip_len,
        0x0001,          # ID
        0x4000,          # flags: DF
        64, 6,           # TTL, protocol=TCP
        0x0000,          # checksum (skipped)
        src_ip, dst_ip,
    ) + tcp

    eth = dst_mac + src_mac + b"\x08\x00" + ip
    return eth


def pcap_file(frames: list[tuple[float, bytes]]) -> bytes:
    """pcap 파일 bytes 생성 (big-endian magic = 0xa1b2c3d4, microsecond)"""
    # Global header — big-endian이 pcap-file crate에서 안전하게 파싱됨
    buf = struct.pack(">IHHiIII",
        0xa1b2c3d4,  # magic: big-endian, microsecond
        2, 4,        # version
        0, 0,        # tz, ts accuracy
        65535,       # snaplen
        1,           # link type: ETHERNET
    )
    for ts, data in frames:
        ts_sec  = int(ts)
        ts_usec = int((ts - ts_sec) * 1_000_000)
        buf += struct.pack(">IIII", ts_sec, ts_usec, len(data), len(data))
        buf += data
    return buf


# ── Modbus 페이로드 ────────────────────────────────────────────────────────────

# FC 0x03 Read Holding Registers — Request (start=0, qty=10)
mb_read_req = bytes([
    0x00, 0x01,  # transaction id
    0x00, 0x00,  # protocol id
    0x00, 0x06,  # length
    0x01,        # unit id
    0x03,        # FC: Read Holding Registers
    0x00, 0x00,  # start addr = 0
    0x00, 0x0a,  # quantity = 10
])

# FC 0x03 Read Holding Registers — Response (10 registers: 1..10)
vals = b"".join(struct.pack(">H", i) for i in range(1, 11))  # 20 bytes
mb_read_res = bytes([
    0x00, 0x01,
    0x00, 0x00,
    0x00, 0x17,  # length = 1+1+1+20 = 23
    0x01,
    0x03,
    0x14,        # byte count = 20
]) + vals

# FC 0x10 Write Multiple Registers — Request (start=1, qty=3, values=10,20,30)
mb_write_req = bytes([
    0x00, 0x02,
    0x00, 0x00,
    0x00, 0x0f,  # length = 1+1+2+2+1+6 = 13... actually 1(unit)+1(FC)+2+2+1+6=13
    0x01,
    0x10,        # FC: Write Multiple Registers
    0x00, 0x01,  # start addr = 1
    0x00, 0x03,  # quantity = 3
    0x06,        # byte count = 6
    0x00, 0x0a,  # register 0 = 10
    0x00, 0x14,  # register 1 = 20
    0x00, 0x1e,  # register 2 = 30
])

# FC 0x10 Write Multiple Registers — Response
mb_write_res = bytes([
    0x00, 0x02,
    0x00, 0x00,
    0x00, 0x06,
    0x01,
    0x10,
    0x00, 0x01,  # start addr = 1
    0x00, 0x03,  # quantity = 3
])

# Exception Response: FC 0x83 (exception for FC 0x03), code 0x02 = Illegal Data Address
mb_exception = bytes([
    0x00, 0x03,
    0x00, 0x00,
    0x00, 0x03,
    0x01,
    0x83,  # FC 3 | 0x80
    0x02,  # Illegal Data Address
])

# ── 프레임 조립 ────────────────────────────────────────────────────────────────

CLIENT_PORT = 12345
SERVER_PORT = 502

frames = [
    (1.000000, eth_ip_tcp(CLIENT_PORT, SERVER_PORT, mb_read_req,  seq=100, ack=0)),
    (1.001000, eth_ip_tcp(SERVER_PORT, CLIENT_PORT, mb_read_res,  seq=200, ack=100)),
    (2.000000, eth_ip_tcp(CLIENT_PORT, SERVER_PORT, mb_write_req, seq=100, ack=0)),
    (2.001000, eth_ip_tcp(SERVER_PORT, CLIENT_PORT, mb_write_res, seq=200, ack=100)),
    (3.000000, eth_ip_tcp(CLIENT_PORT, SERVER_PORT, mb_exception, seq=100, ack=0)),
]

os.makedirs(os.path.dirname(OUT), exist_ok=True)
with open(OUT, "wb") as f:
    f.write(pcap_file(frames))

print(f"생성 완료: {OUT}  ({os.path.getsize(OUT)} bytes, {len(frames)} packets)")
