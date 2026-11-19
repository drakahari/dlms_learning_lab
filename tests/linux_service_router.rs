//! Model composition fixtures only; not additional authored labs or native evidence.
use dlms_learning_lab::{linux_service_router as work, linux_v6_workspace as router};
use serde::Deserialize;
#[derive(Deserialize)]
struct Previous {
    definition: router::Definition,
    solution: Vec<router::Action>,
}
fn fixture() -> (work::Definition, Vec<work::Action>) {
    let old =
        serde_json::from_str::<Vec<Previous>>(include_str!("fixtures/linux-v6-workspace-v1.json"))
            .unwrap()
            .remove(1);
    let task = |router: router::Task, example: bool| work::Task {
        version: 1,
        router,
        initial_services: if example {
            "preview 8443/tcp previews\nclock 123/udp time\n"
        } else {
            "report 8443/tcp reports\nclock 123/udp time\n"
        }
        .into(),
        required_services: if example {
            "preview 443/tcp previews\nclock 123/udp time\n"
        } else {
            "report 443/tcp reports\nclock 123/udp time\n"
        }
        .into(),
    };
    let assignment = task(old.definition.assignment, false);
    let example = task(old.definition.example, true);
    let mut worked = vec![
        work::Action::Edit {
            text: example.required_services.clone(),
        },
        work::Action::Save,
    ];
    worked.extend(
        old.definition
            .worked
            .into_iter()
            .map(|a| work::Action::Router { action: a }),
    );
    let mut solution = vec![
        work::Action::Edit {
            text: assignment.required_services.clone(),
        },
        work::Action::Save,
    ];
    solution.extend(
        old.solution
            .into_iter()
            .map(|a| work::Action::Router { action: a }),
    );
    (
        work::Definition {
            source_id: "router-services-model".into(),
            assignment,
            example,
            worked,
        },
        solution,
    )
}
fn command(line: &str) -> work::Action {
    work::Action::Router {
        action: router::Action::Command { line: line.into() },
    }
}
fn check() -> work::Action {
    work::Action::Router {
        action: router::Action::Check,
    }
}
fn solve(s: &mut work::Session, d: &work::Definition, actions: &[work::Action], practice: bool) {
    for a in actions {
        s.action(d, a, practice).unwrap();
    }
}
#[test]
fn worked_practice_and_assignment_stay_separate_with_outcome_grading() {
    let (d, solution) = fixture();
    d.validate().unwrap();
    let mut s = work::Session::new(&d).unwrap();
    let before = s.assignment.clone();
    solve(&mut s, &d, &d.worked, true);
    assert_eq!(s.assignment, before);
    assert!(!d.solved(&s));
    assert!(d.example.checks(&s.practice).iter().all(|c| c.passed));
    solve(&mut s, &d, &solution, false);
    assert!(d.solved(&s));
    assert!(s.assignment.router.checked);
}
#[test]
fn editing_saving_and_replacing_rules_are_distinct_and_clear_stale_success() {
    let (d, solution) = fixture();
    let mut s = work::Session::new(&d).unwrap();
    solve(&mut s, &d, &solution, false);
    assert!(d.solved(&s));
    let rules = s.assignment.router.state.filter.clone();
    s.action(
        &d,
        &work::Action::Edit {
            text: d.assignment.initial_services.clone(),
        },
        false,
    )
    .unwrap();
    assert!(!d.solved(&s));
    assert!(!s.assignment.router.checked);
    assert!(s.assignment.router.last_trial.is_none());
    assert_eq!(s.assignment.router.state.filter, rules);
    s.action(&d, &work::Action::Save, false).unwrap();
    s.action(&d, &check(), false).unwrap();
    assert!(!d.solved(&s));
    assert_eq!(s.assignment.router.state.filter, rules);
    assert!(
        d.assignment
            .checks(&s.assignment)
            .iter()
            .any(|c| !c.passed && c.label.contains("report/tcp") && c.detail.contains("8443"))
    );
    s.action(
        &d,
        &work::Action::Edit {
            text: d.assignment.required_services.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &check(), false).unwrap();
    assert!(!d.solved(&s));
    s.action(&d, &work::Action::Save, false).unwrap();
    s.action(&d, &check(), false).unwrap();
    assert!(d.solved(&s));
}
#[test]
fn names_are_resolved_from_saved_text_and_do_not_retroactively_update_rules() {
    let (d, _) = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.action(&d, &command("ip6tables -F FORWARD"), false)
        .unwrap();
    let named = "ip6tables -A FORWARD -i lan0 -o wan0 -p tcp --dport reports -j ACCEPT";
    s.action(&d, &command(named), false).unwrap();
    assert_eq!(
        s.assignment.router.state.filter.base.forward.rules[0].destination_port,
        Some(8443)
    );
    s.action(
        &d,
        &work::Action::Edit {
            text: d.assignment.required_services.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &command(named), false).unwrap();
    assert_eq!(
        s.assignment.router.state.filter.base.forward.rules[1].destination_port,
        Some(8443)
    );
    s.action(&d, &work::Action::Save, false).unwrap();
    s.action(&d, &command(named), false).unwrap();
    assert_eq!(
        s.assignment.router.state.filter.base.forward.rules[2].destination_port,
        Some(443)
    );
    assert_eq!(
        s.assignment.router.state.filter.base.forward.rules[0].destination_port,
        Some(8443)
    );
    s.action(&d, &command("getent services reports/tcp"), false)
        .unwrap();
    assert!(s.assignment.router.output.contains("443/tcp"));
    assert!(s.assignment.router.output.contains("[exit 0]"));
}
#[test]
fn malformed_file_and_unsupported_commands_preserve_saved_file_and_environment() {
    let (d, _) = fixture();
    let mut s = work::Session::new(&d).unwrap();
    let saved = s.assignment.saved_services.clone();
    let initial = s.assignment.router.state.clone();
    s.action(
        &d,
        &work::Action::Edit {
            text: "report wrong/tcp reports\n".into(),
        },
        false,
    )
    .unwrap();
    let before = s.clone();
    assert!(s.action(&d, &work::Action::Save, false).is_err());
    assert_eq!(s, before);
    assert_eq!(s.assignment.saved_services, saved);
    for line in [
        "cat /etc/passwd",
        "cat ../../personal",
        "getent services $(id)",
        "ip6tables -A INPUT -p tcp --dport reports -j ACCEPT",
        "ip6tables -A FORWARD -p tcp --dport missing -j ACCEPT",
        "sysctl -w net.ipv6.conf.all.forwarding=0",
    ] {
        s.action(&d, &command(line), false).unwrap();
        assert_eq!(s.assignment.router.state, initial, "{line}");
        assert!(!s.assignment.router.checked);
        assert!(!d.solved(&s));
    }
    assert!(s.assignment.router.output.contains("Unsupported"));
}
#[test]
fn equivalent_file_format_alias_choices_and_supported_rule_alternatives_pass() {
    let (d, solution) = fixture();
    let mut s = work::Session::new(&d).unwrap();
    s.action(
        &d,
        &work::Action::Edit {
            text: "# Equivalent lookup meanings\nclock\t123/udp\ttime\nreports 443/tcp report\n"
                .into(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &work::Action::Save, false).unwrap();
    for a in solution.iter().skip(2) {
        let a = match a {
            work::Action::Router {
                action: router::Action::Command { line },
            } => command(
                &line
                    .replace("--dport 443", "--dport reports")
                    .replace("--sport 443", "--sport report")
                    .replace("-p tcp ", "-p 6 "),
            ),
            a => a.clone(),
        };
        s.action(&d, &a, false).unwrap();
    }
    assert!(d.solved(&s), "{:?}", d.assignment.checks(&s.assignment));
    s.action(
        &d,
        &work::Action::Edit {
            text: "reports 443/tcp report\nclock 124/udp time\n".into(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &work::Action::Save, false).unwrap();
    s.action(&d, &check(), false).unwrap();
    assert!(!d.solved(&s));
}
#[test]
fn reset_serialized_resume_forged_reports_and_invalid_sources_are_checked() {
    let (d, solution) = fixture();
    let mut s = work::Session::new(&d).unwrap();
    solve(&mut s, &d, &solution, false);
    solve(&mut s, &d, &d.worked, true);
    s.action(
        &d,
        &work::Action::Router {
            action: router::Action::Input {
                text: "getent services reports/tcp".into(),
            },
        },
        false,
    )
    .unwrap();
    s.action(
        &d,
        &work::Action::Router {
            action: router::Action::RunTraffic,
        },
        false,
    )
    .unwrap();
    let text = serde_json::to_string(&s).unwrap();
    let mut restored: work::Session = serde_json::from_str(&text).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(!d.solved(&restored));
    let practice = restored.practice.clone();
    restored.reset(&d, false).unwrap();
    assert_eq!(restored.practice, practice);
    assert_eq!(
        restored.assignment,
        work::Scope::new(&d.assignment).unwrap()
    );
    let mut forged = s.clone();
    forged.assignment.router.last_trial.as_mut().unwrap().steps[0]
        .check
        .passed = false;
    assert!(d.bound(&forged).is_err());
    let mut bad = d.clone();
    bad.assignment.version = 2;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example.required_services = bad.assignment.required_services.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.worked.insert(0, command("cat /etc/passwd"));
    assert!(bad.validate().is_err());
    let too_long = serde_json::json!({"kind":"edit","text":"x".repeat(32769)});
    assert!(serde_json::from_value::<work::Action>(too_long).is_err());
}

#[test]
fn reference_actions_keep_normal_logs_and_require_a_fresh_check_after_commands() {
    let (d, solution) = fixture();
    let mut s = work::Session::new(&d).unwrap();
    solve(&mut s, &d, &solution, false);
    assert!(s.assignment.router.checked);
    s.assignment
        .reference(&d.assignment, &command("ip6tables -S"))
        .unwrap();
    assert!(
        !s.assignment.router.checked,
        "Authoring commands must invalidate the same checked state as native commands."
    );
    let mut a = work::Scope::new(&d.example).unwrap();
    let mut b = a.clone();
    for action in &d.worked {
        a.apply(&d.example, action).unwrap();
        b.reference(&d.example, action).unwrap();
        assert_eq!(
            a, b,
            "A valid authoring recipe must reproduce the real learner interaction."
        );
    }
}

fn synthetic_source() -> dlms_learning_lab::pack::Scenario {
    use dlms_learning_lab::{
        engine::{Goal, State},
        linux_service_router_source as binding, pack,
    };
    let (d, solution) = fixture();
    let mut source =
        serde_json::from_str::<pack::Pack>(include_str!("../scenarios/linux-v6-lessons-v1.json"))
            .unwrap()
            .scenarios
            .remove(1);
    source.id = d.source_id.clone();
    source.initial = State::LinuxServiceRouter {
        simulation: Box::new(work::Session::new(&d).unwrap()),
    };
    source.goal = Goal::LinuxServiceRouter {
        definition: Box::new(d),
    };
    source.solution = solution
        .iter()
        .map(|a| format!("{}{}", binding::PREFIX, serde_json::to_string(a).unwrap()))
        .collect();
    source
}
#[test]
fn source_bound_engine_actions_and_progress_require_real_current_outcomes() {
    use dlms_learning_lab::{
        engine, linux_service_router_source as binding,
        progress::{self, Attempt},
    };
    let source = synthetic_source();
    binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    assert!(engine::execute(&mut state, "ip6tables -S").is_err());
    assert!(!engine::solved(&state, &source.goal));
    let old = state.clone();
    let malformed = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&command("cat /etc/passwd")).unwrap()
    );
    assert!(engine::execute_for_goal(&mut state, &source.goal, &malformed).is_err());
    assert_eq!(state, old);
    for line in &source.solution {
        engine::execute_for_goal(&mut state, &source.goal, line).unwrap();
    }
    assert!(engine::solved(&state, &source.goal));
    let mut attempt = Attempt::new(&source, 1);
    attempt.state = state;
    assert!(progress::current_solved(&attempt, &source));
    let edit = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&work::Action::Edit {
            text: "report 9999/tcp reports\nclock 123/udp time\n".into()
        })
        .unwrap()
    );
    engine::execute_for_goal(&mut attempt.state, &source.goal, &edit).unwrap();
    assert!(!progress::current_solved(&attempt, &source));
    let mut bad = source.clone();
    bad.version = 2;
    assert!(binding::source(&bad).is_err());
    let mut bad = source.clone();
    bad.track = "CCNA".into();
    assert!(binding::source(&bad).is_err());
}
#[test]
fn source_bound_progress_roundtrip_keeps_distinct_unfinished_drafts_and_reports() {
    use dlms_learning_lab::{
        engine::State,
        linux_service_router_source as binding,
        progress::{self, Attempt, Progress},
    };
    let source = synthetic_source();
    let d = binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    let State::LinuxServiceRouter { simulation } = &mut state else {
        unreachable!()
    };
    for (practice, text) in [
        (
            false,
            "# assignment draft\nreport 8443/tcp reports\nclock 123/udp time\n",
        ),
        (
            true,
            "# example draft\npreview 8443/tcp previews\nclock 123/udp time\n",
        ),
    ] {
        simulation
            .action(d, &work::Action::Edit { text: text.into() }, practice)
            .unwrap();
        simulation
            .action(
                d,
                &work::Action::Router {
                    action: router::Action::Input {
                        text: if practice {
                            "getent services previews/tcp"
                        } else {
                            "getent services reports/tcp"
                        }
                        .into(),
                    },
                },
                practice,
            )
            .unwrap();
        simulation
            .action(
                d,
                &work::Action::Router {
                    action: router::Action::RunTraffic,
                },
                practice,
            )
            .unwrap();
    }
    let expected = state.clone();
    let mut a = Attempt::new(&source, 1);
    a.state = state;
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(source.id.clone(), a);
    p.favorites.insert(source.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-services-progress.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&source)).unwrap();
    let restored = progress::load(&path, std::slice::from_ref(&source)).unwrap();
    assert_eq!(restored.attempts[&source.id].state, expected);
    assert!(restored.favorites.contains(&source.id));
    binding::context(&restored.attempts[&source.id], &source).unwrap();
    assert!(!progress::current_solved(
        &restored.attempts[&source.id],
        &source
    ));
}
