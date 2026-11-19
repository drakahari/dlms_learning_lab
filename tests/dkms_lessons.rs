use dlms_learning_lab::{
    dkms_source as binding,
    dkms_workspace::{Action, Session},
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/dkms-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn act(
    state: &mut State,
    s: &pack::Scenario,
    a: &Action,
    practice: bool,
) -> Result<String, String> {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!(
            "{}{}",
            binding::PREFIX,
            serde_json::json!({"practice":practice,"action":a})
        ),
    )
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{:?}",
        engine::checks(&state, &s.goal)
    );
    state
}

#[test]
fn both_actual_assignments_and_different_input_examples_reach_real_outcomes() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        for a in &d.example.reference {
            act(&mut state, &s, a, true).unwrap();
        }
        let State::Dkms { simulation } = &state else {
            panic!()
        };
        assert_eq!(simulation.assignment, Session::new(d).unwrap().assignment);
        assert!(d.example.solved(&simulation.example.work));
        assert!(!engine::solved(&state, &s.goal));
        solve(&s);
        assert!(
            s.guide
                .iter()
                .flat_map(|g| &g.commands)
                .all(|c| !c.command.starts_with(binding::PREFIX))
        );
    }
}
#[test]
fn manual_and_chained_supported_alternatives_are_graded_by_per_kernel_state() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let t = &d.assignment;
        let mut state = s.initial.clone();
        let wanted = &t.facts.sources[if t.facts.sources.len() == 1 { 0 } else { 1 }];
        let release = &t.facts.kernels[1].release;
        if t.facts.sources.len() > 1 {
            act(
                &mut state,
                &s,
                &Action::Run {
                    input: format!("dkms add -m {} -v {}", wanted.name, wanted.version),
                },
                false,
            )
            .unwrap();
            act(
                &mut state,
                &s,
                &Action::Run {
                    input: format!(
                        "dkms uninstall {}/{} -k {release}",
                        t.facts.sources[0].name, t.facts.sources[0].version
                    ),
                },
                false,
            )
            .unwrap();
        }
        for input in [
            format!(
                "dkms build -k {release} -m {} -v {}",
                wanted.name, wanted.version
            ),
            format!(
                "dkms install -v {} -m {} -k {release}",
                wanted.version, wanted.name
            ),
        ] {
            act(&mut state, &s, &Action::Run { input }, false).unwrap();
        }
        act(&mut state, &s, &Action::Check, false).unwrap();
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn default_kernel_partial_build_and_broken_success_cannot_get_credit() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let t = &d.assignment;
        let wanted = &t.facts.sources[if t.facts.sources.len() == 1 { 0 } else { 1 }];
        let mut state = s.initial.clone();
        act(&mut state, &s, &Action::Check, false).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        act(
            &mut state,
            &s,
            &Action::Run {
                input: format!(
                    "dkms build {}/{} -k {}",
                    wanted.name, wanted.version, t.facts.kernels[1].release
                ),
            },
            false,
        )
        .unwrap();
        act(&mut state, &s, &Action::Check, false).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        let mut state = solve(&s);
        act(
            &mut state,
            &s,
            &Action::Run {
                input: format!(
                    "dkms uninstall {}/{} -k {}",
                    wanted.name, wanted.version, t.facts.kernels[1].release
                ),
            },
            false,
        )
        .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        act(&mut state, &s, &Action::Check, false).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        assert!(
            engine::checks(&state, &s.goal)
                .iter()
                .any(|(ok, text)| !ok && text.contains("successful build has not placed"))
        );
    }
}
#[test]
fn catalog_bound_save_load_keeps_both_partial_workspaces_and_resets_only_assignment() {
    let catalog = pack::current().unwrap();
    let s = catalog
        .iter()
        .find(|s| s.id == "lpic2-dkms-version-handover")
        .unwrap();
    let d = binding::source(s).unwrap();
    let mut a = Attempt::new(s, 1);
    for practice in [false, true] {
        let t = if practice { &d.example } else { &d.assignment };
        act(&mut a.state, s, &t.reference[0], practice).unwrap();
        act(
            &mut a.state,
            s,
            &Action::CommandDraft {
                text: if practice {
                    "dkms status -m survey-net"
                } else {
                    "dkms install campus-net/1.5 -k"
                }
                .into(),
            },
            practice,
        )
        .unwrap();
        act(&mut a.state, s, &Action::Check, practice).unwrap();
    }
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let mut restored = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    let a = restored.attempts.get_mut(&s.id).unwrap();
    let State::Dkms { simulation } = &a.state else {
        panic!()
    };
    let example = simulation.example.clone();
    act(&mut a.state, s, &Action::Reset, false).unwrap();
    let State::Dkms { simulation } = &a.state else {
        panic!()
    };
    assert_eq!(simulation.example, example);
    assert_eq!(
        simulation.assignment.work,
        dlms_learning_lab::dkms_workspace::Work::new(&d.assignment)
    );
    assert!(!progress::current_solved(a, s));
    assert!(
        progress::save(&path, &restored).is_err(),
        "Never save new source-bound work without its catalog"
    );
}
#[test]
fn forged_task_facts_malformed_actions_and_unbound_execution_are_refused() {
    for s in sources() {
        let mut a = Attempt::new(&s, 1);
        let State::Dkms { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.source_task.push(' ');
        assert!(binding::context(&a, &s).is_err());
        assert!(!progress::current_solved(&a, &s));
        let mut state = s.initial.clone();
        assert!(engine::execute(&mut state, "dkms install campus-net/1.4").is_err());
        for input in [
            format!(
                "{}{{\"practice\":false,\"action\":{{\"op\":\"Run\",\"input\":\"dkms status\"}},\"host\":true}}",
                binding::PREFIX
            ),
            format!("{}{}", binding::PREFIX, "x".repeat(8193)),
        ] {
            let before = serde_json::to_value(&state).unwrap();
            assert!(engine::execute_for_goal(&mut state, &s.goal, &input).is_err());
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        let mut invalid = s.clone();
        if let engine::Goal::Dkms { definition } = &mut invalid.goal {
            definition.assignment.reference = vec![Action::Check];
        }
        assert!(binding::source(&invalid).is_err());
    }
}
#[test]
fn registered_catalog_preserves_all_511_scenarios_and_206_units_exactly() {
    let before = pack::accepted511().unwrap();
    let all = pack::accepted513().unwrap();
    assert_eq!((before.len(), all.len()), (511, 513));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&all[..511]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(sources()).unwrap(),
        serde_json::to_value(&all[511..]).unwrap()
    );
    let old = dlms_learning_lab::course::accepted511(&before).unwrap();
    let units = dlms_learning_lab::course::accepted513(&all).unwrap();
    assert_eq!((old.len(), units.len()), (206, 207));
    let retained = units
        .iter()
        .filter(|u| u.id != "linux-dkms-lifecycle")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let position = units
        .iter()
        .position(|u| u.id == "linux-dkms-lifecycle")
        .unwrap();
    for p in &units[position].prerequisites {
        assert!(units.iter().position(|u| &u.id == p).unwrap() < position);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [115, 154, 164, 80]
    );
}

#[test]
fn harmless_extra_builds_for_the_running_kernel_do_not_make_recovery_impossible() {
    let s = sources().remove(1);
    let mut state = s.initial.clone();
    act(
        &mut state,
        &s,
        &Action::Run {
            input: "dkms build campus-net/1.5".into(),
        },
        false,
    )
    .unwrap();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "An extra uninstalled build preserves every supplied fallback record"
    );
    act(
        &mut state,
        &s,
        &Action::Run {
            input: "dkms uninstall campus-net/1.4".into(),
        },
        false,
    )
    .unwrap();
    act(&mut state, &s, &Action::Check, false).unwrap();
    assert!(!engine::solved(&state, &s.goal));
    act(
        &mut state,
        &s,
        &Action::Run {
            input: "dkms install campus-net/1.4".into(),
        },
        false,
    )
    .unwrap();
    act(&mut state, &s, &Action::Check, false).unwrap();
    assert!(engine::solved(&state, &s.goal));
}

#[test]
fn dkms_lessons_can_be_found_by_utility_name_in_existing_search_and_topic_filters() {
    let f = dlms_learning_lab::catalog::Filter {
        search: "DKMS".into(),
        track: "LPIC-2".into(),
        topic: "DKMS module maintenance".into(),
        ..Default::default()
    };
    for s in sources() {
        assert!(dlms_learning_lab::catalog::matches_filters(
            &s, &f, false, false
        ));
    }
}
