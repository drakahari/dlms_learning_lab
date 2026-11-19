//! Fixed in-process diagnostic observations only. No host I/O or artifact execution.
#[path = "support/command_location_bindings.rs"]
mod bindings;
#[path = "support/automation_digest.rs"]
mod digest;
use dlms_learning_lab::{
    command_location::{self, Context, Mode},
    engine::Node,
    filesystem::Tree,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/command-location-reference-v1.json");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    preferred_mode: u16,
    fallback_mode: u16,
    preferred_dir_mode: u16,
    fallback_dir_mode: u16,
    preferred_directory: bool,
    reverse_path: bool,
    preload_hash: bool,
}
fn node(directory: bool, mode: u16) -> Node {
    Node {
        directory,
        mode,
        owner: "alex".into(),
        group: "staff".into(),
    }
}
fn tree(case: &Case) -> Tree {
    let mut nodes: BTreeMap<String, Node> = BTreeMap::from([
        ("/".into(), node(true, 0o700)),
        ("/work".into(), node(true, 0o700)),
        ("/preferred".into(), node(true, case.preferred_dir_mode)),
        ("/fallback".into(), node(true, case.fallback_dir_mode)),
        (
            "/preferred/report".into(),
            node(case.preferred_directory, case.preferred_mode),
        ),
        ("/fallback/report".into(), node(false, case.fallback_mode)),
    ]);
    let mut files: BTreeMap<String, String> = BTreeMap::from([(
        "/fallback/report".into(),
        "Inert fallback artifact\n".into(),
    )]);
    if !case.preferred_directory {
        files.insert(
            "/preferred/report".into(),
            "Inert preferred artifact\n".into(),
        );
    }
    // Deserialize the normal wire format, including default None for future optional fields.
    let tree: Tree =
        serde_json::from_value(json!({"nodes":std::mem::take(&mut nodes),"files":files,
        "users":{"alex":["staff"]},"user":"alex","home":"/work","cwd":"/work",
        "previous":null,"umask":18,"clock":0,"modified":{}}))
        .unwrap();
    assert!(tree.valid());
    tree
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "Fixed probe has no source, file or case selector"
    );
    let fixture: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.cases.len(), 9);
    let mut ids = BTreeSet::new();
    let mut observations = Vec::new();
    for case in &fixture.cases {
        assert!(case.id.len() <= 48 && ids.insert(&case.id));
        for mode in [
            case.preferred_mode,
            case.fallback_mode,
            case.preferred_dir_mode,
            case.fallback_dir_mode,
        ] {
            assert!([0, 0o100, 0o600, 0o700].contains(&mode));
        }
        let tree = tree(case);
        let held = Context {
            version: 1,
            source_id: format!("reference-{}", case.id),
            path: if case.reverse_path {
                "/fallback:/preferred"
            } else {
                "/preferred:/fallback"
            }
            .into(),
            names: vec!["report".into()],
        };
        for (mode, input) in [
            (Mode::Which, "which report"),
            (Mode::TypeAllPaths, "type -a -p report"),
        ] {
            let result =
                command_location::execute_bound(&held, Some(&held), &tree, &tree, input).unwrap();
            observations.push(json!({"id":case.id,"mode":mode,"status":result.status,"stdout":result.stdout,
                "paths":result.paths,"diagnostic":result.diagnostic,"host_preload_hash":case.preload_hash&&mode==Mode::TypeAllPaths}));
        }
    }
    let pins: Vec<_> = bindings::COMPILED
        .iter()
        .map(|(name, raw)| json!({"path":name,"bytes":raw.len(),"sha256":digest::hash(raw)}))
        .collect();
    println!(
        "{}",
        json!({"purpose":"fixed-command-location-reference-v1","compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),
        "fixture_sha256":digest::hash(FIXTURE),"compiled_inputs":pins,"observations":observations})
    );
}
