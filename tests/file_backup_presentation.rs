//! Borrowed native field admission and complete-attempt refusal, not UI lookalikes.
#[path = "support/file_backup_authored.rs"]
mod fixture;
use dlms_learning_lab::{
    file_backup_teaching as teaching,
    file_backup_workspace::{
        self as work,
        presentation::{self, Edit, present},
    },
    progress::{self, Attempt},
    script_teaching::View,
};
fn fresh() -> (&'static dlms_learning_lab::pack::Scenario, Attempt) {
    let review = fixture::review();
    let id = review["labs"][0]["id"].as_str().unwrap();
    assert_eq!(id, "lpic2-backup-selective-recovery");
    let s = fixture::lab(id);
    let mut a = Attempt::new(s, 9);
    teaching::prepare(&mut a, s.file_backup_teaching.as_ref().unwrap(), true).unwrap();
    (s, a)
}
fn wire(a: &Attempt) -> Vec<u8> {
    serde_json::to_vec(a).unwrap()
}
fn frontier(a: &mut Attempt) {
    a.transcript = vec![String::new(); 120];
    let remaining = work::MAX_ATTEMPT - work::size(a, work::MAX_ATTEMPT).unwrap();
    for (i, t) in a.transcript.iter_mut().enumerate() {
        *t = "x".repeat(remaining / 120 + usize::from(i < remaining % 120));
    }
    assert_eq!(wire(a).len(), work::MAX_ATTEMPT);
}
#[test]
fn separate_commands_views_and_modes_never_execute_or_award_credit() {
    let (s, mut a) = fresh();
    let before = serde_json::to_value(&a.state).unwrap();
    let practice = serde_json::to_value(
        &a.workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .state,
    )
    .unwrap();
    present(
        &mut a,
        s,
        false,
        Edit {
            command: Some("tar -tf /backups/project-full.tar"),
            ..Default::default()
        },
    )
    .unwrap();
    present(
        &mut a,
        s,
        true,
        Edit {
            command: Some("printf 'not a supported backup utility'"),
            view: Some(View::Example),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        a.workspace.as_ref().unwrap().command,
        "tar -tf /backups/project-full.tar"
    );
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .file_backup_learning
            .as_ref()
            .unwrap()
            .command,
        "printf 'not a supported backup utility'"
    );
    presentation::record_mode(&mut a, s, true).unwrap();
    presentation::record_mode(&mut a, s, false).unwrap();
    assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
    assert_eq!(
        serde_json::to_value(
            &a.workspace
                .as_ref()
                .unwrap()
                .file_backup_learning
                .as_ref()
                .unwrap()
                .state
        )
        .unwrap(),
        practice
    );
    assert!(a.transcript.is_empty());
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn exact_frontier_accepts_shrinking_and_rejects_escaped_growth_without_losing_any_field() {
    let (s, mut a) = fresh();
    present(
        &mut a,
        s,
        false,
        Edit {
            command: Some("abcdef"),
            ..Default::default()
        },
    )
    .unwrap();
    frontier(&mut a);
    work::context(&a, s).unwrap();
    let before = wire(&a);
    assert!(
        present(
            &mut a,
            s,
            false,
            Edit {
                command: Some("üü\t\"quoted\""),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(wire(&a), before);
    present(
        &mut a,
        s,
        false,
        Edit {
            command: Some(""),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT - 6);
    assert_eq!(a.workspace.as_ref().unwrap().command, "");
    work::context(&a, s).unwrap();
}
#[test]
fn full_boundary_mode_visit_and_view_changes_are_admitted_by_actual_serialized_delta() {
    let (s, mut a) = fresh();
    presentation::record_mode(&mut a, s, true).unwrap();
    present(
        &mut a,
        s,
        false,
        Edit {
            view: Some(View::Assignment),
            ..Default::default()
        },
    )
    .unwrap();
    frontier(&mut a);
    let before = wire(&a);
    assert!(presentation::record_mode(&mut a, s, false).is_err());
    assert_eq!(wire(&a), before);
    assert!(presentation::record_visit(&mut a, s, 10).is_err());
    assert_eq!(wire(&a), before);
    assert!(
        present(
            &mut a,
            s,
            false,
            Edit {
                view: Some(View::Introduction),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(wire(&a), before);
    presentation::record_mode(&mut a, s, true).unwrap();
    presentation::record_visit(&mut a, s, 9).unwrap();
    assert_eq!(wire(&a), before);
    present(
        &mut a,
        s,
        false,
        Edit {
            checked: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(wire(&a).len(), work::MAX_ATTEMPT - 1);
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn bad_utf8_command_shape_selection_and_backwards_visit_keep_whole_attempt() {
    let (s, mut a) = fresh();
    let before = wire(&a);
    for bad in ["two\ncommands", "nul\0value", "return\rline"] {
        assert!(
            present(
                &mut a,
                s,
                false,
                Edit {
                    command: Some(bad),
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(wire(&a), before);
    }
    assert!(
        present(
            &mut a,
            s,
            true,
            Edit {
                example_command: Some(usize::MAX),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(wire(&a), before);
    assert!(presentation::record_visit(&mut a, s, 8).is_err());
    assert_eq!(wire(&a), before);
}
#[test]
fn foreign_teaching_is_refused_before_any_small_field_can_be_committed() {
    let (s, mut a) = fresh();
    let mut wrong = s.clone();
    wrong.teaching = dlms_learning_lab::pack::bundled()
        .unwrap()
        .into_iter()
        .find_map(|s| s.teaching);
    assert!(wrong.teaching.is_some());
    let before = wire(&a);
    assert!(
        present(
            &mut a,
            &wrong,
            false,
            Edit {
                command: Some("ls"),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(wire(&a), before);
    for command in &s.solution {
        work::perform(&mut a, s, work::Scope::Assignment, command).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    assert!(!progress::current_solved(&a, &wrong));
    assert!(progress::current_solved(&a, s));
}
