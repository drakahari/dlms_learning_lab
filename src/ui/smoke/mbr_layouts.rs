//! Fixed native review over registered MBR data; no learner/host execution.
use super::*;
use dlms_learning_lab::{partition_fs as core, partition_fs_workspace as work};
use std::time::Duration;
pub(in crate::ui) const IDS: [&str; 3] = [
    "lpic1-mbr-primary-plan",
    "lpic1-mbr-logical-space",
    "lpic1-mbr-storage-handover",
];
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Bookmark,
    Unrun,
    Current(bool),
    Practice(bool),
    Reader,
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
    bookmark: Option<core::State>,
}
fn session(a: &LabApp) -> &work::Session {
    let engine::State::PartitionFs { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn reader(a: &LabApp) -> Option<(egui::Rect, egui::Rect)> {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::Goal::PartitionFs { definition } = &s.goal else {
        panic!()
    };
    let expected = definition
        .initial
        .disks
        .iter()
        .find_map(|(id, d)| {
            d.partitions().find_map(|p| {
                if let core::Content::Ext4 { files, .. } = &p.content {
                    files.iter().next().map(|(name, file)| {
                        (
                            format!("{id}{} · {name}:", p.number),
                            file.text.trim_end().to_owned(),
                        )
                    })
                } else {
                    None
                }
            })
        })
        .unwrap();
    a.metrics
        .partition_fs
        .text_rects
        .iter()
        .find(|(text, _, _)| text.contains(&expected.0) && text.contains(&expected.1))
        .map(|(_, r, c)| (*r, *c))
}
fn control(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "challenge-header" => a.metrics.mode_challenge,
        "learn-header" => a.metrics.mode_learn,
        "reader" => reader(a).map(|r| r.0),
        "feedback" => a
            .metrics
            .partition_fs
            .rows
            .iter()
            .map(|r| r.1)
            .reduce(|a, b| a.union(b)),
        _ => a
            .metrics
            .partition_fs
            .fields
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| {
                a.metrics
                    .partition_fs
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            })
            .or_else(|| {
                a.metrics
                    .partition_fs
                    .choices
                    .iter()
                    .find(|r| r.0 == key)
                    .map(|r| r.1)
            }),
    }
}
fn choice(ops: &mut Vec<Op>, pick: &str, key: String) {
    ops.extend([Op::Click(pick.into()), Op::Click(key)]);
}
fn apply(ops: &mut Vec<Op>, op: &work::Operation) {
    match op {
        work::Operation::Mbr {
            operation: core::mbr::Action::Create { .. },
        } => ops.push(Op::Click("create-mbr".into())),
        work::Operation::Mbr {
            operation:
                core::mbr::Action::Add {
                    role,
                    start,
                    end,
                    type_hint,
                    ..
                },
        } => {
            choice(ops, "mbr-role", format!("role-{role:?}"));
            ops.extend([
                Op::Field("start".into(), start.to_string()),
                Op::Field("end".into(), end.to_string()),
            ]);
            if *role != core::mbr::Kind::Extended {
                choice(ops, "hint", format!("hint-{type_hint:?}"));
            }
            ops.push(Op::Click("add".into()));
        }
        work::Operation::Format {
            partition,
            format,
            label,
            uuid,
        } => {
            choice(
                ops,
                "partition",
                format!(
                    "partition-{}",
                    partition.trim_start_matches("/dev/").get(3..).unwrap()
                ),
            );
            choice(ops, "content-type", format!("format-{format:?}"));
            ops.extend([
                Op::Field("label".into(), label.clone()),
                Op::Field("uuid".into(), uuid.clone().unwrap_or_default()),
                Op::Click("format".into()),
            ]);
        }
        _ => panic!("Not an authored MBR operation"),
    }
}
fn route(labs: &[pack::Scenario]) -> Vec<Op> {
    let mut ops = Vec::new();
    for (n, id) in IDS.iter().enumerate() {
        let s = labs.iter().find(|s| s.id == *id).unwrap();
        let engine::Goal::PartitionFs { definition: d } = &s.goal else {
            panic!()
        };
        ops.extend([
            Op::Open(id.to_string()),
            Op::Shot(format!("raw-{id}-introduction"), "reset-header".into()),
            Op::Click("practice".into()),
        ]);
        for (i, step) in d.lesson.worked.iter().enumerate() {
            choice(&mut ops, "example-picker", format!("example-{i}"));
            ops.extend([Op::Bookmark, Op::Click("fill-example".into()), Op::Unrun]);
            ops.push(Op::Click(
                match step.operation {
                    work::Operation::Mbr {
                        operation: core::mbr::Action::Create { .. },
                    } => "create-mbr",
                    work::Operation::Mbr { .. } => "add",
                    work::Operation::Format { .. } => "format",
                    _ => panic!(),
                }
                .into(),
            ));
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Current(false),
            Op::Shot(format!("raw-{id}-practice-pass"), "feedback".into()),
            Op::Click("experiment".into()),
        ]);
        apply(&mut ops, &d.lesson.experiment);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(format!("raw-{id}-practice-adverse"), "feedback".into()),
            Op::Click("reset-practice".into()),
        ]);
        for (i, step) in d.lesson.worked.iter().enumerate() {
            choice(&mut ops, "example-picker", format!("example-{i}"));
            ops.push(Op::Click("fill-example".into()));
            ops.push(Op::Click(
                match step.operation {
                    work::Operation::Mbr {
                        operation: core::mbr::Action::Create { .. },
                    } => "create-mbr",
                    work::Operation::Mbr { .. } => "add",
                    work::Operation::Format { .. } => "format",
                    _ => panic!(),
                }
                .into(),
            ));
        }
        ops.extend([
            Op::Practice(true),
            Op::Current(false),
            Op::Click("assignment".into()),
            Op::Shot(format!("raw-{id}-assignment"), "mbr-role".into()),
        ]);
        apply(&mut ops, &d.reference[0]);
        ops.extend([
            Op::Click("check".into()),
            Op::Current(false),
            Op::Shot(format!("raw-{id}-partial-feedback"), "feedback".into()),
            Op::Save,
            Op::Fresh,
        ]);
        for op in &d.reference[1..] {
            apply(&mut ops, op);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Current(true),
            Op::Shot(format!("raw-{id}-independent-pass"), "feedback".into()),
            Op::Click("inspect".into()),
            Op::Reader,
            Op::Shot(format!("raw-{id}-retained-reader"), "reader".into()),
            Op::Click("inspect".into()),
            Op::Save,
            Op::Fresh,
            Op::Current(true),
        ]);
        let extra = work::Operation::Mbr {
            operation: core::mbr::Action::Add {
                disk: "/dev/vda".into(),
                role: if n == 0 {
                    core::mbr::Kind::Primary
                } else {
                    core::mbr::Kind::Logical
                },
                start: if n == 0 { 49152 } else { 81920 },
                end: if n == 0 { 53247 } else { 86015 },
                type_hint: core::TypeHint::LinuxData,
            },
        };
        apply(&mut ops, &extra);
        ops.extend([
            Op::Current(false),
            Op::Click("check".into()),
            Op::Shot(format!("raw-{id}-current-invalidated"), "feedback".into()),
            Op::Theme(true),
            Op::Field("start".into(), "12345678901234567890".into()),
            Op::Shot(format!("raw-{id}-light150-field"), "start".into()),
            Op::Theme(false),
            Op::Shot(format!("raw-{id}-dark200-field"), "start".into()),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Current(false),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(format!("raw-{id}-challenge"), "reset-header".into()),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
            Op::Click("help".into()),
            Op::Click("references".into()),
            Op::Shot(format!("raw-{id}-references"), "references".into()),
        ]);
    }
    ops.extend([
        Op::Open("lpic1-disk-safe-gap".into()),
        Op::Click("assignment".into()),
        Op::Shot("raw-retained-gpt".into(), "start".into()),
    ]);
    assert!(ops.len() < 1024);
    ops
}
impl LabApp {
    fn mbr_click(&mut self, ctx: &egui::Context, key: &str, r: &mut Route) -> bool {
        let rect = control(self, key).unwrap_or_else(|| panic!("Missing MBR control {key}"));
        let visible = smoke_control_visible(
            rect,
            self.metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect()),
            ctx.pixels_per_point(),
        );
        let done = self.click_visible_smoke_control(ctx, Some(rect));
        if !visible && !done {
            r.not_before = Some(Instant::now() + Duration::from_millis(200));
        }
        done
    }
    pub(in crate::ui) fn mbr_layout_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("mbr-fixed-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "MBR_NATIVE_COMPLETE: actual native fields/actions, separate practice, partial save/load, retained data, current invalidation, reset, Challenge and enlarged controls; no host devices"
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
        self.smoke_stage = 120000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("MBR_NATIVE_STEP {} {op:?}", r.at);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "MBR native action stalled: {} {op:?}",
            r.at
        );
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
            Op::Click(name) => self.mbr_click(ctx, &name, &mut r),
            Op::Field(name, value) => {
                if r.phase == 0 {
                    if self.mbr_click(ctx, &name, &mut r) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let f = self
                        .metrics
                        .partition_fs
                        .fields
                        .iter()
                        .find(|f| f.0 == name)
                        .unwrap();
                    assert_eq!(ctx.memory(|m| m.focused()), Some(f.2));
                    let mods = egui::Modifiers {
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
                            modifiers: mods,
                        });
                    }
                    self.pending_events.push(egui::Event::Paste(value));
                    r.phase = 2;
                    false
                } else {
                    let w = session(self);
                    let e = w.selected_editor(w.view == teaching::View::Example);
                    let d = &e.drafts[&e.selected];
                    assert_eq!(
                        match name.as_str() {
                            "start" => &d.start,
                            "end" => &d.end,
                            "label" => &d.label,
                            "uuid" => &d.uuid,
                            _ => panic!(),
                        },
                        &value
                    );
                    true
                }
            }
            Op::Bookmark => {
                r.bookmark = Some(session(self).example.clone());
                true
            }
            Op::Unrun => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::PartitionFs { definition } = &s.goal else {
                    panic!()
                };
                assert_eq!(&session(self).example, r.bookmark.as_ref().unwrap());
                assert_eq!(session(self).current, definition.initial);
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
            Op::Current(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&s.id], s),
                    want,
                    "{} current outcome",
                    s.id
                );
                true
            }
            Op::Practice(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::PartitionFs { definition } = &s.goal else {
                    panic!()
                };
                assert_eq!(
                    definition
                        .practice_checks(session(self))
                        .iter()
                        .all(|r| r.0),
                    want
                );
                true
            }
            Op::Reader => {
                let viewport = self
                    .metrics
                    .partition_fs
                    .files_viewport
                    .expect("Actual expanded reader viewport");
                if !self.reveal_smoke_control(ctx, viewport) {
                    r.not_before = Some(Instant::now() + Duration::from_millis(200));
                    false
                } else if let Some((rect, clip)) = reader(self) {
                    if smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
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
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                assert_eq!(
                    serde_json::to_value(progress::load(&self.path, &self.scenarios).unwrap())
                        .unwrap(),
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
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1268., 632.)));
                self.lesson_scroll_reset = true;
                r.not_before = Some(Instant::now() + Duration::from_millis(300));
                true
            }
            Op::Challenge(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(self.progress.attempts[&s.id].challenge, want);
                true
            }
            Op::Shot(name, target) => {
                let rect = control(self, &target).expect("Actual MBR capture target");
                let clip = if target == "reader" {
                    reader(self).unwrap().1
                } else {
                    self.metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect())
                };
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
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> (egui::Context, LabApp) {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        app.smoke = false;
        let i = app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
        app.launch(i);
        (ctx, app)
    }
    #[test]
    fn mbr_route_preserves_three_authored_flows_and_actual_partial_resume() {
        let (_, app) = app();
        let ops = route(&app.scenarios);
        assert!(ops.len() < 1024);
        for id in IDS {
            assert!(ops.iter().any(|o| matches!(o, Op::Open(s) if s==id)));
            for suffix in [
                "introduction",
                "practice-pass",
                "practice-adverse",
                "assignment",
                "partial-feedback",
                "independent-pass",
                "retained-reader",
                "current-invalidated",
                "light150-field",
                "dark200-field",
                "challenge",
            ] {
                assert!(
                    ops.iter()
                        .any(|o| matches!(o, Op::Shot(s, _) if s==&format!("raw-{id}-{suffix}")))
                );
            }
        }
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 6);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 6);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Reader)).count(), 3);
        assert!(
            ops.iter()
                .any(|o| matches!(o, Op::Open(s) if s=="lpic1-disk-safe-gap"))
        );
    }
    #[test]
    fn mbr_actual_capture_phase_is_context_safe() {
        let (ctx, mut app) = app();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1400., 950.),
                )),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
        let rect = control(&app, "reset-header").unwrap();
        let clip = app
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        assert!(smoke_control_visible(rect, clip, ctx.pixels_per_point()));
        let key = egui::Id::new("mbr-fixed-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("unit-mbr-capture".into(), "reset-header".into())],
                    phase: 1,
                    stable: Some((rect, clip)),
                    stable_frames: 4,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| app.mbr_layout_tick(ctx));
        assert!(app.capture_pending);
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().phase, 2);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn mbr_wait_deadline_does_not_spend_visibility_attempts() {
        let (ctx, mut app) = app();
        let key = egui::Id::new("mbr-fixed-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Click("start".into())],
                    not_before: Some(Instant::now() + Duration::from_secs(60)),
                    ..Default::default()
                },
            )
        });
        for _ in 0..50 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| app.mbr_layout_tick(ctx));
        }
        let r = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
        assert_eq!(r.at, 0);
        assert_eq!(r.frames, 0);
        assert!(app.pending_events.is_empty());
    }
}
