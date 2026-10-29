//! Fixed development-only ERE observation. No input/selector, I/O or app engine.
#[path = "support/automation_digest.rs"]
mod digest;
#[path = "../src/ere.rs"]
#[allow(dead_code)]
mod ere;
use serde_json::{Value, json};
const FIXED: &[u8] = include_bytes!("../tests/fixtures/ere-fixed-reference.json");
const FIXED_SHA: &str = "c5b0fd3e58698846ab174f6f6ed1d0309c4e164702465a3bfbc5efa9b5ad888b";
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert!(
        args.is_empty() || args == ["--identity"],
        "Only fixed cases or --identity; no patterns, source, paths or stdin"
    );
    assert_eq!(digest::hash(FIXED), FIXED_SHA);
    let fixed: Value = serde_json::from_slice(FIXED).unwrap();
    let rows = fixed["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 42);
    assert!(fixed["actual_comparison"].is_null());
    let identity = json!({
        "kind":"fixed-ere-reference-r1", "manifest_dir":env!("CARGO_MANIFEST_DIR"),
        "fixture_sha256":FIXED_SHA, "cases":42, "host_cases":34, "VM_only":8,
        "compiled_sources": {
            "src/ere.rs":digest::hash(include_bytes!("../src/ere.rs")),
            "examples/ere_reference.rs":digest::hash(include_bytes!("ere_reference.rs")),
            "examples/support/automation_digest.rs":digest::hash(include_bytes!("support/automation_digest.rs")),
            "tests/fixtures/ere-fixed-reference.json":FIXED_SHA,
            "Cargo.toml":digest::hash(include_bytes!("../Cargo.toml")),
            "Cargo.lock":digest::hash(include_bytes!("../Cargo.lock"))
        }
    });
    if !args.is_empty() {
        println!("{identity}");
        return;
    }
    let mut actual = Vec::new();
    for row in rows {
        assert!(row["vm_observation"].is_null() && row["gnu_observation"].is_null());
        let pattern = row["pattern"].as_str().unwrap();
        let line = row["line"].as_str().unwrap();
        let result = match ere::Regex::compile(pattern) {
            Err(e) => Err(e),
            Ok(re) => match re.matcher() {
                Err(e) => Err(e),
                Ok(mut matcher) => matcher.matches_line(line, &mut ere::Budget::default()),
            },
        };
        let (matched, error_kind, diagnostic) = match result {
            Ok(value) => (Some(value), None, None),
            Err(e) => (None, Some(format!("{:?}", e.kind)), Some(e.to_string())),
        };
        actual.push(json!({"id":row["id"],"partition":row["partition"],
            "pattern_sha256":digest::hash(pattern.as_bytes()),"line_sha256":digest::hash(line.as_bytes()),
            "matched":matched,"error_kind":error_kind,"diagnostic":diagnostic}));
    }
    println!("{}", json!({"identity":identity,"outcomes":actual}));
}
