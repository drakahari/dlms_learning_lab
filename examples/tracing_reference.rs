//! Fixed in-process tracing observations; never executes a host script.
#[path = "support/automation_digest.rs"]
mod digest;
use dlms_learning_lab::shell::{self, Case};
use serde_json::{Value, json};
use std::collections::BTreeMap;
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/shell-tracing-reference.json");
const FORMAT: &[u8] = include_bytes!("../tests/fixtures/shell-tracing-format-reference.json");
const COMPILED: &[(&str, &[u8])] = &[
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    (
        "examples/support/automation_digest.rs",
        include_bytes!("../examples/support/automation_digest.rs"),
    ),
    (
        "examples/tracing_reference.rs",
        include_bytes!("../examples/tracing_reference.rs"),
    ),
    ("src/shell.rs", include_bytes!("../src/shell.rs")),
    (
        "src/shell/arithmetic.rs",
        include_bytes!("../src/shell/arithmetic.rs"),
    ),
    (
        "src/shell/capture.rs",
        include_bytes!("../src/shell/capture.rs"),
    ),
    ("src/shell/case.rs", include_bytes!("../src/shell/case.rs")),
    ("src/shell/cron.rs", include_bytes!("../src/shell/cron.rs")),
    (
        "src/shell/execution.rs",
        include_bytes!("../src/shell/execution.rs"),
    ),
    (
        "src/shell/getopts.rs",
        include_bytes!("../src/shell/getopts.rs"),
    ),
    (
        "src/shell/globbing.rs",
        include_bytes!("../src/shell/globbing.rs"),
    ),
    ("src/shell/grep.rs", include_bytes!("../src/shell/grep.rs")),
    (
        "src/shell/literal_trim.rs",
        include_bytes!("../src/shell/literal_trim.rs"),
    ),
    (
        "src/shell/loop_control.rs",
        include_bytes!("../src/shell/loop_control.rs"),
    ),
    (
        "src/shell/one_off.rs",
        include_bytes!("../src/shell/one_off.rs"),
    ),
    (
        "src/shell/parameters.rs",
        include_bytes!("../src/shell/parameters.rs"),
    ),
    (
        "src/shell/preparation.rs",
        include_bytes!("../src/shell/preparation.rs"),
    ),
    (
        "src/shell/substitution.rs",
        include_bytes!("../src/shell/substitution.rs"),
    ),
    (
        "src/shell/text_tools.rs",
        include_bytes!("../src/shell/text_tools.rs"),
    ),
    (
        "src/shell/tracing.rs",
        include_bytes!("../src/shell/tracing.rs"),
    ),
    (
        "src/shell/values.rs",
        include_bytes!("../src/shell/values.rs"),
    ),
    (
        "tests/fixtures/shell-tracing-edge-reference.json",
        include_bytes!("../tests/fixtures/shell-tracing-edge-reference.json"),
    ),
    (
        "tests/fixtures/shell-tracing-format-reference.json",
        include_bytes!("../tests/fixtures/shell-tracing-format-reference.json"),
    ),
    (
        "tests/fixtures/shell-tracing-reference.json",
        include_bytes!("../tests/fixtures/shell-tracing-reference.json"),
    ),
];
const EDGES: &[u8] = include_bytes!("../tests/fixtures/shell-tracing-edge-reference.json");
fn main() {
    assert_eq!(std::env::args_os().len(), 1, "No source/path/engine CLI");
    let data: Value = serde_json::from_slice(FIXTURE).unwrap();
    let formatting: Value = serde_json::from_slice(FORMAT).unwrap();
    let edges: Value = serde_json::from_slice(EDGES).unwrap();
    let rows = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(formatting["cases"].as_array().unwrap())
        .chain(edges["cases"].as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 36);
    let observations = rows
        .iter()
        .map(|row| {
            let c = Case {
                name: row["id"].as_str().unwrap().into(),
                args: vec![],
                stdin: String::new(),
                env: BTreeMap::new(),
                files: BTreeMap::new(),
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                after: BTreeMap::new(),
            };
            let outcome = shell::run(row["source"].as_str().unwrap(), &c);
            let mut v = serde_json::to_value(outcome).unwrap();
            v.as_object_mut()
                .unwrap()
                .insert("id".into(), row["id"].clone());
            v
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        json!({"evaluator_revision":shell::EVALUATOR_REVISION,"fixture_sha256":digest::hash(FIXTURE),"format_sha256":digest::hash(FORMAT),"edge_sha256":digest::hash(EDGES),"compiled_inputs":COMPILED.iter().map(|(name,data)|json!({"path":name,"bytes":data.len(),"sha256":digest::hash(data)})).collect::<Vec<_>>(),"observations":observations})
    );
}
