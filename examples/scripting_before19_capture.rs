//! Fixed genuine18 evidence producer. Never executes host or learner programs.
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
#[path = "../tests/support/actual17_evidence.rs"]
mod actual17;
include!("support/before19_inputs.rs");
mod digest {
    include!("support/automation_digest.rs");
}
const INDEX: &[u8] = include_bytes!("../tests/fixtures/bash-before19-source-index.json");
const EVIDENCE_BINDINGS: &[(&str, &[u8])] = &[
    (
        "examples/scripting_before19_capture.rs",
        include_bytes!("scripting_before19_capture.rs"),
    ),
    (
        "examples/support/before19_inputs.rs",
        include_bytes!("support/before19_inputs.rs"),
    ),
    ("tests/fixtures/bash-before19-source-index.json", INDEX),
];
const DESTINATION: &str = ".build-artifacts/reviews/bash-associative/captures/public18-277-608-r1";
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
    assert_eq!(old.attempts.len(), 158);
    let prior_ids = old.attempts.keys().cloned().collect::<Vec<_>>();
    let mut added = Vec::new();
    let mut expected_records = records(&old);
    let mut executed = 0;
    for row in rows {
        let id = row["id"].as_str().unwrap();
        if !old.attempts.contains_key(id) {
            continue;
        } // No new work invented in inherited158. 
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
        assert_eq!(record.engine_revision, 18);
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
        l.example_revision = 18;
        l.example_result = Some(serde_json::from_value(row["example_outcome"].clone()).unwrap());
        if let Some(before) = old.attempts.get(id) {
            assert_eq!(metadata(a), metadata(before), "{id}");
        }
        assert!(progress::current_solved(a, s));
    }
    assert_eq!(added.len(), 0);
    assert_eq!(p.attempts.len(), old.attempts.len());
    assert_eq!(executed, if drafts { 66 } else { 68 });
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
        vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18]
    );
    assert_eq!(
        fs::read(path.with_extension("before-shell-v18.json")).unwrap(),
        raw
    );
    let saved_raw = fs::read(&path).unwrap();
    write(
        &destination.join(if drafts {
            "writer18-drafts.json"
        } else {
            "writer18-original.json"
        }),
        &saved_raw,
    );
    progress::save(&path, &loaded).unwrap();
    assert_eq!(fs::read(&path).unwrap(), saved_raw);
    assert_eq!(
        fs::read(path.with_extension("before-shell-v18.json")).unwrap(),
        raw
    );
    assert!(folder.starts_with(root));
    json!({"prior_ids":prior_ids,"contexts":158,"current_records":executed,"added_script_ids":added,"non_script_contexts_preserved":90,"unfinished_assignment_and_example_drafts_preserved":if drafts {2}else{0},"all_prior_records_retained":true})
}
// Proposed developer-only capture extension; not observed or sealed evidence.
fn startup_observations(
    rows: Option<&[dlms_learning_lab::startup_workspace::CaseResult]>,
) -> Value {
    match rows{None=>Value::Null,Some(rows)=>json!(rows.iter().map(|r|json!({
        "id":r.id,"label":r.label,"checks":r.checks,"error":r.error,"child":r.child,"passed":r.passed(),
        "parent":r.parent.as_ref().map(|p|json!({"selection":p.selection,"output":p.output,"variables":p.variables,"command_finished":p.command_finished}))
    })).collect::<Vec<_>>())}
}
fn typed_observation(a: &Attempt, s: &pack::Scenario, phase: &str) -> Value {
    use dlms_learning_lab::environment_workspace as e;
    let observations = match (&a.state, &s.goal) {
        (State::Environment { simulation }, Goal::Environment { .. }) => json!({
            "assignment":simulation.assignment.result,"example":simulation.example.result,
            "assignment_verified_in_process":simulation.assignment.result.as_ref().is_some_and(|r|r.verified()),
            "example_verified_in_process":simulation.example.result.as_ref().is_some_and(|r|r.verified()),
            "assignment_checks":e::checks(&simulation.definition.assignment,&simulation.assignment),
            "example_checks":e::checks(&simulation.definition.example,&simulation.example)
        }),
        (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) => json!({
            "assignment":startup_observations(simulation.assignment.results(&definition.initial)),
            "example":startup_observations(simulation.practice.results(&definition.example)),
            "assignment_pending":simulation.assignment.pending(),"example_pending":simulation.practice.pending(),
            "assignment_checks":definition.checks(simulation)
        }),
        _ => panic!("Closed typed activity only"),
    };
    json!({"purpose":"Synthetic developer execution, not personal learner credit","id":s.id,"scenario_version":s.version,"phase":phase,"goal":s.goal,"goal_sha256":digest::hash(&serde_json::to_vec(&value(&s.goal)).unwrap()),"state":a.state,"observations":observations,"current_solved":progress::current_solved(a,s)})
}
fn typed_run(a: &mut Attempt, s: &pack::Scenario) {
    use dlms_learning_lab::environment_workspace as e;
    match (&mut a.state, &s.goal) {
        (State::Environment { simulation }, Goal::Environment { .. }) => {
            let d = &simulation.definition;
            let record = e::run(&d.assignment, &simulation.assignment.source).unwrap();
            assert!(simulation.assignment.store(record, &d.assignment));
            let record = e::run(&d.example, &simulation.example.source).unwrap();
            assert!(simulation.example.store(record, &d.example));
        }
        (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) => {
            simulation.assignment.run(&definition.initial).unwrap();
            simulation.practice.run(&definition.example).unwrap();
        }
        _ => panic!("Closed typed replay only"),
    }
}
fn typed_profiles(destination: &Path, catalog: &[pack::Scenario], index: &Value) -> Value {
    use dlms_learning_lab::{engine, environment_workspace as e};
    let ids = index["typed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 4);
    let mut observations = vec![];
    let mut reports = vec![];
    for drafts in [false, true] {
        let role = if drafts { "drafts" } else { "passed" };
        let folder = destination.join(format!("typed-{role}"));
        fs::create_dir(&folder).unwrap();
        let mut p = Progress {
            schema_version: 1,
            clock: 200,
            ..Default::default()
        };
        p.favorites.insert(ids[0].into());
        for id in &ids {
            let s = catalog.iter().find(|s| s.id == *id).unwrap();
            let mut a = Attempt::new(s, 200);
            workspace::prepare(&mut a);
            assert!(!progress::current_solved(&a, s));
            // Actual unfinished-starter observations precede any reference insertion.
            typed_run(&mut a, s);
            assert!(!progress::current_solved(&a, s));
            observations.push(typed_observation(&a, s, "starter"));
            if drafts {
                match &mut a.state {
                    State::Environment { simulation } => {
                        simulation
                            .assignment
                            .edit("# unfinished assignment\nSITE='".into())
                            .unwrap();
                        simulation
                            .example
                            .edit("# different practice\nLOCATION='".into())
                            .unwrap();
                        simulation.view = script_teaching::View::Example;
                    }
                    State::StartupFiles { simulation } => {
                        let Goal::StartupFiles { definition } = &s.goal else {
                            panic!()
                        };
                        let file = simulation.assignment.selected.clone();
                        simulation
                            .assignment
                            .edit(
                                &definition.initial,
                                &file,
                                "# unfinished assignment\nAPP_NAME='",
                            )
                            .unwrap();
                        let file = simulation.practice.selected.clone();
                        simulation
                            .practice
                            .edit(
                                &definition.example,
                                &file,
                                "# different practice\nTOOL_LABEL='",
                            )
                            .unwrap();
                        simulation.assignment.case = 1;
                        simulation.view = script_teaching::View::Example;
                    }
                    _ => panic!(),
                }
                assert!(!progress::current_solved(&a, s));
            } else {
                match (&mut a.state, &s.goal) {
                    (State::Environment { simulation }, Goal::Environment { .. }) => {
                        let d = &simulation.definition;
                        simulation.example.edit(d.example.starter.clone()).unwrap();
                        let r = e::run(&d.example, &simulation.example.source).unwrap();
                        assert!(simulation.example.store(r, &d.example));
                    }
                    (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) => {
                        simulation
                            .practice
                            .replace(&definition.example, &definition.example_worked)
                            .unwrap();
                        simulation.practice.run(&definition.example).unwrap();
                    }
                    _ => panic!(),
                }
                assert!(
                    !progress::current_solved(&a, s),
                    "Examples never grant assignment credit"
                );
                observations.push(typed_observation(&a, s, "worked-example-only"));
                for line in &s.solution {
                    engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
                }
                assert_eq!(
                    progress::current_solved(&a, s),
                    matches!(s.goal, Goal::StartupFiles { .. }),
                    "Saving an environment draft does not execute it: {}",
                    s.id
                );
                // Re-run through the public typed APIs; never manufacture proof rows.
                typed_run(&mut a, s);
                assert!(progress::current_solved(&a, s));
            }
            observations.push(typed_observation(&a, s, role));
            p.attempts.insert((*id).into(), a);
        }
        let before = value(&p);
        let path = folder.join("progress-v1.json");
        let saved = progress::save_with_catalog(&path, &p, catalog).unwrap();
        assert!(saved.durability_warning.is_none());
        let raw = fs::read(&path).unwrap();
        write(
            &destination.join(format!("writer18-typed-{role}.json")),
            &raw,
        );
        let mut loaded = progress::load(&path, catalog).unwrap();
        assert_eq!(before, value(&loaded));
        for id in &ids {
            let s = catalog.iter().find(|s| s.id == *id).unwrap();
            let a = loaded.attempts.get_mut(*id).unwrap();
            assert!(
                !progress::current_solved(a, s),
                "Reload cannot deserialize verification tokens"
            );
            observations.push(typed_observation(
                a,
                s,
                &format!("{role}-loaded-unverified"),
            ));
            if !drafts {
                typed_run(a, s);
                assert!(progress::current_solved(a, s));
                observations.push(typed_observation(a, s, "passed-loaded-reverified"));
            }
        }
        assert_eq!(
            before,
            value(&loaded),
            "Runtime verification changes no serialized input/observation bytes"
        );
        progress::save_with_catalog(&path, &loaded, catalog).unwrap();
        assert_eq!(raw, fs::read(&path).unwrap());
        reports.push(json!({"role":role,"contexts":ids.len(),"draft_bytes_exact":true,"reload_current_credit_before_execution":false,"reverified":!drafts,"repeat_save_stable":true}));
    }
    json_file(
        &destination.join("TYPED-OBSERVATIONS.json"),
        &json!({"purpose":"Separate synthetic typed contexts, never added to inherited profiles","actual_evaluator_revision":18,"reports":reports,"observations":observations}),
    );
    json!({"contexts_per_profile":4,"reports":reports,"observations":observations.len()})
}

fn new_script_profiles(
    destination: &Path,
    catalog: &[pack::Scenario],
    index: &Value,
    rows: &[Value],
) -> Value {
    let ids = index["new_scripts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
            "shell-stage-report",
            "shell-cleanup-preview",
            "shell-recover-publication"
        ]
    );
    let mut observations = vec![];
    let mut reports = vec![];
    for drafts in [false, true] {
        let role = if drafts { "drafts" } else { "passed" };
        let folder = destination.join(format!("new-{role}"));
        fs::create_dir(&folder).unwrap();
        let mut p = Progress {
            schema_version: 1,
            clock: 300,
            ..Default::default()
        };
        p.favorites.insert(ids[0].into());
        for id in &ids {
            let s = catalog.iter().find(|s| s.id == *id).unwrap();
            let row = rows.iter().find(|r| r["id"] == *id).unwrap();
            let mut a = Attempt::new(s, 300);
            workspace::prepare(&mut a);
            let lesson = s.teaching.as_ref().unwrap();
            let learning = a
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .get_or_insert_with(|| script_teaching::Learning::new(lesson));
            if drafts {
                learning.example_source = format!("# separate unfinished {id} practice\nVALUE='");
                learning.example_result = None;
                a.state = State::Script {
                    source: format!("# unfinished {id} assignment\nKEY='"),
                    run: None,
                };
            } else {
                let record: Record = serde_json::from_value(row["record"].clone()).unwrap();
                assert_eq!(record.engine_revision, 18);
                learning.example_source = lesson.example.clone();
                learning.example_revision = 18;
                learning.example_result =
                    Some(serde_json::from_value(row["example_outcome"].clone()).unwrap());
                a.state = State::Script {
                    source: record.source.clone(),
                    run: Some(record),
                };
            }
            assert_eq!(progress::current_solved(&a, s), !drafts);
            observations.push(json!({"id":id,"role":role,"state":a.state,"learning":a.workspace.as_ref().unwrap().script_learning,"current_solved":progress::current_solved(&a,s)}));
            p.attempts.insert((*id).into(), a);
        }
        let path = folder.join("progress-v1.json");
        let saved = progress::save(&path, &p).unwrap();
        assert!(saved.durability_warning.is_none());
        let raw = fs::read(&path).unwrap();
        let loaded = progress::load(&path, catalog).unwrap();
        assert_eq!(value(&loaded), value(&p));
        for id in &ids {
            let s = catalog.iter().find(|s| s.id == *id).unwrap();
            assert_eq!(progress::current_solved(&loaded.attempts[*id], s), !drafts);
        }
        write(&destination.join(format!("writer18-new-{role}.json")), &raw);
        progress::save(&path, &loaded).unwrap();
        assert_eq!(fs::read(&path).unwrap(), raw);
        reports.push(json!({"role":role,"contexts":3,"loaded_current_solved":!drafts,"repeat_save_stable":true}));
    }
    json_file(
        &destination.join("NEW-SCRIPT-OBSERVATIONS.json"),
        &json!({"purpose":"Separate new synthetic fixtures, never inherited learner work","actual_evaluator_revision":18,"reports":reports,"observations":observations}),
    );
    json!({"contexts_per_profile":3,"observations":observations.len(),"reports":reports})
}

fn main() {
    assert_eq!(
        std::env::args_os().len(),
        1,
        "No source/profile/path/engine CLI"
    );
    assert_eq!(
        shell::EVALUATOR_REVISION,
        18,
        "Never relabel a later evaluator as18"
    );
    assert_eq!(
        std::env::var("DLMS_FIXED_CAPTURE18").as_deref(),
        Ok("before19-r1")
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
            digest::hash(&raw),
            digest::hash(compiled),
            "Compiled source changed: {name}"
        );
        pins.push(json!({"path":name,"size":raw.len(),"sha256":digest::hash(&raw)}));
    }
    let index: Value = serde_json::from_slice(INDEX).unwrap();
    let catalog = pack::ccna_pilot().unwrap();
    assert_eq!(catalog.len(), index["catalog"].as_array().unwrap().len());
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
    assert_eq!(scripts.len(), index["scripts"].as_array().unwrap().len());
    let original = actual17::observed("writer17-original.json");
    let drafts = actual17::observed("writer17-drafts.json");
    assert_eq!(
        digest::hash(&original),
        index["prior"]["writer17-original.json"].as_str().unwrap()
    );
    assert_eq!(
        digest::hash(&drafts),
        index["prior"]["writer17-drafts.json"].as_str().unwrap()
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
        &json!({"actual_evaluator_revision":18,"index_sha256":digest::hash(INDEX),"inputs":pins}),
    );
    write(&destination.join("input17-original.json"), &original);
    write(&destination.join("input17-drafts.json"), &drafts);
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
        rows.push(json!({"id":s.id,"source":s.solution[0],"cases":cases,"record":Record{engine_revision:18,source:s.solution[0].clone(),results},"example_source":lesson.example,"example_case":lesson.fixture,"example_outcome":example,"example_revision":18}));
    }
    assert_eq!(
        count,
        index["counts"]["authored_cases"].as_u64().unwrap() as usize
    );
    assert_eq!(observations.len(), count + scripts.len());
    for (name, data) in [
        ("VM-OBSERVATIONS.jsonl", &observations),
        ("actual18-authored.jsonl", &rows),
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
    let manifest = json!({"purpose":"Actual current18 development fixtures, never personal progress or mastery","actual_evaluator_revision":18,"catalog_contexts":catalog.len(),"script_records":scripts.len(),"authored_cases":count,"examples":scripts.len(),"observations":observations.len(),"typed_profile":typed_profiles(&destination,&catalog,&index),"new_script_profile":new_script_profiles(&destination,&catalog,&index,&rows),"original_profile":original_profile,"draft_profile":draft_profile});
    let raw = serde_json::to_vec(&manifest).unwrap();
    write(&destination.join("MANIFEST.json"), &raw);
    write(
        &destination.join("COMPLETE"),
        format!("{}\n", digest::hash(&raw)).as_bytes(),
    );
    println!(
        "PASS genuine18 before19:71scripts/608cases/71examples;158inherited contexts and separate synthetic typed profiles; no personal mastery"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_synthetic_typed_fixture_roundtrips_without_reconstructing_proof() {
        let folder = tempfile::tempdir().unwrap();
        let catalog = pack::ccna_pilot().unwrap();
        let index: Value = serde_json::from_slice(INDEX).unwrap();
        let report = typed_profiles(folder.path(), &catalog, &index);
        assert_eq!(report["observations"], 32);
        let mut files = vec![];
        fn collect(at: &Path, out: &mut Vec<std::path::PathBuf>) {
            for p in std::fs::read_dir(at).unwrap() {
                let p = p.unwrap().path();
                if p.is_dir() {
                    collect(&p, out);
                } else {
                    out.push(p);
                }
            }
        }
        collect(folder.path(), &mut files);
        assert_eq!(
            files.len(),
            5,
            "Two writer files, two stable progress files and complete observations"
        );
        let rows: Value = serde_json::from_slice(
            &std::fs::read(folder.path().join("TYPED-OBSERVATIONS.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(rows["observations"].as_array().unwrap().len(), 32);
    }
    #[test]
    fn typed_draft_edits_and_loaded_records_require_actual_execution() {
        let catalog = pack::ccna_pilot().unwrap();
        let index: Value = serde_json::from_slice(INDEX).unwrap();
        let mut seen = 0;
        for s in catalog
            .iter()
            .filter(|s| matches!(s.goal, Goal::Environment { .. } | Goal::StartupFiles { .. }))
        {
            seen += 1;
            let indexed = index["typed"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == s.id)
                .unwrap();
            assert_eq!(value(&s.goal), indexed["goal"]);
            assert_eq!(
                digest::hash(&serde_json::to_vec(&value(&s.goal)).unwrap()),
                indexed["goal_sha256"],
                "Rust/Python canonical goal hash must agree"
            );
            let mut a = Attempt::new(s, 10);
            workspace::prepare(&mut a);
            typed_run(&mut a, s);
            assert!(!progress::current_solved(&a, s));
            for line in &s.solution {
                dlms_learning_lab::engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
            }
            assert_eq!(
                progress::current_solved(&a, s),
                matches!(s.goal, Goal::StartupFiles { .. })
            );
            typed_run(&mut a, s);
            assert!(progress::current_solved(&a, s), "{}", s.id);
            let raw = value(&a);
            let mut loaded: Attempt = serde_json::from_value(raw.clone()).unwrap();
            assert!(!progress::current_solved(&loaded, s));
            typed_run(&mut loaded, s);
            assert!(progress::current_solved(&loaded, s));
            assert_eq!(value(&loaded), raw);
        }
        assert_eq!(seen, 4);
    }
}
