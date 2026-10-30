//! Closed native acceptance over the four real OneOff assignments. This driver
//! sends native events only; it never installs learner state, grades or focus.
use super::*;
use dlms_learning_lab::{one_off_workspace as work, script_teaching::View};
use serde_json::Value;
use std::collections::BTreeMap;
pub(in crate::ui) const FIRST: usize = 73000;
const END: usize = 74000;
pub(in crate::ui) const IDS: [&str; 4] = [
    "lpic1-at-report",
    "lpic1-at-capture-repair",
    "lpic1-at-owner-queue",
    "lpic1-at-handover",
];
const UNIT: &str = "linux-one-off-jobs";
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Control(String),
    Field(String),
    Choice(String),
    UnitPicker,
    UnitChoice,
    Card(usize, bool),
    Back,
    Reset,
    Cancel,
    Confirm,
    Next,
    Challenge,
    Learn,
    Goal(bool),
    Intro,
    Output,
    Result,
    Scale,
    Theme,
    HeaderChoice(String),
}
#[derive(Clone, Debug)]
enum Step {
    Library,
    Open(usize),
    Click(Target),
    Fill(String, String),
    ExampleBefore,
    Example(usize),
    ExampleExact,
    Saved(String, String),
    Write(String, String),
    Command(String),
    Env(BTreeMap<String, String>),
    CommandBefore(bool),
    CommandExact(String, bool),
    WaitRun,
    Check(Vec<bool>, bool),
    Invalidated,
    Snapshot,
    ResetLesson,
    CancelExact,
    ResetExact,
    Warm,
    ResumeExact,
    StopActive,
    StopExact,
    StopIdle,
    Viewport(bool),
    Settled(f32, bool, bool),
    WorkspaceGeometry,
    Shot(String, Target),
    InspectCaptured,
    OwnerBefore,
    OwnerDenied,
    Keyboard,
    ChallengeExact,
    UnitComplete,
    Finish,
}
fn control(name: &str) -> Step {
    Step::Click(Target::Control(name.into()))
}
fn field(name: &str, text: &str) -> Step {
    Step::Fill(name.into(), text.into())
}
fn choice(name: &str) -> Step {
    Step::Click(Target::Choice(name.into()))
}
fn sources() -> &'static Vec<Scenario> {
    static SOURCES: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    SOURCES.get_or_init(|| {
        let all = pack::bundled().unwrap();
        assert_eq!(
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all).len(),
            182
        );
        assert_eq!(one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&all).len(),186);
        assert_eq!(one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&all).len(),190);
        assert_eq!(all.len(),193);
        assert_eq!(
            all[176..180]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        all.into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn review() -> &'static Vec<Value> {
    static REVIEW: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    REVIEW.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/one-off-lessons-review-v1.json"
        ))
        .unwrap()
    })
}
fn flags(v: &Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
fn native_flags(v: &Value) -> Vec<bool> {
    let mut b = vec![true];
    b.extend(flags(v));
    b
}
fn check(p: &mut Vec<Step>, rows: Vec<bool>, practice: bool) {
    p.extend([control("check"), Step::Check(rows, practice)]);
}
fn reset_steps() -> [Step; 8] {
    [
        Step::Snapshot,
        Step::Click(Target::Reset),
        Step::Click(Target::Cancel),
        Step::CancelExact,
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::ResetExact,
        control("assignment"),
    ]
}
fn reset(p: &mut Vec<Step>) {
    p.push(Step::ResetLesson);
}

fn actor(p: &mut Vec<Step>, name: &str) {
    p.extend([control("actor"), choice(&format!("actor:{name}"))]);
}
fn command(p: &mut Vec<Step>, text: &str) {
    p.push(Step::Command(text.into()));
}

fn actions(p: &mut Vec<Step>, rows: &Value) {
    for a in rows.as_array().unwrap() {
        match a["kind"].as_str().unwrap() {
            "Write" => {
                p.push(Step::Write(
                    a["path"].as_str().unwrap().into(),
                    a["text"].as_str().unwrap().into(),
                ));
            }
            "Edit" => {
                let path = a["path"].as_str().unwrap();
                p.extend([
                    control("target"),
                    choice(&format!("target:{path}")),
                    field("document", a["text"].as_str().unwrap()),
                ]);
            }
            "Command" => {
                if let Some(who) = a["actor"].as_str() {
                    actor(p, who);
                }
                if a.get("environment").is_some() {
                    p.push(Step::Env(
                        serde_json::from_value(a["environment"].clone()).unwrap(),
                    ));
                }
                command(p, a["text"].as_str().unwrap());
            }
            "Run" => {
                let end: dlms_learning_lab::cron::Moment =
                    serde_json::from_value(a["end"].clone()).unwrap();
                p.extend([field("end", &end.text()), control("run"), Step::WaitRun]);
            }
            _ => panic!("Unreviewed native authoring action"),
        }
    }
}
fn shot(p: &mut Vec<Step>, n: usize, name: &str, target: Target) {
    p.push(Step::Shot(format!("one-off-{n}-{name}"), target));
}
fn plan() -> &'static Vec<Step> {
    static PLAN: std::sync::OnceLock<Vec<Step>> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = vec![
            Step::Library,
            Step::Click(Target::UnitPicker),
            Step::Click(Target::UnitChoice),
            Step::Click(Target::Card(0, true)),
            Step::Click(Target::Card(0, false)),
        ];
        for n in 0..4 {
            let s = &sources()[n];
            let l = s.one_off_teaching.as_ref().unwrap();
            let r = &review()[n];
            p.push(Step::Open(n));
            shot(&mut p, n, "introduction", Target::Intro);
            p.extend([Step::ExampleBefore, control("try-example")]);
            let mut example_drafts = l.definition.initial.files.clone();
            for (i, a) in l.example_actions.iter().enumerate() {
                if let work::Action::Edit { path, text } = &a.action {
                    example_drafts.insert(path.clone(), text.clone());
                }
                p.push(Step::Example(i));
                match &a.action {
                    work::Action::Save { path } => p.extend([
                        control("save"),
                        Step::Saved(path.clone(), example_drafts[path].clone()),
                    ]),
                    work::Action::Command { text } => p.extend([
                        Step::CommandBefore(true),
                        control("command-run"),
                        Step::CommandExact(text.clone(), true),
                    ]),
                    work::Action::Run { .. } => p.extend([control("run"), Step::WaitRun]),
                    _ => {}
                }
            }
            check(&mut p, vec![true; l.try_rows.len()], true);
            p.push(Step::ExampleExact);
            shot(&mut p, n, "example-verified-ungraded", Target::Result);
            actions(&mut p, &serde_json::to_value(&l.try_actions).unwrap());
            check(&mut p, l.try_rows.clone(), true);
            shot(&mut p, n, "example-unsaved-feedback", Target::Result);
            actions(&mut p, &serde_json::to_value(&l.repair).unwrap());
            check(&mut p, vec![true; l.try_rows.len()], true);
            p.push(Step::ExampleExact);
            p.push(control("assignment"));
            shot(&mut p, n, "full-assignment", Target::Goal(false));
            check(&mut p, native_flags(&r["initial_rows"]), false);
            if n == 2 {
                actor(&mut p, "pat");
                p.extend([
                    field("command", "at -c 1"),
                    Step::OwnerBefore,
                    control("command-run"),
                    Step::OwnerDenied,
                ]);
                reset(&mut p);
            }
            actions(&mut p, &r["hardcoded"]["actions"]);
            check(&mut p, native_flags(&r["hardcoded"]["rows"]), false);
            shot(&mut p, n, "wrong-specific-feedback", Target::Result);
            reset(&mut p);
            actions(&mut p, &r["partial"]["actions"]);
            check(&mut p, native_flags(&r["partial"]["rows"]), false);
            reset(&mut p);
            actions(&mut p, &r["reference"]);
            check(
                &mut p,
                vec![true; 1 + r["initial_rows"].as_array().unwrap().len()],
                false,
            );
            shot(&mut p, n, "reference-verified", Target::Result);
            if n == 1 {
                reset(&mut p);
                actor(&mut p, "devon");
                p.push(Step::Env(BTreeMap::from([(
                    "LABEL".into(),
                    "Final".into(),
                )])));
                command(&mut p, "at -c 1");
                p.push(Step::InspectCaptured);
                shot(&mut p, n, "captured-variables", Target::Output);
                reset(&mut p);
                actions(&mut p, &r["reference"]);
                check(
                    &mut p,
                    vec![true; 1 + r["initial_rows"].as_array().unwrap().len()],
                    false,
                );
            }
            actions(&mut p, &r["damage"]["actions"]);
            p.push(Step::Invalidated);
            check(&mut p, native_flags(&r["damage"]["rows"]), false);
            for alt in r["alternatives"].as_array().unwrap() {
                reset(&mut p);
                actions(&mut p, alt);
                check(
                    &mut p,
                    vec![true; 1 + r["initial_rows"].as_array().unwrap().len()],
                    false,
                );
            }
            p.extend([Step::Click(Target::Challenge), Step::ChallengeExact]);
            shot(&mut p, n, "challenge-task", Target::Goal(false));
            p.push(Step::Click(Target::Learn));
            check(
                &mut p,
                vec![true; 1 + r["initial_rows"].as_array().unwrap().len()],
                false,
            );
            if n == 0 {
                p.extend([
                    Step::Keyboard,
                    Step::Warm,
                    Step::Click(Target::Back),
                    Step::Click(Target::Card(0, false)),
                    Step::ResumeExact,
                ]);
                check(
                    &mut p,
                    vec![true; 1 + r["initial_rows"].as_array().unwrap().len()],
                    false,
                );
                for scale in [1., 1.15, 1.25, 1.5, 2.] {
                    for dark in [true, false] {
                        for narrow in [true, false] {
                            p.extend([
                                Step::Click(Target::Scale),
                                Step::Click(Target::HeaderChoice(format!("scale:{scale:.2}"))),
                                Step::Click(Target::Theme),
                                Step::Click(Target::HeaderChoice(format!(
                                    "theme:{}",
                                    if dark { "dlms-dark" } else { "dlms-light" }
                                ))),
                                Step::Viewport(narrow),
                                Step::Settled(scale, narrow, dark),
                            ]);
                            p.push(Step::WorkspaceGeometry);
                            shot(
                                &mut p,
                                n,
                                &format!(
                                    "matrix-{scale:.2}-{}-{}",
                                    if dark { "dark" } else { "light" },
                                    if narrow { "narrow" } else { "wide" }
                                ),
                                Target::Field("document".into()),
                            );
                        }
                    }
                }
                p.extend([
                    Step::Click(Target::Scale),
                    Step::Click(Target::HeaderChoice("scale:1.00".into())),
                    Step::Viewport(false),
                    Step::Settled(1., false, false),
                ]);
            }
            if n == 3 {
                for dark in [true, false] {
                    p.extend([
                        Step::Click(Target::Scale),
                        Step::Click(Target::HeaderChoice("scale:2.00".into())),
                        Step::Click(Target::Theme),
                        Step::Click(Target::HeaderChoice(format!(
                            "theme:{}",
                            if dark { "dlms-dark" } else { "dlms-light" }
                        ))),
                        Step::Viewport(true),
                        Step::Settled(2., true, dark),
                    ]);
                    for last in [false, true] {
                        shot(
                            &mut p,
                            n,
                            &format!(
                                "goal-{}-{}",
                                if dark { "dark" } else { "light" },
                                if last { "last" } else { "first" }
                            ),
                            Target::Goal(last),
                        );
                    }
                }
                p.extend([
                    Step::Click(Target::Scale),
                    Step::Click(Target::HeaderChoice("scale:1.00".into())),
                    Step::Viewport(false),
                    Step::Settled(1., false, false),
                ]);
                // Actual queue/Run cancellation needs real work. A long empty
                // time window completes immediately, and this authored finite
                // Check can finish during the real click acknowledgement.
                // Stage a write followed by31 supported no-output commands in
                // each of four actual jobs, without changing the authored Goal.
                actor(&mut p, "tess");
                p.push(Step::Write(
                    "/work/handover-job.sh".into(),
                    "printf 'staged Stop probe\\n' > /work/handover-stage\n".to_owned()
                        + &"printf '%s' ''\n".repeat(31),
                ));
                for _ in 0..4 {
                    command(&mut p, "at -f /work/handover-job.sh now + 1 minute");
                }
                p.push(field("end", "2026-10-29 10:08"));
                // Queueing deliberately clears the earlier run proof. Snapshot
                // the real prepared work, not the old solved assignment.
                p.extend([
                    Step::Snapshot,
                    control("run"),
                    Step::StopActive,
                    Step::StopExact,
                    Step::StopIdle,
                    Step::Warm,
                    Step::Click(Target::Back),
                    Step::Click(Target::Card(0, false)),
                    Step::ResumeExact,
                    Step::StopIdle,
                ]);
                // Cleanup cannot recreate an earlier captured proof: reset only
                // this attempt and do the actual reference work again.
                reset(&mut p);
                actions(&mut p, &r["reference"]);
                check(
                    &mut p,
                    vec![true; 1 + r["initial_rows"].as_array().unwrap().len()],
                    false,
                );
            }
            if n < 3 {
                p.push(Step::Click(Target::Next));
            } else {
                p.push(Step::UnitComplete);
            }
        }
        p.push(Step::Finish);
        assert!(p.len() < 1000, "Closed OneOff plan{}", p.len());
        assert!(p.iter().filter(|s| matches!(s, Step::Shot(..))).count() <= 128);
        p
    })
}
#[derive(Clone)]
enum EnvironmentEdit {
    Click(Target),
    Fill(String, String),
}
#[derive(Clone, Copy, PartialEq)]
struct ClickGeometry {
    rect: egui::Rect,
    clip: egui::Rect,
    viewport: egui::Rect,
    zoom: u32,
    pixels_per_point: u32,
}
#[derive(Clone, Default)]
struct Qa {
    stage: usize,
    phase: u8,
    polls: usize,
    began: f64,
    last_root: Option<u64>,
    finished: bool,
    baseline: Option<Value>,
    snapshot: Option<Value>,
    receipts: Option<Value>,
    insert: Option<Value>,
    insert_count: u64,
    command_count: u64,
    command_before: Option<Value>,
    warm: Option<super::nat::Warm>,
    geometry: Option<(egui::Rect, u32, u32, Option<bool>)>,
    stable: u8,
    narrow_pixels: Option<egui::Vec2>,
    stop_steps: usize,
    op: u8,
    fill_phase: u8,
    keyboard_phase: u8,
    lab: usize,
    keyboard_text: Option<String>,
    wait_until: Option<f64>,
    practice_receipts: Option<Value>,
    click_target: Option<Target>,
    click_geometry: Option<ClickGeometry>,
    environment_edit: Option<EnvironmentEdit>,
    wheel_target: Option<Target>,
    wheel_geometry: Option<(egui::Rect, egui::Rect, egui::Vec2)>,
    wheel_stable: u8,
    wheel_count: usize,
    settled_wheels: usize,
    environment_clicks: usize,
    environment_fills: usize,
    stop_hovers: usize,
}
fn key() -> egui::Id {
    egui::Id::new("one-off-normal-native")
}
fn portable(a: &LabApp) -> Value {
    serde_json::to_value(&a.progress).unwrap()
}
fn receipts(a: &LabApp) -> Value {
    serde_json::to_value(&a.progress.migration_receipts).unwrap()
}
fn ctrl(k: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: k,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect()
}
#[cfg(test)]
pub(in crate::ui) fn phase_bounds() -> (usize, usize) {
    (FIRST, FIRST + plan().len() - 1)
}
pub(in crate::ui) fn immediate_stop(stage: usize) -> bool {
    if !(FIRST..END).contains(&stage) {
        return false;
    }
    let p = plan();
    let n = stage - FIRST;
    matches!(p.get(n), Some(Step::StopActive | Step::StopExact))
        || matches!(p.get(n + 1), Some(Step::StopActive))
}
impl LabApp {
    pub(in crate::ui) fn one_off_lessons_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("one-off-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn one_off_capture_active(&self) -> bool {
        self.smoke && (FIRST..END).contains(&self.smoke_stage)
    }
    fn oo_current(&self) -> (&Scenario, &Attempt) {
        let s = &self.scenarios[self.selected.unwrap()];
        (s, &self.progress.attempts[&s.id])
    }
    fn oo_practice(&self) -> bool {
        let (s, a) = self.oo_current();
        let _ = s;
        !a.challenge
            && a.workspace
                .as_ref()
                .unwrap()
                .one_off_learning
                .as_ref()
                .unwrap()
                .view
                == View::Example
    }
    fn oo_session(&self) -> &work::Session {
        let (_, a) = self.oo_current();
        let w = a.workspace.as_ref().unwrap();
        if self.oo_practice() {
            &w.one_off_learning.as_ref().unwrap().session
        } else {
            w.one_off_session.as_ref().unwrap()
        }
    }
    fn oo_assignment(&self) -> &work::Assignment {
        let (_, a) = self.oo_current();
        if self.oo_practice() {
            &a.workspace
                .as_ref()
                .unwrap()
                .one_off_learning
                .as_ref()
                .unwrap()
                .assignment
        } else {
            let engine::State::OneOff { simulation } = &a.state else {
                panic!("OneOff kind")
            };
            simulation
        }
    }
    fn oo_target(&self, ctx: &egui::Context, t: &Target) -> (egui::Rect, egui::Rect) {
        let root = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let m = &self.metrics.one_off;
        match t {
            Target::Control(k) => {
                let r = m
                    .controls
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| panic!("Actual control{k}"));
                assert!(r.2);
                return (r.1, r.3.intersect(root));
            }
            Target::Field(k) => {
                let r = m
                    .fields
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| panic!("Actual field{k}"));
                assert!(r.3);
                return (r.1, r.4.intersect(root));
            }
            Target::Choice(k) => {
                let r = m
                    .choices
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| panic!("Actual choice{k}"));
                return (
                    r.1,
                    m.popup.as_ref().unwrap().1.intersect(ctx.content_rect()),
                );
            }
            Target::HeaderChoice(k) => {
                let r = self
                    .metrics
                    .physical_picker_choices
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| {
                        let phase = ctx.data(|d| {
                            d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))
                        });
                        let pointer = ctx.input(|i| (i.pointer.hover_pos(), i.pointer.delta()));
                        panic!(
                            "Actual header choice{k}; stage{} {:?}, clickphase{phase:?}, scale{}, zoom{}, ppp{}, theme{}, viewport{:?}, pointer{pointer:?}, theme_button{:?}, scale_button{:?}, any_popup{}, actual_choices{:?}",
                            self.smoke_stage - FIRST,
                            plan().get(self.smoke_stage - FIRST),
                            self.scale,
                            ctx.zoom_factor(),
                            ctx.pixels_per_point(),
                            self.theme_id,
                            ctx.content_rect(),
                            self.metrics.reopen_theme,
                            self.metrics.reopen_scale,
                            egui::Popup::is_any_open(ctx),
                            self.metrics.physical_picker_choices,
                        )
                    });
                return (r.1, r.2.intersect(ctx.content_rect()));
            }
            Target::UnitChoice => {
                let r = self
                    .metrics
                    .unit_choices
                    .iter()
                    .find(|r| r.0 == UNIT)
                    .unwrap()
                    .1;
                return (
                    r,
                    self.metrics
                        .unit_popup
                        .unwrap()
                        .intersect(ctx.content_rect()),
                );
            }
            Target::Output => {
                let (_, clip) = m.output.expect("Actual output viewport");
                return (clip, root);
            }
            Target::Goal(end) => {
                let r = m.goal.unwrap();
                let font = egui::TextStyle::Body.resolve(&ctx.style());
                let h = ctx.fonts_mut(|f| f.row_height(&font)).min(r.height());
                let top = if *end { r.bottom() - h } else { r.top() };
                return (
                    egui::Rect::from_min_size(egui::pos2(r.left(), top), egui::vec2(r.width(), h)),
                    root,
                );
            }
            _ => {}
        }
        let r = match t {
            Target::UnitPicker => self.metrics.unit_picker,
            Target::Card(n, favorite) => {
                let c = self.metrics.card_actions.get(*n).expect("Real normal card");
                Some(if *favorite { c.1 } else { c.0 })
            }
            Target::Back => self.metrics.back,
            Target::Reset => self.metrics.reset,
            Target::Cancel => self.metrics.reset_cancel,
            Target::Confirm => self.metrics.confirm_reset,
            Target::Next => self.metrics.next_unit,
            Target::Challenge => self.metrics.mode_challenge,
            Target::Learn => self.metrics.mode_learn,
            Target::Intro => m.labels.first().map(|v| v.1),
            Target::Result => m.rows.first().map(|r| r.2),
            Target::Scale => self.metrics.reopen_scale,
            Target::Theme => self.metrics.reopen_theme,
            _ => unreachable!(),
        }
        .unwrap_or_else(|| panic!("Real target{t:?}"));
        (
            r,
            if matches!(
                t,
                Target::Scale | Target::Theme | Target::Cancel | Target::Confirm
            ) {
                ctx.content_rect()
            } else {
                root
            },
        )
    }
    fn oo_reveal(&mut self, ctx: &egui::Context, t: &Target, qa: &mut Qa) -> bool {
        let (r, clip) = self.oo_target(ctx, t);
        let ppp = ctx.pixels_per_point();
        assert!(r.is_finite() && clip.is_finite());
        // A real wheel remains in flight until natural time and observed
        // geometry settle. Never send another delta from an intermediate frame.
        if let Some(target) = &qa.wheel_target {
            assert_eq!(target, t, "Complete the same measured wheel target first");
            let geometry = (r, clip, self.metrics.main_scroll_offset);
            qa.wheel_stable = if qa.wheel_geometry == Some(geometry) {
                qa.wheel_stable.saturating_add(1)
            } else {
                1
            };
            qa.wheel_geometry = Some(geometry);
            if qa.wheel_stable < 2 {
                qa.wait_until = Some(ctx.input(|i| i.time) + 0.02);
                ctx.request_repaint_after(std::time::Duration::from_millis(20));
                return false;
            }
            qa.wheel_target = None;
            qa.wheel_geometry = None;
            qa.wheel_stable = 0;
            qa.settled_wheels += 1;
        }
        if smoke_control_visible(r, clip, ppp) {
            return true;
        }
        let main = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let scrolling = if matches!(
            t,
            Target::Choice(_) | Target::HeaderChoice(_) | Target::UnitChoice
        ) {
            clip
        } else {
            main
        };
        assert!(scrolling.is_positive());
        assert!(
            r.left() >= scrolling.left() - 1. / ppp && r.right() <= scrolling.right() + 1. / ppp,
            "Horizontal clipping {t:?}: {r:?}/{scrolling:?}"
        );
        qa.phase = qa.phase.saturating_add(1);
        assert!(
            qa.phase <= 20,
            "Bounded real-wheel reveal {t:?} {r:?}/{clip:?}"
        );
        let point = if matches!(
            t,
            Target::Choice(_) | Target::HeaderChoice(_) | Target::UnitChoice
        ) {
            // The popup clip can span the full window horizontally. Aim at
            // the actual item's x position so the wheel reaches its popup.
            egui::pos2(r.center().x, clip.center().y)
        } else {
            let main = self
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            egui::pos2(main.left() + 1. / ppp, main.center().y)
        };
        let delta = (scrolling.center().y - r.center().y)
            .clamp(-scrolling.height() * 0.75, scrolling.height() * 0.75);
        self.pending_events.extend([
            egui::Event::PointerMoved(point),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta),
                modifiers: Default::default(),
            },
        ]);
        qa.wheel_target = Some(t.clone());
        qa.wheel_geometry = None;
        qa.wheel_stable = 0;
        qa.wheel_count += 1;
        qa.wait_until = Some(ctx.input(|i| i.time) + 0.15);
        self.started = Instant::now();
        ctx.request_repaint_after(std::time::Duration::from_millis(20));
        false
    }
    fn oo_click(&mut self, ctx: &egui::Context, t: &Target, qa: &mut Qa) -> bool {
        let k = egui::Id::new(("native-click-phases", self.smoke_stage));
        if let Some(target) = &qa.click_target {
            assert_eq!(
                target, t,
                "A delivered native click must be acknowledged by its own target"
            );
        } else {
            qa.click_target = Some(t.clone());
        }
        if ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0) >= 2 {
            assert!(self.click_smoke_control(ctx, None));
            qa.click_target = None;
            qa.click_geometry = None;
            return true;
        }
        if !self.oo_reveal(ctx, t, qa) {
            return false;
        }
        let (r, clip) = self.oo_target(ctx, t);
        assert!(smoke_control_visible(r, clip, ctx.pixels_per_point()));
        let geometry = ClickGeometry {
            rect: r,
            clip,
            viewport: ctx.content_rect(),
            zoom: ctx.zoom_factor().to_bits(),
            pixels_per_point: ctx.pixels_per_point().to_bits(),
        };
        let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        let hovered = ctx.input(|i| i.pointer.hover_pos());
        if phase == 1
            && (qa.click_geometry != Some(geometry)
                || hovered.is_none_or(|p| p.distance(r.center()) > 1. / ctx.pixels_per_point()))
        {
            // Zoom, viewport or popup layout may move a target after phase0's
            // hover. Reach the CURRENT center in its own real frame before the
            // shared phase1 helper sends press/release. Never combine relocation
            // with pressing, force popup/focus, or treat a missing choice as done.
            qa.click_geometry = Some(geometry);
            self.pending_events
                .push(egui::Event::PointerMoved(r.center()));
            self.started = Instant::now();
            ctx.request_repaint();
            return false;
        }
        qa.click_geometry = Some(geometry);
        self.click_smoke_control(ctx, Some(r))
    }
    fn oo_field_value(&self, k: &str) -> &str {
        let s = self.oo_session();
        match k {
            "document" => &s.drafts[&s.selected],
            "command" => &s.command,
            "end" => &s.window_end,
            _ => s
                .environment
                .get(k.strip_prefix("variable:").unwrap())
                .unwrap(),
        }
    }
    fn oo_fill(&mut self, ctx: &egui::Context, k: &str, text: &str, qa: &mut Qa) -> bool {
        if qa.fill_phase == 0 {
            if !self.oo_click(ctx, &Target::Field(k.into()), qa) {
                return false;
            }
            qa.fill_phase = 1;
        }
        let id = self
            .metrics
            .one_off
            .fields
            .iter()
            .find(|f| f.0 == k)
            .expect("Current field id")
            .2;
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(id),
            "Real focus before edit"
        );
        if qa.fill_phase == 1 {
            self.pending_events.extend(ctrl(egui::Key::A));
            self.pending_events.push(egui::Event::Paste(text.into()));
            qa.fill_phase = 2;
            ctx.request_repaint();
            return false;
        }
        assert_eq!(
            self.oo_field_value(k),
            text,
            "Delivered edit must be in the correct saved draft"
        );
        qa.fill_phase = 0;
        true
    }
    fn oo_env(
        &mut self,
        ctx: &egui::Context,
        values: &BTreeMap<String, String>,
        qa: &mut Qa,
    ) -> bool {
        // Map changes happen on delivery, before the shared click/fill has
        // acknowledged that event. Finish the pinned suboperation before
        // choosing a new control or declaring the desired map complete.
        if let Some(edit) = qa.environment_edit.clone() {
            let done = match &edit {
                EnvironmentEdit::Click(target) => self.oo_click(ctx, target, qa),
                EnvironmentEdit::Fill(key, text) => self.oo_fill(ctx, key, text, qa),
            };
            if done {
                match edit {
                    EnvironmentEdit::Click(_) => qa.environment_clicks += 1,
                    EnvironmentEdit::Fill(..) => qa.environment_fills += 1,
                }
                qa.environment_edit = None;
                qa.phase = 0;
            }
            return false;
        }
        let current = &self.oo_session().environment;
        if current == values {
            return true;
        }
        let names = current
            .keys()
            .chain(values.keys())
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        assert!(names.len() <= 16);
        let name = names
            .iter()
            .find(|name| current.get(*name) != values.get(*name))
            .unwrap()
            .clone();
        qa.environment_edit = Some(
            if !self
                .metrics
                .one_off
                .controls
                .iter()
                .any(|r| r.0 == format!("include:{name}"))
            {
                EnvironmentEdit::Click(Target::Control("variables".into()))
            } else if current.contains_key(&name) != values.contains_key(&name) {
                EnvironmentEdit::Click(Target::Control(format!("include:{name}")))
            } else {
                EnvironmentEdit::Fill(format!("variable:{name}"), values[&name].clone())
            },
        );
        ctx.request_repaint();
        false
    }

    fn oo_scope(&self) {
        self.nat_private();
        assert!(!self.synthetic);
        assert_eq!(
            (
                one_off_boundary::loop_registration::lvm_registration::before_scenarios(
                    &self.scenarios
                )
                .len(),
                one_off_boundary::loop_registration::lvm_registration::before_typed_units(
                    &self.units
                )
                .len()
            ),
            (182, 48)
        );
        assert_eq!((one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&self.scenarios).len(),one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_typed_units(&self.units).len()),(186,49));
        assert_eq!((one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&self.scenarios).len(), one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_typed_units(&self.units).len()), (190, 50));
        assert_eq!((self.scenarios.len(), self.units.len()), (193, 51));
        assert_eq!(
            self.scenarios[176..180]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        assert_eq!(self.units[46].id, UNIT);
        assert_eq!(self.units[47].id, "shell-loop-recovery");
        assert!(
            self.progress.attempts.len() <= 4
                && self
                    .progress
                    .attempts
                    .keys()
                    .all(|s| IDS.contains(&s.as_str()))
        );
    }
    fn oo_warm(&mut self) -> super::nat::Warm {
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
        super::nat::exact(&projection, &portable, "Whole warm portable progress");
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        super::nat::exact(
            &loaded,
            &portable,
            "Read-only loaded progress; never installed",
        );
        super::nat::exact(&loaded.migration_receipts, &raw_receipts, "Raw receipts");
        assert_eq!(receipts(self), caller_receipts);
        assert_eq!(
            progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
            raw
        );
        super::nat::Warm {
            raw,
            portable,
            raw_receipts,
            caller_receipts,
        }
    }
    pub(in crate::ui) fn one_off_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let mut qa = ctx
            .data_mut(|d| d.remove_temp::<Qa>(key()))
            .unwrap_or_default();
        if qa.finished {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            store_qa(ctx, qa);
            return (false, None);
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        if qa.last_root == Some(root) {
            store_qa(ctx, qa);
            return (false, None);
        }
        qa.last_root = Some(root);
        let now = ctx.input(|i| i.time);
        if qa.stage != stage {
            qa.stage = stage;
            qa.phase = 0;
            qa.polls = 0;
            qa.began = now;
            qa.fill_phase = 0;
            qa.op = 0;
            qa.geometry = None;
            qa.stable = 0;
            assert!(
                qa.click_target.is_none()
                    && qa.click_geometry.is_none()
                    && qa.environment_edit.is_none()
                    && qa.wheel_target.is_none(),
                "No native suboperation may escape its bounded logical stage"
            );
        }
        assert!(
            (now - qa.began).is_finite() && (0. ..15.).contains(&(now - qa.began)),
            "OneOff stage{} {:?} deadline elapsed{} op{} reveal{}",
            stage - FIRST,
            plan().get(stage - FIRST),
            now - qa.began,
            qa.op,
            qa.phase
        );
        if qa.wait_until.is_some_and(|end| now < end) {
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
            store_qa(ctx, qa);
            return (false, None);
        }
        qa.wait_until = None;
        qa.polls += 1;
        assert!(
            qa.polls <= 300,
            "OneOff stage{} {:?} exceeded event bound",
            stage - FIRST,
            plan().get(stage - FIRST)
        );
        self.oo_scope();
        let step = plan()
            .get(stage - FIRST)
            .expect("Closed OneOff stage")
            .clone();
        let result = self.oo_step(ctx, &mut qa, step);
        store_qa(ctx, qa);
        result
    }
    fn oo_step(&mut self, ctx: &egui::Context, qa: &mut Qa, step: Step) -> (bool, Option<String>) {
        match step {
            Step::Library => {
                assert!(self.selected.is_none());
                self.filter = Filter::default();
                self.filter.track = "LPIC-1".into();
                self.extra_filters = true;
            }
            Step::Open(n) => {
                qa.lab = n;
                let (s, a) = self.oo_current();
                assert_eq!(s.id, IDS[n]);
                assert_eq!(a.state, s.initial);
                assert!(!a.challenge);
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .one_off_learning
                        .as_ref()
                        .unwrap()
                        .view,
                    View::Introduction
                );
            }
            Step::Click(t) => {
                if !self.oo_click(ctx, &t, qa) {
                    return (false, None);
                }
                if matches!(t, Target::Back) {
                    self.filter.search = self
                        .scenarios
                        .iter()
                        .find(|s| s.id == IDS[qa.lab])
                        .unwrap()
                        .title
                        .clone();
                }
                if matches!(t, Target::UnitChoice) {
                    assert_eq!(self.filter.unit, UNIT);
                }
                if let Target::Choice(k) = t {
                    if let Some(actor) = k.strip_prefix("actor:") {
                        assert_eq!(self.oo_session().actor, actor);
                    } else if let Some(path) = k.strip_prefix("target:") {
                        assert_eq!(self.oo_session().selected, path);
                    }
                }
            }
            Step::Fill(k, v) => {
                if !self.oo_fill(ctx, &k, &v, qa) {
                    return (false, None);
                }
            }
            Step::Env(v) => {
                if !self.oo_env(ctx, &v, qa) {
                    return (false, None);
                }
            }
            Step::ExampleBefore => {
                qa.baseline = Some(portable(self));
                qa.practice_receipts = Some(receipts(self));
            }
            Step::Example(n) => {
                if !self.oo_example(ctx, n, qa) {
                    return (false, None);
                }
            }
            Step::ExampleExact => {
                let (s, _) = self.oo_current();
                let id = s.id.clone();
                let mut current = portable(self);
                let mut before = qa.baseline.clone().unwrap();
                current["attempts"][&id]["workspace"]["one_off_learning"] = Value::Null;
                before["attempts"][&id]["workspace"]["one_off_learning"] = Value::Null;
                assert_eq!(
                    current, before,
                    "Practice changes only its explicit separate learning scope"
                );
                assert!(!self.metrics.completion);
                assert_eq!(receipts(self), *qa.practice_receipts.as_ref().unwrap());
            }
            Step::Write(path, text) => {
                if !self.oo_write(ctx, &path, &text, qa) {
                    return (false, None);
                }
            }
            Step::Command(text) => {
                if !self.oo_command(ctx, &text, qa) {
                    return (false, None);
                }
            }
            Step::Saved(path, text) => {
                assert_eq!(self.oo_session().drafts[&path], text);
                assert_eq!(self.oo_assignment().simulation.files[&path], text);
            }
            Step::CommandBefore(success) => {
                qa.command_count = self.one_off_presentation.command_executions();
                qa.command_before = Some(portable(self));
                let _ = success;
            }
            Step::CommandExact(text, success) => {
                assert_eq!(self.oo_session().command, text);
                assert_eq!(
                    self.one_off_presentation.command_executions(),
                    qa.command_count + u64::from(success)
                );
            }
            Step::WaitRun => {
                if self.one_off_presentation.running() {
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::Check(expected, practice) => {
                let (s, a) = self.oo_current();
                if self.one_off_presentation.memo.pending(a, s) {
                    ctx.request_repaint();
                    return (false, None);
                }
                let rows = self
                    .one_off_presentation
                    .memo
                    .rows(a, s, practice)
                    .expect("Actual current completed check rows");
                assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), expected);
                assert_eq!(
                    self.metrics.completion,
                    !practice && expected.iter().all(|r| *r)
                );
            }
            Step::Invalidated => {
                assert!(!self.metrics.completion);
                let (s, a) = self.oo_current();
                assert_ne!(self.one_off_presentation.memo.passed(a, s), Some(true));
            }
            Step::ResetLesson => {
                let steps = reset_steps();
                let (done, capture) = self.oo_step(ctx, qa, steps[usize::from(qa.op)].clone());
                assert!(capture.is_none());
                if done {
                    qa.op += 1;
                    qa.phase = 0;
                }
                if usize::from(qa.op) < steps.len() {
                    ctx.request_repaint();
                    return (false, None);
                }
                qa.op = 0;
            }
            Step::Snapshot => {
                qa.snapshot = Some(portable(self));
                qa.receipts = Some(receipts(self));
            }
            Step::CancelExact => {
                assert_eq!(portable(self), *qa.snapshot.as_ref().unwrap());
                assert_eq!(receipts(self), *qa.receipts.as_ref().unwrap());
                assert!(!self.reset_pending);
            }
            Step::ResetExact => {
                let (s, a) = self.oo_current();
                let id = s.id.clone();
                let visited = qa.snapshot.as_ref().unwrap()["attempts"][&id]["visited"]
                    .as_u64()
                    .unwrap();
                assert_eq!(a.visited, visited);
                assert!(!self.reset_pending);
                let mut fresh = Attempt::new(s, visited);
                work::budget::prepare(&mut fresh, s).unwrap();
                super::nat::exact(a, &fresh, "Complete selected fresh attempt");
                let mut expected = qa.snapshot.clone().unwrap();
                expected["attempts"][&id] = serde_json::to_value(fresh).unwrap();
                assert_eq!(portable(self), expected);
                assert_eq!(receipts(self), *qa.receipts.as_ref().unwrap());
            }
            Step::Warm => qa.warm = Some(self.oo_warm()),
            Step::ResumeExact => {
                let (s, _) = self.oo_current();
                let warm = qa.warm.as_ref().unwrap();
                let mut expected = warm.portable.clone();
                let clock = expected["clock"].as_u64().unwrap() + 1;
                expected["clock"] = clock.into();
                expected["attempts"][&s.id]["visited"] = clock.into();
                assert_eq!(portable(self), expected);
                assert_eq!(receipts(self), warm.caller_receipts);
                let raw: Value = serde_json::from_slice(&warm.raw).unwrap();
                assert_eq!(raw["migration_receipts"], warm.raw_receipts);
            }
            Step::StopActive => {
                let (s, a) = self.oo_current();
                assert!(
                    self.one_off_presentation.running(),
                    "Actual Run must still be active before Stop: stage{} lab{} op{} clickPhase{:?} pendingCheck{} paused{} counters{:?}",
                    self.smoke_stage,
                    qa.lab,
                    qa.op,
                    ctx.data(|d| d
                        .get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage)))),
                    self.one_off_presentation.memo.pending(a, s),
                    self.one_off_presentation.memo.paused(a, s),
                    self.one_off_presentation.memo.counters,
                );
                assert!(!self.one_off_presentation.memo.paused(a, s));
                let (r, clip) = self.oo_target(ctx, &Target::Control("stop".into()));
                assert!(smoke_control_visible(r, clip, ctx.pixels_per_point()));
                if qa.op == 0 {
                    qa.snapshot = Some(portable(self));
                    qa.receipts = Some(receipts(self));
                    self.pending_events
                        .push(egui::Event::PointerMoved(r.center()));
                    qa.stop_hovers += 1;
                    qa.op = 1;
                    ctx.request_repaint();
                    return (false, None);
                }
                assert_eq!(portable(self), *qa.snapshot.as_ref().unwrap());
                assert_eq!(receipts(self), *qa.receipts.as_ref().unwrap());
                let hovered = ctx.input(|i| i.pointer.hover_pos());
                if hovered.is_none_or(|p| p.distance(r.center()) > 1. / ctx.pixels_per_point()) {
                    self.pending_events
                        .push(egui::Event::PointerMoved(r.center()));
                    qa.stop_hovers += 1;
                    ctx.request_repaint();
                    return (false, None);
                }
                // The prior root frame delivered movement with no press. The
                // next real button batch has no parent-scroll drag delta.
                for pressed in [true, false] {
                    self.pending_events.push(egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    });
                }
            }

            Step::StopExact => {
                assert!(
                    ctx.input(|i| i.pointer.delta()).length() <= 1. / ctx.pixels_per_point(),
                    "Actual Stop release has no parent-scroll drag delta after its separate hover frame"
                );
                let (s, a) = self.oo_current();
                assert!(self.one_off_presentation.memo.paused(a, s));
                assert!(!self.one_off_presentation.memo.pending(a, s));
                assert!(!self.one_off_presentation.running());
                assert_eq!(portable(self), *qa.snapshot.as_ref().unwrap());
                assert_eq!(receipts(self), *qa.receipts.as_ref().unwrap());
                qa.stop_steps = self.one_off_presentation.memo.counters.steps;
            }
            Step::StopIdle => {
                let (s, a) = self.oo_current();
                assert!(
                    !self.one_off_presentation.running(),
                    "Stopped Run never restarts during idle or Continue"
                );
                assert!(self.one_off_presentation.memo.paused(a, s));
                assert_eq!(self.one_off_presentation.memo.counters.steps, qa.stop_steps);
                assert!(!self.metrics.completion);
                if qa.polls < 24 {
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::Viewport(narrow) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                if narrow {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
                }
            }
            Step::Settled(scale, narrow, dark) => {
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                let rect = ctx.content_rect();
                let fs = ctx.input(|i| i.viewport().fullscreen);
                let native = ctx
                    .input(|i| i.viewport().native_pixels_per_point)
                    .unwrap_or(1.);
                let pixels = rect.size() * ppp;
                let good = ppp.is_finite()
                    && native.is_finite()
                    && (zoom - scale).abs() < 0.01
                    && (ppp - scale * native).abs() < 0.01
                    && fs == Some(!narrow)
                    && self.theme_id == if dark { "dlms-dark" } else { "dlms-light" }
                    && if narrow {
                        (650. ..=1200.).contains(&pixels.x)
                    } else {
                        pixels.x >= 1500.
                    };
                let sample = (rect, ppp.to_bits(), zoom.to_bits(), fs);
                qa.stable = if good && qa.geometry == Some(sample) {
                    qa.stable + 1
                } else {
                    u8::from(good)
                };
                qa.geometry = Some(sample);
                if qa.stable < 2 {
                    ctx.request_repaint();
                    return (false, None);
                }
                if narrow {
                    qa.narrow_pixels = Some(pixels)
                } else {
                    let prior = qa.narrow_pixels.expect("Actual preceding narrow viewport");
                    assert!(pixels.x > prior.x * 1.25);
                }
                println!(
                    "OneOff geometry actualpixels{pixels:?}, zoom{zoom},nativePPP{native},fullscreen{fs:?}"
                );
            }
            Step::WorkspaceGeometry => {
                let targets = [
                    Target::Field("document".into()),
                    Target::Field("command".into()),
                    Target::Field("end".into()),
                    Target::Control("check".into()),
                    Target::Output,
                ];
                let target = &targets[usize::from(qa.op)];
                if !self.oo_reveal(ctx, target, qa) {
                    return (false, None);
                }
                let (r, clip) = self.oo_target(ctx, target);
                assert!(smoke_control_visible(r, clip, ctx.pixels_per_point()));
                let tolerance = 1. / ctx.pixels_per_point();
                for (key, r, _, _, clip) in &self.metrics.one_off.fields {
                    assert!(
                        r.left() >= clip.left() - tolerance
                            && r.right() <= clip.right() + tolerance,
                        "Actual field {key} fits current horizontal clip"
                    );
                }
                for (key, r, _, clip) in &self.metrics.one_off.controls {
                    assert!(
                        r.left() >= clip.left() - tolerance
                            && r.right() <= clip.right() + tolerance,
                        "Actual control {key} fits current horizontal clip"
                    );
                }
                qa.op += 1;
                qa.phase = 0;
                if usize::from(qa.op) < targets.len() {
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::Shot(name, target) => {
                if name.ends_with("-wrong-specific-feedback") {
                    let (source, attempt) = self.oo_current();
                    let rows = self
                        .one_off_presentation
                        .memo
                        .rows(attempt, source, false)
                        .expect("Actual checked assignment rows");
                    let failed = rows
                        .iter()
                        .position(|row| !row.0)
                        .expect("Wrong attempt has a real failed outcome");
                    let shown = self
                        .metrics
                        .one_off
                        .rows
                        .first()
                        .expect("Actual first displayed feedback row");
                    assert_eq!(
                        (shown.0, shown.1),
                        (failed, false),
                        "The specific actual failure must precede green routine passes"
                    );
                }
                if !self.oo_reveal(ctx, &target, qa) {
                    return (false, None);
                }
                let (r, clip) = self.oo_target(ctx, &target);
                assert!(smoke_control_visible(r, clip, ctx.pixels_per_point()));
                for (k, r, _, _, clip) in &self.metrics.one_off.fields {
                    assert!(
                        r.left() >= clip.left() - 1. / ctx.pixels_per_point()
                            && r.right() <= clip.right() + 1. / ctx.pixels_per_point(),
                        "Field{k} horizontal bounds"
                    );
                }
                return (true, Some(name));
            }
            Step::InspectCaptured => {
                let s = self.oo_session();
                assert_eq!(s.command, "at -c 1");
                let job = &self.oo_assignment().simulation.jobs[&1];
                let marker = "# Captured variables (JSON values, not shell commands): ";
                let line = s
                    .output
                    .lines()
                    .find_map(|v| v.strip_prefix(marker))
                    .unwrap();
                assert_eq!(
                    serde_json::from_str::<BTreeMap<String, String>>(line).unwrap(),
                    job.environment
                );
                assert!(s.output.ends_with(&job.source));
                assert_ne!(s.environment, job.environment);
            }
            Step::OwnerBefore => {
                qa.command_before = Some(portable(self));
                qa.command_count = self.one_off_presentation.command_executions();
            }
            Step::OwnerDenied => {
                assert_eq!(portable(self), *qa.command_before.as_ref().unwrap());
                assert_eq!(
                    self.one_off_presentation.command_executions(),
                    qa.command_count
                );
                assert!(
                    self.metrics
                        .one_off
                        .labels
                        .iter()
                        .any(|r| r.0.contains("belongs to another simulated account"))
                );
            }
            Step::Keyboard => {
                if !self.oo_keyboard(ctx, qa) {
                    return (false, None);
                }
            }
            Step::ChallengeExact => {
                let (_, a) = self.oo_current();
                assert!(a.challenge);
                assert!(!self.metrics.one_off.controls.iter().any(|r| {
                    ["try-example", "practice", "insert", "introduction"].contains(&r.0.as_str())
                }));
                assert!(self.metrics.one_off.goal.is_some());
            }
            Step::UnitComplete => {
                assert!(self.metrics.unit_complete.is_some());
                assert!(self.metrics.next_unit.is_none());
            }
            Step::Finish => {
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert_eq!(
                        self.one_off_presentation
                            .memo
                            .passed(&self.progress.attempts[id], s),
                        Some(true),
                        "Actual exact-key completion certificate, no synchronous replay"
                    );
                }
                let _ = self.oo_warm();
                println!(
                    "PASS native four normal OneOff lessons; real unit/Intro/separate example/assignment/ref+2alts/feedback/owner+capture inspection/reset+resume/Stop/geometry. Actual visual and novice/accessibility acceptance are separate."
                );
                qa.finished = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
    fn oo_insert_exact(&self, n: usize, qa: &Qa) {
        let (s, a) = self.oo_current();
        let spec = &s.one_off_teaching.as_ref().unwrap().example_actions[n];
        assert_eq!(
            self.one_off_presentation.command_executions(),
            qa.insert_count
        );
        let mut expected = qa.insert.clone().unwrap();
        let session = &mut expected["attempts"][&s.id]["workspace"]["one_off_learning"]["session"];
        match &spec.action {
            work::Action::Edit { path, text } => session["drafts"][path] = text.clone().into(),
            work::Action::Save { path } => session["selected"] = path.clone().into(),
            work::Action::Actor { actor } => session["actor"] = actor.clone().into(),
            work::Action::Environment { values } => {
                session["environment"] = serde_json::to_value(values).unwrap()
            }
            work::Action::Command { text } => session["command"] = text.clone().into(),
            work::Action::Run { end } => session["window_end"] = end.text().into(),
            _ => panic!("Closed example Insert action"),
        }
        assert_eq!(
            portable(self),
            expected,
            "Inert Insert changes only its exact documented field, never simulation/assignment/otherwork"
        );
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .one_off_learning
                .as_ref()
                .unwrap()
                .selected_example,
            n
        );
    }
    fn oo_example(&mut self, ctx: &egui::Context, n: usize, qa: &mut Qa) -> bool {
        match qa.op {
            0 => {
                if !self.oo_click(ctx, &Target::Control("example".into()), qa) {
                    return false;
                }
                qa.op = 1;
            }
            1 => {
                if !self.oo_click(ctx, &Target::Choice(format!("example:{n}")), qa) {
                    return false;
                }
                qa.op = 2;
            }
            2 => {
                qa.insert = Some(portable(self));
                qa.insert_count = self.one_off_presentation.command_executions();
                qa.op = 3;
            }
            3 => {
                if !self.oo_click(ctx, &Target::Control("insert".into()), qa) {
                    return false;
                }
                qa.op = 4;
            }
            _ => {
                self.oo_insert_exact(n, qa);
                qa.op = 0;
                return true;
            }
        }
        ctx.request_repaint();
        false
    }
    fn oo_command(&mut self, ctx: &egui::Context, text: &str, qa: &mut Qa) -> bool {
        match qa.op {
            0 => {
                if !self.oo_fill(ctx, "command", text, qa) {
                    return false;
                }
                qa.command_count = self.one_off_presentation.command_executions();
                qa.command_before = Some(portable(self));
                qa.op = 1;
            }
            1 => {
                if !self.oo_click(ctx, &Target::Control("command-run".into()), qa) {
                    return false;
                }
                qa.op = 2;
            }
            _ => {
                assert_eq!(self.oo_session().command, text);
                assert_eq!(
                    self.one_off_presentation.command_executions(),
                    qa.command_count + 1,
                    "A fresh real command, not an old matching receipt"
                );
                qa.op = 0;
                return true;
            }
        }
        ctx.request_repaint();
        false
    }
    fn oo_write(&mut self, ctx: &egui::Context, path: &str, text: &str, qa: &mut Qa) -> bool {
        match qa.op {
            0 => {
                if !self.oo_click(ctx, &Target::Control("target".into()), qa) {
                    return false;
                }
                qa.op = 1;
            }
            1 => {
                if !self.oo_click(ctx, &Target::Choice(format!("target:{path}")), qa) {
                    return false;
                }
                assert_eq!(self.oo_session().selected, path);
                qa.op = 2;
            }
            2 => {
                if !self.oo_fill(ctx, "document", text, qa) {
                    return false;
                }
                qa.op = 3;
            }
            3 => {
                if !self.oo_click(ctx, &Target::Control("save".into()), qa) {
                    return false;
                }
                qa.op = 4;
            }
            _ => {
                assert_eq!(self.oo_session().drafts[path], text);
                assert_eq!(self.oo_assignment().simulation.files[path], text);
                qa.op = 0;
                return true;
            }
        }
        ctx.request_repaint();
        false
    }
    fn oo_keyboard(&mut self, ctx: &egui::Context, qa: &mut Qa) -> bool {
        let field = "document";
        match qa.keyboard_phase {
            0 => {
                if !self.oo_click(ctx, &Target::Field(field.into()), qa) {
                    return false;
                }
                qa.command_before = Some(portable(self));
                qa.keyboard_text = Some(self.oo_field_value(field).into());
                self.pending_events.extend(ctrl(egui::Key::End));
                qa.keyboard_phase = 1;
                false
            }
            1 => {
                let id = self
                    .metrics
                    .one_off
                    .fields
                    .iter()
                    .find(|r| r.0 == field)
                    .unwrap()
                    .2;
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                let state = egui::text_edit::TextEditState::load(ctx, id).unwrap();
                let cursor = state.cursor.char_range().unwrap();
                assert!(cursor.is_empty());
                assert_eq!(
                    cursor.primary.index,
                    qa.keyboard_text.as_ref().unwrap().chars().count()
                );
                qa.wait_until = Some(ctx.input(|i| i.time) + 1.2);
                qa.keyboard_phase = 2;
                false
            }
            2 => {
                self.pending_events
                    .push(egui::Event::Text("\n# native keyboard note".into()));
                qa.keyboard_phase = 3;
                false
            }
            3 => {
                assert_eq!(
                    self.oo_field_value(field),
                    format!(
                        "{}\n# native keyboard note",
                        qa.keyboard_text.as_ref().unwrap()
                    )
                );
                qa.wait_until = Some(ctx.input(|i| i.time) + 1.2);
                qa.keyboard_phase = 4;
                false
            }
            4 => {
                self.pending_events.extend(ctrl(egui::Key::Z));
                qa.keyboard_phase = 5;
                false
            }
            5 => {
                assert_eq!(
                    self.oo_field_value(field),
                    qa.keyboard_text.as_ref().unwrap()
                );
                assert_eq!(
                    portable(self),
                    *qa.command_before.as_ref().unwrap(),
                    "Actual Undo restores whole saved work, not only visible text"
                );
                let id = self
                    .metrics
                    .one_off
                    .fields
                    .iter()
                    .find(|r| r.0 == field)
                    .unwrap()
                    .2;
                let cursor = egui::text_edit::TextEditState::load(ctx, id)
                    .unwrap()
                    .cursor
                    .char_range()
                    .unwrap();
                assert!(cursor.is_empty());
                assert_eq!(
                    cursor.primary.index,
                    qa.keyboard_text.as_ref().unwrap().chars().count()
                );
                for pressed in [true, false] {
                    self.pending_events.push(egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    });
                }
                qa.keyboard_phase = 6;
                false
            }
            6 => {
                let id = self
                    .metrics
                    .one_off
                    .fields
                    .iter()
                    .find(|r| r.0 == field)
                    .unwrap()
                    .2;
                assert_ne!(ctx.memory(|m| m.focused()), Some(id));
                for pressed in [true, false] {
                    self.pending_events.push(egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::SHIFT,
                    });
                }
                qa.keyboard_phase = 7;
                false
            }
            7 => {
                // egui's Previous (Shift+Tab) queues id_next_frame so gained_focus
                // is observed on the next pass. Let that ordinary root frame run
                // before checking the actual focus; never install it ourselves.
                qa.keyboard_phase = 8;
                ctx.request_repaint();
                false
            }
            _ => {
                let id = self
                    .metrics
                    .one_off
                    .fields
                    .iter()
                    .find(|r| r.0 == field)
                    .unwrap()
                    .2;
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                assert_eq!(portable(self), *qa.command_before.as_ref().unwrap());
                qa.keyboard_phase = 0;
                true
            }
        }
    }
}

#[cfg(test)]
pub(in crate::ui) fn planned_captures() -> Vec<String> {
    plan()
        .iter()
        .filter_map(|step| match step {
            Step::Shot(name, _) => Some(name.clone()),
            _ => None,
        })
        .collect()
}
#[cfg(test)]
pub(in crate::ui) fn observations(
    ctx: &egui::Context,
) -> Option<(bool, usize, Option<f64>, usize)> {
    ctx.data(|d| d.get_temp(egui::Id::new("one-off-native-test-facts")))
}
#[cfg(test)]
pub(in crate::ui) fn completed(ctx: &egui::Context) -> bool {
    observations(ctx).is_some_and(|q| q.0)
}
#[cfg(test)]
pub(in crate::ui) fn delivery_observations(
    ctx: &egui::Context,
) -> Option<(usize, usize, usize, usize, usize)> {
    ctx.data(|d| d.get_temp(egui::Id::new("one-off-native-delivery-facts")))
}
#[cfg(test)]
pub(in crate::ui) fn undo_waiting(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp(egui::Id::new("one-off-native-undo-wait")))
        .unwrap_or(false)
}
fn store_qa(ctx: &egui::Context, qa: Qa) {
    #[cfg(test)]
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("one-off-native-undo-wait"),
            [2, 4].contains(&qa.keyboard_phase) && qa.wait_until.is_some(),
        )
    });
    #[cfg(test)]
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("one-off-native-test-facts"),
            (qa.finished, qa.polls, qa.wait_until, qa.lab),
        )
    });
    #[cfg(test)]
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("one-off-native-delivery-facts"),
            (
                qa.environment_clicks,
                qa.environment_fills,
                qa.wheel_count,
                qa.settled_wheels,
                qa.stop_hovers,
            ),
        )
    });
    ctx.data_mut(|d| d.insert_temp(key(), qa));
}
