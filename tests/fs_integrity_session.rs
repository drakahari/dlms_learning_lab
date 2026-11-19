#[path = "support/fs_integrity.rs"]
mod fixture;
use dlms_learning_lab::{
    fs_integrity_terminal as terminal,
    fs_integrity_workspace::{self as w, Action, MAX_TICKS, Run, Scope, Session},
};
fn apply(scope: &mut Scope, task: &w::Task, commands: &[String]) {
    for command in commands {
        scope.execute_recorded(task, command).unwrap();
    }
}
#[test]
fn separate_practice_and_assignment_require_actual_checks_and_edits_clear_stale_success() {
    for recovery in [false, true] {
        let d = fixture::definition(recovery);
        let mut s = Session::new(d.clone()).unwrap();
        let before = serde_json::to_value(&s.assignment).unwrap();
        apply(&mut s.example, &d.example, &d.example_commands);
        assert!(!s.example.passed(&d.example));
        assert!(
            s.example
                .run_checked(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert!(!s.assignment.passed(&d.assignment));
        assert_eq!(serde_json::to_value(&s.assignment).unwrap(), before);
        apply(
            &mut s.assignment,
            &d.assignment,
            &fixture::recipe(&d.assignment, recovery),
        );
        assert!(!s.assignment.passed(&d.assignment));
        s.assignment.run_checked(&d.assignment).unwrap();
        assert!(s.assignment.passed(&d.assignment));
        let history = s.assignment.history().to_vec();
        s.assignment
            .execute_recorded(&d.assignment, "findmnt")
            .unwrap();
        assert_eq!(s.assignment.history(), history);
        assert!(s.assignment.passed(&d.assignment));
        assert!(
            s.assignment
                .execute_recorded(&d.assignment, "sh -c anything")
                .is_err()
        );
        assert!(s.assignment.passed(&d.assignment)); // State did not change.
        s.assignment
            .execute_recorded(&d.assignment, "umount /mnt/data")
            .unwrap();
        assert!(!s.assignment.passed(&d.assignment));
        s.assignment
            .execute_recorded(&d.assignment, "mount /dev/vdb1 /mnt/data")
            .unwrap();
        assert!(!s.assignment.passed(&d.assignment));
        s.assignment.run_checked(&d.assignment).unwrap();
        assert!(s.assignment.passed(&d.assignment));
    }
}
#[test]
fn saved_drafts_and_history_restore_without_trusting_serialized_success() {
    let d = fixture::definition(true);
    let mut s = Session::new(d.clone()).unwrap();
    apply(
        &mut s.assignment,
        &d.assignment,
        &fixture::recipe(&d.assignment, true),
    );
    s.assignment.run_checked(&d.assignment).unwrap();
    apply(&mut s.example, &d.example, &d.example_commands);
    s.assignment.command = "dumpe2fs -h /dev/vdb1".into();
    s.example.command = "ls -li /mnt/workshop".into();
    s.view = dlms_learning_lab::script_teaching::View::Example;
    let current = s.assignment.current().unwrap().clone();
    let raw = serde_json::to_string(&s).unwrap();
    assert!(!raw.contains("\"proof\"") && !raw.contains("\"current\""));
    let mut restored = Session::decode(&raw, &d).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&s).unwrap()
    );
    assert!(restored.assignment.current().is_none());
    assert!(restored.assignment.pending_recheck());
    assert!(!restored.assignment.passed(&d.assignment));
    restored.assignment.restore_current(&d.assignment).unwrap();
    assert_eq!(restored.assignment.current(), Some(&current));
    assert!(!restored.assignment.passed(&d.assignment));
    restored.assignment.run_checked(&d.assignment).unwrap();
    assert!(restored.assignment.passed(&d.assignment));
    assert_eq!(restored.assignment.command, "dumpe2fs -h /dev/vdb1");
    assert_eq!(restored.example.command, "ls -li /mnt/workshop");
    let mut forged = serde_json::to_value(Session::new(d.clone()).unwrap()).unwrap();
    forged["assignment"]["checked"] = serde_json::json!([]);
    let mut loaded = Session::decode(&forged.to_string(), &d).unwrap();
    assert!(!loaded.assignment.passed(&d.assignment));
    loaded.assignment.run_checked(&d.assignment).unwrap();
    assert!(!loaded.assignment.passed(&d.assignment));
}
#[test]
fn stopped_stale_and_foreign_jobs_cannot_finish_and_reset_is_selected_scope_only() {
    let d = fixture::definition(false);
    let mut s = Session::new(d.clone()).unwrap();
    apply(
        &mut s.assignment,
        &d.assignment,
        &fixture::recipe(&d.assignment, false),
    );
    let mut run = s.assignment.begin(&d.assignment).unwrap();
    run.tick();
    run.cancel();
    assert!(s.assignment.finish(&d.assignment, &run).is_err());
    assert!(!s.assignment.passed(&d.assignment));
    let mut run = s.assignment.begin(&d.assignment).unwrap();
    for _ in 0..MAX_TICKS {
        if run.tick() {
            break;
        }
    }
    s.assignment
        .execute_recorded(&d.assignment, "umount /mnt/data")
        .unwrap();
    assert!(s.assignment.finish(&d.assignment, &run).is_err());
    let mut other = Run::new(&d.example, &d.example_commands).unwrap();
    for _ in 0..MAX_TICKS {
        if other.tick() {
            break;
        }
    }
    assert!(s.assignment.finish(&d.assignment, &other).is_err());
    apply(&mut s.example, &d.example, &d.example_commands);
    let saved = serde_json::to_value(&s.example).unwrap();
    s.assignment = Scope::new(&d.assignment).unwrap();
    assert_eq!(serde_json::to_value(&s.example).unwrap(), saved);
    assert!(s.assignment.history().is_empty());
}
#[test]
fn mutation_history_limits_are_atomic_and_readonly_queries_remain_available() {
    let d = fixture::definition(false);
    let mut scope = Scope::new(&d.assignment).unwrap();
    for i in 0..w::MAX_HISTORY {
        scope
            .execute_recorded(
                &d.assignment,
                if i % 2 == 0 {
                    "umount /mnt/data"
                } else {
                    "mount /dev/vdb1 /mnt/data"
                },
            )
            .unwrap();
    }
    assert_eq!(scope.history().len(), 64);
    let current = scope.current().unwrap().clone();
    let history = scope.history().to_vec();
    let e = scope
        .execute_recorded(&d.assignment, "umount /mnt/data")
        .unwrap_err();
    assert_eq!(e.kind, terminal::DiagnosticKind::Limit);
    assert_eq!(scope.current(), Some(&current));
    assert_eq!(scope.history(), history);
    scope.execute_recorded(&d.assignment, "findmnt").unwrap();
    assert_eq!(scope.history(), history);
    scope.validate(&d.assignment).unwrap();
}
#[test]
fn console_storage_is_bounded_and_unicode_limit_feedback_never_panics() {
    let d = fixture::definition(false);
    let mut scope = Scope::new(&d.assignment).unwrap();
    let input = format!("{}é", "x".repeat(2047));
    assert!(scope.execute_recorded(&d.assignment, &input).is_err());
    assert!(scope.transcript.last().unwrap().input.len() <= 2048);
    for _ in 0..120 {
        scope.execute_recorded(&d.assignment, "findmnt").unwrap();
    }
    assert!(scope.transcript.len() <= 40);
    assert!(dlms_learning_lab::cron_workspace::within(
        &scope.transcript,
        w::MAX_TRANSCRIPT_BYTES
    ));
    scope.validate(&d.assignment).unwrap();
    let raw = serde_json::to_string(&Session::new(d.clone()).unwrap()).unwrap();
    let mut bad = serde_json::from_str::<serde_json::Value>(&raw).unwrap();
    bad["assignment"]["history"] = serde_json::json!(["curl https://example.test"]);
    assert!(Session::decode(&bad.to_string(), &d).is_err());
    let mut unknown = serde_json::from_str::<serde_json::Value>(&raw).unwrap();
    unknown["assignment"]["history"] = serde_json::json!(["mount /dev/sda1 /mnt/data"]);
    assert!(Session::decode(&unknown.to_string(), &d).is_err());

    let mut bad = serde_json::from_str::<serde_json::Value>(&raw).unwrap();
    bad["assignment"]["passed"] = serde_json::json!(true);
    assert!(Session::decode(&bad.to_string(), &d).is_err());
    let mut bad = serde_json::from_str::<serde_json::Value>(&raw).unwrap();
    bad["assignment"]["history"] =
        serde_json::Value::Array((0..65).map(|_| serde_json::json!("findmnt")).collect());
    assert!(Session::decode(&bad.to_string(), &d).is_err());
    assert!(Session::decode(&" ".repeat(w::MAX_SESSION_BYTES + 1), &d).is_err());
    let mut other = d.clone();
    other.assignment.instructions.push_str(" changed");
    assert!(Session::decode(&raw, &other).is_err());
}
#[test]
fn internal_reference_codec_checks_actual_behavior_and_never_becomes_a_shell() {
    let d = fixture::definition(true);
    let mut s = Session::new(d.clone()).unwrap();
    for command in fixture::recipe(&d.assignment, true) {
        let action = Action::Command { input: command };
        s.execute_reference(
            &d,
            &format!("{}{}", w::PREFIX, serde_json::to_string(&action).unwrap()),
        )
        .unwrap();
    }
    assert!(!s.assignment.passed(&d.assignment));
    let result = s
        .execute_reference(
            &d,
            &format!(
                "{}{}",
                w::PREFIX,
                serde_json::to_string(&Action::Check {}).unwrap()
            ),
        )
        .unwrap();
    assert!(result.contains("2/2"));
    assert!(s.assignment.passed(&d.assignment));
    assert!(s.execute_reference(&d, "e2fsck -fy /dev/vdb1").is_err());
    assert!(
        s.execute_reference(
            &d,
            &format!("{}{{\"kind\":\"Check\",\"passed\":true}}", w::PREFIX)
        )
        .is_err()
    );
    let before = serde_json::to_value(&s).unwrap();
    assert!(
        s.execute_reference(
            &d,
            &format!(
                "{}{{\"kind\":\"Command\",\"input\":\"sh -c touch\"}}",
                w::PREFIX
            )
        )
        .is_err()
    );
    assert!(s.assignment.passed(&d.assignment));
    assert_eq!(
        before["assignment"]["history"],
        serde_json::to_value(&s).unwrap()["assignment"]["history"]
    );
}

#[test]
fn stopped_unchanged_rechecks_preserve_history_but_require_real_revalidation_after_load() {
    let d = fixture::definition(false);
    let mut s = Session::new(d.clone()).unwrap();
    apply(
        &mut s.assignment,
        &d.assignment,
        &fixture::recipe(&d.assignment, false),
    );
    s.assignment.run_checked(&d.assignment).unwrap();
    let before = s.assignment.history().to_vec();
    let mut run = s.assignment.begin(&d.assignment).unwrap();
    run.tick();
    run.cancel();
    assert!(!s.assignment.passed(&d.assignment));
    assert!(s.assignment.pending_recheck());
    let raw = serde_json::to_string(&s).unwrap();
    let mut loaded = Session::decode(&raw, &d).unwrap();
    assert_eq!(loaded.assignment.history(), before);
    assert!(loaded.assignment.pending_recheck());
    assert!(!loaded.assignment.passed(&d.assignment));
    loaded.assignment.run_checked(&d.assignment).unwrap();
    assert!(loaded.assignment.passed(&d.assignment));
    assert!(loaded.assignment.current().is_some());
}
