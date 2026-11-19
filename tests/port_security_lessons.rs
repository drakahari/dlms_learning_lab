use dlms_learning_lab::{
    backup, course,
    engine::{self, State},
    pack,
    port_security::ViolationMode as V,
    port_security_workspace as w,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const IDS: [&str; 3] = [
    "ccna-security-identity",
    "ccna-security-observe",
    "ccna-security-recover",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted349().unwrap())
}
fn lab(id: &str) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn all346_existing_scenarios_and122_units_are_preserved_and_new_unit_follows_prerequisites() {
    let before = pack::accepted346().unwrap();
    let now = catalog();
    assert_eq!((before.len(), now.len()), (346, 349));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(
            now.iter()
                .filter(|s| !IDS.contains(&s.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let old = course::accepted346(&before).unwrap();
    let current = course::accepted349(now).unwrap();
    assert_eq!((old.len(), current.len()), (122, 123));
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(
            current
                .iter()
                .filter(|u| u.id != "secure-access-ports")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = current
        .iter()
        .position(|u| u.id == "secure-access-ports")
        .unwrap();
    assert_eq!(current[at - 1].id, "ethernet-learning");
    assert_eq!(current[at].labs, IDS);
    for (track, n, u) in [
        ("CCNA", 77, 21),
        ("LPIC-1", 117, 45),
        ("LPIC-2", 78, 31),
        ("Bash/Shell", 77, 26),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
        assert_eq!(current.iter().filter(|s| s.track == track).count(), u);
    }
    assert_eq!(lab(IDS[0]).prerequisites, ["ccna-mac-handover"]);
    assert_eq!(lab(IDS[1]).prerequisites, [IDS[0]]);
    assert_eq!(lab(IDS[2]).prerequisites, [IDS[1]]);
}
#[test]
fn authored_solutions_alternates_partial_work_and_broken_state_are_graded_honestly() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        assert!(!progress::current_solved(&a, s));
        let State::PortSecurity { simulation } = &mut a.state else {
            panic!()
        };
        let partial = simulation.assignment.check(&d.initial).unwrap();
        assert!(partial.iter().any(|r| r.0));
        assert!(partial.iter().any(|r| !r.0));
        for answer in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, answer).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let State::PortSecurity { simulation } = &mut a.state else {
            panic!()
        };
        let current = simulation.assignment.world.port("desk").unwrap().clone();
        if id != IDS[1] {
            simulation
                .assignment
                .world
                .configure("desk", true, 1, V::Restrict, current.configured.clone())
                .unwrap();
            simulation.assignment.drafts = simulation
                .assignment
                .world
                .ports()
                .iter()
                .map(w::Draft::new)
                .collect();
            simulation.assignment.check(&d.initial).unwrap();
            assert!(d.checks(simulation).iter().all(|r| r.0));
        }
        simulation.assignment.admin("desk", false).unwrap();
        assert!(!progress::current_solved(&a, s));
        let State::PortSecurity { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.check(&d.initial).unwrap();
        assert!(d.checks(simulation).iter().any(|r| !r.0));
        simulation.assignment.admin("desk", true).unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn saved_practice_drafts_favorites_and_legacy_work_survive_private_export_restore() {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        for answer in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, answer).unwrap();
        }
        let State::PortSecurity { simulation } = &mut a.state else {
            panic!()
        };
        simulation.practice.fill(&d.example).unwrap();
        simulation.practice.check(&d.example).unwrap();
        simulation.assignment.source = "unfinished replay draft".into();
        p.attempts.insert(id.into(), a);
        p.favorites.insert(id.into());
    }
    let old = lab("lpic1-aging-contract-reset");
    let mut old_attempt = Attempt::new(old, 5);
    workspace::prepare(&mut old_attempt);
    let State::Aging { simulation } = &mut old_attempt.state else {
        panic!()
    };
    simulation.assignment.input = "chage -l unfinished_old_draft".into();
    let old_value = serde_json::to_value(&old_attempt).unwrap();
    p.attempts.insert(old.id.clone(), old_attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let original = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for id in IDS {
        assert!(progress::current_solved(&loaded.attempts[id], lab(id)));
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, catalog()).unwrap();
    let inspected = backup::inspect(
        &export,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let receipt = backup::restore(&path, &inspected, catalog()).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    p.attempts.insert(IDS[0].into(), fresh(lab(IDS[0])));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], lab(IDS[0])));
    assert!(progress::current_solved(&p.attempts[IDS[1]], lab(IDS[1])));
    assert_eq!(
        serde_json::to_value(&p.attempts[&old.id]).unwrap(),
        old_value
    );
    assert!(p.favorites.contains(IDS[0]));
}
#[test]
fn source_mismatch_and_untrusted_imported_checked_flags_do_not_award_credit() {
    let s = lab(IDS[0]);
    let mut a = fresh(s);
    let State::PortSecurity { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.checked = true;
    assert!(!progress::current_solved(&a, s));
    let State::PortSecurity { simulation } = &mut a.state else {
        panic!()
    };
    simulation.origin.0 = IDS[1].into();
    assert!(w::context(&a, s).is_err());
    assert!(!progress::current_solved(&a, s));
    let mut wrong = s.clone();
    wrong.objective.push_str(" changed");
    assert!(w::source(&wrong).is_err());
}
