//! Actual native widgets; injected input is technical evidence, not a novice study.
use super::*;
use dlms_learning_lab::{auth_filter_source as binding, auth_filter_workspace as work};
const ID: &str = "lpic2-auth-filter-detection";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 12000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::AuthFilter { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::AuthFilter { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .auth_filter
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            r,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {r:?} {clip:?}"
    );
    let p = r.center();
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    hit(ctx, a, key);
    let id = a
        .metrics
        .auth_filter
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    paint(
        ctx,
        a,
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
            egui::Event::Text(text.into()),
        ],
    );
    settle(ctx, a);
}
fn visible_text(ctx: &egui::Context, a: &mut LabApp) -> Vec<String> {
    fn collect(shape: &egui::epaint::Shape, text: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, text);
                }
            }
            egui::epaint::Shape::Text(t) => text.push(t.galley.job.text.clone()),
            _ => {}
        }
    }
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1450., 12000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for s in out.shapes {
        collect(&s.shape, &mut text);
    }
    text
}

fn perform(ctx: &egui::Context, a: &mut LabApp, action: &work::Action) {
    match action {
        work::Action::Edit { text } => edit(ctx, a, "editor", text),
        work::Action::Save => hit(ctx, a, "save"),
        work::Action::Run => hit(ctx, a, "run"),
        work::Action::Check => hit(ctx, a, "check"),
        work::Action::Command { input } => edit(ctx, a, "command", input),
        work::Action::Select { .. } => panic!("Popup selection is covered by native fixed flow"),
    }
}
#[test]
fn filter_real_widgets_edit_save_run_check_and_invalidate_unsaved_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let v: serde_json::Value =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        perform(
            &ctx,
            &mut a,
            &serde_json::from_value(v["action"].clone()).unwrap(),
        );
    }
    assert!(a.metrics.completion);
    assert!(session(&a).assignment.last_run.is_some());
    assert!(
        session(&a)
            .assignment
            .console
            .contains("Simulated fail2ban-regex")
    );
    let saved = session(&a).assignment.saved.clone();
    edit(
        &ctx,
        &mut a,
        "editor",
        "[Definition]\nfailregex=missing address tag\n",
    );
    assert!(!a.metrics.completion);
    assert_eq!(session(&a).assignment.saved, saved);
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("samples cannot be checked"))
    );
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("Syntax error:"))
    );
    edit(&ctx, &mut a, "editor", &saved);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    edit(&ctx, &mut a, "command", "cat /etc/passwd");
    hit(&ctx, &mut a, "run");
    assert!(
        session(&a)
            .assignment
            .console
            .contains("no host path is read")
    );
    assert!(
        a.metrics.completion,
        "A refused read-only query does not break the correct saved filter"
    );
}
#[test]
fn filter_three_views_separate_practice_drafts_and_do_not_reveal_assignment_answers() {
    let (ctx, mut a, i) = open(teaching::View::Introduction);
    assert!(
        a.metrics
            .auth_filter
            .controls
            .iter()
            .any(|r| r.0 == "worked")
    );
    let headings = visible_text(&ctx, &mut a);
    assert!(
        headings
            .iter()
            .any(|s| s == "1. Read what the application reported")
    );
    assert!(!headings.iter().any(|s| s.starts_with("1. 1.")));
    let original = session(&a).assignment.clone();
    hit(&ctx, &mut a, "example");
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    for action in &d.worked {
        perform(&ctx, &mut a, action);
    }
    assert!(
        session(&a)
            .example
            .checks(&d.example)
            .iter()
            .all(|c| c.passed)
    );
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, original);
    let text = visible_text(&ctx, &mut a);
    assert!(
        !text
            .iter()
            .any(|s| s.contains("Complete separate example filter"))
    );
    assert!(
        !a.metrics
            .auth_filter
            .controls
            .iter()
            .any(|r| r.0 == "fill-worked")
    );
    hit(&ctx, &mut a, "introduction");
    assert!(
        a.metrics
            .auth_filter
            .controls
            .iter()
            .any(|r| r.0 == "worked")
    );
}
#[test]
fn filter_challenge_retains_the_full_assignment_editor_and_independent_tools() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in [
        "introduction",
        "example",
        "try-example",
        "worked",
        "fill-worked",
    ] {
        assert!(!a.metrics.auth_filter.controls.iter().any(|r| r.0 == key));
    }
    for key in [
        "assignment-content",
        "editor",
        "save",
        "sample",
        "command",
        "run",
        "check",
    ] {
        assert!(
            a.metrics.auth_filter.controls.iter().any(|r| r.0 == key),
            "Missing independent tool {key}"
        );
    }
    assert!(!a.metrics.completion);
}
#[test]
fn filter_layout_reflows_without_inventing_control_offsets_or_shrinking_text() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for scale in [1., 1.5, 2.] {
        ctx.set_zoom_factor(scale);
        a.scale = scale;
        for width in [1450., 700.] {
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 12000., vec![]);
            }
            let controls = &a.metrics.auth_filter.controls;
            for key in [
                "editor-fixed",
                "editor-viewport",
                "save",
                "sample",
                "command",
                "run",
                "check",
            ] {
                let (_, rect, clip) = controls.iter().find(|r| r.0 == key).unwrap();
                assert!(rect.is_positive());
                assert!(
                    rect.min.x >= clip.min.x - 1. && rect.max.x <= clip.max.x + 1.,
                    "{scale} {width} {key} {rect:?} {clip:?}"
                );
            }
            let save = controls.iter().find(|r| r.0 == "save").unwrap().1;
            let editor = controls.iter().find(|r| r.0 == "editor-fixed").unwrap().1;
            assert!(
                save.top() >= editor.bottom() - 1.,
                "Save must stay outside editor scrollbar allocation"
            );
            let command = controls.iter().find(|r| r.0 == "command").unwrap().1;
            let run = controls.iter().find(|r| r.0 == "run").unwrap().1;
            if (command.center().y - run.center().y).abs()
                < command.height().min(run.height()) * 0.5
            {
                assert!(
                    (command.height() - run.height()).abs() < 2.,
                    "Related controls use shared font/padding metrics"
                );
            }
        }
    }
}

#[test]
fn wrapped_log_ink_does_not_overlap_adjacent_rows_at_native_narrow_scales() {
    fn collect(
        shape: &egui::epaint::Shape,
        lines: &[String],
        ink: &mut Vec<(Option<usize>, egui::Rect, egui::Rect)>,
        clip: egui::Rect,
    ) {
        match shape {
            egui::epaint::Shape::Text(text) => {
                if let Some(i) = lines.iter().position(|line| *line == text.galley.text()) {
                    ink.push((
                        Some(i),
                        text.galley.rect.translate(text.pos.to_vec2()),
                        clip,
                    ));
                }
            }
            egui::epaint::Shape::Rect(r) if r.fill.a() > 0 => ink.push((None, r.rect, clip)),
            egui::epaint::Shape::Vec(v) => {
                for s in v {
                    collect(s, lines, ink, clip);
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    let id = "lpic2-auth-filter-exceptions";
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let source = a.scenarios[i].clone();
    let d = binding::source(&source).unwrap();
    let State::AuthFilter { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    simulation
        .action(d, &work::Action::Select { case: 1 }, false)
        .unwrap();
    let lines = d.assignment.cases[1].lines.clone();
    for theme_id in ["dlms-navy", "dlms-light"] {
        theme::apply(&ctx, theme_id);
        a.theme_id = theme_id.into();
        for scale in [1., 1.5, 2.] {
            ctx.set_zoom_factor(scale);
            a.scale = scale;
            let width = 773. / scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 12000., vec![]);
            }
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 12000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            let mut ink = vec![];
            for s in out.shapes {
                collect(&s.shape, &lines, &mut ink, s.clip_rect);
            }
            for (at, (row, bounds, _)) in ink.iter().enumerate() {
                if let Some(row) = row {
                    for (_, cover, clip) in ink.iter().skip(at + 1).filter(|r| r.0.is_none()) {
                        let overlap = bounds.shrink(0.5).intersect(*cover).intersect(*clip);
                        assert!(
                            !overlap.is_positive(),
                            "{theme_id}/{scale}: later row background covers text of row{row}: {bounds:?} {cover:?}"
                        );
                    }
                }
            }
            let mut ink = ink
                .into_iter()
                .filter_map(|(i, r, c)| i.map(|i| (i, r, c)))
                .collect::<Vec<_>>();
            ink.sort_by_key(|r| r.0);
            assert_eq!(
                ink.len(),
                lines.len(),
                "Every complete supplied line must be painted"
            );
            for (i, bounds, clip) in &ink {
                assert!(
                    clip.expand(0.5).contains_rect(*bounds),
                    "{scale}: row{i} text clipped: {bounds:?} {clip:?}"
                );
            }
            for rows in ink.windows(2) {
                assert!(
                    rows[0].1.bottom() <= rows[1].1.top(),
                    "{scale}: adjacent log text overlaps: {rows:?}"
                );
            }
        }
    }
}
