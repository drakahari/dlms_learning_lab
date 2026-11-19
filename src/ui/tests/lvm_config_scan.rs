//! Actual egui interactions using unregistered synthetic sources; not native,
//! curriculum or independent novice acceptance.
use super::*;
use dlms_learning_lab::lvm_workspace::{self as w, config_scan as f};
#[path = "../../../tests/support/lvm_config_scan_fixture.rs"]
mod fixture;
fn frame(ctx: &egui::Context, a: &mut LabApp, width: f32, events: Vec<egui::Event>) -> String {
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, 9000. / a.scale),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    fn text(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    text(s, out);
                }
            }
            _ => {}
        }
    }
    let mut rendered = String::new();
    for shape in out.shapes {
        text(&shape.shape, &mut rendered);
    }
    rendered
}
fn open(file: bool) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic2-lvm-free-space")
        .unwrap();
    a.scenarios[i] = fixture::scenario(file);
    a.launch(i);
    let source = a.scenarios[i].clone();
    let engine::Goal::Lvm { definition } = &source.goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a.progress.attempts.get_mut(&source.id).unwrap().state
    else {
        panic!()
    };
    simulation
        .set_view(definition, teaching::View::Assignment)
        .unwrap();
    frame(&ctx, &mut a, 1500., vec![]);
    (ctx, a, i)
}
fn sim(a: &LabApp, i: usize) -> &w::Session {
    let State::Lvm { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn button_rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .lvm
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .1
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let rect = button_rect(a, key);
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            1500.,
            vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    frame(ctx, a, 1500., vec![]);
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .collect()
}
fn paste(ctx: &egui::Context, a: &mut LabApp, value: &str) {
    let id = a
        .metrics
        .lvm
        .fields
        .iter()
        .find(|f| f.0 == "config-editor")
        .unwrap()
        .2;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(ctx, a, 1500., vec![]);
    let mut events = key(
        egui::Key::A,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    );
    events.push(egui::Event::Paste(value.into()));
    frame(ctx, a, 1500., events);
    frame(ctx, a, 1500., vec![]);
}
fn good_config(file: bool) -> String {
    match fixture::good(false, file).remove(0) {
        f::Operation::SaveConfig { text } => text.0,
        _ => unreachable!(),
    }
}
#[test]
fn assignment_starts_unchecked_without_worked_answer_and_actual_results_are_separate() {
    let (ctx, mut a, i) = open(false);
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(rendered.contains("Not run yet"));
    assert!(rendered.contains("Save configuration"));
    assert!(rendered.contains("Run scan"));
    assert!(a.metrics.lvm.worked_commands.is_empty());
    assert!(a.metrics.lvm.inspected_files.is_empty());
    assert!(!a.metrics.completion);
    let before = sim(&a, i)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .saved
        .clone();
    paste(&ctx, &mut a, &good_config(false));
    assert_eq!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .saved,
        before
    );
    assert!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .last_run
            .is_none()
    );
    click(&ctx, &mut a, "save-config");
    assert!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .last_run
            .is_none()
    );
    click(&ctx, &mut a, "run");
    assert!(
        a.metrics
            .lvm
            .inspected_files
            .iter()
            .any(|r| r.0 == "/dev/appdata" && r.1.contains("PV label read"))
    );
    click(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    paste(&ctx, &mut a, &fixture::config(false, &["a|.*|"]));
    assert!(!a.metrics.completion);
    assert!(!sim(&a, i).checked);
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(rendered.contains("Out of date"));
    click(&ctx, &mut a, "save-config");
    click(&ctx, &mut a, "run");
    click(&ctx, &mut a, "check");
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(rendered.contains("Needs to be excluded"));
    assert!(rendered.contains("Provided arrival check"));
    assert!(!a.metrics.completion);
}
#[test]
fn editor_handles_multiline_and_rejects_oversize_or_control_packets_without_saving() {
    let (ctx, mut a, i) = open(false);
    let before = sim(&a, i).clone();
    paste(&ctx, &mut a, &"x".repeat(4097));
    assert_eq!(sim(&a, i), &before);
    assert!(a.notice.is_some());
    paste(&ctx, &mut a, "devices {\n\u{1b}\n}");
    assert_eq!(sim(&a, i), &before);
    assert!(a.notice.is_some());
    paste(&ctx, &mut a, "# unfinished\ndevices {\n");
    assert_eq!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .draft
            .0,
        "# unfinished\ndevices {\n"
    );
    assert_eq!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .saved,
        before
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .saved
    );
    click(&ctx, &mut a, "save-config");
    click(&ctx, &mut a, "run");
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(rendered.contains("Syntax:"));
    assert_eq!(rendered.matches("Scan failed · Syntax:").count(), 1);
    assert!(a.notice.is_none());
    click(&ctx, &mut a, "check");
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert_eq!(
        rendered
            .matches("Expected a setting or section name")
            .count(),
        1,
        "Check must not repeat the current scan diagnostic: {rendered}"
    );
    assert!(rendered.contains("Needs attention · 1 check"));
    let source = a.scenarios[i].clone();
    w::context(&a.progress.attempts[&source.id], &source).unwrap();
}
#[test]
fn real_selection_controls_edit_drafts_then_save_and_scan_under_device_file_precedence() {
    let (ctx, mut a, i) = open(true);
    paste(&ctx, &mut a, &good_config(true));
    click(&ctx, &mut a, "save-config");
    click(&ctx, &mut a, "run");
    click(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    let before = sim(&a, i)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .saved
        .file
        .clone();
    for key in [
        "select-/dev/recovery",
        "select-/dev/appdata",
        "select-/dev/appspare",
    ] {
        click(&ctx, &mut a, key);
    }
    assert_eq!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .saved
            .file,
        before
    );
    assert!(!a.metrics.completion);
    click(&ctx, &mut a, "save-selection");
    click(&ctx, &mut a, "run");
    click(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(rendered.contains("Provided devices file · filter ignored"));
    click(&ctx, &mut a, "select-/dev/appspare");
    assert!(!a.metrics.completion);
    click(&ctx, &mut a, "save-selection");
    click(&ctx, &mut a, "run");
    click(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
}
#[test]
fn actual_editor_and_buttons_remain_inside_viewport_at_all_scales_in_light_and_dark() {
    let (ctx, mut a, _) = open(true);
    for theme_id in ["dlms-dark", "dlms-light"] {
        a.theme_id = theme_id.into();
        theme::apply(&ctx, theme_id);
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [1500., 900.] {
                frame(&ctx, &mut a, width, vec![]);
                frame(&ctx, &mut a, width, vec![]);
                let field = a
                    .metrics
                    .lvm
                    .fields
                    .iter()
                    .find(|f| f.0 == "config-editor")
                    .unwrap();
                assert!(field.1.width() > 80.);
                assert!(
                    field.1.left() >= field.3.left() - 1.
                        && field.1.right() <= field.3.right() + 1.,
                    "editor {width}/{scale}: {:?}",
                    field
                );
                for (key, r, clip) in &a.metrics.lvm.controls {
                    if ["save-config", "save-selection", "run", "check"].contains(&key.as_str()) {
                        assert!(
                            r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                            "{key}/{width}/{scale}: {r:?}/{clip:?}"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn example_fill_changes_only_example_draft_and_challenge_keeps_worked_help_closed() {
    let (ctx, mut a, i) = open(false);
    let initial = sim(&a, i)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .clone();
    click(&ctx, &mut a, "example");
    click(&ctx, &mut a, "example-worked");
    let example_before = sim(&a, i)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .example
        .clone();
    click(&ctx, &mut a, "fill-config");
    let example = &sim(&a, i).config_scan_runtime.as_ref().unwrap().example;
    assert_ne!(example.draft, example_before.draft);
    assert_eq!(example.saved, example_before.saved);
    assert!(example.last_run.is_none());
    assert_eq!(
        sim(&a, i).config_scan_runtime.as_ref().unwrap().assignment,
        initial
    );
    // Open the actual optional worked panel, then fill without saving or running.
    let editor = a.metrics.lvm.fields[0].2;
    ctx.memory_mut(|m| m.surrender_focus(editor));
    // Introduction is revisitable and does not execute the worked example.
    click(&ctx, &mut a, "introduction");
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(rendered.contains("/dev/demo(one|two)"));
    assert_eq!(
        sim(&a, i).config_scan_runtime.as_ref().unwrap().assignment,
        initial
    );
    a.progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .challenge = true;
    let rendered = frame(&ctx, &mut a, 1500., vec![]);
    assert!(!rendered.contains("/dev/demo(one|two)"));
    assert!(a.metrics.lvm.worked_commands.is_empty());
    assert!(!a.metrics.completion);
}
#[test]
fn native_tab_and_enter_save_configuration_without_indenting_or_running() {
    let (ctx, mut a, i) = open(false);
    paste(&ctx, &mut a, &good_config(false));
    let id = a
        .metrics
        .lvm
        .fields
        .iter()
        .find(|f| f.0 == "config-editor")
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let draft = sim(&a, i)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .draft
        .clone();
    frame(
        &ctx,
        &mut a,
        1500.,
        key(egui::Key::Tab, egui::Modifiers::NONE),
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    assert_eq!(
        sim(&a, i)
            .config_scan_runtime
            .as_ref()
            .unwrap()
            .assignment
            .draft,
        draft
    );
    frame(
        &ctx,
        &mut a,
        1500.,
        key(egui::Key::Enter, egui::Modifiers::NONE),
    );
    frame(&ctx, &mut a, 1500., vec![]);
    let live = &sim(&a, i).config_scan_runtime.as_ref().unwrap().assignment;
    assert_eq!(live.saved.config, draft);
    assert!(live.last_run.is_none());
}
