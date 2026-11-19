use dlms_learning_lab::ldap_schema::{self as s, Category, ClassKind, Syntax};
const OID: &str = "1.3.6.1.4.1.32473.90.1";
fn attribute(fields: &str) -> String {
    format!("( {OID} {fields} )")
}
#[test]
fn attribute_description_is_typed_and_preserves_syntax_hint() {
    let a=s::attribute_type(&attribute("NAME ( 'labCode' 'labAlias' ) DESC 'Room \\27code\\27' EQUALITY caseExactMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{8} SINGLE-VALUE")).unwrap();
    assert_eq!(a.oid, OID);
    assert_eq!(a.names, ["labCode", "labAlias"]);
    assert_eq!(a.description.as_deref(), Some("Room 'code'"));
    assert_eq!(a.equality.as_deref(), Some("caseexactmatch"));
    assert!(a.single_value);
    let syntax = a.syntax.unwrap();
    assert_eq!(syntax.kind, Syntax::DirectoryString);
    assert_eq!(syntax.minimum_size_hint, Some(8));
}
#[test]
fn inherited_types_and_nameless_numeric_definitions_parse() {
    let a = s::attribute_type(&attribute("SUP labCode")).unwrap();
    assert_eq!(a.superior.as_deref(), Some("labcode"));
    assert!(a.syntax.is_none());
    assert!(a.names.is_empty());
    assert!(s::attribute_type("(1.2 SYNTAX 1.3.6.1.4.1.1466.115.121.1.27)").is_ok());
}
#[test]
fn structural_and_abstract_lists_and_default_kind_parse() {
    let a = s::object_class(&attribute(
        "NAME 'labDevice' SUP top STRUCTURAL MUST ( cn $ labCode ) MAY description",
    ))
    .unwrap();
    assert_eq!(a.kind, ClassKind::Structural);
    assert_eq!(a.must, ["cn", "labcode"]);
    assert_eq!(a.may, ["description"]);
    assert_eq!(
        s::object_class("( 1.2 NAME 'abstractLab' ABSTRACT )")
            .unwrap()
            .kind,
        ClassKind::Abstract
    );
    assert_eq!(
        s::object_class("( 1.2 SUP top )").unwrap().kind,
        ClassKind::Structural
    );
}
#[test]
fn canonical_numeric_oids_do_not_have_machine_integer_overflow() {
    for yes in [
        "1.2",
        "0.0",
        "1.99999999999999999999999999999999999999999999999",
    ] {
        assert!(s::numeric_oid(yes), "{yes}");
    }
    for no in [
        "1", "1.02", ".1.2", "1.2.", "1..2", "-1.2", "1.a", "1.2\n", "１.2",
    ] {
        assert!(!s::numeric_oid(no), "{no}");
    }
}
#[test]
fn missing_separators_and_malformed_descriptions_are_errors() {
    for raw in [
        "( 1.2 NAME'x' SUP cn )",
        "(1.2 NAME ( 'x''y' ) SUP cn)",
        "(1.2 NAME 'x'SUP cn)",
        "(1.2 NAME",
        "(1.2 NAME (",
        "(1.2 NAME 'x' NAME 'y' SUP cn)",
        "(1.2 SUP cn NAME 'x')",
        "(1.2 SUP)",
        "(1.2)",
        "(1.2 SUP cn) trailing",
        "(1.2 NAME 'x'",
        "(1.2 NAME '\\78' SUP cn)",
    ] {
        assert!(s::attribute_type(raw).is_err(), "{raw}");
    }
    for raw in [
        "(1.2 SUP (top $))",
        "(1.2 MUST ())",
        "(1.2 STRUCTURAL ABSTRACT)",
        "(1.2 MAY cn MUST sn)",
    ] {
        assert!(s::object_class(raw).is_err(), "{raw}");
    }
}
#[test]
fn unsupported_fields_and_syntax_are_not_silently_approximated() {
    for field in [
        "OBSOLETE",
        "ORDERING integerOrderingMatch",
        "SUBSTR caseIgnoreSubstringsMatch",
        "USAGE directoryOperation",
        "X-ORIGIN 'fixture'",
        "SYNTAX 1.2.999",
    ] {
        let raw = if field.starts_with("SYNTAX") {
            attribute(field)
        } else {
            attribute(&format!("SUP cn {field}"))
        };
        assert_eq!(
            s::attribute_type(&raw).unwrap_err().category,
            Category::Unsupported,
            "{raw}"
        );
    }
    assert_eq!(
        s::object_class("(1.2 AUXILIARY)").unwrap_err().category,
        Category::Unsupported
    );
}
#[test]
fn aliases_are_case_insensitively_distinct_and_bounded() {
    assert_eq!(
        s::attribute_type(&attribute("NAME ('code' 'CODE') SUP cn"))
            .unwrap_err()
            .category,
        Category::Conflict
    );
    assert_eq!(
        s::attribute_type(&attribute("NAME ('a' 'b' 'c' 'd' 'e') SUP cn"))
            .unwrap_err()
            .category,
        Category::Limit
    );
    assert!(s::attribute_type(&attribute("NAME 'a_b' SUP cn")).is_err());
}
#[test]
fn size_hint_is_capability_not_a_value_maximum() {
    for hint in ["0", "1", "256"] {
        assert!(
            s::attribute_type(&attribute(&format!(
                "SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{{{hint}}}"
            )))
            .is_ok()
        );
    }
    assert_eq!(
        s::attribute_type(&attribute("SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{257}"))
            .unwrap_err()
            .category,
        Category::Limit
    );
    assert!(s::attribute_type(&attribute("SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{01}")).is_err());
}
#[test]
fn bounded_whitespace_and_locations_are_usable() {
    assert!(s::attribute_type("( 1.2\r\n\tNAME 'fixture'\n SUP cn )").is_ok());
    let e = s::attribute_type("( 1.2\n NAME 'fixture'\n UNKNOWN value )").unwrap_err();
    assert_eq!(e.line, 3);
    assert!(e.column > 0);
    assert_eq!(
        s::attribute_type("(1.2\rSUP cn)").unwrap_err().category,
        Category::Unsupported
    );
    assert_eq!(
        s::attribute_type(&" ".repeat(2049)).unwrap_err().category,
        Category::Limit
    );
}
#[test]
fn every_prefix_and_non_ascii_escape_boundary_returns_without_panicking() {
    let texts = [
        "( 1.2 NAME 'fixture' SUP cn )",
        "( 1.2 DESC '\\27hello\\27' SUP cn )",
        "( 1.2 NAME 'é' SUP cn )",
        "( 1.2 NAME 'fixture' SUP 💥 )",
        "( 1.2 DESC '\\é' SUP cn )",
    ];
    for text in texts {
        for n in 0..=text.len() {
            if text.is_char_boundary(n) {
                let _ = s::attribute_type(&text[..n]);
                let _ = s::object_class(&text[..n]);
            }
        }
    }
    for control in ['\0', '\u{7f}', '\u{1b}', 'é', '💥'] {
        let raw = attribute(&format!("NAME 'a{control}' SUP cn"));
        assert!(s::attribute_type(&raw).is_err());
    }
}
#[test]
fn valid_but_unmodeled_empty_name_lists_are_not_reported_as_supported() {
    assert_eq!(
        s::attribute_type("(1.2 NAME () SUP cn)")
            .unwrap_err()
            .category,
        Category::Unsupported
    );
    assert_eq!(
        s::object_class("(1.2 NAME () SUP top)")
            .unwrap_err()
            .category,
        Category::Unsupported
    );
}
