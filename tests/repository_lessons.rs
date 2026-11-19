use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    repository_workspace as w,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/repository-lessons-v1.json")).unwrap()
}
fn parts(s: &pack::Scenario) -> (&w::Definition, &w::Session) {
    let (Goal::Repositories { definition }, State::Repositories { simulation }) =
        (&s.goal, &s.initial)
    else {
        panic!()
    };
    (definition, simulation)
}
fn done(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for cmd in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, cmd).unwrap();
    }
    state
}
#[test]
fn repository_four_true_solutions_and_separate_examples() {
    for s in labs() {
        let (d, init) = parts(&s);
        assert!(!engine::solved(&s.initial, &s.goal));
        let state = done(&s);
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        let mut x = init.clone();
        for event in &d.example_actions {
            x.run(d, event, true).unwrap();
        }
        assert!(d.checks(&x, true).iter().all(|r| r.0));
        assert_eq!(x.assignment, init.assignment);
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
    }
}
#[test]
fn repository_queries_cannot_replace_enable_refresh_or_matching_provenance() {
    for s in labs()
        .into_iter()
        .filter(|s| !s.id.ends_with("offline-evidence"))
    {
        let (d, init) = parts(&s);
        let mut x = init.clone();
        for p in ["parcel", "companion"] {
            x.run(d, &w::Event::Query { package: p.into() }, false)
                .unwrap();
        }
        assert!(!d.checks(&x, false).iter().all(|r| r.0), "{}", s.id);
    }
}
#[test]
fn repository_query_evidence_becomes_stale_after_changes_and_dirty_drafts_invalidate_success() {
    for s in labs() {
        let (d, _) = parts(&s);
        let State::Repositories { simulation } = done(&s) else {
            panic!()
        };
        let mut x = *simulation;
        assert!(d.checks(&x, false).iter().all(|r| r.0));
        x.assignment.dirty = true;
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
        x.assignment.dirty = false;
        let src = &d.initial_source;
        let alias = x.assignment.alias.clone();
        let i = x.assignment.world.inspect(src, &alias).unwrap();
        let ev = w::Event::Edit {
            edit: w::Edit::Settings {
                alias: alias.clone(),
                enabled: i.configuration.enabled,
                uri: i.configuration.uri.clone(),
            },
        };
        x.run(d, &ev, false).unwrap();
        assert!(
            d.checks(&x, false)
                .iter()
                .any(|r| !r.0 && r.1.contains("Earlier results"))
        );
        for p in ["parcel", "companion"] {
            x.run(d, &w::Event::Query { package: p.into() }, false)
                .unwrap();
        }
        assert!(d.checks(&x, false).iter().all(|r| r.0));
    }
}
#[test]
fn repository_matching_current_cache_can_be_reused_or_explicitly_refreshed() {
    let s = labs().remove(1);
    let (d, init) = parts(&s);
    for refresh in [false, true] {
        let mut x = init.clone();
        x.run(
            d,
            &w::Event::Edit {
                edit: w::Edit::Settings {
                    alias: "tools".into(),
                    enabled: true,
                    uri: d.initial_source.endpoints[0].uri.clone(),
                },
            },
            false,
        )
        .unwrap();
        if refresh {
            x.run(
                d,
                &w::Event::Edit {
                    edit: w::Edit::Refresh {
                        alias: "tools".into(),
                    },
                },
                false,
            )
            .unwrap();
        }
        for p in ["parcel", "companion"] {
            x.run(d, &w::Event::Query { package: p.into() }, false)
                .unwrap();
        }
        assert!(d.checks(&x, false).iter().all(|r| r.0));
    }
}
#[test]
fn repository_offline_refresh_preserves_old_cache_and_actual_query_snapshot() {
    let s = labs().remove(2);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    x.run(
        d,
        &w::Event::Query {
            package: "parcel".into(),
        },
        false,
    )
    .unwrap();
    let before = x.clone();
    assert!(
        x.run(
            d,
            &w::Event::Edit {
                edit: w::Edit::Refresh {
                    alias: "tools".into()
                }
            },
            false
        )
        .unwrap_err()
        .contains("unavailable")
    );
    assert_eq!(x, before);
    let (p, w, fresh) = d.observed(&x, false).unwrap().unwrap();
    assert_eq!(p, "parcel");
    assert!(fresh);
    let q = w.query_exact(&d.initial_source, &p).unwrap();
    assert_eq!(q.rows.len(), 1);
    assert_eq!(q.rows[0].snapshot.as_str(), "old");
    assert_eq!(q.rows[0].archive.evr.as_str(), "1.0-1");
}
#[test]
fn repository_held_candidates_and_partial_evidence_cannot_false_pass() {
    let s = labs().remove(3);
    let (d, _) = parts(&s);
    let State::Repositories { simulation } = done(&s) else {
        panic!()
    };
    let mut x = *simulation;
    let held = x
        .assignment
        .world
        .inspect(&d.initial_source, "held")
        .unwrap();
    let uri = held.configuration.uri.clone();
    x.run(
        d,
        &w::Event::Edit {
            edit: w::Edit::Settings {
                alias: "held".into(),
                enabled: true,
                uri,
            },
        },
        false,
    )
    .unwrap();
    for p in ["parcel", "companion"] {
        x.run(d, &w::Event::Query { package: p.into() }, false)
            .unwrap();
    }
    let rows = d.checks(&x, false);
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("Retain held")));
    assert!(rows.iter().filter(|r| !r.0).count() >= 3);
    let mut x = parts(&s).1.clone();
    for cmd in s.solution.iter().take(s.solution.len() - 1) {
        x.execute_reference(d, cmd).unwrap();
    }
    assert!(!d.checks(&x, false).iter().all(|r| r.0));
}
#[test]
fn repository_forged_cache_cannot_bypass_successful_operation_history() {
    let labs = labs();
    let s = &labs[2];
    let (d, init) = parts(s);
    let mut x = init.clone();
    let mut value = serde_json::to_value(&x.assignment.world).unwrap();
    value["caches"][0]["snapshot"] = serde_json::json!("new");
    x.assignment.world = serde_json::from_value(value).unwrap();
    assert!(d.bound(&x).is_err());
    assert!(!d.checks(&x, false).iter().all(|r| r.0));
    let mut a = Attempt::new(s, 1);
    a.state = State::Repositories {
        simulation: Box::new(x),
    };
    assert!(!progress::current_solved(&a, s));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&raw, &labs).is_err());
}
#[test]
fn repository_resume_preserves_separate_drafts_history_and_old_actual_results() {
    let labs = labs();
    let s = &labs[0];
    let (d, init) = parts(s);
    let mut x = init.clone();
    x.run(
        d,
        &w::Event::Query {
            package: "parcel".into(),
        },
        false,
    )
    .unwrap();
    x.assignment.uri = "https://unfinished.invalid/".into();
    x.assignment.dirty = true;
    x.practice.package = "sample-companion".into();
    let mut a = Attempt::new(s, 1);
    a.state = State::Repositories {
        simulation: Box::new(x),
    };
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::save(&path, &p).is_err());
    assert!(!progress::current_solved(&restored.attempts[&s.id], s));
}

#[test]
fn repository_current_catalog_preserves_every_221_lab_and_unit() {
    let current = pack::accepted225().unwrap();
    let old = pack::accepted221().unwrap();
    assert_eq!(current.len(), 225);
    assert_eq!(old.len(), 221);
    assert_eq!(
        serde_json::to_value(&current[..221]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let prior = dlms_learning_lab::course::accepted221(&old).unwrap();
    let units = dlms_learning_lab::course::accepted225(&current).unwrap();
    assert_eq!(units.len(), 60);
    assert_eq!(prior.len(), 59);
    assert_eq!(
        serde_json::to_value(&units[..59]).unwrap(),
        serde_json::to_value(&prior).unwrap()
    );
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["rpm-package-investigation"]
    );
    assert_eq!(
        units.last().unwrap().labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
}
#[test]
fn repository_unsupported_queries_and_host_commands_never_change_virtual_state() {
    let s = labs().remove(0);
    let (d, init) = parts(&s);
    for name in ["", "parcel*", "parcel | cat", "$(id)", "../../host"] {
        let mut x = init.clone();
        assert!(
            x.run(
                d,
                &w::Event::Query {
                    package: name.into()
                },
                false
            )
            .is_err()
        );
        assert_eq!(x, *init);
    }
    for text in [
        "zypper refresh",
        "rm /etc/passwd",
        "repository-action {\"kind\":\"Query\",\"package\":\"parcel\",\"extra\":true}",
    ] {
        let mut state = s.initial.clone();
        assert!(engine::execute_for_goal(&mut state, &s.goal, text).is_err());
        assert_eq!(state, s.initial);
    }
    let mut x = init.clone();
    x.run(
        d,
        &w::Event::Edit {
            edit: w::Edit::Settings {
                alias: "tools".into(),
                enabled: true,
                uri: "file:///../../personal".into(),
            },
        },
        false,
    )
    .unwrap();
    let before = x.clone();
    assert!(
        x.run(
            d,
            &w::Event::Edit {
                edit: w::Edit::Refresh {
                    alias: "tools".into()
                }
            },
            false
        )
        .unwrap_err()
        .contains("no supplied")
    );
    assert_eq!(x, before);
}
#[test]
fn repository_history_bound_and_failed_save_preserve_prior_work_exactly() {
    let s = labs().remove(0);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    for _ in 0..32 {
        x.run(
            d,
            &w::Event::Query {
                package: "parcel".into(),
            },
            false,
        )
        .unwrap();
    }
    let before = x.clone();
    assert!(
        x.run(
            d,
            &w::Event::Query {
                package: "companion".into()
            },
            false
        )
        .unwrap_err()
        .contains("32")
    );
    assert_eq!(x, before);
    let labs = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), Attempt::new(&s, 1));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    p.attempts.get_mut(&s.id).unwrap().workspace =
        Some(dlms_learning_lab::workspace::WorkspaceData::new());
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = "foreign work".into();
    assert!(progress::save_with_catalog(&path, &p, &labs).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn repository_corrupt_editor_marker_and_invalid_pack_are_rejected() {
    let s = labs().remove(0);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    x.assignment.uri = "file:///virtual-repositories/unprovided".into();
    x.assignment.dirty = false;
    assert!(d.bound(&x).unwrap_err().contains("unapplied-draft"));
    let raw = include_str!("../scenarios/repository-lessons-v1.json");
    for n in 0..4 {
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let s = &mut v["scenarios"][0];
        match n {
            0 => s["goal"]["definition"]["requirements"] = serde_json::json!([]),
            1 => {
                s["initial"]["simulation"]["assignment"]["history"] =
                    serde_json::json!([{ "kind":"Query","package":"*"}])
            }
            2 => s["goal"]["definition"]["example_actions"][0]["extra"] = serde_json::json!(true),
            _ => {
                s["goal"]["definition"]["initial_source"]["endpoints"][0]["delivery"] =
                    serde_json::json!("unavailable")
            }
        };
        assert!(pack::load(&v.to_string()).is_err(), "invalid pack {n}");
    }
}

#[test]
fn repository_queries_accept_reverse_order_and_feedback_identifies_wrong_versions() {
    for s in labs() {
        let (d, init) = parts(&s);
        let mut x = init.clone();
        for cmd in &s.solution {
            let e: w::Event =
                serde_json::from_str(cmd.strip_prefix("repository-action ").unwrap()).unwrap();
            if matches!(e, w::Event::Edit { .. }) {
                x.run(d, &e, false).unwrap();
            }
        }
        for package in ["companion", "parcel"] {
            x.run(
                d,
                &w::Event::Query {
                    package: package.into(),
                },
                false,
            )
            .unwrap();
        }
        assert!(d.checks(&x, false).iter().all(|r| r.0), "{}", s.id);
    }
    let s = labs().remove(3);
    let (d, init) = parts(&s);
    let mut x = init.clone();
    for package in ["parcel", "companion"] {
        x.run(
            d,
            &w::Event::Query {
                package: package.into(),
            },
            false,
        )
        .unwrap();
    }
    let rows = d.checks(&x, false);
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("Unexpected candidates") && r.1.contains("1.0-1"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.0 && r.1.contains("configured source archive"))
    );
}
