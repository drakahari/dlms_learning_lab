//! Actual widget trees for the restricted-deletion terminal lessons, not novice acceptance.
use super::*;
use dlms_learning_lab::directory_policy_workspace as work;
const IDS: [&str; 2] = [
    "lpic1-sticky-collaboration",
    "lpic1-sticky-collision-handover",
];
fn setup(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = debug_recovery::opened(id);
    let State::DirectoryPolicy { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    (ctx, a, i)
}
fn scope<'a>(a: &'a LabApp, id: &str, practice: bool) -> &'a work::Scope {
    let State::DirectoryPolicy { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    if practice {
        &simulation.practice
    } else {
        &simulation.assignment
    }
}
fn full(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1500., 7000., vec![]);
    }
}
fn control(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .directory_policy
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
fn replace(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = a
        .metrics
        .directory_policy
        .fields
        .iter()
        .find(|r| r.0 == "command")
        .unwrap()
        .1;
    click(ctx, a, r);
    frame(
        ctx,
        a,
        1500.,
        7000.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    full(ctx, a);
}
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    replace(ctx, a, text);
    frame(ctx, a, 1500., 7000., vec![key(egui::Key::Enter, false)]);
    full(ctx, a);
}
fn text(ctx: &egui::Context, a: &mut LabApp) -> String {
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 7000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    fn collect(s: &egui::Shape, t: &mut String) {
        match s {
            egui::Shape::Text(s) => {
                t.push_str(&s.galley.job.text);
                t.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, t)
                }
            }
            _ => {}
        }
    }
    let mut t = String::new();
    for s in out.shapes {
        collect(&s.shape, &mut t)
    }
    t
}
#[test]
fn directory_policy_three_views_teach_real_operations_without_exposing_assignment_answers() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Introduction);
        full(&ctx, &mut a);
        let t = text(&ctx, &mut a);
        assert!(t.contains("WHAT YOU’LL LEARN") && t.contains("Worked example"));
        for c in &dlms_learning_lab::directory_policy_source::source(&a.scenarios[i])
            .unwrap()
            .example_commands
        {
            assert!(t.contains(c), "missing worked {c}");
        }
        let r = control(&a, "example");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        let t = text(&ctx, &mut a);
        assert!(
            t.contains("Insert command")
                && t.contains("Actual practice terminal")
                && t.contains("Run")
        );
        let r = control(&a, "assignment");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        let t = text(&ctx, &mut a);
        assert!(
            t.contains("YOUR TASK")
                && t.contains("Requirements")
                && t.contains("Simulated assignment terminal")
        );
        for c in &a.scenarios[i].solution {
            assert!(!t.contains(c), "assignment exposes {c}");
        }
        assert!(!t.contains("/practice/"));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        full(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!text(&ctx, &mut a).contains("Worked example"));
    }
}
#[test]
fn directory_policy_insert_is_not_execution_and_participant_checks_are_discarded() {
    let id = IDS[0];
    let (ctx, mut a, i) = setup(id, teaching::View::Example);
    full(&ctx, &mut a);
    let before = scope(&a, id, true).world.clone();
    let assignment = scope(&a, id, false).clone();
    let r = control(&a, "insert");
    click(&ctx, &mut a, r);
    full(&ctx, &mut a);
    assert_eq!(scope(&a, id, true).world, before);
    assert!(!scope(&a, id, true).command.is_empty());
    assert!(scope(&a, id, true).transcript.is_empty());
    let r = control(&a, "participants");
    click(&ctx, &mut a, r);
    full(&ctx, &mut a);
    let r = control(&a, "participant-run");
    click(&ctx, &mut a, r);
    full(&ctx, &mut a);
    assert_eq!(scope(&a, id, true).world, before);
    assert_eq!(scope(&a, id, false), &assignment);
    assert!(!progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
    assert!(text(&ctx, &mut a).contains("Expected outcome observed"));
}
#[test]
fn directory_policy_real_enter_checks_partial_failure_alternate_repair_invalidation_and_resume() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Assignment);
        full(&ctx, &mut a);
        let s = a.scenarios[i].clone();
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert!(a.metrics.directory_policy.result.unwrap().1 > 0);
        for c in &s.solution {
            command(&ctx, &mut a, c);
        }
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert_eq!(a.metrics.directory_policy.result.unwrap().1, 0);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        let broken = if id == IDS[0] {
            "chmod 0700 /work/shared"
        } else {
            "cp /work/handover/bea-release.txt /work/handover/release.txt"
        };
        command(&ctx, &mut a, broken);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(a.metrics.directory_policy.result.is_none());
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        let t = text(&ctx, &mut a);
        assert!(t.contains("Needs attention"));
        assert!(if id == IDS[0] {
            t.contains("ordinary") || t.contains("permission")
        } else {
            t.contains("contents") || t.contains("files")
        });
        let repair = if id == IDS[0] {
            "chmod 1770 /work/shared"
        } else {
            "cp /work/retained-release.txt /work/handover/release.txt"
        };
        command(&ctx, &mut a, repair);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        replace(&ctx, &mut a, "ls -ld /work ");
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let read = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&read).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        assert_eq!(scope(&a, id, false).command, "ls -ld /work ");
    }
}
#[test]
fn directory_policy_example_reset_retains_assignment_draft_and_does_not_award_credit() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Assignment);
        full(&ctx, &mut a);
        replace(&ctx, &mut a, "unfinished assignment");
        let original = scope(&a, id, false).clone();
        let r = control(&a, "example");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        let commands = dlms_learning_lab::directory_policy_source::source(&a.scenarios[i])
            .unwrap()
            .example_commands
            .clone();
        for c in commands {
            command(&ctx, &mut a, &c);
        }
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert_eq!(a.metrics.directory_policy.result.unwrap().1, 0);
        assert_eq!(scope(&a, id, false), &original);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let r = control(&a, "reset-example");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert!(scope(&a, id, true).history.is_empty());
        assert!(!scope(&a, id, true).checked);
        assert_eq!(scope(&a, id, false), &original);
    }
}
#[test]
fn directory_policy_controls_reflow_and_focus_at_supported_scales_and_light_dark() {
    for id in IDS {
        for light in [false, true] {
            for scale in [1., 1.5, 2.] {
                for width in [760., 1500.] {
                    let (ctx, mut a, _) = setup(id, teaching::View::Example);
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
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 7000., vec![]);
                    }
                    let viewport = a.metrics.main_viewport.unwrap();
                    let fields = &a.metrics.directory_policy.fields;
                    for (_, r, _) in fields {
                        assert!(
                            r.left() >= viewport.left() - 1. && r.right() <= viewport.right() + 1.,
                            "field clipped {id}/{width}/{scale}: {r:?}/{viewport:?}"
                        );
                        assert!(r.height() > 18.);
                    }
                    for (k, r, _) in &a.metrics.directory_policy.controls {
                        assert!(
                            r.left() >= viewport.left() - 1. && r.right() <= viewport.right() + 1.,
                            "control {k} clipped {id}/{width}/{scale}: {r:?}/{viewport:?}"
                        );
                    }
                    let field = a
                        .metrics
                        .directory_policy
                        .fields
                        .iter()
                        .find(|r| r.0 == "command")
                        .unwrap()
                        .clone();
                    click(&ctx, &mut a, field.1);
                    assert_eq!(ctx.memory(|m| m.focused()), Some(field.2));
                }
            }
        }
    }
}
