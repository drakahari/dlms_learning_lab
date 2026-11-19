//! Fixed native checks for the authored copied-journal recovery lesson.
use super::*;
use dlms_learning_lab::journal_archive_workspace::{Draft, Task};

const ID: &str = "lpic1-copied-journal-incident";
const PRACTICE: &str = "journalctl -D /recovery/workshop-journal -b b38463bf5c7d4561a112cadbd6b0c042 -u reader.service -p warning -o cat";
const ASSIGNMENT: &str = "journalctl -D /recovery/field-journal -b a437ebc8d4f449d8bc1efa49dbb3e24f -u backup.service -p err -o cat";
const WRONG_LEVEL: &str = "journalctl -D /recovery/field-journal -b a437ebc8d4f449d8bc1efa49dbb3e24f -u backup.service -p warning -o cat";
const WRONG_BOOT: &str = "journalctl -D /recovery/field-journal -b d78b985ead2f405cbe3614069e88ac04 -u backup.service -p err -o cat";

#[derive(Clone, Copy)]
enum Step {
    Style(f32, bool),
    Open,
    Click(&'static str),
    Edit(&'static str, &'static str),
    Assert(&'static str),
    Capture(&'static str, &'static str),
    Save,
}

const PLAN: &[Step] = {
    use Step::*;
    &[
        Style(1., false),
        Open,
        Capture("copied-journal-introduction", "top"),
        Click("example"),
        Edit(
            "input",
            "journalctl -D /recovery/workshop-journal --list-boots",
        ),
        Click("run"),
        Assert("practice-list"),
        Capture("copied-journal-practice-boots", "input"),
        Edit("input", PRACTICE),
        Click("run"),
        Click("check"),
        Assert("practice-pass"),
        Capture("copied-journal-practice-evidence", "result"),
        Click("assignment"),
        Assert("assignment-fresh"),
        Capture("copied-journal-assignment", "input"),
        Edit(
            "input",
            "journalctl -D /recovery/field-journal --list-boots",
        ),
        Click("run"),
        Edit("input", WRONG_LEVEL),
        Click("run"),
        Click("check"),
        Assert("wrong-level"),
        Capture("copied-journal-wrong-level", "check"),
        Edit("input", ASSIGNMENT),
        Assert("pending"),
        Click("run"),
        Click("check"),
        Assert("assignment-pass"),
        Capture("copied-journal-assignment-pass", "result"),
        Edit("input", WRONG_BOOT),
        Assert("pending"),
        Click("run"),
        Click("check"),
        Assert("wrong-boot"),
        Capture("copied-journal-wrong-boot", "check"),
        Edit("input", ASSIGNMENT),
        Click("run"),
        Click("check"),
        Assert("assignment-pass"),
        Style(1.5, false),
        Capture("copied-journal-dark150-console", "input"),
        Style(2., true),
        Capture("copied-journal-light200-console", "input"),
        Capture("copied-journal-light200-result", "result"),
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
}

fn lesson_state(app: &LabApp) -> (&Draft, &Task) {
    let engine::State::CopiedJournal { simulation } = &app.progress.attempts[ID].state else {
        panic!()
    };
    let engine::Goal::CopiedJournal { definition } = &app.scenarios[app.selected.unwrap()].goal
    else {
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
    let solved = progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[app.selected.unwrap()],
    );
    match expected {
        "practice-list" => {
            let output = draft.output(task).unwrap().unwrap();
            assert!(output.contains("b38463bf5c7d4561a112cadbd6b0c042"));
            assert!(!solved);
        }
        "practice-pass" => {
            let output = draft.output(task).unwrap().unwrap();
            assert!(output.contains("Input stream stopped responding"));
            assert!(output.contains("Reader gave up"));
            assert!(draft.checks(task).iter().all(|row| row.0));
            assert!(!solved);
        }
        "assignment-fresh" => assert_eq!(*draft, Draft::default()),
        "pending" => {
            assert!(!draft.checked && !draft.ran_current && !solved);
            assert!(draft.output(task).unwrap().is_none());
        }
        "assignment-pass" => assert!(solved),
        "wrong-level" => {
            assert!(!solved);
            assert!(
                draft
                    .output(task)
                    .unwrap()
                    .unwrap()
                    .contains("Retrying storage access")
            );
            assert!(
                draft
                    .checks(task)
                    .iter()
                    .any(|(ok, message)| !ok && message.contains("err"))
            );
        }
        "wrong-boot" => {
            assert!(!solved);
            assert!(
                draft
                    .output(task)
                    .unwrap()
                    .unwrap()
                    .contains("-- No entries --")
            );
            assert!(
                draft
                    .checks(task)
                    .iter()
                    .any(|(ok, message)| !ok && message.contains("boot"))
            );
        }
        _ => panic!("Unknown copied-journal observation"),
    }
}

fn target(app: &LabApp, key: &str) -> Option<egui::Rect> {
    app.metrics
        .copied_journal
        .controls
        .iter()
        .find(|row| row.0 == key)
        .map(|row| row.1)
}

impl LabApp {
    pub(in crate::ui) fn copied_journal_tick(&mut self, ctx: &egui::Context) {
        let id = egui::Id::new("copied-journal-native-flow");
        let mut flow = ctx
            .data_mut(|data| data.get_temp::<Flow>(id))
            .unwrap_or_default();
        if flow.at == PLAN.len() {
            println!("COPIED_JOURNAL_NATIVE_COMPLETE: {} actions", PLAN.len());
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        self.smoke_stage = 132000 + flow.at;
        let now = ctx.input(|input| input.time);
        let started = flow.started.get_or_insert_with(Instant::now);
        let due = native_action_due(&mut flow.polls, now, flow.next, started.elapsed());
        assert!(
            due.is_ok(),
            "copied-journal action {} could not settle: {due:?}",
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
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1268., 1308.,
                        )));
                    }
                    self.lesson_scroll_reset = true;
                    flow.phase = 1;
                    false
                } else {
                    let viewport = ctx.content_rect();
                    let ppp = ctx.pixels_per_point();
                    let proper_size = !narrow
                        || ((viewport.width() * ppp - 1268.).abs() < 4.
                            && (viewport.height() * ppp - 1308.).abs() < 4.);
                    let settled = flow.viewport == Some(viewport)
                        && proper_size
                        && (ctx.zoom_factor() - scale).abs() < 0.01;
                    flow.viewport = Some(viewport);
                    settled
                }
            }
            Step::Open => {
                assert_eq!((self.scenarios.len(), self.units.len()), (402, 154));
                let index = self
                    .scenarios
                    .iter()
                    .position(|scenario| scenario.id == ID)
                    .unwrap();
                self.launch(index);
                true
            }
            Step::Click(key) => self.click_visible_smoke_control(ctx, target(self, key)),
            Step::Edit(key, text) => {
                if flow.phase == 0 {
                    if self.click_visible_smoke_control(ctx, target(self, key)) {
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
                        egui::Event::Paste(text.into()),
                    ]);
                    flow.phase = 2;
                    false
                } else {
                    let (draft, _) = lesson_state(self);
                    let actual = &draft.command;
                    assert_eq!(actual, text);
                    true
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
            println!("COPIED_JOURNAL_NATIVE_STEP {}", flow.at);
            flow.at += 1;
            flow.phase = 0;
            flow.polls = 0;
            flow.started = None;
            flow.viewport = None;
            flow.next = now + 0.025;
        } else {
            flow.next = flow.next.max(now + 0.025);
        }
        ctx.data_mut(|data| data.insert_temp(id, flow));
        ctx.request_repaint_after(std::time::Duration::from_millis(25));
    }
}
