use super::*;
use dlms_learning_lab::boot_entry_workspace as b;
const IDS: [&str; 2] = ["lpic1-grub-persistent-default", "lpic1-grub-argument-scope"];
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
fn persistent_settings_native_edit_generate_test_outdated_and_retry_use_actual_controls() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = definition(&a, i);
        view(&mut a, id, teaching::View::Assignment);
        draw_menu(&ctx, &mut a);
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        edit(&ctx, &mut a, &d.initial.reference);
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        assert!(!a.metrics.completion);
        tap(&ctx, &mut a, "generate");
        assert!(a.metrics.boot_entry.result.is_none());
        tap(&ctx, &mut a, "generated-output");
        let output = a
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|r| r.0 == "generated-text")
            .unwrap()
            .1;
        assert!(output.right() <= ctx.content_rect().right());
        tap(&ctx, &mut a, "generated-output");
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(true));
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        edit(
            &ctx,
            &mut a,
            &d.initial
                .reference
                .replace("GRUB_TIMEOUT=", "GRUB_TIMEOUT=0\n# old "),
        );
        assert!(a.metrics.boot_entry.result.is_none());
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        tap(&ctx, &mut a, "generate");
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        let failure = a
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|r| r.0 == "failed-requirement")
            .unwrap()
            .1;
        let trace = a
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|r| r.0 == "boot-stage")
            .unwrap()
            .1;
        assert!(
            failure.top() < trace.top(),
            "failed task requirements must precede a successful selected-entry trace"
        );
        edit(&ctx, &mut a, "GRUB_DEFAULT=$(id)\n");
        tap(&ctx, &mut a, "generate");
        assert!(a.notice.as_ref().is_some_and(|s| s.contains("Expansion")));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        edit(&ctx, &mut a, &d.initial.reference);
        tap(&ctx, &mut a, "generate");
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(true));
        frame(
            &ctx,
            &mut a,
            1500.,
            1200.,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(ctx.memory(|m| m.focused()).is_some());
    }
}
#[test]
fn generated_practice_drafts_remain_separate_challenge_hides_answers_and_reset_is_selected() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = definition(&a, i);
        view(&mut a, id, teaching::View::Example);
        draw_menu(&ctx, &mut a);
        edit(&ctx, &mut a, &d.example.reference);
        tap(&ctx, &mut a, "generate");
        tap(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(true));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let State::BootEntry { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.source, d.initial.starter);
        assert!(simulation.assignment.generated.is_none());
        let saved_practice = simulation.practice.clone();
        view(&mut a, id, teaching::View::Assignment);
        draw_menu(&ctx, &mut a);
        edit(&ctx, &mut a, &d.initial.reference);
        tap(&ctx, &mut a, "generate");
        tap(&ctx, &mut a, "test");
        let other = if id == IDS[0] { IDS[1] } else { IDS[0] };
        let j = a.scenarios.iter().position(|s| s.id == other).unwrap();
        a.launch(j);
        view(&mut a, other, teaching::View::Assignment);
        draw_menu(&ctx, &mut a);
        let untouched = serde_json::to_value(&a.progress.attempts[other]).unwrap();
        a.launch(i);
        view(&mut a, id, teaching::View::Assignment);
        draw_menu(&ctx, &mut a);
        let challenge = a.metrics.mode_challenge.unwrap();
        click(&ctx, &mut a, challenge);
        draw_menu(&ctx, &mut a);
        assert!(
            !a.metrics
                .boot_entry
                .controls
                .iter()
                .any(|c| matches!(c.0.as_str(), "fill" | "worked" | "example"))
        );
        assert!(
            a.metrics
                .boot_entry
                .controls
                .iter()
                .any(|c| c.0 == "generate")
        );
        let reset = a.metrics.reset.unwrap();
        click(&ctx, &mut a, reset);
        draw_menu(&ctx, &mut a);
        let confirm = a.metrics.confirm_reset.unwrap();
        click(&ctx, &mut a, confirm);
        draw_menu(&ctx, &mut a);
        assert_eq!(
            serde_json::to_value(&a.progress.attempts[other]).unwrap(),
            untouched
        );
        let State::BootEntry { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.source, d.initial.starter);
        assert!(simulation.assignment.generated.is_none());
        assert!(simulation.practice.generated.is_none());
        assert_ne!(simulation.practice, saved_practice);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn persistent_editor_and_related_buttons_reflow_at_supported_sizes_in_light_and_dark() {
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
                        assert!(a.metrics.back.is_some() && a.metrics.reset.is_some());
                        for (key, r, _) in &a.metrics.boot_entry.controls {
                            assert!(r.is_finite() && r.is_positive(), "{key}");
                            assert!(
                                r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                                "{id}/{v:?}/{scale}/{w}/{key} {r:?}"
                            );
                        }
                        if v != teaching::View::Introduction {
                            assert!(
                                a.metrics
                                    .boot_entry
                                    .controls
                                    .iter()
                                    .any(|c| c.0 == "generate")
                            );
                            let r = a.metrics.boot_entry.editor.unwrap().0;
                            let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                            let row = ctx.fonts_mut(|f| f.row_height(&font));
                            assert!(r.height() >= 6. * row + 2. * f32::from(controls::PAD_Y) - 0.1);
                        }
                    }
                }
            }
        }
    }
}
