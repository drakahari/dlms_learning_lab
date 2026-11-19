use dlms_learning_lab::{
    fs_integrity::{Inode, Volume},
    fs_integrity_terminal::{Disk, System},
    fs_integrity_workspace::{Case, Definition, Requirement, Task},
};
use std::collections::BTreeMap;
fn volume() -> Volume {
    Volume {
        version: 1,
        block_count: 32,
        inode_count: 32,
        files: BTreeMap::from([(
            12,
            Inode {
                content: "preserved report\n".into(),
                blocks: vec![8],
                links: 1,
            },
        )]),
        entries: BTreeMap::from([("report.txt".into(), 12)]),
        allocated_blocks: (0..9).collect(),
        allocated_inodes: (1..=12).collect(),
        free_blocks: 23,
        free_inodes: 20,
        mounted: true,
    }
}
fn task(practice: bool, recovery: bool) -> Task {
    let id = if practice {
        "integrity-example"
    } else {
        "integrity-assignment"
    };
    let device = if practice { "/dev/vdd1" } else { "/dev/vdb1" };
    let target = if practice {
        "/mnt/workshop"
    } else {
        "/mnt/data"
    };
    let other = if practice { "/dev/vde1" } else { "/dev/vdc1" };
    let other_target = if practice {
        "/mnt/example-notes"
    } else {
        "/mnt/notes"
    };
    let mut cases = vec![];
    for n in 0..2 {
        let mut v = volume();
        if n == 0 {
            v.allocated_blocks.remove(&8);
        } else {
            v.entries.insert("report-alias.txt".into(), 12);
            v.free_blocks = 30;
        }
        let mut requirements = vec![
            Requirement::Healthy {
                device: device.into(),
            },
            Requirement::Mounted {
                device: device.into(),
            },
            Requirement::Preserved {
                device: device.into(),
                name: "report.txt".into(),
            },
            Requirement::Unchanged {
                device: other.into(),
            },
        ];
        if recovery {
            let inode = if n == 0 { 14 } else { 20 };
            v.files.insert(
                inode,
                Inode {
                    content: format!("retained orphan case {n}\n"),
                    blocks: vec![10],
                    links: 1,
                },
            );
            if n == 1 {
                v.entries.insert(format!("lost+found/#{inode}"), 12);
            }
            requirements.push(Requirement::Recovered {
                device: device.into(),
                inode,
            });
        }
        let initial = System {
            version: 1,
            disks: vec![
                Disk {
                    device: device.into(),
                    mountpoint: target.into(),
                    volume: v,
                },
                Disk {
                    device: other.into(),
                    mountpoint: other_target.into(),
                    volume: volume(),
                },
            ],
        };
        cases.push(Case {
            id: format!("{id}-case-{n}"),
            label: format!("Supplied graph {}", n + 1),
            initial,
            requirements,
        });
    }
    Task{id:id.into(),instructions:"Repair the supplied virtual filesystem, preserve its report and the separate notes volume, verify consistency and restore the mount. Recover retained unnamed data when required.".into(),cases}
}
pub fn recipe(task: &Task, yes: bool) -> Vec<String> {
    let d = &task.cases[0].initial.disks[0];
    vec![
        format!("umount {}", d.mountpoint),
        format!("e2fsck -fn {}", d.device),
        format!("e2fsck -f{} {}", if yes { "y" } else { "p" }, d.device),
        format!("e2fsck -fn {}", d.device),
        format!("mount {} {}", d.device, d.mountpoint),
    ]
}
pub fn definition(recovery: bool) -> Definition {
    let assignment = task(false, recovery);
    let example = task(true, recovery);
    let example_commands = recipe(&example, recovery);
    Definition {
        assignment,
        example,
        example_commands,
    }
}
