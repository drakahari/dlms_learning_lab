use dlms_learning_lab::{
    arp_inspection_workspace as w, course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "ccna-arp-sender-binding",
    "ccna-arp-static-policy",
    "ccna-arp-field-consistency",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
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
fn old351_scenarios124_units_are_exact_and_new_progression_follows_dhcp() {
    let old = pack::accepted351().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (351, 354));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            now.iter()
                .filter(|s| !IDS.contains(&s.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let before = course::accepted351(&old).unwrap();
    let current = course::ccna_pilot(now).unwrap();
    assert_eq!((before.len(), current.len()), (124, 125));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(
            current
                .iter()
                .filter(|u| u.id != "arp-inspection-foundations")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = current
        .iter()
        .position(|u| u.id == "arp-inspection-foundations")
        .unwrap();
    assert_eq!(current[at - 1].id, "dhcp-snooping-foundations");
    assert_eq!(current[at].labs, IDS);
    for (track, n, units) in [
        ("CCNA", 82, 23),
        ("LPIC-1", 117, 45),
        ("LPIC-2", 78, 31),
        ("Bash/Shell", 77, 26),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
        assert_eq!(current.iter().filter(|u| u.track == track).count(), units);
    }
    assert_eq!(lab(IDS[0]).prerequisites, ["ccna-dhcp-client-mac-check"]);
    assert_eq!(lab(IDS[1]).prerequisites, [IDS[0]]);
    assert_eq!(lab(IDS[2]).prerequisites, [IDS[1]]);
    let definitions = IDS.map(|id| w::source(lab(id)).unwrap());
    assert_ne!(definitions[0].initial.cases, definitions[1].initial.cases);
    assert_ne!(definitions[1].initial.cases, definitions[2].initial.cases);
    assert!(
        definitions[1]
            .initial
            .cases
            .iter()
            .any(|c| c.label.contains("Static workstation"))
    );
    assert!(
        definitions[2]
            .initial
            .cases
            .iter()
            .any(|c| c.label.contains("target fields"))
    );
}
#[test]
fn authored_solutions_equivalent_policies_partial_and_broken_work_use_actual_outcomes() {
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        assert!(!progress::current_solved(&a, s));
        for answer in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, answer).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let State::ArpInspection { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.draft.ports.reverse();
        simulation.assignment.draft.vlans = "20, 10".into();
        simulation.assignment.draft.checks.ip = true;
        simulation.assignment.apply().unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, s));
        let State::ArpInspection { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.draft.vlans.clear();
        assert!(d.checks(simulation).iter().any(|r| !r.0));
        simulation.assignment.apply().unwrap();
        simulation.assignment.checked = true;
        assert!(!progress::current_solved(&a, s));
    }
    let s = lab(IDS[2]);
    let d = w::source(s).unwrap();
    let mut a = fresh(s);
    let State::ArpInspection { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.draft.checks.source_mac = true;
    simulation.assignment.apply().unwrap();
    let rows = simulation.assignment.check(&d.initial).unwrap();
    assert!(rows.iter().any(|r| r.0));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("destination")));
    assert!(!progress::current_solved(&a, s));
    let s = lab(IDS[1]);
    let d = w::source(s).unwrap();
    let mut a = fresh(s);
    let State::ArpInspection { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.draft.filters[0].rules[0].mac =
        d.initial.reference.filters[0].rules[0].sender_mac.text();
    simulation.assignment.apply().unwrap();
    let rows = simulation.assignment.check(&d.initial).unwrap();
    assert!(rows.iter().any(|r| r.0));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("skips DHCP records"))
    );
}
#[test]
fn real_practice_drafts_resume_without_assignment_credit_and_reset_is_selected() {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        let State::ArpInspection { simulation } = &mut a.state else {
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
    let old = lab("ccna-dhcp-server-boundary");
    let mut a = Attempt::new(old, 7);
    workspace::prepare(&mut a);
    let State::DhcpSnooping { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.draft.vlans = "unfinished DHCP draft".into();
    let old_value = serde_json::to_value(&a).unwrap();
    p.attempts.insert(old.id.clone(), a);
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
        old_value
    );
    p.attempts.insert(IDS[0].into(), fresh(lab(IDS[0])));
    for id in &IDS[1..] {
        let State::ArpInspection { simulation } = &p.attempts[*id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.draft.vlans, "10,");
    }
    assert!(p.favorites.contains(IDS[0]));
}
#[test]
fn source_tampering_and_forged_checked_flags_never_create_mastery() {
    for id in IDS {
        let s = lab(id);
        let mut a = fresh(s);
        let State::ArpInspection { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.checked = true;
        assert!(!progress::current_solved(&a, s));
        let mut bad = s.clone();
        bad.objective.push_str(" changed");
        assert!(w::source(&bad).is_err());
    }
}
