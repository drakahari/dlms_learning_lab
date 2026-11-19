use super::*;
const ID: &str = "lpic1-host-resolver";
fn view(a: &mut LabApp, v: teaching::View) {
    let State::HostNetwork { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn painted(ctx: &egui::Context, a: &mut LabApp, width: f32) -> Vec<String> {
    fn collect(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(t) => out.push(t.galley.job.text.clone()),
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    for _ in 0..3 {
        frame(ctx, a, width, 7000., vec![]);
    }
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 7000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for shape in out.shapes {
        collect(&shape.shape, &mut text);
    }
    text
}
#[test]
fn direct_dns_teaches_before_practice_and_keeps_independent_answers_closed() {
    assert!(LabApp::current_catalog_for_session(
        true,
        false,
        Some("direct-dns-batch")
    ));
    let (ctx, mut app, _) = super::nm_lessons::opened(ID);
    let intro = painted(&ctx, &mut app, 1600.).join("\n");
    for doubled in ["1. 1.", "2. 2.", "3. 3.", "4. 4."] {
        assert!(
            !intro.contains(doubled),
            "Duplicate concept numbering: {doubled}"
        );
    }
    for word in [
        "@10.73.0.53",
        "+short",
        "A asks for an IPv4 address",
        "final dot",
        "Expected output",
        "host -t A",
        "without @",
        "getent",
    ] {
        assert!(intro.contains(word), "{word}: {intro}");
    }
    view(&mut app, teaching::View::Example);
    let practice = painted(&ctx, &mut app, 1600.).join("\n");
    for word in [
        "Command",
        "Run command",
        "Actual results appear in the terminal",
        "Save resolver",
        "Check example",
    ] {
        assert!(practice.contains(word), "{word}");
    }
    view(&mut app, teaching::View::Assignment);
    let assignment = painted(&ctx, &mut app, 1600.).join("\n");
    assert!(!assignment.contains("nameserver 203.0.113.53"));
    assert!(!assignment.contains("dig @10.73.0.53"));
    assert!(
        !app.metrics
            .host
            .controls
            .iter()
            .any(|c| c.0 == "fill-example")
    );
    assert!(assignment.contains("inventory.lab.test"));
    assert!(assignment.contains("Preserve"));
}
#[test]
fn direct_dns_preserves_shared_control_alignment_at_supported_scales_and_reflow_widths() {
    let (ctx, mut app, _) = super::nm_lessons::opened(ID);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.theme_id = if light {
                theme::inventory()
                    .iter()
                    .find(|t| t.light)
                    .unwrap()
                    .id
                    .clone()
            } else {
                theme::Preferences::default().theme
            };
            theme::apply(&ctx, &app.theme_id);
            for width in [760., 1600.] {
                for v in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    view(&mut app, v);
                    let _ = painted(&ctx, &mut app, width);
                    if v != teaching::View::Introduction {
                        let (_, rect, _) = app
                            .metrics
                            .host
                            .fields
                            .iter()
                            .find(|r| r.0 == "command")
                            .unwrap();
                        assert!(
                            rect.width() > 50. && rect.min.x >= 0. && rect.max.x <= width + 1.,
                            "{scale} {width} {rect:?}"
                        );
                        let (_, button, _) = app
                            .metrics
                            .host
                            .controls
                            .iter()
                            .find(|r| r.0 == "run")
                            .unwrap();
                        assert!(
                            (rect.height() - button.height()).abs() < 1.,
                            "Related single-line input/action heights {rect:?} {button:?}"
                        );
                        assert!(app.metrics.host.fields.iter().any(|r| r.0 == "resolver"));
                    }
                }
            }
        }
    }
}
#[test]
fn direct_dns_actual_command_widget_runs_queries_and_keeps_current_success() {
    let (ctx, mut app, i) = super::nm_lessons::opened(ID);
    view(&mut app, teaching::View::Assignment);
    let engine::Goal::HostNetwork { definition } = app.scenarios[i].goal.clone() else {
        panic!()
    };
    let State::HostNetwork { simulation } = &mut app.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation
        .run(
            &definition,
            &dlms_learning_lab::host_workspace::Event::Resolver {
                text: "nameserver 203.0.113.53\n".into(),
            },
            false,
        )
        .unwrap();
    simulation.assignment.checked = true;
    for input in [
        "dig @203.0.113.54 service.lab.test. A +short",
        "host -t A service.lab.test. 203.0.113.54",
    ] {
        let _ = painted(&ctx, &mut app, 1600.);
        let (_, rect, id) = app
            .metrics
            .host
            .fields
            .iter()
            .find(|r| r.0 == "command")
            .cloned()
            .unwrap();
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut app,
                1600.,
                7000.,
                vec![
                    egui::Event::PointerMoved(rect.center()),
                    egui::Event::PointerButton {
                        pos: rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        frame(
            &ctx,
            &mut app,
            1600.,
            7000.,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                },
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        command: true,
                        ..Default::default()
                    },
                },
                egui::Event::Text(input.into()),
            ],
        );
        let State::HostNetwork { simulation } = &app.progress.attempts[ID].state else {
            panic!()
        };
        assert_eq!(
            simulation.assignment.command, input,
            "Select-all/type must replace the complete previous query"
        );
        frame(
            &ctx,
            &mut app,
            1600.,
            7000.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        let State::HostNetwork { simulation } = &app.progress.attempts[ID].state else {
            panic!()
        };
        assert!(
            simulation
                .assignment
                .transcript
                .last()
                .unwrap()
                .contains("203.0.113.90\n"),
            "Actual result for {input}: command={:?}; output={:?}",
            simulation.assignment.command,
            simulation.assignment.transcript.last()
        );
        assert!(simulation.assignment.checked);
        assert!(definition.checks(simulation, false).iter().all(|r| r.0));
        if input.starts_with("host") {
            assert!(
                simulation
                    .assignment
                    .transcript
                    .last()
                    .unwrap()
                    .contains("Using domain server:")
            );
        }
        frame(
            &ctx,
            &mut app,
            1600.,
            7000.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
}
