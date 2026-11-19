//! Fixed actual native events for the two supplied capacity-planning projects.
use super::*;
use dlms_learning_lab::capacity_planning_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic2-capacity-trend-plan",
    "lpic2-capacity-import-handover",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(String),
    Choose(&'static str, String),
    Field(&'static str, String),
    Assert(&'static str),
    Capture(String, &'static str),
    KeyboardTest,
    Save,
    RememberOthers,
}
fn fixture(n: usize) -> &'static work::Definition {
    static F: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let s = &F.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../tests/fixtures/capacity-planning-lessons-v1.json"
        ))
        .unwrap()
        .scenarios
    })[n];
    let engine::Goal::CapacityPlanning { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn click(key: &str) -> Step {
    Step::Click(key.into())
}
fn choose(key: &'static str, value: impl ToString) -> Step {
    Step::Choose(key, value.to_string())
}
fn actions(p: &mut Vec<Step>, ops: &[work::Action]) {
    for a in ops {
        match a {
            work::Action::Window { first, last } => {
                p.extend([choose("window-first", first), choose("window-last", last)])
            }
            work::Action::Offer { id } => p.push(choose("quote", id)),
            work::Action::OrderDay { value } => p.push(Step::Field("order-day", value.clone())),
            work::Action::Reserve { value } => p.push(Step::Field("reserve", value.clone())),
            work::Action::Test => p.push(click("test")),
        }
    }
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![];
        for (n, id) in IDS.into_iter().enumerate() {
            let d = fixture(n);
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("plan-{n}-introduction"), "top"),
                Step::Capture(format!("plan-{n}-worked"), "intro-end"),
                click("example"),
                Step::Capture(format!("plan-{n}-practice-start"), "plot"),
            ]);
            if n == 1 {
                p.extend([
                    click("test"),
                    Step::Assert("practice-fail"),
                    Step::Capture("plan-1-inherited-worked-failure".into(), "result"),
                ]);
            }
            actions(&mut p, &d.example_reference);
            p.extend([
                Step::Assert("practice-pass"),
                Step::Capture(format!("plan-{n}-practice-plot"), "plot"),
                Step::Capture(format!("plan-{n}-practice-controls"), "controls"),
                Step::Capture(format!("plan-{n}-practice-results"), "result"),
                choose("quote", if n == 0 { "costly" } else { "small-expansion" }),
                click("test"),
                Step::Assert(if n == 0 {
                    "practice-budget"
                } else {
                    "practice-mixed"
                }),
                Step::Capture(
                    format!("plan-{n}-individual-pass-overall-fail"),
                    "failure-band",
                ),
                choose(
                    "quote",
                    if n == 0 {
                        "standard"
                    } else {
                        "rapid-expansion"
                    },
                ),
                click("test"),
                Step::Assert("practice-pass"),
                click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("plan-{n}-assignment"), "top"),
                Step::Capture(format!("plan-{n}-assignment-inputs"), "controls"),
            ]);
            actions(&mut p, &d.reference);
            p.extend([
                Step::Assert("assignment-pass"),
                Step::Capture(format!("plan-{n}-assignment-plot"), "plot"),
                Step::Capture(format!("plan-{n}-assignment-results"), "result"),
                Step::KeyboardTest,
                Step::Assert("assignment-pass"),
                Step::Field("reserve", "0".into()),
                Step::Assert("assignment-unchecked"),
                click("test"),
                Step::Assert("assignment-fail"),
                Step::Capture(format!("plan-{n}-reserve-feedback"), "feedback-1"),
                Step::Field("reserve", d.initial.policy.minimum_reserve_kib.to_string()),
                click("test"),
                Step::Assert("assignment-pass"),
            ]);
            p.extend([
                choose("window-first", if n == 0 { 2 } else { 3 }),
                choose("window-last", if n == 0 { 3 } else { 4 }),
                choose("quote", if n == 0 { "fast" } else { "urgent-expansion" }),
                Step::Field("order-day", if n == 0 { "15" } else { "31" }.into()),
                Step::Field(
                    "reserve",
                    (d.initial.policy.minimum_reserve_kib + 1048576).to_string(),
                ),
                click("test"),
                Step::Assert("assignment-pass"),
                Step::Save,
            ]);
            if n == 1 {
                p.extend([
                    Step::RememberOthers,
                    click("reset"),
                    click("confirm-reset"),
                    Step::Assert("reset"),
                    click("challenge"),
                    Step::Assert("challenge"),
                    Step::Capture("plan-1-challenge".into(), "top"),
                ]);
                actions(&mut p, &d.reference);
                p.extend([Step::Assert("assignment-pass"), Step::Save]);
            }
            p.extend([
                Step::Style(1.5, false),
                Step::Capture(format!("plan-{n}-dark150-controls"), "controls"),
                Step::Style(2., true),
                Step::Capture(format!("plan-{n}-light200-plot"), "plot"),
                Step::Capture(format!("plan-{n}-light200-input"), "reserve"),
                Step::Capture(format!("plan-{n}-light200-summary"), "result"),
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
    others: Option<serde_json::Value>,
}
fn session(app: &LabApp) -> &work::Session {
    let engine::State::CapacityPlanning { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::CapacityPlanning { definition } = &app.scenarios[app.selected.unwrap()].goal
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
    let task = if practice(app) {
        &d.example
    } else {
        &d.initial
    };
    let report = task.assess(scope(app));
    match what {
        "practice-pass" => {
            assert!(s.practice.checked && report.passed());
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "practice-fail" | "practice-budget" | "practice-mixed" => {
            assert!(s.practice.checked && !report.passed());
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
            if what == "practice-budget" {
                assert!(report.cases.iter().all(|c| c.first_failure.is_none()));
                assert!(
                    report
                        .checks
                        .iter()
                        .any(|r| !r.0 && r.1.starts_with("Budget:"))
                );
            }
            if what == "practice-mixed" {
                assert!(report.cases.iter().any(|c| c.first_failure.is_none()));
                assert!(report.cases.iter().any(|c| c.first_failure.is_some()));
            }
        }
        "assignment-pass" => {
            assert!(progress::current_solved(a, &app.scenarios[i]));
            assert!(report.passed());
        }
        "assignment-fail" => {
            assert!(s.assignment.checked && !report.passed());
            assert!(!progress::current_solved(a, &app.scenarios[i]));
            assert!(
                report
                    .checks
                    .iter()
                    .any(|r| !r.0 && r.1.starts_with("Reserve:"))
            );
        }
        "assignment-unchecked" => {
            assert!(!s.assignment.checked);
            assert!(!progress::current_solved(a, &app.scenarios[i]));
            assert!(app.metrics.capacity_planning.result.is_none());
        }
        "fresh" => {
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
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
        _ => panic!("Unknown planning observation"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    let m = &app.metrics.capacity_planning;
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "intro-end" => m.controls.iter().find(|r| r.0 == "start").map(|r| r.1),
        "plot" => m.graph,
        "result" => m.result.map(|r| r.2),
        "controls" => m.controls.iter().find(|r| r.0 == "window-first").map(|r| {
            r.1.union(m.fields.iter().find(|r| r.0 == "reserve").unwrap().1)
        }),
        "failure-band" => m.result.map(|r| {
            r.2.union(
                m.controls
                    .iter()
                    .find(|r| {
                        r.0 == if app.selected.is_some_and(|i| app.scenarios[i].id == IDS[0]) {
                            "feedback-3"
                        } else {
                            "feedback-5"
                        }
                    })
                    .unwrap()
                    .1,
            )
        }),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        _ => m
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .or_else(|| m.fields.iter().find(|r| r.0 == key).map(|r| r.1)),
    }
}
impl LabApp {
    fn planning_click(&mut self, ctx: &egui::Context, r: Option<egui::Rect>) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = r.unwrap();
        self.reveal_smoke_control(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn planning_choice(&mut self, ctx: &egui::Context, key: &str, flow: &mut Flow) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (_, r, clip) = self
            .metrics
            .capacity_planning
            .choices
            .iter()
            .find(|r| r.0 == key)
            .cloned()
            .expect("Actual planning popup row");
        let visible = clip.intersect(ctx.content_rect());
        if !smoke_control_visible(r, visible, ctx.pixels_per_point()) {
            flow.reveals += 1;
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
    pub(in crate::ui) fn capacity_planning_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("capacity-planning-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "CAPACITY_PLANNING_NATIVE_COMPLETE: {} actions",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 145000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Planning step {} {:?}: {due:?}",
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
                assert_eq!((self.scenarios.len(), self.units.len()), (404, 155));
                self.launch(self.scenarios.iter().position(|s| s.id == id).unwrap());
                assert_eq!(
                    definition(self),
                    fixture(IDS.iter().position(|v| *v == id).unwrap())
                );
                true
            }
            Step::Click(key) => self.planning_click(ctx, target(self, &key)),
            Step::Choose(key, value) => {
                let selected = match key {
                    "window-first" => scope(self).inputs.first.map(|i| i.to_string()),
                    "window-last" => scope(self).inputs.last.map(|i| i.to_string()),
                    "quote" => scope(self).inputs.offer.clone(),
                    _ => panic!(),
                };
                if flow.phase == 0 && selected.as_ref() == Some(&value) {
                    true
                } else if flow.phase == 0 {
                    if self.planning_click(ctx, target(self, key)) {
                        flow.phase = 1;
                    }
                    false
                } else {
                    self.planning_choice(ctx, &format!("{key}-{value}"), &mut flow)
                }
            }
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    let (_, r, input, _) = self
                        .metrics
                        .capacity_planning
                        .fields
                        .iter()
                        .find(|r| r.0 == key)
                        .unwrap()
                        .clone();
                    if self.planning_click(ctx, Some(r)) {
                        assert_eq!(ctx.memory(|m| m.focused()), Some(input));
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.case_lesson_type(&value);
                    flow.phase = 2;
                    false
                } else {
                    let actual = if key == "order-day" {
                        &scope(self).inputs.order_day
                    } else {
                        &scope(self).inputs.reserve_kib
                    };
                    assert_eq!(actual, &value);
                    true
                }
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
            Step::KeyboardTest => {
                if flow.phase == 0 {
                    let (_, r, input, _) = self
                        .metrics
                        .capacity_planning
                        .fields
                        .iter()
                        .find(|r| r.0 == "order-day")
                        .unwrap()
                        .clone();
                    if self.planning_click(ctx, Some(r)) {
                        assert_eq!(ctx.memory(|m| m.focused()), Some(input));
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
                    assert!(scope(self).checked);
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
                    if target(self, key).is_some_and(|r| self.reveal_smoke_control(ctx, r)) {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1
                    && !target(self, key).is_some_and(|r| {
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
                "CAPACITY_PLANNING_NATIVE_STEP {} {:?}",
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
fn fixed_native_plan_uses_actual_fields_and_preserves_selected_reset() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 2);
    assert_eq!(
        p.iter().filter(|s| matches!(s, Step::KeyboardTest)).count(),
        2
    );
    for (n, id) in IDS.into_iter().enumerate() {
        let start = p
            .iter()
            .position(|s| matches!(s,Step::Open(value)if *value==id))
            .unwrap();
        let end = p[start + 1..]
            .iter()
            .position(|s| matches!(s, Step::Open(_)))
            .map(|i| start + 1 + i)
            .unwrap_or(p.len());
        let steps = &p[start..end];
        let d = fixture(n);
        for action in d.reference.iter().chain(&d.example_reference) {
            let field = match action {
                work::Action::OrderDay { value } => Some(("order-day", value)),
                work::Action::Reserve { value } => Some(("reserve", value)),
                _ => None,
            };
            if let Some((key, value)) = field {
                assert!(
                    steps
                        .iter()
                        .any(|s| matches!(s,Step::Field(k,v)if *k==key&&v==value)),
                    "Missing actual input for{id}/{key}"
                );
            }
        }
        assert!(
            steps
                .iter()
                .any(|s| matches!(s,Step::Field("reserve",v)if v=="0"))
        );
    }
    assert!(p.iter().any(|s| matches!(s, Step::Assert("reset"))));
    assert!(
        p.iter()
            .any(|s| matches!(s, Step::Assert("practice-budget")))
    );
    assert!(
        p.iter()
            .any(|s| matches!(s, Step::Assert("practice-mixed")))
    );
}
#[cfg(test)]
#[test]
fn completed_planning_popup_click_needs_no_closed_row() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    app.smoke_stage = 145999;
    let id = egui::Id::new(("native-click-phases", app.smoke_stage));
    ctx.data_mut(|d| d.insert_temp(id, 2u8));
    let mut flow = Flow::default();
    assert!(app.planning_choice(&ctx, "quote-absent", &mut flow));
    assert_eq!(flow.reveals, 0);
    assert!(ctx.data(|d| d.get_temp::<u8>(id)).is_none());
}

#[cfg(test)]
#[test]
fn late_clipped_planning_capture_reveals_before_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    let id = egui::Id::new("capacity-planning-native");
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "reserve")))
        .unwrap();
    let clip = egui::Rect::from_min_max(egui::pos2(10., 10.), egui::pos2(450., 350.));
    app.metrics.main_viewport = Some(clip);
    app.metrics.main_scroll_region = Some(clip.expand2(egui::vec2(10., 0.)));
    app.metrics.capacity_planning.fields.push((
        "reserve".into(),
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
        |ctx| app.capacity_planning_tick(ctx),
    );
    assert_eq!(ctx.data(|d| d.get_temp::<Flow>(id)).unwrap().phase, 0);
    assert!(!app.capture_pending);
    app.metrics.capacity_planning.fields[0].1 =
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
        |ctx| app.capacity_planning_tick(ctx),
    );
    assert!(app.capture_pending);
    assert_eq!(ctx.data(|d| d.get_temp::<Flow>(id)).unwrap().phase, 2);
}
