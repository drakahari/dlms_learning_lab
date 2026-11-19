//! Real egui events and paint regressions; these are not native pixel acceptance.
use super::*;
use crate::ui::smoke::shared_libraries_lessons as route;
use std::collections::BTreeSet;
fn flatten<'a>(
    shape: &'a egui::Shape,
    clip: egui::Rect,
    out: &mut Vec<(egui::Rect, &'a egui::Shape)>,
) {
    if let egui::Shape::Vec(items) = shape {
        for item in items {
            flatten(item, clip, out);
        }
    } else {
        out.push((clip, shape));
    }
}
fn actual_text(output: &egui::FullOutput) -> String {
    let mut shapes = vec![];
    for s in &output.shapes {
        flatten(&s.shape, s.clip_rect, &mut shapes);
    }
    shapes
        .into_iter()
        .filter_map(|(_, s)| {
            if let egui::Shape::Text(t) = s {
                Some(t.galley.text())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn actual_label(
    output: &egui::FullOutput,
    rect: egui::Rect,
    words: Option<&str>,
    font: Option<f32>,
) {
    let mut shapes = vec![];
    for s in &output.shapes {
        flatten(&s.shape, s.clip_rect, &mut shapes);
    }
    let tol = 1. / output.pixels_per_point;
    assert!(shapes.iter().enumerate().any(|(at,(clip,shape))|{
        let egui::Shape::Text(t)=shape else{return false};let bounds=t.visual_bounding_rect();
        if words.is_some_and(|w|!t.galley.text().contains(w))||t.galley.elided||t.opacity_factor<=0.||t.angle!=0.
            ||!bounds.is_positive()||!rect.expand(tol).contains_rect(bounds)||!clip.expand(tol).contains_rect(bounds){return false;}
        if let Some(size)=font{assert!(t.galley.job.sections.iter().all(|s|(s.format.font_id.size-size).abs()<0.01));}
        let glyphs=t.galley.rows.iter().any(|r|!r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty());
        glyphs&&!shapes[at+1..].iter().any(|(later,s)|matches!(s,egui::Shape::Rect(r)if r.fill.a()==255&&later.intersect(r.rect).intersects(bounds)))
    }),"Actual unclipped opaque-glyph label missing in {rect:?}: {words:?}");
}
#[test]
fn libraries_full_fixed_route_uses_actual_controls_preserves_scope_and_requests_each_real_capture()
{
    full_route(egui::vec2(1250., 1000.), false);
}
#[test]
fn library_navigation_acknowledges_start_after_an_intervening_native_style_render() {
    full_route(egui::vec2(1268., 632.), true);
}
fn full_route(initial_pixels: egui::Vec2, render_between_inputs: bool) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let expected = route::shared_libraries_expected_captures(&a.scenarios);
    assert!(!expected.is_empty());
    let catalog = serde_json::to_vec(&a.scenarios).unwrap();
    let units = serde_json::to_vec(&a.units).unwrap();
    let mut pixels = initial_pixels;
    let mut shots = BTreeSet::new();
    let mut closed = false;
    let mut goal_layout = None;
    for n in 0_usize..12_000 {
        let events = std::mem::take(&mut a.pending_events);
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                pixels / a.scale,
            )),
            time: Some(n as f64 * 0.05),
            events,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.);
        let output = ctx.run(input, |ctx| {
            if let Some(i) = a.selected {
                lesson_presentation::paint::request(ctx, &a.scenarios[i]);
            }
            a.render(ctx);
            goal_layout = a.selected.and_then(|i| {
                a.metrics.main_viewport.and_then(|clip| {
                    lesson_presentation::paint::collect(
                        ctx,
                        &a.scenarios[i],
                        clip.intersect(ctx.content_rect()),
                    )
                })
            });
            if !render_between_inputs || n.is_multiple_of(2) {
                a.shared_libraries_lessons_tick(ctx);
            }
        });
        for viewport in output.viewport_output.values() {
            for command in &viewport.commands {
                match command {
                    egui::ViewportCommand::InnerSize(size) => {
                        if !render_between_inputs {
                            pixels = *size;
                        }
                        // The constrained fixture retains Fedora's actual tiled size
                        // through zoom/resize requests, so 200% uses a real small viewport.
                    }
                    egui::ViewportCommand::Screenshot(data) => {
                        let name = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap();
                        assert!(expected.contains(name));
                        assert!(
                            shots.insert(name.clone()),
                            "No invented/duplicate capture phase"
                        );
                        let text = actual_text(&output);
                        assert!(text.contains("DLMS Learning Lab"));
                        let source = &a.scenarios[a.selected.unwrap()];
                        let m = &a.metrics.shared_libraries;
                        if name.ends_with("-introduction") {
                            actual_label(&output, m.intro_goal.unwrap(), None, Some(21.));
                            assert!(
                                text.contains("shared library") || text.contains("shared code")
                            );
                        } else if name.ends_with("-challenge-assignment")
                            || name.ends_with("-assignment-initial")
                        {
                            let rows = goal_layout.as_ref().expect("Actual same-pass task rows");
                            let actual = lesson_presentation::paint::output_evidence(
                                &ctx, source, rows, &output,
                            );
                            assert!(
                                actual.all_witnessed(),
                                "Normal-sized whole assignment capture must paint every actual requirement row"
                            );
                            assert_eq!(
                                a.metrics.lesson_tabs.is_empty(),
                                name.ends_with("-challenge-assignment")
                            );
                            assert!(!text.contains("/opt/demo/"));
                            if name.ends_with("-assignment-initial") {
                                let (
                                    engine::State::SharedLibraries { simulation: actual },
                                    engine::State::SharedLibraries {
                                        simulation: initial,
                                    },
                                ) = (&a.progress.attempts[&source.id].state, &source.initial)
                                else {
                                    panic!()
                                };
                                assert_eq!(
                                    serde_json::to_value(&actual.assignment).unwrap(),
                                    serde_json::to_value(&initial.assignment).unwrap(),
                                    "Untouched assignment is shown before any assignment answer"
                                );
                            }
                        } else if name.ends_with("-success-invalidated") {
                            assert!(m.result.is_none());
                            assert!(text.contains("Not checked"));
                        } else if name.ends_with("-practice-workspace") {
                            let key = if source.id == "lpic1-library-search" {
                                "launch:/usr/bin/brief"
                            } else {
                                "configuration"
                            };
                            let field = m
                                .fields
                                .iter()
                                .find(|f| f.0 == key)
                                .expect("Actual practice editor");
                            actual_label(&output, field.1, Some("/opt/demo"), None);
                            let (passed, total, _) = m
                                .result
                                .expect("Practice remains checked while its workspace is shown");
                            assert_eq!(passed, total);
                        } else if name.ends_with("-earlier-inspection") {
                            assert!(text.contains("Earlier inspection"));
                        } else if name.ends_with("-actual-saved-inspection") {
                            assert!(text.contains("Actual saved-setting inspection"));
                            assert!(m.inspection.is_some());
                        } else if name.contains("-light200-") {
                            assert!((a.scale - 2.).abs() < 0.001);
                            assert!(theme::find(&a.theme_id).light);
                            assert!(
                                ctx.content_rect().width() < 900.,
                                "Actually narrow logical viewport"
                            );
                            assert!(
                                text.contains("System search configuration")
                                    || text.contains("Investigation terminal")
                                    || text.contains("Actual command history")
                            );
                        } else {
                            let (passed, total, rect) =
                                m.result.expect("Captured feedback summary");
                            actual_label(
                                &output,
                                rect,
                                Some(if passed == total {
                                    "Passed"
                                } else {
                                    "Needs work"
                                }),
                                None,
                            );
                        }
                        a.capture_pending = false; // Provider acknowledges command delivery; no PNG is fabricated.
                    }
                    egui::ViewportCommand::Close => closed = true,
                    _ => {}
                }
            }
        }
        if closed {
            break;
        }
    }
    assert!(
        closed,
        "Fixed real-event controller reached its bounded end"
    );
    assert_eq!(shots, expected);
    assert_eq!(serde_json::to_vec(&a.scenarios).unwrap(), catalog);
    assert_eq!(serde_json::to_vec(&a.units).unwrap(), units);
}

// Independent small field regression, with product launch only for setup.
// No Session setter, grading mutation or solution installation is used.
fn open_example() -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-library-search")
        .unwrap();
    a.launch(i);
    (ctx, a)
}
fn paint_frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    time: f64,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1250., 1000.) / a.scale,
            )),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    )
}
fn settle(ctx: &egui::Context, a: &mut LabApp, time: &mut f64) {
    for _ in 0..4 {
        *time += 0.05;
        paint_frame(ctx, a, *time, vec![]);
    }
}
fn real_click(ctx: &egui::Context, a: &mut LabApp, time: &mut f64, key: &str) {
    for wheel in 0..=20 {
        let (_, rect, clip) = a
            .metrics
            .shared_libraries
            .controls
            .iter()
            .find(|r| r.0 == key)
            .cloned()
            .expect("Actual current control");
        let clip = clip
            .intersect(a.metrics.main_viewport.unwrap())
            .intersect(ctx.content_rect());
        if crate::ui::smoke::smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            *time += 0.05;
            paint_frame(
                ctx,
                a,
                *time,
                vec![egui::Event::PointerMoved(rect.center())],
            );
            let (_, rect, clip) = a
                .metrics
                .shared_libraries
                .controls
                .iter()
                .find(|r| r.0 == key)
                .cloned()
                .unwrap();
            assert!(crate::ui::smoke::smoke_control_visible(
                rect,
                clip,
                ctx.pixels_per_point()
            ));
            for pressed in [true, false] {
                *time += 0.05;
                paint_frame(
                    ctx,
                    a,
                    *time,
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
            settle(ctx, a, time);
            return;
        }
        assert!(wheel < 20, "Bounded actual control reveal");
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let dy = if rect.bottom() > main.bottom() {
            main.bottom() - rect.bottom()
        } else {
            main.top() - rect.top()
        };
        *time += 0.05;
        paint_frame(
            ctx,
            a,
            *time,
            vec![
                egui::Event::PointerMoved(route::shared_libraries_outer_wheel_point(
                    a,
                    main,
                    ctx.pixels_per_point(),
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        settle(ctx, a, time);
    }
    unreachable!()
}
#[test]
fn library_singleline_placeholders_and_real_values_use_native_centered_geometry_and_scope_ids() {
    let (ctx, mut a) = open_example();
    let mut time = 0.;
    settle(&ctx, &mut a, &mut time);
    real_click(&ctx, &mut a, &mut time, "view:Example");
    reveal_field(&ctx, &mut a, &mut time, "command");
    time += 0.05;
    let output = paint_frame(&ctx, &mut a, time, vec![]);
    let mut shapes = vec![];
    for s in &output.shapes {
        flatten(&s.shape, s.clip_rect, &mut shapes);
    }
    let command = a
        .metrics
        .shared_libraries
        .fields
        .iter()
        .find(|f| f.0 == "command")
        .unwrap();
    let placeholder = shapes
        .iter()
        .find_map(|(clip, s)| {
            if let egui::Shape::Text(t) = s {
                (t.galley.text() == "ldd PROGRAM"
                    && command
                        .1
                        .expand(1.)
                        .contains_rect(egui::Rect::from_min_size(t.pos, t.galley.size())))
                .then_some((*clip, egui::Rect::from_min_size(t.pos, t.galley.size())))
            } else {
                None
            }
        })
        .expect("Actual native placeholder");
    assert!(
        (placeholder.1.center().y - command.1.center().y).abs() <= 2. / output.pixels_per_point,
        "No enlarged outer box/top-pinned placeholder"
    );
    assert!(placeholder.0.contains_rect(placeholder.1));
    let example_id = command.2;
    real_click(&ctx, &mut a, &mut time, "view:Assignment");
    let assignment_id = a
        .metrics
        .shared_libraries
        .fields
        .iter()
        .find(|f| f.0 == "command")
        .unwrap()
        .2;
    assert_ne!(
        example_id, assignment_id,
        "Actual source/scope native IDs are distinct"
    );
    real_click(&ctx, &mut a, &mut time, "view:Example");
    assert_eq!(
        a.metrics
            .shared_libraries
            .fields
            .iter()
            .find(|f| f.0 == "command")
            .unwrap()
            .2,
        example_id
    );
}

fn reveal_field(ctx: &egui::Context, a: &mut LabApp, time: &mut f64, key: &str) {
    for wheel in 0..=20 {
        let (_, rect, _, own) = a
            .metrics
            .shared_libraries
            .fields
            .iter()
            .find(|r| r.0 == key)
            .cloned()
            .expect("Actual field");
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if crate::ui::smoke::smoke_control_visible(
            rect,
            own.intersect(main),
            ctx.pixels_per_point(),
        ) {
            return;
        }
        assert!(
            wheel < 20,
            "Field requires more than20 actual wheel actions"
        );
        let dy = if rect.bottom() > main.bottom() {
            main.bottom() - rect.bottom()
        } else {
            main.top() - rect.top()
        };
        *time += 0.05;
        paint_frame(
            ctx,
            a,
            *time,
            vec![
                egui::Event::PointerMoved(route::shared_libraries_outer_wheel_point(
                    a,
                    main,
                    ctx.pixels_per_point(),
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        settle(ctx, a, time);
    }
}
#[test]
fn library_populated_native_field_remains_centered_and_paste_does_not_run_at_supported_text_sizes()
{
    for scale in [1., 1.5, 2.] {
        let (ctx, mut a) = open_example();
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        let mut time = 0.;
        settle(&ctx, &mut a, &mut time);
        real_click(&ctx, &mut a, &mut time, "view:Example");
        reveal_field(&ctx, &mut a, &mut time, "command");
        let (_, rect, id, _) = a
            .metrics
            .shared_libraries
            .fields
            .iter()
            .find(|r| r.0 == "command")
            .cloned()
            .unwrap();
        time += 0.05;
        paint_frame(
            &ctx,
            &mut a,
            time,
            vec![egui::Event::PointerMoved(rect.center())],
        );
        let (_, rect, _, _) = a
            .metrics
            .shared_libraries
            .fields
            .iter()
            .find(|r| r.0 == "command")
            .cloned()
            .unwrap();
        for pressed in [true, false] {
            time += 0.05;
            paint_frame(
                &ctx,
                &mut a,
                time,
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
        time += 0.05;
        paint_frame(
            &ctx,
            &mut a,
            time,
            vec![egui::Event::Paste("ldd /usr/bin/brief".into())],
        );
        settle(&ctx, &mut a, &mut time);
        let source = &a.scenarios[a.selected.unwrap()];
        let engine::State::SharedLibraries { simulation } = &a.progress.attempts[&source.id].state
        else {
            panic!()
        };
        assert_eq!(simulation.example.command, "ldd /usr/bin/brief");
        assert!(simulation.example.transcript.is_empty());
        assert!(!simulation.current_success(match &source.goal {
            engine::Goal::SharedLibraries { definition } => definition,
            _ => panic!(),
        }));
        time += 0.05;
        let output = paint_frame(&ctx, &mut a, time, vec![]);
        let field = a
            .metrics
            .shared_libraries
            .fields
            .iter()
            .find(|r| r.0 == "command")
            .unwrap()
            .1;
        let mut shapes = vec![];
        for s in &output.shapes {
            flatten(&s.shape, s.clip_rect, &mut shapes);
        }
        let actual = shapes
            .iter()
            .find_map(|(clip, s)| {
                if let egui::Shape::Text(t) = s {
                    let layout = egui::Rect::from_min_size(t.pos, t.galley.size());
                    (t.galley.text() == "ldd /usr/bin/brief"
                        && field.expand(1.).contains_rect(layout))
                    .then_some((*clip, layout, t))
                } else {
                    None
                }
            })
            .expect("Actual populated TextEdit galley, not guide text");
        assert!((actual.1.center().y - field.center().y).abs() <= 2. / output.pixels_per_point);
        assert!(actual.0.contains_rect(actual.2.visual_bounding_rect()));
        assert!(!actual.2.galley.elided);
        assert!(actual.2.opacity_factor > 0.);
    }
}

#[test]
fn library_workspace_actually_splits_at_normal_width_and_stacks_with_enlarged_text() {
    for scale in [1., 1.5, 2.] {
        let (ctx, mut app) = open_example();
        app.scale = scale;
        ctx.set_zoom_factor(scale);
        let mut time = 0.;
        settle(&ctx, &mut app, &mut time);
        real_click(&ctx, &mut app, &mut time, "view:Example");
        let fields = &app.metrics.shared_libraries.fields;
        let launch = fields
            .iter()
            .find(|f| f.0 == "launch:/usr/bin/brief")
            .expect("Actual application input")
            .1;
        let history = app
            .metrics
            .shared_libraries
            .history
            .expect("Actual terminal viewport");
        if scale == 1. {
            assert!(
                history.left() >= launch.right(),
                "Normal workspace must use distinct columns: {launch:?} / {history:?}"
            );
        } else {
            assert!(
                history.top() > launch.bottom(),
                "Enlarged workspace must stack: {launch:?} / {history:?}"
            );
        }
        assert!(launch.is_positive() && history.is_positive());
    }
}
