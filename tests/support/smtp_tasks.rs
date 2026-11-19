//! Synthetic source-owned grading tasks; no catalog registration or authored lab credit.
#[path = "smtp_fixture.rs"]
mod world;
use dlms_learning_lab::{
    smtp_queue::{Facts, Sample},
    smtp_workspace::{Action, Definition, Expected, Probe, Task},
};
use std::collections::{BTreeMap, BTreeSet};
fn delivery(node: &str, user: &str) -> Expected {
    Expected::Delivery {
        node: node.into(),
        mailboxes: BTreeSet::from([user.into()]),
    }
}
fn task() -> Task {
    let (w, good) = world::fixture();
    let mut bad = good.clone();
    bad.insert(
        "relay".into(),
        good["relay"].replace("[delivery.lab.test]", "[gate.lab.test]"),
    );
    Task {
        facts: Facts {
            world: w,
            initial_files: bad.clone(),
            initial_active: bad,
            samples: BTreeMap::from([
                (
                    "notice".into(),
                    Sample {
                        entry: "gate".into(),
                        sender: Some("service@sender.test".into()),
                        recipient: "nora@staff.lab.test".into(),
                        subject: "Notice".into(),
                        body: "Source-owned harmless input.".into(),
                    },
                ),
                (
                    "receipt".into(),
                    Sample {
                        entry: "gate".into(),
                        sender: None,
                        recipient: "visitor@outside.test".into(),
                        subject: "Receipt".into(),
                        body: "A different sample.".into(),
                    },
                ),
            ]),
        },
        editable: BTreeMap::from([("relay".into(), BTreeSet::from(["relayhost".into()]))]),
        probes: vec![
            Probe {
                id: "incoming-staff".into(),
                label: "Outside client reaches the staff mailbox".into(),
                entry: "gate".into(),
                client: world::ip("203.0.113.9"),
                recipient: "nora@staff.lab.test".into(),
                expected: delivery("delivery", "nora"),
            },
            Probe {
                id: "no-outside-relay".into(),
                label: "Outside client cannot relay outward".into(),
                entry: "gate".into(),
                client: world::ip("203.0.113.9"),
                recipient: "visitor@outside.test".into(),
                expected: Expected::Reject {
                    node: "gate".into(),
                },
            },
        ],
        queued: BTreeMap::from([
            ("notice".into(), delivery("delivery", "nora")),
            ("receipt".into(), delivery("outside", "visitor")),
        ]),
    }
}
fn transform(value: &mut serde_json::Value) {
    fn string(s: &str) -> String {
        if ["gate", "relay", "delivery", "outside"].contains(&s) {
            return format!("guide-{s}");
        }
        let mut s = s.to_owned();
        for node in ["delivery", "outside", "relay", "gate"] {
            s = s.replace(
                &format!("{node}.lab.test"),
                &format!("guide-{node}.guide.test"),
            );
        }
        s.replace("staff.lab.test", "staff.guide.test")
            .replace("outside.test", "outside.guide.test")
            .replace("192.0.2.", "198.51.100.")
    }
    match value {
        serde_json::Value::String(s) => *s = string(s),
        serde_json::Value::Array(a) => a.iter_mut().for_each(transform),
        serde_json::Value::Object(map) => {
            let old = std::mem::take(map);
            for (key, mut v) in old {
                transform(&mut v);
                map.insert(string(&key), v);
            }
        }
        _ => {}
    }
}
pub fn definition() -> Definition {
    let a = task();
    let (_, good) = world::fixture();
    let reference = vec![
        Action::Edit {
            node: "relay".into(),
            text: good["relay"].clone(),
        },
        Action::Save {
            node: "relay".into(),
        },
        Action::Reload {
            node: "relay".into(),
        },
        Action::Flush {
            node: "gate".into(),
        },
        Action::Check,
    ];
    let mut example = serde_json::to_value(&a).unwrap();
    transform(&mut example);
    let example = serde_json::from_value(example).unwrap();
    let mut steps = serde_json::to_value(&reference).unwrap();
    transform(&mut steps);
    let steps = serde_json::from_value(steps).unwrap();
    Definition {
        version: 1,
        source_id: "test-mail-routing".into(),
        assignment: a,
        example,
        reference,
        example_steps: steps,
    }
}
