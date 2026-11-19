//! Fixed genuine15 evidence producer. Never executes host or learner programs.
use dlms_learning_lab::{
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Record},
    workspace,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, io::Write, path::Path};
#[path = "../tests/support/actual14_evidence.rs"]
mod actual14;
include!("support/before16_inputs.rs");
const INDEX: &[u8] = include_bytes!("../tests/fixtures/bash-before16-source-index.json");
const EVIDENCE_BINDINGS: &[(&str, &[u8])] = &[
    (
        "examples/scripting_before16_capture.rs",
        include_bytes!("scripting_before16_capture.rs"),
    ),
    (
        "examples/support/before16_inputs.rs",
        include_bytes!("support/before16_inputs.rs"),
    ),
    ("tests/fixtures/bash-before16-source-index.json", INDEX),
];
const DESTINATION: &str = ".build-artifacts/reviews/bash-tracing/captures/public15-255-549-r1";
const DRAFTS: [&str; 2] = ["shell-command-plan", "shell-handover"];

fn value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}
fn write(path: &Path, raw: &[u8]) {
    assert!(raw.len() <= 16_000_000);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(raw).unwrap();
    file.sync_all().unwrap();
}
fn json_file(path: &Path, v: &Value) {
    write(path, &serde_json::to_vec(v).unwrap());
}
fn metadata(a: &Attempt) -> Value {
    let mut v = value(a);
    v.as_object_mut().unwrap().remove("state");
    if let Some(w) = v.get_mut("workspace").and_then(Value::as_object_mut) {
        for key in ["script_history", "script_history_more", "checked"] {
            w.remove(key);
        }
        if let Some(l) = w.get_mut("script_learning").and_then(Value::as_object_mut) {
            l.remove("example_revision");
            l.remove("example_result");
        }
    }
    v
}
fn records(p: &Progress) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for a in p.attempts.values() {
        let current = match &a.state {
            State::Script { run, .. } => run.as_ref(),
            _ => None,
        };
        let histories = a
            .workspace
            .as_ref()
            .into_iter()
            .flat_map(|w| w.script_history.iter().chain(&w.script_history_more));
        for r in current.into_iter().chain(histories) {
            *out.entry(serde_json::to_string(r).unwrap()).or_default() += 1;
        }
    }
    out
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
    assert_eq!(old.attempts.len(), 150);
    let prior_ids = old.attempts.keys().cloned().collect::<Vec<_>>();
    let mut added = Vec::new();
    let mut expected_records = records(&old);
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
            added.push(id.to_owned());
        }
        let a = p
            .attempts
            .entry(id.into())
            .or_insert_with(|| Attempt::new(s, p.clock));
        if let Some(before) = old.attempts.get(id) {
            let State::Script {
                source,
                run: Some(_),
            } = &before.state
            else {
                panic!("Only authentic matching references may rerun")
            };
            assert_eq!(source, row["source"].as_str().unwrap());
        }
        workspace::prepare(a); // Archives real previous-revision records, never relabels.
        let record: Record = serde_json::from_value(row["record"].clone()).unwrap();
        assert_eq!(record.engine_revision, 15);
        *expected_records
            .entry(serde_json::to_string(&record).unwrap())
            .or_default() += 1;
        a.state = State::Script {
            source: record.source.clone(),
            run: Some(record),
        };
        let lesson = s.teaching.as_ref().unwrap();
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .get_or_insert_with(|| script_teaching::Learning::new(lesson));
        assert_eq!(l.example_source, lesson.example);
        l.example_revision = 15;
        l.example_result = Some(serde_json::from_value(row["example_outcome"].clone()).unwrap());
        if let Some(before) = old.attempts.get(id) {
            assert_eq!(metadata(a), metadata(before), "{id}");
        }
        assert!(progress::current_solved(a, s));
    }
    assert_eq!(added.len(), 6);
    assert_eq!(p.attempts.len(), 156);
    assert_eq!(executed, if drafts { 64 } else { 66 });
    assert_eq!(
        records(&p),
        expected_records,
        "No dropped, invented or relabeled history"
    );
    assert_eq!(p.favorites, old.favorites);
    for (id, before) in &old.attempts {
        if !matches!(before.state, State::Script { .. }) || drafts && DRAFTS.contains(&id.as_str())
        {
            assert_eq!(value(&p.attempts[id]), value(before));
        }
    }
    let folder = destination.join(if drafts {
        "draft-profile"
    } else {
        "original-profile"
    });
    fs::create_dir(&folder).unwrap();
    let path = folder.join("progress-v1.json");
    write(&path, raw);
    let saved = progress::save(&path, &p).unwrap();
    assert!(saved.durability_warning.is_none());
    let loaded = progress::load(&path, catalog).unwrap();
    assert_eq!(value(&loaded), value(&p));
    assert_eq!(
        loaded.migration_receipts.as_ref().unwrap().shell,
        vec![7, 8, 9, 10, 11, 12, 13, 14, 15]
    );
    assert_eq!(
        fs::read(path.with_extension("before-shell-v15.json")).unwrap(),
        raw
    );
    let saved_raw = fs::read(&path).unwrap();
    write(
        &destination.join(if drafts {
            "writer15-drafts.json"
        } else {
            "writer15-original.json"
        }),
        &saved_raw,
    );
    progress::save(&path, &loaded).unwrap();
    assert_eq!(fs::read(&path).unwrap(), saved_raw);
    assert_eq!(
        fs::read(path.with_extension("before-shell-v15.json")).unwrap(),
        raw
    );
    assert!(folder.starts_with(root));
    json!({"prior_ids":prior_ids,"contexts":156,"current_records":executed,"added_script_ids":added,"non_script_contexts_preserved":90,"unfinished_assignment_and_example_drafts_preserved":if drafts {2}else{0},"all_prior_records_retained":true})
}
fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No source/profile/path/engine CLI"
    );
    assert_eq!(
        shell::EVALUATOR_REVISION,
        15,
        "Never relabel a later evaluator as15"
    );
    assert_eq!(
        std::env::var("DLMS_FIXED_CAPTURE15").as_deref(),
        Ok("before16-r1")
    );
    let root = fs::canonicalize(env!("CARGO_MANIFEST_DIR")).unwrap();
    assert!(root.join(".git").is_dir(), "MAIN only");
    assert_eq!(
        std::env::var_os("CARGO_TARGET_DIR"),
        Some(root.join("target").into_os_string())
    );
    let mut pins = Vec::new();
    for (name, compiled) in COMPILED.iter().chain(EVIDENCE_BINDINGS) {
        let path = root.join(name);
        assert!(
            !fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        let raw = fs::read(path).unwrap();
        assert_eq!(
            actual14::sha(&raw),
            actual14::sha(compiled),
            "Compiled source changed: {name}"
        );
        pins.push(json!({"path":name,"size":raw.len(),"sha256":actual14::sha(&raw)}));
    }
    let index: Value = serde_json::from_slice(INDEX).unwrap();
    let catalog = pack::ccna_pilot().unwrap();
    assert_eq!(catalog.len(), 255);
    assert_eq!(
        json!(
            catalog
                .iter()
                .map(|s| json!({"id":s.id,"version":s.version}))
                .collect::<Vec<_>>()
        ),
        index["catalog"]
    );
    let scripts = catalog
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 66);
    let original = actual14::observed("writer14-original.json");
    let drafts = actual14::observed("writer14-drafts.json");
    assert_eq!(
        actual14::sha(&original),
        index["prior"]["writer14-original.json"].as_str().unwrap()
    );
    assert_eq!(
        actual14::sha(&drafts),
        index["prior"]["writer14-drafts.json"].as_str().unwrap()
    );
    let destination = root.join(DESTINATION);
    let mut at = root.clone();
    for part in destination
        .parent()
        .unwrap()
        .strip_prefix(&root)
        .unwrap()
        .components()
    {
        at.push(part);
        match fs::symlink_metadata(&at) {
            Ok(info) => assert!(info.is_dir() && !info.file_type().is_symlink()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&at).unwrap(),
            Err(e) => panic!("Capture parent: {e}"),
        }
    }
    fs::create_dir(&destination).expect("Fresh fixed destination only; never replace evidence");
    json_file(
        &destination.join("SOURCE.json"),
        &json!({"actual_evaluator_revision":15,"index_sha256":actual14::sha(INDEX),"inputs":pins}),
    );
    write(&destination.join("input14-original.json"), &original);
    write(&destination.join("input14-drafts.json"), &drafts);
    let mut rows = Vec::new();
    let mut observations = Vec::new();
    let mut count = 0;
    for (s, expected) in scripts.iter().zip(index["scripts"].as_array().unwrap()) {
        let Goal::Script { cases } = &s.goal else {
            unreachable!()
        };
        let lesson = s.teaching.as_ref().unwrap();
        assert_eq!(
            json!({"id":s.id,"version":s.version,"source":s.solution[0],"cases":cases,"example_source":lesson.example,"example_case":lesson.fixture}),
            *expected
        );
        let mut results = Vec::new();
        for (n, c) in cases.iter().enumerate() {
            let observed = shell::run(&s.solution[0], c);
            observations.push(json!({"id":s.id,"role":"assignment","case":n,"outcome":observed}));
            assert!(c.accepts(&observed), "{} case{}", s.id, n);
            results.push(observed);
        }
        count += cases.len();
        let example = shell::run(&lesson.example, &lesson.fixture);
        observations.push(json!({"id":s.id,"role":"example","case":0,"outcome":example}));
        assert!(lesson.fixture.accepts(&example));
        rows.push(json!({"id":s.id,"source":s.solution[0],"cases":cases,"record":Record{engine_revision:15,source:s.solution[0].clone(),results},"example_source":lesson.example,"example_case":lesson.fixture,"example_outcome":example,"example_revision":15}));
    }
    assert_eq!(count, 549);
    assert_eq!(observations.len(), 615);
    for (name, data) in [
        ("VM-OBSERVATIONS.jsonl", &observations),
        ("actual15-authored.jsonl", &rows),
    ] {
        let mut raw = Vec::new();
        for row in data {
            serde_json::to_writer(&mut raw, row).unwrap();
            raw.push(b'\n');
            assert!(raw.len() <= 16_000_000);
        }
        write(&destination.join(name), &raw);
    }
    let original_profile = profile(&root, &destination, &original, &catalog, &rows, false);
    let draft_profile = profile(&root, &destination, &drafts, &catalog, &rows, true);
    let manifest = json!({"purpose":"Actual current15 development fixtures, never personal progress or mastery","actual_evaluator_revision":15,"catalog_contexts":255,"script_records":66,"authored_cases":549,"examples":66,"observations":615,"original_profile":original_profile,"draft_profile":draft_profile});
    let raw = serde_json::to_vec(&manifest).unwrap();
    write(&destination.join("MANIFEST.json"), &raw);
    write(
        &destination.join("COMPLETE"),
        format!("{}\n", actual14::sha(&raw)).as_bytes(),
    );
    println!(
        "PASS genuine15 before16:66scripts/549cases/66examples;156derived contexts/2unfinished drafts preserved; no new non-script mastery"
    );
}
