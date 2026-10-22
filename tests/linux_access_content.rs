use dlms_learning_lab::{
    engine::{self, Goal, State},
    filesystem::{Access, Requirement},
    pack,
    progress::{self, Attempt, Progress},
    terminal_teaching::{self, Learning},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "linux-access-policy")
        .collect()
}
fn run(s: &mut State, commands: &[&str]) {
    for cmd in commands {
        engine::execute(s, cmd).unwrap();
    }
}
#[test]
fn access_assignments_accept_alternates_reject_policy_mistakes_and_clear_current_success() {
    let alternatives = [
        vec!["chmod 0640 ./brief.txt", "chgrp -- review /work/brief.txt"],
        vec!["chmod 0730 /work/drop"],
        vec![
            "cd incoming",
            "cp roster-good.txt ../publish/roster.txt",
            "cp notice-good.txt ../publish/notice.txt",
        ],
        vec![
            "chmod 700 staging",
            "mv staging/ready.txt staging/intermediate.txt",
            "mv staging/intermediate.txt staging/delivered.txt",
            "chmod 500 staging",
        ],
    ];
    let wrongs = [
        vec!["chmod 644 brief.txt"],
        vec!["chmod u=rwx,g=w,o= drop"],
        vec![
            "cp incoming/roster-old.txt publish/roster.txt",
            "cp incoming/notice-good.txt publish/notice.txt",
        ],
        vec![
            "chmod u+w staging",
            "cp staging/ready.txt staging/delivered.txt",
            "rm staging/ready.txt",
            "chmod u-w staging",
        ],
    ];
    let breakage = [
        "chmod o+r brief.txt",
        "chmod g+r drop",
        "chmod u+w /work/publish",
        "chmod u+w staging",
    ];
    let tasks = labs();
    assert_eq!(tasks.len(), 4);
    for (i, s) in tasks.iter().enumerate() {
        let mut state = s.initial.clone();
        run(&mut state, &alternatives[i]);
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        engine::execute(&mut state, breakage[i]).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        let mut wrong = s.initial.clone();
        run(&mut wrong, &wrongs[i]);
        let checks = engine::checks(&wrong, &s.goal);
        assert!(
            checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0),
            "{} {checks:?}",
            s.id
        );
        assert!(!workspace::issues(&wrong, &s.goal).is_empty());
        if i == 3 {
            assert_eq!(
                checks.iter().filter(|c| !c.0).count(),
                1,
                "recreation must fail specifically on ownership"
            );
            assert!(workspace::issues(&wrong, &s.goal)[0].contains("new owner"));
        }
    }
}
#[test]
fn access_examples_and_actual_suggested_changes_have_specific_independent_effects() {
    for (i, s) in labs().iter().enumerate() {
        let lesson = s.terminal_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let mut t = lesson.fixture.clone();
        for cmd in &lesson.example {
            t.execute(cmd).unwrap();
        }
        assert!(lesson.outcomes.iter().all(|r| r.check(&t).0));
        match i {
            0 => {
                t.execute("chmod g-r menu.txt").unwrap();
                assert!(!t.access("blair", "/work/menu.txt", Access::ReadFile));
            }
            1 => {
                t.execute("chmod g+r inbox").unwrap();
                assert!(t.access("blair", "/work/inbox", Access::ListNames));
            }
            2 => {
                let before = t.clone();
                assert!(
                    t.execute("touch cabinet/new.txt")
                        .unwrap_err()
                        .contains("Permission denied")
                );
                assert_eq!(t, before);
            }
            _ => {
                let before = t.clone();
                assert!(
                    t.execute("mv dispatch/sent.txt dispatch/again.txt")
                        .unwrap_err()
                        .contains("Permission denied")
                );
                assert_eq!(t, before);
            }
        }
    }
}
#[test]
fn partial_access_work_is_not_confused_with_a_complete_handover_or_unchanged_output() {
    for (i, s) in labs().iter().enumerate() {
        let mut state = s.initial.clone();
        if i == 1 {
            engine::execute(&mut state, "chmod g=w drop").unwrap();
        } else {
            engine::execute(&mut state, &s.solution[0]).unwrap();
        }
        assert!(!engine::solved(&state, &s.goal));
        let checks = engine::checks(&state, &s.goal);
        assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
    }
    let s = &labs()[2];
    let mut state = s.initial.clone();
    let before = state.clone();
    assert!(
        engine::execute(&mut state, "chmod 666 publish/roster.txt")
            .unwrap_err()
            .contains("only the owner")
    );
    assert_eq!(state, before);
    assert!(
        engine::execute(&mut state, "touch publish/new.txt")
            .unwrap_err()
            .contains("Permission denied")
    );
    assert_eq!(state, before);
}
#[test]
fn all_access_outcomes_reject_a_broken_target_and_ownership_policy_is_explicit() {
    for s in labs() {
        let mut good = s.initial.clone();
        for cmd in &s.solution {
            engine::execute(&mut good, cmd).unwrap();
        }
        assert!(engine::solved(&good, &s.goal));
        let Goal::Filesystem { requirements } = &s.goal else {
            panic!()
        };
        for r in requirements {
            let mut broken = good.clone();
            let State::Filesystem { tree } = &mut broken else {
                panic!()
            };
            match r {
                Requirement::Access { path, allowed, .. } => {
                    tree.nodes.get_mut(path).unwrap().mode = if *allowed { 0 } else { 0o777 };
                }
                Requirement::Contents { path, .. } => {
                    tree.files
                        .insert(path.clone(), "wrong retained text".into());
                }
                Requirement::Ownership { path, owner, .. } => {
                    tree.nodes.get_mut(path).unwrap().owner =
                        if owner == "alex" { "blair" } else { "alex" }.into();
                }
                Requirement::Mode { path, mode } => {
                    tree.nodes.get_mut(path).unwrap().mode = mode ^ 0o200
                }
                Requirement::Absent { path } => {
                    let mut n = tree.nodes["/work"].clone();
                    n.directory = false;
                    tree.nodes.insert(path.clone(), n);
                    tree.files.insert(path.clone(), String::new());
                }
                _ => panic!("unexpected access-unit outcome"),
            }
            assert!(tree.valid(), "{} {r:?}", s.id);
            assert!(!r.check(tree).0, "{} {r:?}", s.id);
            assert!(!engine::solved(&broken, &s.goal));
        }
    }
}
#[test]
fn access_practice_and_assignment_drafts_resume_without_awarding_credit_and_reset_only_selected() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let old = &all[3];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    for s in labs() {
        let mut a = Attempt::new(&s, 2);
        workspace::prepare(&mut a);
        terminal_teaching::prepare(&mut a, s.terminal_teaching.as_ref().unwrap(), true);
        let original = a.state.clone();
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap();
        l.command = "ls -ld /work".into();
        l.execute();
        l.command = "practice unfinished".into();
        assert_eq!(a.state, original);
        assert!(!progress::current_solved(&a, &s));
        a.workspace.as_mut().unwrap().command = "assignment unfinished".into();
        p.attempts.insert(s.id.clone(), a);
        let mut check = Learning::new(s.terminal_teaching.as_ref().unwrap());
        check.command = "id".into();
        check.execute();
        assert!(check.transcript[0].contains("uid=1000(alex)"));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_vec(&restored).unwrap(),
        serde_json::to_vec(&p).unwrap()
    );
    let selected = &labs()[1];
    restored
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 3));
    for (id, a) in p.attempts {
        if id != selected.id {
            assert_eq!(
                serde_json::to_vec(&restored.attempts[&id]).unwrap(),
                serde_json::to_vec(&a).unwrap()
            );
        }
    }
}

#[test]
fn common_incomplete_and_overbroad_repairs_do_not_satisfy_access_policy() {
    let tasks = labs();
    for (i, commands) in [
        (0, vec!["chgrp review brief.txt"]),
        (0, vec!["chmod 640 brief.txt"]),
        (1, vec!["chmod g=rwx drop"]),
        (2, vec!["cp incoming/roster-good.txt publish/roster.txt"]),
        (
            3,
            vec![
                "chmod u+w staging",
                "mv staging/ready.txt staging/delivered.txt",
            ],
        ),
    ] {
        let mut state = tasks[i].initial.clone();
        run(&mut state, &commands);
        assert!(
            !engine::solved(&state, &tasks[i].goal),
            "{} {commands:?}",
            tasks[i].id
        );
    }
    // Perfect text/mode output does not excuse replacing someone else's existing entry.
    let s = &tasks[2];
    let mut state = s.initial.clone();
    run(
        &mut state,
        &[
            "chmod u+w publish",
            "rm publish/roster.txt",
            "cp incoming/roster-good.txt publish/roster.txt",
            "chmod 660 publish/roster.txt",
            "cp incoming/notice-good.txt publish/notice.txt",
            "chmod 550 publish",
        ],
    );
    let failed: Vec<_> = engine::checks(&state, &s.goal)
        .into_iter()
        .filter(|c| !c.0)
        .collect();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].1.contains("belongs to blair"));
}
