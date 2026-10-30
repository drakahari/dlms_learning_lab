//! Private capability fixtures exercise real native controls; no curriculum registration.
use super::*;
use dlms_learning_lab::engine::Goal;
use dlms_learning_lab::partition_fs_workspace as work;
#[path = "../../../tests/support/partition_fs_fixture.rs"]
mod fixture;
fn portable(a: &LabApp) -> serde_json::Value {
    serde_json::json!({"portable":a.progress,"receipts":a.progress.migration_receipts})
}
fn session(a: &LabApp) -> &work::Session {
    fixture::session(&a.progress.attempts[&a.scenarios[a.selected.unwrap()].id])
}
fn control_rect(a: &LabApp, key: &str, field: bool) -> (egui::Rect, egui::Rect) {
    if field {
        let r = a
            .metrics
            .partition_fs
            .fields
            .iter()
            .find(|r| r.0 == key)
            .unwrap();
        (r.1, r.3)
    } else {
        let r = a
            .metrics
            .partition_fs
            .controls
            .iter()
            .find(|r| r.0 == key)
            .unwrap();
        (r.1, r.2)
    }
}
fn reveal(
    ctx: &egui::Context,
    a: &mut LabApp,
    key: &str,
    field: bool,
    w: f32,
    h: f32,
) -> egui::Rect {
    for _ in 0..20 {
        let (r, clip) = control_rect(a, key, field);
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip.intersect(root), ctx.pixels_per_point()) {
            return r;
        }
        let pos = egui::pos2(root.left() + 2., root.center().y);
        frame(ctx, a, w, h, vec![egui::Event::PointerMoved(pos)]);
        frame(
            ctx,
            a,
            w,
            h,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (root.center().y - r.center().y)
                        .clamp(-root.height() * 0.75, root.height() * 0.75),
                ),
                modifiers: Default::default(),
            }],
        );
        for _ in 0..10 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("Actual disk control {key} must be visible within20 settled real wheels")
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect, w: f32, h: f32) {
    frame(ctx, a, w, h, vec![egui::Event::PointerMoved(r.center())]);
    frame(
        ctx,
        a,
        w,
        h,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            })
            .collect(),
    );
    for _ in 0..3 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false, 1200., 850.);
    click(ctx, a, r, 1200., 850.);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, picker: &str, key: &str) {
    tap(ctx, a, picker);
    // Long worked-step labels wrap. Reach clipped options through the popup's
    // actual scroll area, just as a learner must, before using their hit boxes.
    for _ in 0..20 {
        let r = a
            .metrics
            .partition_fs
            .choices
            .iter()
            .find(|r| r.0 == key)
            .unwrap();
        let rect = r.1;
        let clip = r.2.intersect(ctx.content_rect());
        if smoke::smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            click(ctx, a, rect, 1200., 850.);
            return;
        }
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(clip.center())],
        );
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (clip.center().y - rect.center().y)
                        .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                ),
                modifiers: Default::default(),
            }],
        );
        for _ in 0..10 {
            draw(ctx, a);
        }
    }
    panic!("Actual popup option {picker}/{key} must be visible within20 settled wheels");
}
fn select_all() -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        })
        .collect()
}
fn enter() -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = reveal(ctx, a, key, true, 1200., 850.);
    click(ctx, a, r, 1200., 850.);
    let id = a
        .metrics
        .partition_fs
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let mut events = select_all();
    events.push(egui::Event::Paste(text.into()));
    frame(ctx, a, 1200., 850., events);
    draw(ctx, a);
    assert!(a.notice.is_none(), "{:?}", a.notice);
}
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert!(!a.synthetic);
    let base = serde_json::to_vec(&(&a.scenarios, &a.units)).unwrap();
    let mut s = fixture::scenario();
    s.unit = "private-disk-ui".into();
    let raw = serde_json::json!({"schema_version":1,"scenarios":[s]});
    let mut private = pack::load(&raw.to_string()).unwrap();
    let s = private.remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(s.clone());
    a.units.push(course::Unit {
        id: s.unit.clone(),
        track: s.track.clone(),
        title: "Private disk capability fixture".into(),
        summary: "Test fixture, not bundled curriculum".into(),
        prerequisites: vec![],
        labs: vec![s.id.clone()],
    });
    assert_eq!(
        serde_json::to_vec(&(&a.scenarios[..i], &a.units[..a.units.len() - 1])).unwrap(),
        base
    );
    a.filter.track = "LPIC-1".into();
    a.filter.search = "Prepare supplied virtual storage".into();
    draw(&ctx, &mut a);
    let mut start = None;
    for _ in 0..20 {
        let at = a
            .metrics
            .card_verification
            .iter()
            .position(|r| r.0 == s.id)
            .unwrap();
        let r = a.metrics.card_actions[at].0;
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
                egui::Event::PointerMoved(root.center()),
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
            draw(&ctx, &mut a);
        }
    }
    assert!(a.progress.attempts.is_empty());
    click(&ctx, &mut a, start.unwrap(), 1200., 850.);
    assert_eq!(a.selected, Some(i));
    assert_eq!(a.progress.attempts.len(), 1);
    (ctx, a, i)
}
fn solve_native(ctx: &egui::Context, a: &mut LabApp) {
    tap(ctx, a, "assignment");
    tap(ctx, a, "create-gpt");
    edit(ctx, a, "name", "reports");
    edit(ctx, a, "start", "2048");
    edit(ctx, a, "end", "18431");
    tap(ctx, a, "add");
    edit(ctx, a, "label", "reports");
    tap(ctx, a, "format");
    edit(ctx, a, "name", "paging");
    edit(ctx, a, "start", "18432");
    edit(ctx, a, "end", "22527");
    choose(ctx, a, "hint", "hint-LinuxSwap");
    tap(ctx, a, "add");
    choose(ctx, a, "content-type", "format-Swap");
    edit(ctx, a, "label", "reports_swap");
    tap(ctx, a, "format");
    tap(ctx, a, "check");
}
#[test]
fn native_current_table_actions_and_partial_feedback_are_not_draft_execution() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics
            .partition_fs
            .rows
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![false, false, true]
    );
    let before = session(&a).current.clone();
    edit(&ctx, &mut a, "name", "reports");
    edit(&ctx, &mut a, "start", "2048");
    edit(&ctx, &mut a, "end", "18431");
    assert_eq!(session(&a).current, before);
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
    solve_native(&ctx, &mut a);
    assert_eq!(
        a.metrics
            .partition_fs
            .rows
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, true, true]
    );
    assert!(progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
    let current = session(&a).current.clone();
    edit(&ctx, &mut a, "end", "unfinished");
    tap(&ctx, &mut a, "check");
    assert_eq!(session(&a).current, current);
    assert!(a.metrics.completion);
    tap(&ctx, &mut a, "add");
    assert!(a.notice.as_ref().unwrap().contains("whole"));
    assert_eq!(session(&a).current, current);
    assert_eq!(session(&a).example, fixture::definition().lesson.fixture);
}
#[test]
fn native_separate_practice_fills_only_then_explicit_actions_and_reset_do_not_touch_assignment() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "practice");
    let original = session(&a).current.clone();
    let initial = session(&a).example.clone();
    tap(&ctx, &mut a, "fill-example");
    assert_eq!(session(&a).example, initial);
    assert!(session(&a).example_transcript.is_empty());
    for (n, key) in [
        (0, "create-gpt"),
        (1, "add"),
        (2, "format"),
        (3, "add"),
        (4, "format"),
    ] {
        if n > 0 {
            choose(&ctx, &mut a, "example-picker", &format!("example-{n}"));
        }
        let before = session(&a).example.clone();
        tap(&ctx, &mut a, "fill-example");
        assert_eq!(session(&a).example, before);
        tap(&ctx, &mut a, key);
        assert!(a.notice.is_none(), "{:?}", a.notice);
    }
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics
            .partition_fs
            .rows
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, true, true]
    );
    let editor = session(&a).assignment_editor.clone();
    edit(&ctx, &mut a, "name", "extra");
    edit(&ctx, &mut a, "start", "12288");
    edit(&ctx, &mut a, "end", "14335");
    choose(&ctx, &mut a, "hint", "hint-LinuxData");
    tap(&ctx, &mut a, "add");
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics
            .partition_fs
            .rows
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![true, true, false]
    );
    assert_eq!(session(&a).current, original);
    tap(&ctx, &mut a, "reset-practice");
    assert_eq!(session(&a).example, initial);
    assert_eq!(session(&a).current, original);
    assert_eq!(session(&a).assignment_editor, editor);
    assert!(session(&a).example_transcript.is_empty());
    assert!(!session(&a).example_checked);
}
#[test]
fn delivered_raw_and_after_return_and_aggregate_bad_packets_restore_whole_draft_and_native_selection()
 {
    for mode in 0..8 {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        edit(&ctx, &mut a, "name", "retained");
        let r = reveal(&ctx, &mut a, "name", true, 1200., 850.);
        click(&ctx, &mut a, r, 1200., 850.);
        let id = a
            .metrics
            .partition_fs
            .fields
            .iter()
            .find(|r| r.0 == "name")
            .unwrap()
            .2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        let state = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        let cursor = state.cursor.char_range();
        let undo = format!("{:?}", state.undoer());
        let before = portable(&a);
        let mut events = select_all();
        events.extend(match mode {
            0 => vec![egui::Event::Paste(format!("reports{}bad", " ".repeat(36)))],
            1 => vec![egui::Event::Paste("é".repeat(19))],
            2 => vec![egui::Event::Paste(" reports\n".into())],
            3 => vec![egui::Event::Paste("reports\t".into())],
            4 => vec![
                egui::Event::Text("1".into()),
                enter(),
                egui::Event::Text("2".into()),
            ],
            5 => vec![enter(), egui::Event::Paste("reports".into())],
            6 => vec![
                egui::Event::Paste("a".repeat(20)),
                egui::Event::Paste("b".repeat(20)),
            ],
            _ => vec![egui::Event::Paste("bad\0".into())],
        });
        frame(&ctx, &mut a, 1200., 850., events);
        assert_eq!(portable(&a), before);
        assert_eq!(
            egui::text_edit::TextEditState::load(&ctx, id)
                .unwrap()
                .cursor
                .char_range(),
            cursor
        );
        assert_eq!(
            format!(
                "{:?}",
                egui::text_edit::TextEditState::load(&ctx, id)
                    .unwrap()
                    .undoer()
            ),
            undo
        );
        assert!(a.notice.as_ref().unwrap().contains("whole edit"));
        draw(&ctx, &mut a);
        assert_eq!(portable(&a), before);
        // A later ordinary clean field edit is accepted; stale rejection is cleared.
        edit(&ctx, &mut a, "name", "retry");
        assert_eq!(
            session(&a).assignment_editor.drafts["/dev/vda"].name,
            "retry"
        );
        assert!(session(&a).current.disks["/dev/vda"].table.is_none());
    }
}
#[test]
fn progressive_help_is_explicit_persisted_and_isolated_from_worked_answers_and_other_app() {
    let (ctx, mut a, i) = opened();
    let (_, other, _) = opened();
    tap(&ctx, &mut a, "assignment");
    let source = a.scenarios[i].clone();
    let before = session(&a).current.clone();
    assert!(
        !a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|s| source.solution.contains(s))
    );
    tap(&ctx, &mut a, "help");
    assert_eq!(a.progress.attempts[&source.id].hints, 0);
    assert!(
        !a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|s| source.solution.contains(s))
    );
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(a.progress.attempts[&source.id].hints, 1);
    assert_eq!(session(&a).current, before);
    assert_eq!(other.progress.attempts[&source.id].hints, 0);
    tap(&ctx, &mut a, "worked");
    assert!(
        a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|s| source.solution.contains(s))
    );
    tap(&ctx, &mut a, "help");
    tap(&ctx, &mut a, "help");
    assert_eq!(a.progress.attempts[&source.id].hints, 1);
    a.save();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(loaded.attempts[&source.id].hints, 1);
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&a.progress.attempts).unwrap()
    );
    assert_eq!(session(&a).current, before);
}
#[test]
fn every_supported_scale_theme_and_width_uses_real_native_field_clips_and_keeps_whole_profile() {
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        for theme_id in ["Light", "Dark"] {
            for width in [760., 1600.] {
                let (ctx, mut a, _) = opened();
                tap(&ctx, &mut a, "assignment");
                a.scale = scale;
                a.theme_id = theme_id.into();
                theme::apply(&ctx, theme_id);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                let before = portable(&a);
                for name in ["name", "start", "end", "label", "uuid"] {
                    let r = reveal(&ctx, &mut a, name, true, width, 929.);
                    assert!(r.width() > 0. && r.height() > 0.);
                    let root = a
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    let field = a
                        .metrics
                        .partition_fs
                        .fields
                        .iter()
                        .find(|r| r.0 == name)
                        .unwrap();
                    assert!(smoke::smoke_control_visible(
                        r,
                        root.intersect(field.3),
                        ctx.pixels_per_point()
                    ));
                    let text = a
                        .metrics
                        .partition_fs
                        .field_text
                        .iter()
                        .find(|t| t.0 == name)
                        .unwrap();
                    assert!(
                        (text.2.center().y - r.center().y).abs() <= 1.5,
                        "Native field glyph must remain vertically centered"
                    );
                    let field_id = field.2;
                    click(&ctx, &mut a, r, width, 929.);
                    assert_eq!(ctx.memory(|m| m.focused()), Some(field_id));
                }
                assert_eq!(portable(&a), before);
            }
        }
    }
}
#[test]
fn native_reset_cancel_confirm_warm_continue_and_next_preserve_other_context() {
    let (ctx, mut a, i) = opened();
    let mut sibling = a.scenarios[i].clone();
    sibling.id = "private-disk-sibling".into();
    sibling.title = "Private disk sibling".into();
    let sibling_i = a.scenarios.len();
    a.scenarios.push(sibling.clone());
    a.units.last_mut().unwrap().labs.push(sibling.id.clone());
    solve_native(&ctx, &mut a);
    let id = a.scenarios[i].id.clone();
    let old = portable(&a);
    let full = session(&a).clone();
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    assert!(a.reset_pending);
    let cancel = a.metrics.reset_cancel.unwrap();
    click(&ctx, &mut a, cancel, 1200., 850.);
    assert!(!a.reset_pending);
    assert_eq!(portable(&a), old);
    // Shared Next opens the genuinely absent sibling and does not grant it credit.
    let mut next = None;
    for _ in 0..20 {
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let r = a.metrics.next_unit.unwrap();
        if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
            next = Some(r);
            break;
        }
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 2., root.center().y)),
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
    click(&ctx, &mut a, next.unwrap(), 1200., 850.);
    assert_eq!(a.selected, Some(sibling_i));
    assert!(!progress::current_solved(
        &a.progress.attempts[&sibling.id],
        &sibling
    ));
    assert_eq!(fixture::session(&a.progress.attempts[&id]), &full);
    a.save();
    let raw = std::fs::read(&a.path).unwrap();
    let loaded = progress::decode(&raw, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&a.progress.attempts).unwrap()
    );
    // Opening an already saved context uses the same source admission and retains
    // all disk/draft/practice state; only its legitimate recency changes.
    let sibling_before = serde_json::to_value(&a.progress.attempts[&sibling.id]).unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.back);
    assert!(a.selected.is_none());
    let warm_before = portable(&a);
    let at = a
        .metrics
        .card_verification
        .iter()
        .position(|r| r.0 == id)
        .unwrap();
    let continue_rect = a.metrics.card_actions[at].0;
    assert!(smoke::smoke_control_visible(
        continue_rect,
        a.metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect()),
        ctx.pixels_per_point()
    ));
    click(&ctx, &mut a, continue_rect, 1200., 850.);
    assert_eq!(a.selected, Some(i));
    let mut expected = warm_before;
    let clock = expected["portable"]["clock"]
        .as_u64()
        .unwrap()
        .saturating_add(1);
    expected["portable"]["clock"] = serde_json::json!(clock);
    expected["portable"]["attempts"][&id]["visited"] = serde_json::json!(clock);
    assert_eq!(portable(&a), expected);
    assert_eq!(session(&a), &full);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&sibling.id]).unwrap(),
        sibling_before
    );
    let before_reset = portable(&a);
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    accounts_ui_click(&ctx, &mut a, |m| m.confirm_reset);
    assert!(!a.reset_pending);
    let mut fresh = Attempt::new(&a.scenarios[i], a.progress.attempts[&id].visited);
    work::binding::prepare(&mut fresh, &a.scenarios[i]).unwrap();
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&id]).unwrap(),
        serde_json::to_value(&fresh).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&sibling.id]).unwrap(),
        before_reset["portable"]["attempts"][&sibling.id]
    );
    assert_eq!(
        a.progress.clock,
        before_reset["portable"]["clock"].as_u64().unwrap()
    );
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        before_reset["receipts"]
    );
    tap(&ctx, &mut a, "assignment");
    let field = reveal(&ctx, &mut a, "name", true, 1200., 850.);
    click(&ctx, &mut a, field, 1200., 850.);
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
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            })
            .collect(),
    );
    assert!(
        session(&a)
            .assignment_editor
            .drafts
            .values()
            .all(|d| d.name.is_empty())
    );
    assert_eq!(session(&a).current, fixture::definition().initial);
}
#[test]
fn packet_plus_help_or_held_apply_cannot_execute_and_clean_actions_explicitly_repaint() {
    for key in ["create-gpt", "check", "help"] {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        let target = reveal(&ctx, &mut a, key, false, 1200., 850.);
        let before = portable(&a);
        let mut events = vec![
            egui::Event::PointerMoved(target.center()),
            egui::Event::Paste("x".repeat(37)),
            enter(),
            egui::Event::PointerButton {
                pos: target.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ];
        frame(&ctx, &mut a, 1200., 850., std::mem::take(&mut events));
        assert_eq!(portable(&a), before);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerButton {
                pos: target.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        draw(&ctx, &mut a);
        assert_eq!(portable(&a), before);
        let r = reveal(&ctx, &mut a, key, false, 1200., 850.);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        let mut repaint_causes = Vec::new();
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 850.),
                )),
                events: [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::PointerButton {
                        pos: r.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    })
                    .collect(),
                ..Default::default()
            },
            |ctx| {
                // egui may perform several layout passes in one run. Its cause
                // list describes only the preceding pass, so observe each pass.
                repaint_causes.extend(ctx.repaint_causes());
                a.render(ctx);
            },
        );
        assert_eq!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            std::time::Duration::ZERO
        );
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 850.),
                )),
                ..Default::default()
            },
            |ctx| {
                repaint_causes.extend(ctx.repaint_causes());
                a.render(ctx);
            },
        );
        assert!(
            repaint_causes
                .iter()
                .any(|c| c.file.ends_with("src/ui/partition_fs_workspace.rs")),
            "{key} must explicitly repaint; causes {:?}, notice {:?}, pending {:?}, guidance {}",
            repaint_causes,
            a.notice,
            a.metrics.partition_fs.pending_action,
            a.metrics.partition_fs.guidance_intent
        );
        match key {
            "create-gpt" => assert!(session(&a).current.disks["/dev/vda"].table.is_some()),
            "check" => assert!(session(&a).checked),
            _ => assert!(
                a.metrics
                    .partition_fs
                    .controls
                    .iter()
                    .any(|r| r.0 == "show-hint")
            ),
        }
    }
}
#[test]
fn source_admission_precedes_launch_and_current_library_credit_rejects_joint_forgery() {
    let (ctx, mut a, i) = opened();
    solve_native(&ctx, &mut a);
    let source = a.scenarios[i].clone();
    let id = source.id.clone();
    assert!(a.nfs_visible_solved(&a.progress.attempts[&id], &source));
    let held = serde_json::to_value(&a.scenarios).unwrap();
    let State::PartitionFs { simulation } = &mut a.progress.attempts.get_mut(&id).unwrap().state
    else {
        panic!()
    };
    simulation.origin.plan.targets[0].minimum_bytes = 4 * fixture::MIB;
    simulation.validate().unwrap();
    assert!(!a.nfs_visible_solved(&a.progress.attempts[&id], &source));
    let before = portable(&a);
    let selected = a.selected;
    a.launch(i);
    assert_eq!(a.selected, selected);
    assert_eq!(portable(&a), before);
    assert!(a.notice.is_some());
    assert_eq!(serde_json::to_value(&a.scenarios).unwrap(), held);
    // No carried source replacement enters the application via warm saved work.
    let raw = serde_json::to_vec(&a.progress).unwrap();
    assert!(progress::decode(&raw, &a.scenarios).is_err());
}
#[test]
fn native_protection_states_type_hints_requirements_and_retry_feedback_use_plain_labels() {
    fn visible(ctx: &egui::Context, a: &mut LabApp) -> String {
        fn text(shape: &egui::Shape, clip: egui::Rect, out: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(t)
                    if egui::Rect::from_min_size(t.pos, t.galley.size()).intersects(clip) =>
                {
                    out.push(t.galley.job.text.clone())
                }
                egui::Shape::Vec(v) => {
                    for s in v {
                        text(s, clip, out);
                    }
                }
                _ => {}
            }
        }
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 850.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut out = Vec::new();
        for s in output.shapes {
            text(
                &s.shape,
                s.clip_rect.intersect(ctx.content_rect()),
                &mut out,
            );
        }
        out.join("\n")
    }
    fn near(ctx: &egui::Context, a: &mut LabApp, table: bool) -> String {
        for _ in 0..20 {
            let target = if table {
                a.metrics.partition_fs.table.unwrap()
            } else {
                a.metrics.partition_fs.rows[0].1
            };
            let root = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if smoke::smoke_control_visible(target, root, ctx.pixels_per_point()) {
                return visible(ctx, a);
            }
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(egui::pos2(
                    root.left() + 2.,
                    root.center().y,
                ))],
            );
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - target.center().y)
                            .clamp(-root.height() * 0.5, root.height() * 0.5),
                    ),
                    modifiers: Default::default(),
                }],
            );
            for _ in 0..10 {
                frame(ctx, a, 1200., 850., vec![]);
            }
        }
        panic!(
            "Friendly protection/feedback must fit its actual painted region within20 real wheels"
        )
    }
    let (ctx, mut a) = app();
    let mut d = fixture::definition();
    let spare = d.initial.disks.get_mut("/dev/vdd").unwrap();
    spare.writable = false;
    spare.in_use = true;
    spare.occupied_without_table = true;
    // This valid private source is configured before Start, never installed as a
    // learner result. Assignment reference/protection and separate practice stay real.
    let mut source = fixture::scenario();
    source.title = "Inspect protected virtual storage".into();
    source.goal = engine::Goal::PartitionFs {
        definition: Box::new(d.clone()),
    };
    source.initial = State::PartitionFs {
        simulation: Box::new(work::Session::new(&d).unwrap()),
    };
    let mut loaded =
        pack::load(&serde_json::json!({"schema_version":1,"scenarios":[source]}).to_string())
            .unwrap();
    let source = loaded.remove(0);
    let id = source.id.clone();
    let i = a.scenarios.len();
    a.scenarios.push(source);
    a.filter.track = "LPIC-1".into();
    a.filter.search = "Inspect protected virtual storage".into();
    draw(&ctx, &mut a);
    let at = a
        .metrics
        .card_verification
        .iter()
        .position(|r| r.0 == id)
        .unwrap();
    let start = a.metrics.card_actions[at].0;
    assert!(smoke::smoke_control_visible(
        start,
        a.metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect()),
        ctx.pixels_per_point()
    ));
    click(&ctx, &mut a, start, 1200., 850.);
    assert_eq!(a.selected, Some(i));
    tap(&ctx, &mut a, "assignment");
    choose(&ctx, &mut a, "disk", "/dev/vdd");
    let whole = portable(&a);
    let _ = reveal(&ctx, &mut a, "disk", false, 1200., 850.);
    let labels = near(&ctx, &mut a, true);
    for label in ["Read-only", "In use", "Existing data to preserve"] {
        assert!(
            labels.contains(label),
            "Actual painted protection text lacks {label}: {labels}"
        );
    }
    for raw in [
        "writable false",
        "in use true",
        "opaque data true",
        "LinuxData",
        "LinuxSwap",
    ] {
        assert!(
            !labels.contains(raw),
            "Raw state/enum leaked into native paint: {raw}"
        );
    }
    assert_eq!(portable(&a), whole);
    choose(&ctx, &mut a, "disk", "/dev/vda");
    tap(&ctx, &mut a, "hint");
    let choices = visible(&ctx, &mut a);
    assert!(choices.contains("Linux data") && choices.contains("Linux swap"));
    assert!(!choices.contains("LinuxData") && !choices.contains("LinuxSwap"));
    let choice = a
        .metrics
        .partition_fs
        .choices
        .iter()
        .find(|r| r.0 == "hint-LinuxSwap")
        .unwrap()
        .1;
    click(&ctx, &mut a, choice, 1200., 850.);
    tap(&ctx, &mut a, "create-gpt");
    edit(&ctx, &mut a, "name", "reports");
    edit(&ctx, &mut a, "start", "2048");
    edit(&ctx, &mut a, "end", "18431");
    tap(&ctx, &mut a, "add");
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics
            .partition_fs
            .rows
            .iter()
            .map(|r| r.0)
            .collect::<Vec<_>>(),
        vec![false, false, true]
    );
    let _ = reveal(&ctx, &mut a, "check", false, 1200., 850.);
    let feedback = near(&ctx, &mut a, false);
    assert!(feedback.contains("partition type hint is Linux swap; need Linux data"));
    assert!(feedback.contains("2 of 3 checks need attention."));
    assert_eq!(
        feedback
            .matches("partition type hint is Linux swap")
            .count(),
        1
    );
    assert!(feedback.contains("actual content is unformatted; need ext4 labeled \"reports\""));
    assert!(!feedback.contains("LinuxData") && !feedback.contains("LinuxSwap"));
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
}

#[path = "../../../tests/support/partition_fs_capacity.rs"]
mod capacity;
#[test]
fn refused_real_mode_change_preserves_prior_dirty_and_full_profile_then_retries() {
    fn mode(ctx: &egui::Context, a: &mut LabApp, challenge: bool) {
        for _ in 0..20 {
            let r = if challenge {
                a.metrics.mode_challenge.unwrap()
            } else {
                a.metrics.mode_learn.unwrap()
            };
            let root = a
                .metrics
                .main_viewport
                .unwrap()
                .intersect(ctx.content_rect());
            if smoke::smoke_control_visible(r, root, ctx.pixels_per_point()) {
                click(ctx, a, r, 1200., 850.);
                return;
            }
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(egui::pos2(
                    root.left() + 2.,
                    root.center().y,
                ))],
            );
            frame(
                ctx,
                a,
                1200.,
                850.,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - r.center().y)
                            .clamp(-root.height() * 0.75, root.height() * 0.75),
                    ),
                    modifiers: Default::default(),
                }],
            );
            for _ in 0..10 {
                draw(ctx, a);
            }
        }
        panic!("Actual mode header must be visible within20 real wheels");
    }
    for prior_dirty in [false, true] {
        let (ctx, mut a, i) = opened();
        solve_native(&ctx, &mut a);
        mode(&ctx, &mut a, true);
        let source = capacity::padded_source(a.scenarios[i].clone());
        let id = source.id.clone();
        a.scenarios[i] = source.clone();
        capacity::fill_to(
            &source,
            a.progress.attempts.get_mut(&id).unwrap(),
            work::MAX_CONTEXT,
        );
        a.dirty = prior_dirty;
        draw(&ctx, &mut a);
        let before = portable(&a);
        mode(&ctx, &mut a, false);
        assert_eq!(portable(&a), before);
        assert_eq!(a.dirty, prior_dirty);
        assert!(a.progress.attempts[&id].challenge);
        assert!(session(&a).checked && a.metrics.completion);
        assert!(a.notice.as_ref().unwrap().contains("mode change"));
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .transcript
            .remove(0);
        let mut expected = portable(&a);
        expected["portable"]["attempts"][&id]["challenge"] = false.into();
        mode(&ctx, &mut a, false);
        assert_eq!(portable(&a), expected);
        assert!(a.dirty && a.metrics.completion);
    }
}
#[test]
fn full_context_refuses_real_draft_and_hint_growth_without_losing_native_undo_or_current_success() {
    type UndoTrace = (
        Vec<(egui::text::CCursorRange, String)>,
        Vec<(egui::text::CCursorRange, String)>,
        bool,
    );
    fn trace(state: &egui::text_edit::TextEditState, text: &str) -> UndoTrace {
        let current = (state.cursor.char_range().unwrap(), text.to_owned());
        let mut u = state.undoer();
        let mut now = current.clone();
        let mut past = vec![];
        while let Some(v) = u.undo(&now).cloned() {
            now = v.clone();
            past.push(v);
            assert!(past.len() <= 100);
        }
        let mut u = state.undoer();
        let mut now = current;
        let mut future = vec![];
        while let Some(v) = u.redo(&now).cloned() {
            now = v.clone();
            future.push(v);
            assert!(future.len() <= 100);
        }
        (past, future, state.undoer().is_in_flux())
    }
    let (ctx, mut a, i) = opened();
    solve_native(&ctx, &mut a);
    let source = capacity::padded_source(a.scenarios[i].clone());
    let id = source.id.clone();
    a.scenarios[i] = source.clone();
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    attempt.hints = 9;
    capacity::fill_to(&source, attempt, work::MAX_CONTEXT);
    a.dirty = false;
    draw(&ctx, &mut a);
    let r = reveal(&ctx, &mut a, "name", true, 1200., 850.);
    click(&ctx, &mut a, r, 1200., 850.);
    let field = a
        .metrics
        .partition_fs
        .fields
        .iter()
        .find(|r| r.0 == "name")
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(field));
    frame(&ctx, &mut a, 1200., 850., select_all());
    let native = egui::text_edit::TextEditState::load(&ctx, field).unwrap();
    let cursor = native.cursor.char_range();
    let text = session(&a).assignment_editor.drafts["/dev/vda"]
        .name
        .clone();
    let undo = trace(&native, &text);
    let before = portable(&a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Paste(format!("{text}x"))],
    );
    draw(&ctx, &mut a);
    assert_eq!(portable(&a), before);
    assert!(!a.dirty);
    assert!(a.metrics.completion && session(&a).checked);
    let native = egui::text_edit::TextEditState::load(&ctx, field).unwrap();
    assert_eq!(native.cursor.char_range(), cursor);
    assert_eq!(trace(&native, &text), undo);
    assert!(a.notice.as_ref().unwrap().contains("no room"));
    tap(&ctx, &mut a, "help");
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(portable(&a), before);
    assert_eq!(a.progress.attempts[&id].hints, 9);
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .transcript
        .remove(0);
    edit(&ctx, &mut a, "name", &format!("{text}x"));
    assert_eq!(
        session(&a).assignment_editor.drafts["/dev/vda"].name,
        format!("{text}x")
    );
    assert!(a.metrics.completion);
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(a.progress.attempts[&id].hints, 10);
}
#[test]
fn every_disclosure_press_blocks_actual_focused_enter_actions_instant_and_held_then_allows_retry() {
    fn focus(ctx: &egui::Context, a: &mut LabApp, key: &str) {
        for _ in 0..96 {
            let target = if key == "next" {
                a.metrics.next_unit.unwrap()
            } else {
                control_rect(a, key, false).0
            };
            if ctx
                .memory(|m| m.focused())
                .and_then(|id| ctx.read_response(id))
                .is_some_and(|r| r.rect == target)
            {
                return;
            }
            frame(
                ctx,
                a,
                1200.,
                850.,
                [true, false]
                    .into_iter()
                    .map(|pressed| egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    })
                    .collect(),
            );
            draw(ctx, a);
        }
        panic!("Real Tab must focus{key} within96 controls");
    }
    for header in ["inspect", "help", "worked", "references"] {
        for action in ["create-gpt", "add", "format", "check", "next"] {
            for held in [false, true] {
                for paste in [false, true] {
                    let (ctx, mut a, i) = opened();
                    tap(&ctx, &mut a, "assignment");
                    if ["add", "format"].contains(&action) {
                        tap(&ctx, &mut a, "create-gpt");
                        edit(&ctx, &mut a, "name", "reports");
                        edit(&ctx, &mut a, "start", "2048");
                        edit(&ctx, &mut a, "end", "18431");
                        if action == "format" {
                            tap(&ctx, &mut a, "add");
                            edit(&ctx, &mut a, "label", "reports");
                        }
                    }
                    if action == "next" {
                        let mut sibling = a.scenarios[i].clone();
                        sibling.id = "private-disk-guidance-sibling".into();
                        let id = sibling.id.clone();
                        a.scenarios.push(sibling);
                        a.units.last_mut().unwrap().labs.push(id);
                        solve_native(&ctx, &mut a);
                        assert!(a.metrics.next_unit.is_some());
                    }
                    if ["worked", "references"].contains(&header) {
                        tap(&ctx, &mut a, "help");
                    }
                    if paste {
                        let r = reveal(&ctx, &mut a, "name", true, 1200., 850.);
                        click(&ctx, &mut a, r, 1200., 850.);
                        let field = a
                            .metrics
                            .partition_fs
                            .fields
                            .iter()
                            .find(|r| r.0 == "name")
                            .unwrap()
                            .2;
                        assert_eq!(ctx.memory(|m| m.focused()), Some(field));
                    } else {
                        focus(&ctx, &mut a, action);
                    }
                    let target = reveal(&ctx, &mut a, header, false, 1200., 850.);
                    frame(
                        &ctx,
                        &mut a,
                        1200.,
                        850.,
                        vec![egui::Event::PointerMoved(target.center())],
                    );
                    let before = portable(&a);
                    let key = |pressed| egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    };
                    let pointer = |pressed| egui::Event::PointerButton {
                        pos: target.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    };
                    let mut events = vec![];
                    if paste {
                        events.push(egui::Event::Paste("a".into()));
                    }
                    events.extend([key(true), pointer(true)]);
                    if !held {
                        events.extend([pointer(false), key(false)]);
                    }
                    frame(&ctx, &mut a, 1200., 850., events);
                    if !paste {
                        assert!(
                            a.metrics.partition_fs.guidance_intent,
                            "Actual{header} intent must be present"
                        );
                    }
                    if !paste && action != "next" {
                        assert_eq!(
                            a.metrics.partition_fs.pending_action,
                            Some(action),
                            "Actual focused{action} keyboard intent must reach the gate"
                        );
                    }
                    if action == "next" {
                        assert!(a.metrics.next_unit.is_none());
                    }
                    assert_eq!(portable(&a), before, "{header}/{action}/{held}/{paste}");
                    assert_eq!(a.selected, Some(i));
                    if held {
                        frame(&ctx, &mut a, 1200., 850., vec![pointer(false), key(false)]);
                        assert_eq!(portable(&a), before);
                        assert_eq!(a.selected, Some(i));
                    }
                    draw(&ctx, &mut a);
                    assert_eq!(portable(&a), before);
                    if action == "next" {
                        assert!(a.metrics.next_unit.is_some());
                        assert_eq!(a.progress.attempts.len(), 1);
                    } else {
                        tap(&ctx, &mut a, action);
                        assert!(
                            a.notice.is_none(),
                            "Clean real{action} retry must work: {:?}",
                            a.notice
                        );
                        assert_ne!(portable(&a), before);
                    }
                }
            }
        }
    }
}

#[test]
fn ordinary_split_pointer_hints_and_all_headers_remain_enabled_until_their_real_release() {
    fn split(ctx: &egui::Context, a: &mut LabApp, key: &str) {
        let r = reveal(ctx, a, key, false, 1200., 850.);
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        let before = portable(a);
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }],
        );
        assert!(
            a.metrics.partition_fs.guidance_intent,
            "Real{key} press must be observed"
        );
        assert_eq!(
            portable(a),
            before,
            "A plain press is not a completed hint/action"
        );
        assert!(
            !a.partition_fs_presentation.held_for_test(),
            "A noncompeting press must not disable its own release"
        );
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        for _ in 0..3 {
            draw(ctx, a);
        }
    }
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    let id = a.scenarios[i].id.clone();
    let before = portable(&a);
    split(&ctx, &mut a, "help");
    assert_eq!(portable(&a), before);
    assert!(
        a.metrics
            .partition_fs
            .controls
            .iter()
            .any(|r| r.0 == "show-hint")
    );
    assert!(
        !a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|t| t == &a.scenarios[i].hints[0])
    );
    split(&ctx, &mut a, "show-hint");
    let mut expected = before.clone();
    expected["portable"]["attempts"][&id]["hints"] = 1.into();
    assert_eq!(portable(&a), expected);
    assert!(
        a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|t| t == &a.scenarios[i].hints[0])
    );
    let Goal::PartitionFs { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let worked = definition.reference[0].description();
    assert!(!a.metrics.partition_fs.painted_text.contains(&worked));
    split(&ctx, &mut a, "worked");
    assert_eq!(portable(&a), expected);
    assert!(a.metrics.partition_fs.painted_text.contains(&worked));
    split(&ctx, &mut a, "worked");
    assert!(!a.metrics.partition_fs.painted_text.contains(&worked));
    split(&ctx, &mut a, "references");
    assert_eq!(portable(&a), expected);
    assert!(
        a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|t| t == work::HELP)
    );
    split(&ctx, &mut a, "references");
    split(&ctx, &mut a, "help");
    assert_eq!(portable(&a), expected);
    split(&ctx, &mut a, "inspect");
    assert_eq!(portable(&a), expected);
    assert!(
        a.metrics
            .partition_fs
            .painted_text
            .iter()
            .any(|t| t.contains("/keep.txt"))
    );
    split(&ctx, &mut a, "inspect");
    assert_eq!(portable(&a), expected);
    assert_eq!(a.selected, Some(i));
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
}

#[test]
fn delivered_valid_paste_plus_disclosure_press_rolls_back_full_draft_cursor_undo_then_clean_edit_succeeds()
 {
    type UndoTrace = (
        Vec<(egui::text::CCursorRange, String)>,
        Vec<(egui::text::CCursorRange, String)>,
        bool,
    );
    fn trace(state: &egui::text_edit::TextEditState, text: &str) -> UndoTrace {
        let current = (state.cursor.char_range().unwrap(), text.to_owned());
        let mut u = state.undoer();
        let mut now = current.clone();
        let mut past = vec![];
        while let Some(v) = u.undo(&now).cloned() {
            now = v.clone();
            past.push(v);
            assert!(past.len() <= 100);
        }
        let mut u = state.undoer();
        let mut now = current;
        let mut future = vec![];
        while let Some(v) = u.redo(&now).cloned() {
            now = v.clone();
            future.push(v);
            assert!(future.len() <= 100);
        }
        (past, future, state.undoer().is_in_flux())
    }
    // A separate press frame has no pointer click yet: the real focused field
    // receives Paste under egui's default surrender-on-click policy. A batched
    // press/release could instead lose focus and make this premise vacuous.
    for header in ["inspect", "help", "worked", "references", "show-hint"] {
        let (ctx, mut a, i) = opened();
        tap(&ctx, &mut a, "assignment");
        if ["worked", "references", "show-hint"].contains(&header) {
            tap(&ctx, &mut a, "help");
        }
        edit(&ctx, &mut a, "name", "base");
        let field = reveal(&ctx, &mut a, "name", true, 1200., 850.);
        click(&ctx, &mut a, field, 1200., 850.);
        frame(&ctx, &mut a, 1200., 850., select_all());
        // Reveal the real header again using ordinary wheel events, retaining focus.
        let target = reveal(&ctx, &mut a, header, false, 1200., 850.);
        let field_id = a
            .metrics
            .partition_fs
            .fields
            .iter()
            .find(|r| r.0 == "name")
            .unwrap()
            .2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(field_id));
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(target.center())],
        );
        let native = egui::text_edit::TextEditState::load(&ctx, field_id).unwrap();
        let cursor = native.cursor.char_range();
        let undo = trace(&native, "base");
        let before = portable(&a);
        let prior_dirty = a.dirty;
        let events = vec![
            egui::Event::Paste("change".into()),
            egui::Event::PointerButton {
                pos: target.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ];
        assert!(!events.iter().any(|e| matches!(
            e,
            egui::Event::Key {
                key: egui::Key::Enter,
                ..
            }
        )));
        frame(&ctx, &mut a, 1200., 850., events);
        assert_eq!(
            a.metrics.partition_fs.received_edit_bytes, 6,
            "A real valid Paste must reach the field"
        );
        assert!(
            a.metrics.partition_fs.staged_draft_changed,
            "A genuinely changed draft must reach the shared guidance gate"
        );
        assert!(a.metrics.partition_fs.guidance_intent && a.metrics.partition_fs.edit_vetoed);
        assert_eq!(portable(&a), before);
        assert_eq!(a.dirty, prior_dirty);
        let native = egui::text_edit::TextEditState::load(&ctx, field_id).unwrap();
        assert_eq!(native.cursor.char_range(), cursor);
        assert_eq!(trace(&native, "base"), undo);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerButton {
                pos: target.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        for _ in 0..3 {
            draw(&ctx, &mut a);
        }
        assert_eq!(portable(&a), before);
        assert_eq!(a.selected, Some(i));
        let native = egui::text_edit::TextEditState::load(&ctx, field_id).unwrap();
        assert_eq!(native.cursor.char_range(), cursor);
        assert_eq!(trace(&native, "base"), undo);
        edit(&ctx, &mut a, "name", "change");
        assert_eq!(
            session(&a).assignment_editor.drafts["/dev/vda"].name,
            "change"
        );
        assert_eq!(session(&a).current, fixture::definition().initial);
        assert!(!progress::current_solved(
            &a.progress.attempts[&a.scenarios[i].id],
            &a.scenarios[i]
        ));
    }
}

#[test]
fn real_nontext_backspace_and_cut_edits_veto_old_held_add_release_then_clean_add_works() {
    fn key(key: egui::Key, modifiers: egui::Modifiers) -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            })
            .collect()
    }
    for cut in [false, true] {
        let (ctx, mut a, i) = opened();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "create-gpt");
        let original = if cut { "reportsbad" } else { "reportsx" };
        edit(&ctx, &mut a, "name", original);
        edit(&ctx, &mut a, "start", "2048");
        edit(&ctx, &mut a, "end", "18431");
        let r = reveal(&ctx, &mut a, "name", true, 1200., 850.);
        click(&ctx, &mut a, r, 1200., 850.);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            key(egui::Key::End, Default::default()),
        );
        if cut {
            for _ in 0..3 {
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    850.,
                    key(
                        egui::Key::ArrowLeft,
                        egui::Modifiers {
                            shift: true,
                            ..Default::default()
                        },
                    ),
                );
            }
        }
        let target = reveal(&ctx, &mut a, "add", false, 1200., 850.);
        let id = a.scenarios[i].id.clone();
        let field = a
            .metrics
            .partition_fs
            .fields
            .iter()
            .find(|r| r.0 == "name")
            .unwrap()
            .2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(field));
        let range = egui::text_edit::TextEditState::load(&ctx, field)
            .unwrap()
            .cursor
            .char_range()
            .unwrap()
            .as_sorted_char_range();
        assert_eq!(range.end - range.start, if cut { 3 } else { 0 });
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(target.center())],
        );
        a.dirty = false;
        let before = portable(&a);
        let before_model = session(&a).current.clone();
        assert!(
            before_model.disks["/dev/vda"]
                .table
                .as_ref()
                .unwrap()
                .partitions
                .is_empty()
        );
        let mut events = if cut {
            vec![egui::Event::Cut]
        } else {
            key(egui::Key::Backspace, Default::default())
        };
        events.push(egui::Event::PointerButton {
            pos: target.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        );
        frame(&ctx, &mut a, 1200., 850., events);
        assert_eq!(
            a.metrics.partition_fs.received_edit_bytes, 0,
            "This genuine edit has no text packet"
        );
        assert!(
            a.metrics.partition_fs.staged_draft_changed,
            "The real deletion must change the staged draft"
        );
        assert!(a.partition_fs_presentation.held_for_test());
        let mut expected = before;
        expected["portable"]["attempts"][&id]["state"]["simulation"]["assignment_editor"]["drafts"]
            ["/dev/vda"]["name"] = "reports".into();
        assert_eq!(portable(&a), expected);
        assert!(a.dirty);
        assert_eq!(session(&a).current, before_model);
        assert!(!session(&a).checked);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerButton {
                pos: target.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        );
        for _ in 0..3 {
            draw(&ctx, &mut a);
        }
        assert_eq!(
            portable(&a),
            expected,
            "Old held Add release cannot allocate using the changed draft"
        );
        assert_eq!(session(&a).current, before_model);
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
        assert!(!session(&a).checked);
        tap(&ctx, &mut a, "add");
        assert!(
            a.notice.is_none(),
            "Clean later Add must operate: {:?}",
            a.notice
        );
        let parts = &session(&a).current.disks["/dev/vda"]
            .table
            .as_ref()
            .unwrap()
            .partitions;
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "reports");
        assert_eq!(parts[0].start, 2048);
        assert_eq!(parts[0].end, 18431);
        assert!(
            !progress::current_solved(&a.progress.attempts[&id], &a.scenarios[i]),
            "An allocated but unformatted first target is a partial result, not invented mastery"
        );
    }
}
