//! Actual native supplied-project practice; no learner-host execution.
use super::*;
use dlms_learning_lab::source_install_workspace as work;
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-source-project-build",
    "lpic2-source-stage-install",
    "lpic2-source-stale-handover",
];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    ChooseCommand(usize),
    Field(&'static str, String),
    Assert(&'static str),
    Capture(String, &'static str),
    KeyboardRun,
    Save,
    RememberOthers,
}
fn fixture(n: usize) -> &'static work::Definition {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    let all = DATA.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/boot-entry-lessons-v19.json"
        ))
        .unwrap()
        .scenarios
    });
    let engine::Goal::SourceInstall { definition } =
        &all.iter().find(|s| s.id == IDS[n]).unwrap().goal
    else {
        panic!()
    };
    definition
}
fn command(p: &mut Vec<Step>, raw: &str) {
    p.push(Step::Field("command", raw.into()));
    p.push(Step::Click("run"));
}
fn configure(t: &work::Task, greeting: &str) -> String {
    format!(
        "./configure --prefix={} --with-greeting='{greeting}' {}",
        t.required.prefix,
        if t.required.uppercase {
            "--enable-uppercase"
        } else {
            "--disable-uppercase"
        }
    )
}
fn program(t: &work::Task) -> String {
    match &t.destination {
        work::Destination::Build => format!("./{}", t.facts.program.name),
        work::Destination::Staged { directory } => format!(
            "{directory}{}/bin/{}",
            t.required.prefix, t.facts.program.name
        ),
    }
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![];
        for (n, id) in IDS.iter().enumerate() {
            let d = fixture(n);
            let t = &d.initial;
            let example = &d.example;
            p.extend([
                Step::Style(1., false),
                Step::Open(id),
                Step::Capture(format!("source-{n}-introduction"), "top"),
                Step::Capture(format!("source-{n}-worked"), "try"),
                Step::Click("example"),
                Step::Capture(format!("source-{n}-practice-start"), "command"),
                Step::ChooseCommand(1),
                Step::Click("insert"),
                Step::Assert("inserted-inert"),
                Step::Capture(format!("source-{n}-inert-insert"), "command"),
                Step::Field("command", "pwd".into()),
                Step::KeyboardRun,
                Step::Assert("actual-pwd"),
                Step::Capture(format!("source-{n}-actual-pwd"), "last-result"),
            ]);
            command(&mut p, "./configure --with-greeting='unfinished");
            p.extend([
                Step::Assert("syntax"),
                Step::Capture(format!("source-{n}-syntax-error"), "last-result"),
            ]);
            command(&mut p, "bash script.sh");
            p.extend([
                Step::Assert("unsupported"),
                Step::Capture(format!("source-{n}-unsupported"), "last-result"),
            ]);
            for c in &example.reference {
                command(&mut p, &c.0);
            }
            p.extend([
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Capture(format!("source-{n}-practice-pass"), "result"),
                Step::Save,
            ]);
            command(&mut p, &configure(example, "Wrong,"));
            p.extend([
                Step::Assert("practice-unchecked"),
                Step::Click("check"),
                Step::Assert("practice-fail"),
                Step::Capture(format!("source-{n}-practice-feedback"), "result"),
            ]);
            command(&mut p, &configure(example, &example.required.greeting));
            for c in &example.reference {
                command(&mut p, &c.0);
            }
            p.extend([
                Step::Click("check"),
                Step::Assert("practice-pass"),
                Step::Click("assignment"),
                Step::Assert("fresh"),
                Step::Capture(format!("source-{n}-assignment"), "command"),
            ]);
            if n == 0 {
                for c in t.reference.iter().take(4) {
                    command(&mut p, &c.0);
                }
                command(&mut p, &configure(t, "Wrong,"));
                command(&mut p, "make check");
            } else if n == 1 {
                for c in t.reference.iter().take(6) {
                    command(&mut p, &c.0);
                }
            } else {
                command(&mut p, "make check");
            }
            p.extend([
                Step::Click("check"),
                Step::Assert("assignment-fail"),
                Step::Capture(format!("source-{n}-partial-feedback"), "result"),
            ]);
            for c in &t.reference {
                command(&mut p, &c.0);
            }
            p.extend([
                Step::Click("check"),
                Step::Assert("assignment-pass"),
                Step::Capture(format!("source-{n}-assignment-pass"), "result"),
            ]);
            command(&mut p, &format!("{} 'Sam Lee'", program(t)));
            p.extend([
                Step::Assert("actual-spaced-name"),
                Step::Capture(format!("source-{n}-actual-spaced-name"), "last-result"),
            ]);
            command(&mut p, "bash script.sh");
            p.extend([Step::Assert("unsupported-still-solved")]);
            command(&mut p, &configure(t, "Wrong,"));
            p.extend([
                Step::Assert("assignment-unchecked"),
                Step::Click("check"),
                Step::Assert("assignment-fail"),
                Step::Assert("case-pass-overall-fail"),
                Step::Capture(format!("source-{n}-case-pass-overall-fail"), "result"),
                Step::RememberOthers,
                Step::Click("reset"),
                Step::Click("confirm-reset"),
                Step::Assert("reset"),
                Step::Click("challenge"),
                Step::Assert("challenge"),
                Step::Capture(format!("source-{n}-challenge"), "command"),
                Step::Style(1.5, false),
                Step::Capture(format!("source-{n}-dark150-command"), "command"),
                Step::Capture(format!("source-{n}-dark150-state"), "state-Source archive"),
                Step::Style(2., true),
                Step::Capture(format!("source-{n}-light200-command"), "command"),
                Step::Capture(format!("source-{n}-light200-state"), "state-Source archive"),
                Step::Click("learn"),
                Step::Click("introduction"),
                Step::Capture(format!("source-{n}-light200-introduction"), "top"),
                Step::Capture(
                    format!("source-{n}-light200-worked-command"),
                    "worked-value-3",
                ),
                Step::Click("example"),
                Step::ChooseCommand(example.reference.len() - 1),
                Step::Click("insert"),
                Step::Capture(format!("source-{n}-light200-insert"), "command"),
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
    let engine::State::SourceInstall { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn definition(app: &LabApp) -> &work::Definition {
    let engine::Goal::SourceInstall { definition } = &app.scenarios[app.selected.unwrap()].goal
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
fn solved(d: &work::Draft, t: &work::Task) -> bool {
    d.checked
        && t.replay(&d.history)
            .is_ok_and(|f| t.grade(&f).iter().all(|c| c.correct))
}
fn observe(app: &LabApp, what: &str, flow: &Flow) {
    let i = app.selected.unwrap();
    let a = &app.progress.attempts[&app.scenarios[i].id];
    let s = session(app);
    let d = definition(app);
    let current = scope(app);
    let t = task(app);
    match what {
        "inserted-inert" => {
            assert!(practice(app));
            assert_eq!(current.history, t.starter);
            assert!(current.last_run.is_none());
            assert_eq!(current.input, t.reference[1].0);
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
        }
        "actual-pwd" => {
            let out = current.last_output(t).unwrap().unwrap().unwrap();
            assert_eq!(out.status, 0);
            let starting = t.replay(&t.starter).unwrap();
            assert_eq!(out.stdout, format!("{}\n", starting.cwd));
        }
        "syntax" | "unsupported" | "unsupported-still-solved" => {
            let error = current.last_output(t).unwrap().unwrap().unwrap_err();
            assert_eq!(
                error.kind,
                if what == "syntax" {
                    dlms_learning_lab::boot_entry::ErrorKind::Syntax
                } else {
                    dlms_learning_lab::boot_entry::ErrorKind::Unsupported
                }
            );
            if what == "unsupported-still-solved" {
                assert!(progress::current_solved(a, &app.scenarios[i]));
            }
        }
        "practice-pass" => {
            assert!(practice(app) && solved(current, t));
            assert_eq!(s.assignment, work::Draft::new(&d.initial));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "practice-fail" => {
            assert!(practice(app) && current.attempted && !solved(current, t));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "practice-unchecked" | "assignment-unchecked" => {
            assert!(!current.checked && !current.attempted);
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "assignment-pass" => {
            assert!(!practice(app) && solved(current, t));
            assert!(progress::current_solved(a, &app.scenarios[i]));
        }
        "assignment-fail" => {
            assert!(!practice(app) && current.attempted && !solved(current, t));
            assert!(!progress::current_solved(a, &app.scenarios[i]));
        }
        "case-pass-overall-fail" => {
            let frame = t.replay(&current.history).unwrap();
            let rows = t.grade(&frame);
            assert!(
                rows.iter()
                    .filter(|r| r.label.starts_with("Program case"))
                    .all(|r| r.correct)
            );
            assert!(rows.iter().any(|r| !r.correct));
            assert!(
                app.metrics
                    .source_install
                    .summary
                    .as_ref()
                    .unwrap()
                    .starts_with("Needs attention")
            );
        }
        "actual-spaced-name" => {
            let out = current.last_output(t).unwrap().unwrap().unwrap();
            assert_eq!(
                out.stdout,
                t.cases
                    .iter()
                    .find(|c| c.label == "Name with a space")
                    .unwrap()
                    .stdout
            );
            assert_eq!(out.status, 0);
            assert!(progress::current_solved(a, &app.scenarios[i]));
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
            assert!(
                !app.metrics
                    .source_install
                    .controls
                    .iter()
                    .any(|r| r.0 == "insert" || r.0 == "worked-menu")
            );
        }
        _ => panic!("Unknown source-project observation"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    match key {
        "top" => app
            .metrics
            .main_viewport
            .map(|r| egui::Rect::from_min_size(r.min, egui::vec2(1., 1.))),
        "reset" => app.metrics.reset,
        "confirm-reset" => app.metrics.confirm_reset,
        "challenge" => app.metrics.mode_challenge,
        "learn" => app.metrics.mode_learn,
        "result" => {
            let m = &app.metrics.source_install.controls;
            let head = m.iter().find(|r| r.0 == "result")?.1;
            let frame = task(app).replay(&scope(app).history).ok()?;
            let failed = task(app).grade(&frame).iter().position(|c| !c.correct);
            Some(
                failed
                    .and_then(|n| {
                        m.iter()
                            .find(|r| r.0 == format!("feedback-{n}"))
                            .map(|r| head.union(r.1))
                    })
                    .unwrap_or(head),
            )
        }
        _ => app
            .metrics
            .source_install
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1),
    }
}
impl LabApp {
    pub(in crate::ui) fn open_source_install_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), CURRENT_CATALOG_SIZE);
        assert!(IDS.contains(&id));
        self.launch(
            self.scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("Real registered source lesson"),
        );
    }
    fn source_build_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
    fn source_build_click(&mut self, ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = rect.expect("actual source-project control");
        self.source_build_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    fn source_build_choice(&mut self, ctx: &egui::Context, key: &str, flow: &mut Flow) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (_, r, clip) = self
            .metrics
            .source_install
            .controls
            .iter()
            .find(|r| r.0 == key)
            .cloned()
            .expect("actual source-project popup row");
        let clip = clip.intersect(ctx.content_rect());
        if !smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            flow.reveals += 1;
            assert!(flow.reveals < 20);
            self.pending_events.extend([
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-400., 400.)),
                    modifiers: Default::default(),
                },
            ]);
            false
        } else {
            self.click_smoke_control(ctx, Some(r))
        }
    }
    pub(in crate::ui) fn source_install_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("source-install-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("SOURCE_INSTALL_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 147000 + flow.at;
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
                self.open_source_install_capture(id);
                assert_eq!(
                    definition(self),
                    fixture(IDS.iter().position(|v| *v == id).unwrap())
                );
                true
            }
            Step::Click(key) => self.source_build_click(ctx, target(self, key)),
            Step::ChooseCommand(index) => {
                if flow.phase == 0 {
                    if self.source_build_click(ctx, target(self, "worked-menu")) {
                        flow.phase = 1;
                    }
                    false
                } else {
                    self.source_build_choice(ctx, &format!("worked-{index}"), &mut flow)
                }
            }
            Step::Field(key, value) => {
                if flow.phase == 0 {
                    if self.source_build_click(ctx, target(self, key)) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.case_lesson_type(&value);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(scope(self).input, value);
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
            Step::KeyboardRun => {
                if flow.phase == 0 {
                    if self.source_build_click(ctx, target(self, "command")) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
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
                    assert_eq!(scope(self).last_run.as_ref().unwrap().command.0, "pwd");
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
                    if target(self, key).is_some_and(|r| self.source_build_reveal(ctx, r)) {
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
                "SOURCE_INSTALL_NATIVE_STEP {} {:?}",
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
fn source_capture_plan_uses_actual_commands_inert_insert_failures_reset_and_large_text() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 3);
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::RememberOthers))
            .count(),
        3
    );
    for token in [
        "inserted-inert",
        "syntax",
        "unsupported",
        "practice-pass",
        "practice-fail",
        "assignment-pass",
        "assignment-fail",
        "case-pass-overall-fail",
        "actual-spaced-name",
        "reset",
        "challenge",
    ] {
        assert!(p.iter().any(|s| matches!(s,Step::Assert(v)if *v==token)));
    }
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Style(2., true)))
            .count(),
        3
    );
    assert_eq!(
        p.iter().filter(|s| matches!(s, Step::KeyboardRun)).count(),
        3
    );
}

#[cfg(test)]
#[test]
fn source_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == IDS[0]).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(_, "top")))
        .unwrap();
    let id = egui::Id::new("source-install-native");
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
                app.source_install_tick(ctx);
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
    panic!("Actual source capture phases did not complete");
}

#[cfg(test)]
#[test]
fn source_pwd_observation_uses_the_replayed_inherited_working_directory() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    app.open_source_install_capture(IDS[2]);
    let def = definition(&app).clone();
    let expected = def.example.replay(&def.example.starter).unwrap();
    assert_ne!(expected.cwd, def.example.initial_directory);
    let engine::State::SourceInstall { simulation } =
        &mut app.progress.attempts.get_mut(IDS[2]).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Example;
    simulation.practice.input = "pwd".into();
    simulation.practice.run(&def.example).unwrap().unwrap();
    observe(&app, "actual-pwd", &Flow::default());
}
