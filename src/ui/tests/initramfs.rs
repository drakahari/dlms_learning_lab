//! Actual native widgets in a headless frame; Fedora image evidence is separate.
use super::*;
use dlms_learning_lab::{initramfs_source as binding, initramfs_workspace as work};
const ID: &str = "lpic2-initramfs-root-drivers";
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::Initramfs { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::Initramfs { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1500., 16000., events)
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .early_userspace
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
        "Clipped {key}"
    );
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
fn input(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    hit(ctx, a, key);
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
            egui::Event::Paste(value.into()),
        ],
    );
    settle(ctx, a);
}
#[test]
fn initramfs_intro_is_skippable_and_challenge_keeps_independent_controls() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    assert!(
        a.metrics
            .early_userspace
            .controls
            .iter()
            .any(|c| c.0 == "worked")
    );
    hit(&ctx, &mut a, "assignment");
    assert!(
        a.metrics
            .early_userspace
            .fields
            .iter()
            .any(|f| f.0 == "command")
    );
    assert!(
        !a.metrics
            .early_userspace
            .controls
            .iter()
            .any(|c| c.0 == "fill-example")
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert!(
        !a.metrics
            .early_userspace
            .controls
            .iter()
            .any(|c| c.0 == "introduction" || c.0 == "worked-answer")
    );
}
#[test]
fn initramfs_editor_and_console_keep_example_isolated_and_invalidate_repaired_state() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let original = session(&a).assignment.clone();
    input(
        &ctx,
        &mut a,
        "command",
        "lsinitrd /boot/initramfs-6.1.121.img",
    );
    hit(&ctx, &mut a, "run");
    assert!(
        !session(&a)
            .example
            .work
            .log
            .last()
            .unwrap()
            .output
            .contains("/nvme.ko")
    );
    input(&ctx, &mut a, "config", "omit_drivers=\"\"\n");
    hit(&ctx, &mut a, "save-config");
    input(
        &ctx,
        &mut a,
        "command",
        "dracut --force --kver 6.1.121 /boot/initramfs-6.1.121.img",
    );
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "check");
    let d = binding::source(&a.scenarios[i]).unwrap();
    assert!(d.example.solved(&session(&a).example.work));
    assert_eq!(session(&a).assignment, original);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));

    hit(&ctx, &mut a, "assignment");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .early_userspace
            .controls
            .iter()
            .any(|c| c.0 == "failed-case")
    );
    input(&ctx, &mut a, "config", "omit_drivers=\"\"\n");
    hit(&ctx, &mut a, "save-config");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    input(
        &ctx,
        &mut a,
        "command",
        "dracut -f /boot/initramfs-6.6.61.img 6.6.61",
    );
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    input(&ctx, &mut a, "config", "omit_drivers=\"nvme\"\n");
    assert!(!session(&a).assignment.work.checked);
    assert!(!a.metrics.completion);
}
#[test]
fn initramfs_controls_reflow_with_font_height_at_three_scales_and_both_themes() {
    for scale in [1., 1.5, 2.] {
        for light in [false, true] {
            for width in [780., 1500.] {
                let (ctx, mut a, _) = open(teaching::View::Assignment);
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.theme_id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &a.theme_id);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 16000., vec![]);
                }
                let (_, r, clip) = a
                    .metrics
                    .early_userspace
                    .controls
                    .iter()
                    .find(|r| r.0 == "command")
                    .unwrap();
                assert!(r.width() > 80. && r.height() > 16.);
                assert!(r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.);
                let (_, run, _) = a
                    .metrics
                    .early_userspace
                    .controls
                    .iter()
                    .find(|r| r.0 == "run")
                    .unwrap();
                assert!(
                    (r.height() - run.height()).abs() < 3.,
                    "Input/button height must use shared font metrics: {r:?} {run:?}"
                );
            }
        }
    }
}
