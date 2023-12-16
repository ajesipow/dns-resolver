use crate::core::ToBytes;

#[derive(Debug, Default)]
pub(crate) struct DNSHeader {
    id: u16,
    flags: u16,
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16,
}

impl DNSHeader {
    pub fn with_id(mut self, id: u16) -> Self {
        self.id = id;
        self
    }
    
    pub fn with_flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }
    
    pub fn with_num_questions(mut self, num_questions: u16) -> Self {
        self.num_questions = num_questions;
        self
    }
    
    pub fn with_num_answers(mut self, num_answers: u16) -> Self {
        self.num_answers = num_answers;
        self
    }
    
    pub fn with_num_authorities(mut self, num_authorities: u16) -> Self {
        self.num_authorities = num_authorities;
        self
    }
    
    pub fn with_num_additionals(mut self, num_additionals: u16) -> Self {
        self.num_additionals = num_additionals;
        self
    }
}

impl ToBytes for DNSHeader {
    fn to_bytes(self) -> Vec<u8> {
        // DNS headers are 12 bytes long
        let mut v = Vec::with_capacity(12);

        v.extend_from_slice(&self.id.to_be_bytes());
        v.extend_from_slice(&self.flags.to_be_bytes());
        v.extend_from_slice(&self.num_questions.to_be_bytes());
        v.extend_from_slice(&self.num_answers.to_be_bytes());
        v.extend_from_slice(&self.num_authorities.to_be_bytes());
        v.extend_from_slice(&self.num_additionals.to_be_bytes());
        v
    }
}
