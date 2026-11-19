//! Actual native events for the recorded-host curriculum, with separate saved scopes.
use super::*;
use dlms_learning_lab::resource_observation_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-resource-cpu-intervals",
    "lpic2-resource-swap-activity",
    "lpic2-resource-restart-handover",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(String),
    Choose(&'static str, usize),
    Field(String, String),
    Read(work::Probe),
    Attached(String, usize),
    Assert(&'static str),
    Capture(String, &'static str),
    KeyboardRun,
    Save,
    RememberOthers,
}
fn fixture(n: usize) -> &'static work::Definition {
    static F: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let f = &F.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../tests/fixtures/resource-observation-lessons-v1.json"
        ))
        .unwrap()
        .scenarios
    })[n];
    let engine::Goal::ResourceObservations { definition } = &f.goal else {
        panic!()
    };
    definition
}
fn click(key: &str) -> Step {
    Step::Click(key.into())
}
fn actions(p: &mut Vec<Step>, ops: &[work::Action]) {
    for a in ops {
        match a {
            work::Action::Run { probe } => p.extend([
                Step::Choose("observation", probe.start),
                Step::Field("command".into(), probe.command.clone()),
                click("run"),
                Step::Read(probe.clone()),
            ]),
            work::Action::Attach { rule, row } => p.extend([
                Step::Choose("result-row", *row),
                Step::Click(format!("attach-{rule}")),
                Step::Attached(rule.clone(), *row),
            ]),
            work::Action::Value { rule, value } => p.push(Step::Field(rule.clone(), value.clone())),
            work::Action::Check => p.push(click("check")),
        }
    }
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![];
        if std::env::var("DLMS_SMOKE_STAGE").as_deref() == Ok("resource-observation-popup") {
            p.extend([
                Step::Style(1., false),
                Step::Open(IDS[2]),
                click("assignment"),
            ]);
            actions(&mut p, &fixture(2).reference[..5]);
            p.push(Step::Choose("observation", 6));
            p.push(Step::Capture("obs-popup-selected".into(), "observation"));
            return p;
        }

        for (n, id) in IDS.into_iter().enumerate() {
            let d = fixture(n);
            let rule = &d.initial.rules[0].id;
            let example_rule = &d.example.rules[0].id;
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("obs-{n}-introduction"), "top"),
                Step::Capture(format!("obs-{n}-worked"), "intro-end"),
                click("example"),
                Step::Capture(format!("obs-{n}-practice"), "command"),
            ]);
            if n == 2 {
                p.extend([
                    Step::Field("command".into(), "vmstat 1 3".into()),
                    click("run"),
                    Step::Assert("command-error"),
                    Step::Capture("obs-2-incompatible-practice".into(), "output"),
                ]);
            }
            actions(&mut p, &d.example_reference);
            p.extend([
                Step::Assert("practice-pass"),
                Step::Capture(format!("obs-{n}-practice-report"), "result"),
                Step::Field(example_rule.clone(), "9999999".into()),
                click("check"),
                Step::Assert("practice-partial"),
                Step::Capture(format!("obs-{n}-practice-feedback"), "first-feedback"),
            ]);
            let value = d
                .example_reference
                .iter()
                .find_map(|a| {
                    if let work::Action::Value { rule, value } = a {
                        (rule == example_rule).then(|| value.clone())
                    } else {
                        None
                    }
                })
                .unwrap();
            p.extend([
                Step::Field(example_rule.clone(), value),
                click("check"),
                Step::Assert("practice-pass"),
                click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("obs-{n}-assignment"), "top"),
                Step::Field("command".into(), "cat /etc/passwd".into()),
                click("run"),
                Step::Assert("command-error"),
                Step::Capture(format!("obs-{n}-unsupported"), "output"),
            ]);
            actions(&mut p, &d.reference);
            p.extend([
                Step::Assert("assignment-pass"),
                Step::Capture(format!("obs-{n}-report-pass"), "result"),
                Step::KeyboardRun,
                Step::Assert("assignment-pass"),
                Step::Field(rule.clone(), "9999999".into()),
                Step::Assert("assignment-unchecked"),
                click("check"),
                Step::Assert("assignment-partial"),
                Step::Capture(format!("obs-{n}-report-feedback"), "first-feedback"),
            ]);
            let value = d
                .reference
                .iter()
                .find_map(|a| {
                    if let work::Action::Value { rule: r, value } = a {
                        (r == rule).then(|| value.clone())
                    } else {
                        None
                    }
                })
                .unwrap();
            p.extend([
                Step::Field(rule.clone(), value),
                click("check"),
                Step::Assert("assignment-pass"),
                Step::Save,
            ]);
            if n == 2 {
                p.extend([
                    Step::RememberOthers,
                    click("reset"),
                    click("confirm-reset"),
                    Step::Assert("reset"),
                    click("challenge"),
                    Step::Assert("challenge"),
                    Step::Capture("obs-2-challenge".into(), "command"),
                    click("learn"),
                    click("assignment"),
                ]);
                actions(&mut p, &d.reference);
                p.extend([Step::Assert("assignment-pass"), Step::Save]);
            }
            p.extend([
                Step::Style(1.5, false),
                Step::Capture(format!("obs-{n}-dark150-values"), "first-field"),
                Step::Style(2., true),
                Step::Capture(format!("obs-{n}-light200-values"), "first-field"),
                Step::Capture(format!("obs-{n}-light200-report"), "result"),
                Step::Choose("observation", 0),
                Step::Capture(format!("obs-{n}-light200-command"), "command"),
                Step::Save,
            ]);
        }
        p
    })
}
#[derive(Clone, Default)]
struct Flow {
    at: usize,
    phase: u8,
    polls: usize,
    next: f64,
    started: Option<Instant>,
    viewport: Option<egui::Rect>,
    capture_pixels: Option<egui::Vec2>,
    capture_reported: bool,
    reveals: usize,
    history_len: usize,
    others: Option<serde_json::Value>,
}
fn session(app: &LabApp) -> &work::Session {
    let engine::State::ResourceObservations { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::ResourceObservations { definition } =
        &app.scenarios[app.selected.unwrap()].goal
    else {
        panic!()
    };
    definition
}
fn practice(app: &LabApp) -> bool {
    !app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].challenge
        && session(app).view == teaching::View::Example
}
fn scope(app: &LabApp) -> &work::Draft {
    if practice(app) {
        &session(app).practice
    } else {
        &session(app).assignment
    }
}
fn others(app: &LabApp) -> serde_json::Value {
    let id = &app.scenarios[app.selected.unwrap()].id;
    serde_json::to_value(
        app.progress
            .attempts
            .iter()
            .filter(|(key, _)| *key != id)
            .collect::<std::collections::BTreeMap<_, _>>(),
    )
    .unwrap()
}
fn observe(app: &LabApp, what: &str, flow: &Flow) {
    let i = app.selected.unwrap();
    let a = &app.progress.attempts[&app.scenarios[i].id];
    let d = definition(app);
    let s = session(app);
    match what {
        "practice-pass" | "practice-partial" => {
            let rows = d.checks(s, true);
            assert_eq!(
                rows.iter().filter(|r| !r.0).count(),
                if what == "practice-pass" { 0 } else { 1 }
            );
            assert!(!progress::current_solved(a, &app.scenarios[i]));
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
        }
        "assignment-pass" => {
            assert!(progress::current_solved(a, &app.scenarios[i]));
            assert!(d.checks(s, false).iter().all(|r| r.0));
        }
        "assignment-partial" => {
            assert!(!progress::current_solved(a, &app.scenarios[i]));
            assert_eq!(d.checks(s, false).iter().filter(|r| !r.0).count(), 1);
        }
        "assignment-unchecked" => {
            assert!(!s.assignment.checked);
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "fresh" => {
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "command-error" => {
            let t = if practice(app) {
                &d.example
            } else {
                &d.initial
            };
            assert!(t.probe(scope(app).history.last().unwrap()).is_err());
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d));
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
        }
        _ => panic!("Unknown observation"),
    }
}
fn target(app: &LabApp, ctx: &egui::Context, key: &str) -> Option<egui::Rect> {
    let m = &app.metrics.resource_observation;
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "intro-end" => m.controls.iter().find(|r| r.0 == "start").map(|r| r.1),
        "output" => m.output.map(|r| {
            egui::Rect::from_min_size(
                r.min,
                egui::vec2(r.width(), 2. * ctx.style().spacing.interact_size.y),
            )
        }),
        "result" => m.result.map(|r| r.2),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        "first-field" => {
            let key = &definition(app).initial.rules[0].id;
            m.fields.iter().find(|r| r.0 == *key).map(|r| r.1)
        }
        "first-feedback" => m
            .controls
            .iter()
            .find(|r| r.0.starts_with("feedback-"))
            .map(|r| r.1),
        _ => m
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| m.fields.iter().find(|r| r.0 == key).map(|r| r.1)),
    }
}
impl LabApp {
    fn obs_reveal(&mut self, ctx: &egui::Context, r: egui::Rect) -> bool {
        let before = self.pending_events.len();
        let done = self.reveal_smoke_control(ctx, r);
        if !done {
            let v = self.metrics.main_viewport.unwrap();
            let region = self.metrics.main_scroll_region.unwrap();
            let gutter = egui::Rect::from_min_max(
                egui::pos2(v.right(), v.top()),
                egui::pos2(region.right(), v.bottom()),
            )
            .intersect(ctx.content_rect());
            assert!(gutter.width() * ctx.pixels_per_point() > 1.);
            for e in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(p) = e {
                    *p = gutter.center();
                }
            }
        }
        done
    }
    fn obs_click(&mut self, ctx: &egui::Context, r: Option<egui::Rect>) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = r.unwrap();
        self.obs_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn obs_choice(&mut self, ctx: &egui::Context, key: &str, flow: &mut Flow) -> bool {
        let click_id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(click_id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (r, clip) = self
            .metrics
            .resource_observation
            .choices
            .iter()
            .find(|r| r.0 == key)
            .map(|r| (r.1, r.2))
            .expect("actual popup choice");
        let visible = clip.intersect(ctx.content_rect());
        if !smoke_control_visible(r, visible, ctx.pixels_per_point()) {
            flow.reveals += 1;
            if flow.reveals == 1 {
                println!(
                    "OBSERVATION_POPUP_REVEAL stage={} key={key} row={r:?} clip={clip:?} screen={:?}",
                    self.smoke_stage,
                    ctx.content_rect()
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                    format!("obs-popup-diagnostic-{}", self.smoke_stage),
                )));
            }

            assert!(flow.reveals < 20);
            self.pending_events.extend([
                egui::Event::PointerMoved(visible.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (visible.center().y - r.center().y).clamp(-400., 400.)),
                    modifiers: Default::default(),
                },
            ]);
            return false;
        }
        self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn resource_observation_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("resource-observation-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "RESOURCE_OBSERVATION_NATIVE_COMPLETE: {} actions",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 139000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Observation step {} {:?}: {due:?}",
            flow.at,
            plan()[flow.at]
        );
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(id, flow));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match plan()[flow.at].clone() {
            Step::Style(scale, narrow) => {
                if flow.phase == 0 {
                    self.scale = scale;
                    ctx.set_zoom_factor(scale);
                    self.theme_id = if narrow {
                        theme::inventory()
                            .iter()
                            .find(|theme| theme.light)
                            .unwrap()
                            .id
                            .clone()
                    } else {
                        theme::Preferences::default().theme
                    };
                    theme::apply(ctx, &self.theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));

                    self.lesson_scroll_reset = true;
                    flow.phase = 1;
                    false
                } else {
                    let viewport = ctx.content_rect();
                    if flow.capture_pixels.is_some()
                        && !flow.capture_reported
                        && started.elapsed().as_secs_f32() > 1.
                    {
                        println!(
                            "NATIVE_CAPTURE_OBSERVED actual_pixels={:?} requested_pixels={:?} fullscreen={:?} scale={} ppp={}",
                            viewport.size() * ctx.pixels_per_point(),
                            flow.capture_pixels,
                            ctx.input(|input| input.viewport().fullscreen),
                            ctx.zoom_factor(),
                            ctx.pixels_per_point()
                        );
                        flow.capture_reported = true;
                    }
                    let scale_ready = (ctx.zoom_factor() - scale).abs() < 0.01;
                    let mode_ready =
                        ctx.input(|input| input.viewport().fullscreen) == Some(!narrow);
                    if narrow && flow.capture_pixels.is_none() {
                        if scale_ready && mode_ready && flow.viewport == Some(viewport) {
                            flow.capture_pixels = Some(capture_size::request(ctx));
                        }
                        flow.viewport = Some(viewport);
                        false
                    } else {
                        let proper_size = !narrow
                            || capture_size::fits(
                                viewport.size() * ctx.pixels_per_point(),
                                flow.capture_pixels.expect("requested narrow capture size"),
                                ctx.input(|input| input.viewport().native_pixels_per_point)
                                    .expect("native display scale"),
                            );
                        let settled = flow.viewport == Some(viewport)
                            && proper_size
                            && scale_ready
                            && mode_ready;
                        if settled && narrow {
                            println!(
                                "NATIVE_CAPTURE_SETTLED actual_pixels={:?} requested_pixels={:?} scale={} ppp={}",
                                viewport.size() * ctx.pixels_per_point(),
                                flow.capture_pixels,
                                ctx.zoom_factor(),
                                ctx.pixels_per_point()
                            );
                        }
                        flow.viewport = Some(viewport);
                        settled
                    }
                }
            }

            Step::Open(id) => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                self.launch(self.scenarios.iter().position(|s| s.id == id).unwrap());
                let n = IDS.iter().position(|v| *v == id).unwrap();
                assert_eq!(definition(self), fixture(n));
                true
            }
            Step::Click(key) => self.obs_click(ctx, target(self, ctx, &key)),
            Step::Choose(key, value) => {
                let selected = if key == "observation" {
                    scope(self).cursor
                } else {
                    scope(self).result_row
                };
                if flow.phase == 0 && selected == value {
                    true
                } else if flow.phase == 0 {
                    if self.obs_click(ctx, target(self, ctx, key)) {
                        flow.phase = 1;
                    }
                    false
                } else {
                    let option = format!(
                        "{}-{value}",
                        if key == "observation" {
                            "observation"
                        } else {
                            "row"
                        }
                    );
                    self.obs_choice(ctx, &option, &mut flow)
                }
            }
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    let (_, r, input, _) = self
                        .metrics
                        .resource_observation
                        .fields
                        .iter()
                        .find(|r| r.0 == key)
                        .unwrap()
                        .clone();
                    if self.obs_click(ctx, Some(r)) {
                        assert_eq!(ctx.memory(|m| m.focused()), Some(input));
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.case_lesson_type(&value);
                    flow.phase = 2;
                    false
                } else {
                    let actual = if key == "command" {
                        &scope(self).input
                    } else {
                        &scope(self)
                            .claims
                            .iter()
                            .find(|c| c.rule == key)
                            .unwrap()
                            .value
                    };
                    assert_eq!(actual, &value);
                    true
                }
            }
            Step::Read(probe) => {
                assert_eq!(scope(self).history.last(), Some(&probe));
                let d = definition(self);
                let task = if practice(self) {
                    &d.example
                } else {
                    &d.initial
                };
                assert!(task.probe(&probe).is_ok());
                true
            }
            Step::Attached(rule, row) => {
                let s = scope(self);
                let e = s
                    .claims
                    .iter()
                    .find(|c| c.rule == rule)
                    .unwrap()
                    .evidence
                    .as_ref()
                    .unwrap();
                assert_eq!(e.row, row);
                assert_eq!(s.history.last(), Some(&e.probe));
                true
            }
            Step::Assert(what) => {
                observe(self, what, &flow);
                true
            }
            Step::RememberOthers => {
                flow.others = Some(others(self));
                true
            }
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                assert_eq!(
                    serde_json::to_value(progress::load(&self.path, &self.scenarios).unwrap())
                        .unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Step::KeyboardRun => {
                if flow.phase == 0 {
                    let (_, r, input, _) = self
                        .metrics
                        .resource_observation
                        .fields
                        .iter()
                        .find(|r| r.0 == "command")
                        .unwrap()
                        .clone();
                    if self.obs_click(ctx, Some(r)) {
                        assert_eq!(ctx.memory(|m| m.focused()), Some(input));
                        flow.history_len = scope(self).history.len();
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.pending_events
                        .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers: Default::default(),
                        }));
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(scope(self).history.len(), (flow.history_len + 1).min(8));
                    true
                }
            }
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    });
                    if target(self, ctx, key).is_some_and(|r| self.obs_reveal(ctx, r)) {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1
                    && !target(self, ctx, key).is_some_and(|r| {
                        smoke_control_visible(
                            r,
                            self.metrics
                                .main_viewport
                                .unwrap()
                                .intersect(ctx.content_rect()),
                            ctx.pixels_per_point(),
                        )
                    })
                {
                    flow.phase = 0;
                    false
                } else if flow.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    flow.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
        };
        if done {
            println!(
                "RESOURCE_OBSERVATION_NATIVE_STEP {} {:?}",
                flow.at,
                plan()[flow.at]
            );
            flow.at += 1;
            flow.phase = 0;
            flow.polls = 0;
            flow.started = None;
            flow.viewport = None;
            flow.capture_pixels = None;
            flow.capture_reported = false;
            flow.reveals = 0;
            flow.next = now + 0.025;
        } else {
            flow.next = flow.next.max(now + 0.025);
        }
        ctx.data_mut(|d| d.insert_temp(id, flow));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
#[cfg(test)]
#[test]
fn all_three_native_sources_use_actual_fields_results_and_selected_reset() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 3);
    assert_eq!(
        p.iter().filter(|s| matches!(s, Step::KeyboardRun)).count(),
        3
    );
    assert!(p.iter().filter(|s| matches!(s, Step::Field(_, _))).count() > 30);
    assert!(p.iter().any(|s| matches!(s, Step::Assert("reset"))));
}
#[cfg(test)]
#[test]
fn late_clipped_capture_defers_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    let id = egui::Id::new("resource-observation-native");
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "command")))
        .unwrap();
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    app.metrics.main_viewport = Some(clip);
    app.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    app.metrics.resource_observation.fields.push((
        "command".into(),
        clip.translate(egui::vec2(0., 400.)),
        egui::Id::new("field"),
        clip,
    ));
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500., 400.),
            )),
            ..Default::default()
        },
        |ctx| app.resource_observation_tick(ctx),
    );
    assert_eq!(ctx.data(|d| d.get_temp::<Flow>(id)).unwrap().phase, 0);
    assert!(!app.capture_pending);
    app.metrics.resource_observation.fields[0].1 =
        egui::Rect::from_min_size(clip.min, egui::vec2(200., 40.));
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500., 400.),
            )),
            ..Default::default()
        },
        |ctx| app.resource_observation_tick(ctx),
    );
    assert!(app.capture_pending);
    assert_eq!(ctx.data(|d| d.get_temp::<Flow>(id)).unwrap().phase, 2);
}

#[cfg(test)]
#[test]
fn completed_popup_click_does_not_require_the_closed_popup_metrics() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    app.smoke_stage = 139999;
    let id = egui::Id::new(("native-click-phases", app.smoke_stage));
    ctx.data_mut(|d| d.insert_temp(id, 2u8));
    let mut flow = Flow::default();
    assert!(app.obs_choice(&ctx, "row-1", &mut flow));
    assert_eq!(flow.reveals, 0);
    assert!(ctx.data(|d| d.get_temp::<u8>(id)).is_none());
}
