use dlms_learning_lab::ldap_schema::{Category, Directory, EntrySource, Field, Matching, Registry};
const STR: &str = "1.3.6.1.4.1.1466.115.121.1.15";
const INT: &str = "1.3.6.1.4.1.1466.115.121.1.27";
fn rows(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).into()).collect()
}
fn field(name: &str, values: &[&str]) -> Field {
    Field {
        name: name.into(),
        values: rows(values),
    }
}
fn custom() -> Registry {
    let mut r = Registry::teaching_base();
    r.register(&rows(&["(1.3.6.1.4.1.32473.90.1 NAME ('labCode' 'codeAlias') EQUALITY caseExactMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{2} SINGLE-VALUE)","(1.3.6.1.4.1.32473.90.2 NAME 'labCount' EQUALITY integerMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.27)"]),&rows(&["(1.3.6.1.4.1.32473.91.1 NAME ('labPerson' 'personAlias') SUP person STRUCTURAL MUST labCode MAY (uid $ labCount))"])).unwrap();
    r
}
fn person() -> EntrySource {
    EntrySource {
        dn: "cn=Taylor,dc=example,dc=test".into(),
        classes: rows(&["labPerson"]),
        fields: vec![
            field("cn", &["Taylor"]),
            field("sn", &["Learner"]),
            field("labCode", &["WORKSHOP-9"]),
        ],
    }
}
#[test]
fn real_registry_resolves_aliases_oids_and_inherited_constraints() {
    let r = custom();
    let a = r.attribute("CODEALIAS").unwrap();
    assert!(a.single_value);
    assert_eq!(a.equality, Some(Matching::CaseExact));
    assert_eq!(a.definition.oid, "1.3.6.1.4.1.32473.90.1");
    assert_eq!(r.class("personAlias"), r.class("1.3.6.1.4.1.32473.91.1"));
    let e = r.validate_entry(&person()).unwrap();
    assert_eq!(e.structural_class(), "1.3.6.1.4.1.32473.91.1");
    assert!(e.classes().contains("2.5.6.0"));
    assert!(e.classes().contains("2.5.6.6"));
    assert_eq!(e.fields()["1.3.6.1.4.1.32473.90.1"], ["WORKSHOP-9"]);
}
#[test]
fn legitimate_numeric_alias_and_ancestor_solutions_validate() {
    let r = custom();
    let mut p = person();
    p.classes = rows(&["top", "person", "PERSONALIAS"]);
    p.fields[0].name = "2.5.4.3".into();
    p.fields[1].name = "2.5.4.4".into();
    p.fields[2].name = "codeAlias".into();
    assert!(r.validate_entry(&p).is_ok());
}
#[test]
fn syntax_hint_does_not_reject_values_longer_than_hint() {
    let r = custom();
    assert!(
        r.attribute("labCode")
            .unwrap()
            .validate_value(&"x".repeat(200))
            .is_ok()
    );
    assert!(r.validate_entry(&person()).is_ok());
}
#[test]
fn registration_is_atomic_for_parse_semantic_and_duplicate_failures() {
    let good =
        format!("(1.3.6.1.4.1.32473.99.1 NAME 'first' EQUALITY caseIgnoreMatch SYNTAX {STR})");
    for bad in [
        "(1.2 NAME",
        "(2.5.4.3 SUP name)",
        "(1.2 NAME 'cn' SUP name)",
        "(1.2 NAME 'next' SUP missing)",
        "(1.2 NAME 'next' SUP next)",
        "(1.2 NAME 'next' EQUALITY integerMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
    ] {
        let mut r = custom();
        let before = r.clone();
        assert!(
            r.register(&[good.clone(), bad.into()], &[]).is_err(),
            "{bad}"
        );
        assert_eq!(r, before);
    }
    let mut r = custom();
    let before = r.clone();
    assert!(
        r.register(&[good], &rows(&["(1.2 NAME 'invalidClass' SUP missing)"]))
            .is_err()
    );
    assert_eq!(r, before);
}
#[test]
fn dependency_order_unknown_matching_and_refinements_are_honest() {
    let mut r = custom();
    for raw in [
        "(1.2 NAME 'child' SUP later)",
        "(1.2 NAME 'child' SUP labCode)",
        "(1.2 NAME 'child' SUP name EQUALITY caseExactMatch)",
        "(1.2 NAME 'child' SUP name SYNTAX 1.3.6.1.4.1.1466.115.121.1.27)",
        "(1.2 NAME 'child' EQUALITY mystery SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
    ] {
        assert_eq!(
            r.register(&rows(&[raw]), &[]).unwrap_err().category,
            Category::Unsupported,
            "{raw}"
        );
    }
    assert_eq!(
        r.register(
            &rows(&[&format!(
                "(1.2 NAME 'wrong' EQUALITY integerMatch SYNTAX {STR})"
            )]),
            &[]
        )
        .unwrap_err()
        .category,
        Category::Violation
    );
}
#[test]
fn classes_enforce_kind_chain_and_registered_fields() {
    let mut r = custom();
    r.register(
        &[],
        &rows(&["(1.3.6.1.4.1.32473.91.2 NAME 'otherPerson' SUP person STRUCTURAL)"]),
    )
    .unwrap();
    for (raw, category) in [
        (
            "(1.2 NAME 'abstractChild' SUP person ABSTRACT)",
            Category::Violation,
        ),
        (
            "(1.2 NAME 'brokenChain' SUP (labPerson $ otherPerson) STRUCTURAL)",
            Category::Violation,
        ),
        ("(1.2 NAME 'noTop' STRUCTURAL)", Category::Unsupported),
        (
            "(1.2 NAME 'badField' SUP top MUST missing)",
            Category::Unsupported,
        ),
        ("(1.2 NAME 'recursive' SUP recursive)", Category::Conflict),
    ] {
        assert_eq!(
            r.register(&[], &rows(&[raw])).unwrap_err().category,
            category,
            "{raw}"
        );
    }
    let mut p = person();
    p.classes.push("otherPerson".into());
    assert_eq!(
        r.validate_entry(&p).unwrap_err().category,
        Category::Violation
    );
    p.classes = rows(&["top"]);
    assert!(
        r.validate_entry(&p)
            .unwrap_err()
            .to_string()
            .contains("structural")
    );
}
#[test]
fn cardinality_required_allowed_and_naming_failures_identify_the_field() {
    let r = custom();
    let mut p = person();
    p.fields.pop();
    let e = r.validate_entry(&p).unwrap_err();
    assert_eq!(e.subject.as_deref(), Some("labCode"));
    assert!(e.message.contains("MUST"));
    let mut p = person();
    p.fields[2].values.push("SECOND".into());
    assert!(
        r.validate_entry(&p)
            .unwrap_err()
            .message
            .contains("SINGLE-VALUE")
    );
    let mut p = person();
    p.fields.push(field("name", &["stray"]));
    assert!(
        r.validate_entry(&p)
            .unwrap_err()
            .message
            .contains("MUST or MAY")
    );
    let mut p = person();
    p.fields[0].values = rows(&["Someone Else"]);
    let e = r.validate_entry(&p).unwrap_err();
    assert_eq!(e.subject.as_deref(), Some("cn"));
    assert!(e.message.contains("naming value"));
    let mut p = person();
    p.fields[0].values.clear();
    assert!(
        r.validate_entry(&p)
            .unwrap_err()
            .message
            .contains("at least one")
    );
    let mut p = person();
    p.fields.push(field("codeAlias", &["other"]));
    assert!(
        r.validate_entry(&p)
            .unwrap_err()
            .message
            .contains("repeated")
    );
    let mut p = person();
    p.fields.push(field("description;lang-en", &["note"]));
    assert_eq!(
        r.validate_entry(&p).unwrap_err().category,
        Category::Unsupported
    );
}
#[test]
fn equality_has_case_and_space_semantics_without_inventing_unicode() {
    let r = custom();
    let cn = r.attribute("cn").unwrap();
    assert!(
        cn.equivalent("  TAYLOR  Learner ", "taylor learner")
            .unwrap()
    );
    let code = r.attribute("labCode").unwrap();
    assert!(!code.equivalent("Code", "code").unwrap());
    assert!(code.equivalent("Code  A", " Code A ").unwrap());
    assert_eq!(
        cn.equivalent("é", "e").unwrap_err().category,
        Category::Unsupported
    );
    let mut p = person();
    p.fields[0].values.push(" TAYLOR ".into());
    assert!(r.validate_entry(&p).unwrap_err().message.contains("equal"));
}
#[test]
fn integer_is_unlimited_magnitude_within_explicit_resource_bound() {
    let r = custom();
    let a = r.attribute("labCount").unwrap();
    for value in ["0", "-1", "1234567890123456789012345678901234567890"] {
        assert!(a.validate_value(value).is_ok(), "{value}");
    }
    for value in ["", "+1", "-0", "00", "01", "-01", "1.0", " 1", "1 ", "--1"] {
        assert_eq!(
            a.validate_value(value).unwrap_err().category,
            Category::Violation,
            "{value}"
        );
    }
    assert_eq!(
        a.validate_value(&"9".repeat(257)).unwrap_err().category,
        Category::Limit
    );
    let mut p = person();
    p.fields.push(field("labCount", &["7", "7"]));
    assert!(r.validate_entry(&p).is_err());
}
#[test]
fn entry_mutations_preserve_old_state_on_every_rejected_request() {
    let mut d = Directory::new(custom());
    d.add(&person()).unwrap();
    let before = d.clone();
    let mut p = person();
    p.fields[2].values = rows(&["one", "two"]);
    assert!(d.replace(&p).is_err());
    assert_eq!(d, before);
    let mut p = person();
    p.classes = rows(&["person"]);
    p.fields.pop();
    assert!(
        d.replace(&p)
            .unwrap_err()
            .message
            .contains("cannot be changed")
    );
    assert_eq!(d, before);
    assert_eq!(d.add(&person()).unwrap_err().category, Category::Conflict);
    assert_eq!(d, before);
    let mut p = person();
    p.fields[2].values = rows(&["REVISED"]);
    d.replace(&p).unwrap();
    assert_ne!(d, before);
    assert_eq!(
        d.entries()[0].fields()["1.3.6.1.4.1.32473.90.1"],
        ["REVISED"]
    );
}
#[test]
fn escaped_rdn_matching_uses_parsed_value_not_naive_split() {
    let r = custom();
    let mut p = person();
    p.dn = "cn=Taylor\\, Learner,dc=example,dc=test".into();
    p.fields[0].values = rows(&["Taylor, Learner"]);
    assert!(r.validate_entry(&p).is_ok());
    p.dn = "cn=Taylor\\2c Learner,dc=example,dc=test".into();
    assert!(r.validate_entry(&p).is_ok());
}
#[test]
fn resource_and_unmodeled_boundaries_do_not_mutate_state() {
    let mut d = Directory::new(custom());
    d.add(&person()).unwrap();
    let before = d.clone();
    let mut p = person();
    p.fields[0].values = rows(&[&"x".repeat(257)]);
    assert_eq!(d.replace(&p).unwrap_err().category, Category::Limit);
    assert_eq!(d, before);
    let mut p = person();
    p.fields[0].values = rows(&["こんにちは"]);
    assert_eq!(d.replace(&p).unwrap_err().category, Category::Unsupported);
    assert_eq!(d, before);
    let mut r = custom();
    let before = r.clone();
    assert_eq!(
        r.register(&vec![format!("(1.2 SYNTAX {INT})"); 33], &[])
            .unwrap_err()
            .category,
        Category::Limit
    );
    assert_eq!(r, before);
}
#[test]
fn ia5_and_no_equality_are_deliberately_distinct_from_directory_string() {
    let mut r = Registry::teaching_base();
    r.register(&rows(&["(1.3.6.1.4.1.32473.90.8 NAME 'labAscii' EQUALITY caseIgnoreIA5Match SYNTAX 1.3.6.1.4.1.1466.115.121.1.26)","(1.3.6.1.4.1.32473.90.9 NAME 'noMatch' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)"]),&[]).unwrap();
    assert!(r.attribute("labAscii").unwrap().validate_value("").is_ok());
    assert!(r.attribute("cn").unwrap().validate_value("").is_err());
    assert!(
        r.attribute("labAscii")
            .unwrap()
            .equivalent(" A ", "a")
            .unwrap()
    );
    assert_eq!(
        r.attribute("noMatch")
            .unwrap()
            .equivalent("a", "a")
            .unwrap_err()
            .category,
        Category::Unsupported
    );
}
#[test]
fn directory_capacity_and_invalid_names_are_bounded_without_partial_changes() {
    let mut d = Directory::new(custom());
    for n in 0..32 {
        let mut p = person();
        let name = format!("Learner{n}");
        p.dn = format!("cn={name},dc=example,dc=test");
        p.fields[0].values = vec![name];
        d.add(&p).unwrap();
    }
    let before = d.clone();
    assert_eq!(d.add(&person()).unwrap_err().category, Category::Limit);
    assert_eq!(d, before);
    let r = custom();
    let mut p = person();
    p.classes = rows(&["bad\nclass"]);
    assert_eq!(r.validate_entry(&p).unwrap_err().category, Category::Syntax);
    let mut p = person();
    p.fields[2].name = "../private".into();
    assert_eq!(r.validate_entry(&p).unwrap_err().category, Category::Syntax);
}
#[test]
fn reserved_stock_syntax_matching_and_class_metadata_oids_cannot_be_reassigned() {
    for oid in ["2.5.4.0", "2.5.13.2", "1.3.6.1.4.1.1466.115.121.1.15"] {
        let mut r = custom();
        let before = r.clone();
        assert_eq!(
            r.register(&[format!("({oid} NAME 'reserved' SYNTAX {STR})")], &[])
                .unwrap_err()
                .category,
            Category::Conflict
        );
        assert_eq!(r, before);
    }
}
#[test]
fn class_membership_metadata_name_is_reserved_too() {
    let mut r = custom();
    let before = r.clone();
    assert_eq!(
        r.register(
            &[format!(
                "(1.3.6.1.4.1.32473.90.99 NAME 'objectClass' SYNTAX {STR})"
            )],
            &[]
        )
        .unwrap_err()
        .category,
        Category::Conflict
    );
    assert_eq!(r, before);
}
#[test]
fn entry_rule_feedback_does_not_invent_a_schema_editor_line_number() {
    let r = custom();
    let mut p = person();
    p.fields.pop();
    let e = r.validate_entry(&p).unwrap_err();
    let text = e.to_string();
    assert!(text.contains("labCode"));
    assert!(text.contains("Schema rule"));
    assert!(!text.contains("line 1"));
}
