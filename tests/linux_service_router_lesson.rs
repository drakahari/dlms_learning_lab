//! Authored task checks before catalog integration; no native/novice acceptance.
use dlms_learning_lab::{
    engine::{self, State},
    linux_service_router as work, linux_service_router_source as binding, pack,
    progress::{self, Attempt},
};
fn source() -> pack::Scenario {
    serde_json::from_str::<pack::Pack>(include_str!(
        "../scenarios/linux-services-router-lessons-v1.json"
    ))
    .unwrap()
    .scenarios
    .remove(0)
}
fn action(s: &mut State, d: &work::Definition, a: &work::Action, practice: bool) {
    let State::LinuxServiceRouter { simulation } = s else {
        panic!("Actual editor/router state required")
    };
    simulation.action(d, a, practice).unwrap();
}
#[test]
fn authored_example_is_separate_and_assignment_reaches_every_real_requirement() {
    let source = source();
    let d = binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    let before = state.clone();
    for a in &d.worked {
        action(&mut state, d, a, true);
    }
    let (State::LinuxServiceRouter { simulation }, State::LinuxServiceRouter { simulation: old }) =
        (&state, &before)
    else {
        panic!()
    };
    assert_eq!(simulation.assignment, old.assignment);
    assert!(
        d.example
            .checks(&simulation.practice)
            .iter()
            .all(|c| c.passed)
    );
    assert!(!engine::solved(&state, &source.goal));
    for line in &source.solution {
        engine::execute_for_goal(&mut state, &source.goal, line).unwrap();
    }
    assert!(
        engine::solved(&state, &source.goal),
        "{:?}",
        engine::checks(&state, &source.goal)
    );
    let mut a = Attempt::new(&source, 1);
    a.state = state;
    assert!(progress::current_solved(&a, &source));
    let edit = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&work::Action::Edit {
            text: d.assignment.initial_services.clone()
        })
        .unwrap()
    );
    engine::execute_for_goal(&mut a.state, &source.goal, &edit).unwrap();
    assert!(!progress::current_solved(&a, &source));
}
#[test]
fn fixing_only_the_file_or_only_the_numeric_rules_does_not_pass() {
    let source = source();
    let d = binding::source(&source).unwrap();
    let mut s = work::Session::new(d).unwrap();
    for a in [
        work::Action::Edit {
            text: d.assignment.required_services.clone(),
        },
        work::Action::Save,
        work::Action::Router {
            action: dlms_learning_lab::linux_v6_workspace::Action::Check,
        },
    ] {
        s.action(d, &a, false).unwrap();
    }
    assert!(!d.solved(&s));
    assert!(
        d.assignment
            .checks(&s.assignment)
            .iter()
            .any(|c| !c.passed && c.label.contains("TCP/UDP"))
    );
    let mut state = source.initial.clone();
    for line in source.solution.iter().skip(2) {
        let raw = line.strip_prefix(binding::PREFIX).unwrap();
        let a: work::Action = serde_json::from_str(raw).unwrap();
        let a = match a {
            work::Action::Router {
                action: dlms_learning_lab::linux_v6_workspace::Action::Command { line },
            } => {
                let port = if line.contains("-p udp ") { 8853 } else { 7443 };
                work::Action::Router {
                    action: dlms_learning_lab::linux_v6_workspace::Action::Command {
                        line: line
                            .replace("--dport reports", &format!("--dport {port}"))
                            .replace("--sport report", &format!("--sport {port}")),
                    },
                }
            }
            a => a,
        };
        action(&mut state, d, &a, false);
    }
    let State::LinuxServiceRouter { simulation } = &state else {
        panic!()
    };
    assert!(
        d.assignment
            .router
            .checks(&simulation.assignment.router)
            .iter()
            .all(|c| c.passed),
        "Numeric policy must be correct in the file-only negative control."
    );
    assert!(!engine::solved(&state, &source.goal));
}
#[test]
fn authored_prerequisites_and_all_worked_commands_use_current_implemented_capabilities() {
    let source = source();
    let d = binding::source(&source).unwrap();
    let old = pack::accepted490().unwrap();
    assert_eq!(old.len(), 490);
    for id in &source.prerequisites {
        assert!(old.iter().any(|s| s.id == *id), "Missing prerequisite {id}");
    }
    assert_ne!(d.assignment.required_services, d.example.required_services);
    assert_ne!(d.assignment.router.facts, d.example.router.facts);
    for c in source.guide.iter().flat_map(|g| &g.commands) {
        assert!(!c.simulator_only);
        assert!(!c.command.starts_with(binding::PREFIX));
        let mut p = work::Scope::new(&d.example).unwrap();
        p.reference(
            &d.example,
            &work::Action::Router {
                action: dlms_learning_lab::linux_v6_workspace::Action::Command {
                    line: c.command.clone(),
                },
            },
        )
        .unwrap();
    }
}

#[test]
fn displayed_clients_alone_cannot_satisfy_the_complete_prefix_policy() {
    let source = source();
    let d = binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    for raw in &source.solution {
        let a: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        if let work::Action::Router {
            action: dlms_learning_lab::linux_v6_workspace::Action::Command { line },
        } = &a
            && line.contains("fd66:70:80::/64")
        {
            for client in ["55", "99", "100"] {
                let line = line.replace("fd66:70:80::/64", &format!("fd66:70:80::{client}/128"));
                action(
                    &mut state,
                    d,
                    &work::Action::Router {
                        action: dlms_learning_lab::linux_v6_workspace::Action::Command { line },
                    },
                    false,
                );
            }
            continue;
        }
        action(&mut state, d, &a, false);
    }
    let State::LinuxServiceRouter { simulation } = &state else {
        panic!()
    };
    for c in &d.assignment.router.cases {
        assert!(
            d.assignment
                .router
                .trial(&simulation.assignment.router.state, c)
                .unwrap()
                .steps
                .iter()
                .all(|s| s.check.passed),
            "Displayed sample should pass: {}",
            c.id
        );
    }
    assert!(!engine::solved(&state, &source.goal));
    assert!(
        d.assignment
            .checks(&simulation.assignment)
            .iter()
            .any(|c| !c.passed && c.label.starts_with("FORWARD"))
    );
}
#[test]
fn equivalent_saved_aliases_numeric_protocols_and_split_prefixes_are_valid() {
    let source = source();
    let d = binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    for a in [work::Action::Edit{text:"# Same lookup meanings\nclock\t123/udp\ttime\nreports 7443/tcp report\nreports 8853/udp report\n".into()},work::Action::Save]{action(&mut state,d,&a,false);}
    for raw in source.solution.iter().skip(2) {
        let a: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        if let work::Action::Router {
            action: dlms_learning_lab::linux_v6_workspace::Action::Command { line },
        } = &a
        {
            let line = line
                .replace("-p tcp ", "-p 6 ")
                .replace("-p udp ", "-p 17 ");
            if line.contains("fd66:70:80::/64") {
                for prefix in ["fd66:70:80::/65", "fd66:70:80:0:8000::/65"] {
                    action(
                        &mut state,
                        d,
                        &work::Action::Router {
                            action: dlms_learning_lab::linux_v6_workspace::Action::Command {
                                line: line.replace("fd66:70:80::/64", prefix),
                            },
                        },
                        false,
                    );
                }
            } else {
                action(
                    &mut state,
                    d,
                    &work::Action::Router {
                        action: dlms_learning_lab::linux_v6_workspace::Action::Command { line },
                    },
                    false,
                );
            }
        } else {
            action(&mut state, d, &a, false);
        }
    }
    assert!(
        engine::solved(&state, &source.goal),
        "{:?}",
        engine::checks(&state, &source.goal)
    );
}
#[test]
fn registered_catalog_keeps_all489_previous_scenarios_and195_unit_objects() {
    use dlms_learning_lab::course;
    let before = pack::accepted489().unwrap();
    let now = pack::accepted490().unwrap();
    assert_eq!((before.len(), now.len()), (489, 490));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&now[..489]).unwrap()
    );
    assert_eq!(now[489].id, source().id);
    let old_units = course::accepted489(&before).unwrap();
    let units = course::accepted490(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (195, 196));
    assert_eq!(
        serde_json::to_value(&old_units).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-router-service-names")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-router-service-names")
        .unwrap();
    for prerequisite in ["linux-service-names", "linux-ipv6-router-policy"] {
        assert!(units.iter().position(|u| u.id == prerequisite).unwrap() < at);
    }
    let count = |track: &str| now.iter().filter(|s| s.track == track).count();
    assert_eq!(
        (
            count("CCNA"),
            count("LPIC-1"),
            count("LPIC-2"),
            count("Bash/Shell")
        ),
        (113, 154, 143, 80)
    );
}
