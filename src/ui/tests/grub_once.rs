use super::*;

const ID: &str = "lpic2-grub-one-boot-recovery";

fn settle(context: &egui::Context, app: &mut LabApp) {
    for _ in 0..3 {
        frame(context, app, 1500., 1200., vec![]);
    }
}

fn tap(context: &egui::Context, app: &mut LabApp, key: &str) {
    app.smoke_stage += 1;
    for _ in 0..30 {
        let control = app
            .metrics
            .grub_once
            .controls
            .iter()
            .find(|row| row.0 == key)
            .unwrap_or_else(|| panic!("Missing temporary boot control {key}"))
            .1;
        if app.reveal_smoke_control(context, control) {
            click(context, app, control);
            settle(context, app);
            return;
        }
        let events = std::mem::take(&mut app.pending_events);
        frame(context, app, 1500., 1200., events);
    }
    panic!("Temporary boot control {key} did not settle");
}

fn paste_line(context: &egui::Context, app: &mut LabApp, text: String) {
    tap(context, app, "input");
    let id = app.metrics.focus.unwrap();
    context.memory_mut(|memory| memory.request_focus(id));
    frame(
        context,
        app,
        1500.,
        1200.,
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
            egui::Event::Paste(text),
        ],
    );
    settle(context, app);
}

#[test]
fn grub_once_challenge_hides_worked_practice_and_discard_clears_grade() {
    let context = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&context, true, false, true);
    app.smoke = false;
    let index = app
        .scenarios
        .iter()
        .position(|scenario| scenario.id == ID)
        .unwrap();
    app.launch(index);
    settle(&context, &mut app);
    let engine::Goal::GrubOnce { definition } = &app.scenarios[index].goal else {
        panic!()
    };
    let practice = definition.example.reference.linux.clone();
    let assignment = definition.initial.reference.linux.clone();

    tap(&context, &mut app, "example");
    tap(&context, &mut app, "choice");
    tap(&context, &mut app, "menu-inspection");
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.grub_once.result, Some(false));
    paste_line(&context, &mut app, practice);
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.grub_once.result, Some(true));
    assert!(!progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[index]
    ));

    app.lesson_scroll_reset = true;
    settle(&context, &mut app);
    let challenge = app.metrics.mode_challenge.unwrap();
    click(&context, &mut app, challenge);
    settle(&context, &mut app);
    assert!(app.progress.attempts[ID].challenge);
    assert!(app.metrics.lesson_tabs.is_empty());
    assert!(
        !app.metrics
            .grub_once
            .controls
            .iter()
            .any(|row| row.0 == "worked")
    );
    tap(&context, &mut app, "choice");
    tap(&context, &mut app, "menu-inspection");
    paste_line(&context, &mut app, assignment);
    tap(&context, &mut app, "test");
    assert!(progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[index]
    ));
    tap(&context, &mut app, "discard");
    assert!(app.metrics.grub_once.result.is_none());
    assert!(!progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[index]
    ));
    let State::GrubOnce { simulation } = &app.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.practice.observed.is_some());
    assert!(simulation.assignment.observed.is_none());
}
