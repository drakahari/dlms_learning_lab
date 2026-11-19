use super::*;

const ID: &str = "lpic1-grub-efi-registration";

fn open(id: &str) -> (egui::Context, LabApp, usize) {
    let context = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&context, true, false, true);
    app.smoke = false;
    let index = app
        .scenarios
        .iter()
        .position(|scenario| scenario.id == id)
        .unwrap();
    app.launch(index);
    (context, app, index)
}

fn draw(context: &egui::Context, app: &mut LabApp) {
    for _ in 0..3 {
        frame(context, app, 1500., 1200., vec![]);
    }
}

fn tap(context: &egui::Context, app: &mut LabApp, key: &str) {
    app.smoke_stage += 1;
    for _ in 0..30 {
        let control = app
            .metrics
            .efi_loader
            .controls
            .iter()
            .find(|row| row.0 == key)
            .unwrap()
            .1;
        if app.reveal_smoke_control(context, control) {
            click(context, app, control);
            draw(context, app);
            return;
        }
        let events = std::mem::take(&mut app.pending_events);
        frame(context, app, 1500., 1200., events);
    }
    panic!("EFI control {key} did not settle");
}

fn type_command(context: &egui::Context, app: &mut LabApp, command: &str) {
    tap(context, app, "input");
    let id = app.metrics.efi_loader.input.unwrap().1;
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
            egui::Event::Paste(command.into()),
        ],
    );
    draw(context, app);
}

#[test]
fn efi_native_controls_distinguish_file_installation_firmware_registration_and_retry() {
    let (context, mut app, index) = open(ID);
    draw(&context, &mut app);
    tap(&context, &mut app, "example");
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(false));
    tap(&context, &mut app, "worked");
    tap(&context, &mut app, "insert-0");
    let State::EfiLoader { simulation } = &app.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.practice.command.starts_with("grub-install"));
    assert_eq!(simulation.practice.current.boot_order, [9]);
    assert!(
        simulation
            .practice
            .current
            .files
            .iter()
            .all(|file| !matches!(
                file.kind,
                dlms_learning_lab::efi_loader::FileKind::Grub { .. }
            ))
    );
    tap(&context, &mut app, "run");
    assert!(app.metrics.efi_loader.result.is_none());
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(false));
    tap(&context, &mut app, "insert-1");
    tap(&context, &mut app, "run");
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(true));
    assert!(!progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[index]
    ));

    tap(&context, &mut app, "assignment");
    let engine::Goal::EfiLoader { definition } = &app.scenarios[index].goal else {
        panic!()
    };
    let reference = definition.initial.reference.clone();
    for command in &reference[..2] {
        type_command(&context, &mut app, command);
        tap(&context, &mut app, "run");
    }
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(true));
    assert!(progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[index]
    ));
    type_command(&context, &mut app, "efibootmgr -o 7,0");
    tap(&context, &mut app, "run");
    assert!(app.metrics.efi_loader.result.is_none());
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(false));
    assert!(!progress::current_solved(
        &app.progress.attempts[ID],
        &app.scenarios[index]
    ));

    app.lesson_scroll_reset = true;
    draw(&context, &mut app);
    let challenge = app.metrics.mode_challenge.unwrap();
    click(&context, &mut app, challenge);
    draw(&context, &mut app);
    assert!(
        !app.metrics
            .efi_loader
            .controls
            .iter()
            .any(|row| matches!(row.0.as_str(), "worked" | "introduction" | "example"))
    );
}

#[test]
fn efi_native_inputs_reflow_at_supported_scales_in_dark_and_light() {
    let (context, mut app, _) = open(ID);
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        app.scale = scale;
        context.set_zoom_factor(scale);
        for light in [false, true] {
            app.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|theme| theme.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&context, &app.theme_id);
            for (width, height) in [(1500., 1100.), (760., 1000.)] {
                let State::EfiLoader { simulation } =
                    &mut app.progress.attempts.get_mut(ID).unwrap().state
                else {
                    panic!()
                };
                simulation.view = teaching::View::Assignment;
                app.lesson_scroll_reset = true;
                for _ in 0..3 {
                    frame(&context, &mut app, width, height, vec![]);
                }
                assert!(app.metrics.efi_loader.input.is_some());
                if scale == 1. && width == 1500. {
                    assert!(app.metrics.efi_loader.side_by_side);
                }
                if scale == 2. && width == 760. {
                    assert!(!app.metrics.efi_loader.side_by_side);
                }
                for (key, rect, _) in &app.metrics.efi_loader.controls {
                    assert!(rect.is_finite() && rect.is_positive(), "{key}");
                    assert!(
                        rect.left() >= 0. && rect.right() <= context.content_rect().right() + 1.,
                        "{scale}/{width}/{light}/{key} {rect:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn removable_recovery_controls_check_both_paths_with_separate_practice() {
    let id = "lpic2-efi-removable-recovery";
    let (context, mut app, index) = open(id);
    draw(&context, &mut app);
    tap(&context, &mut app, "example");
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(false));
    tap(&context, &mut app, "worked");
    tap(&context, &mut app, "insert-0");
    tap(&context, &mut app, "run");
    assert!(app.metrics.efi_loader.result.is_none());
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(false));
    tap(&context, &mut app, "insert-1");
    tap(&context, &mut app, "run");
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(true));
    assert!(!progress::current_solved(
        &app.progress.attempts[id],
        &app.scenarios[index]
    ));
    let State::EfiLoader { simulation } = &app.progress.attempts[id].state else {
        panic!()
    };
    let engine::Goal::EfiLoader { definition } = &app.scenarios[index].goal else {
        panic!()
    };
    assert_eq!(simulation.assignment.current, definition.initial.initial);
    let repair = definition.initial.reference[0].clone();
    tap(&context, &mut app, "assignment");
    type_command(&context, &mut app, &repair);
    tap(&context, &mut app, "run");
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(true));
    assert!(progress::current_solved(
        &app.progress.attempts[id],
        &app.scenarios[index]
    ));
    let wrong = repair.replace("/rescue/recovery-boot", "/rescue/copied-boot");
    type_command(&context, &mut app, &wrong);
    tap(&context, &mut app, "run");
    assert!(app.metrics.efi_loader.result.is_none());
    tap(&context, &mut app, "test");
    assert_eq!(app.metrics.efi_loader.result, Some(false));
}
