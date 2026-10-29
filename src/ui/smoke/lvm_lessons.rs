//! Fixed ordinary-course native acceptance route. No progress/grade/focus injection.
use super::*;
use dlms_learning_lab::{lvm_workspace as work, script_teaching::View};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};
pub(in crate::ui) const FIRST: usize = 78000;
const RESERVE: usize = 2000;
const IDS: [&str; 4] = [
    "lpic2-lvm-free-space",
    "lpic2-lvm-add-pv",
    "lpic2-lvm-filesystem-gap",
    "lpic2-lvm-handover",
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
    Theme,
    Scale,
    HeaderChoice(String),
    Goal,
    Feedback,
    Notice,
    Files,
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
    RememberWarm(String),
    ResumeExact(String, bool),
    Task(usize),
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
        serde_json::from_str::<pack::Pack>(include_str!("../../../scenarios/lvm-growth-v1.json"))
            .unwrap()
            .scenarios
    })
}
fn review() -> &'static Value {
    static R: OnceLock<Value> = OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/lvm-growth-review.json"
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
fn command(p: &mut Vec<Step>, s: &str, enter: bool) {
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
                "lvm-normal-library-search-placeholder".into(),
                Target::Search,
            ),
        ];
        for (n, s) in sources().iter().enumerate() {
            let r = &review()["labs"][n];
            assert_eq!(r["id"], s.id);
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
            check(&mut p, rows(&r["initial_rows"]), false);
            p.push(Step::Shot(
                format!("{}-whole-assignment", s.id),
                Target::Goal,
            ));
            if n == 0 {
                p.extend([
                    Step::Click(Target::Control("inspect-files")),
                    Step::Shot("lvm-assignment-retained-files".into(), Target::Files),
                    Step::Click(Target::Control("inspect-files")),
                ]);
            }
            p.push(Step::Remember(
                "assignment-before-practice".into(),
                Scope::Assignment,
            ));
            p.push(Step::Click(Target::Control("example")));
            let engine::Goal::Lvm { definition } = &s.goal else {
                unreachable!()
            };
            p.extend([
                Step::Click(Target::ExamplePicker),
                Step::Click(Target::ExampleChoice(0)),
                Step::Click(Target::Control("insert")),
                Step::Command(definition.lesson.worked[0].text.clone()),
                Step::Same("assignment-before-practice".into(), Scope::Assignment),
                Step::Completion(false),
                Step::Shot(format!("{}-example-inserted-not-run", s.id), Target::Field),
            ]);
            for c in &definition.lesson.worked {
                command(&mut p, &c.text, false);
            }
            let pr = vec![true; [5, 5, 5, 9][n]];
            check(&mut p, pr.clone(), true);
            command(&mut p, &definition.lesson.experiment, true);
            let mut changed_practice = pr.clone();
            changed_practice[3] = false;
            check(&mut p, changed_practice, true);
            p.push(Step::Completion(false));
            p.push(Step::Shot(
                format!("{}-example-experiment", s.id),
                Target::Feedback,
            ));
            if n == 0 {
                p.extend([
                    Step::Click(Target::Control("inspect-files")),
                    Step::Shot("lvm-example-retained-files".into(), Target::Files),
                    Step::Click(Target::Control("inspect-files")),
                ]);
            }
            command(&mut p, &definition.lesson.repair, false);
            check(&mut p, pr, true);
            p.extend([
                Step::Same("assignment-before-practice".into(), Scope::Assignment),
                Step::Completion(false),
                Step::Click(Target::Control("assignment")),
                Step::Same("assignment-before-practice".into(), Scope::Assignment),
                Step::Remember("wrong-world".into(), Scope::Current),
            ]);
            for c in r["wrong"]["commands"].as_array().unwrap() {
                command(&mut p, c.as_str().unwrap(), true);
            }
            p.extend([
                Step::Same("wrong-world".into(), Scope::Current),
                Step::Notice(true),
            ]);
            p.push(Step::Shot(
                format!("{}-command-refusal", s.id),
                Target::Notice,
            ));
            check(&mut p, rows(&r["wrong"]["rows"]), false);
            p.push(Step::Shot(
                format!("{}-wrong-retry", s.id),
                Target::Feedback,
            ));
            command(&mut p, "lvs", false);
            p.push(Step::Notice(false));
            for c in r["partial"]["commands"].as_array().unwrap() {
                command(&mut p, c.as_str().unwrap(), false);
            }
            check(&mut p, rows(&r["partial"]["rows"]), false);
            p.push(Step::Shot(
                format!("{}-partial-capacity", s.id),
                Target::Feedback,
            ));
            // Start the reference from an actual selected Reset, not a state replacement.
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
            check(&mut p, vec![true; [4, 4, 4, 7][n]], false);
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
                check(&mut p, vec![true; [4, 4, 4, 7][n]], false);
                p.push(Step::Completion(true));
                p.push(Step::Shot(
                    format!("{}-alternate-{}", s.id, a + 1),
                    Target::Feedback,
                ));
            }
            if n == 0 {
                // The second alternate consumed all free extents; reset and replay
                // the16MiB reference before intentionally damaging the neighbor.
                reset(&mut p);
                for c in &s.solution {
                    command(&mut p, c, false);
                }
                check(&mut p, vec![true; 4], false);
                command(&mut p, "lvextend -L +4M records/neighbor", true);
                p.push(Step::Click(Target::Control("check")));
                p.push(Step::Completion(false));
                p.push(Step::Shot(
                    format!("{}-success-invalidated", s.id),
                    Target::Feedback,
                ));
                reset(&mut p);
                for c in &s.solution {
                    command(&mut p, c, false);
                }
                check(&mut p, vec![true; 4], false);
            }
            // Exact unfinished command survives real Back/Continue. No command executes.
            p.extend([
                Step::Click(Target::Field),
                Step::Replace("unfinished handover command |".into()),
                Step::RememberWarm("resume-attempt".into()),
                Step::Save,
                Step::Click(Target::Back),
                Step::Click(Target::Start(n)),
                Step::Task(n),
                Step::ResumeExact("resume-attempt".into(), false),
            ]);
            p.extend([
                Step::Remember("cancel-progress".into(), Scope::Progress),
                Step::Click(Target::Reset),
                Step::Click(Target::Cancel),
                Step::Same("cancel-progress".into(), Scope::Progress),
            ]);
            p.extend([
                Step::Click(Target::Field),
                Step::Remember("bad-packet".into(), Scope::Progress),
                Step::InvalidPacket,
                Step::Same("bad-packet".into(), Scope::Progress),
                Step::Notice(true),
            ]);
            command(&mut p, "lvs", false);
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
            ]);
            p.extend([
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
                    ]);
                    let name = format!(
                        "lvm-handover-layout-{}-{}-{}",
                        (scale * 100.) as u32,
                        if light { "light" } else { "dark" },
                        if wide { "wide" } else { "narrow" }
                    );
                    p.push(Step::Shot(name, Target::Field));
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
    egui::Id::new("lvm-native186-state")
}
#[cfg(test)]
pub(in crate::ui) fn phase_bounds() -> (usize, usize) {
    (FIRST, FIRST + plan().len() - 1)
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
    pub(in crate::ui) fn lvm_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("lvm-lessons")).then_some(FIRST)
    }
    pub(in crate::ui) fn lvm_capture_active(&self) -> bool {
        self.smoke && (FIRST..FIRST + RESERVE).contains(&self.smoke_stage)
    }
    fn lvm_session(&self) -> &work::Session {
        let i = self.selected.expect("Actual LVM selection");
        let source = &self.scenarios[i];
        let a = &self.progress.attempts[&source.id];
        work::context(a, source).unwrap();
        let engine::State::Lvm { simulation } = &a.state else {
            panic!("Real LVM attempt")
        };
        simulation
    }
    fn lvm_projection(&self, scope: Scope) -> Value {
        let i = self.selected.unwrap();
        let id = &self.scenarios[i].id;
        let s = self.lvm_session();
        match scope {
            Scope::Assignment => {
                json!({"current":s.current,"command":s.command,"transcript":s.transcript,"checked":s.checked})
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
    fn lvm_warm(&mut self) -> Warm {
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
    fn lvm_resume_exact(&mut self, before: &Warm, via_next: bool) {
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
        let now = self.lvm_warm();
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
    fn lvm_target(&self, ctx: &egui::Context, t: &Target) -> (egui::Rect, egui::Rect) {
        let root = self
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        let m = &self.metrics;
        match t {
            Target::Search => (m.library_search.expect("Library search"), root),
            Target::Start(n) => {
                let at = m
                    .card_verification
                    .iter()
                    .position(|r| r.0 == IDS[*n])
                    .expect("Actual filtered LVM card");
                (m.card_actions[at].0, root)
            }
            Target::Field => {
                let r = m
                    .lvm
                    .fields
                    .iter()
                    .find(|r| r.0 == "command")
                    .expect("LVM command field");
                (r.1, r.3.intersect(root))
            }
            Target::Control(k) => {
                let r = m
                    .lvm
                    .controls
                    .iter()
                    .find(|r| r.0 == *k)
                    .unwrap_or_else(|| panic!("Actual LVM control{k}"));
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
                    .lvm
                    .example_choices
                    .iter()
                    .find(|r| r.0 == *n)
                    .expect("Actual worked command choice");
                (r.1, r.2.intersect(ctx.content_rect()))
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
            Target::Files => (
                m.lvm.files_viewport.expect("Actual opened file inspector"),
                root,
            ),
            Target::Goal => (m.lvm.goal.unwrap(), root),
            Target::Notice => (m.lvm.notice.expect("Actual command-error notice"), root),
            Target::Feedback => {
                let r = m
                    .lvm
                    .rows
                    .iter()
                    .find(|r| !r.0)
                    .or_else(|| m.lvm.rows.first())
                    .expect("Actual checked result row");
                (r.1, root)
            }
        }
    }
    fn lvm_reveal(&mut self, ctx: &egui::Context, t: &Target, q: &mut Qa) -> bool {
        let now = ctx.input(|i| i.time);
        let (r, clip) = self.lvm_target(ctx, t);
        let tolerance = 1. / ctx.pixels_per_point();
        if now < q.wheel_until {
            ctx.request_repaint();
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
            "LVM target horizontally clipped: {t:?}, {r:?}, {clip:?}"
        );
        assert!(
            q.wheels < 20,
            "Bounded LVM native reveal failed: {t:?}, {r:?}, {clip:?}, stage{}",
            q.stage
        );
        q.wheels += 1;
        let popup = matches!(t, Target::HeaderChoice(_) | Target::ExampleChoice(_));
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
    fn lvm_click(&mut self, ctx: &egui::Context, t: &Target, q: &mut Qa) -> bool {
        if q.phase == 2 {
            return true;
        }
        if !self.lvm_reveal(ctx, t, q) {
            return false;
        }
        let (r, _) = self.lvm_target(ctx, t);
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
    pub(in crate::ui) fn lvm_lessons_native(
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
        q.polls += 1;
        assert!(
            q.polls <= 300 && now - q.began < 15.,
            "LVM native stage budget: {stage}, {:?}, phase{}, polls{}",
            plan().get(stage - FIRST),
            q.phase,
            q.polls
        );
        let step = plan()
            .get(stage - FIRST)
            .expect("Closed LVM stage inventory")
            .clone();
        let mut advance = true;
        let mut shot = None;
        match step {
            Step::Init => {
                assert!(self.smoke && !self.synthetic && !self.developer);
                assert_eq!((self.scenarios.len(), self.units.len()), (186, 49));
                assert!(
                    self.progress.attempts.is_empty(),
                    "Real fresh isolated profile, no installed successes"
                );
                assert_eq!(self.units[48].id, "lvm-volume-growth");
                for (n, s) in sources().iter().enumerate() {
                    assert_eq!(
                        serde_json::to_value(&self.scenarios[182 + n]).unwrap(),
                        serde_json::to_value(s).unwrap()
                    );
                }
                assert_eq!(
                    IDS,
                    self.units[48]
                        .labs
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_slice()
                );
            }
            Step::Click(t) => advance = self.lvm_click(ctx, &t, &mut q),
            Step::Replace(text) => {
                assert!(text.len() <= 2048 && !text.contains(['\n', '\r', '\0']));
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
                let s = self.lvm_session();
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
                    self.metrics.lvm.fields[0].2
                } else {
                    self.metrics.lvm.run_id.unwrap()
                };
                if ctx.memory(|m| m.focused()) != Some(expected) {
                    advance = false;
                    ctx.request_repaint();
                }
            }
            Step::Remember(name, scope) => {
                q.snapshots.insert(name, self.lvm_projection(scope));
            }
            Step::RememberWarm(name) => {
                q.warm.insert(name, self.lvm_warm());
            }
            Step::ResumeExact(name, via_next) => {
                self.lvm_resume_exact(
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
            Step::Same(name, scope) => assert_eq!(
                &self.lvm_projection(scope),
                q.snapshots.get(&name).unwrap(),
                "Actual scope changed: {name}"
            ),
            Step::Task(n) => assert_eq!(self.selected, Some(182 + n)),
            Step::Rows(expected, practice) => {
                let s = self.lvm_session();
                assert!(if practice {
                    s.example_checked
                } else {
                    s.checked
                });
                assert_eq!(
                    self.metrics
                        .lvm
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
                    "vgs{} unsupported-tail",
                    " ".repeat(2048)
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
                    Target::Field,
                    Target::Control("run"),
                    Target::Control("check"),
                ];
                if !self.lvm_reveal(ctx, &targets[q.geometry_item], &mut q) {
                    advance = false;
                } else {
                    let tolerance = 1. / ctx.pixels_per_point();
                    for (k, r, clip) in &self.metrics.lvm.controls {
                        assert!(
                            r.left() >= clip.left() - tolerance
                                && r.right() <= clip.right() + tolerance,
                            "LVM control{k} horizontal clipping"
                        );
                    }
                    for (k, r, _, clip) in &self.metrics.lvm.fields {
                        assert!(
                            r.left() >= clip.left() - tolerance
                                && r.right() <= clip.right() + tolerance,
                            "LVM field{k} horizontal clipping"
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
                if !self.lvm_reveal(ctx, &target, &mut q) {
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
                    "PASS native LVM normal186: four ordinary tasks, isolated worked examples/experiment repairs, wrong/partial/reference/two alternatives, current invalidation, reset/cancel/warm resume, native keyboard/paste,20 scale/theme/width settings; screenshots{}; no host LVM, novice, screen-reader or other-platform acceptance",
                    q.shots
                );
                advance = false;
            }
        }
        ctx.data_mut(|d| d.insert_temp(key(), q));
        (advance, shot)
    }
}
