use super::*;
use dlms_learning_lab::{engine::Goal, wireless_channel_workspace as work};
const IDS: [&str; 2] = [
    "ccna-wireless-neighbour-channels",
    "ccna-wireless-multisurvey-plan",
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
    let State::WirelessChannels { simulation } =
        &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .wireless_channel
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; notice {:?}", a.notice))
        .1
}
fn has(a: &LabApp, key: &str) -> bool {
    a.metrics
        .wireless_channel
        .controls
        .iter()
        .any(|r| r.0 == key)
}
fn definition(a: &LabApp) -> work::Definition {
    let Goal::WirelessChannels { definition } = &a.scenarios[a.selected.unwrap()].goal else {
        panic!()
    };
    (**definition).clone()
}
#[test]
fn actual_registered_teaching_and_independent_controls_have_separate_scopes() {
    let (ctx, mut a, _) = opened(0);
    for id in IDS {
        a.launch(a.scenarios.iter().position(|s| s.id == id).unwrap());
        draw(&ctx, &mut a);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(has(&a, "channel-strip"));
        assert!(!has(&a, "check"));
        session(&mut a, id).view = teaching::View::Example;
        draw(&ctx, &mut a);
        assert!(has(&a, "fill") && has(&a, "check"));
        session(&mut a, id).view = teaching::View::Assignment;
        draw(&ctx, &mut a);
        assert!(!has(&a, "fill"));
        assert!(has(&a, "surveys"));
        assert!(!a.metrics.wireless_channel.worked_answer_visible);
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!has(&a, "fill"));
        assert!(!a.metrics.wireless_channel.worked_answer_visible);
    }
}
#[test]
fn fill_is_inert_and_example_check_cannot_complete_the_independent_plan() {
    let (ctx, mut a, id) = opened(0);
    let d = definition(&a);
    session(&mut a, &id).view = teaching::View::Example;
    draw(&ctx, &mut a);
    let before = session(&mut a, &id).assignment.clone();
    let r = rect(&a, "fill");
    click(&ctx, &mut a, r);
    assert_eq!(session(&mut a, &id).practice.fields, d.example.reference);
    assert!(!session(&mut a, &id).practice.checked());
    assert!(!a.metrics.completion);
    let r = rect(&a, "check");
    click(&ctx, &mut a, r);
    assert!(session(&mut a, &id).practice.solved(&d.example));
    assert_eq!(session(&mut a, &id).assignment, before);
    assert!(!a.metrics.completion);
    assert_eq!(
        a.metrics.wireless_channel.summary.as_deref(),
        Some("Example meets the plan")
    );
}
#[test]
fn partial_survey_results_name_the_remaining_failure_and_edits_invalidate_success() {
    let (ctx, mut a, id) = opened(1);
    let d = definition(&a);
    session(&mut a, &id).view = teaching::View::Assignment;
    for (radio, value) in [("north", "1"), ("east", "6"), ("south", "1"), ("west", "6")] {
        session(&mut a, &id)
            .assignment
            .set(&d.initial, radio, value.into())
            .unwrap();
    }
    session(&mut a, &id).assignment.check(&d.initial).unwrap();
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics.wireless_channel.summary.as_deref(),
        Some("Needs attention · 1 condition fails")
    );
    assert!(!a.metrics.completion);
    assert!(has(&a, "feedback-survey-partition-open-1"));
    assert!(has(&a, "passing"));
    session(&mut a, &id)
        .assignment
        .fill_reference(&d.initial)
        .unwrap();
    session(&mut a, &id).assignment.check(&d.initial).unwrap();
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    session(&mut a, &id)
        .assignment
        .set(&d.initial, "north", "2".into())
        .unwrap();
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(a.metrics.wireless_channel.summary.is_none());
    assert!(a.metrics.wireless_channel.field_feedback.is_empty());
    let r = rect(&a, "check");
    click(&ctx, &mut a, r);
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .wireless_channel
            .field_feedback
            .iter()
            .any(|(radio, text)| radio == "north" && text.contains("site's 1 / 6 / 11"))
    );
}
#[test]
fn blank_and_malformed_feedback_is_rendered_near_the_named_radio_without_default_answers() {
    let (ctx, mut a, id) = opened(0);
    let d = definition(&a);
    session(&mut a, &id).view = teaching::View::Assignment;
    for (value, needle) in [("", "Enter a channel"), ("6.0", "whole channel")] {
        session(&mut a, &id)
            .assignment
            .set(&d.initial, "reception", value.into())
            .unwrap();
        session(&mut a, &id).assignment.check(&d.initial).unwrap();
        draw(&ctx, &mut a);
        assert!(
            a.metrics
                .wireless_channel
                .field_feedback
                .iter()
                .any(|(radio, text)| radio == "reception" && text.contains(needle))
        );
        assert!(!a.metrics.completion);
        assert!(a.metrics.wireless_channel.surveys_deferred);
        assert!(!a.metrics.wireless_channel.worked_answer_visible);
    }
    assert!(!has(&a, "field-neighbour-west"));
    assert!(has(&a, "fixed-neighbour-west"));
}
#[test]
fn shared_columns_fields_and_labels_reflow_without_horizontal_clipping_or_text_shrinking() {
    let (ctx, mut a, id) = opened(0);
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
        let field = rect(&a, "field-reception");
        let label = rect(&a, "label-reception");
        let region = a.metrics.main_scroll_region.unwrap();
        assert!(field.right() <= region.right() + 1.);
        assert!(
            field.height()
                >= controls::PAD_Y as f32 * 2. + egui::TextStyle::Body.resolve(&ctx.style()).size
        );
        if label.right() < field.left() {
            assert!((label.center().y - field.center().y).abs() < 1.5);
        }
        assert!(rect(&a, "fixed-neighbour-west").right() <= region.right() + 1.);
        assert!(!a.metrics.wireless_channel.worked_answer_visible);
    }
}
#[test]
fn keyboard_tab_and_text_edit_update_only_the_focused_managed_radio() {
    let (ctx, mut a, id) = opened(1);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    let r = rect(&a, "field-north");
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    let select_all = egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    };
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![select_all, egui::Event::Text("11".into())],
    );
    draw(&ctx, &mut a);
    let fields = &session(&mut a, &id).assignment.fields;
    assert_eq!(
        fields.iter().find(|f| f.radio == "north").unwrap().value,
        "1"
    );
    assert_eq!(
        fields.iter().find(|f| f.radio == "east").unwrap().value,
        "11"
    );
    assert!(!a.metrics.completion);
}

#[test]
fn actual_native_clear_helper_deletes_selected_text_and_removes_current_success() {
    let (ctx, mut app, id) = opened(0);
    let d = definition(&app);
    session(&mut app, &id).view = teaching::View::Assignment;
    session(&mut app, &id)
        .assignment
        .fill_reference(&d.initial)
        .unwrap();
    session(&mut app, &id).assignment.check(&d.initial).unwrap();
    draw(&ctx, &mut app);
    assert!(app.metrics.completion);
    let r = rect(&app, "field-reception");
    click(&ctx, &mut app, r);
    app.queue_wireless_field_text("");
    let events = std::mem::take(&mut app.pending_events);
    frame(&ctx, &mut app, 1200., 850., events);
    draw(&ctx, &mut app);
    assert_eq!(session(&mut app, &id).assignment.fields[0].value, "");
    assert!(!app.metrics.completion);
    assert!(app.metrics.wireless_channel.summary.is_none());
    let r = rect(&app, "check");
    click(&ctx, &mut app, r);
    assert!(app.metrics.wireless_channel.surveys_deferred);
    assert!(
        app.metrics
            .wireless_channel
            .field_feedback
            .iter()
            .any(|(_, s)| s.contains("Enter a channel"))
    );
}
