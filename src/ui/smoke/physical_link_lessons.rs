//! Closed actual normal-catalog physical acceptance. All configurations arrive
//! via actual native widgets; no learner State, Progress or checker is installed.
use super::*;
use dlms_learning_lab::{
    physical_link as core, physical_link_assignment as assignment, physical_link_teaching as teach,
    physical_link_workspace as work, script_teaching::View,
};
use serde::Serialize;
const FIRST: usize = 66000;
const END: usize = 66999;
const IDS: [&str; 4] = [
    "ccna-physical-media-span",
    "ccna-physical-duplex-diagnosis",
    "ccna-physical-counter-baseline",
    "ccna-physical-uplink-handover",
];
const UNIT: &str = "physical-uplink-diagnosis";
const SIBLING: &str = "ccna-ipv4-network-route";
const SIBLING_DRAFT: &str = "retained physical sibling draft";
const SCALES: [f32; 5] = [1., 1.15, 1.25, 1.5, 2.];
const QA_BYTES: usize = 32 * 1024 * 1024;
#[derive(Clone)]
struct Fixture {
    source: Scenario,
    references: Vec<core::Configuration>,
    wrong: core::Configuration,
    partial: core::Configuration,
    broken: core::Configuration,
}
fn fixtures() -> &'static [Fixture] {
    static DATA: std::sync::OnceLock<Vec<Fixture>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        let input = include_str!("../../../tests/fixtures/physical-link-author-review.json");
        assert!(input.len() <= 2 * 1024 * 1024);
        let review: serde_json::Value = serde_json::from_str(input).unwrap();
        assert_eq!(review["schema_version"], 1);
        let tasks = review["tasks"].as_array().unwrap();
        assert_eq!(tasks.len(), 4);
        // Read the actual registered pack. Never manufacture or append a source.
        let sources =
            pack::load(include_str!("../../../scenarios/physical-links-v1.json")).unwrap();
        assert_eq!(sources.len(), 4);
        sources
            .into_iter()
            .zip(tasks)
            .enumerate()
            .map(|(n, (source, t))| {
                assert_eq!(source.id, IDS[n]);
                assert_eq!(source.id, t["id"].as_str().unwrap());
                assert_eq!(source.unit, UNIT);
                assert_eq!(source.track, "CCNA");
                let expected: assignment::Definition =
                    serde_json::from_value(t["definition"].clone()).unwrap();
                let lesson: teach::Lesson =
                    serde_json::from_value(t["introduction"].clone()).unwrap();
                let (
                    engine::State::PhysicalLink { simulation },
                    engine::Goal::PhysicalLink { definition },
                ) = (&source.initial, &source.goal)
                else {
                    panic!("Registered physical source kind")
                };
                assert_eq!(simulation, &expected.initial);
                assert_eq!(definition.as_ref(), &expected);
                assert_eq!(source.physical_link_teaching.as_ref(), Some(&lesson));
                assert_eq!(source.difficulty, t["difficulty"].as_str().unwrap());
                let references: Vec<core::Configuration> =
                    serde_json::from_value(t["references"].clone()).unwrap();
                assert_eq!(references.len(), 3);
                Fixture {
                    source,
                    references,
                    wrong: serde_json::from_value(t["wrong"]["configuration"].clone()).unwrap(),
                    partial: serde_json::from_value(t["partial"]["configuration"].clone()).unwrap(),
                    broken: serde_json::from_value(t["break_after_success"].clone()).unwrap(),
                }
            })
            .collect()
    })
}
#[derive(Clone, Copy, Debug)]
enum Config {
    Reference(usize),
    Wrong,
    Partial,
    Broken,
    ExampleAdverse,
    ExampleRepair,
}
#[derive(Clone, Copy, Debug)]
enum Anchor {
    Title,
    Goal,
    Diagram,
    Failure,
    Details,
    Notice,
    Output,
    UnitComplete,
    SiblingInput,
}
#[derive(Clone, Copy, Debug)]
enum Rows {
    All(bool),
    Initial,
    Wrong,
    Partial,
    Pending,
    Broken,
}
#[derive(Clone, Debug)]
enum Step {
    Bootstrap,
    Search(usize),
    SearchSibling,
    SiblingAssignment,
    SiblingAddresses,
    SiblingEdit,
    SiblingBack,
    Start,
    Favorite,
    Tab(View),
    Control(&'static str),
    Header(&'static str),
    SnapshotAssignment,
    AssertAssignment,
    SnapshotExample,
    AssertFill,
    Configure(Config),
    Rows(Rows, bool),
    Details(bool),
    RawObservations,
    CounterBefore,
    CounterDelta,
    ClearCounters,
    SnapshotResume,
    AssertResume,
    SaveReadback,
    SnapshotWhole,
    AssertWhole,
    ModalAttempt,
    ResetExact,
    TypeNumber(usize, &'static str),
    Key(egui::Key, bool, bool),
    Wait(f64),
    Number(usize, &'static str),
    Focus(usize),
    Bounds,
    RejectedPaste,
    Mode(bool),
    Next(usize),
    UnitComplete,
    Theme(bool),
    Geometry(f32, bool),
    Settle(f32, bool),
    Shot(String, Anchor),
    Idle,
    Finish,
}
fn shot(p: &mut Vec<Step>, n: usize, name: &str, anchor: Anchor) {
    p.push(Step::Shot(format!("{}-{name}", IDS[n]), anchor));
}
fn check(p: &mut Vec<Step>, rows: Rows, practice: bool) {
    p.extend([Step::Control("check"), Step::Rows(rows, practice)]);
}
fn configured(p: &mut Vec<Step>, which: Config, rows: Rows, practice: bool) {
    p.extend([Step::Configure(which), Step::Control("apply")]);
    check(p, rows, practice);
}
fn plan() -> &'static [Step] {
    static PLAN: std::sync::OnceLock<Vec<Step>> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| {
        let mut p = vec![
            Step::Bootstrap,
            Step::Geometry(1., false),
            Step::Theme(false),
            Step::Settle(1., false),
            Step::SearchSibling,
            Step::Favorite,
            Step::Start,
            Step::SiblingAssignment,
            Step::SiblingAddresses,
            Step::SiblingEdit,
            Step::Shot(
                "physical-unrelated-sibling-draft-retained".into(),
                Anchor::SiblingInput,
            ),
            Step::SiblingBack,
        ];
        for n in 0..4 {
            p.extend([Step::Search(n), Step::Favorite, Step::Start]);
            shot(&mut p, n, "introduction", Anchor::Title);
            p.extend([Step::SnapshotAssignment, Step::Tab(View::Example)]);
            shot(&mut p, n, "example-initial", Anchor::Goal);
            p.extend([
                Step::SnapshotExample,
                Step::Control("fill-example"),
                Step::AssertFill,
            ]);
            shot(&mut p, n, "example-filled-ungraded", Anchor::Diagram);
            p.extend([Step::Control("apply"), Step::Control("probe")]);
            check(&mut p, Rows::All(true), true);
            p.push(Step::AssertAssignment);
            shot(&mut p, n, "example-verified", Anchor::Output);
            configured(&mut p, Config::ExampleAdverse, Rows::All(false), true);
            shot(&mut p, n, "example-adverse-retry", Anchor::Failure);
            configured(&mut p, Config::ExampleRepair, Rows::All(true), true);
            p.extend([Step::AssertAssignment, Step::Tab(View::Assignment)]);
            shot(&mut p, n, "assignment-initial", Anchor::Goal);
            check(&mut p, Rows::Initial, false);
            if n == 2 {
                p.extend([
                    Step::CounterBefore,
                    Step::Control("probe"),
                    Step::CounterDelta,
                    Step::RawObservations,
                ]);
                shot(&mut p, n, "fresh-counter-evidence", Anchor::Output);
            }
            configured(&mut p, Config::Wrong, Rows::Wrong, false);
            shot(&mut p, n, "wrong-specific-feedback", Anchor::Failure);
            if n == 2 {
                p.extend([
                    Step::Configure(Config::Partial),
                    Step::Control("apply"),
                    Step::ClearCounters,
                ]);
                check(&mut p, Rows::Partial, false);
            } else {
                configured(&mut p, Config::Partial, Rows::Partial, false);
            }
            shot(&mut p, n, "partial-feedback", Anchor::Failure);
            if n == 2 {
                p.push(Step::Control("probe"));
            }
            configured(&mut p, Config::Reference(0), Rows::All(true), false);
            shot(&mut p, n, "reference-verified", Anchor::Diagram);
            p.push(Step::Details(true));
            shot(&mut p, n, "all-objective-details", Anchor::Details);
            p.push(Step::Details(false));
            p.push(Step::Configure(Config::Broken));
            check(&mut p, Rows::Pending, false);
            shot(
                &mut p,
                n,
                "pending-edit-invalidates-success",
                Anchor::Failure,
            );
            p.push(Step::Control("apply"));
            check(&mut p, Rows::Broken, false);
            shot(
                &mut p,
                n,
                "broken-state-invalidates-success",
                Anchor::Failure,
            );
            p.extend([
                Step::SnapshotResume,
                Step::SaveReadback,
                Step::Header("back"),
                Step::Search(n),
                Step::Start,
                Step::AssertResume,
                Step::SnapshotWhole,
                Step::Header("reset"),
                Step::ModalAttempt,
                Step::Header("cancel"),
                Step::AssertWhole,
                Step::Header("reset"),
                Step::Header("confirm"),
                Step::ResetExact,
            ]);
            shot(&mut p, n, "reset-introduction", Anchor::Title);
            p.push(Step::Tab(View::Assignment));
            configured(&mut p, Config::Reference(1), Rows::All(true), false);
            shot(&mut p, n, "alternate-one-verified", Anchor::Diagram);
            configured(&mut p, Config::Reference(2), Rows::All(true), false);
            shot(&mut p, n, "alternate-two-verified", Anchor::Diagram);
            if n == 0 {
                p.extend([
                    Step::TypeNumber(0, "24"),
                    Step::Wait(1.1),
                    Step::Key(egui::Key::Z, false, true),
                    Step::Number(0, "12"),
                    Step::Key(egui::Key::Z, true, true),
                    Step::Number(0, "24"),
                    Step::Key(egui::Key::Tab, false, false),
                    Step::Focus(1),
                    Step::Key(egui::Key::Tab, true, false),
                    Step::Focus(0),
                    Step::SnapshotWhole,
                    Step::RejectedPaste,
                    Step::AssertWhole,
                    Step::TypeNumber(0, "1025"),
                    Step::SnapshotWhole,
                    Step::Bounds,
                    Step::AssertWhole,
                ]);
                shot(
                    &mut p,
                    n,
                    "bounded-sample-specific-feedback",
                    Anchor::Notice,
                );
                p.push(Step::TypeNumber(0, "12"));
            }
            p.push(Step::Mode(true));
            shot(&mut p, n, "challenge-full-assignment", Anchor::Goal);
            p.push(Step::Mode(false));
            if n < 3 {
                p.extend([Step::Next(n + 1), Step::Header("back")]);
            } else {
                p.push(Step::UnitComplete);
                shot(&mut p, n, "unit-complete", Anchor::UnitComplete);
                p.push(Step::Header("back"));
            }
        }
        for n in [1, 3] {
            p.extend([
                Step::Search(n),
                Step::Start,
                Step::Tab(View::Assignment),
                Step::SnapshotWhole,
            ]);
            for narrow in [false, true] {
                for light in [false, true] {
                    for scale in SCALES {
                        p.extend([
                            Step::Geometry(scale, narrow),
                            Step::Theme(light),
                            Step::Settle(scale, narrow),
                            Step::Shot(
                                format!(
                                    "physical-alignment-{n}-{}-{}-{:.0}",
                                    if narrow { "narrow" } else { "wide" },
                                    if light { "light" } else { "dark" },
                                    scale * 100.
                                ),
                                Anchor::Diagram,
                            ),
                        ]);
                    }
                }
            }
            p.extend([
                Step::AssertWhole,
                Step::Geometry(1., false),
                Step::Theme(false),
                Step::Settle(1., false),
                Step::Header("back"),
            ]);
        }
        p.extend([Step::Idle, Step::Finish]);
        assert!(p.len() <= END - FIRST);
        assert_eq!(
            p.iter().filter(|s| matches!(s, Step::Shot(..))).count(),
            108
        );
        p
    })
}
#[derive(Clone, Debug)]
enum Op {
    Choice(String, String),
    Toggle(String),
    Expand(String, String),
}
#[derive(Clone, Default)]
struct Cursor {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    done: bool,
}
fn ability_key(id: &str, a: core::Ability) -> String {
    format!("advertise:{id}:{:?}:{:?}", a.rate, a.duplex)
}
fn config_ops(d: &assignment::Draft, target: &core::Configuration, state: &core::State) -> Cursor {
    let mut c = Cursor::default();
    let mut push = |op| {
        assert!(c.ops.len() < 300);
        c.ops.push(op);
    };
    for (link, cable) in &target.cables {
        if d.configuration.cables[link] != *cable {
            if link == &d.selected_link {
                push(Op::Choice("cable".into(), cable.clone()));
            } else {
                push(Op::Expand("other-cables".into(), format!("cable:{link}")));
                push(Op::Choice(format!("cable:{link}"), cable.clone()));
            }
        }
    }
    let selected = &state.topology().links[&d.selected_link];
    for (id, value) in &target.ports {
        let old = &d.configuration.ports[id];
        if old == value {
            continue;
        }
        if id != &selected.a && id != &selected.b {
            push(Op::Expand("other-interfaces".into(), format!("up:{id}")));
        }
        if old.admin_up != value.admin_up {
            push(Op::Toggle(format!("up:{id}")));
        }
        if old.profile != value.profile {
            push(Op::Choice(
                format!("profile:{id}"),
                physical_link_workspace::profile(value.profile).into(),
            ));
        }
        match &value.negotiation {
            core::Negotiation::Fixed { ability } => {
                push(Op::Choice(format!("mode:{id}"), "Fixed".into()));
                push(Op::Choice(
                    format!("rate:{id}"),
                    physical_link_workspace::rate(ability.rate).into(),
                ));
                push(Op::Choice(
                    format!("duplex:{id}"),
                    physical_link_workspace::duplex(ability.duplex).into(),
                ));
            }
            core::Negotiation::Auto { advertise } => {
                // The native chooser treats even reselecting Auto as a change,
                // which resets advertisements. Keep a current Auto mode intact.
                if !matches!(old.negotiation, core::Negotiation::Auto { .. }) {
                    push(Op::Choice(format!("mode:{id}"), "Auto".into()));
                }
                // Skipping an already selected Auto mode leaves its existing
                // advertisements intact. Switching Fixed -> Auto initializes all
                // supplied abilities in the actual native mode control.
                let current = match &old.negotiation {
                    core::Negotiation::Auto { advertise } => advertise,
                    core::Negotiation::Fixed { .. } => &state.topology().ports[id].abilities,
                };
                for a in &state.topology().ports[id].abilities {
                    if current.contains(a) != advertise.contains(a) {
                        let key = ability_key(id, *a);
                        push(Op::Expand(format!("advertisement:{id}"), key.clone()));
                        push(Op::Toggle(key));
                    }
                }
            }
        }
    }
    c
}
fn raw<T: Serialize + ?Sized>(value: &T) -> Vec<u8> {
    let size = work::size(value, 8 * 1024 * 1024).expect("Private bounded QA serialization");
    struct Bound {
        bytes: Vec<u8>,
        cap: usize,
    }
    impl std::io::Write for Bound {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.bytes.len().saturating_add(b.len()) > self.cap {
                return Err(std::io::Error::other(
                    "Private QA snapshot changed while serializing",
                ));
            }
            self.bytes.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut b = Bound {
        bytes: Vec::with_capacity(size),
        cap: size,
    };
    serde_json::to_writer(&mut b, value).unwrap();
    assert_eq!(b.bytes.len(), size);
    b.bytes
}
#[derive(Serialize)]
struct Resume<'a> {
    scenario_version: u32,
    state: &'a engine::State,
    step: usize,
    challenge: bool,
    hints: usize,
    transcript: &'a [String],
    ipv4: &'a Option<dlms_learning_lab::ipv4::Lesson>,
    workspace: &'a Option<model::WorkspaceData>,
}
#[derive(Serialize)]
struct Other<'a> {
    schema_version: u32,
    clock: u64,
    favorites: &'a std::collections::BTreeSet<String>,
    attempts: std::collections::BTreeMap<&'a str, &'a Attempt>,
    receipts: &'a Option<progress::MigrationReceipts>,
}
fn other<'a>(p: &'a Progress, id: &str) -> Other<'a> {
    let Progress {
        schema_version: _,
        clock: _,
        favorites: _,
        attempts: _,
        migration_receipts: _,
    } = p;
    Other {
        schema_version: p.schema_version,
        clock: p.clock,
        favorites: &p.favorites,
        attempts: p
            .attempts
            .iter()
            .filter(|(name, _)| name.as_str() != id)
            .map(|(name, a)| (name.as_str(), a))
            .collect(),
        receipts: &p.migration_receipts,
    }
}
fn resumed(a: &Attempt) -> Vec<u8> {
    let Attempt {
        scenario_version,
        state,
        step,
        challenge,
        hints,
        transcript,
        visited: _,
        ipv4,
        workspace,
    } = a;
    raw(&Resume {
        scenario_version: *scenario_version,
        state,
        step: *step,
        challenge: *challenge,
        hints: *hints,
        transcript,
        ipv4,
        workspace,
    })
}
#[derive(Clone, Copy, Default)]
struct Geometry {
    sample: Option<(egui::Rect, u32, u32, Option<bool>)>,
    same: usize,
    tries: usize,
    last_root: Option<u64>,
}
type IdleObservation = (
    usize,
    (usize, usize, usize),
    [usize; 2],
    crate::ui::completion_index::Counters,
);
#[derive(Clone, Default)]
struct Qa {
    stage: usize,
    finish_emissions: u8,
    last_tick_root: Option<u64>,
    completed_cursor: Option<Cursor>,
    completed_configs: usize,
    requested_geometry: Option<(f32, bool)>,
    phase: u8,
    tries: usize,
    stage_began: Option<f64>,
    cursor: Option<Cursor>,
    assignment: Vec<u8>,
    example: Vec<u8>,
    resume: Vec<u8>,
    whole: Vec<u8>,
    others: Vec<u8>,
    receipts: Vec<u8>,
    catalog: Vec<u8>,
    units: Vec<u8>,
    sibling: Vec<u8>,
    counters: std::collections::BTreeMap<String, core::Counters>,
    geometry: Geometry,
    wait: Option<f64>,
    tags: std::collections::BTreeSet<String>,
    idle: Option<IdleObservation>,
}
fn expected(n: usize, kind: Rows) -> Vec<bool> {
    match kind {
        Rows::All(true) => vec![true; if n == 3 { 11 } else { 9 }],
        Rows::All(false) => unreachable!(),
        Rows::Initial => match n {
            0 => vec![true, true, true, true, true, false, false, false, false],
            1 => vec![true, true, true, true, true, true, false, false, false],
            2 => vec![true, true, true, true, true, true, true, false, false],
            3 => vec![
                true, true, true, true, true, false, false, false, false, false, false,
            ],
            _ => unreachable!(),
        },
        Rows::Wrong => match n {
            0 => vec![true, true, true, true, true, false, false, false, false],
            1 | 2 => vec![true, true, true, true, true, true, false, false, false],
            3 => vec![
                true, true, false, true, true, true, true, true, true, true, true,
            ],
            _ => unreachable!(),
        },
        Rows::Partial => match n {
            0 => vec![true, true, true, true, true, false, false, false, false],
            1 => vec![true, true, true, true, true, true, false, false, false],
            2 => vec![true, true, true, true, true, true, true, false, false],
            3 => vec![
                true, true, true, true, true, false, false, true, true, true, true,
            ],
            _ => unreachable!(),
        },
        Rows::Pending => {
            let mut rows = vec![true; if n == 3 { 11 } else { 9 }];
            rows[0] = false;
            rows
        }
        Rows::Broken => {
            let mut rows = vec![true; if n == 3 { 11 } else { 9 }];
            let end = rows.len();
            rows[end - 3..].fill(false);
            rows
        }
    }
}
#[derive(Clone, Copy, Default)]
pub(in crate::ui) struct Facts {
    pub stage: usize,
    pub finish_emissions: u8,
    pub completed_configs: usize,
    pub tags: usize,
    pub requested_geometry: Option<(f32, bool)>,
    pub geometry_same: usize,
}
pub(in crate::ui) fn facts(ctx: &egui::Context) -> Option<Facts> {
    ctx.data(|d| d.get_temp(egui::Id::new("physical-native-facts")))
}
#[derive(Clone, Copy)]
enum Slot {
    Assignment,
    Example,
    Resume,
    Whole,
    Others,
    Receipts,
    Catalog,
    Units,
    Sibling,
}
impl Qa {
    fn allocated(&self) -> usize {
        [
            &self.assignment,
            &self.example,
            &self.resume,
            &self.whole,
            &self.others,
            &self.receipts,
            &self.catalog,
            &self.units,
            &self.sibling,
        ]
        .into_iter()
        .map(|v| v.capacity())
        .sum::<usize>()
            + 8 * 1024 * 1024
    }
    fn snapshot<T: Serialize + ?Sized>(&mut self, slot: Slot, value: &T) {
        let size =
            work::size(value, 8 * 1024 * 1024).expect("Fixed QA value admits before copying");
        assert!(
            self.allocated()
                .saturating_add(size)
                .saturating_add(8 * 1024 * 1024)
                <= QA_BYTES,
            "QA aggregate reserve before snapshot copy"
        );
        let bytes = raw(value);
        assert_eq!(size, bytes.len());
        match slot {
            Slot::Assignment => self.assignment = bytes,
            Slot::Example => self.example = bytes,
            Slot::Resume => self.resume = bytes,
            Slot::Whole => self.whole = bytes,
            Slot::Others => self.others = bytes,
            Slot::Receipts => self.receipts = bytes,
            Slot::Catalog => self.catalog = bytes,
            Slot::Units => self.units = bytes,
            Slot::Sibling => self.sibling = bytes,
        }
    }
}
fn simulation(a: &Attempt, practice: bool) -> &core::State {
    if practice {
        &a.workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .current
    } else {
        let engine::State::PhysicalLink { simulation } = &a.state else {
            panic!("Actual physical task")
        };
        simulation
    }
}
#[derive(Serialize)]
struct AssignmentWork<'a> {
    scenario_version: u32,
    state: &'a engine::State,
    step: usize,
    challenge: bool,
    hints: usize,
    transcript: &'a [String],
    visited: u64,
    ipv4: &'a Option<dlms_learning_lab::ipv4::Lesson>,
    workspace_version: u32,
    command: &'a str,
    checked: bool,
    draft: &'a Option<assignment::Draft>,
}
fn assignment_part(a: &Attempt) -> AssignmentWork<'_> {
    let Attempt {
        scenario_version,
        state,
        step,
        challenge,
        hints,
        transcript,
        visited,
        ipv4,
        workspace,
    } = a;
    let w = workspace.as_ref().unwrap();
    assert!(
        !work::foreign(w),
        "Assignment isolation admits no other family metadata"
    );
    AssignmentWork {
        scenario_version: *scenario_version,
        state,
        step: *step,
        challenge: *challenge,
        hints: *hints,
        transcript,
        visited: *visited,
        ipv4,
        workspace_version: w.version,
        command: &w.command,
        checked: w.checked,
        draft: &w.physical_link_draft,
    }
}
fn assignment_bytes(a: &Attempt) -> Vec<u8> {
    raw(&assignment_part(a))
}
fn ctrl(key: egui::Key, shift: bool, command: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: command,
            command,
            shift,
            ..Default::default()
        },
    }
}
impl LabApp {
    pub(in crate::ui) fn physical_stage(
        smoke: bool,
        developer: bool,
        name: Option<&str>,
    ) -> Option<usize> {
        (smoke && developer && name == Some("physical-link-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn physical_capture_active(&self) -> bool {
        self.smoke && self.developer && (FIRST..=END).contains(&self.smoke_stage)
    }
    fn physical_index(&self, n: usize) -> usize {
        self.scenarios
            .iter()
            .position(|s| s.id == IDS[n])
            .expect("Reviewed physical source")
    }
    fn physical_control(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let r = self
            .metrics
            .physical
            .controls
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1);
        self.click_visible_smoke_control(ctx, r)
    }
    fn physical_header(&mut self, ctx: &egui::Context, key: &str) -> bool {
        let r = match key {
            "confirm" => self.metrics.confirm_reset,
            "cancel" => self.metrics.reset_cancel,
            _ => self
                .metrics
                .workspace_header_controls
                .iter()
                .find(|r| r.0 == key)
                .map(|r| r.1),
        };
        self.click_visible_smoke_control(ctx, r)
    }
    fn physical_expand(&mut self, ctx: &egui::Context, key: &str, probe: &str) -> bool {
        let click = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(click)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if self.metrics.physical.controls.iter().any(|r| r.0 == probe) {
            return true;
        }
        let r = self
            .metrics
            .physical
            .expanders
            .iter()
            .find(|r| r.0 == key)
            .expect("Discoverable optional native tools");
        assert!(r.2);
        self.click_visible_smoke_control(ctx, Some(r.1))
    }
    fn physical_popup_click(
        &mut self,
        ctx: &egui::Context,
        rect: Option<egui::Rect>,
        clip: Option<egui::Rect>,
    ) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let rect = rect.expect("Actual native choice");
        let clip = clip
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        if !smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            let delta = if rect.bottom() > clip.bottom() {
                clip.bottom() - rect.bottom()
            } else if rect.top() < clip.top() {
                clip.top() - rect.top()
            } else {
                panic!("Popup row is horizontally clipped: {rect:?} vs{clip:?}")
            };
            self.pending_events.extend([
                // Popup clipping can span the screen horizontally even when
                // its actual scroll area is narrow. Deliver the wheel inside
                // the native row's horizontal span and visible clip height.
                egui::Event::PointerMoved(egui::pos2(
                    rect.center().x.clamp(clip.left(), clip.right()),
                    clip.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ]);
            self.started = Instant::now();
            ctx.request_repaint();
            return false;
        }
        self.click_smoke_control(ctx, Some(rect))
    }
    fn physical_cursor(&mut self, ctx: &egui::Context, c: &mut Cursor) -> bool {
        if c.done {
            return true;
        }
        if c.at >= c.ops.len() {
            c.done = true;
            return true;
        }
        let op = c.ops[c.at].clone();
        let completed = match op {
            Op::Toggle(key) => self.physical_control(ctx, &key),
            Op::Expand(key, probe) => self.physical_expand(ctx, &key, &probe),
            Op::Choice(key, value) => {
                if c.phase == 0 {
                    if self.physical_control(ctx, &key) {
                        c.phase = 1;
                    }
                    false
                } else {
                    let click = egui::Id::new(("native-click-phases", self.smoke_stage));
                    let completed = if ctx.data(|d| d.get_temp::<u8>(click)).unwrap_or(0) >= 2 {
                        self.click_smoke_control(ctx, None)
                    } else {
                        let r = self
                            .metrics
                            .physical
                            .choices
                            .iter()
                            .find(|r| r.0 == key && r.1 == value && r.3)
                            .unwrap_or_else(|| {
                                panic!("Actual enabled physical choice {key}/{value}")
                            });
                        let clip = self
                            .metrics
                            .physical
                            .choice_clips
                            .iter()
                            .find(|r| r.0 == key && r.1 == value)
                            .map(|r| r.2);
                        self.physical_popup_click(ctx, Some(r.2), clip)
                    };
                    if completed {
                        c.phase = 0;
                    }
                    completed
                }
            }
        };
        if completed {
            c.at += 1;
            let id = egui::Id::new(("bounded-native-scroll", self.smoke_stage));
            ctx.data_mut(|d| d.remove::<usize>(id));
        }
        c.done = c.at == c.ops.len();
        c.done
    }
    fn physical_target(&self, n: usize, c: Config) -> &core::Configuration {
        let f = &fixtures()[n];
        match c {
            Config::Reference(i) => &f.references[i],
            Config::Wrong => &f.wrong,
            Config::Partial => &f.partial,
            Config::Broken => &f.broken,
            Config::ExampleAdverse => &f.source.physical_link_teaching.as_ref().unwrap().try_change,
            Config::ExampleRepair => &f.source.physical_link_teaching.as_ref().unwrap().repair,
        }
    }
    fn physical_tab(&mut self, ctx: &egui::Context, target: View) -> bool {
        let click = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(click)).unwrap_or(0) >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        let r = self
            .metrics
            .physical
            .tabs
            .iter()
            .find(|r| r.0 == target)
            .expect("Actual assistance tab")
            .1;
        self.click_visible_smoke_control(ctx, Some(r))
    }
    fn physical_number(&mut self, ctx: &egui::Context, qa: &mut Qa, n: usize, text: &str) -> bool {
        if qa.phase == 0 {
            let r = self
                .metrics
                .physical
                .numbers
                .iter()
                .find(|r| r.0 == n)
                .unwrap()
                .1;
            if self.click_visible_smoke_control(ctx, Some(r)) {
                qa.phase = 1;
            }
            return false;
        }
        if qa.phase == 1 {
            self.pending_events.extend([
                ctrl(egui::Key::A, false, true),
                egui::Event::Paste(text.into()),
            ]);
            qa.phase = 2;
            ctx.request_repaint();
            return false;
        }
        assert_eq!(self.physical_presentation.sample_values()[n], text);
        true
    }
    fn physical_geometry(
        &self,
        ctx: &egui::Context,
        qa: &mut Qa,
        scale: f32,
        narrow: bool,
    ) -> bool {
        let content = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let zoom = ctx.zoom_factor();
        let fullscreen = ctx.input(|i| i.viewport().fullscreen);
        let native = ctx
            .input(|i| i.viewport().native_pixels_per_point)
            .unwrap_or(1.);
        let pixels = content.size() * ppp;
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let ready = (zoom - scale).abs() < 0.01
            && (ppp - native * scale).abs() < 0.01
            && fullscreen == Some(!narrow)
            && if narrow {
                (650. ..=1300.).contains(&pixels.x)
            } else {
                pixels.x >= 1500.
            };
        let sample = (content, ppp.to_bits(), zoom.to_bits(), fullscreen);
        if qa.geometry.last_root != Some(root) {
            qa.geometry.last_root = Some(root);
            qa.geometry.tries += 1;
            qa.geometry.same = if ready && qa.geometry.sample == Some(sample) {
                qa.geometry.same + 1
            } else {
                usize::from(ready)
            };
            qa.geometry.sample = Some(sample);
        }
        assert!(
            qa.geometry.tries <= 60,
            "Actual native geometry did not settle: {pixels:?}/{ppp}/{zoom}/{fullscreen:?}"
        );
        if ready && qa.geometry.same >= 2 {
            println!(
                "MEASURE physical geometry: pixels{pixels:?}, zoom{zoom:.2}, nativePPP{native:.2}, fullscreen{fullscreen:?}"
            );
            true
        } else {
            ctx.request_repaint();
            false
        }
    }
    fn physical_anchor(&self, anchor: Anchor) -> Option<egui::Rect> {
        match anchor {
            Anchor::Title => self.metrics.physical.title,
            Anchor::Goal => self.metrics.physical.goal,
            Anchor::Diagram => self.metrics.physical.topology,
            Anchor::Failure => self.metrics.physical.first_failure,
            Anchor::Details => self.metrics.physical.details_header.map(|head| {
                self.metrics
                    .physical
                    .detail_rows
                    .last()
                    .map_or(head, |row| head.union(row.2))
            }),
            Anchor::Notice => self.metrics.physical.notice,
            Anchor::Output => self.metrics.physical.output,
            Anchor::UnitComplete => self.metrics.unit_complete,
            Anchor::SiblingInput => self.metrics.inputs.first().copied(),
        }
    }
    pub(in crate::ui) fn physical_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        assert!(self.physical_capture_active());
        let offset = stage.checked_sub(FIRST).unwrap();
        let step = plan()
            .get(offset)
            .expect("Closed physical native stage")
            .clone();
        let key = egui::Id::new("physical-native-controller");
        let mut qa = ctx
            .data_mut(|d| d.remove_temp::<Qa>(key))
            .unwrap_or_default();
        let root_frame = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        if qa.last_tick_root == Some(root_frame) {
            ctx.data_mut(|d| d.insert_temp(key, qa));
            return (false, None);
        }
        qa.last_tick_root = Some(root_frame);
        if qa.stage != stage {
            qa.stage = stage;
            qa.phase = 0;
            qa.tries = 0;
            qa.stage_began = Some(ctx.input(|i| i.time));
            qa.cursor = None;
            qa.geometry = Geometry::default();
            qa.wait = None;
        }
        qa.tries += 1;
        let elapsed = ctx.input(|i| i.time) - qa.stage_began.expect("Step clock initialized");
        assert!(
            qa.tries <= 900 && elapsed.is_finite() && (0. ..330.).contains(&elapsed),
            "Physical native stage {offset}/{step:?} exceeded bounded event/scroll wait: {} polls, {elapsed:.2}s",
            qa.tries
        );
        let result = self.physical_step(ctx, &mut qa, step);
        assert!(self.pending_events.len() <= 300);
        assert!(qa.allocated().saturating_add(8 * 1024 * 1024) <= QA_BYTES);
        if let Some(tag) = &result.1 {
            assert!(
                qa.tags.len() < 128 && qa.tags.insert(tag.clone()),
                "Unique capture tag within fixed cap"
            );
        }
        let facts = Facts {
            stage,
            finish_emissions: qa.finish_emissions,
            completed_configs: qa.completed_configs,
            tags: qa.tags.len(),
            requested_geometry: qa.requested_geometry,
            geometry_same: qa.geometry.same,
        };
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("physical-native-facts"), facts);
            d.insert_temp(key, qa);
        });
        result
    }
    #[cfg(test)]
    pub(in crate::ui) fn physical_completed_reentry(&mut self, ctx: &egui::Context) -> bool {
        let key = egui::Id::new("physical-native-controller");
        let mut qa = ctx.data_mut(|d| d.remove_temp::<Qa>(key)).unwrap();
        let result = if let Some(cursor) = qa.completed_cursor.as_mut() {
            let before = self.pending_events.len();
            assert!(cursor.done);
            assert!(self.physical_cursor(ctx, cursor));
            assert_eq!(self.pending_events.len(), before);
            true
        } else {
            false
        };
        ctx.data_mut(|d| d.insert_temp(key, qa));
        result
    }
    fn physical_step(
        &mut self,
        ctx: &egui::Context,
        qa: &mut Qa,
        step: Step,
    ) -> (bool, Option<String>) {
        let selected = self.selected.map(|i| self.scenarios[i].id.as_str());
        let n = selected
            .and_then(|id| IDS.iter().position(|s| *s == id))
            .unwrap_or(0);
        match step {
            Step::Bootstrap => {
                assert!(
                    self.progress.attempts.is_empty(),
                    "Fresh private acceptance profile only"
                );
                assert_eq!(
                    one_off_boundary::before_len(&self.scenarios),
                    176,
                    "Exact normal catalog source boundary"
                );
                assert_eq!(
                    one_off_boundary::before_len(&self.units),
                    46,
                    "Exact normal unit source boundary"
                );
                assert_eq!(dlms_learning_lab::shell::EVALUATOR_REVISION, 12);
                assert_eq!(
                    self.scenarios
                        .iter()
                        .filter(|s| matches!(s.initial, engine::State::PhysicalLink { .. }))
                        .count(),
                    4
                );
                assert!(
                    self.scenarios[..171]
                        .iter()
                        .all(|s| s.physical_link_teaching.is_none())
                );
                assert_eq!(
                    self.scenarios[171..175]
                        .iter()
                        .map(|s| s.id.as_str())
                        .collect::<Vec<_>>(),
                    IDS
                );
                let unit = self
                    .units
                    .iter()
                    .find(|u| u.id == UNIT)
                    .expect("Actual registered unit");
                assert_eq!(
                    unit.labs.iter().map(String::as_str).collect::<Vec<_>>(),
                    IDS
                );
                for n in 0..4 {
                    let i = self.physical_index(n);
                    assert_eq!(
                        raw(&self.scenarios[i]),
                        raw(&fixtures()[n].source),
                        "Live source must equal actual registered pack"
                    );
                }
                assert_eq!(
                    self.scenarios[175].id, "ccna-private-office-plan",
                    "Preserve exact physical175 prefix plus separately reviewed private-addressing task"
                );
                qa.snapshot(Slot::Catalog, &self.scenarios);
                qa.snapshot(Slot::Units, &self.units);
                println!(
                    "Normal physical native acceptance: four registered authored tasks; sources/units are observed, never appended or replaced."
                );
            }
            Step::SearchSibling => {
                let wanted = &self
                    .scenarios
                    .iter()
                    .find(|s| s.id == SIBLING)
                    .expect("Existing sibling source")
                    .title;
                if qa.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.library_search) {
                        qa.phase = 1;
                    }
                    return (false, None);
                }
                if qa.phase == 1 {
                    self.pending_events.extend([
                        ctrl(egui::Key::A, false, true),
                        egui::Event::Paste(wanted.clone()),
                    ]);
                    qa.phase = 2;
                    return (false, None);
                }
                assert_eq!(&self.filter.search, wanted);
                assert_eq!(self.metrics.card_actions.len(), 1);
            }
            Step::SiblingAssignment => {
                assert_eq!(self.scenarios[self.selected.unwrap()].id, SIBLING);
                assert!(self.progress.favorites.contains(SIBLING));
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            Step::SiblingAddresses => {
                if !self.click_visible_smoke_control(ctx, self.metrics.routing_address_tab) {
                    return (false, None);
                }
                let w = self.progress.attempts[SIBLING].workspace.as_ref().unwrap();
                assert!(!w.routing_v4_draft.as_ref().unwrap().show_routes);
            }
            Step::SiblingEdit => {
                if qa.phase == 0 {
                    if self.click_visible_smoke_control(ctx, self.metrics.inputs.first().copied()) {
                        qa.phase = 1;
                    }
                    return (false, None);
                }
                if qa.phase == 1 {
                    let input = self
                        .metrics
                        .focus
                        .expect("Rendered existing address input ID");
                    assert_eq!(
                        ctx.memory(|m| m.focused()),
                        Some(input),
                        "Genuine existing address editor focus"
                    );
                    self.pending_events.extend([
                        ctrl(egui::Key::A, false, true),
                        egui::Event::Text(SIBLING_DRAFT.into()),
                    ]);
                    qa.phase = 2;
                    return (false, None);
                }
                let a = &self.progress.attempts[SIBLING];
                let w = a.workspace.as_ref().unwrap();
                assert!(
                    raw(w.routing_v4_draft.as_ref().unwrap())
                        .windows(SIBLING_DRAFT.len())
                        .any(|s| s == SIBLING_DRAFT.as_bytes())
                );
                let source = self.scenarios.iter().find(|s| s.id == SIBLING).unwrap();
                assert!(!progress::current_solved(a, source));
                qa.snapshot(Slot::Sibling, a);
            }
            Step::SiblingBack => {
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return (false, None);
                }
                assert!(self.selected.is_none());
                assert_eq!(raw(&self.progress.attempts[SIBLING]), qa.sibling);
            }
            Step::Search(which) => {
                let wanted = &fixtures()[which].source.title;
                if qa.phase == 0 {
                    let r = self.metrics.library_search.expect("Native library search");
                    if self.click_visible_smoke_control(ctx, Some(r)) {
                        qa.phase = 1;
                    }
                    return (false, None);
                }
                if qa.phase == 1 {
                    self.pending_events.extend([
                        ctrl(egui::Key::A, false, true),
                        egui::Event::Paste(wanted.clone()),
                    ]);
                    qa.phase = 2;
                    return (false, None);
                }
                assert_eq!(self.filter.search, *wanted);
                assert_eq!(self.metrics.card_actions.len(), 1);
            }
            Step::Start => {
                let click = egui::Id::new(("native-click-phases", self.smoke_stage));
                let acknowledged = ctx.data(|d| d.get_temp::<u8>(click)).unwrap_or(0) >= 2;
                let rect = if acknowledged {
                    None
                } else {
                    Some(
                        self.metrics
                            .card_actions
                            .first()
                            .expect("Actual matching lab card before press")
                            .0,
                    )
                };
                if !self.click_visible_smoke_control(ctx, rect) {
                    return (false, None);
                }
                assert!(self.selected.is_some());
            }
            Step::Favorite => {
                let r = self
                    .metrics
                    .card_actions
                    .first()
                    .expect("Actual matching card")
                    .1;
                if !self.click_visible_smoke_control(ctx, Some(r)) {
                    return (false, None);
                }
            }
            Step::Tab(view) => {
                if !self.physical_tab(ctx, view) {
                    return (false, None);
                }
            }
            Step::Control(key) => {
                if !self.physical_control(ctx, key) {
                    return (false, None);
                }
                if key == "apply" {
                    let a = &self.progress.attempts[IDS[n]];
                    let practice = !a.challenge
                        && a.workspace
                            .as_ref()
                            .unwrap()
                            .physical_link_learning
                            .as_ref()
                            .unwrap()
                            .view
                            == View::Example;
                    assert_eq!(
                        simulation(a, practice).configuration(),
                        &self
                            .physical_presentation
                            .displayed_draft()
                            .unwrap()
                            .configuration,
                        "Actual Apply commits the complete target"
                    );
                }
            }
            Step::Header(key) => {
                if !self.physical_header(ctx, key) {
                    return (false, None);
                }
            }
            Step::SnapshotAssignment => {
                let a = &self.progress.attempts[IDS[n]];
                qa.snapshot(Slot::Assignment, &assignment_part(a));
                qa.snapshot(Slot::Others, &other(&self.progress, IDS[n]));
            }
            Step::AssertAssignment => {
                assert_eq!(
                    assignment_bytes(&self.progress.attempts[IDS[n]]),
                    qa.assignment
                );
                assert_eq!(
                    raw(&other(&self.progress, IDS[n])),
                    qa.others,
                    "Example practice preserves other attempts, favorites, clock and actual migration receipts"
                );
            }
            Step::SnapshotExample => {
                let a = &self.progress.attempts[IDS[n]];
                qa.snapshot(Slot::Example, simulation(a, true));
            }
            Step::AssertFill => {
                let a = &self.progress.attempts[IDS[n]];
                assert_eq!(raw(simulation(a, true)), qa.example);
                assert_eq!(assignment_bytes(a), qa.assignment);
                assert!(!self.metrics.physical.complete);
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .physical_link_learning
                        .as_ref()
                        .unwrap()
                        .draft
                        .configuration,
                    fixtures()[n]
                        .source
                        .physical_link_teaching
                        .as_ref()
                        .unwrap()
                        .reference
                );
            }
            Step::Configure(target) => {
                let practice = matches!(target, Config::ExampleAdverse | Config::ExampleRepair);
                let a = &self.progress.attempts[IDS[n]];
                let desired = self.physical_target(n, target);
                if qa.cursor.is_none() {
                    qa.cursor = Some(config_ops(
                        self.physical_presentation
                            .displayed_draft()
                            .expect("Native configuration draft"),
                        desired,
                        simulation(a, practice),
                    ));
                }
                if !self.physical_cursor(ctx, qa.cursor.as_mut().unwrap()) {
                    return (false, None);
                }
                assert_eq!(
                    &self
                        .physical_presentation
                        .displayed_draft()
                        .unwrap()
                        .configuration,
                    self.physical_target(n, target)
                );
                qa.completed_cursor = qa.cursor.take();
                qa.completed_configs += 1;
            }
            Step::Rows(which, practice) => {
                let i = self.physical_index(n);
                let a = &self.progress.attempts[IDS[n]];
                let rows = self
                    .physical_presentation
                    .rows(a, &self.scenarios[i], practice)
                    .expect("Actual current native check rows");
                let actual = rows.iter().map(|r| r.0).collect::<Vec<_>>();
                match which {
                    Rows::All(false) => assert!(actual.iter().any(|v| !*v)),
                    _ => assert_eq!(
                        actual,
                        expected(n, which),
                        "Independent literal outcome vector for {}/ {which:?}",
                        IDS[n]
                    ),
                };
                assert_eq!(
                    self.metrics.physical.complete,
                    !practice && actual.iter().all(|v| *v)
                );
            }
            Step::Details(open) => {
                if qa.phase == 0 {
                    let r = self.metrics.physical.details_header.unwrap();
                    if self.click_visible_smoke_control(ctx, Some(r)) {
                        qa.phase = 1;
                    }
                    return (false, None);
                }
                if open {
                    let i = self.physical_index(n);
                    let rows = self
                        .physical_presentation
                        .rows(&self.progress.attempts[IDS[n]], &self.scenarios[i], false)
                        .unwrap();
                    assert_eq!(self.metrics.physical.detail_rows.len(), rows.len());
                    for (at, (index, ok, _)) in self.metrics.physical.detail_rows.iter().enumerate()
                    {
                        assert_eq!(*index, at);
                        assert_eq!(*ok, rows[at].0);
                    }
                } else {
                    assert!(self.metrics.physical.detail_rows.is_empty());
                }
            }
            Step::RawObservations => {
                let r = self
                    .metrics
                    .physical
                    .expanders
                    .iter()
                    .find(|r| r.0 == "raw-observations")
                    .expect("Supplied earlier observations")
                    .1;
                if !self.click_visible_smoke_control(ctx, Some(r)) {
                    return (false, None);
                }
                let i = self.physical_index(n);
                let engine::Goal::PhysicalLink { definition } = &self.scenarios[i].goal else {
                    panic!()
                };
                let engine::Goal::PhysicalLink {
                    definition: original,
                } = &fixtures()[n].source.goal
                else {
                    panic!()
                };
                assert_eq!(raw(&definition.observations), raw(&original.observations));
            }
            Step::CounterBefore => {
                qa.counters = simulation(&self.progress.attempts[IDS[n]], false)
                    .counters()
                    .clone();
            }
            Step::CounterDelta => {
                assert_eq!(n, 2);
                let state = simulation(&self.progress.attempts[IDS[n]], false);
                for (id, offered, received, crc) in
                    [("archive-left", 12, 12, 6), ("archive-right", 18, 8, 4)]
                {
                    let a = &state.counters()[id];
                    let b = &qa.counters[id];
                    assert_eq!(
                        (
                            a.offered - b.offered,
                            a.received - b.received,
                            a.crc_errors - b.crc_errors
                        ),
                        (offered, received, crc)
                    );
                    assert_eq!(a.collisions, b.collisions);
                    assert_eq!(a.late_collisions, b.late_collisions);
                }
            }
            Step::ClearCounters => {
                let state = simulation(&self.progress.attempts[IDS[n]], false);
                let link = &state.topology().links[&self
                    .physical_presentation
                    .displayed_draft()
                    .unwrap()
                    .selected_link];
                let port = if qa.phase == 0 {
                    link.a.clone()
                } else {
                    link.b.clone()
                };
                if !self.physical_control(ctx, &format!("clear:{port}")) {
                    return (false, None);
                }
                if qa.phase == 0 {
                    qa.phase = 1;
                    return (false, None);
                }
                let state = simulation(&self.progress.attempts[IDS[n]], false);
                let link = &state.topology().links[&self
                    .physical_presentation
                    .displayed_draft()
                    .unwrap()
                    .selected_link];
                for p in [&link.a, &link.b] {
                    assert_eq!(state.counters()[p], core::Counters::default());
                }
            }
            Step::SnapshotResume => {
                let a = &self.progress.attempts[IDS[n]];
                qa.snapshot(
                    Slot::Resume,
                    &Resume {
                        scenario_version: a.scenario_version,
                        state: &a.state,
                        step: a.step,
                        challenge: a.challenge,
                        hints: a.hints,
                        transcript: &a.transcript,
                        ipv4: &a.ipv4,
                        workspace: &a.workspace,
                    },
                );
            }
            Step::AssertResume => {
                assert_eq!(resumed(&self.progress.attempts[IDS[n]]), qa.resume);
            }
            Step::SaveReadback => {
                // Save writes transition provenance to the main file, without
                // stamping the caller. This fresh QA profile has no imported
                // receipt chain; it never installs the readback as learner work.
                let caller_receipts = raw(&self.progress.migration_receipts);
                assert!(self.progress.migration_receipts.is_none());
                self.save();
                assert!(self.storage_error.is_none());
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(raw(&loaded), raw(&self.progress));
                let disk = loaded.migration_receipts.as_ref().unwrap();
                assert!(disk.workspace);
                assert_eq!(disk.shell, [dlms_learning_lab::shell::EVALUATOR_REVISION]);
                assert_eq!(raw(&self.progress.migration_receipts), caller_receipts);
            }
            Step::SnapshotWhole => {
                qa.snapshot(Slot::Whole, &self.progress);
                qa.snapshot(Slot::Receipts, &self.progress.migration_receipts);
                qa.snapshot(Slot::Others, &other(&self.progress, IDS[n]));
            }
            Step::AssertWhole => {
                assert_eq!(raw(&self.progress), qa.whole);
                assert_eq!(raw(&self.progress.migration_receipts), qa.receipts);
            }
            Step::ResetExact => {
                let i = self.physical_index(n);
                let a = &self.progress.attempts[IDS[n]];
                assert_eq!(a.state, self.scenarios[i].initial);
                assert_eq!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .physical_link_learning
                        .as_ref()
                        .unwrap()
                        .view,
                    View::Introduction
                );
                assert!(!self.metrics.physical.complete);
                assert_eq!(raw(&other(&self.progress, IDS[n])), qa.others);
            }
            Step::ModalAttempt => {
                assert!(self.reset_pending);
                let port = &simulation(&self.progress.attempts[IDS[n]], false)
                    .topology()
                    .links[&self
                    .physical_presentation
                    .displayed_draft()
                    .unwrap()
                    .selected_link]
                    .a;
                let key = format!("up:{port}");
                if !self.physical_control(ctx, &key) {
                    return (false, None);
                }
            }
            Step::TypeNumber(which, text) => {
                if !self.physical_number(ctx, qa, which, text) {
                    return (false, None);
                }
            }
            Step::Key(key, shift, command) => {
                if qa.phase == 0 {
                    self.pending_events.push(ctrl(key, shift, command));
                    qa.phase = 1;
                    return (false, None);
                }
            }
            Step::Wait(seconds) => {
                let now = ctx.input(|i| i.time);
                let start = *qa.wait.get_or_insert(now);
                if now - start < seconds {
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::Number(which, text) => {
                assert_eq!(self.physical_presentation.sample_values()[which], text)
            }
            Step::Focus(which) => {
                let id = self
                    .metrics
                    .physical
                    .numbers
                    .iter()
                    .find(|r| r.0 == which)
                    .unwrap()
                    .2;
                if ctx.memory(|m| m.focused()) != Some(id) {
                    if qa.tries < 3 {
                        return (false, None);
                    }
                    panic!("Natural Tab/Shift-Tab focus did not reach number{which}");
                }
            }
            Step::Bounds => {
                if !self.physical_control(ctx, "probe") {
                    return (false, None);
                }
                assert!(
                    self.physical_presentation
                        .notice()
                        .is_some_and(|n| n.contains("0–1024"))
                );
            }
            Step::RejectedPaste => {
                let r = self
                    .metrics
                    .physical
                    .controls
                    .iter()
                    .find(|r| r.0 == "probe")
                    .unwrap()
                    .1;
                if qa.phase == 0 {
                    let field = self.metrics.physical.numbers[0].1;
                    if self.click_visible_smoke_control(ctx, Some(field)) {
                        qa.phase = 1;
                    }
                    return (false, None);
                }
                if qa.phase == 1 {
                    if !self.reveal_smoke_control(ctx, r) {
                        return (false, None);
                    }
                    self.pending_events.extend([
                        egui::Event::PointerMoved(r.center()),
                        egui::Event::PointerButton {
                            pos: r.center(),
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: Default::default(),
                        },
                    ]);
                    qa.phase = 2;
                    return (false, None);
                }
                if qa.phase == 2 {
                    let id = self.metrics.physical.numbers[0].2;
                    if ctx.memory(|m| m.focused()) != Some(id) {
                        self.pending_events.push(ctrl(egui::Key::Tab, true, false));
                        return (false, None);
                    }
                    self.pending_events.extend([
                        ctrl(egui::Key::A, false, true),
                        egui::Event::Paste("../../outside".into()),
                        egui::Event::PointerMoved(r.center()),
                        egui::Event::PointerButton {
                            pos: r.center(),
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: Default::default(),
                        },
                    ]);
                    qa.phase = 3;
                    return (false, None);
                }
                assert!(self.physical_presentation.notice().is_some());
            }
            Step::Mode(challenge) => {
                if !self.physical_header(ctx, if challenge { "challenge" } else { "learn" }) {
                    return (false, None);
                }
                assert_eq!(self.progress.attempts[IDS[n]].challenge, challenge);
                if challenge {
                    assert!(self.metrics.physical.tabs.is_empty());
                    assert!(self.metrics.physical.detail_rows.is_empty());
                }
            }
            Step::Next(which) => {
                if !self.click_visible_smoke_control(ctx, self.metrics.next_unit) {
                    return (false, None);
                }
                assert_eq!(self.scenarios[self.selected.unwrap()].id, IDS[which]);
            }
            Step::UnitComplete => {
                assert!(self.metrics.unit_complete.is_some());
                for (n, id) in IDS.iter().enumerate() {
                    let i = self.physical_index(n);
                    assert_eq!(
                        self.completion_index
                            .status(&self.progress.attempts[*id], &self.scenarios[i]),
                        Some(crate::ui::completion_index::Status::Complete)
                    );
                }
            }
            Step::Theme(light) => {
                if qa.phase == 0 {
                    let (_, narrow) = qa
                        .requested_geometry
                        .expect("Existing requested native viewport");
                    if !self.physical_geometry(ctx, qa, self.scale, narrow) {
                        return (false, None);
                    }
                    if self.physical_popup_click(ctx, self.metrics.reopen_theme, None) {
                        qa.phase = 1;
                    }
                    return (false, None);
                }
                if qa.phase == 1 {
                    let id = if light { "dlms-light" } else { "dlms-dark" };
                    let click = egui::Id::new(("native-click-phases", self.smoke_stage));
                    let acknowledged = ctx.data(|d| d.get_temp::<u8>(click)).unwrap_or(0) >= 2;
                    let (rect, clip) = if acknowledged {
                        (None, None)
                    } else {
                        let r = self
                            .metrics
                            .physical_picker_choices
                            .iter()
                            .find(|r| r.0 == format!("theme:{id}"))
                            .unwrap_or_else(|| panic!("Actual theme item before press, stage{} phase{} actualscale{} content{:?} choices{:?}", self.smoke_stage, qa.phase, self.scale, ctx.content_rect(), self.metrics.physical_picker_choices));
                        (Some(r.1), Some(r.2))
                    };
                    if self.physical_popup_click(ctx, rect, clip) {
                        qa.phase = 2;
                    }
                    return (false, None);
                }
                assert_eq!(theme::find(&self.theme_id).light, light);
            }
            Step::Geometry(scale, narrow) => {
                qa.requested_geometry = Some((scale, narrow));
                if qa.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!narrow));
                    if narrow {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            946., 929.,
                        )));
                    }
                    qa.phase = 1;
                    return (false, None);
                }
                if qa.phase == 1 {
                    // Native resize/zoom delivery is deferred. Observe a stable
                    // actual window before measuring/clicking header controls.
                    if !self.physical_geometry(ctx, qa, self.scale, narrow) {
                        return (false, None);
                    }
                    if self.physical_popup_click(ctx, self.metrics.reopen_scale, None) {
                        qa.phase = 2;
                    }
                    return (false, None);
                }
                if qa.phase == 2 {
                    let click = egui::Id::new(("native-click-phases", self.smoke_stage));
                    let acknowledged = ctx.data(|d| d.get_temp::<u8>(click)).unwrap_or(0) >= 2;
                    let (rect, clip) = if acknowledged {
                        (None, None)
                    } else {
                        let r = self
                            .metrics
                            .physical_picker_choices
                            .iter()
                            .find(|r| r.0 == format!("scale:{scale:.2}"))
                            .expect("Actual scale item before press");
                        (Some(r.1), Some(r.2))
                    };
                    if self.physical_popup_click(ctx, rect, clip) {
                        qa.phase = 3;
                    }
                    return (false, None);
                }
                assert!((self.scale - scale).abs() < 0.001);
            }
            Step::Settle(scale, narrow) => {
                if !self.physical_geometry(ctx, qa, scale, narrow) {
                    return (false, None);
                }
            }
            Step::Shot(tag, anchor) => {
                if tag.starts_with("physical-alignment-") {
                    let (scale, narrow) = qa.requested_geometry.expect("Actual requested geometry");
                    if !self.physical_geometry(ctx, qa, scale, narrow) {
                        return (false, None);
                    }
                }
                let r = self.physical_anchor(anchor).expect("Actual capture anchor");
                if !self.reveal_smoke_control(ctx, r) {
                    return (false, None);
                }
                let viewport = self
                    .metrics
                    .main_viewport
                    .unwrap_or(ctx.content_rect())
                    .intersect(ctx.content_rect());
                assert!(smoke_control_visible(r, viewport, ctx.pixels_per_point()));
                for (_, rect, _) in &self.metrics.physical.controls {
                    assert!(
                        rect.left() >= viewport.left() - 1. / ctx.pixels_per_point()
                            && rect.right() <= viewport.right() + 1. / ctx.pixels_per_point(),
                        "Physical control is horizontally clipped: {rect:?} vs{viewport:?}"
                    );
                }
                return (true, Some(tag));
            }
            Step::Idle => {
                let current = self.physical_presentation.native_counts();
                let json = dlms_learning_lab::native_observations::json_admission_calls();
                let counts = self.completion_index.observed().2;
                if let Some((frames, before, admissions, keys)) = qa.idle {
                    assert_eq!(current, before);
                    assert_eq!(json, admissions);
                    assert_eq!(
                        (counts.contexts, counts.key_copies, counts.certificates),
                        (keys.contexts, keys.key_copies, keys.certificates)
                    );
                    if frames < 60 {
                        qa.idle = Some((frames + 1, before, admissions, keys));
                        ctx.request_repaint();
                        return (false, None);
                    }
                } else {
                    qa.idle = Some((0, current, json, counts));
                    ctx.request_repaint();
                    return (false, None);
                }
            }
            Step::Finish => {
                let facts = facts(ctx).expect("Controller facts");
                // Native teardown may render another root frame after Close.
                // Keep terminal delivery idempotent without rerunning assertions,
                // copying inputs, publishing grades, or emitting another PASS.
                if qa.finish_emissions != 0 {
                    assert_eq!(qa.finish_emissions, 1);
                    assert_eq!(facts.finish_emissions, 1);
                    assert_eq!(facts.stage, self.smoke_stage);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                assert_eq!(facts.tags, qa.tags.len());
                assert_eq!(facts.completed_configs, qa.completed_configs);
                assert!(facts.stage < self.smoke_stage);
                let _ = (facts.requested_geometry, facts.geometry_same);
                assert_eq!(raw(&self.scenarios), qa.catalog);
                assert_eq!(raw(&self.units), qa.units);
                assert_eq!(qa.tags.len(), 108);
                assert!(
                    self.progress
                        .attempts
                        .keys()
                        .all(|id| IDS.contains(&id.as_str()) || id == SIBLING)
                );
                assert_eq!(
                    raw(&self.progress.attempts[SIBLING]),
                    qa.sibling,
                    "All physical practice, reset, resume and profile writes preserve unrelated existing work"
                );
                assert!(self.progress.favorites.contains(SIBLING));
                assert_eq!(self.progress.attempts.len(), 5);
                qa.finish_emissions = 1;
                println!(
                    "PASS normal-catalog physical native route: four real-workspace assignments, literal failures, references/two alternatives, isolated practice, counter observations, accepted/rejected native input, reset/validated save readback and40 actual picker/geometry captures. Normal curriculum acceptance still requires rendered review/content/full inclusive gates; cold process reopen, physical-device accessibility and novice acceptance remain pending."
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        (true, None)
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::*;
    #[test]
    fn physical_config_cursor_toggles_auto_membership_from_actual_mode_without_reset_assumption() {
        let source = &fixtures()[0].source;
        let engine::State::PhysicalLink { simulation } = &source.initial else {
            panic!("Actual registered physical fixture")
        };
        let (id, spec) = simulation
            .topology()
            .ports
            .iter()
            .find(|(_, spec)| {
                spec.profiles.contains(&core::Profile::CopperRj45) && spec.abilities.len() >= 2
            })
            .expect("Authored copper interface offers distinct abilities");
        let abilities = spec.abilities.iter().copied().collect::<Vec<_>>();
        let first = abilities[0];
        let second = abilities[1];
        let mut draft = assignment::Draft::from_state(simulation).unwrap();
        draft.configuration.ports.get_mut(id).unwrap().negotiation = core::Negotiation::Auto {
            advertise: [first].into_iter().collect(),
        };
        let mut target = draft.configuration.clone();
        target.ports.get_mut(id).unwrap().negotiation = core::Negotiation::Auto {
            advertise: [second].into_iter().collect(),
        };
        let keys = |cursor: Cursor| {
            cursor
                .ops
                .into_iter()
                .filter_map(|op| {
                    if let Op::Toggle(key) = op {
                        Some(key)
                    } else {
                        None
                    }
                })
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(
            keys(config_ops(&draft, &target, simulation)),
            [ability_key(id, first), ability_key(id, second)]
                .into_iter()
                .collect()
        );
        assert!(
            config_ops(&draft, &draft.configuration, simulation)
                .ops
                .is_empty()
        );
        assert!(
            !config_ops(&draft, &target, simulation)
                .ops
                .iter()
                .any(|op| matches!(op, Op::Choice(key, _) if key == &format!("mode:{id}"))),
            "Already-Auto must not be reselected by the actual native cursor"
        );
        draft.configuration.ports.get_mut(id).unwrap().negotiation =
            core::Negotiation::Fixed { ability: first };
        assert_eq!(
            keys(config_ops(&draft, &target, simulation)),
            abilities
                .into_iter()
                .filter(|ability| *ability != second)
                .map(|ability| ability_key(id, ability))
                .collect()
        );
    }
    fn widget_frame(ctx: &egui::Context, app: &mut LabApp, frames: &mut usize) {
        *frames += 1;
        let events = std::mem::take(&mut app.pending_events);
        assert!(events.len() <= 300);
        let modifiers = events
            .iter()
            .rev()
            .find_map(|event| {
                if let egui::Event::Key { modifiers, .. } = event {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920., 1080.),
                )),
                time: Some(*frames as f64 / 20.),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
    }
    fn widget_action(
        ctx: &egui::Context,
        app: &mut LabApp,
        frames: &mut usize,
        mut action: impl FnMut(&egui::Context, &mut LabApp) -> bool,
    ) {
        for _ in 0..900 {
            widget_frame(ctx, app, frames);
            if action(ctx, app) {
                assert!(
                    app.pending_events.is_empty(),
                    "Acknowledge only already-delivered native input"
                );
                widget_frame(ctx, app, frames); // Let the newly selected view render naturally.
                app.smoke_stage += 1;
                return;
            }
        }
        panic!("Actual Auto-advertisement widgets did not finish within the unchanged event bound");
    }
    #[test]
    fn physical_auto_cursor_real_widgets_reach_subset_replacement_without_reselect_reset() {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        app.smoke_stage = 66800;
        let at = app
            .scenarios
            .iter()
            .position(|source| source.id == IDS[0])
            .unwrap();
        app.launch(at); // Production opening only; all configuration arrives through native widgets.
        let mut frames = 0;
        widget_action(&ctx, &mut app, &mut frames, |ctx, app| {
            app.physical_tab(ctx, View::Assignment)
        });
        let id = "campus-left";
        let engine::State::PhysicalLink { simulation } = &app.progress.attempts[IDS[0]].state
        else {
            panic!()
        };
        let abilities = simulation.topology().ports[id]
            .abilities
            .iter()
            .copied()
            .collect::<Vec<_>>();
        assert!(abilities.len() >= 2);
        let first = abilities[0];
        let second = abilities[1];
        let mut target = app
            .physical_presentation
            .displayed_draft()
            .unwrap()
            .configuration
            .clone();
        target.ports.get_mut(id).unwrap().negotiation = core::Negotiation::Auto {
            advertise: [first].into_iter().collect(),
        };
        let mut cursor = config_ops(
            app.physical_presentation.displayed_draft().unwrap(),
            &target,
            simulation,
        );
        widget_action(&ctx, &mut app, &mut frames, |ctx, app| {
            app.physical_cursor(ctx, &mut cursor)
        });
        assert_eq!(
            &app.physical_presentation
                .displayed_draft()
                .unwrap()
                .configuration,
            &target
        );
        widget_action(&ctx, &mut app, &mut frames, |ctx, app| {
            app.physical_control(ctx, "apply")
        });
        let engine::State::PhysicalLink { simulation } = &app.progress.attempts[IDS[0]].state
        else {
            panic!()
        };
        assert_eq!(
            simulation.configuration(),
            &target,
            "Actual widget Apply establishes the old one-ability Auto state"
        );
        target.ports.get_mut(id).unwrap().negotiation = core::Negotiation::Auto {
            advertise: [second].into_iter().collect(),
        };
        let mut cursor = config_ops(
            app.physical_presentation.displayed_draft().unwrap(),
            &target,
            simulation,
        );
        assert!(
            !cursor
                .ops
                .iter()
                .any(|op| matches!(op, Op::Choice(key, _) if key == &format!("mode:{id}")))
        );
        widget_action(&ctx, &mut app, &mut frames, |ctx, app| {
            app.physical_cursor(ctx, &mut cursor)
        });
        assert_eq!(
            &app.physical_presentation
                .displayed_draft()
                .unwrap()
                .configuration,
            &target,
            "Actual native checkboxes reach the replacement subset; planned toggle keys alone are insufficient proof"
        );
        widget_action(&ctx, &mut app, &mut frames, |ctx, app| {
            app.physical_control(ctx, "apply")
        });
        let engine::State::PhysicalLink { simulation } = &app.progress.attempts[IDS[0]].state
        else {
            panic!()
        };
        assert_eq!(
            simulation.configuration(),
            &target,
            "Applied state reaches the exact requested target after native interactions"
        );
        assert!(
            !progress::current_solved(&app.progress.attempts[IDS[0]], &app.scenarios[at]),
            "This isolated advertisement exercise does not falsely solve the different media/span assignment"
        );
        assert!(frames < 2700);
    }
}
