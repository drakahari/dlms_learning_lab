//! Fixed developer comparison probe. No script/path/engine selector or host execution.
use dlms_learning_lab::shell::{self, Case, Vm};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
const FIXTURE: &str = include_str!("../tests/fixtures/shell-loop-control-fixed.json");
#[derive(Deserialize)]
struct Fixed {
    supported_cases: usize,
    honest_exclusions: usize,
    cases: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    id: String,
    partition: String,
    source: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    stdin: String,
    files: BTreeMap<String, String>,
    vm_observation: Option<Value>,
    bash_observation: Option<Value>,
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "Fixed probe takes no source/path arguments"
    );
    assert_eq!(shell::EVALUATOR_REVISION, 13);
    let f: Fixed = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(
        (f.supported_cases, f.honest_exclusions, f.cases.len()),
        (16, 8, 24)
    );
    let mut out = Vec::new();
    for r in f.cases {
        assert!(r.vm_observation.is_none() && r.bash_observation.is_none());
        let c = Case {
            name: r.id.clone(),
            args: r.args,
            env: r.env,
            stdin: r.stdin,
            files: r.files,
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: BTreeMap::new(),
        };
        let mut vm = Vm::new(&r.source, &c).unwrap();
        for _ in 0..100_000 {
            if vm.done() {
                break;
            }
            vm.tick(7)
        }
        assert!(vm.done());
        out.push(json!({"id":r.id,"partition":r.partition,"outcome":vm.outcome}));
    }
    println!(
        "{}",
        json!({"fixture_id":"loop-control24-r1","engine_revision":shell::EVALUATOR_REVISION,"fixture_utf8":FIXTURE,"outcomes":out})
    );
}
