use crate::core::{resolve, TYPE_A};
use clap::Parser;

mod core;
mod header;
mod packet;
mod question;
mod record;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The domain name to lookup
    #[arg(short, long)]
    name: String,

    // The record type
    #[arg(short, long, default_value_t = 1)]
    record_type: u16,
}

fn main() {
    let args = Args::parse();
    let response = resolve(&args.name, args.record_type).unwrap();
    println!("{} is at {}", args.name, response);
}
