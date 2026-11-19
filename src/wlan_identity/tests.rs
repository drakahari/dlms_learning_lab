use super::*;
fn fixture() -> Network {
    let config = Configuration {
        enabled: true,
        ssid: "Office".into(),
        authentication: Authentication::Wpa2Psk,
        management_protection: ManagementProtection::Optional,
        training_passphrase: "Practice-Key-42".into(),
    };
    let laptop = Client {
        id: "laptop".into(),
        label: "Office laptop".into(),
        ssid: "Office".into(),
        allowed_authentication: vec![Authentication::Wpa2Psk],
        management_protection_capable: true,
        training_passphrase: "Practice-Key-42".into(),
    };
    let mut legacy = laptop.clone();
    legacy.id = "legacy".into();
    legacy.label = "Older handheld".into();
    legacy.management_protection_capable = false;
    let visitor = Client {
        id: "visitor".into(),
        label: "Visitor".into(),
        ssid: "Visitors".into(),
        allowed_authentication: vec![Authentication::Open],
        management_protection_capable: false,
        training_passphrase: String::new(),
    };
    Network {
        wlans: vec![
            Wlan {
                id: "office".into(),
                label: "Staff profile".into(),
                managed: true,
                initial: config,
            },
            Wlan {
                id: "guest".into(),
                label: "Protected visitor profile".into(),
                managed: false,
                initial: Configuration {
                    enabled: true,
                    ssid: "Visitors".into(),
                    authentication: Authentication::Open,
                    management_protection: ManagementProtection::Disabled,
                    training_passphrase: String::new(),
                },
            },
        ],
        clients: vec![laptop, legacy, visitor],
    }
}
fn compatible(r: &Report, client: &str) -> bool {
    matches!(r.clients[client], Compatibility::Compatible { .. })
}
#[test]
fn initial_compatibility_is_derived_for_each_supplied_profile_not_a_stored_success() {
    let n = fixture();
    let mut v = n.initial_configurations();
    let r = n.inspect(&v).unwrap();
    assert!(r.issues.is_empty());
    assert!(n.clients.iter().all(|c| compatible(&r, &c.id)));
    v.get_mut("office").unwrap().enabled = false;
    let r = n.inspect(&v).unwrap();
    assert_eq!(r.clients["laptop"], Compatibility::Disabled);
    assert_eq!(r.clients["legacy"], Compatibility::Disabled);
    assert!(compatible(&r, "visitor"));
    v.get_mut("office").unwrap().enabled = true;
    assert!(compatible(&n.inspect(&v).unwrap(), "laptop"));
}
#[test]
fn network_names_preserve_case_and_spaces_instead_of_trimming_into_success() {
    let n = fixture();
    for name in ["office", "Office ", " Office", "Offices"] {
        let mut v = n.initial_configurations();
        v.get_mut("office").unwrap().ssid = name.into();
        let r = n.inspect(&v).unwrap();
        assert!(r.issues.is_empty());
        assert_eq!(r.clients["laptop"], Compatibility::NameNotFound);
        assert!(compatible(&r, "visitor"));
    }
}
#[test]
fn passphrases_are_exact_and_diagnostics_never_echo_either_value() {
    let n = fixture();
    let mut v = n.initial_configurations();
    for key in ["practice-Key-42", "Practice-Key-42 ", " Practice-Key-42"] {
        v.get_mut("office").unwrap().training_passphrase = key.into();
        let r = n.inspect(&v).unwrap();
        assert_eq!(r.clients["laptop"], Compatibility::PassphraseMismatch);
        assert!(compatible(&r, "visitor"));
        let feedback = r.clients["laptop"].feedback();
        assert!(!feedback.contains(key));
        assert!(!feedback.contains("Practice-Key-42"));
    }
}
#[test]
fn two_valid_pmf_policies_are_equivalent_for_legacy_wpa2_but_required_has_real_effects() {
    let n = fixture();
    let mut v = n.initial_configurations();
    for policy in [
        ManagementProtection::Disabled,
        ManagementProtection::Optional,
    ] {
        v.get_mut("office").unwrap().management_protection = policy;
        let r = n.inspect(&v).unwrap();
        assert!(r.issues.is_empty());
        assert!(compatible(&r, "laptop"));
        assert!(compatible(&r, "legacy"));
        let Compatibility::Compatible {
            protected_management,
            ..
        } = r.clients["laptop"]
        else {
            panic!()
        };
        assert_eq!(
            protected_management,
            policy == ManagementProtection::Optional
        );
    }
    v.get_mut("office").unwrap().management_protection = ManagementProtection::Required;
    let r = n.inspect(&v).unwrap();
    assert!(compatible(&r, "laptop"));
    assert_eq!(
        r.clients["legacy"],
        Compatibility::ManagementProtectionRequired
    );
}
#[test]
fn wpa3_is_sae_with_required_pmf_not_an_alias_for_wpa2_psk() {
    let mut n = fixture();
    n.clients[0]
        .allowed_authentication
        .push(Authentication::Wpa3Sae);
    let mut v = n.initial_configurations();
    let cfg = v.get_mut("office").unwrap();
    cfg.authentication = Authentication::Wpa3Sae;
    cfg.management_protection = ManagementProtection::Required;
    let r = n.inspect(&v).unwrap();
    assert!(r.issues.is_empty());
    assert_eq!(
        r.clients["laptop"],
        Compatibility::Compatible {
            wlan: "office".into(),
            authentication: Authentication::Wpa3Sae,
            protected_management: true
        }
    );
    assert_eq!(r.clients["legacy"], Compatibility::AuthenticationMismatch);
    for policy in [
        ManagementProtection::Optional,
        ManagementProtection::Disabled,
    ] {
        v.get_mut("office").unwrap().management_protection = policy;
        let r = n.inspect(&v).unwrap();
        assert_eq!(r.issues.len(), 1);
        assert_eq!(r.issues[0].field, "management_protection");
        assert!(r.clients.values().all(|c| *c == Compatibility::Deferred));
    }
}
#[test]
fn invalid_fields_defer_all_client_results_instead_of_earning_negative_case_credit() {
    let n = fixture();
    for name in ["", &"x".repeat(33), "Café", "Office\n"] {
        let mut v = n.initial_configurations();
        v.get_mut("office").unwrap().ssid = name.into();
        let r = n.inspect(&v).unwrap();
        assert_eq!(r.issues.len(), 1);
        assert_eq!(r.issues[0].field, "ssid");
        assert!(r.clients.values().all(|c| *c == Compatibility::Deferred));
    }
    for key in ["", "short", &"x".repeat(64), "Practice\nKey"] {
        let mut v = n.initial_configurations();
        v.get_mut("office").unwrap().training_passphrase = key.into();
        let r = n.inspect(&v).unwrap();
        assert_eq!(r.issues[0].field, "training_passphrase");
        assert!(r.clients.values().all(|c| *c == Compatibility::Deferred));
        assert!(!r.issues[0].feedback.contains(key) || key.is_empty());
    }
}
#[test]
fn exact_supported_byte_limits_accept_without_silent_truncation() {
    let mut n = fixture();
    n.clients[0].ssid = "x".repeat(32);
    n.clients[0].training_passphrase = "y".repeat(63);
    let mut v = n.initial_configurations();
    let cfg = v.get_mut("office").unwrap();
    cfg.ssid = n.clients[0].ssid.clone();
    cfg.training_passphrase = n.clients[0].training_passphrase.clone();
    let r = n.inspect(&v).unwrap();
    assert!(r.issues.is_empty());
    assert!(compatible(&r, "laptop"));
    assert_eq!(r.clients["legacy"], Compatibility::NameNotFound);
}
#[test]
fn readonly_profiles_and_exact_identifier_sets_cannot_be_replaced_or_omitted() {
    let n = fixture();
    let mut v = n.initial_configurations();
    v.get_mut("guest").unwrap().enabled = false;
    let r = n.inspect(&v).unwrap();
    assert_eq!(r.issues[0].field, "authority");
    assert!(r.clients.values().all(|c| *c == Compatibility::Deferred));
    let mut v = n.initial_configurations();
    v.remove("guest");
    assert!(n.inspect(&v).is_err());
    let mut v = n.initial_configurations();
    let copy = v["office"].clone();
    v.insert("foreign".into(), copy);
    assert!(n.inspect(&v).is_err());
}
#[test]
fn duplicate_name_selection_is_explicitly_outside_this_model() {
    let n = fixture();
    let mut v = n.initial_configurations();
    v.get_mut("office").unwrap().ssid = "Visitors".into();
    let r = n.inspect(&v).unwrap();
    assert!(
        r.issues
            .iter()
            .any(|i| i.field == "ssid" && i.feedback.contains("duplicate"))
    );
    assert!(r.clients.values().all(|c| *c == Compatibility::Deferred));
}
#[test]
fn open_mode_is_distinct_and_does_not_compare_a_unused_passphrase() {
    let mut n = fixture();
    n.clients[0].allowed_authentication = vec![Authentication::Open];
    let mut v = n.initial_configurations();
    let cfg = v.get_mut("office").unwrap();
    cfg.authentication = Authentication::Open;
    cfg.management_protection = ManagementProtection::Disabled;
    cfg.training_passphrase = "NotUsed".into();
    let r = n.inspect(&v).unwrap();
    assert!(r.issues.is_empty());
    assert!(compatible(&r, "laptop"));
    assert_eq!(r.clients["legacy"], Compatibility::AuthenticationMismatch);
    // Compatibility is not security-policy approval; the task layer must reject
    // an open configuration when a secured WLAN is required.
    let Compatibility::Compatible { authentication, .. } = r.clients["laptop"] else {
        panic!()
    };
    assert_eq!(authentication, Authentication::Open);
    v.get_mut("office").unwrap().management_protection = ManagementProtection::Optional;
    assert!(!n.inspect(&v).unwrap().issues.is_empty());
}
#[test]
fn source_shapes_unknown_fields_and_unimplemented_modes_refuse() {
    let n = fixture();
    n.validate().unwrap();
    let mut raw = serde_json::to_value(&n).unwrap();
    raw["host_command"] = "echo forbidden".into();
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut raw = serde_json::to_value(&n).unwrap();
    raw["wlans"][0]["initial"]["authentication"] = "wpa2_wpa3_transition".into();
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut raw = serde_json::to_value(&n).unwrap();
    raw["wlans"][0]["initial"]["ssid"] = "x".repeat(129).into();
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut raw = serde_json::to_value(&n).unwrap();
    let c = raw["clients"][0].clone();
    raw["clients"] = serde_json::Value::Array(vec![c; 13]);
    assert!(serde_json::from_value::<Network>(raw).is_err());
    let mut bad = n.clone();
    bad.clients[0]
        .allowed_authentication
        .push(Authentication::Wpa2Psk);
    assert!(bad.validate().is_err());
    let mut bad = n.clone();
    bad.clients[1]
        .allowed_authentication
        .push(Authentication::Wpa3Sae);
    assert!(bad.validate().is_err());
    let mut bad = n.clone();
    bad.wlans[0].id = "../personal".into();
    assert!(bad.validate().is_err());
    let mut bad = n.clone();
    bad.wlans.iter_mut().for_each(|w| w.managed = false);
    assert!(bad.validate().is_err());
    let mut bad = n;
    bad.clients.push(bad.clients[0].clone());
    assert!(bad.validate().is_err());
}
