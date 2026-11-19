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
            a.metrics.client_names.editor.expect("editor").0
        } else {
            a.metrics
                .client_names
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
    panic!("Name-source target never visible: {key}")
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
    let id = a.metrics.client_names.editor.unwrap().1;
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
    let State::ClientNames { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    let draft = if simulation.view == teaching::View::Example {
        &simulation.practice.draft
    } else {
        &simulation.assignment.draft
    };
    assert_eq!(
        &draft[if simulation.view == teaching::View::Example {
            &simulation.practice.selected
        } else {
            &simulation.assignment.selected
        }],
        text,
        "Ctrl+A and paste must replace the whole multiline editor"
    );
}

const IDS: [&str; 2] = ["lpic1-dns-search-suffixes", "lpic1-dns-absolute-precedence"];
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..70 {
        frame(ctx, a, 1200., 950., vec![]);
        if a.client_names_run.is_none() {
            // Paint the now-enabled editor before sending a new pointer event.
            frame(ctx, a, 1200., 950., vec![]);
            return;
        }
    }
    panic!("bounded name-source run did not finish");
}
fn result(a: &LabApp) -> Option<(usize, usize)> {
    a.metrics.client_names.result.as_ref().map(|r| (r.0, r.1))
}
fn fill(ctx: &egui::Context, a: &mut LabApp, raw: &str) {
    let files: dlms_learning_lab::client_names_workspace::Files =
        serde_json::from_str(raw).unwrap();
    let multiple = files.len() > 1;
    for (name, text) in files {
        if multiple {
            tap(ctx, a, "file");
            tap(ctx, a, &format!("file:{name}"));
        }
        field(ctx, a, "editor", &text);
    }
}
fn definition(s: &Scenario) -> &dlms_learning_lab::client_names_workspace::Definition {
    let engine::Goal::ClientNames { definition } = &s.goal else {
        panic!()
    };
    definition
}
#[test]
fn resolver_search_actual_editor_wrong_correct_stale_retry_and_save_read() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert!(result(&a).unwrap().0 < definition(&s).initial.cases.len());
        let raw = s.solution[0]
            .strip_prefix(dlms_learning_lab::client_names_workspace::PREFIX)
            .unwrap();
        fill(&ctx, &mut a, raw);
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(
            result(&a),
            Some((
                definition(&s).initial.cases.len(),
                definition(&s).initial.cases.len()
            ))
        );
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
        assert_eq!(result(&a), Some((0, definition(&s).initial.cases.len())));
        fill(&ctx, &mut a, raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(
            result(&a),
            Some((
                definition(&s).initial.cases.len(),
                definition(&s).initial.cases.len()
            ))
        );
    }
}
#[test]
fn resolver_search_worked_insert_is_inert_and_examples_remain_separate() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let d = definition(&s);
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "insert");
        let State::ClientNames { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice.draft, d.example_worked);
        assert!(simulation.practice.tested.is_none());
        assert_eq!(simulation.assignment.draft, d.initial.starter);
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(
            result(&a),
            Some((
                definition(&s).initial.cases.len(),
                definition(&s).initial.cases.len()
            ))
        );
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn resolver_search_stop_keyboard_run_escape_and_navigation_cancel_inflight_work() {
    let (ctx, mut a, i) = opened(IDS[1]);
    tap(&ctx, &mut a, "assignment");
    let s = a.scenarios[i].clone();
    let raw = s.solution[0]
        .strip_prefix(dlms_learning_lab::client_names_workspace::PREFIX)
        .unwrap();
    fill(&ctx, &mut a, raw);
    let ctrl_enter = egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::CTRL,
    };
    frame(&ctx, &mut a, 1200., 950., vec![ctrl_enter]);
    assert!(a.client_names_run.is_some());
    let r = target(&ctx, &mut a, "stop", false);
    click(&ctx, &mut a, r);
    assert!(a.client_names_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[IDS[1]], &s));
    let r = target(&ctx, &mut a, "editor", true);
    click(&ctx, &mut a, r);
    let id = a.metrics.client_names.editor.unwrap().1;
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
    assert!(a.client_names_run.is_some());
    a.selected = None;
    frame(&ctx, &mut a, 1200., 950., vec![]);
    assert!(a.client_names_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[IDS[1]], &s));
}
#[test]
fn resolver_search_editor_and_run_fit_supported_scales_light_dark_wide_narrow() {
    let (ctx, mut a, _) = opened(IDS[1]);
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
                let r = a.metrics.client_names.editor.unwrap().0;
                assert!(
                    r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                    "scale{scale} width{width} {r:?} {clip:?}"
                );
                let button = a
                    .metrics
                    .client_names
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
fn resolver_search_actual_three_views_teach_and_keep_assignment_solutions_hidden() {
    fn collect(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                assert!(
                    !t.galley.job.justify,
                    "Unexpected justified text: {:?}",
                    t.galley.job.text
                );
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, text)
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
            let State::ClientNames { simulation } =
                &mut a.progress.attempts.get_mut(id).unwrap().state
            else {
                panic!()
            };
            simulation.view = view;
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
                    assert!(text.contains(&entry.concepts[0].title));
                    assert!(text.contains("Worked example"));
                    assert!(text.contains(&entry.worked[0].value));
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
                    assert!(!text.contains("studio.workshop"));
                }
            }
        }
    }
}
#[test]
fn resolver_search_editor_keeps_readable_height_and_file_selection_is_persistent() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for _ in 0..4 {
            frame(&ctx, &mut a, 900., 950., vec![]);
        }
        let (height, _) = a.metrics.client_names.editor_scroll.unwrap();
        let expected = ctx.style().text_styles[&egui::TextStyle::Monospace].size * 8.;
        assert!(height >= expected, "height{height} expected{expected}");
    }
    let s = a.scenarios[i].clone();
    let State::ClientNames { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.selected, "resolv.conf");
    assert!(!progress::current_solved(&a.progress.attempts[IDS[0]], &s));
}

#[test]
fn resolver_search_actual_trace_names_candidates_and_omits_unrelated_source_controls() {
    fn collect(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, text)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut app, _) = opened(IDS[0]);
    tap(&ctx, &mut app, "assignment");
    tap(&ctx, &mut app, "run");
    settle(&ctx, &mut app);
    let mut text = String::new();
    for _ in 0..4 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 8000.),
                )),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
        text.clear();
        for shape in output.shapes {
            collect(&shape.shape, &mut text);
        }
    }
    assert!(text.contains("Actual lookup traces"));
    assert!(text.contains("/etc/resolv.conf excerpt"));
    assert!(text.contains("Supplied DNS sample"));
    assert!(text.contains("report.south.office.lab.test"));
    assert!(text.contains("No matching A record"));
    assert!(text.contains("Actual result:"));
    assert!(!text.contains("Local hosts source"));
    assert!(
        !app.metrics
            .client_names
            .controls
            .iter()
            .any(|c| c.0 == "file")
    );
}
