//! In-process egui proposals; these do not establish native acceptance.
use super::*;
const ID: &str = "ccna-gateway";
fn client_app(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    a.set_client_view(i, view);
    (ctx, a, i)
}
fn learning(a: &LabApp) -> &dlms_learning_lab::client_teaching::Learning {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .client_learning
        .as_ref()
        .unwrap()
}
fn learning_mut(a: &mut LabApp) -> &mut dlms_learning_lab::client_teaching::Learning {
    a.progress
        .attempts
        .get_mut(ID)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .client_learning
        .as_mut()
        .unwrap()
}
fn prefix(m: &Metrics) -> Option<egui::Rect> {
    m.client_fields
        .iter()
        .find(|(key, _, _)| key == "prefix")
        .map(|(_, _, r)| *r)
}
fn gateway(m: &Metrics) -> Option<egui::Rect> {
    m.client_fields
        .iter()
        .find(|(key, _, _)| key == "gateway")
        .map(|(_, _, r)| *r)
}
fn replace_field(
    ctx: &egui::Context,
    a: &mut LabApp,
    target: fn(&Metrics) -> Option<egui::Rect>,
    text: &str,
) {
    a.focus_input = false;
    accounts_ui_click(ctx, a, target);
    let id = a
        .metrics
        .client_fields
        .iter()
        .find(|(_, _, r)| Some(*r) == target(&a.metrics))
        .unwrap()
        .1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    draw(ctx, a);
}
#[test]
fn client_native_example_fill_insert_apply_enter_experiment_and_assignment_are_isolated() {
    let (ctx, mut a, i) = client_app(teaching::View::Introduction);
    draw(&ctx, &mut a);
    assert!(a.metrics.terminal.is_none() && a.metrics.check.is_none());
    let original = a.progress.attempts[ID].state.clone();
    accounts_ui_click(&ctx, &mut a, |m| m.example_action);
    assert_eq!(a.client_view(i), teaching::View::Example);
    let initial = learning(&a).client;
    accounts_ui_click(&ctx, &mut a, |m| m.command_insert);
    assert_eq!(learning(&a).client, initial);
    assert!(learning(&a).transcript.is_empty());
    // Run the inserted read-only action using actual Enter in its separate input.
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert!(!learning(&a).transcript.is_empty());
    assert_eq!(learning(&a).client, initial);
    accounts_ui_click(&ctx, &mut a, |m| m.example_action);
    assert_eq!(learning(&a).client, initial);
    assert!(!learning(&a).draft.matches(&initial));
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(
        learning(&a).checked
            && learning(&a)
                .checks(a.scenarios[i].client_teaching.as_ref().unwrap())
                .iter()
                .all(|c| c.0)
    );
    assert_eq!(a.progress.attempts[ID].state, original);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    replace_field(&ctx, &mut a, gateway, "198.51.100.130");
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert_eq!(
        learning(&a)
            .checks(a.scenarios[i].client_teaching.as_ref().unwrap())
            .iter()
            .map(|c| c.0)
            .collect::<Vec<_>>(),
        [true, true, false]
    );
    replace_field(&ctx, &mut a, gateway, "198.51.100.129");
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(
        learning(&a)
            .checks(a.scenarios[i].client_teaching.as_ref().unwrap())
            .iter()
            .all(|c| c.0)
    );
    accounts_ui_click(&ctx, &mut a, |m| m.assignment_action);
    assert_eq!(a.client_view(i), teaching::View::Assignment);
    assert_eq!(a.progress.attempts[ID].state, original);
}
#[test]
fn client_native_assignment_typing_atomic_retry_and_modal_keyboard_safety() {
    let (ctx, mut a, i) = client_app(teaching::View::Assignment);
    draw(&ctx, &mut a);
    replace_field(&ctx, &mut a, prefix, "26");
    replace_field(&ctx, &mut a, gateway, "192.168.40.65");
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    let solved = a.progress.attempts[ID].state.clone();
    replace_field(&ctx, &mut a, prefix, "31");
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    assert_eq!(a.progress.attempts[ID].state, solved);
    assert_eq!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .client_draft
            .as_ref()
            .unwrap()
            .prefix,
        "31"
    );
    let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
    let field = a
        .metrics
        .client_fields
        .iter()
        .find(|(key, _, _)| key == "prefix")
        .unwrap()
        .1;
    for settings in [false, true] {
        a.settings_open = settings;
        a.reset_pending = !settings;
        ctx.memory_mut(|m| m.request_focus(field));
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                acl_ctrl(egui::Key::A),
                egui::Event::Text("0".into()),
                key(egui::Key::Enter, false),
            ],
        );
        assert_eq!(
            serde_json::to_value(&a.progress.attempts[ID]).unwrap(),
            before
        );
    }
    a.settings_open = false;
    a.reset_pending = false;
    draw(&ctx, &mut a);
    replace_field(&ctx, &mut a, prefix, "26");
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn client_editor_mode_ids_undo_and_saved_raw_context_are_separate() {
    let (ctx, mut a, i) = client_app(teaching::View::Assignment);
    draw(&ctx, &mut a);
    replace_field(&ctx, &mut a, prefix, "unfinished");
    let assignment_id = a.metrics.client_fields[1].1;
    let assignment_draft = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .client_draft
        .clone();
    a.set_client_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    replace_field(&ctx, &mut a, gateway, "unsubmitted");
    let example_id = a.metrics.client_fields[2].1;
    assert_ne!(assignment_id, example_id);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert_ne!(learning(&a).draft.gateway, "unsubmitted");
    assert_eq!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .client_draft,
        assignment_draft
    );
    learning_mut(&mut a).command = "ip ro".into();
    learning_mut(&mut a).draft.gateway = "unfinished example".into();
    let before = a.progress.attempts[ID].workspace.clone();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-client.json");
    progress::save(&path, &a.progress).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &a.scenarios).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    let (ctx2, mut reopened) = app();
    reopened.progress = loaded;
    reopened.launch(i);
    draw(&ctx2, &mut reopened);
    assert_eq!(
        serde_json::to_value(&reopened.progress.attempts[ID].workspace).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    assert_eq!(reopened.client_view(i), teaching::View::Example);
}
fn assert_visible_run_text_fits(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32) {
    let mut visible = false;
    for _ in 0..20 {
        let r = a.metrics.client_run.expect("practice Run rendered");
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if viewport.contains_rect(r) {
            visible = true;
            break;
        }
        let dy = if r.bottom() > viewport.bottom() {
            viewport.bottom() - r.bottom()
        } else {
            viewport.top() - r.top()
        };
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(viewport.left() + 1., viewport.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..4 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    assert!(
        visible,
        "Run must be revealed before examining actual painted text"
    );
    let scale = a.scale;
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w / scale, h / scale),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    fn collect(shape: &egui::Shape, found: &mut Vec<egui::Rect>) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == "Run" => {
                found.push(egui::Rect::from_min_size(t.pos, t.galley.size()))
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, found);
                }
            }
            _ => {}
        }
    }
    let mut text = Vec::new();
    for shape in &output.shapes {
        collect(&shape.shape, &mut text);
    }
    assert_eq!(text.len(), 1, "actual painted practice Run label");
    assert!(
        a.metrics.client_run.unwrap().contains_rect(text[0]),
        "Run text must fit its actual Response at scale {scale}: {:?} vs {:?}",
        text[0],
        a.metrics.client_run
    );
}
#[test]
fn client_three_views_and_challenge_keep_controls_in_columns_at_all_text_scales() {
    for scale in [1., 1.5, 2.] {
        for (w, h) in [(1200., 850.), (850., 600.)] {
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                let (ctx, mut a, i) = client_app(view);
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                let right = ctx.content_rect().right();
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.terminal.iter())
                    .chain(a.metrics.client_action.iter())
                    .chain(a.metrics.client_run.iter())
                {
                    assert!(
                        r.left() >= -1. && r.right() <= right + 1.,
                        "{scale} {w} {view:?}: {r:?} vs {right}"
                    );
                }
                if view != teaching::View::Introduction {
                    assert_eq!(a.metrics.client_fields.len(), 3);
                    let x = a.metrics.client_fields[0].2.left();
                    assert!(
                        a.metrics
                            .client_fields
                            .iter()
                            .all(|(_, _, r)| (r.left() - x).abs() < 0.1)
                    );
                }
                if view == teaching::View::Example {
                    assert_visible_run_text_fits(&ctx, &mut a, w, h);
                }
                if view == teaching::View::Assignment {
                    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, w, h, vec![]);
                    }
                    assert!(a.metrics.lesson_tabs.is_empty() && a.metrics.command_insert.is_none());
                    assert_eq!(a.client_view(i), teaching::View::Assignment);
                }
            }
        }
    }
}
