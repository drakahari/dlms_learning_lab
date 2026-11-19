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
                .links
                .fields
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        } else {
            a.metrics
                .links
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
    panic!("Link target never visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "command", true);
    click(ctx, a, r);
    let id = a
        .metrics
        .links
        .fields
        .iter()
        .find(|r| r.0 == "command")
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
    tap(ctx, a, "run");
}
#[test]
fn links_lesson_real_terminal_controls_grade_all_four_and_resume_preserves_drafts() {
    for id in [
        "lpic1-links-identity",
        "lpic1-links-relative",
        "lpic1-links-move",
        "lpic1-links-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.links.result.unwrap().1 > 0);
        let s = a.scenarios[i].clone();
        for cmd in &s.solution {
            command(&ctx, &mut a, cmd);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.links.result.unwrap().1, 0, "{id} {:?}", a.notice);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let restored = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        let victim = if id.ends_with("identity") {
            "/delivery/live"
        } else if id.ends_with("relative") {
            "/delivery/latest"
        } else if id.ends_with("move") {
            "/delivery/receipt"
        } else {
            "/delivery/current"
        };
        command(&ctx, &mut a, &format!("rm {victim}"));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.links.result.unwrap().1 > 0);
    }
}
#[test]
fn links_all_three_views_render_left_aligned_and_separate_example_never_earns_credit() {
    for id in [
        "lpic1-links-identity",
        "lpic1-links-relative",
        "lpic1-links-move",
        "lpic1-links-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let engine::State::Links { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let before = simulation.assignment.clone();
        tap(&ctx, &mut a, "example");
        let engine::Goal::Links { definition } = &source.goal else {
            panic!()
        };
        for cmd in &definition.example_commands {
            command(&ctx, &mut a, cmd);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.links.result.unwrap().1, 0);
        let engine::State::Links { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment, before);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "introduction");
        tap(&ctx, &mut a, "assignment");
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            for width in [760., 1600.] {
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                for (_, r, _) in &a.metrics.links.fields {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "{id}: {r:?} {clip:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn links_painted_views_teach_specific_examples_and_never_reveal_assignment_solutions() {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                assert!(!t.galley.job.justify);
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out);
                }
            }
            _ => {}
        }
    }
    for id in [
        "lpic1-links-identity",
        "lpic1-links-relative",
        "lpic1-links-move",
        "lpic1-links-handover",
    ] {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            if let State::Links { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
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
                    assert!(text.contains("Worked example"));
                    assert!(text.contains(&entry.worked[0].value));
                }
                teaching::View::Example => {
                    assert!(text.contains(&entry.practice[0].title));
                    assert!(text.contains("Actual practice terminal"));
                    assert!(text.contains("Not checked"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("Worked example"));
                    for command in &source.solution {
                        assert!(!text.contains(command), "{id} leaked {command}");
                    }
                }
            }
        }
    }
}

#[test]
fn links_worked_commands_stay_complete_inside_columns_at_large_text_and_narrow_widths() {
    fn collect<'a>(shape: &'a egui::Shape, rows: &mut Vec<&'a egui::epaint::TextShape>) {
        match shape {
            egui::Shape::Text(t) => rows.push(t),
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, rows);
                }
            }
            _ => {}
        }
    }
    let catalog = pack::ccna_pilot().unwrap();
    for source in catalog.iter().filter(|s| s.id.starts_with("lpic1-links-")) {
        let expected = &lesson_presentation::entry(source).unwrap().worked;
        for scale in [1., 1.5, 2.] {
            for width in [480., 1200.] {
                let ctx = egui::Context::default();
                ctx.set_zoom_factor(scale);
                let mut allowed = 0.;
                let out = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 9000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let full = ui.available_width();
                            allowed = if full >= ui.text_style_height(&egui::TextStyle::Body) * 38.
                            {
                                (full - ui.spacing().item_spacing.x) / 2.
                            } else {
                                full
                            };
                            lesson_presentation::wrapped_worked_commands(ui, source);
                        });
                    },
                );
                let mut texts = vec![];
                for shape in &out.shapes {
                    collect(&shape.shape, &mut texts);
                }
                for row in expected {
                    let t = texts
                        .iter()
                        .find(|t| t.galley.job.text == row.value)
                        .expect("Full command including destination is painted");
                    assert!(
                        t.galley.size().x <= allowed + 1.,
                        "{} {width}/{scale}: {}",
                        source.id,
                        row.value
                    );
                    assert!(!t.galley.job.justify);
                }
            }
        }
    }
}
