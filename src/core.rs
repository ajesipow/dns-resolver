use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use crate::header::DNSHeader;
use crate::question::DNSQuestion;

const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;

pub(crate) trait ToBytes {
    fn to_bytes(self) -> Vec<u8>;
}

pub(crate) fn try_encode_dns_name(str: &str) -> Result<Vec<u8>, String> {
    let parts = str.split('.').map(try_encode_part).collect::<Result<Vec<_>, _>>()?;
    let mut encoded = parts.into_iter().flatten().collect::<Vec<u8>>();
    encoded.push(0);
    Ok(encoded)
}

fn try_encode_part(part: &str) -> Result<Vec<u8>, String> {
    let part_len = part.len();
    if part_len > u8::MAX as usize {
        return Err("part too long".to_string())
    }
    // We encode the part itself plus its length
    let mut v = Vec::with_capacity(part_len + 1);
    v.push(part_len as u8);
    v.extend_from_slice(part.as_bytes());
    Ok(v)
}

fn build_query(domain_name: &str, record_type: u16) -> Result<Vec<u8>, String> {
    let encoded_domain_name = try_encode_dns_name(domain_name)?;
    let question = DNSQuestion::new(encoded_domain_name, CLASS_IN, record_type);
    let id = SmallRng::seed_from_u64(42).gen();
    let RECURSION_DESIRED = 1 << 8;
    let header = DNSHeader::default().with_id(id).with_flags(RECURSION_DESIRED).with_num_questions(1);
    let mut query = header.to_bytes();
    query.extend(question.to_bytes());
    Ok(query)
}