use crate::header::{parse_header, DNSHeader};
use crate::question::{parse_question, DNSQuestion};
use crate::record::{parse_record, DNSRecord};
use anyhow::Result;
use std::io::Cursor;

#[derive(Debug)]
pub(crate) struct DNSPacket {
    pub header: DNSHeader,
    pub questions: Vec<DNSQuestion>,
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub additionals: Vec<DNSRecord>,
}

pub(crate) fn parse_dns_packet(value: &[u8]) -> Result<DNSPacket> {
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
    Ok(DNSPacket {
        header,
        questions,
        answers,
        authorities,
        additionals,
    })
}
