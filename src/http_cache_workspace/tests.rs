use super::*;
fn origin(time: u64, tag: &str, body: &str) -> model::OriginReply {
    model::OriginReply {
        response_time: time,
        date: time,
        age: 0,
        etag: tag.into(),
        body: body.into(),
    }
}
fn request(path: &str, time: u64, tag: &str, body: &str) -> model::Step {
    model::Step {
        path: path.into(),
        request_time: time,
        request_control: String::new(),
        origin: Some(origin(time, tag, body)),
    }
}
fn expected(delivery: Delivery, tag: &str, body: &str, storage: Storage) -> Expected {
    Expected {
        delivery: vec![delivery],
        body: Body::Version {
            etag: tag.into(),
            body: body.into(),
        },
        storage,
    }
}
fn task(name: &str) -> Task {
    let guide = format!("/{name}-guide");
    let account = format!("/{name}-account");
    Task {
        version: 1,
        id: format!("fixture-{name}"),
        resources: vec![
            Resource {
                path: guide.clone(),
                label: "Public guide".into(),
                initial: "no-store".into(),
                reference: "public, max-age=60".into(),
                keep_max_age: None,
            },
            Resource {
                path: account.clone(),
                label: "Per-user response".into(),
                initial: "public, max-age=600".into(),
                reference: "private, max-age=60".into(),
                keep_max_age: None,
            },
        ],
        traces: vec![
            Trace {
                id: "public-reuse".into(),
                label: "Public repeat".into(),
                steps: vec![
                    request(&guide, 0, "g1", "guide"),
                    request(&guide, 4, "g1", "guide"),
                ],
                expected: vec![
                    expected(Delivery::Downloaded, "g1", "guide", Storage::Required),
                    expected(Delivery::Cached, "g1", "guide", Storage::Required),
                ],
            },
            Trace {
                id: "personal-separation".into(),
                label: "Different personal bodies".into(),
                steps: vec![
                    request(&account, 0, "a", "Alice"),
                    request(&account, 1, "b", "Bob"),
                ],
                expected: vec![
                    expected(Delivery::Downloaded, "a", "Alice", Storage::Forbidden),
                    expected(Delivery::Downloaded, "b", "Bob", Storage::Forbidden),
                ],
            },
        ],
    }
}
fn definition() -> Definition {
    Definition {
        initial: task("assignment"),
        example: task("workshop"),
    }
}
#[test]
fn references_run_real_traces_and_unchecked_edits_cannot_earn_credit() {
    let t = task("assignment");
    t.validate().unwrap();
    let mut d = Draft::new(&t).unwrap();
    assert!(!t.checks(&d).iter().all(|r| r.0));
    assert!(!t.report(&d).unwrap().passed());
    d.edit(&t, t.reference_headers()).unwrap();
    assert!(!d.checked());
    assert!(!t.checks(&d).iter().all(|r| r.0));
    assert!(d.check(&t).unwrap().passed());
    assert!(t.checks(&d).iter().all(|r| r.0));
}
#[test]
fn alternate_directives_and_durations_pass_by_behavior_not_exact_text() {
    let t = task("assignment");
    let mut d = Draft::new(&t).unwrap();
    let mut h = t.reference_headers();
    h[0].value = "MAX-AGE=\"20\", public".into();
    h[1].value = "no-store".into();
    assert_ne!(h, t.reference_headers());
    d.edit(&t, h).unwrap();
    assert!(d.check(&t).unwrap().passed());
}
#[test]
fn partial_storage_and_validation_mistakes_leave_correct_cases_correct() {
    let t = task("assignment");
    let mut d = Draft::new(&t).unwrap();
    let mut h = t.reference_headers();
    h[1].value = "no-cache, max-age=60".into();
    d.edit(&t, h).unwrap();
    let report = d.check(&t).unwrap();
    assert!(!report.passed());
    assert!(report.traces[0].steps.iter().all(|s| s.passed));
    assert!(report.traces[1].steps.iter().all(|s| !s.passed));
    assert!(
        report.traces[1].steps[0]
            .feedback
            .contains("preventing storage")
    );
    let mut h = t.reference_headers();
    h[0].value = "no-store".into();
    d.edit(&t, h).unwrap();
    let report = d.check(&t).unwrap();
    assert!(!report.passed());
    assert!(report.traces[1].steps.iter().all(|s| s.passed));
}
#[test]
fn examples_and_selected_reset_are_isolated_and_real_replay_is_deterministic() {
    let def = definition();
    let mut s = Session::new(&def).unwrap();
    let assignment = s.assignment.clone();
    s.practice
        .edit(&def.example, def.example.reference_headers())
        .unwrap();
    assert!(s.practice.check(&def.example).unwrap().passed());
    assert_eq!(s.assignment, assignment);
    assert!(!def.checks(&s).iter().all(|r| r.0));
    s.assignment
        .edit(&def.initial, def.initial.reference_headers())
        .unwrap();
    let a = s.assignment.check(&def.initial).unwrap();
    let b = def.initial.report(&s.assignment).unwrap();
    assert_eq!(a, b);
    let assignment = s.assignment.clone();
    s.reset(&def, true).unwrap();
    assert_eq!(s.assignment, assignment);
    assert!(!s.practice.checked());
    let practice = s.practice.clone();
    s.reset(&def, false).unwrap();
    assert_eq!(s.practice, practice);
    assert!(!s.assignment.checked());
}
#[test]
fn changed_values_or_direct_mutation_invalidate_success_and_invalid_edits_are_atomic() {
    let t = task("assignment");
    let mut d = Draft::new(&t).unwrap();
    d.edit(&t, t.reference_headers()).unwrap();
    assert!(d.check(&t).unwrap().passed());
    let selected = d.selected_trace.clone();
    d.selected_trace = t.traces[1].id.clone();
    assert!(d.checked());
    d.selected_trace = selected;
    d.headers[0].value = "no-store".into();
    assert!(!d.checked());
    assert!(!t.checks(&d).iter().all(|r| r.0));
    d.checked_stamp = Some(Stamp {
        headers: d.headers.clone(),
    });
    assert!(d.checked());
    assert!(
        !t.checks(&d).iter().all(|r| r.0),
        "A forged checked stamp cannot make broken behavior pass."
    );
    let old = d.clone();
    let mut bad = d.headers.clone();
    bad[0].value = "x".repeat(1025);
    assert!(d.edit(&t, bad).is_err());
    assert_eq!(d, old);
    let mut bad = d.headers.clone();
    bad.reverse();
    assert!(d.edit(&t, bad).is_err());
    assert_eq!(d, old);
}
#[test]
fn bounded_saved_drafts_bind_full_source_facts_and_preserve_unfinished_text() {
    let def = definition();
    let mut s = Session::new(&def).unwrap();
    let mut h = s.assignment.headers.clone();
    h[0].value = "max-age=not-yet".into();
    s.assignment.edit(&def.initial, h).unwrap();
    let bytes = serde_json::to_vec(&s).unwrap();
    let saved: Session = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(s, saved);
    def.bound(&saved).unwrap();
    let mut wrong = def.clone();
    wrong.initial.traces[0].steps[1].request_time = 5;
    assert!(wrong.bound(&saved).is_err());
    let mut wrong = saved.clone();
    wrong.source.initial.resources[0].reference = "max-age=999".into();
    assert!(def.bound(&wrong).is_err());
    let mut bad = serde_json::to_value(s).unwrap();
    bad["assignment"]["headers"][0]["value"] = serde_json::json!("x".repeat(4097));
    assert!(serde_json::from_value::<Session>(bad).is_err());
}
#[test]
fn invalid_expected_counts_unreachable_proofs_and_already_solved_starters_are_rejected() {
    let mut t = task("assignment");
    t.traces[0].expected.pop();
    assert!(t.validate().is_err());
    assert!(t.report_headers(&t.reference_headers()).is_err());
    let mut t = task("assignment");
    t.traces[0].expected[1].delivery = vec![Delivery::Unavailable];
    assert!(t.validate().is_err());
    let mut t = task("assignment");
    for r in &mut t.resources {
        r.initial = r.reference.clone();
    }
    assert!(t.validate().is_err());
    let mut t = task("assignment");
    t.traces.resize(9, t.traces[0].clone());
    assert!(t.report_headers(&t.reference_headers()).is_err());
    let mut d = definition();
    d.example.traces = d.initial.traces.clone();
    assert!(d.validate().is_err());
}
#[test]
fn syntax_and_unsupported_errors_are_educational_and_retryable() {
    let t = task("assignment");
    let mut d = Draft::new(&t).unwrap();
    let mut h = t.reference_headers();
    h[0].value = "max-age=wrong".into();
    d.edit(&t, h).unwrap();
    let report = d.check(&t).unwrap();
    assert!(!report.passed());
    assert!(report.configuration_errors[0].contains("Syntax"));
    let mut h = t.reference_headers();
    h[0].value = "immutable".into();
    d.edit(&t, h).unwrap();
    let report = d.check(&t).unwrap();
    assert!(!report.passed());
    assert!(report.configuration_errors[0].contains("Unsupported"));
    d.edit(&t, t.reference_headers()).unwrap();
    assert!(d.check(&t).unwrap().passed());
}

fn scenario() -> crate::pack::Scenario {
    let d = definition();
    let solution = [
        Action::Edit {
            headers: d.initial.reference_headers(),
        },
        Action::Check {},
    ]
    .into_iter()
    .map(|a| format!("{PREFIX}{}", serde_json::to_string(&a).unwrap()))
    .collect::<Vec<_>>();
    serde_json::from_value(serde_json::json!({
        "id":d.initial.id,"version":1,"track":"LPIC-2","unit":"fixture-cache",
        "topic":"Cache fixture","title":"Source-bound cache fixture",
        "summary":"Developer fixture, not an authored catalog lesson.",
        "objective":"Reuse the public response and keep per-user responses out of shared storage.",
        "difficulty":"Intermediate","minutes":10,
        "steps":["Inspect supplied request facts.","Edit response headers and check actual outcomes."],
        "hints":["Storage permission and freshness are separate decisions."],
        "sources":["https://www.rfc-editor.org/rfc/rfc9111.html"],
        "simplifications":"Fixed virtual origin and supplied GET facts; no network, host command or real browser.",
        "initial":{"kind":"HttpCache","simulation":Session::new(&d).unwrap()},
        "goal":{"kind":"HttpCache","definition":d},"solution":solution
    })).unwrap()
}

#[test]
fn pack_reachability_uses_actual_behavior_and_refuses_prechecked_or_wrong_family_sources() {
    let s = scenario();
    source(&s).unwrap();
    let data = serde_json::to_string(&crate::pack::Pack {
        schema_version: 1,
        scenarios: vec![s.clone()],
    })
    .unwrap();
    crate::pack::load(&data).unwrap();
    let mut state = s.initial.clone();
    assert!(!crate::engine::solved(&state, &s.goal));
    for action in &s.solution {
        crate::engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
    }
    assert!(crate::engine::solved(&state, &s.goal));
    let crate::engine::State::HttpCache { simulation } = &mut state else {
        panic!()
    };
    simulation.assignment.headers[0].value = "no-store".into();
    assert!(!crate::engine::solved(&state, &s.goal));
    let mut bad = s.clone();
    let crate::engine::State::HttpCache { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.assignment.checked_stamp = Some(Stamp {
        headers: simulation.assignment.headers.clone(),
    });
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    bad.goal = crate::pack::bundled().unwrap()[0].goal.clone();
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    bad.initial = crate::pack::bundled().unwrap()[0].initial.clone();
    assert!(source(&bad).is_err());
}

#[test]
fn native_only_actions_reject_host_commands_unknown_fields_and_changes_atomically() {
    let s = scenario();
    let mut state = s.initial.clone();
    let before = state.clone();
    for command in [
        "curl https://example.test",
        "sh anything",
        "@http-cache {\"action\":\"exec\",\"command\":\"anything\"}",
        "@http-cache {\"action\":\"check\",\"extra\":true}",
        "@http-cache {}\n",
    ] {
        assert!(crate::engine::execute_for_goal(&mut state, &s.goal, command).is_err());
        assert_eq!(state, before);
    }
    let mut headers = source(&s).unwrap().initial.reference_headers();
    headers[0].path = "/different".into();
    let command = format!(
        "{PREFIX}{}",
        serde_json::to_string(&Action::Edit { headers }).unwrap()
    );
    assert!(crate::engine::execute_for_goal(&mut state, &s.goal, &command).is_err());
    assert_eq!(state, before);
}

#[test]
fn safe_disk_roundtrip_preserves_independent_drafts_favorites_and_rejects_changed_source_without_overwrite()
 {
    let s = scenario();
    let d = source(&s).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    crate::workspace::prepare(&mut a);
    for action in &s.solution {
        crate::engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    let crate::engine::State::HttpCache { simulation } = &mut a.state else {
        panic!()
    };
    let mut h = simulation.practice.headers.clone();
    h[0].value = "max-age=unfinished".into();
    simulation.practice.edit(&d.example, h).unwrap();
    let mut p = crate::progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    assert!(
        crate::progress::save(&path, &p)
            .unwrap_err()
            .contains("authored catalog")
    );
    crate::progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let loaded = crate::progress::load(&path, std::slice::from_ref(&s)).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&loaded).unwrap()
    );
    assert!(crate::progress::current_solved(&loaded.attempts[&s.id], &s));
    assert!(loaded.favorites.contains(&s.id));
    let mut bad = loaded;
    bad.attempts.get_mut(&s.id).unwrap().scenario_version += 1;
    assert!(crate::progress::save_with_catalog(&path, &bad, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn completed_example_cannot_satisfy_assignment_goal_and_inactive_metadata_is_rejected() {
    let s = scenario();
    let d = source(&s).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    let crate::engine::State::HttpCache { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .practice
        .edit(&d.example, d.example.reference_headers())
        .unwrap();
    assert!(simulation.practice.check(&d.example).unwrap().passed());
    assert!(!crate::progress::current_solved(&a, &s));
    a.scenario_version += 1;
    assert!(context(&a, &s).is_err());
    a.scenario_version -= 1;
    a.ipv4 = Some(crate::ipv4::Lesson::from_legacy(
        "192.0.2.7".parse().unwrap(),
        24,
        0,
    ));
    assert!(a.ipv4.is_some());
    assert!(context(&a, &s).is_err());
}

#[test]
fn prospective_units_have_reachable_distinct_examples_and_accept_equivalent_headers() {
    let defs: Vec<Definition> = serde_json::from_str(include_str!(
        "../../tests/fixtures/http-cache-definitions-preview-v1.json"
    ))
    .unwrap();
    assert_eq!(defs.len(), 2);
    for d in &defs {
        d.validate().unwrap();
        let s = Session::new(d).unwrap();
        assert!(!d.checks(&s).iter().all(|r| r.0));
        assert_ne!(
            d.initial
                .resources
                .iter()
                .map(|r| &r.path)
                .collect::<Vec<_>>(),
            d.example
                .resources
                .iter()
                .map(|r| &r.path)
                .collect::<Vec<_>>()
        );
        assert!(
            d.initial
                .report_headers(&d.initial.reference_headers())
                .unwrap()
                .passed()
        );
        assert!(
            d.example
                .report_headers(&d.example.reference_headers())
                .unwrap()
                .passed()
        );
    }
    let privacy = &defs[0].initial;
    let mut h = privacy.reference_headers();
    h[0].value = "max-age=30".into();
    h[1].value = "no-store".into();
    assert!(privacy.report_headers(&h).unwrap().passed());
    for forbidden in ["public, max-age=600", "no-cache, max-age=60"] {
        h[1].value = forbidden.into();
        let r = privacy.report_headers(&h).unwrap();
        assert!(!r.passed());
        assert!(r.traces[0].steps.iter().all(|s| s.passed));
    }
    let fresh = &defs[1].initial;
    let mut h = fresh.reference_headers();
    h[0].value = "s-maxage=30, max-age=600".into();
    assert!(fresh.report_headers(&h).unwrap().passed());
    for wrong in [
        "max-age=600",
        "max-age=600, s-maxage=10",
        "max-age=600, s-maxage=60",
        "max-age=600, no-store",
        "max-age=600, no-cache",
    ] {
        h[0].value = wrong.into();
        assert!(!fresh.report_headers(&h).unwrap().passed(), "{wrong}");
    }
    h[0].value = "max-age=100, s-maxage=40".into();
    let r = fresh.report_headers(&h).unwrap();
    assert!(!r.passed());
    assert!(
        r.configuration_errors[0].contains("must retain")
            && r.configuration_errors[0].contains("max-age=600")
    );
}
