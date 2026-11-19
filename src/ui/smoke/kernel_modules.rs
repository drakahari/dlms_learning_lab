//! Fixed native teaching flow; learner commands remain in the in-process model.
use super::*;
use dlms_learning_lab::kernel_module_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(&'static str),
    Click(&'static str),
    Command(&'static str),
    Config(&'static str),
    Pick(usize),
    Output(&'static str),
    Check(bool, bool),
    Shot(&'static str),
    ResultShot(&'static str),
    WorkedShot(&'static str),
    ThemeDefault,
    Save,
    Fresh,
    Theme(bool),
    Challenge(bool),
    Unchecked(bool),
    Unsaved(bool),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    until: f64,
    done: bool,
}
fn plan() -> Vec<Op> {
    vec![
        Op::Open("lpic1-module-device-driver"),
        Op::Shot("module-driver-introduction"),
        Op::Click("example"),
        Op::Shot("module-driver-practice"),
        Op::Command("lspci -k"),
        Op::Output("Kernel modules: example_reader"),
        Op::Command("insmod /lib/modules/6.9.0-example/kernel/drivers/example_reader.ko"),
        Op::Output("Unknown symbol"),
        Op::Command("dmesg"),
        Op::Output("unresolved dependency"),
        Op::Command("modprobe example_reader"),
        Op::Command("lspci -k"),
        Op::Output("Kernel driver in use: example_reader"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("module-driver-assignment"),
        Op::Command("modprobe lab_sensor queues=2"),
        Op::Check(false, false),
        Op::ResultShot("module-driver-wrong-value"),
        Op::Command("modprobe -r lab_sensor"),
        Op::Command("modprobe lab_sensor"),
        Op::Check(false, true),
        Op::Save,
        Op::Command("rmmod audit_link"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::Command("modprobe audit_link"),
        Op::Check(false, true),
        Op::Theme(true),
        Op::Command("lsmod"),
        Op::Shot("module-driver-light150"),
        Op::ResultShot("module-driver-light150-controls"),
        Op::Theme(false),
        Op::Command("lspci -k"),
        Op::ResultShot("module-driver-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Open("lpic1-module-index-repair"),
        Op::Shot("module-index-introduction"),
        Op::Click("example"),
        Op::Command("modprobe example_reader"),
        Op::Output("not found"),
        Op::Command("modinfo /lib/modules/6.9.0-example/kernel/drivers/example_reader.ko"),
        Op::Output("reader_bus"),
        Op::Command("depmod -a"),
        Op::Check(true, false),
        Op::Command("modprobe example_reader"),
        Op::Check(true, true),
        Op::Click("assignment"),
        Op::Shot("module-index-assignment"),
        Op::Command("insmod /lib/modules/6.8.0-lab/kernel/drivers/sensor_bus.ko"),
        Op::Command("insmod /lib/modules/6.8.0-lab/kernel/drivers/lab_sensor.ko"),
        Op::Check(false, false),
        Op::ResultShot("module-index-one-off-feedback"),
        Op::Command("depmod -a"),
        Op::Check(false, true),
        Op::Save,
        Op::Command("insmod /lib/modules/6.7.0-lab/kernel/drivers/old_sensor.ko"),
        Op::Unchecked(false),
        Op::Command("dmesg"),
        Op::Output("version magic"),
        Op::Check(false, true),
        Op::Theme(true),
        Op::Command("lsmod"),
        Op::ResultShot("module-index-light150-controls"),
        Op::Theme(false),
        Op::Command("lsmod"),
        Op::ResultShot("module-index-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
        Op::Open("lpic2-module-options-reload"),
        Op::Shot("module-options-introduction"),
        Op::Click("example"),
        Op::Shot("module-options-practice"),
        Op::Command("cat /sys/module/example_reader/parameters/queues"),
        Op::Output("1\n"),
        Op::Pick(2),
        Op::Click("insert"),
        Op::Click("save-options"),
        Op::Command("modprobe example_reader"),
        Op::Command("cat /sys/module/example_reader/parameters/queues"),
        Op::Output("1\n"),
        Op::Check(true, false),
        Op::ResultShot("module-options-saved-not-effective"),
        Op::Command("modprobe -r example_reader"),
        Op::Command("modprobe example_reader"),
        Op::Command("cat /sys/module/example_reader/parameters/queues"),
        Op::Output("3\n"),
        Op::Check(true, true),
        Op::Click("assignment"),
        Op::Shot("module-options-assignment"),
        Op::Config("options lab_sensor queues=4\n"),
        Op::Click("save-options"),
        Op::Check(false, false),
        Op::ResultShot("module-options-current-feedback"),
        Op::Command("modprobe -r lab_sensor"),
        Op::Command("modprobe lab_sensor"),
        Op::Check(false, true),
        Op::Save,
        Op::Config("options lab_sensor queues=2\n"),
        Op::Unchecked(false),
        Op::Check(false, false),
        Op::ResultShot("module-options-unsaved-feedback"),
        Op::Click("save-options"),
        Op::Check(false, false),
        Op::Config("options lab_sensor queues=4\n"),
        Op::Click("save-options"),
        Op::Check(false, true),
        Op::Save,
        Op::Theme(true),
        Op::Command("cat /sys/module/lab_sensor/parameters/queues"),
        Op::Shot("module-options-light150"),
        Op::ResultShot("module-options-light150-controls"),
        Op::Theme(false),
        Op::Command("lsmod"),
        Op::Shot("module-options-dark200"),
        Op::ResultShot("module-options-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
    ]
}
fn names_plan() -> Vec<Op> {
    vec![
        Op::Open("lpic2-module-name-handover"),
        Op::Shot("names-introduction"),
        Op::Click("example"),
        Op::Shot("names-practice"),
        Op::Command("lspci -k"),
        Op::Command("modprobe demo_capture"),
        Op::Command("lspci -k"),
        Op::Output("Kernel modules: example_reader"),
        Op::Check(true, false),
        Op::ResultShot("names-practice-wrong-provider"),
        Op::Pick(3),
        Op::Click("insert"),
        Op::Unsaved(true),
        Op::Shot("names-practice-filled-not-saved"),
        Op::Click("save-options"),
        Op::Command("modprobe demo_capture"),
        Op::Command("lsmod"),
        Op::Output("example_reader"),
        Op::Command("modprobe -r example_reader"),
        Op::Command("modprobe demo_compat"),
        Op::Command("lspci -k"),
        Op::Output("Kernel driver in use: example_reader"),
        Op::Check(true, true),
        Op::Click("reset-example"),
        Op::Unchecked(true),
        Op::Click("assignment"),
        Op::Shot("names-assignment"),
        Op::Command("modprobe capture_current"),
        Op::Command("lspci -k"),
        Op::Check(false, false),
        Op::ResultShot("names-initial-case-feedback"),
        Op::Config(
            "options lab_sensor queues=1\nalias capture_current lab_sensor\nalias audit_query audit_link\n",
        ),
        Op::Click("save-options"),
        Op::Command("modprobe capture_current"),
        Op::Check(false, false),
        Op::ResultShot("names-missing-compatibility-feedback"),
        Op::Config(
            "options lab_sensor queues=1\nalias capture_current lab_sensor\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n",
        ),
        Op::Click("save-options"),
        Op::Command("modprobe -r lab_sensor"),
        Op::Command("modprobe capture_legacy"),
        Op::Command("lspci -k"),
        Op::Output("Kernel driver in use: lab_sensor"),
        Op::Check(false, true),
        Op::Save,
        Op::Config(
            "options lab_sensor queues=1\nalias capture_current audit_link\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n",
        ),
        Op::Unchecked(false),
        Op::Click("save-options"),
        Op::Check(false, false),
        Op::ResultShot("names-current-good-one-request-failed"),
        Op::Config(
            "options lab_sensor queues=1\nalias capture_current lab_sensor\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n",
        ),
        Op::Click("save-options"),
        Op::Check(false, true),
        Op::Save,
        Op::Click("help"),
        Op::Click("worked"),
        Op::WorkedShot("names-requested-worked-editor-contents"),
        Op::Theme(true),
        Op::Command("lsmod"),
        Op::Shot("names-light150"),
        Op::ResultShot("names-light150-controls"),
        Op::Theme(false),
        Op::Command("lspci -k"),
        Op::Shot("names-dark200"),
        Op::ResultShot("names-dark200-controls"),
        Op::ThemeDefault,
        Op::Click("challenge"),
        Op::Challenge(true),
        Op::Shot("names-challenge-default-task"),
        Op::Click("learn"),
        Op::Challenge(false),
        Op::Click("reset"),
        Op::Click("confirm"),
        Op::Fresh,
    ]
}
fn current(a: &LabApp) -> (&Scenario, &work::Session) {
    let s = &a.scenarios[a.selected.unwrap()];
    let engine::State::KernelModules { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    (s, simulation)
}
fn scope(s: &work::Session, p: bool) -> &work::Scope {
    if p { &s.practice } else { &s.assignment }
}
impl LabApp {
    pub(in crate::ui) fn kernel_module_tick(&mut self, ctx: &egui::Context) {
        let root = ctx.cumulative_frame_nr();
        let gate = egui::Id::new("kernel-module-native-root");
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, root));
        let key = egui::Id::new("kernel-module-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("kernel-name-batch") {
                    names_plan()
                } else {
                    plan()
                },
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("kernel-name-batch") {
                println!(
                    "KERNEL_NAME_NATIVE_COMPLETE: real editor/fill-without-save/commands/output, independent practice, success-for-wrong-provider/missing/partial cases, saved-name invalidation, repair/reset/save-read, optional worked editor content, Challenge, light150/dark200"
                );
            } else {
                println!(
                    "KERNEL_MODULE_NATIVE_COMPLETE: actual commands/output/editor/save, independent practice, expected insertion failures, partial/incorrect/saved/effective feedback, index handover, provider preservation, invalidation/reset/save-read, Challenge, light150/dark200"
                );
            }
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 180000 + r.at;
        let op = r.ops[r.at].clone();
        let worked_shot = matches!(&op, Op::WorkedShot(_));
        let result_shot = matches!(&op, Op::ResultShot(_) | Op::WorkedShot(_));
        let config = matches!(&op, Op::Config(_));
        if r.frames == 0 {
            println!("KERNEL_MODULE_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(r.frames < 10000, "Module native step timed out: {op:?}");
        let done = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                if r.at == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
                true
            }
            Op::Click(k) => {
                let rect = match k {
                    "reset" => self.metrics.reset,
                    "confirm" => self.metrics.confirm_reset,
                    "challenge" => self.metrics.mode_challenge,
                    "learn" => self.metrics.mode_learn,
                    _ => self
                        .metrics
                        .kernel_module
                        .controls
                        .iter()
                        .find(|c| c.0 == k)
                        .map(|c| c.1),
                };
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text) | Op::Config(text) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .kernel_module
                        .fields
                        .iter()
                        .find(|f| f.0 == if config { "config" } else { "command" })
                        .map(|f| f.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let (_, s) = current(self);
                    let p = s.view == teaching::View::Example;
                    if config {
                        assert_eq!(scope(s, p).config_draft, text);
                        true
                    } else {
                        assert_eq!(scope(s, p).command, text);
                        self.pending_events = [true, false]
                            .into_iter()
                            .map(|pressed| egui::Event::Key {
                                key: egui::Key::Enter,
                                physical_key: None,
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            })
                            .collect();
                        r.phase = 3;
                        false
                    }
                } else {
                    let (source, s) = current(self);
                    let p = s.view == teaching::View::Example;
                    assert!(
                        scope(s, p)
                            .transcript
                            .last()
                            .is_some_and(|row| row.contains(text)),
                        "real terminal did not record command {text}"
                    );
                    if p {
                        let engine::State::KernelModules {
                            simulation: initial,
                        } = &source.initial
                        else {
                            panic!()
                        };
                        assert_eq!(s.assignment, initial.assignment);
                        assert!(!progress::current_solved(
                            &self.progress.attempts[&source.id],
                            source
                        ));
                    }
                    true
                }
            }
            Op::Pick(n) => {
                let key = if r.phase == 0 {
                    "picker".to_string()
                } else {
                    format!("choice-{n}")
                };
                let rect = self
                    .metrics
                    .kernel_module
                    .controls
                    .iter()
                    .find(|c| c.0 == key)
                    .map(|c| c.1);
                if self.click_visible_smoke_control(ctx, rect) {
                    if r.phase == 0 {
                        r.phase = 1;
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Op::Output(text) => {
                let (_, session) = current(self);
                let p = session.view == teaching::View::Example;
                assert!(
                    scope(session, p).transcript.last().is_some_and(|row| row
                        .split_once('\n')
                        .is_some_and(|(_, out)| out.contains(text))),
                    "Actual query output misses {text}"
                );
                true
            }
            Op::Check(p, want) => {
                if r.phase == 0 {
                    let rect = self
                        .metrics
                        .kernel_module
                        .controls
                        .iter()
                        .find(|c| c.0 == "check")
                        .map(|c| c.1);
                    if self.click_visible_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let (source, s) = current(self);
                    let engine::Goal::KernelModules { definition } = &source.goal else {
                        panic!()
                    };
                    assert!(scope(s, p).checked);
                    assert_eq!(
                        scope(s, p)
                            .checks(if p {
                                &definition.example
                            } else {
                                &definition.assignment
                            })
                            .iter()
                            .all(|r| r.0),
                        want,
                        "unexpected model check"
                    );
                    assert_eq!(
                        self.metrics.kernel_module.result.map(|r| r.1 == 0),
                        Some(want)
                    );
                    if !p {
                        assert_eq!(
                            progress::current_solved(&self.progress.attempts[&source.id], source),
                            want
                        );
                    }
                    true
                }
            }
            Op::Unsaved(p) => {
                let (_, s) = current(self);
                assert_ne!(scope(s, p).config_draft, scope(s, p).runtime.config);
                assert!(!scope(s, p).checked);
                true
            }
            Op::Unchecked(p) => {
                let (_, s) = current(self);
                assert!(!scope(s, p).checked);
                assert!(self.metrics.kernel_module.result.is_none());
                true
            }
            Op::Shot(name) | Op::ResultShot(name) | Op::WorkedShot(name) => {
                if result_shot && r.phase == 0 {
                    let target = self
                        .metrics
                        .kernel_module
                        .capture_target(worked_shot)
                        .expect(
                            "Requested worked-file or checked-feedback capture target is rendered",
                        );
                    if self.reveal_smoke_control(ctx, target) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == u8::from(result_shot) {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name.to_owned(),
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let read = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&read).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let (source, s) = current(self);
                let engine::State::KernelModules {
                    simulation: initial,
                } = &source.initial
                else {
                    panic!()
                };
                assert_eq!(s, &**initial);
                true
            }
            Op::ThemeDefault => {
                self.scale = 1.;
                ctx.set_zoom_factor(1.);
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.lesson_scroll_reset = true;
                true
            }
            Op::Theme(light) => {
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
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1100., 850.)));
                self.lesson_scroll_reset = true;
                true
            }
            Op::Challenge(want) => {
                let (source, _) = current(self);
                assert_eq!(self.progress.attempts[&source.id].challenge, want);
                assert_eq!(
                    self.metrics
                        .kernel_module
                        .controls
                        .iter()
                        .any(|c| c.0 == "introduction"),
                    !want
                );
                true
            }
        };
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            r.until = now + 0.15;
        }
        if done {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.until = now + 0.06;
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_capture_callback_does_not_reenter_context() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let key = egui::Id::new("kernel-module-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("kernel-module-test-capture")],
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| a.kernel_module_tick(ctx));
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn actual_scroll_callback_preserves_cooldown_and_visibility_retry_guard() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        a.metrics.kernel_module.controls.push((
            "below".into(),
            egui::Rect::from_min_size(egui::pos2(20., 1500.), egui::vec2(100., 30.)),
            egui::Rect::EVERYTHING,
        ));
        a.metrics.main_viewport = Some(egui::Rect::from_min_size(
            egui::pos2(0., 0.),
            egui::vec2(700., 400.),
        ));
        let key = egui::Id::new("kernel-module-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Click("below")],
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800., 600.),
                )),
                ..Default::default()
            },
            |ctx| a.kernel_module_tick(ctx),
        );
        assert!(
            a.pending_events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        );
        assert!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().until >= 0.15);
    }
}
