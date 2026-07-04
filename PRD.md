# PRD — Industrial Packet Analyzer

## 1. 서비스 개요
- **서비스 이름**: RustShark
- **한 줄 설명**: Rust 기반 산업용 프로토콜 특화 네트워크 패킷 캡처 및 분석 도구
- **핵심 가치**: 산업 현장에서 자주 사용되는 프로토콜(Modbus, S7comm)의 패킷을 실시간 캡처하고 사람이 읽기 쉬운 형태로 파싱/표시하여 네트워크 디버깅과 보안 모니터링을 돕는다.

## 2. 문제 정의
- Wireshark는 범용 도구로 산업용 프로토콜 분석 시 필터 설정과 해석이 번거롭다.
- 산업 현장 엔지니어에게는 OT(Operational Technology) 프로토콜에 집중된 경량 분석 도구가 필요하다.
- 기존 도구 대부분이 C/C++로 작성되어 메모리 안전성 이슈가 존재하며, 플러그인 확장이 어렵다.

## 3. 타겟 사용자
- **주요 사용자**: OT/ICS 보안 엔지니어, 산업 자동화 엔지니어, 프로토콜 개발자
- **사용자의 니즈**:
  - 산업용 프로토콜 패킷을 실시간으로 캡처/파싱하여 필드 단위로 확인하고 싶다.
  - 특정 디바이스 주소나 Function Code 기반으로 빠르게 필터링하고 싶다.
  - pcap 파일을 불러와서 오프라인 분석도 하고 싶다.

## 4. 핵심 기능

### Must Have
1. **패킷 캡처** — 네트워크 인터페이스를 선택해 실시간 패킷 캡처 (libpcap 바인딩)
2. **기본 프로토콜 디코딩** — Ethernet, IPv4, IPv6, TCP, UDP 헤더 파싱 및 표시
3. **산업용 프로토콜 파싱** — 난이도순 구현:
   - **Modbus TCP** — Function Code, Register Address, Data, Exception Response
   - **S7comm (Siemens S7)** — 구현 범위를 다음으로 한정:
     - TPKT/COTP 헤더 파싱
     - S7comm PDU Header (Job, Ack_Data 타입만)
     - Read Var / Write Var의 Request/Response만 필드 레벨 파싱
     - 그 외 Function/PDU 타입은 raw hex로 표시
     - 참조 자료: Wireshark `packet-s7comm.c` 소스 기반
4. **TUI** — `ratatui` + `crossterm` 기반 터미널 UI, 3단 분할 레이아웃:
   - **패킷 목록 뷰** (상단) — 캡처된 패킷을 시간순으로 표시 (타임스탬프, 출발/도착 IP, 프로토콜, 요약)
   - **패킷 상세 뷰** (중단) — 선택한 패킷의 프로토콜 계층별 필드를 트리 구조로 표시
   - **Hex Dump 뷰** (하단) — 원시 바이트를 Hex + ASCII로 표시
5. **디스플레이 필터** — 프로토콜 종류, 출발/도착 IP, 포트 기반 단순 key-value 필터 (예: `protocol=modbus`, `src_ip=192.168.1.10`)
6. **pcap 파일 입출력** — `.pcap` 파일 읽기/저장

### Nice to Have
1. **Grafana 대시보드** — Prometheus 메트릭 노출 + Grafana 시각화 + Docker Compose 구성
2. **프로토콜 플러그인 시스템** — 정적 trait 기반 `ProtocolParser` 아키텍처
3. **프로토콜 시뮬레이터** — Python 기반 테스트 트래픽 생성기 (pymodbus, python-snap7)
4. **Grafana 알림 연동** — 이상 트래픽 임계치 초과 시 Slack/Email 알림
5. **BPF 캡처 필터** — 캡처 단계에서 BPF 표현식 기반 필터 적용
6. **패킷 북마크 및 코멘트** — 분석 중 특정 패킷에 메모 첨부
7. **세션/스트림 재조합** — TCP 스트림 기반으로 관련 패킷 그룹핑
8. **DNP3 프로토콜 파싱** — Application Layer Object, Function Code, Data Objects
9. **MQTT 프로토콜 파싱** — IoT/IIoT 환경 지원
10. **EtherNet/IP (CIP) 파싱** — Encapsulation Header, CIP Service, Class/Instance/Attribute
11. **OPC UA (Binary) 파싱** — Service Request/Response, NodeId, StatusCode 등
12. **S7comm UserData/기타 PDU 타입 지원 확대**
13. **pcapng 파일 지원** — `.pcapng` 파일 읽기/저장
14. **프로토콜 특화 필드 필터** — `modbus.func_code == 3` 같은 표현식 기반 DSL 필터

## 5. 사용자 시나리오

**시나리오 1: 실시간 Modbus 트래픽 디버깅**
> PLC와 HMI 간 Modbus TCP 통신에서 간헐적 오류가 발생한다. 엔지니어가 터미널에서 RustShark를 실행하고 해당 네트워크 인터페이스를 선택한 뒤, `protocol=modbus` 필터를 적용한다. 패킷 목록에서 Exception Response가 포함된 패킷을 선택하면, 하단 상세 뷰에서 Function Code, Exception Code, 요청된 Register 범위를 한눈에 확인할 수 있다.

**시나리오 2: pcap 파일 기반 보안 감사**
> ICS 보안 담당자가 외부에서 수집된 pcap 파일을 RustShark로 열어 S7comm 트래픽을 필터링한다. PLC에 Write Var 명령을 보낸 IP 주소와 변경된 메모리 영역을 확인하여 비인가 접근 여부를 판단한다.

## 6. 성공 지표
- 지원 프로토콜 2종(Modbus TCP, S7comm)에 대해 지원 범위 내 필드 파싱 정확도 95% 이상
- 실시간 캡처 중 패킷 드랍률 1% 미만
- UI 조작(필터 적용, 패킷 선택, 스크롤) 시 체감 지연 200ms 이내
- pcap 파일 읽기/쓰기가 Wireshark와 호환
- 각 프로토콜 파서별 공개 샘플 pcap 기반 단위 테스트 통과

## 7. 기술 요구사항
- **스레드 구조**: 최소 2개 스레드로 분리:
  - **TUI 스레드** (메인) — ratatui 렌더링 루프
  - **캡처 스레드** — libpcap 패킷 수신 + 파싱, crossbeam-channel의 bounded channel로 TUI 스레드에 전달
  - 캡처 중 UI 조작(필터 변경, 정지/재개)은 AtomicBool 등으로 제어
- **에러 처리**: nom 파서가 파싱 실패 시 panic 하지 않고, 해당 패킷을 "Unknown" 프로토콜로 분류하여 raw hex로 표시. 깨진 패킷(malformed)이 전체 프로세스를 중단시키지 않도록 함
- **보안**: 캡처된 데이터는 로컬에만 저장. 관리자/root 권한 최소화 (캡처 시에만 권한 상승)
- **기술 스택 / 환경**:
  - **언어**: Rust (2021 edition 이상)
  - **UI**: `ratatui` + `crossterm` — 터미널 기반 TUI
  - **패킷 캡처**: `pcap` crate (libpcap 바인딩)
  - **pcap 파일 I/O**: `pcap-file` 또는 `pcap-parser` crate
  - **프로토콜 파싱**: 직접 구현 (nom 기반 파서 콤비네이터 활용). 프로토콜 분기는 단순 match 구문
  - **스레드 통신**: `crossbeam-channel` (캡처 스레드 <-> TUI 스레드)
  - **빌드**: Cargo, 타겟 플랫폼 Linux
  - **개발 환경**: Windows WSL2 또는 Linux 환경에서 개발/테스트
  - **테스트**: 프로토콜별 공개 샘플 pcap 파일 기반 단위 테스트 (실제 네트워크 캡처 불필요)
  - **참조 자료**:
    - S7comm: Wireshark `packet-s7comm.c` 소스, snap7 문서, 공개 pcap 샘플

## 8. 개발 프로세스 참고사항
- **네트워크 캡처 테스트**: 실제 패킷 캡처는 root 권한 + 네트워크 인터페이스가 필요하므로, 개발 중 테스트는 pcap 파일 읽기 기반으로 수행
- **성능 튜닝**: 초기 구현 후 실제 환경에서 프로파일링 → 병목 최적화 이터레이션

## 9. 범위 외 (Out of Scope)
- 무선(Wi-Fi monitor mode) 패킷 캡처
- 패킷 인젝션/리플레이 기능
- 원격 캡처 (rpcap, SSH 터널 등)
- 암호화된 트래픽(TLS) 복호화
- Wireshark dissector 호환 플러그인 로딩
- 동적 플러그인 로딩 (dylib / .so 런타임 로드)
- 모바일 플랫폼 지원
- Windows / macOS 빌드 지원
- GUI (그래픽 UI)

