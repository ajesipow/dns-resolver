use crate::core::{parse_name, read_u16, ToBytes};
use anyhow::Result;
use std::io::Cursor;

#[derive(Debug)]
pub(crate) struct DNSQuestion {
    pub name: Vec<u8>,
    pub class: u16,
    pub type_: u16,
}

impl DNSQuestion {
    pub fn new(name: Vec<u8>, class: u16, type_: u16) -> Self {
        Self { name, class, type_ }
    }
}

impl ToBytes for DNSQuestion {
    fn to_bytes(mut self) -> Vec<u8> {
        self.name.extend_from_slice(&self.class.to_be_bytes());
        self.name.extend_from_slice(&self.type_.to_be_bytes());
        self.name
    }
}

pub(crate) fn parse_question(value: &mut Cursor<&[u8]>) -> Result<DNSQuestion> {
    let name = parse_name(value)?;
    let class = read_u16(value)?;
    let type_ = read_u16(value)?;
    Ok(DNSQuestion { name, class, type_ })
}
