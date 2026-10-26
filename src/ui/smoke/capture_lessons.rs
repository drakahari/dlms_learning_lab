//! Four fixed teaching fixtures, exercised through the native editor and Run controls.
use super::*;
impl LabApp {
    pub(super) fn capture_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        if let Some(kind) =
            ScriptLessonFlow::from_stage(std::env::var("DLMS_SMOKE_STAGE").ok().as_deref())
        {
            return self.case_lessons_native(ctx, stage, kind);
        }
        const IDS: [&str; 4] = [
            "shell-capture-preview",
            "shell-capture-evidence",
            "shell-capture-decision",
            "shell-capture-handover",
        ];
        let ordinal = (stage - 7200) / 100;
        let phase = (stage - 7200) % 100;
        assert!(
            IDS.iter()
                .all(|id| self.scenarios.iter().any(|s| &s.id == id)),
            "Authored capture lessons must be bundled in the normal app"
        );
        assert!(self.units.iter().any(|u| u.id == "shell-command-capture"));
        let id = IDS[ordinal];
        let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let scenario = self.scenarios[i].clone();
        let lesson = scenario.teaching.as_ref().unwrap();
        let rows: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-capture-content-review.json"
        ))
        .unwrap();
        let review = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        let ctrl = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        };
        let mut shot = None;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("capture-lesson {id} phase {phase}");
        }
        match phase {
            0 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.launch(i);
                self.set_script_view(i, teaching::View::Introduction);
                self.progress.favorites.insert(id.into());
            }
            1 => {
                assert_eq!(self.script_view(i), teaching::View::Introduction);
                shot = Some(format!("{id}-introduction"));
            }
            2 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            3 => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            4 | 8 | 14 | 18 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            5 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert!(
                    lesson
                        .fixture
                        .accepts(learning.example_result.as_ref().unwrap())
                );
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                shot = Some(format!("{id}-ungraded-example"));
            }
            6 | 12 | 16 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.editor_viewport) {
                    return (false, None);
                }
            }
            7 => {
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text(review["try_change"]["source"].as_str().unwrap().into()),
                ];
            }
            9 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                let expected: shell::Case =
                    serde_json::from_value(review["try_change"]["expected"].clone()).unwrap();
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert!(expected.accepts(learning.example_result.as_ref().unwrap()));
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            10 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            11 => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                shot = Some(format!("{id}-full-assignment"));
            }
            13 => {
                let partial = review["wrong"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|w| w["name"] == review["partial"])
                    .unwrap()["source"]
                    .as_str()
                    .unwrap();
                self.pending_events = vec![ctrl(egui::Key::A), egui::Event::Text(partial.into())];
            }
            15 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                let engine::State::Script {
                    run: Some(record), ..
                } = &self.progress.attempts[id].state
                else {
                    panic!("partial run missing")
                };
                let engine::Goal::Script { cases } = &scenario.goal else {
                    panic!()
                };
                let passes: Vec<_> = cases
                    .iter()
                    .zip(&record.results)
                    .map(|(c, r)| c.accepts(r))
                    .collect();
                assert!(
                    passes.contains(&true) && passes.contains(&false),
                    "{id} partial {passes:?}"
                );
                assert!(!self.metrics.completion);
                shot = Some(format!("{id}-partial-feedback"));
            }
            17 => {
                self.pending_events = vec![
                    ctrl(egui::Key::A),
                    egui::Event::Text(scenario.solution[0].clone()),
                ];
            }
            19 => {
                if self.script_job.is_some() {
                    return (false, None);
                }
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                assert_eq!(self.progress.attempts[id].step, 0);
                shot = Some(format!("{id}-complete"));
            }
            20 => {
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
                assert!(self.progress.favorites.contains(id));
                self.set_script_view(i, teaching::View::Example);
            }
            21 => {
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    learning.example_source,
                    review["try_change"]["source"].as_str().unwrap()
                );
                assert!(learning.example_result.is_some());
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                self.set_script_view(i, teaching::View::Assignment);
                self.progress.attempts.get_mut(id).unwrap().challenge = true;
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
            }
            22 => {
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                shot = Some(format!("{id}-challenge-light150"));
            }
            23 | 26 => {
                if !self.reveal_smoke_control(ctx, self.metrics.check.unwrap()) {
                    return (false, None);
                }
            }
            24 => {
                shot = Some(format!("{id}-challenge-light150-run"));
            }
            25 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            27 => {
                shot = Some(format!("{id}-challenge-dark200-run"));
            }
            28 => {
                let a = self.progress.attempts.get_mut(id).unwrap();
                let engine::State::Script { source, .. } = &mut a.state else {
                    panic!()
                };
                source.push_str("\nprintf broken");
                assert!(!progress::current_solved(a, &scenario));
                self.dirty = true;
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                self.scale = 1.;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                self.lesson_scroll_reset = true;
            }
            29 => {
                let others: std::collections::BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(other, _)| other.as_str() != id)
                    .map(|(id, a)| (id.clone(), serde_json::to_vec(a).unwrap()))
                    .collect();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("capture-unit-other-attempts"), others)
                });
            }
            30 => {
                if !self.click_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            31 => {
                if !self.click_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            32 => {
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                let others = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::BTreeMap<String, Vec<u8>>>(egui::Id::new(
                            "capture-unit-other-attempts",
                        ))
                    })
                    .unwrap();
                for (id, bytes) in others {
                    assert_eq!(
                        serde_json::to_vec(&self.progress.attempts[&id]).unwrap(),
                        bytes
                    );
                }
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
            }
            33 => {
                self.selected = None;
                self.filter.track = "Bash/Shell".into();
                self.filter.unit = "shell-command-capture".into();
            }
            34 => {
                shot = Some(format!("{id}-library-resume"));
            }
            35 => {
                if ordinal == 3 {
                    println!(
                        "PASS native four command-capture lessons: prerequisite teaching, actual separate example Run/change, full assignment, partial feedback, keyboard repair/Run, Step0 success, exact resume and example drafts, current invalidation, selected Reset, Challenge light150/dark200 and library. Synthetic authored inputs and injected keyboard events; no novice/physical-device/screen-reader/other-platform acceptance."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    self.smoke_stage = 7200 + (ordinal + 1) * 100 - 1;
                }
            }
            _ => panic!("Unexpected capture lesson phase {phase}"),
        }
        (true, shot)
    }
}

// The existing four-capture flow remains above; the three-case flow uses the
// same native widgets, runner and screenshot infrastructure without model answers.
const CASE_LESSON_IDS: [&str; 3] = [
    "shell-case-interface",
    "shell-case-routing",
    "shell-case-publication",
];
const GETOPTS_LESSON_IDS: [&str; 3] = [
    "shell-options-report",
    "shell-options-jobs",
    "shell-options-publication",
];
#[derive(serde::Deserialize)]
struct NativeScriptReview {
    id: String,
    alternates: Vec<String>,
    wrong: Vec<NativeScriptWrong>,
    partial: String,
    try_change: NativeScriptChange,
}
#[derive(serde::Deserialize)]
struct NativeScriptWrong {
    name: String,
    source: String,
}
#[derive(serde::Deserialize)]
struct NativeScriptChange {
    source: String,
    expected: shell::Case,
    repair: String,
}
impl ScriptLessonFlow {
    pub(in crate::ui) fn from_stage(value: Option<&str>) -> Option<Self> {
        match value {
            Some("case-lessons") => Some(Self::Case),
            Some("getopts-lessons") => Some(Self::Getopts),
            _ => None,
        }
    }
    fn ids(self) -> &'static [&'static str; 3] {
        match self {
            Self::Case => &CASE_LESSON_IDS,
            Self::Getopts => &GETOPTS_LESSON_IDS,
        }
    }
    fn unit(self) -> &'static str {
        match self {
            Self::Case => "shell-case-dispatch",
            Self::Getopts => "shell-option-interfaces",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Case => "case",
            Self::Getopts => "getopts",
        }
    }
    fn reviews(self) -> Vec<NativeScriptReview> {
        // Fixed reviewed author data only; never a learner-provided path/source.
        serde_json::from_str(match self {
            Self::Case => include_str!("../../../tests/fixtures/shell-case-content-review.json"),
            Self::Getopts => {
                include_str!("../../../tests/fixtures/shell-getopts-content-review.json")
            }
        })
        .unwrap()
    }
    fn alternate(self, review: &NativeScriptReview) -> &str {
        match self {
            Self::Case => review
                .alternates
                .iter()
                .find(|s| !s.contains("case "))
                .expect("equivalent if/loop solution"),
            Self::Getopts => review
                .alternates
                .first()
                .expect("reviewed manual parser alternative"),
        }
    }
}
fn case_lesson_editor_band(viewport: egui::Rect, height: f32) -> egui::Rect {
    egui::Rect::from_center_size(
        viewport.center(),
        egui::vec2(viewport.width(), height.min(viewport.height())),
    )
}
impl LabApp {
    pub(super) fn case_lesson_focus(&mut self, ctx: &egui::Context) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        let clicked = if phase >= 2 {
            self.click_smoke_control(ctx, None)
        } else {
            let viewport = self
                .metrics
                .editor_viewport
                .expect("actual script editor viewport");
            let band = case_lesson_editor_band(viewport, ctx.style().spacing.interact_size.y);
            assert!(viewport.contains_rect(band));
            if !self.reveal_smoke_control(ctx, band) {
                return false;
            }
            self.click_smoke_control(ctx, Some(band))
        };
        if clicked {
            assert!(
                self.metrics
                    .focus
                    .is_some_and(|id| ctx.memory(|m| m.has_focus(id))),
                "native band must focus the script editor"
            );
        }
        clicked
    }
    pub(super) fn case_lesson_type(&mut self, source: &str) {
        assert!(source.len() <= shell::MAX_SOURCE);
        self.pending_events = [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            })
            .collect();
        self.pending_events.push(egui::Event::Text(source.into()));
    }
    pub(super) fn case_lesson_wait(&self, ctx: &egui::Context) -> bool {
        if self.script_job.is_none() {
            return true;
        }
        let started = ctx.data_mut(|d| {
            let key = egui::Id::new(("case-lesson-native-wait", self.smoke_stage));
            let started = d.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
            d.insert_temp(key, started);
            started
        });
        assert!(
            started.elapsed().as_secs() < 15,
            "bounded lesson script did not finish"
        );
        ctx.request_repaint();
        false
    }
    pub(super) fn case_lesson_geometry(&self, ctx: &egui::Context) -> bool {
        let current = ctx.content_rect();
        let zoom = ctx.zoom_factor();
        let stable = ctx.data_mut(|d| {
            let key = egui::Id::new(("case-lesson-geometry", self.smoke_stage));
            let (previous, count, started) = d
                .get_temp::<(Option<egui::Rect>, usize, Instant)>(key)
                .unwrap_or((None, 0, Instant::now()));
            assert!(
                started.elapsed().as_secs() < 15,
                "native lesson geometry did not settle"
            );
            let count = if previous == Some(current) && (zoom - self.scale).abs() < 0.01 {
                count + 1
            } else {
                0
            };
            d.insert_temp(key, (Some(current), count, started));
            count >= 2
        });
        if !stable {
            ctx.request_repaint();
        }
        stable
    }
    pub(super) fn case_lesson_assignment(&self, i: usize, expected: Option<&str>, pass: bool) {
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        let engine::State::Script {
            source,
            run: Some(record),
        } = &a.state
        else {
            panic!("native script result missing")
        };
        assert_eq!(record.source, *source);
        assert_eq!(record.engine_revision, shell::EVALUATOR_REVISION);
        if let Some(expected) = expected {
            assert_eq!(source, expected);
        }
        assert_eq!(progress::current_solved(a, s), pass);
        assert_eq!(self.metrics.completion, pass);
        if pass {
            assert_eq!(a.step, 0);
        }
    }
    pub(super) fn script_lesson_start_card(&mut self, ctx: &egui::Context, id: &str) -> bool {
        let expected = self.scenarios.iter().position(|s| s.id == id).unwrap();
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        if ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0) >= 2 {
            let done = self.click_visible_smoke_control(ctx, None);
            if done {
                assert_eq!(
                    self.selected,
                    Some(expected),
                    "real Start/Continue selected requested normal lab"
                );
            }
            return done;
        }
        assert!(self.selected.is_none());
        let matches: Vec<_> = self
            .scenarios
            .iter()
            .filter(|s| {
                catalog::matches(
                    s,
                    &self.filter,
                    self.progress.attempts.contains_key(&s.id),
                    self.progress
                        .attempts
                        .get(&s.id)
                        .is_some_and(|a| progress::current_solved(a, s)),
                    self.progress.favorites.contains(&s.id),
                )
            })
            .collect();
        // Search also includes goals: a later task may mention this title.
        // Follow the same normal catalogue order and select the requested card.
        assert!(!self.filter.recent && self.page == 0);
        assert!(matches.len() <= 12);
        let ordinal = matches
            .iter()
            .position(|s| s.id == id)
            .expect("requested normal lab is a search result");
        assert_eq!(
            self.metrics.card_actions.len(),
            matches.len(),
            "all normal search cards rendered before clicking"
        );
        self.click_visible_smoke_control(ctx, self.metrics.card_actions.get(ordinal).map(|r| r.0))
    }
    pub(in crate::ui) fn case_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
        kind: ScriptLessonFlow,
    ) -> (bool, Option<String>) {
        const {
            assert!(shell::EVALUATOR_REVISION >= 8);
        }
        if kind == ScriptLessonFlow::Getopts {
            const {
                assert!(shell::EVALUATOR_REVISION >= 9);
            }
        }
        assert!((7200..7500).contains(&stage));
        let ordinal = (stage - 7200) / 100;
        let phase = (stage - 7200) % 100;
        let ids = kind.ids();
        let id = ids[ordinal];
        assert_eq!(self.scenarios.len(), 140, "normal registered catalogue");
        assert!(
            GETOPTS_LESSON_IDS
                .iter()
                .all(|id| self.scenarios.iter().any(|s| s.id == *id)),
            "all three normal getopts IDs required"
        );
        assert!(
            self.units.iter().any(|u| u.id == kind.unit()
                && u.labs.iter().map(String::as_str).eq(ids.iter().copied()))
        );
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("normal bundled script lesson");
        let scenario = self.scenarios[i].clone();
        let lesson = scenario.teaching.as_ref().unwrap();
        let rows = kind.reviews();
        let review = rows.iter().find(|r| r.id == id).unwrap();
        assert_eq!(review.try_change.repair, lesson.example);
        let reference = &scenario.solution[0];
        let alternate = kind.alternate(review);
        let partial = &review
            .wrong
            .iter()
            .find(|w| w.name == review.partial)
            .unwrap()
            .source;
        let wrong = &review
            .wrong
            .iter()
            .find(|w| w.name != review.partial)
            .unwrap()
            .source;
        let mut shot = None;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!("{}-lesson {id} phase {phase}", kind.label());
        }
        match phase {
            0 => {
                assert!(self.smoke && self._smoke_profile.is_some());
                if kind == ScriptLessonFlow::Getopts {
                    let key = egui::Id::new(("getopts-normal-library", stage));
                    let prepared = ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false);
                    if !prepared {
                        self.selected = None;
                        self.filter = Filter {
                            track: "Bash/Shell".into(),
                            unit: kind.unit().into(),
                            search: scenario.title.clone(),
                            ..Default::default()
                        };
                        self.scale = 1.;
                        self.theme_id = "dlms-dark".into();
                        theme::apply(ctx, &self.theme_id);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                            1200., 850.,
                        )));
                        ctx.data_mut(|d| d.insert_temp(key, true));
                        return (false, None);
                    }
                    if !self.case_lesson_geometry(ctx) {
                        return (false, None);
                    }
                    if !self.script_lesson_start_card(ctx, id) {
                        return (false, None);
                    }
                    self.set_script_view(i, teaching::View::Introduction);
                    self.progress.favorites.insert(id.into());
                    return (true, None);
                }
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
                self.launch(i);
                self.set_script_view(i, teaching::View::Introduction);
                self.progress.favorites.insert(id.into());
            }
            1 => {
                if !self.case_lesson_geometry(ctx) {
                    return (false, None);
                }
                assert_eq!(self.script_view(i), teaching::View::Introduction);
                shot = Some(format!("{id}-introduction"));
            }
            2 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            3 => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(learning.example_source, lesson.example);
                assert!(learning.example_result.is_none());
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                shot = Some(format!("{id}-example-ready-without-execution"));
            }
            4 | 9 | 13 | 19 | 23 | 27 | 32 | 37 | 45 | 52 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            5 | 10 | 14 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                let expected = if phase == 10 {
                    review.try_change.expected.clone()
                } else {
                    lesson.fixture.clone()
                };
                assert!(expected.accepts(learning.example_result.as_ref().unwrap()));
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                shot = Some(format!(
                    "{id}-example-{}",
                    if phase == 5 {
                        "run"
                    } else if phase == 10 {
                        "experiment"
                    } else {
                        "repaired"
                    }
                ));
            }
            6 | 11 | 17 | 21 | 25 | 29 | 35 | 42 | 49 | 55 => {
                if !self.case_lesson_focus(ctx) {
                    return (false, None);
                }
            }
            7 => self.case_lesson_type(&review.try_change.source),
            8 => {
                let learning = self.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(learning.example_source, review.try_change.source);
                assert!(learning.example_result.is_none());
            }
            12 => self.case_lesson_type(&lesson.example),
            15 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            16 => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                shot = Some(format!("{id}-full-assignment"));
            }
            18 => self.case_lesson_type(wrong),
            20 | 24 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                self.case_lesson_assignment(
                    i,
                    Some(if phase == 20 { wrong } else { partial }),
                    false,
                );
                let engine::Goal::Script { cases } = &scenario.goal else {
                    panic!()
                };
                let engine::State::Script {
                    run: Some(record), ..
                } = &self.progress.attempts[id].state
                else {
                    panic!()
                };
                let passes: Vec<_> = cases
                    .iter()
                    .zip(&record.results)
                    .map(|(c, r)| c.accepts(r))
                    .collect();
                assert!(passes.contains(&false));
                if phase == 24 {
                    assert!(
                        passes.contains(&true),
                        "partial must show real successful work"
                    );
                }
                if !self.reveal_smoke_control(
                    ctx,
                    self.metrics
                        .script_feedback
                        .expect("visible real feedback summary"),
                ) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-{}-feedback",
                    if phase == 20 { "wrong" } else { "partial" }
                ));
            }
            22 => self.case_lesson_type(partial),
            26 => self.case_lesson_type(reference),
            28 | 33 | 38 | 46 | 53 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                let expected = match phase {
                    28 => reference.clone(),
                    33 => format!("{reference}\n# Current edit checked again\n"),
                    38 => alternate.into(),
                    46 => format!("{alternate}\n# Typed at150percent\n"),
                    _ => format!("{reference}\n# Typed at200percent\n"),
                };
                self.case_lesson_assignment(i, Some(&expected), true);
                if !self.reveal_smoke_control(
                    ctx,
                    self.metrics
                        .script_feedback
                        .expect("current completion summary"),
                ) {
                    return (false, None);
                }
                shot = Some(format!(
                    "{id}-{}",
                    match phase {
                        28 => "reference-complete",
                        33 => "current-edit-rechecked",
                        38 =>
                            if kind == ScriptLessonFlow::Case {
                                "if-alternate-complete"
                            } else {
                                "manual-alternate-complete"
                            },
                        46 => "challenge-light150-typed-and-run",
                        _ => "challenge-dark200-typed-and-run",
                    }
                ));
            }
            30 => self.case_lesson_type(&format!("{reference}\n# Current edit checked again\n")),
            31 | 44 | 51 => {
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            34 | 54 => {
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                assert_eq!(
                    self.progress.attempts[id]
                        .workspace
                        .as_ref()
                        .unwrap()
                        .script_learning
                        .as_ref()
                        .unwrap()
                        .example_source,
                    lesson.example
                );
                assert!(self.progress.favorites.contains(id));
            }
            36 => self.case_lesson_type(alternate),
            39 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            40 => {
                assert!(self.progress.attempts[id].challenge);
                assert!(self.metrics.lesson_tabs.is_empty());
                assert!(!self.metrics.help_visible);
                self.scale = 1.5;
                self.theme_id = "dlms-light".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(960., 840.)));
                self.lesson_scroll_reset = true;
            }
            41 | 48 => {
                if !self.case_lesson_geometry(ctx) {
                    return (false, None);
                }
            }
            43 => self.case_lesson_type(&format!("{alternate}\n# Typed at150percent\n")),
            47 => {
                self.scale = 2.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                self.lesson_scroll_reset = true;
            }
            50 => self.case_lesson_type(&format!("{reference}\n# Typed at200percent\n")),
            56 => self.case_lesson_type(&format!("{reference}\nprintf 'unfinished assignment")),
            57 => {
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_vec(&self.progress.attempts).unwrap(), before);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                shot = Some(format!("{id}-unfinished-draft-resumed"));
                if kind == ScriptLessonFlow::Getopts {
                    self.smoke_stage = 7200 + ordinal * 100 + 66 - 1;
                }
            }
            58 => {
                let others: std::collections::BTreeMap<_, _> = self
                    .progress
                    .attempts
                    .iter()
                    .filter(|(other, _)| other.as_str() != id)
                    .map(|(id, a)| (id.clone(), serde_json::to_vec(a).unwrap()))
                    .collect();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("case-unit-other-attempts"), others));
                if kind == ScriptLessonFlow::Getopts {
                    let selected = serde_json::to_vec(&self.progress.attempts[id]).unwrap();
                    ctx.data_mut(|d| {
                        d.insert_temp(egui::Id::new("getopts-before-reset-selected"), selected)
                    });
                }
            }
            59 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            60 => {
                if kind == ScriptLessonFlow::Getopts {
                    if !self.click_visible_smoke_control(ctx, self.metrics.reset_cancel) {
                        return (false, None);
                    }
                    self.smoke_stage = 7200 + ordinal * 100 + 77 - 1;
                } else if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            61 => {
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                let others = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::BTreeMap<String, Vec<u8>>>(egui::Id::new(
                            "case-unit-other-attempts",
                        ))
                    })
                    .unwrap();
                for (other, bytes) in others {
                    assert_eq!(
                        serde_json::to_vec(&self.progress.attempts[&other]).unwrap(),
                        bytes
                    );
                }
            }
            62 => {
                self.save();
                self.progress = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(self.progress.favorites.contains(id));
            }
            63 => {
                if kind == ScriptLessonFlow::Getopts {
                    if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                        return (false, None);
                    }
                    assert!(self.selected.is_none());
                    self.filter = Filter {
                        track: "Bash/Shell".into(),
                        unit: kind.unit().into(),
                        search: scenario.title.clone(),
                        ..Default::default()
                    };
                } else {
                    self.selected = None;
                    self.filter.track = "Bash/Shell".into();
                    self.filter.unit = kind.unit().into();
                }
            }
            64 => {
                shot = Some(format!("{id}-library-after-selected-reset"));
            }
            65 => {
                if ordinal + 1 == ids.len() {
                    println!(
                        "PASS native three {} lessons: introduction, separate ready-only example/Run/experiment/repair, full assignment, wrong and partial feedback, reference and alternate, current edit invalidation, actual Challenge Light150/Dark200 typing+Run, exact save/resume and selected reset. App-only author-reference inputs; no novice/accessibility/other-platform acceptance.",
                        kind.label()
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None); // Close is asynchronous: hold final phase.
                } else {
                    self.smoke_stage = 7200 + (ordinal + 1) * 100 - 1;
                }
            }
            66 if kind == ScriptLessonFlow::Getopts => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            67 if kind == ScriptLessonFlow::Getopts => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(1).copied())
                {
                    return (false, None);
                }
            }
            68 if kind == ScriptLessonFlow::Getopts => {
                if !self.case_lesson_focus(ctx) {
                    return (false, None);
                }
            }
            69 if kind == ScriptLessonFlow::Getopts => {
                self.case_lesson_type(&format!("{}\nprintf 'unfinished practice", lesson.example))
            }
            70 if kind == ScriptLessonFlow::Getopts => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                let a = &self.progress.attempts[id];
                assert!(
                    matches!(&a.state,engine::State::Script{source,run:None}if source==&format!("{reference}\nprintf 'unfinished assignment"))
                );
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    l.example_source,
                    format!("{}\nprintf 'unfinished practice", lesson.example)
                );
                assert!(l.example_result.is_none());
                assert!(!progress::current_solved(a, &scenario));
                let before = serde_json::to_vec(&self.progress.attempts).unwrap();
                self.save();
                assert!(self.storage_error.is_none());
                let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let loaded = progress::decode(&raw, &self.scenarios).unwrap();
                assert_eq!(serde_json::to_vec(&loaded.attempts).unwrap(), before);
                self.progress = loaded;
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("getopts-before-library-resume"), before)
                });
                shot = Some(format!("{id}-separate-unfinished-drafts-saved"));
            }
            71 if kind == ScriptLessonFlow::Getopts => {
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            72 if kind == ScriptLessonFlow::Getopts => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: kind.unit().into(),
                    search: scenario.title.clone(),
                    ..Default::default()
                };
            }
            73 if kind == ScriptLessonFlow::Getopts => {
                if !self.script_lesson_start_card(ctx, id) {
                    return (false, None);
                }
            }
            74 if kind == ScriptLessonFlow::Getopts => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(self.script_view(i), teaching::View::Example);
                let before = ctx
                    .data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("getopts-before-library-resume")))
                    .unwrap();
                let mut expected: std::collections::BTreeMap<String, progress::Attempt> =
                    serde_json::from_slice(&before).unwrap();
                // A real Continue legitimately records a new visit; every other field stays exact.
                expected.get_mut(id).unwrap().visited = self.progress.attempts[id].visited;
                assert_eq!(
                    serde_json::to_vec(&self.progress.attempts).unwrap(),
                    serde_json::to_vec(&expected).unwrap()
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                shot = Some(format!("{id}-real-library-continue-separated-drafts"));
            }
            75 if kind == ScriptLessonFlow::Getopts => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(2).copied())
                {
                    return (false, None);
                }
            }
            76 if kind == ScriptLessonFlow::Getopts => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                self.smoke_stage = 7200 + ordinal * 100 + 58 - 1;
            }
            77 if kind == ScriptLessonFlow::Getopts => {
                assert!(!self.reset_pending);
                let before = ctx
                    .data(|d| d.get_temp::<Vec<u8>>(egui::Id::new("getopts-before-reset-selected")))
                    .unwrap();
                assert_eq!(
                    serde_json::to_vec(&self.progress.attempts[id]).unwrap(),
                    before
                );
            }
            78 if kind == ScriptLessonFlow::Getopts => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            79 if kind == ScriptLessonFlow::Getopts => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
                self.smoke_stage = 7200 + ordinal * 100 + 61 - 1;
            }
            _ => panic!("Unexpected case lesson phase {phase}"),
        }
        (true, shot)
    }
}

#[cfg(test)]
mod case_lesson_plan_tests {
    use super::*;
    #[test]
    fn three_case_lessons_are_normal_and_have_bounded_native_keyboard_plans() {
        let mut all = pack::bundled().unwrap();
        let mut units = course::bundled(&all).unwrap();
        if !all.iter().any(|s| s.id == GETOPTS_LESSON_IDS[0]) {
            #[derive(serde::Deserialize)]
            struct AuthoringPack {
                scenarios: Vec<Scenario>,
            }
            let extra: AuthoringPack =
                serde_json::from_str(include_str!("../../../scenarios/shell-getopts-v1.json"))
                    .unwrap();
            all.extend(extra.scenarios);
            units.push(
                serde_json::from_str(include_str!(
                    "../../../tests/fixtures/shell-getopts-unit-proposal.json"
                ))
                .unwrap(),
            );
        }
        assert_eq!(all.len(), 140);
        assert_eq!(units.len(), 35);
        let rows: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/shell-case-content-review.json"
        ))
        .unwrap();
        for (ordinal, id) in CASE_LESSON_IDS.iter().enumerate() {
            assert!(7200 + ordinal * 100 + 65 <= 7599);
            let s = all.iter().find(|s| s.id == *id).unwrap();
            let r = rows
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == *id)
                .unwrap();
            assert!(
                r["alternates"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| !a.as_str().unwrap().contains("case "))
            );
            assert!(s.solution[0].len() + 64 < shell::MAX_SOURCE);
            assert!(s.teaching.as_ref().unwrap().example.len() < shell::MAX_SOURCE);
        }
        for height in [1., 18., 180., 360.] {
            let viewport =
                egui::Rect::from_min_size(egui::pos2(20., 80.), egui::vec2(400., height));
            let band = case_lesson_editor_band(viewport, 35.);
            assert!(viewport.contains_rect(band));
            assert_eq!(band.center(), viewport.center());
        }
    }
}
