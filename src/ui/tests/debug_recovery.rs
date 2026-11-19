//! Focused native widget behavior for the two authored repair projects.
use super::*;
const IDS: [&str; 2] = ["shell-debug-status", "shell-recovery-ledger"];
pub(super) fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    (ctx, a, i)
}
pub(super) fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1500., 5000., events);
}
pub(super) fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..4 {
        paint(ctx, a, vec![]);
    }
}
pub(super) fn tap(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
pub(super) fn type_editor(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = a.metrics.editor_viewport.unwrap();
    let band = egui::Rect::from_min_size(
        r.min + egui::vec2(3., 3.),
        egui::vec2((r.width() - 6.).max(1.), 22.),
    );
    tap(ctx, a, band);
    assert_eq!(ctx.memory(|m| m.focused()), a.metrics.focus);
    paint(
        ctx,
        a,
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
            egui::Event::Text(text.into()),
        ],
    );
    settle(ctx, a);
}
pub(super) fn run(ctx: &egui::Context, a: &mut LabApp) {
    let r = a.metrics.check.unwrap();
    tap(ctx, a, r);
    for _ in 0..5000 {
        if a.script_job.is_none() {
            settle(ctx, a);
            return;
        }
        paint(ctx, a, vec![]);
    }
    panic!("Finite project did not finish within bounded UI polling");
}
#[test]
fn debug_recovery_native_edit_run_partial_wrong_correct_invalidation_stop_retry_and_resume() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        a.set_script_view(i, teaching::View::Assignment);
        settle(&ctx, &mut a);
        let s = a.scenarios[i].clone();
        run(&ctx, &mut a);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(a.metrics.script_feedback.is_some());
        type_editor(&ctx, &mut a, &s.solution[0]);
        run(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        assert!(a.metrics.completion);
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let p = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        type_editor(&ctx, &mut a, "while true; do :; done\n");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        let r = a.metrics.check.unwrap();
        tap(&ctx, &mut a, r);
        assert!(a.script_job.is_some());
        let stop = a.metrics.stop.unwrap();
        tap(&ctx, &mut a, stop);
        assert!(a.script_job.is_none());
        assert!(!a.metrics.completion);
        type_editor(&ctx, &mut a, &s.solution[0]);
        run(&ctx, &mut a);
        assert!(a.metrics.completion);
    }
}
#[test]
fn debug_recovery_native_examples_run_change_retry_and_keep_full_assignment_drafts() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let lesson = s.teaching.as_ref().unwrap();
        let original = a.progress.attempts[id].state.clone();
        a.set_script_view(i, teaching::View::Example);
        settle(&ctx, &mut a);
        run(&ctx, &mut a);
        let result = a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .as_ref()
            .unwrap();
        assert!(lesson.fixture.accepts(result));
        assert_eq!(a.progress.attempts[id].state, original);
        let broken = if id.ends_with("debug-status") {
            lesson.example.replace("result=$?", "result=0")
        } else {
            lesson.example.replace("fern moss", "elm moss")
        };
        type_editor(&ctx, &mut a, &broken);
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
        type_editor(&ctx, &mut a, &lesson.example);
        run(&ctx, &mut a);
        assert_eq!(a.progress.attempts[id].state, original);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn debug_recovery_three_painted_views_hide_assignment_solutions_and_remain_readable() {
    fn collect(s: &egui::Shape, o: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                // A one-line legacy heading may carry an unused justification flag.
                // Wrapped teaching prose must never stretch word spacing.
                if t.galley.rows.len() > 1 && t.galley.job.text.len() > 80 {
                    assert!(
                        !t.galley.job.justify,
                        "Stretched wrapped text: {}",
                        t.galley.job.text
                    );
                }
                o.push_str(&t.galley.job.text);
                o.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, o)
                }
            }
            _ => {}
        }
    }
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.set_script_view(i, view);
            let mut text = String::new();
            for _ in 0..4 {
                let o = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1500., 6000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                text.clear();
                for shape in o.shapes {
                    collect(&shape.shape, &mut text);
                }
            }
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains(&s.teaching.as_ref().unwrap().concepts[0].title));
                    assert!(text.contains("Expected preview"));
                }
                teaching::View::Example => {
                    assert!(text.contains("Actual result"));
                    assert!(text.contains("Not run."));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("Expected preview"));
                    assert!(!text.contains("One supported approach follows"));
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
                        frame(&ctx, &mut a, width, 950., vec![]);
                    }
                    let vp = a.metrics.main_viewport.unwrap();
                    let editor = a.metrics.editor_viewport.unwrap();
                    assert!(
                        editor.left() >= vp.left() - 2. && editor.right() <= vp.right() + 2.,
                        "{id} {scale} {width} {editor:?} {vp:?}"
                    );
                }
            }
        }
    }
}
