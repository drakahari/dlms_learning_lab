use super::*;
use dlms_learning_lab::{
    engine::Goal, wlan_identity::ManagementProtection, wlan_workspace as work,
};
const IDS: [&str; 3] = [
    "ccna-wlan-ssid-repair",
    "ccna-wlan-wpa2-personal",
    "ccna-wlan-wpa3-handover",
];
fn opened(n: usize) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    assert!(a._smoke_profile.is_some());
    a.smoke = false;
    assert_eq!((a.scenarios.len(), a.units.len()), (439, 172));
    a.launch(a.scenarios.iter().position(|s| s.id == IDS[n]).unwrap());
    draw(&ctx, &mut a);
    (ctx, a, IDS[n].into())
}
fn session<'a>(a: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::WlanProfiles { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn definition(a: &LabApp) -> work::Definition {
    let Goal::WlanProfiles { definition } = &a.scenarios[a.selected.unwrap()].goal else {
        panic!()
    };
    (**definition).clone()
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .wlan
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; notice {:?}", a.notice))
        .1
}
fn has(a: &LabApp, key: &str) -> bool {
    a.metrics.wlan.controls.iter().any(|r| r.0 == key)
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..20 {
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
        for _ in 0..8 {
            draw(ctx, a);
        }
    }
    panic!("WLAN control {key} did not become visible")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key);
    click(ctx, a, r);
    draw(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, field_key: &str, text: &str) {
    tap(ctx, a, field_key);
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
#[test]
fn each_real_lesson_has_three_views_task_specific_controls_and_no_unsolicited_challenge_answer() {
    for n in 0..3 {
        let (ctx, mut a, id) = opened(n);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(!has(&a, "check"));
        assert!(
            lesson_presentation::entry(&a.scenarios[a.selected.unwrap()])
                .unwrap()
                .concepts
                .len()
                >= 4
        );
        session(&mut a, &id).view = teaching::View::Example;
        draw(&ctx, &mut a);
        assert!(has(&a, "fill") && has(&a, "check"));
        session(&mut a, &id).view = teaching::View::Assignment;
        draw(&ctx, &mut a);
        assert!(!has(&a, "fill"));
        assert!(has(&a, "profiles"));
        assert!(!a.metrics.wlan.worked_answer_visible);
        assert_eq!(a.metrics.wlan.security_facts_visible, n != 0);
        if n == 0 {
            assert!(has(&a, "ssid-staff") && has(&a, "enabled-staff"));
            assert!(!has(&a, "authentication-staff") && !has(&a, "passphrase-staff"));
        }
        if n == 1 {
            assert!(
                has(&a, "authentication-staff")
                    && has(&a, "passphrase-staff")
                    && has(&a, "protection-staff")
            );
            assert!(!has(&a, "ssid-staff"));
        }
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(has(&a, "check"));
        assert!(!has(&a, "fill"));
        assert!(!a.metrics.wlan.worked_answer_visible);
    }
}
#[test]
fn actual_fill_and_check_keep_example_credit_out_of_the_assignment() {
    let (ctx, mut a, id) = opened(1);
    let d = definition(&a);
    session(&mut a, &id).view = teaching::View::Example;
    draw(&ctx, &mut a);
    let before = session(&mut a, &id).assignment.clone();
    tap(&ctx, &mut a, "fill");
    assert_eq!(session(&mut a, &id).practice.fields, d.example.reference);
    assert!(!session(&mut a, &id).practice.checked());
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    assert!(session(&mut a, &id).practice.solved(&d.example));
    assert_eq!(session(&mut a, &id).assignment, before);
    assert!(!a.metrics.completion);
    assert_eq!(
        a.metrics.wlan.summary.as_deref(),
        Some("Example meets the requirements")
    );
}
#[test]
fn native_name_typing_checkbox_and_check_complete_then_edits_clear_success() {
    let (ctx, mut a, id) = opened(0);
    let d = definition(&a);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    replace(
        &ctx,
        &mut a,
        "ssid-staff",
        &d.initial.reference[0].configuration.ssid,
    );
    assert_eq!(
        session(&mut a, &id).assignment.fields[0].configuration.ssid,
        d.initial.reference[0].configuration.ssid
    );
    tap(&ctx, &mut a, "enabled-staff");
    assert!(
        session(&mut a, &id).assignment.fields[0]
            .configuration
            .enabled
    );
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert!(a.metrics.wlan.case_results.iter().all(|r| r.1));
    replace(&ctx, &mut a, "ssid-staff", "Training-room");
    assert!(!a.metrics.completion);
    assert!(a.metrics.wlan.summary.is_none());
    assert!(a.metrics.wlan.case_results.is_empty());
}
#[test]
fn native_pmf_dropdown_defers_invalid_wpa3_and_clears_all_stale_case_results() {
    let (ctx, mut a, id) = opened(2);
    let d = definition(&a);
    session(&mut a, &id).view = teaching::View::Assignment;
    session(&mut a, &id)
        .assignment
        .fill_reference(&d.initial)
        .unwrap();
    session(&mut a, &id).assignment.check(&d.initial).unwrap();
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    tap(&ctx, &mut a, "protection-staff");
    let choice = a
        .metrics
        .wlan
        .choices
        .iter()
        .find(|r| r.0 == "pmf-Optional")
        .unwrap()
        .1;
    click(&ctx, &mut a, choice);
    draw(&ctx, &mut a);
    assert_eq!(
        session(&mut a, &id).assignment.fields[0]
            .configuration
            .management_protection,
        ManagementProtection::Optional
    );
    assert!(!a.metrics.completion);
    assert!(a.metrics.wlan.case_results.is_empty());
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.wlan.case_results.is_empty());
    assert!(
        a.metrics
            .wlan
            .summary
            .as_ref()
            .unwrap()
            .starts_with("Needs attention")
    );
}
#[test]
fn light_dark_wide_narrow_and_enlarged_controls_keep_shared_heights_and_horizontal_bounds() {
    for n in 0..3 {
        let (ctx, mut a, id) = opened(n);
        session(&mut a, &id).view = teaching::View::Assignment;
        for (width, scale, light) in [
            (1920., 1., false),
            (1920., 1.5, false),
            (946., 2., true),
            (640., 2., true),
        ] {
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
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 1080., vec![]);
            }
            let region = a.metrics.main_scroll_region.unwrap();
            if width == 1920. && scale == 1. {
                assert!(
                    rect(&a, "profiles").left() > rect(&a, "configuration").right(),
                    "A desktop-width workspace must put profile facts beside the controls"
                );
                if n == 0 {
                    assert!(
                        (rect(&a, "enabled-staff").left() - rect(&a, "ssid-staff").left()).abs()
                            < 1.,
                        "Availability and text controls must share a left edge"
                    );
                }
            }
            for (key, r, _) in &a.metrics.wlan.controls {
                assert!(
                    r.left() >= region.left() - 1. && r.right() <= region.right() + 1.,
                    "{n} {width} {scale} {key}: {r:?}, {region:?}"
                );
            }
            for (key, r, _, _) in &a.metrics.wlan.fields {
                let hint = &a
                    .metrics
                    .wlan
                    .field_hints
                    .iter()
                    .find(|h| h.0 == *key)
                    .unwrap()
                    .1;
                let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                let color = ctx.style().visuals.text_color();
                let text_width =
                    ctx.fonts_mut(|f| f.layout_no_wrap(hint.clone(), font, color).size().x);
                assert!(
                    text_width + 2. * controls::PAD_X as f32 <= r.width() + 1.,
                    "Visible format cue {hint:?} must fit the actual field"
                );
                assert!(
                    r.height()
                        >= egui::TextStyle::Body.resolve(&ctx.style()).size
                            + 2. * controls::PAD_Y as f32
                );
            }
            assert!(!a.metrics.wlan.worked_answer_visible);
        }
    }
}
#[test]
fn keyboard_tab_leaves_a_field_without_mutating_unrelated_profile_settings() {
    let (ctx, mut a, id) = opened(0);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    let before = session(&mut a, &id).assignment.fields[0]
        .configuration
        .clone();
    tap(&ctx, &mut a, "ssid-staff");
    let field = a
        .metrics
        .wlan
        .fields
        .iter()
        .find(|r| r.0 == "ssid-staff")
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    assert!(ctx.memory(|m| m.focused()).is_some());
    assert_eq!(
        session(&mut a, &id).assignment.fields[0].configuration,
        before
    );
}

#[test]
fn primary_check_paints_readable_text_on_its_actual_accent_background_in_all_28_themes() {
    fn collect<'a>(
        shape: &'a egui::Shape,
        texts: &mut Vec<&'a egui::epaint::TextShape>,
        backgrounds: &mut Vec<&'a egui::epaint::RectShape>,
    ) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == "Check configuration" => texts.push(t),
            egui::Shape::Rect(r) => backgrounds.push(r),
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, texts, backgrounds);
                }
            }
            _ => (),
        }
    }
    assert_eq!(theme::inventory().len(), 28);
    let (ctx, mut a, id) = opened(1);
    for t in theme::inventory() {
        a.theme_id = t.id.clone();
        theme::apply(&ctx, &t.id);
        for view in [teaching::View::Example, teaching::View::Assignment] {
            session(&mut a, &id).view = view;
            for _ in 0..3 {
                frame(&ctx, &mut a, 1920., 1600., vec![]);
            }
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1920., 1600.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            let button = rect(&a, "check");
            let mut text = Vec::new();
            let mut backgrounds = Vec::new();
            for shape in &output.shapes {
                collect(&shape.shape, &mut text, &mut backgrounds);
            }
            assert_eq!(text.len(), 1, "{} {view:?}: actual check label", t.id);
            let background = backgrounds
                .iter()
                .find(|r| {
                    r.rect.center().distance(button.center()) < 1.
                        && (r.rect.width() - button.width()).abs() < 1.
                        && (r.rect.height() - button.height()).abs() < 1.
                })
                .expect("Actual primary-button painted background");
            assert_eq!(background.fill, t.roles.accent);
            let text = text[0];
            let mut glyphs = 0;
            assert!(button.expand(1.).contains_rect(text.visual_bounding_rect()));
            for row in &text.galley.rows {
                for vertex in &row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()] {
                    let color = text.override_text_color.unwrap_or_else(|| {
                        if vertex.color == egui::Color32::PLACEHOLDER {
                            text.fallback_color
                        } else {
                            vertex.color
                        }
                    });
                    assert!(
                        theme::contrast(color, background.fill) >= 4.5,
                        "{} {view:?}: painted Check configuration contrast {} ({color:?} on {:?})",
                        t.id,
                        theme::contrast(color, background.fill),
                        background.fill
                    );
                    glyphs += 1;
                }
            }
            assert!(glyphs > 0);
        }
    }
}
