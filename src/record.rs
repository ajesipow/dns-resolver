use std::io::Cursor;

use anyhow::Result;

use crate::core::decode_name;
use crate::core::read_n_bytes;
use crate::core::read_u16;
use crate::core::read_u32;
use crate::core::TYPE_A;
use crate::core::TYPE_NS;

#[derive(Debug)]
pub(crate) struct DNSRecord {
    pub _name: Vec<u8>,
    pub type_: u16,
    pub _class: u16,
    pub _ttl: u32,
    pub data: Vec<u8>,
}

impl DNSRecord {
    pub(crate) fn parse(value: &mut Cursor<&[u8]>) -> Result<DNSRecord> {
        let name = decode_name(value)?;
        let type_ = read_u16(value)?;
        let class = read_u16(value)?;
        let ttl = read_u32(value)?;
        let data_len = read_u16(value)?;
        let data = match type_ {
            TYPE_NS => decode_name(value),
            TYPE_A => read_n_bytes(value, data_len as u64),
            _ => read_n_bytes(value, data_len as u64),
        }?;

        Ok(DNSRecord {
            _name: name,
            type_,
            _class: class,
            _ttl: ttl,
            data,
        })
    }
}
