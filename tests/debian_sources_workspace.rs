use dlms_learning_lab::{
    debian_packages::{Architecture, Version},
    debian_source_cache as c, debian_sources as sources, debian_sources_workspace as w,
    script_teaching::View,
};
fn key(prefix: &str, component: &str) -> sources::Key {
    sources::Key {
        uri: format!("https://{prefix}.workshop.test/debian/"),
        suite: "studio".into(),
        component: component.into(),
    }
}
fn text(prefix: &str, components: &str) -> String {
    format!(
        "Types: deb\nURIs: https://{prefix}.workshop.test/debian\nSuites: studio\nComponents: {components}\n"
    )
}
fn row(prefix: &str, v: &str) -> c::Row {
    c::Row {
        name: format!("{prefix}-tool"),
        version: Version::parse(v).unwrap(),
        architecture: Architecture::All,
    }
}
fn task(prefix: &str) -> w::Task {
    let targets = ["main", "tools"]
        .into_iter()
        .map(|component| c::Target {
            key: key(prefix, component),
            snapshots: vec![
                c::Snapshot {
                    id: "old_index".into(),
                    rows: vec![row(prefix, "1.0-1")],
                },
                c::Snapshot {
                    id: "new-index".into(),
                    rows: vec![row(prefix, "2.0-1")],
                },
            ],
            published: Some("new-index".into()),
        })
        .collect();
    w::Task {
        instructions: format!("Make the {prefix} tools available while retaining main."),
        id: format!("{prefix}-sources"),
        model: c::Definition {
            native: Architecture::Amd64,
            targets,
        },
        starter: c::State {
            saved: text(prefix, "main"),
            cache: vec![c::Cached {
                key: key(prefix, "main"),
                snapshot: "old_index".into(),
            }],
        },
        requirements: vec![
            w::Requirement::Selected {
                targets: vec![key(prefix, "main"), key(prefix, "tools")],
            },
            w::Requirement::Cached {
                target: key(prefix, "main"),
                snapshot: "new-index".into(),
            },
            w::Requirement::Available {
                target: key(prefix, "tools"),
                package: format!("{prefix}-tool"),
                version: Version::parse("2.0-1").unwrap(),
            },
        ],
    }
}
fn definition() -> w::Definition {
    w::Definition {
        initial: task("records"),
        example: task("gallery"),
        example_worked: w::Worked {
            sources: text("gallery", "main tools"),
            refresh: true,
        },
        assignment_reference: w::Worked {
            sources: text("records", "main tools"),
            refresh: true,
        },
    }
}
fn solve(t: &w::Task, s: &mut w::Scope, text: &str) {
    s.edit(text).unwrap();
    s.save(t).unwrap();
    s.refresh(t).unwrap();
    assert!(s.check(t).iter().all(|r| r.0));
}
#[test]
fn editor_save_refresh_and_query_keep_distinct_observable_effects() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    assert!(!d.solved(&s));
    let before = s.assignment.state.cache.clone();
    s.assignment.edit(&d.assignment_reference.sources).unwrap();
    assert_eq!(s.assignment.state.saved, text("records", "main"));
    assert!(
        s.assignment
            .refresh(&d.initial)
            .unwrap_err()
            .contains("Save")
    );
    assert_eq!(s.assignment.state.cache, before);
    s.assignment.save(&d.initial).unwrap();
    assert_eq!(s.assignment.state.cache, before);
    assert!(!s.assignment.check(&d.initial).iter().all(|r| r.0));
    s.assignment.set_query("records-tool").unwrap();
    let rows = s.assignment.lookup(&d.initial).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].row.version.as_str(), "1.0-1");
    assert!(!d.solved(&s));
    s.assignment.refresh(&d.initial).unwrap();
    assert!(s.assignment.observed.is_none());
    let rows = s.assignment.lookup(&d.initial).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.row.version.as_str() == "2.0-1"));
    s.assignment.check(&d.initial);
    assert!(d.solved(&s));
}
#[test]
fn equivalent_stanzas_and_order_pass_but_wrong_selection_or_unchecked_work_do_not() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let alternate = format!("{}\n{}", text("records", "tools"), text("records", "main"));
    solve(&d.initial, &mut s.assignment, &alternate);
    assert!(d.solved(&s));
    s.assignment.edit(&text("records", "tools")).unwrap();
    assert!(!s.assignment.checked);
    assert!(!d.solved(&s));
    s.assignment.save(&d.initial).unwrap();
    let rows = s.assignment.check(&d.initial);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("missing")));
    assert!(!d.solved(&s));
    solve(
        &d.initial,
        &mut s.assignment,
        &d.assignment_reference.sources,
    );
    s.assignment.state.cache[0].snapshot = "old_index".into();
    assert!(
        !d.solved(&s),
        "The current outcome is regraded, not trusted from the checked flag"
    );
}
#[test]
fn practice_worked_fill_is_inert_separate_and_cannot_earn_assignment_credit() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    let old = s.practice.state.clone();
    s.practice.edit(&d.example_worked.sources).unwrap();
    assert_eq!(s.practice.state, old);
    assert_eq!(s.assignment, assignment);
    assert!(!d.solved(&s));
    solve(&d.example, &mut s.practice, &d.example_worked.sources);
    assert_eq!(s.assignment, assignment);
    assert!(!d.solved(&s));
}
#[test]
fn invalid_draft_keeps_saved_cache_and_explains_the_specific_error_for_retry() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let old = s.assignment.state.clone();
    s.assignment
        .edit(&(text("records", "main tools") + "Enabled: true\n"))
        .unwrap();
    let e = s.assignment.save(&d.initial).unwrap_err();
    assert!(e.contains("Line 5") && e.contains("Enabled supports yes or no"));
    assert_eq!(s.assignment.state, old);
    assert!(!d.solved(&s));
    solve(
        &d.initial,
        &mut s.assignment,
        &d.assignment_reference.sources,
    );
    assert!(d.solved(&s));
}
#[test]
fn raw_drafts_observations_completed_work_and_selected_reset_roundtrip() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    solve(
        &d.initial,
        &mut s.assignment,
        &d.assignment_reference.sources,
    );
    s.assignment.set_query("records-tool").unwrap();
    s.assignment.lookup(&d.initial).unwrap();
    s.practice
        .edit("unfinished: no shell is executed\n")
        .unwrap();
    s.view = View::Assignment;
    let raw = serde_json::to_string(&s).unwrap();
    let mut restored = w::Session::from_json(&raw, &d).unwrap();
    assert_eq!(restored, s);
    assert!(d.solved(&restored));
    let practice = restored.practice.clone();
    restored.assignment = w::Scope::new(&d.initial);
    assert_eq!(restored.practice, practice);
    assert!(!d.solved(&restored));
    assert_eq!(restored.view, View::Assignment);
}
#[test]
fn admission_checks_real_reachability_origins_and_positive_requirements() {
    let d = definition();
    d.validate().unwrap();
    assert_eq!(
        w::Definition::from_json(&serde_json::to_string(&d).unwrap()).unwrap(),
        d
    );
    let mut bad = d.clone();
    bad.assignment_reference.refresh = false;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example_worked.sources = text("gallery", "main");
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial
        .requirements
        .retain(|r| !matches!(r, w::Requirement::Available { .. }));
    assert!(bad.validate().is_err());
    let mut s = w::Session::new(&d).unwrap();
    s.assignment = s.practice.clone();
    assert!(d.bound(&s).is_err());
    assert!(w::Session::from_json(&serde_json::to_string(&s).unwrap(), &d).is_err());
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.state.cache[0].snapshot = "invented".into();
    assert!(d.bound(&s).is_err());
    assert!(!d.solved(&s));
}
#[test]
fn data_only_reference_replays_real_actions_and_refuses_host_syntax_or_unknown_fields() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let old = s.clone();
    for input in [
        "apt-get update",
        "sh -c anything",
        "APT SOURCES\n{\"action\":\"Refresh\",\"script\":\"escape\"}",
    ] {
        assert!(s.execute_reference(&d, input).is_err());
        assert_eq!(s, old);
    }
    for v in [
        serde_json::json!({"action":"Save","sources":d.assignment_reference.sources}),
        serde_json::json!({"action":"Refresh"}),
        serde_json::json!({"action":"Query","name":"records-tool"}),
        serde_json::json!({"action":"Check"}),
    ] {
        s.execute_reference(&d, &format!("{}{}", w::PREFIX, v))
            .unwrap();
    }
    assert!(d.solved(&s));
}
#[test]
fn bounded_json_and_edit_operations_refuse_without_losing_drafts() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    assert!(
        s.assignment
            .edit(&"x".repeat(sources::MAX_INPUT + 1))
            .is_err()
    );
    assert!(s.assignment.set_query(&"x".repeat(65)).is_err());
    assert_eq!(s, before);
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION + 1), &d).is_err());
    assert!(w::Definition::from_json(&" ".repeat(w::MAX_DEFINITION + 1)).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["assignment"]["draft"] = serde_json::json!("x".repeat(sources::MAX_INPUT + 1));
    assert!(w::Session::from_json(&v.to_string(), &d).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    v["version"] = serde_json::json!(2);
    assert!(w::Session::from_json(&v.to_string(), &d).is_err());
}
