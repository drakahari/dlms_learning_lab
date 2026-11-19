//! Synthetic capability fixture, not a registered authored lesson.
use dlms_learning_lab::{
    engine, lvm,
    lvm_workspace::{self as w, config_scan as f},
    pack::{Pack, Scenario},
    script_teaching::Concept,
};
use std::collections::BTreeMap;
pub fn config(file: bool, rules: &[&str]) -> String {
    format!(
        "devices {{\n  use_devicesfile = {}\n  filter = [{}]\n}}\n",
        u8::from(file),
        rules
            .iter()
            .map(|s| serde_json::to_string(s).unwrap())
            .collect::<Vec<_>>()
            .join(", ")
    )
}
pub fn selection(paths: &[&str]) -> f::Selection {
    f::Selection {
        entries: paths.iter().map(|p| f::DeviceName((*p).into())).collect(),
    }
}
fn inventory(names: &[(&str, bool)]) -> lvm::State {
    lvm::State {
        version: 1,
        provisioning: None,
        activation: None,
        identity: None,
        pv_capacity: None,
        snapshots: None,
        devices: names
            .iter()
            .map(|(p, pv)| {
                (
                    (*p).into(),
                    lvm::Device {
                        bytes: 9 * lvm::MIB,
                        role: if *pv {
                            lvm::DeviceRole::PhysicalVolume
                        } else {
                            lvm::DeviceRole::Foreign
                        },
                        pv_bytes: None,
                        group: String::new(),
                    },
                )
            })
            .collect(),
        groups: BTreeMap::new(),
    }
}
fn context(example: bool, file: bool) -> f::Context {
    let names = if example {
        ["/dev/demoone", "/dev/demotwo", "/dev/scratch", "/dev/extra"]
    } else {
        [
            "/dev/appdata",
            "/dev/appspare",
            "/dev/recovery",
            "/dev/usbcopy",
        ]
    };
    let mut first = vec![(names[0], true), (names[2], false)];
    if file {
        first.push((names[1], true));
    }
    let second = vec![
        (names[0], true),
        (names[1], true),
        (names[2], false),
        (names[3], false),
    ];
    let pattern = if example {
        "a|^/dev/demo(one|two)$|"
    } else {
        "a|^/dev/app(data|spare)$|"
    };
    f::Context {
        initial_config: f::Text(config(
            false,
            &if file {
                [pattern, "r|.*|"]
            } else {
                ["r|.*|", pattern]
            },
        )),
        initial_file: file.then(|| selection(&[names[2]])),
        selection_editable: file,
        required_mode: Some(if file {
            f::RequiredMode::DevicesFile
        } else {
            f::RequiredMode::Filters
        }),
        cases: [
            ("Provided inventory", first),
            ("Provided arrival check", second),
        ]
        .into_iter()
        .map(|(label, names)| {
            let s = inventory(&names);
            f::Case {
                label: label.into(),
                expected: s
                    .devices
                    .iter()
                    .map(|(path, d)| f::Expected {
                        path: path.clone(),
                        allowed: d.role == lvm::DeviceRole::PhysicalVolume,
                    })
                    .collect(),
                inventory: s,
            }
        })
        .collect(),
    }
}
pub fn good(example: bool, file: bool) -> Vec<f::Operation> {
    let mut ops = vec![f::Operation::SaveConfig {
        text: f::Text(config(
            file,
            &[
                if example {
                    "a|^/dev/demo(one|two)$|"
                } else {
                    "a|^/dev/app(data|spare)$|"
                },
                "r|.*|",
            ],
        )),
    }];
    if file {
        ops.push(f::Operation::SaveSelection {
            selection: if example {
                selection(&["/dev/demoone", "/dev/demotwo"])
            } else {
                selection(&["/dev/appdata", "/dev/appspare"])
            },
        });
    }
    ops.push(f::Operation::Scan);
    ops
}
pub fn definition(file: bool) -> w::Definition {
    let c = context(false, file);
    let p = context(true, file);
    let operations = good(true, file);
    let experiment = vec![
        if file {
            f::Operation::SaveSelection {
                selection: selection(&["/dev/demoone"]),
            }
        } else {
            f::Operation::SaveConfig {
                text: f::Text(config(false, &["a|.*|"])),
            }
        },
        f::Operation::Scan,
    ];
    let practice=f::Practice{context:p.clone(),worked:operations.iter().map(|op|f::Step{operation:op.clone(),explanation:match op{
        f::Operation::SaveConfig{..}=>"Save the example's policy. Specific allow rules precede rejection; the devices-file switch selects which policy mechanism will be tested.".into(),
        f::Operation::SaveSelection{..}=>"Include both supplied example data PVs and leave unrelated devices out of the virtual selection.".into(),
        f::Operation::Scan=>"Compare actual allowed and excluded paths in both supplied inventories. Selection and finding a PV label are different results.".into(),
    }}).collect(),experiment,repair:operations};
    w::Definition{initial:c.cases[0].inventory.clone(),targets:vec![],plan:None,retirement:None,availability:None,renaming:None,snapshot_task:None,device_reference_task:None,
        mount_task:None,config_scan_task:Some(f::Task{context:c,reference:good(false,file)}),
        lesson:w::Lesson{prerequisite_note:"Know that a physical volume is storage prepared for LVM. Here only supplied virtual device metadata is read.".into(),concepts:vec![Concept{title:"Decide what LVM can inspect".into(),text:"A filter or an enabled devices file chooses device paths. Being allowed does not itself prove that a device has an LVM label.".into()},Concept{title:"First match decides".into(),text:"Each filter entry starts with a to accept or r to reject. The first matching entry wins; unmatched paths are allowed. Put specific allow rules before a final rejection.".into()}],description:"The worked example uses separate demo device names. Save its configuration, scan both inventories, make the suggested mistake and read the specific feedback.".into(),fixture:p.cases[0].inventory.clone(),targets:vec![],plan:None,retirement:None,availability:None,renaming:None,worked:vec![],snapshot_practice:None,device_reference_practice:None,mount_practice:None,config_scan_practice:Some(practice),try_change:"Make one selection mistake, scan and compare each actual result with its requirement. Restore only the affected policy, then scan again.".into(),experiment:String::new(),repair:String::new()}}
}
pub fn scenario(file: bool) -> Scenario {
    let raw: Pack =
        serde_json::from_str(include_str!("../../scenarios/lvm-growth-v1.json")).unwrap();
    let mut s = raw.scenarios[0].clone();
    let d = definition(file);
    s.id = if file {
        "lpic2-config-file-fixture"
    } else {
        "lpic2-config-filter-fixture"
    }
    .into();
    s.prerequisites.clear();
    s.unit = "test-only-config".into();
    s.title = "Inspect a simulated LVM selection policy".into();
    s.summary = "Repair the policy and scan supplied inventories without changing storage.".into();
    s.objective="Allow the supplied application PVs, exclude unrelated devices and use the required policy mechanism.".into();
    s.solution = d
        .config_scan_task
        .as_ref()
        .unwrap()
        .reference
        .iter()
        .map(|op| op.label().into())
        .collect();
    s.initial = engine::State::Lvm {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    s.goal = engine::Goal::Lvm {
        definition: Box::new(d),
    };
    s
}
