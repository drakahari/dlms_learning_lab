use super::*;
use dlms_learning_lab::{
    modal_editing::Mode,
    modal_workspace::{Attempt as ModalAttempt, File, Fixture, Requirement},
};
fn fixture() -> Fixture {
    Fixture {
        files: std::collections::BTreeMap::from([(
            "/lab/file.txt".into(),
            File {
                text: "red blue\nred green\n".into(),
                writable: true,
                draft: None,
            },
        )]),
        start: "/lab/file.txt".into(),
        requirements: vec![Requirement::Unchanged {
            path: "/lab/file.txt".into(),
        }],
    }
}
fn render(
    ctx: &egui::Context,
    a: &mut ModalAttempt,
    f: &Fixture,
    width: f32,
    height: f32,
    events: Vec<egui::Event>,
    focus: &mut bool,
) -> (super::super::modal_editor::Output, egui::Id) {
    let mut result = None;
    let mut next = None;
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    result = Some(super::super::modal_editor::show(ui, a, f, focus));
                    next = Some(ui.button("Check saved files").id);
                });
            });
        },
    );
    (result.unwrap(), next.unwrap())
}
fn pressed(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}
#[test]
fn modal_editor_keyboard_typing_escape_search_paste_and_tab_use_the_actual_widget() {
    let ctx = egui::Context::default();
    let f = fixture();
    let mut a = ModalAttempt::new(&f).unwrap();
    let mut focus = true;
    for _ in 0..3 {
        render(&ctx, &mut a, &f, 1000., 700., vec![], &mut focus);
    }
    let (out, _) = render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![
            egui::Event::Text("i".into()),
            egui::Event::Paste("HELLO".into()),
        ],
        &mut focus,
    );
    assert!(out.changed);
    assert!(out.diagnostic.is_none());
    assert_eq!(a.editor().contents(), "HELLOred blue\nred green\n");
    assert_eq!(a.editor().mode(), Mode::Insert);
    let (id_rect, _) = render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![pressed(egui::Key::Escape)],
        &mut focus,
    );
    let id = id_rect.surface.unwrap().2;
    assert_eq!(a.editor().mode(), Mode::Normal);
    assert!(
        ctx.memory(|m| m.has_focus(id)),
        "Escape must retain the editor focus"
    );
    render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![egui::Event::Text("u/red".into()), pressed(egui::Key::Enter)],
        &mut focus,
    );
    assert_eq!(a.editor().cursor().unwrap(), (1, 0));
    assert_eq!(a.editor().contents(), f.files["/lab/file.txt"].text);
    let before = a.clone();
    let (out, _) = render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![egui::Event::Paste(":!host".into())],
        &mut focus,
    );
    assert!(out.diagnostic.unwrap().contains("Insert mode"));
    assert_eq!(a, before);
    let (_, next) = render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![pressed(egui::Key::Tab)],
        &mut focus,
    );
    render(&ctx, &mut a, &f, 1000., 700., vec![], &mut focus);
    assert!(
        ctx.memory(|m| m.has_focus(next)),
        "Tab must reach the next application control"
    );
}
#[test]
fn modal_editor_surface_cursor_and_status_reflow_at_five_scales_two_widths_and_real_themes() {
    let mut modes = std::collections::BTreeSet::new();
    for dark in [false, true] {
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            for width in [760., 1600.] {
                let ctx = egui::Context::default();
                let chosen = theme::inventory().iter().find(|p| p.light != dark).unwrap();
                theme::apply(&ctx, &chosen.id);
                modes.insert(ctx.style().visuals.dark_mode);
                assert_eq!(ctx.style().visuals.dark_mode, dark);
                ctx.set_zoom_factor(scale);
                let f = fixture();
                let mut a = ModalAttempt::new(&f).unwrap();
                let mut focus = true;
                let mut result = None;
                for _ in 0..3 {
                    result = Some(render(&ctx, &mut a, &f, width, 900., vec![], &mut focus).0);
                }
                let out = result.unwrap();
                let (rect, clip, _) = out.surface.unwrap();
                let cursor = out.cursor.unwrap();
                assert!(rect.is_finite() && cursor.is_finite());
                assert!(rect.contains_rect(cursor));
                assert!(clip.contains(cursor.center()));
                assert!(clip.right() <= ctx.content_rect().right() + 1.);
                assert!(clip.width() > 100.);
                assert!(out.diagnostic.is_none());
            }
        }
    }
    assert_eq!(modes.len(), 2);
}
#[test]
fn modal_editor_disabled_or_unfocused_surfaces_do_not_accept_input() {
    let ctx = egui::Context::default();
    let f = fixture();
    let mut a = ModalAttempt::new(&f).unwrap();
    let before = a.clone();
    let mut focus = false;
    render(
        &ctx,
        &mut a,
        &f,
        900.,
        700.,
        vec![egui::Event::Text("ix".into())],
        &mut focus,
    );
    assert_eq!(a, before);
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900., 700.),
            )),
            events: vec![egui::Event::Text("ix".into())],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_enabled_ui(false, |ui| {
                    let mut focus = true;
                    let out = super::super::modal_editor::show(ui, &mut a, &f, &mut focus);
                    assert!(!out.changed);
                });
            });
        },
    );
    assert_eq!(a, before);
}
#[test]
fn modal_focus_uses_cursor_colors_readable_on_every_included_input_background() {
    fn rectangles(shape: &egui::Shape, rows: &mut Vec<(egui::Rect, egui::Stroke)>) {
        match shape {
            egui::Shape::Rect(r) => rows.push((r.rect, r.stroke)),
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    rectangles(s, rows);
                }
            }
            _ => {}
        }
    }
    for t in theme::inventory() {
        let ctx = egui::Context::default();
        theme::apply(&ctx, &t.id);
        let f = fixture();
        let mut a = ModalAttempt::new(&f).unwrap();
        let mut focus = true;
        for _ in 0..3 {
            render(&ctx, &mut a, &f, 900., 600., vec![], &mut focus);
        }
        let mut metrics = None;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900., 600.),
                )),
                ..Default::default()
            },
            |ctx| {
                // Preserve the exact parent hierarchy/IDs used by the settled
                // render helper; changing that tree creates a different widget.
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        metrics =
                            Some(super::super::modal_editor::show(ui, &mut a, &f, &mut focus));
                        let _ = ui.button("Check saved files");
                    });
                });
            },
        );
        let metrics = metrics.unwrap();
        let (viewport, _, id) = metrics.surface.unwrap();
        assert!(ctx.memory(|m| m.has_focus(id)));
        let mut shapes = vec![];
        for clipped in output.shapes {
            rectangles(&clipped.shape, &mut shapes);
        }
        for target in [viewport, metrics.cursor.unwrap()] {
            let stroke = shapes
                .iter()
                .find_map(|(rect, stroke)| {
                    (*rect == target && stroke.width >= 2.).then_some(*stroke)
                })
                .unwrap_or_else(|| {
                    panic!("{} lacks an actual focused viewport/cursor outline", t.id)
                });
            assert!(
                theme::contrast(stroke.color, ctx.style().visuals.extreme_bg_color) >= 3.,
                "{} actual focused outline/input contrast",
                t.id
            );
        }
    }
}

#[test]
fn oversized_native_paste_is_refused_without_losing_the_prior_buffer_or_focus() {
    let ctx = egui::Context::default();
    let f = fixture();
    let mut a = ModalAttempt::new(&f).unwrap();
    let mut focus = true;
    for _ in 0..3 {
        render(&ctx, &mut a, &f, 1000., 700., vec![], &mut focus);
    }
    render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![egui::Event::Text("i".into())],
        &mut focus,
    );
    let before = a.clone();
    let (out, _) = render(
        &ctx,
        &mut a,
        &f,
        1000.,
        700.,
        vec![egui::Event::Paste("x".repeat(100_000))],
        &mut focus,
    );
    assert!(out.diagnostic.unwrap().contains("16 KiB"));
    assert_eq!(a, before);
    assert!(ctx.memory(|m| m.has_focus(out.surface.unwrap().2)));
}
#[test]
fn long_valid_paste_keeps_the_viewport_frame_bounded_and_reveals_the_cursor_without_another_key() {
    let ctx = egui::Context::default();
    let f = fixture();
    let mut a = ModalAttempt::new(&f).unwrap();
    let mut focus = true;
    for _ in 0..3 {
        render(&ctx, &mut a, &f, 760., 700., vec![], &mut focus);
    }
    render(
        &ctx,
        &mut a,
        &f,
        760.,
        700.,
        vec![
            egui::Event::Text("i".into()),
            egui::Event::Paste("x".repeat(1000)),
        ],
        &mut focus,
    );
    assert_eq!(a.editor().cursor().unwrap(), (0, 1000));
    let mut out = None;
    for _ in 0..8 {
        out = Some(render(&ctx, &mut a, &f, 760., 700., vec![], &mut focus).0);
    }
    let out = out.unwrap();
    let (rect, clip, id) = out.surface.unwrap();
    assert!(rect.width() <= ctx.content_rect().width());
    assert!(
        rect.contains(out.cursor.unwrap().center()),
        "viewport={rect:?}, clip={clip:?}, cursor={:?}",
        out.cursor
    );
    assert!(clip.contains(out.cursor.unwrap().center()));
    assert!(ctx.memory(|m| m.has_focus(id)));
    assert_eq!(a.editor().file_bytes(), f.files["/lab/file.txt"].text);
}
