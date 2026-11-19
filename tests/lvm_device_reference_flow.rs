//! Standalone source-bound draft and guided-flow behavior; no native admission.
use dlms_learning_lab::{
    lvm::{
        self, device_path,
        device_reference::{Caller, Configuration},
    },
    lvm_workspace::{
        self as work,
        device_reference::{Plan, Target},
        device_reference_flow::*,
    },
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn state() -> lvm::State {
    let v: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    serde_json::from_value(v["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap()
}
fn context(s: &lvm::State) -> Context {
    let file = s
        .volume("team/draft")
        .unwrap()
        .1
        .filesystem
        .as_ref()
        .unwrap()
        .files
        .keys()
        .next()
        .unwrap()
        .clone();
    Context {
        initial: Configuration {
            callers: BTreeMap::from([(
                "reporting".into(),
                Caller {
                    label: "Reporting application".into(),
                    file,
                    path: device_path::modeled_mapper("team", "draft").unwrap(),
                },
            )]),
        },
        plan: Plan {
            targets: vec![Target {
                caller: "reporting".into(),
                original_volume: "team/draft".into(),
                canonical_required: true,
            }],
            renaming: None,
        },
    }
}
fn save(path: &str) -> Operation {
    Operation::SaveReference {
        caller: "reporting".into(),
        path: path.into(),
    }
}
fn command(text: &str) -> Operation {
    Operation::Command { text: text.into() }
}
fn all(l: &Live, c: &Context, s: &lvm::State, current: &lvm::State) -> bool {
    l.checks(c, s, current).iter().all(|r| r.0)
}
fn practice(c: &Context) -> Practice {
    Practice {
        context: c.clone(),
        worked: vec![Step {
            operation: save("/dev/team/draft"),
            explanation:
                "Save the recommended name-based reference, then inspect the actual virtual data."
                    .into(),
        }],
        experiment: save("/dev/no_group/no_volume"),
        repair: vec![save("/dev/team/draft")],
    }
}
#[test]
fn edits_are_inert_and_pending_drafts_invalidate_success_before_explicit_save() {
    let s = state();
    let c = context(&s);
    let mut current = s.clone();
    let mut l = Live::new(&c).unwrap();
    let before = l.configuration.clone();
    l.set_draft(&c, &s, &current, "reporting", "/dev/team/draft".into())
        .unwrap();
    assert_eq!(l.configuration, before);
    assert_eq!(current, s);
    assert!(!all(&l, &c, &s, &current));
    l.save_draft(&c, &s, &mut current, "reporting").unwrap();
    assert!(all(&l, &c, &s, &current));
    let actual = l
        .configuration
        .inspect(&current, "reporting")
        .unwrap()
        .content
        .to_owned();
    l.set_draft(
        &c,
        &s,
        &current,
        "reporting",
        "/dev/no_group/no_volume".into(),
    )
    .unwrap();
    assert!(!all(&l, &c, &s, &current));
    assert_eq!(
        l.configuration
            .inspect(&current, "reporting")
            .unwrap()
            .content,
        actual
    );
    let output = l.save_draft(&c, &s, &mut current, "reporting").unwrap();
    assert!(output.contains("Saved") && output.contains("needs attention"));
    assert!(!all(&l, &c, &s, &current));
    l.apply(&c, &s, &mut current, &save("/dev/team/draft"))
        .unwrap();
    assert!(all(&l, &c, &s, &current));
    assert_eq!(current, s);
}
#[test]
fn separate_practice_live_state_drafts_and_storage_do_not_earn_assignment_credit() {
    let s = state();
    let c = context(&s);
    let assignment = Live::new(&c).unwrap();
    let assignment_storage = s.clone();
    let mut example = Live::new(&c).unwrap();
    let mut example_storage = s.clone();
    // Owned-copy isolation fixture only; actual authored example values/worlds
    // must be distinct at the future Definition admission gate.
    example
        .set_draft(
            &c,
            &s,
            &example_storage,
            "reporting",
            "/dev/team/draft".into(),
        )
        .unwrap();
    example
        .save_draft(&c, &s, &mut example_storage, "reporting")
        .unwrap();
    assert!(all(&example, &c, &s, &example_storage));
    assert!(!all(&assignment, &c, &s, &assignment_storage));
    assert_eq!(assignment.configuration, c.initial);
    assert_eq!(assignment_storage, s);
    example = Live::new(&c).unwrap();
    example_storage = s.clone();
    assert!(!all(&example, &c, &s, &example_storage));
    assert_eq!(assignment.configuration, c.initial);
}
#[test]
fn real_worked_practice_mistake_and_repair_have_actual_changed_outcomes() {
    let s = state();
    let c = context(&s);
    let p = practice(&c);
    p.validate(&s).unwrap();
    let mut storage = s.clone();
    let mut l = Live::new(&c).unwrap();
    for step in &p.worked {
        l.apply(&c, &s, &mut storage, &step.operation).unwrap();
    }
    assert!(all(&l, &c, &s, &storage));
    l.apply(&c, &s, &mut storage, &p.experiment).unwrap();
    assert!(!all(&l, &c, &s, &storage));
    assert!(l.configuration.inspect(&storage, "reporting").is_err());
    for op in &p.repair {
        l.apply(&c, &s, &mut storage, op).unwrap();
    }
    assert!(all(&l, &c, &s, &storage));
    let task = Task {
        context: c,
        reference: vec![command("vgs"), save("/dev/team/draft"), command("lvs")],
    };
    task.validate(&s).unwrap();
}
#[test]
fn partial_saved_config_and_different_unsaved_draft_roundtrip_without_fabricating_passes() {
    let s = state();
    let c = context(&s);
    let mut current = s.clone();
    let mut l = Live::new(&c).unwrap();
    l.apply(&c, &s, &mut current, &save("/dev/no_group/no_volume"))
        .unwrap();
    l.set_draft(&c, &s, &current, "reporting", "/dev/team/draft".into())
        .unwrap();
    let bytes = serde_json::to_vec(&(current.clone(), l.clone())).unwrap();
    let (mut restored, mut live): (lvm::State, Live) = serde_json::from_slice(&bytes).unwrap();
    live.validate(&c, &s, &restored).unwrap();
    assert_eq!(live, l);
    assert_eq!(restored, current);
    assert!(!all(&live, &c, &s, &restored));
    live.save_draft(&c, &s, &mut restored, "reporting").unwrap();
    assert!(all(&live, &c, &s, &restored));
    assert!(!all(&l, &c, &s, &current));
}
#[test]
fn failed_actions_and_oversized_or_control_edits_leave_entire_prior_work_unchanged() {
    let s = state();
    let c = context(&s);
    let mut current = s.clone();
    let mut l = Live::new(&c).unwrap();
    for op in [
        command("sh -c lvs"),
        command("dmsetup splitname team-draft"),
        Operation::SaveReference {
            caller: "unknown".into(),
            path: "/dev/team/draft".into(),
        },
        save("/dev/team/draft\0"),
    ] {
        let before = (current.clone(), l.clone());
        assert!(l.apply(&c, &s, &mut current, &op).is_err());
        assert_eq!((current.clone(), l.clone()), before);
    }
    for text in ["x".repeat(257), "/dev/team/draft\n".into()] {
        let before = l.clone();
        assert!(l.set_draft(&c, &s, &current, "reporting", text).is_err());
        assert_eq!(l, before);
        assert_eq!(current, s);
    }
    l.apply(&c, &s, &mut current, &save("/dev/../host"))
        .unwrap();
    assert!(!all(&l, &c, &s, &current));
    l.apply(&c, &s, &mut current, &save("/dev/team/draft"))
        .unwrap();
    assert!(all(&l, &c, &s, &current));
}
#[test]
fn imported_draft_maps_stop_before_excess_or_duplicate_values_and_reject_foreign_source() {
    let s = state();
    let c = context(&s);
    let l = Live::new(&c).unwrap();
    let cfg = serde_json::to_string(&l.configuration).unwrap();
    for raw in [
        format!(
            "{{\"configuration\":{cfg},\"drafts\":{{\"a\":\"1\",\"b\":\"2\",\"c\":\"3\",\"d\":\"4\",\"e\": NOT_JSON}}}}"
        ),
        format!("{{\"configuration\":{cfg},\"drafts\":{{\"a\":\"1\",\"a\": NOT_JSON}}}}"),
    ] {
        let e = serde_json::from_str::<Live>(&raw).unwrap_err();
        assert!(e.to_string().contains("before value"), "{e}");
    }
    let mut value = serde_json::to_value(&l).unwrap();
    value["drafts"] = json!({"foreign":"/dev/team/draft"});
    let wrong: Live = serde_json::from_value(value).unwrap();
    assert!(wrong.validate(&c, &s, &s).is_err());
    let mut value = serde_json::to_value(&l).unwrap();
    value["configuration"]["callers"]["reporting"]["file"] = "/another.txt".into();
    let wrong: Live = serde_json::from_value(value).unwrap();
    assert!(wrong.validate(&c, &s, &s).is_err());
    let mut value = serde_json::to_value(save("/dev/team/draft")).unwrap();
    value["execute"] = "host shell".into();
    assert!(serde_json::from_value::<Operation>(value).is_err());
}
#[test]
fn authoring_rejects_no_work_noop_exploration_wrong_repairs_and_unexplained_steps() {
    let s = state();
    let c = context(&s);
    let mut t = Task {
        context: c.clone(),
        reference: vec![command("lvs")],
    };
    assert!(t.validate(&s).is_err());
    t.reference.clear();
    assert!(t.validate(&s).is_err());
    let mut done = c.clone();
    done.initial.save("reporting", "/dev/team/draft").unwrap();
    assert!(
        Task {
            context: done,
            reference: vec![command("lvs")]
        }
        .validate(&s)
        .is_err()
    );
    let mut p = practice(&c);
    p.experiment = command("lvs");
    assert!(p.validate(&s).is_err());
    let mut p = practice(&c);
    p.repair = vec![command("lvs")];
    assert!(p.validate(&s).is_err());
    let mut p = practice(&c);
    p.worked[0].explanation.clear();
    assert!(p.validate(&s).is_err());
}
#[test]
fn current_normal_goal_and_session_wire_still_reject_reference_runtime_injection() {
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let d: work::Definition =
        serde_json::from_value(raw["scenarios"][0]["goal"]["definition"].clone()).unwrap();
    let session = work::Session::new(&d).unwrap();
    let mut value = serde_json::to_value(&d).unwrap();
    value["device_reference_task"] = json!({});
    assert!(serde_json::from_value::<work::Definition>(value).is_err());
    let mut value = serde_json::to_value(session).unwrap();
    value["reference_runtime"] = json!({});
    assert!(serde_json::from_value::<work::Session>(value).is_err());
}
