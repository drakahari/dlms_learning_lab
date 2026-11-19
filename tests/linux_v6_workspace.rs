//! Complete typed task fixtures; not yet registered curriculum or native acceptance.
use dlms_learning_lab::linux_v6_workspace::{Action, Definition, Session};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    source_id: String,
    definition: Definition,
    initial: Session,
    solution: Vec<Action>,
}
fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(include_str!("fixtures/linux-v6-workspace-v1.json")).unwrap()
}
fn solve(s: &mut Session, d: &Definition, actions: &[Action], practice: bool) {
    for a in actions {
        s.action(d, a, practice).unwrap();
    }
}
#[test]
fn linux_v6_workspace_separate_examples_work_without_awarding_assignment_credit() {
    for f in fixtures() {
        let d = &f.definition;
        d.validate().unwrap();
        assert_eq!(f.source_id, d.source_id);
        let mut s = Session::new(d).unwrap();
        assert_eq!(s, f.initial);
        s.shape().unwrap();
        let before = s.assignment.clone();
        solve(&mut s, d, &d.worked, true);
        assert_eq!(s.assignment, before);
        assert!(!d.solved(&s));
        assert!(d.example.checks(&s.practice).iter().all(|c| c.passed));
        solve(&mut s, d, &f.solution, false);
        assert!(
            d.solved(&s),
            "{}: {:?}",
            f.source_id,
            d.assignment.checks(&s.assignment)
        );
    }
}
#[test]
fn linux_v6_workspace_legitimate_split_prefix_and_numeric_protocol_alternatives_pass() {
    for f in fixtures() {
        let d = &f.definition;
        let mut s = Session::new(d).unwrap();
        for a in &f.solution {
            if let Action::Command { line } = a {
                let line = line
                    .replace("-p tcp ", "-p 6 ")
                    .replace("-p ipv6-icmp ", "-p 58 ");
                let commands = if line.contains("fd20:30:40::/64") {
                    vec![
                        line.replace("fd20:30:40::/64", "fd20:30:40::/65"),
                        line.replace("fd20:30:40::/64", "fd20:30:40:0:8000::/65"),
                    ]
                } else {
                    vec![line]
                };
                for line in commands {
                    s.action(d, &Action::Command { line }, false).unwrap();
                }
            } else {
                s.action(d, a, false).unwrap();
            }
        }
        assert!(
            d.solved(&s),
            "{}: {:?}",
            f.source_id,
            d.assignment.checks(&s.assignment)
        );
    }
}
#[test]
fn linux_v6_workspace_hardcoded_displayed_clients_pass_samples_but_fail_whole_policy() {
    for f in fixtures() {
        let d = &f.definition;
        let mut s = Session::new(d).unwrap();
        for a in &f.solution {
            if let Action::Command { line } = a {
                let commands = if line.contains("fd20:30:40::/64") {
                    vec![
                        line.replace("fd20:30:40::/64", "fd20:30:40::55/128"),
                        line.replace("fd20:30:40::/64", "fd20:30:40::99/128"),
                    ]
                } else {
                    vec![line.clone()]
                };
                for line in commands {
                    s.action(d, &Action::Command { line }, false).unwrap();
                }
            } else {
                s.action(d, a, false).unwrap();
            }
        }
        assert!(
            d.assignment.cases.iter().all(|c| d
                .assignment
                .trial(&s.assignment.state, c)
                .unwrap()
                .steps
                .iter()
                .all(|s| s.check.passed)),
            "The negative control must actually satisfy displayed clients"
        );
        assert!(
            !d.solved(&s),
            "{} hardcoded clients must not satisfy /64",
            f.source_id
        );
        assert!(
            d.assignment
                .checks(&s.assignment)
                .iter()
                .any(|c| !c.passed && c.label.starts_with("FORWARD"))
        );
    }
}
#[test]
fn linux_v6_workspace_wrong_codes_broad_state_and_incorrect_prefixes_explain_failure() {
    for f in fixtures() {
        let d = &f.definition;
        for mutation in ["code", "prefix", "broad"] {
            let mut s = Session::new(d).unwrap();
            for a in &f.solution {
                let a = if let Action::Command { line } = a {
                    Action::Command {
                        line: match mutation {
                            "code" => line.replace("--icmpv6-type 2/0", "--icmpv6-type 2"),
                            "prefix" => line.replace("fd20:30:40::/64", "fd20:30:41::/64"),
                            _ => line.clone(),
                        },
                    }
                } else {
                    a.clone()
                };
                s.action(d, &a, false).unwrap();
            }
            if mutation == "broad" {
                s.action(
                    d,
                    &Action::Command {
                        line: "ip6tables -P FORWARD ACCEPT".into(),
                    },
                    false,
                )
                .unwrap();
                s.action(d, &Action::Check, false).unwrap();
            }
            assert!(!d.solved(&s), "{} {mutation}", f.source_id);
            let checks = d.assignment.checks(&s.assignment);
            assert!(
                checks
                    .iter()
                    .any(|c| !c.passed && c.detail.contains("Inspect")
                        || !c.passed && c.detail.contains("inspect"))
            );
        }
        if f.source_id.contains("tracked") {
            let mut s = Session::new(d).unwrap();
            for a in &f.solution {
                let a = if let Action::Command { line } = a {
                    Action::Command {
                        line: line.replace("--ctstate ESTABLISHED", "--ctstate NEW,ESTABLISHED"),
                    }
                } else {
                    a.clone()
                };
                s.action(d, &a, false).unwrap();
            }
            assert!(!d.solved(&s));
        }
    }
}
#[test]
fn linux_v6_workspace_partial_results_are_not_complete_and_edits_clear_success() {
    for f in fixtures() {
        let d = &f.definition;
        let mut s = Session::new(d).unwrap();
        solve(&mut s, d, &f.solution[..3], false);
        s.action(d, &Action::Check, false).unwrap();
        let rows = d.assignment.checks(&s.assignment);
        assert!(rows.iter().any(|r| r.passed));
        assert!(rows.iter().any(|r| !r.passed));
        assert!(!d.solved(&s));
        solve(&mut s, d, &f.solution, false);
        assert!(d.solved(&s));
        s.action(
            d,
            &Action::Input {
                text: "ip6tables -S".into(),
            },
            false,
        )
        .unwrap();
        assert!(!s.assignment.checked);
        assert!(!d.solved(&s));
        s.action(d, &Action::Check, false).unwrap();
        assert!(d.solved(&s));
        s.action(
            d,
            &Action::Command {
                line: "ip6tables -F FORWARD".into(),
            },
            false,
        )
        .unwrap();
        assert!(!d.solved(&s));
        s.action(d, &Action::Check, false).unwrap();
        assert!(!d.solved(&s));
    }
}
#[test]
fn linux_v6_workspace_replay_reports_drafts_reset_and_resume_keep_scopes_independent() {
    for f in fixtures() {
        let d = &f.definition;
        let mut s = Session::new(d).unwrap();
        solve(&mut s, d, &d.worked, true);
        solve(&mut s, d, &f.solution, false);
        s.action(d, &Action::RunTraffic, false).unwrap();
        assert!(d.solved(&s));
        s.action(
            d,
            &Action::Input {
                text: "ip6tables -S FORWARD".into(),
            },
            true,
        )
        .unwrap();
        s.action(d, &Action::RunTraffic, true).unwrap();
        let restored: Session = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        d.bound(&restored).unwrap();
        assert_eq!(restored, s);
        assert!(d.solved(&restored));
        let mut wrong = s.clone();
        wrong.assignment.last_trial.as_mut().unwrap().steps[0]
            .check
            .detail
            .push_str(" forged");
        assert!(d.bound(&wrong).is_err());
        let mut wrong = s.clone();
        wrong.assignment.last_trial.as_mut().unwrap().steps[0]
            .check
            .passed = false;
        assert!(d.bound(&wrong).is_err());
        let before = s.practice.clone();
        s.reset(d, false).unwrap();
        assert_eq!(s.assignment, f.initial.assignment);
        assert_eq!(s.practice, before);
        assert!(!d.solved(&s));
        let before = s.assignment.clone();
        s.reset(d, true).unwrap();
        assert_eq!(s.assignment, before);
        assert_eq!(s.practice, f.initial.practice);
    }
}
#[test]
fn linux_v6_workspace_unsupported_commands_cannot_reach_host_or_mutate_configuration() {
    for f in fixtures() {
        let d = &f.definition;
        let mut s = Session::new(d).unwrap();
        for line in [
            "sh -c 'touch /anything'",
            "ip6tables -A FORWARD -p tcp -m conntrack --ctstate RELATED -j ACCEPT",
            "ip6tables -t nat -F",
            "ip6tables -F INPUT",
            "ip6tables -A FORWARD -p tcp -j ACCEPT; echo outside",
            "ip6tables -A FORWARD -d $(anything) -j ACCEPT",
            "sysctl -w net.ipv6.conf.all.forwarding=1",
            "ip -6 route add default dev wan0",
        ] {
            let before = s.assignment.state.clone();
            s.action(d, &Action::Command { line: line.into() }, false)
                .unwrap();
            assert_eq!(s.assignment.state, before);
            assert!(
                s.assignment.output.contains("Unsupported")
                    || s.assignment.output.contains("Syntax")
            );
            assert!(!d.solved(&s));
        }
        let before = s.clone();
        assert!(
            s.action(
                d,
                &Action::Input {
                    text: "x".repeat(2049)
                },
                false
            )
            .is_err()
        );
        assert_eq!(s, before);
        assert!(
            s.action(d, &Action::SelectCase { index: 8 }, false)
                .is_err()
        );
        assert_eq!(s, before);
    }
}
#[test]
fn linux_v6_workspace_invalid_definitions_sources_and_forward_references_are_rejected() {
    for f in fixtures() {
        let d = f.definition;
        let mut bad = d.clone();
        bad.example = bad.assignment.clone();
        assert!(bad.validate().is_err());
        let mut bad = d.clone();
        bad.assignment.cases[0].events[0].packet =
            dlms_learning_lab::linux_v6_workspace::PacketInput::Reply { event: 0 };
        assert!(bad.validate().is_err());
        let mut bad = d.clone();
        bad.assignment.policies[1].class = None;
        assert!(bad.validate().is_err());
        let mut bad = d.clone();
        bad.worked = vec![Action::Check];
        assert!(bad.validate().is_err());
        let mut bad = d.clone();
        bad.assignment.facts.routes[0].device = "missing0".into();
        assert!(bad.validate().is_err());
        let mut s = Session::new(&d).unwrap();
        s.source_id = "other-source".into();
        assert!(d.bound(&s).is_err());
        let mut value = serde_json::to_value(&d).unwrap();
        value["assignment"]["cases"] =
            serde_json::Value::Array(vec![value["assignment"]["cases"][0].clone(); 9]);
        assert!(serde_json::from_value::<Definition>(value).is_err());
    }
}

#[test]
fn linux_v6_workspace_input_drop_is_not_a_forward_filter_refusal() {
    use dlms_learning_lab::{
        linux_v6_filter::{Chain, Protocol},
        linux_v6_path::{Gate, Header},
        linux_v6_workspace::{Case, Event, Expected, PacketInput},
    };
    for f in fixtures() {
        let task = &f.definition.assignment;
        let mut s = Session::new(&f.definition).unwrap();
        solve(&mut s, &f.definition, &f.solution, false);
        let case = Case {
            id: "router-local-negative".into(),
            label: "An INPUT drop is not a FORWARD policy".into(),
            events: vec![Event {
                label: "Router-local destination".into(),
                ingress: "wan0".into(),
                packet: PacketInput::Literal {
                    header: Header {
                        source: "2001:db8:60::42".parse().unwrap(),
                        destination: task.facts.links[1].address,
                        protocol: Protocol::Icmp6,
                        source_port: None,
                        destination_port: None,
                        icmp_type: Some(128),
                        icmp_code: Some(0),
                    },
                },
                expected: Expected::Filtered,
            }],
        };
        let trial = task.trial(&s.assignment.state, &case).unwrap();
        let step = &trial.steps[0];
        let trace = step.trace.as_ref().unwrap();
        assert_eq!(
            (trace.path.gate.clone(), trace.path.chain),
            (Gate::FilterDrop, Some(Chain::Input))
        );
        assert!(step.check.detail.contains("INPUT:"));
        assert!(
            !step.check.passed,
            "An INPUT drop cannot satisfy an expected FORWARD refusal"
        );
    }
}
