//! In-process explicit endpoint tags; no host interfaces, namespace or network.
use dlms_learning_lab::switching::*;
use std::collections::{BTreeMap, BTreeSet};
fn trunk(native: u16) -> Port {
    Port {
        up: true,
        mode: Mode::Trunk {
            native,
            allowed: BTreeSet::from([30, 40, 99]),
        },
    }
}
fn fabric() -> Fabric {
    let ports = BTreeMap::from([
        ("primary".into(), trunk(99)),
        ("native".into(), trunk(30)),
        (
            "reports".into(),
            Port {
                up: true,
                mode: Mode::Access { vlan: 30 },
            },
        ),
        (
            "audit".into(),
            Port {
                up: true,
                mode: Mode::Access { vlan: 40 },
            },
        ),
    ]);
    let names = ["primary", "native", "reports", "audit"];
    Fabric {
        switches: BTreeMap::from([(
            "sw".into(),
            Switch {
                label: "Provided switch".into(),
                vlans: BTreeSet::from([30, 40, 99]),
                ports,
            },
        )]),
        hosts: names
            .iter()
            .map(|n| {
                (
                    (*n).into(),
                    Host {
                        label: (*n).into(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: names
            .iter()
            .map(|n| Cable {
                id: format!("link-{n}"),
                a: Jack::Host { host: (*n).into() },
                b: Jack::Switch {
                    switch: "sw".into(),
                    port: (*n).into(),
                },
                up: true,
            })
            .collect(),
    }
}
fn receivers() -> BTreeMap<String, BTreeSet<u16>> {
    BTreeMap::from([
        ("primary".into(), BTreeSet::from([30])),
        ("native".into(), BTreeSet::from([30])),
    ])
}
#[test]
fn tagged_outbound_and_matching_return_are_both_derived_from_actual_paths() {
    let f = fabric();
    let r = receivers();
    let out = f.broadcast_frame("primary", Some(30), &r).unwrap();
    assert!(out.recipients.contains_key("reports"));
    assert!(!out.recipients.contains_key("audit"));
    assert_eq!(out.recipients["reports"][0].vlan, 30);
    assert_eq!(out.recipients["reports"][0].tag, None);
    let back = f.broadcast_frame("reports", None, &r).unwrap();
    assert!(back.recipients.contains_key("primary"));
    assert_eq!(back.recipients["primary"][0].tag, Some(30));
}
#[test]
fn native_return_can_fail_even_when_outward_delivery_succeeds_and_carrier_is_up() {
    let f = fabric();
    let before = f.clone();
    let r = receivers();
    assert!(
        f.broadcast_frame("native", Some(30), &r)
            .unwrap()
            .recipients
            .contains_key("reports")
    );
    let back = f.broadcast_frame("reports", None, &r).unwrap();
    assert!(!back.recipients.contains_key("native"));
    assert!(
        back.drops
            .iter()
            .any(|s| s.contains("native: untagged frame"))
    );
    assert_eq!(f, before);
    assert!(f.hosts["native"].up);
}
#[test]
fn wrong_tag_selects_another_domain_instead_of_changing_physical_carrier() {
    let f = fabric();
    let r = receivers();
    let out = f.broadcast_frame("primary", Some(40), &r).unwrap();
    assert!(!out.recipients.contains_key("reports"));
    assert!(out.recipients.contains_key("audit"));
    assert!(f.hosts["primary"].up);
    let back = f.broadcast_frame("audit", None, &r).unwrap();
    assert!(!back.recipients.contains_key("primary"));
}
#[test]
fn inactive_and_disallowed_vlans_are_specific_drops() {
    let mut f = fabric();
    f.switches.get_mut("sw").unwrap().vlans.remove(&30);
    let out = f
        .broadcast_frame("primary", Some(30), &receivers())
        .unwrap();
    assert!(out.recipients.is_empty());
    assert!(out.drops.iter().any(|s| s.contains("not active")));
    f.switches.get_mut("sw").unwrap().vlans.insert(30);
    let Mode::Trunk { allowed, .. } = &mut f
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
    let out = f
        .broadcast_frame("primary", Some(30), &receivers())
        .unwrap();
    assert!(out.recipients.is_empty());
    assert!(out.drops.iter().any(|s| s.contains("not allowed into")));
}
#[test]
fn stopped_parent_and_cable_produce_no_cached_delivery() {
    for cable in [false, true] {
        let mut f = fabric();
        if cable {
            f.cables
                .iter_mut()
                .find(|c| c.id == "link-primary")
                .unwrap()
                .up = false;
        } else {
            f.hosts.get_mut("primary").unwrap().up = false;
        }
        let out = f
            .broadcast_frame("primary", Some(30), &receivers())
            .unwrap();
        assert!(out.recipients.is_empty());
        assert!(
            out.drops
                .iter()
                .any(|s| s.contains("down") || s.contains("stopped"))
        );
    }
}
#[test]
fn untagged_legacy_endpoints_keep_their_previous_acceptance_rules() {
    let f = fabric();
    let out = f.broadcast("reports").unwrap();
    assert!(out.recipients.contains_key("native"));
    assert!(!out.recipients.contains_key("primary"));
    assert!(
        out.drops
            .iter()
            .any(|s| s == "primary accepts only untagged frames")
    );
    assert!(!out.recipients.contains_key("audit"));
}
#[test]
fn unsupported_access_tagged_input_stays_an_error_not_success_or_isolation() {
    let f = fabric();
    let error = f
        .broadcast_frame("reports", Some(30), &receivers())
        .unwrap_err();
    assert!(error.contains("Unsupported tagged input"));
}
#[test]
fn priority_reserved_unknown_and_oversized_receiver_inputs_fail_honestly() {
    let f = fabric();
    for tag in [0, 4095, 4096] {
        assert!(
            f.broadcast_frame("primary", Some(tag), &receivers())
                .is_err()
        );
    }
    let mut r = receivers();
    r.insert("outside".into(), BTreeSet::from([30]));
    assert!(f.broadcast_frame("primary", Some(30), &r).is_err());
    let mut r = receivers();
    r.insert("primary".into(), BTreeSet::from([1, 2, 3, 4, 5]));
    assert!(f.broadcast_frame("primary", Some(30), &r).is_err());
    let mut r = receivers();
    r.insert("primary".into(), BTreeSet::from([0]));
    assert!(f.broadcast_frame("primary", Some(30), &r).is_err());
}
#[test]
fn actual_receiver_configuration_controls_delivery_and_empty_means_inactive() {
    let f = fabric();
    let mut r = receivers();
    r.insert("primary".into(), BTreeSet::new());
    assert!(
        !f.broadcast_frame("reports", None, &r)
            .unwrap()
            .recipients
            .contains_key("primary")
    );
    r.insert("primary".into(), BTreeSet::from([30, 40]));
    assert!(
        f.broadcast_frame("reports", None, &r)
            .unwrap()
            .recipients
            .contains_key("primary")
    );
    assert!(
        f.broadcast_frame("audit", None, &r)
            .unwrap()
            .recipients
            .contains_key("primary")
    );
}
