use nom::{bytes::complete::take, number::complete::{be_u16, u8}, IResult};

use crate::types::{ModbusPacket, ModbusPdu};

pub const MODBUS_PORT: u16 = 502;

pub fn parse_modbus(input: &[u8]) -> IResult<&[u8], ModbusPacket> {
    let (input, transaction_id) = be_u16(input)?;
    let (input, _protocol_id) = be_u16(input)?;
    let (input, length) = be_u16(input)?;
    let (input, unit_id) = u8(input)?;
    let (input, function_code) = u8(input)?;

    // remaining PDU bytes = length - 2 (unit_id + function_code already consumed)
    let pdu_len = (length as usize).saturating_sub(2);
    let (rest, pdu_bytes) = take(pdu_len.min(input.len()))(input)?;

    let pdu = parse_modbus_pdu(function_code, pdu_bytes);

    Ok((rest, ModbusPacket { transaction_id, unit_id, pdu }))
}

fn parse_modbus_pdu(function_code: u8, input: &[u8]) -> ModbusPdu {
    // Exception: high bit set on function code
    if function_code & 0x80 != 0 {
        return if let Some(&exception_code) = input.first() {
            ModbusPdu::ExceptionResponse {
                function_code: function_code & 0x7f,
                exception_code,
            }
        } else {
            ModbusPdu::Unknown(input.to_vec())
        };
    }

    match function_code {
        0x03 => parse_read_holding(input),
        0x10 => parse_write_multiple(input),
        _ => ModbusPdu::Unknown(input.to_vec()),
    }
}

fn parse_read_holding(input: &[u8]) -> ModbusPdu {
    // Request: start_addr(2) + quantity(2) = exactly 4 bytes
    // Response: byte_count(1) + register_data(byte_count bytes)
    if input.len() == 4 {
        let start_addr = u16::from_be_bytes([input[0], input[1]]);
        let quantity = u16::from_be_bytes([input[2], input[3]]);
        ModbusPdu::ReadHoldingRegistersReq { start_addr, quantity }
    } else if !input.is_empty() {
        let byte_count = input[0] as usize;
        let data = &input[1..];
        if data.len() == byte_count && byte_count % 2 == 0 {
            let values = data.chunks(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            ModbusPdu::ReadHoldingRegistersRes { values }
        } else {
            ModbusPdu::Unknown(input.to_vec())
        }
    } else {
        ModbusPdu::Unknown(input.to_vec())
    }
}

fn parse_write_multiple(input: &[u8]) -> ModbusPdu {
    // Response: start_addr(2) + quantity(2) = exactly 4 bytes
    // Request: start_addr(2) + quantity(2) + byte_count(1) + data(byte_count bytes)
    if input.len() == 4 {
        let start_addr = u16::from_be_bytes([input[0], input[1]]);
        let quantity = u16::from_be_bytes([input[2], input[3]]);
        ModbusPdu::WriteMultipleRegistersRes { start_addr, quantity }
    } else if input.len() >= 5 {
        let start_addr = u16::from_be_bytes([input[0], input[1]]);
        let byte_count = input[4] as usize;
        let data = &input[5..];
        if data.len() == byte_count && byte_count % 2 == 0 {
            let values = data.chunks(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect();
            ModbusPdu::WriteMultipleRegistersReq { start_addr, values }
        } else {
            ModbusPdu::Unknown(input.to_vec())
        }
    } else {
        ModbusPdu::Unknown(input.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modbus_read_req() {
        let raw = [
            0x00, 0x01, // transaction id = 1
            0x00, 0x00, // protocol id = 0
            0x00, 0x06, // length = 6
            0x01,       // unit id = 1
            0x03,       // FC: Read Holding Registers
            0x00, 0x00, // start addr = 0
            0x00, 0x0a, // quantity = 10
        ];
        let (_, pkt) = parse_modbus(&raw).unwrap();
        assert!(matches!(
            pkt.pdu,
            ModbusPdu::ReadHoldingRegistersReq { start_addr: 0, quantity: 10 }
        ));
        assert_eq!(pkt.transaction_id, 1);
        assert_eq!(pkt.unit_id, 1);
    }

    #[test]
    fn test_parse_modbus_exception() {
        let raw = [
            0x00, 0x01,
            0x00, 0x00,
            0x00, 0x03,
            0x01,
            0x83, // FC 3 | 0x80 = exception for FC 3
            0x02, // exception code: Illegal Data Address
        ];
        let (_, pkt) = parse_modbus(&raw).unwrap();
        assert!(matches!(
            pkt.pdu,
            ModbusPdu::ExceptionResponse { function_code: 3, exception_code: 2 }
        ));
    }

    #[test]
    fn test_parse_modbus_write_multiple_req() {
        let raw = [
            0x00, 0x02,
            0x00, 0x00,
            0x00, 0x0b,
            0x01,
            0x10,       // FC: Write Multiple Registers
            0x00, 0x01, // start addr = 1
            0x00, 0x02, // quantity = 2
            0x04,       // byte count = 4
            0x00, 0x0a, // register 1 = 10
            0x00, 0x14, // register 2 = 20
        ];
        let (_, pkt) = parse_modbus(&raw).unwrap();
        if let ModbusPdu::WriteMultipleRegistersReq { start_addr, values } = pkt.pdu {
            assert_eq!(start_addr, 1);
            assert_eq!(values, vec![10, 20]);
        } else {
            panic!("Expected WriteMultipleRegistersReq");
        }
    }
}
