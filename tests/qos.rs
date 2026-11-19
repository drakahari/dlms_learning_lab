//! Pure QoS-model boundaries; no real traffic or host command execution.
use dlms_learning_lab::qos::*;
use std::collections::{BTreeMap, BTreeSet};
fn fixture() -> Fixture {
    Fixture {
        flows: BTreeSet::from(["bulk".into(), "voice".into()]),
        buffer_budget: 8,
        packets: vec![
            Packet {
                id: "b1".into(),
                flow: "bulk".into(),
                arrival: 0,
                dscp: 0,
            },
            Packet {
                id: "b2".into(),
                flow: "bulk".into(),
                arrival: 0,
                dscp: 0,
            },
            Packet {
                id: "v1".into(),
                flow: "voice".into(),
                arrival: 0,
                dscp: 46,
            },
        ],
    }
}
fn policy(priority: bool) -> Policy {
    Policy {
        marks: BTreeMap::from([("bulk".into(), Mark::Keep), ("voice".into(), Mark::Keep)]),
        scheduler: if priority {
            Scheduler::StrictPriority
        } else {
            Scheduler::Fifo
        },
        priority_dscp: if priority {
            BTreeSet::from([46])
        } else {
            BTreeSet::new()
        },
        high_buffer: if priority { 2 } else { 0 },
        standard_buffer: if priority { 6 } else { 8 },
    }
}
#[test]
fn fifo_and_explicit_priority_have_different_real_departures() {
    let f = fixture();
    let a = replay(&f, &policy(false)).unwrap();
    let b = replay(&f, &policy(true)).unwrap();
    assert_eq!(
        a.packets[2].delivery,
        Delivery::Delivered {
            departure: 3,
            waiting_slots: 2
        }
    );
    assert_eq!(
        b.packets[2].delivery,
        Delivery::Delivered {
            departure: 1,
            waiting_slots: 0
        }
    );
    assert_eq!(
        b.packets[0].delivery,
        Delivery::Delivered {
            departure: 2,
            waiting_slots: 1
        }
    );
}
#[test]
fn dscp_numeric_order_is_not_an_implicit_scheduler() {
    let mut f = fixture();
    f.packets[0].dscp = 63;
    let a = replay(&f, &policy(true)).unwrap();
    assert_eq!(a.packets[0].queue, Queue::Standard);
    assert_eq!(a.packets[2].queue, Queue::High);
}
#[test]
fn marking_and_queueing_are_separate_and_guest_marking_can_be_overridden() {
    let mut f = fixture();
    f.packets[0].dscp = 46;
    let mut p = policy(true);
    let bad = replay(&f, &p).unwrap();
    assert_eq!(bad.packets[0].queue, Queue::High);
    p.marks.insert("bulk".into(), Mark::Set(0));
    let fixed = replay(&f, &p).unwrap();
    assert_eq!(fixed.packets[0].incoming_dscp, 46);
    assert_eq!(fixed.packets[0].outgoing_dscp, 0);
    assert_eq!(fixed.packets[0].queue, Queue::Standard);
    assert_eq!(
        fixed.packets[2].delivery,
        Delivery::Delivered {
            departure: 1,
            waiting_slots: 0
        }
    );
}
#[test]
fn waiting_buffer_is_finite_and_arrival_before_service_ties_are_explicit() {
    let mut p = policy(false);
    p.standard_buffer = 1;
    let a = replay(&fixture(), &p).unwrap();
    assert_eq!(
        a.packets[0].delivery,
        Delivery::Delivered {
            departure: 1,
            waiting_slots: 0
        }
    );
    assert_eq!(a.packets[1].delivery, Delivery::TailDrop { at: 0 });
    assert_eq!(a.packets[2].delivery, Delivery::TailDrop { at: 0 });
}
#[test]
fn queue_fifo_order_survives_priority_and_idle_slots() {
    let mut f = fixture();
    f.packets[0].arrival = 2;
    f.packets[1].arrival = 2;
    f.packets[2].arrival = 4;
    let a = replay(&f, &policy(true)).unwrap();
    assert_eq!(
        a.packets[0].delivery,
        Delivery::Delivered {
            departure: 3,
            waiting_slots: 0
        }
    );
    assert_eq!(
        a.packets[1].delivery,
        Delivery::Delivered {
            departure: 4,
            waiting_slots: 1
        }
    );
    assert_eq!(
        a.packets[2].delivery,
        Delivery::Delivered {
            departure: 5,
            waiting_slots: 0
        }
    );
}
#[test]
fn malformed_ids_codes_order_unknown_flows_and_budget_refuse_without_replay() {
    for n in 0..6 {
        let mut f = fixture();
        match n {
            0 => f.packets[1].id = f.packets[0].id.clone(),
            1 => f.packets[0].dscp = 64,
            2 => f.packets[0].arrival = 64,
            3 => f.packets[0].flow = "foreign".into(),
            4 => f.buffer_budget = 0,
            _ => {
                f.packets[0].arrival = 1;
            }
        }
        assert!(replay(&f, &policy(true)).is_err());
    }
    let f = fixture();
    let mut p = policy(true);
    p.standard_buffer = 8;
    assert!(replay(&f, &p).is_err());
    p = policy(false);
    p.priority_dscp.insert(46);
    assert!(replay(&f, &p).is_err());
}

#[test]
fn bounded_json_roundtrips_inputs_without_trusting_stored_outputs() {
    let f = fixture();
    let p = policy(true);
    let parsed = Fixture::from_json(&serde_json::to_string(&f).unwrap()).unwrap();
    let configured = Policy::from_json(&serde_json::to_string(&p).unwrap(), &parsed).unwrap();
    assert_eq!(parsed, f);
    assert_eq!(configured, p);
    assert_eq!(
        replay(&parsed, &configured).unwrap(),
        replay(&f, &p).unwrap()
    );
    let mut foreign = serde_json::to_value(&f).unwrap();
    foreign["result"] = serde_json::json!({"passed":true});
    assert!(Fixture::from_json(&foreign.to_string()).is_err());
}
#[test]
fn decoding_rejects_duplicate_keys_names_codes_large_fields_and_unavailable_features() {
    let f = fixture();
    for raw in [
        r#"{"marks":{"bulk":"Keep","bulk":"Keep","voice":"Keep"},"scheduler":"Fifo","priority_dscp":[],"high_buffer":0,"standard_buffer":8}"#,
        r#"{"marks":{"bulk":"Keep","voice":"Keep"},"scheduler":"StrictPriority","priority_dscp":[46,46],"high_buffer":2,"standard_buffer":6}"#,
        r#"{"marks":{"bulk":"Keep","voice":"Keep"},"scheduler":"WeightedFair","priority_dscp":[],"high_buffer":0,"standard_buffer":8}"#,
        r#"{"marks":{"bulk":{"Set":64},"voice":"Keep"},"scheduler":"Fifo","priority_dscp":[],"high_buffer":0,"standard_buffer":8}"#,
        r#"{"marks":{"bulk":"Keep","voice":"Keep"},"scheduler":"Fifo","priority_dscp":[],"high_buffer":0,"standard_buffer":8,"shell":"/bin/sh"}"#,
    ] {
        assert!(Policy::from_json(raw, &f).is_err(), "{raw}");
    }
    for n in 0..5 {
        let mut raw = serde_json::to_value(&f).unwrap();
        match n {
            0 => raw["flows"] = serde_json::json!(["bulk", "voice", "bulk"]),
            1 => raw["flows"] = serde_json::json!(["a", "b", "c", "d", "e"]),
            2 => raw["packets"][0]["id"] = serde_json::json!("x".repeat(33)),
            3 => {
                raw["packets"] = serde_json::json!(vec![
                    serde_json::to_value(&f.packets[0]).unwrap();
                    MAX_PACKETS + 1
                ])
            }
            _ => raw["packets"][0]["arrival"] = serde_json::json!(MAX_ARRIVAL + 1),
        }
        assert!(
            Fixture::from_json(&raw.to_string()).is_err(),
            "bad fixture{n}"
        );
    }
    assert!(Fixture::from_json(&" ".repeat(MAX_JSON_BYTES + 1)).is_err());
    assert!(Policy::from_json(&" ".repeat(MAX_JSON_BYTES + 1), &f).is_err());
}
#[test]
fn maximum_declared_input_is_deterministic_drains_and_preserves_inputs() {
    let mut f = fixture();
    f.buffer_budget = 64;
    f.packets = (0..MAX_PACKETS)
        .map(|n| Packet {
            id: format!("p{n}"),
            flow: "bulk".into(),
            arrival: MAX_ARRIVAL,
            dscp: 0,
        })
        .collect();
    let mut p = policy(false);
    p.standard_buffer = 64;
    let before = f.clone();
    let first = replay(&f, &p).unwrap();
    assert_eq!(first, replay(&f, &p).unwrap());
    assert_eq!(f, before);
    assert_eq!(first.packets.len(), MAX_PACKETS);
    for (n, packet) in first.packets.iter().enumerate() {
        assert_eq!(
            packet.delivery,
            if n < 64 {
                Delivery::Delivered {
                    departure: MAX_ARRIVAL + 1 + n as u16,
                    waiting_slots: n as u16,
                }
            } else {
                Delivery::TailDrop { at: MAX_ARRIVAL }
            }
        );
    }
}
#[test]
fn strict_priority_can_hold_standard_traffic_for_an_entire_supplied_burst() {
    let mut f = fixture();
    f.buffer_budget = 2;
    f.packets = vec![Packet {
        id: "bulk-first".into(),
        flow: "bulk".into(),
        arrival: 0,
        dscp: 0,
    }];
    f.packets.extend((0..=MAX_ARRIVAL).map(|slot| Packet {
        id: format!("v{slot}"),
        flow: "voice".into(),
        arrival: slot,
        dscp: 46,
    }));
    let mut p = policy(true);
    p.high_buffer = 1;
    p.standard_buffer = 1;
    let trace = replay(&f, &p).unwrap();
    assert_eq!(
        trace.packets[0].delivery,
        Delivery::Delivered {
            departure: 65,
            waiting_slots: 64
        }
    );
    for packet in &trace.packets[1..] {
        assert_eq!(
            packet.delivery,
            Delivery::Delivered {
                departure: packet.arrival + 1,
                waiting_slots: 0
            }
        );
    }
}
#[test]
fn different_nonpriority_marks_can_produce_equivalent_scheduling() {
    let f = fixture();
    let mut a = policy(true);
    let mut b = a.clone();
    a.marks.insert("bulk".into(), Mark::Set(0));
    b.marks.insert("bulk".into(), Mark::Set(63));
    let a = replay(&f, &a).unwrap();
    let b = replay(&f, &b).unwrap();
    for (a, b) in a.packets.iter().zip(&b.packets) {
        assert_eq!(a.queue, b.queue);
        assert_eq!(a.delivery, b.delivery);
    }
    assert_ne!(a.packets[0].outgoing_dscp, b.packets[0].outgoing_dscp);
}
