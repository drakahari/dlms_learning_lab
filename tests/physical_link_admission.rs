//! Embedded streaming decode gates: poisoned tails must not be consumed before
//! scalar/cardinality/duplicate refusal. These are technical, not authored labs.
use dlms_learning_lab::physical_link as core;
#[path = "support/physical_assignment_fixture.rs"]
mod fixture;
fn error<T: serde::de::DeserializeOwned + std::fmt::Debug>(raw: &str, expected: &str) {
    let e = serde_json::from_str::<T>(raw).unwrap_err().to_string();
    assert!(
        e.contains(expected),
        "Expected early {expected:?}, got {e:?}"
    );
    assert!(
        !e.contains("expected value"),
        "A poisoned later value was read: {e}"
    );
}
fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap()
}
#[test]
fn embedded_core_identity_and_label_fields_refuse_before_poisoned_later_values() {
    for value in [
        "x".repeat(49),
        "Uppercase".into(),
        "bad/id".into(),
        "".into(),
    ] {
        let v = quoted(&value);
        error::<core::PortSpec>(
            &format!(r#"{{"device":{v},"label":POISON}}"#),
            "identifier before body",
        );
        for field in ["a", "b"] {
            error::<core::LinkSpec>(
                &format!(r#"{{"{field}":{v},"required_span_m":POISON}}"#),
                "identifier before body",
            );
        }
    }
    for value in [
        "x".repeat(121),
        "ü".repeat(61),
        " ".into(),
        "bad\nlabel".into(),
    ] {
        let v = quoted(&value);
        error::<core::Cable>(
            &format!(r#"{{"label":{v},"kind":POISON}}"#),
            "label before owned field copy",
        );
        error::<core::PortSpec>(
            &format!(r#"{{"label":{v},"device":POISON}}"#),
            "label before owned field copy",
        );
        error::<core::SuppliedReading>(
            &format!(r#"{{"note":{v},"counters":POISON}}"#),
            "label before owned field copy",
        );
    }
    // The bound applies to decoded bytes, including escapes; not JSON source
    // character count. Serde's bounded caller/raw input still owns parser scratch.
    let escaped = "\\u0061".repeat(121);
    error::<core::Cable>(
        &format!(r#"{{"label":"{escaped}","kind":POISON}}"#),
        "label before owned field copy",
    );
}
#[test]
fn cable_map_values_and_action_identifiers_refuse_in_nested_decode_without_kind_first() {
    let long = quoted(&"x".repeat(49));
    error::<core::Configuration>(
        &format!(r#"{{"cables":{{"link":{long},"next":POISON}},"ports":{{}}}}"#),
        "identifier before body",
    );
    // A tag-derived Action would consume POISON into general Content before
    // reaching field visitors. The streaming replacement rejects each ID first.
    for field in ["port", "link", "cable"] {
        error::<core::Action>(
            &format!(r#"{{"{field}":{long},"kind":POISON}}"#),
            "identifier before body",
        );
    }
    error::<core::Action>(
        &format!(
            r#"{{"configuration":{{"cables":{{"link":{long}}},"ports":POISON}},"kind":"Configure"}}"#
        ),
        "identifier before body",
    );
}
#[test]
fn bounded_map_rejects_duplicate_excess_and_oversize_keys_before_their_value() {
    error::<core::Configuration>(
        r#"{"cables":{"link":"wire","link":POISON},"ports":{}}"#,
        "map limit/duplicate before value",
    );
    let entries = (0..core::MAX_LINKS)
        .map(|i| format!(r#""link-{i}":"wire-{i}""#))
        .collect::<Vec<_>>()
        .join(",");
    error::<core::Configuration>(
        &format!(r#"{{"cables":{{{entries},"extra":POISON}},"ports":{{}}}}"#),
        "map limit/duplicate before value",
    );
    let key = quoted(&"x".repeat(49));
    error::<core::Configuration>(
        &format!(r#"{{"cables":{{{key}:POISON}},"ports":{{}}}}"#),
        "identifier before body",
    );
}
#[test]
fn internally_tagged_configuration_collections_now_refuse_before_poisoned_body() {
    let abilities = r#"{"rate":"M10","duplex":"Half"},{"rate":"M10","duplex":"Full"},{"rate":"M100","duplex":"Half"},{"rate":"M100","duplex":"Full"},{"rate":"M1000","duplex":"Full"}"#;
    let raw = format!(r#"{{"advertise":[{abilities},POISON],"kind":"Auto"}}"#);
    error::<core::Negotiation>(&raw, "Capability limit before body");
    error::<core::Action>(
        &format!(
            r#"{{"configuration":{{"negotiation":{raw},"admin_up":true,"profile":"CopperRj45"}},"port":"uplink","kind":"ConfigurePort"}}"#
        ),
        "Capability limit before body",
    );
    error::<core::Negotiation>(
        r#"{"advertise":[{"rate":"M10","duplex":"Half"},{"rate":"M10","duplex":"Half"},POISON],"kind":"Auto"}"#,
        "Duplicate capability",
    );
}
#[test]
fn repeated_unknown_and_kind_fields_refuse_without_reading_their_payload() {
    error::<core::Action>(
        r#"{"port":"a","port":POISON,"kind":"ClearCounters"}"#,
        "Duplicate action field before value",
    );
    error::<core::Action>(
        r#"{"kind":"Configure","configuration":{"cables":{},"cables":POISON}}"#,
        "Duplicate configuration field before value",
    );
    error::<core::Condition>(
        r#"{"kind":"Clean","kind":POISON}"#,
        "Duplicate condition field before value",
    );
    error::<core::Negotiation>(
        r#"{"kind":"Auto","kind":POISON}"#,
        "Duplicate negotiation field before value",
    );
    let unknown = "x".repeat(4096);
    for raw in [
        format!(r#"{{"kind":"{unknown}","port":POISON}}"#),
        format!(r#"{{"{unknown}":POISON}}"#),
    ] {
        let e = serde_json::from_str::<core::Action>(&raw)
            .unwrap_err()
            .to_string();
        assert!(e.contains("before"));
        assert!(
            e.len() < 160,
            "Diagnostics must not copy an arbitrary unknown key/tag"
        );
    }
}
#[test]
fn all_supported_action_condition_and_negotiation_variants_roundtrip_in_any_field_order() {
    let f = fixture::fixture("admission");
    for config in [&f.reference, &f.alternate, &f.partial, &f.neighbor_broken] {
        let raw = serde_json::to_vec(config).unwrap();
        assert_eq!(
            serde_json::from_slice::<core::Configuration>(&raw).unwrap(),
            *config
        );
    }
    let config = f.reference;
    let port = config.ports["admission-left"].clone();
    let actions = [
        core::Action::Configure {
            configuration: config,
        },
        core::Action::ConfigurePort {
            port: "admission-left".into(),
            configuration: port.clone(),
        },
        core::Action::SelectCable {
            link: "admission-uplink".into(),
            cable: "admission-clean".into(),
        },
        core::Action::ClearCounters {
            port: "admission-left".into(),
        },
        core::Action::Probe {
            link: "admission-uplink".into(),
            a_frames: 3,
            b_frames: 4,
        },
    ];
    for action in actions {
        let value = serde_json::to_value(&action).unwrap();
        // Value sorts keys, placing configuration/cable before kind. No new
        // tag-first wire restriction is permitted by the streaming visitor.
        let raw = serde_json::to_vec(&value).unwrap();
        assert_eq!(
            serde_json::from_slice::<core::Action>(&raw).unwrap(),
            action
        );
        assert_eq!(core::Action::from_json(&raw).unwrap(), action);
    }
    for value in [
        core::Condition::Clean,
        core::Condition::Disconnected,
        core::Condition::ReceiveImpairment { every: 7 },
    ] {
        let raw = serde_json::to_vec(&serde_json::to_value(value).unwrap()).unwrap();
        assert_eq!(
            serde_json::from_slice::<core::Condition>(&raw).unwrap(),
            value
        );
    }
    for value in [
        port.negotiation,
        core::Negotiation::Fixed {
            ability: core::Ability {
                rate: core::Rate::M100,
                duplex: core::Duplex::Full,
            },
        },
    ] {
        let raw = serde_json::to_vec(&serde_json::to_value(&value).unwrap()).unwrap();
        assert_eq!(
            serde_json::from_slice::<core::Negotiation>(&raw).unwrap(),
            value
        );
    }
    let mut cable = f.definition.initial.topology().cables["admission-clean"].clone();
    cable.label = "ü".repeat(60);
    assert_eq!(
        serde_json::from_slice::<core::Cable>(&serde_json::to_vec(&cable).unwrap()).unwrap(),
        cable
    );
}
#[test]
fn wrong_mixed_missing_and_extra_action_fields_remain_honest_errors() {
    for raw in [
        r#"{"kind":"ClearCounters","port":"a","link":"b"}"#,
        r#"{"kind":"Probe","link":"a","a_frames":1}"#,
        r#"{"kind":"Configure","configuration":{"admin_up":true,"profile":"CopperRj45","negotiation":{"kind":"Fixed","ability":{"rate":"M100","duplex":"Full"}}}}"#,
        r#"{"kind":"ConfigurePort","port":"a","configuration":{"ports":{},"cables":{}}}"#,
        r#"{"kind":"Configure","configuration":{"ports":{},"cables":{},"admin_up":true}}"#,
    ] {
        assert!(
            serde_json::from_str::<core::Action>(raw).is_err(),
            "Wrong fields admitted: {raw}"
        );
    }
    for raw in [
        r#"{"kind":"Clean","every":4}"#,
        r#"{"kind":"ReceiveImpairment"}"#,
    ] {
        assert!(serde_json::from_str::<core::Condition>(raw).is_err());
    }
    assert!(
        serde_json::from_str::<core::Negotiation>(r#"{"kind":"Fixed","advertise":[]}"#).is_err()
    );
}
