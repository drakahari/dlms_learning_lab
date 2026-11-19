use dlms_learning_lab::{arp_inspection as core, arp_inspection_workspace::*};
#[path = "support/arp_tasks.rs"]
mod common;
use common::definition;
#[test]
fn three_distinct_tasks_are_reachable_unfinished_and_graded_on_fresh_behaviors() {
    for kind in 1..=3 {
        let d = definition(kind);
        d.validate().unwrap();
        let before = d.initial.reference_world().unwrap();
        assert!(d.initial.checks(&d.initial.initial).iter().any(|r| !r.0));
        assert!(d.initial.checks(&before).iter().all(|r| r.0));
        assert_eq!(d.initial.reference_world().unwrap(), before);
        assert!(
            before.dhcp().bindings().is_empty(),
            "Checks must not populate displayed history"
        );
        assert!(d.initial.reports(&before).iter().all(|r| r.passed()));
    }
}
#[test]
fn practice_execution_credit_and_drafts_are_separate_and_resume_exactly() {
    for kind in 1..=3 {
        let d = definition(kind);
        let mut s = Session::new(&d).unwrap();
        let old = s.assignment.clone();
        s.practice.fill(&d.example).unwrap();
        s.practice.run_sample(&d.example).unwrap();
        assert!(!s.practice.output.is_empty());
        assert_eq!(s.practice.world.dhcp().bindings().len(), 1);
        assert!(s.practice.check(&d.example).unwrap().iter().all(|r| r.0));
        assert_eq!(s.assignment, old);
        assert!(d.checks(&s).iter().any(|r| !r.0));
        s.assignment.draft.vlans = "ten".into();
        s.view = dlms_learning_lab::script_teaching::View::Example;
        let saved = serde_json::to_string(&s).unwrap();
        assert_eq!(Session::from_json(&saved, &d).unwrap(), s);
        s.assignment = Scope::new(&d.initial);
        assert_eq!(s.practice.world.dhcp().bindings().len(), 1);
    }
}
#[test]
fn alternate_server_acl_equivalent_order_and_extra_inactive_selection_are_valid() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let mut c = d.initial.reference.clone();
    c.ports
        .iter_mut()
        .find(|p| p.port == "server")
        .unwrap()
        .trusted = false;
    let Step::Arp { packet, .. } = &d.initial.cases[3].steps[2] else {
        panic!()
    };
    c.filters = vec![core::Filter {
        vlan: 10,
        static_only: false,
        rules: vec![core::Rule {
            action: core::Action::Permit,
            sender_ip: packet.sender_ip,
            sender_mac: packet.sender_mac,
        }],
    }];
    c.vlans = vec![20, 10];
    c.ports.reverse();
    s.assignment.draft = Draft::new(&c);
    s.assignment.apply().unwrap();
    assert!(s.assignment.applied());
    assert!(s.assignment.check(&d.initial).unwrap().iter().all(|r| r.0));
    assert!(d.checks(&s).iter().all(|r| r.0));
}
#[test]
fn wrong_partial_and_broken_policies_cannot_hide_behind_checked_flags_or_previous_history() {
    for kind in 1..=3 {
        let d = definition(kind);
        let mut s = Session::new(&d).unwrap();
        let rows = s.assignment.check(&d.initial).unwrap();
        assert!(rows.iter().any(|r| !r.0));
        if kind == 3 {
            assert!(rows.iter().any(|r| r.0));
        }
        s.assignment.fill(&d.initial).unwrap();
        s.assignment.run_sample(&d.initial).unwrap();
        assert!(s.assignment.check(&d.initial).unwrap().iter().all(|r| r.0));
        let observed = s.assignment.world.dhcp().bindings().to_vec();
        s.assignment.draft.vlans.clear();
        assert!(d.checks(&s).iter().any(|r| !r.0));
        s.assignment.apply().unwrap();
        s.assignment.checked = true;
        assert!(d.checks(&s).iter().any(|r| !r.0));
        assert_eq!(
            s.assignment.world.dhcp().bindings(),
            observed,
            "A checked binding table alone cannot pass"
        );
        s.assignment.fill(&d.initial).unwrap();
        s.assignment.check(&d.initial).unwrap();
        for p in &mut s.assignment.draft.ports {
            p.trusted = true;
        }
        s.assignment.apply().unwrap();
        s.assignment.checked = true;
        assert!(
            d.checks(&s).iter().any(|r| !r.0),
            "Trusting everyone cannot replace inspection"
        );
    }
}
#[test]
fn bad_filter_fields_retain_drafts_and_last_applied_state_without_success() {
    let d = definition(2);
    let mut s = Session::new(&d).unwrap();
    s.assignment.fill(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    let old = s.assignment.world.clone();
    s.assignment.draft.filters[0].rules[0].ip = "wrong".into();
    let err = s.assignment.apply().unwrap_err();
    assert!(err.contains("sender IP"));
    assert_eq!(s.assignment.world, old);
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let saved = serde_json::to_string(&s).unwrap();
    assert_eq!(Session::from_json(&saved, &d).unwrap(), s);
    s.assignment.draft.filters[0].rules[0].ip = "10.1.0.99".into();
    s.assignment.draft.filters[0].rules[0].mac = "invalid".into();
    assert!(s.assignment.apply().unwrap_err().contains("MAC pairs"));
    assert_eq!(s.assignment.world, old);
}
#[test]
fn read_only_dhcp_policy_and_equipment_are_bound_to_the_source() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    let mut c = s.assignment.world.dhcp().config().clone();
    c.enabled = false;
    s.assignment.world.configure_dhcp(c).unwrap();
    assert!(d.bound(&s).unwrap_err().contains("DHCP policy"));
    assert!(Session::from_json(&serde_json::to_string(&s).unwrap(), &d).is_err());
    assert!(d.checks(&s).iter().any(|r| !r.0));
}
#[test]
fn bad_unreachable_and_unsupported_cases_are_not_valid_scenario_packs() {
    let d = definition(1);
    let mut bad = d.clone();
    bad.initial.cases[0].label = bad.initial.cases[1].label.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases.truncate(2);
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example.id = bad.initial.id.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example = bad.initial.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.reference.vlans.clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    let Step::Arp { host, .. } = &mut bad.initial.cases[0].steps[2] else {
        panic!()
    };
    host.0 = "assigned-peer".into();
    assert!(
        bad.validate().is_err(),
        "Unresolved cross-port behavior cannot be called a successful reference"
    );
}
#[test]
fn traffic_replay_is_explicitly_fresh_and_does_not_award_credit() {
    let d = definition(1);
    let mut s = Session::new(&d).unwrap();
    s.assignment.fill(&d.initial).unwrap();
    s.assignment.run_sample(&d.initial).unwrap();
    assert_eq!(s.assignment.world.dhcp().bindings().len(), 1);
    s.assignment.sample = 4;
    s.assignment.run_sample(&d.initial).unwrap();
    assert!(s.assignment.world.dhcp().bindings().is_empty());
    assert!(!s.assignment.checked);
    assert!(d.checks(&s).iter().any(|r| !r.0));
    s.assignment.sample = 0;
    s.assignment.run_sample(&d.initial).unwrap();
    assert_eq!(s.assignment.world.dhcp().bindings().len(), 1);
    assert!(
        s.assignment
            .output
            .iter()
            .any(|l| l.text.contains("current DHCP address record"))
    );
}
#[test]
fn malformed_and_oversized_sessions_bad_samples_and_source_ids_fail_closed() {
    let d = definition(1);
    let s = Session::new(&d).unwrap();
    let mut bad = s.clone();
    bad.schema_version = 2;
    assert!(d.bound(&bad).is_err());
    let mut bad = s.clone();
    bad.assignment.sample = 7;
    assert!(d.bound(&bad).is_err());
    let mut bad = s.clone();
    bad.origin.0 = "another-task".into();
    assert!(d.bound(&bad).is_err());
    let mut bad = s.clone();
    bad.assignment.output = vec![Line {
        text: "x".repeat(2049),
        error: false,
    }];
    assert!(d.bound(&bad).is_err());
    assert!(Session::from_json(&" ".repeat(MAX_SESSION + 1), &d).is_err());
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["shell"] = serde_json::json!("run host code");
    assert!(Session::from_json(&raw.to_string(), &d).is_err());
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["initial"]["cases"][0]["steps"][0]["kind"] = serde_json::json!("host_shell");
    assert!(Definition::from_json(&raw.to_string()).is_err());
}
#[test]
fn unsupported_reference_identifies_the_unmodeled_case_for_authors() {
    let mut d = definition(1);
    let Step::Arp { packet, .. } = &mut d.initial.cases[0].steps[2] else {
        panic!()
    };
    packet.sender_ip = "0.0.0.0".parse().unwrap();
    let e = d.validate().unwrap_err();
    assert!(e.contains("Reference case"));
    assert!(e.contains("zero sender IP"));
}
