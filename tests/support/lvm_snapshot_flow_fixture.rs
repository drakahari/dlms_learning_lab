//! Fixed disposable, raw-record authoring fixture. Never bundled as a lab.
#![allow(dead_code)]
use dlms_learning_lab::{
    lvm::{self, MIB},
    lvm_workspace::{self as work, snapshot as goal, snapshot_flow::*},
    script_teaching::Concept,
};
#[path = "lvm_snapshot_fixture.rs"]
mod storage_fixture;
use storage_fixture::initial;
pub fn command(s: &str) -> Operation {
    Operation::Command { text: s.into() }
}
pub fn context() -> Context {
    Context {
        plan: goal::Plan {
            origins: vec![goal::Origin {
                volume: "work/report".into(),
                records: vec![goal::Record {
                    record: "row0".into(),
                    expected: goal::Value::Text {
                        contents: "edited-0".into(),
                    },
                }],
            }],
            snapshots: vec![goal::Snapshot {
                origin: "work/report".into(),
                name: "saved".into(),
                minimum_bytes: MIB,
                maximum_bytes: 2 * MIB,
                chunk_bytes: Some(128 * 1024),
                records: (0..8)
                    .map(|i| goal::Record {
                        record: format!("row{i}"),
                        expected: goal::Value::Original,
                    })
                    .collect(),
            }],
            retire: vec![],
            reserves: vec![goal::Reserve {
                group: "work".into(),
                minimum_bytes: 5 * MIB,
            }],
        },
        updates: vec![
            Update {
                label: "New report".into(),
                explanation: "The simulated application changes one record after the saved point."
                    .into(),
                writes: vec![Change {
                    origin: "work/report".into(),
                    record: "row0".into(),
                    contents: "edited-0".into(),
                }],
            },
            Update {
                label: "Another update".into(),
                explanation:
                    "This writes the remaining distinct chunks and can exhaust a small COW store."
                        .into(),
                writes: (1..8)
                    .map(|i| Change {
                        origin: "work/report".into(),
                        record: format!("row{i}"),
                        contents: format!("edited-{i}"),
                    })
                    .collect(),
            },
        ],
    }
}
pub fn reference() -> Vec<Operation> {
    vec![
        command("lvcreate -s -L 1M -n saved -c 128 work/report"),
        Operation::ApplyUpdate { index: 0 },
    ]
}
pub fn passes(c: &Context, source: &lvm::State, s: &lvm::State) -> bool {
    c.checks(source, s).iter().all(|r| r.0)
}
pub fn solve(c: &Context, source: &lvm::State) -> lvm::State {
    let mut s = source.clone();
    for op in reference() {
        op.apply(c, source, &mut s).unwrap();
    }
    s
}
pub fn practice() -> Practice {
    Practice {
        context: context(),
        worked: reference()
            .into_iter()
            .map(|operation| Step {
                operation,
                explanation: "Run the operation, then inspect the current and saved records."
                    .into(),
            })
            .collect(),
        experiment: command("lvcreate -s -L 1M -n extra -c 128 work/report"),
        repair: vec![command("lvremove -y work/extra")],
    }
}

fn separate<T: serde::Serialize + serde::de::DeserializeOwned>(input: &T) -> T {
    fn text(s: &str) -> String {
        if s == "work" {
            "sample".into()
        } else if s == "/dev/store" {
            "/dev/sample".into()
        } else if s == "/dev/foreign" {
            "/dev/sample-foreign".into()
        } else if s.starts_with("work/") {
            s.replacen("work/", "sample/", 1)
        } else if s.starts_with("record-") {
            s.replacen("record-", "sample-", 1)
        } else if s.starts_with("edited-") {
            s.replacen("edited-", "worked-", 1)
        } else {
            s.into()
        }
    }
    fn visit(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::String(s) => *s = text(s),
            serde_json::Value::Array(a) => {
                for x in a {
                    visit(x)
                }
            }
            serde_json::Value::Object(o) => {
                let old = std::mem::take(o);
                for (k, mut v) in old {
                    visit(&mut v);
                    o.insert(text(&k), v);
                }
            }
            _ => {}
        }
    }
    let mut v = serde_json::to_value(input).unwrap();
    visit(&mut v);
    serde_json::from_value(v).unwrap()
}
pub fn definition() -> work::Definition {
    let mut practice = separate(&practice());
    // Command source is a literal, not a canonical volume-only string.
    for s in &mut practice.worked {
        if let Operation::Command { text } = &mut s.operation {
            *text = text.replace("work/", "sample/");
        }
    }
    if let Operation::Command { text } = &mut practice.experiment {
        *text = text.replace("work/", "sample/");
    }
    for op in &mut practice.repair {
        if let Operation::Command { text } = op {
            *text = text.replace("work/", "sample/");
        }
    }
    work::Definition{initial:initial(),targets:vec![],plan:None,retirement:None,availability:None,renaming:None,
        mount_task:None,config_scan_task:None,device_reference_task:None,snapshot_task:Some(Task{context:context(),reference:reference()}),
        lesson:work::Lesson{
            prerequisite_note:"Use the supplied linear LVM facts; no real devices are touched.".into(),
            concepts:vec![Concept{title:"A point in time".into(),text:"A saved view keeps the old record when its origin changes. Its COW reservation is separate physical storage.".into()}],
            description:"Create a saved view before applying the supplied update. Compare the old and current record.".into(),
            fixture:separate(&initial()),targets:vec![],plan:None,retirement:None,availability:None,renaming:None,
            worked:vec![],mount_practice:None,config_scan_practice:None,device_reference_practice:None,snapshot_practice:Some(practice),try_change:"Add an extra COW view, check, remove only that extra view and retry.".into(),experiment:String::new(),repair:String::new()}}
}
