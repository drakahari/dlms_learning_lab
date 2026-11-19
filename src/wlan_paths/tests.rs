use super::fixture::world;
use super::*;
use serde_json::json;
fn path(w: &World, id: &str, dest: &str) -> DataPath {
    let Outcome::Data(p) = w.data_path(id, dest).unwrap() else {
        panic!("supplied control path must be ready")
    };
    p
}
#[test]
fn actual_local_ap_mode_uses_controller_even_when_wlan_allows_local_switching() {
    let mut w = world();
    w.fabric
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("ap")
        .unwrap()
        .mode = Mode::Access { vlan: 10 };
    assert_eq!(w.control().unwrap(), Control::Ready);
    let p = path(&w, "staff-wlan", "staff");
    assert!(p.bidirectional());
    assert_eq!(p.forwarding, Forwarding::Central);
    assert_eq!(p.vlan, 20);
    assert_eq!(p.forward.as_ref().unwrap()[0].ingress, "controller");
    assert_eq!(
        p.reverse.as_ref().unwrap().last().unwrap().egress,
        "controller"
    );
}
#[test]
fn flex_local_and_central_choices_derive_different_actual_switch_paths() {
    let mut w = world();
    w.mode = ApMode::FlexConnect;
    let p = path(&w, "staff-wlan", "staff");
    assert!(p.bidirectional());
    assert_eq!(p.forwarding, Forwarding::Local);
    assert_eq!(p.forward.as_ref().unwrap()[0].ingress, "ap");
    w.wlans[0].forwarding = Forwarding::Central;
    let p = path(&w, "staff-wlan", "staff");
    assert!(p.bidirectional());
    assert_eq!(p.forward.as_ref().unwrap()[0].ingress, "controller");
}
#[test]
fn missing_ap_data_vlan_breaks_only_local_data_without_claiming_the_ap_is_disconnected() {
    let mut w = world();
    w.mode = ApMode::FlexConnect;
    w.fabric
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("ap")
        .unwrap()
        .mode = Mode::Trunk {
        native: 10,
        allowed: BTreeSet::from([10, 30]),
    };
    assert_eq!(w.control().unwrap(), Control::Ready);
    assert!(!path(&w, "staff-wlan", "staff").bidirectional());
    assert!(path(&w, "guest-wlan", "guest").bidirectional());
    w.wlans[0].forwarding = Forwarding::Central;
    assert!(path(&w, "staff-wlan", "staff").bidirectional());
}
#[test]
fn controller_data_failure_does_not_invent_a_branch_local_data_failure() {
    let mut w = world();
    w.mode = ApMode::FlexConnect;
    w.fabric
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("controller")
        .unwrap()
        .mode = Mode::Trunk {
        native: 10,
        allowed: BTreeSet::from([10, 20]),
    };
    assert_eq!(w.control().unwrap(), Control::Ready);
    assert!(path(&w, "staff-wlan", "staff").bidirectional());
    assert!(!path(&w, "guest-wlan", "guest").bidirectional());
    w.wlans[1].forwarding = Forwarding::Local;
    assert!(path(&w, "guest-wlan", "guest").bidirectional());
}
#[test]
fn native_mismatch_and_unavailable_management_defer_instead_of_earning_rejection_credit() {
    let mut w = world();
    w.ap_native = 11;
    assert_eq!(
        w.control().unwrap(),
        Control::NativeMismatch {
            configured: 11,
            attached: 10
        }
    );
    assert!(matches!(
        w.data_path("staff-wlan", "staff").unwrap(),
        Outcome::Deferred(_)
    ));
    w.ap_native = 10;
    w.fabric.hosts.get_mut("controller").unwrap().up = false;
    assert_eq!(w.control().unwrap(), Control::NoBidirectionalPath);
    assert!(matches!(
        w.data_path("guest-wlan", "guest").unwrap(),
        Outcome::Deferred(_)
    ));
}
#[test]
fn controller_native_mismatch_cannot_report_the_wrong_untagged_vlan_as_correct() {
    let mut w = world();
    w.controller_native = 20;
    w.wlans[0].controller_vlan = 20;
    assert!(matches!(
        w.data_path("staff-wlan", "staff").unwrap(),
        Outcome::Deferred(Control::ControllerNativeMismatch {
            configured: 20,
            attached: 10
        })
    ));
    assert_eq!(
        w.control().unwrap(),
        Control::ControllerNativeMismatch {
            configured: 20,
            attached: 10
        }
    );
}
#[test]
fn port_link_endpoint_and_destination_errors_are_not_canned_successes() {
    let w = world();
    assert!(w.data_path("not-a-wlan", "staff").is_err());
    assert!(w.data_path("staff-wlan", "ap").is_err());
    for which in ["ap", "controller", "staff"] {
        let mut n = w.clone();
        n.fabric.hosts.get_mut(which).unwrap().up = false;
        match n.data_path("staff-wlan", "staff").unwrap() {
            Outcome::Data(p) => assert!(!p.bidirectional()),
            Outcome::Deferred(_) => {}
        }
    }
    let mut n = w.clone();
    n.fabric
        .cables
        .iter_mut()
        .find(|c| c.id == "cable-staff")
        .unwrap()
        .up = false;
    assert!(!path(&n, "staff-wlan", "staff").bidirectional());
}
#[test]
fn unknown_modes_fields_ids_duplicates_limits_and_serialized_roundtrip_are_explicit() {
    let w = world();
    w.validate().unwrap();
    let raw = serde_json::to_string(&w).unwrap();
    let copy: World = serde_json::from_str(&raw).unwrap();
    assert_eq!(w.control(), copy.control());
    let mut value = serde_json::to_value(&w).unwrap();
    value["mode"] = json!("monitor");
    assert!(serde_json::from_value::<World>(value).is_err());
    let mut value = serde_json::to_value(&w).unwrap();
    value["run_host_command"] = json!("anything");
    assert!(serde_json::from_value::<World>(value).is_err());
    let mut value = serde_json::to_value(&w).unwrap();
    value["wlans"] = json!(vec![w.wlans[0].clone(); 5]);
    assert!(serde_json::from_value::<World>(value).is_err());
    let mut n = w.clone();
    n.wlans[1].id = n.wlans[0].id.clone();
    assert!(n.validate().is_err());
    let mut n = w.clone();
    n.wlans[0].local_vlan = 0;
    assert!(n.validate().is_err());
    let mut n = w.clone();
    n.ap = n.controller.clone();
    assert!(n.validate().is_err());
}

#[test]
fn tagged_data_on_access_identifies_the_actual_ap_attachment_and_subset() {
    let mut w = world();
    w.mode = ApMode::FlexConnect;
    w.fabric
        .switches
        .get_mut("access")
        .unwrap()
        .ports
        .get_mut("ap")
        .unwrap()
        .mode = Mode::Access { vlan: 10 };
    assert_eq!(w.control().unwrap(), Control::Ready);
    let error = w.data_path("staff-wlan", "staff").unwrap_err();
    assert!(
        error.contains("AP attachment")
            && error.contains("tagged VLAN 20")
            && error.contains("Trunk")
    );
    assert!(error.contains("outside its subset"));
}
