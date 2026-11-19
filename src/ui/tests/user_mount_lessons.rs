//! The native editor and caller controls operate the same bounded model as grading.
use super::debug_recovery::{opened, paint, settle, tap, type_editor};
use super::*;
use dlms_learning_lab::{engine::State, user_mounts_workspace as w};
const IDS: [&str; 2] = [
    "lpic1-removable-personal-session",
    "lpic1-removable-shared-handover",
];
fn session<'a>(a: &'a LabApp, id: &str) -> &'a w::Session {
    let State::UserMounts { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn view(a: &mut LabApp, id: &str, v: teaching::View) {
    let State::UserMounts { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.lesson_scroll_reset = true;
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let rect = a
        .metrics
        .user_mounts
        .controls
        .iter()
        .find(|c| c.0 == key)
        .unwrap_or_else(|| panic!("missing {key}"))
        .1;
    tap(ctx, a, rect);
}
fn finish(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..50 {
        paint(ctx, a, vec![]);
        if a.user_mounts_run.is_none() {
            settle(ctx, a);
            return;
        }
    }
    panic!("unbounded mount run");
}
fn apply(ctx: &egui::Context, a: &mut LabApp, action: &w::Action) {
    match action {
        w::Action::Save { configuration } => {
            type_editor(ctx, a, configuration);
            click(ctx, a, "save");
        }
        w::Action::Mount { actor, target } | w::Action::Unmount { actor, target } => {
            click(ctx, a, "actor");
            click(ctx, a, &format!("actor:{actor:?}"));
            click(ctx, a, "target");
            click(ctx, a, &format!("target:{target}"));
            click(
                ctx,
                a,
                if matches!(action, w::Action::Mount { .. }) {
                    "mount"
                } else {
                    "unmount"
                },
            );
        }
    }
}
#[test]
fn user_mount_native_reference_actual_session_feedback_and_resume() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        view(&mut a, id, teaching::View::Assignment);
        settle(&ctx, &mut a);
        click(&ctx, &mut a, "check");
        finish(&ctx, &mut a);
        assert!(!a.metrics.completion);
        assert!(session(&a, id).assignment.result.is_some());
        let solution = s
            .solution
            .iter()
            .map(|c| serde_json::from_str::<w::Action>(c.strip_prefix(w::PREFIX).unwrap()).unwrap())
            .collect::<Vec<_>>();
        apply(&ctx, &mut a, &solution[0]);
        assert!(a.notice.as_deref().unwrap().starts_with("Saved."));
        click(&ctx, &mut a, "check");
        finish(&ctx, &mut a);
        assert!(
            !a.metrics.completion,
            "saved text cannot complete a missing live operation"
        );
        for action in &solution[1..] {
            apply(&ctx, &mut a, action);
        }
        click(&ctx, &mut a, "check");
        finish(&ctx, &mut a);
        assert!(
            a.metrics.completion,
            "{id}: {:?}",
            session(&a, id).assignment.result
        );
        a.save();
        assert!(a.storage_error.is_none());
        let loaded = progress::load(&a.path, &a.scenarios).unwrap();
        assert!(!progress::current_solved(&loaded.attempts[id], &s));
        let State::UserMounts { simulation } = &loaded.attempts[id].state else {
            panic!()
        };
        assert_eq!(
            simulation.assignment.history,
            session(&a, id).assignment.history
        );
        type_editor(&ctx, &mut a, "invalid configuration\n");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        click(&ctx, &mut a, "save");
        assert!(a.notice.is_some());
        assert!(!a.metrics.completion);
    }
}
#[test]
fn user_mount_native_three_views_practice_isolation_stop_reset_and_challenge() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        settle(&ctx, &mut a);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        let initial = session(&a, id).assignment.clone();
        let r = a.metrics.example_action.unwrap();
        tap(&ctx, &mut a, r);
        assert_eq!(session(&a, id).view, teaching::View::Example);
        let actions = session(&a, id).definition.example_actions.clone();
        for action in &actions {
            apply(&ctx, &mut a, action);
        }
        click(&ctx, &mut a, "check");
        finish(&ctx, &mut a);
        assert!(
            session(&a, id)
                .example
                .solved(&session(&a, id).definition.example)
        );
        assert_eq!(session(&a, id).assignment, initial);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        view(&mut a, id, teaching::View::Assignment);
        settle(&ctx, &mut a);
        a.start_user_mounts(i, false, false);
        paint(&ctx, &mut a, vec![]);
        assert!(a.user_mounts_run.is_some());
        click(&ctx, &mut a, "stop");
        assert!(a.user_mounts_run.is_none());
        settle(&ctx, &mut a);
        assert!(session(&a, id).assignment.result.is_none());
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        view(&mut a, id, teaching::View::Introduction);
        settle(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(a.metrics.user_mounts.editor.is_some());
        assert_eq!(session(&a, id).assignment, initial);
    }
}
#[test]
fn user_mount_native_controls_reflow_at_supported_scales_without_font_shrink() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        view(&mut a, id, teaching::View::Assignment);
        for scale in [1., 1.5, 2.] {
            for width in [760., 1500.] {
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 5000., vec![]);
                }
                let controls = &a.metrics.user_mounts.controls;
                assert!(controls.iter().any(|r| r.0 == "save"));
                for (key, rect, _) in controls {
                    assert!(rect.width() > 0. && rect.height() > 0., "{key}");
                    assert!(
                        rect.right() <= ctx.content_rect().right() + 2.,
                        "{id}/{scale}/{width}: {key} clipped at {rect:?}"
                    );
                }
                let h = controls.iter().find(|r| r.0 == "save").unwrap().1.height();
                let other = controls.iter().find(|r| r.0 == "mount").unwrap().1.height();
                assert!((h - other).abs() < 1.);
            }
        }
    }
}
#[test]
fn user_mount_editor_preserves_real_lines_and_native_keyboard_focus() {
    let (ctx, mut a, _) = opened(IDS[0]);
    view(&mut a, IDS[0], teaching::View::Assignment);
    settle(&ctx, &mut a);
    let source = session(&a, IDS[0]).assignment.draft.clone();
    fn find(shape: &egui::Shape, source: &str) -> Option<usize> {
        match shape {
            egui::Shape::Text(t) if t.galley.job.text == source => Some(t.galley.rows.len()),
            egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, source)),
            _ => None,
        }
    }
    for width in [760., 1500.] {
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 5000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let rows = out
            .shapes
            .iter()
            .find_map(|s| find(&s.shape, &source))
            .expect("actual editor text was painted");
        assert_eq!(
            rows,
            source.split('\n').count(),
            "visual wrapping must not look like fstab line breaks"
        );
    }
    type_editor(&ctx, &mut a, "# changed by native input\n");
    assert_eq!(
        session(&a, IDS[0]).assignment.draft,
        "# changed by native input\n"
    );
    assert_eq!(ctx.memory(|m| m.focused()), a.metrics.focus);
}
#[test]
fn user_mount_long_editor_starts_at_its_left_edge_and_reset_reentry_use_fresh_view_state() {
    let (ctx, mut a, i) = opened(IDS[0]);
    view(&mut a, IDS[0], teaching::View::Assignment);
    settle(&ctx, &mut a);
    let source = session(&a, IDS[0]).assignment.draft.clone();
    fn origin(s: &egui::Shape, text: &str) -> Option<egui::Pos2> {
        match s {
            egui::Shape::Text(t) if t.galley.job.text == text => Some(t.pos),
            egui::Shape::Vec(v) => v.iter().find_map(|s| origin(s, text)),
            _ => None,
        }
    }
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 5000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let p = output
        .shapes
        .iter()
        .find_map(|s| origin(&s.shape, &source))
        .expect("actual editor text");
    let viewport = a.metrics.editor_viewport.unwrap();
    assert!(
        p.x >= viewport.left() - 1.,
        "first characters must not be centered outside the viewport: {p:?} / {viewport:?}"
    );
    let (old, offset) = a.metrics.user_mounts.scroll.unwrap();
    assert_eq!(offset.x, 0.);
    let mut scroll = egui::scroll_area::State::load(&ctx, old).unwrap();
    scroll.offset.x = 90.;
    scroll.store(&ctx, old);
    settle(&ctx, &mut a);
    assert!(a.metrics.user_mounts.scroll.unwrap().1.x > 0.);
    a.reset_attempt(i);
    view(&mut a, IDS[0], teaching::View::Assignment);
    settle(&ctx, &mut a);
    let (fresh, offset) = a.metrics.user_mounts.scroll.unwrap();
    assert_ne!(old, fresh);
    assert_eq!(offset.x, 0.);
    assert_eq!(session(&a, IDS[0]).assignment.draft, source);
    a.selected = None;
    a.launch(i);
    settle(&ctx, &mut a);
    assert_ne!(fresh, a.metrics.user_mounts.scroll.unwrap().0);
    assert_eq!(a.metrics.user_mounts.scroll.unwrap().1.x, 0.);
}
