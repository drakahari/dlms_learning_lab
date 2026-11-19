//! Synthetic source/storage fixtures only; no registered authored management labs.
use dlms_learning_lab::{
    aaa_workspace as w, backup,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
#[path = "support/aaa_tasks.rs"]
mod common;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
fn original() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted358().unwrap())
}
fn source(restricted: bool) -> pack::Scenario {
    let d = common::definition(restricted);
    let mut s = original()
        .iter()
        .find(|s| s.id == "ccna-security-identity")
        .unwrap()
        .clone();
    s.id = d.initial.id.clone();
    s.objective = d.initial.instructions.clone();
    s.prerequisites.clear();
    s.solution = vec![format!("{}{}", w::PREFIX, d.initial.reference)];
    s.initial = State::AaaAccess {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    s.goal = Goal::AaaAccess {
        definition: Box::new(d),
    };
    s
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}

#[test]
fn original358_snapshot_and_synthetic_source_engine_loading_are_real_and_unregistered() {
    assert_eq!(original().len(), 358);
    assert_eq!(
        dlms_learning_lab::course::accepted358(original())
            .unwrap()
            .len(),
        127
    );
    assert_eq!(
        serde_json::to_value(original()).unwrap(),
        serde_json::to_value(&pack::ccna_pilot().unwrap()[..358]).unwrap()
    );
    for restricted in [false, true] {
        let s = source(restricted);
        let text = serde_json::to_string(&pack::Pack {
            schema_version: 1,
            scenarios: vec![s.clone()],
        })
        .unwrap();
        let loaded = pack::load(&text).unwrap();
        assert_eq!(loaded.len(), 1);
        let mut a = fresh(&loaded[0]);
        assert!(!progress::current_solved(&a, &s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, &s));
    }
}

#[test]
fn forged_flags_bad_edits_and_foreign_context_cannot_retain_current_success() {
    let s = source(true);
    let mut a = fresh(&s);
    let State::AaaAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.checked = true;
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, &s));
    let State::AaaAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.draft = "unfinished input".into();
    assert!(!progress::current_solved(&a, &s));
    let State::AaaAccess { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin = "another-task".into();
    assert!(w::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    let mut a = fresh(&s);
    a.scenario_version += 1;
    assert!(w::context(&a, &s).is_err());
    let mut bad = s.clone();
    bad.objective.push_str(" altered");
    assert!(w::source(&bad).is_err());
    bad.initial = source(false).initial;
    assert!(
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![bad]
            })
            .unwrap()
        )
        .is_err()
    );
}

#[test]
fn unsupported_internal_input_and_wrong_goals_leave_existing_state_unchanged() {
    let s = source(false);
    let mut a = fresh(&s);
    let before = a.state.clone();
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "ssh localhost").is_err());
    assert_eq!(a.state, before);
    let old = original()
        .iter()
        .find(|s| s.id == "ccna-arp-sender-binding")
        .unwrap();
    assert!(engine::execute_for_goal(&mut a.state, &old.goal, &s.solution[0]).is_err());
    assert_eq!(a.state, before);
    let State::AaaAccess { simulation } = &mut a.state else {
        panic!()
    };
    let d = w::source(&s).unwrap();
    simulation.fill_example(d).unwrap();
    simulation.practice.check(&d.example).unwrap();
    assert!(!progress::current_solved(&a, &s));
}

#[test]
fn private_save_export_restore_preserve_drafts_old_arp_work_and_selected_reset() {
    let mut catalog = original().clone();
    catalog.extend([source(false), source(true)]);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &catalog[358..] {
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let d = w::source(s).unwrap();
        let State::AaaAccess { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.inspect(&d.initial).unwrap();
        simulation.fill_example(d).unwrap();
        simulation.practice.draft = "ip ssh version 2\nline vty unfinished\n".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let old = catalog
        .iter()
        .find(|s| s.id == "ccna-arp-static-policy")
        .unwrap();
    let mut retained = Attempt::new(old, 7);
    workspace::prepare(&mut retained);
    let State::ArpInspection { simulation } = &mut retained.state else {
        panic!()
    };
    simulation.assignment.draft.filters[0].rules[0].mac = "unfinished MAC draft".into();
    let retained_value = serde_json::to_value(&retained).unwrap();
    p.attempts.insert(old.id.clone(), retained);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("progress.json");
    progress::save_with_catalog(&file, &p, &catalog).unwrap();
    let before = std::fs::read(&file).unwrap();
    let loaded = progress::load(&file, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        retained_value
    );
    for s in &catalog[358..] {
        assert!(progress::current_solved(&loaded.attempts[&s.id], s));
        let State::AaaAccess { simulation } = &loaded.attempts[&s.id].state else {
            panic!()
        };
        assert!(simulation.assignment.observation.is_none());
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &catalog).unwrap();
    let inspected = backup::inspect(
        &export,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let restored = backup::restore(&file, &inspected, &catalog).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        before
    );
    let first = &catalog[358];
    p.attempts.insert(first.id.clone(), fresh(first));
    assert!(!progress::current_solved(&p.attempts[&first.id], first));
    assert!(progress::current_solved(
        &p.attempts[&catalog[359].id],
        &catalog[359]
    ));
    assert!(p.favorites.contains(&first.id));
    assert_eq!(
        serde_json::to_value(&p.attempts[&old.id]).unwrap(),
        retained_value
    );
}

#[test]
fn missing_catalog_and_incompatible_workspace_fail_before_writing_progress() {
    let s = source(false);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), fresh(&s));
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no-catalog.json");
    assert!(progress::save(&file, &p).is_err());
    assert!(!file.exists());
    let mut a = fresh(&s);
    a.state = original()
        .iter()
        .find(|s| s.id == "ccna-arp-sender-binding")
        .unwrap()
        .initial
        .clone();
    assert!(w::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
}
