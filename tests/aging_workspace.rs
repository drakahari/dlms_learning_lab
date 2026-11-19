use dlms_learning_lab::{
    accounts::EpochDay, aging as a, aging_workspace as w, script_teaching::View,
};
fn day(s: &str) -> EpochDay {
    EpochDay::parse(s).unwrap()
}
fn days(n: u32) -> Option<a::Days> {
    Some(a::Days::try_from(n).unwrap())
}
fn record(date: &str) -> a::Record {
    a::Record {
        last_change: a::LastChange::Day(day(date)),
        minimum: days(3),
        maximum: days(90),
        warning: days(2),
        inactive: days(5),
        account_expires: None,
    }
}
fn task(id: &str, target: &str, start: &str, dates: &[&str], max: u32, warn: u32) -> w::Task {
    let original = record(start);
    let mut expected = original.clone();
    expected.maximum = days(max);
    expected.warning = days(warn);
    let initial = a::World::new(vec![
        a::Entry {
            login: target.into(),
            record: original,
        },
        a::Entry {
            login: format!("{target}_protected"),
            record: record(start),
        },
    ])
    .unwrap();
    let cases = dates
        .iter()
        .enumerate()
        .map(|(n, s)| {
            let date = day(s);
            w::Case {
                label: format!("Sample {n}"),
                day: date,
                expected: expected.observe(date),
            }
        })
        .collect();
    w::Task{id:id.into(),instructions:"Set the requested maximum age and warning window, retain history and protect the other account.".into(),target:target.into(),initial,expected,expiry_range:None,cases}
}
fn definition() -> w::Definition {
    w::Definition {
        initial: task(
            "rotation-task",
            "morgan",
            "2026-01-01",
            &[
                "2026-01-04",
                "2026-02-05",
                "2026-02-14",
                "2026-02-15",
                "2026-02-20",
            ],
            45,
            10,
        ),
        example: task(
            "separate-practice",
            "taylor",
            "2027-03-10",
            &["2027-03-13", "2027-03-26", "2027-03-30", "2027-04-04"],
            20,
            4,
        ),
        example_worked: vec![w::Command("chage -M 20 -W 4 taylor".into())],
    }
}
fn command(s: &mut w::Scope, t: &w::Task, c: &str) {
    s.edit_input(c).unwrap();
    s.execute(t).unwrap();
}
#[test]
fn distinct_practice_and_equivalent_commands_grade_record_and_date_behavior() {
    let d = definition();
    d.validate().unwrap();
    for cmds in [
        vec!["chage -M 45 -W 10 morgan"],
        vec!["chage --warndays=10 morgan", "chage morgan --maxdays 45"],
    ] {
        let mut s = w::Session::new(&d).unwrap();
        for c in cmds {
            command(&mut s.assignment, &d.initial, c);
        }
        assert!(!d.solved(&s));
        let checks = s.assignment.check(&d.initial).unwrap();
        assert!(checks.iter().all(|r| r.0));
        assert!(d.solved(&s));
        let observations = d.initial.observations(&s.assignment.world).unwrap();
        assert_eq!(observations[1].1.warning_days, Some(10));
        assert_eq!(
            observations[3].1.verdict,
            a::Verdict::PasswordChangeRequired
        );
        assert_eq!(observations[4].1.verdict, a::Verdict::PasswordGraceEnded);
    }
}
#[test]
fn wrong_partial_and_history_rewrites_fail_even_when_one_sample_looks_good() {
    let d = definition();
    for cmd in [
        "chage -M 45 morgan",
        "chage -M -1 -W 10 morgan",
        "chage -M 45 -W 10 -d 2026-02-01 morgan",
        "chage -M 45 -W 10 -E 2026-02-01 morgan",
        "chage -M 45 -W 10 -I -1 morgan",
        "chage -M 45 -W 10 -m 0 morgan",
    ] {
        let mut s = w::Session::new(&d).unwrap();
        command(&mut s.assignment, &d.initial, cmd);
        s.assignment.check(&d.initial).unwrap();
        assert!(!d.solved(&s), "{cmd}");
    }
    let mut s = w::Session::new(&d).unwrap();
    command(&mut s.assignment, &d.initial, "chage -M 45 morgan");
    let rows = s.assignment.check(&d.initial).unwrap();
    assert!(rows.iter().any(|r| r.0));
    assert!(rows.iter().any(|r| !r.0));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("warning window")));
}
#[test]
fn later_breakage_and_protected_record_changes_invalidate_checked_success() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    command(&mut s.assignment, &d.initial, "chage -M 45 -W 10 morgan");
    s.assignment.check(&d.initial).unwrap();
    assert!(d.solved(&s));
    // Even forged/stale saved UI proof must not hide changed real state.
    s.assignment.world.execute("chage -M -1 morgan").unwrap();
    assert!(s.assignment.checked);
    assert!(!d.solved(&s));
    command(&mut s.assignment, &d.initial, "chage -M 45 morgan");
    s.assignment.check(&d.initial).unwrap();
    assert!(d.solved(&s));
    command(&mut s.assignment, &d.initial, "chage -W 0 morgan_protected");
    let rows = s.assignment.check(&d.initial).unwrap();
    assert!(!d.solved(&s));
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("other supplied account"))
    );
}
#[test]
fn requested_example_never_grades_or_overwrites_assignment_and_both_resume() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    command(&mut s.assignment, &d.initial, "chage -M 45 morgan");
    s.assignment.edit_input("chage -W 10 morgan").unwrap();
    let before = s.assignment.clone();
    s.fill_example(&d).unwrap();
    assert_eq!(s.assignment, before);
    assert!(!s.practice.checked);
    s.practice.check(&d.example).unwrap();
    assert!(d.example.checks(&s.practice.world).iter().all(|r| r.0));
    assert!(!d.solved(&s));
    s.view = View::Example;
    let raw = serde_json::to_string(&s).unwrap();
    let restored = w::Session::from_json(&raw, &d).unwrap();
    assert_eq!(s, restored);
    assert_eq!(
        w::Session::new(&d).unwrap().assignment.world,
        d.initial.initial
    );
    assert_eq!(before.input, "chage -W 10 morgan");
}
#[test]
fn parser_errors_retry_diagnostics_and_reference_plans_are_bounded_and_atomic() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let original = s.assignment.world.clone();
    s.assignment
        .edit_input("chage -M 45 --prefix=/ morgan")
        .unwrap();
    assert!(
        s.assignment
            .execute(&d.initial)
            .unwrap_err()
            .contains("Unsupported")
    );
    assert_eq!(s.assignment.world, original);
    assert!(s.assignment.transcript.last().unwrap().error);
    let before = s.clone();
    let invalid = format!(
        "{}{}",
        w::PREFIX,
        serde_json::to_string(&vec!["chage -M 45 morgan", "chage -W no morgan"]).unwrap()
    );
    assert!(s.execute_reference(&d, &invalid).is_err());
    assert_eq!(s, before);
    let valid = format!(
        "{}{}",
        w::PREFIX,
        serde_json::to_string(&vec!["chage --maxdays=45 --warndays=10 morgan"]).unwrap()
    );
    s.execute_reference(&d, &valid).unwrap();
    assert!(d.solved(&s));
    assert!(!s.assignment.transcript.last().unwrap().error);
}
#[test]
fn invalid_source_or_import_cannot_authorize_itself() {
    let d = definition();
    let s = w::Session::new(&d).unwrap();
    let raw = serde_json::to_string(&s).unwrap();
    for changed in [
        raw.replace("rotation-task", "another-task"),
        raw.replace("morgan_protected", "outsider"),
        raw.replace("\"version\":1", "\"version\":2"),
    ] {
        assert!(w::Session::from_json(&changed, &d).is_err());
    }
    let mut bad = d.clone();
    bad.example.target = d.initial.target.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example_worked = vec![w::Command("chage -M -1 taylor".into())];
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0].expected.verdict = a::Verdict::AccountExpired;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[1].day = bad.initial.cases[0].day;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.initial = a::World::new(
        bad.initial
            .initial
            .entries()
            .iter()
            .map(|e| a::Entry {
                login: e.login.clone(),
                record: if e.login == bad.initial.target {
                    bad.initial.expected.clone()
                } else {
                    e.record.clone()
                },
            })
            .collect(),
    )
    .unwrap();
    assert!(bad.validate().is_err());
}
#[test]
fn finite_extension_accepts_multiple_dates_but_never_clearing_reset_or_expiry() {
    let d = definition();
    let mut t = d.initial.clone();
    t.expected.last_change = a::LastChange::ForceChange;
    t.expected.account_expires = Some(day("2026-04-25"));
    t.expiry_range = Some(w::DateRange {
        first: day("2026-04-20"),
        last: day("2026-04-30"),
    });
    let mut initial = t.expected.clone();
    initial.account_expires = Some(day("2026-04-01"));
    t.initial = a::World::new(vec![
        a::Entry {
            login: t.target.clone(),
            record: initial,
        },
        a::Entry {
            login: "protected".into(),
            record: record("2026-01-01"),
        },
    ])
    .unwrap();
    t.cases = ["2026-03-31", "2026-04-15", "2026-05-01"]
        .iter()
        .map(|date| w::Case {
            label: (*date).into(),
            day: day(date),
            expected: t.expected.observe(day(date)),
        })
        .collect();
    t.validate().unwrap();
    for date in ["2026-04-20", "2026-04-25", "2026-04-30"] {
        let mut state = t.initial.clone();
        state.execute(&format!("chage -E {date} morgan")).unwrap();
        assert!(t.checks(&state).iter().all(|r| r.0));
    }
    for cmd in [
        "chage -E -1 morgan",
        "chage -E 2026-05-15 morgan",
        "chage -E 2026-04-25 -d -1 morgan",
        "chage -E 2026-04-25 -d 2026-04-15 morgan",
    ] {
        let mut state = t.initial.clone();
        state.execute(cmd).unwrap();
        assert!(t.checks(&state).iter().any(|r| !r.0));
    }
    t.cases[1].day = day("2026-04-25");
    t.cases[1].expected = t.expected.observe(t.cases[1].day);
    assert!(
        t.validate().is_err(),
        "range must not silently exclude legitimate dates through a contradictory sample"
    );
}
#[test]
fn transcript_editor_and_json_limits_hold_without_losing_current_records() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    for _ in 0..40 {
        command(&mut s.assignment, &d.initial, "chage -l morgan");
    }
    assert!(s.assignment.transcript.len() <= 16);
    assert!(
        s.assignment
            .transcript
            .iter()
            .map(|r| r.text.len())
            .sum::<usize>()
            <= w::MAX_TRANSCRIPT
    );
    assert_eq!(s.assignment.world, d.initial.initial);
    let old = s.assignment.clone();
    assert!(s.assignment.edit_input(&"x".repeat(1025)).is_err());
    assert_eq!(s.assignment, old);
    let raw = serde_json::to_string(&s).unwrap();
    assert_eq!(w::Session::from_json(&raw, &d).unwrap(), s);
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION + 1), &d).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["assignment"]["transcript"] = serde_json::json!(
        (0..17)
            .map(|_| w::LogLine {
                text: "bounded".into(),
                error: false
            })
            .collect::<Vec<_>>()
    );
    assert!(w::Session::from_json(&value.to_string(), &d).is_err());
    assert!(w::Definition::from_json(&" ".repeat(w::MAX_DEFINITION + 1)).is_err());
}
