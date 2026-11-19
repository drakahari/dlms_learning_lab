//! Bounded local notification effects; no host commands.
use dlms_learning_lab::maintenance_notice::*;
fn facts() -> Facts {
    Facts {
        version: 1,
        host: "training-host".into(),
        date: dlms_learning_lab::cron::Moment::parse("2026-04-14 00:00").unwrap(),
        minute: 1200,
        logins: vec![
            Login {
                user: "alex".into(),
                tty: "pts/0".into(),
                since_minute: 1180,
            },
            Login {
                user: "alex".into(),
                tty: "pts/1".into(),
                since_minute: 1190,
            },
            Login {
                user: "sam".into(),
                tty: "tty2".into(),
                since_minute: 1170,
            },
        ],
        initial: Files {
            issue: "Authorized users only".into(),
            motd: "Welcome".into(),
        },
    }
}
#[test]
fn login_notices_and_current_terminal_delivery_have_independent_effects() {
    let f = facts();
    let initial = World::initial(&f).unwrap();
    let saved = World::replay(
        &f,
        &[Action::Save {
            document: Document::Motd,
            text: "Maintenance at22:00".into(),
        }],
    )
    .unwrap();
    assert_eq!(saved.files.motd, "Maintenance at22:00");
    assert!(saved.deliveries.is_empty());
    assert_eq!(saved.files.issue, initial.files.issue);
    let live = World::replay(
        &f,
        &[Action::Broadcast {
            text: "Maintenance at22:00".into(),
        }],
    )
    .unwrap();
    assert_eq!(live.files, initial.files);
    assert_eq!(live.deliveries.len(), 1);
    assert_eq!(
        live.deliveries[0].terminals.len(),
        3,
        "One user has two distinct active terminals"
    );
    assert_eq!(live.receipt_by_terminal().len(), 3);
}
#[test]
fn schedule_time_and_five_minute_login_gate_are_virtual_and_cancelable() {
    let f = facts();
    let start = Action::Schedule {
        after_minutes: 10,
        text: "Restart scheduled".into(),
    };
    let world = World::replay(&f, std::slice::from_ref(&start)).unwrap();
    assert_eq!(world.restart.unwrap().at, 1210);
    let world = World::replay(&f, &[start.clone(), Action::Advance { minutes: 4 }]).unwrap();
    assert!(!world.ordinary_logins_blocked());
    let world = World::replay(&f, &[start.clone(), Action::Advance { minutes: 5 }]).unwrap();
    assert!(world.ordinary_logins_blocked());
    let world = World::replay(
        &f,
        &[
            start,
            Action::Advance { minutes: 5 },
            Action::Cancel {
                text: "Restart cancelled".into(),
            },
        ],
    )
    .unwrap();
    assert!(!world.ordinary_logins_blocked());
    assert!(world.restart.is_none());
    assert_eq!(world.deliveries.len(), 2);
    assert!(!world.stopped);
}
#[test]
fn virtual_restart_stops_at_due_time_not_at_the_requested_later_time() {
    let f = facts();
    let actions = [
        Action::Schedule {
            after_minutes: 10,
            text: "Restart".into(),
        },
        Action::Advance { minutes: 15 },
    ];
    let mut world = World::replay(&f, &actions).unwrap();
    assert_eq!(world.minute, 1210);
    assert!(world.stopped);
    let before = world.clone();
    assert!(
        world
            .apply(
                &f,
                &Action::Broadcast {
                    text: "Too late".into()
                }
            )
            .is_err()
    );
    assert_eq!(world, before);
}
#[test]
fn refused_actions_do_not_partially_mutate_the_owned_world() {
    let f = facts();
    let mut world = World::initial(&f).unwrap();
    let before = world.clone();
    for action in [
        Action::Schedule {
            after_minutes: 0,
            text: "Bad".into(),
        },
        Action::Schedule {
            after_minutes: 1441,
            text: "Bad".into(),
        },
        Action::Cancel {
            text: "Nothing scheduled".into(),
        },
        Action::Broadcast {
            text: "\u{1b}[2J".into(),
        },
        Action::Save {
            document: Document::Motd,
            text: "x".repeat(4097),
        },
    ] {
        assert!(world.apply(&f, &action).is_err());
        assert_eq!(world, before);
    }
    world
        .apply(
            &f,
            &Action::Schedule {
                after_minutes: 10,
                text: "Restart".into(),
            },
        )
        .unwrap();
    let before = world.clone();
    assert!(
        world
            .apply(
                &f,
                &Action::Schedule {
                    after_minutes: 15,
                    text: "Replacement outside subset".into()
                }
            )
            .is_err()
    );
    assert_eq!(world, before);
}
#[test]
fn history_and_clock_bounds_cannot_grow_without_limit() {
    let f = facts();
    let events = vec![Action::Advance { minutes: 1 }; 64];
    assert!(World::replay(&f, &events).is_ok());
    assert!(World::replay(&f, &vec![Action::Advance { minutes: 1 }; 65]).is_err());
    assert!(
        World::replay(
            &f,
            &[
                Action::Advance { minutes: 1440 },
                Action::Advance { minutes: 1440 }
            ]
        )
        .is_err()
    );
}
#[test]
fn duplicate_terminals_and_escaping_paths_are_rejected_without_host_lookup() {
    let mut f = facts();
    f.logins[1].tty = f.logins[0].tty.clone();
    assert!(f.validate().is_err());
    f.logins[1].tty = "../../personal".into();
    assert!(f.validate().is_err());
    assert!(
        serde_json::from_str::<Action>(r#"{"kind":"save","document":"/etc/passwd","text":"x"}"#)
            .is_err()
    );
    assert!(
        serde_json::from_str::<Action>(
            r#"{"kind":"broadcast","text":"x","host_command":"anything"}"#
        )
        .is_err()
    );
}

#[test]
fn literal_terminal_runs_derive_outputs_actions_and_receipts_from_the_current_world() {
    use dlms_learning_lab::maintenance_notice::command as cli;
    let f = facts();
    let world = World::initial(&f).unwrap();
    let clock = cli::run(&f, &world, "date '+%H:%M'", "").unwrap();
    assert_eq!(clock.stdout, "20:00\n");
    assert!(clock.action.is_none());
    let who = cli::run(&f, &world, "who", "").unwrap();
    assert_eq!(who.stdout.lines().count(), 3);
    assert!(who.stdout.contains("2026-04-14 19:40"));
    assert!(who.stdout.contains("pts/1"));
    let notice = "Service: archive\nStarts: 22:00\nEnds: 22:20\nContact: duty\n";
    let sent = cli::run(&f, &world, "wall", notice).unwrap();
    assert!(
        sent.stdout.is_empty(),
        "Delivery observations are separate from real wall stdout"
    );
    assert!(
        world.deliveries.is_empty(),
        "Parsing/run never commits a caller's state"
    );
    let delivered = World::replay(&f, &[sent.action.unwrap()]).unwrap();
    assert_eq!(delivered.deliveries[0].body, notice);
    assert_eq!(delivered.deliveries[0].terminals.len(), 3);
    assert_eq!(delivered.files, world.files);
    let schedule = cli::run(&f, &world, "shutdown -r +0015 'Routine maintenance'", "").unwrap();
    assert!(schedule.stdout.contains("2026-04-14 20:15 UTC"));
    let action = schedule.action.unwrap();
    assert!(
        matches!(&action,Action::Schedule{after_minutes:15,text}if text=="Routine maintenance")
    );
    let pending = World::replay(&f, &[action]).unwrap();
    let cancel = cli::run(&f, &pending, "shutdown -c", "").unwrap();
    assert!(cancel.stdout.is_empty());
    assert!(matches!(cancel.action, Some(Action::Cancel { .. })));
}

#[test]
fn malformed_unsupported_and_escaping_terminal_requests_fail_without_action_or_host_access() {
    use dlms_learning_lab::maintenance_notice::command as cli;
    let f = facts();
    let world = World::initial(&f).unwrap();
    for (raw, kind) in [
        ("wall 'unfinished", cli::Kind::Syntax),
        ("wall /etc/passwd", cli::Kind::Unsupported),
        ("cat ../../personal", cli::Kind::Unsupported),
        ("wall; shutdown now", cli::Kind::Unsupported),
        ("wall $(anything)", cli::Kind::Unsupported),
        ("shutdown -r now", cli::Kind::Unsupported),
        ("shutdown -r +not-a-number", cli::Kind::Syntax),
        ("shutdown -r +0", cli::Kind::Limit),
        ("shutdown -r +1441", cli::Kind::Limit),
        ("systemctl reboot", cli::Kind::Unsupported),
        ("shutdown -c --no-wall", cli::Kind::Unsupported),
    ] {
        let before = world.clone();
        let error = cli::run(&f, &world, raw, "hello").unwrap_err();
        assert_eq!(error.kind, kind, "{raw}: {error}");
        assert_eq!(world, before);
    }
    assert_eq!(
        cli::run(&f, &world, "wall", "").unwrap_err().kind,
        cli::Kind::Syntax
    );
    assert_eq!(
        cli::parse(&"x".repeat(1025), "").unwrap_err().kind,
        cli::Kind::Limit
    );
    assert_eq!(
        cli::parse(&"x ".repeat(25), "").unwrap_err().kind,
        cli::Kind::Limit
    );
}

#[test]
fn source_calendar_login_times_and_plain_issue_subset_are_checked_before_replay() {
    let mut f = facts();
    f.logins[0].since_minute = f.minute + 1;
    assert!(f.validate().is_err());
    f = facts();
    f.date.day = 32;
    assert!(f.validate().is_err());
    f = facts();
    f.date.hour = 1;
    assert!(f.validate().is_err());
    f = facts();
    f.date = dlms_learning_lab::cron::Moment::parse("2099-12-31 00:00").unwrap();
    assert!(f.validate().is_err());
    f = facts();
    f.initial.issue = "Recorded host \\n".into();
    assert!(f.validate().is_err());
    f = facts();
    let mut world = World::initial(&f).unwrap();
    let before = world.clone();
    assert!(
        world
            .apply(
                &f,
                &Action::Save {
                    document: Document::Issue,
                    text: "\\l".into()
                }
            )
            .is_err()
    );
    assert_eq!(world, before);
    let mut value = serde_json::to_value(&f).unwrap();
    value["initial"]["motd"] = serde_json::json!("x".repeat(4097));
    assert!(serde_json::from_value::<Facts>(value).is_err());
}
