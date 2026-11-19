//! Actual native actions for both wired-device assignments and separate practice.
use super::*;
use dlms_learning_lab::device_path_workspace::{Draft, Task};

const PATCH: &str = "ccna-device-path-patching";
const BOUNDARY: &str = "ccna-device-path-broadcast-boundary";

#[derive(Clone, Copy)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Assert(&'static str),
    Capture(&'static str, &'static str),
    Save,
}
const PLAN: &[Step] = {
    use Step::*;
    &[
        Style(1., false),
        Open(PATCH),
        Capture("device-patch-introduction", "top"),
        Click("example"),
        Capture("device-patch-practice-initial", "topology-start"),
        Click("check"),
        Assert("practice-fail"),
        Click("workstation-b"),
        Capture("device-patch-open-dropdown", "workstation-b-choice-5"),
        Click("workstation-b-choice-5"),
        Capture("device-patch-after-dropdown", "workstation-b"),
        Click("check"),
        Capture("device-patch-before-duplicate-assert", "result"),
        Assert("duplicate-jack"),
        Capture("device-patch-practice-duplicate", "result"),
        Click("server-b"),
        Click("server-b-choice-9"),
        Click("check"),
        Assert("practice-pass"),
        Capture("device-patch-practice-pass", "result"),
        Click("workstation-b"),
        Click("workstation-b-choice-8"),
        Assert("unchecked"),
        Click("check"),
        Assert("practice-pass"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("device-patch-assignment", "topology-start"),
        Click("workstation-b"),
        Click("workstation-b-choice-5"),
        Click("server-b"),
        Click("server-b-choice-9"),
        Click("check"),
        Assert("assignment-pass"),
        Capture("device-patch-assignment-pass", "result"),
        Click("monitor-up"),
        Assert("unchecked"),
        Click("check"),
        Assert("monitor-fail"),
        Capture("device-patch-monitor-failure", "result"),
        Click("monitor-up"),
        Click("workstation-b"),
        Click("workstation-b-choice-8"),
        Click("check"),
        Assert("assignment-pass"),
        Save,
        Style(1.5, false),
        Capture("device-patch-dark150", "check"),
        Style(2., true),
        Capture("device-patch-light200-topology", "topology-start"),
        Capture("device-patch-light200-topology-end", "topology-end"),
        Capture("device-patch-light200-wiring", "workstation-b"),
        Capture("device-patch-light200-result", "result"),
        Style(1., false),
        Open(BOUNDARY),
        Capture("device-boundary-introduction", "top"),
        Click("example"),
        Click("check"),
        Assert("broadcast-leak"),
        Capture("device-boundary-practice-leak", "result"),
        Click("bypass-up"),
        Click("check"),
        Assert("practice-pass"),
        Capture("device-boundary-practice-pass", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Click("check"),
        Assert("broadcast-leak"),
        Capture("device-boundary-assignment-leak", "result"),
        Click("bypass-up"),
        Assert("unchecked"),
        Click("check"),
        Assert("assignment-pass"),
        Capture("device-boundary-assignment-pass", "result"),
        Click("router-server-up"),
        Assert("unchecked"),
        Click("check"),
        Assert("remote-fail"),
        Capture("device-boundary-disconnected-router", "result"),
        Click("router-server-up"),
        Click("check"),
        Assert("assignment-pass"),
        Save,
        Style(1.5, false),
        Capture("device-boundary-dark150", "check"),
        Style(2., true),
        Capture("device-boundary-light200-wiring", "bypass-up"),
        Capture("device-boundary-light200-result", "result"),
        Save,
    ]
};

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
}

fn lesson_state(app: &LabApp) -> (&Draft, &Task) {
    let source = &app.scenarios[app.selected.unwrap()];
    let engine::State::DevicePath { simulation } = &app.progress.attempts[&source.id].state else {
        panic!()
    };
    let engine::Goal::DevicePath { definition } = &source.goal else {
        panic!()
    };
    if simulation.view == teaching::View::Example {
        (&simulation.practice, &definition.example)
    } else {
        (&simulation.assignment, &definition.initial)
    }
}
fn check_state(app: &LabApp, expected: &str) {
    let (draft, task) = lesson_state(app);
    let source = &app.scenarios[app.selected.unwrap()];
    let solved = progress::current_solved(&app.progress.attempts[&source.id], source);
    let outcomes = draft.checks(task);
    match expected {
        "practice-fail" => assert!(draft.checked && outcomes.iter().any(|row| !row.0) && !solved),
        "duplicate-jack" => {
            assert!(
                !solved && outcomes.iter().any(|row| !row.0 && row.1.contains("jack")),
                "outcomes={outcomes:?}, draft={draft:?}"
            )
        }
        "practice-pass" => assert!(draft.checked && outcomes.iter().all(|row| row.0) && !solved),
        "assignment-fresh" => assert_eq!(*draft, Draft::new(task)),
        "assignment-pass" => assert!(solved && outcomes.iter().all(|row| row.0)),
        "unchecked" => assert!(!draft.checked && !solved),
        "monitor-fail" => assert!(
            !solved
                && outcomes
                    .iter()
                    .any(|row| !row.0 && row.1.contains("monitor"))
                && outcomes.iter().any(|row| row.0)
        ),
        "broadcast-leak" => assert_eq!(
            outcomes.iter().map(|row| row.0).collect::<Vec<_>>(),
            [true, false, true, true]
        ),
        "remote-fail" => assert!(
            !solved
                && outcomes.iter().any(|row| row.0)
                && outcomes.iter().any(|row| !row.0 && row.1.contains("IPv4"))
        ),
        _ => panic!("Unknown device-path expectation"),
    }
}

fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    app.metrics
        .device_path
        .controls
        .iter()
        .find(|row| row.0 == key)
        .map(|row| row.1)
}

impl LabApp {
    fn device_path_popup_click(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let click_id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|data| data.get_temp::<u8>(click_id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (_, rect, clip) = self
            .metrics
            .device_path
            .controls
            .iter()
            .find(|row| row.0 == key)
            .expect("open cable popup option");
        let rect = *rect;
        let visible = clip.intersect(ctx.content_rect());
        if smoke_control_visible(rect, visible, ctx.pixels_per_point()) {
            return self.click_smoke_control(ctx, Some(rect));
        }
        let id = egui::Id::new(("bounded-cable-popup-scroll", self.smoke_stage));
        let attempts = ctx.data_mut(|data| {
            let next = data.get_temp::<usize>(id).unwrap_or(0) + 1;
            data.insert_temp(id, next);
            next
        });
        assert!(
            attempts <= 20,
            "Cable popup option remains clipped: {rect:?} in {visible:?}"
        );
        self.pending_events.extend([
            egui::Event::PointerMoved(visible.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (visible.center().y - rect.center().y).clamp(-300., 300.),
                ),
                modifiers: Default::default(),
            },
        ]);
        false
    }
    pub(in crate::ui) fn device_path_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("device-path-native-flow");
        let mut flow = ctx
            .data_mut(|data| data.get_temp::<Flow>(id))
            .unwrap_or_default();
        if flow.at == PLAN.len() {
            println!("DEVICE_PATH_NATIVE_COMPLETE: {} actions", PLAN.len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 134000 + flow.at;
        let now = ctx.input(|input| input.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "device-path action {} could not settle: {due:?}",
            flow.at
        );
        if !due.unwrap() {
            ctx.data_mut(|data| data.insert_temp(id, flow));
            ctx.request_repaint_after(std::time::Duration::from_millis(25));
            return;
        }
        let done = match PLAN[flow.at] {
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
                let index = self
                    .scenarios
                    .iter()
                    .position(|scenario| scenario.id == id)
                    .unwrap();
                self.launch(index);
                true
            }
            Step::Click(key) => {
                if key.contains("-choice-") {
                    self.device_path_popup_click(ctx, key)
                } else {
                    self.click_visible_smoke_control(ctx, target(self, key))
                }
            }
            Step::Assert(expected) => {
                check_state(self, expected);
                true
            }
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    if key == "top"
                        || target(self, key).is_some_and(|r| self.reveal_smoke_control(ctx, r))
                    {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
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
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(loaded).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
        };
        if done {
            println!("DEVICE_PATH_NATIVE_STEP {}", flow.at);
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
        ctx.data_mut(|data| data.insert_temp(id, flow));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
