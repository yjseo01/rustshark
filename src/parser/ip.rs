use nom::{bytes::complete::take, number::complete::{be_u16, be_u32, u8}, IResult};
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug)]
pub struct Ipv4Header {
    pub src: Ipv4Addr,
    pub dst: Ipv4Addr,
    pub protocol: u8,
    pub ihl: u8, // 헤더 길이 (4바이트 단위)
}

#[derive(Debug)]
pub struct Ipv6Header {
    pub src: Ipv6Addr,
    pub dst: Ipv6Addr,
    pub next_header: u8,
}

pub const IP_PROTO_TCP: u8 = 6;
pub const IP_PROTO_UDP: u8 = 17;

pub fn parse_ipv4(input: &[u8]) -> IResult<&[u8], Ipv4Header> {
    let (input, ver_ihl) = u8(input)?;
    let ihl = ver_ihl & 0x0f;
    let (input, _dscp_ecn) = u8(input)?;
    let (input, _total_len) = be_u16(input)?;
    let (input, _id) = be_u16(input)?;
    let (input, _flags_frag) = be_u16(input)?;
    let (input, _ttl) = u8(input)?;
    let (input, protocol) = u8(input)?;
    let (input, _checksum) = be_u16(input)?;
    let (input, src_raw) = be_u32(input)?;
    let (input, dst_raw) = be_u32(input)?;

    // options 건너뜀 (ihl > 5이면 options 존재)
    let options_len = (ihl as usize).saturating_sub(5) * 4;
    let (input, _) = take(options_len)(input)?;

    Ok((
        input,
        Ipv4Header {
            src: Ipv4Addr::from(src_raw),
            dst: Ipv4Addr::from(dst_raw),
            protocol,
            ihl,
        },
    ))
}

pub fn parse_ipv6(input: &[u8]) -> IResult<&[u8], Ipv6Header> {
    let (input, _ver_tc_fl) = take(4usize)(input)?; // version + traffic class + flow label
    let (input, _payload_len) = be_u16(input)?;
    let (input, next_header) = u8(input)?;
    let (input, _hop_limit) = u8(input)?;
    let (input, src_raw) = take(16usize)(input)?;
    let (input, dst_raw) = take(16usize)(input)?;

    Ok((
        input,
        Ipv6Header {
            src: Ipv6Addr::from(<[u8; 16]>::try_from(src_raw).unwrap()),
            dst: Ipv6Addr::from(<[u8; 16]>::try_from(dst_raw).unwrap()),
            next_header,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4() {
        // 최소 IPv4 헤더 (ihl=5, no options)
        let raw = [
            0x45,       // version=4, ihl=5
            0x00,       // dscp/ecn
            0x00, 0x28, // total length
            0x00, 0x01, // id
            0x40, 0x00, // flags/fragment
            0x40,       // ttl=64
            0x06,       // protocol=TCP
            0x00, 0x00, // checksum
            192, 168, 1, 1, // src
            192, 168, 1, 2, // dst
        ];
        let (_, header) = parse_ipv4(&raw).unwrap();
        assert_eq!(header.src, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(header.dst, Ipv4Addr::new(192, 168, 1, 2));
        assert_eq!(header.protocol, IP_PROTO_TCP);
    }
}
