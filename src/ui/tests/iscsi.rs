//! Exercise the actual native controls with synthetic events before the Wayland pass.
use super::*;
use dlms_learning_lab::iscsi::session::Session;

const ID: &str = "lpic2-iscsi-startup-policy";

fn session(a: &LabApp) -> &Session {
    let State::Iscsi { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}

fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1500., 10000., events);
}

fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}

fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .iscsi
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(smoke::smoke_control_visible(
        r,
        clip.intersect(ctx.content_rect()),
        ctx.pixels_per_point()
    ));
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

fn run(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    hit(ctx, a, "command");
    paint(ctx, a, vec![egui::Event::Paste(text.into())]);
    settle(ctx, a);
    hit(ctx, a, "run");
}

#[test]
fn iscsi_native_editor_preserves_an_ungraded_example_and_rechecks_active_vs_saved() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    settle(&ctx, &mut a);
    let original = session(&a).assignment.clone();
    assert_eq!(session(&a).view, teaching::View::Introduction);
    hit(&ctx, &mut a, "example");
    run(&ctx, &mut a, "iscsiadm -m session");
    assert_eq!(session(&a).example.work.history.len(), 1);
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "reset-example");
    assert!(session(&a).example.work.history.is_empty());
    hit(&ctx, &mut a, "assignment");
    let lab = a.scenarios[i].clone();
    run(&ctx, &mut a, &lab.solution[0]);
    hit(&ctx, &mut a, "check");
    assert_eq!(a.metrics.iscsi.result.map(|(p, n, _)| (p, n)), Some((3, 4)));
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .iscsi
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
    hit(&ctx, &mut a, "startup-preview");
    run(&ctx, &mut a, &lab.solution[1]);
    assert!(!session(&a).assignment.work.checked);
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let command = lab.solution[0].replace("-v automatic", "-v manual");
    run(&ctx, &mut a, &command);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "check");
    assert_eq!(a.metrics.iscsi.result.map(|(p, n, _)| (p, n)), Some((3, 4)));
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert!(
        a.metrics
            .iscsi
            .controls
            .iter()
            .any(|r| r.0 == "assignment-content")
    );
    assert!(!a.metrics.iscsi.controls.iter().any(|r| r.0 == "example"));
}
