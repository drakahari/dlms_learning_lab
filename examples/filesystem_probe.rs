//! Development-only controlled-fixture interface, never used by the native application.
use dlms_learning_lab::filesystem::Tree;
use serde::Deserialize;
use std::io::Read;
#[derive(Deserialize)]
struct Probe {
    tree: Tree,
    commands: Vec<String>,
}
fn main() {
    let mut input = String::new();
    std::io::stdin()
        .take(600_001)
        .read_to_string(&mut input)
        .unwrap();
    assert!(
        input.len() <= 600_000,
        "Fixed development probe input exceeds600000bytes"
    );
    let mut probe: Probe = serde_json::from_str(&input).unwrap();
    assert!(probe.tree.valid());
    let steps: Vec<_> = probe.commands.iter().map(|command| {
        let outcome = probe.tree.execute(command);
        let error = outcome.as_ref().err().cloned();
        serde_json::json!({"ok": outcome.is_ok(), "output": outcome.unwrap_or_default(), "error": error, "tree": probe.tree})
    }).collect();
    println!("{}", serde_json::to_string(&steps).unwrap());
}
