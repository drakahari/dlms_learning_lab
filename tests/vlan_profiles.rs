//! Explicit capability parser tests; never load a host NetworkManager profile.
use dlms_learning_lab::{nm_policy as n, nm_profiles as p};
#[path = "support/vlan_host.rs"]
mod f;
#[test]
fn old_parser_and_old_policy_do_not_silently_accept_vlan() {
    let files = f::files();
    let text = &files["reports.nmconnection"];
    assert!(p::parse(text).is_err());
    assert!(p::parse_settings(text).is_err());
    assert!(n::Policy::from_texts(&files).is_err());
}
#[test]
fn explicit_parser_retains_parent_id_and_current_snapshot_metadata() {
    let text = &f::files()["reports.nmconnection"];
    let x = p::parse_vlan_settings(text).unwrap();
    assert_eq!(
        x.vlan,
        Some(p::Vlan {
            parent: "enp1s0".into(),
            id: 30
        })
    );
    let env = f::environment();
    let c = f::choose(&env, &f::files());
    assert_eq!(c.for_device("reports0").unwrap().vlan, x.vlan);
}
#[test]
fn repeated_vlan_keys_follow_existing_last_assignment_behavior() {
    let s = f::files()["reports.nmconnection"].replace("id=30\nflags=1", "id=40\nid=30\nflags=1");
    assert_eq!(p::parse_vlan_settings(&s).unwrap().vlan.unwrap().id, 30);
}
#[test]
fn priority_and_reserved_ids_are_diagnosed_separately() {
    let s = &f::files()["reports.nmconnection"];
    let e = p::parse_vlan_settings(&s.replace("id=30\n", "id=0\n")).unwrap_err();
    assert_eq!(e.kind, p::ErrorKind::Unsupported);
    assert!(e.message.contains("Priority tag0 is real"));
    for id in ["4095", "4096", "65536", "-1", "an-ip-octet"] {
        assert!(p::parse_vlan_settings(&s.replace("id=30\n", &format!("id={id}\n"))).is_err());
    }
}
#[test]
fn unsupported_protocol_flags_parent_uuid_and_discovery_are_not_ignored() {
    let s = &f::files()["reports.nmconnection"];
    for text in [
        s.replace("802.1Q", "802.1ad"),
        s.replace("flags=1", "flags=0"),
        s.replace("flags=1\n", ""),
        s.replace(
            "parent=enp1s0",
            "parent=00000000-0000-4000-8000-000000000001",
        ),
        s.replace("parent=enp1s0\n", ""),
        s.replace("id=30\n", "id=30\ningress-priority-map=1:2\n"),
        s.replace("parent=enp1s0", "parent=enp*"),
    ] {
        let e = p::parse_vlan_settings(&text).unwrap_err();
        assert_eq!(e.kind, p::ErrorKind::Unsupported, "{e}");
    }
}
#[test]
fn vlan_properties_on_ethernet_and_other_types_have_accurate_diagnostics() {
    let s = &f::files()["reports.nmconnection"];
    assert!(
        p::parse_vlan_settings(&s.replace("type=vlan", "type=ethernet"))
            .unwrap_err()
            .message
            .contains("require connection.type=vlan")
    );
    let e = p::parse_vlan_settings(&s.replace("type=vlan", "type=bond")).unwrap_err();
    assert!(e.message.contains("VLAN profiles"));
    assert!(p::parse_vlan_settings(&s.replace("[vlan]", "[bond]")).is_err());
}
#[test]
fn bounded_syntax_and_unknown_properties_remain_honest() {
    let s = &f::files()["reports.nmconnection"];
    for text in [
        format!("{}#{}", s, "x".repeat(16384)),
        s.replace("parent=enp1s0", "parent=éthernet"),
        s.replace("parent=enp1s0", "parent=enp1s0\\n"),
        s.replace("id=30\n", "id=30\nmade-up=1\n"),
    ] {
        assert!(p::parse_vlan_settings(&text).is_err());
    }
}
#[test]
fn source_enabled_policy_rejects_unknown_parent_and_ethernet_masquerade() {
    let env = f::environment();
    let mut files = f::files();
    files
        .get_mut("reports.nmconnection")
        .unwrap()
        .push_str("\n[vlan]\nparent=enp9s0\n");
    assert!(n::Policy::from_texts_for(&env, &files).is_err());
    let mut files = f::files();
    let old = files["reports.nmconnection"].clone();
    let s = old.replace("type=vlan", "type=ethernet");
    files.insert("reports.nmconnection".into(), s);
    assert!(n::Policy::from_texts_for(&env, &files).is_err());
}
#[test]
fn absent_source_extension_preserves_old_serialized_environment_shape() {
    let mut env = f::environment();
    env.vlan = None;
    let value = serde_json::to_value(&env).unwrap();
    assert!(!value.as_object().unwrap().contains_key("vlan"));
    let restored: n::Environment = serde_json::from_value(value).unwrap();
    assert_eq!(restored, env);
    assert!(n::Policy::from_texts_for(&env, &f::files()).is_err());
}

#[test]
fn first_and_last_data_vlan_ids_are_valid_explicit_values() {
    let s = &f::files()["reports.nmconnection"];
    for id in [1, 4094] {
        let x = p::parse_vlan_settings(&s.replace("id=30\n", &format!("id={id}\n"))).unwrap();
        assert_eq!(x.vlan.unwrap().id, id);
    }
}
