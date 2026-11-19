//! Actual egui controls/content, separate from Fedora native and novice evidence.
use super::*;
use dlms_learning_lab::lvm_workspace as work;
const IDS: [&str; 3] = smoke::lvm_provisioning::cow::IDS;
fn open(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn sim(a: &LabApp, i: usize) -> &work::Session {
    let State::Lvm { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn view(a: &mut LabApp, i: usize, v: teaching::View) {
    let engine::Goal::Lvm { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation.set_view(definition, v).unwrap();
    a.lesson_scroll_reset = true;
}
fn rendered(ctx: &egui::Context, a: &mut LabApp) -> String {
    rendered_events(ctx, a, vec![])
}
fn rendered_events(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) -> String {
    fn text(shape: &egui::Shape, s: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                assert!(!t.galley.job.justify);
                s.push_str(&t.galley.job.text);
                s.push('\n');
            }
            egui::Shape::Vec(v) => {
                for shape in v {
                    text(shape, s)
                }
            }
            _ => {}
        }
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 9000.),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut s = String::new();
    for shape in output.shapes {
        text(&shape.shape, &mut s);
    }
    s
}
fn control(a: &LabApp, key: &str) -> egui::Rect {
    smoke::lvm_provisioning::control(a, key).unwrap_or_else(|| panic!("Missing {key}"))
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    rendered_events(
        ctx,
        a,
        vec![
            egui::Event::PointerMoved(r.center()),
            egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
    );
    rendered_events(
        ctx,
        a,
        vec![egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
    rendered(ctx, a);
}
#[test]
fn cow_all_three_screens_teach_actual_tools_and_hide_worked_assignments_by_default() {
    for id in IDS {
        let (ctx, mut a, i) = open(id);
        let engine::Goal::Lvm { definition } = &a.scenarios[i].goal.clone() else {
            panic!()
        };
        let text = rendered(&ctx, &mut a);
        assert!(
            text.contains("Worked example") && text.contains("simulator"),
            "{id}: {text}"
        );
        for concept in &definition.lesson.concepts {
            assert!(text.contains(&concept.title), "{}", concept.title);
        }
        for step in definition.example_steps() {
            assert!(text.contains(&step.text));
            assert!(text.contains(&step.explanation));
        }
        view(&mut a, i, teaching::View::Example);
        let text = rendered(&ctx, &mut a);
        assert!(
            text.contains("Current application value")
                && text.contains("Application records and saved views")
        );
        assert!(
            text.contains("Insert selected command")
                || text.contains("Apply selected simulator action")
        );
        view(&mut a, i, teaching::View::Assignment);
        let text = rendered(&ctx, &mut a);
        assert!(text.contains(&a.scenarios[i].title));
        assert!(!text.contains("Worked example") && !text.contains(work::snapshot_flow::PREFIX));
        for cmd in &a.scenarios[i].solution {
            if !cmd.starts_with(work::snapshot_flow::PREFIX) {
                assert!(!text.contains(cmd), "Unrequested solution: {cmd}");
            }
        }
        assert!(!a.metrics.completion);
    }
}
#[test]
fn cow_real_update_click_changes_only_example_and_reset_retains_assignment_draft() {
    let (ctx, mut a, i) = open(IDS[0]);
    let engine::Goal::Lvm { definition } = &a.scenarios[i].goal.clone() else {
        panic!()
    };
    {
        let State::Lvm { simulation } = &mut a.progress.attempts.get_mut(IDS[0]).unwrap().state
        else {
            panic!()
        };
        simulation
            .set_command(definition, "vgs".into(), false)
            .unwrap();
    }
    view(&mut a, i, teaching::View::Example);
    rendered(&ctx, &mut a);
    let before = sim(&a, i).current.clone();
    let r = control(&a, "snapshot-update-0");
    click(&ctx, &mut a, r);
    assert_ne!(sim(&a, i).example, sim(&a, i).example_initial);
    assert_eq!(sim(&a, i).current, before);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    rendered(&ctx, &mut a);
    let r = control(&a, "reset-example");
    click(&ctx, &mut a, r);
    assert_eq!(sim(&a, i).example, sim(&a, i).example_initial);
    assert_eq!(sim(&a, i).command, "vgs");
}
#[test]
fn cow_reader_and_error_states_show_current_bytes_not_expected_values_and_real_copy_invalidates_checks()
 {
    let (ctx, mut a, i) = open(IDS[2]);
    view(&mut a, i, teaching::View::Assignment);
    let text = rendered(&ctx, &mut a);
    assert!(text.contains("Copy unavailable · invalid saved view"));
    assert!(
        text.contains("bad--000")
            && text.contains("good-000")
            && text.contains("Invalid · old data unavailable")
    );
    assert!(
        a.metrics
            .lvm
            .reader_records
            .iter()
            .any(|r| r.0.contains(":expired:") && !r.1)
    );
    {
        let State::Lvm { simulation } = &mut a.progress.attempts.get_mut(IDS[2]).unwrap().state
        else {
            panic!()
        };
        simulation.checked = true;
    }
    let r = control(&a, "snapshot-copy-recovery/report-trusted");
    click(&ctx, &mut a, r);
    assert!(!sim(&a, i).checked);
    assert_eq!(
        dlms_learning_lab::lvm::snapshot::read_record(
            &sim(&a, i).current,
            "recovery/report",
            None,
            "row0"
        )
        .unwrap(),
        "good-000"
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &a.scenarios[i]
    ));
}
#[test]
fn cow_control_sizes_use_shared_metrics_with_reflow_and_no_assignment_solution_at_large_text() {
    for id in IDS {
        let (ctx, mut a, i) = open(id);
        view(&mut a, i, teaching::View::Assignment);
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for w in [1268., 820.] {
                frame(&ctx, &mut a, w, 1000., vec![]);
                frame(&ctx, &mut a, w, 1000., vec![]);
                let r = control(&a, "command");
                assert!(r.width() > 20. && r.height() > 10.);
                let available = a.metrics.main_viewport.unwrap();
                assert!(r.right() <= available.right() + 1., "{id} at {scale}/{w}");
                for (_, r, _) in &a.metrics.lvm.controls {
                    assert!(r.is_finite() && r.width() > 0. && r.height() > 0.);
                }
            }
        }
    }
}
