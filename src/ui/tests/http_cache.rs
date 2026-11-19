//! Real native controls against the registered cache tasks; not novice acceptance.
use super::*;
use dlms_learning_lab::http_cache_workspace as work;
const IDS: [&str; 2] = [
    "lpic2-cache-shared-privacy",
    "lpic2-cache-freshness-handover",
];
fn opened(n: usize) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    assert!(a._smoke_profile.is_some());
    assert_eq!((a.scenarios.len(), a.units.len()), (439, 172));
    a.launch(a.scenarios.iter().position(|s| s.id == IDS[n]).unwrap());
    draw(&ctx, &mut a);
    (ctx, a, IDS[n].into())
}
fn session<'a>(a: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::HttpCache { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .http_cache
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; {:?}", a.notice))
        .1
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        let r = rect(a, key);
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if r.top() >= viewport.top() && r.bottom() <= viewport.bottom() {
            return r;
        }
        let delta = if r.bottom() > viewport.bottom() {
            viewport.bottom() - r.bottom()
        } else {
            viewport.top() - r.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(viewport.right() - 3., viewport.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("Cache {key} did not become visible")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key);
    click(ctx, a, r);
    draw(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, index: usize, text: &str) {
    tap(ctx, a, &format!("header-{index}"));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            key(egui::Key::Delete, false),
            egui::Event::Text(text.into()),
        ],
    );
    draw(ctx, a);
}
fn view(ctx: &egui::Context, a: &mut LabApp, id: &str, v: teaching::View) {
    session(a, id).view = v;
    a.lesson_scroll_reset = true;
    draw(ctx, a);
}
fn reference(a: &LabApp) -> Vec<work::Header> {
    work::source(&a.scenarios[a.selected.unwrap()])
        .unwrap()
        .initial
        .reference_headers()
}
#[test]
fn both_lessons_teach_then_offer_separate_practice_and_complete_unsolved_assignments() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        let e = lesson_presentation::entry(&a.scenarios[a.selected.unwrap()]).unwrap();
        assert_eq!(e.concepts.len(), 4);
        assert!(e.worked.len() >= 4);
        assert_eq!(e.practice.len(), 3);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(!a.metrics.http_cache.controls.iter().any(|c| c.0 == "check"));
        view(&ctx, &mut a, &id, teaching::View::Example);
        assert!(a.metrics.http_cache.controls.iter().any(|c| c.0 == "fill"));
        assert_eq!(a.metrics.http_cache.guidance_order, vec![0, 1, 2]);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        assert!(
            a.metrics
                .http_cache
                .controls
                .iter()
                .any(|c| c.0 == "header-0")
        );
        assert!(!a.metrics.http_cache.controls.iter().any(|c| c.0 == "fill"));
        assert!(!a.metrics.http_cache.worked_answer_visible);
        assert!(!a.metrics.completion);
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        view(&ctx, &mut a, &id, teaching::View::Introduction);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.http_cache.worked_answer_visible);
        assert!(a.metrics.http_cache.controls.iter().any(|c| c.0 == "check"));
    }
}
#[test]
fn worked_fill_and_example_run_do_not_change_or_complete_the_assignment() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        let original = session(&mut a, &id).assignment.clone();
        view(&ctx, &mut a, &id, teaching::View::Example);
        tap(&ctx, &mut a, "fill");
        assert!(!session(&mut a, &id).practice.checked());
        tap(&ctx, &mut a, "check");
        assert!(
            a.metrics
                .http_cache
                .summary
                .as_ref()
                .unwrap()
                .starts_with("All declared")
        );
        assert!(!a.metrics.completion);
        assert_eq!(session(&mut a, &id).assignment, original);
        tap(&ctx, &mut a, "reset-example");
        assert!(!session(&mut a, &id).practice.checked());
        assert_eq!(session(&mut a, &id).assignment, original);
    }
}
#[test]
fn actual_typing_checks_behavior_accepts_alternatives_and_clears_stale_success() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(
            a.metrics
                .http_cache
                .summary
                .as_ref()
                .unwrap()
                .starts_with("Needs attention")
        );
        let mut headers = reference(&a);
        if n == 0 {
            headers[0].value = "max-age=30".into();
            headers[1].value = "no-store".into();
        } else {
            headers[0].value = "s-maxage=30, max-age=600".into();
        }
        for (i, h) in headers.iter().enumerate() {
            replace(&ctx, &mut a, i, &h.value);
            assert_eq!(session(&mut a, &id).assignment.headers[i].value, h.value);
        }
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert!(a.metrics.http_cache.cases.iter().all(|c| c.1));
        replace(&ctx, &mut a, 0, "no-store");
        assert!(!a.metrics.completion);
        assert!(!session(&mut a, &id).assignment.checked());
        assert!(a.metrics.http_cache.cases.is_empty());
        assert!(
            a.metrics
                .http_cache
                .summary
                .as_ref()
                .unwrap()
                .contains("Not checked")
        );
    }
}
#[test]
fn overall_failure_names_failed_traces_while_correct_selected_rows_stay_correct() {
    let (ctx, mut a, id) = opened(0);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    replace(&ctx, &mut a, 0, "max-age=30");
    replace(&ctx, &mut a, 1, "no-cache, max-age=60");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .http_cache
            .controls
            .iter()
            .any(|c| c.0 == "failed-two-people")
    );
    assert!(
        a.metrics
            .http_cache
            .feedback
            .iter()
            .any(|s| s.contains("preventing storage"))
    );
    session(&mut a, &id).assignment.selected_trace = "public-repeat".into();
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(a.metrics.http_cache.cases.iter().all(|c| c.1));
    assert!(
        a.metrics
            .http_cache
            .summary
            .as_ref()
            .unwrap()
            .contains("traces fail")
    );
    assert!(
        a.metrics
            .http_cache
            .controls
            .iter()
            .any(|c| c.0 == "failed-two-people")
    );
}
#[test]
fn syntax_unsupported_and_retry_feedback_remain_distinct() {
    let (ctx, mut a, id) = opened(1);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    for (text, kind) in [("max-age=wrong", "Syntax"), ("immutable", "Unsupported")] {
        replace(&ctx, &mut a, 0, text);
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(
            a.metrics
                .http_cache
                .feedback
                .iter()
                .any(|f| f.contains(kind))
        );
        assert!(a.metrics.http_cache.cases.is_empty());
    }
    let valid = reference(&a)[0].value.clone();
    replace(&ctx, &mut a, 0, &valid);
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}
#[test]
fn single_line_fields_keep_native_focus_and_reflow_without_overrunning_the_viewport() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        tap(&ctx, &mut a, "header-0");
        let field = a.metrics.http_cache.fields[0].2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(field));
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        assert!(!session(&mut a, &id).assignment.checked());
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for width in [760., 1600.] {
                for light in [false, true] {
                    a.scale = scale;
                    a.theme_id = if light {
                        "light".into()
                    } else {
                        "lab-navy".into()
                    };
                    theme::apply(&ctx, &a.theme_id);
                    a.lesson_scroll_reset = true;
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 1000., vec![]);
                    }
                    let bounds = a.metrics.main_viewport.unwrap();
                    for (_, r, _, _) in &a.metrics.http_cache.fields {
                        assert!(
                            r.left() >= bounds.left() - 1. && r.right() <= bounds.right() + 1.,
                            "{width} at{scale}: {r:?} versus{bounds:?}"
                        );
                        assert!(r.height() > 10.);
                    }
                    assert!(!a.metrics.http_cache.worked_answer_visible);
                }
            }
        }
    }
}

#[test]
fn long_populated_and_empty_header_values_do_not_move_the_check_button_or_expand_the_field() {
    let (ctx, mut a, id) = opened(0);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    let before = rect(&a, "header-0");
    let check = rect(&a, "check");
    let long = format!("max-age={}", "9".repeat(900));
    replace(&ctx, &mut a, 0, &long);
    assert_eq!(session(&mut a, &id).assignment.headers[0].value, long);
    let populated = rect(&a, "header-0");
    assert!((populated.width() - before.width()).abs() < 1.);
    assert!((populated.height() - before.height()).abs() < 1.);
    assert!((rect(&a, "check").top() - check.top()).abs() < 1.);
    replace(&ctx, &mut a, 0, "");
    assert!(session(&mut a, &id).assignment.headers[0].value.is_empty());
    assert!((rect(&a, "header-0").height() - before.height()).abs() < 1.);
    assert!(!session(&mut a, &id).assignment.checked());
}
