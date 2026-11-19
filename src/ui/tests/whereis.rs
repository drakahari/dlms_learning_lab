//! Actual native widget trees for the new support-artifact lesson; no host commands.
use super::{
    debug_recovery::{opened, settle, tap},
    *,
};
const ID: &str = "lpic1-whereis-support-handover";
fn texts(shape: &egui::Shape, out: &mut String) {
    match shape {
        egui::Shape::Text(t) => {
            out.push_str(&t.galley.job.text);
            out.push('\n');
        }
        egui::Shape::Vec(v) => {
            for s in v {
                texts(s, out)
            }
        }
        _ => {}
    }
}
fn text(ctx: &egui::Context, a: &mut LabApp, width: f32) -> String {
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 9000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for s in out.shapes {
        texts(&s.shape, &mut text);
    }
    text
}
#[test]
fn whereis_three_views_retain_full_task_separate_practice_and_no_default_assignment_solution() {
    let (ctx, mut a, i) = opened(ID);
    settle(&ctx, &mut a);
    let initial = a.progress.attempts[ID].state.clone();
    let intro = text(&ctx, &mut a, 1500.);
    assert!(
        intro.contains("WHAT YOU’LL LEARN")
            && intro.contains("Read the query and its result")
            && intro.contains("Worked example")
            && intro.contains("whereis -m -M /gallery/man -f democtl")
    );
    assert!(!intro.contains("1. 1."));
    a.set_terminal_view(i, teaching::View::Example);
    settle(&ctx, &mut a);
    let practice = text(&ctx, &mut a, 1500.);
    assert!(
        practice.contains("Find the working binary and missing manual")
            && practice.contains("Insert command")
            && practice.contains("Practice terminal")
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    a.set_terminal_view(i, teaching::View::Assignment);
    settle(&ctx, &mut a);
    let task = text(&ctx, &mut a, 1500.);
    assert!(
        task.contains("YOUR TASK")
            && task.contains("Requirements")
            && task.contains("/apps/stage/release.txt")
            && task.contains("Preserve the binary")
    );
    assert!(!task.contains("cp /apps/stage/reportctl.1 /apps/man/reportctl.1"));
    assert!(!task.contains("/gallery/stage/democtl.1"));
    assert_eq!(a.progress.attempts[ID].state, initial);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Assignment);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
}
#[test]
fn whereis_wrapped_worked_commands_keep_all_flags_visible_at_normal_and_large_sizes() {
    let catalog = pack::ccna_pilot().unwrap();
    let s = catalog.iter().find(|s| s.id == ID).unwrap();
    let expected = &s.terminal_teaching.as_ref().unwrap().example;
    for light in [false, true] {
        for scale in [1., 1.5, 2.] {
            for width in [550., 1300.] {
                let ctx = egui::Context::default();
                ctx.set_zoom_factor(scale);
                let theme = if light {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                theme::apply(&ctx, &theme);
                let mut allowed = 0.;
                let out = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 9000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let full = ui.available_width();
                            let wide = full >= ui.text_style_height(&egui::TextStyle::Body) * 38.;
                            allowed = if wide {
                                (full - ui.spacing().item_spacing.x) / 2.
                            } else {
                                full
                            };
                            lesson_presentation::wrapped_worked_commands(ui, s);
                        });
                    },
                );
                fn collect<'a>(s: &'a egui::Shape, out: &mut Vec<&'a egui::epaint::TextShape>) {
                    match s {
                        egui::Shape::Text(t) => out.push(t),
                        egui::Shape::Vec(v) => {
                            for s in v {
                                collect(s, out)
                            }
                        }
                        _ => {}
                    }
                }
                let mut shapes = vec![];
                for s in &out.shapes {
                    collect(&s.shape, &mut shapes);
                }
                for cmd in expected {
                    let t = shapes
                        .iter()
                        .find(|t| t.galley.job.text == *cmd)
                        .expect("Every complete worked command is rendered");
                    assert!(
                        t.galley.size().x <= allowed + 1.,
                        "command exceeds native column at {width}/{scale}: {cmd}"
                    );
                    assert!(!t.galley.job.justify);
                }
            }
        }
    }
}
#[test]
fn whereis_failure_feedback_distinguishes_a_found_old_release_from_current_work() {
    let (ctx, mut a, i) = opened(ID);
    let s = a.scenarios[i].clone();
    a.set_terminal_view(i, teaching::View::Assignment);
    for c in &s.solution {
        engine::execute_for_goal(
            &mut a.progress.attempts.get_mut(ID).unwrap().state,
            &s.goal,
            c,
        )
        .unwrap();
    }
    assert!(progress::current_solved(&a.progress.attempts[ID], &s));
    engine::execute_for_goal(
        &mut a.progress.attempts.get_mut(ID).unwrap().state,
        &s.goal,
        "cp /apps/archive/reportctl.c /apps/source/reportctl.c",
    )
    .unwrap();
    settle(&ctx, &mut a);
    let check = a.metrics.check.unwrap();
    tap(&ctx, &mut a, check);
    assert!(a.checked);
    let output = text(&ctx, &mut a, 1500.);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    assert!(!a.metrics.completion);
    assert!(output.contains("release contents") || output.contains("different text"));
}
