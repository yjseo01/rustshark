/// 캡처/파싱된 패킷 한 개를 나타내는 공용 타입
#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub timestamp: f64,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: ProtocolData,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum ProtocolData {
    Modbus(ModbusPacket),
    Unknown(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct ModbusPacket {
    pub transaction_id: u16,
    pub unit_id: u8,
    pub pdu: ModbusPdu,
}

#[derive(Debug, Clone)]
pub enum ModbusPdu {
    ReadHoldingRegistersReq { start_addr: u16, quantity: u16 },
    ReadHoldingRegistersRes { values: Vec<u16> },
    WriteMultipleRegistersReq { start_addr: u16, values: Vec<u16> },
    WriteMultipleRegistersRes { start_addr: u16, quantity: u16 },
    ExceptionResponse { function_code: u8, exception_code: u8 },
    Unknown(Vec<u8>),
}
