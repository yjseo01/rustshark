# ARCHITECTURE.md — RustShark

## 기술 스택
- **언어**: Rust 2021 edition
- **UI**: `ratatui` + `crossterm` — 터미널 기반 TUI
- **패킷 캡처**: `pcap` crate (libpcap 바인딩)
- **pcap 파일 I/O**: `pcap-file` 또는 `pcap-parser` crate
- **프로토콜 파싱**: `nom` (파서 콤비네이터)
- **스레드 통신**: `crossbeam-channel` (bounded channel)
- **빌드**: Cargo, 타겟 플랫폼 Linux
- **개발 환경**: Windows WSL2 또는 Linux

## 시스템 구조 (큰 그림)

```
┌─────────────────────────────────────────────────────┐
│                    RustShark 프로세스                  │
│                                                     │
│  ┌──────────────────┐        ┌────────────────────┐ │
│  │   캡처 스레드       │        │    TUI 스레드 (메인)  │ │
│  │                  │        │                    │ │
│  │  libpcap         │        │  ratatui 렌더링 루프  │ │
│  │    ↓             │        │    ↑               │ │
│  │  패킷 수신         │──────→│  패킷 목록 뷰 (상단)   │ │
│  │    ↓             │ bounded│  패킷 상세 뷰 (중단)   │ │
│  │  nom 파서          │channel │  Hex Dump 뷰 (하단)  │ │
│  │  (계층별 파싱)      │        │                    │ │
│  └──────────────────┘        └────────────────────┘ │
│                                      ↑              │
│                              AtomicBool (필터/정지 제어)│
└─────────────────────────────────────────────────────┘
         ↑                              ↑
   네트워크 인터페이스               .pcap 파일 (오프라인)
```

**스레드 간 데이터 흐름**
1. 캡처 스레드: 패킷 수신 → nom으로 파싱 → `ParsedPacket` 구조체 생성 → channel로 전송
2. TUI 스레드: channel에서 수신 → 내부 패킷 목록에 추가 → 렌더링
3. 필터/정지 제어: AtomicBool로 두 스레드 간 상태 공유

## 프로토콜 파싱 계층

```
패킷 raw bytes
    ↓
Ethernet 헤더 파싱
    ↓
IPv4 / IPv6 헤더 파싱
    ↓
TCP / UDP 헤더 파싱
    ↓
포트 번호 기반 프로토콜 분기
    ├── 502 → Modbus TCP 파서
    │         (Function Code, Register Address, Data, Exception)
    ├── 102 → S7comm 파서
    │         TPKT/COTP → S7comm PDU Header
    │         (Job/Ack_Data만, Read Var/Write Var만 필드 파싱)
    │         (그 외 → raw hex)
    └── 기타 → Unknown (raw hex 표시)

※ nom 파싱 실패 시 panic 없이 Unknown으로 fallback
```

## 디렉터리 구조 (예정)

```
rustshark/
├── src/
│   ├── main.rs              # 진입점, 스레드 생성
│   ├── capture/
│   │   └── mod.rs           # libpcap 캡처 루프, pcap 파일 읽기
│   ├── parser/
│   │   ├── mod.rs           # 파싱 진입점, 계층별 디스패치
│   │   ├── ethernet.rs      # Ethernet 헤더 파서
│   │   ├── ip.rs            # IPv4/IPv6 파서
│   │   ├── tcp_udp.rs       # TCP/UDP 파서
│   │   ├── modbus.rs        # Modbus TCP 파서
│   │   └── s7comm.rs        # S7comm 파서 (TPKT/COTP/PDU)
│   ├── tui/
│   │   ├── mod.rs           # ratatui 앱 루프
│   │   ├── layout.rs        # 3단 분할 레이아웃
│   │   ├── packet_list.rs   # 패킷 목록 뷰 (상단)
│   │   ├── packet_detail.rs # 패킷 상세 뷰 (중단)
│   │   └── hex_dump.rs      # Hex Dump 뷰 (하단)
│   ├── filter/
│   │   └── mod.rs           # 디스플레이 필터 (key-value 파싱/적용)
│   └── types.rs             # ParsedPacket 등 공용 타입 정의
├── tests/
│   └── pcap_samples/        # 프로토콜별 샘플 .pcap 파일
├── Cargo.toml
└── Cargo.lock
```

## 주요 기술적 결정

| 결정 | 이유 |
|------|------|
| nom 기반 파서 직접 구현 | 외부 프로토콜 라이브러리 의존성 제거, S7comm 범위 제한 구현 용이 |
| bounded channel 사용 | 캡처 속도가 UI 렌더링보다 빠를 때 메모리 무한 증가 방지 |
| TUI 스레드를 메인으로 | ratatui가 메인 스레드 권장, OS 시그널 처리 용이 |
| 단순 key-value 필터 DSL | MVP 범위 내 구현 복잡도 최소화 (`protocol=modbus`, `src_ip=...`) |
| pcap 파일 기반 테스트 | 개발 환경에서 root 권한 없이 파서 단위 테스트 가능 |
| S7comm 범위 제한 | Job/Ack_Data PDU + Read/Write Var만 필드 파싱, 나머지는 raw hex — 구현 복잡도 대비 실용성 |
