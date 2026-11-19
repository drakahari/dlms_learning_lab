//! Source/Goal/session admission for synthetic, unregistered mount tasks.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace as w,
    pack::{self, Pack},
    progress::{self, Attempt},
    script_teaching::View,
};
#[path = "support/lvm_mount_session_fixture.rs"]
mod source_fixture;
use source_fixture::{definition, model as fixture, scenario};

fn d(s: &pack::Scenario) -> &w::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn old_wire_remains_exact_and_legacy_progress_cannot_import_mount_credit() {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let raw = p["scenarios"][0]["goal"]["definition"].clone();
    let old: w::Definition = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(serde_json::to_value(&old).unwrap(), raw);
    let mut s = w::Session::new(&old).unwrap();
    assert!(
        serde_json::to_value(&s)
            .unwrap()
            .get("mount_runtime")
            .is_none()
    );
    s.mount_runtime = w::Session::new(&definition()).unwrap().mount_runtime;
    assert!(old.bound(&s).is_err());
    assert!(!engine::solved(
        &State::Lvm {
            simulation: Box::new(s)
        },
        &Goal::Lvm {
            definition: Box::new(old)
        }
    ));
}
#[test]
fn complete_source_and_reachable_native_reference_are_admitted_without_shell_packets() {
    let source = scenario();
    let def = d(&source);
    def.validate().unwrap();
    w::source(&source).unwrap();
    pack::load(
        &serde_json::to_string(&Pack {
            schema_version: 1,
            scenarios: vec![source.clone()],
        })
        .unwrap(),
    )
    .unwrap();
    let mut a = Attempt::new(&source, 1);
    assert!(!progress::current_solved(&a, &source));
    for op in &def.mount_task.as_ref().unwrap().reference {
        sim(&mut a).mount_action(def, op, false).unwrap();
    }
    w::context(&a, &source).unwrap();
    assert!(progress::current_solved(&a, &source));
    assert!(
        sim(&mut a)
            .transcript
            .iter()
            .all(|r| !r.input.contains('{'))
    );
    let mut bad = source.clone();
    bad.solution.push("invented action".into());
    assert!(w::source(&bad).is_err());
    bad = source.clone();
    bad.initial = a.state;
    assert!(w::source(&bad).is_err());
}
#[test]
fn practice_actions_and_command_insertion_never_change_or_credit_assignment() {
    let source = scenario();
    let def = d(&source);
    let mut a = Attempt::new(&source, 1);
    let before = sim(&mut a)
        .mount_runtime
        .as_ref()
        .unwrap()
        .assignment
        .clone();
    let storage = sim(&mut a).current.clone();
    sim(&mut a).view = View::Example;
    assert!(sim(&mut a).insert(def, 0).is_err());
    for step in &def.lesson.mount_practice.as_ref().unwrap().worked {
        sim(&mut a)
            .mount_action(def, &step.operation, true)
            .unwrap();
    }
    assert!(def.practice_checks(sim(&mut a)).iter().all(|r| r.0));
    assert_eq!(sim(&mut a).current, storage);
    assert_eq!(
        sim(&mut a).mount_runtime.as_ref().unwrap().assignment,
        before
    );
    assert!(!progress::current_solved(&a, &source));
    let old = sim(&mut a).clone();
    sim(&mut a).insert(def, 2).unwrap();
    assert_eq!(sim(&mut a).example, old.example);
    assert_eq!(sim(&mut a).mount_runtime, old.mount_runtime);
    assert_eq!(sim(&mut a).example_command, "mount /srv/practice");
}
#[test]
fn unfinished_both_drafts_resume_and_reset_affects_only_selected_practice() {
    let source = scenario();
    let def = d(&source);
    let mut a = Attempt::new(&source, 1);
    sim(&mut a)
        .edit_mount(def, "# unfinished assignment\nUUID=".into(), false)
        .unwrap();
    sim(&mut a)
        .edit_mount(def, "# unfinished practice\nUUID=".into(), true)
        .unwrap();
    let before = sim(&mut a).clone();
    let bytes = serde_json::to_vec(&a).unwrap();
    let mut a: Attempt = serde_json::from_slice(&bytes).unwrap();
    w::context(&a, &source).unwrap();
    assert_eq!(sim(&mut a), &before);
    sim(&mut a).reset_example(def).unwrap();
    assert_eq!(
        sim(&mut a).mount_runtime.as_ref().unwrap().assignment,
        before.mount_runtime.as_ref().unwrap().assignment
    );
    assert_eq!(
        sim(&mut a).mount_runtime.as_ref().unwrap().example,
        w::Session::new(def).unwrap().mount_runtime.unwrap().example
    );
    let mut reset = Attempt::new(&source, 2);
    assert_eq!(
        sim(&mut reset).mount_runtime,
        w::Session::new(def).unwrap().mount_runtime
    );
    assert!(!progress::current_solved(&reset, &source));
}
#[test]
fn current_success_clears_after_edit_save_wrong_mount_or_real_unmount() {
    let source = scenario();
    let def = d(&source);
    let mut a = Attempt::new(&source, 1);
    for op in &def.mount_task.as_ref().unwrap().reference {
        sim(&mut a).mount_action(def, op, false).unwrap();
    }
    assert!(progress::current_solved(&a, &source));
    sim(&mut a).checked = true;
    sim(&mut a)
        .edit_mount(def, "unfinished".into(), false)
        .unwrap();
    assert!(!sim(&mut a).checked);
    assert!(!progress::current_solved(&a, &source));
    sim(&mut a).save_mount(def, false).unwrap();
    assert!(!progress::current_solved(&a, &source));
    sim(&mut a)
        .mount_action(def, &fixture::save("/dev/team/draft"), false)
        .unwrap();
    assert!(progress::current_solved(&a, &source));
    sim(&mut a)
        .execute(def, "umount /srv/report", false)
        .unwrap();
    assert!(!progress::current_solved(&a, &source));
    sim(&mut a)
        .execute(def, "mount /srv/report", false)
        .unwrap();
    assert!(progress::current_solved(&a, &source));
}
#[test]
fn mixed_missing_oversized_or_shared_teaching_sources_fail_without_changing_accepted_profiles() {
    let def = definition();
    for change in 0..8 {
        let mut bad = def.clone();
        match change {
            0 => bad.mount_task = None,
            1 => bad.lesson.mount_practice = None,
            2 => bad.lesson.fixture = bad.initial.clone(),
            3 => bad.lesson.concepts.clear(),
            4 => bad.lesson.worked.push(w::Command {
                text: "lvs".into(),
                explanation: "Unrelated old command".into(),
            }),
            5 => bad.mount_task.as_mut().unwrap().reference.clear(),
            6 => {
                bad.lesson
                    .mount_practice
                    .as_mut()
                    .unwrap()
                    .context
                    .inventory
                    .filesystems[0]
                    .original_volume = 999
            }
            _ => bad.lesson.experiment = "lvs".into(),
        }
        assert!(bad.validate().is_err(), "change{change}");
    }
    let mut s = w::Session::new(&def).unwrap();
    let before = s.clone();
    for text in [
        "simulated-lvm-mount {}",
        "mount -a",
        "sh -c true",
        "cat /etc/passwd",
    ] {
        assert!(s.execute(&def, text, false).is_err());
        assert_eq!(s, before);
    }
}
