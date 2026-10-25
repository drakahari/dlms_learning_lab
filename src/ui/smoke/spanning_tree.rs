//! Closed private technical fixture, real input events only. No solved State install.
use super::*;
const FIRST: usize = 33000;
const ID: &str = "test-spanning-tree-workspace";
#[derive(Clone, Copy)]
enum Step {
    Open,
    Settle,
    Capture(&'static str),
    Example,
    Insert,
    Apply,
    Check,
    Field(&'static str, &'static str),
    ExamplePass,
    ExampleFail,
    Assignment,
    BridgeMenu,
    Bridge(&'static str),
    Current(bool),
    Scale(bool),
    SaveResume,
    Reset,
    Cancel,
    Confirm,
    ResetProof,
    Challenge,
    Learn,
    Finish,
}
const STEPS: &[Step] = &[
    Step::Open,
    Step::Settle,
    Step::Capture("stp-introduction"),
    Step::Example,
    Step::Insert,
    Step::Capture("stp-example-insert-only"),
    Step::Apply,
    Step::Check,
    Step::ExamplePass,
    Step::Capture("stp-example-pass"),
    Step::Field("sample-core:priority", "61440"),
    Step::Apply,
    Step::Check,
    Step::ExampleFail,
    Step::Capture("stp-example-change"),
    Step::Field("sample-core:priority", "0"),
    Step::Apply,
    Step::Check,
    Step::ExamplePass,
    Step::Assignment,
    Step::Check,
    Step::Current(false),
    Step::Capture("stp-assignment-initial"),
    Step::BridgeMenu,
    Step::Bridge("core"),
    Step::Field("core:priority", "0"),
    Step::Check,
    Step::Current(false),
    Step::Capture("stp-pending-not-applied"),
    Step::Apply,
    Step::Check,
    Step::Current(true),
    Step::Capture("stp-assignment-reference"),
    Step::Field("core:priority", "4096"),
    Step::Apply,
    Step::Check,
    Step::Current(true),
    Step::Capture("stp-legitimate-alternate"),
    Step::Field("core:priority", "32768"),
    Step::Apply,
    Step::Check,
    Step::Current(false),
    Step::Capture("stp-wrong-root"),
    Step::Field("core:priority", "0"),
    Step::Apply,
    Step::Check,
    Step::Current(true),
    Step::Challenge,
    Step::Capture("stp-challenge"),
    Step::Scale(true),
    Step::Settle,
    Step::Field("core:priority", "4096"),
    Step::Capture("stp-light150-typed-input"),
    Step::Apply,
    Step::Check,
    Step::Current(true),
    Step::Capture("stp-light150-typed-alternate"),
    Step::Scale(false),
    Step::Settle,
    Step::Field("core:priority", "0"),
    Step::Capture("stp-dark200-typed-input"),
    Step::Apply,
    Step::Check,
    Step::Current(true),
    Step::Capture("stp-dark200-typed-reference"),
    Step::Learn,
    Step::Field("core:priority", "unfinished assignment"),
    Step::Example,
    Step::Field("sample-core:priority", "different unfinished example"),
    Step::SaveResume,
    Step::Capture("stp-resumed-example-draft"),
    Step::Assignment,
    Step::Reset,
    Step::Cancel,
    Step::Reset,
    Step::Confirm,
    Step::ResetProof,
    Step::Capture("stp-selected-reset"),
    Step::Finish,
];
pub(in crate::ui) const LAST: usize = FIRST + STEPS.len() - 1;
const _: () = assert!(LAST < 34000);
impl LabApp {
    fn add_spanning_tree_fixture(&mut self) -> usize {
        assert!(
            self.smoke && self._smoke_profile.is_some(),
            "private smoke fixture only"
        );
        if let Some(i) = self.scenarios.iter().position(|s| s.id == ID) {
            return i;
        }
        let s = pack::load(include_str!(
            "../../../tests/fixtures/spanning-tree-workspace-preview.json"
        ))
        .unwrap()
        .remove(0);
        let i = self.scenarios.len();
        self.scenarios.push(s);
        i
    }
    fn stp_geometry(&self, ctx: &egui::Context) -> bool {
        let geometry = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let ppp = ctx.pixels_per_point();
        assert!(ppp.is_finite() && ppp > 0.);
        let key = egui::Id::new(("stp-geometry", self.smoke_stage));
        let stable = ctx.data_mut(|data| {
            let (old, n, start) = data
                .get_temp::<(Option<egui::Rect>, usize, Instant)>(key)
                .unwrap_or((None, 0, Instant::now()));
            assert!(
                start.elapsed().as_secs() < 15,
                "STP geometry did not settle"
            );
            let n = if old == Some(geometry) && (zoom - self.scale).abs() < 0.01 {
                n + 1
            } else {
                0
            };
            data.insert_temp(key, (Some(geometry), n, start));
            n >= 2
        });
        if !stable {
            ctx.request_repaint();
        }
        stable
    }
    fn stp_clicked(&self, ctx: &egui::Context) -> bool {
        ctx.data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0)
            >= 2
    }
    fn stp_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        if self.stp_clicked(ctx) {
            return self.click_visible_smoke_control(ctx, None);
        }
        let rect = rect.expect("STP control before click");
        self.stp_reveal(ctx, rect) && self.click_smoke_control(ctx, Some(rect))
    }
    fn stp_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        let visible = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke_control_visible(rect, visible, ctx.pixels_per_point()) {
            return true;
        }
        let delta = if rect.top() < visible.top() {
            visible.top() - rect.top() + 8.
        } else if rect.bottom() > visible.bottom() {
            visible.bottom() - rect.bottom() - 8.
        } else {
            0.
        };
        let key = egui::Id::new(("stp-control-scroll", self.smoke_stage));
        let attempts = ctx.data_mut(|data| {
            let n = data.get_temp::<usize>(key).unwrap_or(0) + 1;
            data.insert_temp(key, n);
            n
        });
        assert!(
            attempts <= 20,
            "STP control not fully visible at stage {}: {rect:?}; viewport {visible:?}",
            self.smoke_stage
        );
        // Aim into this control's column. The outer clip's edge can lie in a
        // card margin, where a wheel event does not reach the content scroller.
        let x = rect
            .center()
            .x
            .clamp(visible.left() + 2., visible.right() - 2.);
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            let observation = ctx.input(|input| {
                (
                    input.raw_scroll_delta,
                    input.smooth_scroll_delta,
                    input.modifiers,
                    input.pointer.hover_pos(),
                )
            });
            eprintln!(
                "STP reveal stage={} attempt={attempts} target={rect:?} visible={visible:?} offset={:?} size={:?} observation={observation:?} focus={:?} auto_focus={}",
                self.smoke_stage,
                self.metrics.main_scroll_offset,
                self.metrics.main_content_size,
                ctx.memory(|m| m.focused()),
                self.focus_input
            );
        }
        self.pending_events.extend([
            egui::Event::PointerMoved(egui::pos2(x, visible.center().y)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta.clamp(-400., 400.)),
                modifiers: Default::default(),
            },
        ]);
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn stp_type_field(&mut self, ctx: &egui::Context, key: &str, text: &str, i: usize) -> bool {
        assert!(text.len() <= 64);
        let phase_key = egui::Id::new(("stp-type", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(phase_key)).unwrap_or(0);
        match phase {
            0 => {
                let (_, rect, id) = self
                    .metrics
                    .stp_fields
                    .iter()
                    .find(|x| x.0 == key)
                    .expect("native field exists");
                let (rect, id) = (*rect, *id);
                if !self.stp_click(ctx, Some(rect)) {
                    return false;
                }
                assert!(
                    ctx.memory(|m| m.has_focus(id)),
                    "actual clicked field owns focus"
                );
                ctx.data_mut(|d| d.insert_temp(phase_key, 1u8));
                false
            }
            1 => {
                self.pending_events = [true, false]
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
                    .collect();
                self.pending_events.push(egui::Event::Text(text.into()));
                ctx.data_mut(|d| d.insert_temp(phase_key, 2u8));
                false
            }
            2 => {
                let a = &self.progress.attempts[ID];
                let w = a.workspace.as_ref().unwrap();
                let d = if self.spanning_tree_view(i) == teaching::View::Example {
                    &w.spanning_tree_learning.as_ref().unwrap().draft
                } else {
                    w.spanning_tree_draft.as_ref().unwrap()
                };
                let bridge = key
                    .strip_suffix(":priority")
                    .expect("closed priority field");
                assert_eq!(d.bridges[bridge].priority, text);
                true
            }
            _ => unreachable!("bounded field phases"),
        }
    }
    pub(super) fn spanning_tree_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!(self.smoke && self._smoke_profile.is_some());
        assert!((FIRST..=LAST).contains(&stage));
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        let i = self.add_spanning_tree_fixture();
        let scenario = self.scenarios[i].clone();
        let mut shot = None;
        let step = STEPS[stage - FIRST];
        match step {
            Step::Open => {
                assert!(!self.progress.attempts.contains_key(ID));
                self.launch(i);
                assert_eq!(self.spanning_tree_view(i), teaching::View::Introduction);
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            Step::Settle => {
                if !self.stp_geometry(ctx) {
                    return (false, None);
                }
            }
            Step::Capture(name) => {
                let target = if name.ends_with("-typed-input") {
                    let (_, r, id) = self
                        .metrics
                        .stp_fields
                        .iter()
                        .find(|x| x.0 == "core:priority")
                        .expect("actual priority field");
                    assert!(ctx.memory(|m| m.has_focus(*id)));
                    Some(*r)
                } else if self.spanning_tree_view(i) == teaching::View::Introduction {
                    self.metrics.example_action
                } else {
                    self.metrics
                        .feedback
                        .filter(|r| r.height() > 2.)
                        .or(self.metrics.check)
                };
                if !self.stp_reveal(ctx, target.expect("visible STP view target")) {
                    return (false, None);
                }
                shot = Some(name.into());
            }
            Step::Example => {
                let r = self.metrics.lesson_tabs.get(1).copied();
                if !self.stp_click(ctx, r) {
                    return (false, None);
                }
            }
            Step::Assignment => {
                let r = self.metrics.lesson_tabs.get(2).copied();
                if !self.stp_click(ctx, r) {
                    return (false, None);
                }
            }
            Step::Insert => {
                if !self.stp_click(ctx, self.metrics.insert) {
                    return (false, None);
                }
            }
            Step::Apply => {
                if !self.stp_click(ctx, self.metrics.stp_apply) {
                    return (false, None);
                }
            }
            Step::Check => {
                if !self.stp_click(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            Step::Field(key, text) => {
                if !self.stp_type_field(ctx, key, text, i) {
                    return (false, None);
                }
            }
            Step::ExamplePass | Step::ExampleFail => {
                let l = self.progress.attempts[ID]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .spanning_tree_learning
                    .as_ref()
                    .unwrap();
                let rows = l.checks(scenario.spanning_tree_teaching.as_ref().unwrap());
                assert_eq!(rows.iter().all(|r| r.0), matches!(step, Step::ExamplePass));
                assert_eq!(
                    self.progress.attempts[ID].state, scenario.initial,
                    "practice cannot change assignment"
                );
            }
            Step::BridgeMenu => {
                if !self.stp_click(ctx, self.metrics.stp_bridge) {
                    return (false, None);
                }
            }
            Step::Bridge(name) => {
                let r = self
                    .metrics
                    .stp_bridge_items
                    .iter()
                    .find(|x| x.0 == name)
                    .map(|x| x.1);
                if !self.stp_click(ctx, r) {
                    return (false, None);
                }
                assert_eq!(
                    self.progress.attempts[ID]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .spanning_tree_draft
                        .as_ref()
                        .unwrap()
                        .selected_bridge,
                    name
                );
            }
            Step::Current(pass) => {
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[ID], &scenario),
                    pass
                );
                assert_eq!(self.metrics.completion, pass);
                if pass {
                    let engine::State::SpanningTree { simulation } =
                        &self.progress.attempts[ID].state
                    else {
                        unreachable!()
                    };
                    let d = self.progress.attempts[ID]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .spanning_tree_draft
                        .as_ref()
                        .unwrap();
                    assert!(d.matches(simulation));
                }
            }
            Step::Scale(light) => {
                self.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
                self.scale = if light { 1.5 } else { 2. };
                theme::apply(ctx, &self.theme_id);
            }
            Step::Challenge => {
                if !self.stp_click(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            Step::Learn => {
                if !self.stp_click(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            Step::SaveResume => {
                let before = serde_json::to_value(&self.progress.attempts).unwrap();
                self.progress.favorites.insert(ID.into());
                self.dirty = true;
                self.save();
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), before);
                assert!(loaded.favorites.contains(ID));
                self.progress = loaded;
                self.launch(i);
                let mut expected = before[ID].clone();
                expected["visited"] = serde_json::json!(self.progress.clock);
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[ID]).unwrap(),
                    expected
                );
                assert_eq!(self.spanning_tree_view(i), teaching::View::Example);
            }
            Step::Reset => {
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("stp-before-reset"), before));
                if !self.stp_click(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            Step::Cancel => {
                if !self.stp_click(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
                let before = ctx
                    .data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("stp-before-reset")))
                    .unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
            }
            Step::Confirm => {
                if !self.stp_click(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            Step::ResetProof => {
                assert_eq!(self.progress.attempts[ID].state, scenario.initial);
                assert!(self.progress.favorites.contains(ID));
                assert!(!progress::current_solved(
                    &self.progress.attempts[ID],
                    &scenario
                ));
            }
            Step::Finish => {
                println!(
                    "PASS private native ideal STP: actual fields, separate fill/apply/check/experiment, assignment pending/wrong/reference/alternate,150Light/200Dark typing, warm resume and selected reset. No protocol, packets, cold-process, novice or other-OS acceptance."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, shot)
    }
}
