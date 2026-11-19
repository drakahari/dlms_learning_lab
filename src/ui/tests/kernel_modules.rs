//! Native widget behavior for supplied modules; not novice/kernel acceptance.
use super::*;
use dlms_learning_lab::kernel_module_workspace as work;
const IDS: [&str; 3] = [
    "lpic1-module-device-driver",
    "lpic1-module-index-repair",
    "lpic2-module-options-reload",
];
fn setup(id: &str, v: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = debug_recovery::opened(id);
    let State::KernelModules { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
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
fn control(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .kernel_module
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
fn field(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .kernel_module
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = field(a, key);
    click(ctx, a, r);
    frame(
        ctx,
        a,
        1500.,
        7000.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    full(ctx, a);
}
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    replace(ctx, a, "command", text);
    frame(ctx, a, 1500., 7000., vec![key(egui::Key::Enter, false)]);
    full(ctx, a);
}
fn press(ctx: &egui::Context, a: &mut LabApp, k: &str) {
    let r = control(a, k);
    click(ctx, a, r);
    full(ctx, a);
}
fn session<'a>(a: &'a LabApp, id: &str) -> &'a work::Session {
    let State::KernelModules { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn rendered(ctx: &egui::Context, a: &mut LabApp) -> String {
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
    fn text(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    text(s, out)
                }
            }
            _ => {}
        }
    }
    let mut t = String::new();
    for s in out.shapes {
        text(&s.shape, &mut t);
    }
    t
}
#[test]
fn module_views_teach_specific_concepts_and_hide_independent_worked_answers() {
    for id in IDS {
        let (ctx, mut a, _) = setup(id, teaching::View::Introduction);
        full(&ctx, &mut a);
        let t = rendered(&ctx, &mut a);
        assert!(t.contains("WHAT YOU’LL LEARN") && t.contains("Worked example"));
        assert!(t.contains("example_reader"));
        press(&ctx, &mut a, "example");
        let t = rendered(&ctx, &mut a);
        assert!(t.contains("Expected:") && t.contains("Actual practice terminal"));
        press(&ctx, &mut a, "assignment");
        let t = rendered(&ctx, &mut a);
        assert!(t.contains("Requirements") && t.contains("Simulated assignment terminal"));
        assert!(!t.contains("example_reader"));
        assert!(!t.contains("@module-"));
        assert!(
            !t.contains("modprobe lab_sensor"),
            "No assignment command is exposed by default"
        );
        assert_eq!(
            a.metrics
                .kernel_module
                .fields
                .iter()
                .any(|f| f.0 == "config"),
            id == IDS[2]
        );
    }
}
#[test]
fn actual_editor_save_enter_check_retry_and_draft_invalidation_are_independent() {
    let id = IDS[2];
    let (ctx, mut a, i) = setup(id, teaching::View::Example);
    full(&ctx, &mut a);
    let assignment = session(&a, id).assignment.clone();
    replace(&ctx, &mut a, "config", "options example_reader queues=3\n");
    assert_ne!(
        session(&a, id).practice.config_draft,
        session(&a, id).practice.runtime.config
    );
    press(&ctx, &mut a, "save-options");
    command(&ctx, &mut a, "modprobe example_reader");
    press(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.kernel_module.result.map(|r| r.1 == 0),
        Some(false)
    );
    command(&ctx, &mut a, "modprobe -r example_reader");
    command(&ctx, &mut a, "modprobe example_reader");
    press(&ctx, &mut a, "check");
    assert_eq!(a.metrics.kernel_module.result.map(|r| r.1), Some(0));
    assert_eq!(session(&a, id).assignment, assignment);
    assert!(!progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
    press(&ctx, &mut a, "assignment");
    replace(&ctx, &mut a, "config", "options lab_sensor queues=4\n");
    press(&ctx, &mut a, "save-options");
    command(&ctx, &mut a, "modprobe -r lab_sensor");
    command(&ctx, &mut a, "modprobe lab_sensor");
    press(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
    replace(&ctx, &mut a, "config", "options lab_sensor queues=2\n");
    assert!(!session(&a, id).assignment.checked);
    assert!(!progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
    assert!(a.metrics.kernel_module.result.is_none());
    press(&ctx, &mut a, "check");
    assert_eq!(a.metrics.kernel_module.result.map(|r| r.1), Some(1));
    assert!(rendered(&ctx, &mut a).contains("Save or revert"));
    command(&ctx, &mut a, "modprobe --force lab_sensor");
    assert!(
        session(&a, id)
            .assignment
            .transcript
            .last()
            .unwrap()
            .contains("Unsupported")
    );
    let mut changed = a.progress.attempts[id].clone();
    let State::KernelModules { simulation } = &mut changed.state else {
        panic!()
    };
    simulation.origin = IDS[0].into();
    assert!(dlms_learning_lab::kernel_module_source::context(&changed, &a.scenarios[i]).is_err());
}
#[test]
fn module_controls_reflow_and_keep_readable_focused_fields_at_supported_scales() {
    for id in IDS {
        for width in [760., 1500.] {
            for scale in [1., 1.5, 2.] {
                for light in [false, true] {
                    let (ctx, mut a, _) = setup(id, teaching::View::Assignment);
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
                    for (key, r, clip) in &a.metrics.kernel_module.controls {
                        assert!(r.is_finite() && clip.is_finite(), "{id}/{key}");
                        assert!(r.width() > 12. && r.height() > 12.);
                        assert!(
                            r.left() >= -1. && r.right() <= width + 1.,
                            "{id}/{key}/{width}/{scale}: {r:?}"
                        );
                    }
                    let r = field(&a, "command");
                    assert!(r.width() > 80. && r.height() >= 20.);
                    let run = control(&a, "run");
                    assert!((r.center().y - run.center().y).abs() < 2.);
                    assert!(r.right() <= run.left());
                }
            }
        }
    }
}

#[test]
fn optional_worked_answer_renders_file_contents_not_the_internal_save_action() {
    let id = IDS[2];
    let (ctx, mut a, _) = setup(id, teaching::View::Assignment);
    full(&ctx, &mut a);
    let before = session(&a, id).assignment.clone();
    press(&ctx, &mut a, "help");
    press(&ctx, &mut a, "worked");
    let text = rendered(&ctx, &mut a);
    assert!(
        text.contains("Editor contents · Save options")
            && text.contains("options lab_sensor queues=4")
    );
    assert!(!text.contains("@module-save") && !text.contains("@module-check"));
    assert_eq!(session(&a, id).assignment, before);
}
