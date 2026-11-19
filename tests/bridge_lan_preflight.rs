//! Standalone source gate before catalog registration; no lab-count credit.
use dlms_learning_lab::{engine::Goal, nm_workspace as w, pack};
fn source() -> pack::Scenario {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/bridge-lan-lessons-v1.json")).unwrap();
    serde_json::from_value(raw["scenarios"][0].clone()).unwrap()
}
#[test]
fn authored_bridge_source_has_three_real_cases_and_reachable_separate_practice() {
    let s = source();
    let d = w::source(&s).unwrap();
    assert_eq!(s.id, "lpic2-bridge-lan-commission");
    assert_eq!(d.initial.cases.len(), 3);
    assert_eq!(d.example.cases.len(), 3);
    assert_eq!(d.example_worked.len(), 4);
    let Goal::NmProfiles { definition } = &s.goal else {
        panic!()
    };
    assert_eq!(definition.as_ref(), d);
    let good: w::Files =
        serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap();
    assert!(
        d.initial
            .run(&good)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    assert!(
        d.example
            .run(&d.example_worked)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    for t in [&d.initial, &d.example] {
        assert_eq!(
            t.run(&t.starter)
                .unwrap()
                .iter()
                .filter(|r| r.passed())
                .count(),
            0
        );
        let mut partial = t.starter.clone();
        partial.insert(
            "bridge.nmconnection".into(),
            t.cases[0].current_files["bridge.nmconnection"].clone(),
        );
        assert_eq!(
            t.run(&partial)
                .unwrap()
                .iter()
                .map(w::CaseResult::passed)
                .collect::<Vec<_>>(),
            [false, false, true]
        );
    }
    assert_eq!(
        w::Session::new(d).unwrap().assignment.selected,
        "bridge.nmconnection"
    );
}
#[test]
fn independent_network_accepts_another_host_but_rejects_mask_collision_and_audit_changes() {
    let s = source();
    let d = w::source(&s).unwrap();
    let good: w::Files =
        serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap();
    let mut alternate = good.clone();
    alternate
        .get_mut("bridge.nmconnection")
        .unwrap()
        .push_str("\n[bridge]\nstp=false\n[ipv4]\naddress1=10.240.0.21/24\n");
    for name in ["port-enp1s0.nmconnection", "port-enp2s0.nmconnection"] {
        alternate
            .get_mut(name)
            .unwrap()
            .push_str("\n[connection]\nautoconnect=true\n");
    }
    assert!(
        d.initial
            .run(&alternate)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    for (file, tail) in [
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.240.0.21/25\n"),
        ("bridge.nmconnection", "\n[ipv4]\naddress1=10.240.0.50/24\n"),
        ("audit.nmconnection", "\n[ipv4]\naddress1=10.241.0.21/24\n"),
    ] {
        let mut wrong = good.clone();
        wrong.get_mut(file).unwrap().push_str(tail);
        assert!(d.initial.run(&wrong).unwrap().iter().any(|r| !r.passed()));
    }
}
