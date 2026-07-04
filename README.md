# RustShark

Rust 기반 Modbus TCP 특화 네트워크 패킷 캡처/분석 TUI 도구.

```
┌─ RustShark ─────────────────────────────────────────────────────┐
│ [/] 필터  [j/k↑↓] 이동  [p] 일시정지  [s] 저장  [q] 종료       │
├─ Packets ───────────────────────────────────────────────────────┤
│ No.  Time          Source       Destination  Protocol  Info     │
│▶ 1   1.000000      127.0.0.1    127.0.0.1    Modbus    Read ... │
│  2   1.001000      127.0.0.1    127.0.0.1    Modbus    Read ... │
├─ Packet Details ────────────────────────────────────────────────┤
│ ► Network                                                       │
│   Src IP: 127.0.0.1    Dst Port: 502                           │
│ ► Modbus TCP                                                    │
│   Transaction ID: 0x0001   Unit ID: 1                          │
│   ► Read Holding Registers Request                             │
│     Start Address: 0 (0x0000)   Quantity: 10                   │
├─ Hex Dump ──────────────────────────────────────────────────────┤
│ 00000000  00 00 00 01 08 00 45 00  00 34 00 01 40 00 40 06  │
└─────────────────────────────────────────────────────────────────┘
```

## 기능

- **실시간 캡처** — 네트워크 인터페이스 선택 후 라이브 캡처 (libpcap)
- **Modbus TCP 파싱** — FC 0x03 / FC 0x10 / Exception Response 필드 레벨 파싱
- **pcap 파일 입출력** — `.pcap` 파일 읽기 및 저장
- **TUI** — 패킷 목록 / 프로토콜 상세 / Hex Dump 3단 분할 레이아웃
- **디스플레이 필터** — `protocol=modbus`, `src_ip=192.168.1.10` 형식 key-value 필터

## 설치

```bash
# libpcap 설치 (라이브 캡처 시 필요)
sudo apt install libpcap-dev   # Ubuntu/Debian

git clone <repo>
cd rustshark
cargo build --release
```

## 사용법

```bash
# pcap 파일 분석
rustshark -r capture.pcap

# 라이브 캡처 (root 권한 필요)
sudo rustshark -i eth0
```

## 키 바인딩

| 키 | 동작 |
|----|------|
| `j` / `↓` | 다음 패킷 |
| `k` / `↑` | 이전 패킷 |
| `g` / `G` | 처음 / 마지막으로 이동 |
| `/` | 필터 입력 |
| `Esc` | 필터 초기화 |
| `p` | 캡처 일시정지 / 재개 |
| `s` | `capture.pcap`으로 저장 |
| `q` | 종료 |

## 디스플레이 필터

TUI에서 `/` 키로 진입. `Enter`로 적용, `Esc`로 취소.

```
protocol=modbus
src_ip=192.168.1.10
protocol=modbus dst_ip=10.0.0.1 dst_port=502
```

| 키 | 값 예시 |
|----|---------|
| `protocol` | `modbus`, `unknown` |
| `src_ip` | `192.168.1.10` |
| `dst_ip` | `10.0.0.1` |
| `src_port` | `12345` |
| `dst_port` | `502` |

## 기술 스택

| 역할 | 크레이트 |
|------|---------|
| TUI | `ratatui` + `crossterm` |
| 라이브 캡처 | `pcap` (libpcap 바인딩) |
| pcap 파일 I/O | `pcap-file` |
| 프로토콜 파싱 | `nom` |
| 스레드 통신 | `crossbeam-channel` |

## 테스트

```bash
cargo test
```

단위 테스트 7개 + 통합 테스트 6개 (실제 pcap 파일 파싱 검증).

## 제한 사항

- Linux 전용 (Windows/macOS 미지원)
- 라이브 캡처는 root 권한 필요
- 지원 프로토콜: Modbus TCP (FC 0x03, FC 0x10, Exception)
- WSL2 환경에서는 라이브 캡처가 제한될 수 있음
