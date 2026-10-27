//! Prospective teaching/context tests; offline simulation only, no authored new labs.
use dlms_learning_lab::{
    engine::{self, State},
    nat::Action,
    nat_teaching::{self as teach, Learning, Lesson},
    nat_workspace as work,
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    workspace,
};
#[path = "support/actual9_evidence.rs"]
mod evidence;
fn scenario() -> Scenario {
    pack::load(include_str!("fixtures/nat-teaching-preview.json"))
        .unwrap()
        .remove(0)
}
fn lesson(s: &Scenario) -> &Lesson {
    s.nat_teaching.as_ref().unwrap()
}
fn sim(s: &State) -> &dlms_learning_lab::nat::State {
    let State::BasicNat { simulation } = s else {
        panic!()
    };
    simulation
}
fn attempt(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    teach::prepare(&mut a, lesson(s), true).unwrap();
    a
}
fn solve(s: &Scenario, a: &mut Attempt) {
    for command in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    a.workspace.as_mut().unwrap().nat_draft = Some(work::Draft::new(sim(&a.state)).unwrap());
    assert!(progress::current_solved(a, s));
}
fn worked(l: &Lesson) -> Learning {
    let mut v = Learning::new(l).unwrap();
    for step in &l.example_actions {
        work::perform(&l.fixture, &mut v.simulation, &step.action).unwrap();
    }
    v.draft = work::Draft::new(&v.simulation).unwrap();
    v
}
#[test]
fn separate_example_worked_adverse_repair_and_assignment_isolation() {
    let s = scenario();
    let l = lesson(&s);
    l.validate_for(sim(&s.initial)).unwrap();
    let a = attempt(&s);
    let before = serde_json::to_vec(&a).unwrap();
    let mut p = worked(l);
    assert!(p.checks(l).iter().all(|r| r.0));
    for action in &l.try_change.actions {
        work::perform(&l.fixture, &mut p.simulation, action).unwrap();
    }
    p.draft = work::Draft::new(&p.simulation).unwrap();
    assert_eq!(
        p.checks(l).iter().map(|r| r.0).collect::<Vec<_>>(),
        l.try_change.expected
    );
    for action in &l.repair {
        work::perform(&l.fixture, &mut p.simulation, action).unwrap();
    }
    assert!(p.checks(l).iter().all(|r| r.0));
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn insertion_is_not_execution_and_each_probe_remains_explicit() {
    let s = scenario();
    let l = lesson(&s);
    let mut p = Learning::new(l).unwrap();
    let initial = p.simulation.clone();
    p.fill_selected(l).unwrap();
    assert_eq!(p.simulation, initial);
    assert!(!p.draft.matches(&p.simulation));
    p.draft.apply(&p.fixture, &mut p.simulation).unwrap();
    assert!(p.simulation.bindings().is_empty());
    p.selected_action = 1;
    p.fill_selected(l).unwrap();
    assert!(p.simulation.bindings().is_empty());
    let a = &l.example_actions[1].action;
    work::perform(&p.fixture, &mut p.simulation, a).unwrap();
    assert_eq!(p.simulation.bindings().len(), 1);
}
#[test]
fn semantically_wrong_example_cannot_revoke_true_assignment_and_broken_assignment_still_fails() {
    let s = scenario();
    let mut a = attempt(&s);
    solve(&s, &mut a);
    let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
    l.draft.network = "unfinished example policy".into();
    assert!(l.valid());
    assert!(progress::current_solved(&a, &s));
    let State::BasicNat { simulation } = &mut a.state else {
        panic!()
    };
    work::perform(sim(&s.initial), simulation, &Action::ClearDynamic {}).unwrap();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn initialization_retains_existing_raw_work_and_old_first_open_is_assignment() {
    let s = scenario();
    let mut old = Attempt::new(&s, 9);
    workspace::prepare(&mut old);
    teach::prepare(&mut old, lesson(&s), false).unwrap();
    assert_eq!(
        old.workspace
            .as_ref()
            .unwrap()
            .nat_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let mut new = attempt(&s);
    let l = new
        .workspace
        .as_mut()
        .unwrap()
        .nat_learning
        .as_mut()
        .unwrap();
    assert_eq!(l.view, View::Introduction);
    l.view = View::Example;
    l.draft.network = "retained malformed practice".into();
    l.draft.inbound_global = "unparsed observation".into();
    l.checked = true;
    let before = serde_json::to_vec(&new).unwrap();
    teach::prepare(&mut new, lesson(&s), false).unwrap();
    assert_eq!(serde_json::to_vec(&new).unwrap(), before);
}
#[test]
fn list_readers_refuse_seventeenth_typed_body_and_all_direct_empty_lists() {
    let s = scenario();
    let l = lesson(&s);
    for key in ["example_actions", "repair", "try_change.actions"] {
        let mut raw = serde_json::to_value(l).unwrap();
        let xs = if key == "try_change.actions" {
            raw["try_change"]["actions"].as_array_mut().unwrap()
        } else {
            raw[key].as_array_mut().unwrap()
        };
        let v = xs[0].clone();
        *xs = vec![v; 16];
        xs.push(serde_json::json!({"invalid_typed_body":true}));
        let err = serde_json::from_value::<Lesson>(raw)
            .unwrap_err()
            .to_string();
        assert!(err.contains("collection bound"), "{key}: {err}");
        let mut direct = l.clone();
        match key {
            "example_actions" => direct.example_actions.clear(),
            "repair" => direct.repair.clear(),
            _ => direct.try_change.actions.clear(),
        };
        assert!(direct.validate().is_err());
    }
}
#[test]
fn remaining_field_construction_limits_and_strict_wire() {
    let s = scenario();
    let l = lesson(&s);
    for (field, n) in [("prerequisites", 16), ("concepts", 6)] {
        let mut raw = serde_json::to_value(l).unwrap();
        let v = if field == "prerequisites" {
            serde_json::json!("x")
        } else {
            raw[field][0].clone()
        };
        raw[field] = serde_json::Value::Array(vec![v; n]);
        raw[field]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"bad":true}));
        assert!(
            serde_json::from_value::<Lesson>(raw)
                .unwrap_err()
                .to_string()
                .contains("collection bound")
        );
    }
    let mut raw = serde_json::to_value(l).unwrap();
    raw["try_change"]["expected"] = serde_json::json!(vec![true; 38]);
    assert!(serde_json::from_value::<Lesson>(raw).is_err());
    for field in ["unknown", "version"] {
        let mut raw = serde_json::to_value(l).unwrap();
        raw[field] = serde_json::json!(1);
        assert!(serde_json::from_value::<Lesson>(raw).is_err());
    }
    let mut p = serde_json::to_value(Learning::new(l).unwrap()).unwrap();
    p["version"] = serde_json::json!(2);
    assert!(serde_json::from_value::<Learning>(p).is_err());
    let raw = serde_json::to_string(l).unwrap();
    let duplicate = raw.replacen("{", "{\"prerequisites\":[],", 1);
    assert!(serde_json::from_str::<Lesson>(&duplicate).is_err());
}
#[test]
fn raw_scalar_action_and_aggregate_limits_precede_candidate_publication() {
    let s = scenario();
    let l = lesson(&s);
    assert!(
        Lesson::from_json(&vec![b' '; teach::MAX_LESSON + 1])
            .unwrap_err()
            .contains("raw bound")
    );
    assert!(
        Learning::from_json(&vec![b' '; teach::MAX_LEARNING + 1])
            .unwrap_err()
            .contains("raw bound")
    );
    let mut x = l.clone();
    x.prerequisite_note = " ".repeat(1601);
    assert!(x.validate().is_err());
    let mut x = l.clone();
    x.example_actions[0].action = Action::Outbound {
        inside: "x".repeat(65537),
        peer: "demo-peer".into(),
    };
    assert!(x.validate().is_err());
    let mut x = l.clone();
    x.repair = vec![Action::ClearDynamic {}; 17];
    assert!(x.validate().is_err());
    let mut p = worked(l);
    p.draft.network = "a".repeat(65);
    assert!(!p.valid());
    assert!(!work::within(&p, 1));
}
#[test]
fn disjoint_origin_and_wrong_lesson_reject_before_prepare_or_credit() {
    let s = scenario();
    let mut a = attempt(&s);
    solve(&s, &mut a);
    let mut other = lesson(&s).clone();
    other.fixture = sim(&s.initial).clone();
    other.definition.initial = other.fixture.clone();
    assert!(!other.separate_from(sim(&s.initial)));
    let before = serde_json::to_vec(&a).unwrap();
    assert!(teach::prepare(&mut a, &other, false).is_err());
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    let mut bad = a.clone();
    bad.workspace
        .as_mut()
        .unwrap()
        .nat_learning
        .as_mut()
        .unwrap()
        .fixture = sim(&s.initial).clone();
    assert!(!progress::current_solved(&bad, &s));
    let mut changed = s.clone();
    changed.nat_teaching.as_mut().unwrap().example_actions.pop();
    assert!(!progress::current_solved(&a, &changed));
}
#[test]
fn foreign_practice_rejects_current_save_decode_and_good_file_is_unchanged() {
    let s = scenario();
    let all = pack::bundled().unwrap();
    let old = all
        .iter()
        .find(|s| matches!(s.initial, State::Script { .. }))
        .unwrap()
        .clone();
    let mut a = Attempt::new(&old, 0);
    workspace::prepare(&mut a);
    let reference = old.solution.join("\n");
    let engine::Goal::Script { cases } = &old.goal else {
        panic!()
    };
    a.state = State::Script {
        source: reference.clone(),
        run: Some(dlms_learning_lab::shell::Record {
            engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            source: reference.clone(),
            results: cases
                .iter()
                .map(|case| dlms_learning_lab::shell::run(&reference, case))
                .collect(),
        }),
    };
    assert!(
        progress::current_solved(&a, &old),
        "Positive current simulated-script control"
    );
    a.workspace.as_mut().unwrap().nat_learning = Some(Learning::new(lesson(&s)).unwrap());
    let before = serde_json::to_vec(&a).unwrap();
    assert!(!progress::current_solved(&a, &old));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let good = std::fs::read(&path).unwrap();
    p.attempts.insert(old.id.clone(), a.clone());
    assert!(progress::save(&path, &p).is_err());
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), good);
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
}
#[test]
fn old_140_wire_and_views_are_unchanged_and_technical_fixture_unregistered() {
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 143);
    assert!(all[..140].iter().all(|s| s.nat_teaching.is_none()));
    assert_eq!(
        all[140..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        ["ccna-nat-static", "ccna-nat-pool", "ccna-nat-handover"]
    );
    assert!(all[140..].iter().all(|s| s.nat_teaching.is_some()));
    assert!(all.iter().all(|s| s.id != "nat-teaching-preview"));
    let w = workspace::WorkspaceData::new();
    assert!(
        serde_json::to_value(w)
            .unwrap()
            .get("nat_learning")
            .is_none()
    );
    assert!(all[..140].iter().all(|s| {
        serde_json::to_value(s)
            .unwrap()
            .get("nat_teaching")
            .is_none()
    }));
}
#[test]
fn whole_selected_practice_backup_restore_and_genuine_678_history_retention() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let mut all = pack::bundled().unwrap();
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        &all,
    )
    .unwrap();
    for a in p.attempts.values_mut() {
        workspace::prepare(a);
    }
    let originals = serde_json::to_value(&p.attempts).unwrap();
    let s = scenario();
    let mut a = attempt(&s);
    solve(&s, &mut a);
    let l = a.workspace.as_mut().unwrap().nat_learning.as_mut().unwrap();
    *l = worked(lesson(&s));
    l.view = View::Example;
    l.draft.prefix = "unfinished".into();
    l.draft.inbound_global = "retained query".into();
    l.checked = true;
    let selected = serde_json::to_value(&a).unwrap();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    all.push(s.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    for (id, a) in originals.as_object().unwrap() {
        evidence::exact(&loaded.attempts[id], a);
    }
    evidence::exact(&loaded.attempts[&s.id], &selected); // Only backup inspection's established script-practice two-field refresh may differ.
    let mut expected = serde_json::to_value(&loaded.attempts).unwrap();
    for (id, a) in &loaded.attempts {
        if let Some(l) = a
            .workspace
            .as_ref()
            .and_then(|w| w.script_learning.as_ref())
            && l.example_result.is_some()
        {
            let old = all
                .iter()
                .find(|s| &s.id == id)
                .unwrap()
                .teaching
                .as_ref()
                .unwrap();
            expected[id]["workspace"]["script_learning"]["example_result"] = serde_json::to_value(
                dlms_learning_lab::shell::run(&l.example_source, &old.fixture),
            )
            .unwrap();
            expected[id]["workspace"]["script_learning"]["example_revision"] =
                dlms_learning_lab::shell::EVALUATOR_REVISION.into();
        }
    }
    let candidate =
        backup::inspect(&path, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    evidence::exact(&candidate.progress.attempts, &expected);
    evidence::exact(&candidate.progress.attempts[&s.id], &selected);
    let out = dir.path().join("restored.json");
    backup::restore(&out, &candidate, &all).unwrap();
    let restored = progress::load(&out, &all).unwrap();
    evidence::exact(&restored.attempts, &expected);
    assert_eq!(restored.favorites, loaded.favorites);
    assert_eq!(restored.clock, loaded.clock);
    let mut reset = restored;
    reset.attempts.insert(s.id.clone(), attempt(&s));
    for (id, a) in expected.as_object().unwrap() {
        if id != &s.id {
            evidence::exact(&reset.attempts[id], a);
        }
    }
    assert!(reset.favorites.contains(&s.id));
    assert!(!progress::current_solved(&reset.attempts[&s.id], &s));
}
#[test]
#[ignore = "Requires immutable accepted private public9 COMPLETE; no fixture generation"]
fn actual9_profile_plus_practice_retains_every_old_attempt_raw_target_and_receipts() {
    let raw = evidence::original9();
    let mut all = pack::bundled().unwrap();
    let mut p = progress::decode(&raw, &all).unwrap();
    let original = serde_json::to_value(&p.attempts).unwrap();
    let s = scenario();
    p.attempts.insert(s.id.clone(), attempt(&s));
    let selected = serde_json::to_value(&p.attempts[&s.id]).unwrap();
    all.push(s.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    std::fs::write(&path, &raw).unwrap();
    let old7 = include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json");
    std::fs::write(path.with_extension("before-shell-v8.json"), old7).unwrap();
    progress::save(&path, &p).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v10.json")).unwrap(),
        raw
    );
    assert_eq!(
        std::fs::read(path.with_extension("before-shell-v8.json")).unwrap(),
        old7
    );
    let loaded = progress::load(&path, &all).unwrap();
    for (id, a) in original.as_object().unwrap() {
        evidence::exact(&loaded.attempts[id], a);
    }
    evidence::exact(&loaded.attempts[&s.id], &selected);
    assert_eq!(loaded.attempts.len(), 134);
    let receipts = serde_json::to_value(loaded.migration_receipts).unwrap();
    assert_eq!(receipts["shell"], serde_json::json!([7, 8, 9, 10]));
    assert_eq!(evidence::original9(), raw);
}
