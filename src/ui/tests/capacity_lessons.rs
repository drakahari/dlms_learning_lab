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
                .capacity
                .fields
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        } else {
            a.metrics
                .capacity
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
    panic!("Capacity target never visible: {key}")
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
        .capacity
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
fn action(ctx: &egui::Context, a: &mut LabApp, op: &dlms_learning_lab::capacity_workspace::Action) {
    use dlms_learning_lab::capacity_workspace::Action;
    match op {
        Action::Put { path, bytes } => {
            field(ctx, a, "path", path);
            field(ctx, a, "bytes", &bytes.to_string());
            tap(ctx, a, "put");
        }
        Action::Remove { names } => {
            for name in names {
                tap(ctx, a, &format!("select-{name}"));
            }
            tap(ctx, a, "preview");
            tap(ctx, a, "remove");
        }
    }
}
#[test]
fn capacity_real_widgets_grade_all_four_then_invalidate_damage_and_preserve_save() {
    for id in [
        "lpic2-capacity-inodes",
        "lpic2-capacity-blocks",
        "lpic2-capacity-snapshot",
        "lpic2-capacity-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.capacity.result.unwrap().1 > 0);
        let source = a.scenarios[i].clone();
        for cmd in &source.solution {
            let op = serde_json::from_str(cmd.strip_prefix("capacity-action ").unwrap()).unwrap();
            action(&ctx, &mut a, &op);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(
            a.metrics.capacity.result.unwrap().1,
            0,
            "{id} {:?}",
            a.notice
        );
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let restored = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        field(&ctx, &mut a, "bytes", "not a number");
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "put");
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
    }
}
#[test]
fn capacity_all_practice_widgets_are_isolated_and_worked_actions_only_prepare_drafts() {
    for id in [
        "lpic2-capacity-inodes",
        "lpic2-capacity-blocks",
        "lpic2-capacity-snapshot",
        "lpic2-capacity-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let engine::Goal::Capacity { definition } = &source.goal else {
            panic!()
        };
        tap(&ctx, &mut a, "example");
        let before = a.progress.attempts[id].state.clone();
        tap(&ctx, &mut a, "fill-example");
        let State::Capacity { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let State::Capacity { simulation: old } = &before else {
            panic!()
        };
        assert_eq!(simulation.practice.world, old.practice.world);
        assert_eq!(simulation.assignment, old.assignment);
        // Reset only practice selection/draft, then perform every action using its real controls.
        tap(&ctx, &mut a, "reset-example");
        for op in &definition.example_actions {
            action(&ctx, &mut a, op);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.capacity.result.unwrap().1, 0);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "assignment");
        let State::Capacity { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment, old.assignment);
    }
}
#[test]
fn capacity_fields_remain_inside_viewport_at_all_supported_scales_and_themes() {
    let (ctx, mut a, _) = opened("lpic2-capacity-handover");
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
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                for (_, r, _) in &a.metrics.capacity.fields {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "{scale} {width} {r:?} {clip:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn capacity_three_painted_views_teach_before_work_and_do_not_expose_assignment_solutions() {
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
        "lpic2-capacity-inodes",
        "lpic2-capacity-blocks",
        "lpic2-capacity-snapshot",
        "lpic2-capacity-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            if let State::Capacity { simulation } =
                &mut a.progress.attempts.get_mut(id).unwrap().state
            {
                simulation.view = view;
            }
            let mut text = String::new();
            for _ in 0..4 {
                let output = ctx.run(
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
                for s in output.shapes {
                    collect(&s.shape, &mut text);
                }
            }
            let entry = lesson_presentation::entry(&source).unwrap();
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("WHAT YOU’LL LEARN"));
                    for n in 1..=3 {
                        assert!(
                            !text.contains(&format!("{n}. {n}. ")),
                            "duplicated section numbering: {id}"
                        );
                    }
                    assert!(text.contains("Worked example"));
                    assert!(text.contains(&entry.worked[0].value));
                }
                teaching::View::Example => {
                    assert!(text.contains(&entry.practice[0].title));
                    assert!(text.contains("Current capacity"));
                    assert!(text.contains("Not checked"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("Worked example"));
                    assert!(!text.contains("One valid approach"));
                    for command in &source.solution {
                        assert!(!text.contains(command));
                    }
                }
            }
        }
    }
}

#[test]
fn capacity_native_editor_keyboard_order_and_view_changes_retain_unfinished_drafts() {
    let (ctx, mut a, i) = opened("lpic2-capacity-inodes");
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "path", "/unfinished");
    let bytes = a
        .metrics
        .capacity
        .fields
        .iter()
        .find(|r| r.0 == "bytes")
        .unwrap()
        .2;
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_eq!(ctx.memory(|m| m.focused()), Some(bytes));
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Text("4x".into())],
    );
    let before = a.progress.attempts[&a.scenarios[i].id].state.clone();
    tap(&ctx, &mut a, "introduction");
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "assignment");
    let State::Capacity { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    let State::Capacity { simulation: old } = &before else {
        panic!()
    };
    assert_eq!(simulation.assignment, old.assignment);
    assert_eq!(simulation.assignment.path, "/unfinished");
    assert_eq!(simulation.assignment.bytes, "4x");
}
