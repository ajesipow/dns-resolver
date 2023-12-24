use crate::core::lookup_domain;

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
        let ip = lookup_domain(domain);
        println!("{domain}: {ip}");
    }
}
