use super::*;
use crate::wlan_identity::{Client, Wlan};
fn task(tag: &str) -> Task {
    let ssid = format!("{tag}-Staff");
    let key = format!("{tag}-Practice-42");
    let staff = Configuration {
        enabled: false,
        ssid: ssid.clone(),
        authentication: Authentication::Open,
        management_protection: ManagementProtection::Disabled,
        training_passphrase: "Unfinished-Key".into(),
    };
    let guest = Configuration {
        enabled: true,
        ssid: format!("{tag}-Visitors"),
        authentication: Authentication::Open,
        management_protection: ManagementProtection::Disabled,
        training_passphrase: String::new(),
    };
    let mut clients = vec![];
    for (id, label, pmf, pass) in [
        ("laptop", "Staff laptop", true, key.clone()),
        ("scanner", "Older scanner", false, key.clone()),
        (
            "wrong-key",
            "Wrong training key",
            true,
            "Different-Practice-42".into(),
        ),
    ] {
        clients.push(Client {
            id: id.into(),
            label: label.into(),
            ssid: ssid.clone(),
            allowed_authentication: vec![Authentication::Wpa2Psk],
            management_protection_capable: pmf,
            training_passphrase: pass,
        });
    }
    clients.push(Client {
        id: "visitor".into(),
        label: "Existing visitor".into(),
        ssid: guest.ssid.clone(),
        allowed_authentication: vec![Authentication::Open],
        management_protection_capable: false,
        training_passphrase: String::new(),
    });
    let reference = Field {
        wlan: "staff".into(),
        configuration: Configuration {
            enabled: true,
            ssid: ssid.clone(),
            authentication: Authentication::Wpa2Psk,
            management_protection: ManagementProtection::Optional,
            training_passphrase: key,
        },
    };
    Task {
        id: format!("{tag}-wlan"),
        network: Network {
            wlans: vec![
                Wlan {
                    id: "staff".into(),
                    label: "Staff WLAN".into(),
                    managed: true,
                    initial: staff,
                },
                Wlan {
                    id: "guest".into(),
                    label: "Protected guest WLAN".into(),
                    managed: false,
                    initial: guest,
                },
            ],
            clients,
        },
        policies: vec![Policy {
            wlan: "staff".into(),
            ssid,
            enabled: true,
            allowed_authentication: vec![Authentication::Wpa2Psk],
            required_protection: None,
            editable: vec![
                Editable::Enabled,
                Editable::Authentication,
                Editable::ManagementProtection,
                Editable::Passphrase,
            ],
        }],
        cases: vec![
            Case {
                client: "laptop".into(),
                expected: Expected::Compatible {
                    wlan: "staff".into(),
                    authentication: Authentication::Wpa2Psk,
                },
            },
            Case {
                client: "scanner".into(),
                expected: Expected::Compatible {
                    wlan: "staff".into(),
                    authentication: Authentication::Wpa2Psk,
                },
            },
            Case {
                client: "wrong-key".into(),
                expected: Expected::Rejected,
            },
            Case {
                client: "visitor".into(),
                expected: Expected::Compatible {
                    wlan: "guest".into(),
                    authentication: Authentication::Open,
                },
            },
        ],
        reference: vec![reference],
    }
}
fn definition() -> Definition {
    Definition {
        initial: task("assignment"),
        example: task("example"),
    }
}
#[test]
fn actual_reference_and_equivalent_pmf_choice_pass_multiple_positive_and_negative_cases() {
    let t = task("assignment");
    t.validate().unwrap();
    assert!(
        t.rows(&t.initial_fields())
            .unwrap()
            .iter()
            .any(|r| !r.correct)
    );
    let rows = t.rows(&t.reference).unwrap();
    assert_eq!(rows.len(), 8);
    assert!(rows.iter().all(|r| r.correct));
    let mut alternate = t.reference.clone();
    alternate[0].configuration.management_protection = ManagementProtection::Disabled;
    assert!(t.rows(&alternate).unwrap().iter().all(|r| r.correct));
    let mut incompatible = t.reference.clone();
    incompatible[0].configuration.management_protection = ManagementProtection::Required;
    let rows = t.rows(&incompatible).unwrap();
    assert!(rows.iter().any(|r| r.id == "client-laptop" && r.correct));
    let failure = rows.iter().find(|r| r.id == "client-scanner").unwrap();
    assert!(!failure.correct);
    assert!(failure.feedback.contains("does not support"));
}
#[test]
fn one_case_success_wrong_credential_and_disabled_shortcuts_cannot_pass_the_assignment() {
    let t = task("assignment");
    let mut fields = t.reference.clone();
    fields[0].configuration.training_passphrase = "Different-Practice-42".into();
    let rows = t.rows(&fields).unwrap();
    assert!(rows.iter().any(|r| r.id == "client-visitor" && r.correct));
    for id in ["client-laptop", "client-scanner", "client-wrong-key"] {
        assert!(!rows.iter().find(|r| r.id == id).unwrap().correct);
    }
    let mut fields = t.reference.clone();
    fields[0].configuration.enabled = false;
    assert!(t.rows(&fields).unwrap().iter().any(|r| !r.correct));
    let mut fields = t.reference.clone();
    fields[0].configuration.authentication = Authentication::Open;
    fields[0].configuration.management_protection = ManagementProtection::Disabled;
    assert!(
        t.rows(&fields)
            .unwrap()
            .iter()
            .any(|r| r.id == "policy-staff-authentication" && !r.correct)
    );
}
#[test]
fn fill_is_inert_examples_are_separate_and_edits_or_forged_snapshots_cannot_keep_success() {
    let d = definition();
    d.validate().unwrap();
    let mut s = Session::new(&d);
    let untouched = s.assignment.clone();
    s.practice.fill_reference(&d.example).unwrap();
    assert!(!s.practice.checked());
    assert!(!s.practice.solved(&d.example));
    s.practice.check(&d.example).unwrap();
    assert!(s.practice.solved(&d.example));
    assert_eq!(s.assignment, untouched);
    s.assignment.fill_reference(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    assert!(s.assignment.solved(&d.initial));
    let mut wrong = s.assignment.fields[0].configuration.clone();
    wrong.training_passphrase = "Incorrect-Practice-42".into();
    s.assignment.set(&d.initial, "staff", wrong).unwrap();
    assert!(!s.assignment.checked());
    assert!(!s.assignment.solved(&d.initial));
    s.assignment.last_check = Some(s.assignment.fields.clone());
    assert!(!s.assignment.solved(&d.initial));
    assert!(s.practice.solved(&d.example));
}
#[test]
fn saved_session_roundtrip_and_selected_reset_preserve_other_draft_and_view() {
    let d = definition();
    let mut s = Session::new(&d);
    s.view = View::Assignment;
    s.assignment.fill_reference(&d.initial).unwrap();
    s.assignment.check(&d.initial).unwrap();
    s.practice.fill_reference(&d.example).unwrap();
    s.practice.check(&d.example).unwrap();
    let bytes = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&bytes).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(s, restored);
    let assignment = s.assignment.clone();
    s.reset(&d, true).unwrap();
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.view, View::Assignment);
    assert!(!s.practice.checked());
    let practice = s.practice.clone();
    s.reset(&d, false).unwrap();
    assert_eq!(s.practice, practice);
    assert!(!s.assignment.solved(&d.initial));
    let mut bad = restored.clone();
    bad.version = 2;
    assert!(bad.validate(&d).is_err());
    let mut bad = restored;
    bad.assignment.task_id = d.example.id.clone();
    assert!(bad.validate(&d).is_err());
}
#[test]
fn invalid_fields_defer_client_grades_and_explain_the_actual_field_without_leaking_keys() {
    let t = task("assignment");
    let mut fields = t.reference.clone();
    fields[0].configuration.training_passphrase.clear();
    let rows = t.rows(&fields).unwrap();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|r| !r.correct));
    assert!(rows.iter().all(|r| !r.id.starts_with("client-")));
    assert!(
        rows.iter()
            .any(|r| r.id == "field-staff-training_passphrase")
    );
    assert!(
        rows.iter()
            .all(|r| !r.feedback.contains("assignment-Practice-42"))
    );
}
#[test]
fn editing_scope_foreign_fields_and_oversized_text_refuse_atomically() {
    let t = task("assignment");
    let mut draft = Draft::new(&t);
    let before = draft.clone();
    let mut config = draft.fields[0].configuration.clone();
    config.ssid = "Outside-scope".into();
    assert!(draft.set(&t, "staff", config).is_err());
    assert_eq!(draft, before);
    let mut config = draft.fields[0].configuration.clone();
    config.training_passphrase = "x".repeat(129);
    assert!(draft.set(&t, "staff", config).is_err());
    assert_eq!(draft, before);
    let config = draft.fields[0].configuration.clone();
    assert!(draft.set(&t, "guest", config).is_err());
    assert_eq!(draft, before);
    let mut fields = t.reference.clone();
    fields.push(fields[0].clone());
    assert!(t.rows(&fields).is_err());
    let mut fields = t.reference.clone();
    fields[0].wlan = "unknown".into();
    assert!(t.rows(&fields).is_err());
    assert!(t.rows(&[]).is_err());
}
#[test]
fn unreachable_references_incomplete_cases_and_invented_or_already_solved_tasks_refuse() {
    let t = task("assignment");
    let mut bad = t.clone();
    bad.reference[0].configuration.enabled = false;
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases.pop();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.cases[0].client = "foreign".into();
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    for c in &mut bad.cases {
        c.expected = Expected::Rejected;
    }
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.policies.push(bad.policies[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = t.clone();
    bad.policies[0].editable.push(Editable::Enabled);
    assert!(bad.validate().is_err());
    let mut bad = t;
    bad.network.wlans[0].initial = bad.reference[0].configuration.clone();
    assert!(bad.validate().is_err());
    let mut d = definition();
    d.example = d.initial.clone();
    assert!(d.validate().is_err());
}
#[test]
fn bounded_deserialization_rejects_unknown_fields_unsupported_outcomes_and_oversized_collections() {
    let d = definition();
    let session = Session::new(&d);
    let mut raw = serde_json::to_value(&session).unwrap();
    raw["host_run"] = "forbidden".into();
    assert!(serde_json::from_value::<Session>(raw).is_err());
    let mut raw = serde_json::to_value(&session).unwrap();
    let field = raw["assignment"]["fields"][0].clone();
    raw["assignment"]["fields"] = serde_json::Value::Array(vec![field; 5]);
    assert!(serde_json::from_value::<Session>(raw).is_err());
    let mut raw = serde_json::to_value(&d.initial).unwrap();
    raw["cases"][0]["expected"]["outcome"] = "pretend_connected".into();
    assert!(serde_json::from_value::<Task>(raw).is_err());
    let mut raw = serde_json::to_value(&d.initial).unwrap();
    raw["id"] = "x".repeat(65).into();
    assert!(serde_json::from_value::<Task>(raw).is_err());
}

fn scenario() -> crate::pack::Scenario {
    let mut s = crate::pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.id == "ccna-gateway")
        .unwrap();
    let d = definition();
    s.id = d.initial.id.clone();
    // The original gateway's IPv4 teaching does not belong to this synthetic
    // source-binding fixture. Keep the production content validator strict.
    s.client_teaching = None;
    s.initial = crate::engine::State::WlanProfiles {
        simulation: Box::new(Session::new(&d)),
    };
    s.solution = d
        .initial
        .reference
        .iter()
        .map(|f| {
            format!(
                "@wlan-set {} {}",
                f.wlan,
                serde_json::to_string(&f.configuration).unwrap()
            )
        })
        .collect();
    s.solution.push("@wlan-check".into());
    s.goal = crate::engine::Goal::WlanProfiles {
        definition: Box::new(d),
    };
    s
}
#[test]
fn source_engine_reachability_and_metadata_gates_rederive_real_current_success() {
    let s = scenario();
    source(&s).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    assert!(!crate::progress::current_solved(&a, &s));
    let before = serde_json::to_value(&a.state).unwrap();
    for command in [
        "bash /home/private/script",
        "curl https://example.invalid",
        "@wlan-set staff {}",
        "@wlan-set staff {\"authentication\":\"secret-unsupported-mode\"}",
    ] {
        let message = crate::engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap_err();
        assert!(!message.contains("secret-unsupported-mode"));
        assert_eq!(serde_json::to_value(&a.state).unwrap(), before);
    }
    assert!(crate::engine::execute(&mut a.state, "@wlan-check").is_err());
    for command in &s.solution {
        crate::engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    assert!(crate::progress::current_solved(&a, &s));
    let mut foreign = a.clone();
    foreign.scenario_version += 1;
    assert!(!crate::progress::current_solved(&foreign, &s));
    let mut foreign_source = s.clone();
    if let crate::engine::State::WlanProfiles { simulation } = &mut foreign_source.initial {
        simulation.assignment.last_check = Some(simulation.assignment.fields.clone());
    }
    assert!(!crate::progress::current_solved(&a, &foreign_source));
    let mut wrong = definition().initial.reference[0].configuration.clone();
    wrong.training_passphrase = "Wrong-Practice-42".into();
    crate::engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &format!("@wlan-set staff {}", serde_json::to_string(&wrong).unwrap()),
    )
    .unwrap();
    assert!(!crate::progress::current_solved(&a, &s));
    crate::engine::execute_for_goal(&mut a.state, &s.goal, "@wlan-check").unwrap();
    assert!(!crate::progress::current_solved(&a, &s));
}

#[test]
fn incorrect_method_and_unexpected_compatibility_have_distinct_specific_feedback() {
    let mut t = task("assignment");
    t.network.clients[0]
        .allowed_authentication
        .push(Authentication::Wpa3Sae);
    t.policies[0]
        .allowed_authentication
        .push(Authentication::Wpa3Sae);
    t.validate().unwrap();
    let mut values = t.reference.clone();
    values[0].configuration.authentication = Authentication::Wpa3Sae;
    values[0].configuration.management_protection = ManagementProtection::Required;
    let rows = t.rows(&values).unwrap();
    let row = rows.iter().find(|r| r.id == "client-laptop").unwrap();
    assert!(!row.correct);
    assert!(row.feedback.contains("WPA3-Personal"));
    assert!(row.feedback.contains("requires WPA2-Personal"));
    assert!(!row.feedback.contains("required rejection or"));
    let mut values = t.reference.clone();
    values[0].configuration.training_passphrase = "Different-Practice-42".into();
    let rows = t.rows(&values).unwrap();
    let row = rows.iter().find(|r| r.id == "client-wrong-key").unwrap();
    assert!(!row.correct);
    assert!(row.feedback.contains("requires it to remain incompatible"));
}
#[test]
fn source_pack_rejects_foreign_ids_states_and_prechecked_or_unreachable_initial_data() {
    let s = scenario();
    let text =
        serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[s.clone()]}))
            .unwrap();
    assert_eq!(crate::pack::load_catalog(&[&text]).unwrap().len(), 1);
    let mut bad = s.clone();
    if let crate::engine::Goal::WlanProfiles { definition } = &mut bad.goal {
        definition.initial.id = "foreign".into();
    }
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    if let crate::engine::State::WlanProfiles { simulation } = &mut bad.initial {
        simulation.view = View::Assignment;
    }
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    if let crate::engine::Goal::WlanProfiles { definition } = &mut bad.goal {
        definition.initial.reference[0].configuration.enabled = false;
    }
    assert!(source(&bad).is_err());
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::WlanProfiles { simulation } = &mut a.state {
        simulation.assignment.task_id = "foreign".into();
    }
    assert!(context(&a, &s).is_err());
    assert!(!crate::progress::current_solved(&a, &s));
    let mut invalid = definition().initial;
    invalid.cases.clear();
    let mut draft = Draft::new(&invalid);
    draft.last_check = Some(draft.fields.clone());
    assert!(!draft.solved(&invalid));
    assert!(draft.check(&invalid).is_err());
}
#[test]
fn disk_roundtrip_retains_all_421_prior_attempts_favorites_and_separate_wlan_drafts() {
    let mut sources = crate::pack::accepted421().unwrap();
    assert_eq!(sources.len(), 421);
    assert_eq!(crate::course::accepted421(&sources).unwrap().len(), 162);
    let s = scenario();
    assert!(!sources.iter().any(|old| old.id == s.id));
    sources.push(s.clone());
    let mut p = crate::progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for source in &sources[..421] {
        let mut a = crate::progress::Attempt::new(source, 1);
        crate::workspace::prepare(&mut a);
        if crate::command_location_workspace::has_source(source) {
            crate::command_location_workspace::prepare(&mut a, source, true).unwrap();
        }
        if crate::accounts_template_workspace::has_source(source) {
            crate::accounts_template_workspace::prepare(&mut a, source, true).unwrap();
        }
        p.attempts.insert(source.id.clone(), a);
    }
    let before = serde_json::to_value(&p.attempts).unwrap();
    let mut a = crate::progress::Attempt::new(&s, 1);
    if let crate::engine::State::WlanProfiles { simulation } = &mut a.state {
        simulation
            .assignment
            .fill_reference(&definition().initial)
            .unwrap();
        let mut partial = definition().example.reference[0].configuration.clone();
        partial.enabled = false;
        simulation
            .practice
            .set(&definition().example, "staff", partial)
            .unwrap();
    }
    p.favorites.insert(s.id.clone());
    p.favorites.insert(sources[0].id.clone());
    p.attempts.insert(s.id.clone(), a.clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".build-artifacts/continuation/resume-20261111/next-wlan/progress-tests");
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
        serde_json::to_value(&a).unwrap()
    );
    let mut old = restored.attempts.clone();
    old.remove(&s.id);
    assert_eq!(serde_json::to_value(old).unwrap(), before);
    assert!(crate::progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut bad = p;
    if let crate::engine::State::WlanProfiles { simulation } =
        &mut bad.attempts.get_mut(&s.id).unwrap().state
    {
        simulation.version = 2;
    }
    assert!(crate::progress::save_with_catalog(&path, &bad, &sources).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
