//! Closed native-only normal-course acceptance. All script effects occur through
//! actual editor events and Run/Stop; no State/Record or solved result is installed.
use super::*;
pub(in crate::ui) const BASE: usize = 75000;
const STRIDE: usize = 250;
pub(in crate::ui) const LAST: usize = 218;
const IDS: [&str; 2] = ["shell-loop-intake", "shell-loop-fallback"];
const LOOP: &str = "while true; do :; done\n";
struct Plan {
    alternates: Vec<String>,
    partial: String,
    wrong: String,
    wrong_expected: Vec<bool>,
    partial_expected: Vec<bool>,
    experiment: Experiment,
    repair: String,
}
struct Experiment {
    source: String,
    expected: shell::Case,
}
fn plans() -> Vec<Plan> {
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shell-loop-recovery-native-review.json"
    ))
    .unwrap();
    let rows = value["plans"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    rows.iter()
        .zip(IDS)
        .map(|(row, id)| {
            assert_eq!(row["id"], id);
            let alternates: Vec<String> =
                serde_json::from_value(row["alternates"].clone()).unwrap();
            assert_eq!(alternates.len(), 2);
            Plan {
                alternates,
                partial: row["partial"].as_str().unwrap().into(),
                wrong: row["wrong"].as_str().unwrap().into(),
                partial_expected: serde_json::from_value(row["partial_expected"].clone()).unwrap(),
                wrong_expected: serde_json::from_value(row["wrong_expected"].clone()).unwrap(),
                experiment: Experiment {
                    source: row["experiment"]["source"].as_str().unwrap().into(),
                    expected: serde_json::from_value(row["experiment"]["expected"].clone())
                        .unwrap(),
                },
                repair: row["repair"].as_str().unwrap().into(),
            }
        })
        .collect()
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
// Capture-only full-profile laws; no expected progress is installed.
pub(in crate::ui) fn cancel_preserved(
    before: &serde_json::Value,
    observed: &serde_json::Value,
) -> bool {
    observed == before
}
pub(in crate::ui) fn reset_expected(
    before: &serde_json::Value,
    s: &pack::Scenario,
) -> serde_json::Value {
    assert!(matches!(s.initial, engine::State::Script { .. }));
    assert!(matches!(s.goal, engine::Goal::Script { .. }));
    let visited = before["attempts"][&s.id]["visited"]
        .as_u64()
        .expect("Pre-reset selected recency");
    let mut fresh = progress::Attempt::new(s, visited);
    model::prepare(&mut fresh);
    teaching::prepare(
        &mut fresh,
        s.teaching.as_ref().expect("Normal script teaching"),
        true,
    );
    let mut expected = before.clone();
    expected["attempts"][&s.id] = serde_json::to_value(fresh).unwrap();
    expected
}
pub(in crate::ui) fn reset_preserved(
    before: &serde_json::Value,
    observed: &serde_json::Value,
    s: &pack::Scenario,
) -> bool {
    observed == &reset_expected(before, s)
}

#[derive(Clone)]
struct Warm {
    raw: Vec<u8>,
    raw_value: serde_json::Value,
    full: serde_json::Value,
    loaded_full: serde_json::Value,
}
#[derive(Clone)]
struct GoalRows {
    rows: Vec<egui::Rect>,
    clip: egui::Rect,
}
fn painted_goal_rows(
    ctx: &egui::Context,
    objective: &str,
    visible: egui::Rect,
) -> Option<GoalRows> {
    fn find(
        shape: &egui::Shape,
        clip: egui::Rect,
        objective: &str,
        visible: egui::Rect,
    ) -> Option<GoalRows> {
        match shape {
            egui::Shape::Text(text) if text.galley.text() == objective => {
                assert!(
                    !text.galley.elided,
                    "Essential task text must never be elided"
                );
                let rows = text
                    .galley
                    .rows
                    .iter()
                    .filter(|row| !row.text().trim().is_empty())
                    .map(|row| row.rect().translate(text.pos.to_vec2()))
                    .collect::<Vec<_>>();
                assert!(
                    !rows.is_empty() && rows.len() <= 128,
                    "Bounded actual task rows"
                );
                Some(GoalRows {
                    rows,
                    clip: clip.intersect(visible),
                })
            }
            egui::Shape::Vec(shapes) => shapes
                .iter()
                .find_map(|s| find(s, clip, objective, visible)),
            _ => None,
        }
    }
    // CentralPanel uses the background painter. No Context calls within graphics.
    ctx.graphics(|graphics| {
        graphics.get(egui::LayerId::background()).and_then(|list| {
            list.all_entries()
                .find_map(|s| find(&s.shape, s.clip_rect, objective, visible))
        })
    })
}
#[derive(Clone)]
struct GoalProof {
    seen: Vec<bool>,
}
fn editor_band(viewport: egui::Rect, height: f32) -> egui::Rect {
    assert!(viewport.is_positive());
    let half = height.min(viewport.height()) / 2.;
    egui::Rect::from_min_max(
        egui::pos2(
            viewport.left(),
            (viewport.center().y - half).max(viewport.top()),
        ),
        egui::pos2(
            viewport.right(),
            (viewport.center().y + half).min(viewport.bottom()),
        ),
    )
}
impl LabApp {
    pub(in crate::ui) fn loop_recovery_stage(smoke: bool, name: Option<&str>) -> Option<usize> {
        (smoke && name == Some("loop-recovery-lessons")).then_some(BASE)
    }
    fn loop_recovery_window_ready(&mut self, ctx: &egui::Context, n: usize, wide: bool) -> bool {
        let fullscreen = ctx.input(|i| i.viewport().fullscreen);
        let physical = ctx.content_rect().size() * ctx.pixels_per_point();
        let baseline = ctx
            .data(|d| d.get_temp::<f32>(egui::Id::new(("loop-windowed-width", n))))
            .expect("Observed windowed baseline before fullscreen requests");
        let started = ctx.data_mut(|d| {
            let key = egui::Id::new(("loop-window-wait", self.smoke_stage));
            let started = d.get_temp::<Instant>(key).unwrap_or_else(Instant::now);
            d.insert_temp(key, started);
            started
        });
        assert!(
            started.elapsed().as_secs() < 15,
            "Own native window did not acknowledge a distinct requested fullscreen/windowed geometry"
        );
        let distinct_size = if wide {
            physical.x > baseline * 1.25
        } else {
            physical.x <= baseline * 1.05
        };
        if fullscreen != Some(wide) || !distinct_size {
            self.started = Instant::now();
            ctx.request_repaint();
            return false;
        }
        self.case_lesson_geometry(ctx)
    }
    fn loop_recovery_goal_center(
        &mut self,
        ctx: &egui::Context,
        row: egui::Rect,
        clip: egui::Rect,
    ) -> bool {
        let offset = self.metrics.main_scroll_offset.y;
        let key = egui::Id::new(("loop-goal-center", self.smoke_stage));
        let (pending, previous, stable, pulses) = ctx
            .data(|d| d.get_temp::<(bool, f32, usize, usize)>(key))
            .unwrap_or((false, offset, 0, 0));
        if pending {
            let stable = if (offset - previous).abs() < 0.01 {
                stable + 1
            } else {
                0
            };
            ctx.data_mut(|d| d.insert_temp(key, (stable < 2, offset, stable, pulses)));
            if stable < 2 {
                self.started = Instant::now();
                ctx.request_repaint();
                return false;
            }
        }
        let delta = clip.center().y - row.center().y;
        if delta.abs() <= 1. / ctx.pixels_per_point() {
            return true;
        }
        assert!(pulses < 20, "Bounded real middle-task wheel alignment");
        self.pending_events
            .push(egui::Event::PointerMoved(egui::pos2(
                clip.left() + 2.,
                clip.center().y,
            )));
        self.pending_events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0., delta),
            modifiers: Default::default(),
        });
        ctx.data_mut(|d| d.insert_temp(key, (true, offset, 0usize, pulses + 1)));
        self.started = Instant::now();
        ctx.request_repaint();
        false
    }
    fn loop_recovery_focus(&mut self, ctx: &egui::Context) -> bool {
        let key = egui::Id::new(("native-click-phases", self.smoke_stage));
        let phase = ctx.data(|d| d.get_temp::<u8>(key)).unwrap_or(0);
        let clicked = if phase >= 2 {
            self.click_smoke_control(ctx, None)
        } else {
            let viewport = self
                .metrics
                .editor_viewport
                .expect("Actual loop-recovery editor viewport");
            let height = ctx.style().spacing.interact_size.y;
            let legacy = egui::Rect::from_center_size(
                viewport.center(),
                egui::vec2(viewport.width(), height.min(viewport.height())),
            );
            if !viewport.contains_rect(legacy) {
                eprintln!(
                    "loop-recovery exact-band diagnostic stage{}: legacy left delta={:.9} right delta={:.9} top delta={:.9} bottom delta={:.9}; using original viewport edges, no expanded bounds",
                    self.smoke_stage,
                    legacy.left() - viewport.left(),
                    legacy.right() - viewport.right(),
                    legacy.top() - viewport.top(),
                    legacy.bottom() - viewport.bottom()
                );
            }
            let band = editor_band(viewport, height);
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
                    .is_some_and(|f| ctx.memory(|m| m.has_focus(f))),
                "Actual editor click establishes focus"
            );
        }
        clicked
    }
    fn loop_recovery_warm(&mut self) -> Warm {
        let in_memory = full(&self.progress);
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
        exact(
            &full(&self.progress),
            &in_memory,
            "Normal save/load never installs a loaded Progress",
        );
        Warm {
            raw,
            raw_value,
            full: in_memory,
            loaded_full: snapshot,
        }
    }
    fn loop_recovery_shot(&self, ctx: &egui::Context, id: &str, label: &str) -> String {
        let rect = ctx.content_rect();
        let ppp = ctx.pixels_per_point();
        let zoom = ctx.zoom_factor();
        let actual_fullscreen = ctx.input(|i| i.viewport().fullscreen);
        let actual_pixels = rect.size() * ppp;
        eprintln!(
            "loop-recovery geometry {id}/{label}: actual_content={rect:?} actual_pixels_per_point={ppp} actual_zoom={zoom} actual_fullscreen={actual_fullscreen:?} actual_pixels={actual_pixels:?}; requests alone are not geometry proof"
        );
        format!("{id}-{label}")
    }
    fn loop_recovery_feedback(&mut self, ctx: &egui::Context) -> bool {
        self.reveal_smoke_control(
            ctx,
            self.metrics
                .script_feedback
                .expect("Actual script feedback summary"),
        )
    }
    fn loop_recovery_key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        for pressed in [true, false] {
            self.pending_events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            });
        }
    }
    fn loop_recovery_run(&mut self, ctx: &egui::Context, i: usize, stage: usize) -> bool {
        let key = egui::Id::new(("loop-fresh-run", stage));
        if ctx.data(|d| d.get_temp::<bool>(key)) != Some(true) {
            assert!(self.script_job.is_none());
            let a = &self.progress.attempts[&self.scenarios[i].id];
            if self.script_view(i) == teaching::View::Example {
                assert!(
                    a.workspace
                        .as_ref()
                        .unwrap()
                        .script_learning
                        .as_ref()
                        .unwrap()
                        .example_result
                        .is_none()
                );
            } else {
                assert!(matches!(&a.state, engine::State::Script { run: None, .. }));
            }
            ctx.data_mut(|d| d.insert_temp(key, true));
        }
        self.click_visible_smoke_control(ctx, self.metrics.check)
    }
    fn loop_recovery_undo(&mut self, ctx: &egui::Context, i: usize, reference: &str) -> bool {
        let id = self.scenarios[i].id.clone();
        let phase_key = egui::Id::new(("loop-undo-phase", i));
        let phase = ctx.data(|d| d.get_temp::<u8>(phase_key)).unwrap_or(0);
        let now = ctx.input(|i| i.time);
        let cadence_key = egui::Id::new(("loop-undo-cadence", i));
        let (start, next) = ctx
            .data(|d| d.get_temp::<(f64, f64)>(cadence_key))
            .unwrap_or((now, now));
        assert!(now.is_finite() && now >= start && now - start < 15.);
        if now < next {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            return false;
        }
        ctx.data_mut(|d| d.insert_temp(cadence_key, (start, now + 0.05)));
        let time_key = egui::Id::new(("loop-undo-stable", i));
        let modified = format!("{reference}# Native Undo/Redo checkpoint\n");
        let source = match &self.progress.attempts[&id].state {
            engine::State::Script { source, .. } => source.clone(),
            _ => panic!(),
        };
        match phase {
            0 => {
                if !self.loop_recovery_focus(ctx) {
                    return false;
                }
                assert_eq!(source, reference);
                // Move the caret in its own real event frame before the stable
                // checkpoint. Undo must restore this observed caret, not the
                // earlier pointer-click position in the middle of the source.
                self.loop_recovery_key(
                    egui::Key::End,
                    egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                );
                ctx.data_mut(|d| d.insert_temp(time_key, now));
            }
            1 => {
                let cursor = egui::text_edit::TextEditState::load(ctx, self.metrics.focus.unwrap())
                    .unwrap()
                    .cursor
                    .char_range()
                    .unwrap();
                assert!(cursor.is_empty());
                assert_eq!(
                    cursor.primary.index,
                    reference.chars().count(),
                    "Observe Ctrl+End before waiting for the original cursor checkpoint"
                );
                let since = ctx.data(|d| d.get_temp::<f64>(time_key)).unwrap();
                if now - since < 1.2 {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                    return false;
                }
                self.pending_events
                    .push(egui::Event::Text("# Native Undo/Redo checkpoint\n".into()));
                ctx.data_mut(|d| d.insert_temp(time_key, now));
            }
            2 => {
                assert_eq!(source, modified);
                assert!(!progress::current_solved(
                    &self.progress.attempts[&id],
                    &self.scenarios[i]
                ));
                let since = ctx.data(|d| d.get_temp::<f64>(time_key)).unwrap();
                if now - since < 1.2 {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                    return false;
                }
                let value = full(&self.progress);
                ctx.data_mut(|d| d.insert_temp(egui::Id::new(("loop-undo-edited", i)), value));
                self.loop_recovery_key(
                    egui::Key::Z,
                    egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                );
            }
            3..=5 => {
                let wanted = if phase == 4 { &modified } else { reference };
                let field = self.metrics.focus.unwrap();
                assert!(ctx.memory(|m| m.has_focus(field)));
                if source != wanted {
                    let k = egui::Id::new(("loop-undo-retries", i, phase));
                    let n = ctx.data(|d| d.get_temp::<u8>(k)).unwrap_or(0) + 1;
                    assert!(
                        n <= 3,
                        "Actual text/cursor Undo must restore within3 checkpoints"
                    );
                    ctx.data_mut(|d| d.insert_temp(k, n));
                    self.loop_recovery_key(
                        egui::Key::Z,
                        egui::Modifiers {
                            ctrl: true,
                            command: true,
                            shift: phase == 4,
                            ..Default::default()
                        },
                    );
                    return false;
                }
                let mut expected = ctx
                    .data(|d| {
                        d.get_temp::<serde_json::Value>(egui::Id::new(("loop-undo-edited", i)))
                    })
                    .unwrap();
                expected["attempts"][&id]["state"]["source"] = wanted.into();
                exact(
                    &full(&self.progress),
                    &expected,
                    "Undo/Redo preserves entire progress and receipts, changing only source",
                );
                let cursor = egui::text_edit::TextEditState::load(ctx, field)
                    .unwrap()
                    .cursor
                    .char_range()
                    .unwrap();
                assert!(cursor.is_empty());
                assert_eq!(cursor.primary.index, wanted.chars().count());
                if phase < 5 {
                    self.loop_recovery_key(
                        egui::Key::Z,
                        egui::Modifiers {
                            ctrl: true,
                            command: true,
                            shift: phase == 3,
                            ..Default::default()
                        },
                    );
                } else {
                    return true;
                }
            }
            _ => unreachable!(),
        }
        ctx.data_mut(|d| d.insert_temp(phase_key, phase + 1));
        ctx.request_repaint();
        false
    }
    fn loop_recovery_inner(&mut self, ctx: &egui::Context, stage: usize) -> (bool, Option<String>) {
        const {
            assert!(shell::EVALUATOR_REVISION == 14);
        }
        assert!(self.smoke && self._smoke_profile.is_some());
        assert_eq!(shell::EVALUATOR_REVISION, 14);
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
            self.scenarios[180..182]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        assert!((BASE..=BASE + STRIDE + LAST).contains(&stage));
        let n = (stage - BASE) / STRIDE;
        let phase = (stage - BASE) % STRIDE;
        assert!(n < 2 && phase <= LAST);
        let id = IDS[n];
        let i = self
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .expect("Normal loop-recovery lab");
        // Exact two normal specs are checked independently of evolving parent count.
        let authored: pack::Pack = serde_json::from_str(include_str!(
            "../../../scenarios/shell-loop-recovery-v1.json"
        ))
        .unwrap();
        assert_eq!(authored.scenarios.len(), 2);
        for expected in &authored.scenarios {
            let actual = self.scenarios.iter().find(|s| s.id == expected.id).unwrap();
            exact(
                actual,
                expected,
                "Exact authored loop-recovery specification",
            );
        }
        assert!(self.units.iter().any(
            |u| u.id == "shell-loop-recovery" && u.labs.iter().map(String::as_str).eq(IDS)
        ));
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
        let light = format!("{}\n# Actual typing at150percent\n", row.alternates[0]);
        let dark = format!("{}\n# Actual typing at200percent\n", row.alternates[1]);
        let incomplete = format!("{reference}\nprintf 'unfinished assignment");
        let example_incomplete = format!("{}\nprintf 'unfinished practice", lesson.example);
        let mut shot = None;
        if std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some() {
            eprintln!(
                "loop-recovery {id} phase{phase} job={}",
                self.script_job.is_some()
            );
        }
        match phase {
            0 => {
                assert!(self.selected.is_none());
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-loop-recovery".into(),
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
                if !self.case_lesson_geometry(ctx) {
                    return (false, None);
                }
                if ctx.input(|i| i.viewport().fullscreen) != Some(false) {
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                }
                let physical_width = ctx.content_rect().width() * ctx.pixels_per_point();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new(("loop-windowed-width", n)), physical_width)
                });
                let favored = egui::Id::new(("loop-recovery-real-favorite", n));
                if ctx.data(|d| d.get_temp::<bool>(favored)) != Some(true) {
                    assert!(self.selected.is_none());
                    let matches = self
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
                        .collect::<Vec<_>>();
                    let ordinal = matches.iter().position(|s| s.id == id).unwrap();
                    assert_eq!(self.metrics.card_actions.len(), matches.len());
                    if !self.click_visible_smoke_control(
                        ctx,
                        self.metrics.card_actions.get(ordinal).map(|r| r.1),
                    ) {
                        return (false, None);
                    }
                    assert!(self.progress.favorites.contains(id));
                    ctx.data_mut(|d| d.insert_temp(favored, true));
                    return (false, None);
                }
                if !self.script_lesson_start_card(ctx, id) {
                    return (false, None);
                }
                assert_eq!(self.script_view(i), teaching::View::Introduction);
            }
            2 => {
                assert_eq!(self.script_view(i), teaching::View::Introduction);
                shot = Some(self.loop_recovery_shot(ctx, id, "introduction"));
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
                shot = Some(self.loop_recovery_shot(ctx, id, "separate-example-ready-not-run"));
            }
            5 | 9 | 13 | 19 | 23 | 27 | 32 | 37 | 45 | 52 | 58 | 62 | 66 | 71 | 75 => {
                if !self.loop_recovery_run(ctx, i, stage) {
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
                shot = Some(self.loop_recovery_shot(
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
                if !self.loop_recovery_focus(ctx) {
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
                shot = Some(self.loop_recovery_shot(ctx, id, "full-assignment"));
            }
            18 => self.case_lesson_type(&row.wrong),
            22 => self.case_lesson_type(&row.partial),
            26 | 74 => self.case_lesson_type(reference),
            30 => self.case_lesson_type(&edited),
            36 => self.case_lesson_type(&row.alternates[0]),
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
                assert_eq!(
                    flags,
                    if phase == 20 {
                        row.wrong_expected.clone()
                    } else {
                        row.partial_expected.clone()
                    },
                    "Frozen independent wrong/partial vector"
                );
                let failed = flags.iter().position(|p| !p).unwrap();
                let inspected = egui::Id::new(("loop-recovery-inspected-case", stage));
                if !ctx.data(|d| d.get_temp::<bool>(inspected)).unwrap_or(false) {
                    let target = self
                        .metrics
                        .script_case_rows
                        .iter()
                        .find(|(n, _)| *n == failed)
                        .map(|(_, r)| *r);
                    if !self.click_visible_smoke_control(ctx, target) {
                        return (false, None);
                    }
                    ctx.data_mut(|d| d.insert_temp(inspected, true));
                    ctx.request_repaint();
                    return (false, None);
                }
                assert_eq!(
                    self.script_case, failed,
                    "Real case-row click selects the named failure"
                );
                assert!(
                    !self.metrics.completion,
                    "One passing selected case never makes overall success"
                );
                let visible = self.loop_recovery_feedback(ctx);
                if !visible {
                    return (false, None);
                }
                shot = Some(self.loop_recovery_shot(
                    ctx,
                    id,
                    if phase == 20 {
                        "hardcoded-wrong-feedback"
                    } else {
                        "partial-misconception-feedback"
                    },
                ));
                if phase == 20 {
                    self.smoke_stage = BASE + n * STRIDE + 215;
                }
            }
            28 | 33 | 38 | 46 | 53 | 76 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                let expected = match phase {
                    28 | 76 => reference.as_str(),
                    33 => edited.as_str(),
                    38 => row.alternates[0].as_str(),
                    46 => light.as_str(),
                    _ => dark.as_str(),
                };
                self.case_lesson_assignment(i, Some(expected), true);
                if !self.loop_recovery_feedback(ctx) {
                    return (false, None);
                }
                if phase == 28 {
                    self.smoke_stage = BASE + n * STRIDE + 199;
                } else if phase == 53 && n != 0 {
                    self.smoke_stage = BASE + n * STRIDE + if n == 1 { 164 } else { 53 };
                }
                if !matches!(phase, 46 | 53) {
                    shot = Some(self.loop_recovery_shot(
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
                }
            }
            34 | 54 => {
                let before = serde_json::to_value(&self.progress.attempts).unwrap();
                self.loop_recovery_warm();
                exact(&self.progress.attempts, &before, "Warm completed Attempts");
                assert!(progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                assert!(self.progress.favorites.contains(id));
                if phase == 54 {
                    self.smoke_stage = BASE
                        + n * STRIDE
                        + if n == 0
                            && ctx.data(|d| {
                                d.get_temp::<bool>(egui::Id::new(("loop-matrix-finished", n)))
                            }) != Some(true)
                        {
                            104
                        } else {
                            54
                        };
                }
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
                shot = Some(self.loop_recovery_shot(
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
                shot = Some(self.loop_recovery_shot(ctx, id, "real-run-stopped-without-credit"));
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
                let warm = self.loop_recovery_warm();
                ctx.data_mut(|d| d.insert_temp(egui::Id::new(("loop-recovery-warm", n)), warm));
                shot = Some(self.loop_recovery_shot(ctx, id, "separate-unfinished-drafts-saved"));
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
                    unit: "shell-loop-recovery".into(),
                    search: scenario.title.clone(),
                    ..Default::default()
                };
            }
            85 => {
                let key = egui::Id::new(("loop-recovery-raw-before-continue", n));
                let checked = ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false);
                if !checked {
                    let warm = ctx
                        .data(|d| d.get_temp::<Warm>(egui::Id::new(("loop-recovery-warm", n))))
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
                    .data(|d| d.get_temp::<Warm>(egui::Id::new(("loop-recovery-warm", n))))
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
                let mut expected_loaded = warm.loaded_full;
                expected_loaded["clock"] = clock.into();
                expected_loaded["attempts"][id]["visited"] = clock.into();
                exact(
                    &full(&loaded),
                    &expected_loaded,
                    "Normal load full Continue state and receipts",
                );
                assert!(std::fs::read(&self.path).unwrap() == raw);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                shot = Some(self.loop_recovery_shot(ctx, id, "real-continue-retains-both-drafts"));
            }
            87 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(2).copied())
                {
                    return (false, None);
                }
            }
            88 => {
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                let before = full(&self.progress);
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new(("loop-recovery-before-reset", n)), before)
                });
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
                        d.get_temp::<serde_json::Value>(egui::Id::new((
                            "loop-recovery-before-reset",
                            n,
                        )))
                    })
                    .unwrap();
                let observed = full(&self.progress);
                assert!(
                    cancel_preserved(&before, &observed),
                    "Cancel must preserve complete progress, receipts, recency and favorites"
                );
                exact(
                    &observed,
                    &before,
                    "Cancel preserves every progress field and receipt",
                );
                shot = Some(self.loop_recovery_shot(ctx, id, "reset-cancel-keeps-drafts"));
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
                        d.get_temp::<serde_json::Value>(egui::Id::new((
                            "loop-recovery-before-reset",
                            n,
                        )))
                    })
                    .unwrap();
                let observed = full(&self.progress);
                assert!(
                    reset_preserved(&before, &observed, &scenario),
                    "Reset may replace only the selected Attempt with fresh state, default workspace and teaching; preserve clock, visited, favorites, receipts and all other work"
                );
                exact(
                    &observed,
                    &reset_expected(&before, &scenario),
                    "Exact selected-reset profile law",
                );
                for (other, value) in before["attempts"].as_object().unwrap() {
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
                self.loop_recovery_warm();
                assert_eq!(self.progress.attempts[id].state, scenario.initial);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
            }
            97 => {
                self.filter = Filter {
                    track: "Bash/Shell".into(),
                    unit: "shell-loop-recovery".into(),
                    ..Default::default()
                };
            }
            99 | 103 => {
                assert!(self.selected.is_none());
                assert!(self.filter.search.is_empty());
                assert_eq!(self.metrics.card_actions.len(), 2);
                assert!(
                    self.units
                        .iter()
                        .any(|u| u.id == self.filter.unit
                            && u.labs.iter().map(String::as_str).eq(IDS))
                );
                if phase == 103 {
                    shot = Some(self.loop_recovery_shot(
                        ctx,
                        id,
                        "normal-two-lesson-unit-after-reset",
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
            105..=164 => {
                let k = (phase - 105) / 3;
                let part = (phase - 105) % 3;
                let scale: f32 = [1., 1.15, 1.25, 1.5, 2.][k / 4];
                let light = k % 4 < 2;
                let wide = k.is_multiple_of(2);
                let requested = if wide {
                    egui::vec2(1280., 950.)
                } else {
                    egui::vec2(850., 850.)
                };
                let theme_id = if light { "dlms-light" } else { "dlms-dark" };
                let label = format!(
                    "layout-{}-{}-{}",
                    (scale * 100.).round() as usize,
                    if light { "light" } else { "dark" },
                    if wide { "wide" } else { "narrow" }
                );
                let typed = format!("{reference}\n# Actual editor input at {label}\n");
                match part {
                    0 => {
                        let key = egui::Id::new(("loop-recovery-settings-request", stage));
                        let requested_once = ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false);
                        if !requested_once {
                            self.scale = scale;
                            self.theme_id = theme_id.into();
                            theme::apply(ctx, theme_id);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(wide));
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(requested));
                            self.lesson_scroll_reset = true;
                            ctx.data_mut(|d| d.insert_temp(key, true));
                            ctx.request_repaint();
                            return (false, None);
                        }
                        // Wait for observed geometry AFTER request, before using old widget metrics.
                        if !self.loop_recovery_window_ready(ctx, n, wide) {
                            return (false, None);
                        }
                        if !self.loop_recovery_focus(ctx) {
                            return (false, None);
                        }
                    }
                    1 => self.case_lesson_type(&typed),
                    2 => {
                        assert!((ctx.zoom_factor() - scale).abs() < 0.01);
                        assert_eq!(self.theme_id, theme_id);
                        assert_eq!(self.script_view(i), teaching::View::Assignment);
                        assert!(
                            self.metrics
                                .focus
                                .is_some_and(|f| ctx.memory(|m| m.has_focus(f)))
                        );
                        assert!(
                            matches!(&self.progress.attempts[id].state, engine::State::Script { source, run: None } if source == &typed)
                        );
                        assert!(!progress::current_solved(
                            &self.progress.attempts[id],
                            &scenario
                        ));
                        let viewport = self
                            .metrics
                            .editor_viewport
                            .expect("Actual editor viewport");
                        let band = editor_band(viewport, ctx.style().spacing.interact_size.y);
                        // Insertion may scroll the editor caret. Reveal again through
                        // actual wheel input before the measured focused capture.
                        if !self.reveal_smoke_control(ctx, band) {
                            return (false, None);
                        }
                        let actual_clip = self
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        assert!(
                            actual_clip
                                .expand(1. / ctx.pixels_per_point())
                                .contains_rect(band),
                            "Actual capture band {band:?} outside {actual_clip:?} at {label}"
                        );
                        let physical_width = ctx.content_rect().width() * ctx.pixels_per_point();
                        let pair = egui::Id::new(("loop-actual-wide-width", n, k / 2));
                        if wide {
                            ctx.data_mut(|d| d.insert_temp(pair, physical_width));
                        } else {
                            let actual_wide = ctx
                                .data(|d| d.get_temp::<f32>(pair))
                                .expect("Actual wide capture precedes paired narrow capture");
                            assert!(
                                actual_wide > physical_width * 1.25,
                                "Wide/narrow capture pair must have meaningfully different ACTUAL pixel widths"
                            );
                        }
                        shot = Some(self.loop_recovery_shot(ctx, id, &label));
                        if k == 19 {
                            ctx.data_mut(|d| {
                                d.insert_temp(egui::Id::new(("loop-matrix-finished", n)), true)
                            });
                            self.smoke_stage = BASE + n * STRIDE + 54;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            165..=180 => {
                assert_eq!(n, 1, "Only the capstone needs these long-task slices");
                let setting = (phase - 165) / 4;
                let slice = (phase - 165) % 4;
                let top = slice == 0;
                let middle = slice == 1 || slice == 2;
                let bottom = slice == 3;
                let light = setting < 2;
                let wide = setting.is_multiple_of(2);
                let requested = if wide {
                    egui::vec2(1280., 950.)
                } else {
                    egui::vec2(850., 850.)
                };
                let theme_id = if light { "dlms-light" } else { "dlms-dark" };
                let label = format!(
                    "goal200-{}-{}-{}",
                    if light { "light" } else { "dark" },
                    if wide { "wide" } else { "narrow" },
                    if bottom {
                        "bottom"
                    } else if slice == 1 {
                        "middle-one"
                    } else if slice == 2 {
                        "middle-two"
                    } else {
                        "top"
                    }
                );
                let requested_key = egui::Id::new(("loop-goal-setting", setting));
                if top
                    && !ctx
                        .data(|d| d.get_temp::<bool>(requested_key))
                        .unwrap_or(false)
                {
                    self.scale = 2.;
                    self.theme_id = theme_id.into();
                    theme::apply(ctx, theme_id);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(wide));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(requested));
                    // Do not install a scroll offset. Reach the task using real wheel input.
                    ctx.data_mut(|d| d.insert_temp(requested_key, true));
                    ctx.request_repaint();
                    return (false, None);
                }
                if !self.loop_recovery_window_ready(ctx, n, wide) {
                    return (false, None);
                }
                assert!((ctx.zoom_factor() - 2.).abs() < 0.01);
                assert_eq!(self.theme_id, theme_id);
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                let clip = self
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let Some(rows) = painted_goal_rows(ctx, &scenario.objective, clip) else {
                    // From the focused editor below the task, scroll upward normally
                    // until the real label contributes a painted galley. Bounded20.
                    assert!(top, "Middle or bottom slice lost the already observed task");
                    let key = egui::Id::new(("loop-goal-seek", stage));
                    let tries = ctx.data(|d| d.get_temp::<usize>(key)).unwrap_or(0) + 1;
                    assert!(
                        tries <= 20,
                        "Task label must be reachable within20actual wheel pulses"
                    );
                    ctx.data_mut(|d| d.insert_temp(key, tries));
                    self.pending_events
                        .push(egui::Event::PointerMoved(egui::pos2(
                            clip.left() + 2.,
                            clip.center().y,
                        )));
                    self.pending_events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., 180.),
                        modifiers: Default::default(),
                    });
                    self.started = Instant::now();
                    ctx.request_repaint();
                    return (false, None);
                };
                let target = if bottom {
                    *rows.rows.last().unwrap()
                } else if middle {
                    rows.rows[(rows.rows.len() - 1) * slice / 3]
                } else {
                    rows.rows[0]
                };
                if middle || bottom {
                    // A short fully painted goal needs no extra scroll, which
                    // could be impossible at the document's upper boundary.
                    // Center a long goal's final row as well: mere edge containment
                    // can leave its text behind a nested floating scrollbar.
                    let all_visible = rows.rows.iter().all(|rect| {
                        rows.clip
                            .expand(1. / ctx.pixels_per_point())
                            .contains_rect(*rect)
                    });
                    if !all_visible {
                        if !self.loop_recovery_goal_center(ctx, target, rows.clip) {
                            return (false, None);
                        }
                        assert!(
                            (target.center().y - rows.clip.center().y).abs()
                                <= 1. / ctx.pixels_per_point(),
                            "A long goal's middle/final target must paint at the observed clip center"
                        );
                        assert!(
                            target.height() * 2. < rows.clip.height(),
                            "Centered target needs real clearance from edge scrollbars"
                        );
                    }
                } else if !self.reveal_smoke_control(ctx, target) {
                    return (false, None);
                }
                let tolerance = 1. / ctx.pixels_per_point();
                if bottom {
                    assert!(
                        target.bottom() + target.height() <= rows.clip.bottom() + tolerance,
                        "The final task row needs a full observed text-row clearance above edge scrollbars"
                    );
                }
                assert!(
                    rows.clip.expand(tolerance).contains_rect(target),
                    "Target task row must actually paint inside its clip"
                );
                let key = egui::Id::new(("loop-goal-proof", setting));
                let mut proof = if !top {
                    ctx.data(|d| d.get_temp::<GoalProof>(key))
                        .expect("Top slice was actually captured first")
                } else {
                    GoalProof {
                        seen: vec![false; rows.rows.len()],
                    }
                };
                assert_eq!(
                    proof.seen.len(),
                    rows.rows.len(),
                    "Task wraps consistently within this settled setting"
                );
                for (seen, rect) in proof.seen.iter_mut().zip(&rows.rows) {
                    *seen |= rows.clip.expand(tolerance).contains_rect(*rect);
                }
                eprintln!(
                    "loop-recovery actual goal rows {label}: visible_union={}/{} clip={:?}",
                    proof.seen.iter().filter(|x| **x).count(),
                    proof.seen.len(),
                    rows.clip
                );
                if bottom && proof.seen.iter().any(|seen| !*seen) {
                    eprintln!(
                        "goal-union diagnostic {label}: missing={:?} current_visible={:?} rows={:?} clip={:?} offset={:?}",
                        proof
                            .seen
                            .iter()
                            .enumerate()
                            .filter_map(|(i, seen)| (!*seen).then_some(i))
                            .collect::<Vec<_>>(),
                        rows.rows
                            .iter()
                            .enumerate()
                            .filter_map(|(i, rect)| rows
                                .clip
                                .expand(tolerance)
                                .contains_rect(*rect)
                                .then_some(i))
                            .collect::<Vec<_>>(),
                        rows.rows,
                        rows.clip,
                        self.metrics.main_scroll_offset
                    );
                }
                if bottom {
                    assert!(
                        proof.seen.iter().all(|seen| *seen),
                        "Every essential task row must actually appear inside one of the four captured slices"
                    );
                }
                ctx.data_mut(|d| d.insert_temp(key, proof));
                shot = Some(self.loop_recovery_shot(ctx, id, &label));
                if phase == 180 {
                    self.smoke_stage = BASE + n * STRIDE + 54;
                }
            }
            200 => {
                if !self.loop_recovery_undo(ctx, i, reference) {
                    return (false, None);
                }
                shot =
                    Some(self.loop_recovery_shot(ctx, id, "real-undo-redo-cursor-preserves-draft"));
            }
            201 => self.loop_recovery_key(egui::Key::Tab, Default::default()),
            202 => {
                let engine::State::Script { source, run } = &self.progress.attempts[id].state
                else {
                    panic!()
                };
                assert_eq!(source, &format!("{reference}\t"));
                assert!(run.is_none());
                self.loop_recovery_key(
                    egui::Key::Tab,
                    egui::Modifiers {
                        shift: true,
                        ..Default::default()
                    },
                );
            }
            203 => {
                let engine::State::Script { source, run } = &self.progress.attempts[id].state
                else {
                    panic!()
                };
                assert_eq!(source, reference);
                assert!(run.is_none());
                assert!(
                    self.metrics
                        .focus
                        .is_some_and(|f| ctx.memory(|m| m.has_focus(f)))
                );
            }
            204 => {
                if !self.loop_recovery_run(ctx, i, stage) {
                    return (false, None);
                }
            }
            205 => {
                if !self.case_lesson_wait(ctx) {
                    return (false, None);
                }
                self.case_lesson_assignment(i, Some(reference), true);
            }
            206 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_challenge) {
                    return (false, None);
                }
            }
            207 => {
                assert!(self.progress.attempts[id].challenge);
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                assert!(
                    !self.metrics.help_visible
                        && !self.metrics.guide
                        && self.metrics.lesson_tabs.is_empty()
                );
            }
            208 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.help) {
                    return (false, None);
                }
            }
            209 => {
                assert!(self.metrics.help_visible && !self.metrics.guide);
                assert!(self.progress.attempts[id].challenge);
                shot = Some(self.loop_recovery_shot(ctx, id, "challenge-guidance-only-on-request"));
            }
            210 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.help) {
                    return (false, None);
                }
            }
            211 => {
                assert!(!self.metrics.help_visible);
            }
            212 => {
                if !self.click_visible_smoke_control(ctx, self.metrics.mode_learn) {
                    return (false, None);
                }
            }
            213 => {
                assert!(!self.progress.attempts[id].challenge);
                assert_eq!(self.script_view(i), teaching::View::Assignment);
                assert!(!self.metrics.help_visible);
                self.smoke_stage = BASE + n * STRIDE + 28;
            }
            214 | 215 => unreachable!("Closed unused reserved phases"),
            216 => {
                let passed = row
                    .wrong_expected
                    .iter()
                    .position(|p| *p)
                    .expect("Hardcoded control intentionally passes first case");
                let target = self
                    .metrics
                    .script_case_rows
                    .iter()
                    .find(|(n, _)| *n == passed)
                    .map(|(_, r)| *r);
                if !self.click_visible_smoke_control(ctx, target) {
                    return (false, None);
                }
            }
            217 => {
                let passed = row.wrong_expected.iter().position(|p| *p).unwrap();
                assert_eq!(self.script_case, passed);
                assert!(!self.metrics.completion);
                let (selected, pass, rect) = self
                    .metrics
                    .script_case_status
                    .expect("Actual selected case status");
                assert_eq!(selected, passed);
                assert!(pass);
                assert!(!progress::current_solved(
                    &self.progress.attempts[id],
                    &scenario
                ));
                if !self.reveal_smoke_control(ctx, rect) {
                    return (false, None);
                }
                shot =
                    Some(self.loop_recovery_shot(ctx, id, "selected-pass-under-overall-failure"));
            }
            218 => {
                self.smoke_stage = BASE + n * STRIDE + 20;
            }
            104 => {
                if n == 1 {
                    if self.capture_pending {
                        return (false, None);
                    }
                    let finish = egui::Id::new("loop-recovery-finish-held");
                    if ctx.data(|d| d.get_temp::<bool>(finish)) == Some(true) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return (false, None);
                    }
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
                        "PASS normal two loop-recovery native flow: twenty observed scale/theme/width settings on the representative intake workspace, actualStart/Intro/example/experiment/repair/assignment, wrong+partial+reference+twoalternatives, typing/Challenge Light150 Dark200 actualgeometry, syntax+unsupported+limit, realRunStop, fullraw/portable/receipt warmContinue without loadedProgress installation, resetCancel/Confirm and2normalcards. No cold-process, novice/accessibility/otherOS claim."
                    );
                    ctx.data_mut(|d| d.insert_temp(finish, true));
                    ctx.request_repaint();
                    return (false, None);
                }
                self.smoke_stage = BASE + (n + 1) * STRIDE - 1;
            }
            _ => unreachable!("Closed native loop-recovery phase"),
        }
        (true, shot)
    }
}

pub(in crate::ui) fn immediate_stop(stage: usize) -> bool {
    (BASE..=BASE + STRIDE + LAST).contains(&stage) && matches!((stage - BASE) % STRIDE, 66..=68)
}
impl LabApp {
    pub(in crate::ui) fn loop_recovery_lessons_native(
        &mut self,
        ctx: &egui::Context,
        stage: usize,
    ) -> (bool, Option<String>) {
        const {
            assert!((LAST + 1) * 2 <= 1000);
        }
        let before = self.pending_events.len();
        let result = self.loop_recovery_inner(ctx, stage);
        let added = self.pending_events.len().saturating_sub(before);
        let idle = !result.0
            && added == 0
            && self.script_job.is_none()
            && !self.capture_pending
            && (stage - BASE) % STRIDE != 200;
        ctx.data_mut(|d| {
            let key = egui::Id::new(("loop-recovery-budgets", stage));
            let (queued, waits) = d.get_temp::<(usize, usize)>(key).unwrap_or_default();
            let queued = queued + added;
            let waits = if idle { waits + 1 } else { 0 };
            assert!(queued <= 300, "Closed per-step input queue");
            assert!(waits <= 60, "Closed idle step wait");
            d.insert_temp(key, (queued, waits));
            if result.1.is_some() {
                let key = egui::Id::new("loop-recovery-capture-count");
                let n = d.get_temp::<usize>(key).unwrap_or(0) + 1;
                assert!(n <= 128);
                d.insert_temp(key, n);
            }
        });
        result
    }
}
