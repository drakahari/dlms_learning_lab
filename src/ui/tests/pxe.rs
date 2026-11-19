//! Actual native widgets in a headless frame; Fedora image evidence is separate.
use super::*;
use dlms_learning_lab::pxe::{session::Session, source, workspace::Action};
const POLICY: &str = "lpic2-pxe-firmware-selection";
const CONFIG: &str = "lpic2-pxelinux-client-lookup";
fn open(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let State::Pxe { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session<'a>(a: &'a LabApp, id: &str) -> &'a Session {
    let State::Pxe { simulation } = &a.progress.attempts[id].state else {
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
        .network_boot
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
fn pxe_policy_widgets_distinguish_unsaved_rules_and_invalidate_current_success() {
    let (ctx, mut a, _) = open(POLICY, teaching::View::Assignment);
    hit(&ctx, &mut a, "inspect-1");
    assert!(
        a.metrics
            .network_boot
            .controls
            .iter()
            .any(|c| c.0 == "trace")
    );
    hit(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.network_boot.result.map(|(p, n, _)| (p, n)),
        Some((2, 4))
    );
    input(&ctx, &mut a, "uefi-server", "192.0.2.10");
    hit(&ctx, &mut a, "default-enabled");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert_eq!(
        session(&a, POLICY)
            .assignment
            .work
            .policy
            .x64_uefi
            .as_ref()
            .unwrap()
            .server
            .to_string(),
        "192.0.2.11"
    );
    hit(&ctx, &mut a, "save-policy");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    hit(&ctx, &mut a, "default-enabled");
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "save-policy");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert_eq!(
        a.metrics.network_boot.result.map(|(p, n, _)| (p, n)),
        Some((3, 4))
    );
}

#[test]
fn pxe_editor_keeps_practice_separate_and_challenge_skips_its_controls() {
    let (ctx, mut a, i) = open(CONFIG, teaching::View::Introduction);
    hit(&ctx, &mut a, "try-example");
    let original = session(&a, CONFIG).assignment.clone();
    let source = source::source(&a.scenarios[i]).unwrap();
    let Action::EditEntry { draft } = source.example.reference[0].clone() else {
        panic!()
    };
    input(&ctx, &mut a, "entry-text", &draft.source);
    hit(&ctx, &mut a, "save-entry");
    hit(&ctx, &mut a, "check");
    assert_eq!(session(&a, CONFIG).assignment, original);
    assert_eq!(
        a.metrics.network_boot.result.map(|(p, n, _)| (p, n)),
        Some((3, 3))
    );
    assert!(!a.metrics.completion);
    a.progress.attempts.get_mut(CONFIG).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert!(
        a.metrics
            .network_boot
            .controls
            .iter()
            .any(|c| c.0 == "assignment-content")
    );
    assert!(
        !a.metrics
            .network_boot
            .controls
            .iter()
            .any(|c| c.0 == "worked-answer" || c.0 == "example")
    );
    assert_eq!(session(&a, CONFIG).view, teaching::View::Example);
}

#[test]
fn pxe_rule_inputs_share_height_and_equivalent_width_at_each_text_scale() {
    for (scale, width) in [(1., 1500.), (1.5, 820.), (2., 780.)] {
        for light in [false, true] {
            let (ctx, mut a, _) = open(POLICY, teaching::View::Assignment);
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
            let field = |key| {
                a.metrics
                    .network_boot
                    .controls
                    .iter()
                    .find(|(k, _, _)| k == key)
                    .unwrap()
            };
            for [left, right] in [
                ["bios-server", "uefi-server"],
                ["bios-filename", "uefi-filename"],
            ] {
                let (_, first, clip) = field(left);
                let (_, second, _) = field(right);
                assert!((first.width() - second.width()).abs() < 1.);
                assert!((first.height() - second.height()).abs() < 1.);
                assert!(first.width() > 80. && first.height() > 16.);
                assert!(first.left() >= clip.left() - 1. && first.right() <= clip.right() + 1.);
            }
        }
    }
}
