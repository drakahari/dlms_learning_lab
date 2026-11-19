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
        // A cooperative job can enable this widget on the next paint.
        frame(ctx, a, 1200., 950., vec![]);
        let r = if field {
            a.metrics.dhcp_config.editor.expect("editor").0
        } else {
            a.metrics
                .dhcp_config
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
    panic!("DHCP target never visible: {key}")
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
    let id = a.metrics.dhcp_config.editor.unwrap().1;
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
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
            },
            egui::Event::Paste(text.into()),
        ],
    );
    let id = &a.scenarios[a.selected.unwrap()].id;
    let State::DhcpConfig { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    let draft = if simulation.view == teaching::View::Example {
        &simulation.practice.draft
    } else {
        &simulation.assignment.draft
    };
    assert_eq!(
        draft, text,
        "Ctrl+A and paste must replace the whole multiline editor"
    );
}

const IDS: [&str; 2] = ["lpic2-dhcp-dynamic-ranges", "lpic2-dhcp-option-scope"];
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..70 {
        frame(ctx, a, 1200., 950., vec![]);
        if a.dhcp_config_run.is_none() {
            // Paint the now-enabled editor before sending a new pointer event.
            frame(ctx, a, 1200., 950., vec![]);
            return;
        }
    }
    panic!("bounded DHCP run did not finish");
}
fn result(a: &LabApp) -> Option<(usize, usize)> {
    a.metrics.dhcp_config.result.as_ref().map(|r| (r.0, r.1))
}
fn definition(s: &Scenario) -> &dlms_learning_lab::dhcp_server_workspace::Definition {
    let engine::Goal::DhcpConfig { definition } = &s.goal else {
        panic!()
    };
    definition
}
#[test]
fn dhcp_config_actual_editor_wrong_correct_stale_retry_and_save_read() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert!(result(&a).unwrap().0 < 2);
        let raw = s.solution[0]
            .strip_prefix(dlms_learning_lab::dhcp_server_workspace::PREFIX)
            .unwrap();
        field(&ctx, &mut a, "editor", raw);
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((2, 2)));
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let restored = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        field(&ctx, &mut a, "editor", "subnet {");
        assert!(result(&a).is_none());
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((0, 2)));
        field(&ctx, &mut a, "editor", raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((2, 2)));
    }
}
#[test]
fn dhcp_config_worked_insert_is_inert_and_examples_remain_separate() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let d = definition(&s);
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "insert");
        let State::DhcpConfig { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice.draft, d.example_worked);
        assert!(simulation.practice.tested.is_none());
        assert_eq!(simulation.assignment.draft, d.initial.starter);
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((2, 2)));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn dhcp_config_stop_keyboard_run_escape_and_navigation_cancel_inflight_work() {
    let (ctx, mut a, i) = opened(IDS[1]);
    tap(&ctx, &mut a, "assignment");
    let s = a.scenarios[i].clone();
    let raw = s.solution[0]
        .strip_prefix(dlms_learning_lab::dhcp_server_workspace::PREFIX)
        .unwrap();
    field(&ctx, &mut a, "editor", raw);
    let ctrl_enter = egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::CTRL,
    };
    frame(&ctx, &mut a, 1200., 950., vec![ctrl_enter]);
    assert!(a.dhcp_config_run.is_some());
    let r = target(&ctx, &mut a, "stop", false);
    click(&ctx, &mut a, r);
    assert!(a.dhcp_config_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[IDS[1]], &s));
    let r = target(&ctx, &mut a, "editor", true);
    click(&ctx, &mut a, r);
    let id = a.metrics.dhcp_config.editor.unwrap().1;
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(!ctx.memory(|m| m.has_focus(id)));
    let run = target(&ctx, &mut a, "run", false);
    click(&ctx, &mut a, run);
    assert!(a.dhcp_config_run.is_some());
    a.selected = None;
    frame(&ctx, &mut a, 1200., 950., vec![]);
    assert!(a.dhcp_config_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[IDS[1]], &s));
}
#[test]
fn dhcp_config_editor_and_run_fit_supported_scales_light_dark_wide_narrow() {
    let (ctx, mut a, _) = opened("lpic2-dhcp-option-scope");
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
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
                let r = a.metrics.dhcp_config.editor.unwrap().0;
                assert!(
                    r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                    "scale{scale} width{width} {r:?} {clip:?}"
                );
                let button = a
                    .metrics
                    .dhcp_config
                    .controls
                    .iter()
                    .find(|r| r.0 == "run")
                    .unwrap()
                    .1;
                assert!(button.left() >= clip.left() - 2. && button.right() <= clip.right() + 2.);
            }
        }
    }
}
#[test]
fn dhcp_config_three_actual_views_teach_new_concepts_show_unchecked_and_hide_solutions() {
    fn collect(s: &egui::Shape, o: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                if t.galley.rows.len() > 1 && t.galley.job.text.len() > 80 {
                    assert!(!t.galley.job.justify);
                }
                o.push_str(&t.galley.job.text);
                o.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, o)
                }
            }
            _ => {}
        }
    }
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            if let State::DhcpConfig { simulation } =
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
                    assert!(text.contains(&entry.concepts[0].title));
                    assert!(text.contains("Worked example"));
                    assert!(!text.contains("1. 1."));
                }
                teaching::View::Example => {
                    assert!(text.contains(&entry.practice[0].title));
                    assert!(text.contains("Not checked"));
                    assert!(text.contains("Insert worked configuration"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("Insert worked configuration"));
                    assert!(!text.contains("Worked example"));
                    assert!(
                        !text.contains(
                            source.solution[0]
                                .strip_prefix(dlms_learning_lab::dhcp_server_workspace::PREFIX)
                                .unwrap()
                        )
                    );
                }
            }
        }
    }
}

#[test]
fn dhcp_config_mixed_results_name_the_failing_case_and_preserve_the_correct_case() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((1, 2)));
    assert!(!a.metrics.completion);
    fn collect(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
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
    let mut text = String::new();
    for _ in 0..3 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 6000.),
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
    assert!(text.contains("Configuration needs work · 1/2"), "{text}");
    assert!(text.contains("Passed · Available addresses"), "{text}");
    assert!(
        text.contains("Needs work · Two occupied addresses"),
        "{text}"
    );
    assert!(text.contains("No available address"), "{text}");
}

#[test]
fn dhcp_config_editor_reserves_readable_rows_and_new_attempts_do_not_reuse_old_scroll() {
    for source in IDS {
        let (ctx, mut a, i) = opened(source);
        tap(&ctx, &mut a, "assignment");
        for _ in 0..4 {
            frame(&ctx, &mut a, 1600., 3000., vec![]);
        }
        let font = egui::TextStyle::Monospace.resolve(&ctx.style());
        let row = ctx.fonts_mut(|f| f.row_height(&font));
        let (height, offset) = a.metrics.dhcp_config.editor_scroll.unwrap();
        assert!(height >= 9. * row, "editor collapsed: {height}, row{row}");
        assert_eq!(offset, 0., "newly opened editor starts at the header");
        let s = a.scenarios[i].clone();
        let raw = s.solution[0]
            .strip_prefix(dlms_learning_lab::dhcp_server_workspace::PREFIX)
            .unwrap();
        field(&ctx, &mut a, "editor", raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        // A new visit uses the actual new attempt namespace, not an old inner viewport.
        a.launch(i);
        let State::DhcpConfig { simulation } =
            &mut a.progress.attempts.get_mut(source).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        for _ in 0..4 {
            frame(&ctx, &mut a, 1600., 3000., vec![]);
        }
        let (height, offset) = a.metrics.dhcp_config.editor_scroll.unwrap();
        assert!(height >= 9. * row);
        assert_eq!(offset, 0.);
    }
}
