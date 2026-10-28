//! Fixed development observation probe; no host commands or arbitrary-source CLI.
use dlms_learning_lab::shell;
use serde_json::{Value, json};
use std::collections::BTreeSet;
#[path = "support/automation_digest.rs"]
mod digest;
const FIXED: &[u8] = include_bytes!("../tests/fixtures/shell-optional-inputs-reference117.json");
const SHA: &str = "e27b80313df523058f9c8c12d2a02c39770564e0ffc21b57e226a72d8480a584";
const EXCLUDED: [&str; 3] = [
    "shell-optional-report/reference/guard00",
    "shell-optional-report/alternate1/guard00",
    "shell-optional-report/alternate2/guard00",
];
fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(values) => Value::Object(
            values
                .iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .map(|(key, value)| (key.clone(), sorted(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
        _ => value.clone(),
    }
}
fn main() {
    assert_eq!(shell::EVALUATOR_REVISION, 11);
    assert_eq!(digest::hash(FIXED), SHA);
    let fixed: Value = serde_json::from_slice(FIXED).unwrap();
    let original = fixed["rows"].as_array().unwrap();
    assert_eq!(original.len(), 117);
    assert_eq!(
        original
            .iter()
            .map(|r| r["source"].as_str().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        15
    );
    let rows = original
        .iter()
        .filter(|r| !EXCLUDED.contains(&r["id"].as_str().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 114);
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--identity"] {
        println!(
            "{}",
            json!({"kind":"fixed-optional-inputs-reference","engine_revision":11,
            "original_rows":117,"admitted_rows":114,"unique_programs":15,"original117_sha256":SHA})
        );
        return;
    }
    assert!(
        args.is_empty(),
        "Only the fixed probe or --identity is supported"
    );
    for row in rows {
        let source = row["source"].as_str().unwrap();
        let case: shell::Case = serde_json::from_value(row["case"].clone()).unwrap();
        assert!(case.valid());
        // Sort explicitly, even if a future transitive dependency enables
        // serde_json preserve_order. UTF-8 JSON uses the fixed recipe values.
        let case_json = serde_json::to_vec(&sorted(&row["case"])).unwrap();
        println!(
            "{}",
            json!({"id":row["id"],"source_sha256":digest::hash(source.as_bytes()),
            "case_sha256":digest::hash(&case_json),"outcome":shell::run(source,&case)})
        );
    }
}
