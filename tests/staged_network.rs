use dlms_learning_lab::automation_workshop::{
    change_rollout::{Change, PortSettings},
    intent_path::Exchange,
    rollout_task::{Recorded, Task as Rollout},
    workspace::{Action, IntentTask, Scope, Task, Work},
};

fn facts(case: &str) -> (Rollout, IntentTask) {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/staged_network_tasks_v1.json")).unwrap();
    (
        serde_json::from_value(data[case]["rollout"]["task"].clone()).unwrap(),
        serde_json::from_value(data[case]["intent"].clone()).unwrap(),
    )
}

fn rebind(slot: &str) -> Change {
    Change::Rebind {
        slot: slot.into(),
        profile: "approved-service".into(),
    }
}

#[test]
fn pilot_model_reports_shared_scope_and_reconciles_manual_override() {
    let (task, _) = facts("assignment");
    task.validate().unwrap();
    let mut actual = task.initial.clone();
    let pilot = &task.approved[0];
    let manual = actual
        .preview(
            &task.spec,
            &Change::Individual {
                slot: pilot.slot.clone(),
                settings: PortSettings { up: true, vlan: 30 },
            },
        )
        .unwrap();
    assert_eq!(manual.affected.len(), 1);
    let shared = actual
        .preview(
            &task.spec,
            &Change::SharedProfile {
                profile: "intake".into(),
                settings: PortSettings { up: true, vlan: 30 },
            },
        )
        .unwrap();
    assert_eq!(shared.affected.len(), 3);
    assert!(
        shared
            .affected
            .iter()
            .any(|row| row.slot == task.reserved.slot)
    );
    assert!(
        !actual
            .visible(&task.spec, &pilot.host, &task.intended_peer)
            .unwrap()
    );
    actual.apply(&task.spec, manual).unwrap();
    assert!(
        actual
            .visible(&task.spec, &pilot.host, &task.intended_peer)
            .unwrap()
    );
    assert!(
        actual.apply(&task.spec, shared).is_err(),
        "An older preview becomes stale"
    );
    let restore = actual.preview(&task.spec, &Change::Reconcile).unwrap();
    actual.apply(&task.spec, restore).unwrap();
    assert!(
        !actual
            .visible(&task.spec, &pilot.host, &task.intended_peer)
            .unwrap()
    );
}

#[test]
fn either_pilot_works_and_a_skipped_or_overbroad_pilot_fails() {
    let (task, _) = facts("assignment");
    for index in 0..2 {
        let a = Recorded::Apply {
            change: rebind(&task.approved[index].slot),
        };
        let b = Recorded::Apply {
            change: rebind(&task.approved[1 - index].slot),
        };
        let verify = Recorded::VerifyPilot { index };
        let reconcile = Recorded::Apply {
            change: Change::Reconcile,
        };
        let actual = task
            .replay(&[a.clone(), verify.clone(), b.clone(), reconcile.clone()])
            .unwrap();
        assert!(task.checks(&actual).unwrap().iter().all(|row| row.0));
        let late = task.replay(&[a, b, verify, reconcile]).unwrap();
        assert!(!late.pilot_verified);
        assert!(!task.checks(&late).unwrap()[0].0);
    }
    let broad = task
        .replay(&[
            Recorded::Apply {
                change: Change::SharedProfile {
                    profile: "intake".into(),
                    settings: PortSettings { up: true, vlan: 30 },
                },
            },
            Recorded::VerifyPilot { index: 0 },
            Recorded::Apply {
                change: Change::Reconcile,
            },
        ])
        .unwrap();
    let failed = task.checks(&broad).unwrap();
    assert!(!failed[0].0 && !failed[2].0 && !failed[3].0);
}

#[test]
fn path_observation_distinguishes_desired_installed_and_carried() {
    let (_, task) = facts("assignment");
    task.validate().unwrap();
    let mut state = task.starter;
    let request = Exchange { from: 0, to: 1 };
    let before = task.spec.trace(state, request).unwrap();
    assert!(before.desired && !before.installed && before.carrier.is_err());
    state.submit(Exchange { from: 1, to: 0 }, true).unwrap();
    state.publish();
    let published = task.spec.trace(state, request).unwrap();
    assert!(published.desired && published.installed && !published.forwarded());
    state.uplink_up = true;
    let finished = task.spec.trace(state, request).unwrap();
    assert!(finished.forwarded());
    assert_eq!(
        finished.carrier.unwrap().forward.destination_device,
        "office-edge"
    );
    assert!(task.outcomes(state).unwrap().iter().all(|row| row.passed));
    state.submit(Exchange { from: 2, to: 1 }, true).unwrap();
    state.publish();
    assert!(task.outcomes(state).unwrap().iter().any(|row| !row.passed));
}

#[test]
fn invalid_drafts_stay_unapplied_and_edits_invalidate_checks() {
    let (rollout, intent) = facts("example");
    let i = Task::Intent {
        supplied: Box::new(intent),
    };
    let mut work = Work::new(&i);
    work.apply(&i, Action::Check).unwrap();
    let same = Exchange { from: 1, to: 1 };
    work.apply(
        &i,
        Action::IntentDraft {
            request: same,
            allow: false,
        },
    )
    .unwrap();
    assert!(!work.checked);
    let original = work.clone();
    assert!(work.apply(&i, Action::Submit).is_err());
    assert_eq!(work, original);
    let r = Task::Rollout {
        supplied: Box::new(rollout),
    };
    let mut work = Work::new(&r);
    work.apply(&r, Action::Preview).unwrap();
    work.apply(
        &r,
        Action::RolloutDraft {
            draft: rebind("bench-a"),
        },
    )
    .unwrap();
    assert!(matches!(
        &work.current,
        Scope::Rollout { preview: false, .. }
    ));
    assert!(work.apply(&r, Action::Apply).is_err());
}
