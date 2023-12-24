use crate::header::DNSHeader;
use crate::packet::parse_dns_packet;
use crate::question::DNSQuestion;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::net::UdpSocket;

pub(crate) const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;

pub(crate) trait ToBytes {
    fn to_bytes(self) -> Vec<u8>;
}

pub(crate) fn try_encode_dns_name(str: &str) -> Result<Vec<u8>, String> {
    let parts = str
        .split('.')
        .map(try_encode_part)
        .collect::<Result<Vec<_>, _>>()?;
    let mut encoded = parts.into_iter().flatten().collect::<Vec<u8>>();
    encoded.push(0);
    Ok(encoded)
}

pub(crate) fn parse_name(value: &mut Cursor<&[u8]>) -> Result<Vec<u8>, ()> {
    let mut v = vec![];
    // Length must be > 0
    while let Ok(length @ 1..) = read_u8(value) {
        value
            .take(length as u64)
            .read_to_end(&mut v)
            .map_err(|_| ())?;
        v.push(b'.');
    }
    // Remove trailing dot
    v.pop();
    Ok(v)
}

pub(crate) fn decode_name(value: &mut Cursor<&[u8]>) -> Result<Vec<u8>, ()> {
    let mut name = vec![];
    while let Ok(length @ 1..) = read_u8(value) {
        // Top bits set means compressed name
        if (length & 0b1100_0000) != 0 {
            let decoded_name = decode_compressed_name(value, length)?;
            name.push(decoded_name);
            break;
        } else {
            let mut buf = Vec::with_capacity(length as usize);
            value
                .take(length as u64)
                .read_to_end(&mut buf)
                .map_err(|_| ())?;
            name.push(buf);
        }
    }
    Ok(name.join(&b'.'))
}

fn decode_compressed_name(value: &mut Cursor<&[u8]>, length: u8) -> Result<Vec<u8>, ()> {
    let pointer_bytes = [length & 0b0011_1111, read_u8(value)?];
    let pointer = read_u16(&mut &pointer_bytes[..])?;
    let current_position = value.position();
    value
        .seek(SeekFrom::Start(pointer as u64))
        .map_err(|_| ())?;
    let result = decode_name(value)?;
    value
        .seek(SeekFrom::Start(current_position))
        .map_err(|_| ())?;
    Ok(result)
}

fn try_encode_part(part: &str) -> Result<Vec<u8>, String> {
    let part_len = part.len();
    if part_len > u8::MAX as usize {
        return Err("part too long".to_string());
    }
    // We encode the part itself plus its length
    let mut v = Vec::with_capacity(part_len + 1);
    v.push(part_len as u8);
    v.extend_from_slice(part.as_bytes());
    Ok(v)
}

pub(crate) fn build_query(domain_name: &str, record_type: u16) -> Result<Vec<u8>, String> {
    let encoded_domain_name = try_encode_dns_name(domain_name)?;
    let question = DNSQuestion::new(encoded_domain_name, CLASS_IN, record_type);
    let id = SmallRng::seed_from_u64(42).gen();
    let recursion_desired = 1 << 8;
    let header = DNSHeader::default()
        .with_id(id)
        .with_flags(recursion_desired)
        .with_num_questions(1);
    let mut query = header.to_bytes();
    query.extend(question.to_bytes());
    Ok(query)
}

pub(crate) fn read_u8<R: Read>(value: &mut R) -> Result<u8, ()> {
    let mut buf = [0; 1];
    value.read_exact(&mut buf).map_err(|_| ())?;
    Ok(u8::from_be_bytes(buf))
}

pub(crate) fn read_u16<R: Read>(value: &mut R) -> Result<u16, ()> {
    let mut buf = [0; 2];
    value.read_exact(&mut buf).map_err(|_| ())?;
    Ok(u16::from_be_bytes(buf))
}

pub(crate) fn read_u32<R: Read>(value: &mut R) -> Result<u32, ()> {
    let mut buf = [0; 4];
    value.read_exact(&mut buf).map_err(|_| ())?;
    Ok(u32::from_be_bytes(buf))
}

pub(crate) fn ip_to_string(raw_ip: &[u8]) -> String {
    raw_ip
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn lookup_domain(domain_name: &str) -> String {
    let query = build_query(domain_name, TYPE_A).unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.send_to(&query, "8.8.8.8:53").unwrap();
    let mut buf = vec![0; 128];
    socket.recv_from(&mut buf).unwrap();
    let packet = parse_dns_packet(&buf).unwrap();
    ip_to_string(&packet.answers[0].data)
}
