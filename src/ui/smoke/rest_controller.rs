//! Fixed native requests against the in-process learner model, never real HTTP.
use super::*;
use dlms_learning_lab::{rest_controller as api, rest_workspace as work};
use std::sync::OnceLock;
const FIRST: &str = "ccna-rest-change-request";
const SECOND: &str = "ccna-rest-profile-handover";
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(String),
    Field(&'static str, String),
    Assert(&'static str),
    Capture(&'static str, &'static str),
    Save,
}
fn request(plan: &mut Vec<Step>, r: api::Request) {
    plan.push(Step::Click("method".into()));
    plan.push(Step::Click(format!("method:{}", r.method)));
    for (key, value) in [
        ("path", r.path),
        ("authorization", r.authorization),
        ("content-type", r.content_type),
        ("body", r.body),
    ] {
        plan.push(Step::Field(key, value));
    }
    plan.push(Step::Click("send".into()));
}
fn fixed(method: &str, path: &str, body: &str) -> api::Request {
    api::Request {
        method: method.into(),
        path: path.into(),
        authorization: "Bearer lab-operator".into(),
        content_type: if method == "PATCH" {
            "application/merge-patch+json".into()
        } else {
            "application/json".into()
        },
        body: body.into(),
    }
}
fn plan() -> &'static Vec<Step> {
    static PLAN: OnceLock<Vec<Step>> = OnceLock::new();
    PLAN.get_or_init(|| {
        use Step::*;
        let mut p = vec![
            Style(1., false),
            Open(FIRST),
            Capture("rest-update-introduction", "top"),
            Click("example".into()),
            Capture("rest-update-practice", "body"),
            Click("send".into()),
            Assert("403"),
            Capture("rest-update-forbidden", "response"),
            Field("authorization", "Bearer lab-operator".into()),
            Click("send".into()),
            Assert("415"),
            Field("content-type", "application/merge-patch+json".into()),
            Assert("body-guidance"),
            Capture("rest-update-json-guidance-before-answer", "body"),
            Click("send".into()),
            Assert("422"),
            Capture("rest-update-type-error", "diagnostic"),
            Field("body", r#"{"up":true}"#.into()),
            Click("send".into()),
            Click("check".into()),
            Assert("practice-pass"),
            Capture("rest-update-practice-pass", "result"),
            Click("assignment".into()),
            Assert("fresh"),
            Capture("rest-update-assignment", "body"),
        ];
        request(
            &mut p,
            fixed(
                "PUT",
                "/profiles/staff",
                r#"{"vlan":10,"up":true,"description":"Staff desk service"}"#,
            ),
        );
        p.extend([
            Click("check".into()),
            Assert("assignment-pass"),
            Capture("rest-update-alternate-pass", "result"),
            Field("body", "{ unfinished".into()),
            Assert("unchecked"),
            Click("send".into()),
            Assert("400"),
            Capture("rest-update-syntax-error", "response"),
        ]);
        request(&mut p, fixed("PATCH", "/profiles/staff", r#"{"up":false}"#));
        p.extend([
            Click("check".into()),
            Assert("partial"),
            Capture("rest-update-broken-again", "result"),
        ]);
        request(&mut p, fixed("PATCH", "/profiles/staff", r#"{"up":true}"#));
        p.extend([
            Click("check".into()),
            Assert("assignment-pass"),
            Save,
            Style(1.5, false),
            Capture("rest-update-dark150-fields", "authorization"),
            Style(2., true),
            Capture("rest-update-light200-authorization", "authorization"),
            Capture("rest-update-light200-fields", "body"),
            Capture("rest-update-light200-result", "result"),
            Style(1., false),
            Open(SECOND),
            Capture("rest-handover-introduction", "top"),
            Click("example".into()),
            Capture("rest-handover-practice", "body"),
        ]);
        let data: pack::Pack = serde_json::from_str(include_str!(
            "../../../tests/fixtures/rest-controller-preview-v1.json"
        ))
        .unwrap();
        let engine::Goal::RestController { definition } = &data.scenarios[1].goal else {
            panic!()
        };
        for (i, r) in definition.example.reference.iter().enumerate() {
            request(&mut p, r.clone());
            p.push(Click("check".into()));
            p.push(Assert(if i == 3 { "practice-pass" } else { "partial" }));
            if i == 1 {
                p.push(Capture("rest-handover-partial-pair", "result"));
            }
            if i == 2 {
                p.push(Assert("retirement-guidance"));
                p.push(Capture(
                    "rest-handover-retire-guidance-before-delete",
                    "result",
                ));
            }
        }
        p.extend([
            Capture("rest-handover-practice-pass", "result"),
            Click("assignment".into()),
            Assert("fresh"),
            Capture("rest-handover-assignment", "body"),
        ]);
        request(&mut p, fixed("DELETE", "/profiles/visitors", ""));
        p.extend([
            Assert("409"),
            Capture("rest-handover-referenced-delete", "response"),
        ]);
        for r in [
            fixed(
                "PUT",
                "/profiles/review-alternate",
                r#"{"up":true,"vlan":30}"#,
            ),
            fixed(
                "PUT",
                "/bindings/desk-3",
                r#"{"profile":"review-alternate"}"#,
            ),
            fixed(
                "PUT",
                "/bindings/desk-4",
                r#"{"profile":"review-alternate"}"#,
            ),
            fixed("DELETE", "/profiles/visitors", ""),
        ] {
            request(&mut p, r);
        }
        p.extend([
            Click("check".into()),
            Assert("assignment-pass"),
            Capture("rest-handover-alternate-pass", "result"),
            Save,
            Click("challenge".into()),
            Assert("challenge"),
            Capture("rest-handover-challenge", "body"),
            Click("learn".into()),
            Style(1.5, false),
            Capture("rest-handover-dark150-body", "body"),
            Style(2., true),
            Capture("rest-handover-light200-body", "body"),
            Capture("rest-handover-light200-result", "result"),
            Save,
        ]);
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
}
fn session(app: &LabApp) -> &work::Session {
    let engine::State::RestController { simulation } =
        &app.progress.attempts[&app.scenarios[app.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn scope(app: &LabApp) -> (&work::Draft, &work::Task) {
    let source = &app.scenarios[app.selected.unwrap()];
    let engine::Goal::RestController { definition } = &source.goal else {
        panic!()
    };
    let practice = !app.progress.attempts[&source.id].challenge
        && session(app).view == teaching::View::Example;
    if practice {
        (&session(app).practice, &definition.example)
    } else {
        (&session(app).assignment, &definition.initial)
    }
}
fn value(app: &LabApp, key: &str) -> String {
    let r = &scope(app).0.request;
    match key {
        "path" => r.path.clone(),
        "authorization" => r.authorization.clone(),
        "content-type" => r.content_type.clone(),
        "body" => r.body.clone(),
        _ => panic!("fixed field"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<(egui::Rect, egui::Rect)> {
    let viewport = app.metrics.main_viewport?;
    match key {
        "top" => Some((
            egui::Rect::from_min_size(viewport.min, egui::vec2(1., 1.)),
            viewport,
        )),
        "challenge" => app.metrics.mode_challenge.map(|r| (r, viewport)),
        "learn" => app.metrics.mode_learn.map(|r| (r, viewport)),
        _ => app
            .metrics
            .rest_controller
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| (r.1, r.2)),
    }
}
fn observe(app: &LabApp, expected: &str) {
    let (draft, task) = scope(app);
    let checks = draft.checks(task);
    let source = &app.scenarios[app.selected.unwrap()];
    let solved = progress::current_solved(&app.progress.attempts[&source.id], source);
    match expected {
        "400" | "403" | "409" | "415" | "422" => assert!(
            app.notice
                .as_ref()
                .unwrap()
                .starts_with(&format!("Actual response · {expected} "))
                && !draft.checked
        ),
        "body-guidance" => assert!(
            draft.changes.is_empty()
                && !draft.checked
                && app.metrics.rest_controller.practice_level == Some(1)
        ),
        "retirement-guidance" => assert!(
            draft.store(task).unwrap().profiles.contains_key("guests")
                && app.metrics.rest_controller.practice_level == Some(2)
                && !solved
        ),
        "fresh" => assert_eq!(*draft, work::Draft::new(task)),
        "unchecked" => assert!(!draft.checked && !solved),
        "partial" => assert!(
            draft.checked && checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0) && !solved
        ),
        "practice-pass" => assert!(draft.checked && checks.iter().all(|c| c.0) && !solved),
        "assignment-pass" => assert!(solved && checks.iter().all(|c| c.0)),
        "challenge" => assert!(
            solved
                && !app
                    .metrics
                    .rest_controller
                    .controls
                    .iter()
                    .any(|r| matches!(r.0.as_str(), "example" | "introduction"))
        ),
        _ => panic!("fixed observation"),
    }
}
impl LabApp {
    pub(in crate::ui) fn open_rest_controller_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), CURRENT_CATALOG_SIZE);
        assert!(matches!(id, FIRST | SECOND));
        self.launch(self.scenarios.iter().position(|s| s.id == id).unwrap());
    }
    fn rest_capture_reveal(&mut self, ctx: &egui::Context, r: egui::Rect) -> bool {
        let before = self.pending_events.len();
        let done = self.reveal_smoke_control(ctx, r);
        if !done {
            let viewport = self.metrics.main_viewport.unwrap();
            let region = self.metrics.main_scroll_region.unwrap();
            let gutter = egui::Rect::from_min_max(
                egui::pos2(viewport.right(), viewport.top()),
                egui::pos2(region.right(), viewport.bottom()),
            )
            .intersect(ctx.content_rect());
            assert!(
                gutter.width() * ctx.pixels_per_point() > 1.,
                "Actual parent scrollbar gutter"
            );
            for event in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(point) = event {
                    *point = gutter.center();
                }
            }
        }
        done
    }
    fn rest_capture_click(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (r, clip) = target(self, key).expect("Actual REST control");
        if key.starts_with("method:") {
            let clip = clip.intersect(ctx.content_rect());
            if smoke_control_visible(r, clip, ctx.pixels_per_point()) {
                return self.click_smoke_control(ctx, Some(r));
            }
            // ComboBox's own bounded scroll area can hide its last option.
            // Scroll that popup, never the lesson and never click clipped text.
            let attempts = egui::Id::new(("rest-popup-scroll", self.smoke_stage));
            let tries = ctx.data_mut(|d| {
                let n = d.get_temp::<usize>(attempts).unwrap_or(0) + 1;
                d.insert_temp(attempts, n);
                n
            });
            assert!(
                tries <= 20,
                "REST popup option remained clipped: {key} {r:?} {clip:?}"
            );
            let delta = if r.bottom() > clip.bottom() {
                clip.bottom() - r.bottom()
            } else if r.top() < clip.top() {
                clip.top() - r.top()
            } else {
                0.
            };
            assert!(
                delta != 0.,
                "Horizontal popup clipping cannot be repaired by vertical scrolling: {key} {r:?} {clip:?}"
            );
            println!("REST_POPUP_SCROLL {key} target={r:?} clip={clip:?} delta={delta}");
            self.pending_events.extend([
                egui::Event::PointerMoved(egui::pos2(
                    r.center().x.clamp(clip.left(), clip.right()),
                    clip.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ]);
            ctx.request_repaint();
            return false;
        }
        self.rest_capture_reveal(ctx, r) && self.click_smoke_control(ctx, Some(r))
    }
    pub(in crate::ui) fn rest_controller_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("rest-controller-native");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!(
                "REST_CONTROLLER_NATIVE_COMPLETE: {} actions; wide native only",
                plan().len()
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 166000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "REST step {} {:?}: {due:?}",
            flow.at,
            plan()[flow.at]
        );
        if !due.unwrap() {
            ctx.data_mut(|d| d.insert_temp(id, flow));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match plan()[flow.at].clone() {
            Step::Style(scale, light) => {
                if flow.phase == 0 {
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    self.lesson_scroll_reset = true;
                    flow.phase = 1;
                    false
                } else {
                    let viewport = ctx.content_rect();
                    let settled = flow.viewport == Some(viewport)
                        && (ctx.zoom_factor() - scale).abs() < 0.01
                        && ctx.input(|i| i.viewport().fullscreen) == Some(true);
                    flow.viewport = Some(viewport);
                    settled
                }
            }
            Step::Open(id) => {
                self.open_rest_controller_capture(id);
                true
            }
            Step::Click(key) => self.rest_capture_click(ctx, &key),
            Step::Field(key, text) => {
                if flow.phase == 0 {
                    if self.rest_capture_click(ctx, key) {
                        assert!(ctx.memory(|m| m.focused()).is_some());
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.queue_echo_diagnosis_field_text(&text);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(value(self, key), text);
                    true
                }
            }
            Step::Assert(expected) => {
                observe(self, expected);
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
                    if target(self, key).is_some_and(|(r, _)| self.rest_capture_reveal(ctx, r)) {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1
                    && !target(self, key).is_some_and(|(r, c)| {
                        smoke_control_visible(
                            r,
                            c.intersect(self.metrics.main_viewport.unwrap())
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
                        name.to_owned(),
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
                "REST_CONTROLLER_NATIVE_STEP {} {:?}",
                flow.at,
                plan()[flow.at]
            );
            flow.at += 1;
            flow.phase = 0;
            flow.polls = 0;
            flow.started = None;
            flow.viewport = None;
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
fn rest_controller_capture_actual_phases_request_a_screenshot_without_context_reentry() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(app._smoke_profile.is_some());
    app.smoke = false;
    app.launch(app.scenarios.iter().position(|s| s.id == FIRST).unwrap());
    let at = plan()
        .iter()
        .position(|s| matches!(s, Step::Capture(name, "top") if *name=="rest-update-introduction"))
        .unwrap();
    let id = egui::Id::new("rest-controller-native");
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
                app.rest_controller_tick(ctx);
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
    panic!("Actual echo_diagnosis capture phases did not complete");
}

#[cfg(test)]
#[test]
fn rest_controller_clipped_popup_queues_wheel_inside_popup_without_clicking() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    app.metrics.main_viewport = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(1920., 1080.),
    ));
    // egui's popup clip can span the viewport horizontally. Its center can
    // therefore be outside the popup's actual menu column.
    let clip = egui::Rect::from_min_size(egui::pos2(0., 200.), egui::vec2(1920., 110.));
    let option = egui::Rect::from_min_size(egui::pos2(505., 310.), egui::vec2(145., 36.));
    app.metrics
        .rest_controller
        .controls
        .push(("method:DELETE".into(), option, clip));
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920., 1080.),
            )),
            ..Default::default()
        },
        |ctx| {
            assert!(!app.rest_capture_click(ctx, "method:DELETE"));
        },
    );
    assert!(
        matches!(app.pending_events[0],egui::Event::PointerMoved(p) if p.x==option.center().x && p.y==clip.center().y && p.x>=option.left() && p.x<=option.right())
    );
    assert!(matches!(app.pending_events[1],egui::Event::MouseWheel {delta,..} if delta.y<0.));
    assert!(
        !app.pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::PointerButton { .. }))
    );
}
