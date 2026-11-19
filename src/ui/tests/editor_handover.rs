use super::debug_recovery::{paint, settle, tap, type_editor};
use super::*;
use dlms_learning_lab::{editor_handover as handover, engine::State};
const ID: &str = "lpic1-editor-handover";
fn opened() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let new = pack::load(include_str!(
        "../../../scenarios/editor-handover-lessons-v1.json"
    ))
    .unwrap()
    .remove(0);
    let i = if let Some(i) = a.scenarios.iter().position(|s| s.id == ID) {
        i
    } else {
        a.scenarios.push(new);
        a.scenarios.len() - 1
    };
    a.launch(i);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &handover::Session {
    let State::EditorHandover { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn view(a: &mut LabApp, v: teaching::View) {
    let State::EditorHandover { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn text(ctx: &egui::Context, a: &mut LabApp, w: f32) -> String {
    fn collect(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    for _ in 0..3 {
        frame(ctx, a, w, 7000., vec![]);
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w, 7000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut out = String::new();
    for s in output.shapes {
        collect(&s.shape, &mut out)
    }
    out
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    settle(ctx, a);
    let r = a
        .metrics
        .modal
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .1;
    tap(ctx, a, r);
}
fn finish(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..2000 {
        paint(ctx, a, vec![]);
        if a.editor_handover_run.is_none() {
            settle(ctx, a);
            return;
        }
    }
    panic!("Bounded setup did not finish");
}
fn keys(ctx: &egui::Context, a: &mut LabApp, value: &str) {
    settle(ctx, a);
    let (r, _, _) = a.metrics.modal.editor.unwrap();
    tap(
        ctx,
        a,
        egui::Rect::from_min_size(r.min + egui::vec2(10., 10.), egui::vec2(20., 20.)),
    );
    for c in value.chars() {
        let event = match c {
            '\x1b' | '\r' => egui::Event::Key {
                key: if c == '\r' {
                    egui::Key::Enter
                } else {
                    egui::Key::Escape
                },
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            },
            _ => egui::Event::Text(c.to_string()),
        };
        paint(ctx, a, vec![event]);
    }
    settle(ctx, a);
}
#[test]
fn handover_three_views_keep_guidance_separate_and_reflow_at_supported_scales() {
    for stage in ["editor-handover-batch", "modal-editing-batch"] {
        assert!(LabApp::current_catalog_for_session(
            true,
            false,
            Some(stage)
        ));
    }
    let (ctx, mut a, _) = opened();
    for (scale, width, light) in [
        (1., 1600., false),
        (1.15, 960., true),
        (1.25, 960., false),
        (1.5, 1600., true),
        (2., 960., false),
    ] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        a.theme_id = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap()
            .id
            .clone();
        theme::apply(&ctx, &a.theme_id);
        assert_eq!(ctx.style().visuals.dark_mode, !light);
        for v in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            view(&mut a, v);
            let t = text(&ctx, &mut a, width);
            match v {
                teaching::View::Introduction => {
                    assert!(t.contains("Worked example") && t.contains("Expected child report"));
                }
                teaching::View::Example => {
                    assert!(t.contains("Fill worked setup") && t.contains("Run setup"));
                    assert!(!t.contains("editor=vim\nteam=training team"));
                }
                teaching::View::Assignment => {
                    assert!(t.contains("Check handover") && t.contains("Requirements"));
                    assert!(!t.contains("Fill worked setup"));
                    assert!(!t.contains("export EDITOR=\"$PREFERRED_EDITOR\""));
                }
            }
            if v != teaching::View::Introduction {
                let r = a.metrics.editor_viewport.unwrap();
                assert!(
                    r.width() > 100. && r.right() <= ctx.content_rect().right() + 1.,
                    "{scale}/{width}: {r:?}"
                );
            }
        }
    }
}
#[test]
fn handover_actual_script_input_failure_retry_native_file_edit_and_stale_invalidation() {
    let (ctx, mut a, i) = opened();
    view(&mut a, teaching::View::Assignment);
    settle(&ctx, &mut a);
    type_editor(&ctx, &mut a, "if true; then\n");
    click(&ctx, &mut a, "run-setup");
    finish(&ctx, &mut a);
    assert!(text(&ctx, &mut a, 1500.).contains("Syntax error"));
    assert!(!a.metrics.completion);
    type_editor(&ctx, &mut a, "export -p\n");
    click(&ctx, &mut a, "run-setup");
    finish(&ctx, &mut a);
    assert!(text(&ctx, &mut a, 1500.).contains("Unsupported"));
    type_editor(&ctx, &mut a, "EDITOR=$PREFERRED_EDITOR\n");
    click(&ctx, &mut a, "run-setup");
    finish(&ctx, &mut a);
    assert!(text(&ctx, &mut a, 1500.).contains("exported"));
    assert!(!session(&a).ready(
        match &a.scenarios[i].goal {
            engine::Goal::EditorHandover { definition } => definition,
            _ => panic!(),
        },
        false
    ));
    click(&ctx, &mut a, "actual-setup");
    let comparison = text(&ctx, &mut a, 1500.);
    for outcome in [
        "No inherited editor · Needs attention",
        "Existing inherited editor · Passed",
        "Empty inherited editor · Passed",
    ] {
        assert!(
            comparison.contains(outcome),
            "Missing {outcome}: {comparison}"
        );
    }
    click(&ctx, &mut a, "actual-setup");
    type_editor(&ctx, &mut a, "EDITOR=$PREFERRED_EDITOR\nexport EDITOR\n");
    click(&ctx, &mut a, "run-setup");
    finish(&ctx, &mut a);
    click(&ctx, &mut a, "open-handover");
    assert_eq!(session(&a).assignment_launch.as_ref().unwrap().editor, "vi");
    click(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    keys(&ctx, &mut a, "/OWNER\rddGoOWNER platform\rREVIEW ready\x1b");
    click(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(text(&ctx, &mut a, 1500.).contains("Unsaved"));
    keys(&ctx, &mut a, ":w\r");
    click(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    assert!(a.metrics.completion);
    let file = session(&a).files.assignment.editors.clone();
    type_editor(&ctx, &mut a, "EDITOR=wrong\n");
    assert!(!a.metrics.completion);
    assert!(session(&a).assignment_launch.is_none());
    assert_eq!(session(&a).files.assignment.editors, file);
}
#[test]
fn handover_example_fill_is_inert_isolated_and_stop_and_challenge_preserve_drafts() {
    let (ctx, mut a, i) = opened();
    view(&mut a, teaching::View::Example);
    settle(&ctx, &mut a);
    let assignment = session(&a).setup.assignment.clone();
    let files = session(&a).files.assignment.clone();
    click(&ctx, &mut a, "worked-setup");
    assert!(session(&a).setup.example.result.is_none());
    assert!(session(&a).example_launch.is_none());
    assert_eq!(session(&a).setup.assignment, assignment);
    click(&ctx, &mut a, "run-setup");
    finish(&ctx, &mut a);
    click(&ctx, &mut a, "open-handover");
    keys(
        &ctx,
        &mut a,
        "/OWNER\rddGoOWNER training\rREVIEW practice\x1b:w\r",
    );
    click(&ctx, &mut a, "check");
    assert!(session(&a).files.practice.checked);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    assert_eq!(session(&a).files.assignment, files);
    type_editor(&ctx, &mut a, "while :; do :; done\n");
    click(&ctx, &mut a, "run-setup");
    assert!(a.editor_handover_run.is_some());
    click(&ctx, &mut a, "stop-setup");
    assert!(a.editor_handover_run.is_none());
    assert!(session(&a).setup.example.result.is_none());
    assert_eq!(session(&a).setup.example.source, "while :; do :; done\n");
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    let t = text(&ctx, &mut a, 1500.);
    assert!(
        !t.contains("Fill worked setup")
            && !t.contains("Worked example")
            && !t.contains("Try example · ungraded")
    );
    assert!(t.contains("Check handover"));
    assert_eq!(session(&a).setup.assignment, assignment);
}
