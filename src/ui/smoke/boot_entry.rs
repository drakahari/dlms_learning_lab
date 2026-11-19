//! Fixed private native boot-entry fixture. All edits/checks use actual widgets.
use super::*;
use dlms_learning_lab::engine::State;
const IDS: [&str; 3] = [
    "lpic1-boot-root-identity",
    "lpic1-boot-initramfs-match",
    "lpic1-boot-recovery-handoff",
];
#[derive(Clone)]
enum Op {
    Style(f32, bool, bool),
    Open(usize),
    View(&'static str),
    Edit(bool),
    Check(bool),
    Shot(&'static str, bool),
    Save,
    Reset,
    Mode(bool),
    Back,
}
fn route() -> Vec<Op> {
    use Op::*;
    vec![
        Style(1., false, false),
        Open(0),
        Mode(true),
        Mode(false),
        Shot("boot-root-introduction", false),
        Shot("boot-root-introduction-worked", true),
        View("example"),
        Shot("boot-root-practice", true),
        Check(false),
        Shot("boot-root-practice-feedback", true),
        Edit(true),
        Check(true),
        View("assignment"),
        Shot("boot-root-assignment", true),
        Check(false),
        Edit(true),
        Check(true),
        Shot("boot-root-assignment-result", true),
        Save,
        Open(1),
        Shot("boot-initramfs-introduction", false),
        View("example"),
        Check(false),
        Shot("boot-initramfs-practice-feedback", true),
        Edit(true),
        Check(true),
        View("assignment"),
        Shot("boot-initramfs-assignment", true),
        Edit(true),
        Check(true),
        Save,
        Open(2),
        Shot("boot-recovery-introduction", false),
        View("example"),
        Edit(true),
        Check(true),
        View("assignment"),
        Check(false),
        Edit(true),
        Check(true),
        Shot("boot-recovery-assignment-result", true),
        Edit(false),
        Check(false),
        Shot("boot-recovery-invalidated", true),
        Edit(true),
        Check(true),
        Style(1.5, false, false),
        Shot("boot-recovery-dark150", true),
        Style(2., true, true),
        Shot("boot-recovery-light200-editor", true),
        Shot("boot-recovery-light200-result", true),
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
    started: Option<Instant>,
    polls: usize,
    rect: Option<egui::Rect>,
    stable: usize,
    retained: Option<Vec<serde_json::Value>>,
}
fn capture(a: &mut LabApp, ctx: &egui::Context, name: &str) {
    a.capture_pending = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
        name.to_owned(),
    )));
}
fn target(a: &LabApp, key: &str) -> Option<egui::Rect> {
    if key == "editor-text" {
        return a.metrics.boot_entry.editor_text;
    }
    a.metrics
        .boot_entry
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
}
fn click(a: &mut LabApp, ctx: &egui::Context, key: &str) -> bool {
    target(a, key)
        .is_some_and(|r| a.reveal_smoke_control(ctx, r) && a.click_smoke_control(ctx, Some(r)))
}
impl LabApp {
    pub(in crate::ui) fn boot_entry_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("boot-entry-fixed-native-flow");
        let mut f = ctx
            .data_mut(|d| d.get_temp::<Flow>(key))
            .unwrap_or_default();
        let ops = route();
        if f.at == ops.len() {
            println!(
                "BOOT_ENTRY_NATIVE_COMPLETE: {} actions; real editor, practice isolation, partial/error outcomes, invalidation, private save/read/reset and light/dark scaling; no actual boot or novice claim",
                ops.len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 110000 + f.at;
        let start = f.started.get_or_insert_with(Instant::now);
        let now = ctx.input(|i| i.time);
        let due = native_action_due(&mut f.polls, now, f.next, start.elapsed());
        assert!(due.is_ok(),
            "boot_entry native action {} did not settle: {due:?}; phase={} polls={} elapsed={:?} capture_pending={} inputs={:?} viewport={:?} scroll={:?}",
            f.at, f.phase, f.polls, start.elapsed(), self.capture_pending,
            self.metrics.inputs, self.metrics.main_viewport, self.metrics.main_scroll_offset);
        if !due.unwrap() {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            ctx.data_mut(|d| d.insert_temp(key, f));
            return;
        }
        let complete = match ops[f.at].clone() {
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
                    let r = ctx.content_rect();
                    let p = ctx.pixels_per_point();
                    let size = if narrow {
                        (r.width() * p - 1268.).abs() < 4. && (r.height() * p - 1308.).abs() < 4.
                    } else {
                        ctx.input(|i| i.viewport().fullscreen) != Some(false)
                    };
                    if size && (ctx.zoom_factor() - scale).abs() < 0.01 && f.rect == Some(r) {
                        f.stable += 1;
                    } else {
                        f.stable = 0;
                    }
                    f.rect = Some(r);
                    f.stable >= 2
                }
            }
            Op::Open(n) => {
                assert_eq!(self.scenarios.len(), 392);
                assert_eq!(self.units.len(), 154);
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
                    let State::BootEntry { simulation } =
                        &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id].state
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
            Op::Edit(reference) => {
                let i = self.selected.unwrap();
                let id = &self.scenarios[i].id;
                let engine::Goal::BootEntry { definition } = &self.scenarios[i].goal else {
                    panic!()
                };
                let State::BootEntry { simulation } = &self.progress.attempts[id].state else {
                    panic!()
                };
                let practice = simulation.view == teaching::View::Example;
                let t = if practice {
                    &definition.example
                } else {
                    &definition.initial
                };
                let value = if reference {
                    t.reference.clone()
                } else {
                    t.starter.clone()
                };
                if f.phase == 0 {
                    if click(self, ctx, "editor") {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
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
                        egui::Event::Paste(value),
                    ]);
                    f.phase = 2;
                    false
                } else {
                    let State::BootEntry { simulation } = &self.progress.attempts[id].state else {
                        panic!()
                    };
                    let d = if practice {
                        &simulation.practice
                    } else {
                        &simulation.assignment
                    };
                    assert_eq!(d.source, value, "Real editor must receive the actual text");
                    assert!(d.checked.is_none());
                    assert!(!progress::current_solved(
                        &self.progress.attempts[id],
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
                    let a = &self.progress.attempts[&self.scenarios[i].id];
                    let State::BootEntry { simulation } = &a.state else {
                        panic!()
                    };
                    assert_eq!(
                        progress::current_solved(a, &self.scenarios[i]),
                        pass && simulation.view == teaching::View::Assignment
                    );
                    true
                }
            }
            Op::Shot(name, control) => {
                if f.phase == 0 {
                    if !control
                        || target(
                            self,
                            if name.ends_with("worked") {
                                "start"
                            } else if name.contains("result")
                                || name.contains("feedback")
                                || name.contains("invalidated")
                            {
                                "result"
                            } else if name.ends_with("editor") {
                                // A screenshot must reveal every authored line;
                                // empty rows are not a control or a text target.
                                // Real edit clicks still require the full viewport.
                                "editor-text"
                            } else {
                                "test"
                            },
                        )
                        .is_some_and(|r| self.reveal_smoke_control(ctx, r))
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
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Mode(challenge) => {
                if f.phase == 0 {
                    let r = if challenge {
                        self.metrics.mode_challenge
                    } else {
                        self.metrics.mode_learn
                    };
                    if r.is_some_and(|r| {
                        self.reveal_smoke_control(ctx, r) && self.click_smoke_control(ctx, Some(r))
                    }) {
                        f.phase = 1;
                    }
                    false
                } else {
                    let id = &self.scenarios[self.selected.unwrap()].id;
                    assert_eq!(self.progress.attempts[id].challenge, challenge);
                    assert_eq!(self.metrics.lesson_tabs.is_empty(), challenge);
                    true
                }
            }
            Op::Back => {
                if f.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.back) {
                        f.phase = 1;
                    }
                    false
                } else {
                    assert!(self.selected.is_none());
                    true
                }
            }
            Op::Reset => {
                if f.phase == 0 {
                    f.retained = Some(
                        IDS[..2]
                            .iter()
                            .map(|id| serde_json::to_value(&self.progress.attempts[*id]).unwrap())
                            .collect(),
                    );
                    if self.metrics.reset.is_some_and(|r| {
                        self.reveal_smoke_control(ctx, r) && self.click_smoke_control(ctx, Some(r))
                    }) {
                        f.phase = 1;
                    }
                    false
                } else if f.phase == 1 {
                    // A successful click removes this confirmation. Drain the
                    // delivered click before requiring geometry on a new pass.
                    if self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                        f.phase = 2;
                    }
                    false
                } else {
                    let i = self.selected.unwrap();
                    assert!(!self.reset_pending);
                    assert_eq!(
                        self.progress.attempts[IDS[2]].state,
                        self.scenarios[i].initial
                    );
                    for (n, id) in IDS[..2].iter().enumerate() {
                        assert_eq!(
                            serde_json::to_value(&self.progress.attempts[*id]).unwrap(),
                            f.retained.as_ref().unwrap()[n]
                        );
                    }
                    true
                }
            }
        };
        if complete {
            println!("BOOT_ENTRY_NATIVE_STEP {}", f.at);
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
    fn boot_native_confirmation_disappearance_finishes_the_delivered_click() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        for id in IDS {
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            a.launch(i);
        }
        let i = a.selected.unwrap();
        let engine::Goal::BootEntry { definition } = a.scenarios[i].goal.clone() else { panic!() };
        let State::BootEntry { simulation } = &mut a.progress.attempts.get_mut(IDS[2]).unwrap().state else { panic!() };
        simulation.assignment.edit(definition.initial.reference.clone()).unwrap();
        simulation.assignment.check(&definition.initial).unwrap();
        simulation.view = teaching::View::Assignment;
        assert!(progress::current_solved(&a.progress.attempts[IDS[2]], &a.scenarios[i]));
        let retained = IDS[..2].iter().map(|id| serde_json::to_value(&a.progress.attempts[*id]).unwrap()).collect::<Vec<_>>();
        let reset = route().iter().position(|op| matches!(op, Op::Reset)).unwrap();
        let flow_key = egui::Id::new("boot-entry-fixed-native-flow");
        ctx.data_mut(|d| d.insert_temp(flow_key, Flow { at: reset, phase: 1, retained: Some(retained.clone()), ..Default::default() }));
        a.reset_pending = true;
        a.lesson_scroll_reset = true;
        let mut visible = false;
        let mut disappeared = false;
        for n in 0..96 {
            let events = std::mem::take(&mut a.pending_events);
            let _ = ctx.run(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1500.,1200.))),
                time: Some(100. + n as f64 * 0.1), events, ..Default::default()
            }, |ctx| a.render(ctx));
            visible |= a.metrics.confirm_reset.is_some();
            disappeared |= visible && !a.reset_pending;
            if disappeared {
                // The optimized native frame can finish another layout pass
                // after the click removes the confirmation. Observe that
                // settled geometry before asking the driver to acknowledge it.
                let _ = ctx.run(egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1500.,1200.))),
                    time: Some(100.05 + n as f64 * 0.1), ..Default::default()
                }, |ctx| a.render(ctx));
                assert!(a.metrics.confirm_reset.is_none());
            }
            a.boot_entry_tick(&ctx);
            if ctx.data(|d|d.get_temp::<Flow>(flow_key)).unwrap().at == route().len() { break; }
        }
        assert!(visible && disappeared, "The actual confirmation must be clicked and removed");
        assert!(!a.reset_pending);
        assert_eq!(a.progress.attempts[IDS[2]].state, a.scenarios[i].initial);
        assert_eq!(ctx.data(|d|d.get_temp::<Flow>(flow_key)).unwrap().at, route().len(),
            "The driver must finish both Reset and Back after their controls disappear");
        assert!(a.selected.is_none());
        assert!(ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", 110000 + reset)))).is_none());
        for (n,id) in IDS[..2].iter().enumerate() {
            assert_eq!(serde_json::to_value(&a.progress.attempts[*id]).unwrap(), retained[n]);
        }
    }
    #[test]
    fn boot_entry_capture_uses_actual_viewport_api_outside_context_data_lock() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            capture(&mut a, ctx, "boot-entry-capture-regression")
        });
        assert!(a.capture_pending);
        assert_eq!(
            route().iter().filter(|o| matches!(o, Op::Shot(..))).count(),
            15
        );
    }
}
