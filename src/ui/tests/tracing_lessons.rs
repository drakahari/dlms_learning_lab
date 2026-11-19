//! Actual native widgets reuse the approved script workspace; no new wizard.
use super::debug_recovery::{opened, paint, run, settle, tap, type_editor};
use super::*;
const IDS: [&str; 2] = ["shell-trace-delivery", "shell-trace-fallback"];
fn text(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32) -> String {
    fn collect(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w, h),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut value = String::new();
    for s in output.shapes {
        collect(&s.shape, &mut value);
    }
    value
}
#[test]
fn tracing_three_views_teach_prerequisites_and_keep_independent_answers_hidden() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let initial = a.progress.attempts[id].state.clone();
        let l = s.teaching.as_ref().unwrap();
        a.set_script_view(i, teaching::View::Introduction);
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(
            t.contains("WHAT YOU’LL LEARN")
                && t.contains("Worked example")
                && t.contains(&l.concepts[0].title)
        );
        a.set_script_view(i, teaching::View::Example);
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(t.contains("Run example") && t.contains("Not run."));
        for step in &lesson_presentation::entry(&s).unwrap().practice {
            assert!(
                t.contains(&step.title),
                "Missing practice title: {}",
                step.title
            );
            assert!(
                t.contains(&step.action),
                "Missing practice action: {}",
                step.action
            );
        }
        assert!(a.metrics.editor_viewport.is_some());
        run(&ctx, &mut a);
        assert_eq!(a.progress.attempts[id].state, initial);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        let changed = if id.ends_with("delivery") {
            l.example.replace("\"$place\"", "$place")
        } else {
            l.example.replace("first='gallery.log'", "first='live.log'")
        };
        type_editor(&ctx, &mut a, &changed);
        run(&ctx, &mut a);
        assert!(
            !l.fixture.accepts(
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
        type_editor(&ctx, &mut a, &l.example);
        run(&ctx, &mut a);
        assert!(
            l.fixture.accepts(
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
        a.set_script_view(i, teaching::View::Assignment);
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(t.contains("YOUR TASK") && t.contains("Run script"));
        assert!(!t.contains("Example script"));
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(!t.contains("Worked example") && !t.contains("Run example"));
        assert_eq!(a.progress.attempts[id].state, initial);
    }
}
#[test]
fn older_repair_projects_keep_every_authored_practice_action_visible() {
    for id in ["shell-debug-status", "shell-recovery-ledger"] {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        a.set_script_view(i, teaching::View::Example);
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        let steps = &lesson_presentation::entry(&s).unwrap().practice;
        assert!(steps.len() >= 3);
        for step in steps {
            assert!(
                t.contains(&step.title),
                "Missing {} title: {}",
                id,
                step.title
            );
            assert!(
                t.contains(&step.action),
                "Missing {} action: {}",
                id,
                step.action
            );
        }
        assert!(t.contains("Run example") && t.contains("Open full assignment"));
        assert!(!a.metrics.completion);
    }
}
#[test]
fn tracing_keyboard_run_clear_feedback_partial_status_correct_resume_and_invalidation() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        a.set_script_view(i, teaching::View::Assignment);
        settle(&ctx, &mut a);
        let s = a.scenarios[i].clone();
        run(&ctx, &mut a);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        let trace = format!("set -x\n{}", s.solution[0]);
        type_editor(&ctx, &mut a, &trace);
        run(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(
            t.contains("The + lines are debugging traces") && t.contains("Standard error"),
            "Visible trace feedback: {t}"
        );
        assert!(!a.metrics.completion);
        type_editor(&ctx, &mut a, &s.solution[0]);
        // Keyboard execution from the actual focused multiline editor.
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            }],
        );
        for _ in 0..5000 {
            if a.script_job.is_none() {
                break;
            }
            paint(&ctx, &mut a, vec![]);
        }
        settle(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        assert!(a.metrics.completion);
        a.save();
        assert!(a.storage_error.is_none());
        let p = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        if id.ends_with("fallback") {
            let partial = s.solution[0].replace("cat < \"$chosen\"", "cat -- \"$chosen\"");
            type_editor(&ctx, &mut a, &partial);
            run(&ctx, &mut a);
            assert!(!progress::current_solved(&a.progress.attempts[id], &s));
            let first = a
                .metrics
                .script_case_rows
                .iter()
                .find(|(n, _)| *n == 0)
                .unwrap()
                .1;
            tap(&ctx, &mut a, first);
            let t = text(&ctx, &mut a, 1500., 5000.);
            assert!(t.contains("Case 1/12 · Pass") && t.contains("Run: 1 fail"));
        }
        type_editor(&ctx, &mut a, "while true; do :; done\n");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        let r = a.metrics.check.unwrap();
        tap(&ctx, &mut a, r);
        assert!(a.script_job.is_some());
        let stop = a.metrics.stop.unwrap();
        tap(&ctx, &mut a, stop);
        assert!(a.script_job.is_none());
        assert!(!a.metrics.completion);
    }
}
#[test]
fn tracing_workspace_reflows_without_changing_field_font_or_assignment_at_supported_scales() {
    let (ctx, mut a, i) = opened(IDS[1]);
    a.set_script_view(i, teaching::View::Assignment);
    let expected = a.progress.attempts[IDS[1]].state.clone();
    for light in [false, true] {
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
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [760., 1500.] {
                let _ = text(&ctx, &mut a, width, 5000.);
                let _ = text(&ctx, &mut a, width, 5000.);
                let editor = a.metrics.editor_viewport.unwrap();
                let action = a.metrics.check.unwrap();
                assert!(editor.width() > 0. && editor.height() > 0. && action.width() > 0.);
                assert!(editor.max.x <= ctx.content_rect().max.x + 1.);
                assert_eq!(a.progress.attempts[IDS[1]].state, expected);
                assert!(!a.metrics.completion);
            }
        }
    }
}
