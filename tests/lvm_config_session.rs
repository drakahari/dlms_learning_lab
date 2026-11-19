//! Optional native configuration profile; synthetic sources remain unregistered.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace::{self as w, config_scan as f},
    pack::{self, Pack},
    progress::{self, Attempt},
};
#[path = "support/lvm_config_scan_fixture.rs"]
mod fixture;
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn complete_typed_source_and_reachable_reference_are_required_but_old_wire_is_exact() {
    for file in [false, true] {
        let d = fixture::definition(file);
        d.validate().unwrap();
        for change in 0..8 {
            let mut bad = d.clone();
            match change {
                0 => bad.config_scan_task = None,
                1 => bad.lesson.config_scan_practice = None,
                2 => bad.initial = bad.lesson.fixture.clone(),
                3 => bad.lesson.concepts.clear(),
                4 => {
                    bad.plan = Some(
                        serde_json::from_value(serde_json::json!({"groups":[],"volumes":[]}))
                            .unwrap(),
                    )
                }
                5 => bad.lesson.worked.push(w::Command {
                    text: "pvs".into(),
                    explanation: "Unrelated terminal work".into(),
                }),
                6 => bad.lesson.fixture = d.initial.clone(),
                _ => {
                    bad.config_scan_task.as_mut().unwrap().context.cases[0]
                        .inventory
                        .provisioning = Some(dlms_learning_lab::lvm::provision::Profile::Lifecycle)
                }
            }
            assert!(bad.shape().is_err(), "change{change}");
        }
    }
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let value = raw["scenarios"][0]["goal"]["definition"].clone();
    let old: w::Definition = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&old).unwrap(), value);
    let mut old_session = w::Session::new(&old).unwrap();
    assert!(
        serde_json::to_value(&old_session)
            .unwrap()
            .get("config_scan_runtime")
            .is_none()
    );
    old_session.config_scan_runtime = w::Session::new(&fixture::definition(false))
        .unwrap()
        .config_scan_runtime;
    assert!(old.bound(&old_session).is_err());
    assert!(!engine::solved(
        &State::Lvm {
            simulation: Box::new(old_session)
        },
        &Goal::Lvm {
            definition: Box::new(old)
        }
    ));
}
#[test]
fn authored_native_references_run_through_real_pack_validation_not_shell_packets() {
    for file in [false, true] {
        let s = fixture::scenario(file);
        w::source(&s).unwrap();
        definition(&s).validate().unwrap();
        pack::load(
            &serde_json::to_string(&Pack {
                schema_version: 1,
                scenarios: vec![s.clone()],
            })
            .unwrap(),
        )
        .unwrap();
        let d = definition(&s);
        let mut a = Attempt::new(&s, 1);
        assert!(!progress::current_solved(&a, &s));
        for op in &d.config_scan_task.as_ref().unwrap().reference {
            session(&mut a).config_action(d, op, false).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        w::context(&a, &s).unwrap();
        for row in &session(&mut a).transcript {
            assert!(!row.input.contains('{'));
            assert!(!row.input.contains("virtual $"));
        }
        let bytes = serde_json::to_vec(&a).unwrap();
        let restored: Attempt = serde_json::from_slice(&bytes).unwrap();
        w::context(&restored, &s).unwrap();
        assert!(progress::current_solved(&restored, &s));
        let mut wrong = s.clone();
        wrong.solution.push("invented source action".into());
        assert!(w::source(&wrong).is_err());
        let mut smuggled = s.clone();
        smuggled.initial = a.state.clone();
        assert!(w::source(&smuggled).is_err());
    }
}
#[test]
fn separate_guidance_and_both_unfinished_editor_drafts_roundtrip_and_selected_reset() {
    let s = fixture::scenario(false);
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    let before = session(&mut a)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .clone();
    for step in &d.lesson.config_scan_practice.as_ref().unwrap().worked {
        session(&mut a)
            .config_action(d, &step.operation, true)
            .unwrap();
    }
    assert_eq!(
        session(&mut a)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment,
        before
    );
    assert!(!progress::current_solved(&a, &s));
    session(&mut a)
        .config_edit(d, "# unfinished assignment\ndevices {".into(), false)
        .unwrap();
    session(&mut a)
        .config_edit(d, "# unfinished example\ndevices {".into(), true)
        .unwrap();
    let original = session(&mut a).clone();
    let bytes = serde_json::to_vec(&a).unwrap();
    let mut a: Attempt = serde_json::from_slice(&bytes).unwrap();
    w::context(&a, &s).unwrap();
    assert_eq!(session(&mut a), &original);
    session(&mut a).reset_example(d).unwrap();
    assert_eq!(
        session(&mut a)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment,
        original.config_scan_runtime.as_ref().unwrap().assignment
    );
    let reset = Attempt::new(&s, 2);
    assert!(!progress::current_solved(&reset, &s));
    assert_eq!(
        session(&mut a)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .draft
            .0,
        "# unfinished assignment\ndevices {"
    );
}
#[test]
fn bad_saved_configuration_retains_a_real_diagnostic_and_repair_clears_stale_feedback() {
    let s = fixture::scenario(false);
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    for op in fixture::good(false, false) {
        session(&mut a).config_action(d, &op, false).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    session(&mut a).checked = true;
    session(&mut a)
        .config_edit(d, "devices {\n filter = [\"a|.*|\"]\n}".into(), false)
        .unwrap();
    assert!(!session(&mut a).checked);
    assert!(!progress::current_solved(&a, &s));
    let text = session(&mut a)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .draft
        .clone();
    session(&mut a)
        .config_action(d, &f::Operation::SaveConfig { text }, false)
        .unwrap();
    let e = session(&mut a)
        .config_action(d, &f::Operation::Scan, false)
        .unwrap_err();
    assert!(e.starts_with("Unsupported:"));
    assert!(
        session(&mut a)
            .transcript
            .last()
            .unwrap()
            .output
            .contains("explicit use_devicesfile")
    );
    w::context(&a, &s).unwrap();
    assert!(!progress::current_solved(&a, &s));
    for op in fixture::good(false, false) {
        session(&mut a).config_action(d, &op, false).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    session(&mut a)
        .config_action(
            d,
            &f::Operation::SaveConfig {
                text: f::Text(fixture::config(false, &["a|.*|"])),
            },
            false,
        )
        .unwrap();
    session(&mut a)
        .config_action(d, &f::Operation::Scan, false)
        .unwrap();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn config_profile_cannot_execute_terminal_commands_or_mutate_virtual_storage() {
    let s = fixture::scenario(false);
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    for text in [
        "pvs",
        "pvcreate /dev/recovery",
        "lvmdevices --adddev /dev/appdata",
        "simulated-lvm-reference {}",
        "sh -c touch",
    ] {
        let before = a.state.clone();
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, text).is_err());
        assert_eq!(a.state, before);
    }
    let before = session(&mut a).clone();
    assert!(session(&mut a).set_command(d, "pvs".into(), false).is_err());
    assert_eq!(session(&mut a), &before);
    for op in fixture::good(false, false) {
        session(&mut a).config_action(d, &op, false).unwrap();
    }
    assert_eq!(session(&mut a).current, d.initial);
    assert_eq!(session(&mut a).example, d.lesson.fixture);
    let mut forged = session(&mut a).clone();
    forged.current.devices.values_mut().next().unwrap().bytes += 512;
    assert!(d.bound(&forged).is_err());
    assert!(d.checks(&forged).iter().all(|r| !r.0));
}
#[test]
fn source_bound_roster_limits_reject_foreign_or_oversize_edits_atomically() {
    let s = fixture::scenario(true);
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    let before = session(&mut a).clone();
    for text in ["x".repeat(4097), "bad\0text".into()] {
        assert!(session(&mut a).config_edit(d, text, false).is_err());
        assert_eq!(session(&mut a), &before);
    }
    for paths in [
        &["/dev/not_supplied"][..],
        &["/dev/../../home/private"][..],
        &["/dev/appdata", "/dev/appdata"][..],
    ] {
        assert!(
            session(&mut a)
                .config_selection_edit(d, fixture::selection(paths), false)
                .is_err()
        );
        assert_eq!(session(&mut a), &before);
    }
    let mut missing = before.clone();
    missing.config_scan_runtime = None;
    assert!(d.bound(&missing).is_err());
    let mut wrong = before.clone();
    wrong.config_scan_runtime.as_mut().unwrap().version = 2;
    assert!(wrong.validate().is_err());
}
