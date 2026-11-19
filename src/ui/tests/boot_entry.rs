use super::*;
use dlms_learning_lab::boot_entry_workspace as b;
const IDS: [&str; 3] = [
    "lpic1-boot-root-identity",
    "lpic1-boot-initramfs-match",
    "lpic1-boot-recovery-handoff",
];
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
}
fn definition(a: &LabApp, i: usize) -> b::Definition {
    let engine::Goal::BootEntry { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    (**definition).clone()
}
fn click_boot(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = a
        .metrics
        .boot_entry
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1;
    click(ctx, a, r);
    frame(ctx, a, 1500., 1200., vec![]);
}
fn type_boot(ctx: &egui::Context, a: &mut LabApp, text: &str) {
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
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Paste(text.into()),
        ],
    );
    frame(ctx, a, 1500., 1200., vec![]);
}
#[test]
fn boot_entry_three_screens_reflow_at_all_supported_scales_in_both_themes() {
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
                for (w, h) in [(1500., 1000.), (760., 900.)] {
                    for v in [
                        teaching::View::Introduction,
                        teaching::View::Example,
                        teaching::View::Assignment,
                    ] {
                        view(&mut a, id, v);
                        for _ in 0..3 {
                            frame(&ctx, &mut a, w, h, vec![]);
                        }
                        for (key, r, _) in &a.metrics.boot_entry.controls {
                            assert!(
                                r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                                "{id} {v:?} {scale} {w} {key} {r:?}"
                            );
                            assert!(r.is_finite() && r.height() > 0.);
                        }
                        assert!(a.metrics.back.is_some() && a.metrics.reset.is_some());
                        assert!(
                            a.metrics.mode_learn.is_some() && a.metrics.mode_challenge.is_some()
                        );
                        assert!(!a.metrics.guide);
                        assert!(!a.metrics.completion);
                    }
                }
            }
        }
    }
}
#[test]
fn boot_entry_native_editor_keyboard_check_retry_and_selected_reset_keep_real_behavior() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = definition(&a, i);
        view(&mut a, id, teaching::View::Assignment);
        frame(&ctx, &mut a, 1500., 1200., vec![]);
        assert!(ctx.memory(|m| m.has_focus(a.metrics.boot_entry.editor.unwrap().1)));
        assert!(a.metrics.boot_entry.result.is_none());
        click_boot(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        type_boot(&ctx, &mut a, &d.initial.reference);
        assert!(a.metrics.boot_entry.result.is_none());
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        click_boot(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(true));
        assert!(
            a.metrics.boot_entry.side_by_side,
            "A normal1500px desktop must use the editor/facts split"
        );
        let test = a
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|c| c.0 == "test")
            .unwrap()
            .1;
        let result = a
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|c| c.0 == "result")
            .unwrap()
            .1;
        assert!(
            (result.left() - test.left()).abs() < 2. && result.top() < test.bottom() + 100.,
            "Feedback belongs immediately after Test entry, not after the inventory"
        );
        assert!(a.metrics.completion);
        let viewport = a.metrics.boot_entry.editor.unwrap().0;
        let font = egui::TextStyle::Monospace.resolve(&ctx.style());
        for line in d.initial.reference.lines() {
            let width = ctx.fonts_mut(|f| {
                f.layout_no_wrap(line.into(), font.clone(), egui::Color32::WHITE)
                    .size()
                    .x
            });
            assert!(
                width + 2. * f32::from(controls::PAD_X) <= viewport.width(),
                "Normal-width boot lines must fit without hiding kernel arguments"
            );
        }

        type_boot(&ctx, &mut a, "linux=/bad");
        let editor = a.metrics.boot_entry.editor.unwrap().1;
        frame(&ctx, &mut a, 1500., 1200., vec![key(egui::Key::Tab, false)]);
        assert!(
            !ctx.memory(|m| m.has_focus(editor)),
            "Tab must leave the code editor for keyboard navigation"
        );

        assert!(a.metrics.boot_entry.result.is_none());
        assert!(!a.metrics.completion);
        click_boot(&ctx, &mut a, "test");
        assert_eq!(a.metrics.boot_entry.result, Some(false));
        let other = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-permissions")
            .unwrap();
        a.launch(other);
        let retained = serde_json::to_value(&a.progress.attempts["lpic1-permissions"]).unwrap();
        a.selected = Some(i);
        a.reset_attempt(i);
        assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
        assert_eq!(
            serde_json::to_value(&a.progress.attempts["lpic1-permissions"]).unwrap(),
            retained
        );
    }
}
#[test]
fn boot_entry_practice_is_separate_and_challenge_has_no_unsolicited_worked_answer() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let d = definition(&a, i);
    let initial = serde_json::to_value(&a.progress.attempts[IDS[0]].state).unwrap();
    view(&mut a, IDS[0], teaching::View::Example);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    type_boot(&ctx, &mut a, &d.example.reference);
    click_boot(&ctx, &mut a, "test");
    assert_eq!(a.metrics.boot_entry.result, Some(true));
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    let State::BootEntry { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(
        serde_json::to_value(&simulation.assignment).unwrap(),
        initial["simulation"]["assignment"]
    );
    let practice = simulation.practice.clone();
    a.progress.attempts.get_mut(IDS[0]).unwrap().challenge = true;
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(
        a.metrics
            .boot_entry
            .controls
            .iter()
            .all(|r| !matches!(r.0.as_str(), "fill" | "example" | "introduction"))
    );
    let State::BootEntry { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.practice, practice);
    assert_eq!(simulation.assignment.source, d.initial.starter);
}

#[test]
fn boot_entry_shared_header_changes_mode_returns_to_library_and_confirms_only_selected_reset() {
    let (ctx, mut a, i) = opened(IDS[0]);
    view(&mut a, IDS[0], teaching::View::Example);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    let d = definition(&a, i);
    type_boot(&ctx, &mut a, &d.example.reference);
    let retained = serde_json::to_value(&a.progress.attempts[IDS[0]].state).unwrap();
    let r = a.metrics.mode_challenge.unwrap();
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    assert!(a.progress.attempts[IDS[0]].challenge);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[IDS[0]].state).unwrap(),
        retained
    );
    let r = a.metrics.mode_learn.unwrap();
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    assert!(!a.progress.attempts[IDS[0]].challenge);
    let r = a.metrics.back.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.selected.is_none());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[IDS[0]].state).unwrap(),
        retained
    );
    a.launch(i);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    let r = a.metrics.reset.unwrap();
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    assert!(a.reset_pending);
    let r = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1500., 1200., vec![]);
    assert!(!a.reset_pending);
    assert_eq!(a.progress.attempts[IDS[0]].state, a.scenarios[i].initial);
}

#[test]
fn boot_capture_targets_all_authored_text_without_requiring_empty_editor_rows() {
    let (ctx, mut a, i) = opened(IDS[2]);
    view(&mut a, IDS[2], teaching::View::Assignment);
    let d = definition(&a, i);
    let State::BootEntry { simulation } = &mut a.progress.attempts.get_mut(IDS[2]).unwrap().state else {
        panic!()
    };
    simulation.assignment.edit(d.initial.reference).unwrap();
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for (w, h) in [(1500., 1200.), (1268., 1308.)] {
            for _ in 0..3 { frame(&ctx, &mut a, w, h, vec![]); }
            let viewport = a.metrics.boot_entry.editor.unwrap().0;
            let text = a.metrics.boot_entry.editor_text.unwrap();
            assert!(text.is_positive() && text.is_finite());
            assert!(viewport.contains_rect(text), "{scale} {w}: {text:?} outside {viewport:?}");
            assert!(text.height() < viewport.height());
            // The empty lower rows can be below the window while every authored
            // line is visible. Clicks still use the unchanged full editor rect.
            let visible = egui::Rect::from_min_max(viewport.min, egui::pos2(viewport.right(), text.bottom() + 1.));
            assert!(smoke::smoke_control_visible(text, visible, ctx.pixels_per_point()));
            assert!(!smoke::smoke_control_visible(viewport, visible, ctx.pixels_per_point()));
        }
    }
}
