//! Synthetic native widget checks, not independent novice or physical-input acceptance.
use super::debug_recovery::{paint, settle, tap};
use super::*;
use dlms_learning_lab::{samba_share_source as binding, samba_share_workspace as work};
const IDS: [&str; 3] = [
    "lpic2-smb-identity-access",
    "lpic2-smb-team-policy",
    "lpic2-smb-change-handover",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::SambaShare { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    for _ in 0..20 {
        settle(ctx, a);
        let (_, r, clip) = a
            .metrics
            .samba
            .controls
            .iter()
            .find(|r| r.0 == key)
            .unwrap_or_else(|| panic!("Missing {key}"))
            .clone();
        let clip = clip.intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            tap(ctx, a, r);
            return;
        }
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Cannot reveal {key}")
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    click(ctx, a, key);
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
            if text.is_empty() {
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }
            } else {
                egui::Event::Text(text.into())
            },
        ],
    );
    settle(ctx, a);
}
#[test]
fn sharing_actual_controls_keep_practice_separate_and_require_account_terminal_and_policy_repairs()
{
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let d = binding::source(&s).unwrap();
        let initial = session(&a).assignment.clone();
        click(&ctx, &mut a, "example");
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert!(
            a.metrics
                .samba
                .controls
                .iter()
                .any(|r| r.0 == "failed-case")
        );
        if !d.example.editable_accounts.is_empty() {
            for user in &d.example.editable_accounts {
                if !d
                    .example
                    .initial
                    .accounts
                    .iter()
                    .any(|u| u.user == user.id && u.enabled)
                {
                    click(&ctx, &mut a, &format!("account:{}", user.id));
                }
            }
            if id.ends_with("handover") {
                click(&ctx, &mut a, "terminal-help");
            }
            replace(&ctx, &mut a, "terminal", "chmod g+rx /practice/notes");
            click(&ctx, &mut a, "command-run");
        }
        if !id.ends_with("access") {
            click(&ctx, &mut a, "worked-config");
            assert_eq!(
                session(&a).practice.editor,
                d.example.reference.saved_config().unwrap()
            );
            assert_ne!(
                session(&a).practice.world.saved_config().unwrap(),
                session(&a).practice.editor
            );
            click(&ctx, &mut a, "save");
            click(&ctx, &mut a, "apply");
        }
        click(&ctx, &mut a, "trial-run");
        assert!(
            session(&a)
                .practice
                .last_trial
                .as_ref()
                .unwrap()
                .result
                .is_ok()
        );
        click(&ctx, &mut a, "check");
        assert!(
            session(&a).practice.solved(&d.example),
            "{id} {:?}",
            session(&a).practice.checks(&d.example)
        );
        assert_eq!(session(&a).assignment, initial);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        click(&ctx, &mut a, "assignment");
        assert!(
            !a.metrics
                .samba
                .controls
                .iter()
                .any(|r| r.0 == "worked-config")
        );
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        // Source-bound dev method sets up a previously successful independent task;
        // the following user editor input must clear stale feedback immediately.
        for e in &s.solution {
            engine::execute_for_goal(
                &mut a.progress.attempts.get_mut(id).unwrap().state,
                &s.goal,
                e,
            )
            .unwrap();
        }
        a.completion_index.forget(id);
        settle(&ctx, &mut a);
        assert!(a.metrics.completion);
        if id.ends_with("access") {
            click(&ctx, &mut a, "account:mira");
        } else {
            replace(&ctx, &mut a, "config", "[team]\npath = relative\n");
        }
        assert!(!session(&a).assignment.checked);
        assert!(!a.metrics.completion);
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        let r = a.metrics.reset.unwrap();
        tap(&ctx, &mut a, r);
        let r = a.metrics.confirm_reset.unwrap();
        tap(&ctx, &mut a, r);
        assert_eq!(*session(&a), d.new_session().unwrap());
    }
}
#[test]
fn trial_path_typing_clear_and_invalid_paths_remain_editable_without_fabricated_output() {
    let (ctx, mut a, _) = opened(IDS[1]);
    click(&ctx, &mut a, "assignment");
    replace(&ctx, &mut a, "path", "");
    assert!(
        matches!(&session(&a).assignment.request.operation,work::Operation::Read{path}if path.is_empty())
    );
    click(&ctx, &mut a, "trial-run");
    assert!(session(&a).assignment.last_trial.is_none());
    assert!(
        a.notice
            .as_ref()
            .unwrap()
            .contains("inside the selected share")
    );
    replace(&ctx, &mut a, "path", "../outside");
    assert!(
        matches!(&session(&a).assignment.request.operation,work::Operation::Read{path}if path=="../outside")
    );
    click(&ctx, &mut a, "trial-run");
    assert!(session(&a).assignment.last_trial.is_none());
}
#[test]
fn configuration_save_apply_and_aligned_fields_reflow_at_supported_scales_without_challenge_solutions()
 {
    for scale in [1., 1.5, 2.] {
        for width in [950., 1500.] {
            let (ctx, mut a, i) = opened(IDS[1]);
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            click(&ctx, &mut a, "assignment");
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 5000., vec![]);
            }
            for (_, rect, _, _) in &a.metrics.samba.fields {
                assert!(
                    (rect.height() - {
                        let style = ctx.style();
                        ctx.fonts_mut(|f| {
                            [
                                egui::TextStyle::Body,
                                egui::TextStyle::Button,
                                egui::TextStyle::Monospace,
                            ]
                            .iter()
                            .map(|s| f.row_height(&s.resolve(&style)))
                            .fold(0_f32, f32::max)
                        })
                        .ceil()
                            + 2. * controls::PAD_Y as f32
                    })
                    .abs()
                        < 3.
                );
            }
            assert!(a.metrics.samba.editor.is_some());
            assert!(
                !a.metrics
                    .samba
                    .controls
                    .iter()
                    .any(|r| r.0 == "worked-config")
            );
            a.progress.attempts.get_mut(IDS[1]).unwrap().challenge = true;
            settle(&ctx, &mut a);
            assert!(
                !a.metrics
                    .samba
                    .controls
                    .iter()
                    .any(|r| r.0 == "introduction" || r.0 == "worked-config")
            );
            assert!(!progress::current_solved(
                &a.progress.attempts[IDS[1]],
                &a.scenarios[i]
            ));
        }
    }
}
#[test]
fn sharing_keyboard_tab_edit_and_reflow_preserve_native_focus_and_selection() {
    let (ctx, mut a, _) = opened(IDS[1]);
    click(&ctx, &mut a, "assignment");
    click(&ctx, &mut a, "path");
    let id = a
        .metrics
        .samba
        .fields
        .iter()
        .find(|r| r.0 == "path")
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    for width in [950., 1500.] {
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 5000., vec![]);
            }
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        }
    }
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    assert!(ctx.memory(|m| m.focused()).is_some());
}
#[test]
fn ordinary_desktop_uses_two_columns_and_unequal_account_states_do_not_shift_buttons() {
    let (ctx, mut a, _) = opened(IDS[0]);
    click(&ctx, &mut a, "example");
    for _ in 0..3 {
        frame(&ctx, &mut a, 1920., 1080., vec![]);
    }
    assert!(
        a.metrics.samba.split,
        "The native content cap must permit the wide layout at100percent"
    );
    let button = |id: &str| {
        a.metrics
            .samba
            .controls
            .iter()
            .find(|r| r.0 == id)
            .unwrap()
            .1
    };
    assert!((button("account:riley").left() - button("account:sam").left()).abs() < 1.5);
    for _ in 0..3 {
        frame(&ctx, &mut a, 950., 1080., vec![]);
    }
    assert!(!a.metrics.samba.split);
}
#[test]
fn sharing_parent_gutter_stays_allocated_across_fresh_views_zoom_and_reflow() {
    let (ctx, mut a, _) = opened(IDS[2]);
    click(&ctx, &mut a, "assignment");
    for (scale, width) in [(1., 1600.), (1.5, 950.), (2., 950.), (1., 1600.)] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for _ in 0..3 {
            frame(&ctx, &mut a, width, 929., vec![]);
        }
        let view = a.metrics.main_viewport.unwrap();
        let region = a.metrics.main_scroll_region.unwrap();
        assert!(
            (region.right() - view.right()) * ctx.pixels_per_point() > 1.,
            "The Samba-only solid parent gutter must remain usable for actual scrolling"
        );
    }
}
