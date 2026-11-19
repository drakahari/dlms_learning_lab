use dlms_learning_lab::{
    accounts::EpochDay,
    aging::{self as a, Days, Entry, LastChange, Record, Verdict, World},
};
fn day(s: &str) -> EpochDay {
    EpochDay::parse(s).unwrap()
}
fn days(n: u32) -> Option<Days> {
    Some(Days::try_from(n).unwrap())
}
fn record() -> Record {
    Record {
        last_change: LastChange::Day(day("2026-01-01")),
        minimum: days(3),
        maximum: days(30),
        warning: days(7),
        inactive: days(5),
        account_expires: None,
    }
}
fn world() -> World {
    World::new(vec![
        Entry {
            login: "morgan".into(),
            record: record(),
        },
        Entry {
            login: "protected".into(),
            record: record(),
        },
    ])
    .unwrap()
}
#[test]
fn versioned_warning_expiry_and_grace_boundaries_are_distinct() {
    let r = record();
    for (date, verdict, warning, too_soon) in [
        ("2026-01-01", Verdict::Usable, None, true),
        ("2026-01-03", Verdict::Usable, None, true),
        ("2026-01-04", Verdict::Usable, None, false),
        ("2026-01-23", Verdict::Usable, None, false),
        ("2026-01-24", Verdict::Usable, Some(7), false),
        ("2026-01-30", Verdict::Usable, Some(1), false),
        ("2026-01-31", Verdict::PasswordChangeRequired, None, false),
        ("2026-02-04", Verdict::PasswordChangeRequired, None, false),
        ("2026-02-05", Verdict::PasswordGraceEnded, None, false),
    ] {
        let o = r.observe(day(date));
        assert_eq!(
            (o.verdict, o.warning_days, o.change_too_soon),
            (verdict, warning, too_soon),
            "{date}"
        );
    }
}
#[test]
fn account_date_precedes_force_change_and_disabled_password_aging() {
    for stamp in [LastChange::ForceChange, LastChange::Disabled] {
        let mut r = record();
        r.last_change = stamp;
        r.account_expires = Some(day("2026-02-01"));
        assert_eq!(
            r.observe(day("2026-02-01")).verdict,
            Verdict::AccountExpired
        );
        assert_eq!(
            r.observe(day("2026-01-31")).verdict,
            if stamp == LastChange::ForceChange {
                Verdict::PasswordChangeRequired
            } else {
                Verdict::Usable
            }
        );
        r.account_expires = None;
        assert_ne!(
            r.observe(day("2026-02-01")).verdict,
            Verdict::AccountExpired
        );
    }
}
#[test]
fn disabling_maximum_disables_its_warning_and_inactivity_but_not_account_date() {
    let mut r = record();
    r.maximum = None;
    r.inactive = days(0);
    r.warning = days(365000);
    let o = r.observe(day("2027-01-01"));
    assert_eq!(o.verdict, Verdict::Usable);
    assert_eq!(o.warning_days, None);
    r.account_expires = Some(day("2026-12-31"));
    assert_eq!(
        r.observe(day("2027-01-01")).verdict,
        Verdict::AccountExpired
    );
}
#[test]
fn future_change_zero_grace_and_large_warning_are_explicit() {
    let mut r = record();
    let o = r.observe(day("2025-12-31"));
    assert_eq!(o.verdict, Verdict::Usable);
    assert!(o.future_last_change);
    assert!(!o.change_too_soon);
    r.inactive = days(0);
    assert_eq!(
        r.observe(day("2026-01-31")).verdict,
        Verdict::PasswordGraceEnded
    );
    r.warning = days(31);
    assert_eq!(r.observe(day("2026-01-01")).warning_days, Some(30));
    r.warning = days(0);
    assert_eq!(r.observe(day("2026-01-30")).warning_days, None);
    r.maximum = days(0);
    r.inactive = None;
    assert_eq!(
        r.observe(day("2026-01-01")).verdict,
        Verdict::PasswordChangeRequired
    );
}
#[test]
fn equivalent_short_long_combined_and_separate_edits_produce_identical_records() {
    let mut short = world();
    short.execute("chage -M 45 -W 10 -I 6 morgan").unwrap();
    let mut long = world();
    long.execute("chage --maxdays=45 --warndays 10 morgan")
        .unwrap();
    long.execute("chage morgan --inactive=6").unwrap();
    assert_eq!(short, long);
    assert_eq!(short.record("protected"), world().record("protected"));
    assert_eq!(
        short.record("morgan").unwrap().last_change,
        record().last_change
    );
    assert_eq!(
        short.execute("chage -l -i morgan").unwrap(),
        long.execute("chage --iso8601 --list morgan").unwrap()
    );
}
#[test]
fn meaningful_plausible_wrong_policy_is_allowed_and_observed_not_canned_success() {
    let mut w = world();
    w.execute("chage -M -1 -W 0 -I -1 morgan").unwrap();
    assert_eq!(
        w.record("morgan")
            .unwrap()
            .observe(day("2027-01-01"))
            .verdict,
        Verdict::Usable
    );
    w.execute("chage -d 0 morgan").unwrap();
    assert_eq!(
        w.record("morgan")
            .unwrap()
            .observe(day("2027-01-01"))
            .verdict,
        Verdict::PasswordChangeRequired
    );
    w.execute("chage --expiredate=2026-03-01 morgan").unwrap();
    assert_eq!(
        w.record("morgan")
            .unwrap()
            .observe(day("2026-03-01"))
            .verdict,
        Verdict::AccountExpired
    );
    w.execute("chage -E -1 -d -1 morgan").unwrap();
    assert_eq!(
        w.record("morgan")
            .unwrap()
            .observe(day("2027-01-01"))
            .verdict,
        Verdict::Usable
    );
}
#[test]
fn malformed_unsupported_and_missing_commands_are_atomic() {
    let baseline = world();
    for cmd in [
        "chage -M 45 -W nope morgan",
        "chage -M 45 -E 2026-02-30 morgan",
        "chage -M 45 -I 365001 morgan",
        "chage -M 45 -m -2 morgan",
        "chage -M 45 -W 7 absent",
        "chage -M 45 --maxdays=40 morgan",
        "chage -l -M 45 morgan",
        "chage -M 45 -i morgan",
        "chage morgan",
        "chage -l morgan protected",
        "chage --root / morgan",
        "chage -M45 morgan",
        "chage -E 0 morgan",
        "chage -d 1970-01-01 morgan",
        "chage -l ../../etc/shadow",
        "chage -d $(date) morgan",
        "chage -M 45 morgan; id",
        "chage -l morgan\n",
        "sudo chage -M 45 morgan",
        "chage -l -- morgan",
        "chage --unknown=1 morgan",
    ] {
        let mut w = baseline.clone();
        assert!(w.execute(cmd).is_err(), "{cmd}");
        assert_eq!(w, baseline, "{cmd}");
    }
}
#[test]
fn calendar_and_numeric_day_inputs_agree_through_leap_and_year_boundaries() {
    for date in [
        "1970-01-02",
        "2000-02-29",
        "2024-02-29",
        "2025-12-31",
        "9999-12-31",
    ] {
        let d = day(date);
        let mut iso = world();
        let mut epoch = world();
        iso.execute(&format!("chage -d {date} -E {date} morgan"))
            .unwrap();
        epoch
            .execute(&format!(
                "chage --lastday={} --expiredate={} morgan",
                d.number(),
                d.number()
            ))
            .unwrap();
        assert_eq!(iso, epoch);
    }
    assert!(a::parse("chage -d 1900-02-29 morgan").is_err());
    assert!(a::parse("chage -d 2100-02-29 morgan").is_err());
    let mut r = record();
    r.last_change = LastChange::Day(day("9999-12-31"));
    let s = r.list();
    assert!(s.contains("outside supported calendar"));
    assert!(!s.contains("Password expires\t\t\t: never"));
}
#[test]
fn serde_roundtrip_rejects_invalid_bounds_unknown_fields_duplicate_identities() {
    let w = world();
    let json = serde_json::to_string(&w).unwrap();
    assert_eq!(World::from_json(&json).unwrap(), w);
    for raw in [
        json.replace("\"maximum\":30", "\"maximum\":365001"),
        json.replace("\"maximum\":30", "\"maximum\":-1"),
        json.replace("\"protected\"", "\"morgan\""),
        json.replace("\"maximum\":30", "\"unexpected\":30"),
        json.replace(
            "\"last_change\":{\"kind\":\"day\",\"day\":20454}",
            "\"last_change\":{\"kind\":\"day\",\"day\":0}",
        ),
    ] {
        assert_ne!(raw, json, "malformed fixture must actually differ");
        assert!(World::from_json(&raw).is_err(), "{raw}");
    }
}
#[test]
fn resource_limits_and_query_never_mutate_state() {
    let mut w = world();
    let before = w.clone();
    let s = w.execute("chage -l morgan").unwrap();
    assert!(s.contains("Last password change"));
    assert!(s.contains("2026-01-31"));
    assert!(s.len() < 16 * 1024);
    assert_eq!(w, before);
    assert!(w.execute(&"x".repeat(a::MAX_COMMAND + 1)).is_err());
    assert!(
        w.execute(&format!("chage {}morgan", "-l ".repeat(32)))
            .is_err()
    );
    assert_eq!(w, before);
    assert!(World::new(vec![]).is_err());
    assert!(
        World::new(
            (0..17)
                .map(|n| Entry {
                    login: format!("user{n}"),
                    record: record()
                })
                .collect()
        )
        .is_err()
    );
    assert!(World::from_json(&" ".repeat(a::MAX_JSON + 1)).is_err());
    let too_many = serde_json::json!({"entries":(0..17).map(|n|Entry{login:format!("user{n}"),record:record()}).collect::<Vec<_>>()});
    assert!(World::from_json(&too_many.to_string()).is_err());
}
