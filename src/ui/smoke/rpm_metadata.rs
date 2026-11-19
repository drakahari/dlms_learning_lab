//! Fixed, private review flow. All repairs go through native simulated-console controls.
use super::*;
use dlms_learning_lab::rpm_packages_terminal::{Outcome, Session};
#[derive(Clone, Debug)]
enum Op {
    Open(&'static str),
    Tab(teaching::View),
    Command(&'static str),
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
}
fn plan() -> Vec<Op> {
    let mut p = vec![];
    for (id, mixed) in [
        ("lpic1-rpm-metadata-preserve", false),
        ("lpic1-rpm-mixed-recovery", true),
    ] {
        p.extend([
            Op::Open(id),
            Op::Shot(format!("rpm-meta-{id}-introduction"), false),
            Op::Tab(teaching::View::Example),
            Op::Shot(format!("rpm-meta-{id}-practice-before"), false),
            Op::Command("rpm -q gallery-panel"),
            Op::Command("rpm -ql gallery-panel"),
            Op::Command("rpm --restore gallery-panel"),
        ]);
        if mixed {
            p.extend([
                Op::Command("rpm --restore gallery-cards"),
                Op::Check(true, false),
                Op::Shot(format!("rpm-meta-{id}-practice-partial"), true),
                Op::Command("rpm --reinstall /museum/rpms/gallery-cards-v1.rpm"),
            ]);
        }
        p.extend([
            Op::Check(true, true),
            Op::Shot(format!("rpm-meta-{id}-practice-pass"), true),
            Op::Tab(teaching::View::Assignment),
            Op::Shot(format!("rpm-meta-{id}-assignment"), false),
            Op::Inspect,
            Op::Header,
            Op::Pick("queue-agent-v1"),
            Op::Path,
            Op::Pick("/srv/teaching/queue-agent.conf"),
            Op::InspectText("style=site"),
            Op::Shot(format!("rpm-meta-{id}-current-site-text"), false),
            Op::Inspect,
            Op::Check(false, false),
            Op::Command("rpm --restore queue-agent"),
        ]);
        if mixed {
            p.extend([
                Op::Check(false, false),
                Op::Shot(format!("rpm-meta-{id}-assignment-partial"), true),
                Op::Command("rpm --reinstall /rpms/report-pages-v1.rpm"),
            ]);
        }
        p.extend([
            Op::Check(false, true),
            Op::Save,
            Op::Command("rpm -e desk-notes"),
            Op::Check(false, false),
            Op::Shot(format!("rpm-meta-{id}-invalidation"), true),
            Op::Command("rpm -i /rpms/desk-notes-v1.rpm"),
            Op::Check(false, true),
            Op::Save,
            Op::Theme(true),
            Op::Shot(format!("rpm-meta-{id}-light150"), false),
            Op::Shot(format!("rpm-meta-{id}-light150-controls"), true),
            Op::Theme(false),
            Op::Shot(format!("rpm-meta-{id}-dark200"), false),
            Op::Shot(format!("rpm-meta-{id}-dark200-controls"), true),
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
    pub(in crate::ui) fn rpm_metadata_tick(&mut self, ctx: &egui::Context) {
        let gate = egui::Id::new("rpm-metadata-native-frame");
        let frame = ctx.cumulative_frame_nr();
        if ctx.data(|d| d.get_temp::<u64>(gate)) == Some(frame) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(gate, frame));
        let key = egui::Id::new("rpm-metadata-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: plan(),
                ..Default::default()
            });
        if r.done {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "RPM_METADATA_NATIVE_COMPLETE: actual separate practice, read-only site text, partial/content repairs, current invalidation, save/read/reset, light150/dark200"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        let now = ctx.input(|i| i.time);
        if now < r.until {
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        self.smoke_stage = 191000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("RPM_METADATA_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(r.frames < 10000, "RPM native step timed out: {op:?}");
        let done = match op {
            Op::Open(id) => {
                self.scale = 1.;
                ctx.set_zoom_factor(1.);
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                true
            }
            Op::Tab(v) => {
                let rect = self.metrics.rpm.tabs.iter().find(|r| r.0 == v).map(|r| r.1);
                self.click_visible_smoke_control(ctx, rect)
            }
            Op::Command(text) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.rpm.command.map(|r| r.0))
                    {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.case_lesson_type(text);
                    r.phase = 2;
                    false
                } else if r.phase == 2 {
                    let (_, a, p) = selected(self);
                    assert_eq!(console(a, p).command, text);
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
                } else {
                    let (s, a, p) = selected(self);
                    let e = console(a, p).entries.last().unwrap();
                    assert_eq!(e.command, text);
                    assert!(
                        matches!(&e.output, Outcome::Utility { status: 0, .. }),
                        "{text}: {:?}",
                        e.output
                    );
                    if p {
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, s));
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
    fn metadata_capture_phase_uses_real_feedback_and_requests_one_screenshot() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        a.metrics.rpm.result_rows.push((
            0,
            false,
            egui::Rect::from_min_size(egui::pos2(10., 10.), egui::vec2(80., 30.)),
            egui::Rect::EVERYTHING,
        ));
        let key = egui::Id::new("rpm-metadata-native-route");
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
        let out = ctx.run(egui::RawInput::default(), |ctx| a.rpm_metadata_tick(ctx));
        assert!(a.capture_pending);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn metadata_route_retains_both_assignments_practice_partials_and_all_capture_modes() {
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
