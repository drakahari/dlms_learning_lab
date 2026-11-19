//! Actual native events and rendered content for the listener assignment.
use super::debug_recovery::{opened, paint, settle, tap};
use super::*;
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = a.metrics.terminal.unwrap();
    tap(ctx, a, r);
    paint(
        ctx,
        a,
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
            egui::Event::Text(text.into()),
        ],
    );
    paint(
        ctx,
        a,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            })
            .collect(),
    );
    settle(ctx, a);
}
fn check(ctx: &egui::Context, a: &mut LabApp, want: bool) {
    let r = a.metrics.check.unwrap();
    tap(ctx, a, r);
    assert_eq!(a.metrics.completion, want);
}
#[test]
fn listener_audit_real_insert_enter_wrong_partial_retry_invalidation_and_independent_practice() {
    let (ctx, mut a, i) = opened("lpic1-listener-audit");
    let s = a.scenarios[i].clone();
    let id = &s.id;
    let l = s.process_teaching.as_ref().unwrap();
    a.set_process_view(i, teaching::View::Example);
    settle(&ctx, &mut a);
    let insert = a.metrics.insert.unwrap();
    tap(&ctx, &mut a, insert);
    let learn = a.progress.attempts[id]
        .workspace
        .as_ref()
        .unwrap()
        .process_learning
        .as_ref()
        .unwrap();
    assert_eq!(learn.command, l.example[0]);
    assert_eq!(learn.table, l.fixture);
    assert!(learn.transcript.is_empty());
    for c in &l.example {
        command(&ctx, &mut a, c);
    }
    let learn = a.progress.attempts[id]
        .workspace
        .as_ref()
        .unwrap()
        .process_learning
        .as_ref()
        .unwrap();
    assert!(l.outcomes.iter().all(|r| r.check(&learn.table).0));
    assert_eq!(a.progress.attempts[id].state, s.initial);
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    a.set_process_view(i, teaching::View::Assignment);
    settle(&ctx, &mut a);
    command(&ctx, &mut a, "ss -lntup");
    command(&ctx, &mut a, "kill -STOP 410");
    check(&ctx, &mut a, false);
    let issues =
        dlms_learning_lab::workspace::issues(&a.progress.attempts[id].state, &s.goal).join("\n");
    assert!(issues.contains("paused"));
    command(&ctx, &mut a, "kill -TERM 410");
    check(&ctx, &mut a, false);
    command(&ctx, &mut a, "kill -CONT 410");
    check(&ctx, &mut a, true);
    a.save();
    assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(p).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    command(&ctx, &mut a, "kill -STOP 411");
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    check(&ctx, &mut a, false);
    assert!(
        dlms_learning_lab::workspace::issues(&a.progress.attempts[id].state, &s.goal)
            .join("\n")
            .contains("owner is paused")
    );
    command(&ctx, &mut a, "kill -CONT 411");
    check(&ctx, &mut a, true);
}
#[test]
fn listener_audit_three_views_keep_answers_hidden_and_reflow_without_stretched_prose() {
    fn text(s: &egui::Shape, t: &mut String) {
        match s {
            egui::Shape::Text(x) => {
                if x.galley.rows.len() > 1 {
                    assert!(!x.galley.job.justify)
                }
                t.push_str(&x.galley.job.text);
                t.push('\n')
            }
            egui::Shape::Vec(v) => {
                for x in v {
                    text(x, t)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a, i) = opened("lpic1-listener-audit");
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        a.set_process_view(i, view);
        settle(&ctx, &mut a);
        let o = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 7000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut t = String::new();
        for sh in o.shapes {
            text(&sh.shape, &mut t)
        }
        match view {
            teaching::View::Introduction => {
                assert!(t.contains("Read the numeric socket view"));
                assert!(t.contains("Expected after STOP"))
            }
            teaching::View::Example => {
                assert!(t.contains("Pause and re-inspect"));
                assert!(t.contains("Insert command"))
            }
            teaching::View::Assignment => {
                assert!(t.contains("Requirements"));
                assert!(!t.contains("kill -TERM 410"));
                assert!(!t.contains("Expected after STOP"));
                assert!(t.contains("Check work"))
            }
        }
    }
    a.set_process_view(i, teaching::View::Assignment);
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
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 950., vec![])
                }
                let vp = a.metrics.main_viewport.unwrap();
                let input = a.metrics.terminal.unwrap();
                let check = a.metrics.check.unwrap();
                assert!(
                    input.left() >= vp.left() - 2. && input.right() <= vp.right() + 2.,
                    "{scale}/{width}: {input:?}/{vp:?}"
                );
                assert!(check.right() <= vp.right() + 2.);
            }
        }
    }
    a.progress
        .attempts
        .get_mut("lpic1-listener-audit")
        .unwrap()
        .challenge = true;
    settle(&ctx, &mut a);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
    assert_eq!(a.process_view(i), teaching::View::Assignment);
}
