use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::pam_workspace as work;
const ID: &str = "lpic2-pam-auth-bypass";
fn opened() -> (egui::Context, LabApp, usize) {
    super::nm_lessons::opened(ID)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, editor: bool) -> egui::Rect {
    for _ in 0..24 {
        frame(ctx, a, 1200., 950., vec![]);
        let r = if editor {
            a.metrics.pam.editor.unwrap().0
        } else {
            a.metrics
                .pam
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
    panic!("PAM control never visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "editor", true);
    click(ctx, a, r);
    let id = a.metrics.pam.editor.unwrap().1;
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
    let State::PamStack { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    let scope = if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    };
    assert_eq!(scope.draft, text);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..12 {
        frame(ctx, a, 1200., 950., vec![]);
        if a.pam_run.is_none() {
            frame(ctx, a, 1200., 950., vec![]);
            return;
        }
    }
    panic!("PAM check did not finish");
}
fn result(a: &LabApp) -> Option<(usize, usize)> {
    a.metrics.pam.result.map(|r| (r.0, r.1))
}
#[test]
fn pam_views_teach_controls_and_keep_requested_practice_out_of_assignment() {
    let (ctx, mut a, i) = opened();
    let s = a.scenarios[i].clone();
    let text = painted(&ctx, &mut a);
    assert!(
        text.contains("Required remembers")
            && text.contains("Worked example")
            && text.contains("supplied")
    );
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((1, 5)));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Expected Rejected; actual Accepted") && text.contains("failure"));
    tap(&ctx, &mut a, "insert");
    assert!(result(&a).is_none());
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((5, 5)));
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    tap(&ctx, &mut a, "assignment");
    let State::PamStack { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    let engine::Goal::PamStack { definition } = &s.goal else {
        panic!()
    };
    assert_eq!(simulation.assignment.draft, definition.initial.starter);
    assert!(result(&a).is_none());
    let text = painted(&ctx, &mut a);
    assert!(
        !text.contains("Fill worked example")
            && !text.contains("Worked approach · reveals a solution")
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Your task") || text.contains("YOUR TASK"));
    assert!(!text.contains("Required remembers") && !text.contains("Try example · ungraded"));
}
#[test]
fn pam_keyboard_edit_stop_actual_feedback_retry_and_save_are_real() {
    let (ctx, mut a, i) = opened();
    let s = a.scenarios[i].clone();
    tap(&ctx, &mut a, "assignment");
    let r = target(&ctx, &mut a, "run", false);
    click(&ctx, &mut a, r);
    assert!(a.pam_run.is_some());
    let stop = a
        .metrics
        .pam
        .controls
        .iter()
        .find(|r| r.0 == "stop")
        .unwrap()
        .1;
    click(&ctx, &mut a, stop);
    assert!(a.pam_run.is_none());
    let State::PamStack { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.assignment.tested.is_none());
    field(&ctx, &mut a, "auth required pam_unix.so");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((5, 5)));
    assert!(progress::current_solved(&a.progress.attempts[ID], &s));
    a.save();
    assert!(a.storage_error.is_none());
    assert_eq!(
        serde_json::to_value(progress::load(&a.path, &a.scenarios).unwrap()).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    field(&ctx, &mut a, "auth required pam_permit.so");
    assert!(result(&a).is_none());
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((1, 5)));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Stack needs work") && text.contains("before checking"));
    field(&ctx, &mut a, "auth required pam_unix.so nullok");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((0, 5)));
    assert!(painted(&ctx, &mut a).contains("Unsupported in this simulator"));
    field(
        &ctx,
        &mut a,
        "auth sufficient pam_unix.so\nauth required pam_deny.so",
    );
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((5, 5)));
    assert!(progress::current_solved(&a.progress.attempts[ID], &s));
}
#[test]
fn pam_editor_and_actions_fit_five_scales_and_light_dark_widths() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
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
                theme::inventory()[0].id.clone()
            };
            for width in [760., 1600.] {
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                for r in [
                    a.metrics.pam.editor.unwrap().0,
                    a.metrics
                        .pam
                        .controls
                        .iter()
                        .find(|r| r.0 == "run")
                        .unwrap()
                        .1,
                    a.metrics
                        .pam
                        .controls
                        .iter()
                        .find(|r| r.0 == "stop")
                        .unwrap()
                        .1,
                ] {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "{scale}/{width} {r:?} {clip:?}"
                    );
                }
            }
        }
    }
    assert_eq!(work::MAX_SESSION, 72 * 1024);
}

#[test]
fn one_parser_error_is_distinct_from_five_authentication_decisions() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "auth required pam_unix.so nullok");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((0, 5)));
    let text = painted(&ctx, &mut a);
    assert_eq!(text.matches("Unsupported in this simulator").count(), 1);
    assert!(text.contains("Actual results") && text.contains("No authentication decisions"));
}

#[test]
fn actual_outer_wheel_reaches_checked_results_at_native_narrow200_geometry() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    engine::execute_for_goal(
        &mut a.progress.attempts.get_mut(ID).unwrap().state,
        &source.goal,
        &source.solution[0],
    )
    .unwrap();
    let State::PamStack { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    a.scale = 2.;
    ctx.set_zoom_factor(2.);
    a.theme_id = theme::inventory()
        .iter()
        .find(|t| t.light)
        .unwrap()
        .id
        .clone();
    a.smoke_stage = 92052;
    a.lesson_scroll_reset = true;
    for _ in 0..4 {
        frame(&ctx, &mut a, 1268., 1308., vec![]);
    }
    assert_eq!(result(&a), Some((5, 5)));
    let mut reached = false;
    for _ in 0..20 {
        let target = a.metrics.pam.result.unwrap().2;
        if a.reveal_smoke_control(&ctx, target) {
            reached = true;
            break;
        }
        let events = std::mem::take(&mut a.pending_events);
        frame(&ctx, &mut a, 1268., 1308., events);
        for _ in 0..3 {
            frame(&ctx, &mut a, 1268., 1308., vec![]);
        }
    }
    assert!(
        reached,
        "real outer scroll failed, offset={:?}, content={:?}",
        a.metrics.main_scroll_offset, a.metrics.main_content_size
    );
    assert!(a.metrics.main_scroll_offset.y > 0.);
}
