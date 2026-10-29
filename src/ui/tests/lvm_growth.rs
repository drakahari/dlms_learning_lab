//! Prospective real-egui interaction, not installed/native or novice acceptance.
use super::*;
use dlms_learning_lab::lvm_workspace as work;
const LVM_IDS: [&str; 4] = [
    "lpic2-lvm-free-space",
    "lpic2-lvm-add-pv",
    "lpic2-lvm-filesystem-gap",
    "lpic2-lvm-handover",
];
fn opened_at(offset: usize) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert!(!a.synthetic);
    a.filter.track = "LPIC-2".into();
    a.filter.unit = "lvm-volume-growth".into();
    draw(&ctx, &mut a);
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == LVM_IDS[offset])
        .unwrap();
    let mut start = None;
    for _ in 0..20 {
        let position = a
            .metrics
            .card_verification
            .iter()
            .position(|r| r.0 == LVM_IDS[offset])
            .unwrap();
        let r = a.metrics.card_actions[position].0;
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
            start = Some(r);
            break;
        }
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
    }
    assert!(a.progress.attempts.is_empty());
    click_real(
        &ctx,
        &mut a,
        start.expect("Normal Start must become visible within20 real wheels"),
    );
    assert_eq!(a.selected, Some(i));
    assert_eq!(a.progress.attempts.len(), 1);
    assert!(a.progress.attempts.contains_key(LVM_IDS[offset]));
    (ctx, a, i)
}
fn opened() -> (egui::Context, LabApp, usize) {
    opened_at(0)
}
fn session(a: &LabApp) -> &work::Session {
    let engine::State::Lvm { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    for _ in 0..20 {
        let (r, clip) = if field {
            let f = a.metrics.lvm.fields.iter().find(|f| f.0 == key).unwrap();
            (f.1, f.3)
        } else {
            let c = a.metrics.lvm.controls.iter().find(|c| c.0 == key).unwrap();
            (c.1, c.2)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let clip = clip.intersect(root);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            frame(ctx, a, 1200., 850., vec![]);
        }
    }
    panic!("Actual LVM control {key} did not become visible in20 real wheels")
}
fn click_real(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    let mut events = vec![egui::Event::PointerMoved(r.center())];
    for pressed in [true, false] {
        events.push(egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        });
    }
    frame(ctx, a, 1200., 850., events);
    draw(ctx, a);
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click_real(ctx, a, r);
}
fn type_command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "command", true);
    click_real(ctx, a, r);
    let id = a
        .metrics
        .lvm
        .fields
        .iter()
        .find(|f| f.0 == "command")
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let mut events = [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .collect::<Vec<_>>();
    events.push(egui::Event::Paste(text.into()));
    frame(ctx, a, 1200., 850., events);
    draw(ctx, a);
}
#[test]
fn real_intro_inert_insert_separate_practice_assignment_and_challenge_keep_their_scopes() {
    let (ctx, mut a, i) = opened();
    assert_eq!(session(&a).view, teaching::View::Introduction);
    let initial = session(&a).current.clone();
    tap(&ctx, &mut a, "example");
    assert_eq!(session(&a).view, teaching::View::Example);
    let before = session(&a).example.clone();
    tap(&ctx, &mut a, "insert");
    assert_eq!(session(&a).example, before);
    assert!(session(&a).example_transcript.is_empty());
    tap(&ctx, &mut a, "run");
    assert_eq!(session(&a).current, initial);
    assert_eq!(session(&a).example_transcript.len(), 1);
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "assignment");
    let solution = a.scenarios[i].solution.clone();
    for command in solution {
        type_command(&ctx, &mut a, &command);
        assert_eq!(session(&a).command, command);
        tap(&ctx, &mut a, "run");
    }
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert!(!a.metrics.lvm.rows.is_empty());
    assert!(a.metrics.lvm.rows.iter().all(|r| r.0));
    assert_eq!(session(&a).example, before);
    let kept = serde_json::to_value(&a.progress).unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.mode_challenge);
    draw(&ctx, &mut a);
    assert!(a.progress.attempts[&a.scenarios[i].id].challenge);
    assert!(a.metrics.lvm.goal.is_some());
    assert!(
        !a.metrics
            .lvm
            .controls
            .iter()
            .any(|c| c.0 == "example" || c.0 == "insert")
    );
    let mut expected = kept;
    expected["attempts"][&a.scenarios[i].id]["challenge"] = true.into();
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), expected);
}
#[test]
fn real_oversized_paste_and_same_frame_run_refuse_old_command_and_preserve_saved_work() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    type_command(&ctx, &mut a, "vgs");
    let field = target(&ctx, &mut a, "command", true);
    click_real(&ctx, &mut a, field);
    let run = target(&ctx, &mut a, "run", false);
    let before = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    let mut events = vec![
        egui::Event::Paste("é".repeat(2048)),
        egui::Event::PointerMoved(run.center()),
    ];
    for pressed in [true, false] {
        events.push(egui::Event::PointerButton {
            pos: run.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        });
    }
    frame(&ctx, &mut a, 1200., 850., events);
    draw(&ctx, &mut a);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    assert!(session(&a).transcript.is_empty());
    assert_eq!(session(&a).command, "vgs");
}

// The original UTF-8 regression above remains intact. This adds the ASCII
// truncation counterexample and both real dispatch paths without raising limits.
#[test]
fn full_ascii_or_utf8_overflow_packet_with_enter_or_run_retains_exact_work_and_native_selection() {
    for unicode in [false, true] {
        for enter in [false, true] {
            let (ctx, mut a, _) = opened();
            tap(&ctx, &mut a, "assignment");
            type_command(&ctx, &mut a, "vgs");
            let field = target(&ctx, &mut a, "command", true);
            click_real(&ctx, &mut a, field);
            let run = target(&ctx, &mut a, "run", false);
            let id = a.metrics.lvm.fields[0].2;
            let modifiers = egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            };
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers,
                    },
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers,
                    },
                ],
            );
            let old_native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
            let old_cursor = old_native.cursor.char_range();
            let old_undo = format!("{:?}", old_native.undoer());
            assert!(
                old_cursor
                    .unwrap()
                    .primary
                    .index
                    .abs_diff(old_cursor.unwrap().secondary.index)
                    > 0
            );
            let before = serde_json::to_value(&a.progress).unwrap();
            let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
            let prefix = "lvextend -L +8M records/reports";
            let payload = if unicode {
                format!("{prefix}{} unsupported-tail", "é".repeat(2048))
            } else {
                format!("{prefix}{} unsupported-tail", " ".repeat(2048))
            };
            assert!(payload.len() > 2048);
            let mut events = vec![egui::Event::Paste(payload)];
            if enter {
                for pressed in [true, false] {
                    events.push(egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    });
                }
            } else {
                events.push(egui::Event::PointerMoved(run.center()));
                for pressed in [true, false] {
                    events.push(egui::Event::PointerButton {
                        pos: run.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    });
                }
            }
            frame(&ctx, &mut a, 1200., 850., events);
            let native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
            assert_eq!(native.cursor.char_range(), old_cursor);
            assert_eq!(format!("{:?}", native.undoer()), old_undo);
            assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
            assert_eq!(
                serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                receipts
            );
            draw(&ctx, &mut a);
            assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
            assert_eq!(session(&a).command, "vgs");
            assert!(session(&a).transcript.is_empty());
            assert!(a.notice.as_ref().unwrap().contains("nothing ran"));
        }
    }
}
#[test]
fn invalid_command_error_clears_after_real_valid_retry_and_failed_rows_describe_current_capacity() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    type_command(&ctx, &mut a, "not-a-supported-lvm-command");
    tap(&ctx, &mut a, "run");
    assert!(a.notice.is_some());
    let notice = target(&ctx, &mut a, "notice", false);
    let run = a.metrics.terminal_run.unwrap();
    let output = a.metrics.output.unwrap();
    let tolerance = 1. / ctx.pixels_per_point();
    assert!(
        notice.top() >= run.bottom() - tolerance && notice.bottom() <= output.top() + tolerance,
        "The sole command error belongs next to Run, before output and all checks"
    );
    assert!(
        a.metrics.lvm.inspected_files.is_empty(),
        "Full retained-file bodies start optional"
    );
    assert!(
        a.metrics
            .lvm
            .capacity_texts
            .iter()
            .any(|s| s.contains("records/reports: LV 8 MiB · filesystem 8 MiB"))
    );
    type_command(&ctx, &mut a, "vgs");
    tap(&ctx, &mut a, "run");
    assert!(a.notice.is_none());
    assert_eq!(session(&a).transcript.last().unwrap().input, "vgs");
    let engine::Goal::Lvm { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let rows = definition.checks(session(&a));
    assert!(!rows[0].0 && !rows[1].0);
    assert!(
        rows[0]
            .1
            .contains("Current LV 8388608 bytes; required at least 16777216 bytes")
    );
    assert!(
        rows[1]
            .1
            .contains("Current filesystem 8388608 bytes; required at least 16777216 bytes")
    );
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.lvm.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![false, false, true, true]
    );
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .lvm
            .feedback_texts
            .iter()
            .any(|s| s.contains("Current LV 8 MiB; required at least 16 MiB"))
    );
    assert!(
        a.metrics
            .lvm
            .feedback_texts
            .iter()
            .any(|s| s.contains("Current filesystem 8 MiB; required at least 16 MiB"))
    );
    let expected_files = session(&a)
        .current
        .groups
        .values()
        .flat_map(|g| g.volumes.values())
        .filter_map(|v| v.filesystem.as_ref())
        .flat_map(|fs| {
            fs.files
                .iter()
                .map(|(p, f)| (p.clone(), f.contents.clone()))
        })
        .collect::<Vec<_>>();
    let before = serde_json::to_value(&a.progress).unwrap();
    tap(&ctx, &mut a, "inspect-files");
    assert_eq!(
        a.metrics.lvm.inspected_files, expected_files,
        "The optional inspection still renders every full retained path and content"
    );
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
}

#[test]
fn shared_command_controls_fit_and_edit_at_all_supported_scales_themes_and_two_widths() {
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for name in ["dlms-dark", "dlms-light"] {
            for width in [760., 1600.] {
                let (ctx, mut a, _) = opened();
                tap(&ctx, &mut a, "assignment");
                a.scale = scale;
                a.theme_id = name.into();
                theme::apply(&ctx, name);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                let mut visible = false;
                for _ in 0..20 {
                    let (_, field, _, clip) = a.metrics.lvm.fields[0];
                    let root = a
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    if smoke::smoke_control_visible(
                        field,
                        clip.intersect(root),
                        ctx.pixels_per_point(),
                    ) {
                        visible = true;
                        break;
                    }
                    frame(
                        &ctx,
                        &mut a,
                        width,
                        929.,
                        vec![
                            egui::Event::PointerMoved(egui::pos2(
                                root.left() + 1.,
                                root.center().y,
                            )),
                            egui::Event::MouseWheel {
                                unit: egui::MouseWheelUnit::Point,
                                delta: egui::vec2(
                                    0.,
                                    (root.center().y - field.center().y)
                                        .clamp(-root.height() * 0.75, root.height() * 0.75),
                                ),
                                modifiers: Default::default(),
                            },
                        ],
                    );
                    for _ in 0..10 {
                        frame(&ctx, &mut a, width, 929., vec![]);
                    }
                }
                assert!(
                    visible,
                    "Actual command control did not become visible within20 wheels"
                );
                let (_, field, id, _) = a.metrics.lvm.fields[0];
                let root = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let run = a
                    .metrics
                    .lvm
                    .controls
                    .iter()
                    .find(|r| r.0 == "run")
                    .unwrap()
                    .1;
                assert!(
                    (field.height() - run.height()).abs() < 1.,
                    "{scale}/{name}/{width}"
                );
                assert!(field.left() >= root.left() - 1. && field.right() <= root.right() + 1.);
                frame(
                    &ctx,
                    &mut a,
                    width,
                    929.,
                    vec![egui::Event::PointerMoved(field.center())],
                );
                frame(
                    &ctx,
                    &mut a,
                    width,
                    929.,
                    [true, false]
                        .into_iter()
                        .map(|pressed| egui::Event::PointerButton {
                            pos: field.center(),
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        })
                        .collect(),
                );
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                frame(
                    &ctx,
                    &mut a,
                    width,
                    929.,
                    vec![egui::Event::Paste("vgs".into())],
                );
                assert_eq!(session(&a).command, "vgs");
                assert!(session(&a).transcript.is_empty());
                assert!(!a.metrics.completion);
            }
        }
    }
}

#[test]
fn enter_before_paste_or_aggregate_insert_overflow_refuses_the_whole_packet() {
    for mode in 0..3 {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        type_command(&ctx, &mut a, "vgs");
        let field = target(&ctx, &mut a, "command", true);
        click_real(&ctx, &mut a, field);
        let id = a.metrics.lvm.fields[0].2;
        let before = serde_json::to_value(&a.progress).unwrap();
        let old = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        let selection = old.cursor.char_range();
        let undo = format!("{:?}", old.undoer());
        let enter = || {
            [true, false]
                .into_iter()
                .map(|pressed| egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Default::default(),
                })
                .collect::<Vec<_>>()
        };
        let events = match mode {
            0 => {
                let mut e = enter();
                e.push(egui::Event::Paste("pvs".into()));
                e
            }
            1 => {
                let mut e = enter();
                e.push(egui::Event::Paste("x".repeat(2049)));
                e
            }
            _ => {
                let first = " ".repeat(1200);
                let second = " ".repeat(1200);
                assert!(first.len() <= 2048 && second.len() <= 2048);
                assert!(first.len() + second.len() > 2048);
                let mut e = vec![egui::Event::Paste(first), egui::Event::Paste(second)];
                e.extend(enter());
                e
            }
        };
        frame(&ctx, &mut a, 1200., 850., events);
        let native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        assert_eq!(native.cursor.char_range(), selection);
        assert_eq!(format!("{:?}", native.undoer()), undo);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        draw(&ctx, &mut a);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        assert_eq!(session(&a).command, "vgs");
        assert!(session(&a).transcript.is_empty());
        assert!(!a.metrics.completion);
    }
}

#[test]
fn all_four_normal_starts_run_independent_practice_change_repair_and_complete_whole_assignments() {
    for offset in 0..4 {
        let (ctx, mut a, i) = opened_at(offset);
        assert_eq!(session(&a).view, teaching::View::Introduction);
        let initial = session(&a).current.clone();
        tap(&ctx, &mut a, "example");
        let engine::Goal::Lvm { definition } = &a.scenarios[i].goal else {
            panic!()
        };
        let worked = definition
            .lesson
            .worked
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<_>>();
        let experiment = definition.lesson.experiment.clone();
        let repair = definition.lesson.repair.clone();
        for c in worked {
            type_command(&ctx, &mut a, &c);
            tap(&ctx, &mut a, "run");
            assert_eq!(session(&a).current, initial);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(
            a.metrics.lvm.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![true; [5, 5, 5, 9][offset]]
        );
        assert!(!a.metrics.completion);
        type_command(&ctx, &mut a, &experiment);
        tap(&ctx, &mut a, "run");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.lvm.rows.iter().any(|r| !r.0));
        type_command(&ctx, &mut a, &repair);
        tap(&ctx, &mut a, "run");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.lvm.rows.iter().all(|r| r.0));
        assert_eq!(session(&a).current, initial);
        let example = session(&a).example.clone();
        tap(&ctx, &mut a, "assignment");
        for c in a.scenarios[i].solution.clone() {
            type_command(&ctx, &mut a, &c);
            tap(&ctx, &mut a, "run");
        }
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert_eq!(
            a.metrics.lvm.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![true; [4, 4, 4, 7][offset]]
        );
        assert_eq!(session(&a).example, example);
        assert!(progress::current_solved(
            &a.progress.attempts[&a.scenarios[i].id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn normal_shared_next_requires_current_completion_and_a_later_valid_navigation_gesture() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    assert!(a.metrics.next_unit.is_none());
    for c in a.scenarios[i].solution.clone() {
        type_command(&ctx, &mut a, &c);
        tap(&ctx, &mut a, "run");
    }
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let r = target(&ctx, &mut a, "next", false);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    let before = serde_json::to_value(&a.progress).unwrap();
    let mut events = vec![egui::Event::Paste("x".repeat(2049))];
    for pressed in [true, false] {
        events.push(egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        });
    }
    frame(&ctx, &mut a, 1200., 850., events);
    draw(&ctx, &mut a);
    assert_eq!(a.selected, Some(i));
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    tap(&ctx, &mut a, "next");
    assert_eq!(a.selected, Some(i + 1));
    assert_eq!(session(&a).view, teaching::View::Introduction);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id]).unwrap(),
        before["attempts"][&a.scenarios[i].id]
    );
    assert_eq!(a.progress.clock, before["clock"].as_u64().unwrap() + 1);
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i + 1].id],
        &a.scenarios[i + 1]
    ));
}
#[test]
fn normal_reset_cancel_confirm_keeps_other_context_and_clears_discarded_native_undo() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "example");
    type_command(&ctx, &mut a, "vgs");
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "assignment");
    type_command(&ctx, &mut a, "unfinished command |");
    // The second real activity is prepared through the same admitted app path;
    // actual reset/confirmation/Continue controls and edits below remain native.
    a.launch(i + 1);
    draw(&ctx, &mut a);
    tap(&ctx, &mut a, "assignment");
    type_command(&ctx, &mut a, "unfinished second activity |");
    a.launch(i);
    draw(&ctx, &mut a);
    let before = serde_json::to_value(&a.progress).unwrap();
    let receipt = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    let visited = a.progress.attempts[&a.scenarios[i].id].visited;
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    draw(&ctx, &mut a);
    assert!(a.reset_pending);
    accounts_ui_click(&ctx, &mut a, |m| m.reset_cancel);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.confirm_reset);
    draw(&ctx, &mut a);
    let mut expected = Attempt::new(&a.scenarios[i], visited);
    model::prepare(&mut expected);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id]).unwrap(),
        serde_json::to_value(&expected).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i + 1].id]).unwrap(),
        before["attempts"][&a.scenarios[i + 1].id]
    );
    assert_eq!(
        serde_json::to_value(&a.progress.favorites).unwrap(),
        before["favorites"]
    );
    assert_eq!(a.progress.clock, before["clock"].as_u64().unwrap());
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipt
    );
    tap(&ctx, &mut a, "assignment");
    let field = target(&ctx, &mut a, "command", true);
    click_real(&ctx, &mut a, field);
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Z,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            })
            .collect(),
    );
    assert!(session(&a).command.is_empty() && session(&a).example_command.is_empty());
    assert!(session(&a).transcript.is_empty() && session(&a).example_transcript.is_empty());
    assert!(!a.metrics.completion);
}
#[test]
fn optional_help_is_progressive_persists_requests_and_requires_separate_worked_answer_and_references()
 {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    let state = serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap();
    assert!(
        a.metrics.lvm.requested_hints.is_empty()
            && a.metrics.lvm.worked_commands.is_empty()
            && a.metrics.lvm.reference_urls.is_empty()
    );
    tap(&ctx, &mut a, "optional-help");
    assert!(
        a.metrics.lvm.requested_hints.is_empty()
            && a.metrics.lvm.worked_commands.is_empty()
            && a.metrics.lvm.reference_urls.is_empty()
    );
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, 1);
    assert_eq!(a.metrics.lvm.requested_hints, a.scenarios[i].hints[..1]);
    assert!(a.metrics.lvm.worked_commands.is_empty() && a.metrics.lvm.reference_urls.is_empty());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        state
    );
    assert!(!a.metrics.completion);
    a.save();
    let raw = std::fs::read(&a.path).unwrap();
    let saved = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(saved.attempts[&a.scenarios[i].id].hints, 1);
    assert!(saved.migration_receipts.is_some());
    assert_eq!(std::fs::read(&a.path).unwrap(), raw);
    accounts_ui_click(&ctx, &mut a, |m| m.back);
    assert!(a.selected.is_none());
    let mut card = None;
    for _ in 0..20 {
        let at = a
            .metrics
            .card_verification
            .iter()
            .position(|r| r.0 == a.scenarios[i].id)
            .unwrap();
        let r = a.metrics.card_actions[at].0;
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
            card = Some(r);
            break;
        }
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
    }
    click_real(
        &ctx,
        &mut a,
        card.expect("Real Continue card must become visible"),
    );
    assert_eq!(a.selected, Some(i));
    assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, 1);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        state
    );
    tap(&ctx, &mut a, "optional-help");
    assert_eq!(a.metrics.lvm.requested_hints, a.scenarios[i].hints[..1]);
    assert!(a.metrics.lvm.worked_commands.is_empty());
    accounts_ui_click(&ctx, &mut a, |m| m.mode_challenge);
    assert!(a.progress.attempts[&a.scenarios[i].id].challenge);
    assert!(a.metrics.lvm.goal.is_some());
    assert!(a.metrics.lvm.worked_commands.is_empty() && a.metrics.lvm.reference_urls.is_empty());
    tap(&ctx, &mut a, "optional-help");
    assert_eq!(a.metrics.lvm.requested_hints, a.scenarios[i].hints[..1]);
    let before = serde_json::to_value(&a.progress).unwrap();
    tap(&ctx, &mut a, "worked-approach");
    assert_eq!(a.metrics.lvm.worked_commands, a.scenarios[i].solution);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    tap(&ctx, &mut a, "references");
    assert_eq!(a.metrics.lvm.reference_urls, a.scenarios[i].sources);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert!(!a.metrics.completion);
}

// Return the actual painted frame; geometry alone did not catch a collapsed
// nested terminal viewport in the original native Fedora screenshots.
fn transcript_paint_frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, 929. / a.scale),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    )
}
fn transcript_text_paints_inside(
    shape: &egui::Shape,
    clip: egui::Rect,
    viewport: egui::Rect,
    expected: &str,
    tolerance: f32,
) -> bool {
    match shape {
        egui::Shape::Vec(items) => items
            .iter()
            .any(|s| transcript_text_paints_inside(s, clip, viewport, expected, tolerance)),
        egui::Shape::Text(text) => {
            let bounds = text.visual_bounding_rect();
            text.galley.text() == expected
                && text.opacity_factor > 0.
                && bounds.is_positive()
                && clip.expand(tolerance).contains_rect(bounds)
                && viewport.expand(tolerance).contains_rect(bounds)
                && text.galley.rows.iter().any(|r| {
                    !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()].is_empty()
                })
                && text.galley.rows.iter().all(|r| {
                    r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                        .iter()
                        .all(|v| {
                            text.override_text_color
                                .unwrap_or_else(|| {
                                    if v.color == egui::Color32::PLACEHOLDER {
                                        text.fallback_color
                                    } else {
                                        v.color
                                    }
                                })
                                .a()
                                > 0
                        })
                })
        }
        _ => false,
    }
}
#[test]
fn populated_assignment_and_practice_transcripts_keep_six_rows_and_paint_latest_output_at_every_geometry()
 {
    for practice in [false, true] {
        let (ctx, mut a, _) = opened();
        tap(
            &ctx,
            &mut a,
            if practice { "example" } else { "assignment" },
        );
        for command in ["vgs", "vgs", "vgs", "lvs"] {
            type_command(&ctx, &mut a, command);
            tap(&ctx, &mut a, "run");
        }
        let transcript = if practice {
            &session(&a).example_transcript
        } else {
            &session(&a).transcript
        };
        assert_eq!(
            transcript.len(),
            4,
            "Real Run must populate enough content to require scrolling"
        );
        let latest = transcript.last().unwrap().output.clone();
        let older = transcript.first().unwrap().output.clone();
        assert!(!latest.is_empty());
        assert_ne!(older, latest, "History/last output must be distinguishable");
        let before = serde_json::to_value(&a.progress).unwrap();
        let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for theme_id in ["dlms-dark", "dlms-light"] {
                for width in [760., 1600.] {
                    a.scale = scale;
                    a.theme_id = theme_id.into();
                    theme::apply(&ctx, theme_id);
                    for _ in 0..6 {
                        transcript_paint_frame(&ctx, &mut a, width, vec![]);
                    }
                    let mut visible = false;
                    for _ in 0..20 {
                        let output = a.metrics.output.expect("Populated transcript viewport");
                        let root = a
                            .metrics
                            .main_viewport
                            .unwrap()
                            .intersect(ctx.content_rect());
                        let h = ctx.style().text_styles[&egui::TextStyle::Monospace].size;
                        assert!(
                            output.height() >= h * 5.,
                            "Nested transcript collapsed at {practice}/{scale}/{theme_id}/{width}: {output:?}"
                        );
                        if smoke::smoke_control_visible(output, root, ctx.pixels_per_point()) {
                            visible = true;
                            break;
                        }
                        transcript_paint_frame(
                            &ctx,
                            &mut a,
                            width,
                            vec![
                                egui::Event::PointerMoved(egui::pos2(
                                    root.left() + 1.,
                                    root.center().y,
                                )),
                                egui::Event::MouseWheel {
                                    unit: egui::MouseWheelUnit::Point,
                                    delta: egui::vec2(
                                        0.,
                                        (root.center().y - output.center().y)
                                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                                    ),
                                    modifiers: Default::default(),
                                },
                            ],
                        );
                        for _ in 0..10 {
                            transcript_paint_frame(&ctx, &mut a, width, vec![]);
                        }
                    }
                    assert!(
                        visible,
                        "The real outer scroll must reveal the complete output in20 wheels"
                    );
                    let out = transcript_paint_frame(&ctx, &mut a, width, vec![]);
                    let viewport = a.metrics.output.unwrap();
                    let tolerance = 1. / ctx.pixels_per_point();
                    let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                    let row = ctx.fonts_mut(|f| f.row_height(&font));
                    assert!(
                        (viewport.height() - row * 6.).abs() <= tolerance,
                        "Exactly six font-derived rows, independent of remaining outer height"
                    );
                    assert!(
                        out.shapes.iter().any(|s| transcript_text_paints_inside(
                            &s.shape,
                            s.clip_rect,
                            viewport,
                            &latest,
                            tolerance
                        )),
                        "Latest real simulated report output must have visible nontransparent glyphs entirely within transcript clip: {practice}/{scale}/{theme_id}/{width}"
                    );
                    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
                    assert_eq!(
                        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                        receipts
                    );
                }
            }
        }
        // Real inner-wheel browsing can leave the latest output, and return.
        // It must not be trapped at the end or require a programmatic offset.
        for (expected, direction) in [(&older, 1.), (&latest, -1.)] {
            reveal_lvm_scroll_frame(&ctx, &mut a, 1600., false);
            let mut painted = false;
            for _ in 0..20 {
                let viewport = a.metrics.output.unwrap();
                transcript_paint_frame(
                    &ctx,
                    &mut a,
                    1600.,
                    vec![egui::Event::PointerMoved(viewport.center())],
                );
                for _ in 0..3 {
                    transcript_paint_frame(&ctx, &mut a, 1600., vec![]);
                }
                transcript_paint_frame(
                    &ctx,
                    &mut a,
                    1600.,
                    vec![
                        egui::Event::PointerMoved(viewport.center()),
                        egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0., direction * viewport.height() / 3.),
                            modifiers: Default::default(),
                        },
                    ],
                );
                for _ in 0..10 {
                    transcript_paint_frame(&ctx, &mut a, 1600., vec![]);
                }
                let output = reveal_lvm_scroll_frame(&ctx, &mut a, 1600., false);
                let viewport = a.metrics.output.unwrap();
                let root = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                if smoke::smoke_control_visible(viewport, root, ctx.pixels_per_point())
                    && output.shapes.iter().any(|s| {
                        transcript_text_paints_inside(
                            &s.shape,
                            s.clip_rect,
                            viewport,
                            expected,
                            1. / ctx.pixels_per_point(),
                        )
                    })
                {
                    painted = true;
                    break;
                }
            }
            if !painted {
                let debug = transcript_paint_frame(&ctx, &mut a, 1600., vec![]);
                let viewport = a.metrics.output.unwrap();
                let visible = debug
                    .shapes
                    .iter()
                    .filter_map(|s| match &s.shape {
                        egui::Shape::Text(t)
                            if t.visual_bounding_rect()
                                .intersects(viewport.intersect(s.clip_rect)) =>
                        {
                            Some((t.galley.text(), t.visual_bounding_rect(), s.clip_rect))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                eprintln!("Expected history text {expected:?}; visible terminal text {visible:?}");
            }
            assert!(
                painted,
                "Actual inner-wheel browsing must show older output and then the distinct latest report: practice={practice}, direction={direction}, output={:?}, root={:?}",
                a.metrics.output, a.metrics.main_viewport
            );
            assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
            assert_eq!(
                serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                receipts
            );
        }
    }
}

// Keep a real nested viewport on screen after natural edge-wheel propagation.
// No scroll state is loaded or assigned; only bounded ordinary outer wheels.
fn reveal_lvm_scroll_frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    width: f32,
    files: bool,
) -> egui::FullOutput {
    for _ in 0..20 {
        let output = transcript_paint_frame(ctx, a, width, vec![]);
        let target = if files {
            a.metrics.lvm.files_viewport.expect("Opened file viewport")
        } else {
            a.metrics.output.unwrap()
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(target, root, ctx.pixels_per_point()) {
            return output;
        }
        transcript_paint_frame(
            ctx,
            a,
            width,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - target.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            transcript_paint_frame(ctx, a, width, vec![]);
        }
    }
    panic!("Real outer wheel must reveal the bounded nested viewport");
}
#[test]
fn optional_retained_file_inspection_paints_actual_file_contents_at_every_geometry() {
    for practice in [false, true] {
        let (ctx, mut a, _) = opened();
        tap(
            &ctx,
            &mut a,
            if practice { "example" } else { "assignment" },
        );
        type_command(&ctx, &mut a, "vgs");
        tap(&ctx, &mut a, "run");
        let world = if practice {
            &session(&a).example
        } else {
            &session(&a).current
        };
        let contents = world
            .groups
            .values()
            .flat_map(|g| g.volumes.values())
            .filter_map(|v| v.filesystem.as_ref())
            .flat_map(|fs| fs.files.values())
            .next()
            .unwrap()
            .contents
            .clone();
        assert!(!contents.is_empty());
        assert!(
            a.metrics.lvm.files_viewport.is_none(),
            "Inspection starts closed"
        );
        let before = serde_json::to_value(&a.progress).unwrap();
        let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
        tap(&ctx, &mut a, "inspect-files");
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for theme_id in ["dlms-dark", "dlms-light"] {
                for width in [760., 1600.] {
                    a.scale = scale;
                    a.theme_id = theme_id.into();
                    theme::apply(&ctx, theme_id);
                    for _ in 0..10 {
                        transcript_paint_frame(&ctx, &mut a, width, vec![]);
                    }
                    let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                    let row = ctx.fonts_mut(|f| f.row_height(&font));
                    let viewport = a
                        .metrics
                        .lvm
                        .files_viewport
                        .expect("Real click opens optional inspection");
                    assert!(
                        (viewport.height() - row * 6.).abs() <= 1. / ctx.pixels_per_point(),
                        "Optional file contents need the same bounded six-row viewport, not zeroheight"
                    );
                    let mut output = reveal_lvm_scroll_frame(&ctx, &mut a, width, true);
                    let mut painted = false;
                    for _ in 0..20 {
                        let viewport = a.metrics.lvm.files_viewport.unwrap();
                        if output.shapes.iter().any(|s| {
                            transcript_text_paints_inside(
                                &s.shape,
                                s.clip_rect,
                                viewport,
                                &contents,
                                1. / ctx.pixels_per_point(),
                            )
                        }) {
                            painted = true;
                            break;
                        }
                        transcript_paint_frame(
                            &ctx,
                            &mut a,
                            width,
                            vec![
                                egui::Event::PointerMoved(viewport.center()),
                                egui::Event::MouseWheel {
                                    unit: egui::MouseWheelUnit::Point,
                                    delta: egui::vec2(0., -row),
                                    modifiers: Default::default(),
                                },
                            ],
                        );
                        for _ in 0..10 {
                            transcript_paint_frame(&ctx, &mut a, width, vec![]);
                        }
                        output = reveal_lvm_scroll_frame(&ctx, &mut a, width, true);
                    }
                    assert!(
                        painted,
                        "Actual retained file contents must become visible with real bounded wheels and unclipped glyphs: {practice}/{scale}/{theme_id}/{width}"
                    );
                    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
                    assert_eq!(
                        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                        receipts
                    );
                }
            }
        }
    }
}
