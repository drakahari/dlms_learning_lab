//! Exact-name teaching in native controls; no independent novice acceptance.
use super::*;
use dlms_learning_lab::kernel_module_workspace as work;
const ID: &str = "lpic2-module-name-handover";
fn setup(v: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = debug_recovery::opened(ID);
    let State::KernelModules { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    (ctx, a, i)
}
fn full(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1500., 7000., vec![]);
    }
}
fn session(a: &LabApp) -> &work::Session {
    let State::KernelModules { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn button_rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .kernel_module
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
fn press(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = button_rect(a, key);
    click(ctx, a, r);
    full(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = a
        .metrics
        .kernel_module
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1;
    click(ctx, a, r);
    frame(
        ctx,
        a,
        1500.,
        7000.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(value.into())],
    );
    full(ctx, a);
}
fn command(ctx: &egui::Context, a: &mut LabApp, value: &str) {
    replace(ctx, a, "command", value);
    frame(ctx, a, 1500., 7000., vec![key(egui::Key::Enter, false)]);
    full(ctx, a);
}
fn text(ctx: &egui::Context, a: &mut LabApp) -> String {
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 7000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    fn collect(s: &egui::Shape, t: &mut String) {
        match s {
            egui::Shape::Text(s) => {
                t.push_str(&s.galley.job.text);
                t.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, t)
                }
            }
            _ => {}
        }
    }
    let mut t = String::new();
    for s in out.shapes {
        collect(&s.shape, &mut t)
    }
    t
}
#[test]
fn named_introduction_practice_and_independent_assignment_use_clear_subject_specific_controls() {
    let (ctx, mut a, _) = setup(teaching::View::Introduction);
    full(&ctx, &mut a);
    let t = text(&ctx, &mut a);
    assert!(
        t.contains("WHAT YOU’LL LEARN")
            && t.contains("not another loaded module")
            && t.contains("Worked example")
            && t.contains("alias demo_capture example_reader")
    );
    press(&ctx, &mut a, "example");
    let t = text(&ctx, &mut a);
    assert!(
        t.contains("Actual practice terminal")
            && t.contains("Save configuration")
            && t.contains("Observe:")
    );
    press(&ctx, &mut a, "picker");
    full(&ctx, &mut a);
    let t = text(&ctx, &mut a);
    assert!(t.contains("Edit and save the practice name bindings"));
    press(&ctx, &mut a, "choice-3");
    press(&ctx, &mut a, "insert");
    assert_ne!(
        session(&a).practice.config_draft,
        session(&a).practice.runtime.config
    );
    assert!(!session(&a).practice.checked);
    assert!(
        !session(&a)
            .practice
            .runtime
            .loaded
            .contains_key("example_reader")
    );
    press(&ctx, &mut a, "assignment");
    let t = text(&ctx, &mut a);
    assert!(t.contains("Requirements") && t.contains("Saved module names"));
    assert!(!t.contains("demo_capture") && !t.contains("alias capture_current lab_sensor"));
    let before = session(&a).assignment.clone();
    press(&ctx, &mut a, "help");
    let t = text(&ctx, &mut a);
    assert!(t.contains("Only eight unique literal aliases") && !t.contains("No aliases, built-in"));
    press(&ctx, &mut a, "worked");
    let t = text(&ctx, &mut a);
    assert!(
        t.contains("Editor contents · Save configuration")
            && t.contains("alias capture_current lab_sensor")
    );
    assert!(!t.contains("@module-save") && !t.contains("@module-check"));
    assert_eq!(session(&a).assignment, before);
    let block = a
        .metrics
        .kernel_module
        .worked_editor
        .expect("actual worked file block geometry");
    assert!(block.is_finite() && block.height() > 40. && block.width() > 80.);
    assert_eq!(a.metrics.kernel_module.capture_target(true), Some(block));
    assert_eq!(
        a.metrics.kernel_module.capture_target(false),
        a.metrics.kernel_module.result.map(|r| r.2)
    );
    assert_eq!(
        kernel_module_page::Metrics::default().capture_target(true),
        None
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    full(&ctx, &mut a);
    let t = text(&ctx, &mut a);
    assert!(
        !t.contains("alias capture_current lab_sensor") && !t.contains("demo_capture"),
        "Challenge must not inherit an opened Learn worked answer"
    );
}
#[test]
fn native_editor_and_enter_distinguish_current_success_from_one_failing_future_case() {
    let (ctx, mut a, i) = setup(teaching::View::Assignment);
    full(&ctx, &mut a);
    command(&ctx, &mut a, "modprobe capture_current");
    press(&ctx, &mut a, "check");
    assert_eq!(a.metrics.kernel_module.result.map(|r| r.1), Some(3));
    assert!(text(&ctx, &mut a).contains("returned success"));
    replace(
        &ctx,
        &mut a,
        "config",
        "options lab_sensor queues=1\nalias capture_current lab_sensor\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n",
    );
    press(&ctx, &mut a, "save-options");
    command(&ctx, &mut a, "modprobe capture_current");
    press(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    replace(
        &ctx,
        &mut a,
        "config",
        "options lab_sensor queues=1\nalias capture_current audit_link\nalias capture_legacy lab_sensor\nalias audit_query audit_link\n",
    );
    assert!(!session(&a).assignment.checked);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    press(&ctx, &mut a, "save-options");
    press(&ctx, &mut a, "check");
    assert_eq!(a.metrics.kernel_module.result.map(|r| r.1), Some(1));
    let d = dlms_learning_lab::kernel_module_source::source(&a.scenarios[i]).unwrap();
    let rows = session(&a).assignment.checks(&d.assignment);
    assert!(rows[0].0 && !rows[4].0 && rows[5].0);
    assert!(text(&ctx, &mut a).contains("Saved request capture_current"));
    let practice = session(&a).practice.clone();
    command(&ctx, &mut a, "modprobe --force capture_current");
    assert!(
        session(&a)
            .assignment
            .transcript
            .last()
            .unwrap()
            .contains("Unsupported")
    );
    assert_eq!(session(&a).practice, practice);
}
#[test]
fn named_editor_buttons_and_command_controls_fit_wide_narrow_light_dark_and_enlarged_text() {
    for width in [760., 1500.] {
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                let (ctx, mut a, _) = setup(teaching::View::Assignment);
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.theme_id = if light {
                    theme::inventory()
                        .iter()
                        .find(|p| p.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(&ctx, &a.theme_id);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 6000., vec![]);
                }
                for (k, r, _) in &a.metrics.kernel_module.controls {
                    assert!(r.is_finite() && r.width() > 10. && r.height() > 10.);
                    assert!(
                        r.left() >= -1. && r.right() <= width + 1.,
                        "{k}/{width}/{scale}: {r:?}"
                    );
                }
                let input = a
                    .metrics
                    .kernel_module
                    .fields
                    .iter()
                    .find(|r| r.0 == "command")
                    .unwrap()
                    .1;
                let run = button_rect(&a, "run");
                assert!((input.center().y - run.center().y).abs() < 2.);
                assert!(input.width() > 80. && input.right() <= run.left());
                let editor = a
                    .metrics
                    .kernel_module
                    .fields
                    .iter()
                    .find(|r| r.0 == "config")
                    .unwrap()
                    .1;
                let save = button_rect(&a, "save-options");
                assert!(save.top() >= editor.top());
                assert!(
                    (save.left() - editor.left()).abs() < 2.,
                    "save must retain its column when editor scrolls"
                );
            }
        }
    }
}
