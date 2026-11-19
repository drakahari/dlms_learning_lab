use super::*;
use crate::localization::Variable;
fn command(text: &str) -> Action {
    Action::Command { text: text.into() }
}
fn report_task(id: &str) -> Task {
    let available = vec![Locale::C, Locale::Cutf8, Locale::EnUsUtf8, Locale::DeDeUtf8];
    let mut process = localization::configuration("LANG=de_DE.UTF-8\n", &available).unwrap();
    process.set(Variable::All, Some(Locale::C));
    let mut cases = vec![];
    for (scope, suffix) in [(Scope::Current, "current"), (Scope::NewLogin, "new")] {
        for (name, expected) in [
            ("a", "Bestand\nArtikel;Betrag\nCafé;12.50\nSpare;-0.75\n"),
            ("b", "Bestand\nArtikel;Betrag\nMüller;0.00\nLamp;10125.00\n"),
        ] {
            cases.push(Case::Report {
                id: format!("{suffix}-{name}"),
                scope,
                input: format!("/work/report-{name}.csv"),
                expected: Bytes(expected.as_bytes().to_vec()),
            });
        }
    }
    Task {
        id: id.into(),
        facts: Facts {
            available,
            inputs: vec![
                Input {
                    path: "/work/report-a.csv".into(),
                    label: "First report input".into(),
                    bytes: "item;cents\nCafé;1250\nSpare;-75\n".as_bytes().to_vec(),
                },
                Input {
                    path: "/work/report-b.csv".into(),
                    label: "Different report input".into(),
                    bytes: "item;cents\nMüller;0\nLamp;1012500\n".as_bytes().to_vec(),
                },
            ],
        },
        initial: Snapshot {
            saved_locale: "LANG=de_DE.UTF-8\n".into(),
            process,
        },
        cases,
        reference: vec![
            Action::SaveLocale {
                text: "LANG=de_DE.UTF-8\nLC_NUMERIC=C\n".into(),
            },
            Action::NewSession {},
            command("locale charmap"),
        ],
    }
}
fn conversion_task() -> Task {
    Task {
        id: "conversion-fixture".into(),
        facts: Facts {
            available: vec![Locale::C, Locale::Cutf8],
            inputs: vec![
                Input {
                    path: "/work/legacy.txt".into(),
                    label: "Latin-1 import".into(),
                    bytes: b"Caf\xe9;ni\xf1o\n".to_vec(),
                },
                Input {
                    path: "/work/plain.txt".into(),
                    label: "ASCII import".into(),
                    bytes: b"plain;0\n".to_vec(),
                },
                Input {
                    path: "/work/unicode.txt".into(),
                    label: "Existing UTF-8".into(),
                    bytes: "東京;🙂\n".as_bytes().to_vec(),
                },
            ],
        },
        initial: Snapshot {
            saved_locale: "LANG=C.UTF-8\n".into(),
            process: Environment::default(),
        },
        cases: vec![
            Case::File {
                id: "legacy".into(),
                path: "/work/fixed-legacy.txt".into(),
                expected: Bytes("Café;niño\n".as_bytes().to_vec()),
            },
            Case::File {
                id: "plain".into(),
                path: "/work/fixed-plain.txt".into(),
                expected: Bytes(b"plain;0\n".to_vec()),
            },
            Case::File {
                id: "unicode".into(),
                path: "/work/fixed-unicode.txt".into(),
                expected: Bytes("東京;🙂\n".as_bytes().to_vec()),
            },
        ],
        reference: vec![
            command("iconv -f LATIN1 -t UTF8 -o fixed-legacy.txt legacy.txt"),
            command("iconv -f ASCII -t UTF8 -o fixed-plain.txt plain.txt"),
            command("iconv -f UTF8 -t UTF8 -o fixed-unicode.txt unicode.txt"),
        ],
    }
}
#[test]
fn report_and_conversion_tasks_have_unsolved_starters_and_reachable_actual_methods() {
    for task in [report_task("report-fixture"), conversion_task()] {
        task.validate().unwrap();
        let initial = task.replay(&[]).unwrap();
        assert!(task.checks(&initial).iter().any(|r| !r.correct));
        let solved = task.replay(&task.reference).unwrap();
        assert!(task.checks(&solved).iter().all(|r| r.correct));
        let raw = serde_json::to_string(&task).unwrap();
        let restored: Task = serde_json::from_str(&raw).unwrap();
        assert_eq!(restored, task);
    }
}
#[test]
fn equivalent_category_policy_and_current_session_commands_are_valid_alternatives() {
    let task = report_task("report-fixture");
    let history = vec![
        Action::SaveLocale {
            text: "LANG=C.UTF-8\nLC_MESSAGES=de_DE.UTF-8\n".into(),
        },
        command("unset LC_ALL"),
        command("export LC_CTYPE=C.UTF-8 LC_NUMERIC=en_US.UTF-8"),
    ];
    let world = task.replay(&history).unwrap();
    assert!(task.checks(&world).iter().all(|r| r.correct));
    assert_eq!(
        world.saved_locale(),
        "LANG=C.UTF-8\nLC_MESSAGES=de_DE.UTF-8\n"
    );
}
#[test]
fn saved_defaults_and_current_session_cannot_hide_each_others_failures() {
    let task = report_task("report-fixture");
    for history in [
        vec![Action::SaveLocale {
            text: "LANG=de_DE.UTF-8\nLC_NUMERIC=C\n".into(),
        }],
        vec![command("unset LC_ALL"), command("export LC_NUMERIC=C")],
    ] {
        let checks = task.checks(&task.replay(&history).unwrap());
        assert_eq!(checks.iter().filter(|r| r.correct).count(), 2);
        assert_eq!(checks.iter().filter(|r| !r.correct).count(), 2);
        assert!(
            checks
                .iter()
                .filter(|r| !r.correct)
                .all(|r| r.feedback.contains("report") || r.feedback.contains("Report"))
        );
    }
}
#[test]
fn invalid_saved_settings_remain_inspectable_without_mutating_the_live_environment() {
    let task = report_task("report-fixture");
    let mut world = task.replay(&task.reference).unwrap();
    let live = world.process().clone();
    world
        .apply(&Action::SaveLocale {
            text: "LC_ALL=C\n".into(),
        })
        .unwrap();
    assert_eq!(world.process(), &live);
    let saved = world.clone();
    assert!(world.apply(&Action::NewSession {}).is_err());
    assert_eq!(world, saved);
    let checks = task.checks(&world);
    assert_eq!(checks.iter().filter(|r| r.correct).count(), 2);
    assert!(
        checks
            .iter()
            .filter(|r| !r.correct)
            .all(|r| r.feedback.contains("LC_ALL"))
    );
}
#[test]
fn broken_success_is_recomputed_for_reports_and_actual_output_files() {
    let report = report_task("report-fixture");
    let mut world = report.replay(&report.reference).unwrap();
    assert!(report.checks(&world).iter().all(|r| r.correct));
    world.apply(&command("export LC_ALL=C")).unwrap();
    assert_eq!(
        report.checks(&world).iter().filter(|r| r.correct).count(),
        2
    );
    let task = conversion_task();
    let mut world = task.replay(&task.reference).unwrap();
    let out = world
        .apply(&command(
            "iconv -f LATIN1 -t ASCII -o fixed-legacy.txt legacy.txt",
        ))
        .unwrap();
    assert_eq!(out.status, 1);
    let rows = task.checks(&world);
    assert_eq!(rows.iter().filter(|r| !r.correct).count(), 1);
    assert!(rows[0].feedback.contains("partial file"));
    assert_eq!(world.files().read("fixed-legacy.txt").unwrap(), b"Caf");
}
#[test]
fn viewing_or_double_encoding_cannot_replace_correct_saved_outputs() {
    let task = conversion_task();
    let world = task
        .replay(&[command("iconv -f LATIN1 -t UTF8 legacy.txt")])
        .unwrap();
    assert!(task.checks(&world).iter().all(|r| !r.correct));
    assert!(
        task.checks(&world)[0]
            .feedback
            .contains("No output was saved")
    );
    let mut world = task.replay(&task.reference).unwrap();
    world
        .apply(&command(
            "iconv -f LATIN1 -t UTF8 -o fixed-unicode.txt unicode.txt",
        ))
        .unwrap();
    let row = task.checks(&world).pop().unwrap();
    assert!(!row.correct);
    assert!(std::str::from_utf8(row.actual.as_ref().unwrap()).is_ok());
    assert!(row.feedback.contains("valid UTF-8"));
}
#[test]
fn foreign_source_duplicate_cases_missing_methods_and_unreachable_expectations_refuse() {
    let task = report_task("report-fixture");
    let foreign = report_task("other-fixture");
    assert!(!task.checks(&foreign.replay(&foreign.reference).unwrap())[0].correct);
    let mut duplicate = task.clone();
    duplicate.cases[1] = duplicate.cases[0].clone();
    assert!(duplicate.validate().is_err());
    let mut duplicate = task.clone();
    if let Case::Report { input, .. } = &mut duplicate.cases[1] {
        *input = "/work/report-a.csv".into();
    }
    assert!(duplicate.validate().is_err());
    let mut impossible = task.clone();
    if let Case::Report { expected, .. } = &mut impossible.cases[0] {
        expected.0 = b"hardcoded".to_vec();
    }
    assert!(impossible.validate().is_err());
    let mut no_method = task.clone();
    no_method.reference.clear();
    assert!(no_method.validate().is_err());
    let mut host = conversion_task();
    if let Case::File { path, .. } = &mut host.cases[0] {
        *path = "/etc/private".into();
    }
    assert!(host.validate().is_err());
}
#[test]
fn history_config_output_and_source_deserialization_limits_are_enforced() {
    let task = report_task("report-fixture");
    assert!(
        task.replay(&vec![command("locale"); MAX_HISTORY + 1])
            .is_err()
    );
    let mut world = task.replay(&[]).unwrap();
    let before = world.clone();
    assert!(
        world
            .apply(&Action::SaveLocale {
                text: "x".repeat(2049)
            })
            .is_err()
    );
    assert_eq!(world, before);
    assert!(world.apply(&command("cat /etc/passwd")).is_err());
    assert_eq!(world, before);
    let mut value = serde_json::to_value(&task).unwrap();
    value["facts"]["inputs"][0]["bytes"] = serde_json::json!(vec![0; 16385]);
    assert!(serde_json::from_value::<Task>(value).is_err());
    let mut value = serde_json::to_value(&task).unwrap();
    value["cases"][0]["expected"] = serde_json::json!(vec![0; 32769]);
    assert!(serde_json::from_value::<Task>(value).is_err());
}

fn definition() -> Definition {
    let initial = report_task("report-fixture");
    let mut example = report_task("report-example");
    example.facts.inputs[0].bytes = "item;cents\nThé;245\nSpare;-75\n".as_bytes().to_vec();
    for case in &mut example.cases {
        if let Case::Report {
            input, expected, ..
        } = case
            && input == "/work/report-a.csv"
        {
            expected.0 = "Bestand\nArtikel;Betrag\nThé;2.45\nSpare;-0.75\n"
                .as_bytes()
                .to_vec();
        }
    }
    Definition { initial, example }
}
fn solve(draft: &mut Draft, task: &Task) {
    for action in &task.reference {
        draft.perform(task, action.clone()).unwrap().unwrap();
    }
    assert!(draft.check(task).unwrap().iter().all(|r| r.correct));
}
#[test]
fn separate_practice_success_never_awards_assignment_credit_and_json_resume_retains_drafts() {
    let d = definition();
    d.validate().unwrap();
    let mut s = Session::new(&d);
    s.view = crate::script_teaching::View::Example;
    solve(&mut s.practice, &d.example);
    assert!(!s.solved(&d));
    assert_eq!(s.assignment, Draft::new(&d.initial));
    s.assignment.input = "iconv 'unfinished".into();
    s.practice
        .edit_configuration("# unfinished example\nLANG=".into())
        .unwrap();
    let saved = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&saved).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored, s);
    assert!(!restored.solved(&d));
}
#[test]
fn pending_configuration_does_not_hide_behind_earlier_passing_cases() {
    let d = definition();
    let mut s = Session::new(&d);
    solve(&mut s.assignment, &d.initial);
    assert!(s.solved(&d));
    let valid = s.assignment.configuration.clone();
    s.assignment
        .edit_configuration("LC_ALL=C\n".into())
        .unwrap();
    assert!(!s.assignment.checked && !s.assignment.attempted);
    assert!(!s.solved(&d));
    let rows = s.assignment.check(&d.initial).unwrap();
    assert!(!rows[0].correct && rows[0].label.contains("Unsaved"));
    assert!(rows[1..].iter().all(|r| r.correct));
    s.assignment.edit_configuration(valid).unwrap();
    assert!(!s.assignment.checked);
    s.assignment.check(&d.initial).unwrap();
    assert!(s.solved(&d));
}
#[test]
fn state_changes_invalidate_success_but_typing_and_unsupported_commands_do_not_break_valid_state() {
    let d = definition();
    let mut s = Session::new(&d);
    solve(&mut s.assignment, &d.initial);
    s.assignment.input = "bash unsupported.sh".into();
    assert!(s.assignment.run(&d.initial).unwrap().is_err());
    assert!(s.solved(&d));
    s.assignment.input = "export LC_ALL=C".into();
    s.assignment.run(&d.initial).unwrap().unwrap();
    assert!(!s.assignment.checked && !s.assignment.attempted && !s.solved(&d));
    assert!(
        !s.assignment
            .check(&d.initial)
            .unwrap()
            .iter()
            .all(|r| r.correct)
    );
    s.assignment.input = "unset LC_ALL".into();
    s.assignment.run(&d.initial).unwrap().unwrap();
    assert!(!s.solved(&d));
    s.assignment.check(&d.initial).unwrap();
    assert!(s.solved(&d));
}
#[test]
fn failed_literal_input_resumes_with_rederived_error_and_no_inserted_history() {
    let d = definition();
    let mut s = Session::new(&d);
    s.practice.input = "cat 'unfinished".into();
    assert!(s.practice.run(&d.example).unwrap().is_err());
    assert!(s.practice.history.is_empty());
    let raw = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&raw).unwrap();
    restored.validate(&d).unwrap();
    let error = restored
        .practice
        .last_output(&d.example)
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Syntax);
    assert_eq!(restored.practice.input, "cat 'unfinished");
}
#[test]
fn forged_credit_provenance_foreign_input_and_future_version_refuse() {
    let d = definition();
    let mut bad = Session::new(&d);
    bad.assignment.checked = true;
    bad.assignment.attempted = true;
    assert!(bad.validate(&d).is_err());
    assert!(!bad.solved(&d));
    let mut bad = Session::new(&d);
    bad.assignment.task_id = d.example.id.clone();
    assert!(bad.validate(&d).is_err());
    let mut bad = Session::new(&d);
    bad.assignment.report_input = "/etc/passwd".into();
    assert!(bad.validate(&d).is_err());
    let mut bad = Session::new(&d);
    bad.version = 2;
    assert!(bad.validate(&d).is_err());
    let mut bad = Session::new(&d);
    bad.assignment.input = "locale".into();
    bad.assignment.run(&d.initial).unwrap().unwrap();
    bad.assignment.last_run.as_mut().unwrap().before = 2;
    assert!(bad.validate(&d).is_err());
    let mut bad = Session::new(&d);
    bad.assignment.last_run = Some(session::Run {
        before: 0,
        action: command("locale"),
        appended: false,
    });
    assert!(bad.validate(&d).is_err());
    let mut value = serde_json::to_value(Session::new(&d)).unwrap();
    value["assignment"]["invented_world"] = serde_json::json!({"solved":true});
    assert!(serde_json::from_value::<Session>(value).is_err());
}
#[test]
fn reset_changes_only_the_selected_draft_and_restores_source_defaults() {
    let d = definition();
    let mut s = Session::new(&d);
    solve(&mut s.assignment, &d.initial);
    s.practice.input = "locale -k charmap".into();
    s.practice.run(&d.example).unwrap().unwrap();
    let assignment = s.assignment.clone();
    s.reset_practice(&d);
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.practice, Draft::new(&d.example));
    s.practice.input = "locale -a".into();
    let practice = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.practice, practice);
    assert_eq!(s.assignment, Draft::new(&d.initial));
}
#[test]
fn action_limit_is_finite_and_its_last_refusal_survives_replay() {
    let d = definition();
    let mut draft = Draft::new(&d.initial);
    for _ in 0..MAX_HISTORY {
        draft
            .perform(&d.initial, command("locale decimal_point"))
            .unwrap()
            .unwrap();
    }
    let before = draft.history.clone();
    let error = draft
        .perform(&d.initial, command("export LC_ALL=C"))
        .unwrap()
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Limit);
    assert_eq!(draft.history, before);
    draft.validate(&d.initial).unwrap();
    assert_eq!(
        draft
            .last_output(&d.initial)
            .unwrap()
            .unwrap()
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let mut value = serde_json::to_value(&draft).unwrap();
    value["history"] = serde_json::Value::Array(vec![
        serde_json::to_value(command("locale"))
            .unwrap();
        MAX_HISTORY + 1
    ]);
    assert!(serde_json::from_value::<Draft>(value).is_err());
}
#[test]
fn conversion_failure_with_saved_prefix_is_replayed_and_cannot_preserve_completed_credit() {
    let task = conversion_task();
    let mut draft = Draft::new(&task);
    solve(&mut draft, &task);
    assert!(draft.checked);
    draft.input = "iconv -f LATIN1 -t ASCII -o fixed-legacy.txt legacy.txt".into();
    let output = draft.run(&task).unwrap().unwrap();
    assert_eq!(output.status, 1);
    assert!(!draft.checked && !draft.attempted);
    let restored: Draft = serde_json::from_slice(&serde_json::to_vec(&draft).unwrap()).unwrap();
    restored.validate(&task).unwrap();
    assert_eq!(
        restored
            .world(&task)
            .unwrap()
            .files()
            .read("fixed-legacy.txt")
            .unwrap(),
        b"Caf"
    );
    assert_eq!(
        restored
            .last_output(&task)
            .unwrap()
            .unwrap()
            .unwrap()
            .status,
        1
    );
}

fn scenario() -> crate::pack::Scenario {
    let mut d = definition();
    d.initial.id = "locale-capability-binding-fixture".into();
    let mut solution = d
        .initial
        .reference
        .iter()
        .map(|action| format!("{}{}", PREFIX, serde_json::to_string(action).unwrap()))
        .collect::<Vec<_>>();
    solution.push("@locale-check".into());
    let value = serde_json::json!({"schema_version":1,"scenarios":[{
        "id":d.initial.id,"version":1,"track":"LPIC-1","topic":"Locale and encoding",
        "title":"Repair supplied report settings","summary":"Technical capability fixture; not registered authored curriculum.",
        "objective":"Repair current and saved locale settings so every supplied sample report keeps complete UTF-8 names, German messages and decimal-point numbers.",
        "difficulty":"Intermediate","minutes":20,"steps":["Inspect current and saved settings.","Repair the report behavior and check current and new sessions."],
        "hints":["A process-wide override can mask category settings."],
        "sources":["https://www.lpi.org/our-certifications/exam-101-102-objectives/"],
        "explanation":"Source/progress fixture only; complete teaching and native review remain separate gates.",
        "simplifications":"Finite virtual byte documents and an original sample application, not full libc, systemd or a shell. No host execution.",
        "initial":{"kind":"Localization","simulation":Session::new(&d)},
        "goal":{"kind":"Localization","definition":d},"solution":solution,"prerequisites":[]
    }]});
    crate::pack::load(&serde_json::to_string(&value).unwrap())
        .unwrap()
        .remove(0)
}
fn attempt_session(a: &mut crate::progress::Attempt) -> &mut Session {
    let crate::engine::State::Localization { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn actual_pack_engine_and_completion_require_independent_source_bound_work() {
    use crate::{engine, progress};
    let s = scenario();
    let engine::Goal::Localization { definition } = &s.goal else {
        panic!()
    };
    let mut attempt = progress::Attempt::new(&s, 1);
    solve(
        &mut attempt_session(&mut attempt).practice,
        &definition.example,
    );
    assert!(!progress::current_solved(&attempt, &s));
    engine::execute_for_goal(&mut attempt.state, &s.goal, "@locale-check").unwrap();
    assert!(engine::checks(&attempt.state, &s.goal).iter().any(|r| !r.0));
    for raw in &s.solution {
        engine::execute_for_goal(&mut attempt.state, &s.goal, raw).unwrap();
    }
    assert!(progress::current_solved(&attempt, &s));
    let actual =
        engine::execute_for_goal(&mut attempt.state, &s.goal, "locale decimal_point").unwrap();
    assert_eq!(actual, "Exit status 0\n.\n");
    engine::execute_for_goal(&mut attempt.state, &s.goal, "export LC_ALL=C").unwrap();
    assert!(!progress::current_solved(&attempt, &s));
    let before = attempt.state.clone();
    assert!(engine::execute(&mut attempt.state, "@locale-check").is_err());
    assert_eq!(attempt.state, before);
}
#[test]
fn malformed_source_pairs_undeclared_helpers_and_host_forms_refuse_without_mutation() {
    use crate::{engine, progress};
    let s = scenario();
    let mut attempt = progress::Attempt::new(&s, 1);
    for raw in [
        "sh script.sh",
        "cat /etc/passwd",
        "export PATH=/usr/bin",
        "@locale-check extra",
        "LOCALIZATION WORK\n{\"kind\":\"command\",\"text\":\"locale\",\"invented\":true}",
    ] {
        let before = attempt.state.clone();
        assert!(
            engine::execute_for_goal(&mut attempt.state, &s.goal, raw).is_err(),
            "{raw}"
        );
        assert_eq!(attempt.state, before);
    }
    let mut wrong = s.clone();
    if let engine::Goal::Localization { definition } = &mut wrong.goal {
        definition.initial.id = "foreign".into();
    }
    assert!(source(&wrong).is_err());
    let mut wrong = s.clone();
    if let engine::State::Localization { simulation } = &mut wrong.initial {
        simulation.view = crate::script_teaching::View::Example;
    }
    assert!(source(&wrong).is_err());
}
#[test]
fn real_disk_resume_preserves_all_existing415_attempts_and_both_unfinished_drafts() {
    use crate::progress;
    let s = scenario();
    let mut catalog = crate::pack::accepted415().unwrap();
    assert_eq!(catalog.len(), 415);
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for old in &catalog {
        let mut attempt = progress::Attempt::new(old, 1);
        crate::workspace::prepare(&mut attempt);
        if crate::command_location_workspace::has_source(old) {
            crate::command_location_workspace::prepare(&mut attempt, old, true).unwrap();
        }
        if crate::accounts_template_workspace::has_source(old) {
            crate::accounts_template_workspace::prepare(&mut attempt, old, true).unwrap();
        }
        p.attempts.insert(old.id.clone(), attempt);
    }
    let preserved = serde_json::to_value(&p.attempts).unwrap();
    catalog.push(s.clone());
    let mut attempt = progress::Attempt::new(&s, 1);
    attempt_session(&mut attempt).assignment.input = "iconv -f ".into();
    attempt_session(&mut attempt).practice.input = "export LANG='unfinished".into();
    attempt_session(&mut attempt)
        .practice
        .edit_configuration("# unfinished saved-file edit\nLANG=".into())
        .unwrap();
    p.attempts.insert(s.id.clone(), attempt);
    p.favorites.insert(s.id.clone());
    let dir = std::env::temp_dir().join(format!(
        "locale-binding-{}-{}",
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
    let mut restored = progress::load(&file, &catalog).unwrap();
    assert!(restored.favorites.contains(&s.id));
    let current = restored.attempts.remove(&s.id).unwrap();
    assert_eq!(serde_json::to_value(restored.attempts).unwrap(), preserved);
    assert_eq!(
        serde_json::to_value(current).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap()
    );
    assert!(progress::save(&file, &p).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
    let mut bad: progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    let attempt = bad.attempts.get_mut(&s.id).unwrap();
    attempt_session(attempt).assignment.checked = true;
    attempt_session(attempt).assignment.attempted = true;
    assert!(!progress::current_solved(attempt, &s));
    assert!(progress::save_with_catalog(&file, &bad, &catalog).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), before);
}
