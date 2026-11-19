use dlms_learning_lab::{ldap_ldif as l, ldap_query::Attribute};
fn content(body: &str) -> String {
    format!("version: 1\n\ndn: uid=ava,ou=People,dc=example,dc=test\n{body}\n")
}
fn modify(body: &str) -> String {
    format!("version: 1\n\ndn: uid=ava,ou=People,dc=example,dc=test\nchangetype: modify\n{body}")
}
#[test]
fn content_and_add_records_produce_real_typed_values_without_applying_them() {
    let raw = content(
        "objectClass: inetOrgPerson\nuid: ava\ncn: Ava Reed\nsn: Reed\nmail: ava@example.test",
    );
    let l::Document::Content(rows) = l::parse(&raw).unwrap() else {
        panic!()
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].dn, "uid=ava,ou=People,dc=example,dc=test");
    assert_eq!(
        rows[0].attributes[0],
        Attribute {
            name: "objectclass".into(),
            values: vec!["inetOrgPerson".into()]
        }
    );
    let changed = raw.replace("objectClass:", "changetype: add\nobjectClass:");
    let l::Document::Changes(changes) = l::parse(&changed).unwrap() else {
        panic!()
    };
    assert_eq!(changes, [l::Change::Add(rows[0].clone())]);
}
#[test]
fn change_document_preserves_request_order_and_does_not_claim_batch_execution() {
    let raw = "version: 1\n\ndn: uid=ava,dc=test\nchangetype: add\nuid: ava\n\ndn: uid=ava,dc=test\nchangetype: modify\nreplace: description\ndescription: on-call\n-\n\ndn: uid=ava,dc=test\nchangetype: delete\n";
    let l::Document::Changes(rows) = l::parse(raw).unwrap() else {
        panic!()
    };
    assert_eq!(rows.len(), 3);
    assert!(matches!(rows[0], l::Change::Add(_)));
    assert!(matches!(rows[1], l::Change::Modify { .. }));
    assert!(matches!(rows[2], l::Change::Delete(_)));
}
#[test]
fn modification_order_empty_delete_and_empty_replace_are_distinct_from_add() {
    let raw = modify(
        "add: mail\nmail: another@example.test\n-\ndelete: mail\nmail: ava@example.test\n-\nreplace: description\n-\ndelete: description\n-\n",
    );
    let l::Document::Changes(rows) = l::parse(&raw).unwrap() else {
        panic!()
    };
    let l::Change::Modify { edits, .. } = &rows[0] else {
        panic!()
    };
    assert_eq!(
        edits.iter().map(|e| e.kind).collect::<Vec<_>>(),
        [
            l::EditKind::Add,
            l::EditKind::Delete,
            l::EditKind::Replace,
            l::EditKind::Delete
        ]
    );
    assert!(edits[2].attribute.values.is_empty());
    assert!(edits[3].attribute.values.is_empty());
    assert!(matches!(
        l::parse(&modify("add: mail\n-\n")),
        Err(l::Diagnostic {
            category: l::Category::Syntax,
            ..
        })
    ));
}
#[test]
fn comments_are_unfolded_before_they_are_ignored_and_crlf_is_supported() {
    let raw = "# preface\n continued comment\nversion: 1\n\ndn: uid=ava,ou=People,\n dc=example,dc=test\n# comment\n ignored too\ncn: Ava \n Reed\nuid: ava\n";
    let l::Document::Content(rows) = l::parse(raw).unwrap() else {
        panic!()
    };
    assert_eq!(rows[0].dn, "uid=ava,ou=People,dc=example,dc=test");
    assert_eq!(rows[0].attributes[0].values, ["Ava Reed"]);
    assert_eq!(
        l::parse(&raw.replace('\n', "\r\n")).unwrap(),
        l::parse(raw).unwrap()
    );
    let joined = l::parse(&content("description: note\n #literal continuation")).unwrap();
    let l::Document::Content(rows) = joined else {
        panic!()
    };
    assert_eq!(rows[0].attributes[0].values, ["note#literal continuation"]);
}
#[test]
fn repeated_attribute_names_collect_values_without_silent_deduplication() {
    let l::Document::Content(rows) = l::parse(&content(
        "MAIL: ava@example.test\nmail: second@example.test\nmail: ava@example.test",
    ))
    .unwrap() else {
        panic!()
    };
    assert_eq!(rows[0].attributes.len(), 1);
    assert_eq!(rows[0].attributes[0].name, "mail");
    assert_eq!(
        rows[0].attributes[0].values,
        [
            "ava@example.test",
            "second@example.test",
            "ava@example.test"
        ]
    );
}
#[test]
fn standard_base64_decodes_supported_values_and_an_encoded_dn() {
    let raw = "version: 1\n\ndn:: dWlkPWF2YSxkYz10ZXN0\ncn:: QXZhIFJlZWQ=\ndescription:: IGxlYWRpbmc=\ndescription:: OmNvbG9u\ndescription:: PHVybA==\n";
    let l::Document::Content(rows) = l::parse(raw).unwrap() else {
        panic!()
    };
    assert_eq!(rows[0].dn, "uid=ava,dc=test");
    assert_eq!(rows[0].attributes[0].values, ["Ava Reed"]);
    assert_eq!(rows[0].attributes[1].values, [" leading", ":colon", "<url"]);
}
#[test]
fn malformed_padding_alphabet_unused_bits_and_non_ascii_decode_fail_honestly() {
    for text in [
        "Zg=", "Zg===", "=g==", "Zh==", "Zm9=", "Zg==Zg==", "Zg-_", "Z g=", "Zg== ",
    ] {
        assert!(
            matches!(
                l::parse(&content(&format!("description:: {text}"))),
                Err(l::Diagnostic {
                    category: l::Category::Syntax,
                    ..
                })
            ),
            "{text}"
        );
    }
    for text in ["AA==", "w6k=", "CQ==", "Cg=="] {
        assert!(
            matches!(
                l::parse(&content(&format!("description:: {text}"))),
                Err(l::Diagnostic {
                    category: l::Category::Unsupported,
                    ..
                })
            ),
            "{text}"
        );
    }
}
#[test]
fn external_resource_references_controls_and_rename_are_never_resolved() {
    for raw in [
        content("description:< file:///etc/passwd"),
        content("description:< https://example.test/resource"),
        "version: 1\n\ndn: uid=ava,dc=test\ncontrol: 1.2.3 true\nchangetype: delete\n".into(),
        "version: 1\n\ndn: uid=ava,dc=test\nchangetype: modrdn\nnewrdn: uid=other\n".into(),
    ] {
        let e = l::parse(&raw).unwrap_err();
        assert_eq!(e.category, l::Category::Unsupported);
    }
    // Encoded printable '<url' is data, never a resource-opening instruction.
    let l::Document::Content(rows) =
        l::parse(&content("description:: PGZpbGU6Ly9leGFtcGxl")).unwrap()
    else {
        panic!()
    };
    assert_eq!(rows[0].attributes[0].values, ["<file://example"]);
}
#[test]
fn unsupported_schema_naming_and_password_modifications_are_not_approximated() {
    for attr in ["uid", "cn", "objectClass", "userPassword", "unknown"] {
        let e = l::parse(&modify(&format!("replace: {attr}\n{attr}: value\n-\n"))).unwrap_err();
        assert_eq!(e.category, l::Category::Unsupported);
    }
}
#[test]
fn mixed_document_types_extra_delete_data_and_repeated_headers_are_rejected() {
    let mixed = format!(
        "{}\ndn: uid=other,dc=test\nchangetype: delete\n",
        content("cn: Ava")
    );
    assert!(l::parse(&mixed).is_err());
    for raw in [
        "version: 1\n\ndn: uid=ava,dc=test\nchangetype: delete\ncn: surplus\n",
        "version: 1\n\ndn: uid=ava,dc=test\nversion: 1\n",
        "version: 1\n\ndn: uid=ava,dc=test\ndn: uid=other,dc=test\n",
    ] {
        assert!(matches!(
            l::parse(raw),
            Err(l::Diagnostic {
                category: l::Category::Syntax,
                ..
            })
        ));
    }
}
#[test]
fn missing_fields_delimiters_and_invalid_names_have_specific_line_feedback() {
    for raw in [
        "",
        "version: 1\n",
        "dn: uid=ava,dc=test\ncn: Ava\n",
        "version: 2\n\ndn: uid=ava,dc=test\ncn: Ava\n",
        "version: 1\n\ncn: missing dn\n",
        "version: 1\n\ndn: uid=ava,dc=test\n",
        " continued without a preceding line\n",
    ] {
        assert!(l::parse(raw).is_err(), "{raw:?}");
    }
    let e = l::parse("version: 1\n\n# comment\ndn: ../etc/passwd\ncn: Ava\n").unwrap_err();
    assert_eq!(e.line, 4);
    assert!(e.to_string().contains("line 4"));
    assert!(matches!(
        l::parse(&modify("replace: mail\nmail: ava@example.test\n")),
        Err(l::Diagnostic {
            category: l::Category::Syntax,
            ..
        })
    ));
    assert!(matches!(
        l::parse(&modify("replace: mail\ndescription: mismatch\n-\n")),
        Err(l::Diagnostic {
            category: l::Category::Syntax,
            ..
        })
    ));
}
#[test]
fn text_lines_records_attributes_and_value_counts_have_independent_limits() {
    assert!(matches!(
        l::parse(&"x".repeat(l::MAX_TEXT + 1)),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    assert!(matches!(
        l::parse(&format!("version: 1\n{}", "#\n".repeat(1024))),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    assert!(matches!(
        l::parse(&content(&format!("description: {}", "x".repeat(2049)))),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    let record = "dn: uid=ava,dc=test\ncn: Ava\n\n";
    assert!(matches!(
        l::parse(&format!("version: 1\n\n{}", record.repeat(17))),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    assert!(matches!(
        l::parse(&content(
            &(0..17)
                .map(|i| format!("attribute{i}: value\n"))
                .collect::<String>()
        )),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    assert!(matches!(
        l::parse(&content(&"mail: ava@example.test\n".repeat(17))),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
    assert!(matches!(
        l::parse(&modify(
            &"replace: description\ndescription: x\n-\n".repeat(17)
        )),
        Err(l::Diagnostic {
            category: l::Category::Limit,
            ..
        })
    ));
}
#[test]
fn decoded_and_plain_values_are_bounded_before_the_model_receives_them() {
    for raw in [
        content(&format!("description: {}", "x".repeat(257))),
        content(&format!("description:: {}YWE=", "YWFh".repeat(85))),
        format!("version: 1\n\ndn: {}\ncn: Ava\n", "x".repeat(1025)),
    ] {
        assert!(matches!(
            l::parse(&raw),
            Err(l::Diagnostic {
                category: l::Category::Limit,
                ..
            })
        ));
    }
    assert!(l::parse(&content("description: René")).is_err());
    assert!(l::parse(&content("description:\tvalue")).is_err());
    assert!(l::parse(&content("description: ")).is_err());
}
#[test]
fn every_truncated_prefix_is_bounded_and_cannot_panic_or_execute() {
    for raw in [
        content("uid: ava\ncn:: QXZhIFJlZWQ="),
        modify("replace: mail\nmail: ava@example.test\n-\ndelete: description\n-\n"),
    ] {
        for end in 0..=raw.len() {
            let _ = l::parse(&raw[..end]);
        }
    }
}
