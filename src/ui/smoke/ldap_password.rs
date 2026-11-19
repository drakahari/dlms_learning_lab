//! Fixed private native password-workspace checks; no host/network execution.
use super::*;
use dlms_learning_lab::ldap_password_workspace as work;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Style(f32, bool, bool),
    Click(String),
    Field(String, String),
    Grade {
        practice: bool,
        passed: Option<bool>,
    },
    Trial(String),
    Cases(usize, usize),
    Fresh,
    ProbeMismatch,
    Save,
    Shot(String),
    ShotAt(String, String),
}

#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
    frames: usize,
    style_rect: Option<egui::Rect>,
    stable_frames: usize,
    reveal_after: f64,
    action_started: Option<Instant>,
}

const ID: &str = "lpic2-ldap-protected-password";
fn route(labs: &[Scenario]) -> Vec<Op> {
    let d = work::source(labs.iter().find(|s| s.id == ID).unwrap()).unwrap();
    let correct = &d.initial.reference;
    let without_old = correct.command.replace(" -A", "");
    vec![
        Op::Style(1., false, false),
        Op::Open(ID.into()),
        Op::Shot("password-introduction".into()),
        Op::ShotAt("password-introduction-worked".into(), "start".into()),
        Op::Click("example".into()),
        Op::ShotAt("password-practice-workspace".into(), "config".into()),
        Op::Click("run".into()),
        Op::Trial("does not request TLS".into()),
        Op::Click("check".into()),
        Op::Grade {
            practice: true,
            passed: Some(false),
        },
        Op::Cases(0, 3),
        Op::Click("worked".into()),
        Op::Click("fill-example".into()),
        Op::Grade {
            practice: true,
            passed: None,
        },
        Op::Click("run".into()),
        Op::Trial("Exit status: 0".into()),
        Op::ShotAt("password-practice-actual".into(), "output".into()),
        Op::Click("check".into()),
        Op::Grade {
            practice: true,
            passed: Some(true),
        },
        Op::Click("assignment".into()),
        Op::ShotAt("password-assignment-workspace".into(), "config".into()),
        Op::Click("check".into()),
        Op::Grade {
            practice: false,
            passed: Some(false),
        },
        Op::Cases(0, 3),
        Op::Field("config".into(), correct.config.clone()),
        Op::Field("command".into(), correct.command.clone()),
        Op::Click("bind".into()),
        Op::Click("bind-current".into()),
        Op::Click("run".into()),
        Op::Trial("Exit status: 0".into()),
        Op::ShotAt("password-assignment-actual".into(), "output".into()),
        Op::Click("check".into()),
        Op::Grade {
            practice: false,
            passed: Some(true),
        },
        Op::Cases(3, 3),
        Op::Click("replacement".into()),
        Op::Click("replacement-current".into()),
        Op::Grade {
            practice: false,
            passed: None,
        },
        Op::Click("run".into()),
        Op::Trial("Exit status: 0".into()),
        Op::ProbeMismatch,
        Op::Click("check".into()),
        Op::Grade {
            practice: false,
            passed: Some(false),
        },
        Op::Cases(0, 3),
        Op::ShotAt("password-nonrotation-feedback".into(), "check".into()),
        Op::Click("replacement".into()),
        Op::Click("replacement-replacement".into()),
        Op::Field("command".into(), without_old),
        Op::Click("check".into()),
        Op::Grade {
            practice: false,
            passed: Some(false),
        },
        Op::Cases(2, 3),
        Op::ShotAt("password-partial-feedback".into(), "check".into()),
        Op::Field("command".into(), correct.command.clone()),
        Op::Click("bind".into()),
        Op::Click("bind-other".into()),
        Op::Click("check".into()),
        Op::Grade {
            practice: false,
            passed: Some(false),
        },
        Op::Cases(1, 3),
        Op::Click("bind".into()),
        Op::Click("bind-current".into()),
        Op::Click("run".into()),
        Op::Click("check".into()),
        Op::Grade {
            practice: false,
            passed: Some(true),
        },
        Op::Save,
        Op::Style(1.5, false, false),
        Op::ShotAt("password-dark150-wide".into(), "config".into()),
        Op::Style(2., true, true),
        Op::Click("run".into()),
        Op::Trial("Exit status: 0".into()),
        Op::ShotAt("password-light200-command".into(), "command".into()),
        Op::ShotAt("password-light200-config".into(), "config".into()),
        Op::ShotAt("password-light200-actual".into(), "output".into()),
        Op::ShotAt("password-light200-result".into(), "help".into()),
        Op::Click("reset-header".into()),
        Op::Click("confirm-reset".into()),
        Op::Fresh,
    ]
}
impl LabApp {
    /// A collapsing body's widget can exist before its hit area is revealed.
    /// Wait for the actual child clip, rather than clicking through its animation.
    fn ldap_password_click(
        &mut self,
        ctx: &egui::Context,
        target: Option<(egui::Rect, egui::Rect)>,
        popup: bool,
    ) -> Option<bool> {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            // A popup or disclosure target can disappear after a delivered click.
            return Some(self.click_smoke_control(ctx, None));
        }
        let (rect, clip) = target?;
        if popup {
            let clip = clip.intersect(ctx.content_rect());
            if !smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
                let point = egui::pos2(
                    rect.center().x.clamp(clip.left(), clip.right()),
                    clip.center().y,
                );
                assert_eq!(
                    ctx.layer_id_at(point).map(|l| l.order),
                    Some(egui::Order::Foreground),
                    "Popup wheel must reach the actual menu layer"
                );
                self.pending_events.extend([
                    egui::Event::PointerMoved(point),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(
                            0.,
                            (clip.center().y - rect.center().y).clamp(-500., 500.),
                        ),
                        modifiers: Default::default(),
                    },
                ]);
                self.started = Instant::now();
                ctx.request_repaint();
                return Some(false);
            }
            return Some(self.click_smoke_control(ctx, Some(rect)));
        }
        if !self.reveal_smoke_control(ctx, rect) {
            return Some(false);
        }
        if !clip.expand(1. / ctx.pixels_per_point()).contains_rect(rect) {
            return None;
        }
        Some(self.click_smoke_control(ctx, Some(rect)))
    }

    pub(in crate::ui) fn ldap_password_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("ldap_password-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "LDAP_PASSWORD_NATIVE_FLOW_COMPLETE: protected virtual settings and separate prompts; actual fresh authentication; failed successful-nonrotation and partial policy cases; practice isolation, editing, retry, save-read and reset; no host or network execution"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 100000 + r.at;
        let op = r.ops[r.at].clone();
        let began = *r.action_started.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed().as_secs() < 15,
            "LdapPassword native action exceeded15seconds: {} {:?}",
            r.at,
            op
        );
        // Optimized rendering can yield thousands of callbacks during a short
        // asynchronous settle delay. Waiting consumes time, not a work attempt.
        // The independent wall deadline still bounds even a stalled egui clock.
        if matches!(&op, Op::Shot(_) | Op::ShotAt(_, _) if (r.phase==0 && ctx.input(|i|i.time)<r.reveal_after)
            || (r.phase==2 && self.capture_pending))
            || matches!(&op, Op::Click(_) if ctx.input(|i| i.time) < r.reveal_after)
        {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        if r.frames == 0 {
            println!("LDAP_PASSWORD_NATIVE_STEP {} {:?}", r.at, op);
        }
        // A resize acknowledgement still needs geometry observations, but
        // those observations are waiting rather than new work attempts.
        if !matches!(&op, Op::Style(..) if r.phase==1) {
            r.frames += 1;
            assert!(
                r.frames < 1200,
                "LdapPassword native action did not complete: {} {:?}",
                r.at,
                op
            );
        }
        let complete = match op {
            Op::Style(scale, light, narrow) => {
                if r.phase == 0 {
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
                    r.phase = 1;
                } else {
                    let rect = ctx.content_rect();
                    let ppp = ctx.pixels_per_point();
                    let dimensions = if narrow {
                        (rect.width() * ppp - 1268.).abs() <= 4.
                            && (rect.height() * ppp - 1308.).abs() <= 4.
                    } else {
                        ctx.input(|i| i.viewport().fullscreen) != Some(false)
                    };
                    if dimensions
                        && (ctx.zoom_factor() - scale).abs() < 0.01
                        && r.style_rect == Some(rect)
                    {
                        r.stable_frames += 1;
                    } else {
                        r.stable_frames = 0;
                    }
                    r.style_rect = Some(rect);
                    if r.stable_frames >= 2 {
                        r.phase = 2;
                    }
                }
                if r.phase != 2 {
                    assert!(
                        self.started.elapsed().as_secs() < 15,
                        "LdapPassword native window geometry did not settle"
                    );
                    ctx.data_mut(|d| d.insert_temp(key, r));
                    ctx.request_repaint_after(std::time::Duration::from_millis(25));
                    return;
                }
                true
            }

            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(name) => {
                let viewport = self.metrics.main_viewport.unwrap_or(ctx.content_rect());
                let target = if name == "reset-header" {
                    self.metrics.reset.map(|r| (r, ctx.content_rect()))
                } else if name == "confirm-reset" {
                    self.metrics.confirm_reset.map(|r| (r, viewport))
                } else {
                    self.metrics
                        .ldap_password
                        .controls
                        .iter()
                        .find(|c| c.0 == name)
                        .map(|c| (c.1, c.2))
                };
                match self.ldap_password_click(
                    ctx,
                    target,
                    name.starts_with("case-")
                        || name.starts_with("bind-")
                        || name.starts_with("old-")
                        || name.starts_with("replacement-"),
                ) {
                    Some(done) => done,
                    None => {
                        r.reveal_after = ctx.input(|i| i.time) + 0.025;
                        false
                    }
                }
            }
            Op::Field(name, value) => {
                assert!(matches!(name.as_str(), "config" | "command"));
                let field = if name == "command" {
                    self.metrics.ldap_password.command
                } else {
                    self.metrics.ldap_password.config
                }
                .map(|v| (v.0, v.1));
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, field.map(|f| f.0)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        field.map(|f| f.1),
                        "native password workspace field focus"
                    );
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
                    if value.is_empty() {
                        for pressed in [true, false] {
                            self.pending_events.push(egui::Event::Key {
                                key: egui::Key::Backspace,
                                physical_key: None,
                                pressed,
                                repeat: false,
                                modifiers: Default::default(),
                            });
                        }
                    } else {
                        self.pending_events.push(egui::Event::Paste(value));
                    }
                    r.phase = 2;
                    false
                } else {
                    let source = &self.scenarios[self.selected.unwrap()];
                    let engine::State::LdapPassword { simulation } =
                        &self.progress.attempts[&source.id].state
                    else {
                        unreachable!()
                    };
                    let scope = if simulation.view == teaching::View::Example {
                        &simulation.practice
                    } else {
                        &simulation.assignment
                    };
                    assert_eq!(
                        if name == "command" {
                            scope.draft.command.clone()
                        } else {
                            scope.draft.config.clone()
                        },
                        value,
                        "Native paste must reach the actual password workspace field"
                    );
                    true
                }
            }
            Op::Grade { practice, passed } => {
                let source = &self.scenarios[self.selected.unwrap()];
                let definition = work::source(source).unwrap();
                let engine::State::LdapPassword { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let (task, scope) = if practice {
                    (&definition.example, &simulation.practice)
                } else {
                    (&definition.initial, &simulation.assignment)
                };
                assert_eq!(scope.checked, passed.is_some());
                if let Some(expected) = passed {
                    let rows = scope.checks(task);
                    let count = rows.iter().filter(|r| r.0).count();
                    assert_eq!(
                        self.metrics.ldap_password.result.map(|r| (r.0, r.1)),
                        Some((count, rows.len()))
                    );
                    assert_eq!(
                        count == rows.len(),
                        expected,
                        "Actual configuration/ordered-frame grade: {rows:?}"
                    );
                }
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&source.id], source),
                    !practice && passed == Some(true)
                );
                if practice {
                    assert_eq!(
                        simulation.assignment,
                        work::Session::new(definition).unwrap().assignment
                    );
                }
                true
            }
            Op::Cases(passed, total) => {
                assert_eq!(
                    self.metrics.ldap_password.result.map(|r| (r.0, r.1)),
                    Some((passed, total))
                );
                true
            }
            Op::Trial(text) => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::LdapPassword { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let scope = if simulation.view == teaching::View::Example {
                    &simulation.practice
                } else {
                    &simulation.assignment
                };
                assert!(
                    scope
                        .output
                        .as_ref()
                        .is_some_and(|r| r.feedback.to_lowercase().contains(&text.to_lowercase())),
                    "Native trial feedback must match actual observation: {:?}",
                    scope.output
                );
                true
            }
            Op::ProbeMismatch => {
                let source = &self.scenarios[self.selected.unwrap()];
                let engine::State::LdapPassword { simulation } =
                    &self.progress.attempts[&source.id].state
                else {
                    unreachable!()
                };
                let out = simulation.assignment.output.as_ref().unwrap();
                assert!(out.applied && out.probes.iter().any(|p| p.accepted != p.expected));
                assert!(out.directory_preserved);
                true
            }
            Op::Fresh => {
                let source = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[&source.id].state).unwrap(),
                    serde_json::to_value(&source.initial).unwrap()
                );
                true
            }
            Op::Save => {
                self.save();
                assert!(
                    self.storage_error.is_none(),
                    "native save {:?}",
                    self.storage_error
                );
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            op @ (Op::Shot(_) | Op::ShotAt(_, _)) => {
                let (name, target_key) = match op {
                    Op::Shot(n) => (n, None),
                    Op::ShotAt(n, k) => (n, Some(k)),
                    _ => unreachable!(),
                };
                if r.phase == 0 {
                    let now = ctx.input(|i| i.time);
                    r.reveal_after = now + 0.15;
                    let target = if let Some(key) = target_key {
                        if key == "config" {
                            self.metrics.ldap_password.config.map(|v| v.0)
                        } else if key == "config-start" {
                            // Capture the first two of the font-sized twelve rows,
                            // rather than centering a mostly empty editor's bottom.
                            self.metrics.ldap_password.config.map(|v| {
                                egui::Rect::from_min_size(
                                    v.0.min,
                                    egui::vec2(v.0.width(), v.0.height() / 6.),
                                )
                            })
                        } else if key == "output" {
                            self.metrics.ldap_password.output
                        } else {
                            self.metrics
                                .ldap_password
                                .controls
                                .iter()
                                .find(|v| v.0 == key)
                                .map(|v| v.1)
                        }
                    } else {
                        self.metrics.ldap_password.result.map(|r| r.2)
                    };
                    if target.is_none_or(|target| self.reveal_smoke_control(ctx, target)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if complete {
            r.at += 1;
            r.frames = 0;
            r.phase = 0;
            r.style_rect = None;
            r.stable_frames = 0;
            r.reveal_after = 0.;
            r.action_started = None;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_capture_phase_requests_a_screenshot_without_context_reentry() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "lpic2-ldap-protected-password")
            .unwrap();
        app.launch(i);
        let engine::State::LdapPassword { simulation } = &mut app
            .progress
            .attempts
            .get_mut("lpic2-ldap-protected-password")
            .unwrap()
            .state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        for _ in 0..3 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 5000.),
                    )),
                    ..Default::default()
                },
                |ctx| app.render(ctx),
            );
        }
        assert!(app.metrics.ldap_password.config.is_some());
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("ldap_password-native-route"),
                Route {
                    ops: vec![Op::ShotAt(
                        "ldap-password-regression-capture".into(),
                        "config".into(),
                    )],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(Default::default(), |ctx| app.ldap_password_tick(ctx));
        assert!(app.capture_pending);
    }
}
