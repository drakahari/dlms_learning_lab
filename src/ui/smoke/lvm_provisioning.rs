//! Fixed, bounded native inputs over the registered raw-LVM lessons.
use super::*;
pub(in crate::ui) mod config_scan;
pub(in crate::ui) mod cow;
pub(in crate::ui) mod mounts;
pub(in crate::ui) mod references;
use dlms_learning_lab::lvm_workspace as work;
use std::time::Duration;
pub(in crate::ui) const IDS: [&str; 3] = [
    "lpic2-lvm-provision-staging",
    "lpic2-lvm-provision-group",
    "lpic2-lvm-provision-handover",
];
pub(in crate::ui) const RETIRE_IDS: [&str; 2] = ["lpic2-lvm-retire-raw", "lpic2-lvm-retire-pool"];
pub(in crate::ui) const ACTIVE_IDS: [&str; 2] =
    ["lpic2-lvm-activate-data", "lpic2-lvm-availability-handover"];
pub(in crate::ui) const RENAME_IDS: [&str; 2] =
    ["lpic2-lvm-rename-volume", "lpic2-lvm-rename-handover"];
pub(in crate::ui) const PV_IDS: [&str; 2] = ["lpic2-lvm-pv-capacity", "lpic2-lvm-pv-end"];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Provision,
    Retirement,
    Availability,
    Renaming,
    PhysicalCapacity,
    Cow,
    References,
    ConfigScan,
    Mounts,
}
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String),
    ReferenceField(String, String),
    ConfigField(String),
    MountField(String),
    ReferenceRead(String, String),
    Solved(bool),
    Reader(bool),
    RecordRead(String, String),
    Practice(bool),
    Unrun,
    Notice,
    Save,
    Fresh,
    Theme(bool),
    Challenge(bool),
    Shot(String, String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    not_before: Option<Instant>,
    done: bool,
    stable: Option<(egui::Rect, egui::Rect)>,
    stable_frames: u8,
}
pub(in crate::ui) fn control(a: &LabApp, key: &str) -> Option<egui::Rect> {
    if let Some(n) = key
        .strip_prefix("example-choice-")
        .and_then(|n| n.parse::<usize>().ok())
    {
        return a
            .metrics
            .lvm
            .example_choices
            .iter()
            .find(|r| r.0 == n)
            .map(|r| r.1);
    }
    if let Some((_, r, _, _)) = a.metrics.lvm.fields.iter().find(|r| r.0 == key) {
        return Some(*r);
    }
    if let Some((_, r, _)) = a.metrics.lvm.reader_rows.iter().find(|r| r.0 == key) {
        return Some(*r);
    }
    match key {
        "example-picker" => a.metrics.terminal_example_picker,
        "reader-viewport" => a.metrics.lvm.files_viewport,
        "command" => a
            .metrics
            .lvm
            .fields
            .iter()
            .find(|r| r.0 == "command")
            .map(|r| r.1),
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "challenge-header" => a.metrics.mode_challenge,
        "learn-header" => a.metrics.mode_learn,
        _ => a
            .metrics
            .lvm
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn command(ops: &mut Vec<Op>, text: &str) {
    ops.extend([Op::Field(text.into()), Op::Click("run".into())]);
}
fn route(labs: &[pack::Scenario], kind: Kind) -> Vec<Op> {
    if kind == Kind::Mounts {
        return mounts::route(labs);
    }
    if kind == Kind::ConfigScan {
        return config_scan::route(labs);
    }
    if kind == Kind::References {
        return references::route(labs);
    }
    if kind == Kind::Cow {
        return cow::route(labs);
    }
    let retirement = kind == Kind::Retirement;
    let availability = kind == Kind::Availability;
    let renaming = kind == Kind::Renaming;
    let physical = kind == Kind::PhysicalCapacity;
    let mut ops = Vec::new();
    let ids: &[&str] = if physical {
        &PV_IDS
    } else if renaming {
        &RENAME_IDS
    } else if availability {
        &ACTIVE_IDS
    } else if retirement {
        &RETIRE_IDS
    } else {
        &IDS
    };
    for (n, id) in ids.iter().enumerate() {
        let s = labs.iter().find(|s| s.id == *id).unwrap();
        let engine::Goal::Lvm { definition: d } = &s.goal else {
            panic!()
        };
        ops.extend([
            Op::Open((*id).into()),
            Op::Shot(format!("raw-{id}-introduction"), "reset-header".into()),
            Op::Click("practice-start".into()),
            Op::Click("insert".into()),
            Op::Unrun,
            Op::Click("run".into()),
        ]);
        for c in d.lesson.worked.iter().skip(1) {
            command(&mut ops, &c.text);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Shot(format!("raw-{id}-practice-pass"), "check".into()),
        ]);
        command(&mut ops, &d.lesson.experiment);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(format!("raw-{id}-practice-adverse"), "check".into()),
        ]);
        command(&mut ops, &d.lesson.repair);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
        ]);
        if availability {
            ops.extend([
                Op::Click("inspect-files".into()),
                Op::Reader(false),
                Op::Shot(
                    format!("raw-{id}-reader-unavailable"),
                    "reader-viewport".into(),
                ),
                Op::Click("inspect-files".into()),
            ]);
        }
        command(
            &mut ops,
            if physical {
                "pvresize --unsupported"
            } else if renaming {
                "lvrename --unsupported"
            } else if availability {
                "vgchange --unsupported"
            } else {
                "lvcreate --unsupported"
            },
        );
        ops.push(Op::Notice);
        if availability && n == 1 {
            command(&mut ops, "vgchange -an live");
            ops.push(Op::Notice);
        }
        if physical {
            command(
                &mut ops,
                if n == 0 {
                    "lvextend -L 24M vault/index"
                } else {
                    "pvresize --setphysicalvolumesize 17M /dev/storea"
                },
            );
            ops.push(Op::Notice);
        }
        let skip = if physical {
            if n == 0 {
                command(&mut ops, &s.solution[1]);
                2
            } else {
                command(&mut ops, "pvresize --setphysicalvolumesize 29M /dev/storea");
                0
            }
        } else if renaming {
            if n == 0 {
                command(&mut ops, "lvrename team draft reference");
                ops.push(Op::Notice);
                command(&mut ops, "lvrename team draft temporary");
                0
            } else {
                command(&mut ops, &s.solution[1]);
                2
            }
        } else if availability {
            if n == 0 {
                command(&mut ops, "vgchange -ay reserve");
                0
            } else {
                command(&mut ops, &s.solution[1]);
                2
            }
        } else if retirement {
            command(&mut ops, &s.solution[1]);
            2
        } else {
            match n {
                0 => {
                    command(&mut ops, "lvcreate -l 1 -n staging harbor");
                    0
                }
                1 => {
                    command(&mut ops, &s.solution[1]);
                    2
                }
                _ => {
                    command(&mut ops, "lvcreate -l 4 -n queue service");
                    2
                }
            }
        };
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(format!("raw-{id}-partial-feedback"), "check".into()),
        ]);
        if availability && n == 0 {
            command(&mut ops, "vgchange -an reserve");
        }
        if renaming && n == 0 {
            command(&mut ops, "lvrename team temporary draft");
        }
        if !retirement && !availability && !renaming && !physical && n == 0 {
            command(&mut ops, "lvremove -y harbor/staging");
        }
        for c in s.solution.iter().skip(skip) {
            command(&mut ops, c);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Save,
            Op::Fresh,
            Op::Solved(true),
            Op::Shot(format!("raw-{id}-independent-pass"), "check".into()),
        ]);
        if physical {
            ops.extend([
                Op::Click("inspect-files".into()),
                Op::Reader(true),
                Op::Shot(
                    format!("raw-{id}-retained-reader"),
                    "reader-viewport".into(),
                ),
                Op::Click("inspect-files".into()),
            ]);
            let (bad, repair) = if n == 0 {
                (
                    "pvresize --setphysicalvolumesize 33M /dev/vaulta",
                    "pvresize /dev/vaulta",
                )
            } else {
                (
                    "pvresize /dev/storea",
                    "pvresize --setphysicalvolumesize 25M /dev/storea",
                )
            };
            command(&mut ops, bad);
            ops.push(Op::Solved(false));
            command(&mut ops, repair);
        } else if renaming {
            ops.extend([
                Op::Click("inspect-files".into()),
                Op::Reader(true),
                Op::Shot(format!("raw-{id}-renamed-reader"), "reader-viewport".into()),
                Op::Click("inspect-files".into()),
            ]);
            let (bad, repair) = if n == 0 {
                (
                    "lvrename team reports temporary",
                    "lvrename team temporary reports",
                )
            } else {
                (
                    "vgrename warehouse temporary",
                    "vgrename temporary warehouse",
                )
            };
            command(&mut ops, bad);
            ops.push(Op::Solved(false));
            command(&mut ops, repair);
        } else if availability {
            ops.extend([
                Op::Click("inspect-files".into()),
                Op::Reader(true),
                Op::Shot(
                    format!("raw-{id}-reader-available"),
                    "reader-viewport".into(),
                ),
                Op::Click("inspect-files".into()),
            ]);
            let target = &d
                .availability
                .as_ref()
                .unwrap()
                .groups
                .iter()
                .find(|g| g.active)
                .unwrap()
                .name;
            command(&mut ops, &format!("vgchange -an {target}"));
            ops.push(Op::Solved(false));
            command(&mut ops, &format!("vgchange -ay {target}"));
        } else if retirement {
            let (bad, repair) = if n == 0 {
                (
                    "lvcreate -l 1 -n scratch records",
                    "lvremove -y records/scratch",
                )
            } else {
                ("pvcreate /dev/retirea", "pvremove /dev/retirea")
            };
            command(&mut ops, bad);
            ops.push(Op::Solved(false));
            command(&mut ops, repair);
        } else {
            let target = &d.plan.as_ref().unwrap().volumes[0];
            command(&mut ops, &format!("lvremove -y {}", target.id));
            ops.push(Op::Solved(false));
            let (g, name) = target.id.split_once('/').unwrap();
            command(
                &mut ops,
                &format!(
                    "lvcreate -L {}M -n {name} {g}",
                    target.bytes / dlms_learning_lab::lvm::MIB
                ),
            );
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Theme(true),
            Op::Field("lvs".into()),
            Op::Shot(format!("raw-{id}-light150-command"), "command".into()),
            Op::Theme(false),
            Op::Shot(format!("raw-{id}-dark200-command"), "command".into()),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Solved(false),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(format!("raw-{id}-challenge"), "reset-header".into()),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
        ]);
    }
    ops.extend([
        Op::Open("lpic2-lvm-free-space".into()),
        Op::Click("assignment".into()),
        Op::Shot("raw-retained-growth".into(), "command".into()),
    ]);
    ops
}
fn session(a: &LabApp) -> &work::Session {
    let i = a.selected.unwrap();
    let engine::State::Lvm { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
impl LabApp {
    fn raw_click(&mut self, ctx: &egui::Context, key: &str, r: &mut Route) -> bool {
        let rect = control(self, key).unwrap_or_else(|| panic!("Missing raw-LVM control {key}"));
        let clip = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let visible = smoke_control_visible(rect, clip, ctx.pixels_per_point());
        let done = self.click_visible_smoke_control(ctx, Some(rect));
        if !visible && !done {
            r.not_before = Some(Instant::now() + Duration::from_millis(200));
        }
        done
    }
    pub(in crate::ui) fn lvm_provisioning_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("raw-lvm-fixed-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(
                    &self.scenarios,
                    match std::env::var("DLMS_SMOKE_STAGE").as_deref() {
                        Ok("lvm-mount-batch") => Kind::Mounts,
                        Ok("lvm-config-batch") => Kind::ConfigScan,
                        Ok("lvm-reference-batch") => Kind::References,
                        Ok("lvm-cow-batch") => Kind::Cow,
                        Ok("lvm-activation-batch") => Kind::Availability,
                        Ok("lvm-rename-batch") => Kind::Renaming,
                        Ok("lvm-pv-capacity-batch") => Kind::PhysicalCapacity,
                        Ok("lvm-retirement-batch") => Kind::Retirement,
                        _ => Kind::Provision,
                    },
                ),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "RAW_LVM_NATIVE_COMPLETE: real field/run/check, separate practice, adverse/retry, partial/current-invalidation, safe save/load, selected reset, Challenge and scaled controls; no host devices"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        if r.not_before.is_some_and(|t| Instant::now() < t) {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(Duration::from_millis(16));
            return;
        }
        r.not_before = None;
        self.smoke_stage = 110000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("RAW_LVM_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(r.frames < 1200, "Raw LVM action stalled: {} {op:?}", r.at);
        let done = match op {
            Op::Open(id) => {
                self.scale = 1.;
                ctx.set_zoom_factor(1.);
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                r.not_before = Some(Instant::now() + Duration::from_millis(300));
                true
            }
            Op::Click(name) => self.raw_click(ctx, &name, &mut r),
            op @ (Op::Field(_)
            | Op::ReferenceField(_, _)
            | Op::ConfigField(_)
            | Op::MountField(_)) => {
                let (key, value) = match op {
                    Op::Field(v) => ("command".to_owned(), v),
                    Op::MountField(v) => ("mount-editor".to_owned(), v),
                    Op::ConfigField(v) => ("config-editor".to_owned(), v),
                    Op::ReferenceField(k, v) => (format!("reference-{k}"), v),
                    _ => unreachable!(),
                };
                if r.phase == 0 {
                    if self.raw_click(ctx, &key, &mut r) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let field = self.metrics.lvm.fields.iter().find(|f| f.0 == key).unwrap();
                    assert_eq!(ctx.memory(|m| m.focused()), Some(field.2));
                    let modifiers = egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    };
                    for pressed in [true, false] {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers,
                        });
                    }
                    self.pending_events.push(egui::Event::Paste(value));
                    r.phase = 2;
                    false
                } else {
                    let s = session(self);
                    let practice = s.view == teaching::View::Example;
                    let actual = if key == "mount-editor" {
                        let r = s.mount_runtime.as_ref().unwrap();
                        &if practice { &r.example } else { &r.assignment }.draft
                    } else if key == "config-editor" {
                        let r = s.config_scan_runtime.as_ref().unwrap();
                        &if practice { &r.example } else { &r.assignment }.draft.0
                    } else if let Some(caller) = key.strip_prefix("reference-") {
                        let runtime = s.reference_runtime.as_ref().unwrap();
                        let live = if practice {
                            &runtime.example
                        } else {
                            &runtime.assignment
                        };
                        &live.drafts[caller].0
                    } else if practice {
                        &s.example_command
                    } else {
                        &s.command
                    };
                    assert_eq!(actual, &value);
                    true
                }
            }
            Op::Solved(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&s.id], s),
                    want,
                    "{} current outcome",
                    s.id
                );
                true
            }
            Op::ReferenceRead(key, want) => {
                let s = session(self);
                let practice = s.view == teaching::View::Example;
                let r = s.reference_runtime.as_ref().unwrap();
                let live = if practice { &r.example } else { &r.assignment };
                let storage = if practice { &s.example } else { &s.current };
                assert_eq!(
                    live.configuration.inspect(storage, &key).unwrap().content,
                    want
                );
                assert!(
                    self.metrics
                        .lvm
                        .reader_records
                        .iter()
                        .any(|r| r.0 == format!("reference-{key}") && r.1)
                );
                true
            }
            Op::RecordRead(key, want) => {
                let parts = key.split(':').collect::<Vec<_>>();
                assert_eq!(parts.len(), 3);
                let s = session(self);
                let current = if s.view == teaching::View::Example {
                    &s.example
                } else {
                    &s.current
                };
                let saved = if parts[1] == "current" {
                    None
                } else {
                    Some(parts[1])
                };
                assert_eq!(
                    dlms_learning_lab::lvm::snapshot::read_record(
                        current, parts[0], saved, parts[2]
                    )
                    .unwrap(),
                    want
                );
                assert!(
                    self.metrics
                        .lvm
                        .reader_records
                        .iter()
                        .any(|r| r.0 == key && r.1)
                );
                true
            }
            Op::Reader(want) => {
                let engine::Goal::Lvm { definition } = &self.scenarios[self.selected.unwrap()].goal
                else {
                    panic!()
                };
                let (current_path, original_path) = if definition.initial.pv_capacity.is_some() {
                    let path = definition.targets[0]
                        .volume
                        .strip_prefix("/dev/")
                        .unwrap_or(&definition.targets[0].volume)
                        .to_owned();
                    (path.clone(), path)
                } else if let Some(plan) = &definition.renaming {
                    let original = &plan.volumes[0].source;
                    let id = definition.initial.identity.as_ref().unwrap().volumes[original];
                    let current = session(self)
                        .current
                        .identity
                        .as_ref()
                        .unwrap()
                        .volumes
                        .iter()
                        .find(|(_, v)| **v == id)
                        .unwrap()
                        .0
                        .clone();
                    (current, original.clone())
                } else {
                    let name = &definition
                        .availability
                        .as_ref()
                        .unwrap()
                        .groups
                        .iter()
                        .find(|g| g.active)
                        .unwrap()
                        .name;
                    let lv = definition.initial.groups[name]
                        .volumes
                        .iter()
                        .find(|(_, v)| v.filesystem.is_some())
                        .unwrap()
                        .0;
                    let path = format!("{name}/{lv}");
                    (path.clone(), path)
                };
                let path = definition
                    .initial
                    .volume(&original_path)
                    .unwrap()
                    .1
                    .filesystem
                    .as_ref()
                    .unwrap()
                    .files
                    .keys()
                    .next()
                    .unwrap();
                let key = format!("{current_path}:{path}");
                if let Some((_, actual)) =
                    self.metrics.lvm.reader_records.iter().find(|r| r.0 == key)
                {
                    assert_eq!(*actual, want, "Actual rendered reader {key}");
                    let (_, rect, clip) = self
                        .metrics
                        .lvm
                        .reader_rows
                        .iter()
                        .find(|r| r.0 == key)
                        .expect("Actual rendered reader row");
                    if smoke_control_visible(*rect, *clip, ctx.pixels_per_point()) {
                        true
                    } else {
                        self.pending_events.extend([
                            egui::Event::PointerMoved(clip.center()),
                            egui::Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::vec2(
                                    0.,
                                    (clip.center().y - rect.center().y).clamp(-420., 420.),
                                ),
                                modifiers: Default::default(),
                            },
                        ]);
                        r.not_before = Some(Instant::now() + Duration::from_millis(200));
                        false
                    }
                } else {
                    false
                }
            }
            Op::Practice(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::Lvm { definition } = &s.goal else {
                    panic!()
                };
                let session = session(self);
                assert!(session.example_checked);
                assert_eq!(
                    definition.practice_checks(session).iter().all(|r| r.0),
                    want
                );
                assert_eq!(session.current, session.initial);
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
            Op::Unrun => {
                let s = session(self);
                assert_eq!(s.example, s.example_initial);
                assert!(!s.example_checked);
                assert_eq!(s.current, s.initial);
                if let Some(runtime) = &s.config_scan_runtime {
                    assert!(runtime.example.last_run.is_none());
                }
                true
            }
            Op::Notice => {
                assert!(self.notice.is_some());
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let i = self.selected.unwrap();
                let before = serde_json::to_value(&self.progress).unwrap();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_value(&self.progress).unwrap(), before);
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::Theme(light) => {
                if let Some(id) = ctx.memory(|m| m.focused()) {
                    ctx.memory_mut(|m| m.surrender_focus(id));
                }
                self.focus_input = false;
                self.scale = if light { 1.5 } else { 2. };
                ctx.set_zoom_factor(self.scale);
                self.theme_id = if light {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                    if session(self).mount_runtime.is_some()
                        || session(self).config_scan_runtime.is_some()
                    {
                        egui::vec2(1268., 1308.)
                    } else {
                        egui::vec2(1268., 632.)
                    },
                ));
                self.lesson_scroll_reset = true;
                r.not_before = Some(Instant::now() + Duration::from_millis(300));
                true
            }
            Op::Challenge(want) => {
                let i = self.selected.unwrap();
                assert_eq!(
                    self.progress.attempts[&self.scenarios[i].id].challenge,
                    want
                );
                if want {
                    assert!(self.metrics.lesson_tabs.is_empty());
                }
                true
            }
            Op::Shot(name, target) => {
                let rect = control(self, &target).expect("Actual capture target");
                let clip = self
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let geometry = (rect, clip);
                let visible = smoke_control_visible(rect, clip, ctx.pixels_per_point());
                if r.phase == 0 {
                    if visible {
                        if r.stable == Some(geometry) {
                            r.stable_frames += 1;
                        } else {
                            r.stable = Some(geometry);
                            r.stable_frames = 0;
                        }
                        if r.stable_frames >= 4 {
                            r.phase = 1;
                        }
                    } else {
                        r.stable = None;
                        r.stable_frames = 0;
                        self.reveal_smoke_control(ctx, rect);
                        r.not_before = Some(Instant::now() + Duration::from_millis(200));
                    }
                    false
                } else if r.phase == 1 {
                    if !visible || r.stable != Some(geometry) {
                        r.phase = 0;
                        r.stable = None;
                        r.stable_frames = 0;
                        false
                    } else {
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name),
                        ));
                        r.phase = 2;
                        false
                    }
                } else {
                    !self.capture_pending
                }
            }
        };
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.stable = None;
            r.stable_frames = 0;
            r.not_before = r
                .not_before
                .or_else(|| Some(Instant::now() + Duration::from_millis(50)));
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_lvm_native_route_exercises_all_assignments_practice_invalidating_actions_and_scales() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios, Kind::Provision);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 4);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Theme(_))).count(), 6);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 6);
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s.starts_with("lvremove -y")))
        );
    }
    #[test]
    fn retirement_native_route_has_both_tasks_and_actual_label_reintroduction() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios, Kind::Retirement);
        for id in RETIRE_IDS {
            assert!(ops.iter().any(|o| matches!(o,Op::Open(s)if s==id)));
        }
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s=="pvcreate /dev/retirea"))
        );
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s=="pvremove /dev/retirea"))
        );
        assert!(ops.iter().any(|o| matches!(o, Op::Fresh)));
        assert!(ops.iter().any(|o| matches!(o, Op::Practice(false))));
    }
    #[test]
    fn activation_native_route_witnesses_actual_unavailable_and_available_reader() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios, Kind::Availability);
        for id in ACTIVE_IDS {
            assert!(ops.iter().any(|o| matches!(o,Op::Open(s)if s==id)));
        }
        for want in [false, true] {
            assert_eq!(
                ops.iter()
                    .filter(|o| matches!(o,Op::Reader(v)if *v==want))
                    .count(),
                2
            );
        }
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s=="vgchange -an live"))
        );
    }
    #[test]
    fn rename_native_route_requires_both_original_object_readers_and_partial_resume() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios, Kind::Renaming);
        for id in RENAME_IDS {
            assert!(ops.iter().any(|o| matches!(o,Op::Open(s)if s==id)));
        }
        assert_eq!(
            ops.iter().filter(|o| matches!(o, Op::Reader(true))).count(),
            2
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 4);
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s=="lvrename team draft reference"))
        );
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s=="vgrename warehouse temporary"))
        );
        assert!(ops.iter().any(|o| matches!(o, Op::Practice(false))));
    }
    #[test]
    fn physical_capacity_native_route_has_both_tasks_layer_and_high_placement_refusals() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios, Kind::PhysicalCapacity);
        for id in PV_IDS {
            assert!(ops.iter().any(|o| matches!(o,Op::Open(s)if s==id)));
        }
        assert_eq!(
            ops.iter().filter(|o| matches!(o, Op::Reader(true))).count(),
            2
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 4);
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(s)if s=="lvextend -L 24M vault/index"))
        );
        assert!(ops.iter().any(
            |o| matches!(o,Op::Field(s)if s=="pvresize --setphysicalvolumesize 17M /dev/storea")
        ));
        assert!(ops.iter().any(
            |o| matches!(o,Op::Field(s)if s=="pvresize --setphysicalvolumesize 29M /dev/storea")
        ));
        assert!(ops.iter().any(|o| matches!(o, Op::Practice(false))));
    }
    #[test]
    fn raw_lvm_actual_capture_phase_is_context_safe() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
        a.launch(i);
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400., 950.),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw, |ctx| a.render(ctx));
        let rect = control(&a, "reset-header").unwrap();
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        assert!(smoke_control_visible(rect, clip, ctx.pixels_per_point()));
        let key = egui::Id::new("raw-lvm-fixed-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("unit-raw-capture".into(), "reset-header".into())],
                    phase: 1,
                    stable: Some((rect, clip)),
                    stable_frames: 4,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| {
            a.lvm_provisioning_tick(ctx)
        });
        assert!(a.capture_pending);
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().phase, 2);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn raw_lvm_waiting_deadline_does_not_spend_another_visibility_attempt() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
        a.launch(i);
        let key = egui::Id::new("raw-lvm-fixed-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Click("run".into())],
                    not_before: Some(Instant::now() + Duration::from_secs(60)),
                    ..Default::default()
                },
            )
        });
        for _ in 0..50 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                a.lvm_provisioning_tick(ctx)
            });
        }
        let r = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
        assert_eq!(r.at, 0);
        assert_eq!(r.frames, 0);
        assert!(a.pending_events.is_empty());
    }
}
