use dlms_learning_lab::{
    debian_packages::{Architecture as A, Version},
    debian_source_cache as c, debian_sources as s,
};
fn key(component: &str) -> s::Key {
    s::Key {
        uri: "https://packages.workshop.test/debian/".into(),
        suite: "studio".into(),
        component: component.into(),
    }
}
fn text(components: &str) -> String {
    format!(
        "Types: deb\nURIs: https://packages.workshop.test/debian\nSuites: studio\nComponents: {components}\n"
    )
}
fn row(name: &str, v: &str, architecture: A) -> c::Row {
    c::Row {
        name: name.into(),
        version: Version::parse(v).unwrap(),
        architecture,
    }
}
fn target(component: &str) -> c::Target {
    c::Target {
        key: key(component),
        snapshots: vec![
            c::Snapshot {
                id: "old".into(),
                rows: vec![row("ledger-client", "1.9-1", A::All)],
            },
            c::Snapshot {
                id: "new".into(),
                rows: vec![
                    row("ledger-client", "1.10-1", A::All),
                    row("ledger-tool", "2.0-1", A::Amd64),
                    row("foreign-tool", "3.0-1", A::Arm64),
                ],
            },
        ],
        published: Some("new".into()),
    }
}
fn fixture() -> (c::Definition, c::State) {
    let d = c::Definition {
        native: A::Amd64,
        targets: vec![target("main"), target("tools")],
    };
    let state = c::State {
        saved: text("main"),
        cache: vec![c::Cached {
            key: key("main"),
            snapshot: "old".into(),
        }],
    };
    state.validate(&d).unwrap();
    (d, state)
}
#[test]
fn saved_source_and_cached_snapshot_have_different_effects() {
    let (d, mut state) = fixture();
    assert_eq!(
        state.query(&d, "ledger-client").unwrap()[0]
            .row
            .version
            .as_str(),
        "1.9-1"
    );
    assert_eq!(
        state.indexes(&d).unwrap()[0].status,
        c::IndexStatus::Retained
    );
    state.save(&d, &text("tools")).unwrap();
    assert!(state.query(&d, "ledger-client").unwrap().is_empty());
    assert_eq!(state.cache.len(), 1, "Save must not fetch");
    assert_eq!(
        state.indexes(&d).unwrap()[0].status,
        c::IndexStatus::Missing
    );
    state.refresh(&d).unwrap();
    assert_eq!(state.query(&d, "ledger-tool").unwrap()[0].key, key("tools"));
    assert_eq!(state.cache.len(), 2);
    state.save(&d, &text("main")).unwrap();
    assert_eq!(
        state.query(&d, "ledger-client").unwrap()[0]
            .row
            .version
            .as_str(),
        "1.9-1",
        "The old matching main cache remains, but tools cannot leak into selection"
    );
    state.refresh(&d).unwrap();
    assert_eq!(
        state.query(&d, "ledger-client").unwrap()[0]
            .row
            .version
            .as_str(),
        "1.10-1"
    );
    assert_eq!(
        state.indexes(&d).unwrap()[0].status,
        c::IndexStatus::Current
    );
}
#[test]
fn disabled_sources_retain_bytes_without_eligible_metadata_and_reenabling_does_not_fetch() {
    let (d, mut state) = fixture();
    let before = state.cache.clone();
    state.save(&d, &(text("main") + "Enabled: no\n")).unwrap();
    assert!(state.active(&d).unwrap().is_empty());
    assert!(state.indexes(&d).unwrap().is_empty());
    assert!(state.query(&d, "ledger-client").unwrap().is_empty());
    assert!(state.refresh(&d).unwrap().is_empty());
    assert_eq!(state.cache, before);
    state.save(&d, &text("main")).unwrap();
    assert_eq!(state.cache, before);
    assert_eq!(state.query(&d, "ledger-client").unwrap()[0].snapshot, "old");
}
#[test]
fn unavailable_publication_preserves_matching_old_index_and_independent_success() {
    let (mut d, mut state) = fixture();
    d.targets[0].published = None;
    state.save(&d, &text("main tools")).unwrap();
    let results = state.refresh(&d).unwrap();
    assert_eq!(results[0].status, c::RefreshStatus::Unavailable);
    assert_eq!(results[1].status, c::RefreshStatus::Updated);
    let rows = state.query(&d, "ledger-client").unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].snapshot, "old");
    assert_eq!(rows[1].snapshot, "new");
    assert_eq!(
        state.indexes(&d).unwrap()[0].status,
        c::IndexStatus::Retained
    );
}
#[test]
fn unknown_selected_target_has_no_invented_rows_or_acquisition_and_query_is_read_only() {
    let (d, mut state) = fixture();
    state.save(&d, &text("main absent")).unwrap();
    let before = state.clone();
    assert_eq!(state.query(&d, "ledger-client").unwrap().len(), 1);
    assert_eq!(state, before);
    assert_eq!(
        state.indexes(&d).unwrap()[1].status,
        c::IndexStatus::Missing
    );
    let result = state.refresh(&d).unwrap();
    assert_eq!(result[1].status, c::RefreshStatus::Unavailable);
    assert!(!state.cache.iter().any(|x| x.key.component == "absent"));
}
#[test]
fn query_respects_native_architecture_and_keeps_source_provenance_without_apt_policy_claim() {
    let (d, mut state) = fixture();
    state.save(&d, &text("tools main tools")).unwrap();
    state.refresh(&d).unwrap();
    assert!(state.query(&d, "foreign-tool").unwrap().is_empty());
    let rows = state.query(&d, "ledger-client").unwrap();
    assert_eq!(
        rows.len(),
        2,
        "Repeated source selections share an index, distinct sources retain provenance"
    );
    assert_eq!(rows[0].key.component, "tools");
    assert_eq!(rows[1].key.component, "main");
    let mut arm = d.clone();
    arm.native = A::Arm64;
    assert_eq!(state.query(&arm, "foreign-tool").unwrap().len(), 2);
    assert!(state.query(&arm, "ledger-tool").unwrap().is_empty());
    assert_eq!(state.query(&arm, "ledger-client").unwrap().len(), 2);
}
#[test]
fn malformed_and_unmodeled_edits_are_atomic_and_do_not_touch_cached_evidence() {
    let (d, mut state) = fixture();
    let old = state.clone();
    for input in [
        text("main") + "Trusted: yes\n",
        text("main").replace("Types: deb", "Types: deb-src"),
        text("main").replace("Components: main\n", ""),
        "deb https://host.test/repo studio main\n".into(),
        "x".repeat(s::MAX_INPUT + 1),
    ] {
        assert_eq!(state.save(&d, &input).unwrap_err().code, c::Code::Selection);
        assert_eq!(state, old);
    }
    for name in ["../secret", "ledger-client;rm", "ledger_*", ""] {
        assert!(state.query(&d, name).is_err());
        assert_eq!(state, old);
    }
}
#[test]
fn definitions_and_restored_cache_references_reject_foreign_duplicate_and_unbounded_values() {
    let (d, state) = fixture();
    let mut bad = d.clone();
    bad.targets.push(bad.targets[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.targets[0].published = Some("absent".into());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    let repeated = bad.targets[0].snapshots[0].rows[0].clone();
    bad.targets[0].snapshots[0].rows.push(repeated);
    assert!(bad.validate().is_err());
    let mut bad = state.clone();
    bad.cache[0].snapshot = "not-supplied".into();
    assert!(bad.validate(&d).is_err());
    let mut bad = state.clone();
    bad.cache.push(bad.cache[0].clone());
    assert!(bad.validate(&d).is_err());
    let mut bad = state.clone();
    bad.cache[0].key.component = "absent".into();
    assert!(bad.validate(&d).is_err());
    let mut bad = state.clone();
    bad.saved = "x".repeat(s::MAX_INPUT + 1);
    assert!(bad.validate(&d).is_err());
    let mut bad = d.clone();
    bad.targets[0].snapshots[0].rows = vec![row("ledger-client", "1.0-1", A::All); c::MAX_ROWS + 1];
    assert_eq!(bad.validate().unwrap_err().code, c::Code::Limit);
    let mut bad = d.clone();
    bad.targets = vec![target("main"); c::MAX_TARGETS + 1];
    assert_eq!(bad.validate().unwrap_err().code, c::Code::Limit);
}

#[test]
fn json_boundaries_restore_only_supplied_cache_and_limit_before_big_fields_or_lists() {
    let (d, state) = fixture();
    let raw = serde_json::to_string(&d).unwrap();
    assert_eq!(c::Definition::from_json(&raw).unwrap(), d);
    let raw = serde_json::to_string(&state).unwrap();
    assert_eq!(c::State::from_json(&d, &raw).unwrap(), state);
    assert_eq!(
        c::Definition::from_json(&" ".repeat(c::MAX_DEFINITION_JSON + 1))
            .unwrap_err()
            .code,
        c::Code::Limit
    );
    assert_eq!(
        c::State::from_json(&d, &" ".repeat(c::MAX_STATE_JSON + 1))
            .unwrap_err()
            .code,
        c::Code::Limit
    );
    for value in [
        serde_json::json!({"saved":text("main"),"cache":[],"execute":"arbitrary"}),
        serde_json::json!({"saved":"x".repeat(s::MAX_INPUT+1),"cache":[]}),
        serde_json::json!({"saved":text("main"),"cache":[{"key":key("absent"),"snapshot":"old"}]}),
        serde_json::json!({"saved":text("main"),"cache":vec![state.cache[0].clone();c::MAX_TARGETS+1]}),
    ] {
        assert!(c::State::from_json(&d, &value.to_string()).is_err());
    }
    let mut v = serde_json::to_value(&d).unwrap();
    v["targets"][0]["key"]["uri"] = serde_json::json!("x".repeat(161));
    assert!(c::Definition::from_json(&v.to_string()).is_err());
    let mut v = serde_json::to_value(&d).unwrap();
    v["targets"][0]["snapshots"][0]["rows"] =
        serde_json::json!(vec![row("ledger-client", "1.0-1", A::All); c::MAX_ROWS + 1]);
    assert!(c::Definition::from_json(&v.to_string()).is_err());
    let mut v = serde_json::to_value(&d).unwrap();
    v["targets"][0]["published"] = serde_json::json!("x".repeat(65));
    assert!(c::Definition::from_json(&v.to_string()).is_err());
}
#[test]
fn source_diagnostics_keep_the_specific_field_mistake_and_its_line_for_retry() {
    let (d, mut state) = fixture();
    let old = state.clone();
    let error = state
        .save(&d, &(text("main") + "Enabled: true\n"))
        .unwrap_err();
    assert_eq!(error.code, c::Code::Selection);
    let detail = error.selection.as_ref().unwrap();
    assert_eq!(detail.line, 5);
    assert_eq!(detail.code, s::Code::Unsupported);
    assert!(error.to_string().contains("Enabled supports yes or no"));
    assert!(error.to_string().contains("Line 5"));
    assert_eq!(old, state);
}
