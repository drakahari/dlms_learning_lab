//! Presentation regressions use real core outcomes and normal bundled activities.
use super::*;
use dlms_learning_lab::nfs;

fn text(shape: &egui::epaint::Shape, out: &mut String) {
    match shape {
        egui::epaint::Shape::Text(t) => {
            out.push_str(&t.galley.job.text);
            out.push('\n');
        }
        egui::epaint::Shape::Vec(v) => v.iter().for_each(|s| text(s, out)),
        _ => {}
    }
}
fn capture(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) -> String {
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 2200.),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut painted = String::new();
    for s in out.shapes {
        text(&s.shape, &mut painted);
    }
    painted
}
fn script(source: &str) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "shell-command-options")
        .unwrap();
    a.launch(i);
    a.set_script_view(i, teaching::View::Assignment);
    engine::execute(
        &mut a
            .progress
            .attempts
            .get_mut("shell-command-options")
            .unwrap()
            .state,
        source,
    )
    .unwrap();
    for _ in 0..4 {
        capture(&ctx, &mut a, vec![]);
    }
    let r = a.metrics.check.unwrap();
    for pressed in [true, false] {
        capture(
            &ctx,
            &mut a,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    for _ in 0..2000 {
        capture(&ctx, &mut a, vec![]);
        if a.script_job.is_none()
            && matches!(
                &a.progress.attempts["shell-command-options"].state,
                State::Script { run: Some(_), .. }
            )
        {
            return (ctx, a, i);
        }
    }
    panic!("Actual Run did not finish within its finite UI frame budget");
}
#[test]
fn script_overall_failure_and_selected_passing_case_are_explicit_without_changing_results() {
    let (ctx, mut a, i) = script("printf 'A\\nb\\nb\\nz\\n'\n");
    let scenario = a.scenarios[i].clone();
    let checks = engine::checks(&a.progress.attempts[&scenario.id].state, &scenario.goal);
    assert!(checks[0].0 && checks.iter().skip(1).any(|c| !c.0));
    assert_eq!(
        a.script_case,
        checks.iter().position(|c| !c.0).unwrap(),
        "Run initially selects an actual failure"
    );
    let first_failure = capture(&ctx, &mut a, vec![]);
    assert!(first_failure.contains("Case 2/12 · Fail · Run: 11 fail"));
    assert!(first_failure.contains("Fail · Unique entries from an option-looking name"));
    assert!(first_failure.contains("Overall result · Needs another edit · 11 of 12"));
    let before = serde_json::to_value(&a.progress).unwrap();
    let row = a
        .metrics
        .script_case_rows
        .iter()
        .find(|r| r.0 == 0)
        .unwrap()
        .1;
    for pressed in [true, false] {
        capture(
            &ctx,
            &mut a,
            vec![
                egui::Event::PointerMoved(row.center()),
                egui::Event::PointerButton {
                    pos: row.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    let passing = capture(&ctx, &mut a, vec![]);
    assert_eq!(
        a.script_case, 0,
        "A real result-row click selects that input set"
    );
    assert!(passing.contains("Case 1/12 · Pass · Run: 11 fail"));
    assert!(passing.contains("Case 1 · Pass · Ascending with every entry"));
    assert!(passing.contains("Overall result · Needs another edit · 11 of 12"));
    assert!(!passing.contains("Behavior differs"));
    assert!(
        !passing.contains("Expected:"),
        "A passing case must not be presented as a difference"
    );
    for key in [egui::Key::Tab, egui::Key::Enter] {
        for pressed in [true, false] {
            capture(
                &ctx,
                &mut a,
                vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                }],
            );
        }
    }
    let keyboard = capture(&ctx, &mut a, vec![]);
    assert_eq!(
        a.script_case, 1,
        "Tab then Enter must inspect the next result through real native keyboard events"
    );
    assert!(keyboard.contains("Case 2/12 · Fail · Run: 11 fail"));
    assert_eq!(
        serde_json::to_value(&a.progress).unwrap(),
        before,
        "Input inspection changes no record, draft or mastery"
    );
    assert_eq!(
        engine::checks(&a.progress.attempts[&scenario.id].state, &scenario.goal),
        checks
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&scenario.id],
        &scenario
    ));
}
#[test]
fn script_equal_stdout_but_wrong_files_is_a_named_failure_not_an_equal_output_diff() {
    let (ctx, mut a, i) =
        script("printf 'A\\nb\\nb\\nz\\n'\nprintf changed > /work/unrequested.txt\n");
    a.script_case = 0;
    let painted = capture(&ctx, &mut a, vec![]);
    assert!(painted.contains("Case 1/12 · Fail · Run: 12 fail"));
    assert!(painted.contains("Fail · Ascending with every entry · Virtual files differ"));
    let output = a.metrics.output.unwrap();
    capture(
        &ctx,
        &mut a,
        vec![
            egui::Event::PointerMoved(output.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., -3000.),
                modifiers: Default::default(),
            },
        ],
    );
    for _ in 0..20 {
        capture(&ctx, &mut a, vec![]);
    }
    let details = capture(&ctx, &mut a, vec![]);
    assert!(
        details.contains("The virtual files differ."),
        "Actual output scrolling must reveal file-specific advice: {details}"
    );
    assert!(
        !details.contains("Expected:"),
        "Scrolling must not reveal a spurious equal-output difference"
    );
    assert!(
        !painted.contains("Expected:"),
        "Identical stdout is not an output mismatch"
    );
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
}
#[test]
fn script_syntax_unsupported_and_success_keep_their_existing_grade_and_case_identity() {
    for (source, diagnostic) in [
        ("printf 'unfinished", "Syntax error"),
        ("IFS=: read -r row\n", "Unsupported syntax"),
    ] {
        let (ctx, mut a, i) = script(source);
        let painted = capture(&ctx, &mut a, vec![]);
        assert!(painted.contains(diagnostic), "{painted}");
        assert!(painted.contains("Case 1/12"));
        assert!(painted.contains("Overall result · Needs another edit · 12 of 12"));
        assert!(!engine::solved(
            &a.progress.attempts[&a.scenarios[i].id].state,
            &a.scenarios[i].goal
        ));
    }
    let (_, a) = app();
    let reference = a
        .scenarios
        .iter()
        .find(|s| s.id == "shell-command-options")
        .unwrap()
        .solution[0]
        .clone();
    let (ctx, mut a, i) = script(&reference);
    let painted = capture(&ctx, &mut a, vec![]);
    assert!(painted.contains("Overall result · Correct solution · All input sets pass."));
    assert!(painted.contains("Case 1/12 · Pass · Run: all pass"));
    assert!(engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    // A real editor event invalidates the run rather than retaining a stale Pass.
    let editor = a.metrics.focus.unwrap();
    ctx.memory_mut(|m| m.request_focus(editor));
    capture(&ctx, &mut a, vec![egui::Event::Text("broken".into())]);
    let after = capture(&ctx, &mut a, vec![]);
    assert!(!after.contains("All input sets pass."));
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
#[test]
fn nfs_save_rows_and_outer_editor_height_are_stable_across_content_and_reflow() {
    let mut saw_vertical = false;
    let mut saw_columns = false;
    let mut saw_stacking = false;
    for theme_id in ["dlms-dark", "dlms-light"] {
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for width in [1920., 946.] {
                let (ctx, mut a) = app();
                a.scale = scale;
                a.theme_id = theme_id.into();
                theme::apply(&ctx, theme_id);
                let i = a
                    .scenarios
                    .iter()
                    .position(|s| s.id == "lpic2-nfs-reference")
                    .unwrap();
                a.launch(i);
                a.progress
                    .attempts
                    .get_mut("lpic2-nfs-reference")
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .nfs_learning
                    .as_mut()
                    .unwrap()
                    .view = teaching::View::Assignment;
                // Establish fonts and initial zoom before measuring content changes.
                // All subsequent short/long scrollbar transition frames remain checked.
                for _ in 0..8 {
                    frame(&ctx, &mut a, width, 1080., vec![]);
                }
                let mut previous: Option<(f32, f32, f32, f32)> = None;
                // None/one/both axes, then reverse which document requires them.
                for (long_exports, long_fstab) in [
                    (false, false),
                    (true, false),
                    (false, true),
                    (true, true),
                    (false, false),
                ] {
                    let a0 = a.progress.attempts.get_mut("lpic2-nfs-reference").unwrap();
                    let State::Nfs { simulation } = &mut a0.state else {
                        panic!()
                    };
                    let server = simulation.inventory().servers[0].id.clone();
                    let client = simulation.inventory().clients[0].id.clone();
                    let body = |long| {
                        if long {
                            format!(
                                "# {}\n{}",
                                "long-document-token".repeat(25),
                                "# another line\n".repeat(30)
                            )
                        } else {
                            "# short\n".into()
                        }
                    };
                    simulation
                        .apply(&nfs::Action::SetExportsDraft {
                            server,
                            text: body(long_exports),
                        })
                        .unwrap();
                    simulation
                        .apply(&nfs::Action::SetFstabDraft {
                            client,
                            text: body(long_fstab),
                        })
                        .unwrap();
                    let before = serde_json::to_value(&a.progress).unwrap();
                    // Include transition frames, not only the settled scrollbar state.
                    for _ in 0..8 {
                        frame(&ctx, &mut a, width, 1080., vec![]);
                        let docs = &a.metrics.nfs.documents;
                        assert_eq!(docs.len(), 2);
                        for (_, outer, inner, content) in docs {
                            saw_vertical |= content.y > inner.height() + 1.;
                            assert!(outer.height() >= inner.height());
                        }
                        let exports = a
                            .metrics
                            .nfs
                            .controls
                            .iter()
                            .find(|r| r.0 == "save-exports")
                            .unwrap()
                            .1;
                        let fstab = a
                            .metrics
                            .nfs
                            .controls
                            .iter()
                            .find(|r| r.0 == "save-fstab")
                            .unwrap()
                            .1;
                        if a.metrics.nfs.columns {
                            saw_columns = true;
                            assert!(
                                (exports.top() - fstab.top()).abs() < 0.1,
                                "Save baselines differ: {theme_id}/{scale}/{width} {exports:?}/{fstab:?}; docs={docs:?}; controls={:?}",
                                a.metrics.nfs.controls
                            );
                        } else {
                            saw_stacking = true;
                        }
                        let signature = (
                            docs[0].1.height(),
                            docs[1].1.height(),
                            exports.top(),
                            fstab.top(),
                        );
                        if let Some((h0, h1, y0, y1)) = previous {
                            for (old, new) in [
                                (h0, signature.0),
                                (h1, signature.1),
                                (y0, signature.2),
                                (y1, signature.3),
                            ] {
                                assert!(
                                    (old - new).abs() < 0.1,
                                    "Scrollbar changed outer geometry: {theme_id}/{scale}/{width}: {previous:?} -> {signature:?}"
                                );
                            }
                        }
                        previous = Some(signature);
                    }
                    assert_eq!(
                        serde_json::to_value(&a.progress).unwrap(),
                        before,
                        "Reflow must not change any progress/draft/receipt"
                    );
                }
            }
        }
    }
    assert!(
        saw_vertical && saw_columns && saw_stacking,
        "The test must exercise actual overflow and both responsive layouts"
    );
}

#[test]
fn fixed_viewport_contains_real_horizontal_vertical_and_combined_scrollbar_transitions() {
    let mut horizontal = false;
    let mut vertical = false;
    for theme_id in ["dlms-dark", "dlms-light"] {
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for width in [1920., 946.] {
                let ctx = egui::Context::default();
                theme::apply(&ctx, theme_id);
                ctx.set_zoom_factor(scale);
                let mut previous: Option<(f32, f32)> = None;
                for state in 0..6 {
                    // First state initializes fonts/zoom; the last five exercise
                    // no bars, one axis, the other, both, and removal.
                    let overflow = [
                        (false, false),
                        (false, false),
                        (true, false),
                        (false, true),
                        (true, true),
                        (false, false),
                    ][state];
                    for _ in 0..8 {
                        let mut measured = None;
                        let _ = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width / scale, 1080. / scale),
                                )),
                                ..Default::default()
                            },
                            |ctx| {
                                controls::configure(ctx);
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    let height = ui.text_style_height(&egui::TextStyle::Monospace)
                                        * 3.
                                        + 2. * f32::from(controls::PAD_Y);
                                    let size = egui::vec2(
                                        ui.available_width(),
                                        height + ui.spacing().scroll.allocated_width(),
                                    );
                                    let (outer, scroll) =
                                        controls::fixed_viewport(ui, size, |ui| {
                                            egui::ScrollArea::both()
                                                .id_salt("regression-scroll")
                                                .max_height(size.y)
                                                .min_scrolled_height(height)
                                                .auto_shrink([false, false])
                                                .show(ui, |ui| {
                                                    ui.allocate_space(egui::vec2(
                                                        if overflow.0 { 2000. } else { 8. },
                                                        if overflow.1 { 1000. } else { 8. },
                                                    ));
                                                })
                                        });
                                    horizontal |=
                                        scroll.content_size.x > scroll.inner_rect.width() + 1.;
                                    vertical |=
                                        scroll.content_size.y > scroll.inner_rect.height() + 1.;
                                    assert!(
                                        outer.expand(0.1).contains_rect(scroll.inner_rect),
                                        "Viewport clips must remain inside reserved outer geometry"
                                    );
                                    let save = ui.add_sized(
                                        [
                                            controls::button_width(ui, "Save document"),
                                            controls::height(ui),
                                        ],
                                        button("Save document"),
                                    );
                                    measured = Some((outer.height(), save.rect.top()));
                                });
                            },
                        );
                        if state > 0 {
                            let now = measured.unwrap();
                            if let Some(before) = previous {
                                assert!(
                                    (now.0 - before.0).abs() < 0.1
                                        && (now.1 - before.1).abs() < 0.1,
                                    "Actual scrollbar transition moved Save: {theme_id}/{scale}/{width}: {before:?} -> {now:?}"
                                );
                            }
                            previous = Some(now);
                        }
                    }
                }
            }
        }
    }
    assert!(
        horizontal && vertical,
        "Exercise actual horizontal and vertical overflow, not just short wrapped documents"
    );
}

#[test]
fn script_feedback_capture_driver_uses_actual_run_case_selection_and_seven_bounded_captures() {
    assert_eq!(
        LabApp::script_feedback_stage(false, Some("script-feedback")),
        None
    );
    assert_eq!(
        LabApp::script_feedback_stage(true, Some("script-feedback-extra")),
        None
    );
    assert_eq!(
        LabApp::script_feedback_stage(true, Some("script-feedback")),
        Some(47000)
    );
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 47000;
    let mut captures = Vec::new();
    let mut closed = false;
    for n in 0..5000 {
        let events = std::mem::take(&mut a.pending_events);
        let modifiers = events
            .iter()
            .rev()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let width = if a.scale == 1. { 1920. } else { 946. };
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / (a.scale * 1.15), 1080. / (a.scale * 1.15)),
            )),
            events,
            modifiers,
            time: Some(n as f64 / 20.),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        let out = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        });
        for viewport in out.viewport_output.values() {
            for command in &viewport.commands {
                match command {
                    egui::ViewportCommand::Screenshot(data) => {
                        let name = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert!(name.starts_with("script-feedback-"));
                        captures.push(name);
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => closed = true,
                    _ => {}
                }
            }
        }
        if closed {
            break;
        }
    }
    assert!(
        closed,
        "Capture driver must terminate within its actual frame budget"
    );
    assert_eq!(captures.len(), 7);
    for suffix in ["dark100", "light150", "dark200"] {
        assert!(
            captures
                .iter()
                .any(|s| s == &format!("script-feedback-selected-pass-{suffix}"))
        );
        assert!(
            captures
                .iter()
                .any(|s| s == &format!("script-feedback-overall-fail-{suffix}"))
        );
    }
    assert_eq!(a.script_case, 0);
    let i = a.selected.unwrap();
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}

#[test]
fn nfs_alignment_driver_captures_both_editors_at_every_supported_size_without_model_changes() {
    assert_eq!(
        LabApp::nfs_alignment_stage(false, Some("nfs-alignment")),
        None
    );
    assert_eq!(
        LabApp::nfs_alignment_stage(true, Some("nfs-alignment-extra")),
        None
    );
    assert_eq!(
        LabApp::nfs_alignment_stage(true, Some("nfs-alignment")),
        Some(48000)
    );
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = 48000;
    let mut captures = std::collections::BTreeSet::new();
    let mut closed = false;
    for n in 0..10_000 {
        let events = std::mem::take(&mut a.pending_events);
        let case = a.smoke_stage.saturating_sub(48000) / 3;
        let width = if case >= 10 { 946. } else { 1920. };
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / (a.scale * 1.15), 1080. / (a.scale * 1.15)),
            )),
            events,
            time: Some(n as f64 / 20.),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(1.15);
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(case < 10);
        let out = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(ctx);
        });
        for viewport in out.viewport_output.values() {
            for command in &viewport.commands {
                match command {
                    egui::ViewportCommand::Screenshot(data) => {
                        let name = data
                            .data
                            .as_ref()
                            .unwrap()
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        assert!(
                            captures.insert(name),
                            "Every geometry capture must be distinct"
                        );
                        a.capture_pending = false;
                    }
                    egui::ViewportCommand::Close => closed = true,
                    _ => {}
                }
            }
        }
        if closed {
            break;
        }
    }
    assert!(closed, "Bounded actual capture controller must finish");
    assert_eq!(captures.len(), 40);
    for width in ["wide", "narrow"] {
        for theme in ["light", "dark"] {
            for size in [100, 115, 125, 150, 200] {
                for editor in ["exports", "fstab"] {
                    assert!(
                        captures
                            .contains(&format!("nfs-alignment-{width}-{theme}-{size}-{editor}"))
                    );
                }
            }
        }
    }
    let i = a.selected.unwrap();
    assert_eq!(
        a.progress.attempts[&a.scenarios[i].id].state,
        a.scenarios[i].initial
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}
