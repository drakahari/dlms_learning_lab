//! Actual egui edits/actions; not Fedora/native or beginner acceptance.
use super::*;
use dlms_learning_lab::lvm_workspace as w;
#[path = "../../../tests/support/lvm_device_reference_fixture.rs"]
mod fixture;
fn open() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic2-lvm-free-space")
        .unwrap();
    let d = fixture::definition();
    a.scenarios[i].solution = d
        .device_reference_task
        .as_ref()
        .unwrap()
        .reference
        .iter()
        .map(|op| w::device_reference_session::encode(op).unwrap())
        .collect();
    a.scenarios[i].initial = State::Lvm {
        simulation: Box::new(w::Session::new(&d).unwrap()),
    };
    a.scenarios[i].goal = engine::Goal::Lvm {
        definition: Box::new(d),
    };
    a.launch(i);
    let engine::Goal::Lvm { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation
        .set_view(definition, teaching::View::Assignment)
        .unwrap();
    frame(&ctx, &mut a, 1500., vec![]);
    (ctx, a, i)
}
fn sim(a: &LabApp, i: usize) -> &w::Session {
    let State::Lvm { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn frame(ctx: &egui::Context, a: &mut LabApp, width: f32, events: Vec<egui::Event>) {
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, 9000. / a.scale),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
}
fn click(ctx: &egui::Context, a: &mut LabApp, width: f32, r: egui::Rect) {
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            width,
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
    frame(ctx, a, width, vec![]);
}
fn field(a: &LabApp) -> (egui::Rect, egui::Id) {
    let r = a
        .metrics
        .lvm
        .fields
        .iter()
        .find(|f| f.0 == "reference-reporting")
        .unwrap();
    (r.1, r.2)
}
fn save(a: &LabApp) -> egui::Rect {
    a.metrics
        .lvm
        .controls
        .iter()
        .find(|c| c.0 == "reference-save-reporting")
        .unwrap()
        .1
}
fn paste(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = field(a);
    click(ctx, a, 1500., r.0);
    assert_eq!(ctx.memory(|m| m.focused()), Some(r.1));
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let mut events = vec![];
    for pressed in [true, false] {
        events.push(egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        });
    }
    events.push(egui::Event::Paste(text.into()));
    frame(ctx, a, 1500., events);
    frame(ctx, a, 1500., vec![]);
}
#[test]
fn reference_typing_is_not_a_command_or_ignored_packet_and_save_changes_actual_reader() {
    let (ctx, mut a, i) = open();
    let before = sim(&a, i).clone();
    paste(&ctx, &mut a, "/dev/team/draft");
    let s = sim(&a, i);
    assert_eq!(s.current, before.current);
    assert_eq!(s.command, "");
    assert_eq!(
        s.reference_runtime
            .as_ref()
            .unwrap()
            .assignment
            .configuration,
        before
            .reference_runtime
            .as_ref()
            .unwrap()
            .assignment
            .configuration
    );
    assert_eq!(
        s.reference_runtime.as_ref().unwrap().assignment.drafts["reporting"].0,
        "/dev/team/draft"
    );
    assert!(a.notice.is_none(), "{:?}", a.notice);
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    let r = save(&a);
    click(&ctx, &mut a, 1500., r);
    assert!(engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    paste(&ctx, &mut a, "/dev/team/missing");
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    assert!(
        a.metrics
            .lvm
            .inspected_files
            .iter()
            .any(|(_, s)| s.contains("readable"))
    );
    let r = save(&a);
    click(&ctx, &mut a, 1500., r);
    assert!(
        a.metrics
            .lvm
            .inspected_files
            .iter()
            .any(|(_, s)| s.contains("unavailable"))
    );
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
}
#[test]
fn oversized_and_multiline_native_reference_packets_preserve_draft_config_and_selection() {
    for text in ["x".repeat(257), "/dev/team/draft\n".into()] {
        let (ctx, mut a, i) = open();
        let before = sim(&a, i).clone();
        paste(&ctx, &mut a, &text);
        assert_eq!(sim(&a, i).reference_runtime, before.reference_runtime);
        assert_eq!(sim(&a, i).current, before.current);
        assert!(a.notice.as_ref().is_some_and(|s| s.contains("256 bytes")));
        assert_eq!(ctx.memory(|m| m.focused()), Some(field(&a).1));
    }
}
#[test]
fn reference_fields_and_saves_reflow_with_shared_height_in_light_dark_and_enlarged_text() {
    let (ctx, mut a, _) = open();
    for name in ["dlms-dark", "dlms-light"] {
        a.theme_id = name.into();
        theme::apply(&ctx, name);
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [1500., 850.] {
                frame(&ctx, &mut a, width, vec![]);
                frame(&ctx, &mut a, width, vec![]);
                let f = field(&a).0;
                let b = save(&a);
                let clip = a
                    .metrics
                    .lvm
                    .fields
                    .iter()
                    .find(|f| f.0 == "reference-reporting")
                    .unwrap()
                    .3;
                assert!(f.width() > 100.);
                assert!(
                    b.left() >= clip.left() - 1. && b.right() <= clip.right() + 1.,
                    "Save outside viewport {width}/{scale}: {b:?} {clip:?}"
                );
                assert!(
                    (f.height() - b.height()).abs() < 1.,
                    "shared heights {f:?} {b:?}"
                );
                if b.top() < f.bottom() {
                    assert!((b.center().y - f.center().y).abs() < 1.);
                } else {
                    assert!(b.top() >= f.bottom());
                }
            }
        }
    }
}
#[test]
fn keyboard_clear_tab_and_save_retains_actual_reader_until_explicit_save() {
    let (ctx, mut a, i) = open();
    let (rect, id) = field(&a);
    click(&ctx, &mut a, 1500., rect);
    let control = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let mut events = Vec::new();
    for (key, modifiers) in [
        (egui::Key::A, control),
        (egui::Key::Backspace, egui::Modifiers::NONE),
    ] {
        for pressed in [true, false] {
            events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            });
        }
    }
    frame(&ctx, &mut a, 1500., events);
    frame(&ctx, &mut a, 1500., vec![]);
    let live = &sim(&a, i).reference_runtime.as_ref().unwrap().assignment;
    assert_eq!(live.drafts["reporting"].0, "");
    assert!(!live.configuration.callers["reporting"].path.is_empty());
    assert!(a.notice.is_none());
    frame(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(ctx.memory(|m| m.focused()).is_some());
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    for pressed in [true, false] {
        frame(
            &ctx,
            &mut a,
            1500.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
    frame(&ctx, &mut a, 1500., vec![]);
    let live = &sim(&a, i).reference_runtime.as_ref().unwrap().assignment;
    assert_eq!(live.configuration.callers["reporting"].path, "");
    assert!(
        a.metrics
            .lvm
            .inspected_files
            .iter()
            .any(|(_, v)| v.contains("blank"))
    );
    assert_eq!(sim(&a, i).command, "");
}
#[test]
fn reference_transcripts_identify_simulator_saves_without_internal_json_or_shell_prompts() {
    fn text(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    text(s, out)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a, i) = open();
    let engine::Goal::Lvm { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation
        .apply_reference(
            definition,
            &fixture::save("reporting", "/dev/team/draft"),
            false,
        )
        .unwrap();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 9000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut rendered = String::new();
    for s in output.shapes {
        text(&s.shape, &mut rendered);
    }
    assert!(
        rendered.contains("Simulator configuration save · Reporting application"),
        "Missing honest native action description"
    );
    assert!(!rendered.contains("simulated-lvm-reference"));
    assert!(!rendered.contains("save_reference"));
    assert!(!rendered.contains("virtual $ Simulator configuration"));
    let r = a
        .metrics
        .lvm
        .controls
        .iter()
        .find(|c| c.0 == "optional-help")
        .unwrap()
        .1;
    click(&ctx, &mut a, 1500., r);
    let r = a
        .metrics
        .lvm
        .controls
        .iter()
        .find(|c| c.0 == "worked-approach")
        .unwrap()
        .1;
    click(&ctx, &mut a, 1500., r);
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 9000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut revealed = String::new();
    for shape in output.shapes {
        text(&shape.shape, &mut revealed);
    }
    assert!(
        !a.metrics.lvm.worked_commands.is_empty(),
        "Explicitly requested worked content must actually render"
    );
    assert!(!revealed.contains("simulated-lvm-reference"));
    assert!(!revealed.contains("save_reference"));
}
