use dlms_learning_lab::{
    cron::{Moment, State},
    cron_teaching::{AnnotatedAction, Concept, Learning, Lesson},
    cron_workspace::{
        self as work, Action, Definition, Document, ExpectedRun, Probe, Requirement, Session,
    },
    script_teaching::View,
};
use std::collections::{BTreeMap, BTreeSet};
#[path = "support/cron_lessons.rs"]
mod fixture;
fn moment(value: &str) -> Moment {
    Moment::parse(value).unwrap()
}
fn user(name: &str) -> Document {
    Document::User { owner: name.into() }
}
fn definition(owner: &str, source: &str, report: &str) -> Definition {
    let files = BTreeMap::from([
        (source.into(), "cobalt\n".into()),
        ("/work/peer".into(), "keep\n".into()),
    ]);
    let initial = State {
        clock: moment("2026-10-28 09:09"),
        users: BTreeSet::from(["root".into(), owner.into()]),
        tables: BTreeMap::new(),
        system_table: String::new(),
        allow: Some(BTreeSet::from([owner.into()])),
        deny: None,
        files,
    };
    // Literal independently reasoned calendar outcomes; no call to Schedule,
    // matching logic, or expected-job derivation from the tested implementation.
    let starts = [
        ("2026-10-26 09:09", true),
        ("2026-10-27 09:09", true),
        ("2026-10-28 09:09", true),
        ("2026-10-29 09:09", true),
        ("2026-10-30 09:09", true),
        ("2026-10-31 09:09", false),
        ("2026-11-01 09:09", false),
        ("2026-10-28 09:08", false),
        ("2026-10-28 09:10", false),
        ("2026-10-28 08:09", false),
        ("2026-10-28 10:09", false),
        ("2026-11-02 09:09", true),
        ("2027-02-28 09:09", false),
        ("2027-03-01 09:09", true),
        ("2027-01-01 09:09", true),
        ("2026-12-31 09:09", true),
    ];
    let probes = starts
        .into_iter()
        .enumerate()
        .map(|(i, (start, runs))| {
            let text = format!("sample {i}: amber and silver\n");
            let files = BTreeMap::from([
                (source.into(), text.clone()),
                ("/work/peer".into(), "keep\n".into()),
            ]);
            let mut after = files.clone();
            if runs {
                after.insert(report.into(), text);
            }
            Probe {
                name: format!("Calendar and file case {}", i + 1),
                start: moment(start),
                files,
                after,
                expected: if runs {
                    vec![ExpectedRun {
                        owner: owner.into(),
                        stdout: String::new(),
                        stderr: String::new(),
                        status: 0,
                    }]
                } else {
                    vec![]
                },
            }
        })
        .collect();
    Definition {
        initial,
        editable: vec![user(owner)],
        outputs: vec![report.into()],
        requirements: vec![Requirement::CurrentFile {
            path: report.into(),
            text: "cobalt\n".into(),
        }],
        probes,
    }
}
fn act(d: &Definition, s: &mut State, w: &mut Session, action: Action) {
    work::apply(d, s, w, &action).unwrap();
}
fn install(d: &Definition, s: &mut State, w: &mut Session, text: &str) {
    let target = d.editable[0].clone();
    act(
        d,
        s,
        w,
        Action::Edit {
            target: target.clone(),
            text: text.into(),
        },
    );
    act(d, s, w, Action::Install { target });
}
fn solved(d: &Definition, text: &str) -> (State, Session) {
    let mut s = d.initial.clone();
    let mut w = Session::new(d).unwrap();
    w.actor = d
        .initial
        .users
        .iter()
        .find(|s| s.as_str() != "root")
        .unwrap()
        .clone();
    install(d, &mut s, &mut w, text);
    act(
        d,
        &mut s,
        &mut w,
        Action::Run {
            end: moment("2026-10-28 09:10"),
        },
    );
    assert!(work::checks_with_session(d, &s, &w).iter().all(|r| r.0));
    (s, w)
}
#[test]
fn full_solutions_use_actual_files_and_equivalent_calendar_syntax() {
    let d = definition("alex", "/work/source", "/work/report");
    d.validate().unwrap();
    for command in [
        "10 9 * * 1-5 cat /work/source > /work/report\n",
        "10 9 * * MON-FRI cat '/work/source' > '/work/report'\n",
        "10 9 * * 1,2,3,4,5 'cat' /work/source 1> /work/report\n",
    ] {
        let (s, w) = solved(&d, command);
        assert!(d.current_ready(&s));
        assert!(w.applied(&d, &s));
    }
    assert_eq!(d.row_count(), 18);
}
#[test]
fn partial_work_requires_current_run_and_unsaved_edits_clear_success() {
    let d = definition("alex", "/work/source", "/work/report");
    let mut s = d.initial.clone();
    let mut w = Session::new(&d).unwrap();
    install(
        &d,
        &mut s,
        &mut w,
        "10 9 * * 1-5 cat /work/source > /work/report\n",
    );
    let rows = work::checks_with_session(&d, &s, &w);
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        [vec![true, true, false], vec![true; 16]].concat()
    );
    act(
        &d,
        &mut s,
        &mut w,
        Action::Run {
            end: moment("2026-10-28 09:10"),
        },
    );
    assert!(work::checks_with_session(&d, &s, &w).iter().all(|r| r.0));
    let installed = s.clone();
    act(
        &d,
        &mut s,
        &mut w,
        Action::Edit {
            target: user("alex"),
            text: "* * * * * echo wrong\n".into(),
        },
    );
    assert_eq!(s, installed);
    let rows = work::checks_with_session(&d, &s, &w);
    assert!(!rows[0].0);
    assert!(rows[1..].iter().all(|r| r.0));
    let before = (s.clone(), w.clone());
    assert!(
        work::apply(
            &d,
            &mut s,
            &mut w,
            &Action::Run {
                end: moment("2026-10-28 09:11")
            }
        )
        .unwrap_err()
        .contains("pending drafts")
    );
    assert_eq!((s, w), before);
}
#[test]
fn wrong_calendar_hardcoded_output_and_file_damage_do_not_pass() {
    let d = definition("alex", "/work/source", "/work/report");
    for bad in [
        "10 9 * * * cat /work/source > /work/report\n",
        "10 9 1-31 * 1-5 cat /work/source > /work/report\n",
        "10 9 * * 1-5 printf 'cobalt\\n' > /work/report\n",
        "10 9 * * 1-5 cat /work/source > /work/peer\n",
    ] {
        let mut s = d.initial.clone();
        let mut w = Session::new(&d).unwrap();
        install(&d, &mut s, &mut w, bad);
        act(
            &d,
            &mut s,
            &mut w,
            Action::Run {
                end: moment("2026-10-28 09:10"),
            },
        );
        assert!(
            work::checks_with_session(&d, &s, &w).iter().any(|r| !r.0),
            "{bad}"
        );
        assert!(d.bound_state(&s));
        assert!(w.valid_for(&d));
        // Wrong resulting files remain resumable and the learner can edit again.
        let round: State = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(round, s);
        let session: Session = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
        assert_eq!(session, w);
        install(
            &d,
            &mut s,
            &mut w,
            "10 9 * * 1-5 cat /work/source > /work/report\n",
        );
    }
}
#[test]
fn explicit_check_yields_and_cancellation_never_mutates_assignment() {
    let d = definition("alex", "/work/source", "/work/report");
    let (s, w) = solved(&d, "10 9 * * 1-5 cat /work/source > /work/report\n");
    let before = (s.clone(), w.clone());
    let mut check = work::Check::new(&d, &s).unwrap();
    check.tick(1);
    assert!(!check.done());
    check.cancel();
    assert!(check.done());
    assert!(check.rows.iter().any(|r| !r.0));
    assert_eq!((s, w), before);
    let mut check = work::Check::new(&d, &before.0).unwrap();
    for _ in 0..10_000 {
        if check.done() {
            break;
        }
        check.tick(1);
    }
    assert!(check.done());
    assert_eq!(check.rows.len(), 18);
    assert!(check.rows.iter().all(|r| r.0));
}
#[test]
fn foreign_context_and_oversized_drafts_fail_without_mutation() {
    let d = definition("alex", "/work/source", "/work/report");
    let mut s = d.initial.clone();
    let mut w = Session::new(&d).unwrap();
    for action in [
        Action::Edit {
            target: user("alex"),
            text: "x".repeat(8193),
        },
        Action::Select {
            target: user("root"),
        },
        Action::Actor {
            actor: "unknown".into(),
        },
    ] {
        let before = (s.clone(), w.clone());
        assert!(work::apply(&d, &mut s, &mut w, &action).is_err());
        assert_eq!((s.clone(), w.clone()), before);
    }
    let mut other = definition("alex", "/work/source", "/work/report");
    other.initial.clock = moment("2026-10-29 09:09");
    assert!(!w.valid_for(&other));
    let before = (s.clone(), w.clone());
    assert!(
        work::apply(
            &other,
            &mut s,
            &mut w,
            &Action::Select {
                target: user("alex")
            }
        )
        .is_err()
    );
    assert_eq!((s, w), before);
}
fn lesson() -> Lesson {
    let d = definition("mira", "/work/example-input", "/work/example-output");
    let text = "10 9 * * 1-5 cat /work/example-input > /work/example-output\n";
    let annotated = |title: &str, action| AnnotatedAction {
        title: title.into(),
        explanation:
            "Use this separate practice workspace, then inspect its installed schedule and output."
                .into(),
        action,
    };
    Lesson { prerequisite_note:"You have used cat, redirection and a separate assignment draft.".into(),
        concepts:vec![Concept{title:"Schedule a command".into(),text:"The five fields choose a minute, hour, day of month, month and day of week. A user table runs as its owner; it has no username field.".into()}],
        example_description:"Copy a different sample on weekday mornings in the ungraded example.".into(),definition:d,
        example_actions:vec![annotated("Choose the supplied user",Action::Actor{actor:"mira".into()}),annotated("Edit the example table",Action::Edit{target:user("mira"),text:text.into()}),annotated("Install the table",Action::Install{target:user("mira")}),annotated("Run one minute",Action::Run{end:moment("2026-10-28 09:10")})],
        try_change:"Leave an unsaved wrong calendar change. The running schedule is unchanged, but checking must identify the pending draft.".into(),
        try_actions:vec![Action::Edit{target:user("mira"),text:"* * * * * echo unfinished\n".into()}],
        try_rows:[vec![false],vec![true;18]].concat(),
        repair:vec![Action::Edit{target:user("mira"),text:text.into()}],
    }
}
#[test]
fn teaching_fills_only_drafts_and_example_remains_ungraded_and_separate() {
    let l = lesson();
    l.validate().unwrap();
    let d = definition("alex", "/work/source", "/work/report");
    // This test's peer sentinel is deliberately shared; separation must reject it.
    assert!(!l.separate_from(&d));
    let mut l = l;
    l.definition.initial.files.remove("/work/peer");
    for probe in &mut l.definition.probes {
        probe.files.remove("/work/peer");
        probe.after.remove("/work/peer");
    }
    assert!(l.separate_from(&d));
    l.validate().unwrap();
    let mut learning = Learning::new(&l).unwrap();
    let assignment = (d.initial.clone(), Session::new(&d).unwrap());
    let initial = learning.simulation.clone();
    learning.view = View::Example;
    for i in 0..l.example_actions.len() {
        learning.selected_example = i;
        learning.fill(&l).unwrap();
        assert_eq!(learning.simulation, initial);
    }
    assert!(
        !learning
            .session
            .applied(&l.definition, &learning.simulation)
    );
    assert_eq!(learning.session.window_end, "2026-10-28 09:10");
    assert_eq!(assignment, (d.initial.clone(), Session::new(&d).unwrap()));
    let saved: Learning = serde_json::from_str(&serde_json::to_string(&learning).unwrap()).unwrap();
    assert!(saved.valid_for(&l));
    assert_eq!(saved.session, learning.session);
    learning.reset(&l).unwrap();
    assert_eq!(learning.view, View::Example);
    assert_eq!(learning.simulation, initial);
    assert!(learning.session.applied(&l.definition, &initial));
}

#[test]
fn direct_workspace_and_teaching_decode_rejects_excess_before_owning_strings() {
    let d = definition("alex", "/work/source", "/work/report");
    for (pointer, text) in [
        ("/editable/0/owner", "a".repeat(33)),
        ("/outputs/0", "x".repeat(257)),
        ("/requirements/0/path", "x".repeat(257)),
        ("/requirements/0/text", "x".repeat(32769)),
        ("/probes/0/name", "x".repeat(121)),
        ("/probes/0/expected/0/stdout", "x".repeat(32769)),
    ] {
        let mut raw = serde_json::to_value(&d).unwrap();
        *raw.pointer_mut(pointer).unwrap() = text.into();
        let error = serde_json::from_value::<Definition>(raw)
            .unwrap_err()
            .to_string();
        assert!(error.contains("text byte limit"), "{pointer}: {error}");
    }
    let mut raw = serde_json::to_value(&d).unwrap();
    raw["probes"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"arbitrary":"do not decode as Probe"}));
    assert!(
        serde_json::from_value::<Definition>(raw)
            .unwrap_err()
            .to_string()
            .contains("item limit")
    );
    let session = Session::new(&d).unwrap();
    let mut raw = serde_json::to_value(session).unwrap();
    raw["window_end"] = "x".repeat(33).into();
    assert!(
        serde_json::from_value::<Session>(raw)
            .unwrap_err()
            .to_string()
            .contains("text byte limit")
    );
    let mut raw = serde_json::to_value(lesson()).unwrap();
    raw["try_rows"] = serde_json::json!(vec![true; 34]);
    serde_json::from_value::<Lesson>(raw.clone()).unwrap();
    raw["try_rows"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"not":"a bool"}));
    assert!(
        serde_json::from_value::<Lesson>(raw)
            .unwrap_err()
            .to_string()
            .contains("item limit")
    );
    let raw=serde_json::json!({"kind":"Edit","target":{"kind":"User","owner":"alex"},"text":"x".repeat(8193)}).to_string();
    assert!(Action::from_json(&raw).is_err());
    let raw=serde_json::json!({"kind":"Install","actor":"alex","target":{"kind":"User","owner":"alex"},"text":"x".repeat(8193)}).to_string();
    assert!(work::ReferenceAction::from_json(&raw).is_err());
}

#[test]
fn authoring_codec_is_real_state_based_reachability_not_success_strings() {
    let d = definition("alex", "/work/source", "/work/report");
    let mut s = d.initial.clone();
    let install = work::ReferenceAction::Install {
        actor: "alex".into(),
        target: user("alex"),
        text: "10 9 * * MON-FRI cat /work/source > /work/report\n".into(),
    };
    work::execute(
        &d,
        &mut s,
        &format!(
            "{}{}",
            work::PREFIX,
            serde_json::to_string(&install).unwrap()
        ),
    )
    .unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
    let run = work::ReferenceAction::Run {
        end: moment("2026-10-28 09:10"),
    };
    work::execute(
        &d,
        &mut s,
        &format!("{}{}", work::PREFIX, serde_json::to_string(&run).unwrap()),
    )
    .unwrap();
    assert!(d.checks(&s).iter().all(|r| r.0));
    let before = s.clone();
    assert!(work::execute(&d, &mut s, "cat /etc/passwd").is_err());
    assert_eq!(s, before);
    let remove = work::ReferenceAction::Remove {
        actor: "root".into(),
        target: user("alex"),
    };
    work::execute(
        &d,
        &mut s,
        &format!(
            "{}{}",
            work::PREFIX,
            serde_json::to_string(&remove).unwrap()
        ),
    )
    .unwrap();
    assert!(d.checks(&s).iter().any(|r| !r.0));
    assert!(!s.tables.contains_key("alex"));
}

#[test]
fn equivalent_policy_order_is_applied_but_malformed_missing_and_empty_stay_distinct() {
    let mut d = definition("alex", "/work/source", "/work/report");
    d.initial.users.insert("pat".into());
    d.initial.allow = Some(BTreeSet::from(["alex".into(), "pat".into()]));
    d.initial.deny = Some(BTreeSet::from(["alex".into(), "pat".into()]));
    d.editable.extend([Document::Allow, Document::Deny]);
    d.validate().unwrap();
    let mut state = d.initial.clone();
    let mut session = Session::new(&d).unwrap();
    act(
        &d,
        &mut state,
        &mut session,
        Action::Actor {
            actor: "root".into(),
        },
    );
    for target in [Document::Allow, Document::Deny] {
        act(
            &d,
            &mut state,
            &mut session,
            Action::Edit {
                target: target.clone(),
                text: "pat\nalex\n".into(),
            },
        );
        assert!(
            session.applied(&d, &state),
            "equivalent ordering is not a pending behavioral change"
        );
        act(
            &d,
            &mut state,
            &mut session,
            Action::Install {
                target: target.clone(),
            },
        );
        assert!(session.applied(&d, &state));
        assert_eq!(session.drafts[&target.key()], "pat\nalex\n");
        let before = state.clone();
        act(
            &d,
            &mut state,
            &mut session,
            Action::Edit {
                target: target.clone(),
                text: "alex\nunknown\n".into(),
            },
        );
        assert!(!session.applied(&d, &state));
        assert!(
            work::apply(
                &d,
                &mut state,
                &mut session,
                &Action::Install {
                    target: target.clone()
                }
            )
            .is_err()
        );
        assert_eq!(state, before);
        act(
            &d,
            &mut state,
            &mut session,
            Action::Edit {
                target,
                text: "alex\npat\n".into(),
            },
        );
    }
    assert!(
        state.can_manage("alex", "alex"),
        "allow takes precedence over deny"
    );
    act(
        &d,
        &mut state,
        &mut session,
        Action::Remove {
            target: Document::Allow,
        },
    );
    assert!(state.allow.is_none());
    assert!(session.applied(&d, &state));
    assert!(!state.can_manage("alex", "alex"));
    act(
        &d,
        &mut state,
        &mut session,
        Action::Remove {
            target: Document::Deny,
        },
    );
    assert!(state.deny.is_none());
    assert!(session.applied(&d, &state));
    assert!(!state.can_manage("alex", "alex"));
    act(
        &d,
        &mut state,
        &mut session,
        Action::Install {
            target: Document::Deny,
        },
    );
    assert_eq!(state.deny, Some(BTreeSet::new()));
    assert!(session.applied(&d, &state));
    assert!(state.can_manage("alex", "alex"));
    act(
        &d,
        &mut state,
        &mut session,
        Action::Install {
            target: Document::Allow,
        },
    );
    assert_eq!(state.allow, Some(BTreeSet::new()));
    assert!(session.applied(&d, &state));
    assert!(!state.can_manage("alex", "alex"));
}

#[test]
fn admitted_native_document_comparison_matches_validated_pending_semantics_without_admitting_context()
 {
    for s in fixture::lessons() {
        let dlms_learning_lab::engine::Goal::Cron { definition: d } = s.goal else {
            unreachable!()
        };
        let state = &d.initial;
        let initial = Session::new(&d).unwrap();
        assert_eq!(
            initial.installed_matches(&d, state),
            initial.applied(&d, state)
        );
        assert!(initial.applied(&d, state));
        for target in &d.editable {
            let mut draft = initial.clone();
            draft
                .drafts
                .insert(target.key(), "# unfinished replacement\n".into());
            assert!(
                draft.valid_for(&d),
                "Wrong drafts remain editable, rather than being treated as foreign context"
            );
            assert!(!draft.installed_matches(&d, state));
            assert!(!draft.applied(&d, state));
        }
        let mut foreign = initial.clone();
        foreign.actor = "not-a-supplied-account".into();
        assert!(!foreign.valid_for(&d));
        assert!(
            !foreign.applied(&d, state),
            "Full engine admission still rejects an invalid actor even if document strings match"
        );
    }
}
