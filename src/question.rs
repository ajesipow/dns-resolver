use crate::core::ToBytes;

#[derive(Debug)]
pub(crate) struct DNSQuestion {
    name: Vec<u8>,
    class: u16,
    type_: u16,
}

impl DNSQuestion {
    pub fn new(name: Vec<u8>, class: u16, type_:u16) -> Self {
        Self {
            name,
            class,
            type_,
        }
    }
}

impl ToBytes for DNSQuestion {
    fn to_bytes(mut self) -> Vec<u8> {
        self.name.extend_from_slice(&self.class.to_be_bytes());
        self.name.extend_from_slice(&self.type_.to_be_bytes());
        self.name
    }
}
