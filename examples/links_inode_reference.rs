//! Fixed inode-listing model probe; no host utility or arbitrary source/paths.
#[path = "support/links_inode_bindings.rs"]
mod bindings;
#[path = "support/links_inode_digest.rs"]
mod digest;
use dlms_learning_lab::{links::State, links_workspace};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/links-inode-reference-v1.json");
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
    directories: Vec<String>,
    entries: Vec<Entry>,
    queries: Vec<Query>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Entry {
    Regular {
        path: String,
        text: String,
        mode: u16,
    },
    Hard {
        path: String,
        from: String,
    },
    Copy {
        path: String,
        from: String,
    },
    Symbolic {
        path: String,
        target: String,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    id: String,
    classification: String,
    args: Vec<String>,
}
const IDS: [&str; 8] = [
    "default-work",
    "directory-symlink",
    "hidden-only",
    "empty",
    "literal-directory",
    "link-directory",
    "excluded-file",
    "excluded-all",
];
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "Fixed compiled fixture only; no CLI selectors"
    );
    let f: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.queries.len(), 8);
    assert_eq!(f.entries.len(), 10);
    assert_eq!(
        f.queries.iter().map(|q| q.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    let directories = f.directories.iter().map(String::as_str).collect::<Vec<_>>();
    let mut world = State::new(&directories, "/work").unwrap();
    for e in &f.entries {
        match e {
            Entry::Regular { path, text, mode } => {
                assert!([0, 0o600].contains(mode));
                world.create_file(path, text).unwrap();
            }
            Entry::Hard { path, from } => world.hard_link(from, path).unwrap(),
            Entry::Copy { path, from } => world.copy_file(from, path).unwrap(),
            Entry::Symbolic { path, target } => world.symbolic_link(target, path).unwrap(),
        }
    }
    let mut source_inodes = BTreeMap::new();
    for p in f
        .directories
        .iter()
        .chain(f.entries.iter().map(|e| match e {
            Entry::Regular { path, .. }
            | Entry::Hard { path, .. }
            | Entry::Copy { path, .. }
            | Entry::Symbolic { path, .. } => path,
        }))
    {
        source_inodes.insert(p.clone(), world.stat(p).unwrap().inode);
    }
    let mut observations = vec![];
    for (i, q) in f.queries.iter().enumerate() {
        assert_eq!(
            q.classification,
            if i < 6 { "supported" } else { "excluded" }
        );
        assert!(
            q.args
                .iter()
                .all(|a| !a.contains('\'') && !a.contains('\n'))
        );
        let input = format!(
            "ls {}",
            q.args
                .iter()
                .map(|a| format!("'{a}'"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let before = world.to_json().unwrap();
        let result = links_workspace::execute(&mut world, &input);
        assert_eq!(
            world.to_json().unwrap(),
            before,
            "Readonly command changed the actual world"
        );
        match result{
            Ok(stdout)=>observations.push(json!({"id":q.id,"classification":q.classification,"status":0,"stdout":stdout,"stderr":""})),
            Err(e)=>{
                let lower=e.to_ascii_lowercase();
                let excluded=lower.contains("unsupported")||lower.contains("outside this subset");
                observations.push(json!({"id":q.id,"classification":q.classification,"error":e,"honest_exclusion":excluded}));
            }
        }
    }
    let mut rows: Vec<_> = bindings::COMPILED
        .iter()
        .map(|(path, bytes)| Pin {
            bytes: bytes.len(),
            path,
            sha256: digest::hash(bytes),
        })
        .collect();
    rows.sort_by(|a, b| a.path.cmp(b.path));
    assert!(rows.windows(2).all(|w| w[0].path < w[1].path));
    let commitment = digest::hash(&serde_json::to_vec(&rows).unwrap());
    println!(
        "{}",
        json!({"purpose":"fixed-links-inode-reference-v1","compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),"compiled_inputs_count":rows.len(),"compiled_inputs_sha256":commitment,"fixture_sha256":digest::hash(FIXTURE),"source_inodes":source_inodes,"observations":observations})
    );
}
