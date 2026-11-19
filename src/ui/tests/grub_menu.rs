use super::*;
use dlms_learning_lab::boot_entry_workspace as b;
const IDS: [&str; 2] = ["lpic1-grub-default-fallback", "lpic1-grub-recovery-entry"];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    (ctx, a, i)
}
fn view(a: &mut LabApp, id: &str, v: teaching::View) {
    let State::BootEntry { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = v;
    a.focus_input = true;
    a.lesson_scroll_reset = true;
}
fn definition(a: &LabApp, i: usize) -> b::Definition {
    let engine::Goal::BootEntry { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    (**definition).clone()
}
fn draw_menu(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1500., 1200., vec![]);
    }
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = a
        .metrics
        .boot_entry
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1;
    click(ctx, a, r);
    draw_menu(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let id = a.metrics.boot_entry.editor.unwrap().1;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        ctx,
        a,
        1500.,
        1200.,
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
    draw_menu(ctx, a);
}
#[test]
fn grub_three_screens_reflow_at_every_supported_scale_without_losing_header_or_controls() {
    let (ctx, mut a, _) = opened(IDS[0]);
    for id in IDS {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for light in [false, true] {
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
                for (w, h) in [(1500., 1100.), (760., 1000.)] {
                    for v in [
                        teaching::View::Introduction,
                        teaching::View::Example,
                        teaching::View::Assignment,
                    ] {
                        view(&mut a, id, v);
                        for _ in 0..3 {
                            frame(&ctx, &mut a, w, h, vec![]);
                        }
                        assert!(
                            a.metrics.back.is_some()
                                && a.metrics.reset.is_some()
                                && a.metrics.mode_learn.is_some()
                                && a.metrics.mode_challenge.is_some()
                        );
                        assert!(!a.metrics.guide);
                        for (key, r, _) in &a.metrics.boot_entry.controls {
                            assert!(r.is_finite() && r.is_positive());
                            assert!(
                                r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                                "{id}/{v:?}/{scale}/{w}/{key}: {r:?}"
                            );
                        }
                        if v != teaching::View::Introduction {
                            let viewport = a.metrics.boot_entry.editor.unwrap().0;
                            let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                            let row = ctx.fonts_mut(|f| f.row_height(&font));
                            assert!(
                                viewport.height()
                                    >= 15. * row + 2. * f32::from(controls::PAD_Y) - 0.1
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn grub_native_editor_distinguishes_unsupported_partial_correct_and_stale_results() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = definition(&a, i);
        view(&mut a, id, teaching::View::Assignment);
        draw_menu(&ctx, &mut a);
        assert!(a.metrics.boot_entry.result.is_none());
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        assert!(!a.metrics.completion);
        edit(&ctx, &mut a, "set default=saved\nsource /etc/passwd\n");
        assert!(a.metrics.boot_entry.result.is_none());
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        edit(&ctx, &mut a, &d.initial.reference);
        assert!(!a.metrics.completion);
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(true));
        assert!(
            a.metrics.completion
                && progress::current_solved(&a.progress.attempts[id], &a.scenarios[i])
        );
        let bad = if id == IDS[0] {
            d.initial
                .reference
                .replacen("set default=spare", "set default=maintenance", 1)
        } else {
            d.initial.reference.replace(
                "systemd.unit=rescue.target",
                "systemd.unit=multi-user.target",
            )
        };
        edit(&ctx, &mut a, &bad);
        assert!(a.metrics.boot_entry.result.is_none());
        assert!(!a.metrics.completion);
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        edit(&ctx, &mut a, &d.initial.reference);
        tap(&ctx, &mut a, "test");
        assert!(a.metrics.completion);
        let editor = a.metrics.boot_entry.editor.unwrap().1;
        ctx.memory_mut(|m| m.request_focus(editor));
        frame(&ctx, &mut a, 1500., 1200., vec![key(egui::Key::Tab, false)]);
        assert!(
            !ctx.memory(|m| m.has_focus(editor)),
            "Tab must leave the multiline editor"
        );
    }
}
#[test]
fn grub_worked_practice_edits_are_inert_assignment_is_separate_and_challenge_hides_solutions() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = definition(&a, i);
        view(&mut a, id, teaching::View::Example);
        draw_menu(&ctx, &mut a);
        let assignment_before = match &a.progress.attempts[id].state {
            State::BootEntry { simulation } => simulation.assignment.clone(),
            _ => panic!(),
        };
        edit(&ctx, &mut a, &d.example.reference);
        assert!(a.metrics.boot_entry.result.is_none());
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(true));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let State::BootEntry { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment, assignment_before);
        let practice_before = simulation.practice.clone();
        let r = a.metrics.mode_challenge.unwrap();
        click(&ctx, &mut a, r);
        draw_menu(&ctx, &mut a);
        assert!(a.progress.attempts[id].challenge);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(
            a.metrics
                .boot_entry
                .controls
                .iter()
                .all(|r| !matches!(r.0.as_str(), "fill" | "try" | "example" | "introduction"))
        );
        let State::BootEntry { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice, practice_before);
        assert_eq!(simulation.assignment, assignment_before);
        assert!(!a.metrics.completion);
    }
}
#[test]
fn grub_selected_reset_and_back_use_the_shared_header_without_touching_the_other_task() {
    let (ctx, mut a, _) = opened(IDS[0]);
    let i = a.selected.unwrap();
    let d = definition(&a, i);
    view(&mut a, IDS[0], teaching::View::Assignment);
    draw_menu(&ctx, &mut a);
    edit(&ctx, &mut a, &d.initial.reference);
    tap(&ctx, &mut a, "test");
    assert!(a.metrics.completion);
    let first = serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap();
    let j = a.scenarios.iter().position(|s| s.id == IDS[1]).unwrap();
    a.launch(j);
    view(&mut a, IDS[1], teaching::View::Assignment);
    draw_menu(&ctx, &mut a);
    edit(&ctx, &mut a, "set default='unfinished");
    let reset = a.metrics.reset.unwrap();
    click(&ctx, &mut a, reset);
    draw_menu(&ctx, &mut a);
    assert!(a.reset_pending);
    let r = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, r);
    draw_menu(&ctx, &mut a);
    assert!(!a.reset_pending);
    assert_eq!(a.progress.attempts[IDS[1]].state, a.scenarios[j].initial);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap(),
        first
    );
    let r = a.metrics.back.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.selected.is_none());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[IDS[0]]).unwrap(),
        first
    );
}
