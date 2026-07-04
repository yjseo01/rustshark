# 실행 계획: 기본 프로토콜 파서 구현

**범위**: Ethernet, IPv4/IPv6, TCP/UDP 헤더 파싱  
**목표**: raw 패킷 bytes → `ParsedPacket` 타입까지 연결  

## 작업 목록

- [x] 1. `parser/ethernet.rs` — Ethernet II 헤더 파싱
- [x] 2. `parser/ip.rs` — IPv4 헤더 파싱
- [x] 3. `parser/ip.rs` — IPv6 헤더 파싱
- [x] 4. `parser/tcp_udp.rs` — TCP 헤더 파싱
- [x] 5. `parser/tcp_udp.rs` — UDP 헤더 파싱
- [x] 6. `parser/mod.rs` — 계층별 디스패치 연결 (`parse_packet` 완성)
- [x] 7. 단위 테스트 — 각 파서별 샘플 bytes 기반 (4개 통과)

## 구현 순서 이유
Ethernet → IP → TCP/UDP 순으로 계층이 중첩되어 있어,  
상위 계층 파서가 하위 계층 파서의 결과를 받아서 분기함.

## 파싱 실패 정책
nom 에러 → `ProtocolData::Unknown(raw.to_vec())` fallback, panic 없음
