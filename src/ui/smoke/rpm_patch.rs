//! Fixed, private review flow. All repairs go through native simulated-console controls.
use super::*;
use dlms_learning_lab::rpm_packages_terminal::{Outcome, Session};
#[derive(Clone, Debug)]
pub(super) enum Op {
    ColdAppearance,
    ColdFinish,
    FocusCommand,
    Worked,
    Draft(String),
    Open(&'static str),
    Tab(teaching::View),
    Command(String),
    Refuse(String),
    Narrow,
    Check(bool, bool),
    Shot(String, bool),
    Save,
    Theme(bool),
    Reset,
    Fresh,
    Inspect,
    Header,
    Path,
    Pick(&'static str),
    InspectText(&'static str),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    until: f64,
    done: bool,
    began: Option<Instant>,
    requested: Option<egui::Vec2>,
    stable: u8,
    before: Option<(engine::State, bool)>,
}
fn plan() -> Vec<Op> {
    let mut p = vec![];
    for (id, pair, prefix, example) in [
        ("lpic2-vendor-backport", false, "relay", "atrium"),
        ("lpic2-vendor-patch-set", true, "dispatch", "gallery"),
    ] {
        p.extend([
            Op::Open(id),
            Op::Shot(format!("rpm-patch-{prefix}-introduction"), false),
            Op::Worked,
            Op::Shot(format!("rpm-patch-{prefix}-worked"), false),
            Op::Tab(teaching::View::Example),
            Op::Shot(format!("rpm-patch-{prefix}-practice"), false),
            Op::Command(format!("rpm -q {example}-agent")),
        ]);
        let update = if pair {
            format!(
                "rpm -U /media/{example}/{example}-agent-1.0-4.train8.noarch.rpm /media/{example}/{example}-codec-3.0-2.train8.noarch.rpm"
            )
        } else {
            format!("rpm -U /media/{example}/{example}-agent-1.0-4.train8.noarch.rpm")
        };
        p.extend([
            Op::Command(update),
            Op::Check(true, true),
            Op::Shot(format!("rpm-patch-{prefix}-practice-pass"), true),
            Op::Command(format!("rpm -e {example}-notes")),
            Op::Check(true, false),
            Op::Shot(
                format!("rpm-patch-{prefix}-practice-handover-missing"),
                true,
            ),
            Op::Command(format!(
                "rpm -i /media/{example}/{example}-notes-1.0-1.train8.noarch.rpm"
            )),
            Op::Check(true, true),
            Op::Tab(teaching::View::Assignment),
            Op::Shot(format!("rpm-patch-{prefix}-assignment"), false),
            Op::Check(false, false),
        ]);
        if pair {
            p.push(Op::Refuse(
                "rpm -U /media/dispatch/dispatch-agent-1.0-4.train8.noarch.rpm".into(),
            ));
            p.push(Op::Shot(
                "rpm-patch-dispatch-dependency-refusal".into(),
                false,
            ));
            p.push(Op::Command(
                "rpm -U /media/dispatch/dispatch-codec-3.0-2.train8.noarch.rpm".into(),
            ));
            p.push(Op::Check(false, false));
            p.push(Op::Shot("rpm-patch-dispatch-partial".into(), true));
        }
        p.extend([
            Op::Command(format!(
                "rpm -U /media/{prefix}/{prefix}-agent-1.0-4.train8.noarch.rpm"
            )),
            Op::Check(false, true),
            Op::Save,
            Op::Command(format!("rpm -e {prefix}-notes")),
            Op::Check(false, false),
            Op::Shot(format!("rpm-patch-{prefix}-handover-missing"), true),
            Op::Command(format!(
                "rpm -i /media/{prefix}/{prefix}-notes-1.0-1.train8.noarch.rpm"
            )),
            Op::Check(false, true),
            Op::Inspect,
            Op::Header,
            Op::Pick(if pair {
                "dispatch-agent-fixed"
            } else {
                "relay-agent-fixed"
            }),
            Op::Path,
            Op::Pick(if pair {
                "/srv/dispatch/agent.conf"
            } else {
                "/srv/relay/agent.conf"
            }),
            Op::InspectText("zone=site-east"),
            Op::InspectText("agent.conf.rpmnew"),
            Op::InspectText("zone=vendor-v2"),
            Op::Shot(format!("rpm-patch-{prefix}-retained-site-text"), false),
            Op::Inspect,
            Op::Save,
            Op::Command(format!(
                "rpm -U /media/{prefix}/{prefix}-agent-1.0-5.train9.noarch.rpm"
            )),
            Op::Check(false, false),
            Op::Shot(format!("rpm-patch-{prefix}-wrong-release"), true),
            Op::Theme(true),
            Op::Narrow,
            Op::Shot(format!("rpm-patch-{prefix}-light150"), false),
            Op::Shot(format!("rpm-patch-{prefix}-light150-controls"), true),
            Op::Theme(false),
            Op::Narrow,
            Op::Shot(format!("rpm-patch-{prefix}-dark200"), false),
            Op::Shot(format!("rpm-patch-{prefix}-dark200-controls"), true),
            Op::Reset,
            Op::Fresh,
        ]);
    }
    p
}

fn selected(a: &LabApp) -> (&Scenario, &Attempt, bool) {
    let s = &a.scenarios[a.selected.unwrap()];
    let at = &a.progress.attempts[&s.id];
    let p = at
        .workspace
        .as_ref()
        .unwrap()
        .rpm_learning
        .as_ref()
        .unwrap()
        .view
        == teaching::View::Example;
    (s, at, p)
}
fn console(at: &Attempt, p: bool) -> &Session {
    let w = at.workspace.as_ref().unwrap();
    if p {
        &w.rpm_learning.as_ref().unwrap().terminal
    } else {
        w.rpm_session.as_ref().unwrap()
    }
}
impl LabApp {
    pub(in crate::ui) fn rpm_patch_tick(&mut self, ctx: &egui::Context) {
        self.rpm_patch_flow(ctx, None);
    }
    pub(super) fn rpm_patch_flow(&mut self, ctx: &egui::Context, provided: Option<&[Op]>) {
        let gate = egui::Id::new("rpm-patch-native-frame");
        let frame = ctx.cumulative_frame_nr();
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(frame) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, frame));
        let key = egui::Id::new("rpm-patch-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: provided.map_or_else(plan, <[Op]>::to_vec),
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "RPM_PATCH_NATIVE_COMPLETE: actual separate practice, advisory/release errors, dependency refusal/partial repair, retained site text, current invalidation, save/read/reset, light150/dark200"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let began = r.began.get_or_insert_with(Instant::now);
        assert!(
            began.elapsed() < std::time::Duration::from_secs(15),
            "Bounded patch native action deadline"
        );
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        r.until = now + 0.1; // Pace eligible polls; passive redraws do not consume the action budget.
        self.smoke_stage = 204000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("RPM_PATCH_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(r.frames < 1200, "RPM native step timed out: {op:?}");
        let refused = matches!(&op, Op::Refuse(_));
        let draft = matches!(&op, Op::Draft(_));
        let done = match op {
            Op::ColdAppearance => self.samba_reopen_appearance(ctx),
            Op::ColdFinish => {
                self.finish_rpm_patch_reopen(ctx);
                true
            }
            Op::Worked => self.reveal_smoke_control(
                ctx,
                self.metrics
                    .example_action
                    .expect("Introduction example action"),
            ),
            Op::FocusCommand => {
                self.click_visible_smoke_control(ctx, self.metrics.rpm.command.map(|r| r.0))
            }
            Op::Open(id) => {
                if self.reopen.is_none() {
                    self.scale = 1.;
                    ctx.set_zoom_factor(1.);
                    self.theme_id = theme::Preferences::default().theme;
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                true
            }
            Op::Tab(v) => {
                let rect = self.metrics.rpm.tabs.iter().find(|r| r.0 == v).map(|r| r.1);
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text) | Op::Refuse(text) | Op::Draft(text) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.rpm.command.map(|r| r.0))
                    {
                        let (source, attempt, _) = selected(self);
                        r.before = Some((
                            attempt.state.clone(),
                            progress::current_solved(attempt, source),
                        ));
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(&text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let (_, a, p) = selected(self);
                    assert_eq!(console(a, p).command, text);
                    if draft {
                        let (source, attempt, _) = selected(self);
                        let (state, solved) = r.before.take().unwrap();
                        assert_eq!(attempt.state, state);
                        assert_eq!(progress::current_solved(attempt, source), solved);
                        true
                    } else {
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
                    let (s, a, p) = selected(self);
                    let e = console(a, p).entries.last().unwrap();
                    assert_eq!(e.command, text);
                    if refused {
                        assert!(
                            matches!(&e.output, Outcome::SimulatorUnsupported { .. }),
                            "{text}: {:?}",
                            e.output
                        );
                    } else {
                        assert!(
                            matches!(&e.output, Outcome::Utility { status: 0, .. }),
                            "{text}: {:?}",
                            e.output
                        );
                    }
                    if p || refused {
                        let (state, solved) = r.before.take().unwrap();
                        assert_eq!(a.state, state);
                        assert_eq!(progress::current_solved(a, s), solved);
                    }
                    true
                }
            }
            Op::Check(p, want) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.rpm.check) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let (s, a, is_p) = selected(self);
                    assert_eq!(is_p, p);
                    let rows = if p {
                        let l = a.workspace.as_ref().unwrap().rpm_learning.as_ref().unwrap();
                        assert!(l.checked);
                        l.checks(s.rpm_packages_teaching.as_ref().unwrap())
                    } else {
                        assert!(a.workspace.as_ref().unwrap().checked);
                        progress::current_rpm_checks(a, s)
                    };
                    assert_eq!(rows.iter().all(|r| r.0), want);
                    assert_eq!(self.metrics.rpm.result_rows.iter().all(|r| r.1), want);
                    assert_eq!(self.metrics.rpm.result_rows.len(), rows.len());
                    if !p {
                        assert_eq!(progress::current_solved(a, s), want);
                    }
                    true
                }
            }
            Op::Inspect => self.click_visible_smoke_control(ctx, self.metrics.rpm.inspector),
            Op::Header => self.click_visible_smoke_control(ctx, self.metrics.rpm.inspect_header),
            Op::Path => self.click_visible_smoke_control(ctx, self.metrics.rpm.inspect_path),
            Op::Pick(path) => {
                let rect = self
                    .metrics
                    .rpm
                    .inspect_choices
                    .iter()
                    .find(|r| r.0 == path)
                    .map(|r| r.1);
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::InspectText(text) => {
                assert!(
                    self.metrics
                        .rpm
                        .inspection_text
                        .iter()
                        .any(|s| s.contains(text)),
                    "Missing actual inspected site text"
                );
                true
            }
            Op::Shot(name, result) => {
                if result && r.phase == 0 {
                    let row = self
                        .metrics
                        .rpm
                        .result_rows
                        .iter()
                        .find(|r| !r.1)
                        .or(self.metrics.rpm.result_rows.first())
                        .expect("Checked feedback");
                    if self.reveal_smoke_control(ctx, row.2) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == u8::from(result) {
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
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let read = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(read).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
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
            Op::Narrow => {
                if r.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    r.phase = 1;
                    false
                } else if r.phase == 1 {
                    r.requested = Some(capture_size::request(ctx));
                    r.phase = 2;
                    false
                } else {
                    let actual = ctx.content_rect().size() * ctx.pixels_per_point();
                    let native = ctx
                        .input(|i| i.viewport().native_pixels_per_point)
                        .expect("Native geometry");
                    if capture_size::fits(actual, r.requested.unwrap(), native) {
                        r.stable += 1;
                    } else {
                        r.stable = 0;
                    }
                    if r.stable >= 2 {
                        println!(
                            "RPM_PATCH_GEOMETRY actual={actual:?} requested={:?} native={native} zoom={}",
                            r.requested,
                            ctx.zoom_factor()
                        );
                        true
                    } else {
                        false
                    }
                }
            }
            Op::Reset => {
                let rect = if r.phase == 0 {
                    self.metrics.reset
                } else {
                    self.metrics.confirm_reset
                };
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
            Op::Fresh => {
                let (s, a, _) = selected(self);
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
                let w = a.workspace.as_ref().unwrap();
                assert!(w.rpm_session.as_ref().unwrap().entries.is_empty());
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
            r.began = None;
            r.before = None;
            r.requested = None;
            r.stable = 0;
            r.phase = 0;
            r.frames = 0;
            r.until = now + 0.07;
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn patch_capture_phase_uses_real_feedback_and_requests_one_screenshot() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        a.metrics.rpm.result_rows.push((
            0,
            false,
            egui::Rect::from_min_size(egui::pos2(10., 10.), egui::vec2(80., 30.)),
            egui::Rect::EVERYTHING,
        ));
        let key = egui::Id::new("rpm-patch-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("synthetic-capture-only".into(), true)],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| a.rpm_patch_tick(ctx));
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn patch_route_retains_both_assignments_practice_partials_and_all_capture_modes() {
        let p = plan();
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Open(_))).count(), 2);
        assert!(p.iter().any(|o| matches!(o, Op::Check(true, false))));
        for light in [true, false] {
            assert_eq!(
                p.iter()
                    .filter(|o| matches!(o,Op::Theme(v)if *v==light))
                    .count(),
                2
            );
        }
        assert_eq!(p.iter().filter(|o| matches!(o, Op::Fresh)).count(), 2);
    }
}
