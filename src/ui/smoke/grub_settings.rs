//! Fixed native fixture: actual controls/keyboard, separate synthetic progress.
use super::*;
const IDS: [&str; 2] = ["lpic1-grub-persistent-default", "lpic1-grub-argument-scope"];
#[derive(Clone, Copy)]
enum Input {
    Reference,
    WrongTimeout,
    PartialMode,
}
#[derive(Clone, Copy)]
enum Op {
    Style(f32, bool, bool),
    Open(usize),
    View(&'static str),
    Worked,
    Fill,
    Generate,
    Inspect,
    Edit(Input),
    Check(bool),
    Shot(&'static str, &'static str),
    Save,
    Reset,
    Back,
}
fn route() -> Vec<Op> {
    use Op::*;
    vec![
        Style(1., false, false),
        Open(0),
        Shot("grub-settings-introduction", "top"),
        Shot("grub-settings-worked", "try"),
        View("example"),
        Check(false),
        Worked,
        Fill,
        Check(false),
        Shot("grub-settings-before-generation", "result"),
        Generate,
        Inspect,
        Shot("grub-settings-generated-menu", "generated-output"),
        Inspect,
        Check(true),
        Shot("grub-settings-practice-correct", "result"),
        View("assignment"),
        Shot("grub-settings-assignment", "generate"),
        Check(false),
        Edit(Input::Reference),
        Check(false),
        Generate,
        Check(true),
        Save,
        Edit(Input::WrongTimeout),
        Check(false),
        Shot("grub-settings-outdated-menu", "result"),
        Generate,
        Check(false),
        Shot("grub-settings-wrong-timer", "result"),
        Edit(Input::Reference),
        Generate,
        Check(true),
        Open(1),
        Shot("grub-arguments-introduction", "top"),
        Shot("grub-arguments-worked", "try"),
        View("example"),
        Check(false),
        Worked,
        Fill,
        Generate,
        Check(true),
        Shot("grub-arguments-practice-correct", "result"),
        View("assignment"),
        Shot("grub-arguments-assignment", "generate"),
        Check(false),
        Edit(Input::PartialMode),
        Generate,
        Check(false),
        Shot("grub-arguments-partial-feedback", "result"),
        Edit(Input::Reference),
        Generate,
        Check(true),
        Style(1.5, false, false),
        Shot("grub-arguments-dark150-editor", "editor"),
        Style(2., true, true),
        Shot("grub-arguments-light200-editor", "editor"),
        Inspect,
        Shot("grub-arguments-light200-generated-menu", "generated-start"),
        Shot("grub-arguments-light200-result", "result"),
        Save,
        Reset,
        Back,
    ]
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    next: f64,
    polls: usize,
    started: Option<Instant>,
    rect: Option<egui::Rect>,
    stable: usize,
    retained: Option<serde_json::Value>,
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "back" => a.metrics.back,
        "generated-start" => a.metrics.boot_entry.generated_start,
        "reset" => a.metrics.reset,
        "confirm" => a.metrics.confirm_reset,
        _ => a
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
fn click(a: &mut LabApp, ctx: &egui::Context, key: &str) -> bool {
    a.click_visible_smoke_control(ctx, target(a, key))
}
fn capture(a: &mut LabApp, ctx: &egui::Context, name: &str) {
    a.capture_pending = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
        name.to_owned(),
    )));
}
impl LabApp {
    pub(in crate::ui) fn grub_settings_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("grub-settings-fixed-native-flow");
        let mut f = ctx
            .data_mut(|d| d.get_temp::<Flow>(key))
            .unwrap_or_default();
        let ops = route();
        if f.at == ops.len() {
            println!(
                "GRUB_SETTINGS_NATIVE_COMPLETE: {} actions; editor/actual worked fill, explicit generation, separate practice, wrong/alternate/partial outcomes, invalidation, save-read, real selected Reset/Back and100/150/200light/dark captures; no actual GRUB/boot/host execution or novice claim",
                ops.len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 125000 + f.at;
        let now = ctx.input(|i| i.time);
        let started = f.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut f.polls, now, f.next, started.elapsed());
        assert!(
            due.is_ok(),
            "GRUB action{} phase{}: {due:?}; elapsed{:?}, capture_pending{}, viewport{:?}",
            f.at,
            f.phase,
            started.elapsed(),
            self.capture_pending,
            self.metrics.main_viewport
        );
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(key, f));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match ops[f.at] {
            Op::Style(scale, light, narrow) => {
                if f.phase == 0 {
                    self.scale = scale;
                    ctx.set_zoom_factor(scale);
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1268., 1308.,
                        )));
                    }
                    self.lesson_scroll_reset = true;
                    f.phase = 1;
                    false
                } else {
                    let rect = ctx.content_rect();
                    let ppp = ctx.pixels_per_point();
                    let size = if narrow {
                        (rect.width() * ppp - 1268.).abs() < 4.
                            && (rect.height() * ppp - 1308.).abs() < 4.
                    } else {
                        ctx.input(|i| i.viewport().fullscreen) != Some(false)
                    };
                    if size && (ctx.zoom_factor() - scale).abs() < 0.01 && f.rect == Some(rect) {
                        f.stable += 1;
                    } else {
                        f.stable = 0;
                    }
                    f.rect = Some(rect);
                    f.stable >= 2
                }
            }
            Op::Open(n) => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                let i = self.scenarios.iter().position(|s| s.id == IDS[n]).unwrap();
                self.launch(i);
                true
            }
            Op::View(v) => {
                if f.phase == 0 {
                    if click(self, ctx, v) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    let engine::State::BootEntry { simulation } =
                        &self.progress.attempts[&self.scenarios[i].id].state
                    else {
                        panic!()
                    };
                    assert_eq!(
                        simulation.view,
                        if v == "example" {
                            teaching::View::Example
                        } else {
                            teaching::View::Assignment
                        }
                    );
                    true
                }
            }
            Op::Worked => click(self, ctx, "worked"),
            Op::Generate => click(self, ctx, "generate"),
            Op::Inspect => click(self, ctx, "generated-output"),
            Op::Fill => {
                if f.phase == 0 {
                    if click(self, ctx, "fill") {
                        f.phase = 1;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    assert!(self.metrics.boot_entry.result.is_none());
                    assert!(!progress::current_solved(
                        &self.progress.attempts[&self.scenarios[i].id],
                        &self.scenarios[i]
                    ));
                    true
                }
            }
            Op::Edit(kind) => {
                if f.phase == 0 {
                    if click(self, ctx, "editor") {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    let i = self.selected.unwrap();
                    let engine::Goal::BootEntry { definition } = &self.scenarios[i].goal else {
                        panic!()
                    };
                    let mut text = definition.initial.reference.clone();
                    match kind {
                        Input::Reference => {
                            if self.scenarios[i].id == IDS[0] {
                                text = text.replacen("GRUB_DEFAULT=rollback", "GRUB_DEFAULT=0", 1);
                            }
                        }
                        Input::WrongTimeout => {
                            let timeout = definition.initial.generator.as_ref().unwrap().timeout;
                            text =
                                text.replace(&format!("GRUB_TIMEOUT={timeout}"), "GRUB_TIMEOUT=0");
                        }
                        Input::PartialMode => {
                            text = text
                                .replace("GRUB_CMDLINE_LINUX=\"\"", "GRUB_CMDLINE_LINUX=\"rw\"")
                                .replace(
                                    "GRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"",
                                    "GRUB_CMDLINE_LINUX_DEFAULT=\"ro quiet\"",
                                );
                        }
                    }
                    self.pending_events.extend([
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers {
                                command: true,
                                ..egui::Modifiers::CTRL
                            },
                        },
                        egui::Event::Paste(text),
                    ]);
                    f.phase = 2;
                    false
                } else {
                    let i = self.selected.unwrap();
                    assert!(self.metrics.boot_entry.result.is_none());
                    assert!(!progress::current_solved(
                        &self.progress.attempts[&self.scenarios[i].id],
                        &self.scenarios[i]
                    ));
                    true
                }
            }
            Op::Check(pass) => {
                if f.phase == 0 {
                    if click(self, ctx, "test") {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert_eq!(self.metrics.boot_entry.result, Some(pass));
                    let i = self.selected.unwrap();
                    let engine::State::BootEntry { simulation } =
                        &self.progress.attempts[&self.scenarios[i].id].state
                    else {
                        panic!()
                    };
                    assert_eq!(
                        progress::current_solved(
                            &self.progress.attempts[&self.scenarios[i].id],
                            &self.scenarios[i]
                        ),
                        pass && simulation.view == teaching::View::Assignment
                    );
                    true
                }
            }
            Op::Shot(name, control) => {
                if f.phase == 0 {
                    if control == "top"
                        || target(self, control).is_some_and(|r| self.reveal_smoke_control(ctx, r))
                    {
                        f.phase = 1;
                        f.next = now + 0.15;
                    }
                    false
                } else if f.phase == 1 {
                    capture(self, ctx, name);
                    f.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Reset => {
                if f.phase == 0 {
                    f.retained =
                        Some(serde_json::to_value(&self.progress.attempts[IDS[0]]).unwrap());
                    if click(self, ctx, "reset") {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    if click(self, ctx, "confirm") {
                        f.phase = 2;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    assert!(!self.reset_pending);
                    assert_eq!(
                        self.progress.attempts[IDS[1]].state,
                        self.scenarios[i].initial
                    );
                    assert_eq!(
                        serde_json::to_value(&self.progress.attempts[IDS[0]]).unwrap(),
                        f.retained.clone().unwrap()
                    );
                    true
                }
            }
            Op::Back => {
                if f.phase == 0 {
                    if click(self, ctx, "back") {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert!(self.selected.is_none());
                    true
                }
            }
        };
        if done {
            println!("GRUB_SETTINGS_NATIVE_STEP {}", f.at);
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.started = None;
            f.rect = None;
            f.stable = 0;
            f.next = now + 0.025;
        } else {
            f.next = f.next.max(now + 0.025);
        }
        ctx.data_mut(|d| d.insert_temp(key, f));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grub_settings_capture_uses_native_viewport_outside_context_lock() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let out = ctx.run(egui::RawInput::default(), |ctx| {
            capture(&mut a, ctx, "grub-settings-capture-regression")
        });
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
        assert_eq!(
            route()
                .iter()
                .filter(|op| matches!(op, Op::Shot(..)))
                .count(),
            17
        );
        assert_eq!(route().len(), 63);
    }
}
