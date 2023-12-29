use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter;
use std::net::UdpSocket;

use anyhow::anyhow;
use anyhow::Result;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::header::DNSHeader;
use crate::packet::DNSPacket;
use crate::question::DNSQuestion;

pub(crate) const TYPE_A: u16 = 1;
pub(crate) const TYPE_NS: u16 = 2;
const CLASS_IN: u16 = 1;

pub(crate) trait ToBytes {
    fn to_bytes(self) -> Vec<u8>;
}

pub(crate) fn try_encode_dns_name(str: &str) -> Result<Vec<u8>> {
    let parts = str
        .split('.')
        .map(try_encode_domain_label)
        .collect::<Result<Vec<_>, _>>()?;
    // Byte 0 indicates end
    Ok(parts
        .into_iter()
        .flatten()
        .chain(iter::once(0))
        .collect::<Vec<u8>>())
}

pub(crate) fn parse_name<R: Read>(value: &mut R) -> Result<Vec<u8>> {
    let mut v = vec![];
    // Length must be > 0
    while let Ok(length @ 1..) = read_u8(value) {
        value.take(length as u64).read_to_end(&mut v)?;
        v.push(b'.');
    }
    // Remove trailing dot
    v.pop();
    Ok(v)
}

pub(crate) fn decode_name(value: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let mut name = vec![];
    while let Ok(length @ 1..) = read_u8(value) {
        // Top bits set means compressed name
        if (length & 0b1100_0000) != 0 {
            let decoded_name = decode_compressed_name(value, length)?;
            name.push(decoded_name);
            break;
        } else {
            let mut buf = Vec::with_capacity(length as usize);
            value.take(length as u64).read_to_end(&mut buf)?;
            name.push(buf);
        }
    }
    Ok(name.join(&b'.'))
}

fn decode_compressed_name(
    value: &mut Cursor<&[u8]>,
    length: u8,
) -> Result<Vec<u8>> {
    let pointer_bytes = [length & 0b0011_1111, read_u8(value)?];
    let pointer = read_u16(&mut &pointer_bytes[..])?;
    let current_position = value.position();
    value.seek(SeekFrom::Start(pointer as u64))?;
    let result = decode_name(value)?;
    value.seek(SeekFrom::Start(current_position))?;
    Ok(result)
}

fn try_encode_domain_label(part: &str) -> Result<Vec<u8>> {
    let part_len = part.len();
    if part_len > 63 {
        return Err(anyhow!("domain part cannot be longer than 63 characters"));
    }
    Ok(iter::once(part_len as u8)
        .chain(part.as_bytes().iter().copied())
        .collect())
}

pub(crate) fn build_query(
    domain_name: &str,
    record_type: u16,
) -> Result<Vec<u8>> {
    let encoded_domain_name = try_encode_dns_name(domain_name)?;
    let question = DNSQuestion::new(encoded_domain_name, CLASS_IN, record_type);
    let id = SmallRng::seed_from_u64(42).gen();
    let header = DNSHeader::default()
        .with_id(id)
        .with_flags(0)
        .with_num_questions(1);
    Ok(header
        .to_bytes()
        .iter()
        .copied()
        .chain(question.to_bytes().iter().copied())
        .collect())
}

pub(crate) fn read_u8<R: Read>(value: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    value.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

pub(crate) fn read_u16<R: Read>(value: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    value.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub(crate) fn read_u32<R: Read>(value: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    value.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub(crate) fn read_n_bytes<R: Read>(
    value: &mut R,
    n: u64,
) -> Result<Vec<u8>> {
    let mut data = vec![];
    value.take(n).read_to_end(&mut data)?;
    Ok(data)
}

pub(crate) fn ip_to_string(raw_ip: &[u8]) -> String {
    raw_ip
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn send_query(
    ip_address: &str,
    domain_name: &str,
    record_type: u16,
) -> Result<DNSPacket> {
    let query = build_query(domain_name, record_type)?;
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&query, format!("{ip_address}:53"))?;
    let mut buf = vec![0; 1024];
    socket.recv_from(&mut buf)?;
    DNSPacket::parse(&buf)
}

pub(crate) fn resolve(
    domain_name: &str,
    record_type: u16,
) -> Result<String> {
    let mut nameserver = "198.41.0.4".to_string();
    loop {
        println!("Querying: {nameserver} for {domain_name}");
        let packet = send_query(&nameserver, domain_name, record_type)?;
        if let Some(raw_ip) = packet.get_answer() {
            let answer = ip_to_string(raw_ip);
            return Ok(answer);
        }
        if let Some(raw_ns_ip) = packet.get_nameserver_ip() {
            nameserver = ip_to_string(raw_ns_ip);
        } else if let Some(new_name_server) = packet.get_nameserver()? {
            nameserver = resolve(&new_name_server, TYPE_A)?;
        } else {
            return Err(anyhow!("should have found either answer, NS IP or NS name"));
        }
    }
}
