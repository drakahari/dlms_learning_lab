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
            a.metrics.linux_logging.editor.expect("editor").0
        } else {
            a.metrics
                .linux_logging
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
    panic!("Logging target never visible: {key}")
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
    let id = a.metrics.linux_logging.editor.unwrap().1;
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
    let State::LinuxLogging { simulation } = &a.progress.attempts[id].state else {
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

const IDS: [&str; 4] = [
    "lpic1-syslog-threshold",
    "lpic1-syslog-exact",
    "lpic1-syslog-privacy",
    "lpic1-syslog-handover",
];
#[test]
fn linux_logging_native_edit_run_wrong_correct_stale_damage_retry_and_save_read() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        assert!(a.metrics.linux_logging.result.unwrap().0 > 0);
        let text = source.solution[0]
            .strip_prefix(dlms_learning_lab::linux_logging_workspace::PREFIX)
            .unwrap();
        field(&ctx, &mut a, "editor", text);
        tap(&ctx, &mut a, "run");
        assert_eq!(
            a.metrics.linux_logging.result.unwrap().0,
            0,
            "{id} notice={:?}, state={:?}, checks={:?}",
            a.notice,
            a.progress.attempts[id].state,
            engine::checks(&a.progress.attempts[id].state, &source.goal)
        );
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let got = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&got).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        field(&ctx, &mut a, "editor", "*.* /var/log/operations.log\n");
        assert!(a.metrics.linux_logging.result.is_none());
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "run");
        assert!(a.metrics.linux_logging.result.unwrap().0 > 0);
        field(&ctx, &mut a, "editor", text);
        tap(&ctx, &mut a, "run");
        assert!(a.metrics.completion);
    }
}
#[test]
fn linux_logging_worked_fill_does_not_run_and_all_examples_are_separate() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let original = a.progress.attempts[id].state.clone();
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "run");
        assert!(a.metrics.linux_logging.result.unwrap().0 > 0);
        tap(&ctx, &mut a, "insert");
        assert!(a.metrics.linux_logging.result.is_none());
        let State::LinuxLogging { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let State::LinuxLogging { simulation: before } = &original else {
            panic!()
        };
        assert_eq!(simulation.assignment, before.assignment);
        tap(&ctx, &mut a, "run");
        assert_eq!(a.metrics.linux_logging.result.unwrap().0, 0);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "reset-example");
        let State::LinuxLogging { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice, before.practice);
        assert_eq!(simulation.assignment, before.assignment);
    }
}
#[test]
fn linux_logging_editor_and_run_fit_supported_scales_light_dark_wide_narrow() {
    let (ctx, mut a, _) = opened("lpic1-syslog-handover");
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "run");
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
                let r = a.metrics.linux_logging.editor.unwrap().0;
                assert!(
                    r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                    "scale{scale} width{width} {r:?} {clip:?}"
                );
                let button = a
                    .metrics
                    .linux_logging
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
fn linux_logging_three_actual_views_teach_new_concepts_show_unchecked_and_hide_solutions() {
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
            if let State::LinuxLogging { simulation } =
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
                                .strip_prefix(dlms_learning_lab::linux_logging_workspace::PREFIX)
                                .unwrap()
                        )
                    );
                }
            }
        }
    }
}

#[test]
fn linux_logging_multiline_select_all_keyboard_test_and_escape_release_focus() {
    let (ctx, mut a, i) = opened("lpic1-syslog-exact");
    tap(&ctx, &mut a, "assignment");
    let source = a.scenarios[i].clone();
    field(
        &ctx,
        &mut a,
        "editor",
        source.solution[0]
            .strip_prefix(dlms_learning_lab::linux_logging_workspace::PREFIX)
            .unwrap(),
    );
    let id = a.metrics.linux_logging.editor.unwrap().1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        }],
    );
    assert!(progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
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
}
