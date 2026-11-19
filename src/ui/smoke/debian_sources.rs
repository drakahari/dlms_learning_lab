//! Fixed native input flow, private review profile; no learner-host execution.
use super::*;
use dlms_learning_lab::{debian_source_cache as cache, debian_sources_workspace as work};
#[derive(Clone, Debug)]
enum Op {
    Open(String),
    View(usize),
    Type(bool, String),
    Click(String),
    Query(Option<String>),
    Indexes(bool),
    Check(bool, bool),
    Save,
    Shot(String, bool),
    Reset,
    Confirm,
    Fresh,
}
#[derive(Clone, Default)]
struct Route {
    ops: Vec<Op>,
    at: usize,
    phase: u8,
    frames: usize,
    after: f64,
    began: Option<f64>,
    done: bool,
}
fn click(s: &str) -> Op {
    Op::Click(s.into())
}
fn lookup(v: &mut Vec<Op>, name: &str, version: Option<&str>) {
    v.extend([
        Op::Type(false, name.into()),
        click("lookup"),
        Op::Query(version.map(str::to_owned)),
    ]);
}
fn scope(a: &LabApp) -> &work::Scope {
    let id = &a.scenarios[a.selected.unwrap()].id;
    let engine::State::DebianSources { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    }
}
fn definition(a: &LabApp) -> &work::Definition {
    let engine::Goal::DebianSources { definition } = &a.scenarios[a.selected.unwrap()].goal else {
        panic!()
    };
    definition
}
fn control(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .debian_sources
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing source control {key}"))
        .1
}
fn route(a: &LabApp) -> Vec<Op> {
    let mut ops = vec![];
    for id in [
        "lpic1-package-source-selection",
        "lpic1-package-source-recovery",
    ] {
        let recovery = id.ends_with("recovery");
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        let engine::Goal::DebianSources { definition: d } = &s.goal else {
            panic!()
        };
        let example = &d.example_worked.sources;
        let answer = &d.assignment_reference.sources;
        let before = if recovery { Some("1.0-1") } else { None };
        let after = if recovery { "2.0-1" } else { "2.1-1" };
        let p = if recovery {
            "arts-helper"
        } else {
            "gallery-search"
        };
        let q = if recovery {
            "campus-helper"
        } else {
            "desk-indexer"
        };
        ops.extend([
            Op::Open(id.into()),
            Op::View(1),
            Op::Type(true, example.clone()),
            click("save"),
        ]);
        lookup(&mut ops, p, before);
        ops.extend([click("refresh"), Op::Indexes(recovery)]);
        lookup(&mut ops, p, Some(after));
        ops.extend([
            Op::Check(true, false),
            Op::Shot(format!("sources-{id}-practice-passed"), false),
        ]);
        ops.extend([
            Op::Type(
                true,
                example.replace("Components: main tools", "Components: main"),
            ),
            click("save"),
            Op::Check(false, true),
            Op::Type(true, example.clone()),
            click("save"),
            Op::Check(true, false),
            Op::View(2),
            Op::Check(false, false),
        ]);
        ops.extend([Op::Type(true, answer.clone()), click("save")]);
        lookup(&mut ops, q, before);
        ops.extend([click("refresh"), Op::Indexes(recovery)]);
        lookup(&mut ops, q, Some(after));
        ops.push(Op::Shot(format!("sources-{id}-actual-query"), true));
        ops.extend([Op::Check(true, false), Op::Save]);
        let wrong = if recovery {
            format!(
                "{answer}\n{}",
                answer
                    .replace("Suites: campus", "Suites: retired")
                    .replace("Components: main tools", "Components: main")
            )
        } else {
            answer.replace("Suites: studio\n", "Suites: studio studio-updates\n")
        };
        let retired = d
            .initial
            .starter
            .saved
            .split("\n\n")
            .nth(if recovery { 1 } else { 0 })
            .unwrap()
            .trim_end();
        let alternate = format!("{answer}\n{retired}\nEnabled: no\n");
        ops.extend([
            Op::Type(true, wrong),
            click("save"),
            Op::Check(false, true),
            Op::Shot(format!("sources-{id}-mixed-feedback"), false),
            Op::Type(true, alternate),
            click("save"),
            Op::Check(true, false),
            Op::Reset,
            Op::Confirm,
            Op::Fresh,
        ]);
    }
    ops
}
impl LabApp {
    pub(in crate::ui) fn debian_sources_tick(&mut self, ctx: &egui::Context) {
        if ctx.current_pass_index() != 0 {
            return;
        }
        let root = ctx.cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let once = egui::Id::new("debian-sources-root-frame");
        if ctx.data(|d| d.get_temp::<u64>(once)) == Some(root) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(once, root));
        let key = egui::Id::new("debian-sources-native-route");
        let mut r = ctx
            .data_mut(|d| d.get_temp::<Route>(key))
            .unwrap_or_else(|| Route {
                ops: route(self),
                ..Default::default()
            });
        if r.done {
            self.teaching_preview_tick(ctx);
            return;
        }
        if r.at == r.ops.len() {
            println!(
                "DEBIAN_SOURCES_NATIVE_FLOW_COMPLETE: {} actual actions, separate practice, metadata provenance, partial refresh, mixed results, retry, save-read and selected reset; no host APT",
                r.at
            );
            r.done = true;
            ctx.data_mut(|d| d.insert_temp(key, r));
            return;
        }
        self.smoke_stage = 100000 + r.at;
        let op = r.ops[r.at].clone();
        let now = ctx.input(|i| i.time);
        let began = *r.began.get_or_insert(now);
        assert!(
            now - began < 15.,
            "Source native action exceeded15seconds: {op:?}; phase={} offset={:?}",
            r.phase,
            self.metrics.main_scroll_offset
        );
        if now < r.after {
            ctx.data_mut(|d| d.insert_temp(key, r));
            ctx.request_repaint_after(std::time::Duration::from_millis(10));
            return;
        }
        if r.frames == 0 {
            println!("DEBIAN_SOURCES_NATIVE_STEP {} {:?}", r.at, op);
        }
        r.frames += 1;
        assert!(
            r.frames < 1200,
            "Source native interaction budget exhausted: {op:?}"
        );
        r.after = now + 0.01;
        if r.frames == 1
            && matches!(op, Op::Type(true, _))
            && std::env::var_os("DLMS_SMOKE_DIAGNOSTICS").is_some()
        {
            self.capture_pending = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                format!("sources-editor-geometry-{}", r.at),
            )));
        }
        let complete = match op {
            Op::Open(id) => {
                let i = self.scenarios.iter().position(|s| s.id == id).unwrap();
                self.launch(i);
                self.lesson_scroll_reset = true;
                true
            }
            Op::View(n) => {
                self.click_visible_smoke_control(ctx, self.metrics.lesson_tabs.get(n).copied())
            }
            Op::Type(editor, text) => {
                let (rect, id) = if editor {
                    self.metrics.debian_sources.editor.expect("source editor")
                } else {
                    self.metrics.debian_sources.query.expect("query field")
                };
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, Some(rect)) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                    let modifiers = egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    };
                    for pressed in [true, false] {
                        self.pending_events.push(egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed,
                            repeat: false,
                            modifiers,
                        });
                    }
                    self.pending_events.push(egui::Event::Paste(text));
                    r.phase = 2;
                    false
                } else {
                    assert_eq!(
                        if editor {
                            &scope(self).draft
                        } else {
                            &scope(self).query
                        },
                        &text,
                        "Actual focused paste must reach the intended draft"
                    );
                    true
                }
            }
            Op::Click(key) => self.click_visible_smoke_control(ctx, Some(control(self, &key))),
            Op::Query(version) => {
                let w = scope(self);
                assert_eq!(w.observed.as_ref(), Some(&w.query));
                let d = definition(self);
                let t = if w.origin == d.example.id {
                    &d.example
                } else {
                    &d.initial
                };
                let rows = w.state.query(&t.model, &w.query).unwrap();
                if let Some(v) = version {
                    assert_eq!(rows.len(), 1);
                    assert_eq!(rows[0].row.version.as_str(), v);
                } else {
                    assert!(rows.is_empty());
                }
                true
            }
            Op::Indexes(partial) => {
                let d = definition(self);
                let w = scope(self);
                let t = if w.origin == d.example.id {
                    &d.example
                } else {
                    &d.initial
                };
                let rows = w.state.indexes(&t.model).unwrap();
                assert_eq!(
                    rows.iter()
                        .filter(|r| r.status == cache::IndexStatus::Retained)
                        .count(),
                    usize::from(partial)
                );
                assert!(rows.iter().all(|r| r.status != cache::IndexStatus::Missing));
                if partial {
                    assert!(self.notice.as_deref().unwrap().contains("Partly refreshed"));
                }
                true
            }
            Op::Check(expected, mixed) => {
                if r.phase == 0 {
                    if self.click_visible_smoke_control(ctx, Some(control(self, "check"))) {
                        r.phase = 1;
                    }
                    false
                } else {
                    let d = definition(self);
                    let w = scope(self);
                    assert!(w.checked);
                    let practice = w.origin == d.example.id;
                    let t = if practice { &d.example } else { &d.initial };
                    let results = t.checks(w);
                    assert_eq!(results.iter().all(|r| r.0), expected);
                    if mixed {
                        assert!(results.iter().any(|r| r.0) && results.iter().any(|r| !r.0));
                    }
                    let source = &self.scenarios[self.selected.unwrap()];
                    assert_eq!(
                        progress::current_solved(&self.progress.attempts[&source.id], source),
                        !practice && expected
                    );
                    true
                }
            }
            Op::Save => {
                self.save();
                assert!(self.storage_error.is_none());
                let restored = progress::load(&self.path, &self.scenarios).unwrap();
                assert_eq!(
                    serde_json::to_value(restored).unwrap(),
                    serde_json::to_value(&self.progress).unwrap()
                );
                true
            }
            Op::Shot(name, query) => {
                if r.phase == 0 {
                    let rect = if query {
                        self.metrics.debian_sources.observation.unwrap()
                    } else {
                        self.metrics.debian_sources.report.unwrap()
                    };
                    if self.reveal_smoke_control(ctx, rect) {
                        r.phase = 1;
                    }
                    false
                } else if r.phase == 1 {
                    self.capture_pending = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
                        name,
                    )));
                    r.phase = 2;
                    false
                } else {
                    !self.capture_pending
                }
            }
            Op::Reset => self.click_visible_smoke_control(ctx, self.metrics.reset),
            Op::Confirm => self.click_visible_smoke_control(ctx, self.metrics.confirm_reset),
            Op::Fresh => {
                let s = &self.scenarios[self.selected.unwrap()];
                assert_eq!(
                    serde_json::to_value(&self.progress.attempts[&s.id].state).unwrap(),
                    serde_json::to_value(&s.initial).unwrap()
                );
                assert!(!progress::current_solved(&self.progress.attempts[&s.id], s));
                true
            }
        };
        if self
            .pending_events
            .iter()
            .any(|e| matches!(e, egui::Event::MouseWheel { .. }))
        {
            eprintln!(
                "SOURCES_WHEEL root={root} time={now:.6} step={} phase={} editor={:?} viewport={:?} offset={:?} content={:?}",
                r.at,
                r.phase,
                self.metrics.debian_sources.editor,
                self.metrics.main_viewport,
                self.metrics.main_scroll_offset,
                self.metrics.main_content_size
            );
            r.after = now + 0.15;
        }
        if complete {
            r.at += 1;
            r.phase = 0;
            r.frames = 0;
            r.began = None;
            self.started = Instant::now();
        }
        ctx.data_mut(|d| d.insert_temp(key, r));
        ctx.request_repaint_after(std::time::Duration::from_millis(10));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_driver_delivers_actual_focus_paste_save_refresh_and_query_once_per_root_frame() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = true;
        let id = "lpic1-package-source-selection";
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        let engine::Goal::DebianSources { definition: d } = &s.goal else {
            panic!()
        };
        let text = d.assignment_reference.sources.clone();
        let ops = vec![
            Op::Open(id.into()),
            Op::View(2),
            Op::Type(true, text),
            click("save"),
            click("refresh"),
            Op::Type(false, "desk-indexer".into()),
            click("lookup"),
            Op::Query(Some("2.1-1".into())),
            Op::Check(true, false),
        ];
        let len = ops.len();
        let key = egui::Id::new("debian-sources-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops,
                    ..Default::default()
                },
            )
        });
        for n in 0..3000 {
            let mut input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1268., 632.),
                )),
                time: Some(n as f64 / 100.),
                ..Default::default()
            };
            eframe::App::raw_input_hook(&mut a, &ctx, &mut input);
            let _ = ctx.run(input, |ctx| {
                a.render(ctx);
                a.debian_sources_tick(ctx);
                let before = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
                let queued = a.pending_events.len();
                a.debian_sources_tick(ctx);
                let after = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
                assert_eq!(
                    (before.at, before.phase, before.frames, queued),
                    (after.at, after.phase, after.frames, a.pending_events.len())
                );
                if ctx.current_pass_index() == 0 {
                    ctx.request_discard(
                        "Source driver regression: actual input delivered on the next root frame",
                    );
                }
            });
            if ctx.data(|d| d.get_temp::<Route>(key)).unwrap().at == len {
                assert!(progress::current_solved(
                    &a.progress.attempts[id],
                    &a.scenarios[a.selected.unwrap()]
                ));
                return;
            }
        }
        panic!("Actual source input controller must finish without bypassing visibility");
    }
    #[test]
    fn source_capture_phase_requests_the_real_viewport_without_context_data_reentry() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("debian-sources-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Shot("source-capture-regression".into(), false)],
                    phase: 1,
                    ..Default::default()
                },
            )
        });
        let out = ctx.run(egui::RawInput::default(), |ctx| a.debian_sources_tick(ctx));
        assert!(a.capture_pending);
        assert!(
            out.viewport_output
                .values()
                .flat_map(|v| &v.commands)
                .any(|c| matches!(c, egui::ViewportCommand::Screenshot(_)))
        );
    }
    #[test]
    fn source_timed_wait_cannot_consume_interaction_attempts_at_very_fast_render_rates() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("debian-sources-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Open("lpic1-package-source-selection".into())],
                    frames: 1199,
                    after: 0.15,
                    began: Some(0.),
                    ..Default::default()
                },
            )
        });
        for n in 0..1500 {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(n as f64 / 100000.),
                    ..Default::default()
                },
                |ctx| a.debian_sources_tick(ctx),
            );
            let r = ctx.data(|d| d.get_temp::<Route>(key)).unwrap();
            assert_eq!((r.at, r.frames), (0, 1199));
        }
    }
    #[test]
    #[should_panic(expected = "exceeded15seconds")]
    fn source_timed_wait_still_rejects_a_stalled_action_at_its_elapsed_deadline() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        let key = egui::Id::new("debian-sources-native-route");
        ctx.data_mut(|d| {
            d.insert_temp(
                key,
                Route {
                    ops: vec![Op::Open("lpic1-package-source-selection".into())],
                    after: 30.,
                    began: Some(0.),
                    ..Default::default()
                },
            )
        });
        let _ = ctx.run(
            egui::RawInput {
                time: Some(16.),
                ..Default::default()
            },
            |ctx| a.debian_sources_tick(ctx),
        );
    }
}
