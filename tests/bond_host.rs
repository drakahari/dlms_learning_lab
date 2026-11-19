//! Pure composition fixtures; no catalog/learner credit or host networking.
#[path = "support/bond_control.rs"]
mod c;
#[path = "support/vlan_host.rs"]
mod f;
use dlms_learning_lab::{
    bond_control as control, bond_host as b, nm_profiles as nm, switching::Mode,
};
use std::collections::BTreeMap;
fn facts() -> b::Facts {
    b::Facts {
        version: 1,
        profiles: c::source(),
    }
}
fn vlan() -> dlms_learning_lab::vlan_host::Facts {
    f::environment().vlan.unwrap()
}
fn profiles() -> BTreeMap<String, nm::Profile> {
    let env = f::environment();
    let mut p = f::choose(&env, &f::files()).selected().clone();
    for q in p.values_mut() {
        q.vlan.as_mut().unwrap().parent = "bond0".into();
    }
    p
}
fn snapshot(
    v: &dlms_learning_lab::vlan_host::Facts,
    files: &BTreeMap<String, String>,
) -> control::Snapshot {
    let b = facts();
    control::Snapshot::automatic(
        control::Plan::from_files(&b.profiles, files).unwrap(),
        &b.carrier(v).unwrap(),
    )
    .unwrap()
}
#[test]
fn actual_active_control_member_drives_outbound_and_return_tags_for_both_vlans() {
    let b = facts();
    let v = vlan();
    let p = profiles();
    let s = snapshot(&v, &c::files());
    assert_eq!(
        b.selection(&v, &s).unwrap().active.as_deref(),
        Some("enp1s0")
    );
    for (iface, ip) in [("reports0", "10.20.0.50"), ("audit0", "10.21.0.50")] {
        let d = b.delivery(&v, &s, &p, iface, f::ip(ip)).unwrap();
        assert!(d.round_trip(), "{}", d.detail);
        assert!(d.detail.contains("bond0 active enp1s0"));
    }
    assert!(
        p.values()
            .all(|p| p.vlan.as_ref().unwrap().parent == "bond0")
    );
}
#[test]
fn healthy_carrier_does_not_hide_native_return_failure_or_force_service_based_failover() {
    let b = facts();
    let v = vlan();
    let mut fs = c::files();
    let text = fs.get_mut("bond.nmconnection").unwrap();
    *text = text.replace("primary=enp1s0", "primary=enp2s0");
    let s = snapshot(&v, &fs);
    assert!(b.carrier(&v).unwrap().values().all(|x| *x));
    assert_eq!(
        b.selection(&v, &s).unwrap().active.as_deref(),
        Some("enp2s0")
    );
    let d = b
        .delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
        .unwrap();
    assert!(d.outward);
    assert!(!d.returning);
    assert!(d.detail.contains("untagged frame"));
    assert!(
        b.delivery(&v, &s, &profiles(), "audit0", f::ip("10.21.0.50"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn monitored_primary_failure_uses_the_other_member_and_its_actual_switch_policy() {
    let b = facts();
    let mut v = vlan();
    v.fabric.hosts.get_mut("primary").unwrap().up = false;
    let s = snapshot(&v, &c::files());
    assert_eq!(
        b.selection(&v, &s).unwrap().active.as_deref(),
        Some("enp2s0")
    );
    assert!(
        !b.delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
            .unwrap()
            .round_trip()
    );
    let Mode::Trunk { native, .. } = &mut v
        .fabric
        .switches
        .get_mut("sw")
        .unwrap()
        .ports
        .get_mut("native")
        .unwrap()
        .mode
    else {
        panic!()
    };
    *native = 99;
    assert!(
        b.delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn missing_allowed_vlan_does_not_become_a_fake_carrier_failure() {
    let b = facts();
    let mut v = vlan();
    let Mode::Trunk { allowed, .. } = &mut v
        .fabric
        .switches
        .get_mut("sw")
        .unwrap()
        .ports
        .get_mut("primary")
        .unwrap()
        .mode
    else {
        panic!()
    };
    allowed.remove(&30);
    let s = snapshot(&v, &c::files());
    assert!(b.carrier(&v).unwrap()["enp1s0"]);
    assert_eq!(
        b.selection(&v, &s).unwrap().active.as_deref(),
        Some("enp1s0")
    );
    let d = b
        .delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
        .unwrap();
    assert!(!d.round_trip());
    assert!(
        b.delivery(&v, &s, &profiles(), "audit0", f::ip("10.21.0.50"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn stopped_controller_or_automatic_members_cannot_supply_a_phantom_path() {
    let b = facts();
    let v = vlan();
    for (from, to) in [
        ("autoconnect=true", "autoconnect=false"),
        ("autoconnect-ports=1", "autoconnect-ports=0"),
    ] {
        let mut fs = c::files();
        let t = fs.get_mut("bond.nmconnection").unwrap();
        *t = t.replace(from, to);
        let s = snapshot(&v, &fs);
        let d = b
            .delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
            .unwrap();
        assert!(!d.outward && !d.returning);
        assert!(d.detail.contains("no selected monitored member"));
        assert!(b.carrier(&v).unwrap().values().all(|x| *x));
    }
}
#[test]
fn current_snapshot_and_future_plan_have_distinct_derived_paths_without_mutating_logical_profiles()
{
    let b = facts();
    let v = vlan();
    let p = profiles();
    let current = snapshot(&v, &c::files());
    let mut edited = c::files();
    let t = edited.get_mut("bond.nmconnection").unwrap();
    *t = t.replace("primary=enp1s0", "primary=enp2s0");
    let future = snapshot(&v, &edited);
    assert!(
        b.delivery(&v, &current, &p, "reports0", f::ip("10.20.0.50"))
            .unwrap()
            .round_trip()
    );
    assert!(
        !b.delivery(&v, &future, &p, "reports0", f::ip("10.20.0.50"))
            .unwrap()
            .round_trip()
    );
    assert_eq!(p, profiles());
}
#[test]
fn both_members_down_is_an_honest_no_path_state_and_recovers_from_new_observations() {
    let b = facts();
    let mut v = vlan();
    for n in ["primary", "native"] {
        v.fabric.hosts.get_mut(n).unwrap().up = false;
    }
    let s = snapshot(&v, &c::files());
    assert!(b.selection(&v, &s).unwrap().active.is_none());
    let d = b
        .delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
        .unwrap();
    assert!(!d.round_trip());
    v.fabric.hosts.get_mut("primary").unwrap().up = true;
    assert!(
        b.delivery(&v, &s, &profiles(), "reports0", f::ip("10.20.0.50"))
            .unwrap()
            .round_trip()
    );
}
#[test]
fn foreign_contexts_wrong_roles_duplicate_tags_and_control_identity_collisions_fail() {
    let b = facts();
    let v = vlan();
    let s = snapshot(&v, &c::files());
    let mut bad = b.clone();
    bad.version = 2;
    assert!(bad.validate(&v).is_err());
    let mut bad = b.clone();
    bad.profiles.members[1].interface = "enp9s0".into();
    assert!(bad.validate(&v).is_err());
    let mut bad = b.clone();
    bad.profiles.controller.interface = "reports0".into();
    assert!(bad.validate(&v).is_err());
    let mut forged = s.clone();
    forged.plan.source.controller.uuid = "20000000-0000-4000-8000-000000000001".into();
    assert!(b.selection(&v, &forged).is_err());
    for kind in 0..4 {
        let mut p = profiles();
        match kind {
            0 => p.get_mut("reports0").unwrap().vlan.as_mut().unwrap().parent = "enp1s0".into(),
            1 => p.get_mut("audit0").unwrap().vlan.as_mut().unwrap().id = 30,
            2 => p.get_mut("reports0").unwrap().uuid = b.profiles.controller.uuid.clone(),
            _ => p.get_mut("reports0").unwrap().vlan.as_mut().unwrap().id = 0,
        };
        assert!(b.projected(&v, &s, &p).is_err());
    }
}
#[test]
fn source_physical_states_are_independent_of_profile_eligibility_and_known_per_member() {
    let b = facts();
    for kind in 0..3 {
        let mut v = vlan();
        match kind {
            0 => v.fabric.hosts.get_mut("primary").unwrap().up = false,
            1 => {
                v.fabric
                    .cables
                    .iter_mut()
                    .find(|c| c.id == "link-primary")
                    .unwrap()
                    .up = false
            }
            _ => {
                v.fabric
                    .switches
                    .get_mut("sw")
                    .unwrap()
                    .ports
                    .get_mut("primary")
                    .unwrap()
                    .up = false
            }
        };
        let c = b.carrier(&v).unwrap();
        assert!(!c["enp1s0"]);
        assert!(c["enp2s0"]);
    }
}
#[test]
fn facts_decode_closed_schema_and_missing_or_foreign_destinations_never_discover_real_targets() {
    let b = facts();
    let mut x = serde_json::to_value(&b).unwrap();
    x["run-command"] = "host network".into();
    assert!(serde_json::from_value::<b::Facts>(x).is_err());
    let v = vlan();
    let s = snapshot(&v, &c::files());
    assert!(
        b.delivery(&v, &s, &profiles(), "invented", f::ip("10.20.0.50"))
            .is_err()
    );
    assert!(
        b.delivery(&v, &s, &profiles(), "reports0", f::ip("192.0.2.77"))
            .is_err()
    );
}
