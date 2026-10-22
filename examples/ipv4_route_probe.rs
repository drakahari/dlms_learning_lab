//! Development-only IPv4 prefix arithmetic; never emits packets or learner commands.
use dlms_learning_lab::ipv4_routing::Prefix;
use serde::Deserialize;
use std::{io::Read, net::Ipv4Addr};
#[derive(Deserialize)]
struct Input {
    address: Ipv4Addr,
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
    let results:Vec<_>=values.iter().map(|v|Prefix::new(v.address,v.prefix).map(|p|serde_json::json!({"network":p.text(),"last":p.last_address().unwrap().to_string()}))).collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
