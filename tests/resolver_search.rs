use dlms_learning_lab::{
    client_names::{Dns, Kind, Status},
    resolver_search::Settings,
};
use std::collections::BTreeMap;
fn dns(rows: &[(&str, &str)]) -> Dns {
    Dns {
        status: Status::Success,
        records: rows
            .iter()
            .map(|(n, ip)| (n.to_string(), ip.parse().unwrap()))
            .collect(),
    }
}
#[test]
fn short_names_try_each_search_suffix_then_as_is_after_definite_misses() {
    let s = Settings::parse("search north.lab.test south.lab.test\n", "client.lab.test").unwrap();
    assert_eq!(s.ndots(), 1);
    assert_eq!(
        s.plan("report").unwrap(),
        ["report.north.lab.test", "report.south.lab.test", "report"]
    );
    let a = s
        .lookup("report", &dns(&[("report.south.lab.test", "192.0.2.80")]))
        .unwrap();
    assert_eq!(a.address, Some("192.0.2.80".parse().unwrap()));
    assert_eq!(a.attempts.len(), 2);
    assert!(a.attempts[0].address.is_none());
}
#[test]
fn ndots_is_a_threshold_not_a_request_for_exactly_that_many_suffixes() {
    for (n, expected) in [
        (0, vec!["app.zone", "app.zone.branch.lab.test"]),
        (1, vec!["app.zone", "app.zone.branch.lab.test"]),
        (2, vec!["app.zone.branch.lab.test", "app.zone"]),
        (15, vec!["app.zone.branch.lab.test", "app.zone"]),
    ] {
        let s = Settings::parse(
            &format!("search branch.lab.test\noptions ndots:{n}\n"),
            "client.lab.test",
        )
        .unwrap();
        assert_eq!(s.plan("app.zone").unwrap(), expected);
    }
}
#[test]
fn an_absolute_final_dot_never_gets_a_suffix_even_if_missing() {
    let s = Settings::parse(
        "search branch.lab.test\noptions ndots:15",
        "client.lab.test",
    )
    .unwrap();
    assert_eq!(s.plan("APP.ZONE.").unwrap(), ["app.zone"]);
    let a = s
        .lookup(
            "app.zone.",
            &dns(&[("app.zone.branch.lab.test", "192.0.2.80")]),
        )
        .unwrap();
    assert!(a.address.is_none());
    assert_eq!(a.attempts.len(), 1);
}
#[test]
fn the_first_positive_answer_wins_and_search_order_can_choose_a_wrong_site() {
    let rows = dns(&[
        ("report.north.lab.test", "192.0.2.80"),
        ("report.south.lab.test", "192.0.2.81"),
        ("report", "192.0.2.90"),
    ]);
    let a = Settings::parse("search south.lab.test north.lab.test", "client.lab.test")
        .unwrap()
        .lookup("report", &rows)
        .unwrap();
    assert_eq!(a.address, Some("192.0.2.81".parse().unwrap()));
    assert_eq!(a.attempts.len(), 1);
    let a = Settings::parse("search north.lab.test\noptions ndots:0", "client.lab.test")
        .unwrap()
        .lookup("report", &rows)
        .unwrap();
    assert_eq!(a.address, Some("192.0.2.90".parse().unwrap()));
    assert_eq!(a.attempts.len(), 1);
}
#[test]
fn last_search_or_domain_and_ndots_assignments_take_effect() {
    let a = Settings::parse(
        "search old.lab.test\ndomain north.lab.test\noptions ndots:2\noptions ndots:1\n",
        "client.lab.test",
    )
    .unwrap();
    assert_eq!(a.search(), ["north.lab.test"]);
    assert_eq!(a.ndots(), 1);
    let a = Settings::parse(
        "domain old.lab.test\nsearch NORTH.LAB.TEST. south.lab.test\n",
        "client.lab.test",
    )
    .unwrap();
    assert_eq!(a.search(), ["north.lab.test", "south.lab.test"]);
}
#[test]
fn defaults_come_only_from_supplied_hostname_never_the_desktop() {
    let a = Settings::parse("# no search override\n", "CLIENT.NORTH.LAB.TEST").unwrap();
    assert_eq!(a.search(), ["north.lab.test"]);
    assert_eq!(
        a.plan("report").unwrap(),
        ["report.north.lab.test", "report"]
    );
    let a = Settings::parse("; root default\n", "client").unwrap();
    assert_eq!(a.search(), ["."]);
    assert_eq!(a.plan("report").unwrap(), ["report"]);
}
#[test]
fn root_and_repeated_suffixes_preserve_actual_finite_attempt_order() {
    let s = Settings::parse("search north.lab.test .", "client.lab.test").unwrap();
    assert_eq!(
        s.plan("app.zone").unwrap(),
        ["app.zone", "app.zone.north.lab.test", "app.zone"]
    );
    assert_eq!(s.plan("app").unwrap(), ["app.north.lab.test", "app"]);
    let s = Settings::parse(
        "search north.lab.test north.lab.test north.lab.test",
        "client.lab.test",
    )
    .unwrap();
    let a = s.lookup("missing", &dns(&[])).unwrap();
    assert_eq!(a.attempts.len(), 4);
    assert!(a.address.is_none());
    assert_eq!(a.attempts[0].name, a.attempts[1].name);
}
#[test]
fn syntax_options_control_characters_and_resource_limits_never_fake_an_answer() {
    for text in [
        "nameserver 192.0.2.53",
        "options rotate",
        "options ndots:16",
        "domain north.lab.test south.lab.test",
        "search north.lab.test # note",
    ] {
        assert_eq!(
            Settings::parse(text, "client.lab.test").unwrap_err().kind,
            Kind::Unsupported,
            "{text}"
        );
    }
    for text in [
        "search",
        "options",
        "options ndots:-1",
        "options ndots:",
        "search ../private",
        "search north..test",
        "search north.lab.test\0",
    ] {
        assert_eq!(
            Settings::parse(text, "client.lab.test").unwrap_err().kind,
            Kind::Invalid,
            "{text}"
        );
    }
    for text in [
        "#".repeat(16_385),
        "\n".repeat(129),
        "search a.test b.test c.test d.test".into(),
    ] {
        assert_eq!(
            Settings::parse(&text, "client.lab.test").unwrap_err().kind,
            Kind::Limit
        );
    }
    let s = Settings::parse("search north.lab.test", "client.lab.test").unwrap();
    for query in [
        "$(id)",
        "../private",
        "192.0.2.1",
        "app..",
        ".",
        "",
        "app/name",
    ] {
        assert!(s.plan(query).is_err(), "{query}");
    }
}
#[test]
fn overlong_expansion_and_unmodeled_reply_sequences_have_honest_diagnostics() {
    let domain = [
        "x".repeat(63),
        "y".repeat(63),
        "z".repeat(63),
        "q".repeat(58),
    ]
    .join(".");
    let s = Settings::parse(&format!("search {domain}"), "client.lab.test").unwrap();
    assert_eq!(s.plan("report").unwrap_err().kind, Kind::Limit);
    let s = Settings::parse("search north.lab.test", "client.lab.test").unwrap();
    for status in [Status::Unavailable, Status::TryAgain] {
        let d = Dns {
            status,
            records: BTreeMap::new(),
        };
        assert_eq!(s.lookup("report", &d).unwrap_err().kind, Kind::Unsupported);
    }
    let mut d = dns(&[("report.north.lab.test", "192.0.2.80")]);
    d.status = Status::NotFound;
    assert_eq!(s.lookup("report", &d).unwrap_err().kind, Kind::Invalid);
}
