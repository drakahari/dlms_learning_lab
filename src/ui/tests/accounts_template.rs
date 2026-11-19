use super::*;
use crate::ui::accounts_template_fixture as fixture;
use dlms_learning_lab::accounts_template_workspace;

fn setup(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.scenarios.len();
    a.scenarios.push(fixture::scenario());
    a.launch(i);
    a.set_accounts_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn body(m: &Metrics) -> Option<egui::Rect> {
    m.seed_fields
        .iter()
        .find(|(n, _, _)| n == "template-body")
        .map(|(_, r, _)| *r)
}
fn mode(m: &Metrics) -> Option<egui::Rect> {
    m.seed_fields
        .iter()
        .find(|(n, _, _)| n == "template-mode")
        .map(|(_, r, _)| *r)
}
fn replace(
    ctx: &egui::Context,
    a: &mut LabApp,
    field: fn(&Metrics) -> Option<egui::Rect>,
    text: &str,
) {
    accounts_ui_click(ctx, a, field);
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
fn accounts_template_native_edits_save_atomically_and_invalidate_current_success() {
    let (ctx, mut a, i) = setup(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    replace(&ctx, &mut a, body, fixture::ASSIGNMENT_TEXT);
    replace(&ctx, &mut a, mode, "0888");
    let before = a.progress.attempts[&s.id].state.clone();
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    assert_eq!(
        a.progress.attempts[&s.id].state, before,
        "invalid Save is atomic"
    );
    replace(&ctx, &mut a, mode, "640");
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    assert!(accounts_template_workspace::ready(
        &a.progress.attempts[&s.id],
        &s
    ));
    for c in &s.solution {
        accounts_ui_command(&ctx, &mut a, c);
    }
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(a.metrics.completion);
    replace(&ctx, &mut a, body, "unfinished retry\n");
    assert!(
        !a.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .checked
    );
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(!a.metrics.completion);
    assert!(
        engine::checks(&a.progress.attempts[&s.id].state, &s.goal)
            .iter()
            .all(|c| c.0),
        "pending draft invalidates current success without changing applied state"
    );
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    accounts_ui_click(&ctx, &mut a, |m| m.confirm_reset);
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    for scope in ["assignment", "example"] {
        let id = egui::Id::new(("account-template-save-status", (s.id.as_str(), scope)));
        assert!(ctx.data(|d| d.get_temp::<Result<(), String>>(id)).is_none());
    }
}
#[test]
fn accounts_template_native_practice_is_separate_resettable_and_never_auto_solved() {
    let (ctx, mut a, i) = setup(teaching::View::Introduction);
    let s = a.scenarios[i].clone();
    assert!(
        body(&a.metrics).is_none(),
        "Introduction has worked teaching, not graded editing"
    );
    let original = a.progress.attempts[&s.id].state.clone();
    accounts_ui_click(&ctx, &mut a, |m| m.example_action);
    replace(&ctx, &mut a, body, fixture::EXAMPLE_TEXT);
    replace(&ctx, &mut a, mode, "0640");
    accounts_ui_click(&ctx, &mut a, |m| m.editor_save);
    for c in &s.accounts_teaching.as_ref().unwrap().example {
        accounts_ui_command(&ctx, &mut a, c);
    }
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .accounts_learning
        .as_ref()
        .unwrap();
    let checks = l.checks(s.accounts_teaching.as_ref().unwrap());
    assert!(checks.iter().all(|c| c.0), "{checks:?}");
    assert_eq!(a.progress.attempts[&s.id].state, original);
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    accounts_ui_click(&ctx, &mut a, |m| m.example_reset);
    let status = egui::Id::new(("account-template-save-status", (s.id.as_str(), "example")));
    assert!(
        ctx.data(|d| d.get_temp::<Result<(), String>>(status))
            .is_none()
    );
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .accounts_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.simulation, s.accounts_teaching.as_ref().unwrap().fixture);
    assert!(
        l.template_drafts
            .as_ref()
            .unwrap()
            .matches_state(&l.simulation, &l.simulation)
    );
    accounts_ui_click(&ctx, &mut a, |m| m.assignment_action);
    assert_eq!(a.progress.attempts[&s.id].state, original);
}
#[test]
fn accounts_template_native_reflows_without_clipped_fields_or_lost_drafts() {
    for scale in [1., 1.5, 2.] {
        for width in [720., 1440.] {
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                let (ctx, mut a, i) = setup(view);
                a.scale = scale;
                for _ in 0..5 {
                    frame(&ctx, &mut a, width, 900., vec![]);
                }
                let before = a.progress.attempts[&a.scenarios[i].id].clone();
                for (_, r, _) in &a.metrics.seed_fields {
                    assert!(r.is_finite() && r.width() > 0.);
                    assert!(
                        r.left() >= ctx.content_rect().left() - 1.
                            && r.right() <= ctx.content_rect().right() + 1.,
                        "field clipped at {width}/{scale}/{view:?}: {r:?}"
                    );
                }
                assert!(accounts_template_workspace::context(&before, &a.scenarios[i]).is_ok());
                assert_eq!(
                    serde_json::to_vec(&a.progress.attempts[&a.scenarios[i].id]).unwrap(),
                    serde_json::to_vec(&before).unwrap()
                );
            }
        }
    }
}

#[test]
fn accounts_template_native_authored_views_have_real_actions_and_hide_assignment_method() {
    let (ctx, mut a, i) = super::debug_recovery::opened("lpic1-skeleton-handover");
    let s = a.scenarios[i].clone();
    fn collect(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, text)
                }
            }
            _ => {}
        }
    }
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        a.set_accounts_view(i, view);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 9000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut text = String::new();
        for shape in output.shapes {
            collect(&shape.shape, &mut text);
        }
        assert!(a.preview_view_ready(i, view));
        assert_eq!(
            a.preview_action_rect(),
            match view {
                teaching::View::Introduction => a.metrics.example_action,
                teaching::View::Example => a.metrics.accounts_run,
                teaching::View::Assignment => a.metrics.check,
            }
        );
        assert!(a.preview_action_rect().is_some());
        match view {
            teaching::View::Introduction => assert!(text.contains("DEMO_ZONE=harbor")),
            teaching::View::Example => assert!(
                text.contains("Starter-file editor")
                    && text.contains("Compare a later template change")
            ),
            teaching::View::Assignment => {
                assert!(text.contains("Requirements") && text.contains("Starter-file editor"));
                assert!(
                    !text.contains(&s.solution[1]),
                    "method must be hidden until requested"
                );
                assert!(!text.contains("DEMO_ZONE=harbor"));
            }
        }
        assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    }
    a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert_eq!(a.accounts_view(i), teaching::View::Assignment);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.preview_view_ready(i, teaching::View::Introduction));
}

#[test]
fn accounts_template_native_dropdown_uses_actual_popup_bounds_at_enlarged_text() {
    let (ctx, mut a, i) = super::debug_recovery::opened("lpic1-skeleton-handover");
    a.set_accounts_view(i, teaching::View::Assignment);
    a.scale = 1.5;
    for _ in 0..5 {
        frame(&ctx, &mut a, 1200., 850., vec![]);
    }
    accounts_ui_click(&ctx, &mut a, |m| {
        m.seed_fields
            .iter()
            .find(|(k, _, _)| k == "template-selector")
            .map(|(_, r, _)| *r)
    });
    let popup = a
        .metrics
        .accounts_template_popup
        .expect("actual native popup");
    let choices: Vec<_> = a
        .metrics
        .seed_fields
        .iter()
        .filter(|(k, _, _)| k.starts_with("template-choice:"))
        .cloned()
        .collect();
    assert_eq!(choices.len(), 3);
    for (_, r, _) in choices {
        assert!(
            smoke::smoke_control_visible(
                r,
                popup.intersect(ctx.content_rect()),
                ctx.pixels_per_point()
            ),
            "native popup choice clipped: {r:?}, {popup:?}"
        );
    }
    let row = a
        .metrics
        .seed_fields
        .iter()
        .find(|(k, _, _)| k == "template-choice:/etc/skel/welcome.txt")
        .unwrap()
        .1;
    click(&ctx, &mut a, row);
    assert_eq!(
        a.progress.attempts["lpic1-skeleton-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .accounts_template_drafts
            .as_ref()
            .unwrap()
            .selected,
        "/etc/skel/welcome.txt"
    );
}
