//! Fixed owned-directory chmod mode comparison; no host utility execution.
#[path = "support/sticky_mode_bindings.rs"]
mod bindings;
#[path = "support/sticky_mode_digest.rs"]
mod digest;
use dlms_learning_lab::{
    engine::Node,
    filesystem::{
        Tree,
        directory_policy::{CoreWorld, ErrorKind},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/sticky-mode-reference-v1.json");
#[derive(Serialize)]
struct Pin<'a> {
    bytes: usize,
    path: &'a str,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    initial_mode: u16,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    classification: String,
    spec: String,
}
const SPECS: [(&str, &str); 17] = [
    ("numeric-clear", "0777"),
    ("numeric-sticky", "01777"),
    ("add-sticky", "+t"),
    ("remove-sticky", "-t"),
    ("equals-all", "a=rwx"),
    ("equals-other", "o=rx"),
    ("equals-empty-other", "o="),
    ("equals-user", "u=rw"),
    ("equals-group", "g=rx"),
    ("equals-ugo", "ugo=rw"),
    ("mixed-user-group", "u=rw,g=rx"),
    ("mixed-other-middle", "u=rw,o=rx,g=rx"),
    ("mixed-other-first", "o=,u=rwx,g=rw"),
    ("mixed-all-adjust", "a=rw,u+x,o-r"),
    ("mixed-plus-minus", "u+x,g-w,o+r"),
    ("excluded-o-plus-t", "o+t"),
    ("excluded-setuid", "4755"),
];
fn world() -> CoreWorld {
    let node = |mode, owner: &str| Node {
        mode,
        owner: owner.into(),
        group: owner.into(),
        directory: true,
    };
    let tree = Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(0o755, "root")),
            ("/subject".into(), node(0o770, "alex")),
        ]),
        files: BTreeMap::new(),
        users: BTreeMap::from([
            ("root".into(), vec!["root".into()]),
            ("alex".into(), vec!["alex".into()]),
        ]),
        user: "alex".into(),
        home: "/subject".into(),
        cwd: "/".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    CoreWorld::new(tree, BTreeSet::from(["/subject".into()])).unwrap()
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "Fixed compiled fixture only; no selectors"
    );
    let fixture: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.initial_mode, 0o1770);
    assert_eq!(fixture.cases.len(), 17);
    let mut results = Vec::new();
    for (i, case) in fixture.cases.iter().enumerate() {
        assert_eq!((case.id.as_str(), case.spec.as_str()), SPECS[i]);
        assert_eq!(
            case.classification,
            if i < 15 { "supported" } else { "excluded" }
        );
        let mut current = world();
        let initial = current.effective_mode("/subject").unwrap();
        assert_eq!(initial, fixture.initial_mode);
        let before = current.clone();
        let run = current.execute(&format!("chmod {} /subject", case.spec));
        match run{
            Ok(stdout)=>results.push(json!({"id":case.id,"classification":case.classification,"ok":true,"stdout":stdout,"initial_mode":initial,"mode":current.effective_mode("/subject").unwrap()})),
            Err(e)=>{assert_eq!(current,before,"Failed model operation mutated the world");results.push(json!({"id":case.id,"classification":case.classification,"ok":false,"initial_mode":initial,"error":e.message,"honest_exclusion":e.kind==ErrorKind::Unsupported}));}
        }
    }
    let mut rows: Vec<_> = bindings::COMPILED
        .iter()
        .map(|(p, b)| Pin {
            bytes: b.len(),
            path: p,
            sha256: digest::hash(b),
        })
        .collect();
    rows.sort_by(|a, b| a.path.cmp(b.path));
    assert_eq!(rows.len(), bindings::COMPILED.len());
    assert!(rows.windows(2).all(|w| w[0].path < w[1].path));
    let canonical = serde_json::to_vec(&rows).unwrap();
    println!(
        "{}",
        json!({"purpose":"fixed-sticky-mode-reference-v1","compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),"fixture_sha256":digest::hash(FIXTURE),"compiled_inputs_count":rows.len(),"compiled_inputs_sha256":digest::hash(&canonical),"observations":results})
    );
}
