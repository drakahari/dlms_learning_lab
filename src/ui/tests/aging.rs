use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::aging_workspace as work;
const IDS: [&str; 3] = [
    "lpic1-aging-rotation",
    "lpic1-aging-grace",
    "lpic1-aging-contract-reset",
];

fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    super::nm_lessons::opened(id)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, input: bool) -> egui::Rect {
    for _ in 0..24 {
        frame(ctx, a, 1200., 950., vec![]);
        let r = if input {
            a.metrics.aging.input.unwrap().0
        } else {
            a.metrics
                .aging
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
    panic!("Aging control never visible: {key}");
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "input", true);
    click(ctx, a, r);
    let id = a.metrics.aging.input.unwrap().1;
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
}
#[test]
fn aging_keyboard_query_run_retry_and_current_state_grading_are_real() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "query");
    let State::Aging { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert!(simulation.assignment.transcript.is_empty());
    assert_eq!(simulation.assignment.input, "chage -l morgan");
    let input = target(&ctx, &mut a, "input", true);
    click(&ctx, &mut a, input);
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    let State::Aging { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert!(
        simulation
            .assignment
            .transcript
            .last()
            .unwrap()
            .text
            .contains("2026-01-01")
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "chage -M 45 morgan");
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "check");
    let (p, n, _) = a.metrics.aging.result.unwrap();
    assert!(p > 0 && p < n);
    assert!(painted(&ctx, &mut a).contains("warning window"));
    field(&ctx, &mut a, "chage --warndays=10 morgan");
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    field(&ctx, &mut a, "chage -M -1 morgan");
    tap(&ctx, &mut a, "run");
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    field(&ctx, &mut a, "chage -M 45 --root / morgan");
    tap(&ctx, &mut a, "run");
    assert!(painted(&ctx, &mut a).contains("Unsupported chage option"));
}
#[test]
fn all_three_views_teach_and_preserve_separate_work_without_independent_answers() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let d = work::source(&source).unwrap();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Worked example"));
        assert!(text.contains(&d.example.target));
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "query");
        tap(&ctx, &mut a, "run");
        tap(&ctx, &mut a, "check");
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        let State::Aging { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let practice = simulation.practice.clone();
        assert_eq!(simulation.assignment.world, d.initial.initial);
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements"));
        assert!(!text.contains(&d.example_worked[0].0));
        let plan: Vec<work::Command> =
            serde_json::from_str(source.solution[0].strip_prefix(work::PREFIX).unwrap()).unwrap();
        assert!(
            !text.contains(&plan[0].0),
            "independent command solution must stay hidden"
        );
        let State::Aging { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice, practice);
        assert!(!simulation.assignment.checked);
    }
}
#[test]
fn aging_controls_and_case_columns_fit_five_scales_light_dark_and_narrow_wide() {
    let (ctx, mut a, i) = opened(IDS[2]);
    let source = a.scenarios[i].clone();
    let engine::State::Aging { simulation } =
        &mut a.progress.attempts.get_mut(IDS[2]).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    for checked in [false, true] {
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
                    let State::Aging { simulation } =
                        &mut a.progress.attempts.get_mut(IDS[2]).unwrap().state
                    else {
                        panic!()
                    };
                    simulation.assignment.checked = checked;
                    simulation.assignment.input = if checked {
                        "chage --expiredate=2026-10-20 --lastday=-1 sage".into()
                    } else {
                        String::new()
                    };
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 1100., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for key in ["query", "run", "check"] {
                        let r = a
                            .metrics
                            .aging
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap()
                            .1;
                        assert!(
                            r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                            "{scale}/{width}/{light}: {key} {r:?} {clip:?}"
                        );
                    }
                    let input = a.metrics.aging.input.unwrap().0;
                    assert!(input.left() >= clip.left() - 2. && input.right() <= clip.right() + 2.);
                    assert!(
                        a.metrics.main_content_size.x <= clip.width() + 4.,
                        "columns must reflow without horizontal overflow: scale={scale}, width={width}, light={light}, content={:?}, clip={clip:?}",
                        a.metrics.main_content_size
                    );
                }
            }
        }
    }
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[2]],
        &source
    ));
}
