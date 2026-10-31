//! Focused real-event normal199 route. No learner state or result is installed.
use super::*;
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration::literal_registration as registration;
pub(in crate::ui) const BASE: usize = 96000;
pub(in crate::ui) const IDS: [&str; 2] = ["shell-trim-options", "shell-trim-manifest"];
const LOOP: &str = "while true; do :; done\n";
#[derive(Clone, Copy, Debug)]
enum Target {
    Example,
    Assignment,
    Run,
    Stop,
    Back,
    Reset,
    Cancel,
    Confirm,
    Scale,
    Theme,
    ScaleItem(f32),
    ThemeItem(&'static str),
    Challenge,
    Learn,
}
#[derive(Clone, Debug)]
enum Step {
    Setup,
    Start,
    Click(Target),
    Focus,
    Type(String),
    Practice(shell::Case),
    Assignment {
        source: String,
        pass: bool,
        named: Vec<(String, bool)>,
    },
    FreshPractice,
    FreshAssignment,
    Invalidate,
    Stopped,
    Shot(&'static str),
    Goal(bool, &'static str),
    Geometry,
    Snapshot,
    Warm,
    Continued,
    Cancelled,
    ResetDone,
    Mode(bool),
    Window(bool),
    Finish,
}
#[derive(Clone, Debug)]
struct Action {
    task: usize,
    step: Step,
}
fn authored() -> &'static Vec<pack::Scenario> {
    static PACK: std::sync::OnceLock<Vec<pack::Scenario>> = std::sync::OnceLock::new();
    PACK.get_or_init(|| {
        serde_json::from_str::<pack::Pack>(include_str!(
            "../../../scenarios/shell-literal-normalization-draft.json"
        ))
        .unwrap()
        .scenarios
    })
}
fn proof() -> &'static serde_json::Value {
    static REVIEW: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    REVIEW.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-literal-normalization-review.json"
        ))
        .unwrap()
    })
}
fn named(w: &serde_json::Value, s: &pack::Scenario) -> Vec<(String, bool)> {
    let engine::Goal::Script { cases } = &s.goal else {
        panic!()
    };
    // Archived reviewer names remain in supplemental guards. Native checks use
    // explicit currently graded Cases, bound to the exact supported source.
    if s.id == IDS[0] {
        let hash =
            dlms_learning_lab::developer_digest::hash(w["source"].as_str().unwrap().as_bytes());
        let mut current = vec![(
            "Full mode retains inline source spaces and equals at both name bounds".to_owned(),
            false,
        )];
        match hash.as_str() {
            "7d67d6f5a372a30fce068a6afe0d2fe70e20e93f180d5fa86990777c7a19592d" => {}
            "90c02817beeea443b426f56c74d266a25dfab420eda7ee82709a0ca967aff1fa" => {
                current[0].1 = true;
                current.push(("Count handles blank and final nonLF rows with separate source and repeated flag".into(),false));
            }
            _ => panic!("Unreviewed options native witness source"),
        }
        assert!(
            current
                .iter()
                .all(|(name, _)| cases.iter().any(|c| &c.name == name))
        );
        return current;
    }
    let mut result = Vec::new();
    for (key, pass) in [
        ("must_pass_case_names", true),
        ("must_fail_case_names", false),
    ] {
        for n in w[key].as_array().unwrap() {
            let n = n.as_str().unwrap();
            if cases.iter().any(|c| c.name == n) {
                result.push((n.into(), pass));
            }
        }
    }
    assert!(result.iter().any(|(_, p)| !p));
    result
}
// Independent full outcome predictions, asserted only against real UI job records.
// No VM or grading action is invoked by this helper.
fn options_witness_outcome(source: &str, c: &shell::Case) -> Option<shell::Outcome> {
    let hash = dlms_learning_lab::developer_digest::hash(source.as_bytes());
    let full = "Full mode retains inline source spaces and equals at both name bounds";
    let count = "Count handles blank and final nonLF rows with separate source and repeated flag";
    let (stdout, stderr, status, files) = match (hash.as_str(), c.name.as_str()) {
        ("7d67d6f5a372a30fce068a6afe0d2fe70e20e93f180d5fa86990777c7a19592d", name)
            if name == full =>
        {
            assert!(!c.files.contains_key("/work/a1-b_c.d"));
            (String::new(), "Missing source\n".into(), 1, c.files.clone())
        }
        ("90c02817beeea443b426f56c74d266a25dfab420eda7ee82709a0ca967aff1fa", name)
            if name == full =>
        {
            (
                c.stdout.clone(),
                c.stderr.clone(),
                c.status,
                c.after.clone(),
            )
        }
        ("90c02817beeea443b426f56c74d266a25dfab420eda7ee82709a0ca967aff1fa", name)
            if name == count =>
        {
            let mut files = c.files.clone();
            let data = files.get("/work/a").unwrap();
            let text = format!("Source:a\nMode:full\n{data}");
            files.insert("/work/a1-b_c.d=e f=ghi".into(), text);
            (
                "Published:a1-b_c.d=e f=ghi\n".into(),
                String::new(),
                0,
                files,
            )
        }
        _ => return None,
    };
    Some(shell::Outcome {
        stdout,
        stderr,
        status,
        files,
        error: None,
    })
}
fn plan() -> &'static Vec<Action> {
    static PLAN: std::sync::OnceLock<Vec<Action>> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| {
        let mut result = Vec::new();
        for (task, s) in authored().iter().enumerate() {
            assert_eq!(s.id, IDS[task]);
            let lesson = s.teaching.as_ref().unwrap();
            let row = &proof()["labs"][task];
            let reference = s.solution.join("\n");
            let mut steps = vec![
                Step::Setup,
                Step::Geometry,
                Step::Start,
                Step::Shot("introduction"),
                Step::Click(Target::Example),
                Step::FreshPractice,
                Step::Shot("isolated-practice-ready"),
                Step::Click(Target::Run),
                Step::Practice(lesson.fixture.clone()),
                Step::Shot("practice-run"),
                Step::Focus,
                Step::Type(row["try_change"]["source"].as_str().unwrap().into()),
                Step::Click(Target::Run),
                Step::Practice(serde_json::from_value(row["try_change"]["case"].clone()).unwrap()),
                Step::Shot("practice-experiment"),
                Step::Focus,
                Step::Type(lesson.example.clone()),
                Step::FreshPractice,
                Step::Click(Target::Run),
                Step::Practice(lesson.fixture.clone()),
                Step::Shot("practice-repaired-replay"),
                Step::Click(Target::Assignment),
                Step::FreshAssignment,
                Step::Goal(false, "whole-assignment-top"),
                Step::Goal(true, "whole-assignment-bottom"),
            ];
            // The manifest witness finishes input reading before publication;
            // its failure is supported behavior, not active-input Unsupported.
            for (index, label) in [
                (if task == 0 { 0 } else { 7 }, "wrong-feedback"),
                (if task == 0 { 4 } else { 3 }, "partial-feedback"),
            ] {
                let wrong = &row["wrong"][index];
                let source = wrong["source"].as_str().unwrap().to_owned();
                steps.extend([
                    Step::Focus,
                    Step::Type(source.clone()),
                    Step::Click(Target::Run),
                    Step::Assignment {
                        source,
                        pass: false,
                        named: named(wrong, s),
                    },
                    Step::Shot(label),
                ]);
            }
            let alts: Vec<String> = serde_json::from_value(row["alternatives"].clone()).unwrap();
            assert_eq!(alts.len(), 2);
            for (source, label) in [
                (reference.clone(), "reference-complete"),
                (alts[0].clone(), "alternate-one-complete"),
                (alts[1].clone(), "alternate-two-complete"),
            ] {
                steps.extend([
                    Step::Focus,
                    Step::Type(source.clone()),
                    Step::Click(Target::Run),
                    Step::Assignment {
                        source,
                        pass: true,
                        named: vec![],
                    },
                    Step::Shot(label),
                ]);
            }
            let edit = format!("{reference}\n# Changed current source needs a new Run\n");
            steps.extend([
                Step::Focus,
                Step::Type(edit.clone()),
                Step::Invalidate,
                Step::Shot("current-edit-invalidates"),
                Step::Click(Target::Run),
                Step::Assignment {
                    source: edit,
                    pass: true,
                    named: vec![],
                },
                Step::Focus,
                Step::Type(LOOP.into()),
                Step::Click(Target::Run),
                Step::Click(Target::Stop),
                Step::Stopped,
                Step::Shot("real-stop"),
                Step::Focus,
                Step::Type(format!("{reference}\nprintf 'unfinished draft")),
                Step::Invalidate,
                Step::Warm,
                Step::Click(Target::Back),
                Step::Start,
                Step::Continued,
                Step::Shot("normal-continue-keeps-draft"),
                Step::Snapshot,
                Step::Click(Target::Reset),
                Step::Click(Target::Cancel),
                Step::Cancelled,
                Step::Click(Target::Reset),
                Step::Click(Target::Confirm),
                Step::ResetDone,
                Step::Shot("selected-reset"),
                Step::Click(Target::Assignment),
            ]);
            for scale in [1., 1.5, 2.] {
                for (theme, label) in [("dlms-light", "light"), ("dlms-dark", "dark")] {
                    let source = format!(
                        "{reference}\n# Native {label} {:.0} percent edit\n",
                        scale * 100.
                    );
                    steps.extend([
                        Step::Click(Target::Scale),
                        Step::Click(Target::ScaleItem(scale)),
                        Step::Click(Target::Theme),
                        Step::Click(Target::ThemeItem(theme)),
                        Step::Window(scale != 2.),
                        Step::Geometry,
                    ]);
                    if scale == 2. {
                        steps.extend([
                            Step::Goal(
                                false,
                                if theme == "dlms-light" {
                                    "goal200-light-top"
                                } else {
                                    "goal200-dark-top"
                                },
                            ),
                            Step::Goal(
                                true,
                                if theme == "dlms-light" {
                                    "goal200-light-bottom"
                                } else {
                                    "goal200-dark-bottom"
                                },
                            ),
                        ]);
                    }
                    steps.extend([
                        Step::Focus,
                        Step::Type(source.clone()),
                        Step::Invalidate,
                        Step::Click(Target::Run),
                        Step::Assignment {
                            source,
                            pass: true,
                            named: vec![],
                        },
                        Step::Shot(match (scale, theme) {
                            (1., "dlms-light") => "light100-complete",
                            (1., _) => "dark100-complete",
                            (1.5, "dlms-light") => "light150-complete",
                            (1.5, _) => "dark150-complete",
                            (2., "dlms-light") => "light200-complete",
                            _ => "dark200-complete",
                        }),
                    ]);
                }
            }
            steps.extend([
                Step::Snapshot,
                Step::Click(Target::Challenge),
                Step::Mode(true),
                Step::Shot("challenge-independent"),
                Step::Click(Target::Learn),
                Step::Mode(false),
                Step::Warm,
                Step::Click(Target::Back),
            ]);
            result.extend(steps.into_iter().map(|step| Action { task, step }));
        }
        result.push(Action {
            task: 1,
            step: Step::Finish,
        });
        assert!(result.len() < 1000);
        result
    })
}
pub(in crate::ui) fn last_stage() -> usize {
    BASE + plan().len() - 1
}
pub(in crate::ui) fn immediate_stop(stage: usize) -> bool {
    if stage < BASE || stage > last_stage() {
        return false;
    }
    let here = &plan()[stage - BASE].step;
    matches!(here, Step::Click(Target::Stop) | Step::Stopped)
        || matches!(here, Step::Click(Target::Run))
            && plan()
                .get(stage - BASE + 1)
                .is_some_and(|a| matches!(a.step, Step::Click(Target::Stop)))
}
pub(in crate::ui) fn inventory() -> (usize, Vec<String>) {
    let mut shots = Vec::new();
    for a in plan() {
        if let Step::Shot(label) | Step::Goal(_, label) = a.step {
            shots.push(format!("{}-{label}", IDS[a.task]));
        }
    }
    (plan().len(), shots)
}
fn full(p: &Progress) -> serde_json::Value {
    let mut v = serde_json::to_value(p).unwrap();
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
#[derive(Clone)]
struct Warm {
    full: serde_json::Value,
    raw: Vec<u8>,
    raw_value: serde_json::Value,
    loaded: serde_json::Value,
}
pub(in crate::ui) fn goal_rows(
    ctx: &egui::Context,
    text: &str,
    visible: egui::Rect,
) -> Option<(Vec<egui::Rect>, egui::Rect)> {
    fn find(
        s: &egui::Shape,
        clip: egui::Rect,
        text: &str,
        visible: egui::Rect,
    ) -> Option<(Vec<egui::Rect>, egui::Rect)> {
        match s {
            egui::Shape::Text(s) if s.galley.text() == text => {
                assert!(!s.galley.elided);
                let rows = s
                    .galley
                    .rows
                    .iter()
                    .filter(|r| !r.text().trim().is_empty())
                    .map(|r| r.rect().translate(s.pos.to_vec2()))
                    .collect::<Vec<_>>();
                assert!(!rows.is_empty() && rows.len() <= 128);
                Some((rows, clip.intersect(visible)))
            }
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| find(s, clip, text, visible)),
            _ => None,
        }
    }
    ctx.graphics(|g| {
        g.get(egui::LayerId::background()).and_then(|l| {
            l.all_entries()
                .find_map(|s| find(&s.shape, s.clip_rect, text, visible))
        })
    })
}
// Match the existing ERE route: acknowledge observed fullscreen state and a
// distinct physical width, rather than assuming the WM honors InnerSize.
pub(in crate::ui) fn window_acknowledged(
    fullscreen: Option<bool>,
    physical: egui::Vec2,
    baseline: f32,
    wide: bool,
) -> bool {
    fullscreen == Some(wide)
        && physical.x.is_finite()
        && physical.y.is_finite()
        && physical.x > 0.
        && physical.y > 0.
        && baseline.is_finite()
        && baseline > 0.
        && if wide {
            physical.x > baseline * 1.25
        } else {
            physical.x <= baseline * 1.05
        }
}
impl LabApp {
    pub(in crate::ui) fn literal_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("literal-lessons")).then_some(BASE)
    }
    pub(in crate::ui) fn literal_capture_active(&self) -> bool {
        self.smoke && (BASE..=last_stage()).contains(&self.smoke_stage)
    }
    fn literal_window(&mut self, ctx: &egui::Context, wide: bool) -> bool {
        let baseline_key = egui::Id::new("literal-windowed-width");
        if ctx.data(|d| d.get_temp::<f32>(baseline_key)).is_none() {
            let fullscreen = ctx.input(|i| i.viewport().fullscreen);
            let physical = ctx.content_rect().size() * ctx.pixels_per_point();
            let started = ctx.data_mut(|d| {
                let key = egui::Id::new("literal-initial-window-wait");
                let started = d.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
                d.insert_temp(key, started);
                started
            });
            assert!(
                started.elapsed().as_secs() < 15,
                "Initial windowed geometry was not observed: fullscreen{fullscreen:?} physical{physical:?}"
            );
            if fullscreen != Some(false) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                self.started = Instant::now();
                ctx.request_repaint();
                return false;
            }
            assert!(physical.is_finite() && physical.x > 0. && physical.y > 0.);
            eprintln!("literal initial windowed physical{physical:?}");
            ctx.data_mut(|d| d.insert_temp(baseline_key, physical.x));
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(wide));
        if !wide {
            // A window manager may ignore this hint. The observed baseline is authority.
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(760., 900.)));
        }
        let requested = (wide, Instant::now());
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("literal-requested-window"), requested));
        true
    }
    fn literal_stable_click(&mut self, ctx: &egui::Context, rect: egui::Rect, phase: u8) -> bool {
        let key = egui::Id::new(("literal-selector-hover-geometry", self.smoke_stage));
        if phase == 0 {
            ctx.data_mut(|d| d.insert_temp(key, (rect, Instant::now(), 0usize)));
        } else {
            let (hovered, started, moves) = ctx
                .data(|d| d.get_temp::<(egui::Rect, Instant, usize)>(key))
                .expect("A real hover precedes a selector press");
            let tolerance = 1. / ctx.pixels_per_point();
            if rect
                .min
                .distance(hovered.min)
                .max(rect.max.distance(hovered.max))
                > tolerance
            {
                assert!(
                    moves < 20 && started.elapsed().as_secs() < 15,
                    "Selector geometry must settle before a real press"
                );
                self.pending_events
                    .push(egui::Event::PointerMoved(rect.center()));
                ctx.data_mut(|d| d.insert_temp(key, (rect, started, moves + 1)));
                self.started = Instant::now();
                ctx.request_repaint();
                return false;
            }
        }
        self.click_smoke_control(ctx, Some(rect))
    }
    fn literal_click(
        &mut self,
        ctx: &egui::Context,
        target: Target,
        rect: Option<egui::Rect>,
    ) -> bool {
        let phase = ctx
            .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", self.smoke_stage))))
            .unwrap_or(0);
        if phase >= 2 {
            return self.click_smoke_control(ctx, None);
        }
        if matches!(target, Target::Stop) {
            assert!(
                self.script_job.is_some(),
                "Stop must observe the actual running job before press"
            );
        }
        let popup = match target {
            Target::ScaleItem(v) => Some(format!("scale:{v:.2}")),
            Target::ThemeItem(v) => Some(format!("theme:{v}")),
            _ => None,
        };
        if let Some(key) = popup {
            let (_, r, clip) = self
                .metrics
                .physical_picker_choices
                .iter()
                .find(|(name, _, _)| name == &key)
                .unwrap_or_else(|| panic!(
                    "actual painted selector choice: stage={} target={target:?} key={key} phase={phase} scale={} choices={:?} scale_rect={:?} theme_rect={:?} popup={} pointer={:?}",
                    self.smoke_stage, self.scale,
                    self.metrics.physical_picker_choices.iter().map(|r| &r.0).collect::<Vec<_>>(),
                    self.metrics.reopen_scale, self.metrics.reopen_theme,
                    egui::Popup::is_any_open(ctx), ctx.input(|i| i.pointer.hover_pos())
                ));
            let (r, clip) = (*r, *clip);
            if !smoke_control_visible(r, clip, ctx.pixels_per_point()) {
                let key = egui::Id::new(("literal-popup-wheel", self.smoke_stage));
                let pulses = ctx.data(|d| d.get_temp::<usize>(key)).unwrap_or(0);
                assert!(pulses < 20);
                self.pending_events.extend([
                    egui::Event::PointerMoved(egui::pos2(r.center().x, clip.center().y)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., clip.center().y - r.center().y),
                        modifiers: Default::default(),
                    },
                ]);
                ctx.data_mut(|d| d.insert_temp(key, pulses + 1));
                self.started = Instant::now();
                ctx.request_repaint();
                return false;
            }
            return self.literal_stable_click(ctx, r, phase);
        }
        if matches!(target, Target::Scale | Target::Theme) {
            let r = rect.expect("actual header selector");
            assert!(smoke_control_visible(
                r,
                ctx.content_rect(),
                ctx.pixels_per_point()
            ));
            return self.literal_stable_click(ctx, r, phase);
        }
        self.click_visible_smoke_control(ctx, rect)
    }
    fn literal_warm(&mut self) -> Warm {
        let old = full(&self.progress);
        let portable = serde_json::to_value(&self.progress).unwrap();
        self.save();
        assert!(self.storage_error.is_none());
        let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
        let raw_value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let mut projected = raw_value.clone();
        let receipts = projected
            .as_object_mut()
            .unwrap()
            .remove("migration_receipts")
            .unwrap();
        assert_eq!(projected, portable);
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        assert_eq!(serde_json::to_value(&loaded).unwrap(), portable);
        assert_eq!(
            serde_json::to_value(&loaded.migration_receipts).unwrap(),
            receipts
        );
        assert!(
            loaded
                .migration_receipts
                .as_ref()
                .unwrap()
                .shell
                .contains(&15)
        );
        assert_eq!(std::fs::read(&self.path).unwrap(), raw);
        assert_eq!(full(&self.progress), old);
        Warm {
            full: old,
            raw,
            raw_value,
            loaded: full(&loaded),
        }
    }
    pub(in crate::ui) fn literal_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let root_key = egui::Id::new("literal-native-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(root_key)) == Some(root) {
            return (false, None);
        }
        ctx.data_mut(|d| d.insert_temp(root_key, root));
        assert_eq!(stage, self.smoke_stage);
        assert!(self.literal_capture_active() && self._smoke_profile.is_some());
        assert_eq!(shell::EVALUATOR_REVISION, 15);
        assert_eq!(
            (
                registration::before_scenarios(&self.scenarios).len(),
                registration::before_typed_units(&self.units).len()
            ),
            (197, 52)
        );
        assert_eq!((self.scenarios.len(), self.units.len()), (199, 53));
        let Action { task, step } = &plan()[stage - BASE];
        let task = *task;
        let s = &authored()[task];
        let id = IDS[task];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        assert_eq!(
            serde_json::to_value(&self.scenarios[i]).unwrap(),
            serde_json::to_value(s).unwrap()
        );
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        let pending = self.pending_events.len();
        let snap = egui::Id::new(("literal-snapshot", task));
        let warm = egui::Id::new(("literal-warm", task));
        let mut shot = None;
        match step {
            Step::Setup => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-literal-normalization".into(),
                    search: s.title.clone(),
                    ..Default::default()
                };
                if !self.literal_window(ctx, true) {
                    return (false, None);
                }
            }
            Step::Start => {
                if !self.script_lesson_start_card(ctx, id) {
                    return (false, None);
                }
                assert_eq!(self.selected, Some(i));
            }
            Step::Click(target) => {
                let rect = match target {
                    Target::Example => self.metrics.example_action,
                    Target::Assignment => self
                        .metrics
                        .assignment_action
                        .or_else(|| self.metrics.lesson_tabs.get(2).copied()),
                    Target::Run => self.metrics.check,
                    Target::Stop => self.metrics.stop,
                    Target::Back => self.metrics.back,
                    Target::Reset => self.metrics.reset,
                    Target::Cancel => self.metrics.reset_cancel,
                    Target::Confirm => self.metrics.confirm_reset,
                    Target::Scale => self.metrics.reopen_scale,
                    Target::Theme => self.metrics.reopen_theme,
                    Target::ScaleItem(want) => self
                        .metrics
                        .reopen_scale_items
                        .iter()
                        .find(|(n, _)| n == want)
                        .map(|(_, r)| *r),
                    Target::ThemeItem(want) => self
                        .metrics
                        .reopen_theme_items
                        .iter()
                        .find(|(n, _)| n.as_str() == *want)
                        .map(|(_, r)| *r),
                    Target::Challenge => self.metrics.mode_challenge,
                    Target::Learn => self.metrics.mode_learn,
                };
                if !self.literal_click(ctx, *target, rect) {
                    return (false, None);
                }
            }
            Step::Focus => {
                if !self.case_lesson_focus(ctx) {
                    return (false, None);
                }
            }
            Step::Type(source) => self.case_lesson_type(source),
            Step::FreshPractice => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.example_source, s.teaching.as_ref().unwrap().example);
                assert!(l.example_result.is_none());
                assert_eq!(a.state, s.initial);
                assert!(!progress::current_solved(a, s));
            }
            Step::Practice(expected) => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, s.initial);
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert!(expected.accepts(l.example_result.as_ref().unwrap()));
                assert!(!progress::current_solved(a, s));
            }
            Step::FreshAssignment => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                assert_eq!(self.progress.attempts[id].state, s.initial);
                assert!(!progress::current_solved(&self.progress.attempts[id], s));
            }
            Step::Assignment {
                source,
                pass,
                named,
            } => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                self.case_lesson_assignment(i, Some(source), *pass);
                let engine::State::Script { run: Some(r), .. } = &self.progress.attempts[id].state
                else {
                    panic!()
                };
                assert!(r.results.iter().all(|r| r.error.is_none()));
                let engine::Goal::Script { cases } = &s.goal else {
                    panic!()
                };
                assert_eq!(r.results.len(), cases.len());
                for (c, actual) in cases.iter().zip(&r.results) {
                    if let Some(expected) = options_witness_outcome(source, c) {
                        assert_eq!(
                            actual, &expected,
                            "Current graded full native witness/{}",
                            c.name
                        );
                    }
                }
                for (name, pass) in named {
                    let n = cases.iter().position(|c| &c.name == name).unwrap();
                    assert_eq!(cases[n].accepts(&r.results[n]), *pass);
                }
            }
            Step::Invalidate => {
                let a = &self.progress.attempts[id];
                assert!(matches!(a.state, engine::State::Script { run: None, .. }));
                assert!(!progress::current_solved(a, s));
                assert!(!self.metrics.completion);
            }
            Step::Stopped => {
                assert!(self.script_job.is_none());
                assert!(
                    matches!(&self.progress.attempts[id].state,engine::State::Script{source,run:None} if source==LOOP)
                );
                assert!(!progress::current_solved(&self.progress.attempts[id], s));
            }
            Step::Shot(label) => {
                let rect = if self.script_view(i) == teaching::View::Introduction {
                    self.metrics.example_action
                } else if self.script_view(i) == teaching::View::Example {
                    self.metrics.output.or(self.metrics.check)
                } else {
                    self.metrics.script_feedback.or(self.metrics.check)
                };
                if let Some(rect) = rect
                    && !self.reveal_smoke_control(ctx, rect)
                {
                    return (false, None);
                }
                shot = Some(format!("{id}-{label}"));
            }
            Step::Goal(last, label) => {
                let visible = self
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let Some((rows, clip)) = goal_rows(ctx, &s.objective, visible) else {
                    let key = egui::Id::new(("literal-goal-seek", stage));
                    let n = ctx.data(|d| d.get_temp::<usize>(key)).unwrap_or(0);
                    if matches!(n, 0 | 1 | 19) {
                        eprintln!(
                            "literal goal seek stage={stage} pulse={n} scale={} view={:?} visible={visible:?} offset={:?} content={:?} editor={:?} pointer/deltas={:?}",
                            self.scale,
                            self.script_view(i),
                            self.metrics.main_scroll_offset,
                            self.metrics.main_content_size,
                            self.metrics.editor_viewport,
                            ctx.input(|i| (
                                i.pointer.hover_pos(),
                                i.raw_scroll_delta,
                                i.smooth_scroll_delta
                            ))
                        );
                    }
                    assert!(
                        n < 20,
                        "Task galley must be reachable through bounded real wheel"
                    );
                    // The existing Label is laid out even when egui culls its paint.
                    // Seek its actual endpoint, rather than scanning past a long task.
                    let goal = self
                        .metrics
                        .script_goal
                        .expect("Assignment task is laid out");
                    let font = egui::TextStyle::Body.resolve(&ctx.style());
                    let row_height = ctx.fonts_mut(|f| f.row_height(&font)).min(goal.height());
                    let strip = egui::Rect::from_min_max(
                        egui::pos2(
                            goal.left(),
                            if *last {
                                goal.bottom() - row_height
                            } else {
                                goal.top()
                            },
                        ),
                        egui::pos2(
                            goal.right(),
                            if *last {
                                goal.bottom()
                            } else {
                                goal.top() + row_height
                            },
                        ),
                    );
                    // No forced offset or paint: only the ordinary bounded wheel helper.
                    let _ = self.reveal_smoke_control(ctx, strip);
                    ctx.data_mut(|d| d.insert_temp(key, n + 1));
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                };
                let row = if *last {
                    *rows.last().unwrap()
                } else {
                    rows[0]
                };
                if !smoke_control_visible(row, clip, ctx.pixels_per_point()) {
                    if !self.reveal_smoke_control(ctx, row) {
                        return (false, None);
                    }
                    ctx.request_repaint();
                    return (false, None);
                }
                shot = Some(format!("{id}-{label}"));
            }
            Step::Geometry => {
                let (wide, started) = ctx
                    .data(|d| {
                        d.get_temp::<(bool, Instant)>(egui::Id::new("literal-requested-window"))
                    })
                    .unwrap();
                let baseline = ctx
                    .data(|d| d.get_temp::<f32>(egui::Id::new("literal-windowed-width")))
                    .expect("Initial observed windowed physical width");
                let fullscreen = ctx.input(|i| i.viewport().fullscreen);
                let physical = ctx.content_rect().size() * ctx.pixels_per_point();
                assert!(
                    started.elapsed().as_secs() < 15,
                    "Native WM did not acknowledge distinct geometry: fullscreen{fullscreen:?} wide{wide} physical{physical:?} baseline{baseline}"
                );
                if !window_acknowledged(fullscreen, physical, baseline, wide) {
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                }
                if !self.case_lesson_geometry(ctx) {
                    return (false, None);
                }
                eprintln!(
                    "literal geometry acknowledged fullscreen{fullscreen:?} wide{wide} physical{physical:?} baseline{baseline}"
                );
            }
            Step::Snapshot => {
                let value = full(&self.progress);
                ctx.data_mut(|d| d.insert_temp(snap, value));
            }
            Step::Window(wide) => {
                if !self.literal_window(ctx, *wide) {
                    return (false, None);
                }
            }
            Step::Mode(challenge) => {
                let mut expected = ctx.data(|d| d.get_temp::<serde_json::Value>(snap)).unwrap();
                expected["attempts"][id]["challenge"] = (*challenge).into();
                assert_eq!(full(&self.progress), expected);
                assert_eq!(self.progress.attempts[id].challenge, *challenge);
                if *challenge {
                    assert_eq!(self.script_view(i), teaching::View::Assignment);
                    assert!(!self.metrics.help_visible);
                }
            }
            Step::Warm => {
                let value = self.literal_warm();
                ctx.data_mut(|d| d.insert_temp(warm, value));
            }
            Step::Continued => {
                let old = ctx.data(|d| d.get_temp::<Warm>(warm)).unwrap();
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&old.raw).unwrap(),
                    old.raw_value
                );
                let mut expected = old.full;
                let clock = expected["clock"].as_u64().unwrap().checked_add(1).unwrap();
                expected["clock"] = clock.into();
                expected["attempts"][id]["visited"] = clock.into();
                assert_eq!(full(&self.progress), expected);
                self.save();
                let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let mut expected_raw = old.raw_value;
                expected_raw["clock"] = clock.into();
                expected_raw["attempts"][id]["visited"] = clock.into();
                assert_eq!(
                    serde_json::from_slice::<serde_json::Value>(&raw).unwrap(),
                    expected_raw
                );
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                let mut expected_loaded = old.loaded;
                expected_loaded["clock"] = clock.into();
                expected_loaded["attempts"][id]["visited"] = clock.into();
                assert_eq!(full(&loaded), expected_loaded);
                assert_eq!(std::fs::read(&self.path).unwrap(), raw);
            }
            Step::Cancelled => {
                let before = ctx.data(|d| d.get_temp::<serde_json::Value>(snap)).unwrap();
                assert!(!self.reset_pending);
                assert!(ere_evidence::cancel_preserved(
                    &before,
                    &full(&self.progress)
                ));
            }
            Step::ResetDone => {
                let before = ctx.data(|d| d.get_temp::<serde_json::Value>(snap)).unwrap();
                assert!(ere_evidence::reset_preserved(
                    &before,
                    &full(&self.progress),
                    s
                ));
                assert!(!progress::current_solved(&self.progress.attempts[id], s));
            }
            Step::Finish => {
                assert!(self.selected.is_none());
                assert!(IDS.iter().all(|id| {
                    let s = self.scenarios.iter().find(|s| s.id == *id).unwrap();
                    progress::current_solved(&self.progress.attempts[*id], s)
                }));
                let captured = ctx
                    .data(|d| d.get_temp::<Vec<String>>(egui::Id::new("literal-capture-phases")))
                    .unwrap();
                assert_eq!(
                    captured,
                    inventory().1,
                    "Every declared capture phase ran once in order"
                );
                println!(
                    "PASS focused normal199 literal2 actual editor/Run/Stop/reset/Continue route; representative appearance only"
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return (false, None);
            }
        }
        if let Some(name) = &shot {
            let actual = ctx.content_rect();
            let ppp = ctx.pixels_per_point();
            let zoom = ctx.zoom_factor();
            println!(
                "literal capture {name}: actual_content={actual:?} PPP={ppp} zoom={zoom} theme={}",
                self.theme_id
            );
            let key = egui::Id::new("literal-capture-phases");
            ctx.data_mut(|d| {
                let mut names = d.get_temp::<Vec<String>>(key).unwrap_or_default();
                assert!(!names.contains(name));
                assert!(names.len() < 64);
                names.push(name.clone());
                d.insert_temp(key, names);
            });
        }
        assert!(
            self.pending_events.len().saturating_sub(pending) <= 4,
            "Small bounded event batch"
        );
        (true, shot)
    }
}
