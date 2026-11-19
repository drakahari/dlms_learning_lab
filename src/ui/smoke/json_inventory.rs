//! Fixed native JSON editor checks; never a learner execution route.
use super::*;
use dlms_learning_lab::json_inventory_workspace::{Draft, Task};
const FIRST: &str = "ccna-json-port-records";
const SECOND: &str = "ccna-json-inventory-handover";
#[derive(Clone, Copy)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(&'static str),
    Edit(&'static str),
    Assert(&'static str),
    Capture(&'static str, &'static str),
    Save,
    KeyboardApply,
}
const PLAN: &[Step] = {
    use Step::*;
    &[
        Style(1., false),
        Open(FIRST),
        Capture("json-records-introduction", "top"),
        Click("example"),
        Capture("json-records-practice-initial", "editor"),
        Click("apply"),
        Assert("type-error"),
        Capture("json-records-type-feedback", "notice"),
        Edit("syntax"),
        Click("apply"),
        Assert("syntax-error"),
        Capture("json-records-syntax-feedback", "notice"),
        Edit("partial"),
        Click("apply"),
        Click("check"),
        Assert("partial"),
        Capture("json-records-partial-network", "result"),
        Edit("reference"),
        KeyboardApply,
        Click("check"),
        Assert("practice-pass"),
        Capture("json-records-practice-pass", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("json-records-assignment", "editor"),
        Edit("alternate"),
        Click("check"),
        Assert("pending"),
        Click("apply"),
        Click("check"),
        Assert("assignment-pass"),
        Capture("json-records-assignment-pass", "result"),
        Edit("down"),
        Assert("unchecked"),
        Click("apply"),
        Click("check"),
        Assert("down"),
        Capture("json-records-down-feedback", "result"),
        Edit("alternate"),
        Click("apply"),
        Click("check"),
        Assert("assignment-pass"),
        Save,
        Style(1.5, false),
        Capture("json-records-dark150-editor", "editor"),
        Style(2., true),
        Capture("json-records-light200-editor", "editor"),
        Capture("json-records-light200-result", "result"),
        Style(1., false),
        Open(SECOND),
        Capture("json-nested-introduction", "top"),
        Click("example"),
        Edit("partial"),
        Click("apply"),
        Click("check"),
        Assert("partial"),
        Capture("json-nested-practice-partial", "result"),
        Edit("reference"),
        Click("apply"),
        Click("check"),
        Assert("practice-pass"),
        Capture("json-nested-practice-pass", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("json-nested-assignment", "editor"),
        Edit("alternate"),
        Click("apply"),
        Click("check"),
        Assert("assignment-pass"),
        Capture("json-nested-assignment-pass", "result"),
        Edit("wrong-monitor"),
        Click("apply"),
        Click("check"),
        Assert("wrong-monitor"),
        Capture("json-nested-monitor-feedback", "result"),
        Edit("reference"),
        Click("apply"),
        Click("check"),
        Assert("assignment-pass"),
        Save,
        Style(1.5, false),
        Capture("json-nested-dark150", "result"),
        Style(2., true),
        Capture("json-nested-light200-editor", "editor"),
        Capture("json-nested-light200-result", "result"),
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
    let engine::State::JsonInventory { simulation } = &app.progress.attempts[&source.id].state
    else {
        panic!()
    };
    let engine::Goal::JsonInventory { definition } = &source.goal else {
        panic!()
    };
    if simulation.view == teaching::View::Example {
        (&simulation.practice, &definition.example)
    } else {
        (&simulation.assignment, &definition.initial)
    }
}
fn edit_text(task: &Task, kind: &str) -> String {
    if kind == "syntax" {
        return "{ \"switches\": [".into();
    }
    if kind == "partial" {
        return task.initial_applied.clone();
    }
    let mut doc: serde_json::Value = serde_json::from_str(&task.reference).unwrap();
    match kind {
        "reference" => {}
        "alternate" => {
            doc["note"] = serde_json::json!("alternate record order and staff/guest assignment");
            let switches = doc["switches"].as_array_mut().unwrap();
            switches.reverse();
            for sw in switches {
                let ports = sw["ports"].as_array_mut().unwrap();
                ports.reverse();
                for port in ports {
                    match port["vlan"].as_u64().unwrap() {
                        10 => port["vlan"] = 20.into(),
                        20 => port["vlan"] = 10.into(),
                        _ => {}
                    }
                }
            }
        }
        "down" => doc["switches"][0]["ports"][0]["up"] = false.into(),
        "wrong-monitor" => {
            for sw in doc["switches"].as_array_mut().unwrap() {
                sw["ports"][2]["vlan"] = 10.into();
            }
        }
        _ => panic!("unknown JSON fixture"),
    }
    serde_json::to_string_pretty(&doc).unwrap()
}
fn check_state(app: &LabApp, expected: &str) {
    let (draft, task) = lesson_state(app);
    let source = &app.scenarios[app.selected.unwrap()];
    let solved = progress::current_solved(&app.progress.attempts[&source.id], source);
    let checks = draft.checks(task);
    match expected {
        "type-error" => assert!(
            app.notice
                .as_ref()
                .unwrap()
                .contains("Document field error")
                && !draft.checked
                && !solved
        ),
        "syntax-error" => assert!(
            app.notice.as_ref().unwrap().contains("JSON syntax") && !draft.checked && !solved
        ),
        "partial" => assert!(
            draft.checked && checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0) && !solved
        ),
        "practice-pass" => assert!(draft.checked && checks.iter().all(|c| c.0) && !solved),
        "assignment-fresh" => assert_eq!(*draft, Draft::new(task)),
        "pending" => {
            assert!(!draft.checked && !solved && app.notice.as_ref().unwrap().contains("Apply"))
        }
        "unchecked" => assert!(!draft.checked && !solved),
        "assignment-pass" => assert!(solved && checks.iter().all(|c| c.0)),
        "down" => assert!(!solved && checks.iter().any(|c| !c.0 && c.1.contains("down"))),
        "wrong-monitor" => assert!(!solved && checks.iter().any(|c| !c.0 && c.1.contains("99"))),
        _ => panic!("Unknown JSON observation"),
    }
}
fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    app.metrics
        .json_inventory
        .controls
        .iter()
        .find(|r| r.0 == key)
        .map(|r| r.1)
}
impl LabApp {
    fn json_inventory_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
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
            // A wheel inside the JSON editor scrolls its text, not the lesson.
            // Use the actual parent gutter, retaining all existing guard limits.
            for event in &mut self.pending_events[before..] {
                if let egui::Event::PointerMoved(point) = event {
                    *point = gutter.center();
                }
            }
        }
        done
    }
    pub(in crate::ui) fn open_json_inventory_capture(&mut self, id: &str) {
        assert_eq!((self.scenarios.len(), self.units.len()), CURRENT_CATALOG_SIZE);
        assert!(matches!(id, FIRST | SECOND));
        let index = self
            .scenarios
            .iter()
            .position(|scenario| scenario.id == id)
            .unwrap();
        self.launch(index);
    }
    pub(in crate::ui) fn json_inventory_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("json-inventory-native-flow");
        let mut flow = ctx
            .data_mut(|data| data.get_temp::<Flow>(id))
            .unwrap_or_default();
        if flow.at == PLAN.len() {
            println!("JSON_INVENTORY_NATIVE_COMPLETE: {} actions", PLAN.len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 134000 + flow.at;
        let now = ctx.input(|input| input.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "JSON action {} could not settle: {due:?}",
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
                self.open_json_inventory_capture(id);
                true
            }
            Step::Click(key) => self.click_visible_smoke_control(ctx, target(self, key)),
            Step::Edit(kind) => {
                let text = edit_text(lesson_state(self).1, kind);
                if flow.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, "editor")) {
                        flow.phase = 1;
                    }
                    false
                } else if flow.phase == 1 {
                    self.pending_events.extend([
                        egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers {
                                command: true,
                                ..egui::Modifiers::CTRL
                            },
                        },
                        egui::Event::Paste(text),
                    ]);
                    flow.phase = 2;
                    false
                } else {
                    assert_eq!(lesson_state(self).0.source, text);
                    true
                }
            }
            Step::Assert(expected) => {
                check_state(self, expected);
                true
            }
            Step::Capture(name, key) => {
                if flow.phase == 0 {
                    // Capture navigation must not compete with a keyboard
                    // target retained from the earlier editing exercise.
                    ctx.memory_mut(|memory| {
                        if let Some(focused) = memory.focused() {
                            memory.surrender_focus(focused);
                        }
                    });
                    if key == "top"
                        || target(self, key).is_some_and(|r| self.json_inventory_reveal(ctx, r))
                    {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1 && key != "top" && !capture_visible(self, ctx, key) {
                    // The viewport may have moved during the settlement delay.
                    // Preserve the original reveal/deadline/visibility limits.
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
            Step::KeyboardApply => {
                let apply = self
                    .metrics
                    .json_inventory
                    .ids
                    .iter()
                    .find(|r| r.0 == "apply")
                    .unwrap()
                    .1;
                let key = |key, pressed| egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                };
                if flow.phase == 100 {
                    assert_eq!(lesson_state(self).0.source, lesson_state(self).0.applied);
                    true
                } else if ctx.memory(|memory| memory.focused()) == Some(apply) {
                    self.pending_events
                        .extend([key(egui::Key::Enter, true), key(egui::Key::Enter, false)]);
                    flow.phase = 100;
                    false
                } else {
                    assert!(
                        flow.phase < 5,
                        "Tab did not reach Apply from the actual editor"
                    );
                    self.pending_events
                        .extend([key(egui::Key::Tab, true), key(egui::Key::Tab, false)]);
                    flow.phase += 1;
                    false
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
            println!("JSON_INVENTORY_NATIVE_STEP {}", flow.at);
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

#[cfg(test)]
#[test]
fn keyboard_apply_is_scheduled_only_after_editing_not_after_saving() {
    let steps: Vec<_> = PLAN
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, Step::KeyboardApply))
        .collect();
    assert_eq!(steps.len(), 1);
    assert!(matches!(PLAN[steps[0].0 - 1], Step::Edit("reference")));
    assert_eq!(
        PLAN.iter()
            .filter(|step| matches!(step, Step::Save))
            .count(),
        2
    );
}

fn capture_visible(app: &LabApp, ctx: &egui::Context, key: &str) -> bool {
    let Some((_, rect, clip)) = app
        .metrics
        .json_inventory
        .controls
        .iter()
        .find(|r| r.0 == key)
    else {
        return false;
    };
    let visible = clip
        .intersect(app.metrics.main_viewport.expect("actual main viewport"))
        .intersect(ctx.content_rect());
    smoke_control_visible(*rect, visible, ctx.pixels_per_point())
}
#[cfg(test)]
pub(in crate::ui) fn test_late_editor_capture(app: &mut LabApp, ctx: &egui::Context) -> bool {
    let id = egui::Id::new("json-inventory-native-flow");
    let at = PLAN
        .iter()
        .position(|step| matches!(step, Step::Capture("json-records-light200-editor", _)))
        .unwrap();
    ctx.data_mut(|data| {
        data.insert_temp(
            id,
            Flow {
                at,
                phase: 1,
                ..Default::default()
            },
        )
    });
    assert!(
        !capture_visible(app, ctx, "editor"),
        "test must reproduce a clipped editor"
    );
    app.json_inventory_tick(ctx);
    let deferred =
        !app.capture_pending && ctx.data(|data| data.get_temp::<Flow>(id)).unwrap().phase == 0;
    let before = app.pending_events.len();
    assert!(!app.json_inventory_reveal(ctx, target(app, "editor").unwrap()));
    let point = app.pending_events[before..]
        .iter()
        .find_map(|event| {
            if let egui::Event::PointerMoved(point) = event {
                Some(*point)
            } else {
                None
            }
        })
        .expect("actual queued wheel pointer");
    assert!(
        point.x > app.metrics.main_viewport.unwrap().right()
            && app.metrics.main_scroll_region.unwrap().contains(point),
        "Wheel must target the parent gutter, outside the nested editor"
    );
    deferred
}
