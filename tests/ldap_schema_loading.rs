use dlms_learning_lab::{
    ldap_ldif::{self, schema as codec},
    ldap_schema::{self as s, Category, Directory, Registry},
};
fn document(name: &str, fields: &str) -> String {
    format!(
        "version: 1\n\ndn: cn={name},cn=schema,cn=config\nchangetype: add\nobjectClass: olcSchemaConfig\ncn: {name}\n{fields}\n"
    )
}
fn attribute(oid: &str, name: &str) -> String {
    format!(
        "( {oid} NAME '{name}' EQUALITY caseExactMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 SINGLE-VALUE )"
    )
}
fn valid() -> String {
    document(
        "workshop",
        &format!(
            "olcAttributeTypes: {}\nolcObjectClasses: (1.3.6.1.4.1.32473.94.1 NAME 'workshopPerson' SUP person MUST workshopCode)",
            attribute("1.3.6.1.4.1.32473.93.1", "workshopCode")
        ),
    )
}
#[test]
fn schema_ldif_applies_actual_registered_definitions_and_module_inventory() {
    let mut d = Directory::new(Registry::teaching_base());
    let load = d.load(&valid());
    assert_eq!(load.error, None);
    assert_eq!(load.applied.len(), 1);
    assert_eq!(d.modules()[0].dn, "cn={1}workshop,cn=schema,cn=config");
    assert!(d.registry().attribute("workshopCode").is_some());
    assert!(d.registry().class("workshopPerson").is_some());
    let source = s::EntrySource {
        dn: "cn=Sam,dc=example,dc=test".into(),
        classes: vec!["workshopPerson".into()],
        fields: vec![
            s::Field {
                name: "cn".into(),
                values: vec!["Sam".into()],
            },
            s::Field {
                name: "sn".into(),
                values: vec!["Learner".into()],
            },
            s::Field {
                name: "workshopCode".into(),
                values: vec!["ALPHA-71".into()],
            },
        ],
    };
    d.add(&source).unwrap();
    assert_eq!(d.entries().len(), 1);
}
#[test]
fn folded_and_indexed_values_and_crlf_are_real_decoder_paths() {
    let raw = valid()
        .replace("NAME 'workshopCode'", "NAME\n  'workshopCode'")
        .replace("olcAttributeTypes: (", "olcAttributeTypes: {0}(")
        .replace("olcObjectClasses: (", "olcObjectClasses: {0}(")
        .replace('\n', "\r\n");
    let adds = codec::parse(&raw).unwrap();
    assert_eq!(adds.len(), 1);
    assert!(!adds[0].attribute_types[0].contains("{0}"));
    let mut d = Directory::new(Registry::teaching_base());
    assert_eq!(d.load(&raw).error, None);
}
#[test]
fn syntax_preflight_cannot_leave_a_partially_applied_document() {
    let mut d = Directory::new(Registry::teaching_base());
    let before = d.clone();
    let raw = format!(
        "{}\ndn: cn=broken,cn=schema,cn=config\nchangetype: delete\n",
        valid()
    );
    let load = d.load(&raw);
    assert!(load.applied.is_empty());
    assert_eq!(load.error.unwrap().category, Category::Unsupported);
    assert_eq!(d, before);
}
#[test]
fn later_schema_rejection_retains_earlier_add_but_not_partial_failed_module() {
    let mut d = Directory::new(Registry::teaching_base());
    let raw=format!("{}\n{}",valid(),document("second","olcAttributeTypes: (1.3.6.1.4.1.32473.93.2 NAME 'second' SUP missing)\nolcObjectClasses: (1.2 NAME 'bad' SUP missing)" ).trim_start_matches("version: 1\n\n"));
    let load = d.load(&raw);
    assert_eq!(load.applied.len(), 1);
    assert_eq!(load.error.unwrap().category, Category::Unsupported);
    assert_eq!(d.modules().len(), 1);
    assert!(d.registry().attribute("workshopCode").is_some());
    assert!(d.registry().attribute("second").is_none());
}
#[test]
fn duplicate_module_and_oid_preserve_existing_registry_and_entries() {
    let mut d = Directory::new(Registry::teaching_base());
    assert!(d.load(&valid()).error.is_none());
    let before = d.clone();
    assert_eq!(d.load(&valid()).error.unwrap().category, Category::Conflict);
    assert_eq!(d, before);
    let raw = document(
        "other",
        &format!(
            "olcAttributeTypes: {}",
            attribute("1.3.6.1.4.1.32473.93.1", "different")
        ),
    );
    assert_eq!(d.load(&raw).error.unwrap().category, Category::Conflict);
    assert_eq!(d, before);
}
#[test]
fn unrelated_branches_external_values_and_unknown_fields_are_rejected() {
    for raw in [
        valid().replace("cn=schema,cn=config", "cn=config"),
        valid().replace("changetype: add", "changetype: modify"),
        valid().replace("cn: workshop", "cn: other"),
        valid().replace("objectClass: olcSchemaConfig", "objectClass: top"),
        valid().replace("olcAttributeTypes:", "olcAttributeTypes:<"),
        valid().replace("olcAttributeTypes:", "olcModuleLoad:"),
        valid().replace("cn=workshop,", "cn={1}workshop,"),
    ] {
        let mut d = Directory::new(Registry::teaching_base());
        let before = d.clone();
        assert!(d.load(&raw).error.is_some(), "{raw}");
        assert_eq!(d, before);
    }
}
#[test]
fn ordered_tags_do_not_silently_reorder_or_drop_definitions() {
    let field = format!(
        "olcAttributeTypes: {{0}}{}\nolcAttributeTypes: {{1}}{}",
        attribute("1.3.6.1.4.1.32473.93.3", "first"),
        attribute("1.3.6.1.4.1.32473.93.4", "second")
    );
    let good = document("ordered", &field);
    assert!(codec::parse(&good).is_ok());
    for raw in [
        good.replace("{1}", "{2}"),
        good.replace("{1}", ""),
        good.replace("{0}", "{00}"),
    ] {
        assert!(codec::parse(&raw).is_err());
    }
}
#[test]
fn old_contact_and_configuration_routes_do_not_accept_schema_operations() {
    // The legacy LDIF layer is intentionally a generic Add decoder. Schema
    // semantics are rejected by its fixed directory, not by that decoder.
    let ldap_ldif::Document::Changes(rows) = ldap_ldif::parse(&valid()).unwrap() else {
        panic!("explicit Add document");
    };
    let ldap_ldif::Change::Add(entry) = &rows[0] else {
        panic!("Add record");
    };
    let legacy = dlms_learning_lab::ldap_query::Source {
        version: 1,
        suffix: entry.dn.clone(),
        entries: vec![entry.clone()],
    };
    assert!(matches!(
        dlms_learning_lab::ldap_query::Directory::new(&legacy),
        Err(dlms_learning_lab::ldap_query::Error::Fixture(_))
    ));
    assert!(ldap_ldif::configuration::parse(&valid()).is_err());
}
#[test]
fn bounded_documents_and_truncated_input_never_mutate_or_panic() {
    let raw = valid();
    for n in 0..=raw.len() {
        let mut d = Directory::new(Registry::teaching_base());
        let before = d.clone();
        let load = d.load(&raw[..n]);
        if load.error.is_some() {
            assert_eq!(d, before);
        }
    }
    assert_eq!(
        codec::parse(&"x".repeat(65537)).unwrap_err().category,
        ldap_ldif::Category::Limit
    );
}
