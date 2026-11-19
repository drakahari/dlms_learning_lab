use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::port_security_workspace as work;
const IDS: [&str; 3] = [
    "ccna-security-identity",
    "ccna-security-observe",
    "ccna-security-recover",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    super::nm_lessons::opened(id)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, input: bool) -> egui::Rect {
    for _ in 0..24 {
        frame(ctx, a, 1200., 950., vec![]);
        let r = if key == "challenge" {
            a.metrics.mode_challenge.unwrap()
        } else if input {
            a.metrics
                .port_security
                .fields
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        } else {
            a.metrics
                .port_security
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Secure-port control never became visible: {key}");
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key, true);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    let id = a
        .metrics
        .port_security
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(
        ctx.memory(|m| m.focused()),
        Some(id),
        "Field {key} must receive focus in the same measured viewport"
    );
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
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
#[test]
fn secure_port_native_edit_apply_check_and_stale_success_use_actual_widgets() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "maximum-desk", "bad");
    tap(&ctx, &mut a, "apply");
    assert!(!a.metrics.completion);
    assert!(a.notice.as_deref().unwrap().contains("Maximum"));
    field(&ctx, &mut a, "maximum-desk", "1");
    field(&ctx, &mut a, "configured-desk", "02:00:00:00:00:01");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "maximum-desk", "3");
    assert!(!a.metrics.completion);
    assert!(a.metrics.port_security.result.is_none());
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    let (passed, total, _) = a.metrics.port_security.result.unwrap();
    assert!(passed > 0 && passed < total);
    field(&ctx, &mut a, "maximum-desk", "1");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    field(&ctx, &mut a, "source", "malformed");
    assert!(
        !a.metrics.completion,
        "New replay input must not retain a prior checked display"
    );
    tap(&ctx, &mut a, "send");
    let State::PortSecurity { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert!(simulation.assignment.output.last().unwrap().error);
    assert!(!a.metrics.completion);
}
#[test]
fn secure_port_native_introduction_practice_and_assignment_keep_work_separate() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let d = work::source(&source).unwrap();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("WHAT YOU’LL LEARN"));
        assert!(text.contains("Worked"));
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        let State::PortSecurity { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let practice = simulation.practice.clone();
        assert_eq!(simulation.assignment.world, d.initial.initial);
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements"));
        assert!(!text.contains(work::PREFIX));
        assert!(!text.contains(&source.explanation));
        let State::PortSecurity { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice, practice);
        assert!(!simulation.assignment.checked);
    }
}
#[test]
fn secure_port_native_replacement_requires_manual_down_up_and_preserves_neighbor() {
    let (ctx, mut a, i) = opened(IDS[2]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "configured-desk", "02:00:00:00:00:01");
    tap(&ctx, &mut a, "mode-desk");
    tap(&ctx, &mut a, "mode-desk-Restrict");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "admin-desk");
    tap(&ctx, &mut a, "admin-desk");
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &a.scenarios[i]
    ));
    let State::PortSecurity { simulation } = &a.progress.attempts[IDS[2]].state else {
        panic!()
    };
    let d = work::source(&a.scenarios[i]).unwrap();
    assert_eq!(
        simulation.assignment.world.port("ops"),
        d.initial.initial.port("ops")
    );
}
#[test]
fn secure_port_controls_fit_five_scales_light_dark_narrow_wide_populated_and_disabled() {
    let (ctx, mut a, _) = opened(IDS[1]);
    let State::PortSecurity { simulation } =
        &mut a.progress.attempts.get_mut(IDS[1]).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
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
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 1100., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for (key, r, _, _) in &a.metrics.port_security.fields {
                        assert!(
                            r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                            "{key} {scale} {width} {light} {r:?} {clip:?}"
                        );
                        assert!(
                            r.height() > 20.,
                            "Single-line click target must not collapse"
                        );
                    }
                    for key in ["apply", "send", "check"] {
                        let r = a
                            .metrics
                            .port_security
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap()
                            .1;
                        assert!(
                            r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                            "{key} {scale} {width}: {r:?} {clip:?}"
                        );
                    }
                }
                a.reset_pending = false;
            }
        }
    }
}

#[test]
fn secure_port_worked_disclosure_is_an_actual_target_and_only_applies_practice() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "worked-example");
    tap(&ctx, &mut a, "fill-example");
    tap(&ctx, &mut a, "check");
    let State::PortSecurity { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    let d = work::source(&a.scenarios[i]).unwrap();
    assert!(
        d.example
            .checks(&simulation.practice.world)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(simulation.assignment.world, d.initial.initial);
    assert!(!simulation.assignment.checked);
    let result = a.metrics.port_security.result.unwrap().2;
    let clip = a.metrics.main_viewport.unwrap();
    assert!(
        clip.contains(result.center()),
        "Check must reveal its real result rather than leave it below the fold"
    );
}

#[test]
fn secure_port_replay_focus_and_draft_survive_column_reflow_and_tab_reaches_destination() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "source", "02:00:00:00:00:01");
    let source_id = a
        .metrics
        .port_security
        .fields
        .iter()
        .find(|r| r.0 == "source")
        .unwrap()
        .2;
    for width in [1600., 760., 1200.] {
        for _ in 0..3 {
            frame(&ctx, &mut a, width, 950., vec![]);
        }
        let current = a
            .metrics
            .port_security
            .fields
            .iter()
            .find(|r| r.0 == "source")
            .unwrap()
            .2;
        assert_eq!(
            current, source_id,
            "Editing identity must survive column reflow at width {width}"
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(source_id));
    }
    let r = target(&ctx, &mut a, "source", true);
    super::debian_contents::point_click(&ctx, &mut a, 1200., 950., r);
    frame(&ctx, &mut a, 1200., 950., vec![key(egui::Key::Tab, false)]);
    let destination = a
        .metrics
        .port_security
        .fields
        .iter()
        .find(|r| r.0 == "destination")
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(destination));
    let State::PortSecurity { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.source, "02:00:00:00:00:01");
}

#[test]
fn secure_port_optional_worked_controls_do_not_inherit_open_state_from_another_lesson() {
    let (ctx, mut a, _) = opened(IDS[0]);
    for id in IDS {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        for _ in 0..3 {
            frame(&ctx, &mut a, 1200., 950., vec![]);
        }
        tap(&ctx, &mut a, "example");
        assert!(
            !a.metrics
                .port_security
                .controls
                .iter()
                .any(|r| r.0 == "fill-example"),
            "Worked help starts closed in new lesson {id}"
        );
        tap(&ctx, &mut a, "worked-example");
        assert!(
            a.metrics
                .port_security
                .controls
                .iter()
                .any(|r| r.0 == "fill-example")
        );
        tap(&ctx, &mut a, "fill-example");
        tap(&ctx, &mut a, "check");
        let State::PortSecurity { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let d = work::source(&a.scenarios[i]).unwrap();
        assert!(
            d.example
                .checks(&simulation.practice.world)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(simulation.assignment.world, d.initial.initial);
    }
}

#[test]
fn secure_port_challenge_keeps_the_assignment_and_closes_unrequested_worked_help() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "help");
    tap(&ctx, &mut a, "worked-assignment");
    let explanation = a.scenarios[i].explanation.clone();
    assert!(painted(&ctx, &mut a).contains(&explanation));
    let before = a.progress.attempts[IDS[0]].state.clone();
    tap(&ctx, &mut a, "challenge");
    let text = painted(&ctx, &mut a);
    assert!(
        !text.contains(&explanation),
        "Challenge must not inherit an open solution disclosure from Learn"
    );
    assert_eq!(a.progress.attempts[IDS[0]].state, before);
    assert!(a.progress.attempts[IDS[0]].challenge);
}
