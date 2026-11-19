//! Actual native widgets; injected input is technical evidence, not a novice study.
use super::*;
use dlms_learning_lab::{ids_rule_source as binding, ids_rule_workspace as work};
const ID: &str = "lpic2-ids-header-direction";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 12000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::IdsRule { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::IdsRule { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .ids_rule
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            r,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {r:?} {clip:?}"
    );
    let p = r.center();
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    hit(ctx, a, key);
    let id = a
        .metrics
        .ids_rule
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
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
fn visible_text(ctx: &egui::Context, a: &mut LabApp) -> Vec<String> {
    fn collect(shape: &egui::epaint::Shape, text: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, text);
                }
            }
            egui::epaint::Shape::Text(t) => text.push(t.galley.job.text.clone()),
            _ => {}
        }
    }
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1450., 12000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for s in out.shapes {
        collect(&s.shape, &mut text);
    }
    text
}

fn perform(ctx: &egui::Context, a: &mut LabApp, action: &work::Action) {
    match action {
        work::Action::Edit { text } => edit(ctx, a, "editor", text),
        work::Action::Save => hit(ctx, a, "save"),
        work::Action::Replay => hit(ctx, a, "replay"),
        work::Action::Check => hit(ctx, a, "check"),
        work::Action::Select { .. } => panic!("Popup interaction covered by fixed native flow"),
    }
}
#[test]
fn ids_rule_widgets_grade_actual_work_and_invalidate_unsaved_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    assert!(session(&a).assignment.last_run.is_none());
    let before = visible_text(&ctx, &mut a);
    assert!(before.iter().any(|s| s.contains("no actual result yet")));
    hit(&ctx, &mut a, "replay");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let v: serde_json::Value =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        perform(
            &ctx,
            &mut a,
            &serde_json::from_value(v["action"].clone()).unwrap(),
        );
    }
    assert!(a.metrics.completion);
    assert!(session(&a).assignment.last_run.is_some());
    let saved = session(&a).assignment.saved.clone();
    edit(&ctx, &mut a, "editor", "unfinished rule");
    assert!(!a.metrics.completion);
    assert_eq!(session(&a).assignment.saved, saved);
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("Unsaved rule changes"))
    );
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "replay");
    hit(&ctx, &mut a, "check");
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("Syntax error:"))
    );
    edit(&ctx, &mut a, "editor", &saved);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}
#[test]
fn ids_rule_three_views_teach_then_isolate_practice_without_assignment_answers() {
    let (ctx, mut a, i) = open(teaching::View::Introduction);
    let text = visible_text(&ctx, &mut a);
    assert!(
        text.iter()
            .any(|s| s == "1. Read the header before the payload")
    );
    assert!(!text.iter().any(|s| s.starts_with("1. 1.")));
    for label in ["Matching example request", "Contrasting example record"] {
        assert!(
            text.iter().any(|s| s == label),
            "Missing annotated worked row {label}"
        );
    }
    let original = session(&a).assignment.clone();
    hit(&ctx, &mut a, "example");
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    for action in &d.worked {
        perform(&ctx, &mut a, action);
    }
    assert!(
        session(&a)
            .example
            .checks(&d.example)
            .iter()
            .all(|c| c.passed)
    );
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, original);
    let text = visible_text(&ctx, &mut a);
    assert!(
        !text
            .iter()
            .any(|s| s.contains("Separate worked example rule"))
    );
    assert!(
        !a.metrics
            .ids_rule
            .controls
            .iter()
            .any(|r| r.0 == "fill-worked")
    );
    hit(&ctx, &mut a, "introduction");
    assert!(a.metrics.ids_rule.controls.iter().any(|r| r.0 == "worked"));
}
#[test]
fn ids_rule_challenge_keeps_independent_editor_and_replay_without_unsolicited_worked_help() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in [
        "introduction",
        "example",
        "try-example",
        "worked",
        "fill-worked",
    ] {
        assert!(!a.metrics.ids_rule.controls.iter().any(|r| r.0 == key));
    }
    for key in [
        "assignment-content",
        "editor",
        "save",
        "sample",
        "replay",
        "check",
    ] {
        assert!(
            a.metrics.ids_rule.controls.iter().any(|r| r.0 == key),
            "{key}"
        );
    }
    assert!(!a.metrics.completion);
}
#[test]
fn ids_rule_native_control_geometry_reflows_at_supported_scales_in_both_theme_kinds() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for light in [false, true] {
        let id = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap()
            .id
            .clone();
        theme::apply(&ctx, &id);
        for scale in [1., 1.5, 2.] {
            ctx.set_zoom_factor(scale);
            a.scale = scale;
            for width in [1450., 700.] {
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 12000., vec![]);
                }
                let controls = &a.metrics.ids_rule.controls;
                for key in [
                    "editor-fixed",
                    "editor-viewport",
                    "save",
                    "sample",
                    "replay",
                    "check",
                ] {
                    let (_, r, clip) = controls.iter().find(|r| r.0 == key).unwrap();
                    assert!(r.is_positive());
                    assert!(
                        r.min.x >= clip.min.x - 1. && r.max.x <= clip.max.x + 1.,
                        "{light} {scale} {width} {key} {r:?} {clip:?}"
                    );
                }
                let save = controls.iter().find(|r| r.0 == "save").unwrap().1;
                let editor = controls.iter().find(|r| r.0 == "editor-fixed").unwrap().1;
                assert!(
                    save.top() >= editor.bottom() - 1.,
                    "Save must remain outside the editor scrollbar gutter"
                );
            }
        }
    }
}
