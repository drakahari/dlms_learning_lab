use dlms_learning_lab::{json_inventory as inventory, switching as ethernet};
use std::collections::{BTreeMap, BTreeSet};

const DOCUMENT: &str = r#"{
  "note": null,
  "switches": [{"name":"access","ports":[
    {"name":"p1","up":true,"vlan":10},
    {"name":"p2","up":true,"vlan":10},
    {"name":"p3","up":true,"vlan":20},
    {"name":"p4","up":true,"vlan":20}
  ]}]
}"#;

fn fixture() -> inventory::Spec {
    let endpoints = ["staff-a", "staff-b", "guest-a", "guest-b"];
    let fabric = ethernet::Fabric {
        switches: BTreeMap::from([(
            "access".into(),
            ethernet::Switch {
                label: "Access switch".into(),
                vlans: BTreeSet::from([10, 20, 30]),
                ports: (1..=4)
                    .map(|i| {
                        (
                            format!("p{i}"),
                            ethernet::Port {
                                up: true,
                                mode: ethernet::Mode::Access { vlan: 10 },
                            },
                        )
                    })
                    .collect(),
            },
        )]),
        hosts: endpoints
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    ethernet::Host {
                        label: name.into(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: endpoints
            .into_iter()
            .enumerate()
            .map(|(i, endpoint)| ethernet::Cable {
                id: format!("c{i}"),
                a: ethernet::Jack::Host {
                    host: endpoint.into(),
                },
                b: ethernet::Jack::Switch {
                    switch: "access".into(),
                    port: format!("p{}", i + 1),
                },
                up: true,
            })
            .collect(),
    };
    inventory::Spec {
        fabric,
        editable: BTreeMap::from([("access".into(), (1..=4).map(|i| format!("p{i}")).collect())]),
    }
}

#[test]
fn a_real_parsed_document_controls_actual_ethernet_visibility() {
    let spec = fixture();
    let before = spec.clone();
    let result = spec.apply(DOCUMENT).unwrap();
    assert!(result.visibility("staff-a", "staff-b").unwrap().mutual());
    assert!(result.visibility("guest-a", "guest-b").unwrap().mutual());
    assert!(result.visibility("staff-a", "guest-a").unwrap().isolated());
    let merged = spec
        .apply(&DOCUMENT.replace("\"vlan\":20", "\"vlan\":10"))
        .unwrap();
    assert!(merged.visibility("staff-a", "guest-a").unwrap().mutual());
    assert_eq!(spec, before);
}

#[test]
fn equivalent_object_and_record_order_whitespace_and_escaping_work() {
    let spec = fixture();
    let reordered = r#"{"switches":[{"ports":[
      {"vlan":20,"up":true,"name":"p4"},
      {"vlan":20,"name":"p3","up":true},
      {"up":true,"name":"p2","vlan":10},
      {"name":"p\u0031","vlan":10,"up":true}
    ],"name":"access"}],"note":"another annotation"}"#;
    assert_eq!(
        spec.apply(reordered).unwrap(),
        spec.apply(DOCUMENT).unwrap()
    );
}

#[test]
fn syntax_and_typed_field_errors_are_distinct_and_never_apply_partial_changes() {
    let spec = fixture();
    let before = spec.clone();
    let malformed = DOCUMENT.replace("\"note\": null,", "note: null,");
    assert!(
        spec.apply(&malformed)
            .unwrap_err()
            .starts_with("JSON syntax")
    );
    for malformed in [
        DOCUMENT.replacen("true", "truu", 1),
        DOCUMENT.replacen("\"vlan\":10", "\"vlan\":1e", 1),
    ] {
        assert!(
            spec.apply(&malformed)
                .unwrap_err()
                .starts_with("JSON syntax")
        );
    }
    for (bad, field) in [
        (DOCUMENT.replacen("\"up\":true", "\"up\":\"true\"", 1), "up"),
        (
            DOCUMENT.replacen("\"vlan\":10", "\"vlan\":\"10\"", 1),
            "vlan",
        ),
        (DOCUMENT.replacen("\"vlan\":10", "\"vlan\":10.5", 1), "vlan"),
        (DOCUMENT.replacen("\"vlan\":10", "\"vlan\":0", 1), "vlan"),
        (
            DOCUMENT.replacen("\"vlan\":10", "\"vlan\":10.0000000000000001", 1),
            "vlan",
        ),
        (DOCUMENT.replacen("\"vlan\":10", "\"vlan\":1e1", 1), "vlan"),
    ] {
        let error = spec.apply(&bad).unwrap_err();
        assert!(
            error.starts_with("Document field error") && error.contains(field),
            "{error}"
        );
        assert_eq!(spec, before);
    }
    let unknown = DOCUMENT.replace("\"note\": null,", "\"command\":\"touch ../../host\",");
    assert!(spec.apply(&unknown).unwrap_err().contains("unknown field"));
    assert_eq!(spec, before);
}

#[test]
fn false_is_a_real_boolean_that_breaks_a_required_live_path() {
    let spec = fixture();
    let result = spec
        .apply(&DOCUMENT.replacen("\"up\":true", "\"up\":false", 1))
        .unwrap();
    let required = ethernet::Requirement::Visibility {
        label: "Staff path".into(),
        from: "staff-a".into(),
        to: "staff-b".into(),
        visible: true,
    };
    let check = required.check(&result);
    assert!(!check.0 && check.1.contains("down"));
    assert!(result.visibility("guest-a", "guest-b").unwrap().mutual());
}

#[test]
fn duplicates_missing_records_unlisted_ports_and_reserved_links_fail_honestly() {
    let spec = fixture();
    let before = spec.clone();
    let duplicate_field = DOCUMENT.replace("\"name\":\"p1\"", "\"name\":\"p1\",\"name\":\"p1\"");
    assert!(
        spec.apply(&duplicate_field)
            .unwrap_err()
            .contains("duplicate field")
    );
    let duplicate_port = DOCUMENT.replace("\"name\":\"p4\"", "\"name\":\"p3\"");
    assert!(
        spec.apply(&duplicate_port)
            .unwrap_err()
            .contains("Duplicate port")
    );
    let value: serde_json::Value = serde_json::from_str(DOCUMENT).unwrap();
    let mut duplicate_switch = value.clone();
    duplicate_switch["switches"]
        .as_array_mut()
        .unwrap()
        .push(value["switches"][0].clone());
    assert!(
        spec.apply(&duplicate_switch.to_string())
            .unwrap_err()
            .contains("Duplicate switch")
    );
    let mut missing = value.clone();
    missing["switches"][0]["ports"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(
        spec.apply(&missing.to_string())
            .unwrap_err()
            .contains("Incomplete")
    );
    for name in ["uplink", "../../host", "unknown"] {
        assert!(
            spec.apply(&DOCUMENT.replace("\"p1\"", &format!("\"{name}\"")))
                .unwrap_err()
                .contains("outside this inventory")
        );
    }
    assert_eq!(spec, before);
}

#[test]
fn byte_record_note_and_source_limits_refuse_before_any_network_mutation() {
    let spec = fixture();
    assert!(
        spec.apply(&" ".repeat(inventory::MAX_DOCUMENT + 1))
            .is_err()
    );
    let mut value: serde_json::Value = serde_json::from_str(DOCUMENT).unwrap();
    value["note"] = "x".repeat(161).into();
    assert!(spec.apply(&value.to_string()).is_err());
    value["note"] = serde_json::Value::Null;
    let port = value["switches"][0]["ports"][0].clone();
    value["switches"][0]["ports"] = serde_json::Value::Array(vec![port; 7]);
    assert!(spec.apply(&value.to_string()).is_err());
    let mut invalid = spec.clone();
    invalid.editable.clear();
    assert!(invalid.validate().is_err());
    let mut invalid = spec.clone();
    invalid
        .editable
        .get_mut("access")
        .unwrap()
        .insert("uplink".into());
    assert!(invalid.validate().is_err());
}

fn cross_fixture() -> inventory::Spec {
    let names = [
        "staff-a",
        "guest-a",
        "monitor-a",
        "staff-b",
        "guest-b",
        "monitor-b",
    ];
    let switch = |label: &str| ethernet::Switch {
        label: label.into(),
        vlans: BTreeSet::from([10, 20, 99]),
        ports: (1..=4)
            .map(|i| {
                (
                    format!("p{i}"),
                    ethernet::Port {
                        up: true,
                        mode: if i == 4 {
                            ethernet::Mode::Trunk {
                                native: 99,
                                allowed: BTreeSet::from([10, 20, 99]),
                            }
                        } else {
                            ethernet::Mode::Access {
                                vlan: if i == 3 { 99 } else { 10 },
                            }
                        },
                    },
                )
            })
            .collect(),
    };
    let mut fabric = ethernet::Fabric {
        switches: BTreeMap::from([
            ("left".into(), switch("Left access switch")),
            ("right".into(), switch("Right access switch")),
        ]),
        hosts: names
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    ethernet::Host {
                        label: name.into(),
                        up: true,
                    },
                )
            })
            .collect(),
        cables: names
            .into_iter()
            .enumerate()
            .map(|(i, endpoint)| ethernet::Cable {
                id: format!("c{i}"),
                a: ethernet::Jack::Host {
                    host: endpoint.into(),
                },
                b: ethernet::Jack::Switch {
                    switch: if i < 3 { "left" } else { "right" }.into(),
                    port: format!("p{}", i % 3 + 1),
                },
                up: true,
            })
            .collect(),
    };
    fabric.cables.push(ethernet::Cable {
        id: "trunk".into(),
        a: ethernet::Jack::Switch {
            switch: "left".into(),
            port: "p4".into(),
        },
        b: ethernet::Jack::Switch {
            switch: "right".into(),
            port: "p4".into(),
        },
        up: true,
    });
    inventory::Spec {
        fabric,
        editable: ["left", "right"]
            .into_iter()
            .map(|name| (name.into(), (1..=3).map(|i| format!("p{i}")).collect()))
            .collect(),
    }
}
fn cross_document() -> String {
    serde_json::json!({"note":null,"switches":[
        {"name":"left","ports":[{"name":"p1","up":true,"vlan":10},{"name":"p2","up":true,"vlan":20},{"name":"p3","up":true,"vlan":99}]},
        {"name":"right","ports":[{"name":"p1","up":true,"vlan":10},{"name":"p2","up":true,"vlan":20},{"name":"p3","up":true,"vlan":99}]}
    ]}).to_string()
}

#[test]
fn nested_switch_records_drive_cross_switch_groups_and_preserve_the_real_trunk() {
    let spec = cross_fixture();
    spec.validate().unwrap();
    let before = spec.clone();
    let applied = spec.apply(&cross_document()).unwrap();
    for group in ["staff", "guest", "monitor"] {
        assert!(
            applied
                .visibility(&format!("{group}-a"), &format!("{group}-b"))
                .unwrap()
                .mutual()
        );
    }
    for (from, to) in [
        ("staff-a", "guest-b"),
        ("staff-a", "monitor-b"),
        ("guest-a", "monitor-b"),
    ] {
        assert!(applied.visibility(from, to).unwrap().isolated());
    }
    for name in ["left", "right"] {
        assert_eq!(
            applied.switches[name].ports["p4"],
            before.fabric.switches[name].ports["p4"]
        );
    }
    let mut wrong: serde_json::Value = serde_json::from_str(&cross_document()).unwrap();
    wrong["switches"][1]["ports"][0]["vlan"] = 20.into();
    let partial = spec.apply(&wrong.to_string()).unwrap();
    assert!(!partial.visibility("staff-a", "staff-b").unwrap().mutual());
    assert!(partial.visibility("guest-a", "guest-b").unwrap().mutual());
    assert!(
        partial
            .visibility("monitor-a", "monitor-b")
            .unwrap()
            .mutual()
    );
    assert!(partial.visibility("guest-a", "staff-b").unwrap().mutual());
    assert_eq!(spec, before);
}

#[test]
fn shutting_down_all_ports_never_proves_the_required_separation_policy() {
    let spec = cross_fixture();
    let mut value: serde_json::Value = serde_json::from_str(&cross_document()).unwrap();
    for switch in value["switches"].as_array_mut().unwrap() {
        for port in switch["ports"].as_array_mut().unwrap() {
            port["up"] = false.into();
        }
    }
    let fabric = spec.apply(&value.to_string()).unwrap();
    assert!(fabric.visibility("staff-a", "guest-b").unwrap().isolated());
    let requirement = ethernet::Requirement::Visibility {
        label: "Staff/guest separation".into(),
        from: "staff-a".into(),
        to: "guest-b".into(),
        visible: false,
    };
    assert!(!requirement.check(&fabric).0);
}
