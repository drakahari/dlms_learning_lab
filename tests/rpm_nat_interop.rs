//! Mixed-family admission after merging the independently reviewed RPM and NAT adapters.
//! Only in-process fixed fixtures; no host utilities or manufactured run records.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    nat_teaching, nat_workspace, pack,
    progress::{self, Attempt, Progress},
    rpm_packages_teaching as rpm_teaching, workspace,
};
fn scenarios() -> [pack::Scenario; 2] {
    [
        pack::load(include_str!("fixtures/rpm-workspace-preview.json"))
            .unwrap()
            .remove(0),
        pack::load(include_str!("fixtures/nat-teaching-preview.json"))
            .unwrap()
            .remove(0),
    ]
}
fn exact<T: serde::Serialize>(value: &T, expected: &serde_json::Value) {
    assert!(
        serde_json::to_value(value).unwrap() == *expected,
        "Complete mixed-family caller differs"
    );
}
fn rpm_control(s: &pack::Scenario) -> Attempt {
    let Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    rpm_teaching::prepare(
        &mut a,
        definition,
        s.rpm_packages_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    let State::RpmPackages { simulation } = &mut a.state else {
        panic!()
    };
    let session = a.workspace.as_mut().unwrap().rpm_session.as_mut().unwrap();
    for c in &s.solution {
        session.insert(c).unwrap();
        assert!(matches!(
            session.run(definition, simulation).unwrap(),
            dlms_learning_lab::rpm_packages_terminal::Outcome::Utility { status: 0, .. }
        ));
    }
    assert!(session.valid_for(simulation));
    assert!(progress::current_rpm_checks(&a, s).iter().all(|r| r.0));
    assert!(progress::current_solved(&a, s));
    a
}
fn nat_control(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    nat_teaching::prepare(&mut a, s.nat_teaching.as_ref().unwrap(), true).unwrap();
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    let State::BasicNat { simulation } = &a.state else {
        panic!()
    };
    a.workspace.as_mut().unwrap().nat_draft = Some(nat_workspace::Draft::new(simulation).unwrap());
    assert!(progress::current_nat_checks(&a, s).iter().all(|r| r.0));
    assert!(progress::current_solved(&a, s));
    a
}
fn profile(s: &pack::Scenario, a: Attempt) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p
}
fn reject_without_writes(
    a: &Attempt,
    s: &pack::Scenario,
    all: &[pack::Scenario],
    target: &std::path::Path,
    prior: &[u8],
) {
    assert!(!progress::current_solved(a, s));
    let rows = match &s.goal {
        Goal::RpmPackages { .. } => progress::current_rpm_checks(a, s),
        Goal::BasicNat { .. } => progress::current_nat_checks(a, s),
        _ => panic!(),
    };
    assert!(!rows.is_empty());
    assert!(rows.iter().any(|r| !r.0));
    assert!(rows.iter().all(|r| r.1.len() <= 512));
    let p = profile(s, a.clone());
    let caller = serde_json::to_value(&p).unwrap();
    assert!(progress::save(target, &p).is_err());
    assert_eq!(std::fs::read(target).unwrap(), prior);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), all).is_err());
    exact(&p, &caller);
}
#[test]
fn both_real_positive_controls_save_decode_complete_sessions_and_learning() {
    let all = scenarios();
    let controls = [rpm_control(&all[0]), nat_control(&all[1])];
    let dir = tempfile::tempdir().unwrap();
    for (s, a) in all.iter().zip(controls) {
        assert!(a.workspace.as_ref().unwrap().valid());
        let caller = serde_json::to_value(&a).unwrap();
        let p = profile(s, a);
        let path = dir.path().join(format!("{}.json", s.id));
        progress::save(&path, &p).unwrap();
        let loaded = progress::decode(&std::fs::read(path).unwrap(), &all).unwrap();
        exact(&loaded.attempts[&s.id], &caller);
        assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    }
}
#[test]
fn individually_valid_nat_draft_and_learning_reject_rpm_without_mutation() {
    let all = scenarios();
    let good = rpm_control(&all[0]);
    let nat = nat_control(&all[1]);
    let foreign = nat.workspace.as_ref().unwrap();
    assert!(foreign.nat_draft.as_ref().unwrap().valid());
    assert!(foreign.nat_learning.as_ref().unwrap().valid());
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("keep.json");
    progress::save(&target, &profile(&all[0], good.clone())).unwrap();
    let prior = std::fs::read(&target).unwrap();
    for (draft, learning) in [(true, false), (false, true), (true, true)] {
        let mut a = good.clone();
        let w = a.workspace.as_mut().unwrap();
        if draft {
            w.nat_draft = foreign.nat_draft.clone();
        }
        if learning {
            w.nat_learning = foreign.nat_learning.clone();
        }
        assert!(w.valid(), "Foreign metadata must be individually valid");
        let caller = serde_json::to_value(&a).unwrap();
        reject_without_writes(&a, &all[0], &all, &target, &prior);
        let Goal::RpmPackages { definition } = &all[0].goal else {
            panic!()
        };
        assert!(
            rpm_teaching::prepare(
                &mut a,
                definition,
                all[0].rpm_packages_teaching.as_ref().unwrap(),
                false,
            )
            .is_err()
        );
        exact(&a, &caller);
    }
    assert!(progress::current_solved(&good, &all[0]));
}
#[test]
fn individually_valid_rpm_session_and_learning_reject_nat_without_mutation() {
    let all = scenarios();
    let good = nat_control(&all[1]);
    let rpm = rpm_control(&all[0]);
    let foreign = rpm.workspace.as_ref().unwrap();
    assert!(foreign.rpm_session.as_ref().unwrap().valid());
    assert!(foreign.rpm_learning.as_ref().unwrap().valid());
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("keep.json");
    progress::save(&target, &profile(&all[1], good.clone())).unwrap();
    let prior = std::fs::read(&target).unwrap();
    for (session, learning) in [(true, false), (false, true), (true, true)] {
        let mut a = good.clone();
        let w = a.workspace.as_mut().unwrap();
        if session {
            w.rpm_session = foreign.rpm_session.clone();
        }
        if learning {
            w.rpm_learning = foreign.rpm_learning.clone();
        }
        assert!(w.valid(), "Foreign metadata must be individually valid");
        let caller = serde_json::to_value(&a).unwrap();
        reject_without_writes(&a, &all[1], &all, &target, &prior);
        assert!(
            nat_teaching::prepare(&mut a, all[1].nat_teaching.as_ref().unwrap(), false).is_err()
        );
        exact(&a, &caller);
    }
    assert!(progress::current_solved(&good, &all[1]));
}
