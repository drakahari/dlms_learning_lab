//! Synthetic in-process fixtures only; not a catalog lesson or learner profile.
#![allow(dead_code)]
use dlms_learning_lab::{autofs::runtime::*, lvm};
pub const MASTER: &str = "/etc/auto.master";
pub const INDIRECT: &str = "/etc/auto.teams";
pub const DIRECT: &str = "/etc/auto.direct";
pub const EXTRA: &str = "/etc/auto.extra";
pub const REPORT: &str = "/srv/teams/report/keep.txt";
pub const DIRECT_REPORT: &str = "/opt/reports/keep.txt";
pub fn world() -> lvm::State {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let s: lvm::State =
        serde_json::from_value(p["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap();
    s.validate().unwrap();
    s
}
pub fn context(s: &lvm::State) -> Context {
    Context {
        inventory: lvm::mounts::Context {
            filesystems: [("team/draft", 1), ("team/reference", 2)]
                .into_iter()
                .map(|(n, id)| lvm::mounts::Filesystem {
                    original_volume: s.identity.as_ref().unwrap().volumes[n],
                    uuid: format!("12345678-0000-4000-8000-{id:012}"),
                })
                .collect(),
            mountpoints: [
                "/srv/teams/report",
                "/srv/teams/audit",
                "/opt/reports",
                "/data/audit",
            ]
            .into_iter()
            .map(|p| lvm::mounts::Mountpoint(p.into()))
            .collect(),
        },
        map_files: [INDIRECT, DIRECT, EXTRA]
            .into_iter()
            .map(|p| Name(p.into()))
            .collect(),
    }
}
pub fn indirect(source: &str) -> String {
    format!("report -fstype=ext4,rw :{source}\naudit -fstype=ext4,ro :/dev/team/reference\n")
}
pub fn direct(source: &str) -> String {
    format!(
        "/opt/reports -fstype=ext4,rw :{source}\n/data/audit -fstype=ext4,ro :/dev/team/reference\n"
    )
}
pub fn config(is_direct: bool) -> Configuration {
    Configuration {
        master: Text(if is_direct {
            format!("/- {DIRECT} --timeout=30\n")
        } else {
            format!("/srv/teams {INDIRECT} --timeout=30\n")
        }),
        maps: vec![
            MapFile {
                name: Name(INDIRECT.into()),
                text: Text(indirect("/dev/team/draft")),
            },
            MapFile {
                name: Name(DIRECT.into()),
                text: Text(direct("/dev/team/draft")),
            },
            MapFile {
                name: Name(EXTRA.into()),
                text: Text(String::new()),
            },
        ],
    }
}
pub fn fixture(is_direct: bool) -> (lvm::State, Context, State) {
    let s = world();
    let c = context(&s);
    let cfg = config(is_direct);
    let a = State::new(&c, &s, &cfg, cfg.clone()).unwrap();
    (s, c, a)
}
pub fn command(text: &str) -> dlms_learning_lab::autofs_workspace::Operation {
    dlms_learning_lab::autofs_workspace::Operation::Command { text: text.into() }
}
pub fn save(file: &str, text: String) -> dlms_learning_lab::autofs_workspace::Operation {
    dlms_learning_lab::autofs_workspace::Operation::Save {
        file: Name(file.into()),
        text: Text(text),
    }
}
pub fn activity(is_direct: bool) -> dlms_learning_lab::autofs_workspace::Activity {
    use dlms_learning_lab::autofs::{maps::Style, runtime};
    use dlms_learning_lab::autofs_workspace::{Activity, Target};
    let s = world();
    let mut c = context(&s);
    c.inventory.mountpoints.retain(|p| {
        if is_direct {
            p.0 == "/opt/reports" || p.0 == "/data/audit"
        } else {
            p.0.starts_with("/srv/teams/")
        }
    });
    let mut cfg = config(is_direct);
    let file = if is_direct { DIRECT } else { INDIRECT };
    cfg.maps.iter_mut().find(|m| m.name.0 == file).unwrap().text = Text(if is_direct {
        direct("/dev/team/reference")
    } else {
        indirect("/dev/team/reference")
    });
    Activity {
        world: s.clone(),
        context: c,
        active: cfg.clone(),
        saved: cfg,
        warm: if is_direct {
            vec![]
        } else {
            vec![Name(REPORT.into())]
        },
        targets: [
            (
                if is_direct {
                    "/opt/reports"
                } else {
                    "/srv/teams/report"
                },
                "team/draft",
                false,
            ),
            (
                if is_direct {
                    "/data/audit"
                } else {
                    "/srv/teams/audit"
                },
                "team/reference",
                true,
            ),
        ]
        .into_iter()
        .map(|(p, n, read_only)| Target {
            point: runtime::Name(p.into()),
            original_volume: s.identity.as_ref().unwrap().volumes[n],
            file: Name("/keep.txt".into()),
            read_only,
            style: if is_direct {
                Style::Direct
            } else {
                Style::Indirect
            },
            timeout: 30,
        })
        .collect(),
    }
}
pub fn steps(is_direct: bool) -> Vec<dlms_learning_lab::autofs_workspace::Step> {
    use dlms_learning_lab::autofs_workspace::{Operation, Step};
    let mut ops = vec![save(
        if is_direct { DIRECT } else { INDIRECT },
        if is_direct {
            direct("/dev/team/draft")
        } else {
            indirect("/dev/team/draft")
        },
    )];
    if is_direct {
        ops.push(command("systemctl reload autofs"));
    } else {
        ops.push(Operation::Idle { seconds: 30 });
    }
    ops.push(command(if is_direct {
        "cat /opt/reports/keep.txt"
    } else {
        "cat /srv/teams/report/keep.txt"
    }));
    ops.push(command(if is_direct {
        "cat /data/audit/keep.txt"
    } else {
        "cat /srv/teams/audit/keep.txt"
    }));
    ops.into_iter()
        .map(|operation| Step {
            explanation: format!("Observe the simulated state after {}", operation.label()),
            operation,
            status: 0,
        })
        .collect()
}
pub fn definition(is_direct: bool) -> dlms_learning_lab::autofs_workspace::Definition {
    use dlms_learning_lab::autofs_workspace::{Definition, Step};
    let assignment = activity(is_direct);
    let example = serde_json::from_str(
        &serde_json::to_string(&assignment)
            .unwrap()
            .replace("team", "demo")
            .replace("reserve", "demo-unused"),
    )
    .unwrap();
    let worked = steps(is_direct);
    let example_steps = serde_json::from_str(
        &serde_json::to_string(&worked)
            .unwrap()
            .replace("team", "demo"),
    )
    .unwrap();
    let wrong = save(
        if is_direct { DIRECT } else { INDIRECT },
        if is_direct {
            direct("/dev/team/reference")
        } else {
            indirect("/dev/team/reference")
        },
    );
    let experiment = Step {
        operation: serde_json::from_str(
            &serde_json::to_string(&wrong)
                .unwrap()
                .replace("team", "demo"),
        )
        .unwrap(),
        explanation: "Try a wrong source and inspect future lookup feedback".into(),
        status: 0,
    };
    let repair = serde_json::from_str(
        &serde_json::to_string(&worked[..if is_direct { 2 } else { 1 }])
            .unwrap()
            .replace("team", "demo"),
    )
    .unwrap();
    Definition {
        version: 1,
        source_id: if is_direct {
            "autofs-direct-fixture"
        } else {
            "autofs-indirect-fixture"
        }
        .into(),
        assignment,
        example,
        reference: worked,
        example_steps,
        experiment,
        repair,
    }
}
pub fn scenario(is_direct: bool) -> dlms_learning_lab::pack::Scenario {
    use dlms_learning_lab::{
        autofs_workspace::Session,
        engine::{Goal, State},
    };
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut s: dlms_learning_lab::pack::Scenario =
        serde_json::from_value(p["scenarios"][0].clone()).unwrap();
    let d = definition(is_direct);
    s.id = d.source_id.clone();
    s.prerequisites.clear();
    s.unit = "autofs-fixture".into();
    s.initial = State::Autofs {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.solution = d
        .reference
        .iter()
        .map(|s| s.operation.label())
        .chain(std::iter::once("Check work".into()))
        .collect();
    s.goal = Goal::Autofs {
        definition: Box::new(d),
    };
    s
}
