use dlms_learning_lab::{
    cron::Moment,
    maintenance_notice::{self as core, Action, Document, bulletin::Bulletin},
    maintenance_notice_workspace::*,
};
fn task(restart: bool, example: bool) -> Task {
    let (service, starts, ends, contact, now) = if example {
        ("reports", 1080, 1095, "helpdesk", 1050)
    } else {
        ("archive", 1320, 1340, "duty", 1290)
    };
    let required = Bulletin {
        service: service.into(),
        starts,
        ends,
        contact: contact.into(),
    };
    let facts = core::Facts {
        version: 1,
        host: if example {
            "worked-host"
        } else {
            "assignment-host"
        }
        .into(),
        date: Moment::parse("2026-04-14 00:00").unwrap(),
        minute: now,
        logins: vec![
            core::Login {
                user: "alex".into(),
                tty: "pts/0".into(),
                since_minute: now - 10,
            },
            core::Login {
                user: "alex".into(),
                tty: "pts/1".into(),
                since_minute: now - 5,
            },
            core::Login {
                user: "sam".into(),
                tty: "tty2".into(),
                since_minute: now - 15,
            },
        ],
        initial: core::Files {
            issue: "Authorized use only. Retain this identification.".into(),
            motd: "An outdated maintenance announcement".into(),
        },
    };
    let starter = if restart {
        vec![Action::Schedule {
            after_minutes: 10,
            text: "The inherited restart is scheduled too early.".into(),
        }]
    } else {
        vec![]
    };
    let mut reference = vec![];
    if restart {
        reference.push(Action::Cancel {
            text: "The early restart is cancelled; a corrected notice follows.".into(),
        });
    }
    reference.push(Action::Save {
        document: Document::Motd,
        text: required.render(),
    });
    if restart {
        reference.push(Action::Schedule {
            after_minutes: starts - now,
            text: "An approved restart is scheduled.".into(),
        });
    }
    reference.push(Action::Broadcast {
        text: required.render(),
    });
    Task {
        id: format!(
            "{}-{}",
            if restart { "restart" } else { "notices" },
            if example { "worked" } else { "assignment" }
        ),
        facts,
        required,
        requires_restart: restart,
        minimum_lead: if restart { 20 } else { 0 },
        starter,
        reference,
    }
}
fn definition(restart: bool) -> Definition {
    Definition {
        initial: task(restart, false),
        example: task(restart, true),
    }
}
fn solve(draft: &mut Draft, task: &Task) {
    for action in &task.reference {
        draft.apply_action(task, action.clone()).unwrap();
    }
    draft.check(task).unwrap();
    assert!(draft.solved(task));
}
#[test]
fn methods_solve_all_recipients_and_timing_with_separate_worked_data() {
    for restart in [false, true] {
        let def = definition(restart);
        def.validate().unwrap();
        let mut session = Session::new(&def).unwrap();
        assert!(!session.solved(&def));
        solve(&mut session.practice, &def.example);
        assert!(!session.solved(&def));
        assert_eq!(session.assignment, Draft::new(&def.initial).unwrap());
        solve(&mut session.assignment, &def.initial);
        assert!(session.solved(&def));
        assert_eq!(
            session
                .assignment
                .world(&def.initial)
                .unwrap()
                .receipt_by_terminal()
                .len(),
            3
        );
    }
}
#[test]
fn real_editor_save_and_wall_commands_have_partial_then_complete_feedback() {
    let t = task(false, false);
    let mut draft = Draft::new(&t).unwrap();
    draft.edit(t.required.render()).unwrap();
    draft.check(&t).unwrap();
    assert!(!draft.solved(&t));
    assert!(
        draft
            .checks(&t)
            .unwrap()
            .iter()
            .any(|c| c.label == "Saved files and sent message" && !c.correct)
    );
    draft.save(&t).unwrap();
    draft.check(&t).unwrap();
    let checks = draft.checks(&t).unwrap();
    assert!(checks[0].correct);
    assert!(!draft.solved(&t));
    assert_eq!(
        checks
            .iter()
            .filter(|c| c.label.starts_with("Current session") && !c.correct)
            .count(),
        3
    );
    draft.selected = Editor::Stdin;
    draft.edit(t.required.render()).unwrap();
    assert!(draft.run(&t, "wall").unwrap().is_empty());
    draft.check(&t).unwrap();
    assert!(draft.solved(&t));
    let before = draft.clone();
    draft.run(&t, "who").unwrap();
    assert_eq!(
        draft, before,
        "Read-only queries do not destroy a valid outcome"
    );
}
#[test]
fn alternative_bulletin_order_and_clock_adjusted_relative_delay_pass_by_state() {
    let t = task(true, false);
    let mut d = Draft::new(&t).unwrap();
    d.run(&t, "shutdown -c 'Corrected announcement follows'")
        .unwrap();
    d.advance(&t, 5).unwrap();
    d.run(&t, "shutdown -r +0025 'Restart scheduled'").unwrap();
    let alternate = format!(
        "CONTACT : {}\nENDS:22:20\nservice:ARCHIVE\nstarts:22:00\n",
        t.required.contact.to_ascii_uppercase()
    );
    d.edit(alternate.clone()).unwrap();
    d.save(&t).unwrap();
    d.selected = Editor::Stdin;
    d.edit(alternate).unwrap();
    d.run(&t, "wall").unwrap();
    d.check(&t).unwrap();
    assert!(d.solved(&t));
    assert_eq!(d.world(&t).unwrap().restart.unwrap().at, 1320);
}
#[test]
fn unsaved_unsent_broken_announcements_and_late_clock_clear_current_success() {
    let t = task(false, false);
    let mut d = Draft::new(&t).unwrap();
    solve(&mut d, &t);
    d.selected = Editor::Motd;
    d.edit(t.required.render().replace("22:00", "21:50"))
        .unwrap();
    assert!(!d.checked);
    d.check(&t).unwrap();
    assert!(!d.solved(&t));
    d.save(&t).unwrap();
    d.check(&t).unwrap();
    assert!(!d.solved(&t));
    assert!(
        d.checks(&t).unwrap()[0]
            .feedback
            .contains("different start time")
    );
    d.edit(t.required.render()).unwrap();
    d.save(&t).unwrap();
    d.check(&t).unwrap();
    assert!(d.solved(&t));
    d.selected = Editor::Stdin;
    d.edit("not a valid handover".into()).unwrap();
    assert!(!d.checked);
    d.check(&t).unwrap();
    assert!(!d.solved(&t));
    d.discard_stdin();
    d.check(&t).unwrap();
    assert!(d.solved(&t));
    d.edit("not a valid handover".into()).unwrap();
    d.run(&t, "wall").unwrap();
    d.check(&t).unwrap();
    assert!(
        !d.solved(&t),
        "An older correct broadcast cannot hide a newer incorrect one"
    );
    d.edit(t.required.render()).unwrap();
    d.run(&t, "wall").unwrap();
    d.check(&t).unwrap();
    assert!(d.solved(&t));
    d.advance(&t, 30).unwrap();
    d.check(&t).unwrap();
    assert!(
        !d.solved(&t),
        "A stale notice cannot remain currently verified after its start"
    );
}
#[test]
fn inherited_plan_requires_full_repair_and_unrelated_issue_is_preserved() {
    let t = task(true, false);
    let mut d = Draft::new(&t).unwrap();
    d.run(&t, "shutdown -c").unwrap();
    d.check(&t).unwrap();
    assert!(!d.solved(&t));
    assert!(
        d.checks(&t)
            .unwrap()
            .iter()
            .any(|c| c.feedback.contains("only part of the handover"))
    );
    // Cancellation above is already a real transition; continue the method
    // rather than issuing a second cancellation with no pending schedule.
    for action in t.reference.iter().skip(1) {
        d.apply_action(&t, action.clone()).unwrap();
    }
    d.check(&t).unwrap();
    assert!(d.solved(&t));
    d.selected = Editor::Issue;
    d.edit("Replaced the identification".into()).unwrap();
    d.save(&t).unwrap();
    d.check(&t).unwrap();
    assert!(!d.solved(&t));
    assert!(!d.checks(&t).unwrap()[1].correct);
}
#[test]
fn reset_resume_and_foreign_or_forged_saved_input_are_source_bound() {
    let def = definition(true);
    let mut s = Session::new(&def).unwrap();
    solve(&mut s.assignment, &def.initial);
    s.practice.selected = Editor::Stdin;
    s.practice
        .edit("unfinished separate example".into())
        .unwrap();
    s.view = dlms_learning_lab::script_teaching::View::Assignment;
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut loaded: Session = serde_json::from_slice(&bytes).unwrap();
    loaded.validate(&def).unwrap();
    assert_eq!(loaded, s);
    assert!(loaded.solved(&def));
    let practice = loaded.practice.clone();
    loaded.reset_assignment(&def).unwrap();
    assert!(!loaded.solved(&def));
    assert_eq!(loaded.practice, practice);
    let mut forged = s.clone();
    forged.assignment.stdin = "Changed but forged as already sent".into();
    forged.assignment.stdin_pending = false;
    assert!(forged.validate(&def).is_err());
    let mut foreign = s.clone();
    foreign.assignment.task = def.example.id.clone();
    assert!(foreign.validate(&def).is_err());
    let mut erased = s.clone();
    erased.assignment.history.remove(0);
    assert!(erased.validate(&def).is_err());
    let mut version = s;
    version.version = 2;
    assert!(version.validate(&def).is_err());
}
#[test]
fn malformed_sources_unreachable_methods_and_already_solved_starters_are_rejected() {
    let good = task(true, false);
    let mut bad = good.clone();
    bad.minimum_lead = 31;
    assert!(bad.validate().is_err());
    let mut bad = good.clone();
    bad.reference
        .retain(|a| !matches!(a, Action::Broadcast { .. }));
    assert!(bad.validate().is_err());
    let mut bad = task(false, false);
    bad.starter = bad.reference.clone();
    assert!(bad.validate().is_err());
    let mut def = definition(false);
    def.example = def.initial.clone();
    assert!(def.validate().is_err());
    let mut d = Draft::new(&good).unwrap();
    let before = d.clone();
    assert!(
        d.run(&good, "shutdown -r +20 'Do not replace without cancelling'")
            .is_err()
    );
    assert_eq!(d, before);
    assert!(d.edit("x".repeat(4097)).is_err());
    assert_eq!(d, before);
}

#[test]
fn recorded_output_is_recomputed_from_its_history_point_and_survives_separate_resume() {
    let def = definition(false);
    let t = &def.initial;
    let mut d = Draft::new(t).unwrap();
    d.command = "date '+%H:%M'".into();
    assert_eq!(d.run_and_record(t).unwrap(), "21:30\n");
    d.advance(t, 5).unwrap();
    assert_eq!(
        d.last_output(t).unwrap().unwrap(),
        "21:30\n",
        "Last run retains its actual earlier clock, not the current state"
    );
    d.command = "cat ../../personal".into();
    assert!(d.run_and_record(t).is_err());
    assert!(d.world(t).is_ok());
    assert!(d.last_output(t).unwrap().is_err());
    let mut s = Session::new(&def).unwrap();
    s.assignment = d;
    s.practice.command = "who".into();
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    restored.validate(&def).unwrap();
    assert_eq!(restored, s);
    assert_eq!(restored.practice.command, "who");
    assert_eq!(restored.assignment.command, "cat ../../personal");
}
#[test]
fn saved_run_cannot_claim_a_missing_action_or_forged_input() {
    let t = task(false, false);
    let mut d = Draft::new(&t).unwrap();
    d.selected = Editor::Stdin;
    d.edit(t.required.render()).unwrap();
    d.command = "wall".into();
    d.run_and_record(&t).unwrap();
    assert!(d.world(&t).is_ok());
    let mut forged = d.clone();
    forged.last_run.as_mut().unwrap().stdin = "A different input".into();
    assert!(forged.world(&t).is_err());
    let mut missing = d;
    missing.history.clear();
    assert!(missing.world(&t).is_err());
}
#[test]
fn full_history_refuses_new_work_without_recording_a_fictitious_success() {
    let t = task(false, false);
    let mut d = Draft::new(&t).unwrap();
    for _ in 0..64 {
        d.apply_action(
            &t,
            Action::Save {
                document: Document::Issue,
                text: t.facts.initial.issue.clone(),
            },
        )
        .unwrap();
    }
    d.selected = Editor::Stdin;
    d.edit(t.required.render()).unwrap();
    d.command = "wall".into();
    let prior = d.history.clone();
    assert!(d.run_and_record(&t).is_err());
    assert_eq!(d.history, prior);
    assert!(d.last_run.is_none());
    assert!(d.world(&t).is_ok());
}
