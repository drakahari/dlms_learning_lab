//! Fixed developer flow: native keyboard edits and real IPv6 checks, private profile only.
use super::*;
const ID: &str = "ccna-ipv6-modified-eui64";
#[derive(Clone)]
enum Op {
    Style(f32, bool, bool),
    Open,
    View(teaching::View),
    Routes(bool),
    Field(usize, &'static str),
    Check(bool),
    Shot(&'static str, Target),
    Save,
    Reset,
}
#[derive(Clone, Copy)]
enum Target {
    Top,
    Input,
    Check,
    Worked,
}
fn route() -> Vec<Op> {
    use teaching::View::*;
    vec![
        Op::Style(1., false, false),
        Op::Shot("current-normal-library", Target::Top),
        Op::Open,
        Op::Shot("eui64-introduction", Target::Top),
        Op::Shot("eui64-introduction-worked", Target::Worked),
        Op::View(Example),
        Op::Shot("eui64-practice", Target::Input),
        Op::Field(0, "fd64:7::216:3eff:fe12:3456/64"),
        Op::Routes(true),
        Op::Field(1, "fe80::1"),
        Op::Check(true),
        Op::Routes(false),
        Op::Field(0, "fd64:7::16:3eff:fe12:3456/64"),
        Op::Check(false),
        Op::Shot("eui64-practice-bit-feedback", Target::Check),
        Op::View(Assignment),
        Op::Shot("eui64-assignment", Target::Input),
        Op::Field(0, "2001:db8:74:0:425:96ff:feab:cdef/64"),
        Op::Check(false),
        Op::Shot("eui64-partial-address-only", Target::Check),
        Op::Routes(true),
        Op::Field(1, "2001:db8:74::1"),
        Op::Check(true),
        Op::Routes(false),
        Op::Field(0, "2001:db8:74:0:625:96ff:feab:cdef/64"),
        Op::Check(false),
        Op::Shot("eui64-assignment-bit-feedback", Target::Check),
        Op::Field(0, "2001:db8:74:0:425:96ff:feab:cdef/64"),
        Op::Check(true),
        Op::Save,
        Op::Style(1.5, false, false),
        Op::Shot("eui64-dark150-address", Target::Input),
        Op::Style(2., true, true),
        Op::Shot("eui64-light200-address", Target::Input),
        Op::Shot("eui64-light200-result", Target::Check),
        Op::Save,
        Op::Reset,
    ]
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    polls: usize,
    next: f64,
    started: Option<Instant>,
    rect: Option<egui::Rect>,
    stable: usize,
}
fn visible_click(app: &mut LabApp, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
    rect.is_some_and(|r| app.reveal_smoke_control(ctx, r) && app.click_smoke_control(ctx, Some(r)))
}
fn capture(app: &mut LabApp, ctx: &egui::Context, name: &str) {
    app.capture_pending = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
        name.to_owned(),
    )));
}
impl LabApp {
    pub(in crate::ui) fn eui64_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("eui64-fixed-native-flow");
        let mut f = ctx
            .data_mut(|d| d.get_temp::<Flow>(key))
            .unwrap_or_default();
        let ops = route();
        if f.at == ops.len() {
            println!(
                "EUI64_NATIVE_COMPLETE: {} actions; native practice/assignment keyboard edits, alternate next hop, real partial/bit feedback, invalidation, private save-read/reset and themed captures; no host execution; no cold-reopen or novice claim",
                ops.len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 100000 + f.at;
        let started = f.started.get_or_insert_with(Instant::now);
        let now = ctx.input(|i| i.time);
        let due = native_action_due(&mut f.polls, now, f.next, started.elapsed());
        assert!(due.is_ok(),
            "eui64 native action {} did not settle: {due:?}; phase={} polls={} elapsed={:?} capture_pending={} inputs={:?} viewport={:?} scroll={:?}",
            f.at, f.phase, f.polls, started.elapsed(), self.capture_pending,
            self.metrics.inputs, self.metrics.main_viewport, self.metrics.main_scroll_offset);
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(key, f));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let op = ops[f.at].clone();
        let complete = match op {
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
            Op::Open => {
                assert_eq!(self.scenarios.len(), 380);
                assert_eq!(self.units.len(), 141);
                let i = self.scenarios.iter().position(|s| s.id == ID).unwrap();
                self.launch(i);
                assert_eq!(self.ipv6_view(i), teaching::View::Introduction);
                true
            }
            Op::View(view) => {
                self.set_ipv6_view(self.selected.unwrap(), view);
                true
            }
            Op::Routes(value) => {
                let i = self.selected.unwrap();
                let practice = self.ipv6_view(i) == teaching::View::Example;
                let ws = self
                    .progress
                    .attempts
                    .get_mut(ID)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap();
                if practice {
                    ws.ipv6_learning.as_mut().unwrap().draft.show_routes = value;
                } else {
                    ws.ipv6_draft.as_mut().unwrap().show_routes = value;
                }
                self.lesson_scroll_reset = true;
                true
            }
            Op::Field(n, value) => {
                if f.phase == 0 {
                    if let Some(rect) = self.metrics.inputs.get(n).copied()
                        && visible_click(self, ctx, Some(rect))
                    {
                        f.phase = 1;
                    }
                    f.next = now + 0.025;
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
                        egui::Event::Paste(value.into()),
                    ]);
                    f.phase = 2;
                    false
                } else {
                    let i = self.selected.unwrap();
                    let ws = self.progress.attempts[ID].workspace.as_ref().unwrap();
                    let draft = if self.ipv6_view(i) == teaching::View::Example {
                        &ws.ipv6_learning.as_ref().unwrap().draft
                    } else {
                        ws.ipv6_draft.as_ref().unwrap()
                    };
                    let actual = if draft.show_routes {
                        match n {
                            0 => &draft.devices["Client"].routes[0].destination,
                            1 => &draft.devices["Client"].routes[0].next_hop,
                            _ => unreachable!(),
                        }
                    } else {
                        &draft.devices["Client"].interfaces["lan"].address
                    };
                    assert_eq!(actual, value, "Native keyboard must update the real draft");
                    assert!(
                        !progress::current_solved(&self.progress.attempts[ID], &self.scenarios[i]),
                        "An edited draft cannot retain success"
                    );
                    true
                }
            }
            Op::Check(pass) => {
                if f.phase == 0 {
                    let rect = self.metrics.check;
                    if visible_click(self, ctx, rect) {
                        f.phase = 1;
                    }
                    f.next = now + 0.025;
                    false
                } else {
                    let i = self.selected.unwrap();
                    if self.ipv6_view(i) == teaching::View::Example {
                        let l = self.progress.attempts[ID]
                            .workspace
                            .as_ref()
                            .unwrap()
                            .ipv6_learning
                            .as_ref()
                            .unwrap();
                        let checks = l.checks(self.scenarios[i].ipv6_teaching.as_ref().unwrap());
                        assert_eq!(checks.len(), 6);
                        assert_eq!(checks.iter().all(|c| c.0), pass);
                        assert!(!progress::current_solved(
                            &self.progress.attempts[ID],
                            &self.scenarios[i]
                        ));
                    } else {
                        assert_eq!(
                            progress::current_solved(
                                &self.progress.attempts[ID],
                                &self.scenarios[i]
                            ),
                            pass
                        );
                    }
                    true
                }
            }
            Op::Shot(name, target) => {
                if f.phase == 0 {
                    let rect = match target {
                        Target::Top => None,
                        Target::Input => self.metrics.inputs.first().copied(),
                        Target::Check => self.metrics.check,
                        Target::Worked => self.metrics.example_action,
                    };
                    if matches!(target, Target::Top)
                        || rect.is_some_and(|r| self.reveal_smoke_control(ctx, r))
                    {
                        f.phase = 1;
                        f.next = now + 0.15;
                    } else {
                        f.next = now + 0.025;
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
            Op::Reset => {
                let i = self.selected.unwrap();
                self.reset_attempt(i);
                assert_eq!(self.progress.attempts[ID].state, self.scenarios[i].initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &self.scenarios[i]
                ));
                true
            }
        };
        if complete {
            println!("EUI64_NATIVE_STEP {}", f.at);
            f.at += 1;
            f.phase = 0;
            f.polls = 0;
            f.next = now + 0.025;
            f.started = None;
            f.rect = None;
            f.stable = 0;
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
    fn eui64_capture_uses_the_actual_viewport_api_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            capture(&mut a, ctx, "eui64-regression-capture");
        });
        assert!(a.capture_pending);
        assert_eq!(
            route()
                .iter()
                .filter(|op| matches!(op, Op::Shot(..)))
                .count(),
            11
        );
    }
}
