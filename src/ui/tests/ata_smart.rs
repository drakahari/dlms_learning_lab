//! Widget-level coverage of the ticket and saved monitoring editor.
use super::*;
use dlms_learning_lab::ata_smart::{session::Session, source, workspace::Assignment};

const DIAG: &str = "lpic2-smart-short-diagnosis";
const WATCH: &str = "lpic2-smart-monitor-identity";

fn open(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    app.smoke = false;
    let index = app.scenarios.iter().position(|s| s.id == id).unwrap();
    app.launch(index);
    settle(&ctx, &mut app);
    (ctx, app, index)
}
fn session<'a>(app: &'a LabApp, id: &str) -> &'a Session {
    let State::AtaSmart { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn paint(ctx: &egui::Context, app: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, app, 1500., 10000., events);
}
fn settle(ctx: &egui::Context, app: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, app, vec![]);
    }
}
fn hit(ctx: &egui::Context, app: &mut LabApp, name: &str) {
    let (_, r, clip) = app
        .metrics
        .ata_smart
        .controls
        .iter()
        .find(|r| r.0 == name)
        .unwrap_or_else(|| panic!("Missing {name}"))
        .clone();
    assert!(smoke::smoke_control_visible(
        r,
        clip.intersect(ctx.content_rect()),
        ctx.pixels_per_point()
    ));
    for pressed in [true, false] {
        paint(
            ctx,
            app,
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
    settle(ctx, app);
}
fn input(ctx: &egui::Context, app: &mut LabApp, field: &str, value: &str) {
    hit(ctx, app, field);
    let select = egui::Modifiers {
        command: true,
        ..egui::Modifiers::CTRL
    };
    paint(
        ctx,
        app,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: select,
            },
            egui::Event::Paste(value.into()),
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: select,
            },
        ],
    );
    settle(ctx, app);
}
fn run(ctx: &egui::Context, app: &mut LabApp, command: &str) {
    input(ctx, app, "command", command);
    hit(ctx, app, "run");
}

#[test]
fn cycle_a_fresh_test_check_a_bad_ticket_and_invalidate_after_editing() {
    let (ctx, mut a, _) = open(DIAG);
    hit(&ctx, &mut a, "assignment");
    run(&ctx, &mut a, "smartctl -t short /dev/sdb");
    hit(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ata_smart.result.map(|(ok, total, _)| (ok, total)),
        Some((1, 3))
    );
    for _ in 0..3 {
        hit(&ctx, &mut a, "advance");
    }
    run(&ctx, &mut a, "smartctl -l selftest /dev/sdb");
    hit(&ctx, &mut a, "disposition");
    hit(&ctx, &mut a, "ticket-read-failure");
    input(&ctx, &mut a, "lba", "491520");
    hit(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ata_smart.result.map(|(ok, total, _)| (ok, total)),
        Some((2, 3))
    );
    input(&ctx, &mut a, "lba", "497616");
    assert!(!session(&a, DIAG).assignment.work.checked);
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}

#[test]
fn monitor_keeps_example_edits_separate_and_only_saved_rules_drive_previews() {
    let (ctx, mut a, i) = open(WATCH);
    let baseline = session(&a, WATCH).assignment.clone();
    hit(&ctx, &mut a, "example");
    hit(&ctx, &mut a, "maintenance-layout");
    let d = source::source(&a.scenarios[i]).unwrap().clone();
    let Assignment::Monitoring(g) = &d.example.goal else {
        panic!()
    };
    let repair = g.initial_config.replace(
        &d.example.inventory.disks[0].current_name,
        &d.example.inventory.disks[0].stable,
    );
    input(&ctx, &mut a, "config", &repair);
    hit(&ctx, &mut a, "check");
    assert_eq!(
        session(&a, WATCH).example.work.monitor_saved,
        g.initial_config
    );
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ata_smart.result.map(|(ok, total, _)| (ok, total)),
        Some((4, 4))
    );
    assert!(!a.metrics.completion);
    assert_eq!(session(&a, WATCH).assignment, baseline);
    hit(&ctx, &mut a, "reset-example");
    let work = &session(&a, WATCH).example.work;
    assert_eq!(work.monitor_saved, g.initial_config);
    assert!(!work.checked);
}
