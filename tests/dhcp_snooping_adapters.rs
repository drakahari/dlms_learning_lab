//! Synthetic source/context fixtures only. These are not registered authored labs.
use dlms_learning_lab::{
    backup, dhcp_snooping_workspace as w,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
#[path = "support/snoop_tasks.rs"]
mod common;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
fn original() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted349().unwrap())
}
fn source(kind: u8) -> pack::Scenario {
    let d = common::definition(kind);
    let mut s = original()
        .iter()
        .find(|s| s.id == "ccna-security-identity")
        .unwrap()
        .clone();
    s.id = d.initial.id.0.clone();
    s.objective = d.initial.instructions.clone();
    s.prerequisites.clear();
    s.solution = vec![format!(
        "{}{}",
        w::PREFIX,
        serde_json::to_string(&d.initial.reference).unwrap()
    )];
    s.initial = State::DhcpSnooping {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    s.goal = Goal::DhcpSnooping {
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
fn source_data_loads_and_reference_runs_through_the_real_engine_without_catalog_registration() {
    assert_eq!(original().len(), 349);
    for kind in [1, 2] {
        let s = source(kind);
        let text = serde_json::to_string(&pack::Pack {
            schema_version: 1,
            scenarios: vec![s.clone()],
        })
        .unwrap();
        let parsed = pack::load(&text).unwrap();
        assert_eq!(parsed.len(), 1);
        let mut a = fresh(&parsed[0]);
        assert!(!progress::current_solved(&a, &s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, &s));
    }
}
#[test]
fn checked_flags_bad_drafts_and_wrong_context_cannot_retain_current_success() {
    let s = source(1);
    let mut a = fresh(&s);
    let State::DhcpSnooping { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.checked = true;
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, &s));
    let State::DhcpSnooping { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.draft.vlans = "unfinished".into();
    assert!(!progress::current_solved(&a, &s));
    let State::DhcpSnooping { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin.0 = "another-source".into();
    assert!(w::context(&a, &s).is_err());
    assert!(!progress::current_solved(&a, &s));
    let mut a = fresh(&s);
    a.scenario_version += 1;
    assert!(w::context(&a, &s).is_err());
    let mut bad = s.clone();
    bad.objective.push_str(" changed");
    assert!(w::source(&bad).is_err());
    bad.initial = source(2).initial;
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
fn private_save_export_validated_restore_preserve_legacy_work_favorites_and_separate_drafts() {
    let mut catalog = original().clone();
    catalog.extend([source(1), source(2)]);
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &catalog[349..] {
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let d = w::source(s).unwrap();
        let State::DhcpSnooping { simulation } = &mut a.state else {
            panic!()
        };
        simulation.practice.fill(&d.example).unwrap();
        simulation.practice.run_sample(&d.example).unwrap();
        simulation.practice.draft.vlans = "20,".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let legacy = catalog
        .iter()
        .find(|s| s.id == "lpic1-aging-contract-reset")
        .unwrap();
    let mut old = Attempt::new(legacy, 7);
    workspace::prepare(&mut old);
    let State::Aging { simulation } = &mut old.state else {
        panic!()
    };
    simulation.assignment.input = "chage -l unfinished_old_draft".into();
    let old_value = serde_json::to_value(&old).unwrap();
    p.attempts.insert(legacy.id.clone(), old);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let before = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&legacy.id]).unwrap(),
        old_value
    );
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &catalog).unwrap();
    let inspected = backup::inspect(
        &export,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let restored = backup::restore(&path, &inspected, &catalog).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        before
    );
    let first = &catalog[349];
    p.attempts.insert(first.id.clone(), fresh(first));
    assert!(!progress::current_solved(&p.attempts[&first.id], first));
    assert!(progress::current_solved(
        &p.attempts[&catalog[350].id],
        &catalog[350]
    ));
    assert!(p.favorites.contains(&first.id));
    assert_eq!(
        serde_json::to_value(&p.attempts[&legacy.id]).unwrap(),
        old_value
    );
}
#[test]
fn missing_authored_catalog_and_foreign_practice_equipment_fail_safely() {
    let s = source(1);
    let mut a = fresh(&s);
    let d = w::source(&s).unwrap();
    let State::DhcpSnooping { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.world = d.example.initial.clone();
    assert!(w::context(&a, &s).is_err());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), fresh(&s));
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no-catalog.json");
    assert!(progress::save(&file, &p).is_err());
    assert!(!file.exists());
}
