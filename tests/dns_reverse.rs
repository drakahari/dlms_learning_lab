//! Original fixed reverse-DNS fixtures; never consult a host resolver.
use dlms_learning_lab::{
    dns::{self, Code, Data, QueryType, Record, Server, Transfer, Zone},
    dns_workspace::{self as editor, Draft, Requirement, Snapshot},
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::net::{IpAddr, Ipv6Addr};

const ORIGIN: &str = "2.0.192.in-addr.arpa.";
const OWNER: &str = "7.2.0.192.in-addr.arpa.";
fn ptr(owner: &str, target: &str, ttl: u32) -> Record {
    Record {
        owner: owner.into(),
        ttl,
        data: Data::Ptr {
            target: target.into(),
        },
    }
}
fn zone() -> Zone {
    Zone {
        origin: ORIGIN.into(),
        serial: 42,
        records: vec![
            Record {
                owner: ORIGIN.into(),
                ttl: 300,
                data: Data::Ns {
                    target: "ns.example.test.".into(),
                },
            },
            ptr(OWNER, "archive.example.test.", 300),
        ],
    }
}
fn requirement(copy: Snapshot) -> Requirement {
    Requirement::Resolves {
        copy,
        name: OWNER.into(),
        record_type: QueryType::Ptr,
        data: vec![Data::Ptr {
            target: "archive.example.test.".into(),
        }],
    }
}

#[test]
fn ipv4_question_names_reverse_whole_octets_including_extremes() {
    for (input, expected) in [
        ("192.0.2.7", OWNER),
        ("10.2.0.52", "52.0.2.10.in-addr.arpa."),
        ("0.0.0.0", "0.0.0.0.in-addr.arpa."),
        ("255.255.255.255", "255.255.255.255.in-addr.arpa."),
    ] {
        assert_eq!(dns::reverse_name(input.parse().unwrap()), expected);
    }
}

#[test]
fn ipv6_names_expand_every_digit_and_keep_mapped_addresses_in_ipv6_namespace() {
    // Fixed documentation-address example; question name written independently.
    let expected = "b.a.9.8.7.6.5.4.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.8.b.d.0.1.0.0.2.ip6.arpa.";
    for text in [
        "2001:db8::567:89ab",
        "2001:0db8:0000:0000:0000:0000:0567:89ab",
    ] {
        // The separate address below uses a different low32-bit value.
        let name = dns::reverse_name(text.parse().unwrap());
        assert_eq!(name.split('.').count(), 35);
        assert!(name.starts_with("b.a.9.8.7.6.5.0."));
    }
    assert_eq!(
        dns::reverse_name("2001:db8::4567:89ab".parse().unwrap()),
        expected
    );
    let mapped = dns::reverse_name("::ffff:192.0.2.7".parse().unwrap());
    assert!(mapped.starts_with("7.0.2.0.0.0.0.c.f.f.f.f."));
    assert!(mapped.ends_with(".ip6.arpa."));
    assert_eq!(mapped.len(), 73);
    for invalid in ["192.0.2.256", "1.2.3", "2001:::1", "fe80::1%eth0"] {
        assert!(invalid.parse::<IpAddr>().is_err());
    }
}

#[test]
fn every_ipv6_bit_has_the_correct_reverse_digit_and_never_loses_zero_groups() {
    for bit in 0..128 {
        let name = dns::reverse_name(Ipv6Addr::from(1_u128 << bit).into());
        let digits: Vec<_> = name
            .strip_suffix(".ip6.arpa.")
            .unwrap()
            .split('.')
            .collect();
        assert_eq!(digits.len(), 32);
        for (position, digit) in digits.iter().enumerate() {
            let expected = if position == bit / 4 {
                ["1", "2", "4", "8"][bit % 4]
            } else {
                "0"
            };
            assert_eq!(*digit, expected, "bit{bit}, digit{position}");
        }
        assert_eq!(dns::canonical_name(&name).unwrap(), name);
    }
}

#[test]
fn ptr_rrsets_accept_multiple_names_canonicalize_and_normalize_ttl() {
    let mut z = zone();
    z.records.push(ptr(OWNER, "ARCHIVE.EXAMPLE.TEST", 90));
    z.records.push(ptr(OWNER, "files.example.test.", 200));
    let answer = z
        .query(&OWNER.to_ascii_uppercase(), QueryType::Ptr)
        .unwrap();
    assert_eq!(answer.code, Code::Noerror);
    assert!(answer.authoritative);
    assert_eq!(answer.records.len(), 2);
    assert!(answer.records.iter().all(|r| r.ttl == 90));
    assert_eq!(
        editor::data_text(&answer.records[0].data),
        "archive.example.test."
    );
    let mut other = z.clone();
    other.records.reverse();
    assert!(z.equivalent(&other).unwrap());
    assert_eq!(
        answer.records,
        other.query(OWNER, QueryType::Ptr).unwrap().records
    );
}

#[test]
fn missing_name_missing_type_and_outside_authority_are_not_resolution_success() {
    let z = zone();
    let no_type = z.query(OWNER, QueryType::A).unwrap();
    assert_eq!(no_type.code, Code::Noerror);
    assert!(no_type.records.is_empty());
    let missing = z.query("8.2.0.192.in-addr.arpa.", QueryType::Ptr).unwrap();
    assert_eq!(missing.code, Code::Nxdomain);
    assert!(missing.authoritative);
    let outside = z.query("7.2.0.198.in-addr.arpa.", QueryType::Ptr).unwrap();
    assert_eq!(outside.code, Code::Refused);
    assert!(!outside.authoritative);
    // PTR data does not magically create an A record in a different zone.
    assert_eq!(
        z.query("archive.example.test.", QueryType::A).unwrap().code,
        Code::Refused
    );
}

#[test]
fn ptr_query_can_follow_an_owner_alias_but_returns_target_data_without_recursion() {
    let mut z = zone();
    let alias = "8.2.0.192.in-addr.arpa.";
    z.records.push(Record {
        owner: alias.into(),
        ttl: 60,
        data: Data::Cname {
            target: OWNER.into(),
        },
    });
    let answer = z.query(alias, QueryType::Ptr).unwrap();
    assert_eq!(answer.code, Code::Noerror);
    assert_eq!(answer.records.len(), 2);
    assert!(matches!(&answer.records[0].data, Data::Cname { .. }));
    assert_eq!(
        answer.records[1].data,
        Data::Ptr {
            target: "archive.example.test.".into()
        }
    );
    assert_eq!(z.query(alias, QueryType::Cname).unwrap().records.len(), 1);
}

#[test]
fn ptr_is_general_dns_data_but_known_alias_targets_and_owner_conflicts_fail_atomically() {
    let mut z = zone();
    z.origin = "example.test.".into();
    z.records[0].owner = z.origin.clone();
    z.records[1].owner = "catalog.example.test.".into();
    assert!(z.check_configuration().is_ok()); // PTR need not have an ARPA owner.
    let mut s = Server::new(z.clone()).unwrap();
    let loaded = s.primary.clone();
    s.configured.records.push(Record {
        owner: "archive.example.test.".into(),
        ttl: 300,
        data: Data::Cname {
            target: "final.example.test.".into(),
        },
    });
    let e = s.reload_primary().unwrap_err();
    assert_eq!(e.kind, dns::ErrorKind::Configuration);
    assert!(e.message.contains("PTR"));
    assert_eq!(s.primary, loaded);
    z.records.push(Record {
        owner: "catalog.example.test.".into(),
        ttl: 300,
        data: Data::Cname {
            target: "other.example.test.".into(),
        },
    });
    assert!(
        z.check_configuration()
            .unwrap_err()
            .message
            .contains("cannot also hold")
    );
}

#[test]
fn native_record_grammar_roundtrips_ptr_and_refuses_bad_fields_without_touching_state() {
    let s = Server::new(zone()).unwrap();
    let mut d = Draft::new(&s);
    assert!(d.text.contains(" IN PTR archive.example.test."));
    assert_eq!(d.parse(ORIGIN).unwrap(), s.configured);
    d.query_type = QueryType::Ptr;
    d.query_name = OWNER.into();
    d.lookup(&s).unwrap();
    assert!(d.output.contains("PTR"));
    assert!(d.output.contains("archive.example.test."));
    for value in [
        "",
        "relative",
        "bad/name.",
        "name.example.test. extra",
        "192.0.2.7",
    ] {
        let mut bad = Draft::new(&s);
        bad.text
            .push_str(&format!("\n8.2.0.192.in-addr.arpa. 60 IN PTR {value}"));
        let mut candidate = s.clone();
        let error = bad.save(&mut candidate).unwrap_err();
        assert!(error.starts_with("Line "));
        assert_eq!(candidate, s);
    }
    assert_eq!(serde_json::to_value(QueryType::Ptr).unwrap(), "PTR");
    assert_eq!(serde_json::to_value(QueryType::Aaaa).unwrap(), "AAAA");
}

#[test]
fn reverse_publication_keeps_saved_primary_and_secondary_distinct() {
    let mut s = Server::new(zone()).unwrap();
    let old = s.clone();
    s.configured.records[1].data = Data::Ptr {
        target: "new.example.test.".into(),
    };
    assert_eq!(s.primary, old.primary);
    s.reload_primary().unwrap();
    assert_ne!(s.primary, s.secondary);
    assert_eq!(s.update_secondary().unwrap(), Transfer::UnchangedEqual);
    s.configured.serial += 1;
    s.reload_primary().unwrap();
    assert_eq!(s.update_secondary().unwrap(), Transfer::Updated);
    assert_eq!(s.primary, s.secondary);
    let bytes = serde_json::to_vec(&s).unwrap();
    assert_eq!(Server::decode(&bytes).unwrap(), s);
}

#[test]
fn current_outcomes_accept_equivalent_records_and_fail_after_republication_breaks_them() {
    let mut s = Server::new(zone()).unwrap();
    let q = requirement(Snapshot::Primary);
    assert!(q.valid_for(&s) && q.check(&s).0);
    s.configured.records[1].data = Data::Ptr {
        target: "ARCHIVE.EXAMPLE.TEST".into(),
    };
    s.configured.records.push(s.configured.records[1].clone());
    s.reload_primary().unwrap();
    assert!(q.check(&s).0);
    s.configured.records[1].data = Data::Ptr {
        target: "wrong.example.test.".into(),
    };
    assert!(q.check(&s).0); // Explicit reload, not a draft edit, changes served data.
    assert!(!Requirement::Published { secondary: false }.check(&s).0);
    s.reload_primary().unwrap();
    assert!(!q.check(&s).0); // An extra wrong PTR is not hidden by one correct PTR.
}

#[test]
fn malformed_reverse_drafts_resume_exactly_without_current_credit_or_changed_origins() {
    let mut labs = pack::bundled().unwrap();
    let mut lab = labs
        .iter()
        .find(|s| matches!(s.initial, State::AuthoritativeDns { .. }))
        .unwrap()
        .clone();
    let s = Server::new(zone()).unwrap();
    lab.id = "test-reverse-dns-capability".into();
    lab.initial = State::AuthoritativeDns { server: s };
    lab.goal = Goal::AuthoritativeDns {
        requirements: vec![requirement(Snapshot::Primary)],
    };
    lab.dns_teaching = None;
    let mut a = Attempt::new(&lab, 1);
    workspace::prepare(&mut a);
    assert!(progress::current_solved(&a, &lab));
    let d = a.workspace.as_mut().unwrap().dns_draft.as_mut().unwrap();
    d.query_type = QueryType::Ptr;
    d.query_name = OWNER.into();
    d.text.push_str("\nmalformed unfinished record");
    assert!(!progress::current_solved(&a, &lab));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    labs.push(lab.clone());
    let bytes = serde_json::to_vec(&p).unwrap();
    let decoded = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded.attempts[&lab.id]).unwrap(),
        serde_json::to_value(&p.attempts[&lab.id]).unwrap()
    );
    assert!(!progress::current_solved(&decoded.attempts[&lab.id], &lab));
}

#[test]
fn ptr_uses_existing_record_name_and_output_bounds_without_expanding_supported_grammar() {
    let mut z = zone();
    while z.records.len() < dns::MAX_RECORDS {
        let owner = format!("{}.2.0.192.in-addr.arpa.", z.records.len());
        z.records.push(ptr(&owner, "archive.example.test.", 1));
    }
    assert!(z.check_configuration().is_ok());
    z.records
        .push(ptr("250.2.0.192.in-addr.arpa.", "archive.example.test.", 1));
    assert_eq!(z.validate_shape().unwrap_err().kind, dns::ErrorKind::Limit);
    let mut s = Server::new(zone()).unwrap();
    let old = s.primary.clone();
    s.configured.records[1].data = Data::Ptr {
        target: format!("{}.example.test.", "x".repeat(64)),
    };
    assert!(s.reload_primary().is_err());
    assert_eq!(s.primary, old);
    let mut d = Draft::new(&Server::new(zone()).unwrap());
    d.text.push_str("\n$INCLUDE /etc/passwd");
    assert!(d.parse(ORIGIN).unwrap_err().contains("Line "));
}
