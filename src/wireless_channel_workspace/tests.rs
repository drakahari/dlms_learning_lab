use super::*;
use crate::wireless_channels::{Pair, Radio, Survey};
fn task(prefix: &str) -> Task {
    let names = [
        format!("{prefix}-a"),
        format!("{prefix}-b"),
        format!("{prefix}-c"),
    ];
    let radios = names
        .iter()
        .enumerate()
        .map(|(n, id)| Radio {
            id: id.clone(),
            label: format!("Radio {id}"),
            managed: true,
            initial_channel: [1, 1, 6][n],
        })
        .collect();
    let neighbours = [(0, 1), (1, 2), (0, 2)]
        .iter()
        .map(|(a, b)| Pair {
            a: names[*a].clone(),
            b: names[*b].clone(),
        })
        .collect();
    Task {
        id: prefix.into(),
        network: Network {
            radios,
            surveys: vec![Survey {
                id: format!("{prefix}-survey"),
                label: format!("Survey {prefix}"),
                neighbours,
            }],
        },
        reference: names
            .iter()
            .enumerate()
            .map(|(n, id)| Field {
                radio: id.clone(),
                value: [1, 6, 11][n].to_string(),
            })
            .collect(),
    }
}
fn definition() -> Definition {
    Definition {
        initial: task("independent"),
        example: task("practice"),
    }
}
#[test]
fn examples_fill_inertly_and_checking_them_cannot_grant_assignment_credit() {
    let d = definition();
    d.validate().unwrap();
    let mut s = Session::new(&d);
    let independent = s.assignment.clone();
    s.practice.fill_reference(&d.example).unwrap();
    assert!(!s.practice.checked());
    assert!(!s.solved(&d));
    assert!(
        s.practice
            .check(&d.example)
            .unwrap()
            .iter()
            .all(|r| r.correct)
    );
    assert!(s.practice.solved(&d.example));
    assert!(!s.solved(&d));
    assert_eq!(s.assignment, independent);
}
#[test]
fn partial_plans_keep_individual_results_and_supported_alternates_pass() {
    let d = definition();
    let mut s = Session::new(&d);
    s.assignment
        .set(&d.initial, "independent-b", "6".into())
        .unwrap();
    let rows = s.assignment.check(&d.initial).unwrap();
    assert!(rows.iter().any(|r| r.correct));
    assert_eq!(rows.iter().filter(|r| !r.correct).count(), 1);
    assert!(!s.solved(&d));
    for (id, value) in [
        ("independent-a", "11"),
        ("independent-b", "01"),
        ("independent-c", " 6 "),
    ] {
        s.assignment.set(&d.initial, id, value.into()).unwrap();
    }
    assert!(!s.solved(&d));
    s.assignment.check(&d.initial).unwrap();
    assert!(s.solved(&d));
    assert_ne!(s.assignment.fields, d.initial.reference);
}
#[test]
fn edits_and_imported_stale_snapshots_cannot_keep_success_after_breaking_state() {
    let d = definition();
    let mut s = Session::new(&d);
    s.assignment.fill_reference(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.solved(&d));
    let saved = s.assignment.last_check.clone();
    s.assignment
        .set(&d.initial, "independent-b", "1".into())
        .unwrap();
    assert!(!s.assignment.checked());
    assert!(!s.solved(&d));
    s.assignment.last_check = saved;
    s.validate(&d).unwrap();
    assert!(!s.assignment.checked());
    assert!(!s.solved(&d));
    assert!(
        s.assignment
            .check(&d.initial)
            .unwrap()
            .iter()
            .any(|r| !r.correct)
    );
    assert!(!s.solved(&d));
}
#[test]
fn blanks_malformed_values_and_out_of_band_numbers_have_specific_retry_feedback() {
    let d = definition();
    let mut s = Session::new(&d);
    for (value, message) in [
        ("", "Enter a channel"),
        ("6.0", "whole channel"),
        ("0", "1 to 11"),
        ("12", "1 to 11"),
        ("99999999", "1 to 11"),
    ] {
        s.assignment
            .set(&d.initial, "independent-a", value.into())
            .unwrap();
        let rows = s.assignment.check(&d.initial).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].correct);
        assert!(rows[0].feedback.contains(message));
        assert!(!s.solved(&d));
    }
    s.assignment.fill_reference(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.solved(&d));
}
#[test]
fn source_reachability_distinct_practice_and_identifier_rules_are_enforced() {
    let d = definition();
    let mut bad = d.clone();
    bad.initial.reference[1].value = "1".into();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example.network = bad.initial.network.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example.id = bad.initial.id.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.network.radios[1].initial_channel = 6;
    bad.initial.network.radios[2].initial_channel = 11;
    assert!(bad.validate().is_err());
    let mut s = Session::new(&d);
    s.assignment.task_id = d.example.id.clone();
    assert!(s.validate(&d).is_err());
    assert!(!s.solved(&d));
    let mut s = Session::new(&d);
    s.version = 2;
    assert!(s.validate(&d).is_err());
    assert!(!s.solved(&d));
}
#[test]
fn bounded_roundtrip_preserves_both_unfinished_drafts_and_selected_reset_scope() {
    let d = definition();
    let mut s = Session::new(&d);
    s.view = View::Assignment;
    s.assignment
        .set(&d.initial, "independent-a", "6".into())
        .unwrap();
    s.practice
        .set(&d.example, "practice-b", "11".into())
        .unwrap();
    let bytes = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&bytes).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored, s);
    let example = s.practice.clone();
    s.reset_assignment(&d);
    assert_eq!(s.practice, example);
    assert_eq!(s.assignment, Draft::new(&d.initial));
    let assignment = s.assignment.clone();
    s.reset_practice(&d);
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.practice, Draft::new(&d.example));
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["assignment"]["fields"] = serde_json::json!(vec![
        Field {
            radio: "independent-a".into(),
            value: "1".into()
        };
        9
    ]);
    assert!(serde_json::from_value::<Session>(raw).is_err());
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["practice"]["fields"][0]["value"] = serde_json::json!("1".repeat(9));
    assert!(serde_json::from_value::<Session>(raw).is_err());
}
#[test]
fn failed_oversized_or_foreign_edits_are_atomic_and_protected_radios_stay_source_bound() {
    let mut t = task("independent");
    t.network.radios[0].managed = false;
    t.reference.remove(0);
    t.validate().unwrap();
    let mut d = Draft::new(&t);
    let before = d.clone();
    for (id, value) in [
        ("independent-a", "6"),
        ("other", "1"),
        ("independent-b", "123456789"),
        ("independent-b", "1\n"),
    ] {
        assert!(d.set(&t, id, value.into()).is_err());
        assert_eq!(d, before);
    }
    d.fields.push(Field {
        radio: "independent-a".into(),
        value: "6".into(),
    });
    assert!(d.validate(&t).is_err());
    assert!(!d.solved(&t));
}
fn scenario() -> crate::pack::Scenario {
    let mut s = crate::pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "ccna-gateway")
        .unwrap();
    let d = definition();
    s.id = d.initial.id.clone();
    s.initial = crate::engine::State::WirelessChannels {
        simulation: Box::new(Session::new(&d)),
    };
    s.goal = crate::engine::Goal::WirelessChannels {
        definition: Box::new(d),
    };
    s.solution = vec![
        "@wireless-set independent-a 1".into(),
        "@wireless-set independent-b 6".into(),
        "@wireless-set independent-c 11".into(),
        "@wireless-check".into(),
    ];
    s
}
#[test]
fn authored_source_binding_reachability_and_unbound_execution_are_honest() {
    let s = scenario();
    source(&s).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    assert!(!crate::progress::current_solved(&a, &s));
    assert!(crate::engine::execute(&mut a.state, "@wireless-set independent-b 6").is_err());
    let before = serde_json::to_value(&a.state).unwrap();
    assert!(
        crate::engine::execute_for_goal(&mut a.state, &s.goal, "sh /home/private/script").is_err()
    );
    assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
    for action in &s.solution {
        crate::engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
    }
    assert!(crate::progress::current_solved(&a, &s));
    let mut foreign_version = a.clone();
    foreign_version.scenario_version += 1;
    assert!(!crate::progress::current_solved(&foreign_version, &s));
    let mut foreign_source = s.clone();
    if let crate::engine::State::WirelessChannels { simulation } = &mut foreign_source.initial {
        simulation.assignment.last_check = Some(simulation.assignment.fields.clone());
    }
    assert!(!crate::progress::current_solved(&a, &foreign_source));
    crate::engine::execute_for_goal(&mut a.state, &s.goal, "@wireless-set independent-c 6")
        .unwrap();
    assert!(!crate::progress::current_solved(&a, &s));
    assert!(crate::engine::execute_for_goal(&mut a.state, &s.goal, "@wireless-check").is_ok());
    assert!(!crate::progress::current_solved(&a, &s));
}
#[test]
fn fresh_source_and_foreign_progress_cannot_substitute_another_activity() {
    let s = scenario();
    let a = crate::progress::Attempt::new(&s, 1);
    context(&a, &s).unwrap();
    let mut bad = s.clone();
    if let crate::engine::State::WirelessChannels { simulation } = &mut bad.initial {
        simulation.assignment.last_check = Some(simulation.assignment.fields.clone());
    }
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    if let crate::engine::Goal::WirelessChannels { definition } = &mut bad.goal {
        definition.initial.id = "other".into();
    }
    assert!(source(&bad).is_err());
    let mut a = a;
    a.scenario_version += 1;
    assert!(context(&a, &s).is_err());
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::WirelessChannels { simulation } = &mut a.state {
        simulation.assignment.task_id = "other".into();
    }
    assert!(context(&a, &s).is_err());
    assert!(!crate::progress::current_solved(&a, &s));
}
#[test]
fn disk_roundtrip_retains_all_419_old_attempt_values_and_both_new_drafts() {
    let mut sources = crate::pack::accepted419().unwrap();
    assert_eq!(sources.len(), 419);
    let s = scenario();
    assert!(!sources.iter().any(|v| v.id == s.id));
    sources.push(s.clone());
    let mut p = crate::progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &sources[..419] {
        let mut attempt = crate::progress::Attempt::new(source, 1);
        crate::workspace::prepare(&mut attempt);
        if crate::command_location_workspace::has_source(source) {
            crate::command_location_workspace::prepare(&mut attempt, source, true).unwrap();
        }
        if crate::accounts_template_workspace::has_source(source) {
            crate::accounts_template_workspace::prepare(&mut attempt, source, true).unwrap();
        }
        p.attempts.insert(source.id.clone(), attempt);
    }
    let before = serde_json::to_value(&p.attempts).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::WirelessChannels { simulation } = &mut a.state {
        simulation
            .assignment
            .set(&definition().initial, "independent-b", "11".into())
            .unwrap();
        simulation
            .practice
            .set(&definition().example, "practice-c", "1".into())
            .unwrap();
    }
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a.clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261111/next-wireless/progress-tests");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(format!(
        "{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    crate::progress::save_with_catalog(&path, &p, &sources).unwrap();
    let original = std::fs::read(&path).unwrap();
    let restored = crate::progress::load(&path, &sources).unwrap();
    assert_eq!(restored.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        serde_json::to_value(a).unwrap()
    );
    let mut old = restored.attempts.clone();
    old.remove(&s.id);
    assert_eq!(serde_json::to_value(old).unwrap(), before);
    assert!(crate::progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut forged = p;
    if let crate::engine::State::WirelessChannels { simulation } =
        &mut forged.attempts.get_mut(&s.id).unwrap().state
    {
        simulation.version = 2;
    }
    assert!(crate::progress::save_with_catalog(&path, &forged, &sources).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
