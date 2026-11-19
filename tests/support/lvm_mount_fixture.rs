//! Synthetic capability fixture; not registered curriculum or personal progress.
use dlms_learning_lab::{
    lvm,
    lvm_workspace::{mount_flow as f, rename},
};
pub const UUID_A: &str = "12345678-0000-4000-8000-000000000001";
pub fn world() -> lvm::State {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap()
}
pub fn policy(source: &str) -> String {
    format!("{source} /srv/report ext4 defaults 0 2\n")
}
pub fn context(s: &lvm::State) -> f::Context {
    f::Context {
        inventory: lvm::mounts::Context {
            filesystems: vec![
                lvm::mounts::Filesystem {
                    original_volume: s.identity.as_ref().unwrap().volumes["team/draft"],
                    uuid: UUID_A.into(),
                },
                lvm::mounts::Filesystem {
                    original_volume: s.identity.as_ref().unwrap().volumes["team/reference"],
                    uuid: "12345678-0000-4000-8000-000000000002".into(),
                },
            ],
            mountpoints: vec![lvm::mounts::Mountpoint("/srv/report".into())],
        },
        initial_policy: policy("/dev/team/reference"),
        initial_live: vec![f::InitialMount {
            target: "/srv/report".into(),
            original_volume: s.identity.as_ref().unwrap().volumes["team/reference"],
            read_only: false,
        }],
        targets: vec![f::Target {
            target: "/srv/report".into(),
            original_volume: s.identity.as_ref().unwrap().volumes["team/draft"],
            file: "/keep.txt".into(),
            read_only: false,
            automatic: true,
        }],
        renaming: None,
    }
}
pub fn cmd(s: &str) -> f::Operation {
    f::Operation::Command { text: s.into() }
}
pub fn save(s: &str) -> f::Operation {
    f::Operation::SavePolicy { text: policy(s) }
}
pub fn reference() -> Vec<f::Operation> {
    vec![
        save("/dev/team/draft"),
        cmd("umount /srv/report"),
        cmd("mount /srv/report"),
        cmd("cat /srv/report/keep.txt"),
    ]
}
pub fn task() -> f::Task {
    f::Task {
        context: context(&world()),
        reference: reference(),
    }
}
pub fn practice() -> f::Practice {
    f::Practice {
        context: context(&world()),
        worked: reference()
            .into_iter()
            .map(|operation| f::Step {
                explanation: format!("Inspect this simulated change: {}", operation.description()),
                operation,
            })
            .collect(),
        experiment: cmd("umount /srv/report"),
        repair: vec![cmd("mount /srv/report")],
    }
}
pub fn rename_task() -> f::Task {
    let s = world();
    let mut c = context(&s);
    c.initial_policy = policy("/dev/team/draft");
    c.initial_live = vec![f::InitialMount {
        target: "/srv/report".into(),
        original_volume: 4,
        read_only: false,
    }];
    c.renaming = Some(rename::Plan {
        groups: vec![rename::Change {
            source: "team".into(),
            name: "production".into(),
        }],
        volumes: vec![rename::Change {
            source: "team/draft".into(),
            name: "records".into(),
        }],
    });
    f::Task {
        context: c,
        reference: vec![
            cmd("lvrename team draft records"),
            cmd("vgrename team production"),
            save(&format!("UUID={UUID_A}")),
            cmd("cat /srv/report/keep.txt"),
        ],
    }
}
