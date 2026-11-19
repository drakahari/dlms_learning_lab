use dlms_learning_lab::dns_access::{acl::Acl, config::Policy};
use std::collections::BTreeMap;
const CONFIG: &str = r#"options {
 allow-query { any; }; recursion yes;
 allow-recursion { 192.0.2.0/24; }; allow-query-cache { 192.0.2.0/24; };
 allow-transfer { none; };
};
zone "public.example" { type primary; file "db.public"; allow-transfer { 198.51.100.2; }; };
zone "private.example." { type primary; file "db.private"; };
"#;
fn known() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("public.example.".into(), "db.public".into()),
        ("private.example.".into(), "db.private".into()),
    ])
}
#[test]
fn actual_grammar_parses_reordered_comments_and_effective_zone_override() {
    let p = Policy::parse(CONFIG, &known()).unwrap();
    assert!(p.query.equivalent(&Acl::Any).unwrap());
    assert!(p.recursion);
    assert!(
        p.transfer_for("private.example")
            .unwrap()
            .equivalent(&Acl::None)
            .unwrap()
    );
    assert!(
        p.transfer_for("public.example")
            .unwrap()
            .permits("198.51.100.2".parse().unwrap())
            .unwrap()
    );
    assert!(
        !p.transfer_for("public.example")
            .unwrap()
            .permits("198.51.100.3".parse().unwrap())
            .unwrap()
    );
    let text = CONFIG
        .replace("options {", "# header\noptions {/* middle */")
        .replace("recursion yes;", "recursion yes; // end\n");
    assert_eq!(Policy::parse(&text, &known()).unwrap(), p);
    let text = CONFIG.replace("192.0.2.0/24;", "192.0.2.128/25; 192.0.2.0/25;");
    let q = Policy::parse(&text, &known()).unwrap();
    assert!(q.recursive.equivalent(&p.recursive).unwrap());
    assert!(q.cache.equivalent(&p.cache).unwrap());
}
#[test]
fn unsupported_and_ambiguous_configuration_never_silently_applies() {
    for (a, b) in [
        ("recursion yes;", "recursion yes; recursion no;"),
        ("recursion yes;", "recursion yes"),
        ("allow-query { any; };", "allow-query { any; 192.0.2.1; };"),
        ("allow-query { any; };", "allow-query { !192.0.2.1; any; };"),
        ("allow-query { any; };", "allow-query { trusted; };"),
        ("allow-query { any; };", "allow-query { ::1; };"),
        ("allow-query { any; };", "allow-query { 192.0.2.1/24; };"),
        ("allow-query { any; };", ""),
        (
            "recursion yes;",
            "forwarders { 203.0.113.1; }; recursion yes;",
        ),
        ("type primary;", "type secondary;"),
        ("db.public", "/etc/passwd"),
        ("public.example", "foreign.example"),
        (
            "file \"db.public\";",
            "file \"db.public\"; file \"db.public\";",
        ),
    ] {
        assert!(
            Policy::parse(&CONFIG.replace(a, b), &known()).is_err(),
            "{a} => {b}"
        );
    }
    for bad in [
        format!("{CONFIG}include \"/etc/passwd\";"),
        format!("{CONFIG}/* unclosed"),
        format!("{CONFIG}zone \"extra.example\" {{ type primary; file \"extra\"; }};"),
        "x".repeat(8193),
    ] {
        assert!(Policy::parse(&bad, &known()).is_err());
    }
    assert!(Policy::parse(CONFIG, &BTreeMap::new()).is_err());
    assert!(
        Policy::parse(
            &CONFIG.replace(
                "zone \"private.example.\" { type primary; file \"db.private\"; };",
                ""
            ),
            &known()
        )
        .is_err()
    );
}

#[test]
fn bounded_parser_never_panics_on_each_missing_or_inserted_delimiter() {
    for at in 0..CONFIG.len() {
        let mut text = CONFIG.to_owned();
        text.remove(at);
        if let Ok(policy) = Policy::parse(&text, &known()) {
            policy.validate(&known()).unwrap();
        }
    }
    for marker in [
        "\0", "\u{1}", "\u{7f}", "\"", "/*", "}", "{", ";", "!", "include",
    ] {
        for at in (0..CONFIG.len()).step_by(13) {
            let mut text = CONFIG.to_owned();
            text.insert_str(at, marker);
            if let Ok(policy) = Policy::parse(&text, &known()) {
                policy.validate(&known()).unwrap();
            }
        }
    }
    for bad in ["\u{1}", "\u{7f}", "é", "\0"] {
        assert!(Policy::parse(&format!("{CONFIG} // {bad}"), &known()).is_err());
    }
}
