//! Authored package outcomes; all commands evaluate in-process against virtual files.
use dlms_learning_lab::{
    debian_packages_teaching::{self as teaching, Learning},
    debian_packages_workspace as work,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};

fn lessons() -> Vec<Scenario> {
    pack::load(include_str!("../scenarios/debian-packages-v1.json")).unwrap()
}

#[test]
fn optional_guidance_admits_the_actual_parser_subset_and_labels_quoted_helpers() {
    let raw = include_str!("../scenarios/debian-packages-v1.json");
    for bad in [
        "dpkg --audit",
        "apt-get -y install museum-report+",
        "packages {}",
        "help",
        "'help'",
        "dpkg-query -W | cat",
    ] {
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        v["scenarios"][0]["guide"][0]["commands"][0]["command"] = bad.into();
        assert!(pack::load(&v.to_string()).is_err(), "{bad}");
    }
    let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
    v["scenarios"][0]["guide"][0]["commands"][0]["command"] = "'help'".into();
    v["scenarios"][0]["guide"][0]["commands"][0]["simulator_only"] = true.into();
    assert!(pack::load(&v.to_string()).is_ok());
}
fn model(s: &State) -> &dlms_learning_lab::debian_packages::State {
    let State::DebianPackages { simulation } = s else {
        panic!()
    };
    simulation
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::DebianPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn run(s: &Scenario, state: &mut State, command: &str) {
    engine::execute_for_goal(state, &s.goal, command)
        .unwrap_or_else(|e| panic!("{}: {command}: {e}", s.id));
}
fn finish(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        run(s, &mut state, c);
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{}: {:?}",
        s.id,
        definition(s).checks(model(&state))
    );
    state
}

#[test]
fn four_distinct_tasks_have_reachable_real_command_solutions_and_teaching_prerequisites() {
    let labs = lessons();
    assert_eq!(labs.len(), 4);
    for (i, s) in labs.iter().enumerate() {
        assert!(
            !engine::solved(&s.initial, &s.goal),
            "{} initially solved",
            s.id
        );
        assert!(s.solution.iter().all(|c| !c.starts_with("packages ")));
        assert!(!s.explanation.is_empty());
        assert!(
            s.debian_packages_teaching
                .as_ref()
                .unwrap()
                .separate_from(model(&s.initial))
        );
        assert_eq!(
            s.prerequisites,
            if i == 0 {
                vec![]
            } else {
                vec![labs[i - 1].id.clone()]
            }
        );
        finish(s);
    }
    assert_ne!(
        serde_json::to_value(&labs[0].goal).unwrap(),
        serde_json::to_value(&labs[1].goal).unwrap()
    );
}

#[test]
fn dependency_first_explicit_multi_package_and_reverse_retirement_alternatives_pass() {
    let labs = lessons();
    let alternatives: [&[&str]; 4] = [
        &["apt-get -y install museum-lib=1.0-1 museum-report=1.0-1"],
        &[
            "apt-get --error-on=any update",
            "apt-get -y install museum-lib=2.0-1",
            "apt-get -y install museum-report=2.0-1",
        ],
        &[
            "dpkg -r museum-report",
            "apt-get -y install museum-report=1.0-1",
        ],
        &["dpkg -P museum-retired", "dpkg -r museum-report"],
    ];
    for (s, commands) in labs.iter().zip(alternatives) {
        let mut state = s.initial.clone();
        for c in commands {
            run(s, &mut state, c);
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{}: {:?}",
            s.id,
            definition(s).checks(model(&state))
        );
    }
}

#[test]
fn read_only_evidence_never_repairs_and_plausible_wrong_or_partial_work_fails() {
    let labs = lessons();
    let wrong: [&[&str]; 4] = [
        &[
            "dpkg-query -W",
            "apt-cache --important depends museum-report",
        ],
        &[
            "apt-get --error-on=any update",
            "apt-get -y install museum-lib=2.0-1",
        ],
        &[
            "dpkg-query -S /usr/bin/museum-report",
            "apt-get -y install museum-report=1.0-1",
        ],
        &["dpkg -r museum-report", "dpkg -r museum-retired"],
    ];
    for (s, commands) in labs.iter().zip(wrong) {
        let mut state = s.initial.clone();
        for c in commands {
            run(s, &mut state, c);
        }
        let rows = definition(s).checks(model(&state));
        assert!(
            rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0),
            "{} needs honest partial results",
            s.id
        );
        assert!(!engine::solved(&state, &s.goal));
    }
    let s = &labs[1];
    let mut stale = s.initial.clone();
    let before = stale.clone();
    assert!(
        engine::execute_for_goal(
            &mut stale,
            &s.goal,
            "apt-get -y install museum-report=2.0-1"
        )
        .is_err()
    );
    assert_eq!(stale, before);
    // Purge-and-reinstall restores payload but loses local settings: not a repair.
    let s = &labs[2];
    let mut state = s.initial.clone();
    run(s, &mut state, "dpkg -P museum-report");
    run(s, &mut state, "apt-get -y install museum-report=1.0-1");
    assert!(definition(s).checks(model(&state))[1].0);
    assert!(!definition(s).checks(model(&state))[3].0);
    assert!(!engine::solved(&state, &s.goal));
}

#[test]
fn current_success_is_invalidated_by_real_mutation_and_repaired_without_history_tokens() {
    for (i, s) in lessons().iter().enumerate() {
        let mut state = finish(s);
        run(s, &mut state, "dpkg-query -W");
        assert!(engine::solved(&state, &s.goal));
        if i == 3 {
            run(s, &mut state, "apt-get -y install museum-report=1.0-1");
            assert!(!engine::solved(&state, &s.goal));
            run(s, &mut state, "dpkg -r museum-report");
        } else {
            run(s, &mut state, "dpkg -r museum-report");
            assert!(!engine::solved(&state, &s.goal));
            run(
                s,
                &mut state,
                if i == 1 {
                    "apt-get -y install museum-report=2.0-1"
                } else {
                    "apt-get -y install museum-report=1.0-1"
                },
            );
        }
        assert!(engine::solved(&state, &s.goal));
    }
}

#[test]
fn separate_examples_fill_without_execution_and_the_authored_change_has_exact_results() {
    for s in lessons() {
        let l = s.debian_packages_teaching.as_ref().unwrap();
        let assignment = s.initial.clone();
        let mut p = Learning::new(l).unwrap();
        let before = p.simulation.clone();
        p.fill(l).unwrap();
        assert_eq!(p.simulation, before);
        for a in &l.example_actions {
            p.execute(l, &a.action).unwrap();
        }
        assert!(p.checks(l).iter().all(|r| r.0));
        for a in &l.try_change.actions {
            p.execute(l, a).unwrap();
        }
        assert_eq!(
            p.checks(l).iter().map(|r| r.0).collect::<Vec<_>>(),
            l.try_change.expected
        );
        for a in &l.repair {
            p.execute(l, a).unwrap();
        }
        assert!(p.checks(l).iter().all(|r| r.0));
        assert_eq!(s.initial, assignment);
        assert!(!engine::solved(&assignment, &s.goal));
    }
}

#[test]
fn dangerous_or_unmodeled_commands_are_rejected_atomically_with_bounded_feedback() {
    for s in lessons() {
        for command in [
            "cat /etc/passwd",
            "apt-get -y install $(touch /tmp/escape)",
            "dpkg -P museum-lib; reboot",
            "apt-get -y install museum-report+",
            "sudo apt-get update",
            "dpkg-query -W | cat",
        ] {
            let mut state = s.initial.clone();
            let e = engine::execute_for_goal(&mut state, &s.goal, command).unwrap_err();
            assert!(e.len() <= 512 && !e.is_empty());
            assert_eq!(state, s.initial);
        }
    }
}

#[test]
fn every_new_attempt_resumes_independent_example_drafts_and_selected_reset_keeps_other_work() {
    let labs = lessons();
    let mut catalog = pack::bundled().unwrap();
    catalog.extend(labs.clone());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        let l = s.debian_packages_teaching.as_ref().unwrap();
        teaching::prepare(&mut a, l, true).unwrap();
        a.state = finish(s);
        a.workspace
            .as_mut()
            .unwrap()
            .debian_packages_draft
            .as_mut()
            .unwrap()
            .query_argument = "unfinished assignment query".into();
        let practice = a
            .workspace
            .as_mut()
            .unwrap()
            .debian_packages_learning
            .as_mut()
            .unwrap();
        practice.draft.query_argument = "different practice query".into();
        practice.view = dlms_learning_lab::script_teaching::View::Example;
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::load(&path, &catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for s in &labs {
        assert!(progress::current_solved(&restored.attempts[&s.id], s));
    }
    let other = serde_json::to_value(&restored.attempts[&labs[1].id]).unwrap();
    restored
        .attempts
        .insert(labs[0].id.clone(), Attempt::new(&labs[0], 9));
    assert!(!progress::current_solved(
        &restored.attempts[&labs[0].id],
        &labs[0]
    ));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&labs[1].id]).unwrap(),
        other
    );
    assert_eq!(restored.favorites, p.favorites);
}
