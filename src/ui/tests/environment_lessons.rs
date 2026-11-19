//! Actual native editor/input/controller checks; no learner work reaches the host.
use super::debug_recovery::{opened, paint, settle, tap, type_editor};
use super::*;
use dlms_learning_lab::environment_workspace as e;
const IDS: [&str; 2] = ["lpic1-env-child-settings", "lpic1-env-parent-note"];
fn view(a: &mut LabApp, id: &str, v: teaching::View) {
    let engine::State::Environment { simulation } =
        &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn session<'a>(a: &'a LabApp, id: &str) -> &'a e::Session {
    let engine::State::Environment { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn finish(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3000 {
        paint(ctx, a, vec![]);
        if a.environment_run.is_none() {
            settle(ctx, a);
            return;
        }
    }
    panic!("bounded environment run did not finish");
}
fn run(ctx: &egui::Context, a: &mut LabApp) {
    let r = a.metrics.check.unwrap();
    tap(ctx, a, r);
    finish(ctx, a);
}
fn text(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32) -> String {
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
    let out = ctx.run(
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
    for s in out.shapes {
        collect(&s.shape, &mut value)
    }
    value
}
#[test]
fn environment_three_screens_teach_then_isolate_actual_practice_from_assignment() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        settle(&ctx, &mut a);
        assert!(e::context(&a.progress.attempts[id], &a.scenarios[a.selected.unwrap()]).is_ok());
        let original = session(&a, id).assignment.clone();
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(
            t.contains("WHAT YOU’LL LEARN")
                && t.contains("Worked setup script")
                && t.contains("Expected child report")
        );
        view(&mut a, id, teaching::View::Example);
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(
            t.contains("Run example")
                && t.contains("Expected child report · original example")
                && t.contains("Not run.")
        );
        let s = a.scenarios[a.selected.unwrap()].clone();
        for p in &lesson_presentation::entry(&s).unwrap().practice {
            assert!(t.contains(&p.action), "missing action {}", p.action);
        }
        run(&ctx, &mut a);
        assert!(
            e::checks(
                &session(&a, id).definition.example,
                &session(&a, id).example
            )
            .iter()
            .all(|r| r.0)
        );
        let painted = text(&ctx, &mut a, 1500., 5000.);
        for name in ["HOST", "LOCATION", "STYLE"] {
            assert!(
                painted.lines().any(|line| line == name),
                "Parent table hid {name}: {painted}"
            );
        }
        assert_eq!(session(&a, id).assignment, original);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        let worked = session(&a, id).example.source.clone();
        let broken = if id.ends_with("parent-note") {
            worked.replace("export -n REMINDER", "REMINDER=''")
        } else {
            worked.replace("export LOCATION STYLE", "export STYLE")
        };
        type_editor(&ctx, &mut a, &broken);
        assert!(session(&a, id).example.result.is_none());
        run(&ctx, &mut a);
        assert!(
            e::checks(
                &session(&a, id).definition.example,
                &session(&a, id).example
            )
            .iter()
            .any(|r| !r.0)
        );
        assert_eq!(session(&a, id).assignment, original);
        type_editor(&ctx, &mut a, &worked);
        run(&ctx, &mut a);
        view(&mut a, id, teaching::View::Assignment);
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(t.contains("YOUR TASK") && t.contains("Run setup + child"));
        assert!(
            !t.contains("Expected child report · original example")
                && !t.contains("Worked setup script")
        );
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        settle(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(!t.contains("Try example · ungraded"));
    }
}
#[test]
fn environment_actual_edit_keyboard_retry_partial_feedback_success_and_invalidation() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        view(&mut a, id, teaching::View::Assignment);
        settle(&ctx, &mut a);
        run(&ctx, &mut a);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        type_editor(&ctx, &mut a, "if true; then\n");
        run(&ctx, &mut a);
        assert!(text(&ctx, &mut a, 1500., 5000.).contains("Syntax error"));
        type_editor(&ctx, &mut a, "export -p\n");
        run(&ctx, &mut a);
        assert!(text(&ctx, &mut a, 1500., 5000.).contains("Unsupported syntax or feature"));
        type_editor(&ctx, &mut a, &format!("{}exit 7\n", s.solution[0]));
        run(&ctx, &mut a);
        let t = text(&ctx, &mut a, 1500., 5000.);
        assert!(
            t.contains("Setup exit status: 7 (expected 0)") && t.contains("Child exit status: 0"),
            "Painted feedback: {t}\nRecord: {:?}",
            session(&a, id).assignment.result
        );
        type_editor(&ctx, &mut a, &s.solution[0]);
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
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        assert!(a.metrics.completion);
        type_editor(&ctx, &mut a, "SITE=wrong\n");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(!a.metrics.completion);
        type_editor(&ctx, &mut a, &s.solution[0]);
        run(&ctx, &mut a);
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let p = progress::load(&a.path, &a.scenarios).unwrap();
        assert!(!progress::current_solved(&p.attempts[id], &s));
        a.progress = p;
        finish(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn environment_stop_navigation_and_large_light_widgets_keep_drafts_and_no_late_credit() {
    let id = IDS[0];
    let (ctx, mut a, i) = opened(id);
    let s = a.scenarios[i].clone();
    view(&mut a, id, teaching::View::Assignment);
    settle(&ctx, &mut a);
    let long = ":\n".repeat(2000);
    type_editor(&ctx, &mut a, &long);
    let r = a.metrics.check.unwrap();
    tap(&ctx, &mut a, r);
    assert!(a.environment_run.is_some());
    let r = a.metrics.stop.unwrap();
    tap(&ctx, &mut a, r);
    assert!(a.environment_run.is_none());
    assert!(session(&a, id).assignment.result.is_none());
    assert_eq!(session(&a, id).assignment.source, long);
    let r = a.metrics.check.unwrap();
    tap(&ctx, &mut a, r);
    assert!(a.environment_run.is_some());
    view(&mut a, id, teaching::View::Example);
    paint(&ctx, &mut a, vec![]);
    assert!(a.environment_run.is_none());
    assert!(session(&a, id).assignment.result.is_none());
    a.selected = None;
    paint(&ctx, &mut a, vec![]);
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    a.selected = Some(i);
    view(&mut a, id, teaching::View::Assignment);
    let light = theme::inventory()
        .iter()
        .find(|t| t.light)
        .unwrap()
        .id
        .clone();
    theme::apply(&ctx, &light);
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        settle(&ctx, &mut a);
        for width in [760., 1500.] {
            let _ = text(&ctx, &mut a, width, 5000.);
            let editor = a.metrics.editor_viewport.unwrap();
            let check = a.metrics.check.unwrap();
            assert!(editor.width() > 100. && check.width() > 50. && check.height() > 15.);
            assert!(
                check.left() >= -1. && check.right() <= ctx.content_rect().right() + 2.,
                "control clipped: {check:?} width={width} scale={scale}"
            );
        }
    }
}
