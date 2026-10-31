//! Paired actual-egui flow/capture checks; no solved model or loaded profile installed.
use super::*;
use crate::ui::smoke::literal_lessons as route;
#[derive(Clone)]
struct Window {
    size: egui::Vec2,
    fullscreen: bool,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            size: egui::vec2(760., 900.),
            fullscreen: false,
        }
    }
}
fn tick(
    ctx: &egui::Context,
    a: &mut LabApp,
    window: &mut Window,
    time: f64,
) -> (Option<String>, egui::FullOutput) {
    let events = std::mem::take(&mut a.pending_events);
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| {
            if let egui::Event::Key { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let mut input = egui::RawInput {
        time: Some(time),
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            if window.fullscreen {
                egui::vec2(1920., 1080.)
            } else {
                window.size
            } / ctx.zoom_factor(),
        )),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .fullscreen = Some(window.fullscreen);
    let mut capture = None;
    let output = ctx.run(input, |ctx| {
        a.render(ctx);
        a.save();
        let (advance, shot) = a.literal_lessons_native(ctx, a.smoke_stage);
        if advance {
            a.smoke_stage += 1;
        }
        if shot.is_some() {
            assert!(
                capture.is_none(),
                "At most one capture phase per root frame"
            );
            capture = shot;
        }
    });
    for v in output.viewport_output.values() {
        for c in &v.commands {
            match c {
                // Deliberately ignore InnerSize, as the observed native tiling WM did.
                egui::ViewportCommand::InnerSize(_) => {}
                egui::ViewportCommand::Fullscreen(full) => window.fullscreen = *full,
                _ => {}
            }
        }
    }
    (capture, output)
}
fn painted_rows(s: &egui::Shape, text: &str) -> Option<Vec<egui::Rect>> {
    match s {
        egui::Shape::Text(s)
            if s.galley.text() == text && s.override_text_color.is_none_or(|c| c.a() > 0) =>
        {
            assert!(!s.galley.elided && s.opacity_factor > 0.);
            assert!(s.galley.rows.iter().any(|r| {
                !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty()
            }));
            assert!(s.galley.rows.iter().all(|r| {
                r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                    .iter()
                    .all(|v| {
                        s.override_text_color
                            .unwrap_or_else(|| {
                                if v.color == egui::Color32::PLACEHOLDER {
                                    s.fallback_color
                                } else {
                                    v.color
                                }
                            })
                            .a()
                            > 0
                    })
            }));
            let rows = s
                .galley
                .rows
                .iter()
                .filter(|r| !r.text().trim().is_empty())
                .map(|r| r.rect().translate(s.pos.to_vec2()))
                .collect::<Vec<_>>();
            assert!(!rows.is_empty());
            Some(rows)
        }
        egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| painted_rows(s, text)),
        _ => None,
    }
}
#[test]
fn focused_literal_native_route_uses_real_edit_run_stop_reset_continue_and_every_capture_phase() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = route::BASE;
    assert_eq!((a.scenarios.len(), a.units.len()), (199, 53));
    assert!(a.progress.attempts.is_empty());
    assert!(a._smoke_profile.is_some());
    let (actions, expected) = route::inventory();
    assert!(actions < 1000);
    assert_eq!(
        expected
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        expected.len()
    );
    let mut window = Window::default();
    let mut actual = Vec::new();
    let mut old_stage = a.smoke_stage;
    let mut stagnant = 0;
    let mut ended = false;
    let mut settings = std::collections::BTreeSet::new();
    for frame in 0..30_000 {
        let (shot, output) = tick(&ctx, &mut a, &mut window, frame as f64 * 0.05);
        if a.smoke_stage == old_stage {
            stagnant += 1;
        } else {
            old_stage = a.smoke_stage;
            stagnant = 0;
        }
        assert!(
            stagnant < 3000,
            "Actual phase failed to progress within existing bounded frame premise"
        );
        if let Some(name) = shot {
            let i = a.selected.expect("Real capture has a selected normal task");
            let id = a.scenarios[i].id.clone();
            assert!(route::IDS.contains(&id.as_str()));
            if name.contains("whole-assignment") || name.contains("goal200") {
                let task = &a.scenarios[i].objective;
                let viewport = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let found = output
                    .shapes
                    .iter()
                    .find_map(|s| {
                        painted_rows(&s.shape, task)
                            .map(|rows| (rows, s.clip_rect.intersect(viewport)))
                    })
                    .expect("Actual capture must paint the full nonelided task galley");
                let row = if name.ends_with("bottom") {
                    *found.0.last().unwrap()
                } else {
                    found.0[0]
                };
                assert!(smoke::smoke_control_visible(
                    row,
                    found.1,
                    ctx.pixels_per_point()
                ));
            }
            if name.ends_with("-complete")
                && ["100-", "150-", "200-"].iter().any(|p| name.contains(p))
            {
                let summary = "Overall result · Correct solution · All input sets pass.";
                let viewport = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                assert!(
                    output
                        .shapes
                        .iter()
                        .any(|s| painted_rows(&s.shape, summary).is_some_and(|rows| rows
                            .iter()
                            .all(|r| smoke::smoke_control_visible(
                                *r,
                                s.clip_rect.intersect(viewport),
                                ctx.pixels_per_point()
                            )))),
                    "Capture must paint readable current overall completion"
                );
                assert!(progress::current_solved(
                    &a.progress.attempts[&id],
                    &a.scenarios[i]
                ));
                settings.insert((id, (a.scale * 100.).round() as u32, a.theme_id.clone()));
                if a.scale == 2. {
                    assert!(
                        !window.fullscreen && (window.size.x - 760.).abs() < 1.,
                        "Representative200percent capture must use the actual narrow provider"
                    );
                } else {
                    assert!(
                        window.fullscreen,
                        "Wide capture must acknowledge actual fullscreen"
                    );
                }
            }
            actual.push(name);
        }
        if output.viewport_output.values().any(|v| {
            v.commands
                .iter()
                .any(|c| matches!(c, egui::ViewportCommand::Close))
        }) {
            ended = true;
            break;
        }
    }
    assert!(
        ended,
        "Real route must reach its final Close only after all assertions"
    );
    assert_eq!(actual, expected);
    assert_eq!(settings.len(), 12);
    assert!(a.selected.is_none());
    assert!(a.script_job.is_none());
    for id in route::IDS {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn focused_literal_controller_never_dispatches_twice_in_one_root_frame() {
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = route::BASE;
    let mut first = None;
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1100., 900.),
        )),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .fullscreen = Some(false);
    let _ = ctx.run(input, |ctx| {
        a.render(ctx);
        if first.is_none() {
            first = Some(a.literal_lessons_native(ctx, a.smoke_stage));
        }
        let before = (
            serde_json::to_value(&a.progress).unwrap(),
            a.pending_events.clone(),
            a.smoke_stage,
        );
        assert_eq!(a.literal_lessons_native(ctx, a.smoke_stage), (false, None));
        assert_eq!(
            (
                serde_json::to_value(&a.progress).unwrap(),
                a.pending_events.clone(),
                a.smoke_stage
            ),
            before
        );
    });
    assert!(first.is_some_and(|r| r.0));
}

#[test]
fn literal_window_protocol_accepts_ignored_size_hints_only_with_observed_distinct_flags_and_widths()
{
    let baseline = 946.;
    let tiled = egui::vec2(946., 929.);
    let full = egui::vec2(1920., 1080.);
    assert!(route::window_acknowledged(
        Some(false),
        tiled,
        baseline,
        false
    ));
    assert!(!route::window_acknowledged(
        Some(false),
        full,
        baseline,
        true
    ));
    assert!(!route::window_acknowledged(
        Some(true),
        tiled,
        baseline,
        true
    ));
    assert!(route::window_acknowledged(Some(true), full, baseline, true));
    assert!(!route::window_acknowledged(
        Some(true),
        tiled,
        baseline,
        false
    ));
    assert!(!route::window_acknowledged(
        Some(false),
        full,
        baseline,
        false
    ));
    assert!(!route::window_acknowledged(None, tiled, baseline, false));
    assert!(!route::window_acknowledged(
        Some(false),
        tiled,
        f32::NAN,
        false
    ));
    assert!(!route::window_acknowledged(
        Some(true),
        egui::vec2(f32::INFINITY, 1080.),
        baseline,
        true
    ));
}
#[test]
fn literal_header_only_real_split_clicks_repeat_scale_and_theme_without_controller_or_progress() {
    fn frame(ctx: &egui::Context, a: &mut LabApp, time: &mut f64, events: Vec<egui::Event>) {
        *time += 0.05;
        let mut input = egui::RawInput {
            time: Some(*time),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920., 1080.) / ctx.zoom_factor(),
            )),
            events,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(true);
        let _ = ctx.run(input, |ctx| a.render(ctx));
        assert!(a.progress.attempts.is_empty());
        assert!(a.selected.is_none() && a.script_job.is_none() && a.pending_events.is_empty());
        assert_eq!(a.smoke_stage, route::BASE);
    }
    fn click(
        ctx: &egui::Context,
        a: &mut LabApp,
        time: &mut f64,
        rect: egui::Rect,
        clip: egui::Rect,
    ) {
        assert!(smoke::smoke_control_visible(
            rect,
            clip,
            ctx.pixels_per_point()
        ));
        let pos = rect.center();
        frame(ctx, a, time, vec![egui::Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(
                ctx,
                a,
                time,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                }],
            );
        }
    }
    fn choose(ctx: &egui::Context, a: &mut LabApp, time: &mut f64, scale: bool, key: &str) {
        let started = std::time::Instant::now();
        let rect = if scale {
            a.metrics.reopen_scale
        } else {
            a.metrics.reopen_theme
        }
        .expect("Actual painted header selector");
        click(ctx, a, time, rect, ctx.content_rect());
        let mut pulses = 0;
        for _ in 0..40 {
            assert!(
                started.elapsed().as_secs() < 15,
                "Header selector acknowledgement deadline: {key}"
            );
            if let Some((_, row, clip)) = a
                .metrics
                .physical_picker_choices
                .iter()
                .find(|r| r.0 == key)
                .cloned()
            {
                assert!(egui::Popup::is_any_open(ctx));
                if smoke::smoke_control_visible(row, clip, ctx.pixels_per_point()) {
                    click(ctx, a, time, row, clip);
                    for _ in 0..3 {
                        frame(ctx, a, time, vec![]);
                    }
                    assert!(
                        !egui::Popup::is_any_open(ctx),
                        "Chosen header popup must close: {key}"
                    );
                    return;
                }
                assert!(pulses < 20, "Real header popup wheel bound: {key}");
                pulses += 1;
                frame(
                    ctx,
                    a,
                    time,
                    vec![
                        egui::Event::PointerMoved(egui::pos2(row.center().x, clip.center().y)),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0., clip.center().y - row.center().y),
                            modifiers: Default::default(),
                        },
                    ],
                );
                for _ in 0..10 {
                    frame(ctx, a, time, vec![]);
                }
            } else {
                frame(ctx, a, time, vec![]);
            }
        }
        panic!(
            "Header popup did not paint requested choice: {key}; choices={:?}; pointer={:?}; popup={}",
            a.metrics
                .physical_picker_choices
                .iter()
                .map(|r| &r.0)
                .collect::<Vec<_>>(),
            ctx.input(|i| i.pointer.hover_pos()),
            egui::Popup::is_any_open(ctx)
        );
    }
    let (ctx, mut a) = app();
    a.smoke = true; // Capture metrics only: no controller is invoked by frame().
    a.smoke_stage = route::BASE;
    let original = serde_json::to_value(&a.progress).unwrap();
    let mut time = 0.;
    for _ in 0..3 {
        frame(&ctx, &mut a, &mut time, vec![]);
    }
    for (scale, key) in [
        (true, "scale:1.00"),
        (false, "theme:dlms-light"),
        (true, "scale:1.00"),
        (false, "theme:dlms-dark"),
        (true, "scale:1.50"),
    ] {
        choose(&ctx, &mut a, &mut time, scale, key);
        if scale {
            assert_eq!(format!("scale:{:.2}", a.scale), key);
        } else {
            assert_eq!(format!("theme:{}", a.theme_id), key);
        }
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), original);
        assert!(a.progress.attempts.is_empty());
    }
}

#[test]
fn script_assignments_wrap_closed_reference_headers_without_widening_the_main_page() {
    for id in [
        "shell-trim-options",
        "shell-trim-manifest",
        "shell-handover",
        "shell-ere-event-view",
    ] {
        for width in [720., 760., 946.] {
            for scale in [1., 1.15, 1.25, 1.5, 2.] {
                let (ctx, mut a) = app();
                a.scale = scale;
                let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
                a.launch(i);
                a.set_script_view(i, teaching::View::Assignment);
                for n in 0..5 {
                    let input = egui::RawInput {
                        time: Some(n as f64 * 0.05),
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 900.) / ctx.zoom_factor(),
                        )),
                        ..Default::default()
                    };
                    let _ = ctx.run(input, |ctx| a.render(ctx));
                }
                let viewport = a.metrics.main_viewport.unwrap();
                assert!(
                    a.metrics.main_content_size.x <= viewport.width() + 1. / ctx.pixels_per_point(),
                    "{id}, width {width}, scale {scale}: main content {:?} exceeds {:?}; optional headers must wrap instead of adding a horizontal task scrollbar",
                    a.metrics.main_content_size,
                    viewport
                );
                assert!(!progress::current_solved(
                    &a.progress.attempts[id],
                    &a.scenarios[i]
                ));
            }
        }
    }
}
#[test]
fn offscreen_script_children_do_not_paint_scrollbars_over_the_visible_task() {
    fn horizontal_fills(shape: &egui::Shape, out: &mut Vec<egui::Rect>, bar: f32) {
        match shape {
            egui::Shape::Rect(r)
                if r.fill.a() > 0
                    && r.rect.height() > 0.
                    && r.rect.height() <= bar + 0.5
                    && r.rect.width() > 8. * r.rect.height() =>
            {
                out.push(r.rect)
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    horizontal_fills(shape, out, bar);
                }
            }
            _ => {}
        }
    }
    for view in [teaching::View::Assignment, teaching::View::Example] {
        let (ctx, mut a) = app();
        a.scale = 2.;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "shell-trim-manifest")
            .unwrap();
        a.launch(i);
        a.set_script_view(i, view);
        // Ungraded renderer fixture only. No saved result, solution or mastery is installed.
        let long = format!("# {}\n", "wide-source-".repeat(160));
        if view == teaching::View::Assignment {
            let State::Script { source, run } = &mut a
                .progress
                .attempts
                .get_mut("shell-trim-manifest")
                .unwrap()
                .state
            else {
                panic!()
            };
            *source = long;
            assert!(run.is_none());
        } else {
            a.progress
                .attempts
                .get_mut("shell-trim-manifest")
                .unwrap()
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap()
                .example_source = long;
        }
        let mut output = None;
        let mut ready = false;
        let mut pulses = 0;
        for n in 0..100 {
            let mut input = egui::RawInput {
                events: std::mem::take(&mut a.pending_events),
                time: Some(n as f64 * 0.1),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(946., 929.) / (1.15 * ctx.zoom_factor()),
                )),
                ..Default::default()
            };
            input
                .viewports
                .get_mut(&egui::ViewportId::ROOT)
                .unwrap()
                .native_pixels_per_point = Some(1.15);
            output = Some(ctx.run(input, |ctx| {
                a.render(ctx);
                if n >= 5 {
                    ready = if view == teaching::View::Assignment {
                        let goal = a.metrics.script_goal.unwrap();
                        let font = egui::TextStyle::Body.resolve(&ctx.style());
                        let row = ctx.fonts_mut(|f| f.row_height(&font));
                        let target = egui::Rect::from_min_max(
                            goal.min,
                            egui::pos2(goal.right(), goal.top() + row),
                        );
                        let viewport = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        let visible =
                            smoke::smoke_control_visible(target, viewport, ctx.pixels_per_point());
                        if !visible {
                            assert!(
                                pulses < 20,
                                "Task must be reached with bounded real wheel input"
                            );
                            pulses += 1;
                            let delta = if target.bottom() > viewport.bottom() {
                                viewport.bottom() - target.bottom()
                            } else {
                                viewport.top() - target.top()
                            };
                            a.pending_events.extend([
                                egui::Event::PointerMoved(egui::pos2(
                                    viewport.left() + 1.,
                                    viewport.center().y,
                                )),
                                egui::Event::MouseWheel {
                                    unit: egui::MouseWheelUnit::Point,
                                    delta: egui::vec2(0., delta),
                                    modifiers: Default::default(),
                                },
                            ]);
                        }
                        visible
                    } else {
                        true
                    };
                }
            }));
            if ready {
                break;
            }
        }
        assert!(
            ready,
            "Visible task must be reached through real bounded wheel events"
        );
        assert!((ctx.pixels_per_point() - 2.3).abs() < 0.01);
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let editor = a.metrics.editor_viewport.unwrap();
        assert!(
            editor.top() > viewport.bottom(),
            "Fixture must put the actual nested editor below the visible task: {view:?} {editor:?} {viewport:?}"
        );
        let task = if view == teaching::View::Assignment {
            a.metrics.script_goal.unwrap().intersect(viewport)
        } else {
            viewport
        };
        assert!(task.is_positive());
        let bar = ctx.style().spacing.scroll.bar_width;
        let mut leaked = Vec::new();
        for clipped in output.unwrap().shapes {
            let mut rects = Vec::new();
            horizontal_fills(&clipped.shape, &mut rects, bar);
            for r in rects {
                let visible = r.intersect(clipped.clip_rect).intersect(task);
                if visible.is_positive() {
                    leaked.push((r, clipped.clip_rect, visible));
                }
            }
        }
        assert!(
            leaked.is_empty(),
            "Offscreen nested Script scrollbar painted over visible task: {view:?} {leaked:?}"
        );
        assert!(!progress::current_solved(
            &a.progress.attempts["shell-trim-manifest"],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn visible_nested_scroll_keeps_horizontal_navigation_and_restores_parent_clip() {
    const SIBLING: &str = "Visible sibling after the nested viewport";
    fn horizontal_fills(shape: &egui::Shape, out: &mut Vec<egui::Rect>, bar: f32) {
        match shape {
            egui::Shape::Rect(r)
                if r.fill.a() > 0
                    && r.rect.height() > 0.
                    && r.rect.height() <= bar + 0.5
                    && r.rect.width() > 8. * r.rect.height() =>
            {
                out.push(r.rect)
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    horizontal_fills(shape, out, bar);
                }
            }
            _ => {}
        }
    }
    fn frame(
        ctx: &egui::Context,
        time: f64,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, egui::Rect, egui::Vec2, egui::Rect) {
        let mut input = egui::RawInput {
            time: Some(time),
            events,
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(946., 929.) / (1.15 * ctx.zoom_factor()),
            )),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let mut observed = None;
        let output = ctx.run(input, |ctx| {
            controls::configure(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Content before the nested viewport");
                let previous = ui.clip_rect();
                assert!(ui.next_widget_position().y > previous.top());
                let scroll = controls::nested_scroll(
                    ui,
                    egui::ScrollArea::both()
                        .id_salt("visible-nested-scroll-proof")
                        .min_scrolled_height(100.)
                        .max_height(100.)
                        .auto_shrink([false, false]),
                    |ui| {
                        ui.set_min_size(egui::vec2(2400., 240.));
                        ui.add(
                            egui::Label::new("Horizontal content begins at the left and continues beyond the visible viewport")
                                .wrap_mode(egui::TextWrapMode::Extend),
                        )
                        .rect
                    },
                );
                assert_eq!(ui.clip_rect(), previous, "Exact parent clip must be restored");
                assert!(scroll.content_size.x > scroll.inner_rect.width() * 2., "content {:?} viewport {:?} activezoom {} sizing {} discard {}", scroll.content_size, scroll.inner_rect, ctx.zoom_factor(), ui.is_sizing_pass(), ctx.will_discard());
                assert!(scroll.content_size.y > scroll.inner_rect.height());
                let sibling = ui.label(SIBLING);
                assert!(previous.contains_rect(sibling.rect));
                observed = Some((scroll.inner_rect, scroll.state.offset, scroll.inner));
            });
        });
        let (rect, offset, content) = observed.unwrap();
        (output, rect, offset, content)
    }
    for dark in [false, true] {
        let (ctx, _a) = app();
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        // Initialize the real viewport before egui scales its previous bounds.
        // A zoom request before the first pass would scale its 10,000-point fallback.
        let _ = frame(&ctx, -0.1, Vec::new());
        ctx.set_zoom_factor(2.);
        let mut current = frame(&ctx, 0., Vec::new());
        for n in 1..12 {
            current = frame(&ctx, n as f64 * 0.1, Vec::new());
        }
        assert!((ctx.pixels_per_point() - 2.3).abs() < 0.01);
        let before_offset = current.2;
        let before_content = current.3;
        assert_eq!(before_offset, egui::Vec2::ZERO);
        let pointer = current.1.center();
        // Warm the real hover separately, as the native provider does.
        let _ = frame(&ctx, 1.2, vec![egui::Event::PointerMoved(pointer)]);
        let _ = frame(
            &ctx,
            1.3,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(-160., 0.),
                modifiers: Default::default(),
            }],
        );
        for n in 14..26 {
            current = frame(&ctx, n as f64 * 0.1, Vec::new());
        }
        assert!(
            current.2.x > before_offset.x + 1.,
            "Real horizontal wheel must navigate visible content: dark={dark}, {:?}",
            current.2
        );
        assert_eq!(current.2.y, before_offset.y);
        assert!(
            (before_content.left() - current.3.left() - (current.2.x - before_offset.x)).abs() < 1.
        );
        let bar = ctx.style().spacing.scroll.bar_width;
        let strip = egui::Rect::from_min_max(
            egui::pos2(current.1.left(), current.1.bottom() - bar - 1.),
            egui::pos2(
                current.1.right(),
                current.1.bottom() + ctx.style().spacing.scroll.allocated_width() + 1.,
            ),
        );
        let mut visible_bars = Vec::new();
        let mut sibling_painted = false;
        for clipped in &current.0.shapes {
            let mut rects = Vec::new();
            horizontal_fills(&clipped.shape, &mut rects, bar);
            visible_bars.extend(rects.into_iter().filter(|r| {
                r.intersect(clipped.clip_rect)
                    .intersect(strip)
                    .is_positive()
            }));
            if let Some(rows) = painted_rows(&clipped.shape, SIBLING) {
                sibling_painted |= rows.iter().all(|r| clipped.clip_rect.contains_rect(*r));
            }
        }
        assert!(
            !visible_bars.is_empty(),
            "Visible overflowing content must retain a painted horizontal scrollbar: dark={dark}"
        );
        assert!(
            sibling_painted,
            "Restored parent clip must allow actual sibling glyphs to paint: dark={dark}"
        );
    }
}
