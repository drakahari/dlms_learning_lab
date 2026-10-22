//! Development-only JSON oracle interface; never linked into the application UI.
use dlms_learning_lab::shell;
use serde::Deserialize;
use std::io::Read;
#[derive(Deserialize)]
struct Probe {
    source: String,
    case: shell::Case,
}
fn main() {
    let mut input = String::new();
    std::io::stdin()
        .take(300_000)
        .read_to_string(&mut input)
        .unwrap();
    let p: Probe = serde_json::from_str(&input).unwrap();
    println!(
        "{}",
        serde_json::to_string(&shell::run(&p.source, &p.case)).unwrap()
    );
}
