//! Separate typed learner scopes; fixture files are data, never host configuration.
use dlms_learning_lab::{
    linux_filter::{Chain, Cidr, Protocol, Rule, Verdict},
    linux_router_admin as admin,
    linux_router_path::{Header, Origin},
    linux_router_workspace::{Action, Case, Definition, File, Policy, Scope, Session, Task},
};
fn initial(practice: bool) -> admin::State {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/linux-router-foundation-v1.json")).unwrap();
    serde_json::from_value(v[if practice { "practice" } else { "assignment" }].clone()).unwrap()
}
fn task(practice: bool, persistence: bool) -> Task {
    let state = initial(practice);
    let lan = &state.host.profile.links[0].network;
    let web = state.host.profile.nodes[0].address;
    let any = Rule {
        source: None,
        destination: None,
        ingress: None,
        egress: None,
        protocol: None,
        source_port: None,
        destination_port: None,
        target: Verdict::Accept,
    };
    let allowed = Rule {
        source: Some(Cidr {
            network: lan.address,
            prefix: lan.length,
        }),
        ingress: Some("lan0".into()),
        egress: Some("wan0".into()),
        protocol: Some(Protocol::Tcp),
        destination_port: Some(443),
        ..any.clone()
    };
    let h = Header {
        source: (u32::from(lan.address) + 55).into(),
        destination: web,
        protocol: Protocol::Tcp,
        source_port: Some(49152),
        destination_port: Some(443),
    };
    let mut denied = h.clone();
    denied.destination_port = Some(80);
    Task {
        initial: state,
        files: persistence,
        persistence,
        forwarding: true,
        policies: vec![Policy {
            chain: Chain::Forward,
            domains: vec![any],
            allowed: vec![allowed],
        }],
        cases: vec![
            Case {
                id: "web".into(),
                label: "LAN web request".into(),
                origin: Origin::Incoming("lan0".into()),
                header: h,
                allow: true,
            },
            Case {
                id: "other-port".into(),
                label: "Other port stays blocked".into(),
                origin: Origin::Incoming("lan0".into()),
                header: denied,
                allow: false,
            },
        ],
    }
}
fn commands(t: &Task) -> Vec<Action> {
    vec![
        Action::Command {
            line: format!(
                "ip -4 route add default via {} dev wan0 metric 10",
                t.initial.host.profile.gateways[0].address
            ),
        },
        Action::Command {
            line: "sysctl -w net.ipv4.ip_forward=1".into(),
        },
        Action::Command {
            line: format!(
                "iptables -A FORWARD -i lan0 -o wan0 -s {} -p tcp --dport 443 -j ACCEPT",
                t.initial.host.profile.links[0].network.text()
            ),
        },
    ]
}
fn repair(t: &Task) -> Vec<Action> {
    let mut a = commands(t);
    if t.persistence {
        a.extend([
            Action::Command {
                line: format!("iptables-save -t filter -f {}", admin::FILTER_FILE),
            },
            Action::File {
                file: File::Forwarding,
            },
            Action::Edit {
                text: "# Reapplied setting\nnet.ipv4.ip_forward=1\n".into(),
            },
            Action::Save,
        ]);
    }
    a.push(Action::Check);
    a
}
fn definition(persistence: bool) -> Definition {
    let assignment = task(false, persistence);
    let example = task(true, persistence);
    let worked = repair(&example);
    Definition {
        source_id: "router-workspace-fixture".into(),
        assignment,
        example,
        worked,
    }
}
fn run(s: &mut Session, d: &Definition, a: &[Action], practice: bool) {
    for a in a {
        s.action(d, a, practice).unwrap();
    }
}
#[test]
fn working_example_is_separate_and_never_completes_the_independent_assignment() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    let before = s.assignment.clone();
    run(&mut s, &d, &d.worked, true);
    assert!(d.example.checks(&s.practice).iter().all(|c| c.passed));
    assert_eq!(s.assignment, before);
    assert!(!d.solved(&s));
    run(&mut s, &d, &repair(&d.assignment), false);
    assert!(d.solved(&s));
}
#[test]
fn editing_or_breaking_state_clears_success_and_unknown_commands_remain_retryable() {
    let d = definition(false);
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &repair(&d.assignment), false);
    assert!(d.solved(&s));
    s.action(
        &d,
        &Action::Input {
            text: "iptables -L".into(),
        },
        false,
    )
    .unwrap();
    assert!(!d.solved(&s));
    s.action(
        &d,
        &Action::Command {
            line: "iptables -L".into(),
        },
        false,
    )
    .unwrap();
    assert_eq!(s.assignment.input, "iptables -L");
    assert!(s.assignment.output.contains("[Unsupported]"));
    assert!(!s.assignment.checked);
    s.action(
        &d,
        &Action::Command {
            line: "iptables -P FORWARD ACCEPT".into(),
        },
        false,
    )
    .unwrap();
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
fn a_live_fix_does_not_earn_persistence_credit_with_broken_saved_files() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &commands(&d.assignment), false);
    s.action(&d, &Action::Check, false).unwrap();
    let checks = d.assignment.checks(&s.assignment);
    assert!(checks.iter().any(|c| c.passed));
    assert!(
        checks
            .iter()
            .any(|c| !c.passed && c.label.starts_with("Reloaded setup:"))
    );
    assert!(!d.solved(&s));
    s.action(
        &d,
        &Action::Command {
            line: format!("iptables-save -t filter -f {}", admin::FILTER_FILE),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::File {
            file: File::Forwarding,
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "net.ipv4.ip_forward=1\n".into(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Save, false).unwrap();
    s.action(&d, &Action::Check, false).unwrap();
    assert!(d.solved(&s));
}
#[test]
fn inspecting_and_exporting_preserve_unsaved_drafts_until_explicit_reload() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "# Unfinished file\n*filter\n".into(),
        },
        false,
    )
    .unwrap();
    let draft = s.assignment.filter_draft.clone();
    for line in [
        "iptables -S",
        "ip -4 route show",
        "iptables-save -t filter -f /etc/lab-filter.rules",
    ] {
        s.action(&d, &Action::Command { line: line.into() }, false)
            .unwrap();
        assert_eq!(s.assignment.filter_draft, draft);
    }
    s.action(
        &d,
        &Action::Command {
            line: "iptables -A FORWARD -j ACCEPT".into(),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Command {
            line: "iptables-save -t filter -f /etc/lab-filter.rules".into(),
        },
        false,
    )
    .unwrap();
    assert_ne!(s.assignment.state.filter_file, s.assignment.filter_draft);
    assert_eq!(s.assignment.filter_draft, draft);
    assert!(
        s.assignment
            .output
            .contains("unsaved editor draft was retained")
    );
    s.action(&d, &Action::ReloadDraft, false).unwrap();
    assert_eq!(s.assignment.filter_draft, s.assignment.state.filter_file);
}
#[test]
fn resume_keeps_both_scopes_and_selected_reset_does_not_touch_the_other_scope() {
    let d = definition(true);
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.worked, true);
    run(&mut s, &d, &commands(&d.assignment), false);
    s.action(
        &d,
        &Action::Input {
            text: "iptables-save -t filter".into(),
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "# draft to finish\n".into(),
        },
        false,
    )
    .unwrap();
    let raw = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&raw).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(!d.solved(&restored));
    let example = restored.practice.clone();
    restored.reset(&d, false).unwrap();
    assert_eq!(restored.assignment, Scope::new(&d.assignment).unwrap());
    assert_eq!(restored.practice, example);
    let mut swapped = s;
    std::mem::swap(&mut swapped.assignment, &mut swapped.practice);
    assert!(d.bound(&swapped).is_err());
}
#[test]
fn invalid_reference_commands_and_already_satisfied_assignments_are_rejected() {
    let mut d = definition(false);
    d.worked.insert(
        0,
        Action::Command {
            line: "iptables -L".into(),
        },
    );
    assert!(d.validate().is_err());
    let mut d = definition(false);
    let mut working = Scope::new(&d.assignment).unwrap();
    for a in repair(&d.assignment) {
        working.reference(&d.assignment, &a).unwrap();
    }
    d.assignment.initial = working.state;
    assert!(d.validate().is_err());
}
