//! Fixed in-process export observations. No host process or filesystem operation.
#[path = "support/automation_digest.rs"]
mod digest;
use dlms_learning_lab::shell::{self, Case};
use serde_json::{Value, json};
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/shell-environment-reference.json");
const COMPILED: &[(&str, &[u8])] = &[
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    (
        "examples/environment_reference.rs",
        include_bytes!("../examples/environment_reference.rs"),
    ),
    (
        "examples/support/automation_digest.rs",
        include_bytes!("../examples/support/automation_digest.rs"),
    ),
    ("src/ere.rs", include_bytes!("../src/ere.rs")),
    (
        "src/ere/cooperative.rs",
        include_bytes!("../src/ere/cooperative.rs"),
    ),
    ("src/lib.rs", include_bytes!("../src/lib.rs")),
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
        "src/shell/environment.rs",
        include_bytes!("../src/shell/environment.rs"),
    ),
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
        "tests/fixtures/shell-environment-reference.json",
        include_bytes!("../tests/fixtures/shell-environment-reference.json"),
    ),
];

fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No caller source/path/engine selector"
    );
    assert_eq!(shell::EVALUATOR_REVISION, 17);
    let fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    let mut observations = Vec::new();
    for row in fixture["cases"].as_array().unwrap() {
        let files = serde_json::from_value(row["files"].clone()).unwrap();
        let case = Case {
            name: row["id"].as_str().unwrap().into(),
            args: serde_json::from_value(row["args"].clone()).unwrap(),
            stdin: row["stdin"].as_str().unwrap().into(),
            env: serde_json::from_value(row["env"].clone()).unwrap(),
            files,
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: serde_json::from_value(row["files"].clone()).unwrap(),
        };
        let mut vm = shell::Vm::new(row["source"].as_str().unwrap(), &case).unwrap();
        while !vm.done() {
            vm.tick(64);
        }
        let snapshot = serde_json::to_value(
            vm.environment_snapshot()
                .expect("Supported fixed global export program"),
        )
        .unwrap();
        let variables: serde_json::Map<String, Value> = row["observe"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                let name = n.as_str().unwrap();
                (
                    name.to_owned(),
                    snapshot["parent"]
                        .get(name)
                        .cloned()
                        .unwrap_or(json!({"value":null,"exported":false})),
                )
            })
            .collect();
        let r = vm.outcome;
        observations.push(json!({"id":row["id"],"stdout":r.stdout,"stderr":r.stderr,"status":r.status,"files":r.files,"error":r.error,"variables":variables}));
    }
    let pins: Vec<_> = COMPILED
        .iter()
        .map(|(name, raw)| json!({"path":name,"bytes":raw.len(),"sha256":digest::hash(raw)}))
        .collect();
    println!(
        "{}",
        json!({"evaluator_revision":shell::EVALUATOR_REVISION,"fixture_sha256":digest::hash(FIXTURE),"compiled_inputs":pins,"observations":observations})
    );
}
