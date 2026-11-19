use dlms_learning_lab::ldap_query::Truth::{self, *};
use dlms_learning_lab::ldap_schema::{Category, Directory, EntrySource, Field, Filter, Registry};
fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).into()).collect()
}
fn field(name: &str, values: &[&str]) -> Field {
    Field {
        name: name.into(),
        values: strs(values),
    }
}
fn directory() -> Directory {
    let mut registry = Registry::teaching_base();
    registry.register(&strs(&[
        "(1.3.6.1.4.1.32473.230.1 NAME ('badge' 'tag') EQUALITY caseExactMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
        "(1.3.6.1.4.1.32473.230.2 NAME 'folded' EQUALITY caseIgnoreMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
        "(1.3.6.1.4.1.32473.230.3 NAME 'projectCode' SUP folded)",
        "(1.3.6.1.4.1.32473.230.4 NAME 'points' EQUALITY integerMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.27)",
        "(1.3.6.1.4.1.32473.230.5 NAME 'opaque' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
        "(1.3.6.1.4.1.32473.230.6 NAME 'exactAscii' EQUALITY caseExactIA5Match SYNTAX 1.3.6.1.4.1.1466.115.121.1.26)",
        "(1.3.6.1.4.1.32473.230.7 NAME 'ignoreAscii' EQUALITY caseIgnoreIA5Match SYNTAX 1.3.6.1.4.1.1466.115.121.1.26)",
    ]), &strs(&["(1.3.6.1.4.1.32473.230.20 NAME ('colleague' 'staffMember') SUP person STRUCTURAL MAY (uid $ badge $ folded $ points $ opaque $ exactAscii $ ignoreAscii))"])).unwrap();
    let mut directory = Directory::new(registry);
    directory
        .add(&EntrySource {
            dn: "cn=Iris Lane,dc=directory,dc=test".into(),
            classes: strs(&["colleague"]),
            fields: vec![
                field("cn", &["Iris Lane"]),
                field("sn", &["Lane"]),
                field("uid", &["alpha"]),
                field("badge", &["A11", "a*()\\"]),
                field("projectCode", &[" Outer  Wing "]),
                field("points", &["-2", "999999999999999999999999999999999999"]),
                field("exactAscii", &["East Wing"]),
                field("ignoreAscii", &["East Wing"]),
            ],
        })
        .unwrap();
    directory
        .add(&EntrySource {
            dn: "cn=Sam Lake,dc=directory,dc=test".into(),
            classes: strs(&["colleague"]),
            fields: vec![
                field("cn", &["Sam Lake"]),
                field("sn", &["Lake"]),
                field("uid", &["beta"]),
                field("badge", &["a11"]),
            ],
        })
        .unwrap();
    directory
}
fn eval(d: &Directory, text: &str) -> Vec<Truth> {
    d.evaluate_filter(&Filter::parse(text).unwrap())
        .into_iter()
        .map(|(_, t)| t)
        .collect()
}
#[test]
fn equality_uses_actual_registered_rules_and_aliases() {
    let d = directory();
    for (filter, expected) in [
        ("(badge=A11)", vec![True, False]),
        ("(tag=a11)", vec![False, True]),
        ("(TAG= A11 )", vec![True, False]),
        ("(folded=outer wing)", vec![True, False]),
        ("(projectCode=OUTER WING)", vec![True, False]),
        ("(exactAscii=east wing)", vec![False, False]),
        ("(ignoreAscii= east  wing )", vec![True, False]),
        ("(name=iris lane)", vec![True, False]),
        ("(name=LANE)", vec![True, False]),
        ("(points=-2)", vec![True, False]),
        (
            "(points=999999999999999999999999999999999999)",
            vec![True, False],
        ),
    ] {
        assert_eq!(eval(&d, filter), expected, "{filter}");
    }
}
#[test]
fn missing_rules_unknown_fields_and_invalid_assertions_stay_undefined() {
    let d = directory();
    for filter in [
        "(notRegistered=match)",
        "(!(notRegistered=match))",
        "(opaque=match)",
        "(!(opaque=match))",
        "(points=02)",
        "(!(points=02))",
        "(points=+2)",
        "(points=-0)",
        "(points=word)",
        "(notRegistered=*)",
    ] {
        assert_eq!(eval(&d, filter), vec![Undefined, Undefined], "{filter}");
    }
    assert_eq!(eval(&d, "(opaque=*)"), vec![False, False]);
    assert_eq!(eval(&d, "(description=*)"), vec![False, False]);
    assert_eq!(eval(&d, "(name=*)"), vec![True, True]);
    assert_eq!(eval(&d, "(folded=*)"), vec![True, False]);
}
#[test]
fn object_class_uses_actual_inheritance_and_oid_identity() {
    let d = directory();
    for filter in [
        "(objectClass=top)",
        "(objectClass=person)",
        "(objectClass=STAFFMEMBER)",
        "(objectClass=2.5.6.6)",
        "(objectClass=1.3.6.1.4.1.32473.230.20)",
        "(objectClass=*)",
    ] {
        assert_eq!(eval(&d, filter), vec![True, True], "{filter}");
    }
    assert_eq!(
        eval(&d, "(objectClass=1.3.6.1.4.1.32473.999)"),
        vec![False, False]
    );
    for filter in [
        "(objectClass=imaginary)",
        "(objectClass=1..2)",
        "(!(objectClass=imaginary))",
    ] {
        assert_eq!(eval(&d, filter), vec![Undefined, Undefined]);
    }
}
#[test]
fn all_three_valued_boolean_truth_tables_preserve_undefined() {
    let d = directory();
    let atoms = [
        ("(uid=alpha)", True),
        ("(uid=missing)", False),
        ("(unknown=x)", Undefined),
    ];
    let and = [
        [True, False, Undefined],
        [False, False, False],
        [Undefined, False, Undefined],
    ];
    let or = [
        [True, True, True],
        [True, False, Undefined],
        [True, Undefined, Undefined],
    ];
    for (i, (a, _)) in atoms.iter().enumerate() {
        for (j, (b, _)) in atoms.iter().enumerate() {
            assert_eq!(eval(&d, &format!("(&{a}{b})"))[0], and[i][j]);
            assert_eq!(eval(&d, &format!("(|{a}{b})"))[0], or[i][j]);
        }
    }
    for (a, t) in atoms {
        assert_eq!(
            eval(&d, &format!("(!{a})"))[0],
            match t {
                True => False,
                False => True,
                Undefined => Undefined,
            }
        );
    }
}
#[test]
fn escaped_literals_are_values_and_never_filter_syntax() {
    let d = directory();
    assert_eq!(eval(&d, r"(badge=a\2a\28\29\5c)"), vec![True, False]);
    for filter in [
        r"(badge=\00)",
        r"(badge=\c3\a9)",
        "(badge=A*)",
        "(badge>=A)",
        "(badge~=A)",
        "(badge:caseExactMatch:=A)",
        "(badge;lang-en=A)",
        "(1.3.6.1.4.1.32473.230.1=A)",
        "(badge=)",
    ] {
        assert_eq!(
            Filter::parse(filter).unwrap_err().category,
            Category::Unsupported,
            "{filter}"
        );
    }
}
#[test]
fn malformed_and_large_filters_fail_without_mutating_actual_entries() {
    let d = directory();
    let before = d.clone();
    for filter in [
        "badge=A11",
        "(badge=A11",
        "(!(badge=A11)(uid=alpha))",
        "(badge=\\zz)",
    ] {
        assert_eq!(
            Filter::parse(filter).unwrap_err().category,
            Category::Syntax,
            "{filter}"
        );
    }
    for filter in [
        format!("(badge={})", "a".repeat(257)),
        format!("{}(badge=A11){}", "(!".repeat(16), ")".repeat(16)),
        format!("(|{})", "(badge=A11)".repeat(128)),
        "x".repeat(4097),
    ] {
        assert_eq!(
            Filter::parse(&filter).unwrap_err().category,
            Category::Limit
        );
    }
    let _ = eval(&d, "(badge=A11)");
    assert_eq!(d, before);
}

#[test]
fn equalityless_entry_values_remain_honestly_unsupported_and_atomic() {
    let mut d = directory();
    let before = d.clone();
    let mut source = d.source()[0].clone();
    source.fields.push(field("opaque", &["match"]));
    assert_eq!(
        d.replace(&source).unwrap_err().category,
        Category::Unsupported
    );
    assert_eq!(d, before);
    assert_eq!(eval(&d, "(opaque=match)"), vec![Undefined, Undefined]);
    assert_eq!(eval(&d, "(opaque=*)"), vec![False, False]);
}

#[test]
fn filter_diagnostics_name_the_cause_once_without_losing_category() {
    for (text, category, label) in [
        ("(badge=A*)", Category::Unsupported, "Unsupported"),
        ("badge=A", Category::Syntax, "Syntax error"),
    ] {
        let error = Filter::parse(text).unwrap_err();
        assert_eq!(error.category, category);
        assert_eq!(error.subject.as_deref(), Some("filter"));
        assert_eq!(error.to_string().matches(label).count(), 1);
    }
}
