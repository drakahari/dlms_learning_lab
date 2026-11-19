use super::*;
const IDS: [&str; 3] = [
    "lpic1-ssh-server-trust",
    "lpic2-ssh-key-authorization",
    "lpic2-ssh-policy-handover",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    (ctx, a, i)
}
fn view(a: &mut LabApp, id: &str, v: teaching::View) {
    let State::SshAccess { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn paint(ctx: &egui::Context, a: &mut LabApp, width: f32) -> String {
    fn collect(s: &egui::Shape, out: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => out.push(t.galley.job.text.clone()),
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    for _ in 0..3 {
        frame(ctx, a, width, 7000., vec![]);
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 7000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for s in output.shapes {
        collect(&s.shape, &mut text);
    }
    text.join("\n")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    paint(ctx, a, 1600.);
    let r = a
        .metrics
        .ssh
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1;
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            1600.,
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
#[test]
fn ssh_introductions_teach_roles_and_controls_before_graded_assignments() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        let intro = paint(&ctx, &mut a, 1600.);
        assert!(intro.contains("Worked example"));
        assert!(intro.contains("public"));
        assert!(intro.contains("Try this example"));
        view(&mut a, id, teaching::View::Example);
        let example = paint(&ctx, &mut a, 1600.);
        assert!(example.contains("Fill worked edits"));
        assert!(example.contains("Save file"));
        assert!(example.contains("Check example"));
        view(&mut a, id, teaching::View::Assignment);
        let assignment = paint(&ctx, &mut a, 1600.);
        assert!(assignment.contains("Check access"));
        assert!(!assignment.contains("Fill worked edits"));
        assert!(!assignment.contains("practice-trust.lab.test"));
        assert!(!assignment.contains("practice-handover.lab.test"));
    }
}
#[test]
fn actual_fill_button_changes_only_practice_drafts_and_does_not_check_or_apply() {
    let id = IDS[0];
    let (ctx, mut a, _) = opened(id);
    view(&mut a, id, teaching::View::Example);
    paint(&ctx, &mut a, 1600.);
    let State::SshAccess { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    let assignment = simulation.assignment.clone();
    let world = simulation.practice.world.clone();
    tap(&ctx, &mut a, "fill");
    let State::SshAccess { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(simulation.assignment, assignment);
    assert_eq!(simulation.practice.world, world);
    assert!(!simulation.practice.checked);
    assert!(!simulation.practice.drafts.matches(&world));
}
#[test]
fn native_editor_keyboard_save_check_and_wrong_edit_use_live_model_grading() {
    let id = IDS[0];
    let (ctx, mut a, i) = opened(id);
    view(&mut a, id, teaching::View::Assignment);
    paint(&ctx, &mut a, 1600.);
    let engine::Goal::SshAccess { definition } = a.scenarios[i].goal.clone() else {
        panic!()
    };
    let text = format!(
        "store.lab.test {}\narchive.lab.test {} preserved\n",
        definition.initial.scope.host_key.line(),
        definition.initial.scope.identities["other-key"].line()
    );
    // Preserve the actual archive record, which has a different public identity.
    let text = text.lines().next().unwrap().to_owned()
        + "\n"
        + definition
            .initial
            .world
            .known_hosts
            .lines()
            .find(|line| line.starts_with("archive."))
            .unwrap()
        + "\n";
    let (_, rect, edit_id) = a
        .metrics
        .ssh
        .fields
        .iter()
        .find(|r| r.0 == "editor")
        .cloned()
        .unwrap();
    for pressed in [true, false] {
        frame(
            &ctx,
            &mut a,
            1600.,
            7000.,
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
    assert_eq!(ctx.memory(|m| m.focused()), Some(edit_id));
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut a,
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
            egui::Event::Text(text.clone()),
        ],
    );
    let State::SshAccess { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.drafts.known_hosts, text);
    assert_ne!(simulation.assignment.world.known_hosts, text);
    tap(&ctx, &mut a, "save");
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
    assert!(paint(&ctx, &mut a, 1600.).contains("Assignment complete"));
    let State::SshAccess { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation
        .apply(
            &definition,
            &dlms_learning_lab::ssh_workspace::Event::Text {
                document: dlms_learning_lab::ssh_workspace::Document::KnownHosts,
                text: "unfinished".into(),
            },
            false,
        )
        .unwrap();
    paint(&ctx, &mut a, 1600.);
    assert!(!progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
}
#[test]
fn ssh_editor_and_permission_controls_reflow_without_shrinking_text() {
    let id = IDS[1];
    let (ctx, mut a, _) = opened(id);
    view(&mut a, id, teaching::View::Assignment);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for light in [false, true] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            a.theme_id = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap()
                .id
                .clone();
            theme::apply(&ctx, &a.theme_id);
            assert_eq!(ctx.style().visuals.dark_mode, !light);
            for width in [760., 1600.] {
                paint(&ctx, &mut a, width);
                let bounds = ctx.content_rect();
                let (_, mode, _) = a.metrics.ssh.fields.iter().find(|r| r.0 == "mode").unwrap();
                assert!(
                    mode.min.x >= bounds.left() && mode.max.x <= bounds.right() + 1.,
                    "{scale} {width} {mode:?}"
                );
                let owner = a
                    .metrics
                    .ssh
                    .controls
                    .iter()
                    .find(|r| r.0 == "owner")
                    .unwrap()
                    .1;
                let save = a
                    .metrics
                    .ssh
                    .controls
                    .iter()
                    .find(|r| r.0 == "save")
                    .unwrap()
                    .1;
                assert!((mode.height() - owner.height()).abs() < 1.);
                assert!((mode.height() - save.height()).abs() < 1.);
            }
        }
    }
}
