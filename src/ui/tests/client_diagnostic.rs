use super::*;
use dlms_learning_lab::client_names_workspace as work;
const ID: &str = "lpic1-client-name-investigation";
#[test]
fn client_diagnostic_practice_requirements_keep_the_absolute_missing_name_with_its_outcome() {
    fn text(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(t) => out.push(t.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    text(shape, out);
                }
            }
            _ => {}
        }
    }
    let (ctx, mut app, _) = super::nm_lessons::opened(ID);
    view(&mut app, teaching::View::Example);
    for _ in 0..3 {
        frame(&ctx, &mut app, 1600., 7000., vec![]);
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600., 7000.),
            )),
            ..Default::default()
        },
        |ctx| app.render(ctx),
    );
    let mut labels = vec![];
    for shape in &output.shapes {
        text(&shape.shape, &mut labels);
    }
    let rows = labels
        .iter()
        .filter(|s| s.starts_with("• "))
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        4,
        "Actual painted practice requirement rows: {rows:?}"
    );
    assert!(rows[3].ends_with("missing.workshop.lab.test. unresolved."));
    assert!(
        !rows
            .iter()
            .any(|s| s.trim_start_matches("• ") == "unresolved.")
    );
}

fn view(app: &mut LabApp, v: teaching::View) {
    let State::ClientNames { simulation } = &mut app.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    app.lesson_scroll_reset = true;
}
#[test]
fn client_diagnostic_reflows_with_centered_command_and_no_unsolicited_assignment_answers() {
    let (ctx, mut app, _) = super::nm_lessons::opened(ID);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            for width in [760., 1600.] {
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
                for v in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    view(&mut app, v);
                    for _ in 0..3 {
                        frame(&ctx, &mut app, width, 7000., vec![]);
                    }
                    assert!((ctx.zoom_factor() - scale).abs() < 0.01);
                    if v == teaching::View::Introduction {
                        assert!(app.metrics.client_names.diagnostic_input.is_none());
                        continue;
                    }
                    let m = &app.metrics.client_names;
                    let State::ClientNames { simulation } = &app.progress.attempts[ID].state else {
                        panic!()
                    };
                    let scope = if v == teaching::View::Example {
                        &simulation.practice
                    } else {
                        &simulation.assignment
                    };
                    assert_eq!(
                        m.editor_rows,
                        Some(scope.draft[&scope.selected].split('\n').count()),
                        "Configuration rows must not visually turn into new records"
                    );
                    for key in ["file", "run", "diagnostic-input", "diagnostic-run", "case"] {
                        let (_, r, c) = m.controls.iter().find(|r| r.0 == key).unwrap();
                        assert!(
                            r.width() > 0.
                                && r.left() >= c.left() - 1.
                                && r.right() <= c.right() + 1.,
                            "{key} {scale} {width} {r:?} {c:?}"
                        );
                    }
                    let input = m.diagnostic_input.unwrap().0;
                    let run = m
                        .controls
                        .iter()
                        .find(|r| r.0 == "diagnostic-run")
                        .unwrap()
                        .1;
                    assert!((input.height() - run.height()).abs() < 1.1);
                    assert!(m.diagnostic_status.is_none() && m.result.is_none());
                    if v == teaching::View::Assignment {
                        assert!(!m.controls.iter().any(|r| r.0 == "insert"));
                    }
                }
            }
        }
    }
    app.progress.attempts.get_mut(ID).unwrap().challenge = true;
    frame(&ctx, &mut app, 1600., 7000., vec![]);
    assert!(app.metrics.client_names.diagnostic_input.is_some());
    assert!(
        !app.metrics
            .client_names
            .controls
            .iter()
            .any(|r| matches!(r.0.as_str(), "example" | "introduction" | "insert"))
    );
}
#[test]
fn client_diagnostic_enter_runs_current_drafts_and_edits_hide_stale_output_without_awarding_credit()
{
    let (ctx, mut app, _) = super::nm_lessons::opened(ID);
    view(&mut app, teaching::View::Assignment);
    for _ in 0..3 {
        frame(&ctx, &mut app, 1600., 7000., vec![]);
    }
    let (rect, id) = app.metrics.client_names.diagnostic_input.unwrap();
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
                    modifiers: Default::default(),
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
        vec![egui::Event::Text("getent ahostsv4 report".into())],
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
            modifiers: Default::default(),
        }],
    );
    frame(&ctx, &mut app, 1600., 7000., vec![]);
    assert_eq!(app.metrics.client_names.diagnostic_status, Some(Ok(0)));
    assert!(!app.metrics.completion);
    let State::ClientNames { simulation } = &mut app.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation
        .assignment
        .edit("nsswitch.conf", "hosts: dns files")
        .unwrap();
    for _ in 0..2 {
        frame(&ctx, &mut app, 1600., 7000., vec![]);
    }
    assert!(app.metrics.client_names.diagnostic_status.is_none());
    assert!(!app.metrics.completion);
}
#[test]
fn client_diagnostic_current_grade_invalidates_after_damage_and_practice_stays_separate() {
    let (ctx, mut app, index) = super::nm_lessons::opened(ID);
    let source = app.scenarios[index].clone();
    let d = work::source(&source).unwrap();
    let State::ClientNames { simulation } = &mut app.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.practice.replace(&d.example_worked).unwrap();
    simulation.practice.run(&d.example).unwrap();
    assert!(!d.solved(simulation));
    let solution =
        serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
    simulation.assignment.replace(&solution).unwrap();
    simulation.assignment.run(&d.initial).unwrap();
    simulation.view = teaching::View::Assignment;
    for _ in 0..3 {
        frame(&ctx, &mut app, 1600., 7000., vec![]);
    }
    assert_eq!(
        app.metrics.client_names.result.map(|r| (r.0, r.1)),
        Some((2, 2))
    );
    let State::ClientNames { simulation } = &mut app.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation
        .assignment
        .edit("nsswitch.conf", "hosts: files [NOTFOUND=return] dns")
        .unwrap();
    for _ in 0..2 {
        frame(&ctx, &mut app, 1600., 7000., vec![]);
    }
    assert!(app.metrics.client_names.result.is_none());
    assert!(!app.metrics.completion);
}
