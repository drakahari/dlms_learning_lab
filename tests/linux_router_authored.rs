//! Pre-registration author checks: these fixtures are not catalog coverage.
use dlms_learning_lab::linux_router_workspace::{Action, Definition, Session};
use serde::Deserialize;
#[derive(Deserialize)]
struct Prepared {
    definition: Definition,
    initial: Session,
    solution: Vec<Action>,
}
fn prepared() -> Vec<Prepared> {
    serde_json::from_str(include_str!(
        "fixtures/linux-router-authored-preparation-v1.json"
    ))
    .unwrap()
}
fn act(s: &mut Session, d: &Definition, line: &str) {
    s.action(d, &Action::Command { line: line.into() }, false)
        .unwrap();
}
#[test]
fn every_taught_method_and_assignment_repair_reaches_the_actual_outcomes() {
    for p in prepared() {
        let d = p.definition;
        d.validate()
            .unwrap_or_else(|e| panic!("{}: {e}", d.source_id));
        let mut s = Session::new(&d).unwrap();
        assert_eq!(s, p.initial);
        assert!(!d.solved(&s));
        let original = s.assignment.clone();
        for a in &d.worked {
            s.action(&d, a, true).unwrap();
        }
        assert_eq!(s.assignment, original);
        assert!(!d.solved(&s));
        for a in &p.solution {
            s.action(&d, a, false).unwrap();
        }
        assert!(
            d.solved(&s),
            "{}: {:?}",
            d.source_id,
            d.assignment.checks(&s.assignment)
        );
        let raw = serde_json::to_string(&s).unwrap();
        let restored: Session = serde_json::from_str(&raw).unwrap();
        assert_eq!(restored, s);
        assert!(d.solved(&restored));
        act(&mut s, &d, "iptables -P FORWARD ACCEPT");
        assert!(!d.solved(&s));
        s.action(&d, &Action::Check, false).unwrap();
        assert!(!d.solved(&s));
        let checks = d.assignment.checks(&s.assignment);
        assert!(checks.iter().any(|c| c.passed));
        assert!(
            checks
                .iter()
                .any(|c| !c.passed && c.detail.contains("but the task requires"))
        );
    }
}
#[test]
fn default_accept_and_explicit_deny_is_not_a_replacement_for_the_required_policy() {
    let p = prepared().remove(0);
    let d = p.definition;
    let mut s = Session::new(&d).unwrap();
    act(&mut s, &d, "iptables -F");
    act(&mut s, &d, "iptables -P INPUT ACCEPT");
    act(&mut s, &d, "iptables -P FORWARD ACCEPT");
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&s));
    assert!(
        d.assignment
            .checks(&s.assignment)
            .iter()
            .any(|c| !c.passed && c.label.contains("policy"))
    );
}
#[test]
fn equivalent_subnet_splits_pass_but_answering_only_the_visible_clients_does_not() {
    let p = prepared().remove(0);
    let d = p.definition;
    let mut s = Session::new(&d).unwrap();
    for a in &p.solution {
        s.action(&d, a, false).unwrap();
    }
    assert!(d.solved(&s));
    act(&mut s, &d, "iptables -F FORWARD");
    for source in ["10.20.30.0/25", "10.20.30.128/25"] {
        for (destination, protocol, port) in
            [("203.0.113.9", "tcp", 443), ("203.0.113.53", "udp", 53)]
        {
            act(
                &mut s,
                &d,
                &format!(
                    "iptables -A FORWARD -i lan0 -o wan0 -s {source} -d {destination} -p {protocol} --dport {port} -j ACCEPT"
                ),
            );
        }
    }
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.solved(&s), "{:?}", d.assignment.checks(&s.assignment));
    act(&mut s, &d, "iptables -F FORWARD");
    for (destination, protocol, port) in [("203.0.113.9", "tcp", 443), ("203.0.113.53", "udp", 53)]
    {
        act(
            &mut s,
            &d,
            &format!(
                "iptables -A FORWARD -i lan0 -o wan0 -s 10.20.30.55 -d {destination} -p {protocol} --dport {port} -j ACCEPT"
            ),
        );
    }
    s.action(&d, &Action::Check, false).unwrap();
    let checks = d.assignment.checks(&s.assignment);
    assert!(
        checks
            .iter()
            .filter(|c| c.label == "LAN web transit" || c.label == "LAN DNS transit")
            .all(|c| c.passed)
    );
    assert!(
        checks
            .iter()
            .any(|c| !c.passed && c.label.contains("FORWARD policy"))
    );
    assert!(!d.solved(&s));
}
#[test]
fn changing_only_an_example_identifier_does_not_create_independent_practice() {
    let mut p = prepared().remove(0);
    p.definition.example = p.definition.assignment.clone();
    p.definition.example.initial.host.profile.id = "different-label-same-task".into();
    assert!(p.definition.validate().unwrap_err().contains("separate"));
}
#[test]
fn forwarding_alone_does_not_hide_a_more_specific_broken_next_hop() {
    let p = prepared().remove(1);
    let d = p.definition;
    let mut s = Session::new(&d).unwrap();
    act(&mut s, &d, "sysctl -w net.ipv4.ip_forward=1");
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&s));
    let checks = d.assignment.checks(&s.assignment);
    assert!(
        checks
            .iter()
            .any(|c| c.passed && c.label.contains("FORWARD policy"))
    );
    assert!(
        checks
            .iter()
            .any(|c| !c.passed && c.label.contains("LAN web"))
    );
    act(
        &mut s,
        &d,
        "ip -4 route del 203.0.113.0/24 via 198.51.100.253 dev wan0 metric 5",
    );
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.solved(&s), "{:?}", d.assignment.checks(&s.assignment));
}
#[test]
fn live_correctness_does_not_hide_an_unsafe_saved_configuration() {
    let p = prepared().remove(2);
    let d = p.definition;
    let mut s = Session::new(&d).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&s));
    let checks = d.assignment.checks(&s.assignment);
    assert!(
        checks
            .iter()
            .filter(|c| !c.label.starts_with("Reloaded setup:"))
            .all(|c| c.passed)
    );
    assert!(
        checks
            .iter()
            .any(|c| c.label.starts_with("Reloaded setup:") && !c.passed)
    );
    act(&mut s, &d, "iptables-restore --test /etc/lab-filter.rules");
    assert!(!d.solved(&s));
    for a in &p.solution {
        s.action(&d, a, false).unwrap();
    }
    assert!(d.solved(&s));
    s.action(
        &d,
        &Action::File {
            file: dlms_learning_lab::linux_router_workspace::File::Forwarding,
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "net.ipv4.ip_forward=0\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!d.solved(&s));
    s.action(&d, &Action::Save, false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&s));
}

#[test]
fn policy_feedback_identifies_the_matching_rule_or_fallback_without_fake_capture_values() {
    let p = prepared().remove(0);
    let d = p.definition;
    let mut s = Session::new(&d).unwrap();
    let input = |s: &Session| {
        d.assignment
            .checks(&s.assignment)
            .into_iter()
            .find(|c| c.label == "INPUT policy")
            .unwrap()
    };
    let first = input(&s);
    assert!(!first.passed);
    assert!(first.detail.contains("INPUT rule 1"));
    assert!(first.detail.contains("first matching rule"));
    assert!(!first.detail.contains("0.0.0.0"));
    act(&mut s, &d, "iptables -F INPUT");
    let missing = input(&s);
    assert!(!missing.passed);
    assert!(missing.detail.contains("No rule matched in INPUT"));
    assert!(missing.detail.contains("DROP policy"));
    assert!(missing.detail.contains("but the task requires it allowed"));
}
