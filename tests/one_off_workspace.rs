//! Synthetic development fixtures only; never bundled or learner progress.
use dlms_learning_lab::{cron::Moment, one_off as core, one_off_workspace as work};
use std::collections::{BTreeMap, BTreeSet};
fn time(s: &str) -> Moment {
    Moment::parse(s).unwrap()
}
fn initial() -> core::State {
    core::State {
        version: 1,
        clock: time("2026-01-06 10:00"),
        next_id: 1,
        users: BTreeSet::from(["root".into(), "alex".into()]),
        files: BTreeMap::from([
            (
                "/work/job.sh".into(),
                "cat /work/input.txt > /work/output.txt\n".into(),
            ),
            ("/work/input.txt".into(), "live first\n".into()),
            ("/work/keep.txt".into(), "preserve\n".into()),
        ]),
        jobs: BTreeMap::new(),
        records: vec![],
    }
}
fn selector() -> work::Selector {
    work::Selector {
        owner: "alex".into(),
        script_path: "/work/job.sh".into(),
        timing: work::Timing::AfterSubmission { minutes: 5 },
    }
}
fn definition() -> work::Definition {
    let initial = initial();
    let mut probes = Vec::new();
    for (name, input) in [
        ("Original live input", "live first\n"),
        ("Different live input", "second sample\n"),
    ] {
        let mut files = initial.files.clone();
        files.insert("/work/input.txt".into(), input.into());
        let mut after = files.clone();
        after.insert("/work/output.txt".into(), input.into());
        probes.push(work::Probe {
            name: name.into(),
            source: work::CaptureSource::LastRun,
            job: selector(),
            window: work::ProbeWindow::AfterSubmission {
                start_minutes: 0,
                end_minutes: 6,
            },
            files,
            expected: Some(work::ExpectedEvent {
                owner: "alex".into(),
                when: work::ExpectedWhen::Dispatch,
                status: core::Status::Completed,
                exit_status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                error: None,
            }),
            after,
        });
    }
    probes.push(work::Probe {
        name: "Before the due minute".into(),
        source: work::CaptureSource::LastRun,
        job: selector(),
        window: work::ProbeWindow::AfterSubmission {
            start_minutes: 0,
            end_minutes: 4,
        },
        files: initial.files.clone(),
        expected: None,
        after: initial.files.clone(),
    });
    work::Definition {
        initial,
        editable: vec!["/work/job.sh".into(), "/work/input.txt".into()],
        outputs: vec!["/work/output.txt".into()],
        preserve_jobs: vec![],
        requirements: vec![
            work::Requirement::QueueExactly {
                source: work::CaptureSource::Queued,
                jobs: vec![],
            },
            work::Requirement::QueueExactly {
                source: work::CaptureSource::LastRun,
                jobs: vec![selector()],
            },
            work::Requirement::CurrentFile {
                path: "/work/output.txt".into(),
                text: "live first\n".into(),
            },
            work::Requirement::Dispatched { job: selector() },
        ],
        probes,
    }
}
fn codec(a: &mut work::Assignment, d: &work::Definition, v: serde_json::Value) {
    work::execute(d, a, &format!("one-off {v}")).unwrap();
}
fn submit(a: &mut work::Assignment, d: &work::Definition) {
    codec(
        a,
        d,
        serde_json::json!({"kind":"Command","actor":"alex","environment":{},"text":"at -f /work/job.sh now + 5 minutes"}),
    );
}
fn run(a: &mut work::Assignment, d: &work::Definition) {
    codec(
        a,
        d,
        serde_json::json!({"kind":"Run","end":time("2026-01-06 10:06")}),
    );
}
#[test]
fn actual_core_queue_capture_run_and_two_live_file_cases_recompute_current_success() {
    let d = definition();
    d.validate().unwrap();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    run(&mut a, &d);
    assert!(d.checks(&a).iter().all(|r| r.0));
    let before = a.clone();
    a.simulation
        .files
        .insert("/work/output.txt".into(), "broken\n".into());
    assert!(d.checks(&a).iter().any(|r| !r.0));
    a = before;
    assert!(d.checks(&a).iter().all(|r| r.0));
    assert_eq!(work::Assignment::new(&d.initial).unwrap().last_run, None);
}
#[test]
fn equivalent_source_is_accepted_and_constant_first_input_is_not() {
    let d = definition();
    for source in [
        "cat < /work/input.txt > /work/output.txt\n",
        "cat '/work/input.txt' > '/work/output.txt'\n",
    ] {
        let mut a = work::Assignment::new(&d.initial).unwrap();
        codec(
            &mut a,
            &d,
            serde_json::json!({"kind":"Write","path":"/work/job.sh","text":source}),
        );
        submit(&mut a, &d);
        run(&mut a, &d);
        assert!(d.checks(&a).iter().all(|r| r.0));
    }
    let mut a = work::Assignment::new(&d.initial).unwrap();
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/job.sh","text":"printf 'live first\\n' > /work/output.txt\n"}),
    );
    submit(&mut a, &d);
    run(&mut a, &d);
    let rows = d.checks(&a);
    assert!(rows.iter().any(|r| !r.0));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.starts_with("Different live input:"))
    );
}
#[test]
fn records_without_transition_evidence_cannot_award_lifecycle_credit() {
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    run(&mut a, &d);
    assert!(d.checks(&a).iter().all(|r| r.0));
    let records = a.simulation.records.clone();
    a.last_run = None;
    assert_eq!(a.simulation.records, records);
    assert!(d.checks(&a).iter().any(|r| !r.0));
}
#[test]
fn source_is_a_snapshot_but_files_are_read_at_dispatch() {
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/job.sh","text":"printf wrong > /work/output.txt\n"}),
    );
    run(&mut a, &d);
    assert_eq!(a.simulation.files["/work/output.txt"], "live first\n");
    assert!(d.checks(&a).iter().all(|r| r.0));
    let mut b = work::Assignment::new(&d.initial).unwrap();
    submit(&mut b, &d);
    codec(
        &mut b,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/input.txt","text":"changed before dispatch\n"}),
    );
    run(&mut b, &d);
    assert_eq!(
        b.simulation.files["/work/output.txt"],
        "changed before dispatch\n"
    );
    assert!(d.checks(&b).iter().any(|r| !r.0));
}
#[test]
fn forged_proof_changed_owner_time_and_damaged_unrelated_files_fail() {
    let d = definition();
    let mut good = work::Assignment::new(&d.initial).unwrap();
    submit(&mut good, &d);
    run(&mut good, &d);
    let mut broken = good.clone();
    broken
        .last_run
        .as_mut()
        .unwrap()
        .before
        .jobs
        .get_mut(&1)
        .unwrap()
        .owner = "root".into();
    assert!(d.checks(&broken).iter().any(|r| !r.0));
    let mut broken = good.clone();
    broken.last_run.as_mut().unwrap().end = time("2026-01-06 10:07");
    assert!(d.checks(&broken).iter().any(|r| !r.0));
    let mut broken = good;
    broken
        .simulation
        .files
        .insert("/work/keep.txt".into(), "changed\n".into());
    assert!(d.checks(&broken).iter().any(|r| !r.0));
}
#[test]
fn bounded_context_decode_rejects_unknown_fields_duplicate_cases_and_invalid_variables() {
    let d = definition();
    let mut value = serde_json::to_value(&d).unwrap();
    value["executable"] = true.into();
    assert!(work::Definition::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut bad = d.clone();
    bad.probes[1].name = bad.probes[0].name.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.requirements
        .push(work::Requirement::CapturedEnvironment {
            source: work::CaptureSource::Queued,
            job: selector(),
            values: BTreeMap::from([("OPTERR".into(), "1".into())]),
        });
    assert!(bad.validate().is_err());
    assert!(work::Definition::from_json(&vec![b' '; work::MAX_CONTEXT + 1]).is_err());
}
#[test]
fn check_stop_is_bounded_and_never_publishes_probe_files() {
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    run(&mut a, &d);
    let before = a.clone();
    let mut check = work::Check::new(&d, &a).unwrap();
    check.tick(1);
    check.cancel();
    assert!(check.done());
    assert!(check.rows.iter().any(|r| !r.0));
    assert_eq!(a, before);
}

#[test]
fn captured_environment_survives_later_queue_inspection_with_different_variables() {
    let mut d = definition();
    let source = "printf '%s\\n' \"$LABEL\" > /work/output.txt\n";
    d.initial.files.insert("/work/job.sh".into(), source.into());
    for p in &mut d.probes {
        if p.expected.is_some() {
            p.after
                .insert("/work/output.txt".into(), "captured\n".into());
        }
    }
    for r in &mut d.requirements {
        if let work::Requirement::CurrentFile { text, .. } = r {
            *text = "captured\n".into();
        }
    }
    d.requirements.push(work::Requirement::CapturedEnvironment {
        source: work::CaptureSource::LastRun,
        job: selector(),
        values: BTreeMap::from([("LABEL".into(), "captured".into())]),
    });
    let mut a = work::Assignment::new(&d.initial).unwrap();
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Command","actor":"alex","environment":{"LABEL":"captured"},"text":"at -f /work/job.sh now + 5 minutes"}),
    );
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Command","actor":"alex","environment":{"LABEL":"changed later"},"text":"atq"}),
    );
    assert_eq!(a.simulation.jobs[&1].environment["LABEL"], "captured");
    run(&mut a, &d);
    assert_eq!(a.simulation.files["/work/output.txt"], "captured\n");
    assert!(d.checks(&a).iter().all(|r| r.0));
}

fn scenario() -> dlms_learning_lab::pack::Scenario {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/starter-v1.json")).unwrap();
    let mut value = pack["scenarios"][0].clone();
    let object = value.as_object_mut().unwrap();
    object.retain(|k, _| k != "teaching" && !k.ends_with("_teaching"));
    object.insert("id".into(), "synthetic-one-off-binding".into());
    object.insert("unit".into(), "".into());
    object.insert("track".into(), "LPIC-1".into());
    object.insert("topic".into(), "Scheduling".into());
    object.insert("title".into(), "Synthetic one-off binding fixture".into());
    object.insert(
        "steps".into(),
        serde_json::json!(["Use the virtual queue and explicit clock"]),
    );
    object.insert("guide".into(), serde_json::json!([]));
    object.insert("prerequisites".into(), serde_json::json!([]));
    let d = definition();
    object.insert(
        "initial".into(),
        serde_json::to_value(dlms_learning_lab::engine::State::OneOff {
            simulation: Box::new(work::Assignment::new(&d.initial).unwrap()),
        })
        .unwrap(),
    );
    object.insert(
        "goal".into(),
        serde_json::to_value(dlms_learning_lab::engine::Goal::OneOff {
            definition: Box::new(d),
        })
        .unwrap(),
    );
    object.insert("solution".into(),serde_json::json!([
        format!("one-off {}",serde_json::json!({"kind":"Command","actor":"alex","environment":{},"text":"at -f /work/job.sh now + 5 minutes"})),
        format!("one-off {}",serde_json::json!({"kind":"Run","end":time("2026-01-06 10:06")}))
    ]));
    serde_json::from_value(value).unwrap()
}
fn attempt(s: &dlms_learning_lab::pack::Scenario) -> dlms_learning_lab::progress::Attempt {
    let mut a = dlms_learning_lab::progress::Attempt::new(s, 42);
    work::budget::prepare(&mut a, s).unwrap();
    a
}
fn submit_attempt(
    a: &mut dlms_learning_lab::progress::Attempt,
    s: &dlms_learning_lab::pack::Scenario,
) {
    work::budget::apply_attempt(
        a,
        s,
        &work::Action::Command {
            text: "at -f /work/job.sh now + 5 minutes".into(),
        },
        false,
    )
    .unwrap();
}
fn completed_attempt(
    s: &dlms_learning_lab::pack::Scenario,
) -> dlms_learning_lab::progress::Attempt {
    let mut a = attempt(s);
    submit_attempt(&mut a, s);
    let mut run =
        work::budget::PendingAttempt::begin(&a, s, time("2026-01-06 10:06"), false).unwrap();
    for _ in 0..4096 {
        if run.done() {
            break;
        }
        run.tick(8)
    }
    assert!(run.done());
    run.finish(&mut a, s).unwrap();
    a
}
#[test]
fn synthetic_pack_reachability_uses_real_typed_reference_actions_and_keeps_normal_inventory() {
    let s = scenario();
    let raw = serde_json::to_string(&dlms_learning_lab::pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    let loaded = dlms_learning_lab::pack::load(&raw).unwrap();
    assert_eq!(loaded.len(), 1);
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/starter-v1.json")).unwrap();
    assert!(
        original["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["id"] != "synthetic-one-off-binding")
    );
}
#[test]
fn whole_attempt_prepare_projection_matches_owned_wire_and_preserves_retained_drafts() {
    let s = scenario();
    let mut a = dlms_learning_lab::progress::Attempt::new(&s, 42);
    let before = serde_json::to_value(&a).unwrap();
    work::budget::prepare(&mut a, &s).unwrap();
    let expected = serde_json::to_value(&a).unwrap();
    assert_eq!(
        work::budget::fresh_size(&s, 42),
        Some(serde_json::to_vec(&a).unwrap().len())
    );
    assert_eq!(a.visited, 42);
    assert_eq!(a.state, s.initial);
    let dlms_learning_lab::engine::State::OneOff { simulation } = &a.state else {
        panic!()
    };
    let w = a.workspace.as_ref().unwrap();
    let projected = work::budget::projected(
        &a,
        simulation,
        Some(
            dlms_learning_lab::cron_workspace::size(
                w.one_off_session.as_ref().unwrap(),
                work::MAX_ATTEMPT,
            )
            .unwrap(),
        ),
        None,
    )
    .unwrap();
    assert_eq!(projected, serde_json::to_vec(&a).unwrap().len());
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::Edit {
            path: "/work/job.sh".into(),
            text: "# Retained unfinished script\n".into(),
        },
        false,
    )
    .unwrap();
    let retained = serde_json::to_value(&a).unwrap();
    work::budget::prepare(&mut a, &s).unwrap();
    assert_eq!(serde_json::to_value(&a).unwrap(), retained);
    assert_ne!(retained, expected);
    assert!(before["workspace"].is_null());
}
#[test]
fn real_pending_run_stop_and_source_changes_never_publish_candidate_or_replace_saved_work() {
    let s = scenario();
    let mut a = attempt(&s);
    submit_attempt(&mut a, &s);
    let before = serde_json::to_value(&a).unwrap();
    let mut run =
        work::budget::PendingAttempt::begin(&a, &s, time("2026-01-06 10:06"), false).unwrap();
    run.tick(1);
    assert!(!run.done());
    run.cancel();
    assert!(run.done());
    assert!(run.finish(&mut a, &s).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
    let mut run =
        work::budget::PendingAttempt::begin(&a, &s, time("2026-01-06 10:06"), false).unwrap();
    for _ in 0..4096 {
        if run.done() {
            break;
        }
        run.tick(8)
    }
    assert!(run.done());
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::Edit {
            path: "/work/input.txt".into(),
            text: "New pending input\n".into(),
        },
        false,
    )
    .unwrap();
    let changed = serde_json::to_value(&a).unwrap();
    assert!(run.finish(&mut a, &s).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), changed);
}
#[test]
fn current_completion_requires_saved_drafts_and_recomputed_transition_not_cached_checked_flag() {
    let s = scenario();
    let mut a = completed_attempt(&s);
    assert!(dlms_learning_lab::progress::current_solved(&a, &s));
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::Edit {
            path: "/work/job.sh".into(),
            text: "printf 'wrong'\n".into(),
        },
        false,
    )
    .unwrap();
    assert!(!dlms_learning_lab::progress::current_solved(&a, &s));
    a.workspace.as_mut().unwrap().checked = true;
    assert!(!dlms_learning_lab::progress::current_solved(&a, &s));
}
#[test]
fn malformed_whole_attempt_refuses_edits_and_run_before_publication_without_resetting_drafts() {
    let s = scenario();
    let mut a = attempt(&s);
    a.transcript.push("x".repeat(work::MAX_ATTEMPT));
    let before = serde_json::to_value(&a).unwrap();
    assert!(
        work::budget::apply_attempt(
            &mut a,
            &s,
            &work::Action::Edit {
                path: "/work/job.sh".into(),
                text: "printf hello\n".into()
            },
            false
        )
        .is_err()
    );
    assert!(work::budget::PendingAttempt::begin(&a, &s, time("2026-01-06 10:06"), false).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
}
#[test]
fn full_progress_save_decode_and_selected_reset_keep_others_favorites_recency_and_receipts() {
    use dlms_learning_lab::progress::{self, Progress};
    let s = scenario();
    let a = completed_attempt(&s);
    let mut other = s.clone();
    other.id = "synthetic-other-one-off".into();
    let mut other_a = attempt(&other);
    work::budget::apply_attempt(
        &mut other_a,
        &other,
        &work::Action::Edit {
            path: "/work/job.sh".into(),
            text: "# Retain another unfinished draft\n".into(),
        },
        false,
    )
    .unwrap();
    let other_before = serde_json::to_value(&other_a).unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 42,
        favorites: BTreeSet::from([s.id.clone()]),
        attempts: BTreeMap::from([(s.id.clone(), a), (other.id.clone(), other_a)]),
        migration_receipts: None,
    };
    let portable = serde_json::to_value(&p).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::decode(&raw, &[s.clone(), other.clone()]).unwrap();
    assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
    assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    let receipts = serde_json::to_value(&p.migration_receipts).unwrap();
    let clock = p.clock;
    let favorites = p.favorites.clone();
    let visited = p.attempts[&s.id].visited;
    work::budget::reset_attempt(p.attempts.get_mut(&s.id).unwrap(), &s).unwrap();
    assert_eq!(
        serde_json::to_value(&p.attempts[&other.id]).unwrap(),
        other_before
    );
    assert_eq!(p.clock, clock);
    assert_eq!(p.favorites, favorites);
    assert_eq!(
        serde_json::to_value(&p.migration_receipts).unwrap(),
        receipts
    );
    assert_eq!(p.attempts[&s.id].visited, visited);
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
}
#[test]
fn reciprocal_source_and_typed_metadata_are_refused_without_mutating_other_workspace() {
    let s = scenario();
    let a = attempt(&s);
    assert!(work::context(&a, &s).is_ok());
    let mut poisoned = a.clone();
    poisoned.workspace.as_mut().unwrap().script_learning =
        Some(dlms_learning_lab::script_teaching::Learning {
            version: 1,
            view: dlms_learning_lab::script_teaching::View::Introduction,
            example_source: "printf hello".into(),
            example_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            example_result: None,
        });
    let before = serde_json::to_value(&poisoned).unwrap();
    assert!(work::context(&poisoned, &s).is_err());
    assert!(!dlms_learning_lab::progress::current_solved(&poisoned, &s));
    assert_eq!(serde_json::to_value(&poisoned).unwrap(), before);
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/starter-v1.json")).unwrap();
    let other: dlms_learning_lab::pack::Scenario =
        serde_json::from_value(original["scenarios"][0].clone()).unwrap();
    let mut other_a = dlms_learning_lab::progress::Attempt::new(&other, 8);
    dlms_learning_lab::workspace::prepare(&mut other_a);
    other_a.workspace.as_mut().unwrap().one_off_session =
        a.workspace.as_ref().unwrap().one_off_session.clone();
    let before = serde_json::to_value(&other_a).unwrap();
    assert!(work::metadata(&other_a).is_err());
    assert!(!dlms_learning_lab::progress::current_solved(
        &other_a, &other
    ));
    dlms_learning_lab::workspace::prepare(&mut other_a);
    assert_eq!(serde_json::to_value(&other_a).unwrap(), before);
}

fn lesson() -> dlms_learning_lab::one_off_teaching::Lesson {
    use dlms_learning_lab::one_off_teaching::{AnnotatedAction, Concept, Lesson};
    let mut d: work::Definition = serde_json::from_str(
        &serde_json::to_string(&definition())
            .unwrap()
            .replace("/work/", "/work/example-")
            .replace("\"alex\"", "\"sam\"")
            .replace("live first", "example one")
            .replace("second sample", "example two"),
    )
    .unwrap();
    let mut wanted = selector();
    wanted.owner = "sam".into();
    wanted.script_path = "/work/example-job.sh".into();
    for p in &mut d.probes {
        p.source = work::CaptureSource::Queued;
    }
    d.requirements = vec![work::Requirement::QueueExactly {
        source: work::CaptureSource::Queued,
        jobs: vec![wanted],
    }];
    let command = |text: &str| work::Action::Command { text: text.into() };
    Lesson{prerequisite_note:"You can inspect a supplied virtual text file; the example script is already supplied.".into(),concepts:vec![Concept{title:"A queued copy".into(),text:"Submitting captures this script and its variable values. Editing the original file later does not rewrite the queued copy.".into()},Concept{title:"An explicit clock".into(),text:"The clock changes only when you choose a simulated window. Due times here use UTC, independently of the Fedora desktop clock.".into()}],example_description:"Queue Sam's separate example job for five minutes from now. Check uses other sample inputs without changing this queue.".into(),definition:d,example_actions:vec![AnnotatedAction{title:"Choose Sam".into(),explanation:"The selected simulated account owns its new queued work.".into(),action:work::Action::Actor{actor:"sam".into()}},AnnotatedAction{title:"Submit the supplied script".into(),explanation:"The five-minute delay belongs to this example, independently of the assignment.".into(),action:command("at -f /work/example-job.sh now + 5 minutes")}],try_change:"Remove the example job and queue it for six minutes; inspect the due-time feedback.".into(),try_actions:vec![command("atrm 1"),command("at -f /work/example-job.sh now + 6 minutes")],try_rows:vec![true,true,false,true,false,false,false],repair:vec![command("atrm 2"),command("at -f /work/example-job.sh now + 5 minutes")]}
}
#[test]
fn separate_example_experiment_repair_and_views_preserve_the_full_assignment_and_do_not_earn_credit()
 {
    let mut s = scenario();
    let l = lesson();
    l.validate().unwrap();
    assert!(l.separate_from(&definition()));
    s.one_off_teaching = Some(l.clone());
    let mut a = attempt(&s);
    assert!(!dlms_learning_lab::progress::current_solved(&a, &s));
    let original_state = serde_json::to_value(&a.state).unwrap();
    let original_session =
        serde_json::to_value(&a.workspace.as_ref().unwrap().one_off_session).unwrap();
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .assignment
            .simulation
            .jobs
            .is_empty()
    );
    work::budget::set_view(
        &mut a,
        &s,
        dlms_learning_lab::script_teaching::View::Example,
        0,
    )
    .unwrap();
    for action in l.example_actions.iter().map(|r| &r.action) {
        work::budget::apply_attempt(&mut a, &s, action, true).unwrap();
    }
    let learning = a
        .workspace
        .as_ref()
        .unwrap()
        .one_off_learning
        .as_ref()
        .unwrap();
    assert!(
        work::checks_with_session(
            &learning.definition,
            &learning.assignment,
            &learning.session
        )
        .iter()
        .all(|r| r.0)
    );
    for action in &l.try_actions {
        work::budget::apply_attempt(&mut a, &s, action, true).unwrap();
    }
    let learning = a
        .workspace
        .as_ref()
        .unwrap()
        .one_off_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        work::checks_with_session(
            &learning.definition,
            &learning.assignment,
            &learning.session
        )
        .iter()
        .map(|r| r.0)
        .collect::<Vec<_>>(),
        l.try_rows
    );
    for action in &l.repair {
        work::budget::apply_attempt(&mut a, &s, action, true).unwrap();
    }
    let learning = a
        .workspace
        .as_ref()
        .unwrap()
        .one_off_learning
        .as_ref()
        .unwrap();
    assert!(
        work::checks_with_session(
            &learning.definition,
            &learning.assignment,
            &learning.session
        )
        .iter()
        .all(|r| r.0)
    );
    assert_eq!(serde_json::to_value(&a.state).unwrap(), original_state);
    assert_eq!(
        serde_json::to_value(&a.workspace.as_ref().unwrap().one_off_session).unwrap(),
        original_session
    );
    assert!(!dlms_learning_lab::progress::current_solved(&a, &s));
    let retained = serde_json::to_value(&a.workspace.as_ref().unwrap().one_off_learning).unwrap();
    work::budget::prepare(&mut a, &s).unwrap();
    assert_eq!(
        serde_json::to_value(&a.workspace.as_ref().unwrap().one_off_learning).unwrap(),
        retained
    );
}
#[test]
fn missing_editable_file_is_repairable_instead_of_panicking_during_preparation() {
    let s = scenario();
    let mut a = dlms_learning_lab::progress::Attempt::new(&s, 42);
    let dlms_learning_lab::engine::State::OneOff { simulation } = &mut a.state else {
        panic!()
    };
    simulation.simulation.files.remove("/work/job.sh");
    work::budget::prepare(&mut a, &s).unwrap();
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .one_off_session
            .as_ref()
            .unwrap()
            .drafts["/work/job.sh"],
        ""
    );
    assert!(!dlms_learning_lab::progress::current_solved(&a, &s));
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::Edit {
            path: "/work/job.sh".into(),
            text: definition().initial.files["/work/job.sh"].clone(),
        },
        false,
    )
    .unwrap();
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::Save {
            path: "/work/job.sh".into(),
        },
        false,
    )
    .unwrap();
    assert!(work::context(&a, &s).is_ok());
}

#[test]
fn submission_relative_policy_accepts_repaired_retry_without_resetting_the_clock_or_work() {
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/job.sh","text":"printf wrong > /work/output.txt\n"}),
    );
    submit(&mut a, &d);
    run(&mut a, &d);
    assert!(d.checks(&a).iter().any(|r| !r.0));
    assert_eq!(a.simulation.clock, time("2026-01-06 10:06"));
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/job.sh","text":d.initial.files["/work/job.sh"]}),
    );
    submit(&mut a, &d);
    assert_eq!(a.simulation.jobs[&2].submitted, time("2026-01-06 10:06"));
    assert_eq!(a.simulation.jobs[&2].due, time("2026-01-06 10:11"));
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Run","end":time("2026-01-06 10:12")}),
    );
    assert!(d.checks(&a).iter().all(|r| r.0));
    assert_eq!(a.simulation.next_id, 3);
    assert_eq!(a.simulation.records.len(), 2);
    assert_eq!(a.simulation.clock, time("2026-01-06 10:12"));
}
#[test]
fn absolute_past_due_job_uses_actual_core_asap_semantics_without_fake_timeline() {
    let mut d = definition();
    let mut wanted = selector();
    wanted.timing = work::Timing::Absolute {
        submitted: d.initial.clock,
        due: time("2026-01-06 09:55"),
    };
    for r in &mut d.requirements {
        match r {
            work::Requirement::QueueExactly {
                source: work::CaptureSource::LastRun,
                jobs,
            } => *jobs = vec![wanted.clone()],
            work::Requirement::Dispatched { job } => *job = wanted.clone(),
            _ => {}
        }
    }
    d.probes.retain(|p| p.expected.is_some());
    for p in &mut d.probes {
        p.job = wanted.clone();
        p.window = work::ProbeWindow::Absolute {
            start: d.initial.clock,
            end: time("2026-01-06 10:01"),
        };
        p.expected.as_mut().unwrap().when = work::ExpectedWhen::Absolute {
            at: d.initial.clock,
        };
    }
    d.validate().unwrap();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Command","actor":"alex","environment":{},"text":"at -f /work/job.sh -t 202601060955"}),
    );
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Run","end":time("2026-01-06 10:01")}),
    );
    assert_eq!(a.simulation.records[0].when, time("2026-01-06 10:00"));
    assert!(d.checks(&a).iter().all(|r| r.0));
}
#[test]
fn relative_windows_cross_leap_day_and_reject_zero_span_or_excessive_offsets() {
    let mut d = definition();
    d.initial.clock = time("2028-02-28 23:58");
    d.validate().unwrap();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    assert_eq!(a.simulation.jobs[&1].due, time("2028-02-29 00:03"));
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Run","end":time("2028-02-29 00:04")}),
    );
    assert!(d.checks(&a).iter().all(|r| r.0));
    for window in [
        work::ProbeWindow::AfterSubmission {
            start_minutes: 0,
            end_minutes: 0,
        },
        work::ProbeWindow::AfterSubmission {
            start_minutes: u32::MAX,
            end_minutes: u32::MAX,
        },
    ] {
        let mut bad = d.clone();
        bad.probes[0].window = window;
        assert!(bad.validate().is_err());
    }
    let mut bad = definition();
    bad.initial.clock = time("2099-12-31 23:58");
    assert!(bad.validate().is_err());
}
#[test]
fn unsupported_script_is_preflighted_before_any_prefix_file_effects() {
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/job.sh","text":"printf changed > /work/keep.txt\ntouch /work/created.txt\n"}),
    );
    submit(&mut a, &d);
    let files = a.simulation.files.clone();
    run(&mut a, &d);
    assert_eq!(a.simulation.files, files);
    assert_eq!(a.simulation.records[0].status, core::Status::Failed);
    assert_eq!(a.simulation.records[0].exit_status, Some(2));
    assert!(d.checks(&a).iter().any(|r| !r.0));
}

#[test]
fn malformed_keys_and_excess_map_entry_refuse_before_poisoned_values_are_decoded() {
    let d = definition();
    let a = work::Assignment::new(&d.initial).unwrap();
    let s = work::Session::from_state(&d, &a).unwrap();
    let raw = serde_json::to_string(&s).unwrap().replace(
        "\"environment\":{}",
        "\"environment\":{\"OPTERR\":POISON_MUST_NOT_BE_READ}",
    );
    let error = serde_json::from_str::<work::Session>(&raw)
        .unwrap_err()
        .to_string();
    assert!(error.contains("key before value"), "{error}");
    let mut value = serde_json::to_value(&s).unwrap();
    value["drafts"] = serde_json::json!({});
    let raw = serde_json::to_string(&value).unwrap().replace(
        "\"drafts\":{}",
        "\"drafts\":{\"/etc/passwd\":POISON_MUST_NOT_BE_READ}",
    );
    let error = serde_json::from_str::<work::Session>(&raw)
        .unwrap_err()
        .to_string();
    assert!(error.contains("key before value"), "{error}");
    let fields = (0..16)
        .map(|n| format!("\"V{n}\":\"ok\""))
        .collect::<Vec<_>>()
        .join(",");
    let raw = serde_json::to_string(&s).unwrap().replace(
        "\"environment\":{}",
        &format!("\"environment\":{{{fields},\"EXCESS\":POISON_MUST_NOT_BE_READ}}"),
    );
    let error = serde_json::from_str::<work::Session>(&raw)
        .unwrap_err()
        .to_string();
    assert!(error.contains("map count limit"), "{error}");
}

#[test]
fn malformed_example_context_cannot_panic_fresh_projection_or_pack_loading() {
    let mut s = scenario();
    let mut l = lesson();
    l.definition.editable.clear();
    s.one_off_teaching = Some(l);
    assert!(work::budget::fresh_size(&s, 0).is_none());
    let raw = serde_json::to_string(&dlms_learning_lab::pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    assert!(dlms_learning_lab::pack::load(&raw).is_err());
}

#[test]
fn overlapping_absolute_relative_selectors_cannot_hide_an_unwanted_extra_job() {
    let mut d = definition();
    let mut alias = selector();
    alias.timing = work::Timing::Absolute {
        submitted: d.initial.clock,
        due: time("2026-01-06 10:05"),
    };
    for r in &mut d.requirements {
        if let work::Requirement::QueueExactly {
            source: work::CaptureSource::LastRun,
            jobs,
        } = r
        {
            jobs.push(alias.clone());
        }
    }
    let mut cases = d
        .probes
        .iter()
        .filter(|p| p.expected.is_some())
        .cloned()
        .collect::<Vec<_>>();
    for p in &mut cases {
        p.job = alias.clone();
        p.name.push_str(" · absolute binding");
    }
    d.probes.extend(cases);
    // Each new last-run alias must also declare its own completed-window binding.
    // This satisfies the unchanged validator before the bad-queue outcome is tested.
    d.requirements
        .push(work::Requirement::Dispatched { job: alias });
    d.validate().unwrap();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Command","actor":"root","environment":{},"text":"at -f /work/job.sh now + 5 minutes"}),
    );
    run(&mut a, &d);
    let rows = d.checks(&a);
    assert!(
        rows.iter().any(|r| !r.0 && r.1.contains("queue snapshot")),
        "Two selectors matching one Alex job must not count the extra root job"
    );
}

#[test]
fn harmless_consecutive_clock_advance_keeps_bounded_replay_completion_and_noop_save_keeps_evidence()
{
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    run(&mut a, &d);
    let original = a.last_run.as_ref().unwrap().before.clone();
    assert!(d.checks(&a).iter().all(|r| r.0));
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Run","end":time("2026-01-06 10:07")}),
    );
    assert_eq!(a.last_run.as_ref().unwrap().before, original);
    assert_eq!(a.last_run.as_ref().unwrap().end, time("2026-01-06 10:07"));
    assert!(d.checks(&a).iter().all(|r| r.0));
    codec(
        &mut a,
        &d,
        serde_json::json!({"kind":"Write","path":"/work/job.sh","text":d.initial.files["/work/job.sh"]}),
    );
    assert!(a.last_run.is_some());
    assert!(d.checks(&a).iter().all(|r| r.0));
}

#[test]
fn command_insertion_and_invalid_time_drafts_are_inert_and_persist_separately_from_execution() {
    let s = scenario();
    let mut a = attempt(&s);
    let before = serde_json::to_value(&a.state).unwrap();
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::CommandDraft {
            text: "at -f /work/job.sh now + 5 minutes".into(),
        },
        false,
    )
    .unwrap();
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::WindowDraft {
            text: "not a date".into(),
        },
        false,
    )
    .unwrap();
    assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
    let retained = serde_json::to_value(&a).unwrap();
    assert!(work::budget::PendingAttempt::from_draft(&a, &s, false).is_err());
    assert_eq!(serde_json::to_value(&a).unwrap(), retained);
    assert!(!dlms_learning_lab::progress::current_solved(&a, &s));
    work::budget::apply_attempt(
        &mut a,
        &s,
        &work::Action::WindowDraft {
            text: "2026-01-06 10:06".into(),
        },
        false,
    )
    .unwrap();
    let command = a
        .workspace
        .as_ref()
        .unwrap()
        .one_off_session
        .as_ref()
        .unwrap()
        .command
        .clone();
    work::budget::apply_attempt(&mut a, &s, &work::Action::Command { text: command }, false)
        .unwrap();
    let mut run = work::budget::PendingAttempt::from_draft(&a, &s, false).unwrap();
    for _ in 0..4096 {
        if run.done() {
            break;
        }
        run.tick(8)
    }
    assert!(run.done());
    run.finish(&mut a, &s).unwrap();
    assert!(dlms_learning_lab::progress::current_solved(&a, &s));
}

#[test]
fn exceeding_evidence_span_refuses_without_erasing_completed_work_or_history() {
    let d = definition();
    let mut a = work::Assignment::new(&d.initial).unwrap();
    submit(&mut a, &d);
    run(&mut a, &d);
    let before = a.clone();
    let input = format!(
        "one-off {}",
        serde_json::json!({"kind":"Run","end":time("2026-01-13 10:06")})
    );
    assert!(work::execute(&d, &mut a, &input).is_err());
    assert_eq!(a, before);
    assert!(d.checks(&a).iter().all(|r| r.0));
}

// R2 adds admission regressions; all original27 prospective groups stay exact.
fn decode_error<T: for<'de> serde::Deserialize<'de>>(raw: &str, needle: &str) {
    let error = serde_json::from_str::<T>(raw)
        .err()
        .expect("Must refuse malformed bounded input")
        .to_string();
    assert!(error.contains(needle), "{error}");
}
#[test]
fn new_tagged_actions_and_requirements_reject_invalid_keys_before_poisoned_values() {
    for raw in [
        r#"{"kind":"Environment","values":{"OPTERR": POISON_MUST_NOT_BE_READ"#,
        r#"{"values":{"PATH": POISON_MUST_NOT_BE_READ"#,
    ] {
        decode_error::<work::Action>(raw, "Invalid/duplicate one-off key before value");
    }
    for raw in [
        r#"{"kind":"CapturedEnvironment","values":{"LC_ALL": POISON_MUST_NOT_BE_READ"#,
        r#"{"values":{"9BAD": POISON_MUST_NOT_BE_READ"#,
    ] {
        decode_error::<work::Requirement>(raw, "Invalid/duplicate one-off key before value");
    }
    decode_error::<work::Action>(
        r#"{"kind":"Run","unknown": POISON_MUST_NOT_BE_READ"#,
        "Unknown one-off field before value",
    );
    decode_error::<work::Action>(
        r#"{"kind":"Run","kind": POISON_MUST_NOT_BE_READ"#,
        "Duplicate one-off tag before value",
    );
    decode_error::<work::Requirement>(
        r#"{"kind":"Clock","values": POISON_MUST_NOT_BE_READ"#,
        "Incompatible one-off field before value",
    );
}
#[test]
fn tagged_sequence_and_environment_n_plus_one_refuse_poison_before_its_payload() {
    let fields = (0..16)
        .map(|n| format!("\"V{n}\":\"x\""))
        .collect::<Vec<_>>()
        .join(",");
    decode_error::<work::Action>(
        &format!(
            "{{\"kind\":\"Environment\",\"values\":{{{fields},\"EXCESS\":POISON_MUST_NOT_BE_READ"
        ),
        "One-off map count limit before excess key/value",
    );
    let row = serde_json::to_string(&selector()).unwrap();
    let rows = vec![row; 16].join(",");
    decode_error::<work::Requirement>(
        &format!(
            "{{\"kind\":\"QueueExactly\",\"source\":\"Queued\",\"jobs\":[{rows},POISON_MUST_NOT_BE_READ"
        ),
        "Scheduling item limit",
    );
}
#[test]
fn scalar_tagged_timing_window_and_event_time_reject_unknown_duplicate_and_wrong_variant_fields() {
    decode_error::<work::Timing>(
        r#"{"kind":"AfterSubmission","minutes":5,"minutes":POISON_MUST_NOT_BE_READ"#,
        "Duplicate one-off field before value",
    );
    decode_error::<work::Timing>(
        r#"{"kind":"AfterSubmission","due":POISON_MUST_NOT_BE_READ"#,
        "Incompatible one-off field before value",
    );
    decode_error::<work::ProbeWindow>(
        r#"{"kind":"Absolute","unknown":POISON_MUST_NOT_BE_READ"#,
        "Unknown one-off field before value",
    );
    decode_error::<work::ExpectedWhen>(
        r#"{"kind":"Dispatch","at":POISON_MUST_NOT_BE_READ"#,
        "Incompatible one-off field before value",
    );
    decode_error::<work::ExpectedWhen>(
        r#"{"kind":"Dispatch","kind":POISON_MUST_NOT_BE_READ"#,
        "Duplicate one-off tag before value",
    );
}
#[test]
fn canonical_serialization_and_tag_anywhere_roundtrips_match_unchanged_wire_shapes() {
    let actions = [
        serde_json::json!({"kind":"Environment","values":{"MESSAGE":"literal value"}}),
        serde_json::json!({"kind":"Edit","path":"/work/job.sh","text":"echo ready\n"}),
        serde_json::json!({"kind":"Command","text":"atq"}),
        serde_json::json!({"kind":"Run","end":time("2026-01-06 10:06")}),
    ];
    for value in actions {
        let action: work::Action = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(&action).unwrap(), value);
        let mut fields = value
            .as_object()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.as_str() != "kind")
            .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), v))
            .collect::<Vec<_>>();
        fields.push(format!("\"kind\":{}", value["kind"]));
        let reversed: work::Action =
            serde_json::from_str(&format!("{{{}}}", fields.join(","))).unwrap();
        assert_eq!(action, reversed);
    }
    let d = definition();
    let wire = serde_json::to_vec(&d).unwrap();
    assert_eq!(work::Definition::from_json(&wire).unwrap(), d);
    for raw in [
        r#"{"minutes":5,"kind":"AfterSubmission"}"#,
        r#"{"kind":"AfterSubmission","minutes":5}"#,
    ] {
        let t: work::Timing = serde_json::from_str(raw).unwrap();
        assert_eq!(t, work::Timing::AfterSubmission { minutes: 5 });
        assert_eq!(
            serde_json::to_string(&t).unwrap(),
            r#"{"kind":"AfterSubmission","minutes":5}"#
        );
    }
    for raw in [
        r#"{"start_minutes":0,"end_minutes":6,"kind":"AfterSubmission"}"#,
        r#"{"kind":"AfterSubmission","start_minutes":0,"end_minutes":6}"#,
    ] {
        assert_eq!(
            serde_json::from_str::<work::ProbeWindow>(raw).unwrap(),
            work::ProbeWindow::AfterSubmission {
                start_minutes: 0,
                end_minutes: 6
            }
        );
    }
    assert_eq!(
        serde_json::from_str::<work::ExpectedWhen>(r#"{"kind":"Dispatch"}"#).unwrap(),
        work::ExpectedWhen::Dispatch
    );
    for tag_first in [true, false] {
        let text = "x".repeat(2049);
        let raw = if tag_first {
            format!("{{\"kind\":\"Command\",\"text\":\"{text}\"}}")
        } else {
            format!("{{\"text\":\"{text}\",\"kind\":\"Command\"}}")
        };
        assert!(work::Action::from_json(&raw).is_err());
    }
    assert!(
        work::Action::from_json(&" ".repeat(work::MAX_ACTION + 1))
            .unwrap_err()
            .contains("before decoding")
    );
}
#[test]
fn nested_definition_and_lesson_codecs_keep_member_guards_before_malformed_tails() {
    let mut value = serde_json::to_value(definition()).unwrap();
    value["requirements"] = serde_json::json!([{"kind":"CapturedEnvironment","source":"LastRun","job":selector(),"values":{"MESSAGE":"UNIQUE_POISON_SLOT"}}]);
    let raw = serde_json::to_string(&value).unwrap().replace(
        "\"MESSAGE\":\"UNIQUE_POISON_SLOT\"",
        "\"OPTERR\":POISON_MUST_NOT_BE_READ",
    );
    decode_error::<work::Definition>(&raw, "Invalid/duplicate one-off key before value");
    assert!(work::Definition::from_json(raw.as_bytes()).is_err());
    let mut l = serde_json::to_value(lesson()).unwrap();
    l["example_actions"][0]["action"] =
        serde_json::json!({"kind":"Environment","values":{"MESSAGE":"UNIQUE_POISON_SLOT"}});
    let raw = serde_json::to_string(&l).unwrap().replace(
        "\"MESSAGE\":\"UNIQUE_POISON_SLOT\"",
        "\"OPTERR\":POISON_MUST_NOT_BE_READ",
    );
    decode_error::<dlms_learning_lab::one_off_teaching::Lesson>(
        &raw,
        "Invalid/duplicate one-off key before value",
    );
    assert!(dlms_learning_lab::one_off_teaching::Lesson::from_json(raw.as_bytes()).is_err());
    let l = lesson();
    assert_eq!(
        dlms_learning_lab::one_off_teaching::Lesson::from_json(&serde_json::to_vec(&l).unwrap())
            .unwrap(),
        l
    );
    assert!(
        dlms_learning_lab::one_off_teaching::Lesson::from_json(&vec![b' '; work::MAX_CONTEXT + 1])
            .unwrap_err()
            .contains("before decoding")
    );
}
#[test]
fn own_direct_api_and_save_reject_out_of_range_navigation_and_transcript_counts_atomically() {
    use dlms_learning_lab::progress;
    let s = scenario();
    let original = completed_attempt(&s);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("admission.json");
    for mutation in ["step", "hints", "transcript"] {
        let mut a = original.clone();
        match mutation {
            "step" => a.step = s.steps.len(),
            "hints" => a.hints = s.hints.len() + 1,
            "transcript" => a.transcript = vec!["retained".into(); 121],
            _ => unreachable!(),
        }
        let before = serde_json::to_value(&a).unwrap();
        assert!(!progress::current_solved(&a, &s), "{mutation}");
        assert!(work::budget::prepare(&mut a, &s).is_err());
        assert!(
            work::budget::apply_attempt(
                &mut a,
                &s,
                &work::Action::Command { text: "atq".into() },
                false
            )
            .is_err()
        );
        assert!(
            work::budget::PendingAttempt::begin(&a, &s, time("2026-01-06 10:07"), false).is_err()
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        let p = progress::Progress {
            schema_version: 1,
            clock: 1,
            favorites: BTreeSet::new(),
            attempts: BTreeMap::from([(s.id.clone(), a)]),
            migration_receipts: None,
        };
        if mutation == "transcript" {
            assert!(progress::save(&path, &p).is_err());
            assert!(!path.exists());
        } else {
            // Save has no Scenario argument; decoder owns these source-aware bounds.
            progress::save(&path, &p).unwrap();
            assert!(
                progress::decode(&std::fs::read(&path).unwrap(), std::slice::from_ref(&s)).is_err()
            );
            std::fs::remove_file(&path).unwrap();
        }
    }
    let mut a = original;
    a.step = s.steps.len() - 1;
    a.hints = s.hints.len();
    a.transcript = vec![String::new(); 120];
    assert!(work::context(&a, &s).is_ok());
    let p = progress::Progress {
        schema_version: 1,
        clock: 1,
        favorites: BTreeSet::new(),
        attempts: BTreeMap::from([(s.id.clone(), a)]),
        migration_receipts: None,
    };
    progress::save(&path, &p).unwrap();
    assert!(progress::decode(&std::fs::read(&path).unwrap(), &[s]).is_ok());
}
#[test]
fn actual_normal_save_decode_receipts_survive_selected_reset_in_full_and_on_disk() {
    use dlms_learning_lab::progress::{self, Progress};
    let s = scenario();
    let mut other = s.clone();
    other.id = "synthetic-receipt-peer".into();
    let other_a = attempt(&other);
    let p = Progress {
        schema_version: 1,
        clock: 77,
        favorites: BTreeSet::from([s.id.clone(), other.id.clone()]),
        attempts: BTreeMap::from([
            (s.id.clone(), completed_attempt(&s)),
            (other.id.clone(), other_a),
        ]),
        migration_receipts: None,
    };
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("receipts.json");
    progress::save(&path, &p).unwrap();
    assert!(
        p.migration_receipts.is_none(),
        "Normal save does not mutate its caller"
    );
    let raw = std::fs::read(&path).unwrap();
    let mut loaded = progress::decode(&raw, &[s.clone(), other.clone()]).unwrap();
    let receipt = loaded
        .migration_receipts
        .as_ref()
        .expect("Actually decoded normal writer receipts");
    assert!(receipt.workspace);
    assert!(!receipt.shell.is_empty());
    let receipts = serde_json::to_value(&loaded.migration_receipts).unwrap();
    let before = serde_json::to_value(&loaded).unwrap();
    let visit = loaded.attempts[&s.id].visited;
    work::budget::reset_attempt(loaded.attempts.get_mut(&s.id).unwrap(), &s).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.migration_receipts).unwrap(),
        receipts
    );
    assert_eq!(loaded.clock, 77);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
        before["attempts"][&other.id]
    );
    assert_eq!(
        serde_json::to_value(&loaded.favorites).unwrap(),
        before["favorites"]
    );
    assert_eq!(loaded.attempts[&s.id].visited, visit);
    assert_eq!(
        std::fs::read(&path).unwrap(),
        raw,
        "Reset cannot rewrite original disk bytes by itself"
    );
    let fresh = attempt(&s);
    let mut expected = serde_json::to_value(&fresh).unwrap();
    expected["visited"] = serde_json::json!(visit);
    assert_eq!(
        serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
        expected
    );
    progress::save(&path, &loaded).unwrap();
    let restored = progress::decode(&std::fs::read(&path).unwrap(), &[s, other]).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&restored.migration_receipts).unwrap(),
        receipts
    );
}
#[test]
fn nested_scalar_variants_and_large_members_refuse_before_later_poison() {
    let d = serde_json::to_string(&definition()).unwrap();
    for (needle, replacement) in [
        (
            r#""timing":{"kind":"AfterSubmission","minutes":5}"#,
            r#""timing":{"kind":"AfterSubmission","due":POISON_MUST_NOT_BE_READ"#,
        ),
        (
            r#""window":{"kind":"AfterSubmission","start_minutes":0,"end_minutes":6}"#,
            r#""window":{"kind":"AfterSubmission","start":POISON_MUST_NOT_BE_READ"#,
        ),
        (
            r#""when":{"kind":"Dispatch"}"#,
            r#""when":{"kind":"Dispatch","at":POISON_MUST_NOT_BE_READ"#,
        ),
    ] {
        assert!(
            d.contains(needle),
            "The exact new scalar route must actually be exercised: {needle}"
        );
        let raw = d.replacen(needle, replacement, 1);
        decode_error::<work::Definition>(&raw, "Incompatible one-off field before value");
    }
    let oversized = "x".repeat(98305);
    let raw = format!(
        "{{\"kind\":\"CurrentFile\",\"path\":\"/work/out.txt\",\"text\":\"{oversized}\",\"later\":POISON_MUST_NOT_BE_READ"
    );
    decode_error::<work::Requirement>(&raw, "Scheduling UTF-8 text byte limit");
    let text = "x".repeat(33);
    for raw in [
        format!("{{\"kind\":\"WindowDraft\",\"text\":\"{text}\",\"later\":POISON_MUST_NOT_BE_READ"),
        format!("{{\"text\":\"{text}\",\"kind\":\"WindowDraft\",\"later\":POISON_MUST_NOT_BE_READ"),
    ] {
        let error = serde_json::from_str::<work::Action>(&raw)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("text byte limit") || error.contains("variant text limit"),
            "{error}"
        );
    }
}

#[test]
fn queued_jobs_and_completed_replay_roundtrip_through_state_goal_and_separate_practice() {
    use dlms_learning_lab::{
        engine::{Goal, State},
        progress::Attempt,
    };
    let mut s = scenario();
    s.one_off_teaching = Some(lesson());
    let mut a = attempt(&s);
    submit_attempt(&mut a, &s);
    for action in &s.one_off_teaching.as_ref().unwrap().example_actions {
        work::budget::apply_attempt(&mut a, &s, &action.action, true).unwrap();
    }
    let State::OneOff { simulation } = &a.state else {
        panic!()
    };
    assert_eq!(simulation.simulation.jobs.len(), 1);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap()
            .assignment
            .simulation
            .jobs
            .len(),
        1
    );
    for completed in [false, true] {
        if completed {
            let mut pending =
                work::budget::PendingAttempt::begin(&a, &s, time("2026-01-06 10:06"), false)
                    .unwrap();
            for _ in 0..4096 {
                if pending.done() {
                    break;
                }
                pending.tick(8);
            }
            assert!(pending.done());
            pending.finish(&mut a, &s).unwrap();
            let State::OneOff { simulation } = &a.state else {
                panic!()
            };
            assert!(simulation.simulation.jobs.is_empty());
            assert_eq!(simulation.last_run.as_ref().unwrap().before.jobs.len(), 1);
        }
        let raw = serde_json::to_vec(&a).unwrap();
        let parsed: Attempt = serde_json::from_slice(&raw).unwrap();
        assert_eq!(serde_json::to_vec(&parsed).unwrap(), raw);
        let parsed: Attempt = serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&parsed).unwrap(),
            serde_json::to_value(&a).unwrap()
        );
        assert!(work::context(&parsed, &s).is_ok());
    }
    let mut d = definition();
    d.initial
        .submit(
            "alex",
            "/work/job.sh",
            time("2026-01-06 10:05"),
            &BTreeMap::new(),
        )
        .unwrap();
    d.validate().unwrap();
    let goal = Goal::OneOff {
        definition: Box::new(d),
    };
    let raw = serde_json::to_vec(&goal).unwrap();
    let parsed: Goal = serde_json::from_slice(&raw).unwrap();
    assert_eq!(serde_json::to_vec(&parsed).unwrap(), raw);
    let parsed: Goal = serde_json::from_value(serde_json::to_value(&goal).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&parsed).unwrap(),
        serde_json::to_value(&goal).unwrap()
    );
}
