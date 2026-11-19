//! Actual widget trees for the restricted-deletion terminal lessons, not novice acceptance.
use super::*;
use dlms_learning_lab::recursive_tree_workspace as work;
const IDS: [&str; 2] = ["lpic1-tree-private-stage", "lpic1-tree-verified-retirement"];
fn setup(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = debug_recovery::opened(id);
    let State::RecursiveTree { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    (ctx, a, i)
}
fn scope<'a>(a: &'a LabApp, id: &str, practice: bool) -> &'a work::Scope {
    let State::RecursiveTree { simulation } = &a.progress.attempts[id].state else {
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
        .recursive_tree
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
fn replace(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = a
        .metrics
        .recursive_tree
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
fn recursive_tree_three_views_teach_real_operations_without_exposing_assignment_answers() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Introduction);
        full(&ctx, &mut a);
        let t = text(&ctx, &mut a);
        assert!(t.contains("WHAT YOU’LL LEARN") && t.contains("Worked example"));
        for c in &dlms_learning_lab::recursive_tree_source::source(&a.scenarios[i])
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
fn recursive_tree_insert_populates_without_running() {
    let (ctx, mut a, _) = setup(IDS[0], teaching::View::Example);
    full(&ctx, &mut a);
    let original = scope(&a, IDS[0], false).clone();
    let tree = scope(&a, IDS[0], true).tree.clone();
    let r = control(&a, "insert");
    click(&ctx, &mut a, r);
    full(&ctx, &mut a);
    assert!(!scope(&a, IDS[0], true).command.is_empty());
    assert_eq!(scope(&a, IDS[0], true).tree, tree);
    assert!(scope(&a, IDS[0], true).transcript.is_empty());
    assert_eq!(scope(&a, IDS[0], false), &original);
}
#[test]
fn recursive_tree_real_enter_checks_partial_failure_alternate_repair_invalidation_and_resume() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Assignment);
        full(&ctx, &mut a);
        let s = a.scenarios[i].clone();
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert!(a.metrics.recursive_tree.result.unwrap().1 > 0);
        for c in s
            .solution
            .iter()
            .filter(|c| c.as_str() != work::CHECK_ACTION)
        {
            command(&ctx, &mut a, c);
        }
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert_eq!(a.metrics.recursive_tree.result.unwrap().1, 0);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        let broken = if id == IDS[0] {
            "chmod 000 /work/private-copy/README.txt"
        } else {
            "chmod 000 /work/handover/README.txt"
        };
        command(&ctx, &mut a, broken);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(a.metrics.recursive_tree.result.is_none());
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        let t = text(&ctx, &mut a);
        assert!(t.contains("Needs attention"));
        assert!(t.contains("permission") || t.contains("private copy"));
        let repair = if id == IDS[0] {
            "chmod 600 /work/private-copy/README.txt"
        } else {
            "chmod 644 /work/handover/README.txt"
        };
        command(&ctx, &mut a, repair);
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
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
fn recursive_tree_example_reset_retains_assignment_draft_and_does_not_award_credit() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Assignment);
        full(&ctx, &mut a);
        replace(&ctx, &mut a, "unfinished assignment");
        let original = scope(&a, id, false).clone();
        let r = control(&a, "example");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        let commands = dlms_learning_lab::recursive_tree_source::source(&a.scenarios[i])
            .unwrap()
            .example_commands
            .clone();
        for c in commands {
            command(&ctx, &mut a, &c);
        }
        let r = control(&a, "check");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert_eq!(a.metrics.recursive_tree.result.unwrap().1, 0);
        assert_eq!(scope(&a, id, false), &original);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let r = control(&a, "reset-example");
        click(&ctx, &mut a, r);
        full(&ctx, &mut a);
        assert!(scope(&a, id, true).transcript.is_empty());
        assert!(!scope(&a, id, true).checked);
        assert_eq!(scope(&a, id, false), &original);
    }
}
#[test]
fn recursive_tree_controls_reflow_and_focus_at_supported_scales_and_light_dark() {
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
                    let fields = &a.metrics.recursive_tree.fields;
                    for (_, r, _) in fields {
                        assert!(
                            r.left() >= viewport.left() - 1. && r.right() <= viewport.right() + 1.,
                            "field clipped {id}/{width}/{scale}: {r:?}/{viewport:?}"
                        );
                        assert!(r.height() > 18.);
                    }
                    for (k, r, _) in &a.metrics.recursive_tree.controls {
                        assert!(
                            r.left() >= viewport.left() - 1. && r.right() <= viewport.right() + 1.,
                            "control {k} clipped {id}/{width}/{scale}: {r:?}/{viewport:?}"
                        );
                    }
                    let field = a
                        .metrics
                        .recursive_tree
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
