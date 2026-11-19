use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace::{self as w, device_reference_session as packet},
    pack,
    progress::Attempt,
};
#[path = "support/lvm_device_reference_fixture.rs"]
mod fixture;
use fixture::{definition, save};
fn complete(d: &w::Definition, s: &w::Session) -> bool {
    d.checks(s).iter().all(|r| r.0)
}
#[test]
fn explicit_complete_separate_source_is_required_and_old_wire_is_unchanged() {
    let d = definition();
    d.validate().unwrap();
    for which in 0..5 {
        let mut bad = d.clone();
        match which {
            0 => bad.device_reference_task = None,
            1 => bad.lesson.device_reference_practice = None,
            2 => bad.lesson.fixture = d.initial.clone(),
            3 => bad.lesson.worked.push(w::Command {
                text: "lvs".into(),
                explanation: "Inspect".into(),
            }),
            _ => bad
                .lesson
                .device_reference_practice
                .as_mut()
                .unwrap()
                .worked[1]
                .explanation
                .clear(),
        };
        assert!(bad.shape().is_err());
    }
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let value = raw["scenarios"][0]["goal"]["definition"].clone();
    let old: w::Definition = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&old).unwrap(), value);
    let mut s = w::Session::new(&old).unwrap();
    assert!(s.reference_runtime.is_none());
    assert!(
        serde_json::to_value(&s)
            .unwrap()
            .get("reference_runtime")
            .is_none()
    );
    s.reference_runtime = w::device_reference_session::Runtime::new(&d).unwrap();
    assert!(old.bound(&s).is_err());
    assert!(!complete(&old, &s));
}
#[test]
fn separate_example_insert_actions_reset_and_draft_edits_do_not_touch_assignment() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    s.insert(&d, 0).unwrap();
    assert_eq!(s.current, before.current);
    assert_eq!(s.example, before.example);
    assert_eq!(s.reference_runtime, before.reference_runtime);
    let held = s.clone();
    assert!(s.insert(&d, 1).is_err());
    assert_eq!(s, held);
    for step in &d.lesson.device_reference_practice.as_ref().unwrap().worked {
        s.apply_reference(&d, &step.operation, true).unwrap();
    }
    assert!(d.practice_checks(&s).iter().all(|r| r.0));
    assert!(!complete(&d, &s));
    assert_eq!(
        s.reference_runtime.as_ref().unwrap().assignment,
        before.reference_runtime.as_ref().unwrap().assignment
    );
    s.checked = true;
    s.example_checked = true;
    s.edit_reference(&d, "reporting", "/dev/studio/missing".into(), true)
        .unwrap();
    assert!(s.checked && !s.example_checked);
    assert!(d.practice_checks(&s).iter().any(|r| !r.0));
    let assignment = s.reference_runtime.as_ref().unwrap().assignment.clone();
    s.reset_example(&d).unwrap();
    assert_eq!(s.reference_runtime.as_ref().unwrap().assignment, assignment);
    assert_eq!(s.example, d.lesson.fixture);
    assert_eq!(s.example_command, "");
    assert!(s.example_transcript.is_empty());
    assert!(!d.practice_checks(&s).iter().all(|r| r.0));
}
#[test]
fn pending_drafts_saved_reads_partial_resume_and_current_success_invalidation() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.edit_reference(&d, "reporting", "/dev/team/draft".into(), false)
        .unwrap();
    assert!(!complete(&d, &s));
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: w::Session = serde_json::from_slice(&bytes).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    restored.save_reference(&d, "reporting", false).unwrap();
    assert!(complete(&d, &restored));
    restored.checked = true;
    restored
        .edit_reference(&d, "reporting", "/dev/team/reference".into(), false)
        .unwrap();
    assert!(!restored.checked);
    assert!(!complete(&d, &restored));
    assert_eq!(
        restored
            .reference_runtime
            .as_ref()
            .unwrap()
            .assignment
            .configuration
            .callers["reporting"]
            .path,
        "/dev/team/draft"
    );
    restored.save_reference(&d, "reporting", false).unwrap();
    assert!(!complete(&d, &restored));
    assert!(
        d.checks(&restored)
            .iter()
            .any(|r| r.1.contains("another volume"))
    );
    restored
        .apply_reference(&d, &save("reporting", "/dev/team/draft"), false)
        .unwrap();
    assert!(complete(&d, &restored));
    restored
        .execute(&d, "lvrename team draft moved", false)
        .unwrap();
    assert!(!complete(&d, &restored));
}
#[test]
fn malformed_packets_oversize_edits_and_foreign_runtime_fail_atomically() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    for input in [
        format!("{}{{}}", packet::PREFIX),
        format!(
            "{}{{\"kind\":\"command\",\"text\":\"lvs\",\"extra\":true}}",
            packet::PREFIX
        ),
        "dmsetup splitname team-draft".into(),
        format!("{}{}", packet::PREFIX, "x".repeat(2048)),
    ] {
        let old = s.clone();
        assert!(s.execute(&d, &input, false).is_err());
        assert_eq!(s, old);
    }
    for text in ["x".repeat(257), "/dev/team/draft\n".into()] {
        let old = s.clone();
        assert!(s.edit_reference(&d, "reporting", text, false).is_err());
        assert_eq!(s, old);
    }
    let mut foreign = s.clone();
    foreign
        .reference_runtime
        .as_mut()
        .unwrap()
        .assignment
        .configuration
        .callers
        .get_mut("reporting")
        .unwrap()
        .file = "/other".into();
    assert!(d.bound(&foreign).is_err());
    assert!(!complete(&d, &foreign));
    let mut missing = s.clone();
    missing.reference_runtime = None;
    assert!(d.bound(&missing).is_err());
}
#[test]
fn authored_initial_cannot_smuggle_completed_configuration_and_goal_checks_actual_state() {
    let mut source = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-lvm-free-space")
        .unwrap();
    let d = definition();
    source.id = "test-only-device-reference".into();
    source.initial = State::Lvm {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    source.solution = d
        .device_reference_task
        .as_ref()
        .unwrap()
        .reference
        .iter()
        .map(|o| packet::encode(o).unwrap())
        .collect();
    source.goal = Goal::Lvm {
        definition: Box::new(d.clone()),
    };
    w::source(&source).unwrap();
    let mut a = Attempt::new(&source, 1);
    assert!(!engine::solved(&a.state, &source.goal));
    engine::execute_for_goal(&mut a.state, &source.goal, &source.solution[0]).unwrap();
    assert!(engine::solved(&a.state, &source.goal));
    w::context(&a, &source).unwrap();
    let bytes = serde_json::to_vec(&a).unwrap();
    let a: Attempt = serde_json::from_slice(&bytes).unwrap();
    w::context(&a, &source).unwrap();
    assert!(engine::solved(&a.state, &source.goal));
    let mut forged = source.clone();
    if let State::Lvm { simulation } = &mut forged.initial {
        simulation
            .apply_reference(&d, &save("reporting", "/dev/team/draft"), false)
            .unwrap();
        simulation.transcript.clear();
    }
    assert!(w::source(&forged).is_err());
    let reset = Attempt::new(&source, 2);
    assert!(!engine::solved(&reset.state, &source.goal));
    assert!(engine::solved(&a.state, &source.goal));
}
