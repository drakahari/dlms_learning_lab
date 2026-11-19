//! Fixed native disk-layout fixture; edits and actions use real UI events.
use super::*;
use dlms_learning_lab::{partition_fs as core, partition_fs_workspace as work};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic1-layout-log-growth", "lpic1-layout-media-policy"];
#[derive(Clone, Debug)]
enum Step {
    Style(f32, bool),
    Open(&'static str),
    Click(String),
    Field(&'static str, String),
    Choice(String),
    Rows(bool, bool),
    Fresh,
    WorkedDraft(usize),
    NoError,
    Save,
    Capture(String, String),
}
fn click(v: &str) -> Step {
    Step::Click(v.into())
}
fn sources() -> &'static Vec<pack::Scenario> {
    static S: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/boot-entry-lessons-v14.json"
        ))
        .unwrap()
        .scenarios
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
    })
}
fn operation(steps: &mut Vec<Step>, op: &work::Operation) {
    use work::Operation::*;
    let disk = match op {
        CreateGpt { disk } | AddPartition { disk, .. } => disk.clone(),
        Format { partition, .. } => partition
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .into(),
        _ => panic!("GPT fixture only"),
    };
    steps.extend([click("disk"), Step::Choice(disk)]);
    match op {
        CreateGpt { .. } => steps.push(click("create-gpt")),
        AddPartition {
            name,
            start,
            end,
            type_hint,
            ..
        } => steps.extend([
            Step::Field("name", name.clone()),
            Step::Field("start", start.to_string()),
            Step::Field("end", end.to_string()),
            click("hint"),
            Step::Choice(format!("hint-{type_hint:?}")),
            click("add"),
        ]),
        Format {
            partition,
            format,
            label,
            uuid,
        } => {
            let n: String = partition
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            steps.extend([
                click("partition"),
                Step::Choice(format!("partition-{n}")),
                click("content-type"),
                Step::Choice(format!("format-{format:?}")),
                Step::Field("label", label.clone()),
                Step::Field("uuid", uuid.clone().unwrap_or_default()),
                click("format"),
            ]);
        }
        _ => panic!("GPT fixture only"),
    }
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![];
        for (n, s) in sources().iter().enumerate() {
            let d = work::binding::source(s).unwrap();
            p.extend([
                Step::Style(1., false),
                Step::Open(IDS[n]),
                Step::Capture(format!("layout-{n}-introduction"), "top".into()),
                Step::Capture(format!("layout-{n}-worked-table"), "worked-end".into()),
                click("practice"),
                Step::Capture(format!("layout-{n}-practice-initial"), "goal".into()),
            ]);
            for (i, step) in d.lesson.worked.iter().enumerate() {
                p.extend([
                    click("example-picker"),
                    Step::Choice(format!("example-{i}")),
                    click("fill-example"),
                    Step::WorkedDraft(i),
                    click(match step.operation {
                        work::Operation::CreateGpt { .. } => "create-gpt",
                        work::Operation::AddPartition { .. } => "add",
                        work::Operation::Format { .. } => "format",
                        _ => panic!("GPT fixture only"),
                    }),
                    Step::NoError,
                ]);
            }
            p.extend([
                click("check"),
                Step::Rows(true, true),
                Step::Capture(format!("layout-{n}-practice-pass"), "results".into()),
            ]);
            for op in std::slice::from_ref(&d.lesson.experiment) {
                operation(&mut p, op);
            }
            p.extend([
                click("check"),
                Step::Rows(false, true),
                Step::Capture(
                    format!("layout-{n}-practice-growth-failure"),
                    "failure-last".into(),
                ),
                click("reset-practice"),
                click("assignment"),
                Step::Fresh,
                click("check"),
                Step::Rows(false, false),
                Step::Capture(format!("layout-{n}-assignment"), "goal".into()),
            ]);
            if n == 0 {
                let wrong = [
                    work::Operation::AddPartition {
                        disk: "/dev/vda".into(),
                        name: "logs".into(),
                        start: 2048,
                        end: 18431,
                        type_hint: core::TypeHint::LinuxData,
                    },
                    work::Operation::Format {
                        partition: "/dev/vda2".into(),
                        format: work::FormatChoice::Ext4,
                        label: "logs".into(),
                        uuid: None,
                    },
                ];
                for op in &wrong {
                    operation(&mut p, op);
                }
                p.extend([
                    click("check"),
                    Step::Rows(false, false),
                    Step::Capture("layout-0-wrong-adjacency".into(), "failure-last".into()),
                    click("reset"),
                    click("confirm-reset"),
                    click("assignment"),
                    Step::Fresh,
                ]);
            }
            for (i, op) in d.reference.iter().enumerate() {
                operation(&mut p, op);
                if i == 0 {
                    p.extend([click("check"), Step::Rows(false, false)]);
                }
            }
            p.extend([
                click("check"),
                Step::Rows(true, false),
                Step::Save,
                Step::Capture(format!("layout-{n}-assignment-pass"), "results".into()),
                Step::Style(1.5, false),
                Step::Capture(format!("layout-{n}-dark150-fields"), "field-name".into()),
                Step::Style(2., true),
                Step::Capture(format!("layout-{n}-light200-fields"), "field-name".into()),
                Step::Capture(format!("layout-{n}-light200-feedback"), "results".into()),
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
    reveals: u8,
}
fn target(app: &LabApp, ctx: &egui::Context, key: &str) -> Option<(egui::Rect, egui::Rect)> {
    let m = &app.metrics.partition_fs;
    let root = app.metrics.main_viewport.unwrap_or(ctx.content_rect());
    match key {
        "top" => Some((
            egui::Rect::from_min_size(root.min, egui::vec2(1., 1.)),
            root,
        )),
        "goal" => m.goal.map(|r| (r, root)),
        "failure-last" => m.rows.iter().rfind(|r| !r.0).map(|r| (r.1, root)),
        "reset" => app.metrics.reset.map(|r| (r, root)),
        "confirm-reset" => app.metrics.confirm_reset.map(|r| (r, ctx.content_rect())),
        _ => {
            if let Some(k) = key.strip_prefix("field-") {
                m.fields.iter().find(|r| r.0 == k).map(|r| (r.1, r.3))
            } else {
                m.controls
                    .iter()
                    .find(|r| r.0 == key)
                    .or_else(|| m.choices.iter().find(|r| r.0 == key))
                    .map(|r| (r.1, r.2))
            }
        }
    }
}
fn observe(app: &LabApp, pass: bool, practice: bool) {
    let s = &app.scenarios[app.selected.unwrap()];
    let a = &app.progress.attempts[&s.id];
    let engine::State::PartitionFs { simulation } = &a.state else {
        panic!()
    };
    let d = work::binding::source(s).unwrap();
    let rows = if practice {
        d.practice_checks(simulation)
    } else {
        d.checks(simulation)
    };
    assert_eq!(rows.iter().all(|r| r.0), pass, "{rows:?}");
    assert_eq!(progress::current_solved(a, s), pass && !practice);
    assert!(if practice {
        simulation.example_checked
    } else {
        simulation.checked
    });
}
impl LabApp {
    fn disk_layout_choice(&mut self, ctx: &egui::Context, key: &str, flow: &mut Flow) -> bool {
        let id = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(id)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let (rect, clip) = target(self, ctx, key).expect("Actual popup choice");
        let clip = clip.intersect(ctx.content_rect());
        if !smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            let tolerance = 1. / ctx.pixels_per_point();
            assert!(
                rect.left() >= clip.left() - tolerance && rect.right() <= clip.right() + tolerance,
                "Popup horizontally clipped"
            );
            assert!(flow.reveals < 20, "Bounded popup reveal failed");
            flow.reveals += 1;
            self.pending_events.extend([
                egui::Event::PointerMoved(egui::pos2(rect.center().x, clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (clip.center().y - rect.center().y)
                            .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                },
            ]);
            flow.next = ctx.input(|i| i.time) + 0.15;
            return false;
        }
        self.click_smoke_control(ctx, Some(rect))
    }
    fn disk_layout_reveal(&mut self, ctx: &egui::Context, rect: egui::Rect) -> bool {
        let before = self.pending_events.len();
        let done = self.reveal_smoke_control(ctx, rect);
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
    pub(in crate::ui) fn disk_layout_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("disk-layout-native-flow");
        let mut flow = ctx.data_mut(|d| d.get_temp::<Flow>(id)).unwrap_or_default();
        if flow.at == plan().len() {
            println!("DISK_LAYOUT_NATIVE_COMPLETE: {} actions", plan().len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 136000 + flow.at;
        let now = ctx.input(|i| i.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "Disk layout {} {:?} could not settle: {due:?}",
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

            Step::NoError => {
                assert!(
                    self.notice.is_none(),
                    "Native action refused: {:?}",
                    self.notice
                );
                true
            }
            Step::WorkedDraft(index) => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::State::PartitionFs { simulation: w } =
                    &self.progress.attempts[&s.id].state
                else {
                    panic!()
                };
                assert_eq!(w.selected_example, index);
                let expected = &work::binding::source(s).unwrap().lesson.worked[index].operation;
                let e = &w.example_editor;
                let draft = &e.drafts[&e.selected];
                let actual = match expected {
                    work::Operation::CreateGpt { .. } => work::Operation::CreateGpt {
                        disk: e.selected.clone(),
                    },
                    work::Operation::AddPartition { .. } => {
                        draft.add_operation(&e.selected).unwrap()
                    }
                    work::Operation::Format { .. } => draft.format_operation(&e.selected).unwrap(),
                    _ => panic!("GPT fixture only"),
                };
                assert_eq!(
                    &actual, expected,
                    "Worked fill changed only drafts; inspect exact selected disk and values"
                );
                true
            }
            Step::Open(id) => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                self.launch(self.scenarios.iter().position(|s| s.id == id).unwrap());
                true
            }
            Step::Click(key) => {
                self.click_visible_smoke_control(ctx, target(self, ctx, &key).map(|r| r.0))
            }
            Step::Choice(key) => self.disk_layout_choice(ctx, &key, &mut flow),
            Step::Field(key, text) => {
                if flow.phase == 0 {
                    if self.click_visible_smoke_control(
                        ctx,
                        target(self, ctx, &format!("field-{key}")).map(|r| r.0),
                    ) {
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
                        egui::Event::Paste(text.clone()),
                    ]);
                    flow.phase = 2;
                    false
                } else {
                    let s = &self.scenarios[self.selected.unwrap()];
                    let engine::State::PartitionFs { simulation: w } =
                        &self.progress.attempts[&s.id].state
                    else {
                        panic!()
                    };
                    let e = if w.view == teaching::View::Example {
                        &w.example_editor
                    } else {
                        &w.assignment_editor
                    };
                    let d = &e.drafts[&e.selected];
                    let actual = match key {
                        "name" => &d.name,
                        "start" => &d.start,
                        "end" => &d.end,
                        "label" => &d.label,
                        "uuid" => &d.uuid,
                        _ => panic!(),
                    };
                    assert_eq!(actual, &text);
                    true
                }
            }
            Step::Rows(pass, practice) => {
                observe(self, pass, practice);
                true
            }
            Step::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                let engine::State::PartitionFs { simulation: w } =
                    &self.progress.attempts[&s.id].state
                else {
                    panic!()
                };
                assert_eq!(w.current, work::binding::source(s).unwrap().initial);
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
            Step::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let saved = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(saved).unwrap(),
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
                    if target(self, ctx, &key).is_some_and(|r| self.disk_layout_reveal(ctx, r.0)) {
                        flow.phase = 1;
                        flow.next = now + 0.15;
                    }
                    false
                } else if flow.phase == 1
                    && !target(self, ctx, &key).is_some_and(|(r, clip)| {
                        smoke_control_visible(
                            r,
                            clip.intersect(self.metrics.main_viewport.unwrap())
                                .intersect(ctx.content_rect()),
                            ctx.pixels_per_point(),
                        )
                    })
                {
                    flow.phase = 0;
                    flow.reveals = 0;
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
            println!("DISK_LAYOUT_NATIVE_STEP {} {:?}", flow.at, plan()[flow.at]);
            flow.at += 1;
            flow.phase = 0;
            flow.reveals = 0;
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
fn fixture_reaches_both_native_sources_and_never_installs_outcomes() {
    let p = plan();
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Open(_))).count(), 2);
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Rows(true, true)))
            .count(),
        2
    );
    assert_eq!(
        p.iter()
            .filter(|s| matches!(s, Step::Rows(true, false)))
            .count(),
        2
    );
    assert_eq!(p.iter().filter(|s| matches!(s, Step::Save)).count(), 2);
    assert!(p.iter().filter(|s| matches!(s, Step::Field(..))).count() > 20);
}

#[cfg(test)]
#[test]
fn clipped_worked_choice_scrolls_its_popup_before_any_click() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    let clip = egui::Rect::from_min_max(egui::pos2(100., 100.), egui::pos2(400., 180.));
    let row = egui::Rect::from_min_max(egui::pos2(105., 185.), egui::pos2(395., 210.));
    app.metrics
        .partition_fs
        .choices
        .push(("example-5".into(), row, clip));
    let mut flow = Flow::default();
    assert!(!app.disk_layout_choice(&ctx, "example-5", &mut flow));
    assert_eq!(flow.reveals, 1);
    let point = app
        .pending_events
        .iter()
        .find_map(|e| {
            if let egui::Event::PointerMoved(p) = e {
                Some(*p)
            } else {
                None
            }
        })
        .unwrap();
    assert!(clip.contains(point));
    assert!(
        app.pending_events
            .iter()
            .any(|e| matches!(e,egui::Event::MouseWheel{delta,..}if delta.y<0.))
    );
    assert!(
        !app.pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::PointerButton { .. }))
    );
}
