use super::*;
use dlms_learning_lab::service_workspace as work;
const IDS: [&str; 2] = ["lpic1-service-client-targets", "lpic1-services-file-repair"];
fn opened(n: usize) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let current = LabApp::current_catalog_for_session(true, false, Some("service-clients-batch"));
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("service-clients-wide-batch")
    ));
    assert!(
        current,
        "The actual capture route must select the current catalog, not legacy201"
    );
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, current);
    a.smoke = false;
    assert!(a._smoke_profile.is_some());
    assert_eq!((a.scenarios.len(), a.units.len()), (439, 172));
    a.launch(a.scenarios.iter().position(|s| s.id == IDS[n]).unwrap());
    draw(&ctx, &mut a);
    (ctx, a, IDS[n].into())
}
fn session<'a>(a: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::ServiceClients { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!("proxy state")
    };
    simulation
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .services
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
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    draw(ctx, a);
    let r = reveal(ctx, a, key);
    click(ctx, a, r);
    draw(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, field: &str, text: &str) {
    tap(ctx, a, field);
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

#[test]
fn three_views_preserve_subject_tools_and_hide_independent_solutions() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(
            !lesson_presentation::entry(&a.scenarios[a.selected.unwrap()])
                .unwrap()
                .concepts
                .is_empty()
        );
        assert!(!a.metrics.services.controls.iter().any(|r| r.0 == "check"));
        view(&ctx, &mut a, &id, teaching::View::Example);
        assert_eq!(a.metrics.services.guidance_order, [0, 1, 2]);
        assert!(a.metrics.services.controls.iter().any(|r| r.0 == "fill"));
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        assert!(!a.metrics.services.worked_answer_visible);
        assert!(!a.metrics.services.controls.iter().any(|r| r.0 == "fill"));
        assert_eq!(
            a.metrics.services.controls.iter().any(|r| r.0 == "editor"),
            n == 1
        );
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.services.worked_answer_visible);
    }
}
#[test]
fn native_practice_actions_are_ungraded_fill_does_not_check_and_reset_is_local() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        let assignment = session(&mut a, &id).assignment.clone();
        view(&ctx, &mut a, &id, teaching::View::Example);
        tap(&ctx, &mut a, "fill");
        assert!(!session(&mut a, &id).practice.checked());
        if n == 1 {
            tap(&ctx, &mut a, "check");
            assert!(
                a.metrics
                    .services
                    .feedback
                    .iter()
                    .any(|x| x.contains("Unsaved"))
            );
            tap(&ctx, &mut a, "save");
        }
        tap(&ctx, &mut a, "check");
        let source = a.scenarios[a.selected.unwrap()].clone();
        let def = work::source(&source).unwrap();
        assert!(
            def.example
                .report(&session(&mut a, &id).practice)
                .unwrap()
                .passed()
        );
        assert!(!a.metrics.completion);
        assert_eq!(session(&mut a, &id).assignment, assignment);
        tap(&ctx, &mut a, "reset-example");
        assert_eq!(
            session(&mut a, &id).practice,
            work::Draft::new(&def.example).unwrap()
        );
        assert_eq!(session(&mut a, &id).assignment, assignment);
    }
}
#[test]
fn actual_editor_save_check_and_keyboard_changes_clear_success_and_explain_reference_failure() {
    let (ctx, mut a, id) = opened(1);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    let source = a.scenarios[a.selected.unwrap()].clone();
    let def = work::source(&source).unwrap();
    replace(&ctx, &mut a, "editor", &def.initial.required_database);
    tap(&ctx, &mut a, "save");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert!(a.metrics.services.cases.iter().all(|r| r.1));
    replace(
        &ctx,
        &mut a,
        "editor",
        &def.initial.required_database.replace("https 443/tcp\n", ""),
    );
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .services
            .summary
            .as_ref()
            .unwrap()
            .contains("Not checked")
    );
    tap(&ctx, &mut a, "save");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.services.cases.iter().all(|r| r.1));
    assert!(
        a.metrics
            .services
            .feedback
            .iter()
            .any(|s| s.contains("https"))
    );
    assert!(
        a.metrics
            .services
            .summary
            .as_ref()
            .unwrap()
            .contains("Needs attention")
    );
}
#[test]
fn command_input_uses_real_keyboard_and_virtual_results_without_awarding_credit() {
    let (ctx, mut a, id) = opened(0);
    view(&ctx, &mut a, &id, teaching::View::Assignment);
    replace(&ctx, &mut a, "terminal-input", "getent services domain/udp");
    tap(&ctx, &mut a, "run");
    assert!(
        session(&mut a, &id)
            .assignment
            .transcript
            .contains("53/udp")
    );
    assert!(!session(&mut a, &id).assignment.checked() && !a.metrics.completion);
    replace(&ctx, &mut a, "terminal-input", "cat /private");
    tap(&ctx, &mut a, "run");
    assert!(
        session(&mut a, &id)
            .assignment
            .transcript
            .contains("Unsupported")
    );
}
#[test]
fn fields_keep_centered_text_and_fit_at_supported_scales_and_light_dark_styles() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        view(&ctx, &mut a, &id, teaching::View::Assignment);
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                let selected = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &selected);
                for width in [1200., 720.] {
                    frame(&ctx, &mut a, width, 850., vec![]);
                    frame(&ctx, &mut a, width, 850., vec![]);
                    let font = egui::TextStyle::Body.resolve(&ctx.style());
                    let pitch = ctx.fonts_mut(|f| f.row_height(&font)).ceil() + 16.;
                    for prefix in ["listener-role-", "listener-value-"] {
                        let rows = a
                            .metrics
                            .services
                            .controls
                            .iter()
                            .filter(|r| r.0.starts_with(prefix))
                            .map(|r| r.1)
                            .collect::<Vec<_>>();
                        assert_eq!(rows.len(), 3);
                        for pair in rows.windows(2) {
                            assert!(
                                pair[1].top() - pair[0].top() <= pitch + 1.,
                                "Read-only listener rows must use table metrics, not input click-target height"
                            );
                            assert!((pair[1].left() - pair[0].left()).abs() < 1.);
                        }
                    }
                    for (key, r, _, clip) in &a.metrics.services.fields {
                        if key != "editor" {
                            assert!(r.width() > 20. && r.height() > 20.);
                            assert!(
                                r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                                "{key} scale{scale} width{width}: {r:?}/{clip:?}"
                            );
                        }
                    }
                    assert!(!a.metrics.services.worked_answer_visible);
                }
            }
        }
    }
}
