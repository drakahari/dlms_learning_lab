//! Fixed developer-only actual10 evidence; no learner source/CLI or host execution.
//! Ordinary all-target gates exclude this immutable historical producer.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[path = "support/automation_digest.rs"]
mod digest;
#[path = "../tests/support/actual9_evidence.rs"]
mod original9;
const DESTINATION: &str =
    ".build-artifacts/reviews/bash-parameter-defaults/captures/public10-154-350-r1";
const RECIPES: &str = include_str!("../tests/fixtures/bash-parameter-defaults-reference.json");
fn write(path: &Path, raw: &[u8]) {
    assert!(raw.len() <= 16_000_000);
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(raw).unwrap();
    f.sync_all().unwrap();
}
fn json_file(path: &Path, data: &impl serde::Serialize) {
    write(path, &serde_json::to_vec(data).unwrap());
}
fn pin(root: &Path, path: &Path) -> Value {
    let file = File::open(path).unwrap();
    let metadata = file.metadata().unwrap();
    assert!(metadata.is_file() && metadata.len() <= 16_000_000);
    assert!(!fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    let mut raw = Vec::new();
    file.take(16_000_001).read_to_end(&mut raw).unwrap();
    assert_eq!(raw.len() as u64, metadata.len());
    json!({"file":path.strip_prefix(root).unwrap().to_str().unwrap(),"bytes":raw.len(),"sha256":digest::hash(&raw)})
}
fn files(root: &Path, path: &Path, out: &mut Vec<Value>) {
    assert!(!fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    if path.is_dir() {
        let mut paths = fs::read_dir(path)
            .unwrap()
            .map(|x| x.unwrap().path())
            .collect::<Vec<_>>();
        paths.sort();
        for p in paths {
            files(root, &p, out);
        }
    } else {
        out.push(pin(root, path));
    }
    assert!(out.len() <= 1000);
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    name: String,
    source: String,
    case: Case,
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No command/source/path arguments"
    );
    assert_eq!(
        shell::EVALUATOR_REVISION,
        10,
        "Cannot relabel current outcomes as historical10"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(".git").is_dir(), "Capture only from MAIN");
    let root = fs::canonicalize(root).unwrap();
    let destination = root.join(DESTINATION);
    let parent = destination.parent().unwrap();
    fs::create_dir_all(parent).unwrap();
    let mut at = PathBuf::new();
    for component in parent.components() {
        at.push(component);
        assert!(fs::symlink_metadata(&at).unwrap().file_type().is_dir());
    }
    fs::create_dir(&destination)
        .expect("Fresh fixed evidence directory required; preserve old runs");
    let catalog = pack::bundled().unwrap();
    assert_eq!(catalog.len(), 154);
    let scripts = catalog
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 49);
    let cases = scripts
        .iter()
        .map(|s| match &s.goal {
            Goal::Script { cases } => cases.len(),
            _ => unreachable!(),
        })
        .sum::<usize>();
    assert_eq!(cases, 350);
    let mut input_pins = Vec::new();
    for path in ["src", "scenarios"] {
        files(&root, &root.join(path), &mut input_pins);
    }
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "examples/parameters_before11_capture.rs",
        "examples/support/automation_digest.rs",
        "tests/support/actual9_evidence.rs",
        "tests/fixtures/bash-parameter-defaults-reference.json",
    ] {
        input_pins.push(pin(&root, &root.join(path)));
    }
    json_file(
        &destination.join("SOURCE.json"),
        &json!({"actual_evaluator_revision":10,"inputs":input_pins,"recipe_sha256":digest::hash(RECIPES.as_bytes())}),
    );
    let original = original9::original9();
    let mut p = progress::decode(&original, &catalog).unwrap();
    assert_eq!(p.attempts.len(), 133);
    let profile_dir = destination.join("vm-profile");
    fs::create_dir(&profile_dir).unwrap();
    let profile = profile_dir.join("progress-v1.json");
    write(&profile, &original);
    let mut rows = Vec::new();
    for s in scripts {
        p.clock += 1;
        let a = p
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, p.clock));
        workspace::prepare(a); // retain authentic6/7/8/9 history before current execution
        let Goal::Script { cases } = &s.goal else {
            unreachable!()
        };
        let source = &s.solution[0];
        let outcomes = cases
            .iter()
            .map(|case| shell::run(source, case))
            .collect::<Vec<_>>();
        assert!(
            cases.iter().zip(&outcomes).all(|(c, o)| c.accepts(o)),
            "Actual10 reference differs: {}",
            s.id
        );
        let record = Record {
            engine_revision: 10,
            source: source.clone(),
            results: outcomes,
        };
        a.state = State::Script {
            source: source.clone(),
            run: Some(record.clone()),
        };
        let lesson = s.teaching.as_ref().unwrap();
        let example = shell::run(&lesson.example, &lesson.fixture);
        assert!(lesson.fixture.accepts(&example));
        let mut learning = script_teaching::Learning::new(lesson);
        learning.example_result = Some(example.clone());
        a.workspace.as_mut().unwrap().script_learning = Some(learning);
        assert!(progress::current_solved(a, s));
        rows.push(json!({"id":s.id,"source":source,"cases":cases,"record":record,
            "example_source":lesson.example,"example_case":lesson.fixture,"example_outcome":example}));
    }
    // Six new script attempts were explicitly exercised; never invent the15 later
    // non-script contexts while importing the authentic133-context older profile.
    assert_eq!(p.attempts.len(), 139);
    let mut stream = Vec::new();
    for row in &rows {
        serde_json::to_writer(&mut stream, row).unwrap();
        stream.push(b'\n');
    }
    write(&destination.join("actual10-authored.jsonl"), &stream);
    let recipes: Vec<Recipe> = serde_json::from_str(RECIPES).unwrap();
    assert_eq!(recipes.len(), 35);
    let observations=recipes.iter().map(|r| {assert!(r.case.valid());json!({"name":r.name,"source":r.source,"case":r.case,"actual10":shell::run(&r.source,&r.case)})}).collect::<Vec<_>>();
    json_file(&destination.join("actual10-parameters.json"), &observations);
    let saved = progress::save(&profile, &p).unwrap();
    assert!(saved.durability_warning.is_none());
    let loaded = progress::load(&profile, &catalog).unwrap();
    assert_eq!(loaded.attempts.len(), 139);
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10]
    );
    original9::exact(&loaded, &p);
    let bytes = fs::read(&profile).unwrap();
    write(&destination.join("writer10-original.json"), &bytes);
    for id in ["shell-handover", "shell-command-plan"] {
        let a = p.attempts.get_mut(id).unwrap();
        a.state = State::Script {
            source: format!("# separate unfinished assignment {id}\nprintf 'unfinished"),
            run: None,
        };
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source = format!("# distinct unfinished practice {id}\nprintf 'example");
        l.example_result = None;
    }
    progress::save(&profile, &p).unwrap();
    write(
        &destination.join("writer10-drafts.json"),
        &fs::read(&profile).unwrap(),
    );
    let mut outputs = Vec::new();
    files(&destination, &destination, &mut outputs);
    let manifest = json!({"scope":"Fixed private synthetic actual10 VM capture; never personal progress, Bash or a learner execution engine", "actual_evaluator_revision":10,"catalog_contexts":154,"profile_contexts":139,"script_records":49,"authored_cases":350,"examples":49,"parameter_proposals":35,"original9_sha256":digest::hash(&original),"source":pin(&destination,&destination.join("SOURCE.json")),"outputs":outputs});
    let raw = serde_json::to_vec(&manifest).unwrap();
    write(&destination.join("MANIFEST.json"), &raw);
    write(
        &destination.join("COMPLETE"),
        format!("{}\n", digest::hash(&raw)).as_bytes(),
    );
    println!(
        "PASS actual10 before11 capture:49 scripts/350 cases/49 examples/35 proposals/139 owned profile contexts"
    );
}
