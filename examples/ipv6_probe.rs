//! Development-only prefix-math probe; no sockets, host commands or learner execution.
use dlms_learning_lab::ipv6::Prefix;
use serde::Deserialize;
use std::{io::Read, net::Ipv6Addr};
#[derive(Deserialize)]
struct Input {
    address: Ipv6Addr,
    prefix: u8,
}
fn main() {
    let mut input = String::new();
    std::io::stdin()
        .take(300_000)
        .read_to_string(&mut input)
        .unwrap();
    let values: Vec<Input> = serde_json::from_str(&input).unwrap();
    assert!(values.len() <= 1024);
    let results: Vec<_> = values
        .iter()
        .map(|i| Prefix::new(i.address, i.prefix).map(|p| p.text()))
        .collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
