//! Real native editor and view checks for the optional worker-lookup activity.
use super::{
    debug_recovery::{opened, paint, settle, tap, type_editor},
    *,
};
use dlms_learning_lab::environment_workspace as env;
const ID: &str = "lpic1-env-tool-path";
fn session(a: &LabApp) -> &env::Session {
    let engine::State::Environment { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn view(a: &mut LabApp, v: teaching::View) {
    let engine::State::Environment { simulation } =
        &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn finish(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3000 {
        paint(ctx, a, vec![]);
        if a.environment_run.is_none() {
            settle(ctx, a);
            return;
        }
    }
    panic!("bounded lookup run did not finish");
}
fn run(ctx: &egui::Context, a: &mut LabApp) {
    tap(ctx, a, a.metrics.check.unwrap());
    finish(ctx, a);
}
fn text(ctx: &egui::Context, a: &mut LabApp, width: f32) -> String {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
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
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 6000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut s = String::new();
    for shape in out.shapes {
        collect(&shape.shape, &mut s)
    }
    s
}
#[test]
fn lookup_teaching_actual_alternate_partial_and_edit_feedback_use_real_widgets() {
    let (ctx, mut a, i) = opened(ID);
    settle(&ctx, &mut a);
    let s = a.scenarios[i].clone();
    let original = session(&a).assignment.clone();
    let intro = text(&ctx, &mut a, 1500.);
    assert!(
        intro.contains("WHAT YOU’LL LEARN")
            && intro.contains("Expected worker lookup")
            && intro.contains("Worked setup script")
    );
    assert!(!intro.contains("Run setup + lookup"));
    view(&mut a, teaching::View::Example);
    settle(&ctx, &mut a);
    let practice = text(&ctx, &mut a, 1500.);
    assert!(
        practice.contains("Worker looks up:")
            && practice.contains("Run example")
            && practice.contains("Not run.")
    );
    run(&ctx, &mut a);
    let actual = text(&ctx, &mut a, 1500.);
    assert!(
        actual.contains("Worker lookup · actual · no utility run")
            && actual.contains("Matches provider requirement")
            && actual.contains("Child report · actual")
    );
    assert_eq!(session(&a).assignment, original);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    let worked = session(&a).definition.example.starter.clone();
    type_editor(&ctx, &mut a, &worked.replace("$TOOL_DIR", "$LEGACY_DIR"));
    assert!(session(&a).example.result.is_none());
    run(&ctx, &mut a);
    let bad = text(&ctx, &mut a, 1500.);
    assert!(bad.contains("not an approved provider") && bad.contains("Needs an edit"));
    view(&mut a, teaching::View::Assignment);
    settle(&ctx, &mut a);
    let fresh = text(&ctx, &mut a, 1500.);
    assert!(fresh.contains("Run setup + lookup") && fresh.contains("Requirements"));
    assert!(
        !fresh.contains("Expected worker lookup")
            && !fresh.contains("Worked setup script")
            && !fresh.contains("export PATH=\"$TOOL_DIR")
    );
    type_editor(&ctx, &mut a, "PATH=\"$TOOL_DIR:$BASE_PATH\"\n");
    run(&ctx, &mut a);
    let partial = text(&ctx, &mut a, 1500.);
    assert!(partial.contains("3/6") && partial.contains("needs a defined exported PATH"));
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    type_editor(&ctx, &mut a, "export PATH=\"$BASE_PATH:$TOOL_DIR\"\n");
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
    finish(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[ID], &s));
    let good = text(&ctx, &mut a, 1500.);
    assert!(good.contains("6/6") && good.contains("Matches provider requirement"));
    type_editor(&ctx, &mut a, "export PATH=\"$BASE_PATH\"\n");
    assert!(session(&a).assignment.result.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    let stale = text(&ctx, &mut a, 1500.);
    assert!(!stale.contains("Matches provider requirement"));
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    let challenge = text(&ctx, &mut a, 1500.);
    assert!(
        !challenge.contains("Try example · ungraded")
            && !challenge.contains("Expected worker lookup")
            && !challenge.contains("Worked setup script")
    );
}
#[test]
fn lookup_and_legacy_controls_reflow_with_light_dark_and_100_150_200_text() {
    let (ctx, mut a, _) = opened(ID);
    view(&mut a, teaching::View::Assignment);
    settle(&ctx, &mut a);
    type_editor(&ctx, &mut a, "export PATH=\"$TOOL_DIR:$BASE_PATH\"\n");
    run(&ctx, &mut a);
    for light in [false, true] {
        let t = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap()
            .id
            .clone();
        theme::apply(&ctx, &t);
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [760., 1500.] {
                for _ in 0..3 {
                    let rendered = text(&ctx, &mut a, width);
                    assert!(
                        rendered.contains("Worker lookup · actual")
                            && rendered.contains("Matches provider requirement")
                    );
                }
                let check = a.metrics.check.unwrap();
                assert!(
                    check.left() >= -1. && check.right() <= ctx.content_rect().right() + 2.,
                    "{light} {scale} {width} {check:?}"
                );
                assert!(check.height() > 15.);
            }
        }
    }
    assert_eq!(
        environment_lookup::run_label(false, false),
        "Run setup + child"
    );
    assert_eq!(
        environment_lookup::run_label(false, true),
        "Run setup + lookup"
    );
    assert_eq!(environment_lookup::run_label(true, true), "Run example");
}
