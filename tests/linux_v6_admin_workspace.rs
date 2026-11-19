use dlms_learning_lab::linux_v6_admin_workspace::{Action, Session};
#[path = "support/v6_admin_workspace.rs"]
mod f;
#[test]
fn separate_worked_example_never_earns_assignment_credit_and_edits_clear_success() {
    let d = f::definition();
    let mut probe = dlms_learning_lab::linux_v6_admin_workspace::Scope::new(&d.example);
    for a in &d.worked {
        probe.reference(&d.example, a).unwrap();
    }
    let failures = d
        .example
        .checks(&probe.world)
        .into_iter()
        .filter(|c| !c.passed)
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "worked outcomes: {failures:?}");
    d.validate().unwrap();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    for a in &d.worked {
        s.apply(&d, a, true).unwrap();
    }
    assert!(s.practice.checked);
    assert_eq!(s.assignment, original);
    assert!(!d.solved(&s));
    for a in f::commands(false).iter().take(2) {
        s.apply(&d, a, false).unwrap();
    }
    s.apply(&d, &Action::RunFrame, false).unwrap();
    assert!(s.assignment.last_trial.as_ref().unwrap().result.passed);
    assert!(!d.solved(&s));
    s.apply(&d, &Action::Check, false).unwrap();
    assert!(d.solved(&s));
    s.apply(
        &d,
        &Action::Command {
            input: "sysctl -w net.ipv6.conf.all.forwarding=0".into(),
        },
        false,
    )
    .unwrap();
    assert!(!s.assignment.checked && !d.solved(&s));
    assert!(s.assignment.last_trial.is_none());
    s.assignment.checked = true;
    assert!(
        !d.solved(&s),
        "a stale checked flag cannot override current outcomes"
    );
    let before = s.assignment.clone();
    s.reset(&d, true).unwrap();
    assert_eq!(s.assignment, before);
    s.reset(&d, false).unwrap();
    assert_eq!(
        s.assignment,
        dlms_learning_lab::linux_v6_admin_workspace::Scope::new(&d.assignment)
    );
}
#[test]
fn partial_configuration_drafts_reports_and_current_checks_roundtrip_without_invented_credit() {
    let d = f::definition();
    let mut s = Session::new(&d).unwrap();
    s.apply(&d, &f::commands(false)[0], false).unwrap();
    s.apply(
        &d,
        &Action::Input {
            text: "sysctl -w net.ipv6.conf.all.forwarding=1".into(),
        },
        false,
    )
    .unwrap();
    s.apply(&d, &Action::RunFrame, false).unwrap();
    assert!(!s.assignment.last_trial.as_ref().unwrap().result.passed);
    assert!(!d.solved(&s));
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    d.bound(&restored).unwrap();
    assert!(!d.solved(&restored));
    let mut forged = s.clone();
    forged.assignment.world.settings.global_forwarding = true;
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.last_trial.as_mut().unwrap().result.passed = true;
    assert!(d.bound(&forged).is_err());
    let mut wrong = s.clone();
    wrong.source_id = "another-source".into();
    assert!(d.bound(&wrong).is_err());
}
#[test]
fn unsupported_commands_are_explained_and_invalid_worked_actions_refuse() {
    let d = f::definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.world.clone();
    let a = Action::Command {
        input: "bash -c id".into(),
    };
    s.apply(&d, &a, false).unwrap();
    assert_eq!(s.assignment.world, original);
    assert!(s.assignment.history.is_empty());
    assert!(s.assignment.output.contains("[Not applied]"));
    assert_eq!(s.assignment.input, "bash -c id");
    assert!(!d.solved(&s));
    let mut invalid = d.clone();
    invalid.worked.insert(0, a);
    assert!(invalid.validate().is_err());
    let mut invalid = d.clone();
    invalid.worked.pop();
    assert!(
        invalid.validate().is_err(),
        "worked reachability needs explicit Check"
    );
    let mut invalid = d.clone();
    invalid.example.frames[0].ingress = "absent".into();
    assert!(invalid.validate().is_err());
    let mut invalid = d.clone();
    invalid.example.expected_routes.routes.clear();
    assert!(
        invalid.validate().is_err(),
        "expected data must support declared transit"
    );
}
#[test]
fn host_route_hardcoding_can_pass_the_sample_but_not_the_complete_route_plan() {
    let d = f::definition();
    let mut s = Session::new(&d).unwrap();
    for line in [
        "ip -6 route add 2001:db8:90::80/128 via fe80::1 dev ens3 metric 10",
        "sysctl -w net.ipv6.conf.all.forwarding=1",
    ] {
        s.apply(&d, &Action::Command { input: line.into() }, false)
            .unwrap();
    }
    s.apply(&d, &Action::RunFrame, false).unwrap();
    assert!(s.assignment.last_trial.as_ref().unwrap().result.passed);
    s.apply(&d, &Action::Check, false).unwrap();
    assert!(!d.solved(&s));
    assert!(
        d.assignment
            .checks(&s.assignment.world)
            .iter()
            .any(|c| c.label == "Complete route plan" && !c.passed)
    );
}
#[test]
fn equivalent_split_routes_metrics_and_valid_interface_ids_are_supported_solutions() {
    let d = f::definition();
    let mut s = Session::new(&d).unwrap();
    for line in [
        "ip -6 route add 2001:db8:90::/65 via fe80::1 dev ens3 metric 77",
        "ip -6 route add 2001:db8:90:0:8000::/65 via fe80::1 dev ens3 metric 99",
        "ip -6 address del 2001:db8:20::40/64 dev ens4",
        "ip -6 address add 2001:db8:20::77/64 dev ens4",
        "sysctl -w net.ipv6.conf.all.forwarding=1",
    ] {
        s.apply(&d, &Action::Command { input: line.into() }, false)
            .unwrap();
    }
    s.apply(&d, &Action::Check, false).unwrap();
    let failures = d
        .assignment
        .checks(&s.assignment.world)
        .into_iter()
        .filter(|c| !c.passed)
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{failures:?}");
    assert!(d.solved(&s));
}
#[test]
fn message_test_predictions_do_not_fabricate_a_learned_assignment_route() {
    let d = f::ra_definition();
    d.validate().unwrap();
    let mut s = Session::new(&d).unwrap();
    for a in f::ra_commands().iter().take(3) {
        s.apply(&d, a, false).unwrap();
    }
    s.apply(&d, &Action::Check, false).unwrap();
    let rows = d.assignment.checks(&s.assignment.world);
    assert!(
        rows.iter()
            .filter(|c| c.label == "Accept supplied upstream message"
                || c.label == "Ignore client-facing message")
            .all(|c| c.passed)
    );
    assert!(s.assignment.world.learned.is_empty());
    assert!(!d.solved(&s));
    s.apply(&d, &Action::Observe { message: 0 }, false).unwrap();
    s.apply(&d, &Action::Check, false).unwrap();
    let failures = d
        .assignment
        .checks(&s.assignment.world)
        .into_iter()
        .filter(|c| !c.passed)
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{failures:?}");
    assert!(d.solved(&s));
    s.apply(
        &d,
        &Action::Command {
            input: "ip -6 route add default via fe80::1 dev ens3 metric 100".into(),
        },
        false,
    )
    .unwrap();
    s.apply(&d, &Action::Check, false).unwrap();
    assert!(
        !d.solved(&s),
        "a lower-metric manual route is not active RA use"
    );
    s.apply(
        &d,
        &Action::Command {
            input: "ip -6 route del default via fe80::1 dev ens3 metric 100".into(),
        },
        false,
    )
    .unwrap();
    s.apply(&d, &Action::Check, false).unwrap();
    assert!(d.solved(&s));
}

#[test]
fn saved_file_and_runtime_remain_separate_and_restart_tests_do_not_award_credit() {
    let d = f::file_definition();
    d.validate().unwrap();
    let mut s = Session::new(&d).unwrap();
    s.apply(
        &d,
        &Action::EditFile {
            text: f::GOOD_FILE.into(),
        },
        false,
    )
    .unwrap();
    assert_ne!(s.assignment.file_editor, s.assignment.saved_file);
    assert!(!s.assignment.world.settings.global_forwarding);
    s.apply(&d, &Action::SaveFile, false).unwrap();
    assert!(
        !s.assignment.world.settings.global_forwarding,
        "Save must not load runtime settings"
    );
    s.apply(&d, &Action::RunRestart, false).unwrap();
    let trial = s.assignment.last_restart.as_ref().unwrap();
    assert!(trial.checks.iter().all(|c| c.passed), "{:?}", trial.checks);
    assert!(!d.solved(&s));
    assert!(s.assignment.world.learned.is_empty());
    s.apply(
        &d,
        &Action::Command {
            input: format!(
                "sysctl -p {}",
                dlms_learning_lab::linux_v6_saved_settings::FILE
            ),
        },
        false,
    )
    .unwrap();
    assert!(s.assignment.world.settings.global_forwarding);
    assert!(s.assignment.world.learned.is_empty());
    s.apply(&d, &Action::Observe { message: 0 }, false).unwrap();
    s.apply(&d, &Action::Check, false).unwrap();
    let failures = s
        .assignment
        .checks(&d.assignment)
        .into_iter()
        .filter(|c| !c.passed)
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{failures:?}");
    assert!(d.solved(&s));
    s.apply(
        &d,
        &Action::EditFile {
            text: "# unfinished note\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!d.solved(&s));
    assert!(s.assignment.last_restart.is_none());
}

#[test]
fn alternative_saved_text_is_outcome_graded_and_file_errors_preserve_previous_work() {
    let d = f::file_definition();
    let mut s = Session::new(&d).unwrap();
    let alternate = "; same intended policy\nnet.ipv6.conf.ens4.accept_ra=1\nnet.ipv6.conf.ens3.accept_ra = 2\nnet.ipv6.conf.all.forwarding = 1\nnet.ipv6.conf.ens4.accept_ra=0\n";
    for a in [
        Action::EditFile {
            text: alternate.into(),
        },
        Action::SaveFile,
        Action::Command {
            input: format!(
                "sysctl -p {}",
                dlms_learning_lab::linux_v6_saved_settings::FILE
            ),
        },
        Action::Observe { message: 0 },
        Action::Check,
    ] {
        s.apply(&d, &a, false).unwrap();
    }
    assert!(d.solved(&s));
    let saved = s.assignment.saved_file.clone();
    let world = s.assignment.world.clone();
    s.apply(
        &d,
        &Action::EditFile {
            text: "net.ipv6.conf.all.forwarding=1\nvm.swappiness=0\n".into(),
        },
        false,
    )
    .unwrap();
    s.apply(&d, &Action::SaveFile, false).unwrap();
    assert_eq!(s.assignment.saved_file, saved);
    assert_eq!(s.assignment.world, world);
    assert!(s.assignment.output.contains("File not saved: Line 2"));
    assert!(!d.solved(&s));
    let before = s.assignment.world.clone();
    s.apply(
        &d,
        &Action::Command {
            input: "sysctl -p /etc/sysctl.d/../../etc/passwd".into(),
        },
        false,
    )
    .unwrap();
    assert_eq!(s.assignment.world, before);
    assert!(!d.solved(&s));
}

#[test]
fn saved_file_drafts_history_and_restart_feedback_roundtrip_and_reject_forged_caches() {
    let d = f::file_definition();
    let mut s = Session::new(&d).unwrap();
    for a in [
        Action::EditFile {
            text: f::GOOD_FILE.into(),
        },
        Action::SaveFile,
        Action::RunRestart,
    ] {
        s.apply(&d, &a, false).unwrap();
    }
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    d.bound(&restored).unwrap();
    assert!(!d.solved(&restored));
    let mut forged = s.clone();
    forged.assignment.saved_file = "# fake".into();
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.last_restart.as_mut().unwrap().checks[0].passed = false;
    assert!(d.bound(&forged).is_err());
    s.apply(
        &d,
        &Action::EditFile {
            text: format!("{}# unsaved resume note\n", f::GOOD_FILE),
        },
        false,
    )
    .unwrap();
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_ne!(
        restored.assignment.file_editor,
        restored.assignment.saved_file
    );
    d.bound(&restored).unwrap();
    assert!(!d.solved(&restored));
}

fn synthetic_source(file: bool) -> dlms_learning_lab::pack::Scenario {
    use dlms_learning_lab::{
        engine::{Goal, State},
        linux_v6_admin_source as binding, pack,
    };
    let d = if file {
        f::file_definition()
    } else {
        f::definition()
    };
    let solution = if file {
        d.worked.clone()
    } else {
        f::commands(false)
    };
    let mut source =
        serde_json::from_str::<pack::Pack>(include_str!("../scenarios/linux-v6-lessons-v1.json"))
            .unwrap()
            .scenarios
            .remove(1);
    source.id = d.source_id.clone();
    source.title = "Synthetic IPv6 configuration validation fixture".into();
    source.prerequisites.clear();
    source.guide.clear();
    source.initial = State::LinuxV6Administration {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    source.goal = Goal::LinuxV6Administration {
        definition: Box::new(d),
    };
    source.solution = solution
        .iter()
        .map(|a| format!("{}{}", binding::PREFIX, serde_json::to_string(a).unwrap()))
        .collect();
    source
}

#[test]
fn engine_pack_binding_and_progress_cannot_award_stale_or_wrong_source_credit() {
    use dlms_learning_lab::{
        engine, linux_v6_admin_source as binding, pack,
        progress::{self, Attempt},
    };
    let source = synthetic_source(true);
    binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    let old = state.clone();
    assert!(engine::execute(&mut state, "sysctl -w net.ipv6.conf.all.forwarding=1").is_err());
    assert_eq!(state, old);
    for line in &source.solution {
        engine::execute_for_goal(&mut state, &source.goal, line).unwrap();
    }
    assert!(engine::solved(&state, &source.goal));
    let mut attempt = Attempt::new(&source, 1);
    attempt.state = state;
    assert!(progress::current_solved(&attempt, &source));
    let bad = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&Action::EditFile {
            text: "# unfinished\n".into()
        })
        .unwrap()
    );
    engine::execute_for_goal(&mut attempt.state, &source.goal, &bad).unwrap();
    assert!(!progress::current_solved(&attempt, &source));
    let mut wrong = source.clone();
    wrong.track = "CCNA".into();
    assert!(binding::source(&wrong).is_err());
    let mut wrong = source.clone();
    wrong.version = 2;
    assert!(binding::source(&wrong).is_err());
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![source.clone()],
    })
    .unwrap();
    assert_eq!(pack::load(&raw).unwrap().len(), 1);
    let mut invalid_prerequisite = source.clone();
    invalid_prerequisite
        .prerequisites
        .push("missing-synthetic-prerequisite".into());
    assert!(
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![invalid_prerequisite]
            })
            .unwrap()
        )
        .is_err()
    );
    let mut unreachable = source.clone();
    unreachable.solution = vec![format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&Action::Check).unwrap()
    )];
    assert!(
        pack::load(
            &serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: vec![unreachable]
            })
            .unwrap()
        )
        .is_err()
    );
}

#[test]
fn guide_validation_accepts_real_contextual_commands_but_rejects_unsupported_or_internal_helpers() {
    use dlms_learning_lab::{
        engine::{Goal, State},
        linux_v6_admin_source as binding,
        pack::{CommandExample, LessonStep},
    };
    let mut source = synthetic_source(false);
    let Goal::LinuxV6Administration { definition } = &mut source.goal else {
        unreachable!()
    };
    definition.worked.insert(
        0,
        Action::Command {
            input: "ip -6 address del 2001:db8:120::40/64 dev ens4".into(),
        },
    );
    definition.worked.insert(
        1,
        Action::Command {
            input: "ip -6 address add 2001:db8:120::77/64 dev ens4".into(),
        },
    );
    source.initial = State::LinuxV6Administration {
        simulation: Box::new(Session::new(definition).unwrap()),
    };
    source.guide = vec![LessonStep {
        title: "Repair a configured address".into(),
        why: "Remove the old primary before adding another in this bounded activity.".into(),
        action: "Use the command after deleting the old example address.".into(),
        commands: vec![CommandExample {
            command: "ip -6 address add 2001:db8:120::77/64 dev ens4".into(),
            explanation: "An earlier real action frees the primary slot.".into(),
            simulator_only: false,
        }],
    }];
    binding::source(&source).unwrap();
    for command in [
        "bash -c id",
        "config set ipv6=true",
        "v6-admin-workspace {}",
    ] {
        let mut wrong = source.clone();
        wrong.guide[0].commands[0].command = command.into();
        assert!(binding::source(&wrong).is_err(), "{command}");
    }
    let mut too_many = source.clone();
    too_many.guide[0].commands = vec![source.guide[0].commands[0].clone(); 9];
    assert!(binding::source(&too_many).is_err());
}

#[test]
fn actual_catalog_aware_disk_resume_keeps_separate_file_drafts_commands_and_restart_feedback() {
    use dlms_learning_lab::{
        engine::State,
        linux_v6_admin_source as binding,
        progress::{self, Attempt, Progress},
    };
    let source = synthetic_source(true);
    let d = binding::source(&source).unwrap();
    let mut state = source.initial.clone();
    let State::LinuxV6Administration { simulation } = &mut state else {
        unreachable!()
    };
    for (practice, note) in [(false, "assignment"), (true, "example")] {
        simulation
            .apply(
                d,
                &Action::EditFile {
                    text: f::GOOD_FILE.into(),
                },
                practice,
            )
            .unwrap();
        simulation.apply(d, &Action::SaveFile, practice).unwrap();
        simulation.apply(d, &Action::RunRestart, practice).unwrap();
        simulation
            .apply(
                d,
                &Action::Input {
                    text: format!(
                        "cat /proc/sys/net/ipv6/conf/{}/accept_ra",
                        if practice { "ens4" } else { "ens3" }
                    ),
                },
                practice,
            )
            .unwrap();
        simulation
            .apply(
                d,
                &Action::EditFile {
                    text: format!("{}# {note} draft\n", f::GOOD_FILE),
                },
                practice,
            )
            .unwrap();
    }
    let expected = state.clone();
    let mut attempt = Attempt::new(&source, 1);
    attempt.state = state;
    let mut p = Progress {
        schema_version: 1,
        clock: 2,
        ..Default::default()
    };
    p.attempts.insert(source.id.clone(), attempt);
    p.favorites.insert(source.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-v6-configuration-progress.json");
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
