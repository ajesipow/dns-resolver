use crate::core::{decode_name, read_u16, read_u32};
use std::io::{Cursor, Read};

#[derive(Debug)]
pub(crate) struct DNSRecord {
    pub name: Vec<u8>,
    pub type_: u16,
    pub class: u16,
    pub ttl: u32,
    pub data: Vec<u8>,
}

pub(crate) fn parse_record(value: &mut Cursor<&[u8]>) -> Result<DNSRecord, ()> {
    let name = decode_name(value)?;
    let type_ = read_u16(value)?;
    let class = read_u16(value)?;
    let ttl = read_u32(value)?;
    let data_len = read_u16(value)?;
    let mut data = vec![];
    value
        .take(data_len as u64)
        .read_to_end(&mut data)
        .map_err(|_| ())?;
    Ok(DNSRecord {
        name,
        type_,
        class,
        ttl,
        data,
    })
}
