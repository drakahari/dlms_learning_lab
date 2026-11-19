#![allow(dead_code)]
use dlms_learning_lab::{raid_assembly as c, raid_assembly_workspace as w, raid1 as r};
use std::collections::BTreeMap;
pub fn uuid(n: u8) -> c::Uuid {
    c::Uuid::parse(&format!("{n:032x}")).unwrap()
}
pub fn activity(example: bool) -> w::Activity {
    let start = if example { 2 } else { 0 };
    let mut arrays = BTreeMap::new();
    let mut devices = BTreeMap::new();
    let mut identities = vec![];
    for j in 0..2 {
        let name = format!("/dev/md{}", start + j);
        let u = uuid(start + j + 1);
        let content = vec![
            r::Segment {
                file: "/records.txt".into(),
                index: 0,
                text: format!("Original dataset {start}-{j}\n"),
            },
            r::Segment {
                file: "/records.txt".into(),
                index: 1,
                text: "Retain this second sample segment.\n".into(),
            },
        ];
        let mut members = vec![];
        for slot in 0..2 {
            let dev = format!("/dev/vd{}1", char::from(b'a' + start * 2 + j * 2 + slot));
            devices.insert(
                dev.clone(),
                r::Device {
                    raw_sectors: 8192,
                    usable_sectors: 4096,
                    available: true,
                    read_only: false,
                    protected: j == 1,
                    media: r::Media::Md {
                        array: name.clone(),
                    },
                    content: content.clone(),
                },
            );
            members.push(r::Member {
                device: dev,
                slot: Some(slot),
                role: r::Role::Synced,
            });
        }
        arrays.insert(
            name.clone(),
            r::Array {
                component_sectors: 4096,
                members,
            },
        );
        identities.push(c::Identity {
            array: c::Name(name),
            uuid: u,
        });
    }
    let context = c::Context {
        replicas: r::State {
            version: 1,
            devices,
            arrays,
        },
        identities,
    };
    let pair = context.components(&uuid(start + 1)).unwrap();
    let saved = format!(
        "DEVICE {}\nARRAY /dev/md{start} UUID={}\nARRAY /dev/md{} UUID={}\n",
        pair[0],
        uuid(99).display(),
        start + 1,
        uuid(start + 2).display()
    );
    w::Activity {
        context,
        initial: c::Runtime {
            version: 1,
            saved: c::Text(saved),
            live: vec![c::Binding {
                target: c::Name(format!("/dev/md{}", start + 1)),
                uuid: uuid(start + 2),
            }],
        },
        requirements: (0..2)
            .map(|j| w::Requirement {
                target: c::Name(format!("/dev/md{}", start + j)),
                uuid: uuid(start + j + 1),
                file: c::Name("/records.txt".into()),
            })
            .collect(),
    }
}
pub fn correct(a: &w::Activity) -> String {
    format!(
        "DEVICE {}\n{}\n",
        a.context
            .replicas
            .devices
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        a.requirements
            .iter()
            .map(|t| format!("ARRAY {} UUID={}", t.target.0, t.uuid.display()))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
pub fn save(a: &w::Activity) -> w::Operation {
    w::Operation::Save {
        text: c::Text(correct(a)),
    }
}
pub fn cmd(s: &str) -> w::Operation {
    w::Operation::Command { text: s.into() }
}
pub fn step(op: w::Operation) -> w::Step {
    w::Step {
        operation: op,
        explanation:
            "Inspect the actual result and compare saved policy with active original data.".into(),
        status: 0,
    }
}
pub fn definition() -> w::Definition {
    let a = activity(false);
    let e = activity(true);
    let reference = vec![
        step(save(&a)),
        step(cmd("mdadm --assemble --scan /dev/md0")),
        step(cmd("cat /proc/mdstat")),
    ];
    let example_steps = vec![
        step(cmd("mdadm --examine /dev/vde1")),
        step(save(&e)),
        step(cmd("mdadm --assemble --scan /dev/md2")),
        step(cmd("mdadm --detail /dev/md2")),
    ];
    w::Definition {
        version: 1,
        source_id: "assembly-fixture".into(),
        assignment: a,
        example: e,
        reference,
        example_steps,
        experiment: step(cmd("mdadm --stop /dev/md2")),
        repair: vec![step(cmd("mdadm --assemble --scan /dev/md2"))],
    }
}
pub fn solved(d: &w::Definition) -> w::Session {
    let mut s = w::Session::new(d).unwrap();
    for step in &d.reference {
        assert_eq!(
            s.apply(d, &step.operation, false).unwrap().status,
            step.status
        );
    }
    s.run(d, w::CHECK, false).unwrap();
    s
}
