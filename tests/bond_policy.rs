//! Source-enabled controller dependencies through the original IP/profile policy.
#[path = "support/bond_policy.rs"]
mod f;
use dlms_learning_lab::{nm_policy as n, nm_profiles as key};
#[test]
fn controller_ports_vlan_addresses_and_actual_two_direction_paths_are_composed() {
    let e = f::environment(20);
    let fs = f::files(20);
    e.validate().unwrap();
    let p = n::Policy::from_texts_for(&e, &fs).unwrap();
    assert_eq!(p.profiles().count(), 2);
    assert!(p.control().is_some());
    let c = f::future(&e, &fs);
    assert_eq!(c.controls().unwrap().current.members.len(), 2);
    for dst in ["10.20.0.50", "10.21.0.50"] {
        let p = c.probe(&e, f::ip(dst));
        assert!(p.pass, "{}", p.reason);
        assert!(p.reason.contains("bond0 active enp1s0"));
    }
}
#[test]
fn three_physical_cases_require_both_eligible_members_not_one_hardcoded_success() {
    let mut bad = f::files(20);
    let t = bad.get_mut("backup.nmconnection").unwrap();
    *t = t.replace("controller=bond0\nport-type=bond\n", "");
    for (which, expected) in [("both", true), ("primary", false), ("native", true)] {
        let mut e = f::environment(20);
        if which != "both" {
            e.vlan
                .as_mut()
                .unwrap()
                .fabric
                .hosts
                .get_mut(which)
                .unwrap()
                .up = false;
        }
        let c = f::future(&e, &bad);
        assert_eq!(c.probe(&e, f::ip("10.20.0.50")).pass, expected);
        let correct = f::future(&e, &f::files(20));
        assert!(correct.probe(&e, f::ip("10.20.0.50")).pass);
    }
}
#[test]
fn valid_alternate_primary_and_supported_keyfile_forms_pass_the_same_failure_cases() {
    let mut fs = f::files(20);
    let t = fs.get_mut("bond.nmconnection").unwrap();
    *t = t
        .replace("primary=enp1s0", "primary=enp2s0")
        .replace("mode=active-backup", "mode=1")
        .replace("primary_reselect=always", "primary_reselect=0");
    for which in ["both", "primary", "native"] {
        let mut e = f::environment(20);
        if which != "both" {
            e.vlan
                .as_mut()
                .unwrap()
                .fabric
                .hosts
                .get_mut(which)
                .unwrap()
                .up = false;
        }
        let c = f::future(&e, &fs);
        assert!(c.probe(&e, f::ip("10.20.0.50")).pass);
        assert!(c.probe(&e, f::ip("10.21.0.50")).pass);
    }
}
#[test]
fn reloading_keeps_actual_current_control_and_ip_snapshots_while_future_uses_edits() {
    let e = f::environment(20);
    let good = f::files(20);
    let current = f::initial(&e, &good);
    let mut edited = good;
    let t = edited.get_mut("bond.nmconnection").unwrap();
    *t = t.replace("primary=enp1s0", "primary=enp2s0");
    let t = edited.get_mut("reports.nmconnection").unwrap();
    *t = t.replace("10.20.0.2/24", "10.20.0.3/24");
    let p = n::Policy::from_texts_for(&e, &edited).unwrap();
    let warm = current.automatic(&e, &p).unwrap().0;
    assert_eq!(warm, current);
    let next = f::future(&e, &edited);
    assert_ne!(next.controls(), warm.controls());
    assert_ne!(next.for_device("reports0"), warm.for_device("reports0"));
    assert!(next.probe(&e, f::ip("10.20.0.50")).pass);
}
#[test]
fn wrong_ip_wrong_vlan_and_no_controller_cannot_be_hidden_by_correct_bonding() {
    let e = f::environment(20);
    for (from, to) in [
        ("id=30\n", "id=99\n"),
        ("address1=10.20.0.2/24", "address1=10.99.0.2/24"),
    ] {
        let mut fs = f::files(20);
        let t = fs.get_mut("reports.nmconnection").unwrap();
        *t = t.replace(from, to);
        let p = n::Policy::from_texts_for(&e, &fs).unwrap();
        match n::Choices::empty().automatic(&e, &p) {
            Ok((c, _)) => assert!(!c.probe(&e, f::ip("10.20.0.50")).pass),
            Err(x) => assert!(matches!(
                x.kind,
                key::ErrorKind::Unsupported | key::ErrorKind::Invalid
            )),
        }
    }
    let mut fs = f::files(20);
    let t = fs.get_mut("bond.nmconnection").unwrap();
    *t = t.replace("autoconnect=true", "autoconnect=false");
    let p = n::Policy::from_texts_for(&e, &fs).unwrap();
    assert!(
        n::Choices::empty()
            .automatic(&e, &p)
            .unwrap_err()
            .message
            .contains("controller is not selected")
    );
}
#[test]
fn protected_control_signatures_check_actual_settings_not_two_absent_ip_entries() {
    let e = f::environment(20);
    let a = n::Policy::from_texts_for(&e, &f::files(20)).unwrap();
    for file in [
        "bond.nmconnection",
        "primary.nmconnection",
        "backup.nmconnection",
        "audit.nmconnection",
    ] {
        assert!(a.protected_unchanged(&a, file));
    }
    assert!(!a.protected_unchanged(&a, "invented.nmconnection"));
    let mut fs = f::files(20);
    let t = fs.get_mut("backup.nmconnection").unwrap();
    *t = t.replace("autoconnect=false", "autoconnect=true");
    let b = n::Policy::from_texts_for(&e, &fs).unwrap();
    assert!(!a.protected_unchanged(&b, "backup.nmconnection"));
    assert!(a.protected_unchanged(&b, "bond.nmconnection"));
    assert!(a.protected_unchanged(&b, "audit.nmconnection"));
}
#[test]
fn absent_context_wire_old_apis_and_snapshot_authority_remain_strict() {
    let e = f::environment(20);
    let fs = f::files(20);
    assert!(n::Policy::from_texts(&fs).is_err());
    let current = f::initial(&e, &fs);
    let mut old = e.clone();
    old.bond = None;
    let v = serde_json::to_value(&old).unwrap();
    assert!(!v.as_object().unwrap().contains_key("bond"));
    assert!(n::Policy::from_texts_for(&old, &fs).is_err());
    assert!(!current.probe(&old, f::ip("10.20.0.50")).pass);
    let mut no_vlan = e;
    no_vlan.vlan = None;
    assert!(no_vlan.validate().is_err());
}
#[test]
fn manual_current_setup_requires_explicit_supplied_control_membership() {
    let e = f::environment(20);
    let p = n::Policy::from_texts_for(&e, &f::files(20)).unwrap();
    let uuid = p.profiles().next().unwrap().1.uuid.clone();
    assert!(n::Choices::empty().manual(&e, &p, &uuid).is_err());
    let c = n::Choices::empty()
        .seed_current_controls(&e, &p, &f::current())
        .unwrap();
    assert!(c.manual(&e, &p, &uuid).is_ok());
    assert!(c.seed_current_controls(&e, &p, &f::current()).is_err());
}
