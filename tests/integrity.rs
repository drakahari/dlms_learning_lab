use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching, shell, workspace,
};
#[test]
fn long_unknown_commands_remain_persistable_in_assignments_and_examples() {
    let labs = pack::bundled().unwrap();
    let s = &labs[9];
    let lesson = s.teaching.as_ref().unwrap();
    for source in [
        "x".repeat(2000),
        "界".repeat(2000),
        "x".repeat(shell::MAX_SOURCE),
    ] {
        let r = shell::run(&source, &lesson.fixture);
        assert!(r.error.as_ref().unwrap().len() <= 1000);
        let record = shell::Record {
            source: source.clone(),
            results: vec![r.clone()],
        };
        assert!(record.valid());
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, lesson, true);
        a.state = State::Script {
            source: source.clone(),
            run: Some(record),
        };
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source = source;
        l.example_result = Some(r);
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save(&path, &p).unwrap();
        assert!(progress::load(&path, &labs).is_ok());
    }
}
#[test]
fn alex_privacy_checks_every_modeled_user_but_group_traversal_still_works() {
    let labs = pack::bundled().unwrap();
    let s = &labs[3];
    let mut state = s.initial.clone();
    engine::execute(&mut state, "chmod 660 /srv/report.txt").unwrap();
    assert!(engine::solved(&state, &s.goal));
    engine::execute(&mut state, "usermod -aG editors blair").unwrap();
    assert!(!engine::solved(&state, &s.goal));
    engine::execute(&mut state, "chown alex:editors /srv/report.txt").unwrap();
    engine::execute(&mut state, "chmod 600 /srv/report.txt").unwrap();
    assert!(engine::solved(&state, &s.goal));
    let s = &labs[4];
    let mut state = s.initial.clone();
    for command in &s.solution {
        engine::execute(&mut state, command).unwrap();
    }
    assert!(engine::solved(&state, &s.goal));
}
#[test]
fn unsupported_grep_options_and_integer_tests_fail_honestly() {
    let labs = pack::bundled().unwrap();
    let c = &labs[9].teaching.as_ref().unwrap().fixture;
    for source in [
        "printf '%s\\n' '-bb' | grep -F '-b'",
        "[ bad -eq 2 ]; printf 'after\\n'",
        "cat -n",
        "cat -",
        "grep -F ERR -v",
        "grep -F ERR -",
    ] {
        let r = shell::run(source, c);
        assert!(r.error.as_ref().unwrap().contains("Unsupported"));
    }
}
#[test]
fn prior_automatically_supplied_host_requires_review_without_deleting_data() {
    let labs = pack::bundled().unwrap();
    let s = &labs[0];
    let mut a = Attempt::new(s, 1);
    for command in &s.solution {
        engine::execute(&mut a.state, command).unwrap();
    }
    workspace::prepare(&mut a);
    let w = a.workspace.as_mut().unwrap();
    w.migrated_host = true;
    w.ipv4_check = Some(dlms_learning_lab::workspace::snapshot(
        &dlms_learning_lab::ipv4::Exercise::from_state(&a.state).unwrap(),
    ));
    let before = a.state.clone();
    workspace::prepare(&mut a);
    assert_eq!(a.state, before);
    assert!(a.workspace.as_ref().unwrap().ipv4_check.is_none());
    assert!(progress::legacy_credit(&a));
    assert!(!progress::current_solved(&a, s));
    a.workspace.as_mut().unwrap().legacy_host_reviewed = true;
    assert!(progress::current_solved(&a, s));
}
