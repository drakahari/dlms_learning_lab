//! Actual egui interactions for the opt-in synthetic mount profile; not native
//! screen or independent novice acceptance.
use super::*;
use dlms_learning_lab::lvm_workspace as w;
#[path = "../../../tests/support/lvm_mount_session_fixture.rs"]
mod fixture;
fn frame(ctx: &egui::Context, a: &mut LabApp, width: f32, events: Vec<egui::Event>) -> String {
    let output = ctx.run(
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
    let mut s = String::new();
    for shape in output.shapes {
        text(&shape.shape, &mut s);
    }
    s
}
fn open() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic2-lvm-free-space")
        .unwrap();
    a.scenarios[i] = fixture::scenario();
    a.launch(i);
    frame(&ctx, &mut a, 1500., vec![]);
    (ctx, a, i)
}
fn sim(a: &LabApp, i: usize) -> &w::Session {
    let State::Lvm { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn field(a: &LabApp, key: &str) -> (egui::Rect, egui::Id, egui::Rect) {
    let r = a
        .metrics
        .lvm
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"));
    (r.1, r.2, r.3)
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = a
        .metrics
        .lvm
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .1;
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            1500.,
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
    frame(ctx, a, 1500., vec![]);
}
fn view(ctx: &egui::Context, a: &mut LabApp, i: usize, v: teaching::View) {
    let source = a.scenarios[i].clone();
    let engine::Goal::Lvm { definition } = &source.goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a.progress.attempts.get_mut(&source.id).unwrap().state
    else {
        panic!()
    };
    simulation.set_view(definition, v).unwrap();
    frame(ctx, a, 1500., vec![]);
}
fn keys(k: egui::Key, m: egui::Modifiers) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: k,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: m,
        })
        .collect()
}
#[test]
fn introduction_worked_config_and_independent_assignment_have_distinct_visibility() {
    let (ctx, mut a, i) = open();
    let t = frame(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("/dev/demo/draft"));
    assert!(!t.contains("Simulated /etc/fstab"));
    assert!(!t.contains("/dev/team/draft"));
    view(&ctx, &mut a, i, teaching::View::Assignment);
    let t = frame(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Simulated /etc/fstab"));
    assert!(!t.contains("/dev/team/draft"));
    assert!(!t.contains("Apply this approach"));
    assert!(t.contains("/dev/team/reference"));
    assert!(!t.contains("Current allocations and remaining space"));
}
#[test]
fn guided_fill_changes_only_example_draft_not_saved_work_or_assignment() {
    let (ctx, mut a, i) = open();
    view(&ctx, &mut a, i, teaching::View::Example);
    let before = sim(&a, i).clone();
    click(&ctx, &mut a, "insert");
    let after = sim(&a, i);
    assert_ne!(
        after.mount_runtime.as_ref().unwrap().example.draft,
        before.mount_runtime.as_ref().unwrap().example.draft
    );
    assert_eq!(
        after.mount_runtime.as_ref().unwrap().example.mounted,
        before.mount_runtime.as_ref().unwrap().example.mounted
    );
    assert_eq!(
        after.mount_runtime.as_ref().unwrap().assignment,
        before.mount_runtime.as_ref().unwrap().assignment
    );
    assert_eq!(after.current, before.current);
    assert!(after.example_transcript.is_empty());
    click(&ctx, &mut a, "mount-save");
    assert_ne!(
        sim(&a, i).mount_runtime.as_ref().unwrap().example.mounted,
        before.mount_runtime.as_ref().unwrap().example.mounted
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
#[test]
fn native_multiline_edit_save_partial_repair_and_terminal_actions_work() {
    let (ctx, mut a, i) = open();
    view(&ctx, &mut a, i, teaching::View::Assignment);
    let id = field(&a, "mount-editor").1;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(&ctx, &mut a, 1500., vec![]);
    let mut events = keys(
        egui::Key::A,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    );
    events.push(egui::Event::Paste(
        "# intended filesystem\n/dev/team/draft /srv/report ext4 defaults 0 2\n".into(),
    ));
    frame(&ctx, &mut a, 1500., events);
    assert!(a.notice.is_none(), "{:?}", a.notice);
    assert!(
        sim(&a, i)
            .mount_runtime
            .as_ref()
            .unwrap()
            .assignment
            .draft
            .contains("team/draft")
    );
    assert!(
        sim(&a, i)
            .mount_runtime
            .as_ref()
            .unwrap()
            .assignment
            .mounted
            .storage
            .configuration()
            .contains("team/reference")
    );
    frame(&ctx, &mut a, 1500., vec![]);
    assert_eq!(
        sim(&a, i).mount_runtime.as_ref().unwrap().assignment.draft,
        "# intended filesystem\n/dev/team/draft /srv/report ext4 defaults 0 2\n"
    );
    click(&ctx, &mut a, "mount-save");
    assert_eq!(
        sim(&a, i)
            .mount_runtime
            .as_ref()
            .unwrap()
            .assignment
            .mounted
            .storage
            .configuration(),
        sim(&a, i).mount_runtime.as_ref().unwrap().assignment.draft
    );
    click(&ctx, &mut a, "check");
    let t = frame(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("uses another filesystem"));
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
    for text in [
        "umount /srv/report",
        "mount /srv/report",
        "cat /srv/report/keep.txt",
    ] {
        let id = field(&a, "command").1;
        ctx.memory_mut(|m| m.request_focus(id));
        frame(&ctx, &mut a, 1500., vec![]);
        let mut e = keys(
            egui::Key::A,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        );
        e.push(egui::Event::Paste(text.into()));
        frame(&ctx, &mut a, 1500., e);
        assert_eq!(
            sim(&a, i).command,
            text,
            "command edit notice={:?}",
            a.notice
        );
        click(&ctx, &mut a, "run");
        assert!(
            a.notice.is_none(),
            "{text}: {:?}; transcript={:?}",
            a.notice,
            sim(&a, i).transcript
        );
    }
    click(&ctx, &mut a, "check");
    assert!(
        progress::current_solved(&a.progress.attempts[&a.scenarios[i].id], &a.scenarios[i]),
        "checks={:?}; transcript={:?}; notice={:?}",
        fixture::definition().checks(sim(&a, i)),
        sim(&a, i).transcript,
        a.notice
    );
    assert!(
        sim(&a, i)
            .transcript
            .last()
            .unwrap()
            .output
            .starts_with("team/draft:")
    );
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Text("# unsaved change\n".into())],
    );
    assert!(!sim(&a, i).checked);
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
#[test]
fn rejected_multiline_packet_is_atomic_and_cannot_save_or_run() {
    let (ctx, mut a, i) = open();
    view(&ctx, &mut a, i, teaching::View::Assignment);
    let id = field(&a, "mount-editor").1;
    ctx.memory_mut(|m| m.request_focus(id));
    let before = sim(&a, i).clone();
    let t = frame(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Paste("\0mount /srv/report".into())],
    );
    assert!(t.contains("previous draft"));
    assert_eq!(sim(&a, i), &before);
    let id = field(&a, "command").1;
    ctx.memory_mut(|m| m.request_focus(id));
    let before = sim(&a, i).clone();
    frame(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Paste(
            "umount /srv/report\nmount /srv/report".into(),
        )],
    );
    assert_eq!(sim(&a, i), &before);
}
#[test]
fn enlarged_light_and_dark_editors_save_and_command_fields_keep_geometry_and_focus() {
    for theme in ["dlms-dark", "dlms-light"] {
        for scale in [1., 1.5, 2.] {
            let (ctx, mut a, i) = open();
            a.scale = scale;
            a.theme_id = theme.into();
            theme::apply(&ctx, theme);
            view(&ctx, &mut a, i, teaching::View::Assignment);
            frame(&ctx, &mut a, 900., vec![]);
            for key in ["mount-editor", "command"] {
                let (r, _, clip) = field(&a, key);
                assert!(r.is_finite() && r.width() > 0.);
                assert!(
                    r.intersect(clip).width() > r.width() * 0.95,
                    "{theme} {scale} {key}: {r:?} {clip:?}"
                );
            }
            let id = field(&a, "mount-editor").1;
            ctx.memory_mut(|m| m.request_focus(id));
            frame(
                &ctx,
                &mut a,
                900.,
                keys(egui::Key::Tab, egui::Modifiers::NONE),
            );
            assert_ne!(ctx.memory(|m| m.focused()), Some(id));
        }
    }
}
#[test]
fn challenge_hides_example_work_and_cannot_show_assignment_reference_by_default() {
    let (ctx, mut a, i) = open();
    let source = a.scenarios[i].clone();
    a.progress.attempts.get_mut(&source.id).unwrap().challenge = true;
    let t = frame(&ctx, &mut a, 1500., vec![]);
    assert!(!t.contains("/dev/demo/draft"));
    assert!(!t.contains("/dev/team/draft"));
    assert!(!t.contains("Fill selected example"));
    assert!(t.contains("Simulated /etc/fstab"));
    let engine::Goal::Lvm { definition } = &source.goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a.progress.attempts.get_mut(&source.id).unwrap().state
    else {
        panic!()
    };
    for op in &definition.mount_task.as_ref().unwrap().reference {
        simulation.mount_action(definition, op, false).unwrap();
    }
    assert!(progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
}

#[test]
fn syntax_feedback_has_one_primary_diagnostic_and_keeps_complete_grading() {
    let (ctx, mut a, i) = open();
    view(&ctx, &mut a, i, teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    let engine::Goal::Lvm { definition } = &source.goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut a.progress.attempts.get_mut(&source.id).unwrap().state
    else {
        panic!()
    };
    simulation
        .mount_action(
            definition,
            &w::mount_flow::Operation::SavePolicy {
                text: "# incomplete\n/dev/\n".into(),
            },
            false,
        )
        .unwrap();
    simulation.checked = true;
    let rows = definition.checks(simulation);
    assert!(rows.iter().filter(|r| !r.0).count() > 3);
    let t = frame(&ctx, &mut a, 1500., vec![]);
    assert_eq!(t.matches("Use six fields:").count(), 1, "{t}");
    assert!(t.contains("All objective results"));
    assert!(!t.contains("Passed · Save"));
    assert!(!progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
}
