use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::startup_workspace as work;
const IDS: [&str; 2] = ["lpic1-shell-startup-priority", "lpic1-shell-startup-quiet"];
pub(super) fn opened(id: &str) -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a)
}
pub(super) fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        frame(ctx, a, 1200., 950., vec![]);
        let rect = if key == "reset-header" {
            a.metrics.reset.unwrap()
        } else if key == "confirm-reset" {
            a.metrics.confirm_reset.unwrap()
        } else {
            a.metrics
                .startup_files
                .controls
                .iter()
                .find(|c| c.0 == key)
                .unwrap_or_else(|| panic!("Missing {key}"))
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            return rect;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - rect.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Could not reveal {key}")
}
pub(super) fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let rect = target(ctx, a, key);
    click(ctx, a, rect);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn scope(a: &LabApp) -> &work::Scope {
    let s = &a.scenarios[a.selected.unwrap()];
    let State::StartupFiles { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "editor");
    click(ctx, a, r);
    let id = a.metrics.startup_files.editor.unwrap().1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    frame(
        ctx,
        a,
        1200.,
        950.,
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
            egui::Event::Paste(text.into()),
        ],
    );
    if text.is_empty() {
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![egui::Event::Key {
                key: egui::Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
    }
    assert_eq!(scope(a).files.0[&scope(a).selected].text, text);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..1024 {
        frame(ctx, a, 1200., 950., vec![]);
        if a.startup_files_run.is_none() {
            return;
        }
    }
    panic!("Startup checks did not settle");
}
fn result(a: &LabApp) -> Option<(usize, usize)> {
    a.metrics.startup_files.result.map(|r| (r.0, r.1))
}
#[test]
fn startup_three_views_teach_before_testing_without_exposing_assignment_answers() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        let s = a.scenarios[a.selected.unwrap()].clone();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Worked example") && text.contains("TOOL_LABEL"));
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "insert");
        let engine::Goal::StartupFiles { definition } = &s.goal else {
            panic!()
        };
        assert_eq!(scope(&a).files, definition.example_worked);
        assert!(scope(&a).tested.is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(
            result(&a),
            Some((
                if id.ends_with("quiet") { 5 } else { 6 },
                if id.ends_with("quiet") { 5 } else { 6 }
            ))
        );
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements"));
        assert!(!text.contains("TOOL_LABEL") && !text.contains("Insert worked files"));
        assert_eq!(scope(&a).files, definition.initial.starter);
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Worked example") && !text.contains("Try example"));
    }
}
#[test]
fn startup_editor_keyboard_actual_partial_wrong_correct_damage_retry_and_resume() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        let s = a.scenarios[a.selected.unwrap()].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        let (passed, total) = result(&a).unwrap();
        assert!(passed > 0 && passed < total);
        let good: dlms_learning_lab::shell::startup::Files =
            serde_json::from_str(s.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        for (file, data) in &good.0 {
            tap(&ctx, &mut a, "file");
            tap(&ctx, &mut a, &format!("file:{file}"));
            field(&ctx, &mut a, &data.text);
        }
        assert!(result(&a).is_none());
        frame(
            &ctx,
            &mut a,
            1200.,
            950.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            }],
        );
        settle(&ctx, &mut a);
        assert_eq!(
            result(&a),
            Some((
                if id.ends_with("quiet") { 5 } else { 6 },
                if id.ends_with("quiet") { 5 } else { 6 }
            ))
        );
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        let selected = scope(&a).selected.clone();
        field(&ctx, &mut a, "");
        assert!(scope(&a).files.0[&selected].text.is_empty());
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        field(&ctx, &mut a, "BROKEN='");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert!(result(&a).unwrap().0 < result(&a).unwrap().1);
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Needs work") && text.contains("Syntax"));
        field(&ctx, &mut a, &good.0[&selected].text);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(
            result(&a),
            Some((
                if id.ends_with("quiet") { 5 } else { 6 },
                if id.ends_with("quiet") { 5 } else { 6 }
            ))
        );
        let saved = serde_json::to_string(&a.progress).unwrap();
        a.progress = serde_json::from_str(&saved).unwrap();
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        settle(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "reset-header");
        tap(&ctx, &mut a, "confirm-reset");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn startup_controls_reflow_across_supported_zoom_widths_themes_and_focus() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        tap(&ctx, &mut a, "assignment");
        for theme in ["dlms-dark", "dlms-light"] {
            for scale in [1., 1.5, 2.] {
                for width in [800., 1440.] {
                    a.theme_id = if theme == "dlms-light" {
                        theme::inventory()
                            .iter()
                            .find(|t| t.light)
                            .unwrap()
                            .id
                            .clone()
                    } else {
                        theme::Preferences::default().theme
                    };
                    a.scale = scale;
                    ctx.set_zoom_factor(scale);
                    frame(&ctx, &mut a, width, 950., vec![]);
                    frame(&ctx, &mut a, width, 950., vec![]);
                    for (key, rect, clip) in &a.metrics.startup_files.controls {
                        assert!(rect.width().is_finite() && rect.height() > 0., "{key}");
                        if key != "editor" {
                            assert!(rect.width() <= clip.width() + 2., "{key} {rect:?} {clip:?}");
                        }
                    }
                    assert!(a.metrics.startup_files.editor.is_some());
                }
            }
        }
        let r = target(&ctx, &mut a, "editor");
        click(&ctx, &mut a, r);
        let id = a.metrics.startup_files.editor.unwrap().1;
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        frame(
            &ctx,
            &mut a,
            1200.,
            950.,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    }
}
