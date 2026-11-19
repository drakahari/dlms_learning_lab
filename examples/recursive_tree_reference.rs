//! Fixed developer observations, no host files, commands or runtime input.
use dlms_learning_lab::{
    developer_digest,
    engine::Node,
    filesystem::{Tree, recursive as core},
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[path = "support/recursive_tree_bindings.rs"]
mod bindings;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    directory: bool,
    mode: u16,
    #[serde(default)]
    text: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    mask: u16,
    program: String,
    args: Vec<String>,
    modes: BTreeMap<String, u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Excluded {
    id: String,
    command: String,
    setup: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    entries: Vec<Entry>,
    cases: Vec<Case>,
    excluded: Vec<Excluded>,
}
fn world(f: &Fixture, mask: u16, modes: &BTreeMap<String, u16>) -> Tree {
    let mut nodes = BTreeMap::from([
        (
            "/".into(),
            Node {
                directory: true,
                mode: 0o755,
                owner: "root".into(),
                group: "root".into(),
            },
        ),
        (
            "/work".into(),
            Node {
                directory: true,
                mode: 0o755,
                owner: "alex".into(),
                group: "staff".into(),
            },
        ),
    ]);
    let mut files = BTreeMap::new();
    for e in &f.entries {
        let p = format!("/work/{}", e.path);
        nodes.insert(
            p.clone(),
            Node {
                directory: e.directory,
                mode: modes.get(&e.path).copied().unwrap_or(e.mode),
                owner: "alex".into(),
                group: "staff".into(),
            },
        );
        if let Some(t) = &e.text {
            files.insert(p, t.clone());
        }
    }
    let tree = Tree {
        nodes,
        files,
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["staff".into()]),
        ]),
        user: "alex".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: mask,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    assert!(tree.valid());
    tree
}
fn view(t: &Tree) -> BTreeMap<String, serde_json::Value> {
    t.nodes.iter().filter(|(p,_)|p.starts_with("/work/")).map(|(p,n)|(p.trim_start_matches("/work/").into(),serde_json::json!({"directory":n.directory,"mode":n.mode,"caller_owned":n.owner=="alex","caller_group":n.group=="staff","text":t.files.get(p)}))).collect()
}
fn main() {
    assert_eq!(
        std::env::args().len(),
        1,
        "Fixed probe takes no input selector"
    );
    let f: Fixture = serde_json::from_str(include_str!(
        "../tests/fixtures/recursive-tree-reference-v1.json"
    ))
    .unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!((f.cases.len(), f.excluded.len()), (8, 3));
    let mut rows = Vec::new();
    for c in &f.cases {
        let mut t = world(&f, c.mask, &c.modes);
        assert!(matches!(c.program.as_str(), "cp" | "rm"));
        let command = std::iter::once(&c.program)
            .chain(c.args.iter())
            .map(|a| {
                assert!(!a.contains('\''));
                format!("'{a}'")
            })
            .collect::<Vec<_>>()
            .join(" ");
        let outcome = core::execute(&mut t, &command);
        rows.push(serde_json::json!({"id":c.id,"ok":outcome.is_ok(),"diagnostic":outcome.err().map(|e|e.to_string()),"view":view(&t)}));
    }
    let mut exclusions = Vec::new();
    for e in &f.excluded {
        let mut t = world(&f, 0o022, &BTreeMap::new());
        match e.setup.as_str() {
            "none" => {}
            "copy-existing" => {
                core::execute(&mut t, "cp -R src bucket").unwrap();
            }
            "unreadable-file" => {
                t.nodes.get_mut("/work/src/readme").unwrap().mode = 0;
            }
            _ => panic!("fixed setup"),
        };
        let before = t.clone();
        let error = core::execute(&mut t, &e.command).unwrap_err();
        assert_eq!(before, t);
        exclusions.push(serde_json::json!({"id":e.id,"classification":"excluded","reason":e.reason,"diagnostic":error.to_string()}));
    }
    let inputs=bindings::INPUTS.iter().map(|(name,data)|serde_json::json!({"path":name,"bytes":data.len(),"sha256":developer_digest::hash(data)})).collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::json!({"purpose":"fixed-recursive-tree-v1","compiled_inputs":inputs,"rows":rows,"excluded":exclusions})
    );
}
