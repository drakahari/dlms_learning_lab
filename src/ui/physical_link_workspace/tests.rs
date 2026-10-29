//! Prospective actual-widget gates. No native/render/runtime acceptance is claimed.
use super::*;
use crate::ui::tests::physical_workspace as fixture;
struct Harness {
    ctx: egui::Context,
    s: Scenario,
    a: Attempt,
    p: Presentation,
    m: Metrics,
    width: f32,
    height: f32,
    scale: f32,
    time: f64,
    active: bool,
}
impl Harness {
    fn new() -> Self {
        let s = fixture::source();
        let mut a = Attempt::new(&s, 1);
        work::prepare_attempt(&mut a, &s, true).unwrap();
        let mut h = Self {
            ctx: egui::Context::default(),
            s,
            a,
            p: Presentation::default(),
            m: Metrics::default(),
            width: 1600.,
            height: 1200.,
            scale: 1.,
            time: 0.,
            active: true,
        };
        h.idle(3);
        h
    }
    fn frame(&mut self, events: Vec<egui::Event>) {
        self.time += 0.1;
        self.ctx.set_zoom_factor(self.scale);
        self.m = Metrics::default();
        let focus = self.ctx.memory(|m| m.focused());
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(self.width / self.scale, self.height / self.scale),
            )),
            time: Some(self.time),
            events,
            modifiers,
            ..Default::default()
        };
        let _ = self.ctx.run(input, |ctx| {
            controls::configure(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("physical-widget-gate")
                    .show(ui, |ui| {
                        page(
                            ui,
                            &self.s,
                            &mut self.a,
                            &mut self.p,
                            &mut self.m,
                            self.active,
                            focus,
                        );
                    });
            });
        });
    }
    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }
    fn tap(&mut self, r: egui::Rect) {
        for pressed in [true, false] {
            self.frame(vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ]);
        }
        self.idle(2);
    }
    fn wheel(&mut self, delta: f32) {
        self.frame(vec![
            egui::Event::PointerMoved(egui::pos2(60., 60.)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta),
                modifiers: Default::default(),
            },
        ]);
        self.idle(2);
    }
    fn reveal(&mut self, key: &str) -> egui::Rect {
        for _ in 0..80 {
            if let Some((_, r, _)) = self.m.controls.iter().find(|(id, _, _)| id == key) {
                let r = *r;
                if r.top() >= 10. && r.bottom() < self.height / self.scale - 12. {
                    return r;
                }
                self.wheel(if r.top() < 10. { 200. } else { -200. });
            } else {
                self.wheel(-200.);
            }
        }
        panic!("Native control {key} was not reachable through scrolling");
    }
    fn click(&mut self, key: &str) {
        let r = self.reveal(key);
        self.tap(r);
    }
    fn tab(&mut self, view: View) {
        for _ in 0..35 {
            if let Some((_, r)) = self.m.tabs.iter().find(|(v, _)| *v == view) {
                let r = *r;
                if r.top() >= 10. {
                    self.tap(r);
                    return;
                }
            }
            self.wheel(260.);
        }
        panic!("Visible assistance tab was unreachable");
    }
    fn choose(&mut self, key: &str, value: &str) {
        let r = self.reveal(key);
        self.tap(r);
        let r = self
            .m
            .choices
            .iter()
            .find(|(id, v, _, enabled)| id == key && v == value && *enabled)
            .unwrap_or_else(|| panic!("No actual enabled option {key}/{value}"))
            .2;
        self.tap(r);
    }
    fn applied(&self) -> &core::State {
        scope_state(&self.a, false)
    }
    fn assignment(&mut self) {
        self.tab(View::Assignment);
    }
    fn reference(&mut self) {
        self.choose("cable", "assignment-clean");
        self.choose("mode:assignment-right", "Auto");
        self.click("apply");
        self.click("check");
        assert!(self.m.complete);
    }
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn select_all() -> egui::Event {
    key(
        egui::Key::A,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    )
}
#[test]
fn physical_native_intro_example_fill_apply_and_assignment_drafts_are_isolated() {
    let mut h = Harness::new();
    assert!(h.m.controls.iter().any(|(id, _, _)| id == "try-example"));
    let state = h.a.state.clone();
    let draft = scope_draft(&h.a, false).clone();
    h.click("try-example");
    let before = scope_state(&h.a, true).clone();
    h.click("fill-example");
    assert_eq!(scope_state(&h.a, true), &before);
    assert_eq!(h.a.state, state);
    assert_eq!(scope_draft(&h.a, false), &draft);
    assert!(!h.m.complete);
    h.click("apply");
    h.click("check");
    assert!(work::practice_checks(&h.a, &h.s).iter().all(|r| r.0));
    assert!(!h.m.complete);
    let saved = scope_state(&h.a, true).clone();
    h.assignment();
    assert_eq!(h.a.state, state);
    assert_eq!(scope_draft(&h.a, false), &draft);
    h.tab(View::Example);
    assert_eq!(scope_state(&h.a, true), &saved);
}
#[test]
fn physical_native_wrong_partial_reference_and_edit_invalidation_use_real_configuration() {
    let mut h = Harness::new();
    h.assignment();
    let neighbor = h.applied().configuration().ports["assignment-ops-left"].clone();
    h.click("check");
    assert!(!h.m.complete && h.m.rows.iter().any(|(_, ok, _)| !*ok));
    h.choose("mode:assignment-right", "Auto");
    h.click("apply");
    h.click("check");
    assert!(
        !h.m.complete,
        "A compatible mode does not repair the impaired cable"
    );
    h.choose("cable", "assignment-clean");
    h.click("apply");
    h.click("check");
    assert!(h.m.complete);
    assert_eq!(
        h.applied().configuration().ports["assignment-ops-left"],
        neighbor
    );
    h.click("up:assignment-left");
    assert!(!h.m.complete);
    assert!(!checked(&h.a, false));
    assert!(
        h.applied().configuration().ports["assignment-left"].admin_up,
        "Pending draft is distinct from applied state"
    );
    h.click("apply");
    h.click("check");
    assert!(!h.m.complete && !h.applied().configuration().ports["assignment-left"].admin_up);
}
#[test]
fn physical_native_supported_fixed_mode_alternative_is_not_an_exact_solution_match() {
    let mut h = Harness::new();
    h.assignment();
    h.choose("cable", "assignment-clean");
    h.choose("mode:assignment-left", "Fixed");
    h.click("apply");
    h.click("check");
    assert!(h.m.complete);
    for id in ["assignment-left", "assignment-right"] {
        assert!(matches!(
            h.applied().configuration().ports[id].negotiation,
            core::Negotiation::Fixed {
                ability: core::Ability {
                    rate: core::Rate::M100,
                    duplex: core::Duplex::Full
                }
            }
        ));
    }
}
#[test]
fn physical_native_numeric_edit_tab_and_rejected_same_frame_sample_preserve_undo_and_state() {
    let mut h = Harness::new();
    h.assignment();
    let r = h.reveal("probe");
    let (field, _, id) = *h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap();
    assert_eq!(field, 0);
    let field = h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap().1;
    h.tap(field);
    h.frame(vec![select_all(), egui::Event::Paste("24".into())]);
    h.idle(12);
    assert_eq!(h.p.numbers[0], "24");
    h.frame(vec![key(egui::Key::Tab, Default::default())]);
    h.idle(1);
    assert_eq!(
        h.ctx.memory(|m| m.focused()),
        Some(h.m.numbers.iter().find(|(n, _, _)| *n == 1).unwrap().2)
    );
    h.frame(vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)]);
    h.idle(1);
    assert_eq!(h.ctx.memory(|m| m.focused()), Some(id));
    let before = serde_json::to_vec(&h.a).unwrap();
    h.frame(vec![
        select_all(),
        egui::Event::Paste("oops".into()),
        egui::Event::PointerMoved(r.center()),
        egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    h.frame(vec![egui::Event::PointerButton {
        pos: r.center(),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    }]);
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
    assert_eq!(h.p.numbers[0], "24");
    h.tap(field);
    assert_eq!(h.ctx.memory(|m| m.focused()), Some(id));
    let retained = egui::text_edit::TextEditState::load(&h.ctx, id)
        .unwrap()
        .cursor
        .char_range()
        .unwrap();
    assert!(!retained.is_empty());
    assert_eq!((retained.primary.index, retained.secondary.index), (2, 0));
    // The rejected selection was correctly restored. Native egui records
    // cursor+text, so collapse that selection with a real key before text Undo.
    h.frame(vec![key(egui::Key::ArrowRight, Default::default())]);
    h.idle(1);
    assert!(
        egui::text_edit::TextEditState::load(&h.ctx, id)
            .unwrap()
            .cursor
            .char_range()
            .unwrap()
            .is_empty()
    );
    assert_eq!(h.p.numbers[0], "24");
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
    h.frame(vec![key(
        egui::Key::Z,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    )]);
    h.idle(1);
    assert_eq!(h.p.numbers[0], "12");
}
#[test]
fn physical_native_dropped_enter_then_paste_vetoes_sample_and_restores_native_selection() {
    let mut h = Harness::new();
    h.assignment();
    let probe = h.reveal("probe");
    let field = h.m.numbers[0].1;
    h.tap(field);
    let before = serde_json::to_vec(&h.a).unwrap();
    h.frame(vec![
        select_all(),
        egui::Event::Cut,
        key(egui::Key::Enter, Default::default()),
        egui::Event::Paste("7".into()),
        egui::Event::PointerMoved(probe.center()),
        egui::Event::PointerButton {
            pos: probe.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    h.frame(vec![egui::Event::PointerButton {
        pos: probe.center(),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    }]);
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
    assert_eq!(h.p.numbers[0], "12");
    h.tap(field);
    h.frame(vec![select_all(), egui::Event::Paste("7".into())]);
    h.click("probe");
    assert!(scope_state(&h.a, false).counters()["assignment-left"].offered > 0);
}
#[test]
fn physical_native_frontier_probe_and_inactive_modal_refuse_without_changing_attempt() {
    let mut h = Harness::new();
    h.assignment();
    h.a.transcript = vec![String::new(); 120];
    let n = work::MAX_ATTEMPT - work::size(&h.a, work::MAX_ATTEMPT).unwrap();
    for (i, line) in h.a.transcript.iter_mut().enumerate() {
        *line = "x".repeat(n / 120 + usize::from(i < n % 120));
    }
    assert_eq!(work::size(&h.a, work::MAX_ATTEMPT), Some(work::MAX_ATTEMPT));
    work::context(&h.a, &h.s).unwrap();
    h.idle(2);
    let before = serde_json::to_vec(&h.a).unwrap();
    h.click("probe");
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
    assert!(h.p.notice.is_some());
    h.active = false;
    h.click("apply");
    h.click("check");
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
}
#[test]
fn physical_native_reset_example_resume_and_challenge_keep_the_full_assignment() {
    let mut h = Harness::new();
    h.assignment();
    h.reference();
    let assignment = h.a.state.clone();
    h.tab(View::Example);
    h.click("fill-example");
    h.click("apply");
    h.click("reset-example");
    assert_eq!(h.a.state, assignment);
    assert_eq!(
        scope_state(&h.a, true),
        &h.s.physical_link_teaching
            .as_ref()
            .unwrap()
            .practice
            .initial
    );
    let wire = serde_json::to_vec(&h.a).unwrap();
    let resumed: Attempt = serde_json::from_slice(&wire).unwrap();
    work::context(&resumed, &h.s).unwrap();
    h.a = resumed;
    h.p = Presentation::default();
    work::record_mode(&mut h.a, &h.s, true).unwrap();
    h.idle(3);
    assert!(h.m.tabs.is_empty());
    assert!(h.m.controls.iter().any(|(id, _, _)| id == "apply"));
    assert!(h.m.controls.iter().any(|(id, _, _)| id == "check"));
    assert_eq!(h.a.state, assignment);
}
#[test]
fn physical_native_controls_reflow_at_all_five_sizes_without_shrinking_text() {
    for width in [946., 1920.] {
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for dark in [false, true] {
                let mut h = Harness::new();
                h.assignment();
                h.width = width;
                h.height = 1080.;
                h.scale = scale;
                h.ctx.set_visuals(if dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                });
                h.idle(4);
                let mut seen = std::collections::BTreeSet::new();
                for _ in 0..30 {
                    let content = h.ctx.content_rect();
                    for (key, r, _) in &h.m.controls {
                        assert!(
                            r.left() >= content.left() - 0.5 && r.right() <= content.right() + 0.5,
                            "{width}/{scale}/{key}: {r:?} outside {content:?}"
                        );
                        if r.intersects(content) {
                            seen.insert(key.clone());
                        }
                    }
                    h.wheel(-160.);
                }
                assert!(seen.contains("apply") && seen.contains("check") && seen.contains("probe"));
            }
        }
    }
}
#[test]
fn physical_native_more_than_four_attempts_do_not_trigger_idle_regrade_or_false_completion() {
    let s = fixture::source();
    let mut sources = Vec::new();
    let mut memo = memo::Memo::default();
    for n in 0..7 {
        let mut source = s.clone();
        source.id = format!("test-physical-page-{n}");
        let mut a = Attempt::new(&source, 1);
        work::prepare_attempt(&mut a, &source, true).unwrap();
        if n == 0 {
            let core::Action::Configure { configuration } = core::Action::from_json(
                source.solution[0]
                    .strip_prefix(work::PREFIX)
                    .unwrap()
                    .as_bytes(),
            )
            .unwrap() else {
                panic!("Typed reference")
            };
            let mut draft = scope_draft(&a, false).clone();
            draft.configuration = configuration;
            work::set_draft(&mut a, &source, work::Scope::Assignment, &draft).unwrap();
            work::apply(&mut a, &source, work::Scope::Assignment).unwrap();
        }
        memo.prepare(&a, &source, false).unwrap();
        if n == 0 {
            assert_eq!(memo.passed(&a, &source), Some(true));
        }
        sources.push((source, a));
    }
    let (last, a) = sources.last().unwrap();
    let before = memo.counters;
    for _ in 0..60 {
        memo.prepare(a, last, false).unwrap();
        for (s, a) in &sources {
            let _status = memo.passed(a, s);
        }
    }
    assert_eq!(memo.counters.admissions, before.admissions);
    assert_eq!(memo.counters.copies, before.copies);
    assert_eq!(memo.counters.grades, before.grades);
    assert_eq!(
        memo.passed(&sources[0].1, &sources[0].0),
        Some(true),
        "The exact certified completion survives renderer-snapshot eviction"
    );
    assert!(memo.rows(a, last, false).is_some());
}
#[test]
fn physical_native_warm_source_changes_and_valid_foreign_metadata_never_reuse_old_success() {
    let s = fixture::source();
    let mut a = Attempt::new(&s, 1);
    work::prepare_attempt(&mut a, &s, true).unwrap();
    let mut memo = memo::Memo::default();
    memo.prepare(&a, &s, false).unwrap();
    let before = memo.counters;
    let mut altered = s.clone();
    altered
        .sources
        .push("https://www.cisco.com/changed-source".into());
    assert_eq!(memo.passed(&a, &altered), None);
    memo.prepare(&a, &altered, false).unwrap();
    assert_eq!(memo.counters.copies, before.copies + 1);
    let source = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|s| s.cron_teaching.is_some())
        .unwrap();
    a.workspace.as_mut().unwrap().cron_learning = Some(
        dlms_learning_lab::cron_teaching::Learning::new(source.cron_teaching.as_ref().unwrap())
            .unwrap(),
    );
    let before = memo.counters;
    let wire = serde_json::to_vec(&a).unwrap();
    assert!(memo.prepare(&a, &s, false).is_err());
    assert_eq!(memo.counters.copies, before.copies);
    assert_eq!(serde_json::to_vec(&a).unwrap(), wire);
    assert_eq!(memo.passed(&a, &s), None);
}
#[test]
fn physical_native_details_click_exposes_every_unchanged_grading_row_once() {
    let mut h = Harness::new();
    h.assignment();
    h.click("check");
    for _ in 0..40 {
        let r = h.m.details_header.unwrap();
        if r.top() >= 10. && r.bottom() < h.height - 10. {
            h.tap(r);
            break;
        }
        h.wheel(if r.top() < 10. { 200. } else { -200. });
    }
    h.idle(3);
    let expected = work::checks(&h.a, &h.s);
    assert_eq!(h.m.detail_rows.len(), expected.len());
    for (n, (at, passed, _)) in h.m.detail_rows.iter().enumerate() {
        assert_eq!(*at, n);
        assert_eq!(*passed, expected[n].0);
    }
}
#[test]
fn physical_native_valid_empty_selection_and_unrecognized_draft_cable_remain_repairable() {
    let mut h = Harness::new();
    h.assignment();
    let mut draft = scope_draft(&h.a, false).clone();
    draft.selected_link.clear();
    draft
        .configuration
        .cables
        .insert("assignment-uplink".into(), "not-a-supplied-cable".into());
    work::set_draft(&mut h.a, &h.s, work::Scope::Assignment, &draft).unwrap();
    h.p.clear();
    h.idle(3);
    h.choose("cable", "assignment-clean");
    h.choose("mode:assignment-right", "Auto");
    h.click("apply");
    h.click("check");
    assert!(h.m.complete);
}

#[test]
fn physical_native_rejected_paste_release_stays_vetoed_through_discarded_pass_and_retry_clears_notice()
 {
    let mut h = Harness::new();
    h.assignment();
    let probe = h.reveal("probe");
    let field = h.m.numbers[0].1;
    h.tap(field);
    // A real held press, then natural keyboard navigation back to the numeric owner.
    h.frame(vec![
        egui::Event::PointerMoved(probe.center()),
        egui::Event::PointerButton {
            pos: probe.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    let id = h.m.numbers[0].2;
    for _ in 0..20 {
        if h.ctx.memory(|m| m.focused()) == Some(id) {
            break;
        }
        h.frame(vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)]);
        h.idle(1);
    }
    assert_eq!(h.ctx.memory(|m| m.focused()), Some(id));
    let before_cursor = egui::text_edit::TextEditState::load(&h.ctx, id)
        .unwrap()
        .cursor
        .char_range();
    let before = serde_json::to_vec(&h.a).unwrap();
    let events = vec![
        select_all(),
        egui::Event::Paste("../../outside".into()),
        egui::Event::PointerMoved(probe.center()),
        egui::Event::PointerButton {
            pos: probe.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        },
    ];
    let ctx = h.ctx.clone();
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(h.width, h.height),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            h.m = Metrics::default();
            let focus = ctx.memory(|m| m.focused());
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("physical-widget-gate")
                    .show(ui, |ui| {
                        page(ui, &h.s, &mut h.a, &mut h.p, &mut h.m, true, focus);
                    });
            });
            if ctx.current_pass_index() == 0 {
                ctx.request_discard("Reject the actual held-release input in both layout passes");
            }
        },
    );
    assert_eq!(out.platform_output.num_completed_passes, 2);
    assert_eq!(
        egui::text_edit::TextEditState::load(&h.ctx, id)
            .unwrap()
            .cursor
            .char_range(),
        before_cursor
    );
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
    assert_eq!(h.p.numbers[0], "12");
    assert!(h.p.notice.is_some());
    h.idle(2);
    let field = h.m.numbers[0].1;
    h.tap(field);
    h.frame(vec![select_all(), egui::Event::Paste("7".into())]);
    h.idle(1);
    assert_eq!(h.p.numbers[0], "7");
    assert!(h.p.notice.is_none());
    h.click("probe");
    assert!(scope_state(&h.a, false).counters()["assignment-left"].offered > 0);
}

// Development control only: observe cloned native undo tuples; never install history.
fn physical_readonly_undo_projection(
    mut undo: egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
    mut current: (egui::text::CCursorRange, String),
) -> Vec<(egui::text::CCursorRange, String)> {
    let mut out = Vec::new();
    for _ in 0..100 {
        let Some(previous) = undo.undo(&current).cloned() else {
            return out;
        };
        out.push(previous.clone());
        current = previous;
    }
    panic!("Fixed native reference unexpectedly exceeded its native100 undo bound");
}
#[test]
fn physical_undo_reference_deep_history_and_cursor_survive_the_original_rejected_event_trace() {
    let mut h = Harness::new();
    h.assignment();
    let probe = h.reveal("probe");
    let (_, field, id) = *h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap();
    h.tap(field);
    h.frame(vec![select_all(), egui::Event::Paste("24".into())]);
    h.idle(12);
    h.frame(vec![key(egui::Key::Tab, Default::default())]);
    h.idle(1);
    h.frame(vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)]);
    h.idle(1);
    assert_eq!(h.ctx.memory(|m| m.focused()), Some(id));
    let native = egui::text_edit::TextEditState::load(&h.ctx, id).unwrap();
    let cursor = native.cursor.char_range().unwrap();
    let deep = native.undoer();
    let flux = deep.is_in_flux();
    let expected = physical_readonly_undo_projection(deep, (cursor, h.p.numbers[0].clone()));
    let before = serde_json::to_vec(&h.a).unwrap();
    h.frame(vec![
        select_all(),
        egui::Event::Paste("oops".into()),
        egui::Event::PointerMoved(probe.center()),
        egui::Event::PointerButton {
            pos: probe.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    h.frame(vec![egui::Event::PointerButton {
        pos: probe.center(),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    }]);
    assert_eq!(serde_json::to_vec(&h.a).unwrap(), before);
    assert_eq!(h.p.numbers[0], "24");
    let native = egui::text_edit::TextEditState::load(&h.ctx, id).unwrap();
    assert_eq!(native.cursor.char_range(), Some(cursor));
    assert_eq!(native.undoer().is_in_flux(), flux);
    assert_eq!(
        physical_readonly_undo_projection(native.undoer(), (cursor, "24".into())),
        expected
    );
    let fresh = *h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap();
    println!(
        "Original cached refocus geometry: old={field:?}, current={:?}, old-center-in-current={}, old-id={id:?}, current-id={:?}, focused-before={:?}",
        fresh.1,
        fresh.1.contains(field.center()),
        fresh.2,
        h.ctx.memory(|m| m.focused())
    );
    h.tap(field);
    println!(
        "Original cached refocus actual focus={:?}, expected={id:?}, field-now={:?}",
        h.ctx.memory(|m| m.focused()),
        h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap().1
    );
    assert_eq!(
        h.ctx.memory(|m| m.focused()),
        Some(id),
        "Original cached-center tap must actually focus the expected numeric widget"
    );
    let native = egui::text_edit::TextEditState::load(&h.ctx, id).unwrap();
    let current = (native.cursor.char_range().unwrap(), h.p.numbers[0].clone());
    let predictions = physical_readonly_undo_projection(native.undoer(), current.clone());
    println!(
        "Physical original trace before Ctrl+Z: current={current:?}; native undo tuples={predictions:?}"
    );
    let predicted = predictions.first().unwrap().clone();
    h.frame(vec![key(
        egui::Key::Z,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    )]);
    h.idle(1);
    assert_eq!(h.p.numbers[0], predicted.1);
    println!(
        "Physical original trace first native Ctrl+Z text={:?}",
        h.p.numbers[0]
    );
}
struct PhysicalPlainUndoControl {
    ctx: egui::Context,
    text: [String; 2],
    fields: [egui::Rect; 2],
    button: egui::Rect,
    time: f64,
}
impl PhysicalPlainUndoControl {
    fn new() -> Self {
        let mut p = Self {
            ctx: egui::Context::default(),
            text: ["12".into(), "18".into()],
            fields: [egui::Rect::NOTHING; 2],
            button: egui::Rect::NOTHING,
            time: 0.,
        };
        p.idle(3);
        p
    }
    fn frame(&mut self, events: Vec<egui::Event>) {
        self.time += 0.1;
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600., 1200.),
            )),
            time: Some(self.time),
            events,
            modifiers,
            ..Default::default()
        };
        let _ = self.ctx.run(input, |ctx| {
            controls::configure(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for n in 0..2 {
                        let width = controls::button_width(ui, "1024");
                        let r = ui.add(
                            controls::singleline(ui, &mut self.text[n])
                                .id(egui::Id::new(("physical-plain-undo", n)))
                                .desired_width(width)
                                .hint_text("12"),
                        );
                        self.fields[n] = r.rect;
                    }
                    self.button = ui.add(button("Send simulated sample")).rect;
                });
            });
        });
    }
    fn idle(&mut self, n: usize) {
        for _ in 0..n {
            self.frame(vec![]);
        }
    }
    fn tap(&mut self, r: egui::Rect) {
        for pressed in [true, false] {
            self.frame(vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ]);
        }
        self.idle(2);
    }
}
#[test]
fn physical_undo_reference_plain_string_and_numeric_field_match_the_accepted_edit_focus_trace() {
    let mut h = Harness::new();
    h.assignment();
    let probe = h.reveal("probe");
    let field = h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap().1;
    let mut plain = PhysicalPlainUndoControl::new();
    plain.time = h.time;
    h.tap(field);
    plain.tap(plain.fields[0]);
    let edit = vec![select_all(), egui::Event::Paste("24".into())];
    h.frame(edit.clone());
    plain.frame(edit);
    h.idle(12);
    plain.idle(12);
    let tab = vec![key(egui::Key::Tab, Default::default())];
    h.frame(tab.clone());
    plain.frame(tab);
    h.idle(1);
    plain.idle(1);
    let back = vec![key(egui::Key::Tab, egui::Modifiers::SHIFT)];
    h.frame(back.clone());
    plain.frame(back);
    h.idle(1);
    plain.idle(1);
    // No rejected text in this independent control: isolate ordinary native
    // accepted-edit, focus-loss and center-click refocus behavior. The previous
    // test separately proves rejection restores deep history and selection.
    h.tap(probe);
    plain.tap(plain.button);
    h.tap(field);
    plain.tap(plain.fields[0]);
    // Match the actually observed restored selection in the original trace,
    // using ordinary native events in both independent controls.
    let selected = vec![select_all()];
    h.frame(selected.clone());
    plain.frame(selected);
    let id = h.m.numbers.iter().find(|(n, _, _)| *n == 0).unwrap().2;
    let a = egui::text_edit::TextEditState::load(&h.ctx, id).unwrap();
    let b = egui::text_edit::TextEditState::load(
        &plain.ctx,
        egui::Id::new(("physical-plain-undo", 0usize)),
    )
    .unwrap();
    let current_a = (a.cursor.char_range().unwrap(), h.p.numbers[0].clone());
    let current_b = (b.cursor.char_range().unwrap(), plain.text[0].clone());
    let undo_a = physical_readonly_undo_projection(a.undoer(), current_a.clone());
    let undo_b = physical_readonly_undo_projection(b.undoer(), current_b.clone());
    assert_eq!(h.ctx.memory(|m| m.focused()), Some(id));
    assert_eq!(
        plain.ctx.memory(|m| m.focused()),
        Some(egui::Id::new(("physical-plain-undo", 0usize)))
    );
    assert_eq!(
        current_a, current_b,
        "Complete actual native current tuples must match"
    );
    assert_eq!(
        undo_a, undo_b,
        "Complete readonly native Undo projections must match"
    );
    assert_eq!(
        undo_a.iter().map(|r| r.1.as_str()).collect::<Vec<_>>(),
        ["24", "12"]
    );
    println!(
        "Matched original selected tuple product={current_a:?}, undo={undo_a:?}; plain String={current_b:?}, undo={undo_b:?}"
    );
    let collapse = vec![key(egui::Key::ArrowRight, Default::default())];
    h.frame(collapse.clone());
    plain.frame(collapse);
    assert_eq!(h.p.numbers[0], "24");
    assert_eq!(plain.text[0], "24");
    let a = egui::text_edit::TextEditState::load(&h.ctx, id).unwrap();
    let b = egui::text_edit::TextEditState::load(
        &plain.ctx,
        egui::Id::new(("physical-plain-undo", 0usize)),
    )
    .unwrap();
    let current_a = (a.cursor.char_range().unwrap(), h.p.numbers[0].clone());
    let current_b = (b.cursor.char_range().unwrap(), plain.text[0].clone());
    assert!(current_a.0.is_empty() && current_b.0.is_empty());
    assert_eq!(current_a, current_b);
    let undo_a = physical_readonly_undo_projection(a.undoer(), current_a.clone());
    let undo_b = physical_readonly_undo_projection(b.undoer(), current_b.clone());
    assert_eq!(undo_a, undo_b);
    assert_eq!(
        undo_a.iter().map(|r| r.1.as_str()).collect::<Vec<_>>(),
        ["12"]
    );
    println!("Matched real ArrowRight collapse tuple={current_a:?}, next native Undo={undo_a:?}");
    let event = vec![key(
        egui::Key::Z,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    )];
    h.frame(event.clone());
    plain.frame(event);
    h.idle(1);
    plain.idle(1);
    assert_eq!(h.p.numbers[0], undo_a.first().unwrap().1);
    assert_eq!(plain.text[0], undo_b.first().unwrap().1);
    assert_eq!(
        h.p.numbers[0], plain.text[0],
        "Same native accepted-edit/focus trace must give the same first Undo text"
    );
    println!(
        "Matched accepted-edit/focus first native Ctrl+Z text={:?}",
        plain.text[0]
    );
}

// Exercise the investigation section inside the actual normal app layout:
// toolbar, selected-workspace max width, card padding and responsive columns.
// This fixture launches normal content through the existing app/typed writers;
// rendering and real wheels never install state, grading results or focus.
struct CounterLayoutPaint {
    ctx: egui::Context,
    app: LabApp,
    id: String,
    width: f32,
    scale: f32,
    time: f64,
    texts: Vec<(String, egui::Rect, egui::Rect)>,
}
impl CounterLayoutPaint {
    fn new(width: f32, scale: f32, dark: bool) -> Self {
        let ctx = egui::Context::default();
        let mut app = LabApp::from_context(&ctx, true, false);
        app.smoke = false;
        let i = app
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-physical-media-span")
            .unwrap();
        let id = app.scenarios[i].id.clone();
        app.launch(i);
        work::set_view(
            app.progress.attempts.get_mut(&id).unwrap(),
            &app.scenarios[i],
            View::Assignment,
        )
        .unwrap();
        app.scale = scale;
        app.theme_id = if dark { "dlms-dark" } else { "dlms-light" }.into();
        dlms_learning_lab::theme::apply(&ctx, &app.theme_id);
        let mut p = Self {
            ctx,
            app,
            id,
            width,
            scale,
            time: 0.,
            texts: Vec::new(),
        };
        for _ in 0..3 {
            p.frame(vec![]);
        }
        p.assert_native_geometry();
        p
    }
    fn metrics(&self) -> &Metrics {
        &self.app.metrics.physical
    }
    fn attempt(&self) -> &Attempt {
        &self.app.progress.attempts[&self.id]
    }
    fn assert_native_geometry(&self) {
        assert!((self.ctx.zoom_factor() - self.scale).abs() < 0.01);
        assert!((self.ctx.pixels_per_point() - self.scale * 1.15).abs() < 0.01);
        let observed_pixels = self.ctx.content_rect().size() * self.ctx.pixels_per_point();
        assert!(
            (observed_pixels - egui::vec2(self.width, 929.)).length() < 1.,
            "Render fixture must reproduce its requested native pixel geometry: {observed_pixels:?}"
        );
    }
    fn frame(&mut self, events: Vec<egui::Event>) {
        self.time += 0.1;
        self.ctx.set_zoom_factor(self.scale);
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(self.width, 929.) / (self.scale * 1.15),
            )),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let output = self.ctx.run(input, |ctx| self.app.render(ctx));
        self.texts.clear();
        fn text(
            shape: &egui::Shape,
            clip: egui::Rect,
            result: &mut Vec<(String, egui::Rect, egui::Rect)>,
        ) {
            match shape {
                egui::Shape::Text(t) => result.push((
                    t.galley.text().into(),
                    egui::Rect::from_min_size(t.pos, t.galley.size()),
                    clip,
                )),
                egui::Shape::Vec(shapes) => {
                    for s in shapes {
                        text(s, clip, result);
                    }
                }
                _ => {}
            }
        }
        for s in output.shapes {
            text(&s.shape, s.clip_rect, &mut self.texts);
        }
    }
    fn wheel(&mut self, delta: f32) {
        let viewport = self
            .app
            .metrics
            .main_viewport
            .unwrap()
            .intersect(self.ctx.content_rect());
        assert!(viewport.is_positive());
        self.frame(vec![
            egui::Event::PointerMoved(viewport.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta),
                modifiers: Default::default(),
            },
        ]);
        for _ in 0..2 {
            self.frame(vec![]);
        }
        self.assert_native_geometry();
    }
}
#[test]
fn physical_counter_tables_compact_wide_and_stack_at_large_text_without_mutating_work() {
    for width in [946., 1920.] {
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for dark in [false, true] {
                let mut p = CounterLayoutPaint::new(width, scale, dark);
                let before = serde_json::to_vec(p.attempt()).unwrap();
                let whole_progress = serde_json::to_vec(&p.app.progress).unwrap();
                let clears = p
                    .metrics()
                    .controls
                    .iter()
                    .filter(|(id, _, _)| id.starts_with("clear:"))
                    .map(|(_, r, _)| *r)
                    .collect::<Vec<_>>();
                assert_eq!(
                    clears.len(),
                    2,
                    "Both endpoints retain their own Clear action"
                );
                if width == 1920. {
                    assert!(
                        (clears[0].top() - clears[1].top()).abs() < 1.,
                        "Wide tables must share row/action baselines: {width}/{scale}/{clears:?}"
                    );
                    assert!(clears[0].right() < clears[1].left());
                }
                if width == 946. && scale == 2. {
                    assert!(
                        clears[1].top() > clears[0].bottom(),
                        "200% narrow must stack the endpoints: {clears:?}"
                    );
                }
                let empty_height = p.metrics().output.unwrap().height();
                let font = egui::TextStyle::Monospace.resolve(&p.ctx.style());
                let mono = p.ctx.fonts_mut(|fonts| fonts.row_height(&font));
                assert!(
                    empty_height < 6. * mono,
                    "Empty output must not reserve six blank lines"
                );
                let mut seen = std::collections::BTreeSet::new();
                for _ in 0..18 {
                    for (text, rect, clip) in &p.texts {
                        if rect.intersects(*clip) {
                            assert!(
                                rect.left() >= clip.left() - 0.5
                                    && rect.right() <= clip.right() + 0.5,
                                "Counter content clipped horizontally: {width}/{scale}/{text}: {rect:?}/{clip:?}"
                            );
                            if COUNTER_LABELS.contains(&text.as_str()) {
                                seen.insert(text.clone());
                            }
                        }
                    }
                    p.wheel(-120.);
                }
                assert_eq!(seen.len(), 5, "All five counter meanings remain reachable");
                assert_eq!(serde_json::to_vec(p.attempt()).unwrap(), before);
                assert_eq!(serde_json::to_vec(&p.app.progress).unwrap(), whole_progress);
                assert!(p.app.physical_presentation.logs.is_empty());
                assert!(!checked(p.attempt(), false));
            }
        }
    }
}
#[test]
fn physical_populated_sample_output_keeps_its_bounded_scroll_space_and_real_clear_actions() {
    let mut h = Harness::new();
    h.assignment();
    let before_empty = h.m.output.unwrap().height();
    let state = h.applied().configuration().clone();
    h.click("probe");
    assert!(
        !h.p.logs.is_empty(),
        "A real simulated sample creates the output"
    );
    assert!(h.applied().counters()["assignment-left"].offered > 0);
    let populated = h.m.output.unwrap().height();
    assert!(populated > before_empty);
    h.click("probe");
    assert_eq!(
        h.m.output.unwrap().height(),
        populated,
        "More output scrolls inside the same viewport"
    );
    h.click("clear:assignment-left");
    assert_eq!(
        h.applied().counters()["assignment-left"],
        core::Counters::default()
    );
    assert!(
        h.applied().counters()["assignment-right"].offered > 0,
        "Each Clear action still targets only its own endpoint"
    );
    assert_eq!(h.applied().configuration(), &state);
    assert_eq!(h.m.output.unwrap().height(), populated);
    assert!(!h.m.complete);
}
#[test]
fn physical_counter_maximum_values_and_long_endpoint_labels_wrap_without_losing_values() {
    for width in [946., 1920.] {
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for dark in [false, true] {
                let ctx = egui::Context::default();
                dlms_learning_lab::theme::apply(
                    &ctx,
                    if dark { "dlms-dark" } else { "dlms-light" },
                );
                ctx.set_zoom_factor(scale);
                let counters = core::Counters {
                    offered: core::MAX_COUNTER,
                    received: core::MAX_COUNTER - 1,
                    crc_errors: core::MAX_COUNTER - 2,
                    collisions: core::MAX_COUNTER - 3,
                    late_collisions: core::MAX_COUNTER - 4,
                };
                let label = "Distribution uplink to the secondary workshop cabinet";
                let mut bounds = None;
                let mut metrics = Metrics::default();
                let mut intent = Intent::default();
                let mut painted = Vec::new();
                for _ in 0..3 {
                    metrics.controls.clear();
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width / scale, 929. / scale),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            controls::configure(ctx);
                            egui::CentralPanel::default().show(ctx, |ui| {
                                let font = egui::TextStyle::Body.resolve(ui.style());
                                let height = ui
                                    .painter()
                                    .layout(
                                        label.into(),
                                        font,
                                        palette(ui).text,
                                        ui.available_width(),
                                    )
                                    .size()
                                    .y;
                                counter_table(
                                    ui,
                                    ("readonly-long-port", label),
                                    &counters,
                                    height,
                                    &mut metrics,
                                    &mut intent,
                                );
                                bounds = Some(ui.clip_rect());
                            });
                        },
                    );
                    painted = output.shapes;
                }
                let mut values = std::collections::BTreeSet::new();
                for shape in &painted {
                    if let egui::Shape::Text(t) = &shape.shape {
                        let rect = egui::Rect::from_min_size(t.pos, t.galley.size());
                        assert!(
                            rect.left() >= bounds.unwrap().left() - 0.5
                                && rect.right() <= bounds.unwrap().right() + 0.5,
                            "Long readonly counter content widens the table: {width}/{scale}/{rect:?}"
                        );
                        if counter_values(&counters)
                            .iter()
                            .any(|n| n.to_string() == t.galley.text())
                        {
                            values.insert(t.galley.text().to_owned());
                        }
                    }
                }
                assert_eq!(
                    values.len(),
                    5,
                    "Every distinct maximum-sized value is painted"
                );
                assert_eq!(metrics.controls.len(), 1);
                assert!(intent.clear.is_empty(), "Rendering never clears a counter");
                assert!(metrics.controls[0].1.right() <= bounds.unwrap().right() + 0.5);
            }
        }
    }
}
