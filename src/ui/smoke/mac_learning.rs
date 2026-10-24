//! Private MAC-fixture actual-input plan. No solved/reference State publication.
use super::*;
use engine::State;
const START: usize = 16000;
const ID: &str = "development-mac-learning";
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Control {
    Try,
    Assignment,
    Insert,
    ActionPicker,
    Action(usize),
    SourcePicker,
    Source(String),
    Destination,
    Send,
    Check,
    Tools,
    Settings,
    Time,
    Advance,
    Port,
    Apply,
    Reset,
    Cancel,
    Confirm,
    Challenge,
}
#[derive(Clone)]
enum Step {
    Setup,
    Wait,
    Click(Control),
    Type(String),
    Assert(&'static str),
    FeedbackShot(&'static str),
    Shot(&'static str),
    Record,
    Readback,
    Theme(&'static str, f32, [f32; 2]),
    Scale(f32),
    Finish,
}
fn send(plan: &mut Vec<Step>, source: &str, destination: &str) {
    plan.extend([
        Step::Click(Control::SourcePicker),
        Step::Click(Control::Source(source.into())),
        Step::Click(Control::Destination),
        Step::Type(destination.into()),
        Step::Click(Control::Send),
        Step::Wait,
    ]);
}
fn example_send(plan: &mut Vec<Step>, index: usize) {
    plan.extend([
        Step::Click(Control::ActionPicker),
        Step::Click(Control::Action(index)),
        Step::Click(Control::Insert),
        Step::Click(Control::Send),
        Step::Wait,
    ]);
}
fn check(plan: &mut Vec<Step>, assertion: &'static str, shot: &'static str) {
    plan.extend([
        Step::Click(Control::Check),
        Step::Wait,
        Step::Assert(assertion),
        Step::FeedbackShot(shot),
    ]);
}
fn plan(s: &Scenario) -> Vec<Step> {
    assert_eq!(s.id, ID);
    let mut p = vec![
        Step::Setup,
        Step::Wait,
        Step::Shot("mac-development-introduction"),
        Step::Click(Control::Try),
        Step::Wait,
        Step::Click(Control::Insert),
        Step::Assert("insert-only"),
        Step::Shot("mac-development-insert-unexecuted"),
        Step::Click(Control::Send),
        Step::Wait,
    ];
    example_send(&mut p, 1);
    check(&mut p, "example-pass", "mac-development-example-complete");
    p.extend([
        Step::Click(Control::Tools),
        Step::Click(Control::Time),
        Step::Type("300".into()),
        Step::Click(Control::Advance),
        Step::Wait,
    ]);
    check(&mut p, "example-expired", "mac-development-example-expired");
    example_send(&mut p, 0);
    example_send(&mut p, 1);
    check(&mut p, "example-pass", "mac-development-example-repaired");
    p.extend([
        Step::Readback,
        Step::Click(Control::Assignment),
        Step::Wait,
        Step::Shot("mac-development-assignment"),
    ]);
    p.extend([
        Step::Click(Control::Destination),
        Step::Type("not-a-mac".into()),
        Step::Click(Control::Send),
        Step::Wait,
    ]);
    check(&mut p, "wrong-atomic", "mac-development-wrong-input");
    send(&mut p, "c", "02:00:00:00:00:01");
    check(&mut p, "partial", "mac-development-partial");
    send(&mut p, "a", "02:00:00:00:00:03");
    check(&mut p, "pass", "mac-development-complete");
    for (theme, scale, size, pending, wrong, complete) in [
        (
            "dlms-light",
            1.5,
            [1600., 1000.],
            "mac-development-light-150-pending",
            "mac-development-light-150-port-error",
            "mac-development-light-150-complete",
        ),
        (
            "dlms-dark",
            2.,
            [960., 900.],
            "mac-development-dark-200-pending",
            "mac-development-dark-200-port-error",
            "mac-development-dark-200-complete",
        ),
    ] {
        p.extend([
            Step::Theme(theme, scale, size),
            Step::Scale(scale),
            Step::Click(Control::Settings),
            Step::Click(Control::Port),
            Step::Type("20".into()),
            Step::Wait,
            Step::Assert("pending"),
            Step::Shot(pending),
            Step::Click(Control::Apply),
            Step::Wait,
        ]);
        check(&mut p, "wrong-port", wrong);
        p.extend([
            Step::Click(Control::Port),
            Step::Type("10".into()),
            Step::Click(Control::Apply),
            Step::Wait,
        ]);
        send(&mut p, "a", "02:00:00:00:00:03");
        check(&mut p, "pass", complete);
        p.push(Step::Click(Control::Settings));
    }
    p.extend([
        Step::Readback,
        Step::Record,
        Step::Click(Control::Reset),
        Step::Click(Control::Cancel),
        Step::Assert("cancel-exact"),
        Step::Click(Control::Reset),
        Step::Click(Control::Confirm),
        Step::Wait,
        Step::Assert("reset"),
        Step::Click(Control::Assignment),
        Step::Wait,
    ]);
    // Alternate actual send order, never an installed reference state.
    send(&mut p, "a", "02:00:00:00:00:03");
    send(&mut p, "c", "02:00:00:00:00:01");
    check(&mut p, "pass", "mac-development-alternate-complete");
    p.extend([
        Step::Readback,
        Step::Click(Control::Reset),
        Step::Click(Control::Confirm),
        Step::Wait,
        Step::Assert("reset"),
        Step::Click(Control::Challenge),
        Step::Wait,
        Step::Assert("challenge"),
        Step::Shot("mac-development-challenge-dark-200"),
        Step::Readback,
        Step::Finish,
    ]);
    p
}
pub(super) fn ctrl_a() -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect()
}
impl LabApp {
    pub(super) fn mac_native_click(&mut self, ctx: &egui::Context, target: &Control) -> bool {
        let rect = match target {
            Control::Try => self.metrics.example_action,
            Control::Assignment => self.metrics.assignment_action,
            Control::Insert => self.metrics.command_insert,
            Control::ActionPicker => self.metrics.mac_example_action,
            Control::Action(n) => self
                .metrics
                .mac_example_choices
                .iter()
                .find(|(index, _)| index == n)
                .map(|(_, r)| *r),
            Control::SourcePicker => self.metrics.mac_source,
            Control::Source(name) => self
                .metrics
                .mac_sources
                .iter()
                .find(|(id, _)| id == name)
                .map(|(_, r)| *r),
            Control::Destination => self.metrics.mac_destination,
            Control::Send => self.metrics.mac_send,
            Control::Check => self.metrics.check,
            Control::Tools => self.metrics.mac_tools,
            Control::Settings => self.metrics.mac_settings,
            Control::Time => self.metrics.mac_time,
            Control::Advance => self.metrics.mac_advance,
            Control::Port => self.metrics.vlan_fields.first().map(|(_, r)| *r),
            Control::Apply => self.metrics.editor_save,
            Control::Reset => self.metrics.reset,
            Control::Cancel => self.metrics.reset_cancel,
            Control::Confirm => self.metrics.confirm_reset,
            Control::Challenge => self.metrics.mode_challenge,
        };
        if matches!(target, Control::Action(_) | Control::Source(_)) {
            let clip = if matches!(target, Control::Action(_)) {
                self.metrics.mac_action_popup
            } else {
                self.metrics.mac_source_popup
            };
            self.mac_popup_choice(ctx, rect, clip)
        } else {
            self.click_visible_smoke_control(ctx, rect)
        }
    }
    /// Scroll only this actual popup; never click a clipped/offscreen choice.
    pub(in crate::ui) fn mac_popup_choice(
        &mut self,
        ctx: &egui::Context,
        target: Option<egui::Rect>,
        popup: Option<egui::Rect>,
    ) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let target = target.expect("actual popup choice before click");
        let visible = popup
            .expect("actual popup clip before click")
            .intersect(ctx.content_rect());
        if smoke_control_visible(target, visible, ctx.pixels_per_point()) {
            return self.click_smoke_control(ctx, Some(target));
        }
        let count = ctx.data_mut(|d| {
            let key = egui::Id::new(("mac-popup-scroll", self.smoke_stage));
            let n = d.get_temp::<u8>(key).unwrap_or(0) + 1;
            d.insert_temp(key, n);
            n
        });
        assert!(
            count <= 20,
            "popup choice must become visible; target={target:?} actualpopup={visible:?}"
        );
        assert!(
            visible.is_finite() && visible.width() > 0. && visible.height() > 0.,
            "valid actual popup clip"
        );
        let delta = if target.bottom() > visible.bottom() {
            visible.bottom() - target.bottom()
        } else if target.top() < visible.top() {
            visible.top() - target.top()
        } else {
            0.
        };
        self.pending_events
            .push(egui::Event::PointerMoved(visible.center()));
        self.pending_events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0., delta.clamp(-200., 200.)),
            modifiers: egui::Modifiers::default(),
        });
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn mac_native_other_unchanged(&self, ctx: &egui::Context) {
        let (other, value) = ctx
            .data(|d| d.get_temp::<(String, serde_json::Value)>(egui::Id::new("mac-native-other")))
            .expect("private other context recorded");
        assert_eq!(
            serde_json::to_value(&self.progress.attempts[&other]).unwrap(),
            value
        );
    }
    pub(super) fn mac_learning_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/mac-learning-preview.json"
        ))
        .expect("reviewed private fixture")
        .remove(0);
        let steps = plan(&s);
        let step = steps.get(stage - START).expect("finite MAC native plan");
        if matches!(step, Step::Setup) {
            assert!(self.smoke && self._smoke_profile.is_some());
            assert!(
                self.path
                    .starts_with(self._smoke_profile.as_ref().unwrap().path())
            );
            assert!(!pack::bundled().unwrap().iter().any(|lab| lab.id == ID));
            let other = self.scenarios[0].clone();
            self.progress
                .attempts
                .entry(other.id.clone())
                .or_insert_with(|| Attempt::new(&other, 1));
            let value = serde_json::to_value(&self.progress.attempts[&other.id]).unwrap();
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("mac-native-other"), (other.id.clone(), value))
            });
            assert!(!self.scenarios.iter().any(|lab| lab.id == ID));
            self.scenarios.push(s.clone());
            let i = self.scenarios.len() - 1;
            self.launch(i);
            self.set_mac_view(i, teaching::View::Introduction);
            self.theme_id = "dlms-dark".into();
            self.scale = 1.;
            theme::apply(ctx, &self.theme_id);
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            return (true, None);
        }
        let i = self.scenarios.iter().position(|lab| lab.id == ID).unwrap();
        match step {
            Step::Setup => unreachable!(),
            Step::Wait => {}
            Step::Click(target) => {
                if !self.mac_native_click(ctx, target) {
                    return (false, None);
                }
            }
            Step::Type(text) => {
                assert!(text.len() <= 20);
                self.pending_events = ctrl_a();
                self.pending_events.push(egui::Event::Text(text.clone()));
            }
            Step::Shot(name) => return (true, Some((*name).into())),
            Step::FeedbackShot(name) => {
                if !self.reveal_smoke_control(
                    ctx,
                    self.metrics.feedback.expect("actual MAC feedback block"),
                ) {
                    return (false, None);
                }
                return (true, Some((*name).into()));
            }
            Step::Record => {
                let value = serde_json::to_value(&self.progress.attempts[ID]).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("mac-native-before-reset"), value));
            }
            Step::Readback => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(&loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                self.mac_native_other_unchanged(ctx);
            }
            Step::Theme(name, scale, size) => {
                self.theme_id = (*name).into();
                self.scale = *scale;
                theme::apply(ctx, name);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    size[0], size[1],
                )));
            }
            Step::Scale(scale) => {
                if (ctx.zoom_factor() - *scale).abs() > 0.01 {
                    let n = ctx.data_mut(|d| {
                        let id = egui::Id::new(("mac-native-scale", stage));
                        let n = d.get_temp::<u8>(id).unwrap_or(0) + 1;
                        d.insert_temp(id, n);
                        n
                    });
                    assert!(n < 20, "native MAC text scale must settle");
                    return (false, None);
                }
            }
            Step::Assert(name) => {
                let a = &self.progress.attempts[ID];
                let w = a.workspace.as_ref().unwrap();
                let State::MacLearning { simulation } = &a.state else {
                    panic!()
                };
                let learning = w.mac_learning.as_ref().unwrap();
                let l = s.mac_teaching.as_ref().unwrap();
                match *name {
                    "insert-only" => {
                        assert_eq!(a.state, s.initial);
                        assert!(learning.simulation.entries().is_empty());
                        assert!(!learning.checked);
                        assert_eq!(learning.draft.source, "y");
                        assert_eq!(learning.draft.destination, "02:00:00:00:30:01");
                        assert!(!progress::current_solved(a, &s));
                    }
                    "example-pass" => {
                        let checks = learning.checks(l);
                        assert!(!checks.is_empty() && checks.iter().all(|c| c.0));
                        assert_eq!(a.state, s.initial);
                        assert!(!progress::current_solved(a, &s));
                    }
                    "example-expired" => {
                        assert_eq!(learning.simulation.now(), 300);
                        assert!(learning.simulation.entries().is_empty());
                        let checks = learning.checks(l);
                        assert!(!checks.is_empty() && checks.iter().any(|c| !c.0));
                        assert_eq!(a.state, s.initial);
                    }
                    "wrong-atomic" => {
                        assert_eq!(a.state, s.initial);
                        assert!(
                            self.notice
                                .as_ref()
                                .is_some_and(|e| e.contains("hexadecimal"))
                        );
                        assert!(!progress::current_solved(a, &s));
                    }
                    "partial" => {
                        let checks = engine::checks(&a.state, &s.goal);
                        assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
                        assert!(!progress::current_solved(a, &s));
                    }
                    "pass" => {
                        assert!(progress::current_solved(a, &s));
                        assert!(self.metrics.completion);
                    }
                    "pending" => {
                        assert_eq!(
                            w.mac_draft.as_ref().unwrap().configuration.switches["s1"].ports["a"]
                                .access,
                            "20"
                        );
                        assert!(engine::solved(&a.state, &s.goal));
                        assert!(!progress::current_solved(a, &s));
                        assert!(!self.metrics.completion);
                    }
                    "wrong-port" => {
                        assert!(matches!(
                            &simulation.fabric().switches["s1"].ports["a"].mode,
                            dlms_learning_lab::switching::Mode::Access { vlan: 20 }
                        ));
                        assert!(!progress::current_solved(a, &s));
                    }
                    "cancel-exact" => {
                        let before = ctx
                            .data(|d| {
                                d.get_temp::<serde_json::Value>(egui::Id::new(
                                    "mac-native-before-reset",
                                ))
                            })
                            .unwrap();
                        assert_eq!(serde_json::to_value(a).unwrap(), before);
                    }
                    "reset" => {
                        assert_eq!(a.state, s.initial);
                        assert!(w.mac_draft.as_ref().unwrap().matches(simulation));
                        assert_eq!(learning.simulation, l.fixture);
                        assert!(!learning.checked);
                        assert!(!progress::current_solved(a, &s));
                        assert!(
                            ctx.data(|d| d
                                .get_temp::<dlms_learning_lab::mac_learning::Observation>(
                                    egui::Id::new(("mac-trace", i, false))
                                ))
                            .is_none()
                        );
                        assert!(
                            ctx.data(|d| d
                                .get_temp::<dlms_learning_lab::mac_learning::Observation>(
                                    egui::Id::new(("mac-trace", i, true))
                                ))
                            .is_none()
                        );
                    }
                    "challenge" => {
                        assert!(a.challenge && self.mac_view(i) == teaching::View::Assignment);
                        assert!(
                            self.metrics.lesson_tabs.is_empty()
                                && self.metrics.command_insert.is_none()
                                && !self.metrics.help_visible
                        );
                        assert!(!progress::current_solved(a, &s));
                    }
                    _ => panic!("unknown fixed MAC assertion"),
                }
                self.mac_native_other_unchanged(ctx);
            }
            Step::Finish => {
                assert!(self.smoke);
                println!(
                    "PASS scoped private MAC workspace: real example Insert/Send/Check/expiry/repair, full assignment malformed/partial/current/alternate, native port-edit invalidation and repair at light150/dark200, exact saved-state readback, selected Reset Cancel/Confirm and independent Challenge. Unbundled fixture only; no curriculum/novice/other-platform/screen-reader/cold-restart claim."
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
    #[test]
    fn mac_private_native_plan_is_bounded_and_only_populates_actual_input_targets() {
        let s = pack::load(include_str!(
            "../../../tests/fixtures/mac-learning-preview.json"
        ))
        .unwrap()
        .remove(0);
        let steps = plan(&s);
        assert!(steps.len() < 400);
        assert_eq!(steps.iter().filter(|s| matches!(s, Step::Setup)).count(), 1);
        assert_eq!(
            steps.iter().filter(|s| matches!(s, Step::Finish)).count(),
            1
        );
        assert!(
            steps
                .iter()
                .all(|s| !matches!(s,Step::Type(text)if text.len()>20))
        );
        for name in [
            "insert-only",
            "example-expired",
            "partial",
            "pending",
            "cancel-exact",
            "challenge",
        ] {
            assert!(
                steps
                    .iter()
                    .any(|s| matches!(s,Step::Assert(value)if *value==name))
            );
        }
        assert!(
            steps
                .iter()
                .any(|s| matches!(s,Step::Theme(_,scale,_)if *scale==1.5))
        );
        assert!(
            steps
                .iter()
                .any(|s| matches!(s,Step::Theme(_,scale,_)if *scale==2.))
        );
        assert!(!pack::bundled().unwrap().iter().any(|lab| lab.id == ID));
    }
}
