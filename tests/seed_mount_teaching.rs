use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt},
    script_teaching::View,
    seed_mount_teaching::{self as mount, Action, Learning},
    workspace,
};
use serde_json::Value;
fn scenario() -> pack::Scenario {
    pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "lpic2-storage")
        .unwrap()
}
fn perform(l: &mut Learning, lesson: &mount::Lesson, actions: impl IntoIterator<Item = Action>) {
    for a in actions {
        match a {
            Action::Save { draft } => {
                l.draft = draft;
                l.save(lesson).unwrap();
            }
            Action::Command { command } => {
                l.command = command.command;
                l.submit(lesson).unwrap();
            }
        }
    }
}
fn reference(l: &mut Learning, lesson: &mount::Lesson) {
    perform(
        l,
        lesson,
        lesson.example_actions.iter().map(|a| a.action.clone()),
    );
}
fn assert_feedback_keeps_engine_truth(l: &Learning, lesson: &mount::Lesson) {
    let expected = engine::checks(&l.current.state().unwrap(), &lesson.goal().unwrap());
    let actual = l.checks(lesson);
    assert_eq!(
        actual.iter().map(|c| c.0).collect::<Vec<_>>(),
        expected.iter().map(|c| c.0).collect::<Vec<_>>()
    );
    for ((pass, message), (_, original)) in actual.iter().zip(expected) {
        assert!(!message.is_empty() && message.len() <= 512);
        if *pass {
            assert_eq!(message, &original);
        } else {
            assert_ne!(message, &original);
        }
    }
}
#[test]
fn mount_example_math_unmount_experiment_and_real_repair_use_original_current_goal() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    assert!(lesson.valid_for(&s.initial, &s.goal));
    let mut l = Learning::new(lesson).unwrap();
    let initial = l.checks(lesson);
    assert!(initial[0].1.contains("1850 of 2000 MiB, above the 80%"));
    assert!(
        initial[1]
            .1
            .contains("saved mount entry points to a different device")
    );
    assert!(initial[2].1.contains("mounted read-only"));
    assert_feedback_keeps_engine_truth(&l, lesson);
    reference(&mut l, lesson);
    assert_eq!(
        (l.current.capacity, l.current.used, l.current.cache),
        (2000, 1400, 0)
    );
    assert_eq!(
        (l.current.used as u128) * 100 / (l.current.capacity as u128),
        70
    );
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_feedback_keeps_engine_truth(&l, lesson);
    perform(&mut l, lesson, lesson.try_actions.clone());
    let checks = l.checks(lesson);
    assert_eq!(
        checks.iter().map(|c| c.0).collect::<Vec<_>>(),
        [true, true, false]
    );
    assert!(!l.current.mounted && !l.current.writable);
    assert!(checks[2].1.contains("not mounted"));
    assert!(!checks[2].1.contains("read-only"));
    assert_feedback_keeps_engine_truth(&l, lesson);
    assert_eq!(l.current.used, 1400);
    perform(&mut l, lesson, lesson.repair_actions.clone());
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert_eq!(scenario().initial, s.initial);
}
#[test]
fn mount_insert_only_bad_save_and_unsupported_commands_are_atomic() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let before = l.current.clone();
    l.selected_action = lesson
        .example_actions
        .iter()
        .position(|a| matches!(a.action, Action::Save { .. }))
        .unwrap();
    l.insert(lesson).unwrap();
    assert_eq!(l.current, before);
    assert!(l.transcript.is_empty());
    assert_eq!(l.draft.device, "/dev/vde2");
    for bad in ["", "not a device", "/dev/../vde2", "/dev/bad path"] {
        l.draft.device = bad.into();
        let draft = l.draft.clone();
        assert!(l.save(lesson).is_err());
        assert_eq!(l.current, before);
        assert_eq!(l.draft, draft);
    }
    for bad in [
        "rm -r /data/important",
        "mount /other",
        "sh -c mount",
        "rm -r /data/cache",
    ] {
        l.command = bad.into();
        assert!(l.submit(lesson).is_err());
        assert_eq!(l.current, before);
    }
}
#[test]
fn mount_alternate_current_method_needs_no_remount_or_cycle_history_badge() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    l.draft.device = "/dev/vde2".into();
    l.save(lesson).unwrap();
    for command in ["umount /data", "mount /data", "rm -r /data/cache"] {
        l.command = command.into();
        l.submit(lesson).unwrap();
    }
    assert!(l.checks(lesson).iter().all(|c| c.0));
    let good = l.current.clone();
    l.command = "rm -r /data/cache".into();
    l.submit(lesson).unwrap();
    assert_eq!(l.current, good);
    // Goal remains <=80%, not a hidden exact70% condition or removal-event check.
    let mut state = good.state().unwrap();
    let State::Mount { used, cache, .. } = &mut state else {
        panic!()
    };
    *used = 1600;
    *cache = 0;
    assert!(engine::solved(&state, &lesson.goal().unwrap()));
}
#[test]
fn mount_terminal_keeps_legacy_token_acceptance_and_refreshes_only_clean_drafts() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    l.command = "config set device wrong".into();
    l.submit(lesson).unwrap();
    assert_eq!(l.current.configured_device, "wrong");
    assert_eq!(l.draft.device, "wrong");
    assert!(
        l.checks(lesson)[1]
            .1
            .contains("saved mount entry points to a different device")
    );
    assert_feedback_keeps_engine_truth(&l, lesson);
    assert!(l.save(lesson).is_err());
    l.draft.device = "unfinished draft".into();
    l.command = "config set device /dev/vde2".into();
    l.submit(lesson).unwrap();
    assert_eq!(l.current.configured_device, "/dev/vde2");
    assert_eq!(l.draft.device, "unfinished draft");
    assert!(l.checks(lesson).iter().all(|c| !c.0));
    assert!(
        l.checks(lesson)[0]
            .1
            .starts_with("Unsaved example settings")
    );
    l.draft = mount::Draft::new(&l.current).unwrap();
    l.command = "mount /data".into();
    l.submit(lesson).unwrap();
    assert!(l.current.mounted && l.current.writable);
}
#[test]
fn mount_physical_context_pins_do_not_freeze_mutable_usage_pointer_or_flags() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    l.current.used = 950;
    l.current.cache = 300;
    l.current.configured_device = "/dev/vdb1".into();
    assert!(l.valid_for(lesson));
    let mut foreign = l.clone();
    foreign.current.capacity = 1000;
    assert!(!foreign.valid());
    assert!(foreign.submit(lesson).is_err());
    let mut foreign = l.clone();
    foreign.fixture.device = "/dev/other".into();
    assert!(!foreign.valid_for(lesson));
    let mut current = s.initial.clone();
    for command in [
        "config set device /dev/vdb1",
        "mount -o remount,rw /data",
        "rm -r /data/cache",
    ] {
        engine::execute(&mut current, command).unwrap();
        assert!(mount::assignment_context(&current, &s.initial));
    }
    let State::Mount { capacity, .. } = &mut current else {
        panic!()
    };
    *capacity = 2000;
    assert!(!mount::assignment_context(&current, &s.initial));
}
#[test]
fn mount_decode_bounds_prevent_foreign_state_and_roundtrip_worst_legal_escaping() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson).unwrap();
    let mut long_device = l.clone();
    long_device.current.configured_device = "é".repeat(128);
    long_device.draft = mount::Draft::new(&long_device.current).unwrap();
    assert!(long_device.valid_for(lesson));
    assert_feedback_keeps_engine_truth(&long_device, lesson);
    assert!(!long_device.checks(lesson)[1].1.contains('é'));
    let mut value = serde_json::to_value(&l).unwrap();
    value["current"] = serde_json::json!({"kind":"Script","source":"ignored","run":null});
    assert!(serde_json::from_value::<Learning>(value).is_err());
    let mut value = serde_json::to_value(&l).unwrap();
    value["draft"]["device"] = Value::String("x".repeat(257));
    assert!(serde_json::from_value::<Learning>(value).is_err());
    l.transcript = vec!["\u{0001}".repeat(8192); 16];
    l.command = "\\".repeat(4096);
    l.draft.device = "\\".repeat(256);
    assert!(l.valid_for(lesson));
    for bytes in [
        serde_json::to_vec(&l).unwrap(),
        serde_json::to_vec_pretty(&l).unwrap(),
    ] {
        assert!(bytes.len() < 1_048_576);
        assert_eq!(serde_json::from_slice::<Learning>(&bytes).unwrap(), l);
    }
    l.transcript.push("over aggregate".into());
    assert!(!l.valid());
    let mut huge = lesson.fixture.clone();
    huge.device = "x".repeat(257);
    assert!(huge.state().is_err());
    assert!(mount::Draft::new(&huge).is_err());
}
#[test]
fn mount_percentage_boundary_and_large_scalar_capacity_keep_existing_arithmetic() {
    let mut volume = scenario().seed_mount_teaching.unwrap().fixture;
    volume.configured_device = volume.device.clone();
    volume.mounted = true;
    volume.writable = true;
    volume.cache = 0;
    volume.capacity = u64::MAX;
    volume.used = ((volume.capacity as u128) * 80 / 100) as u64;
    assert!(engine::solved(
        &volume.state().unwrap(),
        &engine::Goal::Mount
    ));
    volume.used += 1;
    assert!(!engine::solved(
        &volume.state().unwrap(),
        &engine::Goal::Mount
    ));
    volume.capacity = 2000;
    volume.used = 1600;
    assert!(engine::solved(
        &volume.state().unwrap(),
        &engine::Goal::Mount
    ));
    volume.used = 1601;
    assert!(!engine::solved(
        &volume.state().unwrap(),
        &engine::Goal::Mount
    ));
}
#[test]
fn mount_upgrade_keeps_original_raw_device_and_assignment_while_new_example_is_independent() {
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut a = Attempt::new(&s, 5);
    workspace::prepare(&mut a);
    a.workspace.as_mut().unwrap().mount_device = Some("unfinished device".into());
    a.workspace.as_mut().unwrap().command = "mount unfinished".into();
    let before = serde_json::to_value(&a).unwrap();
    mount::prepare(&mut a, lesson, false).unwrap();
    let mut after = serde_json::to_value(&a).unwrap();
    after["workspace"]
        .as_object_mut()
        .unwrap()
        .remove("seed_mount_learning");
    assert_eq!(after, before);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .seed_mount_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let mut l = Learning::new(lesson).unwrap();
    reference(&mut l, lesson);
    assert_eq!(a.state, s.initial);
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn wrong_kind_and_foreign_fixture_cannot_replace_existing_saved_profile_bytes() {
    let all = pack::bundled().unwrap();
    let s = scenario();
    let lesson = s.seed_mount_teaching.as_ref().unwrap();
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(&s, 1);
    workspace::prepare(&mut a);
    mount::prepare(&mut a, lesson, true).unwrap();
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save(&path, &p).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut value = serde_json::to_value(&p).unwrap();
    value["attempts"][&s.id]["workspace"]["seed_mount_learning"]["fixture"]["capacity"] =
        serde_json::json!(3000);
    assert!(progress::decode(&serde_json::to_vec(&value).unwrap(), &all).is_err());
    let bad = p
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .seed_mount_learning
        .as_mut()
        .unwrap();
    bad.current.device = "/dev/foreign".into();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut wrong = progress::load(&path, &all).unwrap();
    wrong.attempts.get_mut(&s.id).unwrap().state = all
        .iter()
        .find(|s| s.id == "lpic2-dns")
        .unwrap()
        .initial
        .clone();
    assert!(progress::save(&path, &wrong).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
