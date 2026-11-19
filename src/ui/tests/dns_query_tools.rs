use super::*;
const IDS: [&str; 3] = [
    "lpic2-dns-dualstack",
    "lpic2-dns-aliases",
    "lpic2-dns-secondary",
];
fn paint(ctx: &egui::Context, app: &mut LabApp, width: f32) -> String {
    fn collect(s: &egui::Shape, texts: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => texts.push(t.galley.job.text.clone()),
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, texts);
                }
            }
            _ => {}
        }
    }
    for _ in 0..3 {
        frame(ctx, app, width, 7000., vec![]);
    }
    let result = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 7000.),
            )),
            ..Default::default()
        },
        |ctx| app.render(ctx),
    );
    let mut texts = vec![];
    for s in result.shapes {
        collect(&s.shape, &mut texts);
    }
    texts.join("\n")
}
fn tap(ctx: &egui::Context, app: &mut LabApp, r: egui::Rect, width: f32) {
    for pressed in [true, false] {
        frame(
            ctx,
            app,
            width,
            7000.,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
}
fn commands(ctx: &egui::Context, app: &mut LabApp, width: f32) {
    paint(ctx, app, width);
    let r = app
        .metrics
        .dns_choices
        .iter()
        .find(|r| r.0 == "lookup:commands")
        .unwrap()
        .1;
    tap(ctx, app, r, width);
    paint(ctx, app, width);
}
fn type_query(ctx: &egui::Context, app: &mut LabApp, input: &str) {
    paint(ctx, app, 1600.);
    let (_, rect, id) = app
        .metrics
        .dns_fields
        .iter()
        .find(|r| r.0 == "lookup-command")
        .cloned()
        .unwrap();
    tap(ctx, app, rect, 1600.);
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        ctx,
        app,
        1600.,
        7000.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers,
            },
            egui::Event::Text(input.into()),
        ],
    );
}
#[test]
fn authoritative_query_lessons_render_teaching_and_keep_assignment_answers_closed() {
    for id in IDS {
        let (ctx, mut app, i) = super::nm_lessons::opened(id);
        let intro = paint(&ctx, &mut app, 1600.);
        assert!(
            intro.contains("+norecurse") && intro.contains("host -r"),
            "{id}"
        );
        app.set_dns_view(i, teaching::View::Example);
        let example = paint(&ctx, &mut app, 1600.);
        assert!(example.contains("Use lookup commands"));
        assert!(example.contains("Fill worked example"));
        app.set_dns_view(i, teaching::View::Assignment);
        let assignment = paint(&ctx, &mut app, 1600.);
        assert!(assignment.contains("Check configuration"));
        assert!(!assignment.contains("Fill worked example"));
        let w = app.progress.attempts[id].workspace.as_ref().unwrap();
        assert!(w.dns_draft.as_ref().unwrap().utility.is_none());
        assert!(app.metrics.dns_fields.iter().any(|r| r.0 == "query"));
        assert!(
            !app.metrics
                .dns_fields
                .iter()
                .any(|r| r.0 == "lookup-command")
        );
        let old = serde_json::to_vec(&app.progress.attempts[id]).unwrap();
        paint(&ctx, &mut app, 1600.);
        assert_eq!(serde_json::to_vec(&app.progress.attempts[id]).unwrap(), old);
    }
    let (ctx, mut app, i) = super::nm_lessons::opened("lpic2-dns-mail");
    app.set_dns_view(i, teaching::View::Assignment);
    paint(&ctx, &mut app, 1600.);
    assert!(
        !app.metrics
            .dns_choices
            .iter()
            .any(|r| r.0 == "lookup:commands")
    );
}
#[test]
fn authoritative_actual_command_input_enter_stale_output_and_separate_drafts() {
    let id = IDS[0];
    let (ctx, mut app, i) = super::nm_lessons::opened(id);
    app.set_dns_view(i, teaching::View::Assignment);
    commands(&ctx, &mut app, 1600.);
    for (input, want) in [
        (
            "host -r -t AAAA web.workshop.test. 192.0.2.53",
            "has no AAAA record",
        ),
        (
            "dig @192.0.2.53 missing.workshop.test. A +norecurse +short",
            "NXDOMAIN",
        ),
        (
            "host -r -t TXT web.workshop.test. 192.0.2.53",
            "Lookup not evaluated:",
        ),
    ] {
        type_query(&ctx, &mut app, input);
        let w = app.progress.attempts[id].workspace.as_ref().unwrap();
        let draft = w.dns_draft.as_ref().unwrap();
        assert_eq!(draft.utility.as_ref().unwrap().input, input);
        assert!(draft.output.is_empty());
        for pressed in [true, false] {
            frame(
                &ctx,
                &mut app,
                1600.,
                7000.,
                vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                }],
            );
        }
        let a = &app.progress.attempts[id];
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .dns_draft
                .as_ref()
                .unwrap()
                .output
                .contains(want)
        );
        assert!(!progress::current_solved(a, &app.scenarios[i]));
    }
    let assignment = app.progress.attempts[id]
        .workspace
        .as_ref()
        .unwrap()
        .dns_draft
        .clone();
    app.set_dns_view(i, teaching::View::Example);
    commands(&ctx, &mut app, 1600.);
    type_query(
        &ctx,
        &mut app,
        "host -r -t A web.dual-practice.test. 192.0.2.53",
    );
    assert_eq!(
        app.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .dns_draft,
        assignment
    );
    let fill = app.metrics.insert.unwrap();
    tap(&ctx, &mut app, fill, 1600.);
    let l = app.progress.attempts[id]
        .workspace
        .as_ref()
        .unwrap()
        .dns_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        l.draft.utility.as_ref().unwrap().input,
        "host -r -t A web.dual-practice.test. 192.0.2.53"
    );
    assert!(!l.checked);
}
#[test]
fn authoritative_command_controls_reflow_with_shared_centered_fields() {
    let (ctx, mut app, i) = super::nm_lessons::opened(IDS[2]);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            app.scale = scale;
            ctx.set_zoom_factor(scale);
            app.theme_id = if light { "dlms-light" } else { "dlms-dark" }.into();
            theme::apply(&ctx, &app.theme_id);
            for width in [760., 1600.] {
                for view in [teaching::View::Example, teaching::View::Assignment] {
                    app.set_dns_view(i, view);
                    paint(&ctx, &mut app, width);
                    if !app
                        .metrics
                        .dns_fields
                        .iter()
                        .any(|r| r.0 == "lookup-command")
                    {
                        commands(&ctx, &mut app, width);
                    }
                    let (_, r, _) = app
                        .metrics
                        .dns_fields
                        .iter()
                        .find(|r| r.0 == "lookup-command")
                        .unwrap();
                    assert!(
                        r.min.x >= 0. && r.max.x <= width + 1. && r.width() > 50.,
                        "{scale} {width} {r:?}"
                    );
                    let b = app.metrics.probe.unwrap();
                    assert!((r.height() - b.height()).abs() < 1., "{r:?} {b:?}");
                }
            }
        }
    }
}

#[test]
fn authoritative_error_output_reserves_its_lines_in_a_short_desktop_viewport() {
    use dlms_learning_lab::dns_workspace::UtilityDraft;
    let (ctx, mut app, i) = super::nm_lessons::opened(IDS[0]);
    app.set_dns_view(i, teaching::View::Assignment);
    let s = app.scenarios[i].clone();
    let plan = &s
        .dns_teaching
        .as_ref()
        .unwrap()
        .lookup
        .as_ref()
        .unwrap()
        .assignment;
    let a = app.progress.attempts.get_mut(IDS[0]).unwrap();
    let State::AuthoritativeDns { server } = &a.state else {
        panic!()
    };
    let draft = a.workspace.as_mut().unwrap().dns_draft.as_mut().unwrap();
    draft.utility = Some(UtilityDraft {
        enabled: true,
        input: "host -r -t A absent.workshop.test. 192.0.2.53".into(),
    });
    draft.lookup_utility(server, plan).unwrap();
    assert!(draft.output.contains("NXDOMAIN"));
    let lines = draft.output.lines().count().clamp(3, 10);
    for scale in [1., 2.] {
        app.scale = scale;
        ctx.set_zoom_factor(scale);
        for width in [760., 1600.] {
            for _ in 0..4 {
                frame(&ctx, &mut app, width, 650., vec![]);
            }
            let font = ctx.style().text_styles[&egui::TextStyle::Monospace].clone();
            let line_height = ctx.fonts_mut(|f| f.row_height(&font));
            assert!(
                app.metrics.output.unwrap().height() >= line_height * lines as f32,
                "The outer page must scroll rather than compressing the result below its DNS error: scale{scale} width{width}, {:?}",
                app.metrics.output
            );
        }
    }
}
