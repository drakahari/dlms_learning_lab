//! Manual-derived regressions, not a real NetworkManager runtime comparison.
//! Checked2026-11-08: https://networkmanager.dev/docs/api/latest/settings-connection.html
//! autoconnect-ports=1 starts all matching ports; their own autoconnect is unrelated.
#[path = "support/bond_control.rs"]
mod f;
use dlms_learning_lab::bond_control as b;
fn manual_ports() -> std::collections::BTreeMap<String, String> {
    let mut files = f::files();
    for name in ["primary.nmconnection", "backup.nmconnection"] {
        let p = files.get_mut(name).unwrap();
        *p = p.replace("autoconnect=true", "autoconnect=false");
    }
    files
}
#[test]
fn controller_forces_ports_even_when_member_autoconnect_is_false() {
    let plan = b::Plan::from_files(&f::source(), &manual_ports()).unwrap();
    let s = plan.automatic(&f::carrier(true, true)).unwrap();
    assert_eq!(
        s.members.len(),
        2,
        "Controller autoconnect-ports=1 activates both matching bound ports, regardless of their own autoconnect flag"
    );
    assert_eq!(s.active.as_deref(), Some("enp1s0"));
}
#[test]
fn no_forced_activation_and_no_independent_port_autoconnect_leaves_ports_unselected() {
    let mut files = manual_ports();
    let p = files.get_mut("bond.nmconnection").unwrap();
    *p = p.replace("autoconnect-ports=1", "autoconnect-ports=0");
    let plan = b::Plan::from_files(&f::source(), &files).unwrap();
    let s = plan.automatic(&f::carrier(true, true)).unwrap();
    assert!(s.controller);
    assert!(s.members.is_empty());
    assert!(s.active.is_none());
}

#[test]
fn bound_port_ip_groups_may_be_omitted_and_minimal_normalized_groups_are_supported() {
    let source = f::source();
    let base = manual_ports();
    let expected = b::Plan::from_files(&source, &base).unwrap();
    let mut normalized = base;
    for name in ["primary.nmconnection", "backup.nmconnection"] {
        let text = normalized.get_mut(name).unwrap();
        *text = text.replace("[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled\n", "");
        text.push_str("\n[ethernet]\n\n[bond-port]\n");
    }
    let text = normalized.get_mut("bond.nmconnection").unwrap();
    *text = text.replace(
        "[ipv6]\nmethod=disabled",
        "[ipv6]\naddr-gen-mode=default\nmethod=disabled",
    );
    text.push_str("\n[proxy]\n");
    assert_eq!(b::Plan::from_files(&source, &normalized).unwrap(), expected);
}
#[test]
fn forced_bound_port_activation_does_not_depend_on_its_own_autoconnect_flag() {
    let mut files = manual_ports();
    for name in ["primary.nmconnection", "backup.nmconnection"] {
        let text = files.get_mut(name).unwrap();
        *text = text.replace("autoconnect=false", "autoconnect=true");
    }
    let p = b::Plan::from_files(&f::source(), &files).unwrap();
    assert_eq!(
        p.automatic(&f::carrier(true, true)).unwrap().members.len(),
        2
    );
    assert_eq!(
        p.automatic(&f::carrier(false, true))
            .unwrap()
            .active
            .as_deref(),
        Some("enp2s0")
    );
}
#[test]
fn independent_port_activation_is_an_explicit_limit_rather_than_a_false_no_path() {
    for gate in [
        "controller-disabled",
        "no-forced-ports",
        "unbound-autoconnect",
    ] {
        let mut files = manual_ports();
        let text = files.get_mut("backup.nmconnection").unwrap();
        *text = text.replace("autoconnect=false", "autoconnect=true");
        match gate {
            "controller-disabled" => {
                let text = files.get_mut("bond.nmconnection").unwrap();
                *text = text.replace("autoconnect=true", "autoconnect=false");
            }
            "no-forced-ports" => {
                let text = files.get_mut("bond.nmconnection").unwrap();
                *text = text.replace("autoconnect-ports=1", "autoconnect-ports=0");
            }
            _ => {
                let text = files.get_mut("backup.nmconnection").unwrap();
                *text = text.replace("controller=bond0\nport-type=bond\n", "");
            }
        }
        let p = b::Plan::from_files(&f::source(), &files).unwrap();
        let e = p.automatic(&f::carrier(true, true)).unwrap_err();
        assert_eq!(
            e.kind,
            dlms_learning_lab::nm_profiles::ErrorKind::Unsupported
        );
        assert!(e.message.contains("Independent port-driven"));
    }
}
