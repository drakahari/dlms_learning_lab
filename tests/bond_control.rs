//! Controlled in-memory control profiles; no real interfaces or files are read.
use dlms_learning_lab::{bond_control as b, nm_profiles::ErrorKind};
use std::collections::{BTreeMap, BTreeSet};
#[path = "support/bond_control.rs"]
mod fixture;
use fixture::{carrier, files, source};
#[test]
fn actual_controller_and_two_port_keyfiles_select_only_one_active_member() {
    let p = b::Plan::from_files(&source(), &files()).unwrap();
    assert_eq!(p.controller.name, "bond0");
    assert_eq!(p.controller.primary, "enp1s0");
    for (a, c, active) in [
        (true, true, Some("enp1s0")),
        (false, true, Some("enp2s0")),
        (true, false, Some("enp1s0")),
        (false, false, None),
    ] {
        let s = p.automatic(&carrier(a, c)).unwrap();
        assert!(s.controller);
        assert_eq!(s.members.len(), 2);
        assert_eq!(s.active.as_deref(), active);
    }
}
#[test]
fn legitimate_option_forms_alias_and_last_key_values_are_equivalent() {
    let base = b::Plan::from_files(&source(), &files()).unwrap();
    let mut f = files();
    let s = f.get_mut("bond.nmconnection").unwrap();
    *s = s
        .replace("mode=active-backup", "mode=1")
        .replace("miimon=100", "miimon=00100")
        .replace("primary_reselect=always\n", "");
    s.push_str("\n[bond]\nprimary=enp2s0\nprimary=enp1s0\n");
    for n in ["primary.nmconnection", "backup.nmconnection"] {
        let t = f.get_mut(n).unwrap();
        *t = t.replace("type=ethernet", "type=802-3-ethernet");
    }
    assert_eq!(b::Plan::from_files(&source(), &f).unwrap(), base);
}
#[test]
fn changing_primary_changes_the_derived_active_member_not_a_cached_answer() {
    let mut f = files();
    let s = f.get_mut("bond.nmconnection").unwrap();
    *s = s.replace("primary=enp1s0", "primary=enp2s0");
    let p = b::Plan::from_files(&source(), &f).unwrap();
    assert_eq!(
        p.automatic(&carrier(true, true)).unwrap().active.as_deref(),
        Some("enp2s0")
    );
    assert_eq!(
        p.automatic(&carrier(true, false))
            .unwrap()
            .active
            .as_deref(),
        Some("enp1s0")
    );
}
#[test]
fn controller_member_and_automatic_port_eligibility_are_independent() {
    for (file, from, to, controller, count, active) in [
        (
            "bond.nmconnection",
            "autoconnect=true",
            "autoconnect=false",
            false,
            0,
            None,
        ),
        (
            "bond.nmconnection",
            "autoconnect-ports=1",
            "autoconnect-ports=0",
            true,
            0,
            None,
        ),
        (
            "primary.nmconnection",
            "controller=bond0\nport-type=bond\n",
            "",
            true,
            1,
            Some("enp2s0"),
        ),
    ] {
        let mut f = files();
        let s = f.get_mut(file).unwrap();
        *s = s.replace(from, to);
        let p = b::Plan::from_files(&source(), &f).unwrap();
        let s = p.automatic(&carrier(true, true)).unwrap();
        assert_eq!(
            (s.controller, s.members.len(), s.active.as_deref()),
            (controller, count, active)
        );
    }
}
#[test]
fn explicit_current_membership_does_not_get_replaced_by_future_binding_edits() {
    let before = b::Plan::from_files(&source(), &files()).unwrap();
    let current = b::Snapshot::supplied(
        before,
        b::Current {
            controller: true,
            members: vec![
                b::InterfaceName("enp1s0".into()),
                b::InterfaceName("enp2s0".into()),
            ],
        },
        &carrier(true, true),
    )
    .unwrap();
    let mut f = files();
    let s = f.get_mut("primary.nmconnection").unwrap();
    *s = s.replace("controller=bond0\nport-type=bond\n", "");
    let next = b::Plan::from_files(&source(), &f).unwrap();
    assert_eq!(
        current
            .selection(&carrier(true, true))
            .unwrap()
            .active
            .as_deref(),
        Some("enp1s0")
    );
    assert_eq!(
        next.automatic(&carrier(true, true))
            .unwrap()
            .active
            .as_deref(),
        Some("enp2s0")
    );
    assert!(
        next.selected(
            true,
            &std::collections::BTreeSet::from(["enp1s0".into(), "enp2s0".into()]),
            &carrier(true, true)
        )
        .is_err()
    );
}
#[test]
fn unsupported_modes_monitoring_and_reselection_do_not_silently_approximate_linux() {
    for (from, to) in [
        ("mode=active-backup", "mode=802.3ad"),
        ("mode=active-backup\n", ""),
        ("miimon=100", "miimon=0"),
        ("miimon=100", "miimon=200"),
        ("primary_reselect=always", "primary_reselect=failure"),
        ("autoconnect-ports=1", "autoconnect-ports=-1"),
        ("miimon=100", "options=mode=active-backup,miimon=100"),
    ] {
        let mut f = files();
        let s = f.get_mut("bond.nmconnection").unwrap();
        *s = s.replace(from, to);
        let e = b::Plan::from_files(&source(), &f).unwrap_err();
        assert!(
            matches!(e.kind, ErrorKind::Unsupported | ErrorKind::Invalid),
            "{e}"
        );
    }
}
#[test]
fn wrong_roles_identities_foreign_bindings_and_ip_settings_fail() {
    for (file, from, to) in [
        (
            "primary.nmconnection",
            "controller=bond0",
            "controller=other0",
        ),
        ("primary.nmconnection", "port-type=bond", "port-type=bridge"),
        ("primary.nmconnection", "type=ethernet", "type=vlan"),
        (
            "bond.nmconnection",
            "interface-name=bond0",
            "interface-name=reports0",
        ),
        ("bond.nmconnection", "000000000001", "000000000099"),
        ("bond.nmconnection", "primary=enp1s0", "primary=bond0"),
        (
            "bond.nmconnection",
            "[ipv4]\nmethod=disabled",
            "[ipv4]\nmethod=manual\naddress1=10.20.0.2/24",
        ),
    ] {
        let mut f = files();
        let s = f.get_mut(file).unwrap();
        *s = s.replace(from, to);
        assert!(b::Plan::from_files(&source(), &f).is_err(), "{file} {to}");
    }
}
#[test]
fn malformed_and_unsupported_syntax_have_line_specific_diagnostics() {
    for text in [
        "[bond\nmode=active-backup",
        "outside=value",
        "[bond]\nthis is not a key",
        "[bond]\narp_interval=100",
        "[proxy]\nmethod=none",
    ] {
        let mut f = files();
        f.insert("bond.nmconnection".into(), text.into());
        let e = b::Plan::from_files(&source(), &f).unwrap_err();
        assert!(e.line > 0, "{e}");
        assert!(
            matches!(e.kind, ErrorKind::Syntax | ErrorKind::Unsupported),
            "{e}"
        );
    }
}
#[test]
fn source_and_input_bounds_refuse_traversal_duplicates_and_unexplained_carriers() {
    let s = source();
    s.validate().unwrap();
    for kind in 0..4 {
        let mut bad = s.clone();
        match kind {
            0 => bad.version = 2,
            1 => bad.members.pop().map(|_| ()).unwrap(),
            2 => bad.members[1] = bad.members[0].clone(),
            _ => bad.controller.file = "../real.nmconnection".into(),
        };
        assert!(bad.validate().is_err());
    }
    let mut f = files();
    f.insert("../host.nmconnection".into(), "not read".into());
    assert!(b::Plan::from_files(&s, &f).is_err());
    let mut f = files();
    f.insert("bond.nmconnection".into(), "x".repeat(16385));
    assert_eq!(
        b::Plan::from_files(&s, &f).unwrap_err().kind,
        ErrorKind::Limit
    );
    let p = b::Plan::from_files(&s, &files()).unwrap();
    let mut c = carrier(true, true);
    c.insert("invented".into(), true);
    assert!(p.automatic(&c).is_err());
    assert!(
        p.selected(
            true,
            &BTreeSet::from(["invented".into()]),
            &carrier(true, true)
        )
        .is_err()
    );
    assert!(
        p.selected(
            false,
            &BTreeSet::from(["enp1s0".into()]),
            &carrier(true, true)
        )
        .is_err()
    );
    let mut v = serde_json::to_value(&s).unwrap();
    let extra = v["controller"].clone();
    v["members"].as_array_mut().unwrap().push(extra);
    assert!(serde_json::from_value::<b::Source>(v).is_err());
}
#[test]
fn empty_or_missing_control_files_and_unmodeled_keys_fail_without_host_fallback() {
    let s = source();
    assert!(b::Plan::from_files(&s, &BTreeMap::new()).is_err());
    for bad in [
        "[bond]\nupdelay=200\n",
        "[connection]\ncontroller=other0\n",
        "[connection]\nautoconnect-slaves=1\n",
    ] {
        let mut f = files();
        f.get_mut("bond.nmconnection").unwrap().push_str(bad);
        assert!(b::Plan::from_files(&s, &f).is_err());
    }
}

#[test]
fn saved_future_changes_cannot_preempt_a_supplied_current_control_snapshot() {
    let current = b::Snapshot::supplied(
        b::Plan::from_files(&source(), &files()).unwrap(),
        b::Current {
            controller: true,
            members: vec![
                b::InterfaceName("enp1s0".into()),
                b::InterfaceName("enp2s0".into()),
            ],
        },
        &carrier(true, true),
    )
    .unwrap();
    let mut fs = files();
    let c = fs.get_mut("bond.nmconnection").unwrap();
    *c = c.replace("primary=enp1s0", "primary=enp2s0");
    let next = b::Snapshot::automatic(
        b::Plan::from_files(&source(), &fs).unwrap(),
        &carrier(true, true),
    )
    .unwrap();
    assert_eq!(
        current
            .selection(&carrier(true, true))
            .unwrap()
            .active
            .as_deref(),
        Some("enp1s0")
    );
    assert_eq!(
        next.selection(&carrier(true, true))
            .unwrap()
            .active
            .as_deref(),
        Some("enp2s0")
    );
    assert_eq!(
        current
            .selection(&carrier(false, true))
            .unwrap()
            .active
            .as_deref(),
        Some("enp2s0")
    );
    assert_ne!(
        current.plan.profile("bond.nmconnection"),
        next.plan.profile("bond.nmconnection")
    );
    assert_eq!(
        current.plan.profile("primary.nmconnection"),
        next.plan.profile("primary.nmconnection")
    );
}
#[test]
fn duplicate_current_members_and_forged_plan_shapes_are_rejected_before_selection() {
    let p = b::Plan::from_files(&source(), &files()).unwrap();
    assert!(
        b::Snapshot::supplied(
            p.clone(),
            b::Current {
                controller: true,
                members: vec![
                    b::InterfaceName("enp1s0".into()),
                    b::InterfaceName("enp1s0".into())
                ]
            },
            &carrier(true, true)
        )
        .is_err()
    );
    let mut malformed = p;
    malformed.members.remove("enp2s0");
    assert!(
        malformed
            .automatic(&BTreeMap::from([("enp1s0".into(), true)]))
            .is_err()
    );
}
