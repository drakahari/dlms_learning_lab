//! Closed normal-catalog scheduling lessons, driven only through actual widgets.
//! Development profiles and screenshots are private; no grade/model is installed.
use super::nat::{Op as GlobalOp, Queue, Target as Global, Warm, exact};
use super::*;
use dlms_learning_lab::{cron as core, cron_workspace as work, script_teaching::View};
use serde::Deserialize;
use serde_json::Value;
pub(in crate::ui) const FIRST: usize = 61000;
const END: usize = 62000;
pub(in crate::ui) const IDS: [&str; 4] = [
    "lpic1-cron-weekday-report",
    "lpic1-cron-environment-output",
    "lpic1-cron-access-retirement",
    "lpic1-cron-calendar-handover",
];
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wrong {
    name: String,
    actions: Vec<work::ReferenceAction>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    reference: Vec<work::ReferenceAction>,
    alternatives: Vec<Vec<work::ReferenceAction>>,
    wrong: Vec<Wrong>,
    try_action: Vec<work::ReferenceAction>,
    try_cases: Vec<bool>,
    repair: Vec<work::ReferenceAction>,
}
fn reviews() -> &'static Vec<Review> {
    static REVIEW: std::sync::OnceLock<Vec<Review>> = std::sync::OnceLock::new();
    REVIEW.get_or_init(|| {
        let r: Vec<Review> = serde_json::from_str(include_str!(
            "../../../tests/fixtures/cron-lessons-review-v1.json"
        ))
        .unwrap();
        assert_eq!(r.len(), 4);
        for (r, id) in r.iter().zip(IDS) {
            assert_eq!(r.id, id);
            assert_eq!(r.alternatives.len(), 2);
            assert_eq!(r.try_cases.len(), 16);
            assert!(!r.try_action.is_empty() && !r.repair.is_empty());
            for wrong in &r.wrong {
                assert!(!wrong.name.is_empty() && !wrong.actions.is_empty());
            }
        }
        r
    })
}
#[path = "../../../tests/support/cron_lessons.rs"]
mod fixture;
pub(in crate::ui) fn fixtures() -> Vec<Scenario> {
    fixture::lessons()
}
#[derive(Clone, Debug)]
enum Target {
    Button(&'static str),
    Field(&'static str),
    Choice(String),
    Row(usize),
    Output,
    Failure,
    Heading,
    Card(bool),
    Back,
    Reset,
    Cancel,
    Confirm,
    Challenge,
    Learn,
    UnitPicker,
    UnitChoice,
    NextUnit,
    UnitComplete,
}
#[derive(Clone, Debug)]
enum Op {
    Click(Target),
    Text(&'static str, String),
    Select(&'static str, String),
}
#[derive(Clone, Debug)]
enum Step {
    Select(usize),
    Next(usize),
    Click(Target),
    Open,
    BeforePractice,
    BeforeFill,
    FillOnly,
    PracticeExact,
    Action(work::ReferenceAction, bool),
    Check(bool, bool),
    Rows(Vec<bool>, bool),
    Snapshot,
    ResetExact,
    CancelExact,
    Warm,
    Back,
    ResumeExact,
    Theme(bool),
    Geometry(f32, bool),
    Shot(String, Target),
    LongWindow,
    Running,
    RawDraft,
    RepairDraft,
    Guided(work::Action),
    Pending,
    Global(Global),
    Blocked,
    GuardExact,
    Finish,
}
fn action_steps(p: &mut Vec<Step>, a: &[work::ReferenceAction], practice: bool) {
    for a in a {
        p.push(Step::Action(a.clone(), practice));
    }
}
fn check(p: &mut Vec<Step>, pass: bool, practice: bool) {
    p.push(Step::Click(Target::Button("check")));
    p.push(Step::Check(pass, practice));
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Snapshot,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::ResetExact,
        Step::Click(Target::Button("assignment")),
    ]);
}
fn rows(value: &Value) -> Vec<bool> {
    serde_json::from_value(value.clone()).unwrap()
}
fn plan() -> &'static Vec<Step> {
    static PLAN: std::sync::OnceLock<Vec<Step>> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| {
        let sources = fixtures();
        let vectors: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/cron-lessons-rows-v1.json"
        ))
        .unwrap();
        let mut p = Vec::new();
        for (n, (s, r)) in sources.iter().zip(reviews()).enumerate() {
            let name = |part: &str| format!("cron-{n}-{part}");
            p.extend([
                Step::Select(n),
                Step::Click(Target::Card(true)),
                Step::Click(Target::Card(false)),
                Step::Open,
                Step::Shot(name("introduction"), Target::Heading),
                Step::Click(Target::Button("try-example")),
                Step::BeforePractice,
                Step::BeforeFill,
                Step::Click(Target::Button("fill")),
                Step::FillOnly,
            ]);
            let lesson = s.cron_teaching.as_ref().unwrap();
            // Every annotated example command still requires its actual editor,
            // install/remove and Run controls. Fill is checked separately above.
            for a in &lesson.example_actions {
                p.push(Step::Guided(a.action.clone()));
            }
            check(&mut p, true, true);
            p.push(Step::Shot(name("example-output"), Target::Output));
            for action in &lesson.try_actions {
                p.push(Step::Guided(action.clone()));
            }
            check(&mut p, lesson.try_rows.iter().all(|b| *b), true);
            p.push(Step::Rows(lesson.try_rows[1..].to_vec(), true));
            for action in &lesson.repair {
                p.push(Step::Guided(action.clone()));
            }
            check(&mut p, true, true);
            p.extend([
                Step::PracticeExact,
                Step::Warm,
                Step::Click(Target::Button("assignment")),
            ]);
            p.push(Step::Click(Target::Button("check")));
            p.push(Step::Rows(rows(&vectors["rows"][n]["initial"]), false));
            // A plausible wrong schedule is installed and actually run. Fresh
            // calendar/input cases, not a hand-set model, expose the error.
            action_steps(&mut p, &r.wrong[0].actions, false);
            check(&mut p, false, false);
            p.push(Step::Shot(name("wrong-feedback"), Target::Failure));
            reset(&mut p);
            if n == 0 {
                p.extend([
                    Step::LongWindow,
                    Step::Snapshot,
                    Step::Click(Target::Button("run")),
                    Step::Running,
                    Step::Click(Target::Button("stop")),
                    Step::CancelExact,
                ]);
            }
            let split = r
                .reference
                .iter()
                .position(|a| matches!(a, work::ReferenceAction::Run { .. }))
                .unwrap();
            action_steps(&mut p, &r.reference[..split], false);
            check(&mut p, false, false);
            action_steps(&mut p, &r.reference[split..], false);
            check(&mut p, true, false);
            p.extend([
                Step::Shot(name("assignment-output"), Target::Output),
                Step::Shot(name("objectives-first"), Target::Row(0)),
                Step::Click(Target::Button("cases")),
                Step::Shot(name("objectives-last"), Target::Row(usize::MAX)),
                Step::Warm,
                Step::Snapshot,
                Step::Global(Global::Settings),
                Step::Blocked,
                Step::GuardExact,
                Step::Global(Global::SettingsClose),
                Step::GuardExact,
                Step::RawDraft,
                Step::Pending,
                Step::Warm,
                Step::Back,
                Step::Click(Target::Card(false)),
                Step::ResumeExact,
            ]);
            // Repair only the actual previously retained draft; installed
            // tables and the already elapsed virtual clock stay unchanged.
            p.push(Step::RepairDraft);
            check(&mut p, true, false);
            for (alt, a) in r.alternatives.iter().enumerate() {
                reset(&mut p);
                action_steps(&mut p, a, false);
                check(&mut p, true, false);
                p.push(Step::Shot(
                    name(&format!("alternative-{alt}")),
                    Target::Output,
                ));
            }
            p.extend([
                Step::Click(Target::Challenge),
                Step::Shot(name("challenge-full-task"), Target::Heading),
                Step::Click(Target::Learn),
                Step::Snapshot,
                Step::Click(Target::Reset),
                Step::Click(Target::Cancel),
                Step::CancelExact,
            ]);
            // Switching mode deliberately clears presentation feedback. Ask for
            // an actual fresh Check again before using its completion navigation.
            check(&mut p, true, false);
            // A read-only alignment sweep uses the actual completed task, not
            // a special hidden renderer. Every supported text-size is observed.
            if n == 3 {
                p.push(Step::Snapshot);
                for narrow in [false, true] {
                    for light in [false, true] {
                        for scale in [1., 1.15, 1.25, 1.5, 2.] {
                            p.extend([
                                Step::Theme(light),
                                Step::Geometry(scale, narrow),
                                Step::Click(Target::Field("document")),
                                Step::Shot(
                                    format!(
                                        "cron-alignment-{}-{}-{}-editor",
                                        if narrow { "narrow" } else { "wide" },
                                        if light { "light" } else { "dark" },
                                        (scale * 100f32).round() as u32
                                    ),
                                    Target::Field("document"),
                                ),
                                Step::Shot(
                                    format!(
                                        "cron-alignment-{}-{}-{}-output",
                                        if narrow { "narrow" } else { "wide" },
                                        if light { "light" } else { "dark" },
                                        (scale * 100f32).round() as u32
                                    ),
                                    Target::Output,
                                ),
                            ]);
                        }
                    }
                }
                p.push(Step::GuardExact);
            }
            p.push(Step::Warm);
            if n < 3 {
                p.push(Step::Next(n + 1));
            } else {
                p.push(Step::Shot(
                    "cron-unit-complete".into(),
                    Target::UnitComplete,
                ));
            }
            p.push(Step::Back);
        }
        p.push(Step::Finish);
        assert!(p.len() < 1000, "Cron controller actualsteps={}", p.len());
        assert!(p.iter().filter(|s| matches!(s, Step::Shot(..))).count() <= 128);
        p
    })
}
pub(in crate::ui) fn immediate_stop(stage: usize) -> bool {
    (FIRST..END).contains(&stage)
        && plan().get(stage - FIRST).is_some_and(|s| {
            matches!(
                s,
                Step::LongWindow
                    | Step::Running
                    | Step::Snapshot
                    | Step::CancelExact
                    | Step::Click(Target::Button("run" | "stop"))
            )
        })
}
pub(in crate::ui) fn phase_bounds() -> (usize, usize) {
    (FIRST, FIRST + plan().len() - 1)
}
fn key(name: &str) -> egui::Id {
    egui::Id::new(("cron-native", name))
}
fn ctrl(k: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: k,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    }
}
fn learning(a: &Attempt) -> &dlms_learning_lab::cron_teaching::Learning {
    a.workspace
        .as_ref()
        .unwrap()
        .cron_learning
        .as_ref()
        .unwrap()
}
fn session(a: &Attempt, practice: bool) -> &work::Session {
    if practice {
        &learning(a).session
    } else {
        a.workspace.as_ref().unwrap().cron_session.as_ref().unwrap()
    }
}
fn state(a: &Attempt, practice: bool) -> &core::State {
    if practice {
        &learning(a).simulation
    } else {
        let engine::State::Cron { simulation } = &a.state else {
            unreachable!()
        };
        simulation
    }
}
fn portable(a: &LabApp) -> Value {
    serde_json::to_value(&a.progress).unwrap()
}
fn receipts(a: &LabApp) -> Value {
    serde_json::to_value(&a.progress.migration_receipts).unwrap()
}
impl LabApp {
    pub(in crate::ui) fn cron_lessons_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("cron-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn cron_lessons_contains_stage(&self, n: usize) -> bool {
        (FIRST..END).contains(&n) && n <= phase_bounds().1
    }
    fn cr_put<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, name: &str, v: T) {
        ctx.data_mut(|d| d.insert_temp(key(name), v));
    }
    fn cr_get<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, name: &str) -> T {
        ctx.data(|d| d.get_temp(key(name)))
            .expect("Prior actual native observation")
    }
    fn cr_scope(&self) {
        self.nat_private();
        assert_eq!((self.scenarios.len(), self.units.len()), (164, 42));
        assert_eq!(
            self.scenarios[160..164]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        assert!(
            self.progress.attempts.len() <= 4
                && self
                    .progress
                    .attempts
                    .keys()
                    .all(|id| IDS.contains(&id.as_str()))
        );
        assert!(!self.synthetic);
    }
    fn cr_target(&self, ctx: &egui::Context, t: &Target) -> (egui::Rect, egui::Rect, bool) {
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let m = &self.metrics.cron;
        match t {
            Target::UnitChoice => {
                let r = self
                    .metrics
                    .unit_choices
                    .iter()
                    .find(|(id, _)| id == "linux-cron-foundations")
                    .expect("Real course choice")
                    .1;
                (
                    r,
                    self.metrics
                        .unit_popup
                        .expect("Actual unit popup clip")
                        .intersect(ctx.content_rect()),
                    true,
                )
            }
            Target::Choice(k) => {
                let r = m.choices.iter().find(|r| &r.0 == k).unwrap();
                (
                    r.1,
                    m.popup.as_ref().unwrap().1.intersect(ctx.content_rect()),
                    true,
                )
            }
            Target::Field(k) => {
                let r = m.fields.iter().find(|r| r.0 == *k).unwrap();
                (r.1, r.4.intersect(root), false)
            }
            Target::Output => {
                let (r, c) = m.output.expect("Actual virtual-file viewer");
                (r, c.intersect(root), true)
            }
            Target::Failure => {
                let r = m
                    .rows
                    .iter()
                    .find(|r| !r.1)
                    .expect("A current meaningful failing objective");
                (r.2, root, false)
            }
            Target::Row(n) => {
                let r = if *n == usize::MAX {
                    m.rows.last().unwrap()
                } else {
                    m.rows.iter().find(|r| r.0 == *n).unwrap()
                };
                (r.2, root, false)
            }
            _ => {
                let r = match t {
                    Target::Button(k) => m.controls.iter().find(|r| r.0 == *k).map(|r| r.1),
                    Target::Heading => m.labels.first().map(|r| r.1),
                    Target::Card(f) => {
                        assert_eq!(self.metrics.card_actions.len(), 1);
                        let r = self.metrics.card_actions[0];
                        Some(if *f { r.1 } else { r.0 })
                    }
                    Target::Back => self.metrics.back,
                    Target::Reset => self.metrics.reset,
                    Target::Cancel => self.metrics.reset_cancel,
                    Target::Confirm => self.metrics.confirm_reset,
                    Target::Challenge => self.metrics.mode_challenge,
                    Target::Learn => self.metrics.mode_learn,
                    Target::UnitPicker => self.metrics.unit_picker,
                    Target::NextUnit => self.metrics.next_unit,
                    Target::UnitComplete => self.metrics.unit_complete,
                    _ => unreachable!(),
                }
                .expect("Actual painted Cron control");
                (r, root, false)
            }
        }
    }
    fn cr_reveal(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        let (r, v, inner) = self.cr_target(ctx, t);
        let k = egui::Id::new(("cron-reveal", self.smoke_stage));
        let mut q = ctx.data(|d| d.get_temp::<Queue>(k)).unwrap_or_default();
        let done = self.nat_reveal(ctx, r, v, inner, &mut q);
        ctx.data_mut(|d| d.insert_temp(k, q));
        done
    }
    fn cr_click(&mut self, ctx: &egui::Context, t: &Target) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(t, Target::Choice(_) | Target::UnitChoice) {
            let (r, v, _) = self.cr_target(ctx, t);
            return self.mac_popup_choice(ctx, Some(r), Some(v));
        }
        if !self.cr_reveal(ctx, t) {
            return false;
        }
        let (r, _, _) = self.cr_target(ctx, t);
        self.click_smoke_control(ctx, Some(r))
    }
    fn cr_text(&mut self, ctx: &egui::Context, k: &str, text: &str) {
        let r = self.metrics.cron.fields.iter().find(|r| r.0 == k).unwrap();
        assert!(r.3 && ctx.memory(|m| m.has_focus(r.2)));
        assert!(text.len() <= if k == "document" { 8192 } else { 32 });
        self.pending_events
            .extend([ctrl(egui::Key::A, true), ctrl(egui::Key::A, false)]);
        if text.is_empty() {
            for pressed in [true, false] {
                self.pending_events.push(egui::Event::Key {
                    key: egui::Key::Backspace,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
        } else {
            self.pending_events.push(egui::Event::Paste(text.into()));
        }
    }
    fn cr_op(&mut self, ctx: &egui::Context, op: &Op, sub: &mut u8) -> bool {
        match op {
            Op::Click(t) => self.cr_click(ctx, t),
            Op::Text(k, text) => match *sub {
                0 => {
                    if self.cr_click(ctx, &Target::Field(k)) {
                        *sub = 1
                    }
                    false
                }
                1 => {
                    self.cr_text(ctx, k, text);
                    *sub = 2;
                    false
                }
                _ => true,
            },
            Op::Select(k, value) => match *sub {
                0 => {
                    if self.cr_click(ctx, &Target::Button(k)) {
                        *sub = 1
                    }
                    false
                }
                1 => {
                    if self.cr_click(ctx, &Target::Choice(format!("{k}:{value}"))) {
                        *sub = 2
                    }
                    false
                }
                _ => {
                    let a = &self.progress.attempts[&self.scenarios[self.selected.unwrap()].id];
                    let t = session(a, learning(a).view == View::Example && !a.challenge);
                    if *k == "actor" {
                        assert_eq!(&t.actor, value)
                    } else if *k == "target" {
                        assert_eq!(t.selected.key(), *value)
                    }
                    true
                }
            },
        }
    }
    fn cr_ops(&mut self, ctx: &egui::Context, ops: &[Op]) -> bool {
        let k = egui::Id::new(("cron-operation", self.smoke_stage));
        let (mut n, mut sub, mut ticks) = ctx
            .data(|d| d.get_temp::<(usize, u8, u16)>(k))
            .unwrap_or_default();
        ticks += 1;
        assert!(ticks <= 300 && ops.len() <= 16);
        if n >= ops.len() {
            return true;
        }
        let done = self.cr_op(ctx, &ops[n], &mut sub);
        if done {
            n += 1;
            sub = 0;
            ctx.data_mut(|d| {
                d.remove::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)));
            });
        }
        ctx.data_mut(|d| d.insert_temp(k, (n, sub, ticks)));
        self.started = Instant::now();
        ctx.request_repaint();
        n >= ops.len()
    }
    fn cr_wait(&self, ctx: &egui::Context, scale: f32, narrow: bool) -> bool {
        let sample = (
            ctx.content_rect(),
            ctx.pixels_per_point(),
            ctx.zoom_factor(),
            ctx.input(|i| i.viewport().fullscreen),
        );
        let px = sample.0.size() * sample.1;
        let expected = (sample.2 - scale).abs() < 0.01
            && sample.3 == Some(!narrow)
            && if narrow {
                (650. ..=1300.).contains(&px.x)
            } else {
                px.x >= 1500.
            };
        let k = egui::Id::new(("cron-geometry", self.smoke_stage));
        let ready = ctx.data_mut(|d| {
            let (old, n) = d
                .get_temp::<(Option<(egui::Rect, f32, f32, Option<bool>)>, u8)>(k)
                .unwrap_or((None, 0));
            if expected && old == Some(sample) {
                return true;
            }
            assert!(n < 60, "Cron measured geometry did not settle: {sample:?}");
            d.insert_temp(k, (Some(sample), n + 1));
            false
        });
        if !ready {
            ctx.request_repaint()
        }
        ready
    }
    fn cr_warm(&mut self, ctx: &egui::Context) {
        let portable = portable(self);
        let caller_receipts = receipts(self);
        self.save();
        assert!(!self.dirty && self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap();
        let mut projection: Value = serde_json::from_slice(&raw).unwrap();
        let raw_receipts = projection
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .unwrap();
        exact(&projection, &portable, "Warm disk complete portable fields");
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        exact(
            &loaded,
            &portable,
            "Normal loader complete readback without installing Progress",
        );
        exact(
            &loaded.migration_receipts,
            &raw_receipts,
            "Readback receipt bytes",
        );
        exact(
            &self.progress.migration_receipts,
            &caller_receipts,
            "Save caller receipts unchanged",
        );
        assert_eq!(
            progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
            raw
        );
        self.cr_put(
            ctx,
            "warm",
            Warm {
                raw,
                portable,
                raw_receipts,
                caller_receipts,
            },
        );
    }
    fn cr_choose_unit(&mut self, ctx: &egui::Context) -> bool {
        let k = key("unit-choice-phase");
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        if phase < 2 {
            let target = if phase == 0 {
                Target::UnitPicker
            } else {
                Target::UnitChoice
            };
            if !self.cr_click(ctx, &target) {
                return false;
            }
            ctx.data_mut(|d| d.insert_temp(k, phase + 1));
            ctx.request_repaint();
            return false;
        }
        assert_eq!(self.filter.unit, "linux-cron-foundations");
        assert_eq!(
            self.metrics.card_actions.len(),
            4,
            "Actual course selection exposes all four normal cards"
        );
        assert!(
            self.progress.attempts.is_empty(),
            "Choosing a unit creates no work or credit"
        );
        true
    }
    pub(in crate::ui) fn cron_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        self.cr_scope();
        let p = plan();
        let offset = stage - FIRST;
        assert!(offset < p.len());
        let step = &p[offset];
        let n = p[..=offset]
            .iter()
            .rev()
            .find_map(|s| {
                if let Step::Select(n) = s {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap();
        let id = IDS[n];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let s = self.scenarios[i].clone();
        let k = egui::Id::new(("cron-step-ticks", stage));
        let ticks = ctx.data_mut(|d| {
            let n = d.get_temp::<u16>(k).unwrap_or(0) + 1;
            d.insert_temp(k, n);
            n
        });
        assert!(
            ticks <= 300,
            "Cron stage{stage}/{step:?} exceeded bounded event/continuation wait"
        );
        match step {
            Step::Select(n) => {
                assert!(self.selected.is_none());
                assert_eq!(
                    self.progress.attempts.len(),
                    if *n == 0 { 0 } else { *n + 1 }
                );
                if *n == 0 && !self.cr_choose_unit(ctx) {
                    return (false, None);
                }
                assert_eq!(self.filter.unit, "linux-cron-foundations");
                self.filter.search = s.title.clone();
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.cr_put(ctx, "narrow", false);
            }
            Step::Next(next) => {
                let prior = key("next-prior");
                if ctx.data(|d| d.get_temp::<Value>(prior)).is_none() {
                    assert_eq!(self.selected, Some(i));
                    assert!(self.metrics.completion);
                    assert!(!self.progress.attempts.contains_key(IDS[*next]));
                    self.cr_put(ctx, "next-prior", portable(self));
                    self.cr_put(ctx, "next-receipts", receipts(self));
                }
                if !self.cr_click(ctx, &Target::NextUnit) {
                    return (false, None);
                }
                let mut expected: Value = self.cr_get(ctx, "next-prior");
                let clock = expected["clock"].as_u64().unwrap() + 1;
                let target = self.scenarios.iter().find(|s| s.id == IDS[*next]).unwrap();
                let mut fresh = Attempt::new(target, clock);
                work::budget::prepare(&mut fresh, target).unwrap();
                expected["clock"] = clock.into();
                expected["attempts"][IDS[*next]] = serde_json::to_value(fresh).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Actual Next creates only the fresh next assignment and genuine recency",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.cr_get::<Value>(ctx, "next-receipts"),
                    "Actual Next retains all migration receipts",
                );
                assert_eq!(
                    self.selected,
                    self.scenarios.iter().position(|s| s.id == IDS[*next])
                );
                assert_eq!(
                    learning(&self.progress.attempts[IDS[*next]]).view,
                    View::Introduction
                );
                ctx.data_mut(|d| {
                    d.remove::<Value>(prior);
                });
            }
            Step::Click(t) => {
                if !self.cr_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Open => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(
                    learning(&self.progress.attempts[id]).view,
                    View::Introduction
                );
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!self.metrics.completion);
            }
            Step::BeforePractice => {
                assert_eq!(learning(&self.progress.attempts[id]).view, View::Example);
                self.cr_put(ctx, "practice-before", portable(self));
                self.cr_put(ctx, "practice-receipts", receipts(self));
            }
            Step::BeforeFill => {
                self.cr_put(ctx, "fill-before", portable(self));
                self.cr_put(ctx, "fill-receipts", receipts(self));
            }
            Step::FillOnly => {
                let mut expected: Value = self.cr_get(ctx, "fill-before");
                let lesson = s.cron_teaching.as_ref().unwrap();
                let work::Action::Actor { actor } = &lesson.example_actions[0].action else {
                    panic!("Closed four fixture first actions are explicit actor fills")
                };
                expected["attempts"][id]["workspace"]["cron_learning"]["session"]["actor"] =
                    actor.clone().into();
                exact(
                    &self.progress,
                    &expected,
                    "First Fill changes exactly the separate example actor, with no draft install, virtual run, history or assignment change",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.cr_get::<Value>(ctx, "fill-receipts"),
                    "Fill retains every receipt",
                );
                assert!(!self.metrics.completion);
            }
            Step::PracticeExact => {
                let mut expected: Value = self.cr_get(ctx, "practice-before");
                expected["attempts"][id]["workspace"]["cron_learning"] =
                    serde_json::to_value(learning(&self.progress.attempts[id])).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "All example actions preserve assignment and every other progress field",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.cr_get::<Value>(ctx, "practice-receipts"),
                    "Practice receipt isolation",
                );
            }
            Step::Action(action, practice) => {
                if ctx.data(|d| d.get_temp::<usize>(key("action-owner"))) != Some(stage) {
                    self.cr_put(ctx, "action-before", portable(self));
                    self.cr_put(ctx, "action-receipts", receipts(self));
                    self.cr_put(ctx, "action-owner", stage);
                }
                let ops = match action {
                    work::ReferenceAction::Install {
                        actor,
                        target,
                        text,
                    } => vec![
                        Op::Select("actor", actor.clone()),
                        Op::Select("target", target.key()),
                        Op::Text("document", text.clone()),
                        Op::Click(Target::Button("install")),
                    ],
                    work::ReferenceAction::Remove { actor, target } => vec![
                        Op::Select("actor", actor.clone()),
                        Op::Select("target", target.key()),
                        Op::Click(Target::Button("remove")),
                    ],
                    work::ReferenceAction::Run { end } => vec![
                        Op::Text("end", end.text()),
                        Op::Click(Target::Button("run")),
                    ],
                };
                if !self.cr_ops(ctx, &ops) {
                    return (false, None);
                }
                if self.cron_presentation.running() {
                    ctx.request_repaint();
                    return (false, None);
                }
                let a = &self.progress.attempts[id];
                let t = session(a, *practice);
                let state = state(a, *practice);
                match action {
                    work::ReferenceAction::Install { target, text, .. } => {
                        assert_eq!(t.selected, *target);
                        assert_eq!(&t.drafts[&target.key()], text);
                        if matches!(target, work::Document::Allow | work::Document::Deny) {
                            assert!(t.applied(
                                if *practice {
                                    &s.cron_teaching.as_ref().unwrap().definition
                                } else {
                                    let engine::Goal::Cron { definition } = &s.goal else {
                                        unreachable!()
                                    };
                                    definition
                                },
                                state
                            ));
                        } else {
                            assert_eq!(target.installed(state), *text)
                        }
                    }
                    work::ReferenceAction::Remove { target, .. } => assert!(!target.exists(state)),
                    work::ReferenceAction::Run { end } => {
                        assert_eq!(state.clock, *end);
                        let log = t.last_run.as_ref().expect("Actual completed Run records");
                        assert_eq!(log.end, *end);
                        assert!(log.events.iter().all(|r| r.when > log.start
                            && r.when <= log.end
                            && state.users.contains(&r.owner)));
                    }
                }
                let mut expected: Value = self.cr_get(ctx, "action-before");
                if *practice {
                    expected["attempts"][id]["workspace"]["cron_learning"] =
                        serde_json::to_value(learning(a)).unwrap();
                } else {
                    expected["attempts"][id]["state"] = serde_json::to_value(&a.state).unwrap();
                    expected["attempts"][id]["workspace"]["cron_session"] =
                        serde_json::to_value(session(a, false)).unwrap();
                }
                exact(
                    &self.progress,
                    &expected,
                    "Actual action only changes selected model/session or its separate practice; all raw fields, other work and global history remain exact",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.cr_get::<Value>(ctx, "action-receipts"),
                    "Actual action receipt isolation",
                );
            }
            Step::Check(pass, practice) => {
                let a = &self.progress.attempts[id];
                if self.cron_presentation.memo.pending(a, &s) {
                    ctx.request_repaint();
                    return (false, None);
                }
                let actual = self
                    .cron_presentation
                    .memo
                    .rows(a, &s, *practice)
                    .expect("Actual continuation rows");
                assert_eq!(
                    actual.iter().all(|r| r.0),
                    *pass,
                    "{id} current assignment/practice={practice}: {actual:?}"
                );
                assert_eq!(self.metrics.completion, !*practice && *pass);
                if !practice {
                    assert_eq!(progress::current_solved(a, &s), *pass);
                }
                assert!(!actual.is_empty());
            }
            Step::Rows(expected, practice) => {
                let a = &self.progress.attempts[id];
                if self.cron_presentation.memo.pending(a, &s) {
                    ctx.request_repaint();
                    return (false, None);
                }
                let actual = self.cron_presentation.memo.rows(a, &s, *practice).unwrap();
                assert_eq!(
                    actual.iter().skip(1).map(|r| r.0).collect::<Vec<_>>(),
                    *expected
                );
            }
            Step::Snapshot => {
                self.cr_put(ctx, "snapshot", portable(self));
                self.cr_put(ctx, "snapshot-receipts", receipts(self));
            }
            Step::ResetExact => {
                let mut expected: Value = self.cr_get(ctx, "snapshot");
                let a = &self.progress.attempts[id];
                assert!(!self.reset_pending);
                assert_eq!(self.selected, Some(i));
                assert_eq!(a.state, s.initial);
                assert!(!a.challenge && a.step == 0 && a.hints == 0 && a.transcript.is_empty());
                assert_eq!(
                    a.visited,
                    expected["attempts"][id]["visited"].as_u64().unwrap()
                );
                let d = work::context(a, &s).unwrap();
                assert_eq!(session(a, false), &work::Session::new(d).unwrap());
                let lesson = s.cron_teaching.as_ref().unwrap();
                assert_eq!(
                    learning(a),
                    &dlms_learning_lab::cron_teaching::Learning::new(lesson).unwrap()
                );
                expected["attempts"][id] = serde_json::to_value(a).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Only selected attempt resets, old recency and all other progress exact",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.cr_get::<Value>(ctx, "snapshot-receipts"),
                    "Reset retains full receipts",
                );
            }
            Step::CancelExact | Step::GuardExact => {
                exact(
                    &self.progress,
                    &self.cr_get::<Value>(ctx, "snapshot"),
                    "Cancel/modal retains entire Progress",
                );
                exact(
                    &self.progress.migration_receipts,
                    &self.cr_get::<Value>(ctx, "snapshot-receipts"),
                    "Cancel/modal retains full receipts",
                );
            }
            Step::Warm => self.cr_warm(ctx),
            Step::Back => {
                if !self.cr_click(ctx, &Target::Back) {
                    return (false, None);
                }
            }
            Step::ResumeExact => {
                let warm: Warm = self.cr_get(ctx, "warm");
                let mut expected = warm.portable;
                assert_eq!(self.selected, Some(i));
                assert_eq!(self.progress.clock, expected["clock"].as_u64().unwrap() + 1);
                expected["clock"] = self.progress.clock.into();
                expected["attempts"][id]["visited"] = self.progress.clock.into();
                exact(
                    &self.progress,
                    &expected,
                    "Resume only changes genuine recency, retaining assignment/example drafts and run logs",
                );
                exact(
                    &self.progress.migration_receipts,
                    &warm.caller_receipts,
                    "Resume receipt isolation",
                );
                let disk: Value = serde_json::from_slice(&warm.raw).unwrap();
                exact(
                    &disk["migration_receipts"],
                    &warm.raw_receipts,
                    "Warm raw receipts still pinned",
                );
            }
            Step::LongWindow => {
                let end = state(&self.progress.attempts[id], false)
                    .clock
                    .after(7 * 24 * 60)
                    .unwrap();
                if !self.cr_ops(ctx, &[Op::Text("end", end.text())]) {
                    return (false, None);
                }
            }
            Step::Running => {
                assert!(
                    self.cron_presentation.running(),
                    "The long-window Stop probe must actually be active"
                );
            }
            Step::RawDraft => {
                if ctx
                    .data(|d| d.get_temp::<String>(key("draft-before")))
                    .is_none()
                {
                    let t = session(&self.progress.attempts[id], false);
                    self.cr_put(ctx, "draft-before", t.drafts[&t.selected.key()].clone());
                    self.cr_put(ctx, "draft-target", t.selected.clone());
                }
                if !self.cr_ops(
                    ctx,
                    &[Op::Text(
                        "document",
                        "# unfinished scheduling edit\n".into(),
                    )],
                ) {
                    return (false, None);
                }
            }
            Step::RepairDraft => {
                let text: String = self.cr_get(ctx, "draft-before");
                let target: work::Document = self.cr_get(ctx, "draft-target");
                if !self.cr_ops(
                    ctx,
                    &[
                        Op::Select("target", target.key()),
                        Op::Text("document", text),
                    ],
                ) {
                    return (false, None);
                }
                ctx.data_mut(|d| d.remove::<String>(key("draft-before")));
            }
            Step::Guided(action) => {
                let ops = match action {
                    work::Action::Actor { actor } => vec![Op::Select("actor", actor.clone())],
                    work::Action::Select { target } => vec![Op::Select("target", target.key())],
                    work::Action::Edit { target, text } => vec![
                        Op::Select("target", target.key()),
                        Op::Text("document", text.clone()),
                    ],
                    work::Action::Install { target } => vec![
                        Op::Select("target", target.key()),
                        Op::Click(Target::Button("install")),
                    ],
                    work::Action::Remove { target } => vec![
                        Op::Select("target", target.key()),
                        Op::Click(Target::Button("remove")),
                    ],
                    work::Action::Run { end } => vec![
                        Op::Text("end", end.text()),
                        Op::Click(Target::Button("run")),
                    ],
                };
                if !self.cr_ops(ctx, &ops) {
                    return (false, None);
                }
                if self.cron_presentation.running() {
                    ctx.request_repaint();
                    return (false, None);
                }
                if let work::Action::Run { end } = action {
                    let a = &self.progress.attempts[id];
                    assert_eq!(state(a, true).clock, *end);
                    assert_eq!(
                        session(a, true)
                            .last_run
                            .as_ref()
                            .expect("Real practice run history")
                            .end,
                        *end
                    );
                }
            }
            Step::Pending => {
                let a = &self.progress.attempts[id];
                assert!(
                    session(a, false)
                        .drafts
                        .values()
                        .any(|d| d == "# unfinished scheduling edit\n")
                );
                assert!(!self.metrics.completion);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Global(t) => {
                if !self.nat_queue_for(ctx, vec![GlobalOp::Click(t.clone())], id, false) {
                    return (false, None);
                }
            }
            Step::Blocked => {
                assert!(self.settings_open);
                assert!(
                    self.metrics.cron.fields.iter().all(|r| !r.3)
                        && self.metrics.cron.controls.iter().all(|r| !r.2)
                );
                self.pending_events.extend([
                    egui::Event::Text("blocked cron text".into()),
                    ctrl(egui::Key::Enter, true),
                    ctrl(egui::Key::Enter, false),
                ]);
            }
            Step::Theme(light) => {
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
            }
            Step::Geometry(scale, narrow) => {
                self.cr_put(ctx, "narrow", *narrow);
                self.scale = *scale;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                if *narrow {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)))
                }
                if !self.cr_wait(ctx, *scale, *narrow) {
                    return (false, None);
                }
            }
            Step::Shot(name, target) => {
                let narrow = ctx
                    .data(|d| d.get_temp::<bool>(key("narrow")))
                    .unwrap_or(false);
                if !self.cr_wait(ctx, self.scale, narrow) {
                    return (false, None);
                }
                if !self.cr_reveal(ctx, target) {
                    return (false, None);
                }
                let (r, v, _) = self.cr_target(ctx, target);
                assert!(smoke_control_visible(r, v, ctx.pixels_per_point()));
                if let Target::Field(k) = target {
                    let f = self.metrics.cron.fields.iter().find(|r| r.0 == *k).unwrap();
                    assert!(f.3 && ctx.memory(|m| m.has_focus(f.2)));
                    assert!(
                        !session(&self.progress.attempts[id], false).drafts
                            [&session(&self.progress.attempts[id], false).selected.key()]
                            .is_empty()
                    );
                }
                let content = ctx.content_rect();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                println!(
                    "Cron {id}/{name} actual_content={content:?} PPP={ppp} zoom={zoom}; requested size is not measured geometry"
                );
                return (true, Some(format!("{name}-zoom{zoom:.2}-ppp{ppp:.2}")));
            }
            Step::Finish => {
                if ctx.data(|d| d.get_temp::<bool>(key("finished"))) == Some(true) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                self.cr_warm(ctx);
                println!(
                    "PASS native four bundled Cron lessons: actual unit picker/four cards/Next/unit completion/library/intro/separate practice, editor/install/remove/virtual Run and Stop-independent fresh checks, plausible wrong/partial/reference/two alternate schedules, stale-success invalidation, reset/cancel/resume/full progress+receipt readback, all five text sizes light/dark wide/narrow. Control-flow completion does not itself establish rendered alignment, novice or screen-reader acceptance."
                );
                ctx.data_mut(|d| d.insert_temp(key("finished"), true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}
