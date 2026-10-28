use dlms_learning_lab::{cron::*, shell};
use std::collections::{BTreeMap, BTreeSet};
fn moment(s: &str) -> Moment {
    Moment::parse(s).unwrap()
}
fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|s| (*s).into()).collect()
}
fn base() -> State {
    State {
        clock: moment("2026-10-28 08:59"),
        users: set(&["root", "alex", "sam"]),
        tables: BTreeMap::new(),
        system_table: String::new(),
        allow: Some(set(&["alex"])),
        deny: None,
        files: BTreeMap::from([
            ("/work/source".into(), "cobalt\n".into()),
            ("/work/peer".into(), "Keep exactly\n".into()),
        ]),
    }
}
fn complete(state: &State, end: Moment) -> Result<Completed> {
    let mut run = Run::begin(state, end)?;
    for _ in 0..200_000 {
        if run.done() {
            let result = run.take_result().unwrap();
            assert!(run.done());
            return result;
        }
        run.tick(64);
    }
    panic!("Bounded scheduling simulation did not finish");
}
fn schedule(s: &str) -> Schedule {
    let v: Vec<_> = s.split_whitespace().collect();
    Schedule::parse(v.try_into().unwrap()).unwrap()
}

#[test]
fn explicit_calendar_validation_and_real_boundaries() {
    for bad in [
        "2026-02-29 00:00",
        "2100-01-01 00:00",
        "1999-12-31 23:59",
        "2026-00-01 00:00",
        "2026-04-31 00:00",
        "2026-01-00 00:00",
        "2026-01-01 24:00",
        "2026-01-01 00:60",
        "2026-+1-01 00:00",
        "2026-01-01 0:00",
        "2026-01-01T00:00",
        "２０２６-01-01 00:00",
    ] {
        assert!(Moment::parse(bad).is_err(), "{bad}");
    }
    for (before, after) in [
        ("2000-02-28 23:59", "2000-02-29 00:00"),
        ("2000-02-29 23:59", "2000-03-01 00:00"),
        ("2026-02-28 23:59", "2026-03-01 00:00"),
        ("2026-12-31 23:59", "2027-01-01 00:00"),
    ] {
        assert_eq!(moment(before).next().unwrap(), moment(after));
    }
    assert!(moment("2099-12-31 23:59").next().is_err());
    assert_eq!(moment("2000-01-01 00:00").weekday().unwrap(), 6);
    assert_eq!(moment("2000-02-29 00:00").weekday().unwrap(), 2);
    assert_eq!(moment("2026-10-28 00:00").weekday().unwrap(), 3);
}
#[test]
fn every_supported_day_advances_and_weekday_is_consistent() {
    // Independent day-by-day weekday counter, rather than repeating ordinal math.
    let mut weekday = 6;
    let mut count = 0;
    let mut previous: Option<Moment> = None;
    for year in 2000u16..=2099 {
        for (month, length) in [
            31,
            28 + u8::from(year.is_multiple_of(4)),
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ]
        .into_iter()
        .enumerate()
        {
            for day in 1..=length {
                let m = Moment {
                    year,
                    month: month as u8 + 1,
                    day,
                    hour: 0,
                    minute: 0,
                };
                assert_eq!(m.weekday().unwrap(), weekday);
                assert_eq!(Moment::parse(&m.text()).unwrap(), m);
                if let Some(last) = previous {
                    assert_eq!(last.next().unwrap(), m);
                }
                previous = Some(Moment {
                    hour: 23,
                    minute: 59,
                    ..m
                });
                count += 1;
                weekday = (weekday + 1) % 7;
            }
        }
    }
    assert_eq!(count, 36525);
}
#[test]
fn calendar_fields_names_lists_steps_and_sunday_aliases() {
    let s = schedule("*/15 9-11/2 * OCT,nov Mon-Fri");
    for (time, wanted) in [
        ("2026-10-28 09:00", true),
        ("2026-10-28 09:15", true),
        ("2026-10-28 09:14", false),
        ("2026-10-28 10:00", false),
        ("2026-10-28 11:45", true),
        ("2026-10-31 09:00", false),
        ("2026-12-01 09:00", false),
    ] {
        assert_eq!(s.matches(moment(time)).unwrap(), wanted, "{time}");
    }
    let sun0 = schedule("0 0 * * 0");
    let sun7 = schedule("0 0 * * 7");
    let named = schedule("0 0 * * SUN");
    for date in ["2026-11-01 00:00", "2026-11-02 00:00", "2000-01-02 00:00"] {
        let m = moment(date);
        assert_eq!(sun0.matches(m), sun7.matches(m));
        assert_eq!(sun7.matches(m), named.matches(m));
    }
    let every35 = schedule("*/35 * * * *");
    for hour in 0..24 {
        for minute in 0..60 {
            assert_eq!(
                every35
                    .matches(Moment {
                        hour,
                        minute,
                        ..moment("2026-10-28 00:00")
                    })
                    .unwrap(),
                matches!(minute, 0 | 35)
            );
        }
    }
}
#[test]
fn wildcard_provenance_cannot_be_replaced_by_numeric_set_equality() {
    let anyday = schedule("0 9 * * 1");
    let fullrange = schedule("0 9 1-31 * 1");
    let wed = moment("2026-10-28 09:00");
    assert!(!anyday.matches(wed).unwrap());
    assert!(fullrange.matches(wed).unwrap());
    let stepped = schedule("0 9 */2 * 1");
    //1,3,... matches odd Monday only with the original wildcard, not all odd days.
    assert!(stepped.matches(moment("2026-11-09 09:00")).unwrap());
    assert!(!stepped.matches(moment("2026-11-11 09:00")).unwrap());
    assert!(!stepped.matches(moment("2026-11-02 09:00")).unwrap());
    let either = schedule("0 9 15 * fri");
    for (date, yes) in [
        ("2026-10-15 09:00", true),
        ("2026-10-16 09:00", true),
        ("2026-10-14 09:00", false),
    ] {
        assert_eq!(either.matches(moment(date)).unwrap(), yes);
    }
    assert!(
        !schedule("0 0 31 feb *")
            .matches(moment("2028-02-29 00:00"))
            .unwrap()
    );
}
#[test]
fn malformed_and_deliberately_unsupported_calendar_fields_do_not_approximate() {
    for source in [
        "60 * * * *",
        "0 24 * * *",
        "0 0 0 * *",
        "0 0 * 13 *",
        "0 0 * * 8",
        "0 0 * * fri-mon",
        "*/0 * * * *",
        "5/2 * * * *",
        "*,1 * * * *",
        "0,,1 * * * *",
        "@daily 0 0 * *",
        "~ * * * *",
        "0 0 * september *",
        "0-59/99999999 * * * *",
    ] {
        let v: Vec<_> = source.split_whitespace().collect();
        assert!(Schedule::parse(v.try_into().unwrap()).is_err(), "{source}");
    }
}
#[test]
fn user_system_and_literal_ordered_environment_are_distinct() {
    let table = "# Example\nTITLE = 'Blue desk'\n0 9 * * mon-fri printf '\\%s\\n' \"$TITLE\"\nTITLE=Amber\n15 9 * * * echo \"${TITLE}\"\n";
    let entries = parse_table(table, Some("alex")).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].owner, "alex");
    assert_eq!(entries[0].line, 3);
    assert_eq!(entries[0].environment["TITLE"], "Blue desk");
    assert_eq!(entries[1].environment["TITLE"], "Amber");
    let literal = parse_table("A=$HOME\n0 9 * * * echo \"$A\"\n", Some("alex")).unwrap();
    assert_eq!(literal[0].environment["A"], "$HOME");
    assert_eq!(
        parse_table("0 9 * * * sam echo ready\n", None).unwrap()[0].owner,
        "sam"
    );
    assert!(parse_table("0 9 * * * sam echo ready\n", Some("alex")).is_err());
    assert!(parse_table("0 9 * * * echo ready\n", None).is_err());
    assert!(
        parse_table("0 9 * * * echo ready", Some("alex"))
            .unwrap_err()
            .message
            .contains("newline")
    );
}
#[test]
fn percent_is_processed_before_shell_quotes_with_backslash_parity() {
    assert_eq!(
        shell::cron::prepare("printf '\\%s\\n' ready").unwrap(),
        "printf '%s\\n' ready"
    );
    assert!(
        shell::cron::prepare("printf '%s\\n' ready")
            .unwrap_err()
            .contains("percent")
    );
    assert!(shell::cron::prepare(r"echo \\%").is_err());
    assert_eq!(shell::cron::prepare(r"echo \\\%").unwrap(), r"echo \\%");
    assert_eq!(shell::cron::prepare("echo '\\%'").unwrap(), "echo '%'");
}
#[test]
fn closed_shell_preflight_reuses_real_parser_but_never_enables_bash_features() {
    for source in [
        "printf '\\%s\\n' \"${LABEL}\" >> /work/report",
        "'pri'\"ntf\" 'Ready\\n' > /work/report",
        "cat /work/source 2>/work/errors",
    ] {
        assert!(shell::cron::prepare(source).is_ok(), "{source}");
    }
    for source in [
        "items=(a b)",
        "echo \"${items[@]}\"",
        "echo \"${LABEL:-fallback}\"",
        "echo \"$((1+2))\"",
        "echo $(cat /work/source)",
        "echo value | cat /work/source",
        "echo a; echo b",
        "if true; then echo x; fi",
        "f() { echo x; }; f",
        "NAME=x printf x",
        "$TOOL x",
        "echo $LABEL",
        "cat /work/*",
        "echo \"unterminated",
        "cd /work",
        "/bin/sh /work/script",
        "curl https://example.test",
        "rm /work/source",
    ] {
        assert!(shell::cron::prepare(source).is_err(), "{source}");
    }
}
#[test]
fn access_policy_controls_management_not_already_installed_jobs() {
    let mut s = base();
    s.install_user("alex", "alex", "0 9 * * * echo ready\n")
        .unwrap();
    let installed = s.tables.clone();
    s.set_access("root", Some(&set(&["sam"])), Some(&set(&["sam"])))
        .unwrap();
    assert!(!s.can_manage("alex", "alex"));
    assert!(s.can_manage("sam", "sam"));
    assert!(!s.can_manage("sam", "alex"));
    assert!(s.can_manage("root", "alex"));
    let before = s.clone();
    assert_eq!(
        s.remove_user("alex", "alex").unwrap_err().kind,
        ErrorKind::Permission
    );
    assert_eq!(s, before);
    let finished = complete(&s, moment("2026-10-28 09:00")).unwrap();
    assert_eq!(finished.records.len(), 1);
    assert_eq!(finished.records[0].stdout, "ready\n");
    assert_eq!(s.tables, installed);
    s.set_access("root", None, Some(&set(&["alex"]))).unwrap();
    assert!(!s.can_manage("alex", "alex"));
    assert!(s.can_manage("sam", "sam"));
    s.set_access("root", None, None).unwrap();
    assert!(!s.can_manage("sam", "sam"));
    s.remove_user("root", "alex").unwrap();
    assert!(
        complete(&s, moment("2026-10-28 09:00"))
            .unwrap()
            .records
            .is_empty()
    );
}
#[test]
fn management_rejections_and_entry_limits_are_atomic_even_equal_jobs() {
    let mut s = base();
    s.install_user("root", "sam", "0 9 * * * echo shared\n")
        .unwrap();
    let before = s.clone();
    for wrong in [
        "0 9 * * * echo missing-newline",
        "0 25 * * * echo bad\n",
        "0 9 * * * printf '%s' wrong\n",
        "@reboot echo wrong\n",
    ] {
        assert!(s.install_user("root", "alex", wrong).is_err());
        assert_eq!(s, before);
    }
    assert!(
        s.install_user("alex", "sam", "0 9 * * * echo no\n")
            .is_err()
    );
    assert_eq!(s, before);
    assert!(s.install_system("sam", "0 9 * * * sam echo no\n").is_err());
    assert_eq!(s, before);
    assert!(
        s.install_system("root", "0 9 * * * absent echo no\n")
            .is_err()
    );
    assert_eq!(s, before);
    assert!(s.set_access("root", Some(&set(&["absent"])), None).is_err());
    assert_eq!(s, before);
    s.install_system("root", &"0 9 * * * sam echo shared\n".repeat(15))
        .unwrap();
    assert_eq!(s.entries().unwrap().len(), 16);
    let full = s.clone();
    assert!(
        s.install_user("root", "alex", "0 9 * * * echo extra\n")
            .is_err()
    );
    assert_eq!(s, full);
    // Replacing one user row still leaves the15 equal system rows distinct.
    s.install_user("root", "sam", "0 10 * * * echo replacement\n")
        .unwrap();
    assert_eq!(s.entries().unwrap().len(), 16);
}
#[test]
fn scheduling_uses_only_explicit_environment_and_real_virtual_file_changes() {
    for (label, input) in [
        ("North desk", "cobalt\n"),
        ("East % desk", "line one\nline two\n"),
        ("Empty", ""),
    ] {
        let mut s = base();
        s.files.insert("/work/source".into(), input.into());
        s.install_user("root","alex",&format!("LABEL='{label}'\n0 9 * * * printf '\\%s\\n' \"$LABEL\" >> /work/report\n1 9 * * * cat /work/source >> /work/report\n")).unwrap();
        let initial = s.clone();
        let finished = complete(&s, moment("2026-10-28 09:01")).unwrap();
        assert_eq!(finished.records.len(), 2);
        assert_eq!(
            finished.state.files["/work/report"],
            format!("{label}\n{input}")
        );
        assert_eq!(finished.state.files["/work/peer"], "Keep exactly\n");
        assert_eq!(s, initial);
    }
    let mut s = base();
    s.install_system("root","0 9 * * * sam printf '\\%s\\n' \"$LOGNAME\" \"$USER\" \"$HOME\" \"$SHELL\" \"$INTERACTIVE_ONLY\"\n").unwrap();
    let finished = complete(&s, moment("2026-10-28 09:00")).unwrap();
    assert_eq!(finished.records[0].owner, "sam");
    assert_eq!(finished.records[0].stdout, "sam\nsam\n/work\n/bin/sh\n\n");
}
#[test]
fn window_excludes_start_includes_end_and_repeated_runs_append_correctly() {
    let mut s = base();
    s.clock = moment("2026-10-28 09:00");
    s.install_user("root", "alex", "* * * * * echo entry >> /work/report\n")
        .unwrap();
    let first = complete(&s, moment("2026-10-28 09:02")).unwrap();
    assert_eq!(
        first
            .records
            .iter()
            .map(|r| r.when.minute)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(first.state.files["/work/report"], "entry\nentry\n");
    let second = complete(&first.state, moment("2026-10-28 09:03")).unwrap();
    assert_eq!(second.records.len(), 1);
    assert_eq!(second.state.files["/work/report"], "entry\nentry\nentry\n");
    assert!(Run::begin(&s, s.clock).is_err());
    assert!(Run::begin(&s, moment("2026-10-28 08:00")).is_err());
    assert!(s.clock.after(MAX_WINDOW_MINUTES + 1).is_err());
}
#[test]
fn ordinary_command_failure_is_not_scheduling_or_parser_success() {
    let mut s = base();
    s.install_user(
        "root",
        "alex",
        "0 9 * * * cat /work/missing > /work/report\n",
    )
    .unwrap();
    let finished = complete(&s, moment("2026-10-28 09:00")).unwrap();
    assert_eq!(finished.records[0].status, 1);
    assert!(finished.records[0].stderr.contains("No such file"));
    assert_eq!(finished.state.files["/work/report"], "");
    // Host-looking paths are rejected by the existing bounded VM, never opened.
    s.install_user("root", "alex", "0 9 * * * cat /etc/passwd\n")
        .unwrap();
    let before = s.clone();
    assert!(complete(&s, moment("2026-10-28 09:00")).is_err());
    assert_eq!(s, before);
    s.install_user("root", "alex", "0 9 * * * cat /work/../personal\n")
        .unwrap();
    assert!(complete(&s, moment("2026-10-28 09:00")).is_err());
}
#[test]
fn cancellation_dispatch_output_and_file_limits_do_not_commit_partial_state() {
    let mut s = base();
    s.install_user("root", "alex", "* * * * * echo entry >> /work/report\n")
        .unwrap();
    let before = s.clone();
    let mut run = Run::begin(&s, s.clock.after(1440).unwrap()).unwrap();
    run.tick(usize::MAX);
    assert!(!run.done());
    run.cancel();
    assert_eq!(
        run.take_result().unwrap().unwrap_err().kind,
        ErrorKind::Cancelled
    );
    assert!(run.done());
    assert_eq!(s, before);
    assert_eq!(
        complete(&s, s.clock.after(65).unwrap()).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
    s.files.insert("/work/source".into(), "x".repeat(32768));
    s.install_user("root", "alex", "* * * * * cat /work/source\n")
        .unwrap();
    let before = s.clone();
    assert_eq!(
        complete(&s, s.clock.after(3).unwrap()).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(s, before);
}
#[test]
fn unsupported_environment_and_bounded_json_fail_honestly() {
    for assignment in [
        "PATH=/usr/local/bin",
        "SHELL=/bin/bash",
        "HOME=/home/alex",
        "LOGNAME=root",
        "USER=root",
        "MAILTO=alex",
        "CRON_TZ=UTC",
        "RANDOM_DELAY=5",
        "RANDOM=5",
        "OPTIND=2",
        "IFS=,",
    ] {
        assert_eq!(
            parse_table(&format!("{assignment}\n0 9 * * * echo no\n"), Some("alex"))
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
    let s = base();
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(State::from_json(&json).unwrap(), s);
    let duplicate = json.replacen(
        "\"tables\":{}",
        "\"tables\":{\"alex\":\"\",\"alex\":\"\"}",
        1,
    );
    assert_ne!(json, duplicate);
    assert!(State::from_json(&duplicate).is_err());
    let duplicate_user = json.replacen(
        "\"users\":[\"alex\",\"root\",\"sam\"]",
        "\"users\":[\"alex\",\"root\",\"sam\",\"alex\"]",
        1,
    );
    assert_ne!(json, duplicate_user);
    assert!(State::from_json(&duplicate_user).is_err());
    assert!(
        State::from_json(&json.replacen(
            "\"system_table\":\"\"",
            "\"system_table\":\"\",\"unknown\":true",
            1
        ))
        .is_err()
    );
    assert!(State::from_json(&" ".repeat(MAX_JSON_BYTES + 1)).is_err());
    let mut bad = s.clone();
    bad.files.insert("/work/../escape".into(), "x".into());
    assert!(bad.validate().is_err());
    bad = s.clone();
    bad.users.extend((0..8).map(|n| format!("person{n}")));
    assert!(bad.validate().is_err());
}

// Exercise direct derived deserialization too: packs/progress do not need to
// call State::from_json. Limits must hold before making an owned oversized value.
#[test]
fn derived_state_admission_bounds_text_and_rejects_excess_before_value_decode() {
    let original = serde_json::to_value(base()).unwrap();
    let reject = |value: serde_json::Value, reason: &str| {
        let raw = serde_json::to_string(&value).unwrap();
        let error = serde_json::from_str::<State>(&raw).unwrap_err().to_string();
        assert!(error.contains(reason), "{error}; expected {reason}");
    };
    let mut system = original.clone();
    system["system_table"] = "x".repeat(MAX_TABLE_BYTES).into();
    assert_eq!(
        serde_json::from_value::<State>(system.clone())
            .unwrap()
            .system_table
            .len(),
        MAX_TABLE_BYTES
    );
    system["system_table"] = "x".repeat(MAX_TABLE_BYTES + 1).into();
    reject(system, "text byte limit");
    for (field, key, bytes) in [
        ("tables", "alex", MAX_TABLE_BYTES),
        ("files", "/work/source", 32768),
    ] {
        let mut value = original.clone();
        value[field] = serde_json::json!({(key): "x".repeat(bytes)});
        serde_json::from_value::<State>(value.clone()).unwrap();
        value[field][key] = "x".repeat(bytes + 1).into();
        reject(value.clone(), "text byte limit");
        value[field] = serde_json::json!({("k".repeat(257)): ""});
        reject(value, "text byte limit");
    }
    let mut users = original.clone();
    users["users"] = serde_json::json!(["root", "aa", "ab", "ac", "ad", "ae", "af", "ag"]);
    serde_json::from_value::<State>(users.clone()).unwrap();
    users["users"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"not": "an account"}));
    reject(users, "account count limit");
    let mut oversized_name = original.clone();
    oversized_name["users"] = serde_json::json!(["root", "a".repeat(33)]);
    reject(oversized_name, "text byte limit");
    for (field, count) in [("tables", MAX_USERS), ("files", MAX_FILES)] {
        let mut value = original.clone();
        let map: serde_json::Map<String, serde_json::Value> = (0..count)
            .map(|i| (format!("k{i:02}"), serde_json::json!("")))
            .collect();
        value[field] = map.into();
        serde_json::from_value::<State>(value.clone()).unwrap();
        // Ordered last, its non-text value must never be decoded as String.
        value[field]["zz_excess"] = serde_json::json!({"wrong": [1,2,3]});
        reject(value, "map count limit");
    }
    // A duplicate key is rejected before trying to decode its malformed value.
    let json = serde_json::to_string(&base()).unwrap();
    let duplicate = json.replacen(
        "\"tables\":{}",
        "\"tables\":{\"alex\":\"\",\"alex\":{\"wrong\":true}}",
        1,
    );
    assert_ne!(duplicate, json);
    assert!(
        serde_json::from_str::<State>(&duplicate)
            .unwrap_err()
            .to_string()
            .contains("Duplicate scheduling map key")
    );
    // UTF-8 bytes, rather than scalar count, enforce the same allocation budget.
    let mut unicode = original;
    unicode["system_table"] = "é".repeat(MAX_TABLE_BYTES / 2 + 1).into();
    reject(unicode, "text byte limit");
}

#[test]
fn unsupported_control_characters_in_virtual_names_fail_at_admission() {
    let mut state = base();
    state
        .files
        .insert(format!("/work/{}", "x".repeat(250)), "".into());
    state.validate().unwrap();
    for control in ['\0', '\n', '\r', '\t', '\u{7f}', '\u{85}'] {
        let mut bad = base();
        bad.files
            .insert(format!("/work/a{control}b"), "retained".into());
        let before = bad.clone();
        let error = bad.validate().unwrap_err();
        assert_eq!(error.kind, ErrorKind::Invalid);
        assert!(error.message.contains("without control characters"));
        assert_eq!(bad, before);
        assert!(Run::begin(&bad, moment("2026-10-28 09:00")).is_err());
    }
}
