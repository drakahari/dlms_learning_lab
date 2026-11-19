//! Fixed in-process probe, never a host/reference executor.
use dlms_learning_lab::shell::{self, Case};
use serde_json::{Value, json};
mod digest {
    include!("support/automation_digest.rs");
}
include!("support/associative_inputs.rs");
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/shell-associative-reference.json");
fn main() {
    assert_eq!(std::env::args_os().len(), 1, "No source/path/case selector");
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 16);
    let outcomes = rows
        .iter()
        .map(|r| {
            let files = r["files"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(p, s)| (format!("/work/{p}"), s.as_str().unwrap().to_owned()))
                .collect();
            let case = Case {
                name: r["id"].as_str().unwrap().into(),
                args: vec![],
                stdin: String::new(),
                env: Default::default(),
                files,
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                after: Default::default(),
            };
            json!({"id":r["id"],"outcome":shell::run(r["source"].as_str().unwrap(),&case)})
        })
        .collect::<Vec<_>>();
    let mut pins = COMPILED
        .iter()
        .map(|(n, b)| json!({"path":n,"bytes":b.len(),"sha256":digest::hash(b)}))
        .collect::<Vec<_>>();
    for (n, b) in [
        (
            "examples/associative_reference.rs",
            include_bytes!("associative_reference.rs").as_slice(),
        ),
        (
            "examples/support/associative_inputs.rs",
            include_bytes!("support/associative_inputs.rs").as_slice(),
        ),
        ("tests/fixtures/shell-associative-reference.json", FIXTURE),
    ] {
        pins.push(json!({"path":n,"bytes":b.len(),"sha256":digest::hash(b)}));
    }
    println!(
        "{}",
        json!({"evaluator":shell::EVALUATOR_REVISION,"fixture_sha256":digest::hash(FIXTURE),"source_bindings":pins,"outcomes":outcomes})
    );
}
