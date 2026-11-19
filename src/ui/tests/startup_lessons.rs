use super::*;
use dlms_learning_lab::{unit_manager_terminal, unit_manager_workspace as manager};
const IDS: [&str; 3] = [
    "lpic1-startup-current-future",
    "lpic1-startup-default-target",
    "lpic1-startup-install-links",
];
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    a.set_unit_manager_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn scope(
    a: &LabApp,
    i: usize,
    practice: bool,
) -> (&dlms_learning_lab::unit_manager::State, &manager::Draft) {
    let attempt = &a.progress.attempts[&a.scenarios[i].id];
    let w = attempt.workspace.as_ref().unwrap();
    if practice {
        let l = w.unit_manager_learning.as_ref().unwrap();
        (&l.state, &l.draft)
    } else {
        let State::UnitManager { simulation } = &attempt.state else {
            panic!()
        };
        (simulation, w.unit_manager_draft.as_ref().unwrap())
    }
}
#[test]
fn startup_intro_teaches_all_three_concepts_and_challenge_hides_worked_help() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Introduction);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(a.metrics.example_action.is_some());
        let s = a.scenarios[i].clone();
        assert!(!a.metrics.completion);
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert_eq!(a.unit_manager_view(i), teaching::View::Assignment);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.help_visible);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn startup_actual_terminal_typing_keeps_example_and_assignment_separate() {
    let (ctx, mut a, i) = opened(IDS[0], teaching::View::Example);
    let before = a.progress.attempts[IDS[0]].state.clone();
    let command_id = egui::Id::new(("manager-command", (IDS[0], true)));
    for cmd in [
        "systemctl stop sketchplan-app.service",
        "systemctl enable sketchplan-app.service",
        "systemctl start sketchplan-inspection.service",
        "systemctl disable sketchplan-inspection.service",
    ] {
        ctx.memory_mut(|m| m.request_focus(command_id));
        frame(
            &ctx,
            &mut a,
            1200.,
            950.,
            vec![acl_ctrl(egui::Key::A), egui::Event::Paste(cmd.into())],
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            950.,
            vec![key(egui::Key::Enter, false)],
        );
    }
    let s = &a.scenarios[i];
    let (state, draft) = scope(&a, i, true);
    assert!(manager::solved(
        &s.unit_manager_teaching.as_ref().unwrap().definition,
        state,
        draft
    ));
    assert_eq!(a.progress.attempts[IDS[0]].state, before);
    assert!(!progress::current_solved(&a.progress.attempts[IDS[0]], s));
}
#[test]
fn startup_assignment_pass_queries_and_pending_editor_changes_regrade_current_state() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Assignment);
        let s = a.scenarios[i].clone();
        for cmd in &s.solution {
            engine::execute_for_goal(
                &mut a.progress.attempts.get_mut(id).unwrap().state,
                &s.goal,
                cmd,
            )
            .unwrap();
        }
        let State::UnitManager { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let d = manager::Draft::new(simulation).unwrap();
        let w = a
            .progress
            .attempts
            .get_mut(id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        w.unit_manager_draft = Some(d);
        w.checked = true;
        draw(&ctx, &mut a);
        assert!(a.metrics.completion);
        let engine::Goal::UnitManager { definition } = &s.goal else {
            panic!()
        };
        let State::UnitManager { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
        else {
            panic!()
        };
        unit_manager_terminal::execute(definition, simulation, "systemctl get-default").unwrap();
        draw(&ctx, &mut a);
        assert!(a.metrics.completion);
        let w = a
            .progress
            .attempts
            .get_mut(id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        let d = w.unit_manager_draft.as_mut().unwrap();
        d.documents
            .get_mut(&d.selected_unit)
            .unwrap()
            .push_str("# unsaved change\n");
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn startup_approved_editor_terminal_workspace_renders_at_supported_scales_and_themes() {
    for id in IDS {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            let (ctx, mut a, _) = opened(id, view);
            for scale in [1., 1.15, 1.25, 1.5, 2.] {
                for light in [false, true] {
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
                    a.scale = scale;
                    ctx.set_zoom_factor(scale);
                    for width in [760., 1600.] {
                        for _ in 0..3 {
                            frame(&ctx, &mut a, width, 950., vec![]);
                        }
                        assert!(!a.metrics.completion);
                        let clip = a.metrics.main_viewport.unwrap();
                        assert!(clip.is_finite());
                        if view != teaching::View::Introduction {
                            let r = a.metrics.editor_viewport.expect("existing native editor");
                            assert!(
                                r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                                "{id} scale{scale} width{width} {r:?} {clip:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn startup_failed_requirement_is_labeled_in_text_instead_of_color_alone() {
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
    let (ctx, mut a, i) = opened(IDS[2], teaching::View::Assignment);
    a.progress
        .attempts
        .get_mut(IDS[2])
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .checked = true;
    let mut painted = String::new();
    for _ in 0..3 {
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 1200.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        for s in out.shapes {
            text(&s.shape, &mut painted);
        }
    }
    assert!(
        painted.contains("Needs work · press-app.service: enabled for future press.target"),
        "{painted}"
    );
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &a.scenarios[i]
    ));
}
