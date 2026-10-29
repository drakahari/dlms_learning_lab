//! Fixed normal RPM acceptance inputs. Every model change comes from real GUI Run.
use super::nat::{Op as GlobalOp, Queue, Target as Global};
use super::*;
use dlms_learning_lab::{
    rpm_packages_teaching::Learning,
    rpm_packages_terminal::{Outcome, Session},
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
pub(in crate::ui) const FIRST: usize = 40000;
const END: usize = 41000;
pub(in crate::ui) const IDS: [&str; 4] = [
    "lpic1-rpm-provider",
    "lpic1-rpm-epoch",
    "lpic1-rpm-payload",
    "lpic1-rpm-handover",
];
const UNIT: &str = "rpm-package-investigation";
const PROFILE_CAP: usize = 2 * 1024 * 1024;
const ASSIGNMENT_DRAFT: &str = "rpm -q 'unfinished assignment";
const EXAMPLE_DRAFT: &str = "rpm -qp 'different unfinished example";
const _: () = assert!(FIRST > super::nat_lessons::LAST && END == 41000);
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    name: String,
    commands: Vec<String>,
    rows: Vec<bool>,
    outcomes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    initial: Vec<bool>,
    reference: Plan,
    alternatives: Vec<Plan>,
    wrong: Plan,
    partial: Plan,
    read_only: Plan,
    after_pass_break: Plan,
    repair: Plan,
    practice_initial: Vec<bool>,
    configuration_path: String,
    configuration_tamper_rows: Vec<bool>,
    neighbor_path: String,
    neighbor_tamper_rows: Vec<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reviews {
    schema_version: u32,
    expected_origin: String,
    lessons: Vec<Review>,
}
fn reviews() -> Vec<Review> {
    let r: Reviews = serde_json::from_str(include_str!(
        "../../../tests/fixtures/rpm-packages-review.json"
    ))
    .unwrap();
    assert_eq!(r.schema_version, 1);
    assert!(r.expected_origin.contains("not recorded"));
    assert_eq!(r.lessons.len(), 4);
    for (r, id) in r.lessons.iter().zip(IDS) {
        assert_eq!(r.id, id);
        assert_eq!(r.alternatives.len(), 2);
        for p in std::iter::once(&r.reference).chain(&r.alternatives).chain([
            &r.wrong,
            &r.partial,
            &r.read_only,
            &r.after_pass_break,
            &r.repair,
        ]) {
            assert!(
                !p.name.is_empty()
                    && !p.commands.is_empty()
                    && p.commands.len() <= 8
                    && p.commands.len() == p.outcomes.len()
            );
            assert!(p.commands.iter().all(|s| !s.is_empty() && s.len() <= 2048));
            assert!(
                p.outcomes
                    .iter()
                    .all(|s| s == "utility" || s == "unsupported")
            );
        }
        assert!(!r.configuration_path.is_empty() && !r.neighbor_path.is_empty());
        assert_eq!(r.configuration_tamper_rows.len(), r.initial.len());
        assert_eq!(r.neighbor_tamper_rows.len(), r.initial.len());
    }
    r.lessons
}
#[derive(Clone, Debug)]
enum Control {
    Card(bool),
    Try,
    Tab(teaching::View),
    Input,
    Run,
    Check,
    Picker,
    Choice(usize),
    Insert,
    ExampleReset,
    Inspect,
    Available,
    HeaderPicker,
    PathPicker,
    InspectChoice(String),
    InspectResult,
    Reset,
    Cancel,
    Confirm,
    Back,
    Challenge,
    Learn,
    Feedback(usize),
    Failure,
    Output,
    Heading,
    Library(usize),
}
#[derive(Clone, Debug)]
enum Step {
    Select(usize),
    Wait,
    Click(Control),
    Global(Global),
    FileName,
    Open,
    BeforePractice,
    InsertBefore,
    InsertOnly,
    Command(String, bool, String, bool, bool),
    Type(String),
    BeforeRun,
    Enter,
    CtrlCheck,
    CheckNoRun(bool),
    Ran(String, bool, String, bool),
    Rows(Vec<bool>, bool),
    Practice(bool),
    Current(bool),
    PracticeExact,
    RecordReset,
    CancelExact,
    ResetExact,
    Warm,
    RecordDrafts,
    Filter,
    Continue,
    ResumeExact,
    Theme(bool),
    Geometry(f32),
    Capture(String, Control),
    InspectorBefore,
    InspectorExact(String, String),
    GuardBefore(bool),
    BlockedKeys(bool),
    GuardExact(bool),
    Library,
    Finish,
}
fn commands(p: &mut Vec<Step>, c: &[String], k: &[String], practice: bool, readonly: bool) {
    assert_eq!(c.len(), k.len());
    for (n, (c, k)) in c.iter().zip(k).enumerate() {
        p.push(Step::Command(
            c.clone(),
            practice,
            k.clone(),
            readonly,
            n % 2 != 0,
        ));
    }
}
fn utilities(p: &mut Vec<Step>, c: &[String], practice: bool) {
    commands(p, c, &vec!["utility".into(); c.len()], practice, false)
}
fn checked(p: &mut Vec<Step>, rows: Step, name: &str) {
    let pass = match &rows {
        Step::Current(v) | Step::Practice(v) => *v,
        Step::Rows(v, _) => v.iter().all(|b| *b),
        _ => unreachable!(),
    };
    let both = [
        "reference",
        "example-worked",
        "alternate-1",
        "dark200",
        "after-reset-repaired",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix));
    p.extend([Step::Click(Control::Check), Step::Wait, rows]);
    if both {
        p.extend([
            Step::Capture(format!("{name}-first"), Control::Feedback(0)),
            Step::Capture(format!("{name}-last"), Control::Feedback(usize::MAX)),
        ]);
    } else {
        p.push(Step::Capture(
            format!("{name}-{}", if pass { "last" } else { "failure" }),
            if pass {
                Control::Feedback(usize::MAX)
            } else {
                Control::Failure
            },
        ));
    }
}

fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Warm,
        Step::RecordReset,
        Step::Click(Control::Reset),
        Step::Click(Control::Confirm),
        Step::Wait,
        Step::ResetExact,
        Step::Click(Control::Tab(teaching::View::Assignment)),
        Step::Wait,
    ]);
}
fn modal(p: &mut Vec<Step>, practice: bool) {
    p.extend([
        Step::Warm,
        Step::GuardBefore(practice),
        Step::Global(Global::Settings),
        Step::Wait,
        Step::BlockedKeys(practice),
        Step::Wait,
        Step::GuardExact(practice),
        Step::Global(Global::Tab("appearance")),
        Step::Global(Global::Samples),
        Step::Wait,
        Step::BlockedKeys(practice),
        Step::Wait,
        Step::GuardExact(practice),
        Step::Global(Global::Samples),
        Step::Global(Global::Tab("data")),
        Step::Global(Global::DataExport),
        Step::Wait,
        Step::FileName,
        Step::BlockedKeys(practice),
        Step::Wait,
        Step::GuardExact(practice),
        Step::Global(Global::FileCancel),
        Step::Global(Global::SettingsClose),
        Step::Wait,
        Step::GuardExact(practice),
    ]);
}

fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = Vec::new();
    for (n, r) in reviews().iter().enumerate() {
        let s = scenarios.iter().find(|s| s.id == r.id).unwrap();
        assert_eq!(s.unit, UNIT);
        assert!(matches!(s.initial, engine::State::RpmPackages { .. }));
        let l = s.rpm_packages_teaching.as_ref().unwrap();
        let name = |s: &str| format!("rpm-{n}-{s}");
        p.extend([
            Step::Select(n),
            Step::Wait,
            Step::Click(Control::Card(true)),
            Step::Click(Control::Card(false)),
            Step::Open,
            Step::Capture(name("intro"), Control::Heading),
            Step::Click(Control::Try),
            Step::Wait,
            Step::Click(Control::ExampleReset),
            Step::Wait,
            Step::BeforePractice,
        ]);
        checked(
            &mut p,
            Step::Rows(r.practice_initial.clone(), true),
            &name("example-initial"),
        );
        p.extend([
            Step::Click(Control::Picker),
            Step::Click(Control::Choice(l.example_commands.len() - 1)),
            Step::Click(Control::Picker),
            Step::Click(Control::Choice(0)),
            Step::InsertBefore,
            Step::Click(Control::Insert),
            Step::InsertOnly,
            Step::BeforeRun,
            Step::Click(Control::Run),
            Step::Ran(
                l.example_commands[0].command.clone(),
                true,
                "utility".into(),
                false,
            ),
        ]);
        utilities(
            &mut p,
            &l.example_commands[1..]
                .iter()
                .map(|c| c.command.clone())
                .collect::<Vec<_>>(),
            true,
        );
        checked(&mut p, Step::Practice(true), &name("example-worked"));
        utilities(
            &mut p,
            &l.try_change
                .commands
                .iter()
                .map(|c| c.command.clone())
                .collect::<Vec<_>>(),
            true,
        );
        checked(
            &mut p,
            Step::Rows(l.try_change.expected.clone(), true),
            &name("example-adverse"),
        );
        utilities(
            &mut p,
            &l.repair
                .iter()
                .map(|c| c.command.clone())
                .collect::<Vec<_>>(),
            true,
        );
        checked(&mut p, Step::Practice(true), &name("example-repaired"));
        p.push(Step::PracticeExact);
        if n == 0 {
            modal(&mut p, true);
        }
        p.extend([
            Step::Click(Control::Tab(teaching::View::Assignment)),
            Step::Wait,
            Step::Capture(name("assignment"), Control::Heading),
        ]);
        checked(
            &mut p,
            Step::Rows(r.initial.clone(), false),
            &name("initial"),
        );
        // Inspect a supplied real local header; payload task additionally compares the missing manual.
        let archive = match n {
            0 => "/rpms/bridge-a.rpm",
            1 => "/rpms/engine-new.rpm",
            _ => "/rpms/client.rpm",
        };
        p.extend([
            Step::InspectorBefore,
            Step::Click(Control::Inspect),
            Step::Wait,
            Step::Click(Control::Available),
            Step::Wait,
            Step::Click(Control::HeaderPicker),
            Step::Click(Control::InspectChoice(archive.into())),
            Step::Wait,
            Step::Click(Control::PathPicker),
        ]);
        let path = if n == 2 {
            "/usr/share/archive-client/manual".into()
        } else {
            r.configuration_path.clone()
        };
        // Provider/epoch archive files differ from consumer config: use actual declared first path.
        let path = if n < 2 {
            let engine::Goal::RpmPackages { definition } = &s.goal else {
                unreachable!()
            };
            let id = &definition.archive_paths[archive];
            definition.initial.catalog().archives[id]
                .files
                .keys()
                .next()
                .unwrap()
                .clone()
        } else {
            path
        };
        p.extend([
            Step::Click(Control::InspectChoice(path.clone())),
            Step::Wait,
            Step::InspectorExact(
                {
                    let engine::Goal::RpmPackages { definition } = &s.goal else {
                        unreachable!()
                    };
                    definition.archive_paths[archive].clone()
                },
                path,
            ),
            Step::Capture(name("declared-data-inspection"), Control::InspectResult),
            Step::Click(Control::Inspect),
            Step::Wait,
        ]);
        commands(&mut p, &r.wrong.commands, &r.wrong.outcomes, false, false);
        checked(
            &mut p,
            Step::Rows(r.wrong.rows.clone(), false),
            &name("wrong"),
        );
        p.push(Step::Capture(
            name("wrong-specific-failure"),
            Control::Failure,
        ));
        reset(&mut p);
        commands(
            &mut p,
            &r.partial.commands,
            &r.partial.outcomes,
            false,
            false,
        );
        checked(
            &mut p,
            Step::Rows(r.partial.rows.clone(), false),
            &name("partial"),
        );
        reset(&mut p);
        utilities(&mut p, &s.solution, false);
        checked(&mut p, Step::Current(true), &name("reference"));
        commands(
            &mut p,
            &r.read_only.commands,
            &r.read_only.outcomes,
            false,
            true,
        );
        p.push(Step::Current(true));
        let unsupported = "rpm -V ignored-name".to_string();
        commands(
            &mut p,
            std::slice::from_ref(&unsupported),
            &["unsupported".into()],
            false,
            true,
        );
        p.push(Step::Current(true));
        p.push(Step::Capture(name("truthful-unsupported"), Control::Output));
        commands(
            &mut p,
            &r.after_pass_break.commands,
            &r.after_pass_break.outcomes,
            false,
            false,
        );
        checked(
            &mut p,
            Step::Rows(r.after_pass_break.rows.clone(), false),
            &name("invalidated"),
        );
        commands(&mut p, &r.repair.commands, &r.repair.outcomes, false, false);
        checked(&mut p, Step::Current(true), &name("repaired"));
        for (a, alt) in r.alternatives.iter().enumerate() {
            reset(&mut p);
            commands(&mut p, &alt.commands, &alt.outcomes, false, false);
            checked(
                &mut p,
                Step::Rows(alt.rows.clone(), false),
                &name(&format!("alternate-{a}")),
            );
        }
        // Ctrl+Enter is a pure Check, not a second Run, even with a valid focused command.
        let query = r.read_only.commands[0].clone();
        p.extend([
            Step::Click(Control::Input),
            Step::Type(query.clone()),
            Step::BeforeRun,
            Step::CtrlCheck,
            Step::Wait,
            Step::CheckNoRun(false),
            Step::Current(true),
        ]);
        if n == 0 {
            modal(&mut p, false);
        }
        for light in [true, false] {
            p.extend([
                Step::Theme(light),
                Step::Geometry(if light { 1.5 } else { 2. }),
                Step::Click(Control::Input),
                Step::Type(query.clone()),
                Step::Capture(
                    name(if light {
                        "light150-focused-input"
                    } else {
                        "dark200-focused-input"
                    }),
                    Control::Input,
                ),
                Step::BeforeRun,
                Step::Enter,
                Step::Ran(query.clone(), false, "utility".into(), true),
                Step::Capture(
                    name(if light {
                        "light150-query-output"
                    } else {
                        "dark200-query-output"
                    }),
                    Control::Output,
                ),
            ]);
            checked(
                &mut p,
                Step::Current(true),
                &name(if light { "light150" } else { "dark200" }),
            );
        }
        p.extend([
            Step::Click(Control::Input),
            Step::Type(ASSIGNMENT_DRAFT.into()),
            Step::Click(Control::Tab(teaching::View::Example)),
            Step::Wait,
        ]);
        utilities(
            &mut p,
            &l.example_commands
                .iter()
                .map(|c| c.command.clone())
                .collect::<Vec<_>>(),
            true,
        );
        p.extend([
            Step::Click(Control::Input),
            Step::Type(EXAMPLE_DRAFT.into()),
            Step::RecordDrafts,
            Step::Warm,
            Step::Click(Control::Back),
            Step::Filter,
            Step::Wait,
            Step::Continue,
            Step::Wait,
            Step::ResumeExact,
            Step::Capture(name("resumed-separate-draft"), Control::Input),
            Step::Click(Control::Tab(teaching::View::Assignment)),
            Step::Wait,
            Step::Warm,
            Step::RecordReset,
            Step::Click(Control::Reset),
            Step::Click(Control::Cancel),
            Step::Wait,
            Step::CancelExact,
            Step::Click(Control::Challenge),
            Step::Wait,
            Step::Current(true),
            Step::Capture(name("challenge-full-task"), Control::Heading),
            Step::Click(Control::Learn),
            Step::Wait,
        ]);
        reset(&mut p);
        utilities(&mut p, &s.solution, false);
        checked(&mut p, Step::Current(true), &name("after-reset-repaired"));
        p.extend([Step::Warm, Step::Click(Control::Back), Step::Wait]);
    }
    p.extend([
        Step::Library,
        Step::Wait,
        Step::Capture("rpm-normal-unit-first".into(), Control::Library(0)),
        Step::Capture("rpm-normal-unit-last".into(), Control::Library(3)),
        Step::Finish,
    ]);
    assert!(p.len() < END - FIRST, "Closed1000phase reservation");
    assert!(p.iter().filter(|s| matches!(s, Step::Capture(..))).count() <= 128);
    p
}
pub(in crate::ui) fn phase_bounds(scenarios: &[Scenario]) -> (usize, usize) {
    let p = plan(scenarios);
    (FIRST, FIRST + p.len() - 1)
}
fn session(a: &Attempt, practice: bool) -> &Session {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        &w.rpm_learning.as_ref().unwrap().terminal
    } else {
        w.rpm_session.as_ref().unwrap()
    }
}
fn learning(a: &Attempt) -> &Learning {
    a.workspace.as_ref().unwrap().rpm_learning.as_ref().unwrap()
}
fn model(a: &Attempt, practice: bool) -> Value {
    if practice {
        serde_json::to_value(&learning(a).simulation).unwrap()
    } else {
        serde_json::to_value(&a.state).unwrap()
    }
}
fn exact<A: serde::Serialize + ?Sized, B: serde::Serialize + ?Sized>(a: &A, b: &B, why: &str) {
    super::nat::exact(a, b, why)
}
fn key(name: &str) -> egui::Id {
    egui::Id::new(("rpm-normal-lessons", name))
}
fn ctrl(k: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: k,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: pressed,
            command: pressed,
            ..Default::default()
        },
    }
}
#[derive(Clone)]
struct Saved {
    portable: Value,
    raw: Vec<u8>,
    receipts: Value,
    caller_receipts: Value,
}
#[derive(Clone)]
struct Reset {
    progress: Value,
    receipts: Value,
    files: BTreeMap<String, Vec<u8>>,
}
impl LabApp {
    pub(in crate::ui) fn rpm_lessons_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("rpm-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn rpm_lessons_contains_stage(&self, stage: usize) -> bool {
        (FIRST..END).contains(&stage) && stage <= phase_bounds(&self.scenarios).1
    }
    fn rpm_current(&self, id: &str) -> &Attempt {
        &self.progress.attempts[id]
    }
    fn rpm_put<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, name: &str, v: T) {
        ctx.data_mut(|d| d.insert_temp(key(name), v));
    }
    fn rpm_get<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, name: &str) -> T {
        ctx.data(|d| d.get_temp::<T>(key(name)))
            .expect("actual prior GUI observation")
    }
    fn rpm_rows(&self, id: &str, practice: bool) -> Vec<bool> {
        let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
        let a = self.rpm_current(id);
        if practice {
            learning(a)
                .checks(s.rpm_packages_teaching.as_ref().unwrap())
                .iter()
                .map(|r| r.0)
                .collect()
        } else {
            progress::current_rpm_checks(a, s)
                .iter()
                .map(|r| r.0)
                .collect()
        }
    }
    fn rpm_scope(&self) {
        self.nat_private();
        assert!(!self.synthetic);
        assert_eq!((self.scenarios.len(), self.units.len()), (176, 46));
        assert_eq!(
            self.scenarios[143..147]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        assert_eq!(
            self.scenarios[147..150]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            [
                "shell-command-options",
                "shell-command-dispatch",
                "shell-command-plan",
            ]
        );
        assert!(self.scenarios[147..150].iter().all(|s| {
            matches!(&s.initial, engine::State::Script { .. })
                && matches!(&s.goal, engine::Goal::Script { .. })
        }));
        assert_eq!(
            self.scenarios[150..154]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            [
                "lpic2-nfs-reference",
                "lpic2-nfs-publication",
                "lpic2-nfs-handover",
                "lpic2-nfs-capstone",
            ]
        );
        assert!(self.scenarios[150..154].iter().all(|s| matches!(
            &s.initial,
            engine::State::Nfs { .. }
        ) && matches!(
            &s.goal,
            engine::Goal::Nfs { .. }
        )));
        assert!(
            self.progress.attempts.len() <= 4
                && self
                    .progress
                    .attempts
                    .keys()
                    .all(|id| IDS.contains(&id.as_str()))
        );
        for a in self.progress.attempts.values() {
            assert!(a.transcript.is_empty());
            let w = a.workspace.as_ref().unwrap();
            assert!(w.script_history.is_none() && w.script_history_more.is_empty());
            assert!(session(a, false).entries.len() <= 16 && session(a, true).entries.len() <= 16);
        }
    }
    fn rpm_output_row(&self) -> &(String, egui::Rect, egui::Rect) {
        let a = self.rpm_current(&self.scenarios[self.selected.unwrap()].id);
        let practice = learning(a).view == teaching::View::Example && !a.challenge;
        let n = session(a, practice).entries.len().checked_sub(1).unwrap();
        ["stdout","unsupported","status","summary","command"].iter().find_map(|suffix| {
            self.metrics.rpm.output_rows.iter().find(|r|r.0==format!("{n}:{suffix}")
                && r.1.is_finite()&&r.1.width()>0.&&r.1.height()>0.)
        }).unwrap_or_else(||panic!("Current RPM entry{n} has no nonempty actual painted output/status/command label; child={:?}; candidate geometry={:?}",self.metrics.rpm.output,self.metrics.rpm.output_rows.iter().filter(|r|r.0.starts_with(&format!("{n}:"))).collect::<Vec<_>>()))
    }
    fn rpm_target(&self, ctx: &egui::Context, t: &Control) -> (egui::Rect, egui::Rect, bool) {
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let m = &self.metrics.rpm;
        match t {
            Control::Output => {
                let row = self.rpm_output_row();
                (
                    row.1,
                    row.2.intersect(m.output.unwrap()).intersect(root),
                    true,
                )
            }
            Control::Feedback(n) => {
                let rows = &m.result_rows;
                let row = if *n == usize::MAX {
                    rows.last().unwrap()
                } else {
                    rows.iter().find(|r| r.0 == *n).unwrap()
                };
                (row.2, row.3.intersect(root), false)
            }
            Control::Failure => {
                let row = m.result_rows.iter().find(|r| !r.1).unwrap();
                (row.2, row.3.intersect(root), false)
            }
            Control::Choice(n) => {
                let row = m.choices.iter().find(|r| r.0 == *n).unwrap();
                (row.1, m.popup.unwrap().intersect(ctx.content_rect()), true)
            }
            Control::InspectChoice(path) => {
                let row = m.inspect_choices.iter().find(|r| r.0 == *path).unwrap();
                (
                    row.1,
                    m.inspect_popup.unwrap().intersect(ctx.content_rect()),
                    true,
                )
            }
            Control::Library(n) => (self.metrics.card_actions[*n].0, root, false),
            _ => {
                let r = match t {
                    Control::Card(fav) => {
                        assert_eq!(self.metrics.card_actions.len(), 1);
                        let r = self.metrics.card_actions[0];
                        Some(if *fav { r.1 } else { r.0 })
                    }
                    Control::Try => self.metrics.example_action,
                    Control::Tab(v) => m.tabs.iter().find(|r| r.0 == *v).map(|r| r.1),
                    Control::Input => m.command.map(|r| r.0),
                    Control::Run => m.run.map(|r| r.0),
                    Control::Check => m.check,
                    Control::Picker => m.example,
                    Control::Insert => m.insert,
                    Control::ExampleReset => m.example_reset,
                    Control::Inspect => m.inspector,
                    Control::Available => m.inspect_available,
                    Control::HeaderPicker => m.inspect_header,
                    Control::PathPicker => m.inspect_path,
                    Control::InspectResult => m.inspect_result,
                    Control::Reset => self.metrics.reset,
                    Control::Cancel => self.metrics.reset_cancel,
                    Control::Confirm => self.metrics.confirm_reset,
                    Control::Back => self.metrics.back,
                    Control::Challenge => self.metrics.mode_challenge,
                    Control::Learn => self.metrics.mode_learn,
                    Control::Heading => self.metrics.package_heading,
                    _ => unreachable!(),
                }
                .expect("actual painted RPM control");
                (r, root, false)
            }
        }
    }
    fn rpm_reveal(&mut self, ctx: &egui::Context, t: &Control) -> bool {
        let (r, v, inner) = self.rpm_target(ctx, t);
        let k = egui::Id::new(("rpm-reveal", self.smoke_stage));
        let mut q = ctx.data(|d| d.get_temp::<Queue>(k)).unwrap_or_default();
        if matches!(t, Control::Output) {
            let root = self
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            let child = self.metrics.rpm.output.unwrap();
            assert!(
                child.width() > 0. && child.height() > 0.,
                "RPM output pre-reveal child has no actual area: selected_row={} target={r:?} target_clip={v:?} child={child:?} root={root:?} command={:?}",
                self.rpm_output_row().0,
                self.metrics.rpm.command
            );
            if let Some(piece) =
                super::nat::static_root_reveal_piece(r, v, child, root, ctx.pixels_per_point())
                && !self.nat_reveal(ctx, piece, root, false, &mut q)
            {
                ctx.data_mut(|d| d.insert_temp(k, q));
                return false;
            }
        }
        let done = self.nat_reveal(ctx, r, v, inner, &mut q);
        ctx.data_mut(|d| d.insert_temp(k, q));
        done
    }
    fn rpm_click(&mut self, ctx: &egui::Context, t: &Control) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(t, Control::Choice(_) | Control::InspectChoice(_)) {
            let (r, v, _) = self.rpm_target(ctx, t);
            return self.mac_popup_choice(ctx, Some(r), Some(v));
        }
        if !self.rpm_reveal(ctx, t) {
            return false;
        }
        let (r, _, _) = self.rpm_target(ctx, t);
        self.click_smoke_control(ctx, Some(r))
    }
    fn rpm_type(&mut self, ctx: &egui::Context, text: &str) {
        assert!(text.len() <= 2048 && !text.contains(['\n', '\r', '\0']));
        let (_, id, enabled) = self.metrics.rpm.command.unwrap();
        assert!(enabled && ctx.memory(|m| m.has_focus(id)));
        self.pending_events.extend([
            ctrl(egui::Key::A, true),
            egui::Event::Text(text.into()),
            ctrl(egui::Key::A, false),
        ]);
    }
    fn rpm_before_run(&self, ctx: &egui::Context) {
        self.rpm_put(
            ctx,
            "before-run",
            serde_json::to_value(&self.progress).unwrap(),
        );
    }
    fn rpm_ran(
        &self,
        ctx: &egui::Context,
        id: &str,
        text: &str,
        practice: bool,
        kind: &str,
        readonly: bool,
    ) {
        let before: Value = self.rpm_get(ctx, "before-run");
        let a = self.rpm_current(id);
        let t = session(a, practice);
        let e = t.entries.last().expect("actual Run evidence");
        assert_eq!(e.command, text);
        assert_eq!(
            t.command, text,
            "RPM retains literal raw command after submission"
        );
        match (&e.output, kind) {
            (Outcome::Utility { status, stderr, .. }, "utility") => {
                assert_eq!(*status, 0);
                assert!(stderr.is_empty())
            }
            (Outcome::SimulatorUnsupported { reason }, "unsupported") => {
                assert!(!reason.is_empty())
            }
            _ => panic!("Actual RPM Outcome kind differs; no fake numeric failure status"),
        }
        let before_a = &before["attempts"][id];
        let old_session = if practice {
            &before_a["workspace"]["rpm_learning"]["terminal"]
        } else {
            &before_a["workspace"]["rpm_session"]
        };
        let old_entries = old_session["entries"].as_array().unwrap();
        assert_eq!(t.entries.len(), (old_entries.len() + 1).min(16));
        let kept = t.entries.len() - 1;
        exact(
            &t.entries[..kept],
            &&old_entries[old_entries.len() - kept..],
            "Exact retained original Session tail, no output rewriting",
        );
        if readonly || kind == "unsupported" {
            let old_model = if practice {
                before_a["workspace"]["rpm_learning"]["simulation"].clone()
            } else {
                before_a["state"].clone()
            };
            exact(
                &model(a, practice),
                &old_model,
                "Read-only/refused command preserves complete actual State",
            );
        }
        let mut expected = before;
        if practice {
            expected["attempts"][id]["workspace"]["rpm_learning"] =
                serde_json::to_value(learning(a)).unwrap()
        } else {
            expected["attempts"][id]["state"] = serde_json::to_value(&a.state).unwrap();
            expected["attempts"][id]["workspace"]["rpm_session"] = serde_json::to_value(t).unwrap();
        }
        exact(
            &self.progress,
            &expected,
            "Actual Run changes only selected model/Session or own Learning; other fields/Attempts/favorites/clock exact",
        );
    }
    fn rpm_command(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        args: (&str, bool, &str, bool, bool),
    ) -> bool {
        let (text, practice, kind, readonly, enter) = args;
        let k = egui::Id::new(("rpm-command-substep", self.smoke_stage));
        let sub = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
        let done = match sub {
            0 => self.rpm_click(ctx, &Control::Input),
            1 => {
                self.rpm_type(ctx, text);
                true
            }
            2 => {
                assert_eq!(session(self.rpm_current(id), practice).command, text);
                self.rpm_before_run(ctx);
                true
            }
            3 => {
                if enter {
                    let (_, input, enabled) = self.metrics.rpm.command.unwrap();
                    assert!(enabled && ctx.memory(|m| m.has_focus(input)));
                    self.pending_events.extend([
                        egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: Default::default(),
                        },
                        egui::Event::Key {
                            key: egui::Key::Enter,
                            physical_key: None,
                            pressed: false,
                            repeat: false,
                            modifiers: Default::default(),
                        },
                    ]);
                    true
                } else {
                    self.rpm_click(ctx, &Control::Run)
                }
            }
            4 => {
                self.rpm_ran(ctx, id, text, practice, kind, readonly);
                return true;
            }
            _ => unreachable!(),
        };
        if done {
            ctx.data_mut(|d| d.insert_temp(k, sub + 1));
        }
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn rpm_warm(&mut self, ctx: &egui::Context) {
        let portable = serde_json::to_value(&self.progress).unwrap();
        let caller_receipts = serde_json::to_value(&self.progress.migration_receipts).unwrap();
        self.save();
        assert!(!self.dirty && self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, PROFILE_CAP).unwrap();
        let mut projected: Value = serde_json::from_slice(&raw).unwrap();
        let receipts = projected
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .unwrap();
        exact(
            &projected,
            &portable,
            "Complete raw/portable projection, only established top-level receipts removed",
        );
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        exact(
            &loaded,
            &portable,
            "Actual normal load complete progress; not installed into live app",
        );
        exact(
            &loaded.migration_receipts,
            &receipts,
            "Actual disk receipts",
        );
        exact(
            &self.progress.migration_receipts,
            &caller_receipts,
            "Save does not mutate caller receipts",
        );
        assert!(
            progress::read_bounded(&self.path, PROFILE_CAP).unwrap() == raw,
            "Raw private profile bytes changed during bounded load; values withheld"
        );
        self.rpm_put(
            ctx,
            "warm",
            Saved {
                portable,
                raw,
                receipts,
                caller_receipts,
            },
        );
    }
    fn rpm_files(&self) -> BTreeMap<String, Vec<u8>> {
        let mut result = BTreeMap::new();
        let mut actual = 0usize;
        let mut preflight = 0usize;
        for (n, item) in std::fs::read_dir(self.path.parent().unwrap())
            .unwrap()
            .enumerate()
        {
            assert!(n < 16);
            let item = item.unwrap();
            let t = item.file_type().unwrap();
            assert!(t.is_file() && !t.is_symlink());
            let bytes = usize::try_from(item.metadata().unwrap().len()).unwrap();
            assert!(bytes <= PROFILE_CAP);
            preflight = preflight.checked_add(bytes).unwrap();
            assert!(preflight <= 8 * 1024 * 1024);
            let name = item.file_name().into_string().unwrap();
            assert!(name.len() <= 256);
            let raw = progress::read_bounded(&item.path(), PROFILE_CAP).unwrap();
            actual = actual.checked_add(raw.len()).unwrap();
            assert!(actual <= 8 * 1024 * 1024);
            result.insert(name, raw);
        }
        result
    }
    fn rpm_record_reset(&self, ctx: &egui::Context) {
        self.rpm_put(
            ctx,
            "reset",
            Reset {
                progress: serde_json::to_value(&self.progress).unwrap(),
                receipts: serde_json::to_value(&self.progress.migration_receipts).unwrap(),
                files: self.rpm_files(),
            },
        );
    }
    fn rpm_geometry(&self, ctx: &egui::Context, scale: f32) -> bool {
        let rect = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let zoom = ctx.zoom_factor();
        assert!(rect.is_finite() && ppp.is_finite() && ppp > 0.);
        let sample = (rect, ppp, zoom);
        let k = egui::Id::new(("rpm-geometry", self.smoke_stage));
        let ready = ctx.data_mut(|d| {
            let (old, n) = d
                .get_temp::<(Option<(egui::Rect, f32, f32)>, usize)>(k)
                .unwrap_or((None, 0));
            if old == Some(sample) && (zoom - scale).abs() < 0.01 {
                return true;
            }
            let n = n + 1;
            assert!(n <= 20, "RPM actual geometry must settle");
            d.insert_temp(k, (Some(sample), n));
            false
        });
        if !ready {
            ctx.request_repaint();
        }
        ready
    }
    pub(in crate::ui) fn rpm_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        self.rpm_scope();
        let p = plan(&self.scenarios);
        assert!((FIRST..FIRST + p.len()).contains(&stage));
        let offset = stage - FIRST;
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
        let tick = egui::Id::new(("rpm-step-ticks", stage));
        let ticks = ctx.data_mut(|d| {
            let n = d.get_temp::<u16>(tick).unwrap_or(0) + 1;
            d.insert_temp(tick, n);
            n
        });
        assert!(
            ticks <= 300,
            "RPM fixed event step{stage} exceeded300ticks at {step:?}; command={:?}, root={:?}, PPP={}, focus={:?}",
            self.metrics.rpm.command,
            self.metrics.main_viewport,
            ctx.pixels_per_point(),
            ctx.memory(|m| m.focused())
        );
        match step {
            Step::Select(_) => {
                assert!(self.selected.is_none());
                assert_eq!(self.progress.attempts.len(), n);
                for old in &IDS[..n] {
                    let old_s = self.scenarios.iter().find(|s| s.id == *old).unwrap();
                    assert!(progress::current_solved(self.rpm_current(old), old_s));
                }
                self.filter = Filter {
                    search: s.title.clone(),
                    ..Default::default()
                };
                self.page = 0;
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            Step::Wait => {}
            Step::Click(t) => {
                if !self.rpm_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Global(t) => {
                if !self.nat_queue_for(ctx, vec![GlobalOp::Click(t.clone())], id, false) {
                    return (false, None);
                }
            }
            Step::FileName => {
                if !self.nat_queue_for(
                    ctx,
                    vec![GlobalOp::FileText("../invalid-backup".into())],
                    id,
                    false,
                ) {
                    return (false, None);
                }
            }
            Step::Open => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(
                    learning(self.rpm_current(id)).view,
                    teaching::View::Introduction
                );
                exact(
                    &self.rpm_current(id).state,
                    &s.initial,
                    "Real normal Start creates the exact authored initial State",
                );
                assert!(self.progress.favorites.contains(id));
                self.rpm_put(
                    ctx,
                    &format!("initial-{id}"),
                    serde_json::to_value(self.rpm_current(id)).unwrap(),
                );
            }
            Step::BeforePractice => {
                assert_eq!(learning(self.rpm_current(id)).view, teaching::View::Example);
                self.rpm_put(
                    ctx,
                    "practice-before",
                    serde_json::to_value(&self.progress).unwrap(),
                );
            }
            Step::InsertBefore => {
                self.rpm_put(
                    ctx,
                    "insert-before",
                    serde_json::to_value(&self.progress).unwrap(),
                );
            }
            Step::InsertOnly => {
                let a = self.rpm_current(id);
                assert_eq!(learning(a).selected_example, 0);
                assert_eq!(
                    session(a, true).command,
                    s.rpm_packages_teaching.as_ref().unwrap().example_commands[0].command
                );
                let mut expected: Value = self.rpm_get(ctx, "insert-before");
                expected["attempts"][id]["workspace"]["rpm_learning"]["terminal"]["command"] =
                    session(a, true).command.clone().into();
                exact(
                    &self.progress,
                    &expected,
                    "Insert only fills one command, no execution/state/history change",
                );
            }
            Step::Command(text, practice, kind, readonly, enter) => {
                if !self.rpm_command(ctx, id, (text, *practice, kind, *readonly, *enter)) {
                    return (false, None);
                }
            }
            Step::Type(text) => self.rpm_type(ctx, text),
            Step::BeforeRun => self.rpm_before_run(ctx),
            Step::Enter => {
                let (_, input, enabled) = self.metrics.rpm.command.unwrap();
                assert!(enabled && ctx.memory(|m| m.has_focus(input)));
                self.pending_events.extend([
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                ]);
            }
            Step::CtrlCheck => {
                let (_, input, enabled) = self.metrics.rpm.command.unwrap();
                assert!(enabled && ctx.memory(|m| m.has_focus(input)));
                let k = egui::Id::new(("rpm-control-check", stage));
                let phase = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
                if phase < 2 {
                    self.pending_events.push(ctrl(egui::Key::Enter, phase == 0));
                    ctx.data_mut(|d| d.insert_temp(k, phase + 1));
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::CheckNoRun(practice) => {
                let before: Value = self.rpm_get(ctx, "before-run");
                let a = self.rpm_current(id);
                let expected = if *practice {
                    &before["attempts"][id]["workspace"]["rpm_learning"]["terminal"]
                } else {
                    &before["attempts"][id]["workspace"]["rpm_session"]
                };
                exact(
                    session(a, *practice),
                    expected,
                    "Ctrl+Enter checks, never runs a valid focused command",
                );
                let old_model = if *practice {
                    before["attempts"][id]["workspace"]["rpm_learning"]["simulation"].clone()
                } else {
                    before["attempts"][id]["state"].clone()
                };
                exact(
                    &model(a, *practice),
                    &old_model,
                    "Check never changes model",
                );
            }
            Step::Ran(text, practice, kind, readonly) => {
                self.rpm_ran(ctx, id, text, *practice, kind, *readonly)
            }
            Step::Rows(expected, practice) => {
                assert_eq!(self.rpm_rows(id, *practice), *expected);
                assert_eq!(
                    progress::current_solved(self.rpm_current(id), &s),
                    if *practice {
                        false
                    } else {
                        expected.iter().all(|r| *r)
                    }
                );
            }
            Step::Practice(pass) => {
                assert_eq!(self.rpm_rows(id, true).iter().all(|v| *v), *pass);
                assert!(learning(self.rpm_current(id)).checked);
                assert!(!self.metrics.completion);
            }
            Step::Current(pass) => {
                assert_eq!(self.rpm_rows(id, false).iter().all(|v| *v), *pass);
                assert_eq!(progress::current_solved(self.rpm_current(id), &s), *pass);
                assert_eq!(self.metrics.completion, *pass);
            }
            Step::PracticeExact => {
                let mut before: Value = self.rpm_get(ctx, "practice-before");
                before["attempts"][id]["workspace"]["rpm_learning"] =
                    serde_json::to_value(learning(self.rpm_current(id))).unwrap();
                exact(&self.progress, &before, "Whole separate practice isolation");
            }
            Step::InspectorBefore => self.rpm_put(
                ctx,
                "inspector-before",
                serde_json::to_value(&self.progress).unwrap(),
            ),
            Step::InspectorExact(archive, path) => {
                let before: Value = self.rpm_get(ctx, "inspector-before");
                exact(
                    &self.progress,
                    &before,
                    "Inspector is read-only, not a repair/grade credential",
                );
                let selected = self.metrics.rpm.inspect_selected.as_ref().unwrap();
                assert_eq!(&selected.0, archive, "Real chosen local header");
                assert_eq!(selected.1.as_deref(), Some(path.as_str()));
                assert!(!self.metrics.rpm.inspection_text.is_empty());
            }
            Step::Warm => self.rpm_warm(ctx),
            Step::RecordDrafts => {
                let a = self.rpm_current(id);
                assert_eq!(session(a, false).command, ASSIGNMENT_DRAFT);
                assert_eq!(session(a, true).command, EXAMPLE_DRAFT);
                assert!(progress::current_solved(a, &s));
                self.rpm_put(
                    ctx,
                    "resume-before",
                    serde_json::to_value(&self.progress).unwrap(),
                );
                self.rpm_put(
                    ctx,
                    "resume-receipts",
                    serde_json::to_value(&self.progress.migration_receipts).unwrap(),
                );
            }
            Step::Filter => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    search: s.title.clone(),
                    ..Default::default()
                };
                self.page = 0;
            }
            Step::Continue => {
                if !self.rpm_click(ctx, &Control::Card(false)) {
                    return (false, None);
                }
                assert_eq!(self.selected, Some(i));
            }
            Step::ResumeExact => {
                let saved: Saved = self.rpm_get(ctx, "warm");
                let mut raw: Value = serde_json::from_slice(&saved.raw).unwrap();
                let receipts = raw
                    .as_object_mut()
                    .unwrap()
                    .remove("migration_receipts")
                    .unwrap();
                exact(
                    &raw,
                    &saved.portable,
                    "Original complete warm raw projection",
                );
                exact(&receipts, &saved.receipts, "Original warm disk receipts");
                let caller: Value = self.rpm_get(ctx, "resume-receipts");
                exact(
                    &saved.caller_receipts,
                    &caller,
                    "Original saved caller receipts",
                );
                let mut expected: Value = self.rpm_get(ctx, "resume-before");
                exact(
                    &saved.portable,
                    &expected,
                    "Saved whole Progress anchor before real Back/Continue",
                );
                let clock = expected["clock"].as_u64().unwrap() + 1;
                expected["clock"] = clock.into();
                expected["attempts"][id]["visited"] = clock.into();
                exact(
                    &self.progress,
                    &expected,
                    "Real Continue only clock+visited, both full sessions and all other fields exact",
                );
                let receipts: Value = self.rpm_get(ctx, "resume-receipts");
                exact(
                    &self.progress.migration_receipts,
                    &receipts,
                    "Continue caller receipts",
                );
                assert_eq!(learning(self.rpm_current(id)).view, teaching::View::Example);
                self.rpm_warm(ctx);
            }
            Step::RecordReset => self.rpm_record_reset(ctx),
            Step::CancelExact => {
                let before: Reset = self.rpm_get(ctx, "reset");
                exact(
                    &self.progress,
                    &before.progress,
                    "ResetCancel complete progress",
                );
                exact(
                    &self.progress.migration_receipts,
                    &before.receipts,
                    "ResetCancel all receipts",
                );
                assert!(
                    self.rpm_files() == before.files,
                    "ResetCancel raw private files differ; values withheld"
                );
                assert!(!self.reset_pending);
            }
            Step::ResetExact => {
                let before: Reset = self.rpm_get(ctx, "reset");
                let mut expected = before.progress;
                let visited = expected["attempts"][id]["visited"].clone();
                let mut initial: Value = self.rpm_get(ctx, &format!("initial-{id}"));
                initial["visited"] = visited;
                expected["attempts"][id] = initial;
                exact(
                    &self.progress,
                    &expected,
                    "Selected reset equals actual first Start, complete others/favorites/clock preserved",
                );
                assert_eq!(
                    learning(self.rpm_current(id)).view,
                    teaching::View::Introduction
                );
            }
            Step::Theme(light) => {
                self.scale = if *light { 1.5 } else { 2. };
                self.theme_id = if *light { "dlms-light" } else { "dlms-dark" }.into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
            }
            Step::Geometry(scale) => {
                if !self.rpm_geometry(ctx, *scale) {
                    return (false, None);
                }
            }
            Step::Capture(name, target) => {
                if !self.rpm_geometry(ctx, self.scale) || !self.rpm_reveal(ctx, target) {
                    return (false, None);
                }
                if matches!(target, Control::Input) {
                    let (_, input, enabled) = self.metrics.rpm.command.unwrap();
                    assert!(enabled && ctx.memory(|m| m.has_focus(input)));
                    assert!(
                        !session(
                            self.rpm_current(id),
                            learning(self.rpm_current(id)).view == teaching::View::Example
                                && !self.rpm_current(id).challenge
                        )
                        .command
                        .is_empty()
                    );
                }
                let content = ctx.content_rect();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                let outer = ctx.input(|i| i.viewport().outer_rect);
                let full = ctx.input(|i| i.viewport().fullscreen);
                println!(
                    "RPM {id} {name} actual_content={content:?} outer={outer:?} fullscreen={full:?} zoom={zoom} PPP={ppp}; requested dimensions are not observed dimensions"
                );
                return (true, Some(format!("{name}-zoom{zoom:.2}-ppp{ppp:.2}")));
            }
            Step::GuardBefore(practice) => {
                self.rpm_record_reset(ctx);
                self.rpm_put(ctx, "guard-practice", *practice);
            }
            Step::BlockedKeys(practice) => {
                assert!(self.settings_open);
                assert!(
                    !self.metrics.rpm.command.unwrap().2
                        && !self.metrics.rpm.run.unwrap().1
                        && !self.metrics.rpm.check_enabled
                );
                assert_eq!(self.rpm_get::<bool>(ctx, "guard-practice"), *practice);
                self.pending_events.extend([
                    egui::Event::Text("blocked RPM input".into()),
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: Default::default(),
                    },
                    ctrl(egui::Key::Enter, true),
                    ctrl(egui::Key::Enter, false),
                ]);
            }
            Step::GuardExact(practice) => {
                assert_eq!(self.rpm_get::<bool>(ctx, "guard-practice"), *practice);
                let before: Reset = self.rpm_get(ctx, "reset");
                exact(
                    &self.progress,
                    &before.progress,
                    "Modal keyboard input cannot run/check/edit either session",
                );
                exact(
                    &self.progress.migration_receipts,
                    &before.receipts,
                    "Modal receipts unchanged",
                );
                assert!(
                    self.rpm_files() == before.files,
                    "Settings modal writes no profile/file changes"
                );
            }
            Step::Library => {
                assert!(self.selected.is_none());
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.filter = Filter {
                    unit: UNIT.into(),
                    ..Default::default()
                };
                self.page = 0;
            }
            Step::Finish => {
                assert_eq!(stage, FIRST + p.len() - 1);
                if ctx.data(|d| d.get_temp::<bool>(key("finished"))) == Some(true) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                assert_eq!(self.metrics.card_actions.len(), 4);
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(self.rpm_current(id), s));
                    assert!(self.progress.favorites.contains(id));
                }
                self.rpm_warm(ctx);
                exact(
                    &self.scenarios,
                    &pack::bundled().unwrap(),
                    "Every normal147 specification exact, no appended fixture",
                );
                println!(
                    "PASS four normal RPM lessons through actual controls, separate persisted practice, two alternatives, current state and exact warm progress; no host RPM, cold-process, novice or other-OS claim."
                );
                ctx.data_mut(|d| d.insert_temp(key("finished"), true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
pub(in crate::ui) fn checkpoints(scenarios: &[Scenario]) -> Vec<(String, usize)> {
    plan(scenarios)
        .iter()
        .enumerate()
        .filter_map(|(n, s)| match s {
            Step::Select(i) => Some((format!("select-{i}"), FIRST + n)),
            Step::Open => Some((format!("opened-{n}"), FIRST + n)),
            Step::Capture(name, _) => Some((name.clone(), FIRST + n)),
            Step::Finish => Some(("finish".into(), FIRST + n)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
pub(in crate::ui) fn output_probe(
    a: &mut LabApp,
    ctx: &egui::Context,
) -> (bool, String, egui::Rect, egui::Rect) {
    let visible = a.rpm_reveal(ctx, &Control::Output);
    let row = a.rpm_output_row();
    let (r, v, _) = a.rpm_target(ctx, &Control::Output);
    (visible, row.0.clone(), r, v)
}
