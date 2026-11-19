//! Typed authoritative fixtures only; no real DNS, host, dig or subprocess.
use dlms_learning_lab::{
    dns::{self, Code, Data, QueryType, Record, Server, Zone},
    dns_query_tools as q,
    host_direct_dns::Tool,
};
fn row(owner: &str, data: Data) -> Record {
    Record {
        owner: owner.into(),
        ttl: 300,
        data,
    }
}
fn fixture() -> Server {
    Server::new(Zone {
        origin: "example.test.".into(),
        serial: 1,
        records: vec![
            row(
                "example.test.",
                Data::Ns {
                    target: "ns1.example.test.".into(),
                },
            ),
            row(
                "ns1.example.test.",
                Data::A {
                    address: "192.0.2.53".parse().unwrap(),
                },
            ),
            row(
                "web.example.test.",
                Data::A {
                    address: "192.0.2.80".parse().unwrap(),
                },
            ),
            row(
                "web.example.test.",
                Data::Aaaa {
                    address: "2001:db8::80".parse().unwrap(),
                },
            ),
            row(
                "alias.example.test.",
                Data::Cname {
                    target: "web.example.test.".into(),
                },
            ),
            row(
                "example.test.",
                Data::Mx {
                    preference: 10,
                    exchange: "mail.example.test.".into(),
                },
            ),
            row(
                "mail.example.test.",
                Data::A {
                    address: "192.0.2.25".parse().unwrap(),
                },
            ),
        ],
    })
    .unwrap()
}
fn plan() -> q::Plan {
    q::Plan {
        endpoints: vec![
            q::Endpoint {
                address: "192.0.2.53".parse().unwrap(),
                served: q::Served::Primary,
            },
            q::Endpoint {
                address: "192.0.2.54".parse().unwrap(),
                served: q::Served::Secondary,
            },
        ],
    }
}
#[test]
fn all_supported_record_types_and_real_alternates_use_actual_typed_data() {
    let server = fixture();
    let before = server.clone();
    let p = plan();
    for (input, kind, fragment) in [
        (
            "dig @192.0.2.53 web.example.test. A +norecurse +short",
            QueryType::A,
            "192.0.2.80\n",
        ),
        (
            "dig +short @192.0.2.53 WEB.EXAMPLE.TEST. -t aaaa +norecurse",
            QueryType::Aaaa,
            "2001:db8::80\n",
        ),
        (
            "host -r -t CNAME alias.example.test. 192.0.2.53",
            QueryType::Cname,
            "alias.example.test is an alias for web.example.test.\n",
        ),
        (
            "host -4 -r -t MX example.test. 192.0.2.53",
            QueryType::Mx,
            "example.test mail is handled by 10 mail.example.test.\n",
        ),
        (
            "dig @192.0.2.53 example.test. -t NS +short +norecurse",
            QueryType::Ns,
            "ns1.example.test.\n",
        ),
    ] {
        let observed = p.observe(&server, input).unwrap();
        assert_eq!(observed.question.kind, kind);
        assert_eq!(observed.answer.code, Code::Noerror);
        assert_eq!(observed.status(), 0);
        assert!(observed.output().unwrap().contains(fragment), "{input}");
        assert_eq!(server, before);
    }
    let mut reverse = Server::new(Zone {
        origin: "2.0.192.in-addr.arpa.".into(),
        serial: 1,
        records: vec![
            row(
                "2.0.192.in-addr.arpa.",
                Data::Ns {
                    target: "ns1.example.test.".into(),
                },
            ),
            row(
                "10.2.0.192.in-addr.arpa.",
                Data::Ptr {
                    target: "workstation.example.test.".into(),
                },
            ),
        ],
    })
    .unwrap();
    let out = p
        .observe(
            &reverse,
            "host -r -4 -t PTR 10.2.0.192.in-addr.arpa. 192.0.2.53",
        )
        .unwrap()
        .output()
        .unwrap();
    assert!(out.contains("10.2.0.192.in-addr.arpa domain name pointer workstation.example.test."));
    reverse.primary.records[1].data = Data::Ptr {
        target: "changed.example.test.".into(),
    };
    let out = p
        .observe(
            &reverse,
            "dig @192.0.2.53 10.2.0.192.in-addr.arpa. PTR +norecurse +short",
        )
        .unwrap()
        .output()
        .unwrap();
    assert!(out.starts_with("changed.example.test.\n"));
}
#[test]
fn dns_negative_responses_are_distinct_from_utility_status_and_from_missing_types() {
    let s = fixture();
    let p = plan();
    for (name, kind, code, host_status, message) in [
        (
            "missing.example.test.",
            "A",
            Code::Nxdomain,
            1,
            "Host missing.example.test. not found: 3(NXDOMAIN)",
        ),
        (
            "web.example.test.",
            "MX",
            Code::Noerror,
            0,
            "web.example.test has no MX record",
        ),
        (
            "outside.test.",
            "A",
            Code::Refused,
            1,
            "Host outside.test not found: 5(REFUSED)",
        ),
    ] {
        let host = p
            .observe(&s, &format!("host -r -t {kind} {name} 192.0.2.53"))
            .unwrap();
        assert_eq!(host.answer.code, code);
        assert_eq!(host.status(), host_status);
        assert!(host.output().unwrap().contains(message));
        let dig = p
            .observe(
                &s,
                &format!("dig @192.0.2.53 {name} {kind} +norecurse +short"),
            )
            .unwrap();
        assert_eq!(dig.answer.code, code);
        assert_eq!(dig.status(), 0);
        assert!(dig.answer.records.is_empty());
        assert!(dig.output().unwrap().contains(q::code_text(code)));
    }
}
#[test]
fn alias_chains_and_failures_derive_from_zone_state_without_external_recursion() {
    let p = plan();
    let mut s = fixture();
    let a = p
        .observe(
            &s,
            "dig @192.0.2.53 alias.example.test. A +norecurse +short",
        )
        .unwrap();
    assert_eq!(a.answer.code, Code::Noerror);
    assert!(
        a.output()
            .unwrap()
            .starts_with("web.example.test.\n192.0.2.80\n")
    );
    s.primary
        .records
        .iter_mut()
        .find(|r| r.owner == "alias.example.test.")
        .unwrap()
        .data = Data::Cname {
        target: "missing.example.test.".into(),
    };
    let a = p
        .observe(
            &s,
            "dig @192.0.2.53 alias.example.test. A +norecurse +short",
        )
        .unwrap();
    assert_eq!(a.answer.code, Code::Nxdomain);
    assert!(a.output().unwrap().starts_with("missing.example.test.\n"));
    assert_eq!(a.status(), 0);
    s.primary
        .records
        .iter_mut()
        .find(|r| r.owner == "alias.example.test.")
        .unwrap()
        .data = Data::Cname {
        target: "outside.test.".into(),
    };
    let a = p
        .observe(
            &s,
            "dig @192.0.2.53 alias.example.test. A +norecurse +short",
        )
        .unwrap();
    assert_eq!(a.answer.code, Code::Noerror);
    assert!(a.output().unwrap().starts_with("outside.test.\n"));
    assert_eq!(a.answer.records.len(), 1);
    assert!(a.answer.note.contains("outside"));
    s.primary
        .records
        .iter_mut()
        .find(|r| r.owner == "alias.example.test.")
        .unwrap()
        .data = Data::Cname {
        target: "alias.example.test.".into(),
    };
    for input in [
        "dig @192.0.2.53 alias.example.test. A +norecurse +short",
        "host -r -t A alias.example.test. 192.0.2.53",
    ] {
        let a = p.observe(&s, input).unwrap();
        assert_eq!(a.answer.code, Code::Servfail);
        assert_eq!(a.status(), if a.question.tool == Tool::Dig { 0 } else { 1 });
        assert!(a.output().unwrap().contains("SERVFAIL"));
    }
}
#[test]
fn explicit_endpoints_read_published_copies_never_saved_or_unsaved_configuration() {
    let p = plan();
    let mut s = fixture();
    let read = |s: &Server, ip: &str| {
        p.observe(
            s,
            &format!("dig @{ip} web.example.test. A +norecurse +short"),
        )
        .unwrap()
        .output()
        .unwrap()
    };
    s.configured.serial = 2;
    s.configured
        .records
        .iter_mut()
        .find(|r| r.owner == "web.example.test." && matches!(r.data, Data::A { .. }))
        .unwrap()
        .data = Data::A {
        address: "192.0.2.81".parse().unwrap(),
    };
    assert!(read(&s, "192.0.2.53").starts_with("192.0.2.80\n"));
    s.reload_primary().unwrap();
    assert!(read(&s, "192.0.2.53").starts_with("192.0.2.81\n"));
    assert!(read(&s, "192.0.2.54").starts_with("192.0.2.80\n"));
    s.update_secondary().unwrap();
    assert!(read(&s, "192.0.2.54").starts_with("192.0.2.81\n"));
    s.configured.records.push(row(
        "web.example.test.",
        Data::Cname {
            target: "alias.example.test.".into(),
        },
    ));
    assert!(s.reload_primary().is_err());
    assert!(read(&s, "192.0.2.53").starts_with("192.0.2.81\n"));
}
#[test]
fn unsupported_syntax_and_servers_refuse_without_changing_any_copy() {
    let s = fixture();
    let before = s.clone();
    let p = plan();
    for input in [
        "dig @192.0.2.53 web.example.test. A +short",
        "host -t A web.example.test. 192.0.2.53",
        "dig @192.0.2.53 web.example.test. SOA +norecurse +short",
        "dig @192.0.2.53 web.example.test A +norecurse +short",
        "dig @1.1.1.1 web.example.test. A +norecurse +short",
        "host -r -t A web.example.test. localhost",
        "dig @192.0.2.53 /etc/passwd A +norecurse +short",
        "dig @192.0.2.53 $(id).example.test. A +norecurse +short",
        "dig @192.0.2.53 web.example.test. A +norecurse +short;id",
        "dig @192.0.2.53 web.example.test. A +norecurse +short\nid",
        "host -r -t A web.example.test. 192.0.2.53 > /tmp/x",
        "dig @192.0.2.53 . NS +norecurse +short",
    ] {
        assert!(p.observe(&s, input).is_err(), "{input}");
        assert_eq!(s, before);
    }
    assert!(q::Question::parse(&"x".repeat(513)).is_err());
    let mut bad = p.clone();
    bad.endpoints[1].served = q::Served::Primary;
    assert!(bad.validate(&s).is_err());
    let mut bad = p.clone();
    bad.endpoints[1].address = bad.endpoints[0].address;
    assert!(bad.validate(&s).is_err());
    let mut bad = p;
    bad.endpoints.clear();
    assert!(bad.validate(&s).is_err());
}
#[test]
fn large_valid_rrsets_fail_the_output_limit_honestly_without_partial_success() {
    let owner = format!(
        "{}.{}.{}.{}.example.test.",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(48)
    );
    let exchange = format!(
        "{}.{}.{}.{}.outside.test.",
        "m".repeat(63),
        "n".repeat(63),
        "o".repeat(63),
        "p".repeat(48)
    );
    let mut z = fixture().primary;
    z.records.truncate(2);
    for preference in 1..=(dns::MAX_RECORDS - 2) as u16 {
        z.records.push(row(
            &owner,
            Data::Mx {
                preference,
                exchange: exchange.clone(),
            },
        ));
    }
    let s = Server::new(z).unwrap();
    let a = plan()
        .observe(&s, &format!("host -r -t MX {owner} 192.0.2.53"))
        .unwrap();
    assert!(a.output().unwrap_err().contains("output limit"));
    let dig = plan()
        .observe(&s, &format!("dig @192.0.2.53 {owner} MX +norecurse +short"))
        .unwrap();
    assert!(dig.output().unwrap().len() <= q::MAX_OUTPUT);
}
