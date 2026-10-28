//! Genuine evaluator11 development fixture producer, never learner input.
//! Feature-gated after evaluator12; original9/10 producers/evidence stay intact.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Record},
    workspace,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[path = "../tests/support/actual10_evidence.rs"]
mod actual10;
const INDEX: &[u8] = include_bytes!("../tests/fixtures/bash-before12-source-index.json");
const INDEX_SHA: &str = "26645b623e5ae983634ca591259ab54f3154bd333fe10c3b43cf7a0ac8f13d91";
const DESTINATION: &str =
    ".build-artifacts/reviews/bash-numeric-validation/captures/public11-164-384-r1";
const NEW: [&str; 3] = [
    "shell-optional-policy",
    "shell-optional-report",
    "shell-optional-handover",
];
const DRAFTS: [&str; 2] = ["shell-handover", "shell-command-plan"];
const COMPILED_CORE: &[(&str, &[u8])] = &[
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
    ("src/engine.rs", include_bytes!("../src/engine.rs")),
    ("src/pack.rs", include_bytes!("../src/pack.rs")),
    ("src/progress.rs", include_bytes!("../src/progress.rs")),
    ("src/workspace.rs", include_bytes!("../src/workspace.rs")),
    (
        "src/script_teaching.rs",
        include_bytes!("../src/script_teaching.rs"),
    ),
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
        "src/shell/values.rs",
        include_bytes!("../src/shell/values.rs"),
    ),
    (
        "examples/scripting_before12_capture.rs",
        include_bytes!("../examples/scripting_before12_capture.rs"),
    ),
    (
        "examples/support/automation_digest.rs",
        include_bytes!("../examples/support/automation_digest.rs"),
    ),
    (
        "tests/support/actual10_evidence.rs",
        include_bytes!("../tests/support/actual10_evidence.rs"),
    ),
];
fn value(v: &impl serde::Serialize) -> Value {
    serde_json::to_value(v).unwrap()
}
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
    assert!(path.starts_with(root));
    let mut at = root.to_path_buf();
    for part in path.strip_prefix(root).unwrap().components() {
        at.push(part);
        assert!(!fs::symlink_metadata(&at).unwrap().file_type().is_symlink());
    }
    let file = File::open(path).unwrap();
    let info = file.metadata().unwrap();
    assert!(info.is_file() && info.len() <= 16_000_000);
    let mut bytes = Vec::new();
    file.take(16_000_001).read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes.len() as u64, info.len());
    json!({"file":path.strip_prefix(root).unwrap().to_str().unwrap(),"bytes":bytes.len(),"sha256":actual10::sha(&bytes)})
}
fn files(root: &Path, path: &Path, out: &mut Vec<Value>) {
    assert!(!fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    if path.is_dir() {
        let mut paths = fs::read_dir(path)
            .unwrap()
            .map(|p| p.unwrap().path())
            .collect::<Vec<_>>();
        paths.sort();
        for p in paths {
            files(root, &p, out);
        }
    } else {
        out.push(pin(root, path));
    }
    assert!(out.len() <= 1500);
}
fn records(a: &Attempt, current: bool) -> BTreeMap<String, usize> {
    let mut rows = BTreeMap::new();
    if current && let State::Script { run: Some(r), .. } = &a.state {
        *rows.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
    }
    if let Some(w) = &a.workspace {
        for r in w.script_history.iter().chain(&w.script_history_more) {
            *rows.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
        }
    }
    rows
}
fn metadata(mut v: Value) -> Value {
    v.as_object_mut().unwrap().remove("state");
    if let Some(w) = v.get_mut("workspace").and_then(Value::as_object_mut) {
        for k in ["script_history", "script_history_more", "checked"] {
            w.remove(k);
        }
        if let Some(l) = w.get_mut("script_learning").and_then(Value::as_object_mut) {
            l.remove("example_revision");
            l.remove("example_result");
        }
    }
    v
}
fn profile(
    root: &Path,
    destination: &Path,
    raw: &[u8],
    catalog: &[pack::Scenario],
    rows: &[Value],
    drafts: bool,
) -> Value {
    let old: Progress = serde_json::from_slice(raw).unwrap();
    let mut p = progress::decode(raw, catalog).unwrap();
    assert_eq!(value(&old), value(&p));
    assert_eq!(p.attempts.len(), 139);
    assert_eq!(
        value(&p.migration_receipts),
        json!({"workspace":true,"shell":[7,8,9,10]})
    );
    let mut added = Vec::new();
    let mut executed = 0;
    for row in rows {
        let id = row["id"].as_str().unwrap();
        let s = catalog.iter().find(|s| s.id == id).unwrap();
        if drafts
            && old
                .attempts
                .get(id)
                .is_some_and(|a| matches!(a.state, State::Script { run: None, .. }))
        {
            assert!(DRAFTS.contains(&id));
            assert_eq!(value(&p.attempts[id]), value(&old.attempts[id]));
            continue;
        }
        p.clock += 1;
        executed += 1;
        if !p.attempts.contains_key(id) {
            assert!(NEW.contains(&id));
            added.push(id);
        }
        let a = p
            .attempts
            .entry(id.into())
            .or_insert_with(|| Attempt::new(s, p.clock));
        let before = old.attempts.get(id);
        if let Some(before) = before {
            assert!(matches!(before.state, State::Script { run: Some(_), .. }));
        }
        workspace::prepare(a);
        let record: Record = serde_json::from_value(row["record"].clone()).unwrap();
        assert_eq!(record.engine_revision, 11);
        a.state = State::Script {
            source: record.source.clone(),
            run: Some(record),
        };
        let lesson = s.teaching.as_ref().unwrap();
        let w = a.workspace.as_mut().unwrap();
        let l = w
            .script_learning
            .get_or_insert_with(|| script_teaching::Learning::new(lesson));
        assert_eq!(
            l.example_source, lesson.example,
            "Never attach worked-example results to another draft"
        );
        l.example_revision = 11;
        l.example_result = Some(serde_json::from_value(row["example_outcome"].clone()).unwrap());
        if let Some(before) = before {
            assert_eq!(
                metadata(value(a)),
                metadata(value(before)),
                "Unchanged learner metadata: {id}"
            );
            assert_eq!(
                records(a, false),
                records(before, true),
                "Every genuine prior record retained: {id}"
            );
        }
        assert!(progress::current_solved(a, s));
    }
    assert_eq!(added, NEW);
    assert_eq!(executed, if drafts { 50 } else { 52 });
    assert_eq!(p.attempts.len(), 142);
    assert_eq!(p.favorites, old.favorites);
    for (id, a) in &old.attempts {
        if !matches!(a.state, State::Script { .. }) {
            assert_eq!(value(&p.attempts[id]), value(a));
        }
    }
    let missing = catalog
        .iter()
        .filter(|s| !p.attempts.contains_key(&s.id))
        .collect::<Vec<_>>();
    assert_eq!(missing.len(), 22);
    assert!(
        missing
            .iter()
            .all(|s| !matches!(s.initial, State::Script { .. }))
    );
    if drafts {
        for id in DRAFTS {
            assert_eq!(value(&p.attempts[id]), value(&old.attempts[id]));
        }
    }
    let dir = destination.join(if drafts {
        "draft-profile"
    } else {
        "original-profile"
    });
    fs::create_dir(&dir).unwrap();
    let path = dir.join("progress-v1.json");
    write(&path, raw);
    let saved = progress::save(&path, &p).unwrap();
    assert!(saved.durability_warning.is_none());
    let loaded = progress::load(&path, catalog).unwrap();
    assert_eq!(value(&loaded), value(&p));
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 11]
    );
    let backup = path.with_extension("before-shell-v11.json");
    assert_eq!(fs::read(&backup).unwrap(), raw);
    let saved_raw = fs::read(&path).unwrap();
    write(
        &destination.join(if drafts {
            "writer11-drafts.json"
        } else {
            "writer11-original.json"
        }),
        &saved_raw,
    );
    progress::save(&path, &loaded).unwrap();
    assert_eq!(fs::read(&path).unwrap(), saved_raw);
    assert_eq!(fs::read(&backup).unwrap(), raw);
    json!({"prior_contexts":139,"contexts":142,"actual11_current_records":executed,"actual11_examples":executed,
        "retained_unfinished_assignment_and_practice":if drafts {2}else{0},"explicit_new_script_ids":added,
        "missing_non_script_ids":missing.iter().map(|s|s.id.as_str()).collect::<Vec<_>>(),
        "non_script_original_attempts_exact":90,"all_prior_records_retained":true,"raw10_backup":pin(root,&backup),
        "receipts":[7,8,9,10,11]})
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No source/input/path/engine CLI"
    );
    assert_eq!(
        shell::EVALUATOR_REVISION,
        11,
        "Cannot label later evaluator observations as historical11"
    );
    assert_eq!(
        std::env::var("DLMS_FIXED_CAPTURE11").as_deref(),
        Ok("before12-r1")
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(".git").is_dir(), "MAIN only");
    let root = fs::canonicalize(root).unwrap();
    assert_eq!(
        std::env::var_os("CARGO_TARGET_DIR"),
        Some(root.join("target").into_os_string())
    );
    // Bind observed semantics to the actual compiled source, not just a runtime
    // revision label. A stale same-revision ELF cannot attest newer core bytes.
    for (name, compiled) in COMPILED_CORE {
        assert_eq!(
            pin(&root, &root.join(name))["sha256"],
            actual10::sha(compiled),
            "Compiled evaluator/progress core differs from current source: {name}"
        );
    }
    assert_eq!(actual10::sha(INDEX), INDEX_SHA);
    let index: Value = serde_json::from_slice(INDEX).unwrap();
    let catalog = pack::bundled().unwrap();
    assert_eq!(catalog.len(), 164);
    assert_eq!(
        json!(
            catalog
                .iter()
                .map(|s| json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        index["catalog"]
    );
    for row in index["bundled_packs"].as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        assert!(path.starts_with("scenarios/") && Path::new(path).components().count() == 2);
        assert_eq!(pin(&root, &root.join(path))["sha256"], row["sha256"]);
    }
    let scripts = catalog
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 52);
    assert_eq!(index["scripts"].as_array().unwrap().len(), 52);
    let original = actual10::observed("writer10-original.json");
    let drafts = actual10::observed("writer10-drafts.json");
    assert_eq!(
        actual10::sha(&original),
        index["prior_original_sha256"].as_str().unwrap()
    );
    assert_eq!(
        actual10::sha(&drafts),
        index["prior_drafts_sha256"].as_str().unwrap()
    );
    let destination = root.join(DESTINATION);
    let parent = destination.parent().unwrap();
    let mut at = root.clone();
    for c in parent.strip_prefix(&root).unwrap().components() {
        at.push(c);
        match fs::symlink_metadata(&at) {
            Ok(info) => assert!(info.is_dir() && !info.file_type().is_symlink()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&at).unwrap(),
            Err(e) => panic!("Capture parent observation failed: {e}"),
        }
    }
    fs::create_dir(&destination).expect("Fresh fixed capture only; preserve all prior evidence");
    let mut inputs = Vec::new();
    for path in ["src", "scenarios"] {
        files(&root, &root.join(path), &mut inputs);
    }
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "examples/scripting_before12_capture.rs",
        "examples/support/automation_digest.rs",
        "tests/support/actual10_evidence.rs",
        "tests/support/actual9_evidence.rs",
        "examples/automation_before10_capture.rs",
        "examples/parameters_before11_capture.rs",
        "tests/fixtures/bash-before12-source-index.json",
        "tools/scripting_before12_verify.py",
        "tools/tests/test_scripting_before12_verify.py",
        "tools/parameters_before11_verify.py",
        "tools/case_bash_oracle.py",
        "tools/automation_before10_oracle.py",
        "tools/native_reopen.py",
        "tools/dev.py",
        "tools/build_safety.py",
        "docs/BUILD_WORKFLOW.md",
    ] {
        inputs.push(pin(&root, &root.join(path)));
    }
    json_file(
        &destination.join("SOURCE.json"),
        &json!({"actual_evaluator_revision":11,"source_index_sha256":INDEX_SHA,"inputs":inputs}),
    );
    write(&destination.join("input10-original.json"), &original);
    write(&destination.join("input10-drafts.json"), &drafts);
    let mut rows = Vec::new();
    let mut count = 0;
    for (s, expected) in scripts.iter().zip(index["scripts"].as_array().unwrap()) {
        let Goal::Script { cases } = &s.goal else {
            unreachable!()
        };
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(
            json!({"id":s.id,"version":s.version,"source":s.solution[0],"cases":cases,
            "example_source":lesson.example,"example_case":lesson.fixture}),
            *expected
        );
        let outcomes = cases
            .iter()
            .map(|c| shell::run(&s.solution[0], c))
            .collect::<Vec<_>>();
        assert!(cases.iter().zip(&outcomes).all(|(c, o)| c.accepts(o)));
        count += cases.len();
        let example = shell::run(&lesson.example, &lesson.fixture);
        assert!(lesson.fixture.accepts(&example));
        rows.push(json!({"id":s.id,"source":s.solution[0],"cases":cases,
            "record":Record{engine_revision:11,source:s.solution[0].clone(),results:outcomes},
            "example_source":lesson.example,"example_case":lesson.fixture,"example_outcome":example,"example_revision":11}));
    }
    assert_eq!(count, 384);
    let mut stream = Vec::new();
    for row in &rows {
        serde_json::to_writer(&mut stream, row).unwrap();
        stream.push(b'\n');
    }
    write(&destination.join("actual11-authored.jsonl"), &stream);
    let original_profile = profile(&root, &destination, &original, &catalog, &rows, false);
    let draft_profile = profile(&root, &destination, &drafts, &catalog, &rows, true);
    let mut outputs = Vec::new();
    files(&destination, &destination, &mut outputs);
    let manifest = json!({"scope":"Synthetic development evidence: actual current11 VM reference executions, never personal progress or learner mastery",
        "actual_evaluator_revision":11,"catalog_contexts":164,"script_records":52,"authored_cases":384,"examples":52,
        "prior_actual10_manifest_sha256":"220cebac329a62adcc59e8338df80b94a07a383807d5832dd6905bbf332776a1",
        "source_index_sha256":INDEX_SHA,"original_profile":original_profile,"draft_profile":draft_profile,
        "source":pin(&destination,&destination.join("SOURCE.json")),"outputs":outputs});
    let raw = serde_json::to_vec(&manifest).unwrap();
    write(&destination.join("MANIFEST.json"), &raw);
    write(
        &destination.join("COMPLETE"),
        format!("{}\n", actual10::sha(&raw)).as_bytes(),
    );
    println!(
        "PASS genuine actual11 before12:52 scripts/384cases/52 examples;142 explicit profile contexts;2 authentic drafts retained; no Ether/Cron mastery"
    );
}
