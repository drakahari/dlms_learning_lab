use super::*;
const ID: &str = "lpic1-grub-bios-install";
fn open() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    (ctx, a, i)
}
fn view(a: &mut LabApp, v: teaching::View) {
    let State::BiosLoader { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
    a.focus_input = true;
}
fn draw(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1500., 1200., vec![]);
    }
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    a.smoke_stage += 1;
    for _ in 0..30 {
        let r = a
            .metrics
            .bios_loader
            .controls
            .iter()
            .find(|r| r.0 == key)
            .unwrap()
            .1;
        if a.reveal_smoke_control(ctx, r) {
            click(ctx, a, r);
            draw(ctx, a);
            return;
        }
        let events = std::mem::take(&mut a.pending_events);
        frame(ctx, a, 1500., 1200., events);
    }
    panic!("native BIOS control {key} did not settle");
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    tap(ctx, a, "input");
    let id = a.metrics.bios_loader.input.unwrap().1;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        ctx,
        a,
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
            egui::Event::Paste(text.into()),
        ],
    );
    draw(ctx, a);
}
#[test]
fn bios_console_real_input_run_test_feedback_and_invalidation() {
    let (ctx, mut a, i) = open();
    view(&mut a, teaching::View::Assignment);
    draw(&ctx, &mut a);
    tap(&ctx, &mut a, "test");
    assert_eq!(a.metrics.bios_loader.result, Some(false));
    for cmd in [
        "grub-install /dev/vda",
        "grub-install --boot-directory='/target/boot' /dev/vda",
    ] {
        edit(&ctx, &mut a, cmd);
        tap(&ctx, &mut a, "run");
        assert!(a.metrics.bios_loader.result.is_none());
        tap(&ctx, &mut a, "test");
        assert_eq!(
            a.metrics.bios_loader.result,
            Some(cmd.contains("/target/boot"))
        );
    }
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    edit(&ctx, &mut a, "grub-install /dev/vda");
    assert!(
        progress::current_solved(&a.progress.attempts[ID], &a.scenarios[i]),
        "typing is pending input, not an installation"
    );
    tap(&ctx, &mut a, "run");
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    tap(&ctx, &mut a, "test");
    assert_eq!(a.metrics.bios_loader.result, Some(false));
    edit(&ctx, &mut a, "grub-install --force /dev/vda");
    tap(&ctx, &mut a, "run");
    let State::BiosLoader { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.assignment.transcript.contains("Unsupported"));
    assert_eq!(
        simulation.assignment.command,
        "grub-install --force /dev/vda"
    );
    frame(
        &ctx,
        &mut a,
        1500.,
        1200.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(ctx.memory(|m| m.focused()).is_some());
}
#[test]
fn bios_practice_is_separate_insertion_is_inert_challenge_hides_worked_answers_and_reset_is_selected()
 {
    let (ctx, mut a, i) = open();
    let engine::Goal::BiosLoader { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let d = (**definition).clone();
    view(&mut a, teaching::View::Example);
    draw(&ctx, &mut a);
    tap(&ctx, &mut a, "worked");
    tap(&ctx, &mut a, "insert-0");
    let State::BiosLoader { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.practice.command, d.example.reference[0]);
    assert_eq!(simulation.practice.current, d.example.initial);
    assert!(simulation.practice.current.checked.is_none());
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "test");
    assert_eq!(a.metrics.bios_loader.result, Some(true));
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    let State::BiosLoader { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.current, d.initial.initial);
    let other = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-grub-persistent-default")
        .unwrap();
    a.launch(other);
    let old = serde_json::to_value(&a.progress.attempts["lpic1-grub-persistent-default"]).unwrap();
    a.launch(i);
    view(&mut a, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let reset = a.metrics.reset.unwrap();
    click(&ctx, &mut a, reset);
    draw(&ctx, &mut a);
    let confirm = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, confirm);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts["lpic1-grub-persistent-default"]).unwrap(),
        old
    );
    // Switch through the real mode control; no reference commands or tabs appear.
    let challenge = a.metrics.mode_challenge.unwrap();
    click(&ctx, &mut a, challenge);
    draw(&ctx, &mut a);
    assert!(!a.metrics.bios_loader.controls.iter().any(|r| matches!(
        r.0.as_str(),
        "example" | "introduction" | "worked" | "insert-0"
    )));
}
#[test]
fn bios_controls_reflow_at_supported_scales_widths_and_themes() {
    let (ctx, mut a, _) = open();
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for light in [false, true] {
            a.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&ctx, &a.theme_id);
            for (w, h) in [(1500., 1100.), (760., 1000.)] {
                for v in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    view(&mut a, v);
                    for _ in 0..3 {
                        frame(&ctx, &mut a, w, h, vec![]);
                    }
                    assert!(a.metrics.back.is_some() && a.metrics.reset.is_some());
                    if v != teaching::View::Introduction {
                        if scale == 1. && w == 1500. {
                            assert!(a.metrics.bios_loader.side_by_side);
                        }
                        if scale == 2. && w == 760. {
                            assert!(!a.metrics.bios_loader.side_by_side);
                        }
                    }
                    for (key, r, _) in &a.metrics.bios_loader.controls {
                        assert!(r.is_finite() && r.is_positive(), "{key}");
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "{v:?}/{scale}/{w}/{key} {r:?}"
                        );
                    }
                }
            }
        }
    }
}
