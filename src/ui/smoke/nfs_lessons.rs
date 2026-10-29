//! Four normal authored NFS activities. Model effects only through real widgets.
use super::nat::{Op as GlobalOp, Queue, Target as Global, Warm};
use super::*;
use dlms_learning_lab::{nfs as core, nfs_teaching as learn, nfs_workspace as work};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
pub(in crate::ui) const FIRST: usize = 45000;
const END: usize = 46000;
pub(in crate::ui) const IDS: [&str; 4] = [
    "lpic2-nfs-reference",
    "lpic2-nfs-publication",
    "lpic2-nfs-handover",
    "lpic2-nfs-capstone",
];
const UNIT: &str = "nfs-network-file-sharing";
const _: () = assert!(FIRST > 43504 && END == 46000);
const ASSIGNMENT_RAW: &str = "../unfinished assignment request";
const EXAMPLE_RAW: &str = "/different unfinished example request";
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    name: String,
    actions: Vec<core::Action>,
    rows: Vec<bool>,
    outcomes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    reason: String,
    initial: Vec<bool>,
    reference: Plan,
    alternatives: Vec<Plan>,
    wrong: Plan,
    partial: Plan,
    no_op: Plan,
    read_only: Plan,
    after_pass_break: Plan,
    repair: Plan,
    practice_initial: Vec<bool>,
    practice_worked: Vec<bool>,
    practice_adverse: Vec<bool>,
    preservation: Vec<work::Requirement>,
    fault_probe: core::Action,
    current_vs_fresh_break: Vec<bool>,
    extra_known_reader: Option<Plan>,
    server_rw: Option<Plan>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reviews {
    schema_version: u32,
    expected_origin: String,
    lessons: Vec<Review>,
}
fn reviews() -> Vec<Review> {
    let r: Reviews =
        serde_json::from_str(include_str!("../../../tests/fixtures/nfs-review.json")).unwrap();
    assert_eq!(r.schema_version, 1);
    assert!(r.expected_origin.contains("not recorded"));
    assert_eq!(r.lessons.len(), 4);
    for (r, id) in r.lessons.iter().zip(IDS) {
        assert_eq!(r.id, id);
        assert!(!r.reason.is_empty());
        assert_eq!(r.alternatives.len(), 2);
        for p in std::iter::once(&r.reference)
            .chain(&r.alternatives)
            .chain([
                &r.wrong,
                &r.partial,
                &r.read_only,
                &r.after_pass_break,
                &r.repair,
            ])
            .chain(r.extra_known_reader.iter())
            .chain(r.server_rw.iter())
        {
            assert!(!p.name.is_empty() && !p.actions.is_empty() && p.actions.len() <= 16);
            assert_eq!(p.actions.len(), p.outcomes.len());
            assert_eq!(p.rows.len(), r.initial.len());
        }
        // The authored no-op is deliberately empty and must retain the exact
        // initial negative vector. Every executable plan above remains nonempty.
        assert!(!r.no_op.name.is_empty());
        assert!(r.no_op.actions.is_empty() && r.no_op.outcomes.is_empty());
        assert_eq!(r.no_op.rows, r.initial);
        assert!(!r.preservation.is_empty());
        assert_eq!(r.current_vs_fresh_break.len(), r.initial.len());
        assert!(matches!(r.fault_probe, core::Action::ReadExisting { .. }));
    }
    r.lessons
}
#[derive(Clone, Debug)]
enum Control {
    Card(bool),
    Try,
    Tab(teaching::View),
    Button(&'static str),
    Field(&'static str),
    Choice(String),
    Row(usize),
    Failure,
    Heading,
    ReadOutput,
    Reset,
    Cancel,
    Confirm,
    Back,
    Challenge,
    Learn,
    Library(usize),
}
#[derive(Clone, Debug)]
enum Op {
    Click(Control),
    Text(&'static str, String),
    Select(&'static str, String),
    Expand(&'static str, &'static str),
}
#[derive(Clone, Debug)]
enum Step {
    Select(usize),
    Wait,
    Click(Control),
    Global(Global),
    Open,
    BeforePractice,
    FillBefore,
    FillOnly,
    Act(core::Action, bool, bool, Option<String>),
    Rows(Vec<bool>, bool),
    Current(bool),
    PracticeExact,
    Warm,
    SnapshotReset,
    CancelExact,
    ResetExact,
    Raw(&'static str, String),
    SnapshotDrafts,
    Filter,
    Continue,
    ResumeExact,
    Theme(bool),
    Geometry(f32),
    Shot(String, Control),
    BeforeCheck,
    CtrlCheck,
    PureCheck,
    GuardBefore(bool),
    Blocked(bool),
    GuardExact(bool),
    Library,
    Finish,
}
fn actions(
    p: &mut Vec<Step>,
    actions: &[core::Action],
    practice: bool,
    readonly: bool,
    kinds: Option<&[String]>,
) {
    if let Some(k) = kinds {
        assert_eq!(k.len(), actions.len());
    }
    for (n, a) in actions.iter().enumerate() {
        p.push(Step::Act(
            a.clone(),
            practice,
            readonly,
            kinds.map(|v| v[n].clone()),
        ));
    }
}
fn checked(p: &mut Vec<Step>, rows: Vec<bool>, practice: bool, label: &str, both: bool) {
    let pass = rows.iter().all(|x| *x);
    // The real click helper acknowledges only after release was rendered.
    // Rows is evaluated on the next rendered/paced phase; another idle phase
    // adds no event delivery or geometry evidence.
    p.extend([
        Step::Click(Control::Button("check")),
        Step::Rows(rows, practice),
    ]);
    if both {
        p.push(Step::Shot(format!("{label}-first"), Control::Row(0)));
    }
    p.push(Step::Shot(
        format!("{label}-{}", if pass { "last" } else { "failure" }),
        if pass {
            Control::Row(usize::MAX)
        } else {
            Control::Failure
        },
    ));
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Warm,
        Step::SnapshotReset,
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
        Step::Blocked(practice),
        Step::Wait,
        Step::GuardExact(practice),
        Step::Global(Global::Tab("appearance")),
        Step::Global(Global::Samples),
        Step::Wait,
        Step::Blocked(practice),
        Step::Wait,
        Step::GuardExact(practice),
        Step::Global(Global::Samples),
        Step::Global(Global::Tab("data")),
        Step::Global(Global::DataExport),
        Step::Wait,
        Step::Blocked(practice),
        Step::Wait,
        Step::GuardExact(practice),
        Step::Global(Global::FileCancel),
        Step::Global(Global::SettingsClose),
        Step::Wait,
        Step::GuardExact(practice),
    ]);
}
// Initial-state read-only probes are not necessarily valid after a handover.
// Choose a positive current-access requirement from the immutable assignment.
pub(in crate::ui) fn completed_read(s: &Scenario) -> (core::Action, String) {
    let engine::Goal::Nfs { definition } = &s.goal else {
        unreachable!()
    };
    definition
        .requirements
        .iter()
        .find_map(|r| match r {
            work::Requirement::CurrentAccess {
                request:
                    core::Request::Read {
                        client,
                        mountpoint,
                        profile,
                        relative,
                    },
                expected: core::Expected::Read { text },
            } => Some((
                core::Action::ReadExisting {
                    client: client.clone(),
                    mountpoint: mountpoint.clone(),
                    profile: profile.clone(),
                    relative: relative.clone(),
                },
                format!("read:{text}"),
            )),
            _ => None,
        })
        .expect("Each completed assignment declares a positive current read with exact text")
}
fn plan(scenarios: &[Scenario]) -> Vec<Step> {
    let mut p = Vec::new();
    for (n, r) in reviews().iter().enumerate() {
        let s = scenarios.iter().find(|s| s.id == r.id).unwrap();
        assert_eq!(s.unit, UNIT);
        let lesson = s.nfs_teaching.as_ref().unwrap();
        let (read_after_completion, read_kind) = completed_read(s);
        let name = |v: &str| format!("nfs-{n}-{v}");
        p.extend([
            Step::Select(n),
            Step::Wait,
            Step::Click(Control::Card(true)),
            Step::Click(Control::Card(false)),
            Step::Open,
            Step::Shot(name("introduction"), Control::Heading),
            Step::Click(Control::Try),
            Step::Wait,
            Step::Click(Control::Button("reset-example")),
            Step::Wait,
            Step::BeforePractice,
        ]);
        checked(
            &mut p,
            r.practice_initial.clone(),
            true,
            &name("example-initial"),
            false,
        );
        p.extend([
            Step::Click(Control::Button("example-action")),
            Step::Click(Control::Choice(format!(
                "example-action:{}",
                lesson.example_actions.len() - 1
            ))),
            Step::Click(Control::Button("example-action")),
            Step::Click(Control::Choice("example-action:0".into())),
            Step::FillBefore,
            Step::Click(Control::Button("fill")),
            Step::Wait,
            Step::FillOnly,
        ]);
        actions(
            &mut p,
            &lesson
                .example_actions
                .iter()
                .map(|s| s.action.clone())
                .collect::<Vec<_>>(),
            true,
            false,
            None,
        );
        checked(
            &mut p,
            r.practice_worked.clone(),
            true,
            &name("example-worked"),
            true,
        );
        actions(&mut p, &lesson.try_change.actions, true, false, None);
        checked(
            &mut p,
            r.practice_adverse.clone(),
            true,
            &name("example-adverse"),
            false,
        );
        actions(&mut p, &lesson.repair, true, false, None);
        checked(
            &mut p,
            r.practice_worked.clone(),
            true,
            &name("example-repaired"),
            false,
        );
        p.push(Step::PracticeExact);
        if n == 0 {
            modal(&mut p, true);
        }
        p.extend([
            Step::Click(Control::Tab(teaching::View::Assignment)),
            Step::Wait,
            Step::Shot(name("assignment"), Control::Heading),
        ]);
        checked(&mut p, r.initial.clone(), false, &name("initial"), false);
        actions(
            &mut p,
            &r.wrong.actions,
            false,
            false,
            Some(&r.wrong.outcomes),
        );
        checked(&mut p, r.wrong.rows.clone(), false, &name("wrong"), false);
        reset(&mut p);
        actions(
            &mut p,
            &r.partial.actions,
            false,
            false,
            Some(&r.partial.outcomes),
        );
        checked(
            &mut p,
            r.partial.rows.clone(),
            false,
            &name("partial"),
            false,
        );
        reset(&mut p);
        actions(
            &mut p,
            &r.reference.actions,
            false,
            false,
            Some(&r.reference.outcomes),
        );
        checked(
            &mut p,
            r.reference.rows.clone(),
            false,
            &name("reference"),
            true,
        );
        actions(
            &mut p,
            std::slice::from_ref(&read_after_completion),
            false,
            true,
            Some(std::slice::from_ref(&read_kind)),
        );
        p.extend([
            Step::Current(true),
            Step::Shot(name("actual-read"), Control::ReadOutput),
        ]);
        actions(
            &mut p,
            &r.after_pass_break.actions,
            false,
            false,
            Some(&r.after_pass_break.outcomes),
        );
        checked(
            &mut p,
            r.after_pass_break.rows.clone(),
            false,
            &name("invalidated"),
            false,
        );
        actions(
            &mut p,
            &r.repair.actions,
            false,
            false,
            Some(&r.repair.outcomes),
        );
        checked(
            &mut p,
            r.repair.rows.clone(),
            false,
            &name("repaired"),
            false,
        );
        for (a, alt) in r.alternatives.iter().enumerate() {
            reset(&mut p);
            actions(&mut p, &alt.actions, false, false, Some(&alt.outcomes));
            checked(
                &mut p,
                alt.rows.clone(),
                false,
                &name(&format!("alternate-{a}")),
                false,
            );
        }
        p.extend([
            Step::Click(Control::Field("relative")),
            Step::Raw("relative", ASSIGNMENT_RAW.into()),
            Step::BeforeCheck,
            Step::CtrlCheck,
            Step::Wait,
            Step::PureCheck,
            Step::Current(true),
        ]);
        if n == 0 {
            modal(&mut p, false);
        }
        for light in [true, false] {
            p.extend([
                Step::Theme(light),
                Step::Geometry(if light { 1.5 } else { 2. }),
                Step::Click(Control::Field("relative")),
                Step::Raw("relative", ASSIGNMENT_RAW.into()),
                Step::Shot(
                    name(if light {
                        "light150-focused-relative"
                    } else {
                        "dark200-focused-relative"
                    }),
                    Control::Field("relative"),
                ),
            ]);
            actions(
                &mut p,
                std::slice::from_ref(&read_after_completion),
                false,
                true,
                Some(std::slice::from_ref(&read_kind)),
            );
            p.push(Step::Shot(
                name(if light {
                    "light150-read-output"
                } else {
                    "dark200-read-output"
                }),
                Control::ReadOutput,
            ));
            checked(
                &mut p,
                r.reference.rows.clone(),
                false,
                &name(if light { "light150" } else { "dark200" }),
                false,
            );
        }
        p.extend([
            Step::Click(Control::Field("relative")),
            Step::Raw("relative", ASSIGNMENT_RAW.into()),
            Step::Click(Control::Tab(teaching::View::Example)),
            Step::Wait,
            Step::Click(Control::Field("relative")),
            Step::Raw("relative", EXAMPLE_RAW.into()),
            Step::SnapshotDrafts,
            Step::Warm,
            Step::Click(Control::Back),
            Step::Filter,
            Step::Wait,
            Step::Continue,
            Step::Wait,
            Step::ResumeExact,
            Step::Click(Control::Field("relative")),
            Step::Shot(name("resumed-example-draft"), Control::Field("relative")),
            Step::Click(Control::Tab(teaching::View::Assignment)),
            Step::Wait,
            Step::Warm,
            Step::SnapshotReset,
            Step::Click(Control::Reset),
            Step::Click(Control::Cancel),
            Step::Wait,
            Step::CancelExact,
            Step::Click(Control::Challenge),
            Step::Wait,
            Step::Current(true),
            Step::Shot(name("challenge-full-task"), Control::Heading),
            Step::Click(Control::Learn),
            Step::Wait,
        ]);
        reset(&mut p);
        actions(
            &mut p,
            &r.reference.actions,
            false,
            false,
            Some(&r.reference.outcomes),
        );
        checked(
            &mut p,
            r.reference.rows.clone(),
            false,
            &name("after-reset-repaired"),
            false,
        );
        p.extend([Step::Warm, Step::Click(Control::Back), Step::Wait]);
    }
    p.extend([
        Step::Library,
        Step::Wait,
        Step::Shot("nfs-normal-unit-first".into(), Control::Library(0)),
        Step::Shot("nfs-normal-unit-last".into(), Control::Library(3)),
        Step::Finish,
    ]);
    let captures = p.iter().filter(|s| matches!(s, Step::Shot(..))).count();
    assert!(
        p.len() < END - FIRST,
        "Normal NFS plan has {} phases and {captures} captures; phase budget is{}",
        p.len(),
        END - FIRST,
    );
    assert!(captures <= 128, "Normal NFS plan has {captures} captures");
    p
}
pub(in crate::ui) fn phase_bounds(scenarios: &[Scenario]) -> (usize, usize) {
    let p = plan(scenarios);
    (FIRST, FIRST + p.len() - 1)
}
fn key(name: &str) -> egui::Id {
    egui::Id::new(("nfs-normal-lessons", name))
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
fn session(a: &Attempt, practice: bool) -> &work::Session {
    let w = a.workspace.as_ref().unwrap();
    if practice {
        &w.nfs_learning.as_ref().unwrap().session
    } else {
        w.nfs_session.as_ref().unwrap()
    }
}
fn learning(a: &Attempt) -> &learn::Learning {
    a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap()
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
pub(in crate::ui) fn verify_reset_receipts(
    actual: &Option<progress::MigrationReceipts>,
    before: &Value,
) {
    exact(
        actual,
        before,
        "Confirmed Reset retains complete caller migration receipts",
    );
}
pub(in crate::ui) fn verify_reset_recency(clock: u64, visited: u64, before: &Value, id: &str) {
    assert_eq!(
        clock,
        before["clock"].as_u64().unwrap(),
        "Reset preserves the clock; only opening a lesson advances it"
    );
    assert_eq!(
        visited,
        before["attempts"][id]["visited"].as_u64().unwrap(),
        "Reset preserves this lesson's previous visit"
    );
}
#[derive(Clone, Default)]
struct ActionQueue {
    at: usize,
    sub: u8,
    ticks: usize,
}
#[derive(Clone)]
struct Reset {
    progress: Value,
    receipts: Value,
    files: BTreeMap<String, Vec<u8>>,
}
fn click(k: &'static str) -> Op {
    Op::Click(Control::Button(k))
}
fn select(k: &'static str, v: &str) -> Op {
    Op::Select(k, v.into())
}
fn route(a: &core::Action) -> Vec<Op> {
    use core::Action::*;
    match a {
        SetExportsDraft { server, text } => {
            vec![select("server", server), Op::Text("exports", text.clone())]
        }
        SaveExportsDraft { server } => vec![select("server", server), click("save-exports")],
        ApplySavedExports { server } => vec![select("server", server), click("apply-exports")],
        SetFstabDraft { client, text } => {
            vec![select("client", client), Op::Text("fstab", text.clone())]
        }
        SaveFstabDraft { client } => vec![select("client", client), click("save-fstab")],
        MountSaved { client, mountpoint } => vec![
            select("client", client),
            select("mountpoint", mountpoint),
            click("mount-saved"),
        ],
        Unmount { client, mountpoint } => vec![
            select("client", client),
            select("mountpoint", mountpoint),
            click("unmount"),
        ],
        MountExplicit {
            client,
            specification: s,
        } => vec![
            select("client", client),
            select("mountpoint", &s.mountpoint),
            Op::Expand("manual-toggle", "root"),
            select("manual-server", &s.server),
            select("root", &s.root),
            click(if s.access == core::Access::Ro {
                "access-ro"
            } else {
                "access-rw"
            }),
            click("mount-explicit"),
        ],
        ReadExisting {
            client,
            mountpoint,
            profile,
            relative,
        } => vec![
            select("client", client),
            select("mountpoint", mountpoint),
            select("profile", profile),
            Op::Text("relative", relative.clone()),
            click("read"),
        ],
        WriteExisting {
            client,
            mountpoint,
            profile,
            relative,
            text,
        } => vec![
            select("client", client),
            select("mountpoint", mountpoint),
            select("profile", profile),
            Op::Text("relative", relative.clone()),
            Op::Expand("write-toggle", "write-text"),
            Op::Text("write-text", text.clone()),
            click("write"),
        ],
    }
}
fn raw_field<'a>(a: &'a Attempt, practice: bool, k: &str) -> &'a str {
    let t = session(a, practice);
    let state = if practice {
        &learning(a).simulation
    } else {
        let engine::State::Nfs { simulation } = &a.state else {
            unreachable!()
        };
        simulation
    };
    match k {
        "relative" => &t.relative,
        "write-text" => &t.write_text,
        "exports" => {
            let id = if t.server.is_empty() {
                state.inventory().servers[0].id.as_str()
            } else {
                &t.server
            };
            &state
                .runtime()
                .servers
                .iter()
                .find(|s| s.server == id)
                .unwrap()
                .draft
        }
        "fstab" => {
            let id = if t.client.is_empty() {
                state.inventory().clients[0].id.as_str()
            } else {
                &t.client
            };
            &state
                .runtime()
                .clients
                .iter()
                .find(|s| s.client == id)
                .unwrap()
                .draft
        }
        _ => unreachable!(),
    }
}
impl LabApp {
    // Reuse the same native action routing for the closed two-process fixture.
    // This never installs State or bypasses editor/Save/Apply event handling.
    pub(in crate::ui) fn nfs_reopen_action(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        action: &core::Action,
        practice: bool,
    ) -> bool {
        self.nf_action(ctx, id, action, practice, false, None)
    }
    pub(in crate::ui) fn nfs_reopen_check(&mut self, ctx: &egui::Context) -> bool {
        self.nf_click(ctx, &Control::Button("check"))
    }
    pub(in crate::ui) fn nfs_reopen_try(&mut self, ctx: &egui::Context) -> bool {
        self.nf_click(ctx, &Control::Try)
    }
    pub(in crate::ui) fn nfs_reopen_tab(&mut self, ctx: &egui::Context, tab: usize) -> bool {
        let view = match tab {
            1 => teaching::View::Example,
            2 => teaching::View::Assignment,
            _ => panic!("closed NFS tab"),
        };
        self.nf_click(ctx, &Control::Tab(view))
    }
    pub(in crate::ui) fn nfs_reopen_reveal(&mut self, ctx: &egui::Context, result: bool) -> bool {
        self.nf_reveal(
            ctx,
            &if result {
                Control::Row(0)
            } else {
                Control::Field("exports")
            },
        )
    }
    pub(in crate::ui) fn nfs_lessons_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("nfs-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn nfs_lessons_contains_stage(&self, stage: usize) -> bool {
        (FIRST..END).contains(&stage) && stage <= phase_bounds(&self.scenarios).1
    }
    fn nf_put<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, n: &str, v: T) {
        ctx.data_mut(|d| d.insert_temp(key(n), v));
    }
    fn nf_get<T: Clone + Send + Sync + 'static>(&self, ctx: &egui::Context, n: &str) -> T {
        ctx.data(|d| d.get_temp(key(n)))
            .expect("Actual prior GUI observation")
    }
    fn nf_rows(&self, id: &str, practice: bool) -> Vec<bool> {
        let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
        let a = &self.progress.attempts[id];
        if practice {
            learning(a)
                .checks(s.nfs_teaching.as_ref().unwrap())
                .iter()
                .map(|r| r.0)
                .collect()
        } else {
            progress::current_nfs_checks(a, s)
                .iter()
                .map(|r| r.0)
                .collect()
        }
    }
    fn nf_scope(&self) {
        self.nat_private();
        assert!(!self.synthetic);
        assert_eq!((self.scenarios.len(), self.units.len()), (176, 46));
        assert_eq!(
            self.scenarios[150..154]
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
    }
    fn nf_target(&self, ctx: &egui::Context, t: &Control) -> (egui::Rect, egui::Rect, bool) {
        let root = self
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let m = &self.metrics.nfs;
        match t {
            Control::Choice(k) => {
                let r = m.choices.iter().find(|r| &r.0 == k).unwrap();
                (
                    r.1,
                    m.popup.as_ref().unwrap().1.intersect(ctx.content_rect()),
                    true,
                )
            }
            Control::Field(k) => {
                let r = m.fields.iter().find(|r| r.0 == *k).unwrap();
                (r.1, r.4.intersect(root), false)
            }
            Control::Row(n) => {
                let r = if *n == usize::MAX {
                    m.rows.last().unwrap()
                } else {
                    m.rows.iter().find(|r| r.0 == *n).unwrap()
                };
                let clip = m.row_clips.iter().find(|c| c.0 == r.0).unwrap().1;
                (r.2, clip.intersect(root), false)
            }
            Control::Failure => {
                let r = m.rows.iter().find(|r| !r.1).unwrap();
                let clip = m.row_clips.iter().find(|c| c.0 == r.0).unwrap().1;
                (r.2, clip.intersect(root), false)
            }
            Control::ReadOutput => {
                let (r, v) = m.read_output.expect("Actual successful read output");
                (r, v.intersect(root), true)
            }
            Control::Library(n) => (self.metrics.card_actions[*n].0, root, false),
            _ => {
                let r = match t {
                    Control::Card(fav) => {
                        assert_eq!(self.metrics.card_actions.len(), 1);
                        let r = self.metrics.card_actions[0];
                        Some(if *fav { r.1 } else { r.0 })
                    }
                    Control::Try => m
                        .controls
                        .iter()
                        .find(|r| r.0 == "try-example")
                        .map(|r| r.1),
                    Control::Tab(v) => m.tabs.iter().find(|r| r.0 == *v).map(|r| r.1),
                    Control::Button(k) => m.controls.iter().find(|r| r.0 == *k).map(|r| r.1),
                    Control::Heading => m.heading,
                    Control::Reset => self.metrics.reset,
                    Control::Cancel => self.metrics.reset_cancel,
                    Control::Confirm => self.metrics.confirm_reset,
                    Control::Back => self.metrics.back,
                    Control::Challenge => self.metrics.mode_challenge,
                    Control::Learn => self.metrics.mode_learn,
                    _ => unreachable!(),
                }
                .expect("Actual painted NFS control");
                (r, root, false)
            }
        }
    }
    fn nf_reveal(&mut self, ctx: &egui::Context, t: &Control) -> bool {
        let (r, v, inner) = self.nf_target(ctx, t);
        let k = egui::Id::new(("nfs-reveal", self.smoke_stage));
        let mut q = ctx.data(|d| d.get_temp::<Queue>(k)).unwrap_or_default();
        let done = self.nat_reveal(ctx, r, v, inner, &mut q);
        ctx.data_mut(|d| d.insert_temp(k, q));
        done
    }
    fn nf_click(&mut self, ctx: &egui::Context, t: &Control) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(t, Control::Choice(_)) {
            let (r, v, _) = self.nf_target(ctx, t);
            return self.mac_popup_choice(ctx, Some(r), Some(v));
        }
        if !self.nf_reveal(ctx, t) {
            return false;
        }
        let (r, _, _) = self.nf_target(ctx, t);
        self.click_smoke_control(ctx, Some(r))
    }
    fn nf_type(&mut self, ctx: &egui::Context, k: &'static str, text: &str) {
        let r = self.metrics.nfs.fields.iter().find(|r| r.0 == k).unwrap();
        assert!(r.3 && ctx.memory(|m| m.has_focus(r.2)));
        assert!(
            text.len()
                <= if k == "write-text" {
                    4096
                } else if k == "relative" {
                    160
                } else {
                    8192
                }
        );
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
    fn nf_op(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        practice: bool,
        op: &Op,
        sub: &mut u8,
    ) -> bool {
        match op {
            Op::Click(t) => self.nf_click(ctx, t),
            Op::Expand(button, visible) => match *sub {
                0 => {
                    if self.metrics.nfs.fields.iter().any(|r| r.0 == *visible)
                        || self.metrics.nfs.controls.iter().any(|r| r.0 == *visible)
                    {
                        return true;
                    }
                    *sub = 1;
                    false
                }
                1 => {
                    if self.nf_click(ctx, &Control::Button(button)) {
                        *sub = 2;
                    }
                    false
                }
                _ => {
                    self.metrics.nfs.fields.iter().any(|r| r.0 == *visible)
                        || self.metrics.nfs.controls.iter().any(|r| r.0 == *visible)
                }
            },
            Op::Text(k, text) => match *sub {
                0 => {
                    if self.nf_click(ctx, &Control::Field(k)) {
                        *sub = 1;
                    }
                    false
                }
                1 => {
                    self.nf_type(ctx, k, text);
                    *sub = 2;
                    false
                }
                _ => {
                    assert_eq!(raw_field(&self.progress.attempts[id], practice, k), text);
                    true
                }
            },
            Op::Select(k, value) => {
                if *sub == 0 {
                    if self
                        .metrics
                        .nfs
                        .selections
                        .iter()
                        .any(|r| r.0 == *k && r.1 == *value)
                    {
                        return true;
                    }
                    if self.nf_click(ctx, &Control::Button(k)) {
                        *sub = 1;
                    }
                    false
                } else if self.nf_click(ctx, &Control::Choice(format!("{k}:{value}"))) {
                    assert!(
                        self.metrics
                            .nfs
                            .selections
                            .iter()
                            .any(|r| r.0 == *k && r.1 == *value)
                    );
                    true
                } else {
                    false
                }
            }
        }
    }
    fn nf_action(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        action: &core::Action,
        practice: bool,
        readonly: bool,
        kind: Option<&str>,
    ) -> bool {
        let k = egui::Id::new(("nfs-action-queue", self.smoke_stage));
        let mut q = ctx
            .data(|d| d.get_temp::<ActionQueue>(k))
            .unwrap_or_default();
        if q.ticks == 0 {
            self.nf_put(
                ctx,
                "before-action",
                serde_json::to_value(&self.progress).unwrap(),
            );
        }
        q.ticks += 1;
        assert!(
            q.ticks <= 300,
            "NFS action queue exhausted at{} op{} sub{} action{action:?}",
            self.smoke_stage,
            q.at,
            q.sub
        );
        let ops = route(action);
        assert!(ops.len() <= 24);
        if q.at == ops.len() {
            self.nf_action_proof(ctx, id, action, practice, readonly, kind);
            return true;
        }
        if self.nf_op(ctx, id, practice, &ops[q.at], &mut q.sub) {
            q.at += 1;
            q.sub = 0;
            ctx.data_mut(|d| d.remove::<Queue>(egui::Id::new(("nfs-reveal", self.smoke_stage))));
        }
        ctx.data_mut(|d| d.insert_temp(k, q));
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn nf_action_proof(
        &self,
        ctx: &egui::Context,
        id: &str,
        action: &core::Action,
        practice: bool,
        readonly: bool,
        kind: Option<&str>,
    ) {
        let before: Value = self.nf_get(ctx, "before-action");
        let a = &self.progress.attempts[id];
        let t = session(a, practice);
        let ba = &before["attempts"][id];
        let old = if practice {
            &ba["workspace"]["nfs_learning"]["session"]
        } else {
            &ba["workspace"]["nfs_session"]
        };
        let old_model = if practice {
            ba["workspace"]["nfs_learning"]["simulation"].clone()
        } else {
            ba["state"].clone()
        };
        let operation = match action {
            core::Action::SetExportsDraft { .. } => "Edit exports",
            core::Action::SaveExportsDraft { .. } => "Save exports",
            core::Action::ApplySavedExports { .. } => "Apply exports",
            core::Action::SetFstabDraft { .. } => "Edit saved-mount draft",
            core::Action::SaveFstabDraft { .. } => "Save mount document",
            core::Action::MountExplicit { .. } => "Mount explicitly",
            core::Action::MountSaved { .. } => "Mount saved entry",
            core::Action::Unmount { .. } => "Unmount",
            core::Action::ReadExisting { .. } => "Read existing file",
            core::Action::WriteExisting { .. } => "Write existing file",
        };
        let old_entries = old["entries"].as_array().unwrap();
        let identical_draft = matches!(
            action,
            core::Action::SetExportsDraft { .. } | core::Action::SetFstabDraft { .. }
        ) && serde_json::to_value(&t.entries).unwrap() == old["entries"];
        if identical_draft {
            exact(
                &model(a, practice),
                &old_model,
                "Idempotent local editor replacement does not execute a model action",
            );
        } else {
            let entry = t.entries.last().expect("Actual action history");
            assert_eq!(entry.operation, operation);
            assert_eq!(t.entries.len(), (old_entries.len() + 1).min(16));
            let kept = t.entries.len() - 1;
            exact(
                &t.entries[..kept],
                &&old_entries[old_entries.len() - kept..],
                "Original bounded history tail retained",
            );
            if let Some(kind) = kind {
                match kind {
                    "changed" => {
                        assert!(entry.detail.starts_with("Declared operation changed data:"))
                    }
                    "mounted" => assert_eq!(entry.detail, "Virtual mount is present"),
                    "unmounted" => assert_eq!(entry.detail, "Virtual mount removed"),
                    "write" => assert_eq!(entry.detail, "Existing virtual file written"),
                    read if read == "read" || read.starts_with("read:") => {
                        let core::Action::ReadExisting {
                            client,
                            mountpoint,
                            profile,
                            relative,
                        } = action
                        else {
                            panic!("Read outcome requires an actual Read action")
                        };
                        assert_eq!(
                            (&t.client, &t.mountpoint, &t.profile, &t.relative),
                            (client, mountpoint, profile, relative)
                        );
                        let live = if practice {
                            &learning(a).simulation
                        } else {
                            let engine::State::Nfs { simulation } = &a.state else {
                                unreachable!()
                            };
                            simulation
                        };
                        let mount = live
                            .runtime()
                            .mounts
                            .iter()
                            .find(|m| {
                                m.client == *client && m.specification.mountpoint == *mountpoint
                            })
                            .expect("A positive read has a current mount");
                        let path = format!("{}/{relative}", mount.specification.root);
                        let content = live
                            .runtime()
                            .contents
                            .iter()
                            .find(|f| f.server == mount.specification.server && f.path == path)
                            .expect("A positive read names a supplied existing file");
                        assert_eq!(
                            entry.detail, content.text,
                            "Actual history contains exactly the requested virtual file's text"
                        );
                        if let Some(expected) = read.strip_prefix("read:") {
                            assert_eq!(
                                entry.detail, expected,
                                "Completed-task read matches the exact authored current-access expectation"
                            );
                        }
                    }
                    denied if denied.starts_with("denied:") => {
                        let message = match &denied[7..] {
                            "mount_absent" => "no current mount is present",
                            "export_admission" => "the export does not admit this client",
                            "client_read_only" => "the client mount is read-only",
                            "export_read_only" => "the export is read-only",
                            "directory_search" => "directory search permission is denied",
                            "file_permission" => "file permissions deny this operation",
                            _ => panic!("Unknown exact denial stage"),
                        };
                        assert_eq!(entry.detail, format!("Simulated access denied: {message}"));
                    }
                    _ => panic!("Undeclared outcome kind"),
                }
            }
        }
        if readonly {
            exact(
                &model(a, practice),
                &old_model,
                "Read-only operation retains every model field",
            );
        }
        let mut expected = before;
        if practice {
            expected["attempts"][id]["workspace"]["nfs_learning"] =
                serde_json::to_value(learning(a)).unwrap();
        } else {
            expected["attempts"][id]["state"] = serde_json::to_value(&a.state).unwrap();
            expected["attempts"][id]["workspace"]["nfs_session"] = serde_json::to_value(t).unwrap();
        }
        exact(
            &self.progress,
            &expected,
            "Only selected assignment State/Session or separate Learning may change through real controls",
        );
    }
    fn nf_files(&self) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        let mut total = 0usize;
        for (n, e) in std::fs::read_dir(self.path.parent().unwrap())
            .unwrap()
            .enumerate()
        {
            assert!(n < 16);
            let e = e.unwrap();
            let kind = e.file_type().unwrap();
            assert!(kind.is_file() && !kind.is_symlink());
            let bytes = usize::try_from(e.metadata().unwrap().len()).unwrap();
            assert!(bytes <= 2 * 1024 * 1024);
            total = total.checked_add(bytes).unwrap();
            assert!(total <= 8 * 1024 * 1024);
            let name = e.file_name().into_string().unwrap();
            assert!(name.len() <= 256);
            out.insert(
                name,
                progress::read_bounded(&e.path(), 2 * 1024 * 1024).unwrap(),
            );
        }
        out
    }
    fn nf_warm(&mut self, ctx: &egui::Context) {
        let portable = serde_json::to_value(&self.progress).unwrap();
        let caller_receipts = serde_json::to_value(&self.progress.migration_receipts).unwrap();
        self.save();
        assert!(!self.dirty && self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap();
        let mut projected: Value = serde_json::from_slice(&raw).unwrap();
        let raw_receipts = projected
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .unwrap();
        exact(
            &projected,
            &portable,
            "Raw profile projection retains every portable field",
        );
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        exact(
            &loaded,
            &portable,
            "Actual normal loader retains complete progress without installation into live app",
        );
        exact(
            &loaded.migration_receipts,
            &raw_receipts,
            "Actual readback receipts",
        );
        exact(
            &self.progress.migration_receipts,
            &caller_receipts,
            "Save does not stamp caller provenance",
        );
        assert_eq!(
            progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
            raw
        );
        self.nf_put(
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
    fn nf_snapshot_reset(&self, ctx: &egui::Context) {
        self.nf_put(
            ctx,
            "reset",
            Reset {
                progress: serde_json::to_value(&self.progress).unwrap(),
                receipts: serde_json::to_value(&self.progress.migration_receipts).unwrap(),
                files: self.nf_files(),
            },
        );
    }
    fn nf_geometry(&self, ctx: &egui::Context, scale: f32) -> bool {
        let sample = (
            ctx.content_rect(),
            ctx.pixels_per_point(),
            ctx.zoom_factor(),
        );
        assert!(sample.0.is_finite() && sample.1.is_finite() && sample.1 > 0.);
        let k = egui::Id::new(("nfs-geometry", self.smoke_stage));
        let ready = ctx.data_mut(|d| {
            let (old, n) = d
                .get_temp::<(Option<(egui::Rect, f32, f32)>, usize)>(k)
                .unwrap_or((None, 0));
            if old == Some(sample) && (sample.2 - scale).abs() < 0.01 {
                return true;
            }
            let n = n + 1;
            assert!(n <= 20, "NFS measured native geometry must settle");
            d.insert_temp(k, (Some(sample), n));
            false
        });
        if !ready {
            ctx.request_repaint();
        }
        ready
    }
    pub(in crate::ui) fn nfs_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        if self.metrics.nfs.columns && self.metrics.nfs.documents.len() == 2 {
            let exports = self
                .metrics
                .nfs
                .controls
                .iter()
                .find(|r| r.0 == "save-exports")
                .unwrap()
                .1;
            let fstab = self
                .metrics
                .nfs
                .controls
                .iter()
                .find(|r| r.0 == "save-fstab")
                .unwrap()
                .1;
            assert!(
                (exports.top() - fstab.top()).abs() <= 1. / ctx.pixels_per_point(),
                "Actual native NFS Save rows must align, including scrollbar transitions: {exports:?}/{fstab:?}"
            );
        }

        self.nf_scope();
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
        let ticks_key = egui::Id::new(("nfs-step-ticks", stage));
        let ticks = ctx.data_mut(|d| {
            let n = d.get_temp::<u16>(ticks_key).unwrap_or(0) + 1;
            d.insert_temp(ticks_key, n);
            n
        });
        assert!(
            ticks <= 300,
            "NFS stage{stage} exceeded300ticks at{step:?}; root{:?} fields{:?}",
            self.metrics.main_viewport,
            self.metrics.nfs.fields
        );
        match step {
            Step::Select(n) => {
                assert!(self.selected.is_none());
                assert_eq!(self.progress.attempts.len(), *n);
                for id in &IDS[..*n] {
                    let old = self.scenarios.iter().find(|s| s.id == *id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[*id], old));
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
                if !self.nf_click(ctx, t) {
                    return (false, None);
                }
            }
            Step::Global(t) => {
                if !self.nat_queue_for(ctx, vec![GlobalOp::Click(t.clone())], id, false) {
                    return (false, None);
                }
            }
            Step::Open => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(
                    learning(&self.progress.attempts[id]).view,
                    teaching::View::Introduction
                );
                exact(
                    &self.progress.attempts[id].state,
                    &s.initial,
                    "Real normal Start creates authored initial State",
                );
                assert!(self.progress.favorites.contains(id));
            }
            Step::BeforePractice => {
                assert_eq!(
                    learning(&self.progress.attempts[id]).view,
                    teaching::View::Example
                );
                self.nf_put(
                    ctx,
                    "practice-before",
                    serde_json::to_value(&self.progress).unwrap(),
                );
            }
            Step::FillBefore => self.nf_put(
                ctx,
                "fill-before",
                serde_json::to_value(&self.progress).unwrap(),
            ),
            Step::FillOnly => {
                let a = &self.progress.attempts[id];
                let l = learning(a);
                assert_eq!(l.selected_example, 0);
                let mut expected: Value = self.nf_get(ctx, "fill-before");
                let old = &expected["attempts"][id]["workspace"]["nfs_learning"];
                let mut allowed = old["simulation"].clone();
                match &s.nfs_teaching.as_ref().unwrap().example_actions[0].action {
                    core::Action::SetExportsDraft { server, text } => {
                        let row = allowed["runtime"]["servers"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|r| r["server"] == server.as_str())
                            .unwrap();
                        row["draft"] = text.clone().into();
                    }
                    core::Action::SetFstabDraft { client, text } => {
                        let row = allowed["runtime"]["clients"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|r| r["client"] == client.as_str())
                            .unwrap();
                        row["draft"] = text.clone().into();
                    }
                    _ => {}
                }
                exact(
                    &l.simulation,
                    &allowed,
                    "Fill changes only its draft document, never saved/applied/mount/file state",
                );
                exact(
                    &l.session.entries,
                    &old["session"]["entries"],
                    "Fill appends no operation history",
                );
                expected["attempts"][id]["workspace"]["nfs_learning"] =
                    serde_json::to_value(l).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Fill is isolated to separate ungraded Learning",
                );
            }
            Step::Act(action, practice, readonly, kind) => {
                if !self.nf_action(ctx, id, action, *practice, *readonly, kind.as_deref()) {
                    return (false, None);
                }
            }
            Step::Rows(rows, practice) => {
                assert_eq!(self.nf_rows(id, *practice), *rows);
                assert_eq!(
                    self.metrics
                        .nfs
                        .rows
                        .iter()
                        .map(|r| r.1)
                        .collect::<Vec<_>>(),
                    *rows
                );
                assert_eq!(
                    self.metrics.completion,
                    !practice && rows.iter().all(|b| *b)
                );
            }
            Step::Current(pass) => assert_eq!(
                progress::current_solved(&self.progress.attempts[id], &s),
                *pass
            ),
            Step::PracticeExact => {
                let mut expected: Value = self.nf_get(ctx, "practice-before");
                expected["attempts"][id]["workspace"]["nfs_learning"] =
                    serde_json::to_value(learning(&self.progress.attempts[id])).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Practice action/experiment/repair never changes assignment or other progress",
                );
            }
            Step::Warm => self.nf_warm(ctx),
            Step::SnapshotReset => self.nf_snapshot_reset(ctx),
            Step::CancelExact => {
                let before: Reset = self.nf_get(ctx, "reset");
                exact(
                    &self.progress,
                    &before.progress,
                    "Reset Cancel retains every portable field",
                );
                exact(
                    &self.progress.migration_receipts,
                    &before.receipts,
                    "Reset Cancel receipts",
                );
                assert_eq!(self.nf_files(), before.files);
            }
            Step::ResetExact => {
                let before: Reset = self.nf_get(ctx, "reset");
                verify_reset_receipts(&self.progress.migration_receipts, &before.receipts);
                let mut expected = before.progress;
                let a = &self.progress.attempts[id];
                assert!(
                    !self.reset_pending,
                    "Actual Confirm reset closed its pending prompt"
                );
                assert_eq!(self.selected, Some(i));
                verify_reset_recency(self.progress.clock, a.visited, &expected, id);
                assert_eq!(a.scenario_version, s.version);
                assert!(!a.challenge && a.step == 0 && a.hints == 0 && a.transcript.is_empty());
                expected["attempts"][id] = serde_json::to_value(a).unwrap();
                exact(
                    &self.progress,
                    &expected,
                    "Only selected attempt resets; clock, prior visit and all other progress remain exact",
                );
                exact(
                    &a.state,
                    &s.initial,
                    "Actual Reset restores selected initial State",
                );
                exact(
                    &learning(a).simulation,
                    &s.nfs_teaching.as_ref().unwrap().definition.initial,
                    "Only own practice resets",
                );
                assert_eq!(*session(a, false), work::Session::default());
                assert_eq!(*session(a, true), work::Session::default());
                assert_eq!(learning(a).view, teaching::View::Introduction);
                assert_eq!(learning(a).selected_example, 0);
                assert!(!learning(a).checked);
                assert!(!progress::current_solved(a, &s));
            }
            Step::Raw(k, text) => self.nf_type(ctx, k, text),
            Step::SnapshotDrafts => {
                let a = &self.progress.attempts[id];
                assert_eq!(session(a, false).relative, ASSIGNMENT_RAW);
                assert_eq!(session(a, true).relative, EXAMPLE_RAW);
                self.nf_put(ctx, "drafts", serde_json::to_value(&self.progress).unwrap());
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
                if !self.nf_click(ctx, &Control::Card(false)) {
                    return (false, None);
                }
            }
            Step::ResumeExact => {
                assert_eq!(self.selected, Some(i));
                let saved: Warm = self.nf_get(ctx, "warm");
                let mut expected: Value = self.nf_get(ctx, "drafts");
                exact(
                    &saved.portable,
                    &expected,
                    "Warm capture exactly matches both-draft checkpoint",
                );
                assert_eq!(self.progress.clock, expected["clock"].as_u64().unwrap() + 1);
                let a = &self.progress.attempts[id];
                assert_eq!(a.visited, self.progress.clock);
                expected["clock"] = self.progress.clock.into();
                expected["attempts"][id]["visited"] = a.visited.into();
                exact(
                    &self.progress,
                    &expected,
                    "Continue changes only legitimate recency, retaining both raw drafts, views, histories and other attempts",
                );
                exact(
                    &self.progress.migration_receipts,
                    &saved.caller_receipts,
                    "Continue caller receipts",
                );
                let old: Value = serde_json::from_slice(&saved.raw).unwrap();
                exact(
                    &old["migration_receipts"],
                    &saved.raw_receipts,
                    "Recorded raw receipts remain exact",
                );
                self.nf_warm(ctx);
            }
            Step::Theme(light) => {
                let t = theme::inventory()
                    .iter()
                    .find(|t| t.light == *light)
                    .unwrap()
                    .id
                    .clone();
                self.theme_id = t;
                theme::apply(ctx, &self.theme_id);
                assert_eq!(theme::find(&self.theme_id).light, *light);
            }
            Step::Geometry(scale) => {
                if self.scale != *scale {
                    self.scale = *scale;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(946., 929.)));
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                }
                if !self.nf_geometry(ctx, *scale) {
                    return (false, None);
                }
            }
            Step::Shot(name, target) => {
                if !self.nf_geometry(ctx, self.scale) || !self.nf_reveal(ctx, target) {
                    return (false, None);
                }
                if let Control::Field(k) = target {
                    let r = self.metrics.nfs.fields.iter().find(|r| r.0 == *k).unwrap();
                    assert!(r.3 && ctx.memory(|m| m.has_focus(r.2)));
                    let practice = learning(&self.progress.attempts[id]).view
                        == teaching::View::Example
                        && !self.progress.attempts[id].challenge;
                    assert!(!raw_field(&self.progress.attempts[id], practice, k).is_empty());
                }
                if matches!(target, Control::ReadOutput) {
                    assert!(self.metrics.nfs.read_shown);
                }
                let content = ctx.content_rect();
                let ppp = ctx.pixels_per_point();
                let zoom = ctx.zoom_factor();
                let outer = ctx.input(|i| i.viewport().outer_rect);
                let fullscreen = ctx.input(|i| i.viewport().fullscreen);
                println!(
                    "NFS {id} {name} actual_content={content:?} outer={outer:?} fullscreen={fullscreen:?} zoom={zoom} PPP={ppp}; requested size is not observed size"
                );
                return (true, Some(format!("{name}-zoom{zoom:.2}-ppp{ppp:.2}")));
            }
            Step::BeforeCheck => self.nf_put(
                ctx,
                "before-check",
                serde_json::to_value(&self.progress).unwrap(),
            ),
            Step::CtrlCheck => {
                let field = self
                    .metrics
                    .nfs
                    .fields
                    .iter()
                    .find(|r| r.0 == "relative")
                    .unwrap();
                assert!(field.3 && ctx.memory(|m| m.has_focus(field.2)));
                let k = egui::Id::new(("nfs-control-check", stage));
                let sub = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0);
                if sub < 2 {
                    self.pending_events.push(ctrl(egui::Key::Enter, sub == 0));
                    ctx.data_mut(|d| d.insert_temp(k, sub + 1));
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::PureCheck => {
                let before: Value = self.nf_get(ctx, "before-check");
                exact(
                    &self.progress,
                    &before,
                    "Focused Ctrl+Enter checks without executing Read/Write/Save/Apply",
                );
            }
            Step::GuardBefore(practice) => {
                assert_eq!(
                    learning(&self.progress.attempts[id]).view == teaching::View::Example,
                    *practice
                );
                self.nf_snapshot_reset(ctx);
            }
            Step::Blocked(practice) => {
                assert!(self.settings_open || self.data_tools.modal_open());
                assert_eq!(
                    learning(&self.progress.attempts[id]).view == teaching::View::Example,
                    *practice
                );
                assert!(
                    self.metrics.nfs.fields.iter().all(|r| !r.3)
                        && self.metrics.nfs.controls.iter().all(|r| !r.2)
                );
                self.pending_events.extend([
                    egui::Event::Text("blocked native NFS input".into()),
                    ctrl(egui::Key::Enter, true),
                    ctrl(egui::Key::Enter, false),
                ]);
            }
            Step::GuardExact(practice) => {
                assert_eq!(
                    learning(&self.progress.attempts[id]).view == teaching::View::Example,
                    *practice
                );
                let before: Reset = self.nf_get(ctx, "reset");
                exact(
                    &self.progress,
                    &before.progress,
                    "Modal blocks background NFS inputs and actions",
                );
                exact(
                    &self.progress.migration_receipts,
                    &before.receipts,
                    "Modal caller receipts",
                );
                assert_eq!(self.nf_files(), before.files);
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
                    assert!(
                        progress::current_solved(&self.progress.attempts[id], s)
                            && self.progress.favorites.contains(id)
                    );
                }
                self.nf_warm(ctx);
                exact(
                    &self.scenarios,
                    &pack::bundled().unwrap(),
                    "Every normal154 scenario exact; no injected technical fixture",
                );
                println!(
                    "PASS four normal NFS lessons through actual editors/actions, separate practice, two alternatives, current/fresh state and complete warm progress; no host NFS, cold-process, novice or other-OS claim."
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
            Step::Shot(name, _) => Some((name.clone(), FIRST + n)),
            Step::Finish => Some(("finish".into(), FIRST + n)),
            _ => None,
        })
        .collect()
}
