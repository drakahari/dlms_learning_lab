use dlms_learning_lab::linux_logging as l;
fn paths() -> Vec<String> {
    [
        "/var/log/general.log",
        "/var/log/private.log",
        "/var/log/service.log",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
fn rules(s: &str) -> Vec<l::Rule> {
    l::parse(s, &paths()).unwrap()
}
#[test]
fn classic_threshold_all_eight_boundaries_and_all_facilities() {
    for n in 0..8 {
        let r = rules(&format!("*.{} /var/log/general.log\n", l::SEVERITIES[n]));
        for facility in 0..24 {
            for sev in 0..8 {
                assert_eq!(
                    !l::deliveries(&r, facility, sev).is_empty(),
                    sev as usize <= n
                );
            }
        }
    }
}
#[test]
fn exact_and_all_levels_have_distinct_meanings() {
    for n in 0..8 {
        let r = rules(&format!(
            "daemon.={} /var/log/service.log\n",
            l::SEVERITIES[n]
        ));
        for f in 0..24 {
            for sev in 0..8 {
                assert_eq!(
                    l::deliveries(&r, f, sev).len(),
                    usize::from(f == 3 && sev as usize == n)
                );
            }
        }
    }
    let r = rules("authpriv.* /var/log/private.log");
    for sev in 0..8 {
        assert_eq!(l::deliveries(&r, 10, sev).len(), 1);
    }
}
#[test]
fn semicolon_none_excludes_from_that_rule_only() {
    let r = rules("*.notice;authpriv.none /var/log/general.log\nauthpriv.* /var/log/private.log\n");
    for f in 0..24 {
        for s in 0..8 {
            let expected = if f == 10 {
                vec!["/var/log/private.log".to_string()]
            } else if s <= 5 {
                vec!["/var/log/general.log".to_string()]
            } else {
                vec![]
            };
            assert_eq!(l::deliveries(&r, f, s), expected);
        }
    }
}
#[test]
fn independent_matching_lines_deliver_duplicates_and_order_is_not_first_match() {
    for text in [
        "daemon.err /var/log/general.log\ndaemon.err /var/log/general.log\n",
        "daemon.err /var/log/general.log\n*.err /var/log/general.log\n",
    ] {
        let r = rules(text);
        assert_eq!(
            l::deliveries(&r, 3, 3),
            ["/var/log/general.log", "/var/log/general.log"]
        );
        assert!(l::deliveries(&r, 3, 4).is_empty());
    }
}
#[test]
fn comma_grouping_shares_one_level_without_changing_other_facilities() {
    let r = rules("daemon,cron.=info /var/log/service.log\n");
    for f in 0..24 {
        for s in 0..8 {
            assert_eq!(
                l::deliveries(&r, f, s).len(),
                usize::from([3, 9].contains(&f) && s == 6)
            );
        }
    }
}
#[test]
fn supported_case_spacing_comments_and_crlf_are_equivalent() {
    let a = rules("daemon.warning /var/log/general.log\n");
    for text in [
        "DAEMON.WARNING /var/log/general.log\n",
        " # comment\n\tdaemon.warning\t/var/log/general.log\r\n",
        "\n daemon.warning  /var/log/general.log\n",
    ] {
        assert_eq!(rules(text), a);
    }
}
#[test]
fn unsupported_real_rsyslog_features_fail_honestly_instead_of_approximation() {
    for text in [
        "*.debug;local6.err /var/log/general.log",
        "*.!info /var/log/general.log",
        "*.!=info /var/log/general.log",
        "*.err @192.0.2.5",
        "*.err -/var/log/general.log",
        "& stop",
        "stop",
        "if $msg contains 'x' then /var/log/general.log",
        "module(load=\"imfile\")",
        "$IncludeConfig /etc/rsyslog.d/*.conf",
        "security.info /var/log/general.log",
        "mark.info /var/log/general.log",
        "daemon.error /var/log/general.log",
        "3.4 /var/log/general.log",
    ] {
        assert!(l::parse(text, &paths()).is_err(), "{text}");
    }
}
#[test]
fn malformed_and_control_inputs_never_become_rules() {
    for text in ["", "# only a comment"] {
        assert!(rules(text).is_empty());
    }
    for text in [
        "daemon /var/log/general.log",
        "daemon..err /var/log/general.log",
        "daemon,daemon.err /var/log/general.log",
        "daemon.err /var/log/general.log extra",
        "daemon.err /var/log/general.log\0",
        "daemon.err\u{a0}/var/log/general.log",
        "daemon.err /var/log/general.log\u{1b}",
    ] {
        assert!(l::parse(text, &paths()).is_err(), "{text:?}");
    }
}
#[test]
fn literal_shell_like_message_is_data_and_controls_are_refused() {
    let e = l::Event {
        facility: 3,
        severity: 4,
        message: "$(id); rm /etc/hosts".into(),
    };
    e.validate().unwrap();
    for (f, s, msg) in [
        (24, 0, "x"),
        (0, 8, "x"),
        (0, 0, "line\nnext"),
        (0, 0, "x\0"),
    ] {
        assert!(
            l::Event {
                facility: f,
                severity: s,
                message: msg.into()
            }
            .validate()
            .is_err()
        );
    }
}
#[test]
fn only_supplied_safe_virtual_paths_are_admitted() {
    for p in [
        "/etc/passwd",
        "/var/log/../private",
        "/var/log//x",
        "/var/log/",
        "relative",
        "/var/log/x/./y",
        "/var/log/x;id",
    ] {
        assert!(!l::valid_destination(p));
    }
    assert!(l::parse("*.err /var/log/unprovided", &paths()).is_err());
    assert!(l::parse("", &[]).is_err());
    let mut ps = paths();
    ps.push(ps[0].clone());
    assert!(l::parse("", &ps).is_err());
}
#[test]
fn byte_line_destination_and_event_bounds_are_enforced() {
    assert!(l::parse(&"#".repeat(8193), &paths()).is_err());
    assert!(l::parse(&"# x\n".repeat(33), &paths()).is_err());
    assert_eq!(
        rules(&"daemon.err /var/log/general.log\n".repeat(32)).len(),
        32
    );
    let e = l::Event {
        facility: 1,
        severity: 1,
        message: "a".repeat(257),
    };
    assert!(e.validate().is_err());
}
#[test]
fn matrix_grading_accepts_exact_split_alternatives_and_identifies_missing_or_extra() {
    let policy = [l::Policy {
        destination: "/var/log/general.log".into(),
        facilities: (0..24).collect(),
        severities: (0..=3).collect(),
        copies: 1,
    }];
    let a = rules("*.err /var/log/general.log");
    let b = rules(
        "*.=emerg /var/log/general.log\n*.=alert /var/log/general.log\n*.=crit /var/log/general.log\n*.=err /var/log/general.log",
    );
    assert!(l::compare(&a, &policy).iter().all(|r| r.0));
    assert!(l::compare(&b, &policy).iter().all(|r| r.0));
    for text in [
        "*.=err /var/log/general.log",
        "*.warning /var/log/general.log",
        "*.err /var/log/general.log\n*.err /var/log/general.log",
    ] {
        let rows = l::compare(&rules(text), &policy);
        assert!(rows.iter().any(|r| !r.0));
        assert!(rows[0].1.contains("required") && rows[0].1.contains("actual"));
    }
}
