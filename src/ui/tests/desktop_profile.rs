//! Widget-level checks for keyboard, remote settings and accessibility examples.
use super::*;
use dlms_learning_lab::desktop::session::Session;

const XORG: &str = "lpic1-xorg-keyboard-scope";
const REMOTE: &str = "lpic1-remote-desktop-arrangement";
const A11Y: &str = "lpic1-accessible-input-profile";

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
    let State::Desktop { simulation } = &app.progress.attempts[id].state else {
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
        .desktop_profile
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

fn result(app: &LabApp) -> Option<(usize, usize)> {
    app.metrics
        .desktop_profile
        .result
        .map(|(count, total, _)| (count, total))
}

fn select(ctx: &egui::Context, app: &mut LabApp, chooser: &str, option: &str) {
    hit(ctx, app, chooser);
    hit(ctx, app, option);
}

#[test]
fn xorg_workspace_checks_applied_keyboard_results_and_invalidates_edits() {
    let (ctx, mut app, _) = open(XORG);
    hit(&ctx, &mut app, "assignment");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((2, 4)));
    let all: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/desktop_profile_tasks_v1.json"
    ))
    .unwrap();
    let repaired = all[XORG]["answers"]["assignment"].as_str().unwrap();
    input(&ctx, &mut app, "keyboard-editor", repaired);
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((1, 4)));
    hit(&ctx, &mut app, "apply-keyboard");
    hit(&ctx, &mut app, "key-Q");
    assert!(!app.metrics.completion);
    hit(&ctx, &mut app, "check");
    assert!(app.metrics.completion);
    input(&ctx, &mut app, "keyboard-editor", "# new unfinished draft");
    assert!(!session(&app, XORG).assignment.work.checked);
    assert!(!app.metrics.completion);
}

#[test]
fn remote_workspace_grades_observer_control_separately_from_login() {
    let (ctx, mut app, _) = open(REMOTE);
    hit(&ctx, &mut app, "assignment");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((0, 2)));
    select(&ctx, &mut app, "sharing", "sharing-1");
    select(&ctx, &mut app, "observer-port", "observer-port-1");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((1, 2)));
    select(&ctx, &mut app, "employee-client", "employee-client-0");
    hit(&ctx, &mut app, "check");
    assert!(app.metrics.completion);
    select(&ctx, &mut app, "sharing", "sharing-2");
    assert!(!app.metrics.completion);
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((1, 2)));
}

#[test]
fn accessible_practice_changes_actual_virtual_probes_without_assignment_credit() {
    let (ctx, mut app, _) = open(A11Y);
    let original = session(&app, A11Y).assignment.clone();
    hit(&ctx, &mut app, "example");
    select(&ctx, &mut app, "text-size", "text-size-1");
    hit(&ctx, &mut app, "sticky");
    select(&ctx, &mut app, "typing-filter", "typing-filter-1");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((3, 3)));
    assert!(!app.metrics.completion);
    assert_eq!(session(&app, A11Y).assignment, original);
    hit(&ctx, &mut app, "reset-example");
    assert!(!session(&app, A11Y).example.work.checked);
    assert_eq!(session(&app, A11Y).assignment, original);
}
