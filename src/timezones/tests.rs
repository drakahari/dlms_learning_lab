//! Finite-calendar, zone, command and isolation regressions.
use super::*;
fn epoch(y: i32, m: u8, d: u8, h: u8, min: u8, s: u8) -> i64 {
    Civil::new(y, m, d, h, min, s).unwrap().epoch().unwrap()
}
#[test]
fn bounded_calendar_roundtrips_month_edges_and_leap_years() {
    for y in 1970..=2100 {
        for m in 1..=12 {
            for d in [1, month_days(y, m)] {
                let c = Civil::new(y, m, d, 23, 59, 59).unwrap();
                assert_eq!(Civil::from_epoch(c.epoch().unwrap()).unwrap(), c);
            }
        }
    }
    assert!(Civil::new(2026, 2, 29, 0, 0, 0).is_err());
    assert!(Civil::new(2024, 2, 29, 0, 0, 0).is_ok());
    assert!(Civil::new(2100, 2, 29, 0, 0, 0).is_err());
    assert!(Civil::new(2000, 2, 29, 0, 0, 0).is_ok());
    for bad in [
        (2026, 0, 1, 0, 0, 0),
        (2026, 13, 1, 0, 0, 0),
        (2026, 4, 31, 0, 0, 0),
        (2026, 1, 0, 0, 0, 0),
        (2026, 1, 1, 24, 0, 0),
        (2026, 1, 1, 0, 60, 0),
        (2026, 1, 1, 0, 0, 60),
    ] {
        assert!(Civil::new(bad.0, bad.1, bad.2, bad.3, bad.4, bad.5).is_err());
    }
}
#[test]
fn only_declared2026_utc_instants_are_admitted_without_overflow() {
    assert_eq!(epoch(2026, 1, 1, 0, 0, 0), FIRST);
    assert_eq!(epoch(2027, 1, 1, 0, 0, 0), AFTER);
    for z in Zone::ALL {
        for e in [i64::MIN, -1, 0, FIRST - 1, AFTER, i64::MAX] {
            assert!(z.at(e).is_err());
        }
        assert!(z.at(FIRST).is_ok());
        assert!(z.at(AFTER - 1).is_ok());
    }
}
#[test]
fn spring_and_autumn_transitions_change_at_the_exact_utc_second() {
    for (zone, start, end, before, after) in [
        (
            Zone::Berlin,
            epoch(2026, 3, 29, 1, 0, 0),
            epoch(2026, 10, 25, 1, 0, 0),
            3600,
            7200,
        ),
        (
            Zone::NewYork,
            epoch(2026, 3, 8, 7, 0, 0),
            epoch(2026, 11, 1, 6, 0, 0),
            -18000,
            -14400,
        ),
    ] {
        assert_eq!(zone.at(start - 1).unwrap().offset, before);
        assert_eq!(zone.at(start).unwrap().offset, after);
        assert_eq!(zone.at(end - 1).unwrap().offset, after);
        assert_eq!(zone.at(end).unwrap().offset, before);
    }
    let before = Zone::NewYork.at(epoch(2026, 3, 8, 6, 59, 59)).unwrap();
    let after = Zone::NewYork.at(epoch(2026, 3, 8, 7, 0, 0)).unwrap();
    assert_eq!(before.civil.iso(), "2026-03-08 01:59:59");
    assert_eq!(after.civil.iso(), "2026-03-08 03:00:00");
}
#[test]
fn repeated_wall_times_have_different_offsets_and_half_hour_zone_crosses_dates() {
    let a = Zone::NewYork.at(epoch(2026, 11, 1, 5, 30, 0)).unwrap();
    let b = Zone::NewYork.at(epoch(2026, 11, 1, 6, 30, 0)).unwrap();
    assert_eq!(a.civil, b.civil);
    assert_ne!(a.numeric_offset(false), b.numeric_offset(false));
    let india = Zone::Kolkata.at(epoch(2026, 1, 1, 23, 45, 0)).unwrap();
    assert_eq!(india.civil.iso(), "2026-01-02 05:15:00");
    assert_eq!(india.numeric_offset(false), "+0530");
    assert_eq!(india.numeric_offset(true), "+05:30");
    assert_eq!(
        Zone::NewYork.at(FIRST).unwrap().civil.iso(),
        "2025-12-31 19:00:00"
    );
    assert_eq!(
        Zone::Kolkata.at(AFTER - 1).unwrap().civil.iso(),
        "2027-01-01 05:29:59"
    );
}
#[test]
fn changing_display_zone_never_changes_the_represented_instant() {
    for e in [
        FIRST,
        FIRST + 12_345,
        epoch(2026, 7, 4, 23, 59, 59),
        AFTER - 1,
    ] {
        for z in Zone::ALL {
            let local = z.at(e).unwrap();
            assert_eq!(local.civil.epoch().unwrap() - i64::from(local.offset), e);
        }
    }
}
#[test]
fn empty_tz_means_utc_but_unset_uses_the_configured_zone() {
    let mut env = Environment { tz: None };
    assert_eq!(env.effective(Zone::Kolkata).unwrap(), Zone::Kolkata);
    env.tz = Some(String::new());
    assert_eq!(env.effective(Zone::Kolkata).unwrap(), Zone::Utc);
    env.tz = Some("America/New_York".into());
    assert_eq!(env.effective(Zone::Kolkata).unwrap(), Zone::NewYork);
    for value in [
        "../etc/localtime",
        "/etc/passwd",
        ":/etc/passwd",
        "UTC0",
        "$(id)",
        "Other/Unlisted",
        "UTC\0",
    ] {
        env.tz = Some(value.into());
        assert!(env.validate().is_err());
        assert!(env.effective(Zone::Utc).is_err());
    }
}
#[test]
fn date_formatting_uses_actual_calendar_offset_and_original_epoch() {
    let e = FIRST;
    assert_eq!(
        format::render(Zone::Utc, e, format::DEFAULT).unwrap(),
        "Thu Jan  1 00:00:00 UTC 2026"
    );
    assert_eq!(
        format::render(Zone::Berlin, e, "%F %T %z %Z").unwrap(),
        "2026-01-01 01:00:00 +0100 CET"
    );
    assert_eq!(
        format::render(Zone::Kolkata, e, "%Y/%m/%d %H:%M:%S %:z").unwrap(),
        "2026/01/01 05:30:00 +05:30"
    );
    assert_eq!(
        format::render(Zone::NewYork, e, "%s %% %R%n").unwrap(),
        format!("{FIRST} % 19:00\n")
    );
    for bad in ["%", "%Q", "%::z", "%-d", "%123456Y", "%c", "%x", "%N", "\0"] {
        assert!(format::render(Zone::Utc, e, bad).is_err());
    }
    assert!(format::render(Zone::Utc, e, &"x".repeat(257)).is_err());
}
#[test]
fn real_parsed_date_forms_evaluate_the_same_instant_without_mutating_the_clock() {
    let mut w =
        command::World::new("Europe/Berlin".into(), Environment { tz: None }, FIRST).unwrap();
    let original = w.clone();
    assert_eq!(
        w.command("date").unwrap().stdout,
        "Thu Jan  1 01:00:00 CET 2026\n"
    );
    for raw in [
        "date -u '+%F %T %z %Z'",
        "date --utc --date=@1767225600 '+%F %T %z %Z'",
        "date '+%F %T %z %Z' -d @1767225600 --universal",
        "date -u -d@1767225600 -- '+%F %T %z %Z'",
    ] {
        assert_eq!(
            w.command(raw).unwrap().stdout,
            "2026-01-01 00:00:00 +0000 UTC\n"
        );
        assert_eq!(w, original);
    }
    assert_eq!(
        w.command("date -d @1767311999 '+%F %T'").unwrap().stdout,
        "2026-01-02 00:59:59\n"
    );
    assert_eq!(w.clock(), FIRST);
}
#[test]
fn configured_zone_process_override_and_utc_view_remain_distinct() {
    let mut w = command::World::new("UTC".into(), Environment { tz: None }, FIRST).unwrap();
    assert_eq!(w.command("printenv TZ").unwrap().status, 1);
    w.command("timedatectl set-timezone Europe/Berlin").unwrap();
    assert_eq!(
        w.command("readlink /etc/localtime").unwrap().stdout,
        "../usr/share/zoneinfo/Europe/Berlin\n"
    );
    for raw in [
        "timedatectl show -p Timezone --value",
        "timedatectl show --value --property=Timezone",
        "timedatectl show -pTimezone --value",
    ] {
        assert_eq!(w.command(raw).unwrap().stdout, "Europe/Berlin\n");
    }
    assert_eq!(
        w.command("timedatectl show --property Timezone")
            .unwrap()
            .stdout,
        "Timezone=Europe/Berlin\n"
    );
    w.command("export TZ=Asia/Kolkata").unwrap();
    assert_eq!(w.command("printenv TZ").unwrap().stdout, "Asia/Kolkata\n");
    assert_eq!(
        w.command("date '+%F %T %z'").unwrap().stdout,
        "2026-01-01 05:30:00 +0530\n"
    );
    assert_eq!(
        w.command("date -u '+%F %T %z'").unwrap().stdout,
        "2026-01-01 00:00:00 +0000\n"
    );
    assert_eq!(w.environment().tz.as_deref(), Some("Asia/Kolkata"));
    w.command("export TZ=''").unwrap();
    assert_eq!(w.command("printenv TZ").unwrap().stdout, "\n");
    assert_eq!(w.command("date +%z").unwrap().stdout, "+0000\n");
    w.command("unset TZ").unwrap();
    assert_eq!(w.command("date +%z").unwrap().stdout, "+0100\n");
    assert_eq!(w.fresh_environment().tz, None);
    assert_eq!(w.clock(), FIRST);
}
#[test]
fn unsupported_commands_formats_paths_and_scripts_never_mutate_virtual_state() {
    let mut w =
        command::World::new("Europe/Berlin".into(), Environment { tz: None }, FIRST).unwrap();
    let old = w.clone();
    for raw in [
        "",
        "date --set='2026-06-01'",
        "date -d tomorrow",
        "date --date=@1798761600",
        "date --date=@-1",
        "date +%Q",
        "date +%-d",
        "date +%::z",
        "date -u -u",
        "date -d @1767225600 -d @1767225601",
        "timedatectl set-timezone ../../etc/passwd",
        "timedatectl show -p NTP",
        "timedatectl show",
        "timedatectl set-ntp true",
        "export TZ=Other/Unknown",
        "export HOME=/etc",
        "unset HOME",
        "printenv HOME",
        "readlink /etc/shadow",
        "sh script.sh",
        "while true; do date; done",
        "date; id",
        "date $(id)",
        "date > /etc/localtime",
        "cat /etc/localtime",
        "ln -sfn /etc/passwd /etc/localtime",
    ] {
        assert!(w.command(raw).is_err(), "{raw}");
        assert_eq!(w, old, "{raw}");
    }
    assert!(w.command(&"date ".repeat(300)).is_err());
    assert_eq!(w, old);
}
