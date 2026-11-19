//! Normal430 native widgets; technical checks, not novice acceptance.
use super::*;
use dlms_learning_lab::proxy_workspace as work;
const IDS: [&str; 2] = ["lpic2-proxy-client-boundary", "lpic2-proxy-connect-order"];
fn opened(n: usize) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    assert!(a._smoke_profile.is_some());
    let expected = pack::load_catalog(&[include_str!(
        "../../../tests/fixtures/proxy-policy-preview-v1.json"
    )])
    .unwrap();
    for source in expected {
        let actual = a
            .scenarios
            .iter()
            .find(|s| s.id == source.id)
            .expect("normal catalog registration");
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(source).unwrap()
        );
    }
    let proposal: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/proxy-policy-unit-preview-v1.json"
    ))
    .unwrap();
    for raw in proposal["units"].as_array().unwrap() {
        let unit: course::Unit = serde_json::from_value(raw.clone()).unwrap();
        let actual = a
            .units
            .iter()
            .find(|u| u.id == unit.id)
            .expect("normal unit registration");
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(unit).unwrap()
        );
    }
    assert_eq!((a.scenarios.len(), a.units.len()), (439, 172));
    a.launch(a.scenarios.iter().position(|s| s.id == IDS[n]).unwrap());
    draw(&ctx, &mut a);
    (ctx, a, IDS[n].into())
}
fn session<'a>(a: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::ProxyPolicy { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!("proxy state")
    };
    simulation
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .proxy
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
    panic!("Proxy {key} did not become visible")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key);
    click(ctx, a, r);
    draw(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    tap(ctx, a, "editor");
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
fn required(a: &LabApp) -> String {
    work::source(&a.scenarios[a.selected.unwrap()])
        .unwrap()
        .initial
        .required_policy
        .clone()
}
#[test]
fn both_lessons_have_teaching_real_editors_full_tasks_and_no_unsolicited_solutions() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        let e = lesson_presentation::entry(&a.scenarios[a.selected.unwrap()]).unwrap();
        assert_eq!(e.concepts.len(), 4);
        assert!(!e.worked.is_empty());
        assert!(!a.metrics.proxy.controls.iter().any(|r| r.0 == "check"));
        view(&ctx, &mut a, &id, teaching::View::Example);
        assert!(a.metrics.proxy.controls.iter().any(|r| r.0 == "fill"));
        assert!(a.metrics.proxy.controls.iter().any(|r| r.0 == "inspect"));
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        assert!(a.metrics.proxy.controls.iter().any(|r| r.0 == "editor"));
        assert!(!a.metrics.proxy.controls.iter().any(|r| r.0 == "fill"));
        assert!(!a.metrics.proxy.worked_answer_visible);
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        view(&ctx, &mut a, &id, teaching::View::Introduction);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.proxy.worked_answer_visible);
        assert!(a.metrics.proxy.controls.iter().any(|r| r.0 == "check"));
    }
}
#[test]
fn practice_fill_does_not_save_apply_or_complete_the_assignment_and_reset_is_local() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        let original = session(&mut a, &id).assignment.clone();
        view(&ctx, &mut a, &id, teaching::View::Example);
        let world = session(&mut a, &id).practice.world.clone();
        tap(&ctx, &mut a, "fill");
        assert_eq!(session(&mut a, &id).practice.world, world);
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(
            a.metrics
                .proxy
                .feedback
                .iter()
                .any(|s| s.contains("unsaved"))
        );
        for key in ["save", "syntax", "apply", "check"] {
            tap(&ctx, &mut a, key);
        }
        assert!(
            a.metrics
                .proxy
                .summary
                .as_ref()
                .unwrap()
                .starts_with("All request")
        );
        assert_eq!(session(&mut a, &id).assignment, original);
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[a.selected.unwrap()]
        ));
        tap(&ctx, &mut a, "reset-example");
        assert!(!session(&mut a, &id).practice.checked());
        assert_eq!(session(&mut a, &id).assignment, original);
    }
}
#[test]
fn actual_editor_reports_partial_passes_requires_application_and_invalidates_edits() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.proxy.cases.iter().any(|c| c.1));
        assert!(a.metrics.proxy.cases.iter().any(|c| !c.1));
        assert!(!a.metrics.completion);
        let config = required(&a);
        replace(&ctx, &mut a, &config);
        assert_eq!(session(&mut a, &id).assignment.editor, config);
        tap(&ctx, &mut a, "save");
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(
            a.metrics
                .proxy
                .feedback
                .iter()
                .any(|s| s.starts_with("Applied policy needs attention"))
        );
        tap(&ctx, &mut a, "apply");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert!(a.metrics.proxy.cases.iter().all(|c| c.1));
        replace(&ctx, &mut a, "http_access allow all\n");
        assert!(!a.metrics.completion);
        assert!(!session(&mut a, &id).assignment.checked());
        assert!(
            a.metrics
                .proxy
                .summary
                .as_ref()
                .unwrap()
                .contains("not checked")
        );
    }
}
#[test]
fn unsupported_saved_syntax_is_explained_application_is_atomic_and_retry_works() {
    let (ctx, mut a, id) = opened(0);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    let before = session(&mut a, &id).assignment.world.applied().to_owned();
    replace(&ctx, &mut a, "include /etc/passwd\n");
    tap(&ctx, &mut a, "save");
    tap(&ctx, &mut a, "syntax");
    assert!(a.notice.as_ref().unwrap().contains("Only acl"));
    tap(&ctx, &mut a, "apply");
    assert_eq!(session(&mut a, &id).assignment.world.applied(), before);
    tap(&ctx, &mut a, "check");
    assert!(
        a.metrics
            .proxy
            .feedback
            .iter()
            .any(|s| s.contains("Saved configuration") && s.contains("Line 1"))
    );
    let config = required(&a);
    replace(&ctx, &mut a, &config);
    for key in ["save", "apply", "check"] {
        tap(&ctx, &mut a, key);
    }
    assert!(a.metrics.completion);
}
#[test]
fn hardcoding_all_named_clients_keeps_case_passes_but_fails_the_full_policy_clearly() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        let config = if n == 0 {
            required(&a).replace("10.44.16.0/20", "10.44.16.1 10.44.31.254")
        } else {
            required(&a).replace("10.70.8.0/24", "10.70.8.12")
        };
        replace(&ctx, &mut a, &config);
        for key in ["save", "apply", "check"] {
            tap(&ctx, &mut a, key);
        }
        assert!(a.metrics.proxy.cases.iter().all(|c| c.1));
        assert_eq!(a.metrics.proxy.cases.len(), 7);
        let witnesses = a
            .metrics
            .proxy
            .feedback
            .iter()
            .filter(|s| s.contains("whole required range"))
            .collect::<Vec<_>>();
        assert_eq!(
            witnesses.len(),
            1,
            "identical saved/applied errors should be combined"
        );
        let feedback = rect(&a, "policy-saved");
        let check = rect(&a, "check");
        assert_eq!(feedback, rect(&a, "policy-applied"));
        assert!(feedback.top() >= check.bottom());
        assert!(
            feedback.bottom() - check.bottom()
                < ctx.style().text_styles[&egui::TextStyle::Body].size * 14.
        );
        assert!(!a.metrics.completion);
        assert!(
            a.metrics
                .proxy
                .summary
                .as_ref()
                .unwrap()
                .starts_with("Needs attention")
        );
        assert!(
            a.metrics
                .proxy
                .feedback
                .iter()
                .any(|s| s.contains("whole required range") && s.contains("must be"))
        );
    }
}
#[test]
fn keyboard_enter_in_the_editor_does_not_save_or_apply_and_tab_retains_navigation() {
    let (ctx, mut a, id) = opened(0);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    tap(&ctx, &mut a, "editor");
    let field = a.metrics.proxy.fields[0].2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(field));
    let world = session(&mut a, &id).assignment.world.clone();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert_eq!(session(&mut a, &id).assignment.world, world);
    assert!(!session(&mut a, &id).assignment.checked());
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(field));
}
#[test]
fn editor_scrollbars_do_not_move_save_buttons_across_supported_sizes_and_themes() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        for (theme_id, scale, width) in [
            ("lab-navy", 1., 1920.),
            ("lab-navy", 1.5, 1100.),
            ("light", 2., 946.),
        ] {
            a.theme_id = theme_id.into();
            theme::apply(&ctx, theme_id);
            a.scale = scale;
            session(&mut a, &id)
                .assignment
                .edit("# short\nhttp_access allow all".into())
                .unwrap();
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 1080., vec![]);
            }
            let before = rect(&a, "save");
            let outer = rect(&a, "editor-viewport");
            session(&mut a, &id)
                .assignment
                .edit(format!(
                    "{}\n{}",
                    "# many lines\n".repeat(50),
                    "# very long data ".repeat(100)
                ))
                .unwrap();
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 1080., vec![]);
            }
            let after = rect(&a, "save");
            let new_outer = rect(&a, "editor-viewport");
            assert!(
                (before.top() - after.top()).abs() < 1.,
                "{n}/{theme_id}/{scale}: {before:?} {after:?}"
            );
            assert!((outer.height() - new_outer.height()).abs() < 1.);
            let main = a.metrics.main_viewport.unwrap();
            for key in [
                "editor-viewport",
                "save",
                "syntax",
                "apply",
                "check",
                "request",
            ] {
                let r = rect(&a, key);
                assert!(
                    r.left() >= main.left() - 1. && r.right() <= main.right() + 1.,
                    "{n}/{theme_id}/{scale}/{key}: {r:?} outside{main:?}"
                );
            }
            assert!(!a.metrics.proxy.worked_answer_visible);
        }
    }
}
#[test]
fn wide_workspace_really_splits_while_narrow_practice_keeps_one_two_three_order() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Example);
        for width in [1200., 1920.] {
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 1080., vec![]);
            }
            assert!(a.metrics.proxy.split_layout, "wide {width}");
            assert!(rect(&a, "request").left() > rect(&a, "editor-viewport").right());
            assert_eq!(a.metrics.proxy.guidance_order, [0, 1, 2]);
        }
        a.scale = 2.;
        theme::apply(&ctx, "light");
        for _ in 0..4 {
            frame(&ctx, &mut a, 946., 1080., vec![]);
        }
        assert!(!a.metrics.proxy.split_layout);
        assert!(rect(&a, "request").top() > rect(&a, "editor-viewport").bottom());
        assert_eq!(a.metrics.proxy.guidance_order, [0, 1, 2]);
        let entry = lesson_presentation::entry(&a.scenarios[a.selected.unwrap()]).unwrap();
        assert!(entry.practice[1].action.contains("Save configuration"));
        assert!(entry.practice[1].action.contains("Apply saved policy"));
        assert!(entry.practice[2].action.contains("Request"));
    }
}
