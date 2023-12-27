use crate::core::{resolve, TYPE_A};

mod core;
mod header;
mod packet;
mod question;
mod record;

fn main() {
    let domains = [
        "www.example.com",
        "recurse.com",
        "metafilter.com",
        "www.facebook.com",
        "www.metafilter.com",
    ];
    for domain in domains {
        let response = resolve(domain, TYPE_A).unwrap();
        println!("{} is at {}", domain, response);
    }
}
