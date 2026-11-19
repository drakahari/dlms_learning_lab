//! Registered outcomes, source binding and progress round trips, not recipe matching.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack, progress, recursive_tree_source as source, recursive_tree_workspace as work,
};
fn lessons() -> Vec<pack::Scenario> {
    pack::accepted298()
        .unwrap()
        .into_iter()
        .filter(|s| s.id.starts_with("lpic1-tree-"))
        .collect()
}
fn d(s: &pack::Scenario) -> &work::Definition {
    source::source(s).unwrap()
}

#[test]
fn registered_sources_reach_both_worlds_without_example_credit_and_keep_all296_prior_values() {
    let prior = pack::accepted296().unwrap();
    let all = pack::accepted298().unwrap();
    assert_eq!(all.len(), 298);
    assert_eq!(
        serde_json::to_value(&all[..296]).unwrap(),
        serde_json::to_value(&prior).unwrap()
    );
    let old = course::accepted296(&prior).unwrap();
    let units = course::accepted298(&all).unwrap();
    assert_eq!((old.len(), units.len()), (96, 97));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-recursive-handovers")
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(old).unwrap()
    );
    for (track, n) in [
        ("CCNA", 74),
        ("LPIC-1", 103),
        ("LPIC-2", 45),
        ("Bash/Shell", 76),
    ] {
        let actual = all
            .iter()
            .filter(|s| {
                s.track == track || (track == "Bash/Shell" && s.track == "Bash/Shell Scripting")
            })
            .count();
        assert_eq!(actual, n, "{track}");
    }
    for s in lessons() {
        d(&s).validate_reachability().unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for c in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        let mut session = work::Session::new(d(&s)).unwrap();
        let before = session.assignment.clone();
        for c in &d(&s).example_commands {
            session.run(d(&s), c, true).unwrap();
        }
        assert!(session.check(d(&s), true).unwrap().iter().all(|r| r.0));
        assert_eq!(session.assignment, before);
        assert!(!d(&s).checks(&session).iter().all(|r| r.0));
        assert!(
            s.guide
                .iter()
                .all(|g| !g.why.is_empty() && !g.action.is_empty())
        );
        assert!(s.solution.last().unwrap() == work::CHECK_ACTION);
    }
}

#[test]
fn supported_alternatives_partial_missing_hidden_and_current_invalidation_are_state_based() {
    let labs = lessons();
    let s = &labs[0];
    let mut state = s.initial.clone();
    for c in [
        "umask 077",
        "mkdir /work/private-copy",
        "mkdir /work/private-copy/docs",
        "mkdir /work/private-copy/empty-cache",
        "cp /work/incoming/README.txt /work/private-copy/README.txt",
        "cp /work/incoming/.index /work/private-copy/.index",
        "cp /work/incoming/docs/runbook.txt /work/private-copy/docs/runbook.txt",
        work::CHECK_ACTION,
    ] {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
    engine::execute_for_goal(
        &mut state,
        &s.goal,
        "chmod 644 /work/private-copy/README.txt",
    )
    .unwrap();
    assert!(!engine::solved(&state, &s.goal));
    engine::execute_for_goal(&mut state, &s.goal, work::CHECK_ACTION).unwrap();
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|r| !r.0 && r.1.contains("private copy"))
    );
    let s = &labs[1];
    let mut state = s.initial.clone();
    engine::execute_for_goal(&mut state, &s.goal, "rm -r /work/old-project").unwrap();
    engine::execute_for_goal(&mut state, &s.goal, work::CHECK_ACTION).unwrap();
    let rows = engine::checks(&state, &s.goal);
    assert!(rows[4].0 && !rows[0].0 && !rows[1].0);
    assert!(rows[0].1.contains(".index"));
    let mut state = s.initial.clone();
    for c in [
        "rm -r /work/handover",
        "cp -R /work/old-project /work/handover",
        "rm -r /work/old-project",
        work::CHECK_ACTION,
    ] {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
}

#[test]
fn symmetric_source_and_save_guards_reject_foreign_context_without_rewriting_originals() {
    for s in lessons() {
        let mut p = progress::Progress::default();
        p.attempts
            .insert(s.id.clone(), progress::Attempt::new(&s, 1));
        let a = p.attempts.get_mut(&s.id).unwrap();
        source::context(a, &s).unwrap();
        let State::RecursiveTree { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "unfinished assignment".into();
        simulation.practice.command = "unfinished example".into();
        simulation.assignment.checked = true;
        assert!(!progress::current_solved(a, &s));
        let raw = serde_json::to_vec(&p).unwrap();
        let restored: progress::Progress = serde_json::from_slice(&raw).unwrap();
        source::context(&restored.attempts[&s.id], &s).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        let mut wrong = p.attempts[&s.id].clone();
        let State::RecursiveTree { simulation } = &mut wrong.state else {
            panic!()
        };
        simulation.assignment.role = work::Role::Practice;
        assert!(source::context(&wrong, &s).is_err());
        let mut foreign = s.initial.clone();
        let goal = Goal::Service { port: 8080 };
        let before = foreign.clone();
        assert!(engine::execute_for_goal(&mut foreign, &goal, "rm -r /work/incoming").is_err());
        assert_eq!(foreign, before);
        let mut source = s.clone();
        source.id = "foreign-source".into();
        assert!(source::source(&source).is_err());
    }
}

#[test]
fn actual_safe_save_load_preserves_favorites_both_drafts_and_current_state_with_reset() {
    let labs = lessons();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = progress::Attempt::new(s, 1);
        for c in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        let State::RecursiveTree { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "ls -la /work ".into();
        simulation.practice.command = "unfinished practice".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let root = std::env::temp_dir().join(format!("recursive-tree-progress-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let old = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(old, std::fs::read(&path).unwrap());
    let mut read = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&read).unwrap()
    );
    for s in &labs {
        let a = read.attempts.get_mut(&s.id).unwrap();
        source::context(a, s).unwrap();
        assert!(progress::current_solved(a, s));
        let State::RecursiveTree { simulation } = &mut a.state else {
            panic!()
        };
        assert_eq!(simulation.assignment.command, "ls -la /work ");
        assert_eq!(simulation.practice.command, "unfinished practice");
        simulation
            .run(
                d(s),
                &format!(
                    "chmod 000 {}/README.txt",
                    d(s).assignment.task.destination()
                ),
                false,
            )
            .unwrap();
        assert!(!progress::current_solved(a, s));
    }
    let second = read.attempts[&labs[1].id].clone();
    read.attempts
        .insert(labs[0].id.clone(), progress::Attempt::new(&labs[0], 2));
    assert_eq!(
        serde_json::to_value(second).unwrap(),
        serde_json::to_value(&read.attempts[&labs[1].id]).unwrap()
    );
    assert!(!progress::current_solved(
        &read.attempts[&labs[0].id],
        &labs[0]
    ));
    assert_eq!(read.favorites, p.favorites);
}
