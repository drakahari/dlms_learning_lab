use dlms_learning_lab::{
    course, dhcp_snooping_workspace as w,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["ccna-dhcp-server-boundary", "ccna-dhcp-client-mac-check"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted351().unwrap())
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
fn all349_previous_scenarios_and123_units_remain_exact_and_new_prerequisites_are_ordered() {
    let old = pack::accepted349().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (349, 351));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            now.iter()
                .filter(|s| !IDS.contains(&s.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let before = course::accepted349(&old).unwrap();
    let current = course::accepted351(now).unwrap();
    assert_eq!((before.len(), current.len()), (123, 124));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(
            current
                .iter()
                .filter(|u| u.id != "dhcp-snooping-foundations")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = current
        .iter()
        .position(|u| u.id == "dhcp-snooping-foundations")
        .unwrap();
    assert_eq!(current[at - 1].id, "secure-access-ports");
    assert_eq!(current[at].labs, IDS);
    for (track, n, u) in [
        ("CCNA", 79, 22),
        ("LPIC-1", 117, 45),
        ("LPIC-2", 78, 31),
        ("Bash/Shell", 77, 26),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
        assert_eq!(current.iter().filter(|s| s.track == track).count(), u);
    }
    assert_eq!(
        lab(IDS[0]).prerequisites,
        ["ccna-security-recover", "ccna-dhcp-lease-handover"]
    );
    assert_eq!(lab(IDS[1]).prerequisites, [IDS[0]]);
    let first = w::source(lab(IDS[0])).unwrap();
    let second = w::source(lab(IDS[1])).unwrap();
    assert_ne!(first.initial.cases, second.initial.cases);
    assert!(
        first
            .initial
            .cases
            .iter()
            .any(|c| c.label.contains("Offer does not"))
    );
    assert!(
        second
            .initial
            .cases
            .iter()
            .any(|c| c.label.contains("not authentication"))
    );
}
#[test]
fn authored_solutions_supported_alternatives_wrong_and_partial_policies_are_real_outcomes() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        assert!(!progress::current_solved(&a, s));
        for answer in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, answer).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let State::DhcpSnooping { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.draft.ports.reverse();
        simulation.assignment.apply().unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::DhcpSnooping { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.draft.enabled = false;
        assert!(!progress::current_solved(&a, s));
        let State::DhcpSnooping { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.apply().unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        assert!(d.checks(simulation).iter().any(|r| !r.0));
        assert!(!progress::current_solved(&a, s));
    }
    let s = lab(IDS[1]);
    let d = w::source(s).unwrap();
    let mut a = fresh(s);
    let State::DhcpSnooping { simulation } = &mut a.state else {
        panic!()
    };
    let rows = simulation.assignment.check(&d.initial).unwrap();
    assert!(rows.iter().any(|r| r.0));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("MAC addresses disagree"))
    );
}
#[test]
fn real_authored_practice_drafts_resume_without_earning_assignment_credit_and_reset_is_selected() {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        let State::DhcpSnooping { simulation } = &mut a.state else {
            panic!()
        };
        simulation.practice.fill(&d.example).unwrap();
        simulation.practice.run_sample(&d.example).unwrap();
        simulation.practice.check(&d.example).unwrap();
        assert!(d.checks(simulation).iter().any(|r| !r.0));
        simulation.assignment.draft.vlans = "10,".into();
        p.attempts.insert(id.into(), a);
        p.favorites.insert(id.into());
    }
    let old = lab("ccna-security-recover");
    let old_attempt = Attempt::new(old, 7);
    let preserved = serde_json::to_value(&old_attempt).unwrap();
    p.attempts.insert(old.id.clone(), old_attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let loaded = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&old.id]).unwrap(),
        preserved
    );
    p.attempts.insert(IDS[0].into(), fresh(lab(IDS[0])));
    let State::DhcpSnooping { simulation } = &p.attempts[IDS[1]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.draft.vlans, "10,");
    assert!(p.favorites.contains(IDS[0]));
}
#[test]
fn source_tampering_and_checked_flags_never_fabricate_current_completion() {
    for id in IDS {
        let s = lab(id);
        let mut a = fresh(s);
        let State::DhcpSnooping { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.checked = true;
        assert!(!progress::current_solved(&a, s));
        let mut bad = s.clone();
        bad.objective.push_str(" changed");
        assert!(w::source(&bad).is_err());
    }
}
