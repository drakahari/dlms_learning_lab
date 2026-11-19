use dlms_learning_lab::{
    script_teaching::View,
    sieve_vacation_policy::RunResult,
    sieve_workspace::{Action, Definition, Scope, Session, Task},
};
use serde_json::json;
fn task(practice: bool) -> Task {
    let site = if practice {
        "practice.test"
    } else {
        "away.test"
    };
    let owner = format!("learner@{site}");
    let contact = format!("contact team@{site}");
    let ordinary = vec![("To", owner.as_str()), ("Subject", "A personal request")];
    let sample = |id: &str, at: u64, sender: &str, headers: Vec<(&str, &str)>, reply: bool| {
        json!({"id":id,"label":format!("{id} arrival"),"headers":headers,"body":"Same message body.",
            "expected":["INBOX"],"envelope":{"from":sender,"to":owner},
            "arrival_seconds":at,"expected_reply":reply})
    };
    serde_json::from_value(json!({
        "address_tests":true,"delivery_actions":true,
        "vacation":{"user":{"primary":owner,"aliases":[]},"required_reply_text":contact},
        "mailboxes":["INBOX"],"starter":"# Keep mail and repair the vacation responder.\nkeep;\n",
        "reference":format!("require \"vacation\"; vacation :days 1 \"I am away; {contact}.\";"),
        "cases":[
            sample("first",0,"visitor@outside.test",ordinary.clone(),true),
            sample("too-soon",86399,"visitor@outside.test",ordinary.clone(),false),
            sample("exact-expiry",86400,"visitor@outside.test",ordinary.clone(),true),
            sample("different-sender",86400,"second@outside.test",ordinary.clone(),true),
            sample("automatic",86401,"robot@outside.test",vec![("To",&owner),("Auto-Submitted","auto-generated")],false),
            sample("list",86402,"list@outside.test",vec![("To",&owner),("List-Id","<group.outside.test>")],false),
            sample("null",86403,"",ordinary.clone(),false),
            sample("not-personal",86404,"other@outside.test",vec![("To","other@elsewhere.test")],false),
            sample("self",86405,&owner,ordinary.clone(),false),
            sample("same-interval",86406,"visitor@outside.test",ordinary.clone(),false),
            sample("next-expiry",172800,"visitor@outside.test",ordinary.clone(),true),
            sample("human",172800,"postmaster@outside.test",vec![("To",&owner),("Auto-Submitted","no")],true)
        ]
    })).unwrap()
}
fn definition() -> Definition {
    Definition {
        schema: 1,
        source_id: "vacation-workspace-fixture".into(),
        assignment: task(false),
        example: task(true),
    }
}
fn run(s: &mut Session, d: &Definition, source: &str, practice: bool) {
    s.action(
        d,
        &Action::Edit {
            script: source.into(),
        },
        practice,
    )
    .unwrap();
    s.action(d, &Action::Run, practice).unwrap();
    for _ in 0..12 {
        s.action(d, &Action::Tick, practice).unwrap();
    }
}
#[test]
fn real_timeline_drives_each_entry_point_and_equivalent_solutions_pass() {
    let d = definition();
    d.validate().unwrap();
    assert_eq!(d.assignment.cases[0].headers, d.assignment.cases[1].headers);
    assert_eq!(d.assignment.cases[0].headers, d.assignment.cases[2].headers);
    for source in [
        &d.assignment.reference,
        "require \"vacation\"; if exists \"To\" {vacation :days 1 :handle \"office\" \"Please contact team@away.test while I am away.\";} keep;",
    ] {
        let mut s = Session::new(&d).unwrap();
        run(&mut s, &d, source, false);
        assert!(d.solved(&s), "{:?}", d.checks(&s));
        assert!(d.checks(&s).iter().all(|c| c.passed));
        for (case, saved) in d.assignment.cases.iter().zip(&s.assignment.results) {
            assert_eq!(
                *saved,
                d.assignment
                    .evaluate(case, &d.assignment.parse(source).unwrap())
            );
        }
        assert!(matches!(
            s.assignment.results[1].vacation,
            Some(RunResult::Recent {
                last_sent: 0,
                remaining_seconds: 1
            })
        ));
        assert!(matches!(
            s.assignment.results[2].vacation,
            Some(RunResult::Sent { .. })
        ));
        assert!(matches!(
            s.assignment.results[9].vacation,
            Some(RunResult::Recent {
                last_sent: 86400,
                remaining_seconds: 86394
            })
        ));
        d.bound(&s).unwrap();
    }
}
#[test]
fn wrong_intervals_missing_reply_and_missing_contact_have_distinct_feedback() {
    let d = definition();
    for (source, needle) in [
        (
            "require \"vacation\"; vacation \"Please contact team@away.test.\";",
            "arrival with the last reply",
        ),
        ("keep;", "needs a vacation reply"),
        (
            "require \"vacation\"; vacation :days 1 \"I am away.\";",
            "missing the required text",
        ),
    ] {
        let mut s = Session::new(&d).unwrap();
        run(&mut s, &d, source, false);
        assert!(!d.solved(&s));
        assert!(s.assignment.results.iter().any(|r| r.passed));
        assert!(
            d.checks(&s)
                .iter()
                .any(|c| !c.passed && c.detail.contains(needle)),
            "{:?}",
            d.checks(&s)
        );
        run(&mut s, &d, &d.assignment.reference, false);
        assert!(d.solved(&s));
    }
}
#[test]
fn no_header_condition_can_hardcode_the_different_times_of_identical_mail() {
    let d = definition();
    let source = "require \"vacation\"; if header :is \"Subject\" \"A personal request\" { vacation :days 365 \"contact team@away.test\"; }";
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, source, false);
    assert!(s.assignment.results[0].passed && s.assignment.results[1].passed);
    assert!(!s.assignment.results[2].passed && !s.assignment.results[10].passed);
    assert!(!d.solved(&s));
}
#[test]
fn partial_stop_resume_is_source_bound_and_run_restarts_an_independent_history() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.example.reference, true);
    assert!(s.practice.solved(&d.example));
    assert!(!d.solved(&s));
    let example = s.practice.clone();
    s.action(
        &d,
        &Action::Edit {
            script: d.assignment.reference.clone(),
        },
        false,
    )
    .unwrap();
    s.action(&d, &Action::Run, false).unwrap();
    for _ in 0..2 {
        s.action(&d, &Action::Tick, false).unwrap();
    }
    s.action(&d, &Action::Stop, false).unwrap();
    assert!(s.assignment.stopped && !s.assignment.checked);
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    restored.action(&d, &Action::Run, false).unwrap();
    for _ in 0..12 {
        restored.action(&d, &Action::Tick, false).unwrap();
    }
    assert!(d.solved(&restored));
    assert_eq!(restored.practice, example);
    restored.action(&d, &Action::Reset, false).unwrap();
    assert_eq!(restored.assignment, Scope::new(&d.assignment));
    assert_eq!(restored.practice, example);
}
#[test]
fn edits_and_forged_reply_history_or_metadata_cannot_retain_credit() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.assignment.reference, false);
    let original = s.clone();
    for change in 0..6 {
        let mut forged = original.clone();
        match change {
            0 => {
                if let Some(RunResult::Recent { last_sent, .. }) =
                    &mut forged.assignment.results[1].vacation
                {
                    *last_sent = 1;
                }
            }
            1 => {
                if let Some(RunResult::Sent { reply, .. }) =
                    &mut forged.assignment.results[0].vacation
                {
                    reply.to = "stolen@outside.test".into();
                }
            }
            2 => {
                forged.assignment.results[1].vacation =
                    forged.assignment.results[0].vacation.clone();
            }
            3 => {
                if let Some(RunResult::Sent { reply, .. }) =
                    &mut forged.assignment.results[0].vacation
                {
                    reply.body = "contact team@away.test but forged".into();
                }
            }
            4 => {
                forged.assignment.results[0].vacation = None;
            }
            _ => {
                forged.assignment.draft = String::from("keep;");
            }
        }
        assert!(d.bound(&forged).is_err(), "{change}");
        assert!(!d.solved(&forged));
    }
    s.action(
        &d,
        &Action::Edit {
            script: "keep;".into(),
        },
        false,
    )
    .unwrap();
    assert!(s.assignment.results.is_empty());
    assert!(!d.solved(&s));
    assert!(d.checks(&s).iter().any(|c| c.detail.contains("Unchecked")));
}
#[test]
fn malformed_timeline_profiles_and_oversized_imported_replies_fail_closed() {
    let d = definition();
    for change in 0..10 {
        let mut bad = d.clone();
        match change {
            0 => bad.assignment.delivery_actions = false,
            1 => bad.assignment.cases[2].arrival_seconds = Some(0),
            2 => bad.assignment.cases[0].arrival_seconds = None,
            3 => bad.assignment.cases[0].expected_reply = None,
            4 => bad
                .assignment
                .vacation
                .as_mut()
                .unwrap()
                .required_reply_text
                .clear(),
            5 => bad.example.vacation = None,
            6 => bad.assignment.cases[0].envelope = None,
            7 => bad.assignment.vacation.as_mut().unwrap().user.primary = "bad".into(),
            8 => bad.assignment.cases[0].expected_rejection = true,
            _ => bad.assignment.cases.push(bad.assignment.cases[0].clone()),
        }
        assert!(bad.validate().is_err(), "{change}");
    }
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.assignment.reference, false);
    if let Some(RunResult::Sent { reply, .. }) = &mut s.assignment.results[0].vacation {
        reply.references = vec!["<id@outside.test>".into(); 33];
    }
    assert!(s.validate().is_err());
    assert!(d.bound(&s).is_err());
    s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.assignment.reference, false);
    if let Some(RunResult::Recent {
        remaining_seconds, ..
    }) = &mut s.assignment.results[1].vacation
    {
        *remaining_seconds = u64::MAX;
    }
    assert!(s.validate().is_err());
}
#[test]
fn selected_sample_and_view_restore_without_merging_practice_and_assignment() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.view = View::Example;
    s.action(&d, &Action::Select { index: 7 }, true).unwrap();
    s.action(
        &d,
        &Action::Edit {
            script: "# unfinished independent draft\nkeep;".into(),
        },
        false,
    )
    .unwrap();
    run(&mut s, &d, &d.example.reference, true);
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert_eq!(restored.practice.selected, 7);
    assert!(!d.solved(&restored));
    assert!(restored.assignment.draft.starts_with("# unfinished"));
}
#[test]
fn existing_eight_sieve_definitions_and_scopes_serialize_without_timeline_fields() {
    for raw in [
        include_str!("../scenarios/sieve-filter-lessons-v1.json"),
        include_str!("../scenarios/sieve-address-lessons-v1.json"),
        include_str!("../scenarios/sieve-delivery-lessons-v1.json"),
    ] {
        let pack: serde_json::Value = serde_json::from_str(raw).unwrap();
        for scenario in pack["scenarios"].as_array().unwrap() {
            let raw = scenario["goal"]["definition"].clone();
            let d: Definition = serde_json::from_value(raw.clone()).unwrap();
            assert_eq!(serde_json::to_value(&d).unwrap(), raw);
            let mut s = Session::new(&d).unwrap();
            run(&mut s, &d, &d.assignment.reference, false);
            assert!(d.solved(&s));
            d.bound(&s).unwrap();
            let value = serde_json::to_value(&s).unwrap();
            for scope in ["practice", "assignment"] {
                for r in value[scope]["results"].as_array().unwrap() {
                    assert!(r.get("vacation").is_none());
                }
            }
        }
    }
}

#[test]
fn generated_reply_subjects_with_tabs_and_the_maximum_header_size_restore() {
    let mut d = definition();
    for task in [&mut d.assignment, &mut d.example] {
        for case in &mut task.cases {
            case.headers.push(("X-Original".into(), "untouched".into()));
            for (name, value) in &mut case.headers {
                if name == "Subject" {
                    *value = format!("{}\t", "s".repeat(1023));
                }
            }
        }
    }
    d.validate().unwrap();
    let mut s = Session::new(&d).unwrap();
    run(&mut s, &d, &d.assignment.reference, false);
    assert!(d.solved(&s));
    let saved: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    saved.validate().unwrap();
    d.bound(&saved).unwrap();
    let Some(RunResult::Sent { reply, .. }) = &saved.assignment.results[0].vacation else {
        panic!()
    };
    assert_eq!(reply.subject.len(), 1030);
    assert!(reply.subject.ends_with('\t'));
}
#[test]
fn direct_api_cannot_replay_more_than_the_disclosed_twelve_samples() {
    let mut task = task(false);
    let mut c = task.cases[11].clone();
    c.id = "beyond-limit".into();
    task.cases.push(c.clone());
    let result = task.evaluate(&c, &task.parse(&task.reference).unwrap());
    assert!(!result.passed);
    assert_eq!(
        result.fault_kind,
        Some(dlms_learning_lab::sieve_lex::Kind::Limit)
    );
}
