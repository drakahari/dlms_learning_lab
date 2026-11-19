//! Fixed developer queries only, entirely in-process; no host files or commands.
use dlms_learning_lab::{
    developer_digest,
    engine::Node,
    filesystem::{Tree, size_predicate},
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[path = "support/find_size_bindings.rs"]
mod bindings;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    args: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exclusion {
    id: String,
    command: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    files: BTreeMap<String, String>,
    cases: Vec<Query>,
    excluded: Vec<Exclusion>,
}
fn main() {
    assert_eq!(
        std::env::args().len(),
        1,
        "No runtime source, path or query selector"
    );
    let f: Fixture = serde_json::from_str(include_str!(
        "../tests/fixtures/find-size-reference-v1.json"
    ))
    .unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.cases.len(), 31);
    assert_eq!(f.excluded.len(), 2);
    let dir = Node {
        directory: true,
        mode: 0o755,
        owner: "alex".into(),
        group: "staff".into(),
    };
    let mut t = Tree {
        nodes: BTreeMap::from([("/".into(), dir.clone()), ("/work".into(), dir)]),
        files: BTreeMap::new(),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
        ]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    for (name, text) in &f.files {
        let p = format!("/work/{name}");
        t.nodes.insert(
            p.clone(),
            Node {
                directory: false,
                mode: 0o600,
                owner: "alex".into(),
                group: "staff".into(),
            },
        );
        t.files.insert(p, text.clone());
    }
    assert!(t.valid());
    let before = t.clone();
    let mut rows = Vec::new();
    for q in &f.cases {
        assert!(q.args.iter().all(|a| !a.contains('\'')));
        let command = std::iter::once("find")
            .chain(q.args.iter().map(String::as_str))
            .map(|s| format!("'{s}'"))
            .collect::<Vec<_>>()
            .join(" ");
        let actual = size_predicate::execute(&mut t, &command);
        assert_eq!(t, before);
        rows.push(serde_json::json!({"id":q.id,"ok":actual.is_ok(),"stdout":actual.as_ref().ok(),"error":actual.as_ref().err()}));
    }
    let mut excluded = Vec::new();
    for q in &f.excluded {
        let error = size_predicate::execute(&mut t, &q.command).unwrap_err();
        assert_eq!(t, before);
        excluded.push(serde_json::json!({"id":q.id,"classification":"excluded","reason":q.reason,"error":error}));
    }
    let pins=bindings::INPUTS.iter().map(|(p,b)|serde_json::json!({"path":p,"bytes":b.len(),"sha256":developer_digest::hash(b)})).collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::json!({"purpose":"fixed-file-size-v1","compiled_inputs":pins,"rows":rows,"excluded":excluded})
    );
}
