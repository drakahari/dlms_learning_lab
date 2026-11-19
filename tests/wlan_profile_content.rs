//! Actual authored policy/client behavior and normal registered-catalog coverage.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt},
    wlan_identity::{Authentication, ManagementProtection},
    wlan_workspace::Session,
};
const IDS: [&str; 3] = [
    "ccna-wlan-ssid-repair",
    "ccna-wlan-wpa2-personal",
    "ccna-wlan-wpa3-handover",
];
fn proposed() -> Vec<Scenario> {
    let p: Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v25.json")).unwrap();
    assert_eq!(p.scenarios.len(), 51);
    p.scenarios.into_iter().skip(48).collect()
}
fn definition(s: &Scenario) -> &dlms_learning_lab::wlan_workspace::Definition {
    let Goal::WlanProfiles { definition } = &s.goal else {
        panic!("authored WLAN goal")
    };
    definition
}
fn session(a: &mut Attempt) -> &mut Session {
    let State::WlanProfiles { simulation } = &mut a.state else {
        panic!("authored WLAN state")
    };
    simulation
}
#[test]
fn every_taught_configuration_and_separate_example_reaches_real_case_outcomes() {
    let labs = proposed();
    assert_eq!(labs.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
    for s in labs {
        dlms_learning_lab::wlan_workspace::source(&s).unwrap();
        let d = definition(&s);
        assert_ne!(d.initial.network, d.example.network);
        let mut a = Attempt::new(&s, 1);
        session(&mut a).practice.fill_reference(&d.example).unwrap();
        assert!(!session(&mut a).practice.checked());
        session(&mut a).practice.check(&d.example).unwrap();
        assert!(session(&mut a).practice.solved(&d.example));
        assert!(!progress::current_solved(&a, &s));
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        assert!(progress::current_solved(&a, &s), "{}", s.id);
        assert!(engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
    }
}
#[test]
fn valid_wpa2_alternate_policy_and_independent_edit_order_pass_without_reference_equality() {
    for s in proposed() {
        let d = definition(&s);
        let mut a = Attempt::new(&s, 1);
        let reference = &d.initial.reference[0];
        let mut value = session(&mut a).assignment.fields[0].configuration.clone();
        // Editing a complete assignment is not an exact action-string puzzle.
        value.training_passphrase = reference.configuration.training_passphrase.clone();
        session(&mut a)
            .assignment
            .set(&d.initial, &reference.wlan, value.clone())
            .unwrap();
        value.ssid = reference.configuration.ssid.clone();
        value.enabled = reference.configuration.enabled;
        session(&mut a)
            .assignment
            .set(&d.initial, &reference.wlan, value.clone())
            .unwrap();
        value.authentication = reference.configuration.authentication;
        value.management_protection = reference.configuration.management_protection;
        if s.id == IDS[1] {
            value.management_protection = ManagementProtection::Disabled;
            assert_ne!(value, reference.configuration);
        }
        session(&mut a)
            .assignment
            .set(&d.initial, &reference.wlan, value)
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
        session(&mut a).assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, &s));
    }
}
#[test]
fn incorrect_and_partial_name_security_credential_and_pmf_settings_fail_specifically() {
    for (s, index) in proposed().into_iter().zip(0..) {
        let d = definition(&s);
        let mut draft = dlms_learning_lab::wlan_workspace::Draft::new(&d.initial);
        draft.check(&d.initial).unwrap();
        assert!(!draft.solved(&d.initial));
        let rows = d.initial.rows(&draft.fields).unwrap();
        assert!(rows.iter().any(|r| !r.correct));
        assert!(
            rows.iter()
                .filter(|r| !r.correct)
                .all(|r| !r.feedback.is_empty())
        );
        draft.fill_reference(&d.initial).unwrap();
        let mut value = draft.fields[0].configuration.clone();
        match index {
            0 => {
                value.ssid = value.ssid.to_lowercase();
            }
            1 => {
                value.training_passphrase = "Stale-Practice-42".into();
            }
            2 => {
                value.authentication = Authentication::Wpa2Psk;
                value.management_protection = ManagementProtection::Required;
            }
            _ => unreachable!(),
        }
        draft.set(&d.initial, "staff", value).unwrap();
        draft.check(&d.initial).unwrap();
        assert!(!draft.solved(&d.initial));
        let rows = d.initial.rows(&draft.fields).unwrap();
        assert!(rows.iter().any(|r| r.correct));
        assert!(
            rows.iter()
                .any(|r| !r.correct && r.id.starts_with("client-"))
        );
        if index == 0 {
            assert!(
                rows.iter()
                    .any(|r| !r.correct && r.feedback.contains("case-sensitive"))
            );
        }
        if index == 1 {
            assert!(rows.iter().any(|r| r.id == "client-stale-key"
                && !r.correct
                && r.feedback.contains("remain incompatible")));
        }
        if index == 2 {
            assert!(rows.iter().any(|r| r.id == "client-laptop"
                && !r.correct
                && r.feedback.contains("requires WPA3-Personal")));
        }
    }
}
#[test]
fn malformed_credentials_and_incomplete_wpa3_do_not_earn_negative_case_credit() {
    let labs = proposed();
    for (index, field) in [(1, "training_passphrase"), (2, "management_protection")] {
        let s = &labs[index];
        let d = definition(s);
        let mut values = d.initial.reference.clone();
        if index == 1 {
            values[0].configuration.training_passphrase = "short".into();
        } else {
            values[0].configuration.management_protection = ManagementProtection::Optional;
        }
        let rows = d.initial.rows(&values).unwrap();
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|r| !r.correct));
        assert!(rows.iter().any(|r| r.id == format!("field-staff-{field}")));
        assert!(rows.iter().all(|r| !r.id.starts_with("client-")));
    }
}
#[test]
fn success_invalidation_saved_views_and_selected_reset_preserve_the_other_draft() {
    for s in proposed() {
        let d = definition(&s);
        let mut state = Session::new(d);
        state.assignment.fill_reference(&d.initial).unwrap();
        state.assignment.check(&d.initial).unwrap();
        state.practice.fill_reference(&d.example).unwrap();
        state.practice.check(&d.example).unwrap();
        let encoded = serde_json::to_vec(&state).unwrap();
        let restored: Session = serde_json::from_slice(&encoded).unwrap();
        restored.validate(d).unwrap();
        assert_eq!(state, restored);
        state
            .assignment
            .set(
                &d.initial,
                "staff",
                d.initial.initial_fields()[0].configuration.clone(),
            )
            .unwrap();
        assert!(!state.assignment.checked());
        assert!(!state.assignment.solved(&d.initial));
        assert!(state.practice.solved(&d.example));
        let independent = state.assignment.clone();
        state.reset(d, true).unwrap();
        assert_eq!(state.assignment, independent);
        assert_eq!(
            state.practice,
            dlms_learning_lab::wlan_workspace::Draft::new(&d.example)
        );
    }
}
#[test]
fn old_421_values_and_all_48_tail_objects_remain_exact_and_prerequisites_are_real() {
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v24.json")).unwrap();
    let next: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v25.json")).unwrap();
    assert_eq!(
        &next["scenarios"].as_array().unwrap()[..48],
        old["scenarios"].as_array().unwrap()
    );
    let old = pack::accepted421().unwrap();
    assert_eq!(old.len(), 421);
    let old_units = dlms_learning_lab::course::accepted421(&old).unwrap();
    assert_eq!(old_units.len(), 162);
    let mut all = old.clone();
    all.extend(proposed());
    let chunks = all
        .chunks(8)
        .map(|s| {
            serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":s})).unwrap()
        })
        .collect::<Vec<_>>();
    let refs = chunks.iter().map(String::as_str).collect::<Vec<_>>();
    let checked = pack::load_catalog(&refs).unwrap();
    assert_eq!(checked.len(), 424);
    for before in old {
        let actual = checked.iter().find(|s| s.id == before.id).unwrap();
        assert_eq!(
            serde_json::to_value(&before).unwrap(),
            serde_json::to_value(actual).unwrap()
        );
    }
    let fragment: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/wlan-profile-units-v1.json")).unwrap();
    let mut units = serde_json::to_value(&old_units)
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    let at = units
        .iter()
        .position(|u| u["id"] == "ccna-wireless-channel-foundations")
        .unwrap()
        + 1;
    units.splice(at..at, fragment["units"].as_array().unwrap().clone());
    let actual = dlms_learning_lab::course::load(
        &serde_json::to_string(&serde_json::json!({"version":1,"units":units})).unwrap(),
        &checked,
    )
    .unwrap();
    assert_eq!(actual.len(), 163);
    for before in old_units {
        assert_eq!(
            serde_json::to_value(actual.iter().find(|u| u.id == before.id).unwrap()).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
}

#[test]
fn normal_catalog_registers_all_three_real_tasks_after_the_channel_prerequisites() {
    let current = pack::current().unwrap();
    assert_eq!(current.len(), 424);
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!(units.len(), 163);
    let at = units
        .iter()
        .position(|u| u.id == "ccna-wireless-channel-foundations")
        .unwrap();
    assert_eq!(units[at + 1].id, "ccna-wlan-personal-security");
    assert_eq!(units[at + 1].labs, IDS);
    for s in proposed() {
        let actual = current.iter().find(|v| v.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(&s).unwrap()
        );
    }
}
