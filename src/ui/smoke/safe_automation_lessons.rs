//! Closed native-only normal-course acceptance. All script effects occur through
//! actual editor events and Run/Stop; no State/Record or solved result is installed.
use super::*;
const BASE: usize = 37000;
const STRIDE: usize = 200;
const LAST: usize = 104;
const IDS: [&str; 3] = [
    "shell-preserve-lines",
    "shell-pipeline-status",
    "shell-safe-handover",
];
const LOOP: &str = "while true; do :; done\n";
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    id: String,
    alternate: String,
    incorrect: String,
    misconception: String,
    partial: String,
    wrong: String,
    experiment: Experiment,
    repair: String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Experiment {
    source: String,
    expected: shell::Case,
}
fn plans() -> Vec<Plan> {
    let rows: Vec<Plan> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shell-safe-automation-native-review.json"
    ))
    .unwrap();
    assert_eq!(rows.len(), 3);
    for (row, id) in rows.iter().zip(IDS) {
        assert_eq!(row.id, id);
        assert!(!row.misconception.is_empty());
        assert_eq!(row.partial, row.incorrect);
        for s in [
            &row.alternate,
            &row.partial,
            &row.wrong,
            &row.experiment.source,
            &row.repair,
        ] {
            assert!(!s.is_empty() && s.len() <= shell::MAX_SOURCE);
        }
        assert!(row.experiment.expected.valid());
    }
    rows
}
fn resource_source() -> String {
    format!(
        "pad='{}'\nprintf '%s%s%s%s%s%s%s%s%s' \"$pad\" \"$pad\" \"$pad\" \"$pad\" \"$pad\" \"$pad\" \"$pad\" \"$pad\" \"$pad\"\n",
        "x".repeat(4096)
    )
}
fn full(p: &Progress) -> serde_json::Value {
    let mut value = serde_json::to_value(p).unwrap();
    value["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    value
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize, label: &str) {
    fn diff(a: &serde_json::Value, b: &serde_json::Value) -> Option<String> {
        if a == b {
            return None;
        }
        match (a, b) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                if a.len() != b.len() {
                    return Some("object key count".into());
                }
                for (k, v) in a {
                    let Some(w) = b.get(k) else {
                        return Some(format!("missing {:.60}", k));
                    };
                    if let Some(d) = diff(v, w) {
                        return Some(format!("{:.60}.{:.120}", k, d));
                    }
                }
                Some("object".into())
            }
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                if a.len() != b.len() {
                    return Some("array length".into());
                }
                a.iter()
                    .zip(b)
                    .enumerate()
                    .find_map(|(i, (a, b))| diff(a, b).map(|d| format!("[{i}].{d:.120}")))
            }
            _ => Some("scalar/type (values withheld)".into()),
        }
    }
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    let d = diff(&a, &b);
    assert!(d.is_none(), "{label}: {}", d.unwrap_or_default());
}
#[derive(Clone)]
struct Warm {
    raw: Vec<u8>,
    raw_value: serde_json::Value,
    full: serde_json::Value,
}
impl LabApp {
    pub(in crate::ui) fn safe_automation_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("safe-automation-lessons")).then_some(BASE)
    }
    fn safe_automation_warm(&mut self) -> Warm {
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
            .expect("Normal writer provenance");
        exact(
            &projected,
            &portable,
            "All portable progress fields after normal save",
        );
        let loaded = progress::load(&self.path, &self.scenarios).unwrap();
        exact(
            &serde_json::to_value(&loaded).unwrap(),
            &portable,
            "Normal load portable fields",
        );
        exact(
            &loaded.migration_receipts,
            &receipts,
            "Normal loaded receipts equal exact raw main field",
        );
        assert!(
            loaded
                .migration_receipts
                .as_ref()
                .unwrap()
                .shell
                .contains(&shell::EVALUATOR_REVISION)
        );
        assert!(
            std::fs::read(&self.path).unwrap() == raw,
            "Reading never rewrites raw bytes"
        );
        let snapshot = full(&loaded);
        self.progress = loaded;
        Warm {
            raw,
            raw_value,
            full: snapshot,
        }
    }
    fn safe_automation_shot(&self, ctx: &egui::Context, id: &str, label: &str) -> String {
        let rect = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let zoom = ctx.zoom_factor();
        eprintln!(
            "safe-automation geometry {id}/{label}: actual_content={rect:?} actual_pixels_per_point={ppp} actual_zoom={zoom}; viewport request is not a physical-window guarantee"
        );
        format!("{id}-{label}")
    }
    fn safe_automation_feedback(&mut self, ctx: &egui::Context) -> bool {
        self.reveal_smoke_control(
            ctx,
            self.metrics
                .script_feedback
                .expect("Actual script feedback summary"),
        )
    }
    // Only the first label-task wrong result has the Expected/Actual text pair
    // in this output panel. File-only mismatches retain their existing captures.
    fn safe_automation_wrong_output(&mut self, ctx: &egui::Context) -> bool {
        assert_eq!(self.smoke_stage, BASE + 20);
        assert!((self.scale - 1.).abs() < 0.01);
        let output = self.metrics.output.expect("Measured actual output frame");
        if !self.reveal_smoke_control(ctx, output) {
            return false;
        }
        let key = egui::Id::new(("safe-wrong-output-bottom", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        assert!(phase <= 18, "Finite inner-output capture sequence");
        if phase == 18 {
            return true;
        }
        // Hover inside the measured inner content, away from borders/scrollbars.
        // The next actual frame receives one wheel pulse; later frames only settle
        // egui's smooth scroll. No ScrollArea State or learning State is installed.
        let inner = output.shrink(12.);
        assert!(inner.is_positive());
        self.pending_events
            .push(egui::Event::PointerMoved(inner.center()));
        if phase == 1 {
            self.pending_events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., -4096.),
                modifiers: Default::default(),
            });
        }
        ctx.data_mut(|d| d.insert_temp(key, phase + 1));
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    pub(in crate::ui) fn safe_automation_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        const {
            assert!(shell::EVALUATOR_REVISION >= 10);
        }
        assert!(self.smoke && self._smoke_profile.is_some());
        assert!((BASE..=BASE + 2 * STRIDE + LAST).contains(&stage));
        let n = (stage - BASE) / STRIDE;
        let phase = (stage - BASE) % STRIDE;
        assert!(n < 3 && phase <= LAST);
        let id = IDS[n];
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("Normal safe-automation lab");
        assert_eq!((self.scenarios.len(), self.units.len()), (147, 37));
        assert_eq!(
            self.scenarios
                .iter()
                .filter(|s| s.track == "Bash/Shell")
                .count(),
            46
        );
        assert!(
            self.units
                .iter()
                .any(|u| u.id == "shell-safe-automation"
                    && u.labs.iter().map(String::as_str).eq(IDS))
        );
        assert_eq!(
            self.path.parent(),
            Some(self._smoke_profile.as_ref().unwrap().path())
        );
        let scenario = self.scenarios[i].clone();
        let lesson = scenario.teaching.as_ref().unwrap();
        let rows = plans();
        let row = &rows[n];
        assert_eq!(row.repair, lesson.example);
        let reference = &scenario.solution[0];
        let edited = format!("{reference}\n# Current edit checked again\n");
        let light = format!("{}\n# Actual typing at150percent\n", row.alternate);
        let dark = format!("{reference}\n# Actual typing at200percent\n");
        let incomplete = format!("{reference}\nprintf 'unfinished assignment");
        let example_incomplete = format!("{}\nprintf 'unfinished practice", lesson.example);
        let mut shot = None;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!(
                "safe-automation {id} phase{phase} job={}",
                self.script_job.is_some()
            );
        }
        match phase {
            0 => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-safe-automation".into(),
                    search: scenario.title.clone(),
                    ..Default::default()
                };
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            1 => {
                if !self.case_lesson_geometry(ctx) || !self.script_lesson_start_card(ctx, id) {
                    return (false, None);
                }
                assert_eq!(self.script_view(i), teaching::View::Introduction);
                self.progress.favorites.insert(id.into());
                self.dirty = true;
            }
            2 => {
                assert_eq!(self.script_view(i), teaching::View::Introduction);
                shot = Some(self.safe_automation_shot(ctx, id, "introduction"));
            }
            3 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.example_action) {
                    return (false, None);
                }
            }
            4 => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, scenario.initial);
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.example_source, lesson.example);
                assert!(l.example_result.is_none());
                assert!(!progress::current_solved(a, &scenario));
                shot = Some(self.safe_automation_shot(ctx, id, "separate-example-ready-not-run"));
            }
            5 | 9 | 13 | 19 | 23 | 27 | 32 | 37 | 45 | 52 | 58 | 62 | 66 | 71 | 75 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.check) {
                    return (false, None);
                }
            }
            6 | 10 | 14 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                assert_eq!(self.script_view(i), teaching::View::Example);
                let a = &self.progress.attempts[id];
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                let expected = if phase == 10 {
                    &row.experiment.expected
                } else {
                    &lesson.fixture
                };
                assert!(
                    expected.accepts(l.example_result.as_ref().unwrap()),
                    "Practice Outcome differs from independent author table"
                );
                assert_eq!(a.state, scenario.initial);
                assert!(!progress::current_solved(a, &scenario));
                shot = Some(self.safe_automation_shot(
                    ctx,
                    id,
                    match phase {
                        6 => "ungraded-example-run",
                        10 => "ungraded-example-experiment",
                        _ => "ungraded-example-repaired",
                    },
                ));
            }
            7 | 11 | 17 | 21 | 25 | 29 | 35 | 42 | 49 | 56 | 60 | 64 | 69 | 73 | 77 | 80 => {
                if !self.case_lesson_focus(ctx) {
                    return (false, None);
                }
            }
            8 => self.case_lesson_type(&row.experiment.source),
            12 => self.case_lesson_type(&lesson.example),
            15 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.assignment_action) {
                    return (false, None);
                }
            }
            16 => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                shot = Some(self.safe_automation_shot(ctx, id, "full-assignment"));
            }
            18 => self.case_lesson_type(&row.wrong),
            22 => self.case_lesson_type(&row.partial),
            26 | 74 => self.case_lesson_type(reference),
            30 => self.case_lesson_type(&edited),
            36 => self.case_lesson_type(&row.alternate),
            43 => self.case_lesson_type(&light),
            50 => self.case_lesson_type(&dark),
            57 => self.case_lesson_type(&incomplete),
            61 => self.case_lesson_type("IFS=: read -r row\n"),
            65 => self.case_lesson_type(LOOP),
            70 => self.case_lesson_type(&resource_source()),
            78 => self.case_lesson_type(&incomplete),
            81 => self.case_lesson_type(&example_incomplete),
            20 | 24 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                self.case_lesson_assignment(
                    i,
                    Some(if phase == 20 {
                        &row.wrong
                    } else {
                        &row.partial
                    }),
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
                let flags = cases
                    .iter()
                    .zip(&record.results)
                    .map(|(c, r)| c.accepts(r))
                    .collect::<Vec<_>>();
                assert!(flags.contains(&false));
                if phase == 24 {
                    assert!(flags.contains(&true));
                }
                let visible = if n == 0 && phase == 20 {
                    self.safe_automation_wrong_output(ctx)
                } else {
                    self.safe_automation_feedback(ctx)
                };
                if !visible {
                    return (false, None);
                }
                shot = Some(self.safe_automation_shot(
                    ctx,
                    id,
                    if phase == 20 {
                        "hardcoded-wrong-feedback"
                    } else {
                        "partial-misconception-feedback"
                    },
                ));
            }
            28 | 33 | 38 | 46 | 53 | 76 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                let expected = match phase {
                    28 | 76 => reference.as_str(),
                    33 => edited.as_str(),
                    38 => row.alternate.as_str(),
                    46 => light.as_str(),
                    _ => dark.as_str(),
                };
                self.case_lesson_assignment(i, Some(expected), true);
                if !self.safe_automation_feedback(ctx) {
                    return (false, None);
                }
                shot = Some(self.safe_automation_shot(
                    ctx,
                    id,
                    match phase {
                        28 => "reference-complete",
                        33 => "edited-source-rechecked",
                        38 => "genuine-alternate-complete",
                        46 => "light150-typed-and-run",
                        53 => "dark200-typed-and-run",
                        _ => "after-stop-limit-repaired",
                    },
                ));
            }
            31 | 44 | 51 => {
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                if phase != 31 {
                    let expected = if phase == 44 { &light } else { &dark };
                    assert!(
                        matches!(&self.progress.attempts[id].state,engine::State::Script{source,run:None}if source==expected)
                    );
                    assert!(
                        self.metrics
                            .focus
                            .is_some_and(|id| ctx.memory(|m| m.has_focus(id)))
                    );
                    shot = Some(self.safe_automation_shot(
                        ctx,
                        id,
                        if phase == 44 {
                            "light150-focused-input"
                        } else {
                            "dark200-focused-input"
                        },
                    ));
                }
            }
            34 | 54 => {
                let before = serde_json::to_value(&self.progress.attempts).unwrap();
                self.safe_automation_warm();
                exact(&self.progress.attempts, &before, "Warm completed Attempts");
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                assert!(self.progress.favorites.contains(id));
            }
            39 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            40 | 47 => {
                assert!(self.progress.attempts[id].challenge);
                assert!(!self.metrics.help_visible && self.metrics.lesson_tabs.is_empty());
                self.scale = if phase == 40 { 1.5 } else { 2. };
                self.theme_id = if phase == 40 {
                    "dlms-light"
                } else {
                    "dlms-dark"
                }
                .into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(if phase == 40 {
                    egui::vec2(960., 840.)
                } else {
                    egui::vec2(1600., 1000.)
                }));
                self.lesson_scroll_reset = true;
            }
            41 | 48 | 98 | 102 => {
                if !self.case_lesson_geometry(ctx) {
                    return (false, None);
                }
            }
            55 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            59 | 63 | 72 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                let engine::State::Script {
                    source,
                    run: Some(record),
                } = &self.progress.attempts[id].state
                else {
                    panic!()
                };
                let expected = if phase == 59 {
                    incomplete.clone()
                } else if phase == 63 {
                    "IFS=: read -r row\n".into()
                } else {
                    resource_source()
                };
                assert_eq!(source, &expected);
                assert_eq!(record.source, *source);
                assert_eq!(record.engine_revision, shell::EVALUATOR_REVISION);
                assert!(
                    record
                        .results
                        .iter()
                        .all(|r| r.error.as_deref().is_some_and(|message| matches!(
                            (phase, teaching::diagnostic(message)),
                            (59, teaching::Diagnostic::Syntax)
                                | (63, teaching::Diagnostic::Unsupported)
                                | (72, teaching::Diagnostic::Limit)
                        )))
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                if !self.reveal_smoke_control(
                    ctx,
                    self.metrics
                        .script_diagnostic
                        .expect("Visible real parser/resource diagnostic"),
                ) {
                    return (false, None);
                }
                shot = Some(self.safe_automation_shot(
                    ctx,
                    id,
                    match phase {
                        59 => "syntax-feedback",
                        63 => "unsupported-feedback",
                        _ => "bounded-resource-feedback",
                    },
                ));
            }
            67 => {
                // The final helper frame acknowledges the already delivered click.
                // Require a live job before hover/click, then observe cancellation
                // in phase 68 rather than demanding it survives its own Stop.
                let click_phase = ctx
                    .data(|d| d.get_temp::<u8>(egui::Id::new(("native-click-phases", stage))))
                    .unwrap_or(0);
                if click_phase < 2 {
                    assert!(
                        self.script_job.is_some(),
                        "Stop must occur during actual bounded execution"
                    );
                }
                if !self.click_visible_smoke_control(ctx, self.metrics.stop) {
                    return (false, None);
                }
            }
            68 => {
                assert!(self.script_job.is_none());
                assert!(
                    self.notice
                        .as_deref()
                        .is_some_and(|s| s.starts_with("Stopped"))
                );
                assert!(
                    matches!(&self.progress.attempts[id].state,engine::State::Script{source,run:None}if source==LOOP)
                );
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                shot = Some(self.safe_automation_shot(ctx, id, "real-run-stopped-without-credit"));
            }
            79 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(1).copied())
                {
                    return (false, None);
                }
            }
            82 => {
                assert_eq!(self.script_view(i), teaching::View::Example);
                let a = &self.progress.attempts[id];
                assert!(
                    matches!(&a.state,engine::State::Script{source,run:None}if source==&incomplete)
                );
                let l = a
                    .workspace
                    .as_ref()
                    .unwrap()
                    .script_learning
                    .as_ref()
                    .unwrap();
                assert_eq!(l.example_source, example_incomplete);
                assert!(l.example_result.is_none());
                assert!(!progress::current_solved(a, &scenario));
                let warm = self.safe_automation_warm();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new(("safe-warm", n)), warm));
                shot = Some(self.safe_automation_shot(ctx, id, "separate-unfinished-drafts-saved"));
            }
            83 | 96 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.back) {
                    return (false, None);
                }
                assert!(self.selected.is_none());
            }
            84 => {
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-safe-automation".into(),
                    search: scenario.title.clone(),
                    ..Default::default()
                };
            }
            85 => {
                let key = egui::Id::new(("safe-raw-before-continue", n));
                let checked = ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false);
                if !checked {
                    let warm = ctx
                        .data(|d| d.get_temp::<Warm>(egui::Id::new(("safe-warm", n))))
                        .unwrap();
                    assert!(
                        std::fs::read(&self.path).unwrap() == warm.raw,
                        "Before Continue every saved byte stays exact"
                    );
                    ctx.data_mut(|d| d.insert_temp(key, true));
                }
                if !self.script_lesson_start_card(ctx, id) {
                    return (false, None);
                }
            }
            86 => {
                assert_eq!(self.selected, Some(i));
                assert_eq!(self.script_view(i), teaching::View::Example);
                let warm = ctx
                    .data(|d| d.get_temp::<Warm>(egui::Id::new(("safe-warm", n))))
                    .unwrap();
                exact(
                    &serde_json::from_slice::<serde_json::Value>(&warm.raw).unwrap(),
                    &warm.raw_value,
                    "Retained pre-Continue raw JSON",
                );
                let mut expected = warm.full;
                let clock = expected["clock"].as_u64().unwrap().saturating_add(1);
                expected["clock"] = clock.into();
                expected["attempts"][id]["visited"] = clock.into();
                exact(
                    &full(&self.progress),
                    &expected,
                    "Continue changes exactly visit and clock",
                );
                self.save();
                assert!(self.storage_error.is_none());
                let raw = progress::read_bounded(&self.path, 16_000_000).unwrap();
                let mut expected_raw = warm.raw_value;
                expected_raw["clock"] = clock.into();
                expected_raw["attempts"][id]["visited"] = clock.into();
                exact(
                    &serde_json::from_slice::<serde_json::Value>(&raw).unwrap(),
                    &expected_raw,
                    "Normal autosave/explicit save changes only legitimate Continue fields",
                );
                let loaded = progress::load(&self.path, &self.scenarios).unwrap();
                exact(
                    &full(&loaded),
                    &expected,
                    "Normal load full Continue state and receipts",
                );
                assert!(std::fs::read(&self.path).unwrap() == raw);
                self.progress = loaded;
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                shot =
                    Some(self.safe_automation_shot(ctx, id, "real-continue-retains-both-drafts"));
            }
            87 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(2).copied())
                {
                    return (false, None);
                }
            }
            88 => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                let before = serde_json::to_value(&self.progress.attempts).unwrap();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new(("safe-before-reset", n)), before));
            }
            89 | 92 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset) {
                    return (false, None);
                }
            }
            90 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.reset_cancel) {
                    return (false, None);
                }
            }
            91 => {
                assert!(!self.reset_pending);
                let before = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new(("safe-before-reset", n)))
                    })
                    .unwrap();
                exact(
                    &self.progress.attempts,
                    &before,
                    "Cancel retains all exact Attempts",
                );
                shot = Some(self.safe_automation_shot(ctx, id, "reset-cancel-keeps-drafts"));
            }
            93 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.confirm_reset) {
                    return (false, None);
                }
            }
            94 => {
                let a = &self.progress.attempts[id];
                assert_eq!(a.state, scenario.initial);
                assert!(!progress::current_solved(a, &scenario));
                assert!(
                    a.workspace.as_ref().is_none_or(
                        |w| w.script_history.is_none() && w.script_history_more.is_empty()
                    )
                );
                let before = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new(("safe-before-reset", n)))
                    })
                    .unwrap();
                for (other, value) in before.as_object().unwrap() {
                    if other != id {
                        exact(
                            &self.progress.attempts[other],
                            value,
                            "Reset retains every other Attempt",
                        );
                    }
                }
                assert!(self.progress.favorites.contains(id));
            }
            95 => {
                self.safe_automation_warm();
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            97 => {
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-safe-automation".into(),
                    ..Default::default()
                };
            }
            99 | 103 => {
                assert!(self.selected.is_none());
                assert!(self.filter.search.is_empty());
                assert_eq!(self.metrics.card_actions.len(), 3);
                assert!(
                    self.units
                        .iter()
                        .any(|u| u.id == self.filter.unit
                            && u.labs.iter().map(String::as_str).eq(IDS))
                );
                if phase == 103 {
                    shot = Some(self.safe_automation_shot(
                        ctx,
                        id,
                        "normal-three-lesson-unit-after-reset",
                    ));
                }
            }
            100 => {
                assert!(self.progress.favorites.contains(id));
                assert!(!self.reset_pending);
            }
            101 => {
                self.scale = 1.;
                self.theme_id = "dlms-dark".into();
                theme::apply(ctx, &self.theme_id);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1200., 850.)));
            }
            104 => {
                if n == 2 {
                    for completed_id in IDS {
                        let completed_index = self
                            .scenarios
                            .iter()
                            .position(|s| s.id == completed_id)
                            .unwrap();
                        let completed = self
                            .progress
                            .attempts
                            .get(completed_id)
                            .expect("Every normal lesson was started and reset");
                        assert_eq!(completed.state, self.scenarios[completed_index].initial);
                        assert!(self.progress.favorites.contains(completed_id));
                        assert!(!progress::current_solved(
                            completed,
                            &self.scenarios[completed_index]
                        ));
                    }
                    println!(
                        "PASS normal three safe-automation native flow: actualStart/Intro/example/experiment/repair/assignment, wrong+partial+reference+alternate, typing/Challenge Light150 Dark200 actualgeometry, syntax+unsupported+limit, realRunStop, fullraw/portable/receipt warmContinue, resetCancel/Confirm and3normalcards. No cold-process, novice/accessibility/otherOS claim."
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return (false, None);
                }
                self.smoke_stage = BASE + (n + 1) * STRIDE - 1;
            }
            _ => unreachable!("Closed native safe-automation phase"),
        }
        (true, shot)
    }
}
