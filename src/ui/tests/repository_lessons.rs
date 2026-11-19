use super::*;
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    for _ in 0..24 {
        let r = if field {
            a.metrics
                .repository
                .fields
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        } else {
            a.metrics
                .repository
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Repository target never visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key, true);
    click(ctx, a, r);
    let id = a
        .metrics
        .repository
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    frame(
        ctx,
        a,
        1200.,
        950.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            },
            egui::Event::Text(text.into()),
        ],
    );
}

fn action(
    ctx: &egui::Context,
    a: &mut LabApp,
    event: &dlms_learning_lab::repository_workspace::Event,
) {
    use dlms_learning_lab::repository_workspace::{Edit, Event};
    match event {
        Event::Edit {
            edit:
                Edit::Settings {
                    alias,
                    enabled,
                    uri,
                },
        } => {
            tap(ctx, a, &format!("edit-{alias}"));
            let source = &a.scenarios[a.selected.unwrap()];
            let State::Repositories { simulation } = &a.progress.attempts[&source.id].state else {
                panic!()
            };
            let current = if a
                .metrics
                .repository
                .controls
                .iter()
                .any(|r| r.0 == "reset-example")
            {
                simulation.practice.enabled
            } else {
                simulation.assignment.enabled
            };
            if current != *enabled {
                tap(ctx, a, "enabled");
            }
            field(ctx, a, "uri", uri);
            tap(ctx, a, "apply");
        }
        Event::Edit {
            edit: Edit::Refresh { alias },
        } => {
            tap(ctx, a, &format!("edit-{alias}"));
            tap(ctx, a, "refresh");
        }
        Event::Query { package } => {
            field(ctx, a, "package", package);
            tap(ctx, a, "query");
        }
    }
}
#[test]
fn repository_all_four_real_native_edit_query_flows_and_damage_clear_success() {
    for id in [
        "lpic1-repo-visibility",
        "lpic1-repo-provenance",
        "lpic1-repo-offline-evidence",
        "lpic1-repo-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.repository.result.unwrap().1 > 0);
        for cmd in &source.solution {
            let e = serde_json::from_str(cmd.strip_prefix("repository-action ").unwrap()).unwrap();
            action(&ctx, &mut a, &e);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(
            a.metrics.repository.result.unwrap().1,
            0,
            "{id} {:?}",
            a.notice
        );
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let got = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&got).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        field(
            &ctx,
            &mut a,
            "uri",
            "file:///virtual-repositories/unprovided",
        );
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "apply");
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.repository.result.unwrap().1 > 0);
    }
}
#[test]
fn repository_worked_preparation_never_executes_and_all_separate_examples_pass() {
    for id in [
        "lpic1-repo-visibility",
        "lpic1-repo-provenance",
        "lpic1-repo-offline-evidence",
        "lpic1-repo-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let engine::Goal::Repositories { definition } = &source.goal else {
            panic!()
        };
        tap(&ctx, &mut a, "example");
        let before = a.progress.attempts[id].state.clone();
        tap(&ctx, &mut a, "fill-example");
        let State::Repositories { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let State::Repositories { simulation: old } = &before else {
            panic!()
        };
        assert_eq!(simulation.practice.world, old.practice.world);
        assert_eq!(simulation.practice.history, old.practice.history);
        assert_eq!(simulation.assignment, old.assignment);
        tap(&ctx, &mut a, "reset-example");
        for event in &definition.example_actions {
            action(&ctx, &mut a, event);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(
            a.metrics.repository.result.unwrap().1,
            0,
            "{id} {:?}",
            a.notice
        );
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "assignment");
        let State::Repositories { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment, old.assignment);
    }
}
#[test]
fn repository_fields_fit_light_dark_and_supported_scales_with_populated_values() {
    let (ctx, mut a, _) = opened("lpic1-repo-handover");
    tap(&ctx, &mut a, "assignment");
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
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
            for width in [760., 1600.] {
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                assert_eq!(a.metrics.repository.headers.len(), 4);
                assert_eq!(a.metrics.repository.rows.len(), 2);
                for row in &a.metrics.repository.rows {
                    for (cell, header) in row.iter().zip(&a.metrics.repository.headers) {
                        assert!(
                            (cell.left() - header.left()).abs() < 1.,
                            "unaligned repository column at scale {scale}, width {width}: {cell:?} {header:?}"
                        );
                    }
                }
                for (_, r, _) in &a.metrics.repository.fields {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "scale={scale} width={width} field={r:?} clip={clip:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn repository_actual_result_and_stale_result_have_truthful_labels_and_no_assignment_answers() {
    fn collect(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                assert!(!t.galley.job.justify);
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    for id in [
        "lpic1-repo-visibility",
        "lpic1-repo-provenance",
        "lpic1-repo-offline-evidence",
        "lpic1-repo-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            if let State::Repositories { simulation } =
                &mut a.progress.attempts.get_mut(id).unwrap().state
            {
                simulation.view = view;
            }
            let mut text = String::new();
            for _ in 0..4 {
                let o = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1500., 6000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                text.clear();
                for s in o.shapes {
                    collect(&s.shape, &mut text);
                }
            }
            let entry = lesson_presentation::entry(&source).unwrap();
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("WHAT YOU’LL LEARN"));
                    assert!(text.contains("Worked example"));
                    assert!(text.contains(&entry.worked[0].value));
                    assert!(!text.contains("1. 1."));
                }
                teaching::View::Example => {
                    assert!(text.contains(&entry.practice[0].title));
                    assert!(text.contains("No actual query yet"));
                    assert!(text.contains("Not checked"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("One valid approach"));
                    assert!(!text.contains("Worked example"));
                    for cmd in &source.solution {
                        assert!(!text.contains(cmd));
                    }
                }
            }
        }
    }
}
