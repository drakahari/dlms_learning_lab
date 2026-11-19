//! Actual editor/Run controls for the new project; no scripted state injection.
use super::debug_recovery::{opened, run, settle, type_editor};
use super::*;
#[test]
fn managed_config_native_edits_runs_stop_retry_and_separate_practice() {
    let (ctx, mut a, i) = opened("shell-managed-config");
    let s = a.scenarios[i].clone();
    let id = &s.id;
    let lesson = s.teaching.as_ref().unwrap();
    a.set_script_view(i, teaching::View::Example);
    settle(&ctx, &mut a);
    run(&ctx, &mut a);
    assert!(
        lesson.fixture.accepts(
            a.progress.attempts[id]
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_result
                .as_ref()
                .unwrap()
        )
    );
    assert_eq!(a.progress.attempts[id].state, s.initial);
    let changed = lesson
        .example
        .replace("while IFS= read -r", "while read -r");
    type_editor(&ctx, &mut a, &changed);
    assert!(
        a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .is_none()
    );
    run(&ctx, &mut a);
    assert!(
        !lesson.fixture.accepts(
            a.progress.attempts[id]
                .workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_result
                .as_ref()
                .unwrap()
        )
    );
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    a.set_script_view(i, teaching::View::Assignment);
    settle(&ctx, &mut a);
    assert_eq!(a.progress.attempts[id].state, s.initial);
    run(&ctx, &mut a);
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    assert!(a.metrics.script_feedback.is_some());
    type_editor(&ctx, &mut a, &s.solution[0]);
    run(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.save();
    assert!(a.storage_error.is_none());
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    type_editor(&ctx, &mut a, "while true; do :; done\n");
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    let r = a.metrics.check.unwrap();
    debug_recovery::tap(&ctx, &mut a, r);
    assert!(a.script_job.is_some());
    let stop = a.metrics.stop.unwrap();
    debug_recovery::tap(&ctx, &mut a, stop);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
    type_editor(&ctx, &mut a, &s.solution[0]);
    run(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.set_script_view(i, teaching::View::Example);
    settle(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source,
        changed
    );
}
#[test]
fn managed_config_three_views_use_reviewed_text_and_do_not_reveal_assignment_answers() {
    fn text(sh: &egui::Shape, out: &mut String) {
        match sh {
            egui::Shape::Text(t) => {
                if t.galley.rows.len() > 1 {
                    assert!(!t.galley.job.justify)
                }
                out.push_str(&t.galley.job.text);
                out.push('\n')
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    text(s, out)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a, i) = opened("shell-managed-config");
    let s = a.scenarios[i].clone();
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        a.set_script_view(i, view);
        settle(&ctx, &mut a);
        let o = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 8000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut t = String::new();
        for sh in o.shapes {
            text(&sh.shape, &mut t)
        }
        match view {
            teaching::View::Introduction => {
                assert!(t.contains("Keep literal lines"));
                assert!(t.contains("Expected candidate"))
            }
            teaching::View::Example => {
                assert!(t.contains("Find a whitespace-preservation mistake"));
                assert!(t.contains("Not run."))
            }
            teaching::View::Assignment => {
                assert!(t.contains("Requirements"));
                assert!(!t.contains("Expected candidate"));
                assert!(!t.contains("before=$(cat service.conf"))
            }
        }
    }
    a.set_script_view(i, teaching::View::Assignment);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
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
            for width in [760., 1600.] {
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 950., vec![])
                }
                let vp = a.metrics.main_viewport.unwrap();
                let editor = a.metrics.editor_viewport.unwrap();
                assert!(
                    editor.left() >= vp.left() - 2. && editor.right() <= vp.right() + 2.,
                    "{scale}/{width}: {editor:?} {vp:?}"
                )
            }
        }
    }
    assert_eq!(s.id, "shell-managed-config");
}
