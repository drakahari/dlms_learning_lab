use super::*;
use dlms_learning_lab::{qos as core, qos_workspace as work};
const IDS: [&str; 3] = [
    "ccna-qos-mark-boundary",
    "ccna-qos-queue-bursts",
    "ccna-qos-trusted-handover",
];
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let State::Qos { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state else {
        panic!()
    };
    simulation.view = view;
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp, i: usize) -> &work::Session {
    let State::Qos { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn rect(ctx: &egui::Context, a: &mut LabApp, key: &str, number: bool) -> egui::Rect {
    for _ in 0..40 {
        let r = if number {
            a.metrics.qos.fields.iter().find(|r| r.0 == key).unwrap().1
        } else {
            a.metrics
                .qos
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
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-400., 400.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..6 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("QoS control never visible {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = rect(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn number(ctx: &egui::Context, a: &mut LabApp, key: &str, value: u8) {
    let r = rect(ctx, a, key, true);
    click(ctx, a, r);
    let id = a.metrics.qos.fields.iter().find(|r| r.0 == key).unwrap().2;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        ctx,
        a,
        1200.,
        950.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Paste(value.to_string()),
        ],
    );
    frame(ctx, a, 1200., 950., vec![key_event(egui::Key::Enter)]);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn key_event(k: egui::Key) -> egui::Event {
    key(k, false)
}
#[test]
fn qos_worked_fill_is_not_execution_and_practice_does_not_credit_assignment() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Example);
        let original = session(&a, i).assignment.clone();
        tap(&ctx, &mut a, "insert");
        assert!(session(&a, i).practice.tested.is_none());
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        tap(&ctx, &mut a, "run");
        let engine::Goal::Qos { definition } = &a.scenarios[i].goal else {
            panic!()
        };
        assert!(session(&a, i).practice.passed(&definition.example));
        assert_eq!(session(&a, i).assignment, original);
        assert!(!a.metrics.completion);
        assert_eq!(
            a.metrics.qos.result.as_ref().map(|r| (r.0, r.1)),
            Some((3, 3))
        );
    }
}
#[test]
fn qos_actual_numeric_edit_invalidates_success_and_retry_uses_new_value() {
    let (ctx, mut a, i) = opened(IDS[0], teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    for action in &s.solution {
        engine::execute_for_goal(
            &mut a.progress.attempts.get_mut(&s.id).unwrap().state,
            &s.goal,
            action,
        )
        .unwrap();
    }
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    number(&ctx, &mut a, "mark:office-interactive:code", 45);
    assert_eq!(
        session(&a, i).assignment.draft.marks["office-interactive"],
        core::Mark::Set(45)
    );
    assert!(!a.metrics.completion);
    assert!(session(&a, i).assignment.tested.is_none());
    tap(&ctx, &mut a, "run");
    assert!(!a.metrics.completion);
    number(&ctx, &mut a, "mark:office-interactive:code", 46);
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "run");
    assert!(a.metrics.completion);
}
#[test]
fn qos_three_views_have_optional_guidance_and_challenge_keeps_task_without_worked_policy() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Introduction);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(a.metrics.qos.controls.iter().any(|r| r.0 == "try"));
        tap(&ctx, &mut a, "assignment");
        assert!(a.metrics.check.is_some());
        assert!(!a.metrics.completion);
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.help_visible);
        assert!(session(&a, i).assignment.tested.is_none());
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn qos_fields_and_controls_reflow_at_readable_sizes_in_light_and_dark() {
    let (ctx, mut a, _) = opened(IDS[2], teaching::View::Assignment);
    tap(&ctx, &mut a, "run");
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
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
            for width in [760., 1600.] {
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                for (key, r, _) in &a.metrics.qos.fields {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "{key} scale{scale} width{width} {r:?} {clip:?}"
                    );
                }
                let run = a.metrics.check.unwrap();
                assert!(run.left() >= clip.left() - 2. && run.right() <= clip.right() + 2.);
            }
        }
    }
}

#[test]
fn qos_overall_failure_names_bad_patterns_and_explains_them_while_a_correct_case_stays_passed() {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a, i) = opened(IDS[0], teaching::View::Assignment);
    let engine::Goal::Qos { definition } = &mut a.scenarios[i].goal else {
        panic!()
    };
    for packet in &mut definition.initial.workloads[0].fixture.packets {
        if packet.flow.ends_with("-interactive") {
            packet.dscp = 46;
        } else if packet.flow.ends_with("-guest") {
            packet.dscp = 0;
        }
    }
    definition.validate().unwrap();
    let s = a.scenarios[i].clone();
    let State::Qos { simulation } = &mut a.progress.attempts.get_mut(IDS[0]).unwrap().state else {
        panic!()
    };
    let engine::Goal::Qos { definition } = &s.goal else {
        panic!()
    };
    simulation.assignment.run(&definition.initial).unwrap();
    let mut text = String::new();
    for _ in 0..3 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 6000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        for s in output.shapes {
            collect(&s.shape, &mut text);
        }
    }
    assert_eq!(
        a.metrics.qos.result.as_ref().map(|r| (r.0, r.1)),
        Some((1, 3))
    );
    assert!(text.contains("Passed · Opening burst"), "{text}");
    assert!(
        text.contains("Needs work · Later interactive arrivals"),
        "{text}"
    );
    assert!(
        text.contains("First failing packet pattern · Later interactive arrivals"),
        "{text}"
    );
    assert!(text.contains("required outgoing DSCP 46; actual"), "{text}");
    assert!(!a.metrics.completion);
}
