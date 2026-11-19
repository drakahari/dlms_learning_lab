//! Fixed in-process whereis fixture observations; never starts a host utility.
#[path = "support/whereis_bindings.rs"]
mod bindings;
#[path = "support/whereis_digest.rs"]
mod digest;
use dlms_learning_lab::{
    engine::Node,
    filesystem::Tree,
    whereis::{self, Category, Context, Directory},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/whereis-reference-v1.json");
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
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    classification: String,
    category: Category,
    directory: String,
    name: String,
    nodes: Vec<Leaf>,
    mutations: Vec<Mutation>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Leaf {
    path: String,
    directory: bool,
    mode: u16,
    #[serde(default)]
    text: Option<String>,
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Mutation {
    Chmod { path: String, mode: u16 },
    Remove { path: String },
    Add { node: Leaf },
}
fn node(directory: bool, mode: u16) -> Node {
    Node {
        directory,
        mode,
        owner: "alex".into(),
        group: "staff".into(),
    }
}
fn insert(tree: &mut Tree, n: &Leaf) {
    assert!(
        tree.nodes
            .insert(n.path.clone(), node(n.directory, n.mode))
            .is_none()
    );
    if n.directory {
        assert!(n.text.is_none());
    } else {
        assert!(
            tree.files
                .insert(n.path.clone(), n.text.as_ref().unwrap().clone())
                .is_none()
        );
    }
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No arbitrary CLI path, script or case selectors"
    );
    let f: Fixture = serde_json::from_slice(FIXTURE).unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.cases.len(), 24);
    let mut observations = vec![];
    for (i, c) in f.cases.iter().enumerate() {
        assert_eq!(
            c.classification,
            if i < 18 { "supported" } else { "unsupported" }
        );
        let initial: Tree = serde_json::from_value(json!({
            "nodes":{"/":node(true,0o700),"/work":node(true,0o700)},
            "files":{},"users":{"alex":["staff"]},"user":"alex","home":"/work",
            "cwd":"/work","previous":null,"umask":18,"clock":0,"modified":{}
        }))
        .unwrap();
        let mut source = initial;
        for n in &c.nodes {
            insert(&mut source, n);
        }
        assert!(source.valid());
        let mut current = source.clone();
        for m in &c.mutations {
            match m {
                Mutation::Chmod { path, mode } => current.nodes.get_mut(path).unwrap().mode = *mode,
                Mutation::Remove { path } => {
                    assert!(current.nodes.remove(path).is_some());
                    assert!(current.files.remove(path).is_some());
                }
                Mutation::Add { node: n } => insert(&mut current, n),
            }
        }
        assert!(current.valid());
        let context = Context {
            version: 1,
            source_id: format!("fixed-whereis-{}", c.id),
            names: vec![c.name.clone()],
            directories: vec![Directory {
                category: c.category,
                path: c.directory.clone(),
            }],
        };
        let (small, big) = match c.category {
            Category::Binary => ("-b", "-B"),
            Category::Manual => ("-m", "-M"),
            Category::Source => ("-s", "-S"),
        };
        let input = format!("whereis {small} {big} {} -f {}", c.directory, c.name);
        match whereis::execute_bound(&context,Some(&context),&source,&current,&input){
            Ok(o)=>observations.push(json!({"id":c.id,"classification":c.classification,"status":o.exit_status,"paths":o.paths,"stdout":o.stdout,"scan":o.scan})),
            Err(e)=>observations.push(json!({"id":c.id,"classification":c.classification,"error_kind":format!("{:?}",e.kind),"diagnostic":e.message})),
        }
    }
    let mut pins: Vec<_> = bindings::COMPILED
        .iter()
        .map(|(p, b)| Pin {
            bytes: b.len(),
            path: p,
            sha256: digest::hash(b),
        })
        .collect();
    pins.sort_by(|a, b| a.path.cmp(b.path));
    assert!(pins.windows(2).all(|w| w[0].path < w[1].path));
    // Pin field declaration order is bytes,path,sha256. Python uses sorted keys.
    let commitment = digest::hash(&serde_json::to_vec(&pins).unwrap());
    println!(
        "{}",
        json!({"purpose":"fixed-whereis-reference-v1","compiled_manifest_dir":env!("CARGO_MANIFEST_DIR"),"fixture_sha256":digest::hash(FIXTURE),"compiled_inputs_count":pins.len(),"compiled_inputs_sha256":commitment,"observations":observations})
    );
}
