//! Independent practice state and bounded raw sessions, not assignment credentials.
use dlms_learning_lab::{
    engine, pack,
    progress::{self, Attempt},
    rpm_packages_teaching as teaching, rpm_packages_terminal as terminal, workspace,
};
fn definition(s: &pack::Scenario) -> &dlms_learning_lab::rpm_packages_workspace::Definition {
    let engine::Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/rpm-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
#[test]
fn intro_insert_only_run_experiment_repair_are_real_and_separate() {
    let s = scenario();
    let source = s.rpm_packages_teaching.as_ref().unwrap();
    source.validate().unwrap();
    let mut a = Attempt::new(&s, 1);
    let initial = serde_json::to_value(&a.state).unwrap();
    teaching::prepare(&mut a, definition(&s), source, true).unwrap();
    let l = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
    assert_eq!(
        l.view,
        dlms_learning_lab::script_teaching::View::Introduction
    );
    l.view = dlms_learning_lab::script_teaching::View::Example;
    let state = l.simulation.clone();
    l.insert(source, &source.example_commands[0].command)
        .unwrap();
    assert_eq!(l.simulation, state);
    for c in &source.example_commands {
        l.insert(source, &c.command).unwrap();
        assert!(matches!(
            l.run(source).unwrap(),
            terminal::Outcome::Utility { status: 0, .. }
        ));
    }
    assert!(l.checks(source).iter().all(|r| r.0));
    for c in &source.try_change.commands {
        l.insert(source, &c.command).unwrap();
        l.run(source).unwrap();
    }
    assert_eq!(
        l.checks(source).iter().map(|r| r.0).collect::<Vec<_>>(),
        source.try_change.expected
    );
    for c in &source.repair {
        l.insert(source, &c.command).unwrap();
        l.run(source).unwrap();
    }
    assert!(l.checks(source).iter().all(|r| r.0));
    assert_eq!(serde_json::to_value(&a.state).unwrap(), initial);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn old_attempt_first_open_assignment_and_wrong_kind_prepare_are_atomic() {
    let s = scenario();
    let source = s.rpm_packages_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 1);
    teaching::prepare(&mut a, definition(&s), source, false).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .rpm_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Assignment
    );
    let all = pack::bundled().unwrap();
    let old = all.iter().find(|s| s.id == "ccna-gateway").unwrap();
    let mut a = Attempt::new(old, 1);
    let before = serde_json::to_value(&a).unwrap();
    assert!(teaching::prepare(&mut a, definition(&s), source, true).is_err());
    assert!(serde_json::to_value(&a).unwrap() == before);
}
#[test]
fn invalid_existing_learning_or_overlarge_lesson_never_replaces_previous_work() {
    let s = scenario();
    let source = s.rpm_packages_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 1);
    teaching::prepare(&mut a, definition(&s), source, true).unwrap();
    a.workspace
        .as_mut()
        .unwrap()
        .rpm_learning
        .as_mut()
        .unwrap()
        .context
        .archive_paths
        .remove("/museum/rpms/panel-a.rpm");
    let before = serde_json::to_value(&a).unwrap();
    assert!(teaching::prepare(&mut a, definition(&s), source, true).is_err());
    assert!(serde_json::to_value(&a).unwrap() == before);
    let mut l = source.clone();
    l.example_commands[0].command = "x".repeat(100_000);
    assert!(!l.bounded());
    let mut a = Attempt::new(&s, 1);
    assert!(teaching::prepare(&mut a, definition(&s), &l, true).is_err());
    assert!(a.workspace.is_none());
}
#[test]
fn valid_alien_authored_context_fails_current_and_catalog_aware_decode() {
    let s = scenario();
    let source = s.rpm_packages_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    teaching::prepare(&mut a, definition(&s), source, true).unwrap();
    let l = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
    l.context.archive_paths.remove("/museum/rpms/panel-a.rpm");
    l.context
        .archive_paths
        .insert("/museum/rpms/renamed.rpm".into(), "panel-a".into());
    assert!(l.valid());
    assert!(!l.valid_for(source));
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(!progress::current_solved(&a, &s));
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &[s]).is_err());
}

#[test]
fn a_shared_archive_path_is_rejected_even_with_a_valid_independent_worked_example() {
    let s = scenario();
    let mut lesson = s.rpm_packages_teaching.as_ref().unwrap().clone();
    lesson
        .definition
        .archive_paths
        .remove("/museum/rpms/panel-a.rpm");
    lesson
        .definition
        .archive_paths
        .insert("/pkgs/client-a.rpm".into(), "panel-a".into());
    for step in lesson
        .example_commands
        .iter_mut()
        .chain(&mut lesson.try_change.commands)
        .chain(&mut lesson.repair)
    {
        step.command = step
            .command
            .replace("/museum/rpms/panel-a.rpm", "/pkgs/client-a.rpm");
    }
    lesson.validate().unwrap();
    assert!(!lesson.separate_from_definition(definition(&s)));
    let mut a = Attempt::new(&s, 1);
    let before = serde_json::to_value(&a).unwrap();
    assert!(teaching::prepare(&mut a, definition(&s), &lesson, true).is_err());
    assert!(serde_json::to_value(&a).unwrap() == before);
}
