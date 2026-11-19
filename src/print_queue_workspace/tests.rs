use super::*;
use crate::print_queues::{Document, Job, Printer};
fn task() -> Task {
    let room = QueueName("Room".into());
    let labels = QueueName("Labels".into());
    Task {
        id: "print-capability-fixture".into(),
        facts: Facts {
            printers: vec![room.clone(), labels.clone()],
            documents: vec![
                Document {
                    path: "/work/brief.txt".into(),
                    title: "Room briefing".into(),
                    text: "A supplied briefing\n".into(),
                    pages: 2,
                },
                Document {
                    path: "/work/labels.txt".into(),
                    title: "Inventory labels".into(),
                    text: "Inventory labels\n".into(),
                    pages: 1,
                },
            ],
            operator: "learner".into(),
        },
        initial: World {
            printers: vec![
                Printer {
                    name: room.clone(),
                    accepting: false,
                    enabled: false,
                    rejection_reason: "Maintenance completed; verify acceptance".into(),
                    stopped_reason: "Paper replaced".into(),
                },
                Printer {
                    name: labels.clone(),
                    accepting: false,
                    enabled: true,
                    rejection_reason: "Reserved for stock".into(),
                    stopped_reason: String::new(),
                },
            ],
            jobs: vec![Job {
                id: 40,
                printer: labels.clone(),
                document: "/work/labels.txt".into(),
                copies: 1,
                state: JobState::Held,
            }],
            local_default: Some(room.clone()),
            next_id: 81,
        },
        required: Required {
            printers: vec![PrinterRequirement {
                printer: room.clone(),
                accepting: true,
                enabled: true,
            }],
            default: DefaultRequirement::Preserve {},
            existing_jobs: vec![],
            deliveries: vec![Delivery {
                document: Path("/work/brief.txt".into()),
                printer: room.clone(),
                copies: 2,
            }],
        },
        cases: vec![
            Case {
                label: "Next default request".into(),
                destination: None,
                document: Path("/work/brief.txt".into()),
                copies: 1,
                held: false,
                expected: Expected::Printed,
            },
            Case {
                label: "Different explicit document".into(),
                destination: Some(room.clone()),
                document: Path("/work/labels.txt".into()),
                copies: 1,
                held: false,
                expected: Expected::Printed,
            },
            Case {
                label: "Protected queue still rejects".into(),
                destination: Some(labels),
                document: Path("/work/labels.txt".into()),
                copies: 1,
                held: false,
                expected: Expected::Rejected,
            },
            Case {
                label: "Individual hold remains meaningful".into(),
                destination: Some(room),
                document: Path("/work/brief.txt".into()),
                copies: 2,
                held: true,
                expected: Expected::Held,
            },
        ],
        reference: ["cupsaccept Room", "cupsenable Room", "lp -n 2 brief.txt"]
            .into_iter()
            .map(|s| Command(s.into()))
            .collect(),
    }
}
fn run(t: &Task, commands: &[&str]) -> World {
    t.replay(
        &commands
            .iter()
            .map(|s| Command((*s).into()))
            .collect::<Vec<_>>(),
    )
    .unwrap()
}
fn solved(t: &Task, w: &World) -> bool {
    t.grade(w).iter().all(|c| c.correct)
}
#[test]
fn taught_and_legitimate_alternate_orders_and_copy_requests_pass_by_outcome() {
    let t = task();
    t.validate().unwrap();
    assert!(!solved(&t, &t.initial));
    for commands in [
        vec!["cupsaccept Room", "cupsenable Room", "lp -n 2 brief.txt"],
        vec![
            "cupsaccept Room",
            "lp brief.txt",
            "lp -d 'room' brief.txt",
            "cupsenable Room",
        ],
        vec![
            "cupsenable Room",
            "cupsaccept Room",
            "lp -d Room -n 2 /work/brief.txt",
        ],
    ] {
        let w = run(&t, &commands);
        assert!(solved(&t, &w), "{commands:?}: {:?}", t.grade(&w));
    }
}
#[test]
fn partial_repairs_different_documents_and_extra_held_work_never_pass() {
    let t = task();
    for commands in [
        vec!["cupsaccept Room", "lp -n 2 brief.txt"],
        vec!["cupsenable Room"],
        vec!["cupsaccept Room", "cupsenable Room", "lp -n 2 labels.txt"],
        vec![
            "cupsaccept Room",
            "cupsenable Room",
            "lp -n 2 brief.txt",
            "lp -H hold brief.txt",
        ],
    ] {
        let w = run(&t, &commands);
        let rows = t.grade(&w);
        assert!(rows.iter().any(|r| !r.correct));
        assert!(rows.iter().any(|r| r.correct));
        assert!(
            rows.iter()
                .filter(|r| !r.correct)
                .all(|r| !r.feedback.is_empty())
        );
    }
}
#[test]
fn successful_state_is_invalidated_by_stopping_rejecting_wrong_default_or_extra_prints() {
    let t = task();
    let complete = t.replay(&t.reference).unwrap();
    assert!(solved(&t, &complete));
    for raw in [
        "cupsdisable Room",
        "cupsreject Room",
        "lpoptions -d Labels",
        "lp labels.txt",
        "lp brief.txt",
    ] {
        let mut broken = complete.clone();
        command::run(&mut broken, &t.facts, raw).unwrap();
        assert!(!solved(&t, &broken), "{raw}");
    }
}
#[test]
fn actual_request_identity_and_protected_neighbors_cannot_be_replaced_by_new_jobs() {
    let mut t = task();
    t.initial.jobs.push(Job {
        id: 72,
        printer: QueueName("Room".into()),
        document: "/work/brief.txt".into(),
        copies: 2,
        state: JobState::Held,
    });
    t.required.existing_jobs = vec![ExistingJob {
        id: 72,
        state: JobState::Completed,
    }];
    t.reference = [
        "cupsaccept Room",
        "cupsenable Room",
        "lp -i Room-72 -H resume",
    ]
    .into_iter()
    .map(|s| Command(s.into()))
    .collect();
    t.validate().unwrap();
    let correct = t.replay(&t.reference).unwrap();
    assert!(solved(&t, &correct));
    let replacement = run(
        &t,
        &[
            "cupsaccept Room",
            "cupsenable Room",
            "cancel 72",
            "lp -n 2 brief.txt",
        ],
    );
    assert!(
        t.grade(&replacement)
            .iter()
            .any(|r| r.label.contains("72") && !r.correct)
    );
    let mut broken = correct;
    command::run(&mut broken, &t.facts, "cancel 40").unwrap();
    assert!(!solved(&t, &broken));
}
#[test]
fn separate_deterministic_cases_observe_current_configuration_without_changing_work() {
    let t = task();
    let current = t.replay(&t.reference).unwrap();
    let before = current.clone();
    let rows = t.grade(&current);
    assert_eq!(current, before);
    let cases = rows
        .iter()
        .filter(|r| r.label.starts_with("Fresh request case"))
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 4);
    assert!(
        cases
            .iter()
            .all(|r| r.correct && r.expected.is_some() && r.actual.is_some())
    );
    let mut broken = current;
    command::run(&mut broken, &t.facts, "cupsdisable Room").unwrap();
    let cases = t
        .grade(&broken)
        .into_iter()
        .filter(|r| r.label.starts_with("Fresh request case"))
        .collect::<Vec<_>>();
    assert!(cases.iter().any(|r| r.correct));
    assert!(cases.iter().any(|r| !r.correct));
}
#[test]
fn unbounded_foreign_duplicate_unreachable_and_solved_sources_are_rejected() {
    let valid = task();
    valid.validate().unwrap();
    let mut bad = valid.clone();
    bad.cases[1] = bad.cases[0].clone();
    assert!(bad.validate().is_err());
    let mut bad = valid.clone();
    bad.required.existing_jobs.push(ExistingJob {
        id: 999,
        state: JobState::Completed,
    });
    assert!(bad.validate().is_err());
    let mut bad = valid.clone();
    bad.cases[0].document.0 = "/etc/private-file".into();
    assert!(bad.validate().is_err());
    let mut bad = valid.clone();
    bad.reference = vec![Command("lpstat -t".into())];
    assert!(bad.validate().is_err());
    let mut bad = valid.clone();
    bad.initial = valid.replay(&valid.reference).unwrap();
    bad.required.deliveries.clear();
    assert!(bad.validate().is_err());
    let long = vec![Command("lpstat -d".into()); MAX_HISTORY + 1];
    assert!(valid.replay(&long).is_err());
    assert!(valid.replay(&[Command("echo 'completed'".into())]).is_err());
    let mut foreign = valid.initial.clone();
    foreign.printers[0].name.0 = "Other".into();
    assert!(!solved(&valid, &foreign));
}
#[test]
fn withdrawing_an_unfinished_mistake_is_valid_but_completed_wrong_output_cannot_be_erased() {
    let t = task();
    let repaired = run(
        &t,
        &[
            "cupsaccept Room",
            "lp -H hold labels.txt",
            "cancel 81",
            "cupsenable Room",
            "lp -n 2 brief.txt",
        ],
    );
    assert!(solved(&t, &repaired));
    let mut wrong = run(
        &t,
        &[
            "cupsaccept Room",
            "cupsenable Room",
            "lp labels.txt",
            "lp -n 2 brief.txt",
        ],
    );
    assert!(!solved(&t, &wrong));
    let before = wrong.clone();
    assert!(command::run(&mut wrong, &t.facts, "cancel 81").is_err());
    assert_eq!(wrong, before);
}

fn definition() -> Definition {
    let initial = task();
    let raw = serde_json::to_string(&initial)
        .unwrap()
        .replace("Room", "Workshop")
        .replace("Labels", "Supplies")
        .replace("brief.txt", "demo-note.txt")
        .replace("labels.txt", "demo-codes.txt");
    let mut example: Task = serde_json::from_str(&raw).unwrap();
    example.id = "print-example-fixture".into();
    let d = Definition { initial, example };
    d.validate().unwrap();
    d
}
fn execute(d: &mut session::Draft, t: &Task, raw: &str) {
    d.input = Command(raw.into());
    d.run(t).unwrap().unwrap();
}
fn complete(d: &mut session::Draft, t: &Task) {
    for c in &t.reference {
        execute(d, t, &c.0);
    }
    assert!(d.check(t).unwrap().iter().all(|r| r.correct));
}
#[test]
fn individual_request_cases_cannot_hide_an_unintended_completed_document() {
    let t = task();
    let mut world = t.replay(&t.reference).unwrap();
    command::run(&mut world, &t.facts, "lp labels.txt").unwrap();
    let rows = t.grade(&world);
    let cases = rows
        .iter()
        .filter(|r| r.label.starts_with("Fresh request case"))
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 4);
    assert!(cases.iter().all(|r| r.correct));
    assert!(
        rows.iter()
            .any(|r| r.label == "Additional requests" && !r.correct)
    );
    assert!(!solved(&t, &world));
}
#[test]
fn practice_completion_is_isolated_from_independent_assignment_state() {
    let d = definition();
    let mut s = Session::new(&d);
    s.view = crate::script_teaching::View::Example;
    let untouched = s.assignment.clone();
    complete(&mut s.practice, &d.example);
    assert!(s.practice.checked && !s.assignment.checked);
    assert_eq!(s.assignment, untouched);
    s.validate(&d).unwrap();
}
#[test]
fn resumed_session_preserves_success_and_a_separate_unfinished_command_without_cached_world() {
    let d = definition();
    let mut s = Session::new(&d);
    complete(&mut s.assignment, &d.initial);
    s.practice.input = Command("lp 'unfinished example".into());
    s.assignment.input = Command("lp -H hold".into());
    let wire = serde_json::to_value(&s).unwrap();
    assert!(wire["assignment"].get("world").is_none());
    assert!(wire["assignment"].get("output").is_none());
    let restored: Session = serde_json::from_value(wire).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored, s);
    assert!(restored.assignment.checked);
    assert_eq!(restored.practice.history.len(), 0);
}
#[test]
fn last_action_output_is_rederived_and_forged_provenance_refuses() {
    let t = task();
    let mut d = session::Draft::new(&t);
    execute(&mut d, &t, "lpstat -t");
    let original = d.last_output(&t).unwrap().unwrap().unwrap();
    assert!(original.stdout.contains("disabled"));
    let mut bad = d.clone();
    bad.last_run.as_mut().unwrap().appended = false;
    assert!(bad.validate(&t).is_err());
    let mut bad = d.clone();
    bad.last_run.as_mut().unwrap().command.0 = "echo 'done'".into();
    assert!(bad.validate(&t).is_err());
    let mut bad = d.clone();
    bad.history.clear();
    assert!(bad.validate(&t).is_err());
    let mut wire = serde_json::to_value(&d).unwrap();
    wire["last_run"]["stdout"] = serde_json::json!("complete");
    assert!(serde_json::from_value::<session::Draft>(wire).is_err());
    let restored: session::Draft =
        serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(
        restored.last_output(&t).unwrap().unwrap().unwrap(),
        original
    );
}
#[test]
fn syntax_unsupported_and_state_failure_are_replayable_without_changing_outcome() {
    let t = task();
    let mut d = session::Draft::new(&t);
    for (raw, kind) in [
        ("lp 'broken", crate::boot_entry::ErrorKind::Syntax),
        ("sh script.sh", crate::boot_entry::ErrorKind::Unsupported),
        ("lp brief.txt", crate::boot_entry::ErrorKind::Invalid),
    ] {
        d.input = Command(raw.into());
        let history = d.history.clone();
        let actual = d.run(&t).unwrap().unwrap_err();
        assert_eq!(actual.kind, kind);
        assert_eq!(d.history, history);
        assert_eq!(d.last_output(&t).unwrap().unwrap().unwrap_err(), actual);
        d.validate(&t).unwrap();
    }
    complete(&mut d, &t);
    d.input = Command("echo success".into());
    assert!(d.run(&t).unwrap().is_err());
    assert!(d.checked);
    d.validate(&t).unwrap();
}
#[test]
fn current_success_clears_after_a_state_change_and_only_returns_after_real_repair() {
    let t = task();
    let mut d = session::Draft::new(&t);
    complete(&mut d, &t);
    execute(&mut d, &t, "cupsdisable Room");
    assert!(!d.checked && !d.attempted);
    assert!(d.check(&t).unwrap().iter().any(|r| !r.correct));
    execute(&mut d, &t, "cupsenable Room");
    assert!(!d.checked);
    assert!(d.check(&t).unwrap().iter().all(|r| r.correct));
    execute(&mut d, &t, "lp labels.txt");
    assert!(!d.checked && !d.attempted);
    assert!(
        d.check(&t)
            .unwrap()
            .iter()
            .any(|r| r.label == "Additional requests" && !r.correct)
    );
    d.validate(&t).unwrap();
}
#[test]
fn forged_success_foreign_drafts_unknown_fields_and_future_versions_are_rejected() {
    let d = definition();
    let original = Session::new(&d);
    let mut bad = original.clone();
    bad.assignment.checked = true;
    bad.assignment.attempted = true;
    assert!(bad.validate(&d).is_err());
    let mut bad = original.clone();
    bad.assignment = bad.practice.clone();
    assert!(bad.validate(&d).is_err());
    let mut bad = original.clone();
    bad.version = 2;
    assert!(bad.validate(&d).is_err());
    let mut wire = serde_json::to_value(&original).unwrap();
    wire["assignment"]["world"] = serde_json::json!({"printed":true});
    assert!(serde_json::from_value::<Session>(wire).is_err());
    let mut bad = original.clone();
    bad.practice.input.0 = "x".repeat(1025);
    assert!(bad.validate(&d).is_err());
    assert_eq!(original, Session::new(&d));
}
#[test]
fn finite_history_refusal_preserves_state_and_can_be_restored_honestly() {
    let t = task();
    let mut d = session::Draft::new(&t);
    d.history = vec![Command("lpstat -d".into()); MAX_HISTORY];
    d.validate(&t).unwrap();
    let before = d.world(&t).unwrap();
    d.input = Command("cupsenable Room".into());
    let error = d.run(&t).unwrap().unwrap_err();
    assert_eq!(error.kind, crate::boot_entry::ErrorKind::Limit);
    assert_eq!(d.history.len(), MAX_HISTORY);
    assert_eq!(d.world(&t).unwrap(), before);
    assert_eq!(d.last_output(&t).unwrap().unwrap().unwrap_err(), error);
    let restored: session::Draft =
        serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    restored.validate(&t).unwrap();
    assert!(!restored.checked);
}

#[test]
fn bsd_alternate_submission_passes_behavior_grading_and_replays_as_ordinary_work() {
    let t = task();
    let world = run(
        &t,
        &[
            "cupsaccept Room",
            "lpr -#2 -PRoom brief.txt",
            "cupsenable Room",
        ],
    );
    assert!(solved(&t, &world));
    let mut draft = session::Draft::new(&t);
    for raw in [
        "cupsaccept Room",
        "lpr -P Room -# 2 brief.txt",
        "cupsenable Room",
    ] {
        execute(&mut draft, &t, raw);
    }
    assert!(draft.check(&t).unwrap().iter().all(|r| r.correct));
    let restored: session::Draft =
        serde_json::from_str(&serde_json::to_string(&draft).unwrap()).unwrap();
    restored.validate(&t).unwrap();
    assert_eq!(restored.world(&t).unwrap(), world);
}

fn scenario() -> crate::pack::Scenario {
    let mut d = definition();
    d.initial.id = "print-capability-binding-fixture".into();
    let mut solution = d
        .initial
        .reference
        .iter()
        .map(|c| c.0.clone())
        .collect::<Vec<_>>();
    solution.push("@print-check".into());
    let value = serde_json::json!({"schema_version":1,"scenarios":[{"id":"print-capability-binding-fixture","version":1,"track":"LPIC-1","topic":"Printing","title":"Repair a supplied print queue","summary":"Technical capability fixture, not registered authored curriculum.","objective":"Deliver the supplied briefing, repair queue acceptance/processing and preserve the protected queue and held request.","difficulty":"Intermediate","minutes":20,"steps":["Inspect the supplied queue and request states.","Repair the intended queue, submit the document and check the actual results."],"hints":["Accepting and processing are separate settings."],"sources":["https://www.lpi.org/our-certifications/exam-101-102-objectives/"],"explanation":"Technical engine/source binding only; native teaching and curriculum review are separate gates.","simplifications":"Fixed virtual files, an authorized fictional operator and finite immediate scheduling. No host printing, daemon, network or device.","initial":{"kind":"PrintQueues","simulation":Session::new(&d)},"goal":{"kind":"PrintQueues","definition":d},"solution":solution,"prerequisites":[]}]});
    crate::pack::load(&serde_json::to_string(&value).unwrap())
        .unwrap()
        .remove(0)
}
fn attempt_session(a: &mut crate::progress::Attempt) -> &mut Session {
    let crate::engine::State::PrintQueues { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn authored_pack_engine_and_current_progress_require_actual_independent_work() {
    use crate::{engine, progress};
    let s = scenario();
    let engine::Goal::PrintQueues { definition } = &s.goal else {
        panic!()
    };
    let mut a = progress::Attempt::new(&s, 1);
    complete(&mut attempt_session(&mut a).practice, &definition.example);
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "@print-check").unwrap();
    assert!(engine::checks(&a.state, &s.goal).iter().any(|r| !r.0));
    for raw in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "cupsdisable Room").unwrap();
    assert!(!progress::current_solved(&a, &s));
    let before = a.state.clone();
    assert!(engine::execute(&mut a.state, "@print-check").is_err());
    assert_eq!(a.state, before);
}
#[test]
fn malformed_pair_initial_state_unknown_helpers_and_host_commands_refuse_atomically() {
    use crate::{engine, progress};
    let s = scenario();
    let mut a = progress::Attempt::new(&s, 1);
    for raw in [
        "sh script.sh",
        "cat /etc/passwd",
        "lp -h real-host brief.txt",
        "lpr -r brief.txt",
        "@print-check extra",
        "print-action {\"printed\":true}",
    ] {
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, raw).is_err(),
            "{raw}"
        );
        assert_eq!(a.state, before);
    }
    let mut bad = s.clone();
    bad.goal = engine::Goal::Subnet;
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    let engine::State::PrintQueues { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.assignment.input.0 = "invented initial input".into();
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    let engine::Goal::PrintQueues { definition } = &mut bad.goal else {
        panic!()
    };
    definition.initial.reference = vec![Command("lpstat -t".into())];
    assert!(source(&bad).is_err());
}
#[test]
fn scoped_save_restore_preserves_two_unfinished_drafts_and_original_attempts() {
    use crate::{engine, progress};
    let s = scenario();
    let mut catalog = crate::pack::bundled().unwrap();
    catalog.push(s.clone());
    let mut p = progress::Progress {
        schema_version: 1,
        ..progress::Progress::default()
    };
    for old in catalog.iter().filter(|v| v.id != s.id) {
        p.attempts
            .insert(old.id.clone(), progress::Attempt::new(old, 1));
    }
    let preserved = serde_json::to_value(&p.attempts).unwrap();
    let mut a = progress::Attempt::new(&s, 1);
    attempt_session(&mut a).assignment.input.0 = "lp -n ".into();
    attempt_session(&mut a).practice.input.0 = "lpr '-#' 2".into();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    let dir = std::env::temp_dir().join(format!(
        "print-binding-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(
        dir.starts_with(
            std::env::current_dir()
                .unwrap()
                .join(".build-artifacts/tmp")
        )
    );
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("progress.json");
    progress::save_with_catalog(&file, &p, &catalog).unwrap();
    let before = std::fs::read(&file).unwrap();
    let mut loaded = progress::load(&file, &catalog).unwrap();
    assert!(loaded.favorites.contains(&s.id));
    let current = loaded.attempts.remove(&s.id).unwrap();
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), preserved);
    assert_eq!(
        serde_json::to_value(&current).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap()
    );
    assert!(progress::save(&file, &p).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
    let mut bad: progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    bad.attempts.get_mut(&s.id).unwrap().scenario_version = 2;
    assert!(progress::save_with_catalog(&file, &bad, &catalog).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
    let mut bad: progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    let attempt = bad.attempts.get_mut(&s.id).unwrap();
    attempt_session(attempt).assignment.checked = true;
    attempt_session(attempt).assignment.attempted = true;
    assert!(!progress::current_solved(attempt, &s));
    assert!(progress::save_with_catalog(&file, &bad, &catalog).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
    let mut wrong = current;
    wrong.state = engine::State::PrintQueues {
        simulation: Box::new(Session::new(&definition())),
    };
    attempt_session(&mut wrong).assignment.task_id = "other-task".into();
    assert!(!progress::current_solved(&wrong, &s));
}
