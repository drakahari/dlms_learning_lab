//! Fixed native editor/terminal review; never host swap or learner-host execution.
use super::*;
use dlms_learning_lab::storage_workspace as work;
use std::time::Duration;
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    Click(String),
    Field(String, String),
    Solved(bool),
    Example(bool),
    Unrun,
    Save,
    Fresh,
    Challenge(bool),
    Theme(bool),
    Shot(String, String),
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    settle: u8,
    not_before: Option<Instant>,
    done: bool,
    stable_target: Option<(egui::Rect, egui::Rect)>,
    stable_frames: u8,
}
pub(in crate::ui) fn control(a: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "introduction" => a.metrics.lesson_tabs.first().copied(),
        "example" => a.metrics.lesson_tabs.get(1).copied(),
        "assignment" => a.metrics.lesson_tabs.get(2).copied(),
        "editor" => a.metrics.terminal,
        "command" => a.metrics.storage_command,
        "save" => a.metrics.editor_save,
        "run" => a.metrics.storage_run,
        "check" => a.metrics.check,
        "fill" => a.metrics.insert,
        "fresh" => a.metrics.storage_test,
        "reset-example" => a.metrics.example_reset,
        "reset-header" => a.metrics.reset,
        "confirm-reset" => a.metrics.confirm_reset,
        "challenge-header" => a.metrics.mode_challenge,
        "learn-header" => a.metrics.mode_learn,
        _ => panic!("Unknown fixed storage control {key}"),
    }
}
fn action(ops: &mut Vec<Op>, text: &str) {
    if let Some(raw) = text.strip_prefix("storage ") {
        let work::Action::Save { text } = serde_json::from_str(raw).unwrap() else {
            panic!()
        };
        ops.push(Op::Field("editor".into(), text));
        ops.push(Op::Click("save".into()));
    } else {
        ops.push(Op::Field("command".into(), text.into()));
        ops.push(Op::Click("run".into()));
    }
}
fn route(labs: &[Scenario]) -> Vec<Op> {
    let mut ops = vec![];
    for id in [
        "lpic1-swap-prepared-activation",
        "lpic1-swap-replacement-handover",
    ] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let lesson = s.storage_teaching.as_ref().unwrap();
        ops.extend([
            Op::Open(id.into()),
            Op::Shot(format!("swap-{id}-introduction"), "reset-header".into()),
            Op::Click("example".into()),
            Op::Shot(format!("swap-{id}-practice-before"), "editor".into()),
            Op::Click("fill".into()),
            Op::Unrun,
        ]);
        for c in &lesson.example_solution {
            action(&mut ops, c);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Shot(format!("swap-{id}-practice-pass"), "check".into()),
        ]);
        for c in &lesson.try_actions {
            action(&mut ops, c);
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Example(false),
            Op::Shot(format!("swap-{id}-practice-adverse"), "check".into()),
        ]);
        action(&mut ops, &lesson.example_solution[0]);
        ops.extend([
            Op::Click("check".into()),
            Op::Example(true),
            Op::Click("assignment".into()),
            Op::Click("check".into()),
            Op::Solved(false),
            Op::Shot(format!("swap-{id}-initial-feedback"), "check".into()),
        ]);
        for (n, c) in s.solution.iter().enumerate() {
            action(&mut ops, c);
            if n == 0 {
                ops.extend([
                    Op::Click("check".into()),
                    Op::Solved(false),
                    Op::Shot(format!("swap-{id}-saved-only-feedback"), "check".into()),
                ]);
            }
        }
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Click("fresh".into()),
            Op::Save,
            Op::Shot(format!("swap-{id}-independent-pass"), "check".into()),
        ]);
        action(&mut ops, "swapoff /dev/vdc1");
        ops.push(Op::Solved(false));
        action(&mut ops, "swapon -a");
        ops.extend([
            Op::Click("check".into()),
            Op::Solved(true),
            Op::Theme(true),
            Op::Field("command".into(), "swapon --show --bytes".into()),
            Op::Shot(format!("swap-{id}-light150-command"), "command".into()),
            Op::Theme(false),
            Op::Shot(format!("swap-{id}-dark200-command"), "command".into()),
            Op::Save,
            Op::Click("reset-header".into()),
            Op::Click("confirm-reset".into()),
            Op::Fresh,
            Op::Click("challenge-header".into()),
            Op::Challenge(true),
            Op::Shot(
                format!("swap-{id}-challenge-default"),
                "reset-header".into(),
            ),
            Op::Click("learn-header".into()),
            Op::Challenge(false),
        ]);
    }
    ops.extend([
        Op::Open("lpic2-mount-identity".into()),
        Op::Click("assignment".into()),
        Op::Shot("swap-retained-mount-only".into(), "editor".into()),
    ]);
    ops
}
fn draft(a: &LabApp) -> &work::Draft {
    let i = a.selected.unwrap();
    let w = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap();
    if a.storage_view(i) == teaching::View::Example {
        &w.storage_learning.as_ref().unwrap().draft
    } else {
        w.storage_draft.as_ref().unwrap()
    }
}
fn defer_scroll(r: &mut Route) {
    r.settle = 10;
    r.not_before = Some(Instant::now() + Duration::from_millis(200));
}
impl LabApp {
    // A bounded visibility attempt queues a wheel event; do not consume the
    // next attempt until its smooth-scroll animation has had time to settle.
    // Keep the shared visibility predicate and twenty-attempt refusal intact.
    fn swap_settled_click(&mut self, ctx: &egui::Context, key: &str, r: &mut Route) -> bool {
        let rect = control(self, key).expect("Actual swap control before click");
        let clip = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let visible = smoke_control_visible(rect, clip, ctx.pixels_per_point());
        let complete = self.click_visible_smoke_control(ctx, Some(rect));
        if !visible && !complete {
            defer_scroll(r);
        }
        complete
    }
    pub(in crate::ui) fn swap_policy_tick(&mut self, ctx: &egui::Context) {
        let key = egui::Id::new("swap-policy-fixed-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(&self.scenarios),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "SWAP_POLICY_NATIVE_COMPLETE: actual editor/terminal/practice/adverse/retry/partial/current-invalidation/save-read/reset/Challenge and scaled controls; no host swap/memory operation"
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        // An optimized binary may render ten frames before a smooth-scroll
        // or compositor resize has elapsed. Wait real time without blocking
        // the UI or consuming another bounded visibility attempt.
        if r.not_before.is_some_and(|at| Instant::now() < at) {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(Duration::from_millis(16));
            return;
        }
        r.not_before = None;
        if r.settle > 0 {
            r.settle -= 1;
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint();
            return;
        }
        self.smoke_stage = 90000 + r.at;
        let op = r.ops[r.at].clone();
        if r.frames == 0 {
            println!("SWAP_POLICY_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Swap native action did not complete: {} {op:?}",
            r.at
        );
        let complete = match op {
            Op::Open(id) => {
                self.scale = 1.;
                ctx.set_zoom_factor(1.);
                self.theme_id = theme::Preferences::default().theme;
                theme::apply(ctx, &self.theme_id);
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                true
            }
            Op::Click(name) => self.swap_settled_click(ctx, &name, &mut r),
            Op::Field(name, value) => {
                if r.phase == 0 {
                    if self.swap_settled_click(ctx, &name, &mut r) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert!(ctx.memory(|m| m.focused()).is_some(), "Native {name} focus");
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
                    self.pending_events.push(egui::Event::Paste(value));
                    r.phase = 2;
                    false
                } else {
                    assert_eq!(
                        if name == "editor" {
                            &draft(self).text
                        } else {
                            &draft(self).command
                        },
                        &value,
                        "Native paste must edit the actual {name} draft"
                    );
                    true
                }
            }
            Op::Solved(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    progress::current_solved(&self.progress.attempts[&s.id], s),
                    want,
                    "{} current state",
                    s.id
                );
                true
            }
            Op::Example(want) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let a = &self.progress.attempts[&s.id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .storage_learning
                    .as_ref()
                    .unwrap();
                assert!(l.checked);
                assert_eq!(
                    l.checks(s.storage_teaching.as_ref().unwrap())
                        .iter()
                        .all(|r| r.0),
                    want
                );
                assert_eq!(
                    serde_json::to_value(&a.state).unwrap(),
                    serde_json::to_value(&s.initial).unwrap()
                );
                assert!(!progress::current_solved(a, s));
                true
            }
            Op::Unrun => {
                let s = &self.scenarios[self.selected.unwrap()];
                let l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .storage_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.state, s.storage_teaching.as_ref().unwrap().fixture);
                assert!(!l.checked);
                true
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none(), "{:?}", self.storage_error);
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[&s.id].state).unwrap(),
                    serde_json::to_value(&s.initial).unwrap()
                );
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
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
                    assert_eq!(self.storage_view(i), teaching::View::Assignment);
                }
                true
            }
            Op::Theme(light) => {
                // A real toolbar selection moves focus out of the text field.
                // Programmatic QA scaling must do the same; otherwise egui's
                // focused-cursor scroll fights the capture's outer wheel.
                if let Some(id) = ctx.memory(|m| m.focused()) {
                    ctx.memory_mut(|m| m.surrender_focus(id));
                }
                self.focus_input = false;
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
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1268., 632.)));
                self.lesson_scroll_reset = true;
                r.not_before = Some(Instant::now() + Duration::from_millis(300));
                true
            }
            Op::Shot(name, target) => {
                let rect = control(self, &target).expect("Actual swap capture target");
                let clip = self
                    .metrics
                    .main_viewport
                    .unwrap_or(ctx.content_rect())
                    .intersect(ctx.content_rect());
                let geometry = (rect, clip);
                let visible = smoke_control_visible(rect, clip, ctx.pixels_per_point());
                if r.phase == 0 {
                    if visible {
                        if r.stable_target == Some(geometry) {
                            r.stable_frames += 1;
                        } else {
                            r.stable_target = Some(geometry);
                            r.stable_frames = 0;
                        }
                        if r.stable_frames >= 4 {
                            r.phase = 1;
                        }
                    } else {
                        r.stable_target = None;
                        r.stable_frames = 0;
                        self.reveal_smoke_control(ctx, rect);
                        // Allow the queued wheel and smooth-scroll animation
                        // to settle before consuming another bounded attempt.
                        defer_scroll(&mut r);
                    }
                    false
                } else if r.phase == 1 {
                    if !visible || r.stable_target != Some(geometry) {
                        r.phase = 0;
                        r.stable_frames = 0;
                        r.stable_target = None;
                        ctx.data_mut(|d| d.insert_temp(key, r));
                        ctx.request_repaint();
                        return;
                    }
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
            r.phase = 0;
            r.frames = 0;
            r.settle = 5;
            r.stable_target = None;
            r.stable_frames = 0;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swap_native_route_has_actual_editor_and_command_actions_and_both_scales() {
        let labs = dlms_learning_lab::pack::ccna_pilot().unwrap();
        let ops = route(&labs);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Open(_))).count(), 3);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Theme(_))).count(), 4);
        assert_eq!(ops.iter().filter(|o| matches!(o, Op::Save)).count(), 4);
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(k,v)if k=="editor"&&v.contains("none swap")))
        );
        assert!(
            ops.iter()
                .any(|o| matches!(o,Op::Field(k,v)if k=="command"&&v=="swapoff /dev/vdc1"))
        );
    }
    #[test]
    fn swap_native_actual_capture_phase_is_context_safe() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-swap-prepared-activation")
            .unwrap();
        a.launch(i);
        a.set_storage_view(i, teaching::View::Assignment);
        for _ in 0..4 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1400., 950.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
        }
        let key = egui::Id::new("swap-policy-fixed-native-route");
        let target = control(&a, "reset-header").unwrap();
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        assert!(smoke_control_visible(target, clip, ctx.pixels_per_point()));
        let geometry = (target, clip);
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("unit-swap-capture".into(), "reset-header".into())],
                    phase: 1,
                    stable_target: Some(geometry),
                    stable_frames: 4,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| a.swap_policy_tick(ctx));
        assert!(a.capture_pending);
        assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().phase, 2);
        assert!(out.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        }));
    }
    #[test]
    fn swap_native_style_change_releases_edit_focus_like_a_toolbar_selection() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-swap-prepared-activation")
            .unwrap();
        a.launch(i);
        a.set_storage_view(i, teaching::View::Assignment);
        for _ in 0..4 {
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1400., 950.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
        }
        assert!(
            ctx.memory(|m| m.focused()).is_some(),
            "Actual initial editor focus"
        );
        let key = egui::Id::new("swap-policy-fixed-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Theme(false)],
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(egui::RawInput::default(), |ctx| a.swap_policy_tick(ctx));
        assert!(ctx.memory(|m| m.focused()).is_none());
        assert!(!a.focus_input);
        assert_eq!(a.scale, 2.);
    }
    #[test]
    fn swap_native_click_and_edit_wait_for_each_queued_scroll_without_extra_attempts() {
        for op in [
            Op::Click("run".into()),
            Op::Field("command".into(), "swapon --show --bytes".into()),
        ] {
            let ctx = egui::Context::default();
            let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
            a.smoke = false;
            let i = a
                .scenarios
                .iter()
                .position(|s| s.id == "lpic1-swap-prepared-activation")
                .unwrap();
            a.launch(i);
            a.set_storage_view(i, teaching::View::Assignment);
            let key = egui::Id::new("swap-policy-fixed-native-route");
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(845., 421.),
                )),
                ..Default::default()
            };
            let _ = ctx.run(raw, |ctx| {
                a.render(ctx);
                let below = egui::Rect::from_min_size(egui::pos2(35., 404.), egui::vec2(450., 35.));
                a.metrics.storage_run = Some(below);
                a.metrics.storage_command = Some(below);
                ctx.data_mut(|d| {
                    d.insert_temp(
                        key,
                        Route {
                            ops: vec![op.clone()],
                            ..Default::default()
                        },
                    )
                });
                a.swap_policy_tick(ctx);
            });
            let first = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!(
                first.settle, 10,
                "Both click and edit must wait after a failed visibility attempt"
            );
            let scroll = egui::Id::new(("bounded-native-scroll", 90000usize));
            assert_eq!(ctx.data(|d| d.get_temp::<usize>(scroll)), Some(1));
            assert!(
                first.not_before.is_some(),
                "Frame count alone cannot settle an optimized native run"
            );
            ctx.data_mut(|d| {
                let mut route = d.get_temp::<Route>(key).unwrap();
                route.not_before = Some(Instant::now() + Duration::from_secs(60));
                d.insert_temp(key, route);
            });
            for _ in 0..30 {
                let _ = ctx.run(egui::RawInput::default(), |ctx| a.swap_policy_tick(ctx));
                assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().settle, 10);
            }
            // Simulate elapsed time without sleeping or changing shared bounds.
            ctx.data_mut(|d| {
                let mut route = d.get_temp::<Route>(key).unwrap();
                route.not_before = Some(Instant::now() - Duration::from_millis(1));
                d.insert_temp(key, route);
            });
            for _ in 0..10 {
                let _ = ctx.run(egui::RawInput::default(), |ctx| a.swap_policy_tick(ctx));
                assert_eq!(
                    ctx.data(|d| d.get_temp::<usize>(scroll)),
                    Some(1),
                    "Settling must not spend another shared scroll attempt"
                );
            }
            assert_eq!(ctx.data(|d| d.get_temp::<Route>(key)).unwrap().settle, 0);
        }
    }
}
