//! Actual outcome certificates, malformed keys and bounds; not authored labs.
use super::*;
use crate::ui::tests::physical_workspace as fixture;
use dlms_learning_lab::{physical_link as core, physical_link_workspace as work};
fn solved(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    work::prepare_attempt(&mut a, s, true).unwrap();
    let core::Action::Configure { configuration } =
        core::Action::from_json(s.solution[0].strip_prefix(work::PREFIX).unwrap().as_bytes())
            .unwrap()
    else {
        panic!("Typed reference")
    };
    let mut draft = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .clone()
        .unwrap();
    draft.configuration = configuration;
    work::set_draft(&mut a, s, work::Scope::Assignment, &draft).unwrap();
    work::apply(&mut a, s, work::Scope::Assignment).unwrap();
    assert!(work::checks(&a, s).iter().all(|r| r.0));
    a
}
fn publish(index: &Handle, a: &Attempt, s: &Scenario) {
    let (key, _) = index.admit(a, s, Family::Physical).unwrap();
    let rows = work::checks(a, s);
    index
        .publish(&key, a, s, !rows.is_empty() && rows.iter().all(|r| r.0))
        .unwrap();
}
#[test]
fn seven_large_exact_completed_keys_survive_selected_four_snapshot_rotation_and_idle_has_no_json() {
    let index = Handle::default();
    let mut memo = physical_link_workspace::memo::Memo::default();
    memo.certificates = index.clone();
    let mut sources = Vec::new();
    for n in 0..7 {
        let mut s = fixture::source();
        s.id = format!("test-physical-certificate-{n}");
        let mut a = solved(&s);
        a.transcript = vec!["x".repeat(10_000); 100];
        work::context(&a, &s).unwrap();
        memo.prepare(&a, &s, false).unwrap();
        sources.push((s, a));
    }
    assert!(memo.entries_count() <= 4);
    assert_eq!(index.observed().0, 7);
    for (s, a) in &sources {
        assert_eq!(index.status(a, s), Some(Status::Complete));
    }
    let json = dlms_learning_lab::native_observations::json_admission_calls();
    let before = index.observed();
    let counts = memo.counters;
    for _ in 0..80 {
        let (s, a) = sources.last().unwrap();
        memo.prepare(a, s, false).unwrap();
        for (s, a) in &sources {
            assert_eq!(index.status(a, s), Some(Status::Complete));
        }
    }
    let after = index.observed();
    assert_eq!(
        (
            after.0,
            after.1,
            after.2.contexts,
            after.2.key_copies,
            after.2.certificates
        ),
        (
            before.0,
            before.1,
            before.2.contexts,
            before.2.key_copies,
            before.2.certificates
        )
    );
    assert_eq!(memo.counters.grades, counts.grades);
    assert_eq!(memo.counters.copies, counts.copies);
    assert_eq!(
        dlms_learning_lab::native_observations::json_admission_calls(),
        json
    );
    assert!(after.1 + META_RESERVE <= INDEX_BYTES);
}
#[test]
fn direct_broken_state_missed_hook_source_goal_and_foreign_warm_keys_fail_closed() {
    let s = fixture::source();
    let mut a = solved(&s);
    let index = Handle::default();
    publish(&index, &a, &s);
    let original = a.clone();
    let engine::State::PhysicalLink { simulation } = &mut a.state else {
        panic!()
    };
    let mut configuration = simulation.configuration().clone();
    configuration
        .ports
        .get_mut("assignment-left")
        .unwrap()
        .admin_up = false;
    simulation
        .perform(&core::Action::Configure { configuration })
        .unwrap();
    assert_eq!(index.status(&a, &s), Some(Status::Pending));
    assert!(!index.rekey_metadata(&a, &s).unwrap());
    publish(&index, &a, &s);
    assert_eq!(index.status(&a, &s), Some(Status::Incomplete));
    publish(&index, &original, &s);
    let before = index.observed();
    let mut foreign = s.clone();
    foreign.goal = pack::bundled().unwrap()[0].goal.clone();
    assert_eq!(index.status(&original, &foreign), Some(Status::Pending));
    assert!(index.admit(&original, &foreign, Family::Physical).is_err());
    let mut changed = s.clone();
    changed.hints.push("An additional reviewed hint".into());
    assert_eq!(index.status(&original, &changed), Some(Status::Pending));
    assert!(!index.rekey_metadata(&original, &changed).unwrap());
    let mut malformed = original.clone();
    let cron = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.cron_teaching.is_some())
        .unwrap();
    malformed.workspace.as_mut().unwrap().cron_learning = Some(
        dlms_learning_lab::cron_teaching::Learning::new(cron.cron_teaching.as_ref().unwrap())
            .unwrap(),
    );
    assert_eq!(index.status(&malformed, &s), Some(Status::Pending));
    assert!(index.admit(&malformed, &s, Family::Physical).is_err());
    assert_eq!(index.observed().2.key_copies, before.2.key_copies);
}
#[test]
fn only_validated_monotonic_visit_and_mode_changes_may_rekey_without_grading() {
    let mut s = fixture::source();
    s.steps.push("Review the current outcome".into());
    let a = solved(&s);
    let index = Handle::default();
    publish(&index, &a, &s);
    let mut metadata = a.clone();
    work::record_visit(&mut metadata, &s, 2).unwrap();
    work::record_mode(&mut metadata, &s, true).unwrap();
    assert!(index.rekey_metadata(&metadata, &s).unwrap());
    assert_eq!(index.status(&metadata, &s), Some(Status::Complete));
    for n in 0..6 {
        let mut other = metadata.clone();
        match n {
            0 => other.step = 1,
            1 => other.hints = 1,
            2 => other.transcript.push("changed history".into()),
            3 => other.workspace.as_mut().unwrap().checked = true,
            4 => {
                other
                    .workspace
                    .as_mut()
                    .unwrap()
                    .physical_link_draft
                    .as_mut()
                    .unwrap()
                    .selected_port = "assignment-right".into()
            }
            5 => other.visited = 0,
            _ => unreachable!(),
        };
        assert!(!index.rekey_metadata(&other, &s).unwrap());
        assert_eq!(index.status(&other, &s), Some(Status::Pending));
    }
    let before = index.observed().2;
    let json = dlms_learning_lab::native_observations::json_admission_calls();
    for _ in 0..80 {
        assert_eq!(index.status(&metadata, &s), Some(Status::Complete));
    }
    assert_eq!(index.observed().2.contexts, before.contexts);
    assert_eq!(index.observed().2.key_copies, before.key_copies);
    assert_eq!(
        dlms_learning_lab::native_observations::json_admission_calls(),
        json
    );
}
#[test]
fn selected_reset_restore_and_actual_source_replacement_discard_only_their_authority() {
    let s = fixture::source();
    let a = solved(&s);
    let index = Handle::default();
    publish(&index, &a, &s);
    let mut second = s.clone();
    second.id = "test-physical-second".into();
    let b = solved(&second);
    publish(&index, &b, &second);
    index.forget(&s.id);
    assert_eq!(index.status(&a, &s), Some(Status::Pending));
    assert_eq!(index.status(&b, &second), Some(Status::Complete));
    publish(&index, &a, &s);
    let mut source = s.clone();
    source.explanation.push_str(" Extra supporting reference.");
    assert_eq!(index.status(&a, &source), Some(Status::Pending));
    assert!(!index.rekey_metadata(&a, &source).unwrap());
    let generation = index.generation();
    index.clear();
    assert_eq!(index.generation(), generation + 1);
    assert_eq!(index.status(&a, &s), Some(Status::Pending));
    assert_eq!(index.status(&b, &second), Some(Status::Pending));
}
#[test]
fn index_key_bound_refuses_before_copy_and_refusals_do_not_idle_retry() {
    let s = fixture::source();
    let a = solved(&s);
    let index = Handle::default();
    publish(&index, &a, &s);
    let mut huge = a.clone();
    huge.transcript = vec!["x".repeat(32768); 120];
    let before = index.observed();
    let wire = serde_json::to_vec(&huge).unwrap();
    assert!(index.admit(&huge, &s, Family::Physical).is_err());
    assert_eq!(index.observed().2.key_copies, before.2.key_copies);
    assert_eq!(serde_json::to_vec(&huge).unwrap(), wire);
    index.refuse(&s.id);
    assert!(!index.can_queue(&s.id));
    index.forget(&s.id);
    assert!(index.can_queue(&s.id));
    let mut p = progress::Progress::default();
    p.attempts.insert(s.id.clone(), huge);
    index.page(std::iter::once(s.id.clone()));
    index.refuse(&s.id);
    let before = index.observed().2;
    let json = dlms_learning_lab::native_observations::json_admission_calls();
    for _ in 0..80 {
        assert!(index.pending(&p, std::slice::from_ref(&s)).is_none());
    }
    assert_eq!(index.observed().2.contexts, before.contexts);
    assert_eq!(index.observed().2.key_copies, before.key_copies);
    assert_eq!(
        dlms_learning_lab::native_observations::json_admission_calls(),
        json
    );
}
#[test]
fn selection_only_retains_requested_check_and_config_change_invalidates_both_scopes() {
    let s = fixture::source();
    let mut a = solved(&s);
    work::set_checked(&mut a, &s, work::Scope::Assignment, true).unwrap();
    let mut d = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_draft
        .clone()
        .unwrap();
    d.selected_link = "assignment-operations".into();
    d.selected_port = "assignment-ops-left".into();
    work::set_draft(&mut a, &s, work::Scope::Assignment, &d).unwrap();
    assert!(a.workspace.as_ref().unwrap().checked);
    assert!(work::checks(&a, &s).iter().all(|r| r.0));
    d.configuration
        .ports
        .get_mut("assignment-left")
        .unwrap()
        .admin_up = false;
    work::set_draft(&mut a, &s, work::Scope::Assignment, &d).unwrap();
    assert!(!a.workspace.as_ref().unwrap().checked);
    work::fill_example(&mut a, &s).unwrap();
    work::apply(&mut a, &s, work::Scope::Practice).unwrap();
    work::set_checked(&mut a, &s, work::Scope::Practice, true).unwrap();
    let mut d = a
        .workspace
        .as_ref()
        .unwrap()
        .physical_link_learning
        .as_ref()
        .unwrap()
        .draft
        .clone();
    d.selected_link = "example-operations".into();
    work::set_draft(&mut a, &s, work::Scope::Practice, &d).unwrap();
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .checked
    );
    d.configuration
        .ports
        .get_mut("example-left")
        .unwrap()
        .admin_up = false;
    work::set_draft(&mut a, &s, work::Scope::Practice, &d).unwrap();
    assert!(
        !a.workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .checked
    );
}
#[test]
fn legacy_binary_filters_remain_exact_and_pending_is_not_incomplete_or_complete() {
    let s = fixture::source();
    for status in ["", "Complete", "In progress", "Not started", "unrecognized"] {
        for started in [false, true] {
            for complete in [false, true] {
                let f = catalog::Filter {
                    status: status.into(),
                    ..Default::default()
                };
                assert_eq!(
                    catalog::matches(&s, &f, started, complete, false),
                    catalog::matches_verification(&s, &f, started, Some(complete), false)
                );
            }
        }
    }
    for status in ["Complete", "In progress"] {
        let f = catalog::Filter {
            status: status.into(),
            ..Default::default()
        };
        assert!(!catalog::matches_verification(&s, &f, true, None, false));
    }
    let f = catalog::Filter {
        status: "Verification pending".into(),
        ..Default::default()
    };
    assert!(catalog::matches_verification(&s, &f, true, None, false));
    assert!(!catalog::matches_verification(
        &s,
        &f,
        true,
        Some(false),
        false
    ));
    assert!(!catalog::matches_verification(
        &s,
        &f,
        true,
        Some(true),
        false
    ));
}

#[test]
fn lazy_page_queue_iterator_reenters_exact_status_before_bounded_visible_assignment() {
    let index = Handle::default();
    let mut sources = Vec::new();
    let mut attempts = Vec::new();
    for n in 0..20 {
        let mut source = fixture::source();
        source.id = format!("test-page-pending-{n}");
        let mut attempt = Attempt::new(&source, 1);
        work::prepare_attempt(&mut attempt, &source, true).unwrap();
        if n == 0 {
            attempt = solved(&source);
        }
        if n < 2 {
            publish(&index, &attempt, &source);
        }
        sources.push(source);
        attempts.push(attempt);
    }
    assert_eq!(
        index.status(&attempts[0], &sources[0]),
        Some(Status::Complete)
    );
    assert_eq!(
        index.status(&attempts[1], &sources[1]),
        Some(Status::Incomplete)
    );
    index.refuse(&sources[2].id);
    let previous = vec!["previous-visible".to_owned()];
    index.page(previous.clone().into_iter());
    let counts = index.observed().2;
    index.page(
        sources
            .iter()
            .zip(&attempts)
            .filter(|(s, a)| {
                // This is the actual library lazy closure shape. No RefCell guard may
                // be held by page while it calls can_queue/status on the same handle.
                assert_eq!(index.0.borrow().visible, previous);
                index.can_queue(&s.id) && index.status(a, s) == Some(Status::Pending)
            })
            .map(|(s, _)| s.id.clone()),
    );
    assert_eq!(
        index.0.borrow().visible,
        sources
            .iter()
            .skip(3)
            .take(12)
            .map(|s| s.id.clone())
            .collect::<Vec<_>>()
    );
    let after = index.observed().2;
    assert_eq!(
        (after.contexts, after.key_copies, after.certificates),
        (counts.contexts, counts.key_copies, counts.certificates)
    );
    // Preserve the exact previous take-before-length-filter semantics and cap.
    index.page((0..30).map(|n| {
        if n == 0 {
            "x".repeat(81)
        } else {
            format!("short-{n}")
        }
    }));
    assert_eq!(
        index.0.borrow().visible,
        (1..12).map(|n| format!("short-{n}")).collect::<Vec<_>>()
    );
}
