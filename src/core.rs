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

/// A trait for converting a structure to bytes
pub(crate) trait ToBytes {
    fn to_bytes(self) -> Vec<u8>;
}

/// Encode a domain name for a DNS query, prepending each label with its length and
/// a final 0-byte.
/// # Errors
/// Encoding can fail if any of the domain labels are longer than 63 characters.
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

/// Decodes a potentially compressed domain name from a DNS record.
/// # Errors
/// Errors if the domain name cannot be decompressed or there is not enough data to read
/// from.
pub(crate) fn decode_name(value: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let mut name = vec![];
    while let Ok(length @ 1..) = read_u8(value) {
        // Top bits set means the name is compressed.
        // A valid domain label is max 63 chars long and will therefore never have its
        // top bits set otherwise.
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

fn build_query(
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

/// Converts raw IPv4 bytes to a String.
/// # Errors
/// Errors if `raw_ip` does not have exactly 4 bytes.
pub(crate) fn ipv4_to_string(raw_ip: &[u8]) -> Result<String> {
    if raw_ip.len() != 4 {
        return Err(anyhow!("raw_ip does not look like IPv4 bytes"));
    }
    Ok(raw_ip
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("."))
}

/// Sends a DNS query for `domain_name` to the given `ip_address` via UDP on port 53.
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

/// Recursively queries nameservers for a given domain name and record type.
pub(crate) fn resolve(
    domain_name: &str,
    record_type: u16,
) -> Result<String> {
    let mut nameserver = "198.41.0.4".to_string();
    loop {
        println!("Querying: {nameserver} for {domain_name}");
        let packet = send_query(&nameserver, domain_name, record_type)?;
        if let Some(raw_ip) = packet.get_answer() {
            let answer = ipv4_to_string(raw_ip)?;
            return Ok(answer);
        }
        if let Some(raw_ns_ip) = packet.get_nameserver_ip() {
            nameserver = ipv4_to_string(raw_ns_ip)?;
        } else if let Some(new_name_server) = packet.get_nameserver()? {
            nameserver = resolve(&new_name_server, TYPE_A)?;
        } else {
            return Err(anyhow!("should have found either answer, NS IP or NS name"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_dns_name_works() {
        assert_eq!(
            try_encode_dns_name("www.google.com").unwrap(),
            vec![3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0]
        );
        assert_eq!(
            try_encode_dns_name("google.com").unwrap(),
            vec![6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0]
        );
    }

    #[test]
    fn test_encoding_and_decoding_dns_name_works() {
        let domain_name = "google.com";
        let encoded = try_encode_dns_name(domain_name).unwrap();
        let decoded = decode_name(&mut Cursor::new(&encoded)).unwrap();
        assert_eq!(domain_name, &String::from_utf8(decoded).unwrap());
    }

    #[test]
    fn test_encoding_dns_name_fails_for_label_too_long() {
        assert!(try_encode_dns_name(
            "www.thisissomereallyreallylonglabelthatistoolongtobeavaliddomainnamelabel.com"
        )
        .is_err());
    }

    #[test]
    fn test_ipv4_string_conversion() {
        assert_eq!(&ipv4_to_string(&[192, 168, 0, 1]).unwrap(), "192.168.0.1");
        assert!(ipv4_to_string(&[192, 168, 0]).is_err());
        assert!(ipv4_to_string(&[192, 168, 0, 1, 2]).is_err());
    }
}
