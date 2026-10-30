//! Fixed ordinary-course native acceptance route. No progress/grade/focus injection.
use super::*;
use dlms_learning_lab::{ntp_workspace as work, script_teaching::View};
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as disk_registration;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};
pub(in crate::ui) const FIRST: usize = 80000;
const RESERVE: usize = 2000;
const IDS: [&str; 4] = [
    "ccna-ntp-client-chain",
    "ccna-ntp-cycle-repair",
    "ccna-ntp-asymmetric-delivery",
    "ccna-ntp-policy-handover",
];
#[derive(Clone, Debug)]
enum Target {
    Search,
    Start(usize),
    Field,
    Control(&'static str),
    Back,
    Reset,
    Cancel,
    Confirm,
    Mode(bool),
    ExamplePicker,
    ExampleChoice(usize),
    DeviceChoice(String),
    Topology,
    Theme,
    Scale,
    HeaderChoice(String),
    Goal,
    Feedback,
    Notice,
    Text(String),
}
#[derive(Clone, Copy, Debug)]
enum Scope {
    Assignment,
    Current,
    Progress,
    Others,
}
#[derive(Clone, Debug)]
enum Step {
    Init,
    Click(Target),
    Replace(String),
    Command(String),
    Key(egui::Key, egui::Modifiers),
    Focus(bool),
    Remember(String, Scope),
    Same(String, Scope),
    GuidanceDelta(String, usize),
    RememberWarm(String),
    ResumeExact(String, bool),
    Task(usize),
    Selected(String),
    Inspect,
    Rows(Vec<bool>, bool),
    Completion(bool),
    Notice(bool),
    Fresh,
    Save,
    InvalidPacket,
    Window(bool),
    Setting(f32, String),
    Geometry,
    Shot(String, Target),
    Done,
}
fn sources() -> &'static Vec<pack::Scenario> {
    static S: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/ntp-client-server-v1.json"
        ))
        .unwrap()
        .scenarios
    })
}
fn review() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/ntp-client-server-review.json"
        ))
        .unwrap()
    })
}
fn rows(v: &Value) -> Vec<bool> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_bool().unwrap())
        .collect()
}
fn select(p: &mut Vec<Step>, ip: &str) {
    p.extend([
        Step::Click(Target::Control("device")),
        Step::Click(Target::DeviceChoice(ip.into())),
        Step::Selected(ip.into()),
    ]);
}
fn command(p: &mut Vec<Step>, s: &str, enter: bool) {
    if let Some(ip) = s.strip_prefix("SIM select ") {
        select(p, ip);
        return;
    }
    p.extend([
        Step::Click(Target::Field),
        Step::Replace(s.into()),
        Step::Command(s.into()),
    ]);
    p.push(if enter {
        Step::Key(egui::Key::Enter, Default::default())
    } else {
        Step::Click(Target::Control("run"))
    });
}
fn check(p: &mut Vec<Step>, expected: Vec<bool>, practice: bool) {
    p.extend([
        Step::Click(Target::Control("check")),
        Step::Rows(expected, practice),
    ]);
}
fn reset(p: &mut Vec<Step>) {
    p.extend([
        Step::Remember("other-before-reset".into(), Scope::Others),
        Step::Click(Target::Reset),
        Step::Click(Target::Confirm),
        Step::Fresh,
        Step::Same("other-before-reset".into(), Scope::Others),
        Step::Click(Target::Control("assignment")),
    ]);
}
fn plan() -> &'static Vec<Step> {
    static P: OnceLock<Vec<Step>> = OnceLock::new();
    P.get_or_init(|| {
        let mut p = vec![
            Step::Init,
            Step::Window(false),
            Step::Shot(
                "ntp-normal-library-search-placeholder".into(),
                Target::Search,
            ),
        ];
        for (n, s) in sources().iter().enumerate() {
            let r = &review()["labs"][n];
            assert_eq!(r["id"], s.id);
            let engine::Goal::Ntp { definition } = &s.goal else {
                unreachable!()
            };
            let pass = vec![true; r["initial_expected"].as_array().unwrap().len()];
            p.extend([
                Step::Click(Target::Search),
                Step::Replace(s.title.clone()),
                Step::Click(Target::Start(n)),
                Step::Task(n),
                Step::Shot(
                    format!("{}-introduction", s.id),
                    Target::Control("assignment"),
                ),
                Step::Click(Target::Control("assignment")),
            ]);
            check(&mut p, rows(&r["initial_expected"]), false);
            p.extend([
                Step::Shot(format!("{}-whole-assignment", s.id), Target::Goal),
                Step::Shot(format!("{}-dependency-evidence", s.id), Target::Topology),
            ]);
            if n == 0 {
                // Only one requested hint is persisted. Disclosure controls must
                // not run/check/edit the meaningful assignment or practice.
                p.extend([
                    Step::Remember("before-guidance".into(), Scope::Progress),
                    Step::Click(Target::Control("optional-help")),
                    Step::Click(Target::Control("show-hint")),
                    Step::GuidanceDelta("before-guidance".into(), 1),
                    Step::Shot(
                        format!("{}-requested-progressive-hint", s.id),
                        Target::Text(format!("Hint 1: {}", s.hints[0])),
                    ),
                    Step::Remember("after-requested-hint".into(), Scope::Progress),
                    Step::Click(Target::Control("worked-approach")),
                    Step::Same("after-requested-hint".into(), Scope::Progress),
                    Step::Shot(
                        format!("{}-requested-worked-approach", s.id),
                        Target::Text(s.explanation.clone()),
                    ),
                    Step::Click(Target::Control("worked-approach")),
                    Step::Click(Target::Control("references")),
                    Step::Same("after-requested-hint".into(), Scope::Progress),
                    Step::Shot(
                        format!("{}-requested-references", s.id),
                        Target::Text(s.sources[0].clone()),
                    ),
                    Step::Click(Target::Control("references")),
                    Step::Click(Target::Control("command-help")),
                    Step::Same("after-requested-hint".into(), Scope::Progress),
                    Step::Shot(
                        format!("{}-requested-supported-commands", s.id),
                        Target::Text(dlms_learning_lab::ntp::HELP.into()),
                    ),
                    Step::Click(Target::Control("command-help")),
                    Step::Click(Target::Control("optional-help")),
                    Step::Same("after-requested-hint".into(), Scope::Progress),
                ]);
            }
            p.extend([
                Step::Remember("assignment-before-practice".into(), Scope::Assignment),
                Step::Click(Target::Control("example")),
                Step::Click(Target::ExamplePicker),
                Step::Click(Target::ExampleChoice(0)),
                Step::Click(Target::Control("insert")),
                Step::Command(definition.lesson.worked[0].text.clone()),
                Step::Same("assignment-before-practice".into(), Scope::Assignment),
                Step::Completion(false),
                Step::Shot(format!("{}-example-inserted-not-run", s.id), Target::Field),
            ]);
            for c in &definition.lesson.worked {
                select(&mut p, &c.device.to_string());
                command(&mut p, &c.text, false);
            }
            check(&mut p, vec![true; 3], true);
            select(&mut p, &definition.lesson.experiment.device.to_string());
            command(&mut p, &definition.lesson.experiment.text, true);
            check(&mut p, vec![false, false, true], true);
            p.extend([
                Step::Completion(false),
                Step::Shot(format!("{}-example-experiment", s.id), Target::Feedback),
            ]);
            select(&mut p, &definition.lesson.repair.device.to_string());
            command(&mut p, &definition.lesson.repair.text, false);
            check(&mut p, vec![true; 3], true);
            p.extend([
                Step::Same("assignment-before-practice".into(), Scope::Assignment),
                Step::Completion(false),
                Step::Click(Target::Control("assignment")),
                Step::Same("assignment-before-practice".into(), Scope::Assignment),
                Step::Remember("refusal-world".into(), Scope::Current),
            ]);
            command(&mut p, "ntp peer 192.0.2.1", true);
            p.extend([
                Step::Same("refusal-world".into(), Scope::Current),
                Step::Notice(true),
                Step::Shot(format!("{}-command-refusal", s.id), Target::Notice),
            ]);
            command(&mut p, "show ntp status", false);
            p.push(Step::Notice(false));
            for c in r["wrong"]["commands"].as_array().unwrap() {
                command(&mut p, c.as_str().unwrap(), true);
            }
            p.push(Step::Notice(r["wrong"]["expect_error"].as_bool().unwrap()));
            check(&mut p, rows(&r["wrong"]["expected"]), false);
            p.push(Step::Shot(
                format!("{}-wrong-retry", s.id),
                Target::Feedback,
            ));
            if n == 2 {
                p.extend([
                    Step::Remember("inspection-only".into(), Scope::Progress),
                    Step::Click(Target::Control("inspection")),
                    Step::Click(Target::Control("all-facts")),
                    Step::Inspect,
                    Step::Same("inspection-only".into(), Scope::Progress),
                    Step::Click(Target::Control("inspection")),
                ]);
            }
            reset(&mut p);
            for c in r["partial"]["commands"].as_array().unwrap() {
                command(&mut p, c.as_str().unwrap(), false);
            }
            check(&mut p, rows(&r["partial"]["expected"]), false);
            p.push(Step::Shot(
                format!("{}-partial-configuration", s.id),
                Target::Feedback,
            ));
            reset(&mut p);
            p.extend([
                Step::Click(Target::Field),
                Step::Remember("after-reset-before-undo".into(), Scope::Progress),
                Step::Key(
                    egui::Key::Z,
                    egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                ),
                Step::Command(String::new()),
                Step::Same("after-reset-before-undo".into(), Scope::Progress),
            ]);
            for c in &s.solution {
                command(&mut p, c, false);
            }
            check(&mut p, pass.clone(), false);
            p.extend([
                Step::Completion(true),
                Step::Shot(format!("{}-reference-complete", s.id), Target::Feedback),
                Step::Save,
            ]);
            if n == 0 {
                p.extend([
                    Step::RememberWarm("before-next".into()),
                    Step::Click(Target::Control("next")),
                    Step::Task(1),
                    Step::Fresh,
                    Step::Completion(false),
                    Step::Click(Target::Back),
                    Step::Click(Target::Search),
                    Step::Replace(s.title.clone()),
                    Step::Click(Target::Start(0)),
                    Step::Task(0),
                    Step::ResumeExact("before-next".into(), true),
                ]);
            }
            for (a, commands) in r["alternatives"].as_array().unwrap().iter().enumerate() {
                reset(&mut p);
                for c in commands.as_array().unwrap() {
                    command(&mut p, c.as_str().unwrap(), a == 1);
                }
                check(&mut p, pass.clone(), false);
                p.extend([
                    Step::Completion(true),
                    Step::Shot(format!("{}-alternate-{}", s.id, a + 1), Target::Feedback),
                ]);
            }
            if n == 0 {
                select(&mut p, "10.71.0.1");
                command(&mut p, "no ntp server 10.71.0.103", true);
                check(&mut p, rows(&r["initial_expected"]), false);
                p.extend([
                    Step::Completion(false),
                    Step::Shot(format!("{}-success-invalidated", s.id), Target::Feedback),
                ]);
                reset(&mut p);
                for c in &s.solution {
                    command(&mut p, c, false);
                }
                check(&mut p, pass.clone(), false);
            }
            p.extend([
                Step::Click(Target::Field),
                Step::Replace("unfinished NTP command |".into()),
                Step::RememberWarm("resume-attempt".into()),
                Step::Save,
                Step::Click(Target::Back),
                Step::Click(Target::Start(n)),
                Step::Task(n),
                Step::ResumeExact("resume-attempt".into(), false),
                Step::Remember("cancel-progress".into(), Scope::Progress),
                Step::Click(Target::Reset),
                Step::Click(Target::Cancel),
                Step::Same("cancel-progress".into(), Scope::Progress),
                Step::Click(Target::Field),
                Step::Remember("bad-packet".into(), Scope::Progress),
                Step::InvalidPacket,
                Step::Same("bad-packet".into(), Scope::Progress),
                Step::Notice(true),
            ]);
            command(&mut p, "show ntp status", false);
            p.push(Step::Notice(false));
            p.extend([
                Step::Click(Target::Field),
                Step::Key(egui::Key::Tab, Default::default()),
                Step::Focus(false),
                Step::Key(
                    egui::Key::Tab,
                    egui::Modifiers {
                        shift: true,
                        ..Default::default()
                    },
                ),
                Step::Focus(true),
                Step::Click(Target::Mode(true)),
                Step::Shot(format!("{}-challenge", s.id), Target::Field),
                Step::Click(Target::Mode(false)),
                Step::Click(Target::Control("assignment")),
                Step::Save,
            ]);
            if n < 3 {
                p.push(Step::Click(Target::Back));
            }
        }
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for light in [true, false] {
                for wide in [true, false] {
                    let theme = if light { "dlms-light" } else { "dlms-dark" };
                    p.extend([
                        Step::Click(Target::Theme),
                        Step::Click(Target::HeaderChoice(format!("theme:{theme}"))),
                        Step::Click(Target::Scale),
                        Step::Click(Target::HeaderChoice(format!("scale:{scale:.2}"))),
                        Step::Window(wide),
                        Step::Setting(scale, theme.into()),
                        Step::Click(Target::Field),
                        Step::Geometry,
                        Step::Shot(
                            format!(
                                "ntp-handover-layout-{}-{}-{}",
                                (scale * 100.) as u32,
                                if light { "light" } else { "dark" },
                                if wide { "wide" } else { "narrow" }
                            ),
                            Target::Field,
                        ),
                        Step::Shot(
                            format!(
                                "ntp-handover-dependency-{}-{}-{}",
                                (scale * 100.) as u32,
                                if light { "light" } else { "dark" },
                                if wide { "wide" } else { "narrow" }
                            ),
                            Target::Topology,
                        ),
                    ]);
                }
            }
        }
        p.extend([
            Step::Click(Target::Control("check")),
            Step::Rows(vec![true; 7], false),
            Step::Save,
            Step::Done,
        ]);
        assert!(p.len() < RESERVE);
        p
    })
}
#[derive(Clone)]
struct Warm {
    portable: Value,
    caller_receipts: Value,
    raw: Vec<u8>,
    raw_receipts: Value,
}
#[derive(Clone, Default)]
struct Qa {
    stage: usize,
    last_root: Option<u64>,
    phase: u8,
    polls: usize,
    began: f64,
    geometry: Option<egui::Rect>,
    stable: u8,
    wheels: usize,
    wheel_until: f64,
    snapshots: BTreeMap<String, Value>,
    warm: BTreeMap<String, Warm>,
    next_returns: usize,
    continues: usize,
    geometry_item: usize,
    shots: usize,
    finished: bool,
    closing: Option<Value>,
}
fn key() -> egui::Id {
    egui::Id::new("ntp-native190-state")
}
#[cfg(test)]
pub(in crate::ui) fn phase_bounds() -> (usize, usize) {
    (FIRST, FIRST + plan().len() - 1)
}
#[cfg(test)]
pub(in crate::ui) fn first_hint_capture_stage() -> usize {
    FIRST
        + plan()
            .iter()
            .position(
                |s| matches!(s,Step::Shot(name,_)if name.ends_with("-requested-progressive-hint")),
            )
            .unwrap()
}
pub(in crate::ui) fn planned_captures() -> Vec<String> {
    plan()
        .iter()
        .filter_map(|s| {
            if let Step::Shot(n, _) = s {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect()
}
#[cfg(test)]
pub(in crate::ui) fn completed(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Qa>(key()))
        .is_some_and(|q| q.finished)
}
#[cfg(test)]
pub(in crate::ui) fn recency_observations(ctx: &egui::Context) -> Option<(usize, usize)> {
    ctx.data(|d| d.get_temp::<Qa>(key()))
        .map(|q| (q.next_returns, q.continues))
}
impl LabApp {
    pub(in crate::ui) fn ntp_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("ntp-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn ntp_capture_active(&self) -> bool {
        self.smoke && (FIRST..FIRST + RESERVE).contains(&self.smoke_stage)
    }
    fn ntp_session(&self) -> &work::Session {
        let i = self.selected.expect("Actual NTP selection");
        let source = &self.scenarios[i];
        let a = &self.progress.attempts[&source.id];
        work::context(a, source).unwrap();
        let engine::State::Ntp { simulation } = &a.state else {
            panic!("Real NTP attempt")
        };
        simulation
    }
    fn ntp_projection(&self, scope: Scope) -> Value {
        let i = self.selected.unwrap();
        let id = &self.scenarios[i].id;
        let s = self.ntp_session();
        match scope {
            Scope::Assignment => {
                json!({"selected":s.selected,"current":s.current,"command":s.command,"transcript":s.transcript,"checked":s.checked})
            }
            Scope::Current => serde_json::to_value(&s.current).unwrap(),
            Scope::Progress => {
                json!({"portable":self.progress,"caller_receipts":self.progress.migration_receipts})
            }
            Scope::Others => {
                let mut p = serde_json::to_value(&self.progress).unwrap();
                p["attempts"].as_object_mut().unwrap().remove(id);
                json!({"portable":p,"caller_receipts":self.progress.migration_receipts})
            }
        }
    }
    fn ntp_warm(&mut self) -> Warm {
        let portable = serde_json::to_value(&self.progress).unwrap();
        let caller_receipts = serde_json::to_value(&self.progress.migration_receipts).unwrap();
        self.save();
        assert!(!self.dirty && self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap();
        let mut saved: Value = serde_json::from_slice(&raw).unwrap();
        let raw_receipts = saved
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .expect("Real saved provenance");
        assert_eq!(saved, portable, "Every saved portable field is current");
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            portable,
            "Read-only load preserves the entire portable profile"
        );
        assert_eq!(
            serde_json::to_value(&loaded.migration_receipts).unwrap(),
            raw_receipts,
            "Real raw writer receipts are retained separately from skipped portable serialization"
        );
        assert_eq!(
            serde_json::to_value(&self.progress).unwrap(),
            portable,
            "Loading never installs replacement progress"
        );
        assert_eq!(
            serde_json::to_value(&self.progress.migration_receipts).unwrap(),
            caller_receipts,
            "Save/load never stamps caller receipts"
        );
        assert_eq!(
            progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
            raw
        );
        Warm {
            portable,
            caller_receipts,
            raw,
            raw_receipts,
        }
    }
    fn ntp_resume_exact(&mut self, before: &Warm, via_next: bool) {
        let i = self.selected.expect("Real returned activity");
        let source = &self.scenarios[i];
        let mut expected = before.portable.clone();
        let old_clock = expected["clock"].as_u64().unwrap();
        let clock = old_clock.checked_add(if via_next { 2 } else { 1 }).unwrap();
        expected["clock"] = json!(clock);
        assert!(
            expected["attempts"]
                .as_object()
                .unwrap()
                .contains_key(&source.id)
        );
        expected["attempts"][&source.id]["visited"] = json!(clock);
        if via_next {
            assert_eq!(source.id, IDS[0]);
            let peer = &self.scenarios[i + 1];
            assert_eq!(peer.id, IDS[1]);
            assert!(
                !expected["attempts"]
                    .as_object()
                    .unwrap()
                    .contains_key(&peer.id),
                "Next creates only the genuinely absent peer"
            );
            let mut fresh = progress::Attempt::new(peer, old_clock.checked_add(1).unwrap());
            model::prepare(&mut fresh); // Expected value only; never installed.
            work::context(&fresh, peer).unwrap();
            assert!(!progress::current_solved(&fresh, peer));
            expected["attempts"][&peer.id] = serde_json::to_value(&fresh).unwrap();
            assert!(
                !progress::current_solved(&self.progress.attempts[&peer.id], peer),
                "Next never invents peer mastery"
            );
        }
        assert_eq!(
            serde_json::to_value(&self.progress).unwrap(),
            expected,
            "Whole profile changes only exact clock/visit recency and the genuinely fresh Next peer"
        );
        assert_eq!(
            serde_json::to_value(&self.progress.migration_receipts).unwrap(),
            before.caller_receipts
        );
        let now = self.ntp_warm();
        assert_eq!(now.portable, expected);
        assert_eq!(now.caller_receipts, before.caller_receipts);
        assert_eq!(now.raw_receipts, before.raw_receipts);
        let mut expected_raw: Value = serde_json::from_slice(&before.raw).unwrap();
        expected_raw["clock"] = expected["clock"].clone();
        expected_raw["attempts"] = expected["attempts"].clone();
        assert_eq!(
            serde_json::from_slice::<Value>(&now.raw).unwrap(),
            expected_raw,
            "Every persisted field including raw receipts follows the same exact recency law"
        );
    }
    fn ntp_target(&self, ctx: &egui::Context, t: &Target) -> (egui::Rect, egui::Rect) {
        let root = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let m = &self.metrics;
        match t {
            Target::Text(wanted) => {
                let (_, rect, clip) = m
                    .ntp
                    .help_bodies
                    .iter()
                    .find(|(text, _, _)| text == wanted)
                    .unwrap_or_else(|| {
                        panic!(
                            "Real requested help response missing atstage{}",
                            self.smoke_stage
                        )
                    });
                assert!(rect.is_positive());
                (*rect, clip.intersect(root))
            }
            Target::Search => (m.library_search.expect("Library search"), root),
            Target::Start(n) => {
                let at = m
                    .card_verification
                    .iter()
                    .position(|r| r.0 == IDS[*n])
                    .expect("Actual filtered NTP card");
                (m.card_actions[at].0, root)
            }
            Target::Field => {
                let r = m
                    .ntp
                    .fields
                    .iter()
                    .find(|r| r.0 == "command")
                    .expect("NTP command field");
                (r.1, r.3.intersect(root))
            }
            Target::Control(k) => {
                let r = m
                    .ntp
                    .controls
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| panic!("Actual NTP control{k}"));
                (r.1, r.2.intersect(root))
            }
            Target::Back => (m.back.unwrap(), root),
            Target::Reset => (m.reset.unwrap(), root),
            Target::Cancel => (m.reset_cancel.unwrap(), ctx.content_rect()),
            Target::Confirm => (m.confirm_reset.unwrap(), ctx.content_rect()),
            Target::Mode(challenge) => (
                if *challenge {
                    m.mode_challenge.unwrap()
                } else {
                    m.mode_learn.unwrap()
                },
                root,
            ),
            Target::ExamplePicker => (m.terminal_example_picker.unwrap(), root),
            Target::ExampleChoice(n) => {
                let r = m
                    .ntp
                    .example_choices
                    .iter()
                    .find(|r| r.0 == *n)
                    .expect("Actual worked command choice");
                (r.1, r.2.intersect(ctx.content_rect()))
            }
            Target::DeviceChoice(ip) => {
                let row = m
                    .ntp
                    .choices
                    .iter()
                    .find(|r| r.0 == *ip)
                    .expect("Actual device picker choice");
                (row.1, row.2.intersect(ctx.content_rect()))
            }
            Target::Topology => {
                let (rect, clip) = m.ntp.topology.expect("Actual selected dependency");
                (rect, clip.intersect(root))
            }
            Target::Theme => (
                m.reopen_theme.expect("Actual header theme picker"),
                ctx.content_rect(),
            ),
            Target::Scale => (
                m.reopen_scale.expect("Actual header scale picker"),
                ctx.content_rect(),
            ),
            Target::HeaderChoice(k) => {
                let r = m
                    .physical_picker_choices
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| {
                        panic!(
                            "Actual header choice{k}, stage{}, choices{:?}",
                            self.smoke_stage, m.physical_picker_choices
                        )
                    });
                (r.1, r.2.intersect(ctx.content_rect()))
            }
            Target::Goal => (m.ntp.goal.unwrap(), root),
            Target::Notice => (m.ntp.notice.expect("Actual command-error notice"), root),
            Target::Feedback => {
                let r = m
                    .ntp
                    .rows
                    .iter()
                    .find(|r| !r.0)
                    .or_else(|| m.ntp.rows.first())
                    .expect("Actual checked result row");
                (r.1, root)
            }
        }
    }
    fn ntp_reveal(&mut self, ctx: &egui::Context, t: &Target, q: &mut Qa) -> bool {
        let now = ctx.input(|i| i.time);
        let (r, clip) = self.ntp_target(ctx, t);
        let tolerance = 1. / ctx.pixels_per_point();
        if now < q.wheel_until {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(q.wheel_until - now));
            return false;
        }
        let same = q.geometry.is_some_and(|old| {
            (old.min - r.min).length() < tolerance * 0.25
                && (old.max - r.max).length() < tolerance * 0.25
        });
        q.stable = if same { q.stable.saturating_add(1) } else { 0 };
        q.geometry = Some(r);
        if q.stable < 2 {
            ctx.request_repaint();
            return false;
        }
        if smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return true;
        }
        assert!(
            r.left() >= clip.left() - tolerance && r.right() <= clip.right() + tolerance,
            "NTP target horizontally clipped: {t:?}, {r:?}, {clip:?}"
        );
        assert!(
            q.wheels < 20,
            "Bounded NTP native reveal failed: {t:?}, {r:?}, {clip:?}, stage{}",
            q.stage
        );
        q.wheels += 1;
        let popup = matches!(
            t,
            Target::HeaderChoice(_) | Target::ExampleChoice(_) | Target::DeviceChoice(_)
        );
        let point = egui::pos2(
            if popup {
                r.center().x
            } else {
                clip.left() + 2.
            },
            clip.center().y,
        );
        self.pending_events.extend([
            egui::Event::PointerMoved(point),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (clip.center().y - r.center().y)
                        .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                ),
                modifiers: Default::default(),
            },
        ]);
        q.stable = 0;
        q.geometry = None;
        q.wheel_until = now + 0.15;
        ctx.request_repaint();
        false
    }
    fn ntp_click(&mut self, ctx: &egui::Context, t: &Target, q: &mut Qa) -> bool {
        if q.phase == 2 {
            return true;
        }
        if !self.ntp_reveal(ctx, t, q) {
            return false;
        }
        let (r, _) = self.ntp_target(ctx, t);
        let point = r.center();
        let hover = ctx.input(|i| i.pointer.hover_pos());
        if q.phase == 0 || hover.is_none_or(|p| p.distance(point) > 0.25 / ctx.pixels_per_point()) {
            self.pending_events.push(egui::Event::PointerMoved(point));
            q.phase = 1;
            ctx.request_repaint();
            return false;
        }
        if ctx.input(|i| i.pointer.delta().length()) >= 0.25 / ctx.pixels_per_point() {
            ctx.request_repaint();
            return false;
        }
        assert!(
            ctx.input(|i| i.pointer.delta().length()) < 0.25 / ctx.pixels_per_point(),
            "Hover must settle before press, never create a scroll drag"
        );
        self.pending_events
            .extend(
                [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    }),
            );
        q.phase = 2;
        ctx.request_repaint();
        false
    }
    pub(in crate::ui) fn ntp_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let mut q = ctx
            .data_mut(|d| d.remove_temp::<Qa>(key()))
            .unwrap_or_default();
        if q.finished {
            assert_eq!(
                q.closing.as_ref().unwrap(),
                &serde_json::to_value(&self.progress).unwrap(),
                "Delayed close may not change completed work"
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ctx.data_mut(|d| d.insert_temp(key(), q));
            return (false, None);
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        if q.last_root == Some(root) {
            ctx.data_mut(|d| d.insert_temp(key(), q));
            return (false, None);
        }
        q.last_root = Some(root);
        let now = ctx.input(|i| i.time);
        if q.stage != stage {
            q.stage = stage;
            q.phase = 0;
            q.polls = 0;
            q.began = now;
            q.geometry = None;
            q.stable = 0;
            q.wheels = 0;
            q.geometry_item = 0;
        }
        let waiting_for_wheel = now < q.wheel_until;
        if !waiting_for_wheel {
            q.polls += 1;
        }
        assert!(
            q.polls <= 300 && now - q.began < 15.,
            "NTP native stage budget: {stage}, {:?}, phase{}, polls{}, elapsed{}, now{}, wheel_until{}, wheels{}, stable{}, geometry{:?}, back{:?}, main{:?}, scroll{:?}, content{:?}, pointer{:?}, pointer_delta{:?}, selected{:?}",
            plan().get(stage - FIRST),
            q.phase,
            q.polls,
            now - q.began,
            now,
            q.wheel_until,
            q.wheels,
            q.stable,
            q.geometry,
            self.metrics.back,
            self.metrics.main_viewport,
            self.metrics.main_scroll_offset,
            self.metrics.main_content_size,
            ctx.input(|i| i.pointer.hover_pos()),
            ctx.input(|i| i.pointer.delta()),
            self.selected
        );
        // A deliberate smooth-wheel deadline is not another geometry/click
        // attempt. Fast optimized frames must not consume all300 work polls
        // while waiting150ms; the original15-second wall bound still applies.
        if waiting_for_wheel {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64(q.wheel_until - now));
            ctx.data_mut(|d| d.insert_temp(key(), q));
            return (false, None);
        }
        let step = plan()
            .get(stage - FIRST)
            .expect("Closed NTP stage inventory")
            .clone();
        let mut advance = true;
        let mut shot = None;
        match step {
            Step::Init => {
                assert!(self.smoke && !self.synthetic && !self.developer);
                assert_eq!((one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&self.scenarios).len(), one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_typed_units(&self.units).len()),(190,50));
                assert_eq!(
                    (
                        disk_registration::before_scenarios(&self.scenarios).len(),
                        disk_registration::before_typed_units(&self.units).len()
                    ),
                    (193, 51)
                );
                assert_eq!((self.scenarios.len(), self.units.len()), (197, 52));
                let prior = crate::ui::one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&self.scenarios);
                let units = crate::ui::one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_typed_units(&self.units);
                assert_eq!((prior.len(), units.len()), (186, 49));
                assert!(
                    self.progress.attempts.is_empty(),
                    "Real fresh isolated profile, no installed successes"
                );
                assert_eq!(self.units[49].id, "ntp-client-server");
                for (n, s) in sources().iter().enumerate() {
                    assert_eq!(
                        serde_json::to_value(&self.scenarios[186 + n]).unwrap(),
                        serde_json::to_value(s).unwrap()
                    );
                }
                assert_eq!(
                    IDS,
                    self.units[49]
                        .labs
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_slice()
                );
            }
            Step::Click(t) => advance = self.ntp_click(ctx, &t, &mut q),
            Step::Replace(text) => {
                assert!(text.len() <= 512 && !text.contains(['\n', '\r', '\0']));
                self.pending_events
                    .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            command: true,
                            ..Default::default()
                        },
                    }));
                self.pending_events.push(egui::Event::Paste(text));
            }
            Step::Command(text) => {
                let s = self.ntp_session();
                assert_eq!(
                    if self.progress.attempts[&self.scenarios[self.selected.unwrap()].id].challenge
                        || s.view != View::Example
                    {
                        &s.command
                    } else {
                        &s.example_command
                    },
                    &text
                );
            }
            Step::Key(key, modifiers) => {
                self.pending_events
                    .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    }))
            }
            Step::Focus(command) => {
                let expected = if command {
                    self.metrics.ntp.fields[0].2
                } else {
                    self.metrics.ntp.run_id.unwrap()
                };
                if ctx.memory(|m| m.focused()) != Some(expected) {
                    advance = false;
                    ctx.request_repaint();
                }
            }
            Step::Remember(name, scope) => {
                q.snapshots.insert(name, self.ntp_projection(scope));
            }
            Step::RememberWarm(name) => {
                q.warm.insert(name, self.ntp_warm());
            }
            Step::ResumeExact(name, via_next) => {
                self.ntp_resume_exact(
                    q.warm
                        .get(&name)
                        .expect("Observed preceding full warm snapshot"),
                    via_next,
                );
                if via_next {
                    q.next_returns += 1;
                } else {
                    q.continues += 1;
                }
            }
            Step::GuidanceDelta(name, count) => {
                let mut expected = q
                    .snapshots
                    .get(&name)
                    .expect("Actual pre-guidance fullprofile snapshot")
                    .clone();
                let id = &self.scenarios[self.selected.unwrap()].id;
                let old = expected["portable"]["attempts"][id]["hints"]
                    .as_u64()
                    .unwrap();
                let next = old.checked_add(count as u64).unwrap();
                assert!(next <= self.scenarios[self.selected.unwrap()].hints.len() as u64);
                expected["portable"]["attempts"][id]["hints"] = json!(next);
                assert_eq!(
                    self.ntp_projection(Scope::Progress),
                    expected,
                    "Requestedhelp changesonlytheobserved persistedhintcount; nograde/world/draft/practice/receipt/recency/favorite edits"
                );
            }
            Step::Same(name, scope) => assert_eq!(
                &self.ntp_projection(scope),
                q.snapshots.get(&name).unwrap(),
                "Actual scope changed: {name}"
            ),
            Step::Task(n) => assert_eq!(self.selected, Some(186 + n)),
            Step::Selected(ip) => {
                let session = self.ntp_session();
                let selected = if session.view == View::Example {
                    session.example_selected
                } else {
                    session.selected
                };
                assert_eq!(
                    selected.to_string(),
                    ip,
                    "Real picker selects actual supplied device"
                );
            }
            Step::Inspect => {
                let s = self.ntp_session();
                let expected = s
                    .current
                    .nodes
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                assert_eq!(
                    self.metrics
                        .ntp
                        .inspection_nodes
                        .iter()
                        .map(|r| r.0.clone())
                        .collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(
                    self.metrics.ntp.inspection_facts.len(),
                    s.current.deliveries.len()
                );
            }
            Step::Rows(expected, practice) => {
                let s = self.ntp_session();
                assert!(if practice {
                    s.example_checked
                } else {
                    s.checked
                });
                assert_eq!(
                    self.metrics
                        .ntp
                        .rows
                        .iter()
                        .map(|r| r.0)
                        .collect::<Vec<_>>(),
                    expected
                );
                if !practice {
                    assert_eq!(self.metrics.completion, expected.iter().all(|v| *v));
                }
            }
            Step::Completion(expected) => assert_eq!(self.metrics.completion, expected),
            Step::Notice(expected) => assert_eq!(self.notice.is_some(), expected),
            Step::Fresh => {
                let i = self.selected.unwrap();
                assert_eq!(
                    self.progress.attempts[&self.scenarios[i].id].state,
                    self.scenarios[i].initial
                );
            }
            Step::Save => {
                self.save();
                assert!(!self.dirty && self.storage_error.is_none());
                let raw = progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap();
                let mut saved: Value = serde_json::from_slice(&raw).unwrap();
                saved.as_object_mut().unwrap().remove("migration_receipts");
                let mut caller = serde_json::to_value(&self.progress).unwrap();
                caller.as_object_mut().unwrap().remove("migration_receipts");
                assert_eq!(saved, caller);
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let mut observed = serde_json::to_value(loaded).unwrap();
                observed
                    .as_object_mut()
                    .unwrap()
                    .remove("migration_receipts");
                assert_eq!(
                    observed, caller,
                    "Read-only reopened profile is never installed"
                );
                assert_eq!(
                    progress::read_bounded(&self.path, 2 * 1024 * 1024).unwrap(),
                    raw
                );
            }
            Step::InvalidPacket => {
                self.pending_events.push(egui::Event::Paste(format!(
                    "show ntp status{} unsupported-tail",
                    " ".repeat(512)
                )));
                self.pending_events
                    .extend([true, false].into_iter().map(|pressed| egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    }));
            }
            Step::Window(wide) => {
                if q.phase == 0 {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(wide));
                    if !wide {
                        let n = ctx.input(|i| i.viewport().native_pixels_per_point.unwrap_or(1.));
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1100. / n,
                            950. / n,
                        )));
                    }
                    q.phase = 1;
                    advance = false;
                } else {
                    let p = ctx.content_rect().size() * ctx.pixels_per_point();
                    let yes = ctx.input(|i| i.viewport().fullscreen) == Some(wide)
                        && if wide { p.x > 1400. } else { p.x <= 1300. };
                    if !yes {
                        advance = false;
                        ctx.request_repaint();
                    }
                }
            }
            Step::Setting(scale, theme) => {
                assert_eq!(self.theme_id, theme);
                assert!((self.scale - scale).abs() < 0.001);
            }
            Step::Geometry => {
                let targets = [
                    Target::Goal,
                    Target::Control("device"),
                    Target::Field,
                    Target::Control("run"),
                    Target::Control("check"),
                    Target::Topology,
                ];
                if !self.ntp_reveal(ctx, &targets[q.geometry_item], &mut q) {
                    advance = false;
                } else {
                    let tolerance = 1. / ctx.pixels_per_point();
                    for (k, r, clip) in &self.metrics.ntp.controls {
                        assert!(
                            r.left() >= clip.left() - tolerance
                                && r.right() <= clip.right() + tolerance,
                            "NTP control{k} horizontal clipping"
                        );
                    }
                    for (k, r, _, clip) in &self.metrics.ntp.fields {
                        assert!(
                            r.left() >= clip.left() - tolerance
                                && r.right() <= clip.right() + tolerance,
                            "NTP field{k} horizontal clipping"
                        );
                    }
                    q.geometry_item += 1;
                    if q.geometry_item < targets.len() {
                        q.geometry = None;
                        q.stable = 0;
                        q.wheels = 0;
                        advance = false;
                    }
                }
            }
            Step::Shot(name, target) => {
                if !self.ntp_reveal(ctx, &target, &mut q) {
                    advance = false;
                } else {
                    q.shots += 1;
                    assert!(q.shots <= planned_captures().len());
                    shot = Some(name);
                }
            }
            Step::Done => {
                assert_eq!(q.shots, planned_captures().len());
                for id in IDS {
                    let s = self.scenarios.iter().find(|s| s.id == id).unwrap();
                    assert!(progress::current_solved(&self.progress.attempts[id], s));
                }
                assert_eq!(
                    self.progress.attempts.len(),
                    4,
                    "Only real visited tasks exist"
                );
                q.closing = Some(serde_json::to_value(&self.progress).unwrap());
                q.finished = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                println!(
                    "PASS native NTP normal190: four ordinary tasks, isolated worked examples/experiment repairs, wrong/partial/reference/two alternatives, current invalidation, reset/cancel/warm resume, native keyboard/paste,20 scale/theme/width settings; screenshots{}; no host clocks/network, novice, screen-reader or other-platform acceptance",
                    q.shots
                );
                advance = false;
            }
        }
        ctx.data_mut(|d| d.insert_temp(key(), q));
        (advance, shot)
    }
}

#[cfg(test)]
pub(in crate::ui) fn first_back_stage() -> usize {
    FIRST
        + plan()
            .iter()
            .position(|s| matches!(s, Step::Click(Target::Back)))
            .unwrap()
}
#[cfg(test)]
pub(in crate::ui) fn wheel_wait_observation(ctx: &egui::Context) -> Option<(usize, f64, usize)> {
    ctx.data(|d| {
        d.get_temp::<Qa>(key())
            .map(|q| (q.wheels, q.wheel_until, q.polls))
    })
}
