# Code Style — RustShark

## 네이밍 컨벤션

| 항목 | 규칙 | 예시 |
|------|------|------|
| 변수, 함수, 모듈 | `snake_case` | `parse_modbus`, `src_ip` |
| 구조체, 열거형, 트레이트 | `PascalCase` | `ParsedPacket`, `ProtocolKind` |
| 열거형 변형(variant) | `PascalCase` | `ProtocolKind::ModbusTcp` |
| 상수, 정적 변수 | `SCREAMING_SNAKE_CASE` | `MAX_CHANNEL_SIZE` |
| 제네릭 타입 파라미터 | 단일 대문자 또는 `PascalCase` | `T`, `Input` |

## 에러 처리

- `unwrap()`, `expect()` 사용 금지 — 파싱 실패가 프로세스를 중단시키면 안 됨
- nom 파서는 `IResult`를 반환하고 실패 시 호출자가 `Unknown`으로 fallback 처리
- 애플리케이션 레벨 에러는 `thiserror` 기반 커스텀 에러 타입 사용

```rust
// Bad
let packet = parse_modbus(data).unwrap();

// Good
match parse_modbus(data) {
    Ok((_, packet)) => ProtocolData::Modbus(packet),
    Err(_) => ProtocolData::Unknown(data.to_vec()),
}
```

## 파서 작성 규칙 (nom)

- 파서 함수 시그니처: `fn parse_xxx(input: &[u8]) -> IResult<&[u8], XxxPacket>`
- 파서는 순수 함수로 작성 — 사이드 이펙트 없음
- 파싱 범위 외 PDU 타입은 raw bytes로 보존

```rust
fn parse_modbus_header(input: &[u8]) -> IResult<&[u8], ModbusHeader> {
    let (input, transaction_id) = be_u16(input)?;
    let (input, protocol_id) = be_u16(input)?;
    let (input, length) = be_u16(input)?;
    let (input, unit_id) = u8(input)?;
    Ok((input, ModbusHeader { transaction_id, protocol_id, length, unit_id }))
}
```

## 구조체 / 열거형

- `#[derive]`는 필요한 것만: `Debug`는 기본, `Clone`은 필요할 때만
- 파싱 결과 타입에는 `#[derive(Debug)]` 필수 (TUI 출력용)

```rust
#[derive(Debug)]
pub struct ModbusPacket {
    pub header: ModbusHeader,
    pub pdu: ModbusPdu,
}

#[derive(Debug)]
pub enum ModbusPdu {
    ReadHoldingRegisters { start_addr: u16, quantity: u16 },
    WriteMultipleRegisters { start_addr: u16, values: Vec<u16> },
    ExceptionResponse { function_code: u8, exception_code: u8 },
    Unknown(Vec<u8>),
}
```

## 모듈 구조

- `pub use`로 모듈 내부 타입을 상위로 re-export하지 않음 — 호출 경로를 명확히 유지
- 각 프로토콜 파서는 독립 모듈 (`parser::modbus`, `parser::s7comm`)
- 공용 타입은 `types.rs`에 집중

## 포매팅 & 린트

```bash
cargo fmt        # 코드 포매팅 (rustfmt 기본 설정)
cargo clippy     # 린트 — warning을 error로 취급
```

- PR 전 `cargo fmt`와 `cargo clippy -- -D warnings` 통과 필수
- `#[allow(clippy::...)]` 억제는 이유 주석 없이 사용 금지

## 주석

- 주석은 WHY만 — WHAT은 코드가 설명
- `// TODO:` 는 구체적인 작업 내용 포함

```rust
// Modbus 예외 응답은 Function Code에 0x80을 OR한 값으로 표시됨
let is_exception = function_code & 0x80 != 0;
```

## 테스트

- 프로토콜 파서는 샘플 pcap 기반 단위 테스트 필수
- 테스트 함수명: `test_parse_<프로토콜>_<시나리오>`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modbus_read_holding_registers() {
        let raw = include_bytes!("../../tests/pcap_samples/modbus_read.bin");
        let result = parse_modbus_pdu(raw);
        assert!(result.is_ok());
    }
}
```
