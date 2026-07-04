use nom::{bytes::complete::take, number::complete::be_u16, IResult};

#[derive(Debug)]
pub struct EthernetHeader {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ether_type: u16,
}

pub const ETHER_TYPE_IPV4: u16 = 0x0800;
pub const ETHER_TYPE_IPV6: u16 = 0x86DD;

pub fn parse_ethernet(input: &[u8]) -> IResult<&[u8], EthernetHeader> {
    let (input, dst) = take(6usize)(input)?;
    let (input, src) = take(6usize)(input)?;
    let (input, ether_type) = be_u16(input)?;

    Ok((
        input,
        EthernetHeader {
            dst_mac: dst.try_into().unwrap(),
            src_mac: src.try_into().unwrap(),
            ether_type,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ethernet_ipv4() {
        let raw = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
            0x08, 0x00,                          // EtherType IPv4
            0xde, 0xad,                          // 나머지 payload
        ];
        let (rest, header) = parse_ethernet(&raw).unwrap();
        assert_eq!(header.ether_type, ETHER_TYPE_IPV4);
        assert_eq!(rest, &[0xde, 0xad]);
    }
}
