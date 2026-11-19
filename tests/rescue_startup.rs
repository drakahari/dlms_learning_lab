use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack, progress,
    unit_manager::{self as core, Runtime},
    unit_manager_workspace::{self as work, Definition, Draft, EditAction},
};

fn scenario() -> pack::Scenario {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v8.json")).unwrap();
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

fn definition(scenario: &pack::Scenario) -> &Definition {
    let Goal::UnitManager { definition } = &scenario.goal else {
        panic!()
    };
    definition
}

#[test]
fn failed_ordinary_graph_and_recovery_are_independent_and_default_is_unchanged() {
    let source = scenario();
    let definition = definition(&source);
    let saved = &definition.initial;
    let (ordinary, report) = saved.fresh_session().unwrap();
    assert_eq!(report.requested.as_deref(), Some("multi-user.target"));
    assert_eq!(report.requested_succeeded(), Some(false));
    assert!(report.jobs.iter().any(|job| {
        job.unit == "ridge-verify.service" && matches!(job.result, core::JobResult::Failed { .. })
    }));
    assert!(
        !ordinary
            .runtime()
            .get("multi-user.target")
            .is_some_and(Runtime::active)
    );
    let (rescue, report, ordering) = saved.fresh_session_at("rescue.target").unwrap();
    assert_eq!(report.requested_succeeded(), Some(true));
    assert_eq!(rescue.default_target(), saved.default_target());
    assert_eq!(rescue.configuration(), saved.configuration());
    assert!(ordering.contains("ridge-rescue-access.service"));
    assert!(!ordering.contains("ridge-app.service"));
    assert!(rescue.runtime()["rescue.target"].active());
    assert!(saved.runtime().is_empty());
    assert!(saved.fresh_session_at("ridge-app.service").is_err());
    assert!(saved.fresh_session_at("emergency.target").is_err());
    assert!(saved.fresh_session_at("../../host").is_err());
}

#[test]
fn correct_repair_alternatives_pass_and_valid_but_wrong_data_fail() {
    let source = scenario();
    let definition = definition(&source);
    let mut state = definition.initial.clone();
    work::apply(
        &mut state,
        &definition.initial,
        &EditAction::Startup {
            target: Some("rescue.target".into()),
        },
    )
    .unwrap();
    assert!(!work::solved(
        definition,
        &state,
        &Draft::new(&state).unwrap()
    ));
    let mut draft = Draft::new(&state).unwrap();
    let path = "/etc/ridge/startup.conf";
    draft
        .configuration
        .insert(path.into(), "# Local handover\n MODE = ready-east\n".into());
    assert!(!work::solved(definition, &state, &draft));
    draft.apply(&mut state).unwrap();
    assert!(work::solved(definition, &state, &draft));
    let (verified, start, _, _) = work::inspect_fresh(&state).unwrap();
    assert_eq!(start.requested_succeeded(), Some(true));
    assert_eq!(
        verified.result("ridge-app.service").unwrap().unwrap().port,
        Some(9143)
    );
    assert!(state.runtime()["rescue.target"].active());
    assert_eq!(state.default_target(), "multi-user.target");

    draft
        .configuration
        .insert(path.into(), "MODE=old-east\n".into());
    draft.apply(&mut state).unwrap();
    assert!(!work::solved(definition, &state, &draft));
    let feedback = work::checks(definition, &state, &draft);
    assert!(feedback.iter().any(|(pass, text)| {
        !pass
            && text.contains("Ordinary startup")
            && text.contains("old-east")
            && text.contains("ready-east")
    }));
    let (fresh, _, _, _) = work::inspect_fresh(&state).unwrap();
    assert!(!fresh.runtime()["ridge-app.service"].active());
}

#[test]
fn authored_startup_actions_grade_and_saved_state_reloads() {
    let scenario = scenario();
    let lesson = scenario.unit_manager_teaching.as_ref().unwrap();
    lesson.validate().unwrap();
    let definition = definition(&scenario);
    let mut attempt = progress::Attempt::new(&scenario, 1);
    for action in &scenario.solution {
        engine::execute_for_goal(&mut attempt.state, &scenario.goal, action).unwrap();
    }
    let State::UnitManager { simulation } = &attempt.state else {
        panic!()
    };
    let draft = Draft::new(simulation).unwrap();
    assert!(work::solved(definition, simulation, &draft));
    dlms_learning_lab::workspace::prepare(&mut attempt);
    dlms_learning_lab::unit_manager_teaching::prepare(&mut attempt, lesson, true).unwrap();
    attempt.workspace.as_mut().unwrap().checked = true;
    assert!(progress::current_solved(&attempt, &scenario));
    let catalog = std::slice::from_ref(&scenario);
    let mut profile = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    profile
        .attempts
        .insert(scenario.id.clone(), attempt.clone());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress-v1.json");
    progress::save_with_catalog(&path, &profile, catalog).unwrap();
    let reopened = progress::load(&path, catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened.attempts[&scenario.id]).unwrap(),
        serde_json::to_value(attempt).unwrap()
    );
}

#[test]
fn new_recovery_unit_retains_every_preceding_scenario_and_course_value() {
    use dlms_learning_lab::{course, pack};
    let before = pack::accepted384().unwrap();
    let current = pack::accepted385().unwrap();
    assert_eq!((before.len(), current.len()), (384, 385));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&current[..384]).unwrap()
    );
    let old = course::accepted384(&before).unwrap();
    let new = course::accepted385(&current).unwrap();
    assert_eq!((old.len(), new.len()), (145, 146));
    let position = new
        .iter()
        .position(|unit| unit.id == "linux-rescue-target-repair")
        .unwrap();
    assert_eq!(new[position - 1].id, "linux-temporary-grub-recovery");
    assert_eq!(new[position].labs, ["lpic2-rescue-startup-repair"]);
    let retained = new
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != position)
        .map(|(_, unit)| unit)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
}
