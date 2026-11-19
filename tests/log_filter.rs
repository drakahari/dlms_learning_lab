use dlms_learning_lab::log_filter::*;
fn lines(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}
fn parse(pattern: &str) -> Filter {
    Filter::parse(&format!("[Definition]\nfailregex = {pattern}\n")).unwrap()
}
#[test]
fn actual_lines_are_classified_and_keep_input_exact() {
    let filter = parse(r"^portal rejected user=[a-z]+ from <HOST> port [0-9]+$");
    let source = lines(&[
        "portal rejected user=alex from 192.0.2.9 port 53142",
        "portal accepted user=alex from 192.0.2.9 port 53143",
        "portal rejected user=mira from 203.0.113.5 port 122",
        "ordinary message",
    ]);
    let before = source.clone();
    let result = inspect(&filter, &source).unwrap();
    assert_eq!(source, before);
    assert_eq!(
        result.iter().map(|r| r.disposition).collect::<Vec<_>>(),
        vec![
            Disposition::Matched,
            Disposition::Missed,
            Disposition::Matched,
            Disposition::Missed
        ]
    );
    assert_eq!(result[2].address, Some("203.0.113.5".parse().unwrap()));
    assert_eq!(result[2].line, 3);
    assert_eq!(result[2].text, source[2]);
}
#[test]
fn equivalent_common_ascii_patterns_can_classify_the_same_data() {
    let source = lines(&[
        "portal rejected user=alex from 192.0.2.9 port 53142",
        "portal accepted user=mira from 203.0.113.5 port 122",
    ]);
    let a = parse(r"^portal rejected user=[a-z]+ from <HOST> port [0-9]+$");
    let b = parse(r"^portal (rejected) user=[a-z][a-z]* from <HOST> port [0-9][0-9]*$");
    assert_eq!(inspect(&a, &source).unwrap(), inspect(&b, &source).unwrap());
}
#[test]
fn ignore_changes_actual_observations_and_filter_edits_clear_prior_assumptions() {
    let source = lines(&[
        "portal rejected user=probe from 192.0.2.9 port 44",
        "portal rejected user=mira from 203.0.113.5 port 122",
    ]);
    let a = parse(r"^portal rejected user=[a-z]+ from <HOST> port [0-9]+$");
    let mut b = a.clone();
    b.ignore = Some(r"^portal rejected user=probe ".into());
    assert_eq!(
        inspect(&a, &source).unwrap()[0].disposition,
        Disposition::Matched
    );
    let result = inspect(&b, &source).unwrap();
    assert_eq!(result[0].disposition, Disposition::Ignored);
    assert_eq!(result[1].disposition, Disposition::Matched);
    b.fail = "^portal accepted user=[a-z]+ from <HOST> port [0-9]+$".into();
    assert_eq!(
        inspect(&b, &source).unwrap()[1].disposition,
        Disposition::Missed
    );
}
#[test]
fn unsupported_python_or_posix_extensions_are_diagnosed_before_results() {
    for p in [
        r"^(?i)portal from <HOST>$",
        r"^portal \w+ from <HOST>$",
        r"^portal [[:alpha:]]+ from <HOST>$",
        r"^portal .*? from <HOST>$",
        r"^portal ([a-z]+) \1 from <HOST>$",
        r"^portal from [<HOST>]$",
        r"^portal from <HOST>?$",
    ] {
        let result = Filter::parse(&format!("[Definition]\nfailregex={p}\n"));
        assert!(result.is_err(), "Must refuse {p}");
        assert!(matches!(result.unwrap_err().kind, Kind::Unsupported));
    }
}
#[test]
fn config_does_not_execute_or_ignore_unknown_settings() {
    for source in [
        "[INCLUDES]\nbefore=/etc/passwd\n",
        "[Definition]\nfailregex=from <HOST>\nactionban=touch /tmp/pwn\n",
        "[Definition]\nfailregex=from <HOST>\nignorecommand=exit 0\n",
        "[Definition]\nfailregex=from <HOST>\ndatepattern=Epoch\n",
        "[Definition]\nfailregex=from <HOST>\nfailregex=other <HOST>\n",
    ] {
        assert!(Filter::parse(source).is_err());
    }
    assert!(
        Filter::parse("[Definition]\nfailregex=missing address tag\n")
            .unwrap_err()
            .detail
            .contains("<HOST>")
    );
    assert!(
        Filter::parse("failregex=from <HOST>\n")
            .unwrap_err()
            .detail
            .contains("[Definition]")
    );
}
#[test]
fn malformed_and_ambiguous_logs_refuse_without_partial_results() {
    let filter = parse(r"^portal rejected user=[a-z]+ from <HOST>$");
    for source in [
        lines(&["portal rejected user=a from 192.0.2.9", "bad\nline"]),
        lines(&[
            "portal rejected user=a from 192.0.2.9",
            "both 192.0.2.3 and 203.0.113.9",
        ]),
        lines(&["nonascii é from 192.0.2.9"]),
    ] {
        assert!(inspect(&filter, &source).is_err());
    }
    let invalid = lines(&[
        "portal rejected user=a from 999.0.2.9",
        "portal rejected user=a from 192x0x2x9",
        "portal rejected user=a from 192.0.2.90suffix",
    ]);
    let result = inspect(&filter, &invalid).unwrap();
    assert!(result.iter().all(|r| r.disposition == Disposition::Missed));
}
#[test]
fn whole_report_limits_and_public_structs_cannot_skip_validation() {
    let filter = parse("from <HOST>");
    assert_eq!(
        inspect(&filter, &vec!["from 192.0.2.9".into(); 65])
            .unwrap_err()
            .kind,
        Kind::Limit
    );
    assert_eq!(
        inspect(&filter, &["x".repeat(513)]).unwrap_err().kind,
        Kind::Limit
    );
    let bypass = Filter {
        fail: ".*".into(),
        ignore: None,
    };
    assert!(inspect(&bypass, &lines(&["from 192.0.2.9"])).is_err());
    assert_eq!(
        Filter::parse(&"x".repeat(2049)).unwrap_err().kind,
        Kind::Limit
    );
}

#[test]
fn address_tag_cannot_be_optional_bypassed_or_capture_only_part_of_a_token() {
    for pattern in [
        r"^portal (from <HOST>)?$",
        r"^portal from <HOST>|ordinary message$",
        r"^portal from (<HOST>)$",
        r"^portal from <HOST>0$",
        r"^portal from <HOST>[0-9]$",
        r"^portal from [0-9]<HOST>$",
    ] {
        let result = Filter::parse(&format!("[Definition]\nfailregex={pattern}\n"));
        assert!(
            result.is_err(),
            "Unsupported capture shape must refuse: {pattern}"
        );
        assert_eq!(result.unwrap_err().kind, Kind::Unsupported);
    }
    // Ordinary alternation for an event still allows equivalent real patterns;
    // every successful path requires the complete, separated address token.
    let filter = parse(r"^portal (failed|rejected) from <HOST> port [0-9]+$");
    assert_eq!(
        inspect(&filter, &lines(&["portal rejected from 192.0.2.9 port 23"])).unwrap()[0]
            .disposition,
        Disposition::Matched
    );
}
