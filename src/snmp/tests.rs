use super::*;
fn agent() -> Agent {
    Agent {
        uptime_ticks: 12345,
        available: true,
        interfaces: vec![
            Interface {
                index: 10,
                description: "Storage uplink".into(),
                admin: LinkState::Up,
                oper: LinkState::Down,
                speed_bits_per_second: 1_000_000_000,
                in_octets: (1u64 << 32) + 27,
                out_octets: 2000,
                discontinuity_ticks: 200,
            },
            Interface {
                index: 2,
                description: "Office uplink".into(),
                admin: LinkState::Up,
                oper: LinkState::Up,
                speed_bits_per_second: 100_000_000,
                in_octets: 100,
                out_octets: 200,
                discontinuity_ticks: 0,
            },
        ],
    }
}
fn oid(base: &str, index: u32) -> Oid {
    Oid::instance(base, index)
}
fn sample(at: u64, uptime: u32, value: u64) -> Sample {
    Sample {
        at_seconds: at,
        uptime_ticks: uptime,
        discontinuity_ticks: 0,
        value,
    }
}
#[test]
fn numeric_oid_order_not_text_order_controls_next() {
    let a = agent();
    assert_eq!(
        a.get_next(&[oid(DESCR, 2)]).unwrap(),
        vec![Reply::Value {
            oid: oid(DESCR, 10),
            value: Value::Text("Storage uplink".into())
        }]
    );
    assert!(oid(DESCR, 2) < oid(DESCR, 10));
    assert_eq!(Oid::parse(".1.3.6.1.2.1.1.3.0").unwrap(), oid(UPTIME, 0));
}
#[test]
fn get_derives_typed_state_and_distinguishes_unknown_object_from_instance() {
    let a = agent();
    assert_eq!(
        a.get(&[oid(UPTIME, 0), oid(OPER, 10), oid(IN32, 10), oid(IN64, 10)])
            .unwrap(),
        vec![
            Reply::Value {
                oid: oid(UPTIME, 0),
                value: Value::TimeTicks(12345)
            },
            Reply::Value {
                oid: oid(OPER, 10),
                value: Value::Integer(2)
            },
            Reply::Value {
                oid: oid(IN32, 10),
                value: Value::Counter32(27)
            },
            Reply::Value {
                oid: oid(IN64, 10),
                value: Value::Counter64((1u64 << 32) + 27)
            }
        ]
    );
    assert_eq!(
        a.get(&[
            oid(UPTIME, 1),
            oid(DESCR, 999),
            Oid::parse("1.3.6.1.2.1.99.0").unwrap()
        ])
        .unwrap(),
        vec![
            Reply::NoSuchInstance,
            Reply::NoSuchInstance,
            Reply::NoSuchObject
        ]
    );
}
#[test]
fn get_next_crosses_columns_and_finishes_without_inventing_rows() {
    let a = agent();
    assert_eq!(
        a.get_next(&[oid(DESCR, 10)]).unwrap(),
        vec![Reply::Value {
            oid: oid(SPEED, 2),
            value: Value::Gauge32(100_000_000)
        }]
    );
    assert_eq!(
        a.get_next(&[oid(DISCONT, 10)]).unwrap(),
        vec![Reply::EndOfMibView]
    );
}
#[test]
fn state_changes_affect_real_values_not_cached_command_responses() {
    let mut a = agent();
    let first = a.get(&[oid(OPER, 10), oid(IN64, 10)]).unwrap();
    a.interfaces[0].oper = LinkState::Up;
    a.interfaces[0].in_octets += 345;
    assert_ne!(first, a.get(&[oid(OPER, 10), oid(IN64, 10)]).unwrap());
    let restored: Agent = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
    assert_eq!(a, restored);
}
#[test]
fn malformed_and_oversized_queries_refuse_and_unavailability_is_explicit() {
    for raw in [
        "",
        "1",
        "3.1",
        "1.40",
        "-1.3",
        "1..3",
        "1.3.4294967296",
        "1.3; id",
        "1.3\n.4",
    ] {
        assert!(Oid::parse(raw).is_err(), "{raw:?}")
    }
    assert!(Oid::parse(&format!("1.{}", vec!["1"; 33].join("."))).is_err());
    let mut a = agent();
    assert!(a.get(&[]).is_err());
    assert!(a.get(&vec![oid(UPTIME, 0); 17]).is_err());
    assert!(a.get(&[Oid(vec![])]).is_err());
    a.available = false;
    assert!(
        a.get(&[oid(UPTIME, 0)])
            .unwrap_err()
            .contains("not an observed network timeout")
    );
}
#[test]
fn invalid_virtual_agent_metadata_refuses_before_answering() {
    let mut a = agent();
    a.interfaces[1].index = 10;
    assert!(a.get(&[oid(UPTIME, 0)]).is_err());
    a.interfaces[1].index = 0;
    assert!(a.validate().is_err());
    let mut a = agent();
    a.interfaces[0].description = "bad\nrow".into();
    assert!(a.validate().is_err());
    let mut a = agent();
    a.interfaces = vec![a.interfaces[0].clone(); 13];
    assert!(a.validate().is_err());
    let raw = serde_json::to_value(agent()).unwrap();
    let mut bad = raw.clone();
    bad["execute"] = serde_json::json!("host command");
    assert!(serde_json::from_value::<Agent>(bad).is_err());
}
#[test]
fn rates_use_octet_deltas_and_exact_units_with_threshold_alternatives() {
    let r = rate(
        sample(0, 1000, 1000),
        sample(10, 2000, 2000),
        Width::Counter64,
        100_000_000,
    )
    .unwrap();
    assert_eq!(
        r,
        Rate {
            octets: 1000,
            seconds: 10
        }
    );
    assert!(r.at_least_bits_per_second(800));
    assert!(r.at_least_bits_per_second(750));
    assert!(!r.at_least_bits_per_second(801));
}
#[test]
fn a_single_counter_wrap_is_mathematically_correct_at_both_widths() {
    for (width, max) in [
        (Width::Counter32, u32::MAX as u64),
        (Width::Counter64, u64::MAX),
    ] {
        let r = rate(sample(0, 1000, max - 10), sample(10, 2000, 9), width, 1000).unwrap();
        assert_eq!(r.octets, 20);
    }
}
#[test]
fn high_capacity_counters_avoid_ambiguous_whole_wraps() {
    let b = sample(0, 0, 1000);
    let e = sample(60, 6000, 2000);
    assert!(
        rate(b, e, Width::Counter32, 1_000_000_000)
            .unwrap_err()
            .contains("counter")
    );
    assert_eq!(
        rate(b, e, Width::Counter64, 1_000_000_000).unwrap().octets,
        1000
    );
}
#[test]
fn counter_discontinuity_and_restart_do_not_masquerade_as_low_or_high_utilization() {
    let b = sample(0, 10000, 1000);
    let mut e = sample(10, 11000, 2000);
    e.discontinuity_ticks = 10050;
    assert!(
        rate(b, e, Width::Counter64, 1000)
            .unwrap_err()
            .contains("discontinuity")
    );
    e.discontinuity_ticks = 0;
    e.uptime_ticks = 200;
    assert!(
        rate(b, e, Width::Counter64, 1000)
            .unwrap_err()
            .contains("uptime")
    );
}
#[test]
fn time_ticks_wrap_is_distinct_from_restart_in_supplied_exact_interval_samples() {
    let r = rate(
        sample(0, u32::MAX - 999, 100),
        sample(20, 1000, 200),
        Width::Counter64,
        1000,
    )
    .unwrap();
    assert_eq!(r.octets, 100);
}
#[test]
fn wrong_intervals_widths_and_capacity_cannot_create_a_fabricated_rate() {
    let b = sample(10, 1000, 0);
    for e in [
        sample(10, 1000, 10),
        sample(9, 900, 10),
        sample(3611, 361100, 10),
    ] {
        assert!(rate(b, e, Width::Counter64, 1000).is_err())
    }
    assert!(
        rate(sample(0, 0, 0), sample(1, 100, 1000), Width::Counter64, 100)
            .unwrap_err()
            .contains("capacity")
    );
    assert!(
        rate(
            sample(0, 0, 1u64 << 32),
            sample(1, 100, 1u64 << 32),
            Width::Counter32,
            100
        )
        .unwrap_err()
        .contains("width")
    );
    assert!(rate(sample(0, 0, 0), sample(1, 100, 0), Width::Counter64, 0).is_err());
}

#[test]
fn source_text_and_collection_bounds_are_enforced_during_deserialization() {
    let mut raw = serde_json::to_value(agent()).unwrap();
    let item = raw["interfaces"][0].clone();
    raw["interfaces"] = serde_json::Value::Array(vec![item; 13]);
    assert!(serde_json::from_value::<Agent>(raw).is_err());
    let mut raw = serde_json::to_value(agent()).unwrap();
    raw["interfaces"][0]["description"] = serde_json::json!("x".repeat(65));
    assert!(serde_json::from_value::<Agent>(raw).is_err());
    assert!(serde_json::from_value::<Oid>(serde_json::json!(vec![1u32; 33])).is_err());
    let raw =
        serde_json::json!({"kind":"link","description":"x".repeat(65),"rule":"unplanned_down"});
    assert!(serde_json::from_value::<super::monitor::Plan>(raw).is_err());
}
