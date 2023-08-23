use std::net::Ipv4Addr;

use clap::Parser;

pub const REQUEST_DELAY: u64 = 2_000;
pub const CRAWL_DELAY: u64 = 3_600_000;

#[derive(Parser)]
pub struct Args {
    #[arg(long, short, default_value_t = Ipv4Addr::new(127, 0, 0, 1))]
    pub ip: Ipv4Addr,
    #[arg(long, short, default_value_t = 9000)]
    pub port: u16,
    #[arg(long, short, default_value_t = CRAWL_DELAY)]
    pub crawl_delay: u64,
    #[arg(long, short, default_value_t = REQUEST_DELAY)]
    pub request_delay: u64,
}
