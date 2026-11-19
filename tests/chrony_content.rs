use dlms_learning_lab::{
    chrony_workspace::{self as work, Draft, Session},
    engine::{self, Goal, State},
    pack, progress,
};

const ID: &str = "lpic1-chrony-source-diagnosis";

fn scenario() -> pack::Scenario {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v9.json")).unwrap();
    serde_json::from_value(
        pack["scenarios"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone(),
    )
    .unwrap()
}

fn definition(scenario: &pack::Scenario) -> &work::Definition {
    let Goal::Chrony { definition } = &scenario.goal else {
        panic!()
    };
    definition
}

#[test]
fn practice_uses_separate_observations_and_can_teach_the_old_source_failure() {
    let scenario = scenario();
    let definition = work::source(&scenario).unwrap();
    let mut session = Session::new(definition);
    session
        .practice
        .set_command("chronyc sources".into())
        .unwrap();
    session.practice.run(&definition.example).unwrap();
    assert!(
        session
            .practice
            .output(&definition.example)
            .unwrap()
            .unwrap()
            .contains("^? 198.51.100.8")
    );
    session
        .practice
        .set_command("chronyc -n tracking".into())
        .unwrap();
    session.practice.run(&definition.example).unwrap();
    assert!(
        session
            .practice
            .output(&definition.example)
            .unwrap()
            .unwrap()
            .contains("Not synchronised")
    );
    session
        .practice
        .edit(definition.example.reference.clone())
        .unwrap();
    assert!(session.practice.test(&definition.example).is_err());
    session.practice.apply(&definition.example).unwrap();
    assert!(
        session
            .practice
            .output(&definition.example)
            .unwrap()
            .is_none()
    );
    session.practice.test(&definition.example).unwrap();
    assert!(
        session
            .practice
            .checks(&definition.example)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(session.assignment, Draft::new(&definition.initial));
    assert!(!definition.checks(&session).iter().all(|r| r.0));
}

#[test]
fn alternate_source_order_and_whitespace_pass_and_incomplete_changes_fail() {
    let scenario = scenario();
    let definition = definition(&scenario);
    let mut draft = Draft::new(&definition.initial);
    draft
        .edit("server 198.51.100.22\n server\t192.0.2.44\n".into())
        .unwrap();
    draft.apply(&definition.initial).unwrap();
    draft.test(&definition.initial).unwrap();
    assert!(draft.checks(&definition.initial).iter().all(|r| r.0));
    let last_applied = draft.applied.clone();

    draft.edit("server 192.0.2.44\n".into()).unwrap();
    assert!(draft.checked.is_none());
    draft.apply(&definition.initial).unwrap();
    draft.test(&definition.initial).unwrap();
    assert!(
        draft
            .checks(&definition.initial)
            .iter()
            .any(|(pass, text)| !pass && text.contains("backup"))
    );
    assert_ne!(draft.applied, last_applied);

    let before = draft.applied.clone();
    draft.edit("include /etc/chrony.conf\n".into()).unwrap();
    assert!(draft.apply(&definition.initial).is_err());
    assert_eq!(draft.applied, before);
    draft
        .set_command("chronyc tracking && date -s 0".into())
        .unwrap();
    assert!(draft.run(&definition.initial).is_err());
    assert!(draft.checked.is_none());
}

#[test]
fn actual_progress_roundtrip_preserves_separate_drafts_and_clears_stale_grade() {
    let scenario = scenario();
    let definition = definition(&scenario);
    let mut attempt = progress::Attempt::new(&scenario, 1);
    engine::execute_for_goal(&mut attempt.state, &scenario.goal, &scenario.solution[0]).unwrap();
    let State::Chrony { simulation } = &mut attempt.state else {
        panic!()
    };
    simulation.practice.edit("server pending".into()).unwrap();
    assert_eq!(simulation.practice.applied, definition.example.initial);
    assert!(progress::current_solved(&attempt, &scenario));
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    profile.attempts.insert(ID.into(), attempt.clone());
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("progress.json");
    progress::save_with_catalog(&file, &profile, std::slice::from_ref(&scenario)).unwrap();
    let mut restored = progress::load(&file, std::slice::from_ref(&scenario)).unwrap();
    let saved = restored.attempts.get_mut(ID).unwrap();
    assert_eq!(
        serde_json::to_value(&*saved).unwrap(),
        serde_json::to_value(&attempt).unwrap()
    );
    let State::Chrony { simulation } = &mut saved.state else {
        panic!()
    };
    simulation
        .assignment
        .edit(definition.initial.initial.clone())
        .unwrap();
    assert!(!progress::current_solved(saved, &scenario));
}

#[test]
fn time_source_unit_preserves_every_preceding_scenario_and_unit() {
    use dlms_learning_lab::{course, pack};
    let old = pack::accepted385().unwrap();
    let now = pack::accepted386().unwrap();
    assert_eq!((old.len(), now.len()), (385, 386));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..385]).unwrap()
    );
    let old_units = course::accepted385(&old).unwrap();
    let units = course::accepted386(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (146, 147));
    let unit = units
        .iter()
        .find(|unit| unit.id == "linux-clock-source-diagnosis")
        .unwrap();
    assert_eq!(unit.labs, [ID]);
    assert_eq!(
        unit.prerequisites,
        ["linux-host-networking", "linux-startup-policy"]
    );
    let retained = units
        .iter()
        .filter(|unit| unit.id != "linux-clock-source-diagnosis")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}
