//! Widget-level checks for draft, applied settings and separate examples.
use super::*;
use dlms_learning_lab::apache_https::{session::Session, source};

const CHAIN: &str = "lpic2-apache-https-chain";
const SNI: &str = "lpic2-apache-https-sni";

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
    let State::ApacheHttps { simulation } = &app.progress.attempts[id].state else {
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
        .apache_https
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
        .apache_https
        .result
        .map(|(count, total, _)| (count, total))
}

fn answer(id: &str, scope: &str) -> String {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/apache_https_tasks_v1.json"
    ))
    .unwrap();
    fixture[id]["answers"][scope].as_str().unwrap().into()
}

#[test]
fn https_editor_changes_the_client_only_after_apply_and_requires_a_check() {
    let (ctx, mut app, _) = open(CHAIN);
    hit(&ctx, &mut app, "assignment");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((2, 3)));
    let applied = session(&app, CHAIN).assignment.work.applied.clone();
    input(&ctx, &mut app, "config", &answer(CHAIN, "assignment"));
    assert!(!session(&app, CHAIN).assignment.work.checked);
    hit(&ctx, &mut app, "save");
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((1, 3)));
    assert_eq!(session(&app, CHAIN).assignment.work.applied, applied);
    hit(&ctx, &mut app, "apply");
    assert!(!app.metrics.completion);
    hit(&ctx, &mut app, "check");
    assert!(app.metrics.completion);
    input(&ctx, &mut app, "config", &applied);
    assert!(!app.metrics.completion);
}

#[test]
fn https_example_dropdown_and_repair_leave_the_assignment_untouched() {
    let (ctx, mut app, index) = open(SNI);
    let before = session(&app, SNI).assignment.clone();
    let starter = source::source(&app.scenarios[index])
        .unwrap()
        .example
        .starter
        .clone();
    hit(&ctx, &mut app, "example");
    input(&ctx, &mut app, "config", &answer(SNI, "example"));
    hit(&ctx, &mut app, "save");
    hit(&ctx, &mut app, "apply");
    hit(&ctx, &mut app, "client");
    hit(&ctx, &mut app, "trial-3");
    assert!(
        app.metrics
            .apache_https
            .controls
            .iter()
            .any(|r| r.0 == "selected-trial-3")
    );
    hit(&ctx, &mut app, "check");
    assert_eq!(result(&app), Some((5, 5)));
    assert!(!app.metrics.completion);
    assert_eq!(session(&app, SNI).assignment, before);
    hit(&ctx, &mut app, "reset-example");
    assert_eq!(session(&app, SNI).example.work.draft, starter);
}
