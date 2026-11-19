use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    links_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
fn labs() -> Vec<pack::Scenario> {
    pack::accepted213()
        .unwrap()
        .into_iter()
        .filter(|s| matches!(s.initial, State::Links { .. }))
        .collect()
}
#[test]
fn links_unit_real_solutions_and_current_registration_preserve_every_accepted209_object() {
    let all = pack::accepted213().unwrap();
    let old = pack::accepted209().unwrap();
    assert_eq!(all.len(), 213);
    assert_eq!(
        serde_json::to_value(&all[..209]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let units = course::accepted213(&all).unwrap();
    assert_eq!(units.len(), 57);
    assert_eq!(
        serde_json::to_value(&units[..56]).unwrap(),
        serde_json::to_value(course::accepted209(&old).unwrap()).unwrap()
    );
    for s in labs() {
        let mut state = s.initial.clone();
        assert!(!engine::solved(&state, &s.goal));
        for cmd in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, cmd).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
    }
}
#[test]
fn links_practice_real_commands_never_solve_assignment_and_saved_unfinished_commands_resume() {
    let all = pack::accepted213().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs() {
        let mut a = Attempt::new(&s, 1);
        let (State::Links { simulation }, Goal::Links { definition }) = (&mut a.state, &s.goal)
        else {
            panic!()
        };
        for cmd in &definition.example_commands {
            simulation.run(definition, cmd, true).unwrap();
        }
        assert!(definition.checks(simulation, true).iter().all(|r| r.0));
        assert!(!engine::solved(&a.state, &s.goal));
        let State::Links { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.command = "ln '/work/a name' ".into();
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
#[test]
fn link_commands_have_honest_flags_quotes_and_no_shell_execution() {
    let s = labs().remove(0);
    let Goal::Links { definition } = &s.goal else {
        panic!()
    };
    let mut world = definition.initial.clone();
    work::execute(&mut world, "cp -- /work/report '/work/-s'").unwrap();
    work::execute(&mut world, "ln -- -s /delivery/literal").unwrap();
    assert_eq!(
        world.stat("/work/-s").unwrap().inode,
        world.stat("/delivery/literal").unwrap().inode
    );
    for bad in [
        "ln -f /work/report /delivery/new",
        "ln /work/report -f",
        "cp /work/report /archive/snapshot; rm /work/report",
        "cat $(cat /etc/passwd)",
        "cat /work/report > /work/keep",
        "rm -r /work/report",
        "stat -c %i /work/report",
        "cd /archive",
    ] {
        let before = world.clone();
        assert!(work::execute(&mut world, bad).is_err(), "{bad}");
        assert_eq!(world, before);
    }
    assert!(work::execute(&mut world, "cat /home/learner/.ssh/id_ed25519").is_err());
}
#[test]
fn link_alternatives_wrong_copies_partial_repairs_and_post_success_damage_are_state_graded() {
    for s in labs() {
        let mut state = s.initial.clone();
        for command in &s.solution {
            let alt = if command.starts_with("ln -s ../work/") {
                command.replace("ln -s ../work/", "ln -s /work/")
            } else if command.starts_with("ln -s ../archive/") {
                command.replace("ln -s ../archive/", "ln -s /archive/")
            } else {
                command.clone()
            };
            engine::execute_for_goal(&mut state, &s.goal, &alt).unwrap();
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let victim = if s.id.ends_with("identity") {
            "/delivery/live"
        } else if s.id.ends_with("relative") {
            "/delivery/latest"
        } else if s.id.ends_with("move") {
            "/delivery/receipt"
        } else {
            "/delivery/current"
        };
        engine::execute_for_goal(&mut state, &s.goal, &format!("rm {victim}")).unwrap();
        assert!(!engine::solved(&state, &s.goal));
    }
    let s = labs().remove(0);
    let mut state = s.initial.clone();
    engine::execute_for_goal(&mut state, &s.goal, "rm /delivery/live").unwrap();
    engine::execute_for_goal(&mut state, &s.goal, "cp /work/report /delivery/live").unwrap();
    engine::execute_for_goal(&mut state, &s.goal, "cp /work/report /archive/snapshot").unwrap();
    let rows = engine::checks(&state, &s.goal);
    assert!(!rows[0].0);
    assert!(rows.iter().any(|r| r.0));
    assert!(rows[0].1.contains("Equal text"));
}
#[test]
fn link_editor_experiments_are_isolated_and_unsaved_edits_invalidate_success() {
    let s = labs().remove(0);
    let Goal::Links { definition } = &s.goal else {
        panic!()
    };
    let mut session = work::Session::new(definition);
    for command in &definition.example_commands {
        session.run(definition, command, true).unwrap();
    }
    session.practice.editor_path = "/practice/in/brief".into();
    session.practice.editor_open_path = "/practice/in/brief".into();
    session.practice.editor_text = "Changed through an alias\n".into();
    session.practice.editor_loaded = true;
    assert!(!definition.checks(&session, true).iter().all(|r| r.0));
    session.save_editor(definition, true).unwrap();
    assert_eq!(
        session
            .practice
            .world
            .read_file("/practice/out/live")
            .unwrap(),
        "Changed through an alias\n"
    );
    assert_eq!(
        session
            .practice
            .world
            .read_file("/practice/archive/copy")
            .unwrap(),
        "Practice brief\n"
    );
    assert_eq!(session.assignment.world, definition.initial);
    let before = session.clone();
    session.practice.editor_path = "/practice/archive/copy".into();
    assert!(session.save_editor(definition, true).is_err());
    session.practice.editor_path = before.practice.editor_path.clone();
    assert_eq!(session, before);
}
#[test]
fn nested_map_roundtrip_forged_history_and_source_free_save_are_checked() {
    let all = pack::accepted213().unwrap();
    let s = labs().remove(0);
    let mut a = Attempt::new(&s, 1);
    let value = serde_json::to_value(&a).unwrap();
    let restored: Attempt = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), value);
    let State::Links { simulation } = &mut a.state else {
        panic!()
    };
    simulation.assignment.world.unlink("/work/keep").unwrap();
    assert!(work::context(&a, &s).is_err());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private.json");
    assert!(progress::save_with_catalog(&path, &p, &all).is_err());
    assert!(!path.exists());
    assert!(progress::save(&path, &p).is_err());
    let mut bad = s.clone();
    if let State::Links { simulation } = &mut bad.initial {
        simulation.assignment.command = "invented initial draft".into();
    }
    let text = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![bad],
    })
    .unwrap();
    assert!(pack::load(&text).is_err());
}
