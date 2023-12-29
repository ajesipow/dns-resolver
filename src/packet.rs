use crate::core::{TYPE_A, TYPE_NS};
use crate::header::{parse_header, DNSHeader};
use crate::question::{parse_question, DNSQuestion};
use crate::record::{parse_record, DNSRecord};
use anyhow::Result;
use std::io::Cursor;

#[derive(Debug)]
pub(crate) struct DNSPacket {
    pub _header: DNSHeader,
    pub _questions: Vec<DNSQuestion>,
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub additionals: Vec<DNSRecord>,
}

impl DNSPacket {
    pub(crate) fn parse(value: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(value);
        let header = parse_header(&mut cursor)?;
        let questions: Vec<DNSQuestion> = (0..header.num_questions)
            .map(|_| parse_question(&mut cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let answers: Vec<DNSRecord> = (0..header.num_answers)
            .map(|_| parse_record(&mut cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let authorities: Vec<DNSRecord> = (0..header.num_authorities)
            .map(|_| parse_record(&mut cursor))
            .collect::<Result<Vec<_>, _>>()?;
        let additionals: Vec<DNSRecord> = (0..header.num_additionals)
            .map(|_| parse_record(&mut cursor))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            _header: header,
            _questions: questions,
            answers,
            authorities,
            additionals,
        })
    }

    pub(crate) fn get_answer(&self) -> Option<&[u8]> {
        for answer in self.answers.iter() {
            if answer.type_ == TYPE_A {
                return Some(&answer.data);
            }
        }
        None
    }

    pub(crate) fn get_nameserver_ip(&self) -> Option<&[u8]> {
        for additional in self.additionals.iter() {
            if additional.type_ == TYPE_A {
                return Some(&additional.data);
            }
        }
        None
    }

    pub(crate) fn get_nameserver(&self) -> Result<Option<String>> {
        for authority in self.authorities.iter() {
            if authority.type_ == TYPE_NS {
                let nameserver = String::from_utf8(authority.data.clone())?;
                return Ok(Some(nameserver));
            }
        }
        Ok(None)
    }
}
