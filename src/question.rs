use std::io::Cursor;

use anyhow::Result;

use crate::core::decode_name;
use crate::core::read_u16;
use crate::core::ToBytes;

#[derive(Debug)]
pub(crate) struct DNSQuestion {
    pub name: Vec<u8>,
    pub class: u16,
    pub type_: u16,
}

impl DNSQuestion {
    pub fn new(
        name: Vec<u8>,
        class: u16,
        type_: u16,
    ) -> Self {
        Self { name, class, type_ }
    }

    pub(crate) fn parse(value: &mut Cursor<&[u8]>) -> Result<Self> {
        let name = decode_name(value)?;
        let class = read_u16(value)?;
        let type_ = read_u16(value)?;
        Ok(Self { name, class, type_ })
    }
}

impl ToBytes for DNSQuestion {
    fn to_bytes(mut self) -> Vec<u8> {
        self.name.extend_from_slice(&self.class.to_be_bytes());
        self.name.extend_from_slice(&self.type_.to_be_bytes());
        self.name
    }
}
