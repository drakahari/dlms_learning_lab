//! Actual authored exploration, with assignment and practice drafts isolated.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    links_workspace as work, pack,
    progress::{self, Attempt, Progress},
};

fn labs() -> Vec<pack::Scenario> {
    pack::ccna_pilot()
        .unwrap()
        .into_iter()
        .filter(|s| s.id.starts_with("lpic1-links-"))
        .collect()
}
fn inspect(session: &mut work::Session, definition: &work::Definition, path: &str) {
    let before = session.practice.clone();
    let assignment = session.assignment.clone();
    let output = session
        .run(definition, &format!("ls -i {path}"), true)
        .unwrap();
    let rows: Vec<(String, u32)> = output
        .lines()
        .map(|line| {
            let (id, name) = line.trim_start().split_once(' ').unwrap();
            (name.into(), id.parse().unwrap())
        })
        .collect();
    let expected: Vec<_> = before
        .world
        .directory_inode_entries(path)
        .unwrap()
        .into_iter()
        .filter(|(n, _)| !n.starts_with('.'))
        .collect();
    assert_eq!(rows, expected);
    assert_eq!(session.practice.world, before.world);
    assert_eq!(session.practice.history, before.history);
    assert_eq!(session.practice.command, before.command);
    assert_eq!(session.practice.editor_text, before.editor_text);
    assert_eq!(session.assignment, assignment);
}
#[test]
fn every_taught_inode_query_uses_real_practice_state_without_assignment_credit() {
    let all = labs();
    assert_eq!(all.len(), 4);
    for source in all {
        let Goal::Links { definition } = &source.goal else {
            panic!()
        };
        let mut session = work::Session::new(definition);
        session.assignment.command = "unfinished assignment draft".into();
        session.practice.command = "unfinished practice draft".into();
        session.practice.editor_text = "separate unsaved editor text".into();
        for directory in ["/practice/in", "/practice/out", "/practice/archive"] {
            inspect(&mut session, definition, directory);
        }
        for cmd in &definition.example_commands {
            session.run(definition, cmd, true).unwrap();
        }
        assert!(
            definition.checks(&session, true).iter().all(|r| r.0),
            "{}",
            source.id
        );
        for directory in ["/practice/in", "/practice/out", "/practice/archive"] {
            inspect(&mut session, definition, directory);
        }
        assert!(!engine::solved(
            &State::Links {
                simulation: Box::new(session)
            },
            &source.goal
        ));
    }
}
#[test]
fn revised_move_experiment_is_dangling_only_during_move_and_restores_full_goal() {
    let source = labs()
        .into_iter()
        .find(|s| s.id == "lpic1-links-move")
        .unwrap();
    let Goal::Links { definition } = &source.goal else {
        panic!()
    };
    let mut session = work::Session::new(definition);
    let inode = session
        .practice
        .world
        .stat("/practice/in/receipt")
        .unwrap()
        .inode;
    for cmd in &definition.example_commands {
        session.run(definition, cmd, true).unwrap();
    }
    assert!(definition.checks(&session, true).iter().all(|r| r.0));
    session
        .run(
            definition,
            "mv /practice/archive/receipt /practice/in/receipt",
            true,
        )
        .unwrap();
    assert_eq!(
        session
            .practice
            .world
            .stat("/practice/in/receipt")
            .unwrap()
            .inode,
        inode
    );
    assert!(
        session
            .run(definition, "cat /practice/out/receipt", true)
            .is_err()
    );
    assert!(!definition.checks(&session, true).iter().all(|r| r.0));
    session
        .run(
            definition,
            "mv /practice/in/receipt /practice/archive/receipt",
            true,
        )
        .unwrap();
    assert_eq!(
        session
            .practice
            .world
            .stat("/practice/archive/receipt")
            .unwrap()
            .inode,
        inode
    );
    session
        .run(definition, "cat /practice/out/receipt", true)
        .unwrap();
    assert!(definition.checks(&session, true).iter().all(|r| r.0));
}
#[test]
fn readonly_listings_preserve_completed_credit_and_both_drafts_on_catalog_aware_resume() {
    let all = pack::ccna_pilot().unwrap();
    let source = all.iter().find(|s| s.id == "lpic1-links-identity").unwrap();
    let mut attempt = Attempt::new(source, 1);
    for cmd in &source.solution {
        engine::execute_for_goal(&mut attempt.state, &source.goal, cmd).unwrap();
    }
    assert!(engine::solved(&attempt.state, &source.goal));
    let (State::Links { simulation }, Goal::Links { definition }) =
        (&mut attempt.state, &source.goal)
    else {
        panic!()
    };
    simulation.assignment.command = "ls -i /delivery".into();
    simulation.practice.command = "ln /practice/in/brief ".into();
    let assignment_world = simulation.assignment.world.clone();
    simulation
        .run(definition, "ls -i /delivery", false)
        .unwrap();
    assert_eq!(simulation.assignment.world, assignment_world);
    assert!(engine::solved(&attempt.state, &source.goal));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(source.id.clone(), attempt);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
