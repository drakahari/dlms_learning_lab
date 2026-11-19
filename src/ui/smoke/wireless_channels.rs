//! Fixed native AP table exercise; no scans, host radios or learner commands.
use super::*;
use dlms_learning_lab::wireless_channel_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "ccna-wireless-neighbour-channels",
    "ccna-wireless-multisurvey-plan",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Field(String, String),
    Assert(&'static str),
    Capture(String, &'static str),
    Save,
    RememberOthers,
}
fn fixture(n: usize) -> &'static work::Definition {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let all = DATA.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/boot-entry-lessons-v24.json"
        ))
        .unwrap()
        .scenarios
    });
    let engine::Goal::WirelessChannels { definition } =
        &all.iter().find(|s| s.id == IDS[n]).unwrap().goal
    else {
        panic!()
    };
    definition
}
fn field(p: &mut Vec<Step>, id: &str, value: &str) {
    p.push(Step::Field(format!("field-{id}"), value.into()));
}
fn method(p: &mut Vec<Step>, t: &work::Task) {
    for f in &t.reference {
        field(p, &f.radio, &f.value);
    }
    p.push(Step::Click("check"));
}
fn plan() -> &'static Vec<Step> {
    static DATA: OnceLock<Vec<Step>> = OnceLock::new();
    DATA.get_or_init(|| {
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let d = fixture(n);
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("wireless-{n}-introduction"), "top"),
                Step::Capture(format!("wireless-{n}-channels"), "channel-strip"),
                Step::Capture(format!("wireless-{n}-worked"), "try"),
                Step::Click("example"),
                Step::Capture(format!("wireless-{n}-practice"), "plan-controls"),
                Step::Click("fill"),
                Step::Assert("filled-inert"),
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Capture(format!("wireless-{n}-practice-pass"), "result"),
            ]);
            field(&mut p, &d.example.reference[0].radio, "2");
            p.extend([
                Step::Assert("unchecked"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(format!("wireless-{n}-practice-broken"), "result"),
                Step::Click("fill"),
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Save,
                Step::Click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("wireless-{n}-assignment"), "plan-controls"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(format!("wireless-{n}-initial-feedback"), "result"),
            ]);
            field(&mut p, &d.initial.reference[0].radio, "");
            p.extend([
                Step::Click("check"),
                Step::Assert("blank"),
                Step::Capture(format!("wireless-{n}-blank-feedback"), "result"),
            ]);
            if n == 0 {
                field(&mut p, "reception", "1");
            } else {
                for (id, value) in [("north", "1"), ("east", "6"), ("south", "1"), ("west", "6")] {
                    field(&mut p, id, value);
                }
            }
            p.extend([
                Step::Click("check"),
                Step::Assert("partial"),
                Step::Capture(format!("wireless-{n}-partial-feedback"), "result"),
            ]);
            method(&mut p, &d.initial);
            p.extend([
                Step::Assert("assignment-pass"),
                Step::Capture(format!("wireless-{n}-assignment-pass"), "result"),
                Step::Save,
            ]);
            field(&mut p, &d.initial.reference[0].radio, "2");
            p.extend([
                Step::Assert("unchecked"),
                Step::Click("check"),
                Step::Assert("fail"),
                Step::Capture(format!("wireless-{n}-invalidated"), "result"),
            ]);
            method(&mut p, &d.initial);
            p.extend([
                Step::Assert("assignment-pass"),
                Step::RememberOthers,
                Step::Click("reset"),
                Step::Click("confirm-reset"),
                Step::Assert("reset"),
                Step::Save,
                Step::Click("challenge"),
                Step::Assert("challenge"),
                Step::Capture(format!("wireless-{n}-challenge"), "plan-controls"),
                Step::Style(1.5, false),
                Step::Capture(format!("wireless-{n}-dark150-fields"), "plan-controls"),
                Step::Style(2., true),
                Step::Capture(format!("wireless-{n}-light200-fields"), "plan-controls"),
            ]);
            for (index, key) in ["radio-0", "radio-1", "radio-2", "radio-3"]
                .iter()
                .enumerate()
                .take(d.initial.network.radios.len())
            {
                p.push(Step::Capture(
                    format!("wireless-{n}-light200-radio-{index}"),
                    key,
                ));
            }
            field(&mut p, &d.initial.reference[0].radio, "");
            p.extend([
                Step::Capture(
                    format!("wireless-{n}-light200-placeholder"),
                    "plan-controls",
                ),
                Step::Click("check"),
                Step::Assert("blank"),
                Step::Capture(format!("wireless-{n}-light200-feedback"), "result"),
                Step::Click("learn"),
                Step::Click("introduction"),
                Step::Capture(format!("wireless-{n}-light200-introduction"), "top"),
                Step::Capture(format!("wireless-{n}-light200-channels"), "channel-strip"),
                Step::Click("example"),
                Step::Click("fill"),
                Step::Capture(format!("wireless-{n}-light200-practice"), "plan-controls"),
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
    others: Option<serde_json::Value>,
}
fn session(app: &LabApp) -> &work::Session {
    let engine::State::WirelessChannels { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::WirelessChannels { definition } = &app.scenarios[app.selected.unwrap()].goal
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
fn task(app: &LabApp) -> &work::Task {
    if practice(app) {
        &definition(app).example
    } else {
        &definition(app).initial
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
    let s = session(app);
    let d = definition(app);
    let source = &app.scenarios[app.selected.unwrap()];
    let a = &app.progress.attempts[&source.id];
    match what {
        "filled-inert" => {
            assert_eq!(s.practice.fields, d.example.reference);
            assert!(!s.practice.checked());
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, source));
        }
        "practice-pass" => {
            assert!(practice(app));
            assert!(scope(app).solved(task(app)));
            assert!(!progress::current_solved(a, source));
            assert_eq!(
                app.metrics.wireless_channel.summary.as_deref(),
                Some("Example meets the plan")
            );
        }
        "assignment-pass" => {
            assert!(!practice(app));
            assert!(progress::current_solved(a, source) && app.metrics.completion);
            assert_eq!(
                app.metrics.wireless_channel.summary.as_deref(),
                Some("Assignment complete")
            );
        }
        "unchecked" => {
            assert!(!scope(app).checked());
            assert!(app.metrics.wireless_channel.summary.is_none());
            assert!(!app.metrics.completion);
        }
        "fail" => {
            assert!(scope(app).checked());
            assert!(
                task(app)
                    .rows(&scope(app).fields)
                    .unwrap()
                    .iter()
                    .any(|r| !r.correct)
            );
            assert!(!app.metrics.completion);
            assert!(
                app.metrics
                    .wireless_channel
                    .summary
                    .as_ref()
                    .unwrap()
                    .starts_with("Needs attention")
            );
        }
        "blank" => {
            assert!(scope(app).checked());
            assert!(app.metrics.wireless_channel.surveys_deferred);
            assert!(
                app.metrics
                    .wireless_channel
                    .field_feedback
                    .iter()
                    .any(|(_, s)| s.contains("Enter a channel"))
            );
            assert!(!app.metrics.completion);
        }
        "partial" => {
            let rows = task(app).rows(&scope(app).fields).unwrap();
            assert!(rows.iter().any(|r| r.correct) && rows.iter().any(|r| !r.correct));
            if source.id == IDS[1] {
                assert!(
                    rows.iter()
                        .filter(|r| !r.correct)
                        .all(|r| r.label.contains("Partition open"))
                );
            }
            assert!(!app.metrics.completion);
        }
        "fresh" => {
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, source));
        }
        "reset" => {
            assert_eq!(*s, work::Session::new(d));
            assert_eq!(others(app), *flow.others.as_ref().unwrap());
        }
        "challenge" => {
            assert!(a.challenge && app.metrics.lesson_tabs.is_empty());
            assert!(
                !app.metrics
                    .wireless_channel
                    .controls
                    .iter()
                    .any(|r| r.0 == "fill")
            );
            assert!(!app.metrics.wireless_channel.worked_answer_visible);
        }
        _ => panic!("Unknown wireless observation"),
    }
}
fn radio_card(app: &LabApp, index: usize) -> Option<egui::Rect> {
    let radio = task(app).network.radios.get(index)?;
    let controls = &app.metrics.wireless_channel.controls;
    let label = controls
        .iter()
        .find(|r| r.0 == format!("label-{}", radio.id))?
        .1;
    let value = controls
        .iter()
        .find(|r| {
            r.0 == format!(
                "{}-{}",
                if radio.managed { "field" } else { "fixed" },
                radio.id
            )
        })?
        .1;
    let mut card = label.union(value);
    if let Some(role) = controls
        .iter()
        .find(|r| r.0 == format!("role-{}", radio.id))
    {
        card = card.union(role.1);
    }
    Some(card)
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    let m = &app.metrics.wireless_channel.controls;
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "plan-controls" => radio_card(
            app,
            task(app).network.radios.iter().position(|r| r.managed)?,
        ),
        key if key.starts_with("radio-") => {
            radio_card(app, key.strip_prefix("radio-")?.parse().ok()?)
        }
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        "result" => {
            let head = m.iter().find(|r| r.0 == "result")?.1;
            Some(
                m.iter()
                    .find(|r| r.0.starts_with("feedback-"))
                    .map(|r| head.union(r.1))
                    .unwrap_or(head),
            )
        }
        _ => m.iter().find(|r| r.0 == key).map(|r| r.1),
    }
}
impl LabApp {
    pub(in crate::ui) fn queue_wireless_field_text(&mut self, value: &str) {
        self.case_lesson_type(value);
        if value.is_empty() {
            assert!(
                matches!(self.pending_events.pop(), Some(egui::Event::Text(text)) if text.is_empty())
            );
            self.pending_events
                .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                    key: egui::Key::Delete,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                }));
        }
    }
    pub(in crate::ui) fn open_wireless_channel_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), (439, 172));
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered wireless_channel lesson"),
        );
    }
    fn wireless_channel_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        let before = self.pending_events.len();
        let done = self.reveal_smoke_control(ctx, rect);
        if !done {
            let viewport = self.metrics.main_viewport.expect("actual main viewport");
            let region = self
                .metrics
                .main_scroll_region
                .expect("actual main scroll area");
            let gutter = egui::Rect::from_min_max(
                egui::pos2(viewport.right(), viewport.top()),
                egui::pos2(region.right(), viewport.bottom()),
            )
            .intersect(ctx.content_rect());
            assert!(
                gutter.width() * ctx.pixels_per_point() > 1.,
                "Need an actual outer scrollbar gutter"
            );
            // A wheel inside nested output scrolls its text, not the lesson.
            // Use the actual parent gutter, retaining all existing guard limits.
            for event in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(point) = event {
                    *point = gutter.center();
                }
            }
        }
        done
    }
    fn wireless_channel_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual wireless_channel control");
        self.wireless_channel_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn wireless_channel_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("wireless_channel-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("WIRELESS_CHANNEL_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 158000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Source step {} {:?}: {due:?}",
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
                self.open_wireless_channel_capture(id);
                assert_eq!(
                    definition(self),
                    fixture(IDS.iter().position(|v| *v == id).unwrap())
                );
                true
            }
            Step::Click(key) => self.wireless_channel_click(ctx, target(self, key)),
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.wireless_channel_click(ctx, target(self, &key)) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_wireless_field_text(&value);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(
                        &scope(self)
                            .fields
                            .iter()
                            .find(|f| format!("field-{}", f.radio) == key)
                            .unwrap()
                            .value,
                        &value
                    );
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
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    ctx.memory_mut(|m| {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    });
                    if target(self, key).is_some_and(|r| self.wireless_channel_reveal(ctx, r)) {
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
                "WIRELESS_CHANNEL_NATIVE_STEP {} {:?}",
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
fn wireless_channel_capture_plan_covers_real_edits_wrong_partial_reset_and_scaled_fields() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 2);
    for key in [
        "filled-inert",
        "practice-pass",
        "assignment-pass",
        "partial",
        "unchecked",
        "blank",
        "fail",
        "reset",
        "challenge",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if *v==key)));
    }
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Style(2., true)))
            .count(),
        2
    );
}
#[cfg(test)]
#[test]
fn wireless_channel_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "top")))
        .unwrap();
    let id = egui::Id::new("wireless_channel-native");
    let mut flow = Flow {
        at,
        ..Default::default()
    };
    flow.next = 0.;
    ctx.data_mut(|d| d.insert_temp(id, flow));
    let mut requested = false;
    for frame in 0..12 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920., 1080.),
                )),
                time: Some(frame as f64 + 1.),
                ..Default::default()
            },
            |ctx| {
                app.render(ctx);
                app.wireless_channel_tick(ctx);
            },
        );
        requested |= output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        });
        let flow = ctx.data(|d| d.get_temp::<Flow>(id)).unwrap();
        if flow.phase == 2 {
            assert!(requested && app.capture_pending);
            app.capture_pending = false;
        }
        if flow.at > at {
            assert_eq!(flow.at, at + 1);
            assert!(requested);
            return;
        }
    }
    panic!("Actual wireless_channel capture phases did not complete");
}

#[cfg(test)]
#[test]
fn wireless_channel_large_text_captures_complete_individual_cards_not_an_oversized_page() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.scale = 2.;
    ctx.set_zoom_factor(2.);
    app.open_wireless_channel_capture(IDS[1]);
    if let engine::State::WirelessChannels { simulation } =
        &mut app.progress.attempts.get_mut(IDS[1]).unwrap().state
    {
        simulation.view = teaching::View::Example;
    }
    for _ in 0..4 {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(946., 929.) / 2.3,
                )),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
    }
    let viewport = app.metrics.main_viewport.unwrap();
    let whole = app
        .metrics
        .wireless_channel
        .controls
        .iter()
        .filter(|r| r.0.starts_with("field-") || r.0.starts_with("label-"))
        .map(|r| r.1)
        .reduce(|a, b| a.union(b))
        .unwrap();
    assert!(
        whole.height() > viewport.height(),
        "This fixture must reproduce the oversized aggregate: {whole:?} / {viewport:?}"
    );
    for n in 0..task(&app).network.radios.len() {
        let card = radio_card(&app, n).unwrap();
        assert!(card.height() < viewport.height());
        let radio = &task(&app).network.radios[n];
        for prefix in ["label", if radio.managed { "field" } else { "fixed" }] {
            let control = app
                .metrics
                .wireless_channel
                .controls
                .iter()
                .find(|r| r.0 == format!("{prefix}-{}", radio.id))
                .unwrap();
            assert!(card.contains_rect(control.1));
        }
    }
    assert_eq!(target(&app, "plan-controls"), radio_card(&app, 0));
}
