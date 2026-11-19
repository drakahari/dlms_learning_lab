//! Capability-first fixture. No catalog/coverage credit until teaching admission.
use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt},
    transport_core as core, transport_workspace as w,
};
use std::sync::OnceLock;
fn source() -> &'static pack::Scenario {
    static SOURCE: OnceLock<pack::Scenario> = OnceLock::new();
    SOURCE.get_or_init(|| {
        pack::load(include_str!("fixtures/protocol-evidence-preview-v1.json"))
            .unwrap()
            .remove(0)
    })
}
fn definition() -> &'static w::Definition {
    let engine::Goal::Transport { definition } = &source().goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::Transport { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn edit(a: &mut Attempt, draft: w::Draft) {
    session(a)
        .assignment
        .edit(&definition().assignment, draft)
        .unwrap();
}
fn check(a: &mut Attempt) -> Vec<(bool, String)> {
    session(a)
        .assignment
        .check(&definition().assignment)
        .unwrap()
}
fn solved(a: &Attempt) -> bool {
    progress::current_solved(a, source())
}
fn reference(a: &mut Attempt) {
    for action in &source().solution {
        engine::execute_for_goal(&mut a.state, &source().goal, action).unwrap();
    }
}
#[test]
fn source_taught_practice_is_distinct_and_cannot_award_assignment_credit() {
    let d = definition();
    d.validate().unwrap();
    assert_ne!(d.assignment.transport.hosts, d.example.transport.hosts);
    assert_eq!(
        (
            d.assignment.transport.exchanges.len(),
            d.assignment.echoes.len(),
            d.assignment.requirements.len()
        ),
        (3, 2, 5)
    );
    let mut a = Attempt::new(source(), 1);
    let original = session(&mut a).assignment.clone();
    session(&mut a)
        .practice
        .edit(&d.example, d.worked_example.clone())
        .unwrap();
    assert!(session(&mut a).practice.tested.is_none());
    assert!(
        session(&mut a)
            .practice
            .check(&d.example)
            .unwrap()
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(session(&mut a).assignment, original);
    assert!(!solved(&a));
    reference(&mut a);
    assert!(solved(&a));
}
#[test]
fn echo_only_repair_produces_specific_partial_feedback_while_tcp_is_still_filtered() {
    let d = definition();
    let mut a = Attempt::new(source(), 1);
    let mut partial = session(&mut a).assignment.draft.clone();
    let server = d.assignment.transport.hosts[1].address;
    let client = d.assignment.transport.hosts[0].address;
    let text = partial.documents.routers.get_mut("R1").unwrap();
    *text = text.replace(
        "20 deny icmp any any",
        &format!("20 permit icmp host {server} host {client}"),
    );
    edit(&mut a, partial);
    let rows = check(&mut a);
    assert_eq!(rows.iter().filter(|r| r.0).count(), 2, "{rows:?}");
    assert!(rows[3].0 && rows[4].0);
    assert!(!rows[0].0 && rows[0].1.contains("SYN-ACK"));
    assert!(!solved(&a));
    assert!(rows[2].1.contains("no required service"));
    let reports = w::reports(&d.assignment, &session(&mut a).assignment.draft).unwrap();
    assert_eq!(
        reports[2].outcome,
        core::Outcome::PacketDropped {
            kind: core::PacketKind::Datagram
        }
    );
}
#[test]
fn supported_alternate_rule_shapes_pass_while_hardcoded_practice_values_do_not() {
    let d = definition();
    let mut a = Attempt::new(source(), 1);
    let mut alternative = d.worked_assignment.clone();
    alternative.documents.routers.insert("R1".into(),"ip access-list extended TRAFFIC\n 10 permit icmp any any\n 20 permit tcp any any\n 30 deny udp any any\n 40 deny ip any any\nexit\ninterface office\n ip access-group TRAFFIC in\n ip access-group TRAFFIC out\nexit\n".into());
    edit(&mut a, alternative);
    assert!(check(&mut a).iter().all(|r| r.0));
    assert!(solved(&a));
    let mut wrong = d.worked_assignment.clone();
    wrong.documents = d.worked_example.documents.clone();
    edit(&mut a, wrong);
    assert!(!check(&mut a).iter().all(|r| r.0));
    assert!(!solved(&a));
}
#[test]
fn broad_allow_broad_drop_tcp_only_and_echo_only_cannot_satisfy_the_complete_assignment() {
    for rules in [
        "10 permit ip any any",
        "10 deny ip any any",
        "10 permit tcp any any\n 20 deny ip any any",
        "10 permit icmp any any\n 20 deny ip any any",
    ] {
        let mut a = Attempt::new(source(), 1);
        let mut bad = definition().worked_assignment.clone();
        bad.documents.routers.insert("R1".into(),format!("ip access-list extended TRAFFIC\n {rules}\nexit\ninterface office\n ip access-group TRAFFIC in\n ip access-group TRAFFIC out\nexit\n"));
        edit(&mut a, bad);
        assert!(!check(&mut a).iter().all(|r| r.0), "{rules}");
        assert!(!solved(&a));
    }
}
#[test]
fn changing_one_return_rule_clears_success_and_resume_preserves_drafts_not_invented_mastery() {
    let s = source();
    let d = definition();
    let mut a = Attempt::new(s, 1);
    reference(&mut a);
    assert!(solved(&a));
    let mut broken = d.worked_assignment.clone();
    let server = d.assignment.transport.hosts[1].address;
    let client = d.assignment.transport.hosts[0].address;
    *broken.documents.routers.get_mut("R1").unwrap() = broken.documents.routers["R1"].replace(
        &format!("20 permit icmp host {server} host {client}"),
        "20 deny icmp any any",
    );
    edit(&mut a, broken);
    assert!(session(&mut a).assignment.tested.is_none());
    assert!(!solved(&a));
    let rows = check(&mut a);
    assert!(rows[0].0 && rows[1].0 && rows[2].0);
    assert!(!rows[3].0 && rows[3].1.contains("reply was filtered"));
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save_with_catalog(&path, &p, std::slice::from_ref(s)).unwrap();
    let mut loaded = progress::load(&path, std::slice::from_ref(s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(!solved(&loaded.attempts[&s.id]));
    loaded.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    assert!(loaded.favorites.contains(&s.id));
    assert!(!solved(&loaded.attempts[&s.id]));
}
#[test]
fn protected_endpoints_malformed_policy_and_shell_strings_fail_honestly() {
    let d = definition();
    let mut a = Attempt::new(source(), 1);
    reference(&mut a);
    let mut protected = d.worked_assignment.clone();
    protected.listeners[1].port = "9999".into();
    assert!(
        session(&mut a)
            .assignment
            .edit(&d.assignment, protected)
            .is_err()
    );
    assert!(solved(&a));
    for input in [
        "sh /private/script",
        "ping 192.0.2.1",
        "@transport {\"kind\":\"run\",\"command\":\"sh\"}",
    ] {
        let before = serde_json::to_value(&a).unwrap();
        assert!(engine::execute_for_goal(&mut a.state, &source().goal, input).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
    }
    let mut unsupported = d.worked_assignment.clone();
    unsupported.documents.routers.insert(
        "R1".into(),
        "ip access-list extended BAD\n 10 permit icmp any any echo\nexit\n".into(),
    );
    edit(&mut a, unsupported);
    assert!(!check(&mut a).iter().all(|r| r.0));
    assert!(!solved(&a));
}

#[test]
fn current438_registers_the_full_task_without_changing_any437_scenario_or170_unit_objects() {
    let before = pack::accepted437().unwrap();
    let current = pack::accepted438().unwrap();
    assert_eq!((before.len(), current.len()), (437, 438));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..437]).unwrap()
    );
    let added = current.iter().find(|s| s.id == source().id).unwrap();
    let mut expected = source().clone();
    expected.prerequisites = vec![
        "lpic1-host-mixed-ip".into(),
        "lpic1-service-client-targets".into(),
    ];
    assert_eq!(
        serde_json::to_value(added).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    let old = dlms_learning_lab::course::accepted437(&before).unwrap();
    let units = dlms_learning_lab::course::accepted438(&current).unwrap();
    assert_eq!((old.len(), units.len()), (170, 171));
    let retained = units
        .iter()
        .filter(|u| u.id != source().unit)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units.iter().position(|u| u.id == source().unit).unwrap();
    for name in ["linux-mixed-ip-foundations", "linux-service-names"] {
        assert!(units.iter().position(|u| u.id == name).unwrap() < at);
    }
}
