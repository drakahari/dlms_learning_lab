//! Fixed startup observations. No host filesystem/process/network operations.
#[path = "support/automation_digest.rs"]
mod digest;
use dlms_learning_lab::shell::{self, Case, startup};
use serde_json::{Value, json};
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/shell-startup-reference.json");
const COMPILED: &[(&str, &[u8])] = &[
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    (
        "examples/startup_reference.rs",
        include_bytes!("../examples/startup_reference.rs"),
    ),
    (
        "examples/support/automation_digest.rs",
        include_bytes!("../examples/support/automation_digest.rs"),
    ),
    ("src/cron.rs", include_bytes!("../src/cron.rs")),
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
        "src/shell/startup.rs",
        include_bytes!("../src/shell/startup.rs"),
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
        "tests/fixtures/shell-startup-reference.json",
        include_bytes!("../tests/fixtures/shell-startup-reference.json"),
    ),
];
fn main() {
    assert_eq!(shell::EVALUATOR_REVISION, 17);
    let fixture: Value = serde_json::from_slice(FIXTURE).unwrap();
    let mut observations = vec![];
    for row in fixture["cases"].as_array().unwrap() {
        let case = Case {
            name: row["id"].as_str().unwrap().into(),
            args: vec![],
            stdin: String::new(),
            env: serde_json::from_value(row["env"].clone()).unwrap(),
            files: std::collections::BTreeMap::from([(
                "/work/keep.txt".into(),
                "Retained fixture bytes.\n".into(),
            )]),
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
            after: std::collections::BTreeMap::new(),
        };
        let mode = serde_json::from_value(row["invocation"].clone()).unwrap();
        let files = serde_json::from_value(row["files"].clone()).unwrap();
        let mut run =
            startup::Run::new(&mode, &files, &case, row["command"].as_str().unwrap()).unwrap();
        for _ in 0..1000 {
            run.tick();
            if run.done() {
                break;
            }
        }
        assert!(run.done());
        let outcome = run.outcome().unwrap();
        assert!(outcome.error.is_none(), "{:?}", outcome.error);
        let snapshot = run.environment().unwrap();
        let variables: serde_json::Map<String, Value> = row["observe"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                let name = n.as_str().unwrap();
                (
                    name.into(),
                    snapshot
                        .parent
                        .get(name)
                        .map_or(json!({"value":null,"exported":false}), |v| {
                            serde_json::to_value(v).unwrap()
                        }),
                )
            })
            .collect();
        observations.push(json!({"id":row["id"],"selection":run.selected().iter().map(|s|s.path.clone()).collect::<Vec<_>>(),"stdout":outcome.stdout,"stderr":outcome.stderr,"status":outcome.status,"files":outcome.files,"error":outcome.error,"variables":variables}));
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
