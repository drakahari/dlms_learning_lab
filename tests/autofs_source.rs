//! Actual pack/engine/progress boundaries using a synthetic unregistered pair.
#[path = "support/autofs_fixture.rs"]
mod fixture;
use dlms_learning_lab::autofs::runtime::Name;
use dlms_learning_lab::{
    autofs_source as source, autofs_workspace as work,
    engine::{self, Goal, State},
    pack::{self, Pack},
    progress::{self, Attempt, Progress},
    workspace,
};
use fixture::*;
fn catalog() -> Vec<pack::Scenario> {
    pack::load(
        &serde_json::to_string(&Pack {
            schema_version: 1,
            scenarios: vec![scenario(false), scenario(true)],
        })
        .unwrap(),
    )
    .unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    source::context(&a, s).unwrap();
    a
}
fn solve(s: &pack::Scenario) -> Attempt {
    let mut a = fresh(s);
    let (State::Autofs { simulation }, Goal::Autofs { definition }) = (&mut a.state, &s.goal)
    else {
        panic!()
    };
    for step in &definition.reference {
        simulation
            .apply(definition, &step.operation, false)
            .unwrap();
    }
    simulation.run(definition, work::CHECK, false).unwrap();
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn native_typed_reference_is_reachable_without_teaching_save_as_a_command() {
    let labs = catalog();
    assert_eq!(labs.len(), 2);
    for s in &labs {
        source::source(s).unwrap();
        assert!(!progress::current_solved(&fresh(s), s));
        let a = solve(s);
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn naked_and_cross_family_execution_cannot_mutate_source_bound_work() {
    let labs = catalog();
    let s = &labs[0];
    let mut a = fresh(s);
    let before = a.state.clone();
    assert!(engine::execute(&mut a.state, "systemctl reload autofs").is_err());
    assert_eq!(a.state, before);
    let foreign: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    assert!(engine::execute_for_goal(&mut a.state, &foreign.scenarios[0].goal, "help").is_err());
    assert_eq!(a.state, before);
    let mut other = foreign.scenarios[0].initial.clone();
    let original = other.clone();
    assert!(engine::execute_for_goal(&mut other, &s.goal, "help").is_err());
    assert_eq!(other, original);
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "cat /etc/auto.master").is_ok());
}
#[test]
fn save_reload_retain_drafts_actual_state_favorites_and_honest_credit() {
    let labs = catalog();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.favorites.insert(labs[0].id.clone());
    p.attempts.insert(labs[0].id.clone(), solve(&labs[0]));
    let mut a = fresh(&labs[1]);
    let State::Autofs { simulation } = &mut a.state else {
        panic!()
    };
    let Goal::Autofs { definition } = &labs[1].goal else {
        panic!()
    };
    simulation.assignment.selected = Name(DIRECT.into());
    simulation
        .edit(definition, "# unfinished assignment\n".into(), false)
        .unwrap();
    simulation.practice.selected = Name(DIRECT.into());
    simulation
        .edit(definition, "# separate example\n".into(), true)
        .unwrap();
    p.attempts.insert(labs[1].id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let q = progress::load(&path, &labs).unwrap();
    assert_eq!(q.favorites, p.favorites);
    assert!(progress::current_solved(&q.attempts[&labs[0].id], &labs[0]));
    assert!(!progress::current_solved(
        &q.attempts[&labs[1].id],
        &labs[1]
    ));
    assert_eq!(
        serde_json::to_value(&q.attempts[&labs[1].id]).unwrap(),
        serde_json::to_value(&p.attempts[&labs[1].id]).unwrap()
    );
    let bytes = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn corrupt_attempt_metadata_roles_and_effects_are_never_repaired_into_mastery() {
    let labs = catalog();
    let s = &labs[0];
    let good = solve(s);
    let mut variants = vec![];
    let mut a = good.clone();
    a.step = s.steps.len();
    variants.push(a);
    let mut a = good.clone();
    a.hints = s.hints.len() + 1;
    variants.push(a);
    let mut a = good.clone();
    a.scenario_version += 1;
    variants.push(a);
    let mut a = good.clone();
    a.workspace.as_mut().unwrap().command = "foreign legacy work".into();
    variants.push(a);
    let mut a = good.clone();
    let State::Autofs { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.history.clear();
    variants.push(a);
    let mut a = good.clone();
    let State::Autofs { simulation } = &mut a.state else {
        panic!()
    };
    std::mem::swap(&mut simulation.assignment, &mut simulation.practice);
    variants.push(a);
    for bad in variants {
        assert!(source::context(&bad, s).is_err());
        assert!(!progress::current_solved(&bad, s));
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), bad);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("retained");
        std::fs::write(&path, b"retain me").unwrap();
        assert!(progress::save_with_catalog(&path, &p, &labs).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"retain me");
    }
}
#[test]
fn changing_checked_runtime_or_source_clears_success_and_reset_restores_selected_work() {
    let labs = catalog();
    let s = &labs[0];
    let mut a = solve(s);
    let (State::Autofs { simulation }, Goal::Autofs { definition }) = (&mut a.state, &s.goal)
    else {
        panic!()
    };
    simulation
        .apply(
            definition,
            &save(INDIRECT, indirect("/dev/team/reference")),
            false,
        )
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    let before = a.state.clone();
    let mut rebound = s.clone();
    rebound.id = "another-source".into();
    assert!(source::context(&a, &rebound).is_err());
    assert_eq!(a.state, before);
    let reset = fresh(s);
    assert_eq!(reset.state, s.initial);
    assert!(!progress::current_solved(&reset, s));
}
