use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::management_workspace as work;
const IDS: [&str; 2] = ["ccna-management-protocols", "ccna-management-source-policy"];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    super::nm_lessons::opened(id)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        frame(ctx, a, 1200., 950., vec![]);
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let (r, clip) = if key == "editor" {
            let (r, _, c) = a.metrics.management.editor.unwrap();
            (r, c)
        } else if key == "challenge" {
            (a.metrics.mode_challenge.unwrap(), main)
        } else {
            let (_, r, c) = a
                .metrics
                .management
                .controls
                .iter()
                .find(|v| v.0 == key)
                .unwrap();
            (*r, *c)
        };
        let clip = main.intersect(clip);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(main.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (main.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Management target not visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "editor");
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    let id = a.metrics.management.editor.unwrap().1;
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
    let s = &a.scenarios[a.selected.unwrap()];
    let State::ManagementPolicy { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    let scope = if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    };
    assert_eq!(
        scope.draft, text,
        "Actual Ctrl+A/Paste must replace the document"
    );
}
fn state(a: &LabApp) -> &work::Session {
    let id = &a.scenarios[a.selected.unwrap()].id;
    let State::ManagementPolicy { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
#[test]
fn management_editor_checks_specific_partial_cases_syntax_unsupported_and_stale_retry() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = work::source(&a.scenarios[i]).unwrap().clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        let (p, t, _) = a.metrics.management.result.unwrap();
        assert!(p < t);
        assert!(!a.metrics.completion);
        edit(&ctx, &mut a, "ip ssh version");
        tap(&ctx, &mut a, "apply");
        assert!(a.notice.as_deref().unwrap().contains("Syntax error"));
        assert!(a.metrics.management.result.is_none());
        edit(&ctx, &mut a, "hostname Host");
        tap(&ctx, &mut a, "check");
        assert!(a.notice.as_deref().unwrap().contains("Unsupported"));
        edit(&ctx, &mut a, &d.initial.reference);
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        assert!(a.metrics.completion);
        edit(&ctx, &mut a, &d.initial.starter);
        assert!(!a.metrics.completion);
        assert!(a.metrics.management.result.is_none());
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
    }
}
#[test]
fn management_trial_requires_apply_and_selector_changes_clear_actual_output() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let d = work::source(&a.scenarios[i]).unwrap().clone();
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, &d.initial.reference);
    tap(&ctx, &mut a, "inspect");
    assert!(
        state(&a)
            .assignment
            .observation
            .as_deref()
            .unwrap()
            .starts_with("Trial not run: Apply")
    );
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "inspect");
    assert!(
        state(&a)
            .assignment
            .observation
            .as_deref()
            .unwrap()
            .contains("IPv4 forward path")
    );
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "case");
    tap(&ctx, &mut a, "case-2");
    assert!(state(&a).assignment.observation.is_none());
    tap(&ctx, &mut a, "inspect");
    assert!(
        state(&a)
            .assignment
            .observation
            .as_deref()
            .unwrap()
            .contains("protocol is disabled")
    );
}
#[test]
fn management_practice_is_separate_and_challenge_does_not_render_worked_answers() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let original = state(&a).assignment.clone();
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "worked-example");
        tap(&ctx, &mut a, "fill-example");
        assert!(!state(&a).practice.checked);
        tap(&ctx, &mut a, "check");
        assert_eq!(state(&a).assignment, original);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let practice = state(&a).practice.clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "challenge");
        assert_eq!(state(&a).practice, practice);
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Expected source comparison"));
        assert!(!text.contains("Sample exact permits"));
        assert!(!text.contains("Fill worked example"));
        assert!(!text.contains("172.18.40."));
    }
}
#[test]
fn management_controls_reflow_and_editor_focus_stays_stable_at_all_supported_scales() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    target(&ctx, &mut a, "editor");
    let id = a.metrics.management.editor.unwrap().1;
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        for width in [760., 1600.] {
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
                for disabled in [false, true] {
                    a.reset_pending = disabled;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 1100., vec![]);
                    }
                    let main = a.metrics.main_viewport.unwrap();
                    let (editor, current, _) = a.metrics.management.editor.unwrap();
                    assert_eq!(current, id);
                    if scale == 1. && width == 1600. {
                        assert!(
                            a.metrics.management.side_by_side,
                            "Wide workspaces must use the editor/trial split"
                        );
                    }
                    if width == 760. {
                        assert!(
                            !a.metrics.management.side_by_side,
                            "Narrow workspaces stack without shrinking text"
                        );
                    }
                    assert!(editor.width() > 100.);
                    assert!(editor.right() <= main.right() + 1.);
                    let frame_rect = a.metrics.management.editor_frame.unwrap();
                    assert!(
                        frame_rect.left() >= editor.left() - 1.
                            && frame_rect.right() <= editor.right() + 1.,
                        "Native editor frame must fit its allocated width: {frame_rect:?} {editor:?}"
                    );
                    for key in ["apply", "check", "case", "line", "inspect"] {
                        let (_, r, _) = a
                            .metrics
                            .management
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap();
                        assert!(
                            r.left() >= main.left() - 1. && r.right() <= main.right() + 1.,
                            "{key} {scale}/{width}/{light}: {r:?} {main:?}"
                        );
                    }
                    let out = a.metrics.management.output.unwrap();
                    assert!(out.right() <= main.right() + 1.);
                }
            }
        }
    }
    a.reset_pending = false;
    a.scale = 1.;
    ctx.set_zoom_factor(1.);
    frame(&ctx, &mut a, 1200., 950., vec![]);
    let draft = state(&a).assignment.draft.clone();
    edit(&ctx, &mut a, &draft);
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
}
