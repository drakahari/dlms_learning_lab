use super::*;
const ID: &str = "ccna-ipv6-modified-eui64";
fn opened() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    (ctx, a, i)
}
#[test]
fn eui64_three_views_keep_controls_in_bounds_at_five_scales_and_two_themes() {
    let (ctx, mut a, i) = opened();
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        for light in [false, true] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
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
            for (w, h) in [(1500., 1000.), (760., 900.)] {
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    a.set_ipv6_view(i, view);
                    for routes in [false, true] {
                        {
                            let ws = a
                                .progress
                                .attempts
                                .get_mut(ID)
                                .unwrap()
                                .workspace
                                .as_mut()
                                .unwrap();
                            ws.ipv6_draft.as_mut().unwrap().show_routes = routes;
                            ws.ipv6_learning.as_mut().unwrap().draft.show_routes = routes;
                        }
                        for _ in 0..3 {
                            frame(&ctx, &mut a, w, h, vec![]);
                        }
                        for r in a
                            .metrics
                            .inputs
                            .iter()
                            .chain(a.metrics.check.iter())
                            .chain(a.metrics.lesson_tabs.iter())
                        {
                            assert!(
                                r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                                "{view:?} {w} {scale} {r:?}"
                            );
                            assert!(r.height() > 0.);
                        }
                        assert!(!a.metrics.guide);
                        assert!(!a.metrics.help_visible);
                    }
                }
            }
        }
    }
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert_eq!(a.ipv6_view(i), teaching::View::Assignment);
    assert!(a.metrics.lesson_tabs.is_empty());
}
#[test]
fn eui64_keyboard_retry_partial_feedback_and_selected_reset_use_real_widgets() {
    let (ctx, mut a, i) = opened();
    a.set_ipv6_view(i, teaching::View::Assignment);
    for _ in 0..3 {
        draw(&ctx, &mut a);
    }
    assert!(ctx.memory(|m| m.has_focus(a.metrics.focus.unwrap())));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text(
            "2001:db8:74:0:425:96ff:feab:cdef/64".into(),
        )],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let checks = engine::checks(&a.progress.attempts[ID].state, &a.scenarios[i].goal);
    assert!(checks[0].0);
    assert!(checks.iter().any(|r| !r.0));
    {
        let ws = a
            .progress
            .attempts
            .get_mut(ID)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        ws.ipv6_draft
            .as_mut()
            .unwrap()
            .devices
            .get_mut("Client")
            .unwrap()
            .routes[0]
            .next_hop = "fe80::1".into();
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
            },
            egui::Event::Text("malformed".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    check(&ctx, &mut a);
    assert!(a.notice.as_ref().unwrap().contains("Malformed"));
    let focus = a.metrics.focus.unwrap();
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(focus)));
    a.reset_attempt(i);
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
}
#[test]
fn eui64_worked_practice_does_not_fill_or_grade_the_independent_assignment() {
    let (ctx, mut a, i) = opened();
    let before = a.progress.attempts[ID].clone();
    let lesson = a.scenarios[i].ipv6_teaching.clone().unwrap();
    a.set_ipv6_view(i, teaching::View::Example);
    let l = a
        .progress
        .attempts
        .get_mut(ID)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .ipv6_learning
        .as_mut()
        .unwrap();
    l.fill_example(&lesson).unwrap();
    assert!(!l.checked);
    l.apply().unwrap();
    assert!(l.checks(&lesson).iter().all(|r| r.0));
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[ID].state, before.state);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    a.set_ipv6_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .as_ref()
            .unwrap()
            .devices["Client"]
            .interfaces["lan"]
            .address
            .is_empty()
    );
}

#[test]
fn eui64_full_address_fits_the_normal_width_single_line_control() {
    let (ctx, mut a, i) = opened();
    a.set_ipv6_view(i, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1500., 1000., vec![]);
    }
    let field = a.metrics.inputs[0];
    let font = egui::TextStyle::Body.resolve(&ctx.style());
    let required = ctx.fonts_mut(|f| {
        f.layout_no_wrap(
            "2001:db8:74:0:425:96ff:feab:cdef/64".into(),
            font,
            egui::Color32::WHITE,
        )
        .size()
        .x
    }) + 2. * controls::PAD_X as f32;
    assert!(
        field.width() + 1. >= required,
        "full address needs{required}, actual{field:?}"
    );
}
