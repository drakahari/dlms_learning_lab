//! Real widgets for each new source task; not a novice-comprehension study.
use super::*;
use dlms_learning_lab::{debian_sources_workspace as work, engine::Goal};
const IDS: [&str; 2] = [
    "lpic1-package-source-selection",
    "lpic1-package-source-recovery",
];
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    session_mut(&mut a).view = view;
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let id = &a.scenarios[a.selected.unwrap()].id;
    let State::DebianSources { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn session_mut(a: &mut LabApp) -> &mut work::Session {
    let id = a.scenarios[a.selected.unwrap()].id.clone();
    let State::DebianSources { simulation } = &mut a.progress.attempts.get_mut(&id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn scope(a: &LabApp) -> &work::Scope {
    let s = session(a);
    if s.view == teaching::View::Example {
        &s.practice
    } else {
        &s.assignment
    }
}
fn source(a: &LabApp) -> &work::Definition {
    let Goal::DebianSources { definition } = &a.scenarios[a.selected.unwrap()].goal else {
        panic!()
    };
    definition
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    match key {
        "reset" => a.metrics.reset.unwrap(),
        "confirm-reset" => a.metrics.confirm_reset.unwrap(),
        "editor" => a.metrics.debian_sources.editor.unwrap().0,
        "query" => a.metrics.debian_sources.query.unwrap().0,
        _ => {
            a.metrics
                .debian_sources
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap_or_else(|| panic!("missing source control {key}"))
                .1
        }
    }
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        frame(ctx, a, 1200., 850., vec![]);
        let r = rect(a, key);
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 850., vec![]);
        }
    }
    panic!("Actual source control did not become visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    click(ctx, a, r);
}
fn type_in(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key);
    click(ctx, a, r);
    let id = if key == "editor" {
        a.metrics.debian_sources.editor.unwrap().1
    } else {
        a.metrics.debian_sources.query.unwrap().1
    };
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Paste(text.into())],
    );
    draw(ctx, a);
    assert_eq!(
        if key == "editor" {
            &scope(a).draft
        } else {
            &scope(a).query
        },
        text,
        "Actual typing updates only the intended input"
    );
}
fn painted(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn append(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    append(s, text)
                }
            }
            _ => {}
        }
    }
    let o = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 2800.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for s in o.shapes {
        append(&s.shape, &mut text);
    }
    text
}
#[test]
fn source_three_views_teach_their_specific_concept_and_keep_assignment_answers_optional() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id, teaching::View::Introduction);
        let text = painted(&ctx, &mut a);
        assert!(text.contains("WHAT YOU’LL LEARN") && text.contains("Worked example"));
        assert!(
            !text.contains("1. 1."),
            "The shared renderer supplies section numbering once"
        );
        assert!(text.contains(if id.ends_with("recovery") {
            "A refresh has a result for each source"
        } else {
            "A repository is selected by three values"
        }));
        session_mut(&mut a).view = teaching::View::Example;
        a.lesson_scroll_reset = true;
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Fill worked source excerpt"));
        assert!(text.contains("Save sources") && text.contains("Look up package"));
        assert!(
            a.metrics.debian_sources.editor.is_some() && a.metrics.debian_sources.query.is_some()
        );
        session_mut(&mut a).view = teaching::View::Assignment;
        a.lesson_scroll_reset = true;
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements") && text.contains("Check work"));
        assert!(!text.contains("Fill worked source excerpt"));
        assert!(
            !text.contains(source(&a).assignment_reference.sources.trim()),
            "Independent solution is not painted by default"
        );
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let text = painted(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!text.contains("Fill worked source excerpt"));
        assert!(!text.contains(source(&a).assignment_reference.sources.trim()));
    }
}
#[test]
fn source_actual_keyboard_save_refresh_lookup_retry_and_current_credit_work_for_both_tasks() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Assignment);
        let d = source(&a).clone();
        let original_saved = scope(&a).state.clone();
        type_in(
            &ctx,
            &mut a,
            "editor",
            &(d.assignment_reference.sources.clone() + "Enabled: true\n"),
        );
        tap(&ctx, &mut a, "save");
        assert_eq!(scope(&a).state, original_saved);
        assert!(
            painted(&ctx, &mut a).contains("Enabled supports yes or no"),
            "The useful save error must actually be painted, not merely stored"
        );
        type_in(&ctx, &mut a, "editor", &d.assignment_reference.sources);
        assert!(
            !painted(&ctx, &mut a).contains("Enabled supports yes or no"),
            "Editing clears obsolete save feedback"
        );
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        type_in(&ctx, &mut a, "editor", &d.assignment_reference.sources);
        tap(&ctx, &mut a, "save");
        tap(&ctx, &mut a, "check");
        assert!(
            !a.metrics.completion,
            "Saving did not fetch the needed indexes"
        );
        tap(&ctx, &mut a, "refresh");
        let package = if id.ends_with("recovery") {
            "campus-helper"
        } else {
            "desk-indexer"
        };
        type_in(&ctx, &mut a, "query", package);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        draw(&ctx, &mut a);
        assert_eq!(
            scope(&a).observed.as_deref(),
            Some(package),
            "Enter runs the focused query"
        );
        tap(&ctx, &mut a, "lookup");
        assert_eq!(scope(&a).observed.as_deref(), Some(package));
        let r = target(&ctx, &mut a, "query");
        click(&ctx, &mut a, r);
        let field = a.metrics.debian_sources.query.unwrap().1;
        frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
        draw(&ctx, &mut a);
        assert_ne!(
            ctx.memory(|m| m.focused()),
            Some(field),
            "Tab leaves a single-line query for the next native control"
        );
        assert!(ctx.memory(|m| m.focused()).is_some());
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        type_in(
            &ctx,
            &mut a,
            "editor",
            &d.assignment_reference
                .sources
                .replace("Components: main tools", "Components: main"),
        );
        assert!(!scope(&a).checked);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        tap(&ctx, &mut a, "save");
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(a.metrics.debian_sources.results.iter().any(|r| *r));
        assert!(a.metrics.debian_sources.results.iter().any(|r| !*r));
        type_in(&ctx, &mut a, "editor", &d.assignment_reference.sources);
        tap(&ctx, &mut a, "save");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        a.save();
        assert!(a.storage_error.is_none());
        let restored = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
    }
}
#[test]
fn source_fill_is_inert_and_navigation_preserves_distinct_raw_drafts() {
    let (ctx, mut a, _) = opened(IDS[0], teaching::View::Example);
    let original = session(&a).assignment.clone();
    let before = session(&a).practice.state.clone();
    tap(&ctx, &mut a, "fill-worked");
    assert_eq!(session(&a).practice.state, before);
    assert_eq!(session(&a).assignment, original);
    assert!(!session(&a).practice.checked);
    type_in(&ctx, &mut a, "editor", "# unfinished example\nSuites");
    tap(&ctx, &mut a, "assignment");
    type_in(
        &ctx,
        &mut a,
        "editor",
        "# unfinished assignment\nComponents",
    );
    tap(&ctx, &mut a, "example");
    assert_eq!(scope(&a).draft, "# unfinished example\nSuites");
    assert_eq!(
        session(&a).assignment.draft,
        "# unfinished assignment\nComponents"
    );
    a.save();
    assert!(a.storage_error.is_none());
    let restored = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
}
#[test]
fn source_controls_reflow_at_supported_scales_themes_and_widths_without_shrinking_text() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id, teaching::View::Assignment);
        for light in [false, true] {
            a.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&ctx, &a.theme_id);
            for scale in [1., 1.15, 1.25, 1.5, 2.] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                for width in [760., 1600.] {
                    for view in [teaching::View::Example, teaching::View::Assignment] {
                        session_mut(&mut a).view = view;
                        a.lesson_scroll_reset = true;
                        for _ in 0..3 {
                            frame(&ctx, &mut a, width, 1000., vec![]);
                        }
                        let clip = a.metrics.main_viewport.unwrap();
                        for (key, r, _) in &a.metrics.debian_sources.controls {
                            assert!(r.width() > 0. && r.height() > 0.);
                            assert!(
                                r.right() <= clip.right() + 1.,
                                "{id}: {key} clips at {width}px/{scale}"
                            );
                            assert!(r.left() >= clip.left() - 1.);
                        }
                        let (r, _) = a.metrics.debian_sources.query.unwrap();
                        assert!(r.right() <= clip.right() + 1.);
                        assert!(r.height() > 0.);
                        assert!(!a.metrics.completion);
                    }
                }
            }
        }
    }
}
#[test]
fn source_editor_scrollbar_does_not_move_save_buttons_or_their_click_targets() {
    let (ctx, mut a, _) = opened(IDS[0], teaching::View::Assignment);
    for width in [760., 1600.] {
        for scale in [1., 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            session_mut(&mut a)
                .assignment
                .edit("# short source draft\n")
                .unwrap();
            a.lesson_scroll_reset = true;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 1000., vec![]);
            }
            let unsettled = rect(&a, "save");
            // Finish the OUTER scrollbar/zoom animation caused by the preceding
            // resize before isolating the editor's content/scrollbar change.
            let mut prior = rect(&a, "save");
            let mut stable = 0;
            for _ in 0..60 {
                frame(&ctx, &mut a, width, 1000., vec![]);
                let next = rect(&a, "save");
                if (next.min - prior.min).length() < 0.01 {
                    stable += 1;
                } else {
                    stable = 0;
                }
                prior = next;
                if stable >= 6 {
                    break;
                }
            }
            assert!(
                stable >= 6,
                "Outer resize geometry must settle before the content comparison"
            );
            let before = rect(&a, "save");
            if (unsettled.min - before.min).length() > 0.5 {
                eprintln!(
                    "Source layout diagnostic: unchanged short draft moved during OUTER resize animation: {unsettled:?} -> {before:?} at {width}/{scale}"
                );
            }
            let before_offset = a.metrics.main_scroll_offset;
            let before_content = a.metrics.main_content_size;
            let refresh_before = rect(&a, "refresh");
            session_mut(&mut a)
                .assignment
                .edit(&"# retained long edit for scrolling\n".repeat(160))
                .unwrap();
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 1000., vec![]);
            }
            let after = rect(&a, "save");
            let refresh_after = rect(&a, "refresh");
            assert!(
                (before.min - after.min).length() < 0.5,
                "Scrollbar/content must not shift Save; width={width} scale={scale} before={before:?} after={after:?} offsets={before_offset:?}/{:?} content={before_content:?}/{:?}",
                a.metrics.main_scroll_offset,
                a.metrics.main_content_size
            );
            assert!((refresh_before.min - refresh_after.min).length() < 0.5);
        }
    }
}

#[test]
fn source_editor_reveal_uses_full_local_bounds_instead_of_an_outer_clipped_sliver() {
    let (ctx, mut a, _) = opened(IDS[0], teaching::View::Example);
    // Match the actual native632px window, with the first editor crossing bottom.
    a.lesson_scroll_reset = true;
    for _ in 0..3 {
        frame(&ctx, &mut a, 1268., 632., vec![]);
    }
    let r = a.metrics.debian_sources.editor.unwrap().0;
    let clip = a.metrics.main_viewport.unwrap();
    assert!(
        r.bottom() > clip.bottom() + 1.,
        "The offscreen part remains represented, so reveal requests its full missing extent"
    );
    assert!(r.height() > a.metrics.main_viewport.unwrap().bottom() - r.top());
}

#[test]
fn source_selected_reset_clears_transient_errors_as_well_as_the_saved_task_state() {
    let (ctx, mut a, _) = opened(IDS[0], teaching::View::Assignment);
    let d = source(&a).clone();
    type_in(
        &ctx,
        &mut a,
        "editor",
        &(d.assignment_reference.sources.clone() + "Enabled: true\n"),
    );
    tap(&ctx, &mut a, "save");
    assert!(painted(&ctx, &mut a).contains("Enabled supports yes or no"));
    tap(&ctx, &mut a, "reset");
    tap(&ctx, &mut a, "confirm-reset");
    tap(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment.draft, d.initial.starter.saved);
    assert!(!painted(&ctx, &mut a).contains("Enabled supports yes or no"));
    assert!(!a.metrics.completion);
}
