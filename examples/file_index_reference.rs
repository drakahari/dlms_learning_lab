//! Fixed in-process private-index observations only. No host utility execution or external input.
#[path = "support/file_index_bindings.rs"]
mod bindings;
#[path = "support/file_index_digest.rs"]
mod digest;
use dlms_learning_lab::{
    engine::Node,
    file_index::{self, Context, Status},
    filesystem::Tree,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize)]
struct Pin<'a> {
    bytes: usize,
    path: &'a str,
    sha256: String,
}
use serde_json::json;
use std::collections::BTreeMap;
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/file-index-reference-v1.json");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    directories: Vec<String>,
    files: Vec<File>,
    phases: Vec<Phase>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    path: String,
    text: String,
    mode: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase {
    id: String,
    prunes: Vec<String>,
    mutations: Vec<Mutation>,
    rebuild: bool,
    queries: Vec<Query>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mutation {
    op: String,
    path: String,
    #[serde(default)]
    from: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    literal: String,
    existing: bool,
}
fn node(directory: bool, mode: u16) -> Node {
    Node {
        directory,
        mode,
        owner: "alex".into(),
        group: "staff".into(),
    }
}
fn config(prunes: &[String]) -> String {
    format!("PRUNEPATHS = \"{}\"\n", prunes.join(" "))
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "Fixed probe accepts no paths/scripts/case selectors"
    );
    let f: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.phases.len(), 5);
    assert_eq!(f.files.len(), 7);
    assert_eq!(f.directories.len(), 6);
    assert_eq!(
        f.phases.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        [
            "initial",
            "stale-live",
            "rebuilt-live",
            "config-only",
            "config-rebuilt"
        ]
    );
    let mut nodes = BTreeMap::from([
        ("/".into(), node(true, 0o700)),
        ("/work".into(), node(true, 0o700)),
    ]);
    let mut files = BTreeMap::new();
    for p in &f.directories {
        nodes.insert(p.clone(), node(true, 0o700));
    }
    for leaf in &f.files {
        nodes.insert(leaf.path.clone(), node(false, leaf.mode));
        files.insert(leaf.path.clone(), leaf.text.clone());
    }
    nodes.insert("/work/index.conf".into(), node(false, 0o600));
    files.insert("/work/index.conf".into(), config(&f.phases[0].prunes));
    let source:Tree=serde_json::from_value(json!({"nodes":nodes,"files":files,"users":{"alex":["staff"]},"user":"alex","home":"/work","cwd":"/work","previous":null,"umask":18,"clock":0,"modified":{}})).unwrap();
    let held = Context {
        version: 1,
        source_id: "fixed-file-index-reference-v1".into(),
        root: "/scan".into(),
        database: "/work/private.db".into(),
        config: "/work/index.conf".into(),
    };
    held.validate_source(&source).unwrap();
    let mut current = source.clone();
    let mut saved = None;
    let mut observations = vec![];
    let mut phases = vec![];
    for p in &f.phases {
        for m in &p.mutations {
            match m.op.as_str() {
                "remove" => {
                    assert!(m.from.is_none());
                    current.execute(&format!("rm {}", m.path)).unwrap();
                }
                "copy" => {
                    current
                        .execute(&format!("cp {} {}", m.from.as_ref().unwrap(), m.path))
                        .unwrap();
                }
                _ => panic!("Fixed mutation roster changed"),
            }
        }
        current.files.insert(held.config.clone(), config(&p.prunes));
        assert!(current.valid());
        if p.rebuild {
            let o = file_index::execute_bound(
                &held,
                Some(&held),
                &source,
                &current,
                &mut saved,
                "updatedb -l 0 -o /work/private.db -U /scan --config-file /work/index.conf",
            )
            .unwrap();
            assert_eq!(o.status, Status::Rebuilt);
        }
        let i = saved.as_ref().unwrap();
        phases.push(json!({"id":p.id,"rebuild":p.rebuild,"index_entries":i.entries(),"index_prunes":i.pruned(),"fresh":file_index::matches_current(i,&held,Some(&held),&source,&current).unwrap()}));
        for q in &p.queries {
            let command = format!(
                "locate {}-d /work/private.db {}",
                if q.existing { "-e " } else { "" },
                q.literal
            );
            let o = file_index::execute_bound(
                &held,
                Some(&held),
                &source,
                &current,
                &mut saved,
                &command,
            )
            .unwrap();
            let status = match o.status {
                Status::Found => "Found",
                Status::NoMatch => "NoMatch",
                Status::Rebuilt => panic!("Wrong query kind"),
            };
            observations.push(json!({"phase":p.id,"id":q.id,"existing":q.existing,"literal":q.literal,"status":status,"paths":o.paths,"stdout":o.stdout}));
        }
    }
    assert_eq!(observations.len(), 17);
    let mut compiled: Vec<_> = bindings::COMPILED
        .iter()
        .map(|(p, b)| Pin {
            bytes: b.len(),
            path: p,
            sha256: digest::hash(b),
        })
        .collect();
    compiled.sort_by(|a, b| std::cmp::Ord::cmp(&a.path, &b.path));
    assert!(compiled.windows(2).all(|w| w[0].path < w[1].path));
    let commitment = digest::hash(&serde_json::to_vec(&compiled).unwrap());
    println!(
        "{}",
        json!({"purpose":"fixed-file-index-reference-v1","compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),"fixture_sha256":digest::hash(FIXTURE),"compiled_inputs_count":compiled.len(),"compiled_inputs_sha256":commitment,"phases":phases,"observations":observations})
    );
}
