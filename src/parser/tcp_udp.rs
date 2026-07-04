use nom::{bytes::complete::take, number::complete::{be_u16, u8}, IResult};

#[derive(Debug)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
}

#[derive(Debug)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
}

pub fn parse_tcp(input: &[u8]) -> IResult<&[u8], TcpHeader> {
    let (input, src_port) = be_u16(input)?;
    let (input, dst_port) = be_u16(input)?;
    let (input, _seq) = take(4usize)(input)?;
    let (input, _ack) = take(4usize)(input)?;
    let (input, data_offset_flags) = u8(input)?;
    let (input, _flags) = u8(input)?;
    let (input, _window) = be_u16(input)?;
    let (input, _checksum) = be_u16(input)?;
    let (input, _urgent) = be_u16(input)?;

    // options 건너뜀
    let data_offset = (data_offset_flags >> 4) as usize;
    let options_len = data_offset.saturating_sub(5) * 4;
    let (input, _) = take(options_len)(input)?;

    Ok((input, TcpHeader { src_port, dst_port }))
}

pub fn parse_udp(input: &[u8]) -> IResult<&[u8], UdpHeader> {
    let (input, src_port) = be_u16(input)?;
    let (input, dst_port) = be_u16(input)?;
    let (input, _length) = be_u16(input)?;
    let (input, _checksum) = be_u16(input)?;

    Ok((input, UdpHeader { src_port, dst_port }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tcp() {
        let raw = [
            0x1f, 0x90, // src_port = 8080
            0x00, 0x50, // dst_port = 80
            0x00, 0x00, 0x00, 0x01, // seq
            0x00, 0x00, 0x00, 0x00, // ack
            0x50,       // data offset = 5
            0x02,       // flags (SYN)
            0x20, 0x00, // window
            0x00, 0x00, // checksum
            0x00, 0x00, // urgent
        ];
        let (_, header) = parse_tcp(&raw).unwrap();
        assert_eq!(header.src_port, 8080);
        assert_eq!(header.dst_port, 80);
    }

    #[test]
    fn test_parse_udp() {
        let raw = [
            0x00, 0x35, // src_port = 53 (DNS)
            0x04, 0x00, // dst_port = 1024
            0x00, 0x0c, // length
            0x00, 0x00, // checksum
        ];
        let (_, header) = parse_udp(&raw).unwrap();
        assert_eq!(header.src_port, 53);
        assert_eq!(header.dst_port, 1024);
    }
}
