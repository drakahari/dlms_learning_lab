use dlms_learning_lab::ids_rule::{Decision, Event, Fault, Protocol, Rule, inspect};

fn event(source: &str, sp: u16, destination: &str, dp: u16, payload: &str) -> Event {
    Event {
        protocol: Protocol::Tcp,
        source: source.parse().unwrap(),
        source_port: sp,
        destination: destination.parse().unwrap(),
        destination_port: dp,
        payload: payload.into(),
    }
}
fn rule(header: &str, options: &str) -> Rule {
    Rule::parse(&format!("alert {header} (sid:1000001; {options})")).unwrap()
}

#[test]
fn header_decisions_are_independent_of_messages_and_rule_ids() {
    let r = rule(
        "tcp 192.0.2.0/24 any -> 198.51.100.20 8080",
        "msg:\"Example\";",
    );
    let yes = event("192.0.2.4", 40000, "198.51.100.20", 8080, "hello");
    let mut udp = yes.clone();
    udp.protocol = Protocol::Udp;
    let mut wrong_port = yes.clone();
    wrong_port.destination_port = 80;
    let mut wrong_source = yes.clone();
    wrong_source.source = "192.0.3.4".parse().unwrap();
    assert_eq!(
        inspect(&r, &[yes, udp, wrong_port, wrong_source]).unwrap(),
        vec![
            Decision::Alert,
            Decision::ProtocolMismatch,
            Decision::EndpointMismatch,
            Decision::EndpointMismatch
        ]
    );
}

#[test]
fn bidirectional_headers_reverse_whole_endpoint_pairs_only() {
    let bidirectional = rule("tcp 192.0.2.7 4000 <> 198.51.100.9 8080", "");
    let forward = event("192.0.2.7", 4000, "198.51.100.9", 8080, "");
    let reverse = event("198.51.100.9", 8080, "192.0.2.7", 4000, "");
    let mixed = event("198.51.100.9", 4000, "192.0.2.7", 8080, "");
    assert_eq!(
        inspect(&bidirectional, &[forward, reverse.clone(), mixed]).unwrap(),
        vec![Decision::Alert, Decision::Alert, Decision::EndpointMismatch]
    );
    let one_way = rule("tcp 192.0.2.7 4000 -> 198.51.100.9 8080", "");
    assert_eq!(
        inspect(&one_way, &[reverse]).unwrap(),
        vec![Decision::EndpointMismatch]
    );
}

#[test]
fn cidr_membership_normalizes_host_bits_and_crosses_octet_boundaries() {
    let cases = [
        ("0.0.0.0/0", "0.0.0.0", true),
        ("203.0.113.255/0", "255.255.255.255", true),
        ("10.23.47.91/20", "10.23.32.0", true),
        ("10.23.47.91/20", "10.23.47.255", true),
        ("10.23.47.91/20", "10.23.31.255", false),
        ("10.23.47.91/20", "10.23.48.0", false),
        ("192.0.2.9/31", "192.0.2.8", true),
        ("192.0.2.9/31", "192.0.2.9", true),
        ("192.0.2.9/31", "192.0.2.10", false),
        ("192.0.2.9/32", "192.0.2.9", true),
        ("192.0.2.9/32", "192.0.2.8", false),
        ("10.8.99.22/16", "10.8.0.0", true),
        ("10.8.99.22/16", "10.9.0.0", false),
    ];
    for (network, source, expected) in cases {
        let r = rule(&format!("tcp {network} any -> any any"), "");
        assert_eq!(
            inspect(&r, &[event(source, 1, "198.51.100.1", 2, "")]).unwrap()[0] == Decision::Alert,
            expected,
            "{network} / {source}"
        );
    }
}

#[test]
fn literal_checks_are_all_required_case_sensitive_and_not_regexes() {
    let r = rule(
        "tcp any any -> any any",
        "content:\"A.B\"; content:\"token\", nocase;",
    );
    let inputs = [
        "A.B TOKEN",
        "token A.B",
        "AxB token",
        "a.b token",
        "A.B",
        "token",
    ];
    let events: Vec<_> = inputs
        .iter()
        .map(|p| event("192.0.2.1", 1, "198.51.100.1", 2, p))
        .collect();
    assert_eq!(
        inspect(&r, &events).unwrap(),
        vec![
            Decision::Alert,
            Decision::Alert,
            Decision::MissingContent(0),
            Decision::MissingContent(0),
            Decision::MissingContent(1),
            Decision::MissingContent(0)
        ]
    );
}

#[test]
fn quotes_semicolons_backslashes_and_hashes_are_literal() {
    let r = Rule::parse(concat!(
        "# full-line comment\nalert tcp any any -> any any (",
        r#"msg:"A # message\; with \"quotes\""; content:"x\;\"\\#"; sid:1000002; rev:2;"#,
        ")"
    ))
    .unwrap();
    assert_eq!(r.message.as_deref(), Some("A # message; with \"quotes\""));
    assert_eq!(r.revision, Some(2));
    assert_eq!(
        inspect(
            &r,
            &[event("192.0.2.1", 0, "198.51.100.1", 65535, "x;\"\\#")]
        )
        .unwrap(),
        vec![Decision::Alert]
    );
}

#[test]
fn equivalent_whitespace_option_order_and_udp_are_supported() {
    let r = Rule::parse("  alert\tudp any 0\n -> 198.51.100.1 65535 ( content: \"query\" , nocase ; rev: 1 ; sid: 1000003 ; )\n").unwrap();
    let mut e = event("192.0.2.1", 0, "198.51.100.1", 65535, "QUERY");
    e.protocol = Protocol::Udp;
    assert_eq!(inspect(&r, &[e]).unwrap(), vec![Decision::Alert]);
}

#[test]
fn unsupported_headers_options_and_content_never_become_matches() {
    for input in [
        "drop tcp any any -> any any (sid:1;)",
        "alert icmp any any -> any any (sid:1;)",
        "alert tcp $HOME_NET any -> any any (sid:1;)",
        "alert tcp !192.0.2.1 any -> any any (sid:1;)",
        "alert tcp any 80:90 -> any any (sid:1;)",
        "alert tcp any [80,90] -> any any (sid:1;)",
        "alert tcp any any <- any any (sid:1;)",
        "alert tcp any any -> any any (sid:1; flow:established;)",
        "alert tcp any any -> any any (sid:1; service:http;)",
        "alert tcp any any -> any any (sid:1; pcre:\"/x/\";)",
        "alert tcp any any -> any any (sid:1; content:\"|41|\";)",
        "alert tcp any any -> any any (sid:1; content:!\"x\";)",
        "alert tcp any any -> any any (sid:1; content:\"x\", offset 1;)",
        "alert tcp any any -> any any (sid:1; content:\"\";)",
        "alert tcp any any -> any any (sid:1; content:\"\\n\";)",
        "alert tcp any any -> any any (sid:1; content:\"é\";)",
        "alert tcp any any -> any any (sid:1; content:\"x\ny\";)",
    ] {
        assert!(
            matches!(Rule::parse(input), Err(Fault::Unsupported(_))),
            "{input}"
        );
    }
}

#[test]
fn malformed_rules_have_syntax_feedback_instead_of_nonmatch_results() {
    for input in [
        "",
        "alert tcp any any -> any any",
        "alert tcp any -> any any (sid:1;)",
        "alert tcp 999.0.0.1 any -> any any (sid:1;)",
        "alert tcp any nope -> any any (sid:1;)",
        "alert tcp any any -> any any (sid:1)",
        "alert tcp any any -> any any (sid:1;;)",
        "alert tcp any any -> any any (sid:1; sid:2;)",
        "alert tcp any any -> any any (sid:1; msg:\"x\"; msg:\"y\";)",
        "alert tcp any any -> any any (sid:1; rev:1; rev:2;)",
        "alert tcp any any -> any any (sid:1; content:\"unclosed;)",
        "alert tcp any any -> any any (sid:1; content:\"a;b\";)",
        "alert tcp any any -> any any (sid:1;) alert tcp any any -> any any (sid:2;)",
    ] {
        assert!(
            matches!(Rule::parse(input), Err(Fault::Syntax(_))),
            "{input}"
        );
    }
}

#[test]
fn activity_sid_policy_is_explicit_and_not_an_upstream_syntax_claim() {
    for input in [
        "alert tcp any any -> any any ()",
        "alert tcp any any -> any any (sid:0;)",
    ] {
        let Err(Fault::Unsupported(message)) = Rule::parse(input) else {
            panic!("{input}")
        };
        assert!(message.contains("Snort permits omission"));
    }
    assert_eq!(rule("tcp any any -> any any", "").sid, 1000001);
}

#[test]
fn syntax_and_resource_boundaries_refuse_excesses() {
    for input in [
        "alert tcp any 65536 -> any any (sid:1;)".to_owned(),
        "alert tcp 192.0.2.1/33 any -> any any (sid:1;)".into(),
        format!("alert tcp any any -> any any (sid:1; content:\"{}\";)", "x".repeat(129)),
        format!("alert tcp any any -> any any (sid:1; msg:\"{}\";)", "x".repeat(161)),
        "x".repeat(2049),
        "alert tcp any any -> any any (sid:1; content:\"a\";content:\"b\";content:\"c\";content:\"d\";content:\"e\";)".into(),
    ] {
        assert!(matches!(Rule::parse(&input), Err(Fault::Limit(_))), "{input}");
    }
    assert!(matches!(
        Rule::parse("alert tcp any 080 -> any any (sid:1;)"),
        Err(Fault::Unsupported(_))
    ));
    assert!(matches!(
        Rule::parse("alert tcp any any -> any any (sid:1;)\0"),
        Err(Fault::Unsupported(_))
    ));
}

#[test]
fn all_records_are_validated_even_when_earlier_headers_do_not_match() {
    let r = rule("tcp 203.0.113.1 9 -> 203.0.113.2 9", "");
    for payload in ["x".repeat(4097), "\0".into(), "\x1b[31m".into(), "é".into()] {
        let events = vec![
            event("192.0.2.1", 1, "198.51.100.1", 2, "ok"),
            event("192.0.2.1", 1, "198.51.100.1", 2, &payload),
        ];
        assert!(matches!(inspect(&r, &events), Err(Fault::Unsupported(_))));
    }
    let events = vec![event("192.0.2.1", 1, "198.51.100.1", 2, "ok"); 65];
    assert!(matches!(inspect(&r, &events), Err(Fault::Limit(_))));
}

#[test]
fn aggregate_work_limit_returns_no_partial_report_and_is_repeatable() {
    let r = rule(
        "tcp any any -> any any",
        &format!("content:\"{}b\";", "a".repeat(127)),
    );
    let e = event("192.0.2.1", 1, "198.51.100.1", 2, &"a".repeat(4096));
    assert!(matches!(inspect(&r, &[e]), Err(Fault::Limit(_))));
    let short = event("192.0.2.1", 1, "198.51.100.1", 2, "a");
    assert_eq!(
        inspect(&r, &[short]).unwrap(),
        vec![Decision::MissingContent(0)]
    );
    let easy = rule("tcp any any -> any any", "content:\"z\";");
    let e = event("192.0.2.1", 1, "198.51.100.1", 2, &"a".repeat(4096));
    assert!(matches!(inspect(&easy, &vec![e; 64]), Err(Fault::Limit(_))));
}

#[test]
fn maximum_supported_content_count_and_record_payload_sizes_work() {
    let r = rule(
        "tcp any any -> any any",
        "msg:\"Example\";rev:1;content:\"a\";content:\"b\";content:\"c\";content:\"d\";",
    );
    let payload = format!("abcd{}", " ".repeat(4092));
    let e = event("192.0.2.1", 1, "198.51.100.1", 2, &payload);
    assert_eq!(
        inspect(&r, &vec![e; 64]).unwrap(),
        vec![Decision::Alert; 64]
    );
    assert!(inspect(&r, &[]).unwrap().is_empty());
}

#[test]
fn persisted_event_shape_refuses_unknown_fields_and_oversized_text() {
    let e = event("192.0.2.1", 1, "198.51.100.1", 2, "abc");
    let mut value = serde_json::to_value(&e).unwrap();
    assert_eq!(serde_json::from_value::<Event>(value.clone()).unwrap(), e);
    value["host_path"] = serde_json::json!("/etc/passwd");
    assert!(serde_json::from_value::<Event>(value).is_err());
    let mut value = serde_json::to_value(&e).unwrap();
    value["payload"] = serde_json::json!("x".repeat(4097));
    assert!(serde_json::from_value::<Event>(value).is_err());
}
