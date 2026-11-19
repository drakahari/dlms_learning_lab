use a::{Level as L, Reason, Request as R, Subject as S};
use dlms_learning_lab::{ldap_access as a, ldap_query::Dn};
fn dn(s: &str) -> Dn {
    Dn::parse(s).unwrap()
}
const ROOT: &str = "dc=example,dc=test";
const ALICE: &str = "uid=alice,ou=People,dc=example,dc=test";
const BOB: &str = "uid=bob,ou=People,dc=example,dc=test";
fn decision(p: &a::Policy, s: &S, target: &str, attr: &str, r: R) -> a::Decision {
    p.evaluate(s, &dn(target), attr, r, None).unwrap()
}
#[test]
fn empty_list_defaults_to_read_while_a_nonempty_unmatched_list_denies() {
    let p = a::Policy::parse("# no applicable combined ACL\n\n").unwrap();
    for s in [S::Anonymous, S::Authenticated(dn(ALICE))] {
        let r = decision(&p, &s, BOB, "mail", R::Read);
        assert!(r.allowed);
        assert_eq!(r.reason, Reason::EmptyDefaultRead);
        assert!(!decision(&p, &s, BOB, "mail", R::Write).allowed);
    }
    let p = a::Policy::parse("access to attrs=description by * read").unwrap();
    let r = decision(&p, &S::Anonymous, BOB, "mail", R::Read);
    assert!(!r.allowed);
    assert_eq!(r.reason, Reason::NoTargetMatch);
    assert_eq!(r.granted, L::None);
}
#[test]
fn first_matching_target_decides_instead_of_combining_later_broader_rules() {
    let p=a::Policy::parse("access to dn.subtree=\"ou=People,dc=example,dc=test\" attrs=mail by self write by * none\naccess to * by * read").unwrap();
    let alice = S::Authenticated(dn(ALICE));
    assert!(decision(&p, &alice, ALICE, "mail", R::Write).allowed);
    let denied = decision(&p, &alice, BOB, "mail", R::Read);
    assert!(!denied.allowed);
    assert_eq!((denied.rule, denied.clause), (Some(1), Some(2)));
    assert_eq!(
        decision(&p, &alice, BOB, "description", R::Read).rule,
        Some(2)
    );
    assert!(decision(&p, &S::Anonymous, ROOT, "mail", R::Read).allowed);
}
#[test]
fn first_matching_subject_stops_even_when_a_later_subject_would_allow_more() {
    let p = a::Policy::parse("access to * by anonymous auth by * read").unwrap();
    let r = decision(&p, &S::Anonymous, ALICE, "mail", R::Read);
    assert!(!r.allowed);
    assert_eq!(r.granted, L::Auth);
    assert_eq!(r.clause, Some(1));
    assert!(decision(&p, &S::Anonymous, ALICE, "mail", R::Auth).allowed);
    assert!(decision(&p, &S::Authenticated(dn(BOB)), ALICE, "mail", R::Read).allowed);
}
#[test]
fn no_matching_subject_denies_without_falling_through_to_the_next_target() {
    let p = a::Policy::parse("access to attrs=mail by self write\naccess to * by * read").unwrap();
    let r = decision(&p, &S::Anonymous, ALICE, "mail", R::Read);
    assert!(!r.allowed);
    assert_eq!(
        (r.reason, r.rule, r.clause),
        (Reason::NoSubjectMatch, Some(1), None)
    );
    assert!(!decision(&p, &S::Authenticated(dn(BOB)), ALICE, "mail", R::Read).allowed);
}
#[test]
fn self_uses_authenticated_normalized_dn_equality_not_text_prefix_or_anonymous() {
    let p = a::Policy::parse("access to * by self write by users read by * none").unwrap();
    assert!(
        decision(
            &p,
            &S::Authenticated(dn("UID=ALICE,OU=people,DC=EXAMPLE,DC=TEST")),
            ALICE,
            "mail",
            R::Write
        )
        .allowed
    );
    assert!(!decision(&p, &S::Authenticated(dn(BOB)), ALICE, "mail", R::Write).allowed);
    assert!(decision(&p, &S::Authenticated(dn(BOB)), ALICE, "mail", R::Read).allowed);
    assert!(!decision(&p, &S::Anonymous, ALICE, "mail", R::Read).allowed);
}
#[test]
fn target_scopes_include_only_the_correct_components_and_depth() {
    let base = "ou=People,dc=example,dc=test";
    let nested = "cn=desk,uid=alice,ou=People,dc=example,dc=test";
    for (kind, expected) in [
        ("base", [true, false, false]),
        ("exact", [true, false, false]),
        ("one", [false, true, false]),
        ("subtree", [true, true, true]),
        ("children", [false, true, true]),
    ] {
        let p = a::Policy::parse(&format!("access to dn.{kind}=\"{base}\" by * read")).unwrap();
        for (target, expected) in [base, ALICE, nested].into_iter().zip(expected) {
            assert_eq!(
                decision(&p, &S::Anonymous, target, "mail", R::Read).allowed,
                expected,
                "{kind} {target}"
            );
        }
        for outside in [
            ROOT,
            "uid=alice,ou=OtherPeople,dc=example,dc=test",
            "uid=alice,ou=People,dc=otherexample,dc=test",
        ] {
            assert!(!decision(&p, &S::Anonymous, outside, "mail", R::Read).allowed);
        }
    }
}
#[test]
fn dn_subject_scopes_match_the_requester_not_the_target() {
    let p =
        a::Policy::parse("access to * by dn.one=\"ou=People,dc=example,dc=test\" write by * none")
            .unwrap();
    assert!(decision(&p, &S::Authenticated(dn(ALICE)), ROOT, "mail", R::Write).allowed);
    assert!(!decision(&p, &S::Authenticated(dn(ROOT)), ALICE, "mail", R::Write).allowed);
    assert!(!decision(&p, &S::Anonymous, ALICE, "mail", R::Write).allowed);
}
#[test]
fn every_level_implies_all_lower_levels_and_none_is_never_a_requested_privilege() {
    let levels = [
        ("none", L::None),
        ("disclose", L::Disclose),
        ("auth", L::Auth),
        ("compare", L::Compare),
        ("search", L::Search),
        ("read", L::Read),
        ("write", L::Write),
        ("manage", L::Manage),
    ];
    let requests = [
        R::Disclose,
        R::Auth,
        R::Compare,
        R::Search,
        R::Read,
        R::Write,
        R::Manage,
    ];
    for (rank, (name, level)) in levels.into_iter().enumerate() {
        let p = a::Policy::parse(&format!("access to * by * {name}")).unwrap();
        for (n, r) in requests.into_iter().enumerate() {
            let d = decision(&p, &S::Anonymous, ALICE, "mail", r);
            assert_eq!(d.allowed, rank > n);
            assert_eq!(d.granted, level);
        }
    }
}
#[test]
fn rootdn_bypass_requires_the_authenticated_configured_identity() {
    let p = a::Policy::parse("access to * by * none").unwrap();
    let root = dn("cn=manager,dc=example,dc=test");
    let d = p
        .evaluate(
            &S::Authenticated(root.clone()),
            &dn(ALICE),
            "userPassword",
            R::Manage,
            Some(&root),
        )
        .unwrap();
    assert!(d.allowed);
    assert_eq!(d.reason, Reason::RootDn);
    assert_eq!(d.rule, None);
    for subject in [S::Anonymous, S::Authenticated(dn(ALICE))] {
        assert!(
            !p.evaluate(&subject, &dn(ALICE), "mail", R::Read, Some(&root))
                .unwrap()
                .allowed
        );
    }
    assert!(
        !p.evaluate(&S::Authenticated(root), &dn(ALICE), "mail", R::Read, None)
            .unwrap()
            .allowed
    );
}
#[test]
fn attribute_selection_is_case_insensitive_and_pseudo_attributes_remain_separate() {
    let p = a::Policy::parse(
        "access to attrs=MAIL,description by users read\naccess to attrs=entry,children by * none",
    )
    .unwrap();
    let alice = S::Authenticated(dn(ALICE));
    assert!(decision(&p, &alice, ALICE, "Mail", R::Read).allowed);
    assert!(!decision(&p, &alice, ALICE, "entry", R::Read).allowed);
    assert!(!decision(&p, &alice, ALICE, "children", R::Write).allowed);
    // A positive field check alone is not authorization for a complete LDAP operation.
    assert_eq!(decision(&p, &alice, ALICE, "entry", R::Read).rule, Some(2));
    assert!(
        p.evaluate(&alice, &dn(ALICE), "mail;binary", R::Read, None)
            .is_err()
    );
    assert!(
        p.evaluate(&alice, &dn(ALICE), &"x".repeat(100_000), R::Read, None)
            .is_err()
    );
}
#[test]
fn comments_crlf_tabs_and_indented_by_lines_retain_source_header_numbers() {
    let p=a::Policy::parse("# sample\r\n\r\naccess to attrs=mail\r\n\tby self write stop\r\n    by * none\r\naccess to * by * read\r\n").unwrap();
    let d = decision(&p, &S::Authenticated(dn(ALICE)), ALICE, "mail", R::Write);
    assert!(d.allowed);
    assert_eq!(d.source_line, Some(3));
    assert_eq!(d.clause, Some(1));
    let d = decision(&p, &S::Anonymous, ALICE, "mail", R::Read);
    assert!(!d.allowed);
    assert_eq!(d.clause, Some(2));
    assert_eq!(
        decision(&p, &S::Anonymous, ALICE, "description", R::Read).source_line,
        Some(6)
    );
}
#[test]
fn a_quoted_dn_can_contain_spaces_but_no_configuration_escape_is_guessed() {
    let p = a::Policy::parse(
        "access to dn.exact=\"uid=Alex Smith,ou=People,dc=example,dc=test\" by self write",
    )
    .unwrap();
    let name = "uid=alex smith,ou=People,dc=example,dc=test";
    assert!(decision(&p, &S::Authenticated(dn(name)), name, "mail", R::Write).allowed);
    for s in [
        "access to dn.exact=\"uid=a\\,b,dc=example,dc=test\" by * read",
        "acc\"ess\" to * by * read",
        "access to dn.exact=\"uid=a,dc=test\"suffix by * read",
    ] {
        assert!(a::Policy::parse(s).is_err());
    }
}
#[test]
fn unsupported_real_features_fail_explicitly_instead_of_becoming_an_allow_rule() {
    for s in [
        "access to * by * read continue",
        "access to * by * read break",
        "access to * by * =r",
        "access to * by self selfwrite",
        "access to * by group=cn=staff,dc=test read",
        "access to * by ssf=128 users read",
        "access to filter=(uid=*) by * read",
        "access to dn.regex=.* by * read",
        "access to dn=uid=a,dc=test by * read",
        "olcAccess: to * by * read",
        "include /etc/ldap/slapd.conf",
        "access to attrs=mail,MAIL by * read",
        "access to attrs=mail dn.base=dc=test by * read",
        "access to * attrs=mail by * read",
        "access to * by * read # ignored?",
    ] {
        assert!(
            matches!(a::Policy::parse(s), Err(a::Error::Unsupported(..))),
            "{s}"
        );
    }
}
#[test]
fn incomplete_syntax_unknown_subject_and_bare_cr_do_not_become_empty_default_read() {
    for s in [
        "access to",
        "access to *",
        "access to * by",
        "access to * by users",
        " by * read",
        "access to * by * read\raccess to * by * none",
        "access to dn.exact=\"dc=test by * read",
        "access to * by somebody read",
    ] {
        assert!(a::Policy::parse(s).is_err(), "{s}");
    }
}
#[test]
fn exact_rule_clause_line_and_byte_limits_are_enforced_without_echoing_input() {
    let sixteen = "access to * by * none\n".repeat(16);
    assert!(a::Policy::parse(&sixteen).is_ok());
    assert!(matches!(
        a::Policy::parse(&(sixteen + "access to * by * read\n")),
        Err(a::Error::Limit(_))
    ));
    let sixteen = format!("access to *{}", " by users none".repeat(16));
    assert!(a::Policy::parse(&sixteen).is_ok());
    assert!(matches!(
        a::Policy::parse(&(sixteen + " by * read")),
        Err(a::Error::Limit(_))
    ));
    assert!(a::Policy::parse(&"\n".repeat(a::MAX_LINES)).is_ok());
    assert!(matches!(
        a::Policy::parse(&"\n".repeat(a::MAX_LINES + 1)),
        Err(a::Error::Limit(_))
    ));
    assert!(matches!(
        a::Policy::parse(&"x".repeat(a::MAX_BYTES + 1)),
        Err(a::Error::Limit(_))
    ));
    let error = a::Policy::parse(&format!(
        "access to * by {} read",
        "secret-shaped-text".repeat(40)
    ))
    .unwrap_err()
    .to_string();
    assert!(error.len() < 240);
    assert!(!error.contains("secret-shaped-text"));
    assert!(a::Policy::parse("access to * by * read\0").is_err());
}
