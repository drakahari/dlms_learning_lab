use dlms_learning_lab::{removable_mount as m, removable_workspace as w, storage as s};
use std::collections::BTreeMap;
fn uuid(n: u8) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}
pub fn configuration(base: u8) -> String {
    format!(
        "UUID={} /project xfs rw,auto 0 2\nUUID={} /media/audit ext4 ro,auto,nofail 0 0\n",
        uuid(base),
        uuid(base + 1)
    )
}
fn environment(base: u8, present: &[&str]) -> m::Environment {
    m::Environment {
        inventory: s::Inventory::new(
            BTreeMap::from([
                (
                    "data".into(),
                    s::Volume {
                        uuid: uuid(base),
                        filesystem: s::Filesystem::Xfs,
                    },
                ),
                (
                    "media".into(),
                    s::Volume {
                        uuid: uuid(base + 1),
                        filesystem: s::Filesystem::Ext4,
                    },
                ),
            ]),
            BTreeMap::from([
                ("/dev/sim-a".into(), "data".into()),
                ("/dev/sim-b".into(), "media".into()),
            ]),
            vec!["/project".into(), "/media/audit".into()],
        )
        .unwrap(),
        present: present.iter().map(|s| (*s).into()).collect(),
    }
}
pub fn task(id: &str, base: u8) -> w::Task {
    let mut cases = vec![];
    for (n, present) in [&["data", "media"][..], &["data"][..], &["media"][..]]
        .into_iter()
        .enumerate()
    {
        let mut goals = vec![
            w::Requirement::Policy {
                target: "/project".into(),
                mode: m::Dependency::Required,
            },
            w::Requirement::Policy {
                target: "/media/audit".into(),
                mode: m::Dependency::Wanted,
            },
            w::Requirement::EntriesOnly {
                targets: vec!["/project".into(), "/media/audit".into()],
            },
            w::Requirement::RequiredAttempts { satisfied: n != 2 },
        ];
        for (target, volume, ro) in [("/project", "data", false), ("/media/audit", "media", true)] {
            goals.push(if present.contains(&volume) {
                w::Requirement::Mounted {
                    target: target.into(),
                    volume: volume.into(),
                    read_only: ro,
                }
            } else {
                w::Requirement::NotMounted {
                    target: target.into(),
                }
            });
        }
        cases.push(w::Case {
            id: format!("sample-{n}"),
            label: [
                "Both media available",
                "Optional media absent",
                "Required project storage absent",
            ][n]
                .into(),
            environment: environment(base, present),
            requirements: goals,
        });
    }
    w::Task{id:id.into(),instructions:"Keep the required project automatic/read-write and removable audit media automatic/optional/read-only. Use the two intended filesystems only; report absent required storage as a problem.".into(),starter:configuration(base).replace("rw,auto","rw,auto,nofail").replace("ro,auto,nofail","ro,noauto"),cases}
}
pub fn definition() -> w::Definition {
    w::Definition {
        initial: task("optional-assignment", 10),
        example: task("optional-practice", 30),
        example_worked: configuration(30),
    }
}
