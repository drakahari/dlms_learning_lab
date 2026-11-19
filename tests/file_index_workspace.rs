//! Source-bound private-index workspace regressions; never host commands or reference tools.
#[path = "support/file_index_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    file_index_workspace::{
        CheckKey, Definition, ErrorKind, MAX_HISTORY_BYTES, MAX_SESSION, Outcome, Scope, Session,
        Step,
    },
    script_teaching::View,
};
fn bytes(s: &Session) -> Vec<u8> {
    serde_json::to_vec(s).unwrap()
}
fn d() -> Definition {
    fixture::definition()
}
#[test]
fn constructor_clones_only_stale_initial_data_and_stores_no_definition_or_expected_answers() {
    let d = d();
    d.validate().unwrap();
    let s = Session::new(&d).unwrap();
    assert_eq!(s.assignment.tree, d.assignment.initial.tree);
    assert_eq!(s.assignment.index, d.assignment.initial.index);
    assert_eq!(s.view, View::Introduction);
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
    let v = serde_json::to_value(&s).unwrap();
    assert!(
        v.get("expected_tree").is_none()
            && v.get("reference").is_none()
            && v.get("teaching").is_none()
    );
    assert!(v["assignment"].get("expected_tree").is_none());
    assert!(!s.assignment.checked);
}
#[test]
fn explicit_author_reachability_and_worked_try_repair_use_real_state_actions() {
    let d = d();
    d.validate_reachability().unwrap();
    let s = Session::new(&d).unwrap();
    assert!(!s.checks(&d, Scope::Example).unwrap().passed());
    for scope in [Scope::Assignment, Scope::Example] {
        let mut s = Session::new(&d).unwrap();
        fixture::solve(&mut s, &d, scope);
        assert!(s.check(&d, scope).unwrap().passed());
        assert!(s.current(scope).checked);
    }
}
#[test]
fn constructor_does_not_run_semantically_unsupported_author_reference() {
    let mut d = d();
    d.assignment.reference = vec![Step::Command {
        input: "unimplemented-tool".into(),
    }];
    assert!(d.validate().is_ok());
    let s = Session::new(&d).unwrap();
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
    assert!(d.validate_reachability().is_err());
}
#[test]
fn config_only_index_only_and_correct_copy_with_stale_index_have_specific_partial_results() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    s.set_config_draft(
        &d,
        Scope::Assignment,
        d.assignment.expected_tree.files[&d.assignment.initial.context.config].clone(),
    )
    .unwrap();
    s.save_config(&d, Scope::Assignment).unwrap();
    let r = s.check(&d, Scope::Assignment).unwrap();
    assert!(
        r.checks
            .iter()
            .find(|r| r.key == CheckKey::SavedPruning)
            .unwrap()
            .passed
    );
    assert!(
        !r.checks
            .iter()
            .find(|r| r.key == CheckKey::FreshIndex)
            .unwrap()
            .passed
    );
    fixture::run(
        &mut s,
        &d,
        Scope::Assignment,
        "cp /catalog/live/current.txt /work/support.txt",
    );
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
    fixture::run(
        &mut s,
        &d,
        Scope::Assignment,
        "updatedb -l 0 -o /indexes/names.db -U /catalog --config-file /work/index.conf",
    );
    assert!(s.checks(&d, Scope::Assignment).unwrap().passed());
    let mut index_only = Session::new(&d).unwrap();
    fixture::run(
        &mut index_only,
        &d,
        Scope::Assignment,
        "updatedb -l 0 -o /indexes/names.db -U /catalog --config-file /work/index.conf",
    );
    let r = index_only.checks(&d, Scope::Assignment).unwrap();
    assert!(
        r.checks
            .iter()
            .find(|r| r.key == CheckKey::FreshIndex)
            .unwrap()
            .passed
    );
    assert!(!r.passed());
}
#[test]
fn alternate_semantic_config_and_relative_copy_delete_recreate_are_valid() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    s.set_config_draft(&d,Scope::Assignment,"# meaningful alternate\nPRUNEPATHS=\"/catalog/archive /catalog/archive /catalog/archive/old\" # same exclusion\n".into()).unwrap();
    s.save_config(&d, Scope::Assignment).unwrap();
    fixture::run(&mut s, &d, Scope::Assignment, "rm support.txt");
    fixture::run(
        &mut s,
        &d,
        Scope::Assignment,
        "cp ../catalog/live/current.txt support.txt",
    );
    fixture::run(
        &mut s,
        &d,
        Scope::Assignment,
        "updatedb --config-file /work/index.conf -U /catalog -o /indexes/names.db -l 0",
    );
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn wrong_archive_and_excess_handover_modes_fail_then_retry_and_invalidation_work() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    fixture::solve(&mut s, &d, Scope::Assignment);
    assert!(s.check(&d, Scope::Assignment).unwrap().passed());
    fixture::run(
        &mut s,
        &d,
        Scope::Assignment,
        "cp /catalog/archive/current.txt /work/support.txt",
    );
    let r = s.checks(&d, Scope::Assignment).unwrap();
    assert!(!r.passed());
    assert!(
        r.checks
            .iter()
            .find(|r| r.key == CheckKey::Handover)
            .unwrap()
            .message
            .contains("wrong text")
    );
    fixture::run(
        &mut s,
        &d,
        Scope::Assignment,
        "cp /catalog/live/current.txt /work/support.txt",
    );
    fixture::run(&mut s, &d, Scope::Assignment, "chmod 777 /work/support.txt");
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
    fixture::run(&mut s, &d, Scope::Assignment, "chmod 640 /work/support.txt");
    assert!(s.checks(&d, Scope::Assignment).unwrap().passed());
    s.set_config_draft(&d, Scope::Assignment, "PRUNEPATHS=\"\"".into())
        .unwrap();
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn entered_invalid_draft_and_command_survive_failed_action_and_disk_resume() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    s.set_config_draft(
        &d,
        Scope::Assignment,
        "PRUNEPATHS = \"unterminated\0".into(),
    )
    .unwrap();
    s.set_command(
        &d,
        Scope::Assignment,
        "locate -d /indexes/names.db new | external".into(),
    )
    .unwrap();
    let before = bytes(&s);
    assert!(s.save_config(&d, Scope::Assignment).is_err());
    assert_eq!(bytes(&s), before);
    assert_eq!(
        s.run(&d, Scope::Assignment).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(bytes(&s), before);
    let restored = Session::decode(&before, &d).unwrap();
    assert_eq!(bytes(&restored), before);
    assert!(!restored.checks(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn ordinary_investigation_preserves_dirty_editor_and_quiet_update_has_separate_diagnostic() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    s.set_config_draft(&d, Scope::Assignment, "PRUNEPATHS = \"".into())
        .unwrap();
    let draft = s.assignment.config_draft.clone();
    fixture::run(&mut s, &d, Scope::Assignment, "cat /work/index.conf");
    assert_eq!(s.assignment.config_draft, draft);
    s.set_command(
        &d,
        Scope::Assignment,
        "updatedb -l 0 -o /indexes/names.db -U /catalog --config-file /work/index.conf".into(),
    )
    .unwrap();
    let r = s.run(&d, Scope::Assignment).unwrap();
    assert_eq!(r.outcome, Outcome::IndexRebuilt);
    assert!(r.stdout.is_empty());
    assert!(r.diagnostic.contains("refreshed"));
    assert_eq!(s.assignment.config_draft, draft);
}
#[test]
fn practice_success_reset_and_assignment_state_are_independent() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    s.set_command(
        &d,
        Scope::Assignment,
        "find /catalog -name current.txt".into(),
    )
    .unwrap();
    s.set_config_draft(&d, Scope::Assignment, "unfinished assignment".into())
        .unwrap();
    let assignment = serde_json::to_vec(&s.assignment).unwrap();
    fixture::solve(&mut s, &d, Scope::Example);
    assert!(s.check(&d, Scope::Example).unwrap().passed());
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
    s.set_view(&d, View::Assignment).unwrap();
    assert_eq!(serde_json::to_vec(&s.assignment).unwrap(), assignment);
    s.reset_example(&d).unwrap();
    assert_eq!(serde_json::to_vec(&s.assignment).unwrap(), assignment);
    assert!(!s.checks(&d, Scope::Example).unwrap().passed());
    assert!(s.example.transcript.is_empty());
}
#[test]
fn all_role_version_index_erasure_and_applied_state_forgery_are_rejected_without_backfill() {
    let d = d();
    let original = Session::new(&d).unwrap();
    for mutation in 0..6 {
        let mut s = original.clone();
        match mutation {
            0 => s.assignment.context = s.example.context.clone(),
            1 => s.assignment.index = None,
            2 => s.source_version += 1,
            3 => s
                .assignment
                .tree
                .users
                .insert("intruder".into(), vec!["staff".into()])
                .map(|_| ())
                .unwrap_or(()),
            4 => s
                .assignment
                .tree
                .files
                .insert("/work/private-note.txt".into(), "forged".into())
                .map(|_| ())
                .unwrap_or(()),
            _ => {
                s.assignment.tree.command_location =
                    Some(dlms_learning_lab::command_location::Context {
                        version: 1,
                        source_id: "wrong-family".into(),
                        path: "/catalog".into(),
                        names: vec!["report".into()],
                    })
            }
        }
        assert!(s.validate_for(&d).is_err());
        assert!(Session::decode(&bytes(&s), &d).is_err());
    }
}
#[test]
fn protected_edits_opaque_database_parent_and_traversal_are_honestly_atomic() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    for command in [
        "chmod 777 /catalog/live/current.txt",
        "rm /work/private-note.txt",
        "touch /work/unlisted.txt",
        "cat /indexes/names.db",
        "cp /indexes/names.db /work/support.txt",
        "ls -l /indexes",
        "find / -name names.db",
        "cat /work/../indexes/names.db",
    ] {
        s.set_command(&d, Scope::Assignment, command.into())
            .unwrap();
        let before = bytes(&s);
        assert!(s.run(&d, Scope::Assignment).is_err(), "{command}");
        assert_eq!(bytes(&s), before);
    }
    fixture::run(&mut s, &d, Scope::Assignment, "cd /indexes");
    s.set_command(&d, Scope::Assignment, "ls".into()).unwrap();
    let before = bytes(&s);
    assert_eq!(
        s.run(&d, Scope::Assignment).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    assert_eq!(bytes(&s), before);
}
#[test]
fn history_is_aggregate_bounded_and_never_a_success_certificate() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    for _ in 0..60 {
        fixture::run(
            &mut s,
            &d,
            Scope::Assignment,
            "find /catalog -name current.txt",
        );
    }
    assert!(s.assignment.transcript.len() <= 20);
    assert!(
        s.assignment
            .transcript
            .iter()
            .map(|r| r.input.len() + r.output.len() + 128)
            .sum::<usize>()
            <= MAX_HISTORY_BYTES
    );
    s.assignment.checked = true;
    s.assignment.transcript[0].output = "All goals passed".into();
    assert!(!s.checks(&d, Scope::Assignment).unwrap().passed());
}
#[test]
fn malformed_duplicates_oversize_selection_and_source_free_shape_do_not_confer_authority() {
    let d = d();
    let s = Session::new(&d).unwrap();
    assert!(s.validate_shape().is_ok());
    assert!(Session::decode(&vec![b' '; MAX_SESSION + 1], &d).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["assignment"]
        .as_object_mut()
        .unwrap()
        .remove("config_draft");
    assert!(Session::decode(&serde_json::to_vec(&v).unwrap(), &d).is_err());
    let mut wrong = s.clone();
    wrong.source_id = "other-source".into();
    assert!(wrong.validate_shape().is_ok());
    assert!(wrong.validate_for(&d).is_err());
    let mut selected = s.clone();
    selected.selected_example = d.teaching.try_steps.len();
    assert!(selected.validate_for(&d).is_err());
    let mut huge = s.clone();
    huge.assignment.config_draft = "x".repeat(4097);
    assert!(huge.validate_for(&d).is_err());
    let json = String::from_utf8(bytes(&s)).unwrap();
    let duplicate = json.replacen("{", "{\"version\":1,", 1);
    assert!(Session::decode(duplicate.as_bytes(), &d).is_err());
}
#[test]
fn declared_example_experiments_are_partial_until_real_restoration_not_cached_history() {
    let d = d();
    let mut s = Session::new(&d).unwrap();
    fixture::solve(&mut s, &d, Scope::Example);
    fixture::run(&mut s, &d, Scope::Example, "rm /gallery/live/current.txt");
    fixture::run(
        &mut s,
        &d,
        Scope::Example,
        "cp /gallery/seed.txt /gallery/live/experiment.txt",
    );
    assert!(!s.checks(&d, Scope::Example).unwrap().passed());
    fixture::run(
        &mut s,
        &d,
        Scope::Example,
        "cp /gallery/seed.txt /gallery/live/current.txt",
    );
    fixture::run(
        &mut s,
        &d,
        Scope::Example,
        "rm /gallery/live/experiment.txt",
    );
    assert!(
        s.checks(&d, Scope::Example).unwrap().passed(),
        "Semantic freshness accepts restored exact names without demanding a generation counter"
    );
}
