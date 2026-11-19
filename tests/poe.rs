//! Source capacity, compatibility and bounded shared-budget outcomes.
use dlms_learning_lab::poe::{Class, Device, Facts, Plan, Port, Source, State};
fn facts() -> Facts {
    Facts {
        budget_mw: 37_000,
        ports: vec![
            Port {
                id: "Gi1/0/1".into(),
                source: Source::Ieee8023af,
            },
            Port {
                id: "Gi1/0/2".into(),
                source: Source::Ieee8023at,
            },
            Port {
                id: "Gi1/0/3".into(),
                source: Source::Ieee8023at,
            },
            Port {
                id: "Gi1/0/4".into(),
                source: Source::DataOnly,
            },
        ],
        devices: vec![
            Device {
                id: "phone".into(),
                class: Class::Two,
            },
            Device {
                id: "ap".into(),
                class: Class::Four,
            },
            Device {
                id: "display".into(),
                class: Class::Three,
            },
        ],
    }
}
fn plan() -> Plan {
    Plan {
        attachments: vec![Some(0), Some(1), Some(2)],
        enabled: vec![true; 4],
    }
}
#[test]
fn source_limits_and_reservations_are_distinct_from_consumption() {
    assert_eq!(Source::Ieee8023af.capacity_mw(), 15_400);
    assert_eq!(Source::Ieee8023at.capacity_mw(), 30_000);
    assert_eq!(Class::Unknown0.reservation_mw(), 15_400);
    assert_eq!(Class::One.reservation_mw(), 4_000);
    assert_eq!(Class::Two.reservation_mw(), 7_000);
    assert_eq!(Class::Three.reservation_mw(), 15_400);
    assert_eq!(Class::Four.reservation_mw(), 30_000);
}
#[test]
fn exact_source_budget_and_one_milliwatt_below_differ() {
    let mut f = facts();
    let p = plan();
    let r = f.observe(&p).unwrap();
    assert_eq!(r.reserved_mw, 37_000);
    assert_eq!(r.remaining_mw, 0);
    assert_eq!(r.devices[0].state, State::Powered);
    assert_eq!(r.devices[1].state, State::Powered);
    assert_eq!(r.devices[2].state, State::BudgetDenied);
    f.budget_mw -= 1;
    let r = f.observe(&p).unwrap();
    assert_eq!(r.devices[1].state, State::BudgetDenied);
    assert_eq!(r.devices[2].state, State::Powered);
    assert_eq!(r.reserved_mw, 22_400);
}
#[test]
fn incompatible_device_does_not_reserve_power_and_cannot_be_fixed_by_enabling() {
    let f = facts();
    let mut p = plan();
    p.attachments.swap(0, 1);
    let r = f.observe(&p).unwrap();
    assert_eq!(r.devices[1].state, State::Incompatible);
    assert_eq!(r.devices[1].reservation_mw, 0);
    assert_eq!(r.devices[0].state, State::Powered);
}
#[test]
fn disconnected_disabled_and_data_only_are_different_causes() {
    let f = facts();
    let mut p = plan();
    p.attachments[0] = None;
    p.enabled[1] = false;
    p.attachments[2] = Some(3);
    let r = f.observe(&p).unwrap();
    assert_eq!(
        r.devices.iter().map(|d| d.state).collect::<Vec<_>>(),
        vec![State::Disconnected, State::Disabled, State::DataOnly]
    );
    assert_eq!(r.reserved_mw, 0);
    assert_eq!(r.remaining_mw, f.budget_mw);
    p.enabled[3] = false;
    assert_eq!(
        f.observe(&p).unwrap().devices[2].state,
        State::DataOnly,
        "A latent flag cannot turn a data-only port into PoE hardware"
    );
}
#[test]
fn declared_port_order_is_deterministic_not_an_unstated_vendor_scheduler() {
    let f = facts();
    let mut p = plan();
    p.attachments.swap(1, 2);
    let r = f.observe(&p).unwrap();
    assert_eq!(r.devices[2].state, State::Powered);
    assert_eq!(r.devices[1].state, State::BudgetDenied);
    assert_eq!(r.devices[1].remaining_before_mw, 14_600);
    assert_eq!(r, f.observe(&p).unwrap());
}
#[test]
fn duplicate_dangling_and_extra_equipment_fail_before_observation() {
    let f = facts();
    for p in [
        Plan {
            attachments: vec![Some(0), Some(0), Some(2)],
            enabled: vec![true; 4],
        },
        Plan {
            attachments: vec![Some(0), Some(1), Some(4)],
            enabled: vec![true; 4],
        },
        Plan {
            attachments: vec![Some(0), Some(1)],
            enabled: vec![true; 4],
        },
        Plan {
            attachments: plan().attachments,
            enabled: vec![true; 5],
        },
    ] {
        assert!(f.observe(&p).is_err());
    }
    let mut bad = f;
    bad.ports[1].id = bad.ports[0].id.clone();
    assert!(bad.observe(&plan()).is_err());
}
