//! Private capability fixtures only: no catalog registration or authored-count credit.
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    syslog as core, syslog_workspace as work,
    workspace::WorkspaceData,
};
use std::{
    net::Ipv4Addr,
    sync::atomic::{AtomicBool, AtomicUsize},
};
fn fixture() -> Scenario {
    pack::load(include_str!("fixtures/syslog-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::Syslog { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &Attempt) -> &work::Session {
    let State::Syslog { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn session_mut(a: &mut Attempt) -> &mut work::Session {
    let State::Syslog { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn value<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).unwrap()
}
fn complete(a: &mut Attempt, s: &Scenario) {
    for step in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
    }
    assert!(progress::current_solved(a, s));
}
fn profile(s: &Scenario) -> Progress {
    Progress {
        schema_version: 1,
        clock: 1,
        attempts: [(s.id.clone(), Attempt::new(s, 1))].into(),
        favorites: [s.id.clone()].into(),
        ..Default::default()
    }
}
fn raw_action(s: &Scenario) -> work::Action {
    serde_json::from_str(&s.solution[0]).unwrap()
}

#[test]
fn private_pack_initial_and_reference_are_valid_without_changing_bundled_catalog() {
    let before = value(&pack::bundled().unwrap());
    let s = fixture();
    definition(&s).validate().unwrap();
    work::source(&s).unwrap();
    let mut a = Attempt::new(&s, 1);
    work::context(&a, &s).unwrap();
    assert!(!progress::current_solved(&a, &s));
    complete(&mut a, &s);
    assert_eq!(value(&pack::bundled().unwrap()), before);
    assert!(s.id.starts_with("dev-"));
}
#[test]
fn multiple_threshold_and_unused_selector_alternatives_pass_positive_and_negative_cases() {
    let s = fixture();
    let d = definition(&s);
    for n in 0..3 {
        let mut a = Attempt::new(&s, 1);
        let work::Action::ApplySource { device, mut policy } = raw_action(&s) else {
            panic!()
        };
        if n > 0 {
            policy.buffer_threshold = Some(core::Severity::Notice);
            policy.remote_threshold = Some(core::Severity::Debug);
        }
        session_mut(&mut a)
            .action(d, false, &work::Action::ApplySource { device, policy })
            .unwrap();
        if n == 2 {
            let mut p = session(&a)
                .current
                .collector_policy(ip("10.92.0.100"))
                .unwrap()
                .clone();
            p.selectors
                .insert(core::Facility::new(16).unwrap(), core::Severity::Debug);
            session_mut(&mut a)
                .action(
                    d,
                    false,
                    &work::Action::ApplyCollector {
                        collector: ip("10.92.0.100"),
                        policy: p,
                    },
                )
                .unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        assert!(d.checks(session(&a)).iter().all(|r| r.0));
    }
}
#[test]
fn broad_debug_all_disabled_wrong_facility_and_partial_solutions_fail_for_specific_reasons() {
    let s = fixture();
    let d = definition(&s);
    for case in 0..4 {
        let mut a = Attempt::new(&s, 1);
        let work::Action::ApplySource { device, mut policy } = raw_action(&s) else {
            panic!()
        };
        match case {
            0 => {
                policy.buffer_threshold = Some(core::Severity::Debug);
                policy.remote_threshold = Some(core::Severity::Debug);
            }
            1 => policy.process_enabled = false,
            2 => policy.export_facility = core::Facility::new(22).unwrap(),
            _ => policy.buffer_threshold = None,
        }
        session_mut(&mut a)
            .action(d, false, &work::Action::ApplySource { device, policy })
            .unwrap();
        let rows = d.checks(session(&a));
        assert!(rows.iter().any(|r| !r.0));
        assert!(!progress::current_solved(&a, &s));
        let why = rows
            .iter()
            .filter(|r| !r.0)
            .map(|r| r.1.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(!why.is_empty());
    }
}
#[test]
fn pending_drafts_clear_current_success_and_explain_unapplied_not_changed_policy() {
    let s = fixture();
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    complete(&mut a, &s);
    let applied = session(&a).current.clone();
    let mut draft = session(&a).editor.sources[&ip("10.92.0.10")].clone();
    draft.hosts[0] = "192.0.".into();
    session_mut(&mut a)
        .set_source_draft(d, false, ip("10.92.0.10"), draft.clone())
        .unwrap();
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(session(&a).current, applied);
    assert!(d.checks(session(&a))[0].1.contains("Unapplied changes"));
    let before = value(&a);
    let err = session_mut(&mut a).apply_selected(d, false).unwrap_err();
    assert!(err.contains("row 1") && err.contains("complete IPv4"));
    assert_eq!(value(&a), before);
    let bytes = serde_json::to_vec(session(&a)).unwrap();
    let restored = work::Session::decode(&bytes).unwrap();
    assert_eq!(restored.editor.sources[&ip("10.92.0.10")], draft);
    draft =
        work::SourceDraft::from_policy(applied.device_policy(ip("10.92.0.10")).unwrap()).unwrap();
    session_mut(&mut a)
        .set_source_draft(d, false, ip("10.92.0.10"), draft)
        .unwrap();
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn stale_history_and_checked_flags_cannot_keep_success_after_policy_damage() {
    let s = fixture();
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    complete(&mut a, &s);
    session_mut(&mut a)
        .action(
            d,
            false,
            &work::Action::Generate {
                device: ip("10.92.0.10"),
                event: core::EventId::new("port_fault").unwrap(),
            },
        )
        .unwrap();
    let receipt = session(&a)
        .current
        .collector_history(ip("10.92.0.100"))
        .unwrap()
        .to_vec();
    assert_eq!(receipt.len(), 1);
    let mut draft = session(&a).editor.sources[&ip("10.92.0.10")].clone();
    draft.remote = None;
    session_mut(&mut a)
        .set_source_draft(d, false, ip("10.92.0.10"), draft)
        .unwrap();
    session_mut(&mut a).apply_selected(d, false).unwrap();
    session_mut(&mut a).checked = true;
    session_mut(&mut a).example_checked = true;
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(
        session(&a)
            .current
            .collector_history(ip("10.92.0.100"))
            .unwrap(),
        receipt
    );
    assert!(d.checks(session(&a)).iter().any(|r| !r.0));
    complete(&mut a, &s);
    session_mut(&mut a).clear_selected(d, false).unwrap();
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn separate_fill_apply_generate_experiment_and_repair_never_change_assignment() {
    let s = fixture();
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    let assignment = session(&a).current.clone();
    let draft = session(&a).editor.clone();
    let example = session(&a).example.clone();
    let msg = session_mut(&mut a).fill_worked(d, 0).unwrap();
    assert!(msg.contains("Apply settings"));
    assert_eq!(session(&a).example, example);
    assert_eq!(session(&a).current, assignment);
    assert!(!d.practice_checks(session(&a)).iter().all(|r| r.0));
    session_mut(&mut a).apply_selected(d, true).unwrap();
    assert!(d.practice_checks(session(&a)).iter().all(|r| r.0));
    assert!(!progress::current_solved(&a, &s));
    session_mut(&mut a)
        .action(
            d,
            true,
            &work::Action::Generate {
                device: ip("10.93.0.20"),
                event: core::EventId::new("printer_fault").unwrap(),
            },
        )
        .unwrap();
    session_mut(&mut a)
        .action(d, true, &d.lesson.experiment)
        .unwrap();
    assert!(d.practice_checks(session(&a)).iter().any(|r| !r.0));
    session_mut(&mut a)
        .action(d, true, &d.lesson.repair)
        .unwrap();
    assert!(d.practice_checks(session(&a)).iter().all(|r| r.0));
    assert_eq!(session(&a).current, assignment);
    assert_eq!(session(&a).editor, draft);
    session_mut(&mut a).reset_example(d).unwrap();
    assert_eq!(session(&a).example, d.lesson.fixture);
    assert_eq!(session(&a).current, assignment);
    assert!(!session(&a).example_checked);
}
#[test]
fn selectors_hosts_and_component_switch_preserve_invalid_drafts_and_refuse_atomic_apply() {
    let s = fixture();
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    let mut draft = session(&a).editor.sources[&ip("10.92.0.10")].clone();
    draft.hosts.push(draft.hosts[0].clone());
    session_mut(&mut a)
        .set_source_draft(d, false, ip("10.92.0.10"), draft.clone())
        .unwrap();
    let before = value(&a);
    assert!(
        session_mut(&mut a)
            .apply_selected(d, false)
            .unwrap_err()
            .contains("repeats")
    );
    assert_eq!(value(&a), before);
    session_mut(&mut a)
        .select(d, false, work::Component::Collector(ip("10.92.0.100")))
        .unwrap();
    let old = session(&a).current.clone();
    let invalid = work::CollectorDraft {
        rows: vec![work::SelectorDraft {
            facility: None,
            threshold: core::Severity::Warning,
        }],
    };
    session_mut(&mut a)
        .set_collector_draft(d, false, ip("10.92.0.100"), invalid)
        .unwrap();
    assert!(
        session_mut(&mut a)
            .apply_selected(d, false)
            .unwrap_err()
            .contains("row 1")
    );
    assert_eq!(session(&a).current, old);
    session_mut(&mut a)
        .select(d, false, work::Component::Source(ip("10.92.0.10")))
        .unwrap();
    assert_eq!(session(&a).editor.sources[&ip("10.92.0.10")], draft);
}
#[test]
fn authoritative_source_binds_templates_delivery_capacity_practice_and_goal_pairing() {
    let s = fixture();
    let d = definition(&s);
    let a = Attempt::new(&s, 1);
    for key in ["message", "delivery", "capacity", "practice"] {
        let mut v = value(&a);
        let sim = &mut v["state"]["simulation"];
        match key {
            "message" => {
                sim["current"]["environment"]["events"]["port_fault"]["text"] =
                    "Other supplied event".into()
            }
            "delivery" => {
                sim["current"]["environment"]["deliveries"][0]["available"] = false.into()
            }
            "capacity" => {
                sim["current"]["environment"]["devices"]["10.92.0.10"]["history_capacity"] =
                    9.into()
            }
            _ => sim["example"] = sim["current"].clone(),
        };
        let bad: Attempt = serde_json::from_value(v).unwrap();
        assert!(work::context(&bad, &s).is_err());
        assert!(!progress::current_solved(&bad, &s));
    }
    let mut wrong = s.clone();
    wrong.goal = pack::bundled().unwrap()[0].goal.clone();
    assert!(work::context(&a, &wrong).is_err());
    let mut wrong = a.clone();
    wrong.state = pack::bundled().unwrap()[0].initial.clone();
    assert!(work::context(&wrong, &s).is_err());
    let mut invalid = d.clone();
    invalid.targets[0]
        .remote
        .values_mut()
        .next()
        .unwrap()
        .facilities
        .push(core::Facility::new(23).unwrap());
    assert!(invalid.shape().is_err());
}
#[test]
fn foreign_old_metadata_and_pretaught_source_solution_are_rejected() {
    let s = fixture();
    let mut a = Attempt::new(&s, 1);
    a.workspace = Some(WorkspaceData::new());
    a.workspace.as_mut().unwrap().command = "old terminal draft".into();
    assert!(work::metadata(&a).is_err());
    assert!(work::context(&a, &s).is_err());
    let mut source = s.clone();
    let script = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.teaching.is_some())
        .unwrap();
    source.teaching = script.teaching;
    assert!(work::source(&source).is_err());
    let mut source = s.clone();
    let State::Syslog { simulation } = &mut source.initial else {
        panic!()
    };
    simulation.checked = true;
    assert!(work::source(&source).is_err());
}
#[test]
fn session_codec_rejects_limits_duplicates_unknown_fields_and_preserves_partial_text() {
    let s = fixture();
    let mut a = Attempt::new(&s, 1);
    let mut draft = session(&a).editor.sources[&ip("10.92.0.10")].clone();
    draft.hosts[0] = "10.92.".into();
    session_mut(&mut a)
        .set_source_draft(definition(&s), false, ip("10.92.0.10"), draft)
        .unwrap();
    let raw = serde_json::to_vec(session(&a)).unwrap();
    assert_eq!(work::Session::decode(&raw).unwrap(), *session(&a));
    for n in 0..4 {
        let mut v = value(session(&a));
        match n {
            0 => v["version"] = 2.into(),
            1 => v["unexpected"] = true.into(),
            2 => {
                v["editor"]["sources"]["10.92.0.10"]["hosts"] = serde_json::json!(["x".repeat(16)])
            }
            _ => {
                v["editor"]["collectors"]["10.92.0.100"]["rows"] = serde_json::Value::Array(vec![
                    serde_json::json!({"facility":null,"threshold":"warning"});
                    25
                ])
            }
        };
        assert!(work::Session::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    assert!(work::Session::decode(&vec![b' '; work::MAX_SESSION + 1]).is_err());
    let text = String::from_utf8(raw).unwrap();
    let bad = text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(work::Session::decode(bad.as_bytes()).is_err());
}
#[test]
fn all_twenty_four_facilities_have_honest_distinct_labels() {
    for n in 0..24 {
        let label = work::facility_label(core::Facility::new(n).unwrap());
        assert!(label.starts_with(&format!("{n} ·")));
        if n < 16 {
            assert!(!label.contains("local"));
            assert!(!label.contains("local"));
        } else {
            assert_eq!(label, format!("{n} · local{}", n - 16));
        }
    }
}
#[test]
fn authored_action_transport_is_strict_bounded_and_never_a_learner_shell() {
    let s = fixture();
    let mut a = Attempt::new(&s, 1);
    let before = value(&a);
    for input in [
        "logging trap warnings",
        "rm -rf /",
        "$(touch /private)",
        "{\"action\":\"generate\",\"device\":\"10.92.0.10\",\"event\":\"port_fault\",\"extra\":true}",
    ] {
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, input).is_err());
        assert_eq!(value(&a), before);
    }
    assert!(
        engine::execute_for_goal(&mut a.state, &s.goal, &"x".repeat(work::MAX_ACTION + 1)).is_err()
    );
    assert!(engine::execute(&mut a.state, &s.solution[0]).is_err());
    assert!(
        engine::execute(&mut a.state, "help")
            .unwrap()
            .contains("native")
    );
    assert_eq!(value(&a), before);
}
#[test]
fn source_aware_save_load_export_restore_keep_drafts_and_other_progress_exact() {
    let s = fixture();
    let old = pack::bundled().unwrap()[0].clone();
    let catalog = vec![s.clone(), old.clone()];
    let mut p = profile(&s);
    p.attempts.insert(old.id.clone(), Attempt::new(&old, 0));
    let legacy = value(&p.attempts[&old.id]);
    complete(p.attempts.get_mut(&s.id).unwrap(), &s);
    let mut draft = session(&p.attempts[&s.id]).editor.sources[&ip("10.92.0.10")].clone();
    draft.hosts[0] = "10.92.".into();
    session_mut(p.attempts.get_mut(&s.id).unwrap())
        .set_source_draft(definition(&s), false, ip("10.92.0.10"), draft)
        .unwrap();
    let portable = value(&p);
    let receipts = value(&p.migration_receipts);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    assert!(progress::save(&path, &p).is_err());
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    assert_eq!(value(&loaded), portable);
    assert_eq!(value(&p.migration_receipts), receipts);
    assert_eq!(value(&loaded.attempts[&old.id]), legacy);
    assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &catalog).unwrap();
    let candidate = backup::inspect(
        &export,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(value(&candidate.progress), portable);
    let restored = dir.path().join("restored.json");
    backup::restore(&restored, &candidate, &catalog).unwrap();
    assert_eq!(
        value(&progress::load(&restored, &catalog).unwrap()),
        portable
    );
}
#[test]
fn inactive_wrong_unknown_or_forged_source_refuses_save_and_retains_original() {
    let s = fixture();
    let mut second = s.clone();
    second.id = "dev-syslog-second".into();
    let catalog = vec![s.clone(), second.clone()];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let p = profile(&s);
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut p = p;
    let mut bad = Attempt::new(&second, 0);
    bad.scenario_version += 1;
    p.attempts.insert(second.id.clone(), bad);
    assert!(progress::save_with_catalog(&path, &p, &catalog).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    p.attempts.get_mut(&second.id).unwrap().scenario_version = 1;
    assert!(progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut forged = second.clone();
    forged.goal = pack::bundled().unwrap()[0].goal.clone();
    assert!(progress::save_with_catalog(&path, &p, &[s, forged]).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
#[test]
fn whole_pack_cap_remains_two_mb_and_typed_actions_stay_below_existing_field_cap() {
    let s = fixture();
    assert!(include_str!("fixtures/syslog-workspace-preview.json").len() < pack::MAX_PACK_BYTES);
    for a in &s.solution {
        assert!(a.len() <= work::MAX_ACTION && a.len() <= 4000);
    }
    let scenarios = (0..300)
        .map(|n| {
            let mut c = s.clone();
            c.id = format!("dev-capacity-only-{n}");
            c
        })
        .collect::<Vec<_>>();
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios,
    })
    .unwrap();
    assert!(raw.len() > pack::MAX_PACK_BYTES);
    assert_eq!(pack::load(&raw).unwrap_err(), "Pack exceeds 2 MB");
}
#[test]
fn aggregate_progress_cap_refuses_large_bounded_attempts_without_replacing_profile() {
    let s = fixture();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let empty = Progress {
        schema_version: 1,
        ..Default::default()
    };
    progress::save(&path, &empty).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut catalog = Vec::new();
    let mut p = Progress {
        schema_version: 1,
        clock: 22,
        ..Default::default()
    };
    for n in 0..22 {
        let mut c = s.clone();
        c.id = format!("dev-aggregate-only-{n}");
        let mut a = Attempt::new(&c, n + 1);
        a.transcript = vec!["x".repeat(8192); 100];
        work::context(&a, &c).unwrap();
        p.attempts.insert(c.id.clone(), a);
        catalog.push(c);
    }
    assert!(serde_json::to_vec(&p).unwrap().len() > 16_000_000);
    let err = progress::save_with_catalog(&path, &p, &catalog).unwrap_err();
    assert!(err.contains("16 MB"));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(progress::decode(&vec![b' '; 16_000_001], &catalog).is_err());
}
#[test]
fn invalid_targets_nonseparate_practice_and_unreachable_worked_examples_fail_authoring() {
    let s = fixture();
    let original = definition(&s);
    for n in 0..4 {
        let mut d = original.clone();
        match n {
            0 => d.targets[0]
                .remote
                .values_mut()
                .next()
                .unwrap()
                .facilities
                .clear(),
            1 => d.lesson.fixture = d.initial.clone(),
            2 => d.lesson.worked[0].action = d.lesson.experiment.clone(),
            _ => d.lesson.repair = d.lesson.experiment.clone(),
        };
        assert!(d.validate().is_err());
    }
    let mut s = s.clone();
    s.solution = vec!["{\"action\":\"clear_local\",\"device\":\"10.92.0.10\"}".into()];
    let text = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s],
    })
    .unwrap();
    assert!(
        pack::load(&text).is_err(),
        "Assignment reachability belongs to pack validation, not ordinary context/save"
    );
}

#[test]
fn typed_oversize_drafts_and_invalid_session_refuse_before_candidate_publication() {
    let s = fixture();
    let d = definition(&s);
    let mut a = Attempt::new(&s, 1);
    let before = value(&a);
    let mut p = session(&a)
        .current
        .device_policy(ip("10.92.0.10"))
        .unwrap()
        .clone();
    p.hosts = (1..=9).map(|n| Ipv4Addr::new(192, 0, 2, n)).collect();
    assert!(work::SourceDraft::from_policy(&p).is_err());
    let mut draft = session(&a).editor.sources[&ip("10.92.0.10")].clone();
    draft.hosts = vec!["192.0.2.1".into(); 9];
    assert!(
        session_mut(&mut a)
            .set_source_draft(d, false, ip("10.92.0.10"), draft)
            .is_err()
    );
    assert_eq!(value(&a), before);
    let draft = work::CollectorDraft {
        rows: vec![
            work::SelectorDraft {
                facility: None,
                threshold: core::Severity::Debug
            };
            25
        ],
    };
    assert!(
        session_mut(&mut a)
            .set_collector_draft(d, false, ip("10.92.0.100"), draft)
            .is_err()
    );
    assert_eq!(value(&a), before);
    session_mut(&mut a).version = 99;
    let before = value(&a);
    assert!(session_mut(&mut a).apply_selected(d, false).is_err());
    assert!(session_mut(&mut a).check(d, false).is_err());
    assert!(session_mut(&mut a).reset_example(d).is_err());
    assert!(session(&a).inspect(d, false).is_err());
    assert_eq!(value(&a), before);
}

#[path = "support/syslog_capacity.rs"]
mod capacity;
#[test]
fn whole_attempt_projection_refuses_draft_hint_mode_and_recency_growth_atomically() {
    let source = capacity::padded_source(fixture());
    let d = definition(&source);
    let mut a = Attempt::new(&source, 1);
    a.workspace = Some(WorkspaceData::new());
    a.hints = 9;
    a.challenge = true;
    complete(&mut a, &source);
    session_mut(&mut a).check(d, false).unwrap();
    capacity::fill_to(&source, &mut a, work::MAX_CONTEXT);
    let old = value(&a);
    let mut candidate = session(&a).clone();
    let device = ip("10.92.0.10");
    let mut draft = candidate.editor.sources[&device].clone();
    draft.hosts[0].push('x'); // Supported partial draft, but one byte cannot fit.
    candidate.set_source_draft(d, false, device, draft).unwrap();
    assert!(!d.checks(&candidate).iter().all(|r| r.0));
    assert!(work::binding::publish(&mut a, &source, &candidate, 9).is_err());
    assert_eq!(value(&a), old);
    assert!(progress::current_solved(&a, &source));
    assert!(session(&a).checked);
    assert!(work::binding::request_hint(&mut a, &source).is_err());
    assert_eq!(value(&a), old); // 9→10 needs one more encoded digit.
    assert!(work::binding::set_challenge(&mut a, &source, false).is_err());
    assert_eq!(value(&a), old); // true→false needs one more encoded byte.
    assert!(work::binding::admit_launch(Some(&a), &source, u64::MAX).is_err());
    assert_eq!(value(&a), old);
    // Release bounded room and retry the same legitimate changes.
    a.transcript.remove(0);
    work::binding::publish(&mut a, &source, &candidate, 9).unwrap();
    assert!(!progress::current_solved(&a, &source));
    work::binding::request_hint(&mut a, &source).unwrap();
    assert_eq!(a.hints, 10);
    work::binding::set_challenge(&mut a, &source, false).unwrap();
    work::binding::admit_launch(Some(&a), &source, u64::MAX).unwrap();
}
#[test]
fn complete_projection_matches_real_wire_and_preparation_refuses_without_partial_commit() {
    let source = capacity::padded_source(fixture());
    let mut a = Attempt::new(&source, 1);
    capacity::fill_to(&source, &mut a, work::MAX_CONTEXT);
    let before = value(&a);
    assert!(a.workspace.is_none());
    assert!(work::binding::prepare(&mut a, &source).is_err());
    assert_eq!(value(&a), before);
    assert!(work::binding::admit_launch(Some(&a), &source, 1).is_err());
    assert_eq!(value(&a), before);
    // Publication of the exact unchanged view proves the borrowed wire adapter
    // includes neither accidental field names nor missing skipped metadata.
    let same = session(&a).clone();
    work::binding::publish(&mut a, &source, &same, 0).unwrap();
    assert_eq!(value(&a), before);
    // A growing view must use the same exact full-context check.
    let mut other = same.clone();
    other
        .set_view(
            definition(&source),
            dlms_learning_lab::script_teaching::View::Assignment,
        )
        .unwrap();
    assert!(capacity::size(&source, &a) == work::MAX_CONTEXT);
    let mut direct: Attempt = serde_json::from_value(value(&a)).unwrap();
    direct.state = State::Syslog {
        simulation: Box::new(other.clone()),
    };
    if capacity::size(&source, &direct) > work::MAX_CONTEXT {
        assert!(work::binding::publish(&mut a, &source, &other, 0).is_err());
        assert_eq!(value(&a), before);
    }
}

#[test]
fn fresh_source_admission_rejects_widest_attempt_even_when_source_itself_fits_pack_cap() {
    use std::collections::BTreeMap;
    fn world(octet: u8) -> core::State {
        let address = |n| Ipv4Addr::new(10, octet, 0, n);
        let endpoints = |start| {
            (start..start + 8)
                .map(|n| {
                    (
                        address(n),
                        core::Endpoint {
                            name: "\\\"".repeat(32),
                            history_capacity: 32,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        let devices = endpoints(if octet == 92 { 10 } else { 20 });
        let collectors = endpoints(if octet == 92 { 100 } else { 200 });
        let mut events = BTreeMap::new();
        for n in 0..32 {
            let id = match n {
                0 => "port_fault".into(),
                1 => "debug_detail".into(),
                _ => format!("event_{n:026}"),
            };
            events.insert(
                core::EventId::new(&id).unwrap(),
                core::Event {
                    subsystem: "X".repeat(16),
                    severity: if n == 0 {
                        core::Severity::Warning
                    } else {
                        core::Severity::Debug
                    },
                    mnemonic: "X".repeat(24),
                    text: "\\\"".repeat(128),
                },
            );
        }
        let delivery = devices
            .keys()
            .flat_map(|d| {
                collectors.keys().map(move |c| core::Delivery {
                    device: *d,
                    collector: *c,
                    available: true,
                })
            })
            .collect();
        let policies = devices
            .keys()
            .map(|d| {
                (
                    *d,
                    core::DevicePolicy {
                        process_enabled: true,
                        buffer_threshold: Some(core::Severity::Debug),
                        remote_threshold: Some(core::Severity::Debug),
                        export_facility: core::Facility::new(23).unwrap(),
                        hosts: collectors.keys().copied().collect(),
                    },
                )
            })
            .collect();
        let selectors = collectors
            .keys()
            .map(|c| {
                (
                    *c,
                    core::CollectorPolicy {
                        selectors: (0..24)
                            .map(|n| (core::Facility::new(n).unwrap(), core::Severity::Debug))
                            .collect(),
                    },
                )
            })
            .collect();
        let mut world = core::State::new(
            core::Environment {
                devices,
                collectors,
                events,
                deliveries: delivery,
            },
            policies,
            selectors,
        )
        .unwrap();
        let first = if octet == 92 { 10 } else { 20 };
        for n in first..first + 8 {
            for e in 0..32 {
                let id = match e {
                    0 => "port_fault".into(),
                    1 => "debug_detail".into(),
                    _ => format!("event_{e:026}"),
                };
                world
                    .emit(address(n), &core::EventId::new(&id).unwrap())
                    .unwrap();
            }
        }
        let mut raw = serde_json::to_value(&world).unwrap();
        raw["next_sequence"] = u64::MAX.into();
        for family in ["local", "remote"] {
            for rows in raw[family].as_object_mut().unwrap().values_mut() {
                for row in rows.as_array_mut().unwrap() {
                    row["sequence"] = (u64::MAX - 1024 + row["sequence"].as_u64().unwrap()).into();
                }
            }
        }
        core::State::decode(&serde_json::to_vec(&raw).unwrap()).unwrap()
    }
    let mut s = fixture();
    let Goal::Syslog { definition: d } = &mut s.goal else {
        panic!()
    };
    d.initial = world(92);
    d.lesson.fixture = world(93);
    // The maximal fixture has eight endpoints per side, while the original
    // teaching targets mention only one. Bind every remaining supplied policy
    // explicitly rather than weakening Definition's full-inventory admission.
    d.protected_sources = (11..18).map(|n|ip(&format!("10.92.0.{n}"))).collect();
    d.protected_collectors = (101..108).map(|n|ip(&format!("10.92.0.{n}"))).collect();
    d.lesson.protected_sources = (21..28).map(|n|ip(&format!("10.93.0.{n}"))).collect();
    d.lesson.protected_collectors = (201..208).map(|n|ip(&format!("10.93.0.{n}"))).collect();
    // Separate event identifiers are mandatory, even in this capacity-only fixture.
    let mut raw = serde_json::to_value(&d.lesson.fixture).unwrap();
    fn rename_events(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(map) => {
                if let Some(events) = map
                    .get_mut("events")
                    .and_then(serde_json::Value::as_object_mut)
                {
                    let old = std::mem::take(events);
                    for (k, v) in old {
                        events.insert(format!("p_{k}"), v);
                    }
                }
                for (k, v) in map.iter_mut() {
                    if k == "event" {
                        if let Some(e) = v.as_str() {
                            *v = format!("p_{e}").into();
                        }
                    } else {
                        rename_events(v);
                    }
                }
            }
            serde_json::Value::Array(rows) => {
                for row in rows {
                    rename_events(row)
                }
            }
            _ => (),
        }
    }
    // 32-byte main IDs would exceed32 after prefixing: use the same long length
    // by replacing their first two characters in the independent example.
    fn shrink_ids(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(map) => {
                if let Some(events) = map
                    .get_mut("events")
                    .and_then(serde_json::Value::as_object_mut)
                {
                    let old = std::mem::take(events);
                    for (k, v) in old {
                        events.insert(if k.len() > 30 { k[2..].into() } else { k }, v);
                    }
                }
                for (k, v) in map.iter_mut() {
                    if k == "event" {
                        if let Some(e) = v.as_str()
                            && e.len() > 30
                        {
                            *v = e[2..].into();
                        }
                    } else {
                        shrink_ids(v);
                    }
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    shrink_ids(v)
                }
            }
            _ => (),
        }
    }
    shrink_ids(&mut raw);
    rename_events(&mut raw);
    d.lesson.fixture = core::State::decode(&serde_json::to_vec(&raw).unwrap()).unwrap();
    d.lesson.targets = d.targets.clone();
    for t in &mut d.lesson.targets {
        t.device = ip("10.93.0.20");
        t.event = core::EventId::new(&format!("p_{}", t.event.as_str())).unwrap();
        let old = std::mem::take(&mut t.remote);
        for (_, v) in old {
            t.remote.insert(ip("10.93.0.200"), v);
        }
    }
    let good = core::DevicePolicy {
        process_enabled: true,
        buffer_threshold: Some(core::Severity::Warning),
        remote_threshold: Some(core::Severity::Warning),
        export_facility: core::Facility::new(23).unwrap(),
        hosts: [ip("10.93.0.200")].into(),
    };
    d.lesson.worked[0].action = work::Action::ApplySource {
        device: ip("10.93.0.20"),
        policy: good.clone(),
    };
    let mut off = good.clone();
    off.buffer_threshold = None;
    d.lesson.experiment = work::Action::ApplySource {
        device: ip("10.93.0.20"),
        policy: off,
    };
    d.lesson.repair = work::Action::ApplySource {
        device: ip("10.93.0.20"),
        policy: good,
    };
    d.validate().unwrap();
    s.initial = State::Syslog {
        simulation: Box::new(work::Session::new(d).unwrap()),
    };
    let fresh_size = serde_json::to_vec(&Attempt::new(&s, u64::MAX))
        .unwrap()
        .len();
    assert!(fresh_size > work::MAX_CONTEXT - pack::MAX_PACK_BYTES + 64);
    // Existing bounded guide fields provide enough source padding without changing
    // model limits, authored scenario counts or any execution semantics.
    s.steps = vec!["x".repeat(4000); 50];
    s.hints = vec!["h".repeat(4000); 50];
    s.sources = vec!["https://".to_owned() + &"s".repeat(3992); 50];
    s.guide = (0..50)
        .map(|_| pack::LessonStep {
            title: "t".repeat(4000),
            why: "w".repeat(4000),
            action: "a".repeat(4000),
            commands: vec![
                pack::CommandExample {
                    command: "Private capacity reference note".into(),
                    explanation: "e".into(),
                    simulator_only: true
                };
                10
            ],
        })
        .collect();
    let target = pack::MAX_PACK_BYTES - 64;
    let current = serde_json::to_vec(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s.clone()],
    })
    .unwrap()
    .len();
    assert!(current < target);
    let mut remaining = target - current;
    for g in &mut s.guide {
        for c in &mut g.commands {
            let n = remaining.min(3999);
            c.explanation.push_str(&"e".repeat(n));
            remaining -= n;
        }
    }
    assert_eq!(remaining, 0);
    let raw = serde_json::to_string(&pack::Pack {
        schema_version: 1,
        scenarios: vec![s.clone()],
    })
    .unwrap();
    assert_eq!(raw.len(), target);
    assert!(raw.len() < pack::MAX_PACK_BYTES);
    let old = value(&s);
    let Goal::Syslog { definition: d } = &s.goal else {
        panic!()
    };
    d.shape().unwrap();
    assert!(serde_json::to_vec(&s).unwrap().len() < work::MAX_CONTEXT);
    assert!(
        work::source(&s).is_err(),
        "Source alone fits; widest fresh complete Attempt cannot"
    );
    assert_eq!(value(&s), old);
    assert!(pack::load(&raw).is_err());
}

#[test]
fn example_to_introduction_growth_is_nonvacuous_and_refused_as_a_whole_attempt() {
    let source = capacity::padded_source(fixture());
    let d = definition(&source);
    let mut a = Attempt::new(&source, 1);
    a.workspace = Some(WorkspaceData::new());
    complete(&mut a, &source);
    session_mut(&mut a).check(d, false).unwrap();
    session_mut(&mut a).check(d, true).unwrap();
    session_mut(&mut a)
        .set_view(d, dlms_learning_lab::script_teaching::View::Example)
        .unwrap();
    capacity::fill_to(&source, &mut a, work::MAX_CONTEXT);
    let before = value(&a);
    let mut candidate = session(&a).clone();
    candidate
        .set_view(d, dlms_learning_lab::script_teaching::View::Introduction)
        .unwrap();
    let mut actual_wire: Attempt = serde_json::from_value(before.clone()).unwrap();
    actual_wire.state = State::Syslog {
        simulation: Box::new(candidate.clone()),
    };
    assert_eq!(capacity::size(&source, &actual_wire), work::MAX_CONTEXT + 5);
    assert!(work::binding::publish(&mut a, &source, &candidate, 0).is_err());
    assert_eq!(value(&a), before);
    assert!(session(&a).checked && session(&a).example_checked);
    assert!(progress::current_solved(&a, &source));
    a.transcript.remove(0);
    let hints = a.hints;
    work::binding::publish(&mut a, &source, &candidate, hints).unwrap();
    assert_eq!(
        session(&a).view,
        dlms_learning_lab::script_teaching::View::Introduction
    );
    assert!(progress::current_solved(&a, &source));
}
