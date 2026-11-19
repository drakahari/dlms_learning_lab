//! Fixed native event route over the assembly capstone; no solved-state injection.
use super::*;
use dlms_learning_lab::raid_assembly_workspace as work;
use std::time::Duration;
const IDS: [&str; 1] = ["lpic2-raid1-assembly-policy"];
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Status(u8),
    Notice(bool),
    Solved(bool),
    Practice(bool),
    Save,
    Fresh,
    Scale(bool, f32),
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
    stable: Option<(egui::Rect, egui::Rect)>,
    stable_frames: u8,
}
fn action(ops: &mut Vec<Op>, op: &work::Operation, status: u8) {
    match op {
        work::Operation::Save { text } => ops.extend([
            Op::Field("editor".into(), text.0.clone()),
            Op::Click("save".into()),
            Op::Status(status),
        ]),
        work::Operation::Command { text } => ops.extend([
            Op::Field("command".into(), text.clone()),
            Op::Click("run".into()),
            Op::Status(status),
        ]),
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for id in IDS {
        let source = labs.iter().find(|s| s.id == id).unwrap();
        let engine::Goal::RaidAssembly { definition: d } = &source.goal else {
            panic!()
        };
        ops.extend([
            Op::Open(id.into()),
            Op::Shot(
                format!("raid_assembly-{id}-introduction"),
                "reset-header".into(),
            ),
            Op::Shot(format!("raid_assembly-{id}-worked-example"), "try".into()),
            Op::Click("example".into()),
            Op::Solved(false),
            Op::Shot(format!("raid_assembly-{id}-guided-start"), "editor".into()),
        ]);
        for step in &d.example_steps {
            action(&mut ops, &step.operation, step.status);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Solved(false),
            Op::Shot(format!("raid_assembly-{id}-example-passed"), "check".into()),
        ]);
        action(&mut ops, &d.experiment.operation, d.experiment.status);
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(false),
            Op::Shot(
                format!("raid_assembly-{id}-example-mistake"),
                "check".into(),
            ),
        ]);
        for step in &d.repair {
            action(&mut ops, &step.operation, step.status);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Practice(true),
            Op::Click("assignment".into()),
            Op::Solved(false),
            Op::Shot(
                format!("raid_assembly-{id}-assignment-unstarted"),
                "editor".into(),
            ),
        ]);
        ops.extend([
            Op::Field("command".into(), "ls".into()),
            Op::Click("run".into()),
            Op::Notice(true),
            Op::Shot(
                format!("raid_assembly-{id}-unsupported-feedback"),
                "notice".into(),
            ),
            Op::Field("command".into(), "cat /etc/mdadm.conf".into()),
            Op::Notice(false),
            Op::Click("run".into()),
            Op::Status(0),
        ]);
        ops.extend([
            Op::Field("editor".into(), "ARRAY /dev/md10 name=unsupported\n".into()),
            Op::Click("save".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(
                format!("raid_assembly-{id}-syntax-feedback"),
                "check".into(),
            ),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
        ]);
        for step in &d.reference {
            action(&mut ops, &step.operation, step.status);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Shot(
                format!("raid_assembly-{id}-assignment-passed"),
                "check".into(),
            ),
            Op::Click("data".into()),
            Op::Shot(format!("raid_assembly-{id}-original-data"), "data".into()),
            Op::Save,
            Op::Fresh,
            Op::Solved(true),
            Op::Field("editor".into(), "# unfinished change\n".into()),
            Op::Solved(false),
            Op::Save,
            Op::Fresh,
            Op::Solved(false),
            Op::Shot(
                format!("raid_assembly-{id}-unsaved-resume"),
                "editor".into(),
            ),
            Op::Scale(true, 2.),
            Op::Shot(
                format!("raid_assembly-{id}-light200-controls"),
                "command".into(),
            ),
            Op::Scale(false, 1.5),
            Op::Shot(
                format!("raid_assembly-{id}-dark150-controls"),
                "command".into(),
            ),
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Solved(false),
            Op::Scale(false, 1.),
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(format!("raid_assembly-{id}-challenge"), "editor".into()),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
            Op::Click("introduction".into()),
            Op::Scale(true, 2.),
            Op::Shot(
                format!("raid_assembly-{id}-light200-introduction"),
                "reset-header".into(),
            ),
            Op::Click("example".into()),
            Op::Shot(
                format!("raid_assembly-{id}-light200-guided"),
                "editor".into(),
            ),
            Op::Click("assignment".into()),
            Op::Shot(
                format!("raid_assembly-{id}-light200-assignment"),
                "editor".into(),
            ),
        ]);
    }
    ops
}
fn session(a: &LabApp) -> &work::Session {
    let source = &a.scenarios[a.selected.unwrap()];
    let engine::State::RaidAssembly { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    simulation
}
fn control(a: &LabApp, key: &str) -> Option<egui::Rect> {
    if key == "notice" {
        return a.metrics.raid_assembly.action_notice;
    }
    match key {
        "reset-header" => return a.metrics.reset,
        "confirm-reset" => return a.metrics.confirm_reset,
        "challenge-header" => return a.metrics.mode_challenge,
        "learn-header" => return a.metrics.mode_learn,
        _ => {}
    }
    a.metrics
        .raid_assembly
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
        .or_else(|| {
            a.metrics
                .raid_assembly
                .fields
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1)
        })
}
impl LabApp {
    fn raid_assembly_click(&mut self, ctx: &egui::Context, key: &str, r: &mut Route) -> bool {
        let rect = control(self, key);
        let done = self.click_visible_smoke_control(ctx, rect);
        if !done {
            r.not_before = Some(Instant::now() + Duration::from_millis(100));
        }
        done
    }
    pub(in crate::ui) fn raid_assembly_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("raid_assembly-fixed-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.at == r.ops.len() {
            println!(
                "RAID_ASSEMBLY_NATIVE_COMPLETE: {} actual actions; separate examples, typed editor save, failed scan/retry, current/future identity/data grading, resume, invalidation, reset, Challenge, light/dark scaled controls",
                r.at
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        if r.not_before.is_some_and(|t| Instant::now() < t) {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(Duration::from_millis(16));
            return;
        }
        r.not_before = None;
        self.smoke_stage = 275000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("RAID_ASSEMBLY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "RAID assembly action stalled {} {op:?}",
            r.at
        );
        let done = match op {
            Op::Open(id) => {
                self.scale = 1.;
                ctx.set_zoom_factor(1.);
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                r.not_before = Some(Instant::now() + Duration::from_millis(300));
                true
            }
            Op::Click(k) => self.raid_assembly_click(ctx, &k, &mut r),
            Op::Field(k, value) => {
                if r.phase == 0 {
                    if self.raid_assembly_click(ctx, &k, &mut r) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    let field = self
                        .metrics
                        .raid_assembly
                        .fields
                        .iter()
                        .find(|f| f.0 == k)
                        .unwrap();
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
                    self.pending_events.push(egui::Event::Paste(value.clone()));
                    r.phase = 2;
                    false
                } else {
                    let s = session(self);
                    let w = s.work(s.view == teaching::View::Example);
                    assert_eq!(
                        if k == "editor" {
                            w.draft.0.as_str()
                        } else {
                            w.command.as_str()
                        },
                        value
                    );
                    true
                }
            }
            Op::Notice(want) => {
                assert_eq!(
                    self.metrics.raid_assembly.action_notice.is_some(),
                    want,
                    "Unsupported feedback must persist across settled frames, then clear after editing"
                );
                true
            }
            Op::Status(status) => {
                let s = session(self);
                assert_eq!(
                    s.work(s.view == teaching::View::Example)
                        .transcript
                        .last()
                        .unwrap()
                        .result
                        .status,
                    status
                );
                assert!(self.notice.is_none(), "{:?}", self.notice);
                true
            }
            Op::Solved(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&s.id], s),
                    want,
                    "{} current grade",
                    s.id
                );
                true
            }
            Op::Practice(want) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::Goal::RaidAssembly { definition: d } = &source.goal else {
                    panic!()
                };
                let s = session(self);
                assert!(s.practice.checked);
                assert_eq!(s.practice.checks(&d.example).iter().all(|r| r.0), want);
                assert_eq!(s.assignment, work::Work::new(&d.assignment).unwrap());
                assert!(!progress::current_solved(
                    &self.progress.attempts[&source.id],
                    source
                ));
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let p = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&p).unwrap(),
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
            Op::Scale(light, scale) => {
                if let Some(id) = ctx.memory(|m| m.focused()) {
                    ctx.memory_mut(|m| m.surrender_focus(id));
                }
                self.focus_input = false;
                self.scale = scale;
                ctx.set_zoom_factor(scale);
                self.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1268., 1308.)));
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
                let rect = control(self, &target).expect("Actual RAID assembly screenshot target");
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
                    } else {
                        self.capture_pending = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::new(name),
                        ));
                        r.phase = 2;
                    }
                    false
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
    fn fixed_route_covers_authored_policy_and_only_explicit_native_operations() {
        let ctx = egui::Context::default();
        let a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let ops = route(&a.scenarios);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 1);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Fresh)).count(), 3);
        assert_eq!(
            ops.iter().filter(|o| matches!(o, Op::Shot(_, _))).count(),
            17
        );
        assert!(ops.iter().any(|o| matches!(o, Op::Status(1))));
        assert!(ops.iter().any(|o| matches!(o, Op::Practice(false))));
    }
    #[test]
    fn actual_capture_phase_requests_screenshot_without_context_reentry_or_work_changes() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a.scenarios.iter().position(|s| s.id == IDS[0]).unwrap();
        a.launch(i);
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 9000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let rect = control(&a, "reset-header").unwrap();
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let before = serde_json::to_value(&a.progress).unwrap();
        let key = egui::Id::new("raid_assembly-fixed-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot(
                        "unit-raid_assembly-capture".into(),
                        "reset-header".into(),
                    )],
                    phase: 1,
                    stable: Some((rect, clip)),
                    stable_frames: 4,
                    ..Default::default()
                },
            )
        });
        let output = ctx.run(egui::RawInput::default(), |ctx| a.raid_assembly_tick(ctx));
        assert!(a.capture_pending);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        assert!(output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
}
