use dlms_learning_lab::{ldap_changes as c, ldap_ldif as l, ldap_query as q};
#[path = "support/ldap_directory.rs"]
mod fixture;
use fixture::{attr, fixture};
const BASE: &str = "ou=People,dc=example,dc=test";
const ALEX: &str = "uid=alex,ou=People,dc=example,dc=test";
fn dn(s: &str) -> q::Dn {
    q::Dn::parse(s).unwrap()
}
fn grant() -> c::Grant {
    c::Grant::new(dn(ALEX), dn(BASE), true, true, &["mail", "description"]).unwrap()
}
fn directory() -> c::Directory {
    c::Directory::new(&fixture()).unwrap()
}
fn entry() -> q::EntrySource {
    q::EntrySource {
        dn: format!("uid=ava,{BASE}"),
        attributes: vec![
            attr("uid", &["ava"]),
            attr("cn", &["Ava Reed"]),
            attr("sn", &["Reed"]),
            attr("objectClass", &["inetOrgPerson"]),
            attr("mail", &["ava@example.test"]),
        ],
    }
}
fn edit(kind: l::EditKind, name: &str, values: &[&str]) -> l::Edit {
    l::Edit {
        kind,
        attribute: attr(name, values),
    }
}
fn modify(edits: Vec<l::Edit>) -> l::Change {
    l::Change::Modify {
        dn: dn(ALEX),
        edits,
    }
}
fn field<'a>(d: &'a c::Directory, name: &str) -> &'a [String] {
    d.source()
        .entries
        .iter()
        .find(|e| e.dn == ALEX)
        .unwrap()
        .attributes
        .iter()
        .find(|a| a.name == name)
        .map_or(&[], |a| a.values.as_slice())
}
fn code(result: Result<c::Applied, c::Error>, expected: c::Code) {
    assert!(
        matches!(result,Err(c::Error::Result{code,..}) if code==expected),
        "{expected:?}"
    );
}
#[test]
fn add_read_modify_delete_changes_actual_entries_and_expands_superclasses() {
    let mut d = directory();
    let g = grant();
    let e = entry();
    let newdn = dn(&e.dn);
    assert_eq!(
        d.apply(&l::Change::Add(e), &g).unwrap().operation,
        c::Operation::Added
    );
    let q::SearchResult::Entries(rows) = d.read().search(
        &newdn,
        q::Scope::Base,
        &q::Filter::parse("(objectClass=person)").unwrap(),
    ) else {
        panic!()
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0]
            .attributes()
            .iter()
            .find(|a| a.name == "objectclass")
            .unwrap()
            .values,
        ["top", "person", "organizationalperson", "inetorgperson"]
    );
    d.apply(
        &l::Change::Modify {
            dn: newdn.clone(),
            edits: vec![edit(l::EditKind::Replace, "mail", &["new@example.test"])],
        },
        &g,
    )
    .unwrap();
    assert_eq!(
        d.apply(&l::Change::Delete(newdn.clone()), &g)
            .unwrap()
            .operation,
        c::Operation::Deleted
    );
    assert!(matches!(
        d.read().search(
            &newdn,
            q::Scope::Base,
            &q::Filter::parse("(objectClass=*)").unwrap()
        ),
        q::SearchResult::NoSuchObject
    ));
}
#[test]
fn add_duplicate_and_missing_parent_leave_the_directory_unchanged() {
    let mut d = directory();
    let g = grant();
    let original = d.clone();
    code(
        d.apply(&l::Change::Add(d.source().entries[2].clone()), &g),
        c::Code::EntryAlreadyExists,
    );
    let mut e = entry();
    e.dn = format!("uid=ava,ou=Missing,{BASE}");
    code(d.apply(&l::Change::Add(e), &g), c::Code::NoSuchObject);
    assert_eq!(d, original);
}
#[test]
fn ordered_modify_can_add_then_delete_and_preserves_unrelated_values() {
    let mut d = directory();
    let g = grant();
    d.apply(
        &modify(vec![
            edit(l::EditKind::Add, "MAIL", &["new@example.test"]),
            edit(l::EditKind::Delete, "mail", &["ALEX@example.test"]),
        ]),
        &g,
    )
    .unwrap();
    assert_eq!(
        field(&d, "mail"),
        ["a.rivera@example.test", "new@example.test"]
    );
    assert_eq!(field(&d, "sn"), ["Rivera"]);
}
#[test]
fn later_failed_edit_rolls_back_every_edit_in_that_request() {
    let mut d = directory();
    let g = grant();
    let before = d.clone();
    code(
        d.apply(
            &modify(vec![
                edit(l::EditKind::Replace, "description", &["changed"]),
                edit(l::EditKind::Delete, "mail", &["missing@example.test"]),
            ]),
            &g,
        ),
        c::Code::NoSuchAttribute,
    );
    assert_eq!(d, before);
    code(
        d.apply(
            &modify(vec![
                edit(l::EditKind::Add, "description", &["on-call"]),
                edit(l::EditKind::Add, "mail", &[" Alex@example.test "]),
            ]),
            &g,
        ),
        c::Code::AttributeOrValueExists,
    );
    assert_eq!(d, before);
}
#[test]
fn replacement_and_deletion_have_different_absent_attribute_semantics() {
    let mut d = directory();
    let g = grant();
    d.apply(
        &modify(vec![edit(l::EditKind::Replace, "description", &[])]),
        &g,
    )
    .unwrap();
    code(
        d.apply(
            &modify(vec![edit(l::EditKind::Delete, "description", &[])]),
            &g,
        ),
        c::Code::NoSuchAttribute,
    );
    d.apply(
        &modify(vec![edit(
            l::EditKind::Replace,
            "description",
            &["New role"],
        )]),
        &g,
    )
    .unwrap();
    assert_eq!(field(&d, "description"), ["New role"]);
    d.apply(
        &modify(vec![edit(
            l::EditKind::Delete,
            "description",
            &[" new  ROLE "],
        )]),
        &g,
    )
    .unwrap();
    assert!(field(&d, "description").is_empty());
    d.apply(&modify(vec![edit(l::EditKind::Delete, "mail", &[])]), &g)
        .unwrap();
    assert!(field(&d, "mail").is_empty());
}
#[test]
fn delete_of_a_non_leaf_person_fails_without_recursive_side_effects() {
    let mut s = fixture();
    let mut child = entry();
    child.dn = format!("uid=ava,{ALEX}");
    s.entries.push(child);
    let mut d = c::Directory::new(&s).unwrap();
    let before = d.clone();
    code(
        d.apply(&l::Change::Delete(dn(ALEX)), &grant()),
        c::Code::NotAllowedOnNonLeaf,
    );
    assert_eq!(d, before);
    code(
        d.apply(
            &l::Change::Delete(dn(&format!("uid=absent,{BASE}"))),
            &grant(),
        ),
        c::Code::NoSuchObject,
    );
}
#[test]
fn content_import_and_change_batches_stop_but_keep_prior_requests() {
    let mut d = directory();
    let g = grant();
    let e = entry();
    let b = d
        .apply_document(
            &l::Document::Changes(vec![
                l::Change::Add(e.clone()),
                l::Change::Add(e.clone()),
                modify(vec![edit(
                    l::EditKind::Replace,
                    "mail",
                    &["never@example.test"],
                )]),
            ]),
            &g,
        )
        .unwrap();
    assert!(!b.succeeded());
    assert_eq!(b.results.len(), 2);
    assert_eq!(b.unattempted, 1);
    assert!(b.results[0].is_ok());
    assert!(matches!(
        b.results[1],
        Err(c::Error::Result {
            code: c::Code::EntryAlreadyExists,
            ..
        })
    ));
    assert_eq!(d.read().entries().len(), 8);
    assert_eq!(field(&d, "mail").len(), 2);
    let mut d = directory();
    let b = d
        .apply_document(&l::Document::Content(vec![e]), &g)
        .unwrap();
    assert!(b.succeeded());
    assert_eq!(b.unattempted, 0);
}
#[test]
fn parser_to_evaluator_is_data_not_a_command_or_external_resource() {
    let raw = format!(
        "version: 1\n\ndn: {ALEX}\nchangetype: modify\nreplace: description\ndescription: $(touch /not-a-host-command); file:///not-opened\n-\n"
    );
    let doc = l::parse(&raw).unwrap();
    let mut d = directory();
    assert!(d.apply_document(&doc, &grant()).unwrap().succeeded());
    assert_eq!(
        field(&d, "description"),
        ["$(touch /not-a-host-command); file:///not-opened"]
    );
    assert!(
        l::parse(&raw.replace(
            "description: $(touch /not-a-host-command); file:///not-opened",
            "description:< file:///not-opened"
        ))
        .is_err()
    );
}
#[test]
fn authorization_is_explicit_and_cannot_be_requested_by_ldif() {
    let mut d = directory();
    let original = d.clone();
    let request = modify(vec![edit(
        l::EditKind::Replace,
        "mail",
        &["new@example.test"],
    )]);
    for g in [
        c::Grant::new(dn(ALEX), dn(BASE), false, false, &[]).unwrap(),
        c::Grant::new(
            dn(ALEX),
            dn("ou=Teams,dc=example,dc=test"),
            true,
            true,
            &["mail"],
        )
        .unwrap(),
        c::Grant::new(
            dn("uid=absent,dc=example,dc=test"),
            dn(BASE),
            true,
            true,
            &["mail"],
        )
        .unwrap(),
        c::Grant::new(dn(BASE), dn(BASE), true, true, &["mail"]).unwrap(),
        c::Grant::new(
            dn(ALEX),
            dn("ou=Absent,dc=example,dc=test"),
            true,
            true,
            &["mail"],
        )
        .unwrap(),
    ] {
        code(d.apply(&request, &g), c::Code::InsufficientAccessRights);
        assert_eq!(d, original);
    }
    let limited = c::Grant::new(dn(ALEX), dn(BASE), false, false, &["description"]).unwrap();
    code(
        d.apply(
            &modify(vec![
                edit(l::EditKind::Replace, "description", &["changed"]),
                edit(l::EditKind::Replace, "mail", &["new@example.test"]),
            ]),
            &limited,
        ),
        c::Code::InsufficientAccessRights,
    );
    code(
        d.apply(&l::Change::Add(entry()), &limited),
        c::Code::InsufficientAccessRights,
    );
    code(
        d.apply(&l::Change::Delete(dn(ALEX)), &limited),
        c::Code::InsufficientAccessRights,
    );
    assert_eq!(d, original);
    assert!(c::Grant::new(dn(ALEX), dn(BASE), true, true, &["cn"]).is_err());
    assert!(c::Grant::new(dn(ALEX), dn(BASE), true, true, &["mail", "mail"]).is_err());
}
#[test]
fn grant_applies_to_parsed_dn_components_not_a_string_suffix() {
    let mut s = fixture();
    let mut e = entry();
    e.dn = "uid=ava,ou=Teams,dc=example,dc=test".into();
    s.entries.push(e.clone());
    let mut d = c::Directory::new(&s).unwrap();
    let before = d.clone();
    code(
        d.apply(&l::Change::Delete(dn(&e.dn)), &grant()),
        c::Code::InsufficientAccessRights,
    );
    assert_eq!(d, before);
    let g = c::Grant::new(
        dn("UID=Alex,OU=people,DC=EXAMPLE,DC=test"),
        dn("OU=people,DC=example,DC=TEST"),
        true,
        true,
        &["mail"],
    )
    .unwrap();
    d.apply(
        &modify(vec![edit(
            l::EditKind::Replace,
            "mail",
            &["new@example.test"],
        )]),
        &g,
    )
    .unwrap();
}
#[test]
fn controlling_schema_requires_person_fields_and_rejects_invalid_class_mixes() {
    let mut d = directory();
    let before = d.clone();
    let g = grant();
    for missing in ["cn", "sn"] {
        let mut e = entry();
        e.attributes.retain(|a| a.name != missing);
        code(
            d.apply(&l::Change::Add(e), &g),
            c::Code::ObjectClassViolation,
        );
        assert_eq!(d, before);
    }
    let mut e = entry();
    e.attributes
        .iter_mut()
        .find(|a| a.name == "objectClass")
        .unwrap()
        .values
        .push("domain".into());
    code(
        d.apply(&l::Change::Add(e), &g),
        c::Code::ObjectClassViolation,
    );
    let mut e = entry();
    e.attributes
        .iter_mut()
        .find(|a| a.name == "objectClass")
        .unwrap()
        .values = vec!["person".into()];
    code(
        d.apply(&l::Change::Add(e), &g),
        c::Code::ObjectClassViolation,
    );
    let mut e = entry();
    e.attributes.push(attr("ou", &["wrong class"]));
    code(
        d.apply(&l::Change::Add(e), &g),
        c::Code::ObjectClassViolation,
    );
    assert_eq!(d, before);
    let mut s = fixture();
    s.entries[2].attributes.retain(|a| a.name != "sn");
    assert!(q::Directory::new(&s).is_ok());
    assert!(c::Directory::new(&s).is_err());
}
#[test]
fn unsupported_naming_password_class_and_container_operations_are_honest_errors() {
    let mut d = directory();
    let before = d.clone();
    let g = grant();
    for name in ["uid", "cn", "objectClass", "userPassword", "unknown"] {
        assert!(matches!(
            d.apply(
                &modify(vec![edit(l::EditKind::Replace, name, &["new"])]),
                &g
            ),
            Err(c::Error::Unsupported(_))
        ));
        assert_eq!(d, before);
    }
    assert!(matches!(
        d.apply(&l::Change::Delete(dn(BASE)), &g),
        Err(c::Error::Unsupported(_))
    ));
    let mut container = entry();
    container.dn = format!("ou=New,{BASE}");
    container.attributes = vec![
        attr("ou", &["New"]),
        attr("objectClass", &["organizationalUnit"]),
    ];
    assert!(matches!(
        d.apply(&l::Change::Add(container), &g),
        Err(c::Error::Unsupported(_))
    ));
    assert_eq!(d, before);
}
#[test]
fn invalid_hand_constructed_values_are_bounded_before_copying_or_mutating() {
    let mut d = directory();
    let before = d.clone();
    let g = grant();
    let mut e = entry();
    e.attributes.push(attr("description", &[&"a".repeat(257)]));
    assert!(matches!(
        d.apply(&l::Change::Add(e), &g),
        Err(c::Error::Limit(_))
    ));
    for bad in [
        vec![],
        vec![edit(l::EditKind::Add, "mail", &[])],
        vec![edit(l::EditKind::Replace, "mail", &["same", " SAME "])],
        vec![edit(l::EditKind::Replace, "mail", &["bad\nvalue"])],
        vec![edit(l::EditKind::Replace, "mail", &[""])],
    ] {
        assert!(d.apply(&modify(bad), &g).is_err());
        assert_eq!(d, before);
    }
    assert!(d.apply_document(&l::Document::Changes(vec![]), &g).is_err());
    assert!(
        d.apply_document(
            &l::Document::Changes(vec![l::Change::Delete(dn(ALEX)); 17]),
            &g
        )
        .is_err()
    );
    assert_eq!(d, before);
}
#[test]
fn final_result_must_satisfy_schema_and_limits_even_when_intermediate_edits_were_valid() {
    let mut d = directory();
    let g = grant();
    let before = d.clone();
    let values: Vec<String> = (0..16)
        .map(|i| format!("address{i}@example.test"))
        .collect();
    let request = modify(vec![l::Edit {
        kind: l::EditKind::Add,
        attribute: q::Attribute {
            name: "mail".into(),
            values,
        },
    }]);
    assert!(matches!(d.apply(&request, &g), Err(c::Error::Limit(_))));
    assert_eq!(d, before);
    let mut s = fixture();
    s.entries[2].dn = "cn=Alex Rivera,ou=People,dc=example,dc=test".into();
    s.entries[2]
        .attributes
        .retain(|a| !matches!(a.name.as_str(), "uid" | "mail"));
    s.entries[2]
        .attributes
        .iter_mut()
        .find(|a| a.name == "objectClass")
        .unwrap()
        .values = vec!["person".into()];
    let principal = dn(&s.entries[2].dn);
    let g = c::Grant::new(principal.clone(), dn(BASE), true, true, &["mail"]).unwrap();
    let mut d = c::Directory::new(&s).unwrap();
    let before = d.clone();
    code(
        d.apply(
            &l::Change::Modify {
                dn: principal,
                edits: vec![edit(
                    l::EditKind::Replace,
                    "mail",
                    &["not-allowed@example.test"],
                )],
            },
            &g,
        ),
        c::Code::ObjectClassViolation,
    );
    assert_eq!(d, before);
}

#[test]
fn domain_components_follow_the_declared_label_syntax() {
    for label in [
        "two words",
        "-start",
        "end-",
        "under_score",
        &"a".repeat(64),
    ] {
        let mut source = fixture();
        source.suffix = format!("dc={label},dc=test");
        source.entries = vec![q::EntrySource {
            dn: source.suffix.clone(),
            attributes: vec![attr("dc", &[label]), attr("objectClass", &["domain"])],
        }];
        assert!(c::Directory::new(&source).is_err(), "{label}");
    }
    let source = q::Source {
        version: 1,
        suffix: "dc=sample-lab,dc=test".into(),
        entries: vec![q::EntrySource {
            dn: "dc=sample-lab,dc=test".into(),
            attributes: vec![
                attr("dc", &["sample-lab"]),
                attr("objectClass", &["domain"]),
            ],
        }],
    };
    assert!(c::Directory::new(&source).is_ok());
}

#[test]
fn add_combines_original_rdn_value_with_supplied_attributes_without_duplicate_equality() {
    let g = grant();
    let mut d = directory();
    let mut e = entry();
    e.attributes.retain(|a| a.name != "uid");
    d.apply(&l::Change::Add(e.clone()), &g).unwrap();
    let added = d.source().entries.last().unwrap();
    assert_eq!(
        added
            .attributes
            .iter()
            .find(|a| a.name == "uid")
            .unwrap()
            .values,
        ["ava"]
    );
    let mut d = directory();
    e.dn = r"cn=Reed\, Ava,ou=People,dc=example,dc=test".into();
    e.attributes.retain(|a| a.name != "cn");
    d.apply(&l::Change::Add(e), &g).unwrap();
    assert_eq!(
        d.source()
            .entries
            .last()
            .unwrap()
            .attributes
            .iter()
            .find(|a| a.name == "cn")
            .unwrap()
            .values,
        ["Reed, Ava"]
    );
    let mut d = directory();
    let mut e = entry();
    e.attributes
        .iter_mut()
        .find(|a| a.name == "uid")
        .unwrap()
        .values = vec![" AVA ".into()];
    d.apply(&l::Change::Add(e), &g).unwrap();
    assert_eq!(
        d.source()
            .entries
            .last()
            .unwrap()
            .attributes
            .iter()
            .find(|a| a.name == "uid")
            .unwrap()
            .values,
        [" AVA "]
    );
    let mut d = directory();
    let mut e = entry();
    e.attributes
        .iter_mut()
        .find(|a| a.name == "uid")
        .unwrap()
        .values = vec!["alias".into()];
    d.apply(&l::Change::Add(e), &g).unwrap();
    assert_eq!(
        d.source()
            .entries
            .last()
            .unwrap()
            .attributes
            .iter()
            .find(|a| a.name == "uid")
            .unwrap()
            .values,
        ["alias", "ava"]
    );
}
