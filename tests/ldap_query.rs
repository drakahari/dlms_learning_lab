use dlms_learning_lab::ldap_query as l;
use l::{Scope, SearchResult as S, Truth as T};
#[path = "support/ldap_directory.rs"]
mod fixture;
use fixture::{attr, fixture};
fn names(d: &l::Directory, base: &str, scope: Scope, filter: &str) -> Vec<String> {
    match d.search(
        &l::Dn::parse(base).unwrap(),
        scope,
        &l::Filter::parse(filter).unwrap(),
    ) {
        S::Entries(entries) => entries.iter().map(|e| e.dn().text().into()).collect(),
        S::NoSuchObject => panic!("missing base"),
    }
}
#[test]
fn base_one_and_subtree_use_actual_parsed_parent_relationships() {
    let d = l::Directory::new(&fixture()).unwrap();
    let b = "ou=People,dc=example,dc=test";
    assert_eq!(names(&d, b, Scope::Base, "(objectClass=*)"), [b]);
    let one = names(&d, b, Scope::One, "(objectClass=*)");
    assert_eq!(one.len(), 3);
    assert!(!one.iter().any(|s| s.starts_with("uid=lee")));
    assert_eq!(names(&d, b, Scope::Sub, "(objectClass=*)").len(), 5);
    assert_eq!(names(&d, b, Scope::Sub, "(objectClass=person)").len(), 3);
    assert_eq!(names(&d, b, Scope::One, "(objectClass=person)").len(), 2);
    assert_eq!(names(&d, b, Scope::Base, "(uid=alex)").len(), 0);
}
#[test]
fn missing_base_is_not_a_successful_empty_result() {
    let d = l::Directory::new(&fixture()).unwrap();
    let f = l::Filter::parse("(uid=missing)").unwrap();
    assert_eq!(
        d.search(
            &l::Dn::parse("ou=Missing,dc=example,dc=test").unwrap(),
            Scope::Sub,
            &f
        ),
        S::NoSuchObject
    );
    assert_eq!(d.search(d.suffix(), Scope::Sub, &f), S::Entries(vec![]));
}
#[test]
fn dn_escapes_and_declared_ascii_matching_rules_are_component_based() {
    for (a, b) in [
        (
            r"cn=Rivera\, Alex,ou=People,dc=EXAMPLE,dc=test",
            r"CN=rivera\2c  alex,OU=people,DC=example,DC=TEST",
        ),
        (
            r"cn=\#Team,dc=example,dc=test",
            r"cn=\23team,dc=example,dc=test",
        ),
        (
            r"cn=\ Alex\ ,dc=example,dc=test",
            r"cn=alex,dc=example,dc=test",
        ),
    ] {
        assert!(
            l::Dn::parse(a)
                .unwrap()
                .equivalent(&l::Dn::parse(b).unwrap()),
            "{a}"
        );
    }
    let a = l::Dn::parse("uid=alex,ou=People,dc=example,dc=test").unwrap();
    let b = l::Dn::parse("ou=People,dc=example,dc=test").unwrap();
    assert!(a.within(&b, Scope::One));
    assert!(
        !l::Dn::parse("uid=alex,ou=OtherPeople,dc=example,dc=test")
            .unwrap()
            .within(&b, Scope::Sub)
    );
    assert!(!a.equivalent(&l::Dn::parse("cn=alex,ou=People,dc=example,dc=test").unwrap()));
}
#[test]
fn unsupported_and_malformed_names_do_not_become_paths_or_silently_normalize() {
    for bad in [
        "",
        "../People",
        "cn=",
        "cn=Alex,",
        "cn= Alex,dc=test",
        "cn=Alex ,dc=test",
        "cn=#4444,dc=test",
        "cn=Alex+uid=alex,dc=test",
        "o=Example,dc=test",
        "cn=René,dc=test",
        "cn=Alex\n,dc=test",
        r"cn=Alex\00,dc=test",
        r"cn=Alex\c3\a9,dc=test",
        r"cn=Alex\q,dc=test",
        r"cn=Alex\",
        "cn=Alex;ou=People,dc=test",
        "cn=\"Alex\",dc=test",
    ] {
        assert!(l::Dn::parse(bad).is_err(), "{bad:?}");
    }
    assert!(matches!(
        l::Dn::parse(&"x".repeat(1025)),
        Err(l::Error::Limit(_))
    ));
    assert!(matches!(
        l::Dn::parse(&vec!["dc=a"; 17].join(",")),
        Err(l::Error::Limit(_))
    ));
}
#[test]
fn equality_presence_multivalues_and_ascii_spaces_follow_declared_rules() {
    let d = l::Directory::new(&fixture()).unwrap();
    let alex = &d.entries()[2];
    for f in [
        "(UID=ALEX)",
        "(cn=  ALEX   Rivera  )",
        "(mail=A.RIVERA@EXAMPLE.TEST)",
        "(objectClass=inetOrgPerson)",
        "(mail=*)",
    ] {
        assert_eq!(l::Filter::parse(f).unwrap().evaluate(alex), T::True, "{f}");
    }
    for f in ["(uid=sam)", "(description=*)", "(description=absent)"] {
        assert_eq!(l::Filter::parse(f).unwrap().evaluate(alex), T::False, "{f}");
    }
}
#[test]
fn unknown_attributes_and_class_identifiers_are_undefined_even_after_not() {
    let d = l::Directory::new(&fixture()).unwrap();
    let alex = &d.entries()[2];
    for f in [
        "(shoeSize=*)",
        "(shoeSize=12)",
        "(!(shoeSize=12))",
        "(!(shoeSize=*))",
        "(objectClass=notARegisteredClass)",
        "(!(objectClass=notARegisteredClass))",
    ] {
        assert_eq!(
            l::Filter::parse(f).unwrap().evaluate(alex),
            T::Undefined,
            "{f}"
        );
        assert_eq!(names(&d, "dc=example,dc=test", Scope::Sub, f).len(), 0);
    }
}
#[test]
fn three_valued_boolean_composition_preserves_truth_tables() {
    let d = l::Directory::new(&fixture()).unwrap();
    let e = &d.entries()[2];
    let cases = [
        ("(&(uid=alex)(shoeSize=1))", T::Undefined),
        ("(&(uid=sam)(shoeSize=1))", T::False),
        ("(|(uid=alex)(shoeSize=1))", T::True),
        ("(|(uid=sam)(shoeSize=1))", T::Undefined),
        ("(!(uid=sam))", T::True),
        ("(!(uid=alex))", T::False),
        ("(&(|(uid=alex)(uid=sam))(!(uid=lee)))", T::True),
    ];
    for (f, t) in cases {
        assert_eq!(l::Filter::parse(f).unwrap().evaluate(e), t, "{f}");
    }
}
#[test]
fn escaped_asterisk_is_literal_and_never_becomes_a_substring() {
    let d = l::Directory::new(&fixture()).unwrap();
    assert_eq!(
        names(
            &d,
            "ou=People,dc=example,dc=test",
            Scope::Sub,
            r"(cn=Sam \2a Visitor)"
        )
        .len(),
        1
    );
    assert!(matches!(
        l::Filter::parse("(cn=Sam*Visitor)"),
        Err(l::Error::Unsupported(_))
    ));
    assert!(matches!(
        l::Filter::parse("(cn=**)"),
        Err(l::Error::Unsupported(_))
    ));
    assert_eq!(
        names(&d, "dc=example,dc=test", Scope::Sub, r"(cn=\2a)").len(),
        0
    );
}
#[test]
fn malformed_and_unsupported_filters_fail_without_approximating() {
    for f in [
        "uid=alex",
        "(uid=alex",
        "(uid=alex))",
        "(uid=alex)(uid=sam)",
        "(&)",
        "(|)",
        "(!)",
        "(!(uid=alex)(uid=sam))",
        r"(cn=Alex\)",
        r"(cn=Alex\q)",
        "(cn=Alex(Rivera))",
    ] {
        assert!(l::Filter::parse(f).is_err(), "{f:?}");
    }
    for f in [
        "(uid>=a)",
        "(uid<=z)",
        "(cn~=alex)",
        "(uid:caseExactMatch:=alex)",
        "(uid;lang-en=alex)",
        "(2.5.4.3=alex)",
        "(cn=René)",
        r"(cn=\00)",
        "(cn=)",
    ] {
        assert!(
            matches!(l::Filter::parse(f), Err(l::Error::Unsupported(_))),
            "{f}"
        );
    }
}
#[test]
fn filters_have_independent_size_depth_node_and_assertion_limits() {
    assert!(matches!(
        l::Filter::parse(&"x".repeat(4097)),
        Err(l::Error::Limit(_))
    ));
    assert!(matches!(
        l::Filter::parse(&format!("{}(uid=a){}", "(!".repeat(16), ")".repeat(16))),
        Err(l::Error::Limit(_))
    ));
    assert!(matches!(
        l::Filter::parse(&format!("(|{})", "(uid=a)".repeat(128))),
        Err(l::Error::Limit(_))
    ));
    assert!(matches!(
        l::Filter::parse(&format!("(cn={})", "a".repeat(257))),
        Err(l::Error::Limit(_))
    ));
    assert!(l::Filter::parse(&format!("(|{})", "(uid=a)".repeat(127))).is_ok());
}
#[test]
fn source_roundtrip_is_bounded_and_unknown_duplicate_fields_are_rejected() {
    let s = fixture();
    let d = l::Directory::new(&s).unwrap();
    assert_eq!(
        l::Directory::from_json(&serde_json::to_string(&s).unwrap()).unwrap(),
        d
    );
    let mut v = serde_json::to_value(&s).unwrap();
    v["extra"] = true.into();
    assert!(l::Directory::from_json(&v.to_string()).is_err());
    let raw = serde_json::to_string(&s).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(l::Directory::from_json(&raw).is_err());
    assert!(matches!(
        l::Directory::from_json(&" ".repeat(l::MAX_JSON + 1)),
        Err(l::Error::Limit(_))
    ));
}
#[test]
fn source_rejects_duplicate_equivalent_names_and_mismatched_rdns() {
    let s = fixture();
    let mut b = s.clone();
    b.entries.push(s.entries[2].clone());
    b.entries.last_mut().unwrap().dn = "UID=ALEX,OU=people,DC=EXAMPLE,DC=TEST".into();
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries[2].attributes[0].values[0] = "other".into();
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries[2].attributes.push(attr("UID", &["other"]));
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries[2].attributes[0].values.push("ALEX".into());
    assert!(l::Directory::new(&b).is_err());
}
#[test]
fn source_rejects_wrong_context_missing_parents_missing_schema_and_invalid_values() {
    let s = fixture();
    let mut b = s.clone();
    b.entries[2].dn = "uid=alex,dc=outside,dc=test".into();
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries.remove(1);
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries.remove(0);
    assert!(l::Directory::new(&b).is_err());
    for (name, val) in [
        ("password", "secret"),
        ("cn", "René"),
        ("objectClass", "unknown"),
        ("cn", "bad\nvalue"),
    ] {
        let mut b = s.clone();
        b.entries[2].attributes.push(attr(name, &[val]));
        assert!(l::Directory::new(&b).is_err());
    }
    let mut b = s.clone();
    b.entries[0].attributes.retain(|a| a.name != "objectClass");
    assert!(l::Directory::new(&b).is_err());
}
#[test]
fn typed_and_json_sources_have_shape_and_total_resource_bounds() {
    let s = fixture();
    let mut b = s.clone();
    b.version = 2;
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries = vec![s.entries[0].clone(); 129];
    assert!(l::Directory::new(&b).is_err());
    let mut b = s.clone();
    b.entries[2].attributes = vec![attr("cn", &["a"]); 17];
    assert!(l::Directory::new(&b).is_err());
    assert!(l::Directory::from_json(&serde_json::to_string(&b).unwrap()).is_err());
    let mut b = s.clone();
    b.entries[2].attributes[0].values = vec!["a".into(); 17];
    assert!(l::Directory::new(&b).is_err());
    assert!(l::Directory::from_json(&serde_json::to_string(&b).unwrap()).is_err());
    let mut b = s.clone();
    b.entries[2].attributes[0].values = vec!["a".repeat(257)];
    assert!(l::Directory::new(&b).is_err());
    assert!(l::Directory::from_json(&serde_json::to_string(&b).unwrap()).is_err());
}
#[test]
fn reads_do_not_mutate_source_and_no_numeric_variant_is_a_new_lab() {
    let s = fixture();
    let before = s.clone();
    let d = l::Directory::new(&s).unwrap();
    for f in ["(uid=alex)", "(uid=sam)", "(uid=lee)", "(uid=missing)"] {
        let _ = names(&d, &s.suffix, Scope::Sub, f);
    }
    assert_eq!(s, before);
    assert_eq!(l::Scope::parse("base").unwrap(), Scope::Base);
    assert!(l::Scope::parse("children").is_err());
}

#[test]
fn aggregate_typed_size_fails_before_cloning_and_truncated_inputs_never_panic() {
    let mut s = fixture();
    s.entries = vec![s.entries[0].clone(); 128];
    for e in &mut s.entries {
        e.attributes = vec![attr("cn", &["x"]); 16];
        for a in &mut e.attributes {
            a.values = vec!["x".repeat(256); 16];
        }
    }
    assert!(matches!(l::Directory::new(&s), Err(l::Error::Limit(_))));
    for raw in [
        "(&(uid=alex)(!(mail=*)))",
        r"cn=Rivera\2c Alex,ou=People,dc=example,dc=test",
    ] {
        for i in 0..=raw.len() {
            let p = &raw[..i];
            let _ = l::Dn::parse(p);
            let _ = l::Filter::parse(p);
        }
    }
}
