//! Fixed native teaching acceptance. Mutations use only real UI input events.
use super::*;
use dlms_learning_lab::{seed_dns_teaching as dns, seed_mount_teaching as mount};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const START: usize = 25000;
const END: usize = 26000;
const IDS: [&str; 2] = ["lpic2-dns", "lpic2-storage"];
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Try,
    Assignment,
    Example,
    Input,
    Run,
    Insert,
    Picker,
    Choice(usize),
    Field(String),
    Save,
    Check,
    Reset,
    Confirm,
    Cancel,
    Challenge,
    Learn,
}
#[derive(Clone, Debug, PartialEq)]
enum Step {
    Library,
    Filter(usize),
    Start,
    Wait,
    Click(Target),
    Text(String),
    BeforeInsert,
    InsertOnly(usize),
    Ran(String, bool, bool),
    Practice(Vec<bool>),
    Current(Vec<bool>, bool),
    Snapshot,
    CancelExact,
    ResetExact,
    Drafts,
    ResumeExact,
    SaveReadback,
    Theme(bool),
    Geometry(f32),
    Mode(bool),
    Shot(String, bool),
    Finish,
}
#[derive(Clone)]
enum Action {
    Save(Vec<(String, String)>),
    Command(String),
}
fn actions(s: &Scenario, group: usize) -> Vec<Action> {
    if let Some(l) = &s.seed_dns_teaching {
        let actions: Vec<_> = match group {
            0 => l.example_actions.iter().map(|a| a.action.clone()).collect(),
            1 => l.try_actions.clone(),
            _ => l.repair_actions.clone(),
        };
        actions
            .into_iter()
            .map(|a| match a {
                dns::Action::Save { draft } => Action::Save(vec![
                    ("seed-dns-address".into(), draft.address),
                    ("seed-dns-serial".into(), draft.serial),
                ]),
                dns::Action::Command { command } => Action::Command(command.command),
            })
            .collect()
    } else {
        let l = s.seed_mount_teaching.as_ref().unwrap();
        let actions: Vec<_> = match group {
            0 => l.example_actions.iter().map(|a| a.action.clone()).collect(),
            1 => l.try_actions.clone(),
            _ => l.repair_actions.clone(),
        };
        actions
            .into_iter()
            .map(|a| match a {
                mount::Action::Save { draft } => {
                    Action::Save(vec![("seed-mount-device".into(), draft.device)])
                }
                mount::Action::Command { command } => Action::Command(command.command),
            })
            .collect()
    }
}
fn field(p: &mut Vec<Step>, name: &str, value: &str) {
    p.extend([
        Step::Click(Target::Field(name.into())),
        Step::Text(value.into()),
    ]);
}
fn command(p: &mut Vec<Step>, text: &str, practice: bool, success: bool) {
    p.extend([
        Step::Click(Target::Input),
        Step::Text(text.into()),
        Step::Click(Target::Run),
        Step::Ran(text.into(), practice, success),
    ]);
}
fn apply(p: &mut Vec<Step>, action: &Action, inserted: bool) {
    match action {
        Action::Save(fields) => {
            if !inserted {
                for (name, value) in fields {
                    field(p, name, value);
                }
            }
            p.push(Step::Click(Target::Save));
        }
        Action::Command(text) if inserted => p.extend([
            Step::Click(Target::Run),
            Step::Ran(text.clone(), true, true),
        ]),
        Action::Command(text) => command(p, text, true, true),
    }
}
fn check(p: &mut Vec<Step>, assertion: Step, name: String) {
    p.extend([
        Step::Click(Target::Check),
        Step::Wait,
        assertion,
        Step::Shot(name, true),
    ]);
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Snapshot,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::Wait,
        Step::ResetExact,
        Step::Click(Target::Assignment),
        Step::Wait,
    ]);
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = Vec::new();
    for (n, id) in IDS.iter().enumerate() {
        let s = scenarios
            .iter()
            .find(|s| s.id == *id)
            .expect("normal original lesson");
        let shot = |label: &str| format!("seed-{n}-{label}");
        let pass = if n == 0 {
            vec![true, true]
        } else {
            vec![true, true, true]
        };
        p.extend([
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::Shot(shot("introduction"), false),
            Step::Click(Target::Try),
            Step::Wait,
        ]);
        for (index, a) in actions(s, 0).iter().enumerate() {
            p.extend([
                Step::Click(Target::Picker),
                Step::Click(Target::Choice(index)),
                Step::BeforeInsert,
                Step::Click(Target::Insert),
                Step::InsertOnly(index),
            ]);
            if matches!(a, Action::Command(command) if command == "named-checkzone lab.test") {
                p.push(Step::Shot(shot("simulator-caption"), false));
            }
            apply(&mut p, a, true);
        }
        check(
            &mut p,
            Step::Practice(pass.clone()),
            shot("example-complete"),
        );
        for a in actions(s, 1) {
            apply(&mut p, &a, false);
        }
        check(
            &mut p,
            Step::Practice(if n == 0 {
                vec![false, false]
            } else {
                vec![true, true, false]
            }),
            shot("example-experiment"),
        );
        for a in actions(s, 2) {
            apply(&mut p, &a, false);
        }
        check(
            &mut p,
            Step::Practice(pass.clone()),
            shot("example-repaired"),
        );
        p.extend([
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Shot(shot("assignment"), false),
        ]);
        if n == 0 {
            field(&mut p, "dns-address", "192.0.2.81");
            field(&mut p, "dns-serial", "2026102102");
            p.push(Step::Click(Target::Save));
            command(&mut p, "systemctl reload named", false, true);
            command(&mut p, "dig portal.lab.test", false, true);
            check(
                &mut p,
                Step::Current(vec![false, false], false),
                shot("wrong"),
            );
            field(&mut p, "dns-address", "192.0.2.80");
            field(&mut p, "dns-serial", "2026102103");
            p.push(Step::Click(Target::Save));
            check(
                &mut p,
                Step::Current(vec![true, false], false),
                shot("partial"),
            );
            command(&mut p, "systemctl reload named", false, true);
        } else {
            command(&mut p, "mount /data", false, false);
            check(
                &mut p,
                Step::Current(vec![false, false, false], false),
                shot("wrong"),
            );
            command(&mut p, "mount -o remount,rw /data", false, true);
            command(&mut p, "rm -r /data/cache", false, true);
            check(
                &mut p,
                Step::Current(vec![true, false, true], false),
                shot("partial"),
            );
            field(&mut p, "mount-device", "/dev/vdb1");
            p.push(Step::Click(Target::Save));
        }
        check(&mut p, Step::Current(pass.clone(), true), shot("complete"));
        // Pending raw fields invalidate the current completion without publishing
        // an invalid model. Failed Save must keep both bytes and simulation.
        p.push(Step::Snapshot);
        field(
            &mut p,
            if n == 0 { "dns-serial" } else { "mount-device" },
            "unfinished invalid",
        );
        p.push(Step::Click(Target::Save));
        check(
            &mut p,
            Step::Current(pass.clone(), false),
            shot("pending-invalid"),
        );
        field(
            &mut p,
            if n == 0 { "dns-serial" } else { "mount-device" },
            if n == 0 { "2026102103" } else { "/dev/vdb1" },
        );
        p.push(Step::Click(Target::Save));
        check(
            &mut p,
            Step::Current(pass.clone(), true),
            shot("pending-repaired"),
        );
        for light in [true, false] {
            p.extend([
                Step::Theme(light),
                Step::Geometry(if light { 1.5 } else { 2. }),
            ]);
            field(
                &mut p,
                if n == 0 {
                    "dns-address"
                } else {
                    "mount-device"
                },
                if n == 0 { "192.0.2.80" } else { "/dev/vdb1" },
            );
            p.push(Step::Click(Target::Save));
            command(
                &mut p,
                if n == 0 {
                    "dig portal.lab.test"
                } else {
                    "df -h"
                },
                false,
                true,
            );
            check(
                &mut p,
                Step::Current(pass.clone(), true),
                shot(if light {
                    "light150-typed"
                } else {
                    "dark200-typed"
                }),
            );
            p.extend([
                Step::Click(Target::Challenge),
                Step::Wait,
                Step::Mode(true),
                Step::Shot(
                    shot(if light {
                        "light150-challenge"
                    } else {
                        "dark200-challenge"
                    }),
                    false,
                ),
                Step::Click(Target::Learn),
                Step::Wait,
                Step::Mode(false),
            ]);
        }
        // Leave distinct unfinished drafts in both independent contexts.
        field(
            &mut p,
            if n == 0 { "dns-serial" } else { "mount-device" },
            "unfinished assignment",
        );
        p.extend([
            Step::Click(Target::Input),
            Step::Text("unfinished assignment command".into()),
            Step::Click(Target::Example),
            Step::Wait,
        ]);
        field(
            &mut p,
            if n == 0 {
                "seed-dns-serial"
            } else {
                "seed-mount-device"
            },
            "unfinished practice",
        );
        p.extend([
            Step::Click(Target::Input),
            Step::Text("unfinished practice command".into()),
            Step::Drafts,
            Step::SaveReadback,
            Step::Library,
            Step::Filter(n),
            Step::Wait,
            Step::Start,
            Step::Wait,
            Step::ResumeExact,
            Step::Shot(shot("resumed-drafts"), false),
            Step::Click(Target::Assignment),
            Step::Wait,
            Step::Snapshot,
            Step::Click(Target::Reset),
            Step::Click(Target::Cancel),
            Step::CancelExact,
        ]);
        reset(&mut p);
        if n == 0 {
            for c in [
                "dns set portal.lab.test 192.0.2.80",
                "dns serial 2026102110",
                "systemctl reload named",
                "dig portal.lab.test",
            ] {
                command(&mut p, c, false, true);
            }
        } else {
            field(&mut p, "mount-device", "/dev/vdb1");
            p.push(Step::Click(Target::Save));
            for c in ["umount /data", "mount /data", "rm -r /data/cache", "df -h"] {
                command(&mut p, c, false, true);
            }
        }
        check(&mut p, Step::Current(pass, true), shot("alternate"));
        p.push(Step::SaveReadback);
    }
    p.push(Step::Finish);
    assert!(p.len() < END - START);
    p
}
fn ctrl_a() -> egui::Event {
    egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    }
}
impl LabApp {
    fn seed_native_target(&self, target: &Target) -> Option<egui::Rect> {
        match target {
            Target::Try => self.metrics.example_action,
            Target::Assignment => self.metrics.assignment_action,
            Target::Example => self.metrics.lesson_tabs.get(1).copied(),
            Target::Input => self.metrics.terminal,
            Target::Run => self.metrics.terminal_run,
            Target::Insert => self.metrics.insert,
            Target::Picker => self.metrics.terminal_example_picker,
            Target::Choice(index) => self
                .metrics
                .terminal_example_choices
                .iter()
                .find(|(i, _)| i == index)
                .map(|(_, r)| *r),
            Target::Field(name) => self
                .metrics
                .seed_fields
                .iter()
                .find(|(key, _, _)| key == name)
                .map(|(_, r, _)| *r),
            Target::Save => self.metrics.editor_save,
            Target::Check => self.metrics.check,
            Target::Reset => self.metrics.reset,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Learn => self.metrics.mode_learn,
        }
    }
    fn seed_native_practice(&self, i: usize) -> Value {
        let w = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap();
        if let Some(l) = &w.seed_dns_learning {
            serde_json::to_value(l).unwrap()
        } else {
            serde_json::to_value(w.seed_mount_learning.as_ref().unwrap()).unwrap()
        }
    }
    fn seed_native_practice_checks(&self, i: usize) -> Vec<(bool, String)> {
        let s = &self.scenarios[i];
        let w = self.progress.attempts[&s.id].workspace.as_ref().unwrap();
        if let Some(l) = &w.seed_dns_learning {
            l.checks(s.seed_dns_teaching.as_ref().unwrap())
        } else {
            w.seed_mount_learning
                .as_ref()
                .unwrap()
                .checks(s.seed_mount_teaching.as_ref().unwrap())
        }
    }
    fn seed_native_geometry(&self, ctx: &egui::Context, scale: f32, stage: usize) -> bool {
        let zoom = ctx.zoom_factor();
        let pixels = ctx.pixels_per_point();
        let rect = ctx.content_rect();
        assert!(pixels.is_finite() && pixels > 0. && rect.is_finite());
        let (tries, stable) = ctx.data_mut(|d| {
            let key = egui::Id::new(("seed-native-geometry", stage));
            let (tries, old, stable) =
                d.get_temp::<(u8, egui::Rect, u8)>(key)
                    .unwrap_or((0, egui::Rect::NOTHING, 0));
            let stable = if old == rect && (zoom - scale).abs() < 0.01 {
                stable + 1
            } else {
                0
            };
            d.insert_temp(key, (tries + 1, rect, stable));
            (tries + 1, stable)
        });
        assert!(tries < 30, "bounded real font/window geometry settling");
        if stable < 2 {
            ctx.request_repaint();
        }
        stable >= 2
    }
    fn seed_native_start(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            let done = self.click_visible_smoke_control(ctx, None);
            assert_eq!(
                self.selected,
                Some(i),
                "real Start selected the requested lab"
            );
            return done;
        }
        assert!(self.selected.is_none());
        let visible: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| {
                catalog::matches(
                    s,
                    &self.filter,
                    self.progress.attempts.contains_key(&s.id),
                    self.progress
                        .attempts
                        .get(&s.id)
                        .is_some_and(|a| progress::current_solved(a, s)),
                    self.progress.favorites.contains(&s.id),
                )
            })
            .collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, id);
        assert_eq!(
            self.metrics.card_actions.len(),
            1,
            "one real card before Start click"
        );
        self.click_visible_smoke_control(ctx, self.metrics.card_actions.first().map(|r| r.0))
    }
    pub(super) fn seed_teaching_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!((START..END).contains(&stage));
        let p = plan(&self.scenarios);
        let offset = stage - START;
        let step = &p[offset];
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Filter(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let scenario = self.scenarios[i].clone();
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("seed-native {stage} {id} {step:?}");
        }
        match step {
            Step::Library => {
                let key = egui::Id::new(("native-click-phases", stage));
                let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
                let done = if phase >= 2 {
                    self.click_visible_smoke_control(ctx, None)
                } else if self.selected.is_some() {
                    self.click_visible_smoke_control(ctx, self.metrics.back)
                } else {
                    true
                };
                if !done {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            Step::Filter(_) => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.search = scenario.title.clone();
                self.filter.max_minutes = 120;
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            Step::Start => {
                if !self.seed_native_start(ctx, id) {
                    return (false, None);
                }
            }
            Step::Wait => {}
            Step::Click(target) => {
                let ready = if matches!(target, Target::Choice(_)) {
                    self.mac_popup_choice(
                        ctx,
                        self.seed_native_target(target),
                        self.metrics.terminal_example_popup,
                    )
                } else {
                    self.click_visible_smoke_control(ctx, self.seed_native_target(target))
                };
                if !ready {
                    return (false, None);
                }
            }
            Step::Text(text) => {
                let expected =
                    if let Some(Step::Click(Target::Field(name))) = p.get(offset.wrapping_sub(1)) {
                        Some(
                            self.metrics
                                .seed_fields
                                .iter()
                                .find(|(key, _, _)| key == name)
                                .unwrap()
                                .2,
                        )
                    } else {
                        self.metrics.focus
                    };
                assert!(expected.is_some());
                assert_eq!(
                    ctx.memory(|m| m.focused()),
                    expected,
                    "real input owns focus before typing"
                );
                self.pending_events = vec![ctrl_a(), egui::Event::Text(text.clone())];
            }
            Step::BeforeInsert => {
                let a = &self.progress.attempts[id];
                let value = (
                    self.seed_native_practice(i),
                    serde_json::to_value(&a.state).unwrap(),
                    a.transcript.clone(),
                    a.workspace.as_ref().unwrap().command.clone(),
                );
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("seed-native-insert"), value));
            }
            Step::InsertOnly(index) => {
                let before = ctx
                    .data(|d| {
                        d.get_temp::<(Value, Value, Vec<String>, String)>(egui::Id::new(
                            "seed-native-insert",
                        ))
                    })
                    .unwrap();
                let current = self.seed_native_practice(i);
                let a = &self.progress.attempts[id];
                assert_eq!(current["selected_action"], *index);
                assert_eq!(current["current"], before.0["current"]);
                assert_eq!(current["transcript"], before.0["transcript"]);
                assert_eq!(serde_json::to_value(&a.state).unwrap(), before.1);
                assert_eq!(a.transcript, before.2);
                assert_eq!(a.workspace.as_ref().unwrap().command, before.3);
                match &actions(&scenario, 0)[*index] {
                    Action::Command(text) => assert_eq!(current["command"], text.as_str()),
                    Action::Save(fields) => {
                        for (key, value) in fields {
                            let key = if key.ends_with("address") {
                                "address"
                            } else if key.ends_with("serial") {
                                "serial"
                            } else {
                                "device"
                            };
                            assert_eq!(current["draft"][key], value.as_str());
                        }
                    }
                }
            }
            Step::Ran(command, practice, success) => {
                let a = &self.progress.attempts[id];
                if *practice {
                    let current = self.seed_native_practice(i);
                    assert_eq!(current["command"], "");
                    let marker = format!("practice # {command}");
                    assert!(
                        current["transcript"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|r| r.as_str() == Some(marker.as_str()))
                    );
                    assert!(
                        !current["transcript"]
                            .as_array()
                            .unwrap()
                            .last()
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .starts_with("Command failed:")
                    );
                    assert_eq!(a.state, scenario.initial);
                    assert!(!progress::current_solved(a, &scenario));
                } else {
                    assert!(self.input.is_empty());
                    let row = a.transcript.last().unwrap();
                    assert!(row.contains(command));
                    if !success {
                        assert!(
                            row.contains("fstab references an unknown device"),
                            "actual terminal error {row}"
                        );
                        assert_eq!(a.state, scenario.initial);
                    } else {
                        assert!(!row.contains("fstab references an unknown device"));
                    }
                }
            }
            Step::Practice(expected) => {
                if n == 0 && *expected == [false, false] {
                    let example = self.seed_native_practice(i);
                    assert_eq!(example["current"]["saved"], "198.51.100.45");
                    assert_eq!(example["current"]["serial"], 43);
                    assert_eq!(example["current"]["served"], "198.51.100.44");
                    assert_eq!(example["current"]["loaded_serial"], 42);
                    assert!(
                        example["transcript"]
                            .as_array()
                            .unwrap()
                            .last()
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .contains("mirror.lab.test IN A 198.51.100.44")
                    );
                }
                let rows = self.seed_native_practice_checks(i);
                assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), *expected);
                assert_eq!(
                    self.metrics
                        .seed_feedback
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    *expected
                );
                assert!(
                    self.metrics.seed_summary.is_some() && self.metrics.seed_explanation.is_some()
                );
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            Step::Current(expected, pass) => {
                let a = &self.progress.attempts[id];
                assert_eq!(
                    engine::checks(&a.state, &scenario.goal)
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    *expected
                );
                assert_eq!(progress::current_solved(a, &scenario), *pass);
                assert_eq!(self.metrics.completion, *pass);
                if !pass {
                    assert!(self.metrics.feedback.is_some());
                }
                if expected.iter().all(|v| *v) && !pass {
                    assert!(
                        self.notice.is_some(),
                        "failed Save keeps visible diagnostic"
                    );
                    let before = ctx
                        .data(|d| {
                            d.get_temp::<(Value, BTreeMap<String, Value>, BTreeSet<String>)>(
                                egui::Id::new("seed-native-reset"),
                            )
                        })
                        .unwrap();
                    assert_eq!(
                        serde_json::to_value(&a.state).unwrap(),
                        before.0["state"],
                        "failed Save cannot alter simulation"
                    );
                    let w = a.workspace.as_ref().unwrap();
                    assert_eq!(
                        if n == 0 {
                            w.dns_serial.as_deref()
                        } else {
                            w.mount_device.as_deref()
                        },
                        Some("unfinished invalid")
                    );
                }
            }
            Step::Snapshot => {
                let a = serde_json::to_value(&self.progress.attempts[id]).unwrap();
                let others: BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(key, _)| key.as_str() != id)
                    .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
                    .collect();
                let favorites = self.progress.favorites.clone();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("seed-native-reset"), (a, others, favorites))
                });
            }
            Step::CancelExact | Step::ResetExact => {
                let record = ctx
                    .data(|d| {
                        d.get_temp::<(Value, BTreeMap<String, Value>, BTreeSet<String>)>(
                            egui::Id::new("seed-native-reset"),
                        )
                    })
                    .unwrap();
                let a = &self.progress.attempts[id];
                if matches!(step, Step::CancelExact) {
                    assert_eq!(serde_json::to_value(a).unwrap(), record.0);
                } else {
                    assert_eq!(a.state, scenario.initial);
                    assert!(a.transcript.is_empty());
                    assert!(!progress::current_solved(a, &scenario));
                    let current = self.seed_native_practice(i);
                    assert_eq!(current["current"], current["fixture"]);
                    assert_eq!(current["command"], "");
                    assert!(current["transcript"].as_array().unwrap().is_empty());
                }
                assert!(!self.reset_pending);
                assert_eq!(self.progress.favorites, record.2);
                for (id, expected) in record.1 {
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[&id]).unwrap(),
                        expected
                    );
                }
            }
            Step::Drafts => {
                let a = &self.progress.attempts[id];
                let w = a.workspace.as_ref().unwrap();
                let example = self.seed_native_practice(i);
                assert_eq!(w.command, "unfinished assignment command");
                assert_eq!(example["command"], "unfinished practice command");
                assert_eq!(
                    example["draft"][if n == 0 { "serial" } else { "device" }],
                    "unfinished practice"
                );
                assert_eq!(
                    if n == 0 {
                        w.dns_serial.as_deref()
                    } else {
                        w.mount_device.as_deref()
                    },
                    Some("unfinished assignment")
                );
                let value = serde_json::to_value(a).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("seed-native-resume"), value));
            }
            Step::ResumeExact => {
                let mut before = ctx
                    .data(|d| d.get_temp::<Value>(egui::Id::new("seed-native-resume")))
                    .unwrap();
                before["visited"] = serde_json::json!(self.progress.clock);
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[id]).unwrap(),
                    before
                );
                assert_eq!(self.selected, Some(i));
                assert_eq!(self.seed_view(i), teaching::View::Example);
            }
            Step::SaveReadback => {
                self.save();
                assert!(self.storage_error.is_none());
                let bytes = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let raw: Value = serde_json::from_slice(&bytes).unwrap();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let portable = serde_json::to_value(&self.progress).unwrap();
                assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
                let mut exact = portable;
                exact["migration_receipts"] =
                    serde_json::to_value(&loaded.migration_receipts).unwrap();
                assert_eq!(raw, exact);
                assert_eq!(
                    progress::read_bounded(&self.path, 16_000_000).unwrap(),
                    bytes
                );
            }
            Step::Theme(light) => {
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                self.scale = if *light { 1.5 } else { 2. };
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            Step::Geometry(scale) => {
                if !self.seed_native_geometry(ctx, *scale, stage) {
                    return (false, None);
                }
            }
            Step::Mode(challenge) => {
                assert_eq!(self.progress.attempts[id].challenge, *challenge);
                assert_eq!(self.seed_view(i), teaching::View::Assignment);
                if *challenge {
                    assert!(self.metrics.lesson_tabs.is_empty());
                    assert!(!self.help_open);
                }
            }
            Step::Shot(name, feedback) => {
                if name.ends_with("introduction") {
                    assert_eq!(self.seed_view(i), teaching::View::Introduction);
                    assert!(
                        self.metrics.example_action.is_some() && self.metrics.terminal.is_none()
                    );
                }
                let target = if name.ends_with("simulator-caption") {
                    self.metrics.seed_explanation
                } else if *feedback {
                    self.metrics
                        .seed_feedback
                        .first()
                        .map(|(_, r)| *r)
                        .or_else(|| {
                            self.metrics.feedback.map(|r| {
                                egui::Rect::from_min_size(
                                    r.min,
                                    egui::vec2(
                                        r.width(),
                                        r.height().min(
                                            self.metrics
                                                .check
                                                .map_or(r.height(), |c| 2. * c.height()),
                                        ),
                                    ),
                                )
                            })
                        })
                } else {
                    None
                };
                if let Some(r) = target
                    && !self.reveal_smoke_control(ctx, r)
                {
                    return (false, None);
                }
                return (true, Some(name.clone()));
            }
            Step::Finish => {
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                println!(
                    "PASS two original DNS/storage native teaching flows: actual inputs, separate examples, wrong/partial/alternate/current checks, exact drafts/save/readback/navigation resume/reset and Light150/Dark200. No cold-process, novice or cross-platform claim."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200. / a.scale, 850. / a.scale),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
    }
    fn draw(ctx: &egui::Context, a: &mut LabApp) {
        for _ in 0..3 {
            frame(ctx, a, vec![]);
        }
    }
    fn click(ctx: &egui::Context, a: &mut LabApp, target: Target) {
        for _ in 0..24 {
            let r = a
                .seed_native_target(&target)
                .expect("real teaching control");
            let clip = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if smoke_control_visible(r, clip, ctx.pixels_per_point()) {
                for pressed in [true, false] {
                    frame(
                        ctx,
                        a,
                        vec![
                            egui::Event::PointerMoved(r.center()),
                            egui::Event::PointerButton {
                                pos: r.center(),
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: Default::default(),
                            },
                        ],
                    );
                }
                draw(ctx, a);
                return;
            }
            let delta = if r.bottom() > clip.bottom() {
                clip.bottom() - r.bottom()
            } else {
                clip.top() - r.top()
            };
            frame(
                ctx,
                a,
                vec![
                    egui::Event::PointerMoved(clip.center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., delta),
                        modifiers: Default::default(),
                    },
                ],
            );
            draw(ctx, a);
        }
        panic!("real control must be reachable: {target:?}");
    }
    #[test]
    fn seed_native_plan_is_bounded_fixed_and_has_no_simulation_publish_path() {
        let p = plan(&pack::bundled().unwrap());
        assert!(START > 23999 && START + p.len() < END);
        assert_eq!(p.iter().filter(|s| matches!(s, Step::Filter(_))).count(), 4);
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::ResumeExact)).count(),
            2
        );
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::ResetExact)).count(),
            2
        );
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::Mode(true))).count(),
            4
        );
        assert!(
            p.iter()
                .all(|s| !matches!(s, Step::Text(x) if x.len() > 4096))
        );
        let source = include_str!("seed_teaching.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(
            !source.contains("engine::execute")
                && !source.contains(".state =")
                && !source.contains(".launch(")
        );
    }
    #[test]
    fn seed_native_actual_geometry_phase_has_no_context_lock_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        let p = plan(&app.scenarios);
        for scale in [1.5, 2.] {
            let offset = p
                .iter()
                .position(|s| matches!(s, Step::Geometry(v) if *v == scale))
                .unwrap();
            ctx.set_zoom_factor(scale);
            app.scale = scale;
            app.smoke_stage = START + offset;
            let mut done = false;
            for _ in 0..4 {
                let _ = ctx.run(egui::RawInput::default(), |ctx| {
                    done = app.seed_teaching_native(ctx, START + offset).0;
                });
            }
            assert!(done);
        }
    }
    #[test]
    fn seed_native_insert_run_keyed_typing_and_failed_save_use_real_controls() {
        for (n, id) in IDS.iter().enumerate() {
            let ctx = egui::Context::default();
            let mut a = LabApp::from_context(&ctx, true, false);
            a.smoke = false;
            let i = a.scenarios.iter().position(|s| s.id == *id).unwrap();
            a.launch(i);
            draw(&ctx, &mut a);
            assert_eq!(a.seed_view(i), teaching::View::Introduction);
            click(&ctx, &mut a, Target::Try);
            let before = a.seed_native_practice(i);
            let assignment = a.progress.attempts[*id].state.clone();
            click(&ctx, &mut a, Target::Insert);
            let filled = a.seed_native_practice(i);
            assert_eq!(filled["current"], before["current"]);
            assert_eq!(filled["transcript"], before["transcript"]);
            assert!(!filled["command"].as_str().unwrap().is_empty());
            click(&ctx, &mut a, Target::Run);
            let ran = a.seed_native_practice(i);
            assert_eq!(ran["command"], "");
            assert!(ran["transcript"].as_array().unwrap().len() >= 2);
            assert_eq!(a.progress.attempts[*id].state, assignment);
            let field_name = if n == 0 {
                "seed-dns-serial"
            } else {
                "seed-mount-device"
            };
            let expected = a
                .metrics
                .seed_fields
                .iter()
                .find(|(key, _, _)| key == field_name)
                .unwrap()
                .2;
            click(&ctx, &mut a, Target::Field(field_name.into()));
            assert_eq!(ctx.memory(|m| m.focused()), Some(expected));
            frame(
                &ctx,
                &mut a,
                vec![ctrl_a(), egui::Event::Text("unfinished invalid".into())],
            );
            draw(&ctx, &mut a);
            click(&ctx, &mut a, Target::Save);
            click(&ctx, &mut a, Target::Check);
            let invalid = a.seed_native_practice(i);
            assert_eq!(
                invalid["draft"][if n == 0 { "serial" } else { "device" }],
                "unfinished invalid"
            );
            assert_eq!(invalid["current"], ran["current"]);
            assert_eq!(
                a.metrics
                    .seed_feedback
                    .iter()
                    .map(|r| r.0)
                    .collect::<Vec<_>>(),
                [false]
            );
            assert_eq!(a.progress.attempts[*id].state, assignment);
            assert!(!progress::current_solved(
                &a.progress.attempts[*id],
                &a.scenarios[i]
            ));
        }
    }
}
