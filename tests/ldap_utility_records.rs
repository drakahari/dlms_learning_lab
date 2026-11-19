use dlms_learning_lab::ldap_ldif as l;
fn records(raw: &str, add: bool) -> Vec<Result<l::Change, l::Diagnostic>> {
    l::UtilityRecords::new(raw, add).unwrap().collect()
}
const CONTENT: &str = "dn: uid=new,ou=People,dc=example,dc=test\nobjectClass: inetOrgPerson\nuid: new\ncn: New Person\nsn: Person\n";
const MODIFY: &str = "dn: uid=alex,ou=People,dc=example,dc=test\nchangetype: modify\nreplace: description\ndescription: verified\n-\n";
#[test]
fn explicit_operations_override_the_add_default() {
    for (record, operation) in [
        (MODIFY, "modify"),
        (
            "dn: uid=sam,ou=People,dc=example,dc=test\nchangetype: delete\n",
            "delete",
        ),
    ] {
        let raw = format!("version: 1\n\n{record}");
        let normal = records(&raw, false);
        assert_eq!(normal, records(&raw, true));
        assert_eq!(normal.len(), 1);
        match operation {
            "modify" => assert!(matches!(normal[0], Ok(l::Change::Modify { .. }))),
            _ => assert!(matches!(normal[0], Ok(l::Change::Delete(_)))),
        }
    }
}
#[test]
fn content_needs_add_default_but_is_not_an_implicit_modify() {
    let raw = format!("version: 1\n{CONTENT}");
    assert!(matches!(records(&raw, true)[0], Ok(l::Change::Add(_))));
    assert!(matches!(
        records(&raw, false)[0],
        Err(l::Diagnostic {
            category: l::Category::Unsupported,
            ..
        })
    ));
}
#[test]
fn each_record_is_parsed_only_when_requested_and_errors_are_fused() {
    let raw = format!("version: 1\n{MODIFY}\nthis is not a dn\n\n{CONTENT}");
    let mut input = l::UtilityRecords::new(&raw, true).unwrap();
    assert!(matches!(input.next(), Some(Ok(l::Change::Modify { .. }))));
    let e = input.next().unwrap().unwrap_err();
    assert_eq!(e.line, 8);
    assert_eq!(e.category, l::Category::Syntax);
    assert!(input.next().is_none());
    assert!(input.next().is_none());
}
#[test]
fn physical_error_lines_survive_comments_crlf_and_folding() {
    let raw = format!(
        "# initial\r\nversion: 1\r\n\r\n{}\r\n# second\r\ndn: uid=bad,dc=example,dc=test\r\nchangetype: modrdn\r\n",
        MODIFY.replace('\n', "\r\n")
    );
    let got = records(&raw, false);
    assert!(got[0].is_ok());
    let e = got[1].as_ref().unwrap_err();
    assert_eq!(e.line, 12);
    assert_eq!(e.category, l::Category::Unsupported);
    let folded = "version: 1\n\n# ignored\n continuation\ndn: uid=new,ou=People,dc=example,dc=test\nchangetype: add\nobjectClass: inetOrgPerson\nuid: new\ncn: New\n  Person\nsn: Person\n";
    let got = records(folded, true);
    let Ok(l::Change::Add(e)) = &got[0] else {
        panic!("{got:?}")
    };
    assert_eq!(
        e.attributes.iter().find(|a| a.name == "cn").unwrap().values,
        ["New Person"]
    );
}
#[test]
fn aggregate_resource_limits_fail_before_any_record_is_available() {
    assert!(matches!(
        l::UtilityRecords::new(&"x".repeat(l::MAX_TEXT + 1), true),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    assert!(l::UtilityRecords::new(&"\n".repeat(l::MAX_LINES + 1), true).is_err());
    let raw = format!(
        "version: 1\n\n{}",
        format!("{MODIFY}\n").repeat(l::MAX_RECORDS + 1)
    );
    let got = records(&raw, false);
    assert_eq!(got.len(), l::MAX_RECORDS + 1);
    assert!(got[..l::MAX_RECORDS].iter().all(Result::is_ok));
    assert!(matches!(
        got.last(),
        Some(Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        }))
    ));
}
#[test]
fn header_empty_input_malformed_fields_and_unsupported_resources_are_honest() {
    for raw in [
        "",
        "# comment only",
        "version: 1\n",
        "version: 2\n",
        CONTENT,
    ] {
        let got = records(raw, true);
        assert_eq!(got.len(), 1, "{raw}");
        assert!(got[0].is_err());
    }
    for tail in [
        "bad field",
        "mail:< file:///home/personal",
        "mail: héllo",
        "mail: value\u{0}",
    ] {
        let raw = format!("version: 1\n{MODIFY}\n{CONTENT}{tail}\n");
        let got = records(&raw, true);
        assert!(got[0].is_ok());
        assert!(got[1].is_err());
        assert_eq!(got.len(), 2);
    }
}
#[test]
fn content_and_explicit_change_records_may_follow_each_other_in_the_utility() {
    let raw = format!("version: 1\n{CONTENT}\n{MODIFY}\n");
    let got = records(&raw, true);
    assert_eq!(got.len(), 2);
    assert!(got.iter().all(Result::is_ok));
    // Preserve the established editor action's stricter whole-document contract.
    assert!(l::parse(&raw).is_err());
}
#[test]
fn arbitrary_prefixes_and_host_escape_text_never_panic_or_trigger_io() {
    let text = format!("version: 1\n{CONTENT}\n{MODIFY}");
    for end in 0..=text.len() {
        let raw = &text[..end];
        let got = records(raw, true);
        assert!(got.len() <= l::MAX_RECORDS + 1);
    }
    for text in [
        "$(touch /tmp/no)",
        "`id`",
        "../../etc/passwd",
        "https://example.test",
        "include: /etc/ldap.conf",
    ] {
        assert!(records(&format!("version: 1\n{text}"), true)[0].is_err());
    }
}
