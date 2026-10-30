//! Prospective real-egui interaction, not installed/native or novice acceptance.
use super::*;
use dlms_learning_lab::ntp_workspace as work;
const NTP_IDS: [&str; 4] = [
    "ccna-ntp-client-chain",
    "ccna-ntp-cycle-repair",
    "ccna-ntp-asymmetric-delivery",
    "ccna-ntp-policy-handover",
];
fn opened_at(offset: usize) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert!(!a.synthetic);
    a.filter.track = "CCNA".into();
    a.filter.unit = "ntp-client-server".into();
    draw(&ctx, &mut a);
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == NTP_IDS[offset])
        .unwrap();
    let mut start = None;
    for _ in 0..20 {
        let position = a
            .metrics
            .card_verification
            .iter()
            .position(|r| r.0 == NTP_IDS[offset])
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
    assert!(a.progress.attempts.contains_key(NTP_IDS[offset]));
    (ctx, a, i)
}
fn opened() -> (egui::Context, LabApp, usize) {
    opened_at(0)
}
fn session(a: &LabApp) -> &work::Session {
    let engine::State::Ntp { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    for _ in 0..20 {
        let (r, clip) = if field {
            let f = a.metrics.ntp.fields.iter().find(|f| f.0 == key).unwrap();
            (f.1, f.3)
        } else {
            let c = a.metrics.ntp.controls.iter().find(|c| c.0 == key).unwrap();
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
    panic!("Actual NTP control {key} did not become visible in20 real wheels")
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
        .ntp
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

fn select_device(ctx: &egui::Context, a: &mut LabApp, ip: &str) {
    tap(ctx, a, "device");
    let (rect, clip) = a
        .metrics
        .ntp
        .choices
        .iter()
        .find(|c| c.0 == ip)
        .map(|c| (c.1, c.2))
        .unwrap();
    assert!(smoke::smoke_control_visible(
        rect,
        clip,
        ctx.pixels_per_point()
    ));
    click_real(ctx, a, rect);
    assert_eq!(session(a).selected.to_string(), ip);
}
#[test]
fn real_intro_separate_inert_practice_actual_device_selection_full_assignment_and_challenge() {
    for offset in 0..4 {
        let (ctx, mut a, i) = opened_at(offset);
        assert_eq!(session(&a).view, teaching::View::Introduction);
        let initial = session(&a).current.clone();
        tap(&ctx, &mut a, "example");
        let example = session(&a).example.clone();
        tap(&ctx, &mut a, "insert");
        assert_eq!(session(&a).example, example);
        assert!(session(&a).example_transcript.is_empty());
        tap(&ctx, &mut a, "run");
        assert_eq!(session(&a).current, initial);
        assert_eq!(session(&a).example_transcript.len(), 1);
        assert!(!a.metrics.completion);
        tap(&ctx, &mut a, "assignment");
        for c in a.scenarios[i].solution.clone() {
            if let Some(ip) = c.strip_prefix("SIM select ") {
                select_device(&ctx, &mut a, ip);
            } else {
                type_command(&ctx, &mut a, &c);
                tap(&ctx, &mut a, "run");
            }
        }
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert_eq!(
            a.metrics.ntp.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![true; [4, 4, 5, 7][offset]]
        );
        assert_eq!(session(&a).example, example);
        let before = serde_json::to_value(&a.progress).unwrap();
        accounts_ui_click(&ctx, &mut a, |m| m.mode_challenge);
        draw(&ctx, &mut a);
        assert!(a.progress.attempts[&a.scenarios[i].id].challenge);
        assert!(a.metrics.ntp.goal.is_some());
        assert!(
            !a.metrics
                .ntp
                .controls
                .iter()
                .any(|c| c.0 == "insert" || c.0 == "example")
        );
        let mut expected = before;
        expected["attempts"][&a.scenarios[i].id]["challenge"] = true.into();
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), expected);
    }
}
#[test]
fn full_oversized_or_late_paste_with_enter_or_run_refuses_old_command_and_preserves_cursor() {
    for mode in 0..4 {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        type_command(&ctx, &mut a, "show ntp status");
        let field = target(&ctx, &mut a, "command", true);
        click_real(&ctx, &mut a, field);
        let run = target(&ctx, &mut a, "run", false);
        let id = a.metrics.ntp.fields[0].2;
        let before = serde_json::to_value(&a.progress).unwrap();
        let native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        let cursor = native.cursor.char_range();
        let undo = format!("{:?}", native.undoer());
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
        let mut events = match mode {
            0 => vec![egui::Event::Paste(format!(
                "ntp server 10.71.0.101{} unsupported",
                " ".repeat(512)
            ))],
            1 => vec![egui::Event::Paste("é".repeat(512))],
            2 => {
                let mut e = enter();
                e.push(egui::Event::Paste("ntp server 10.71.0.101".into()));
                e
            }
            _ => vec![
                egui::Event::Paste(" ".repeat(300)),
                egui::Event::Paste(" ".repeat(300)),
            ],
        };
        if mode == 0 {
            events.push(egui::Event::PointerMoved(run.center()));
            for pressed in [true, false] {
                events.push(egui::Event::PointerButton {
                    pos: run.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                });
            }
        } else if mode != 2 {
            events.extend(enter());
        }
        frame(&ctx, &mut a, 1200., 850., events);
        let native = egui::text_edit::TextEditState::load(&ctx, id).unwrap();
        assert_eq!(native.cursor.char_range(), cursor);
        assert_eq!(format!("{:?}", native.undoer()), undo);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        draw(&ctx, &mut a);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
        assert!(session(&a).transcript.is_empty());
        assert!(!a.metrics.completion);
    }
}
#[test]
fn native_wrong_command_retry_clears_error_and_reset_cancel_confirm_preserve_other_work() {
    let (ctx, mut a, i) = opened();
    // Populate another activity through normal app preparation and real editing.
    a.launch(i + 1);
    draw(&ctx, &mut a);
    tap(&ctx, &mut a, "assignment");
    type_command(&ctx, &mut a, "show running-config");
    a.launch(i);
    draw(&ctx, &mut a);
    tap(&ctx, &mut a, "assignment");
    type_command(&ctx, &mut a, "ntp master 1");
    tap(&ctx, &mut a, "run");
    assert!(a.notice.is_some());
    type_command(&ctx, &mut a, "show ntp status");
    tap(&ctx, &mut a, "run");
    assert!(a.notice.is_none());
    let before = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    draw(&ctx, &mut a);
    assert!(a.metrics.reset_cancel.is_some());
    accounts_ui_click(&ctx, &mut a, |m| m.reset_cancel);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    accounts_ui_click(&ctx, &mut a, |m| m.reset);
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.confirm_reset);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&a.scenarios[i].id].state,
        a.scenarios[i].initial
    );
    let mut expected = before;
    expected["attempts"]
        .as_object_mut()
        .unwrap()
        .remove(&a.scenarios[i].id);
    let mut actual = serde_json::to_value(&a.progress).unwrap();
    actual["attempts"]
        .as_object_mut()
        .unwrap()
        .remove(&a.scenarios[i].id);
    // Reset retains the genuine visit/clock and all other profile data.
    assert_eq!(actual, expected);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
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
    assert!(session(&a).command.is_empty());
    assert!(session(&a).transcript.is_empty());
    assert!(!a.metrics.completion);
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
                    let (_, field, _, clip) = a.metrics.ntp.fields[0];
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
                let (_, field, id, _) = a.metrics.ntp.fields[0];
                let root = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                let run = a
                    .metrics
                    .ntp
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
                    vec![egui::Event::Paste("show ntp status".into())],
                );
                assert_eq!(session(&a).command, "show ntp status");
                assert!(session(&a).transcript.is_empty());
                assert!(!a.metrics.completion);
            }
        }
    }
}

#[test]
fn forged_initial_and_current_world_is_refused_before_open_visit_or_library_credit() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    for c in a.scenarios[i].solution.clone() {
        if let Some(ip) = c.strip_prefix("SIM select ") {
            select_device(&ctx, &mut a, ip);
        } else {
            type_command(&ctx, &mut a, &c);
            tap(&ctx, &mut a, "run");
        }
    }
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let id = a.scenarios[i].id.clone();
    let engine::State::Ntp { simulation } = &mut a.progress.attempts.get_mut(&id).unwrap().state
    else {
        panic!()
    };
    // Explicit untrusted-state injection is a negative boundary control only.
    for world in [&mut simulation.initial, &mut simulation.current] {
        world
            .nodes
            .get_mut(&"10.71.0.104".parse().unwrap())
            .unwrap()
            .role = dlms_learning_lab::ntp::Role::Reference {
            stratum: 2,
            healthy: true,
        };
    }
    simulation.validate().unwrap();
    let before = serde_json::to_value(&a.progress).unwrap();
    let selected = a.selected;
    assert!(!a.nfs_visible_solved(&a.progress.attempts[&id], &a.scenarios[i]));
    a.launch(i);
    assert!(a.notice.is_some());
    assert_eq!(a.selected, selected);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    draw(&ctx, &mut a);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert!(!a.metrics.completion);
}

#[test]
fn insert_and_view_navigation_with_same_frame_enter_do_not_dispatch_a_command() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "example");
    type_command(&ctx, &mut a, "ntp server 10.81.0.101");
    for key in ["insert", "assignment"] {
        let button = target(&ctx, &mut a, key, false);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(button.center())],
        );
        let before = session(&a).example.clone();
        let current = session(&a).current.clone();
        let mut events = Vec::new();
        for pressed in [true, false] {
            events.push(egui::Event::PointerButton {
                pos: button.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            });
        }
        for pressed in [true, false] {
            events.push(egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            });
        }
        frame(&ctx, &mut a, 1200., 850., events);
        draw(&ctx, &mut a);
        assert_eq!(session(&a).example, before);
        assert_eq!(session(&a).current, current);
        assert!(session(&a).transcript.is_empty());
        assert!(session(&a).example_transcript.is_empty());
        assert!(!a.metrics.completion);
    }
    assert_eq!(session(&a).view, teaching::View::Assignment);
}

fn sized_compact_frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    w: f32,
    h: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w / a.scale, h / a.scale),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    )
}
fn reveal_sized(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, key: &str) -> egui::Rect {
    for _ in 0..20 {
        let (r, clip) = if key == "topology" {
            a.metrics.ntp.topology.unwrap()
        } else if key == "command" {
            let f = &a.metrics.ntp.fields[0];
            (f.1, f.3)
        } else {
            let c = a.metrics.ntp.controls.iter().find(|c| c.0 == key).unwrap();
            (c.1, c.2)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip.intersect(root), ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            w,
            h,
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
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("Actual compact NTP {key} not visible within20 real settled wheels")
}
fn click_sized(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, r: egui::Rect) {
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
    frame(ctx, a, w, h, vec![]);
}
fn painted_pairs(
    shape: &egui::epaint::Shape,
    bounds: egui::Rect,
    cards: &mut Vec<egui::Rect>,
    lines: &mut Vec<[egui::Pos2; 2]>,
) {
    match shape {
        egui::epaint::Shape::Vec(v) => {
            for s in v {
                painted_pairs(s, bounds, cards, lines)
            }
        }
        egui::epaint::Shape::Rect(r)
            if bounds.contains_rect(r.rect)
                && r.rect.width() > 80.
                && r.rect.height() > 30.
                && r.stroke.width > 0. =>
        {
            cards.push(r.rect)
        }
        egui::epaint::Shape::LineSegment { points, .. }
            if points.iter().all(|p| bounds.contains(*p)) =>
        {
            lines.push(*points)
        }
        _ => {}
    }
}
#[test]
fn compact_pair_paints_equal_cards_centered_connectors_and_adjacent_tools_at_all_supported_geometry()
 {
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for name in ["dlms-dark", "dlms-light"] {
            for width in [760., 1600.] {
                let (ctx, mut a, _) = opened_at(2);
                tap(&ctx, &mut a, "assignment");
                a.scale = scale;
                a.theme_id = name.into();
                theme::apply(&ctx, name);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                let before = session(&a).current.clone();
                let example = session(&a).example.clone();
                let field = reveal_sized(&ctx, &mut a, width, 929., "command");
                let run = a
                    .metrics
                    .ntp
                    .controls
                    .iter()
                    .find(|c| c.0 == "run")
                    .unwrap()
                    .1;
                let check = a
                    .metrics
                    .ntp
                    .controls
                    .iter()
                    .find(|c| c.0 == "check")
                    .unwrap()
                    .1;
                assert!(run.top() >= field.bottom() - 1. && run.top() <= field.bottom() + 16.);
                assert!(check.top() <= run.bottom() + 16.);
                assert!((field.height() - run.height()).abs() < 1.);
                assert!(a.metrics.ntp.inventory.is_none());
                reveal_sized(&ctx, &mut a, width, 929., "topology");
                let out = sized_compact_frame(&ctx, &mut a, width, 929., vec![]);
                let (rect, clip) = a.metrics.ntp.topology.unwrap();
                assert!(rect.left() >= clip.left() - 1. && rect.right() <= clip.right() + 1.);
                let mut cards = Vec::new();
                let mut lines = Vec::new();
                for s in out.shapes {
                    painted_pairs(&s.shape, rect, &mut cards, &mut lines);
                }
                assert_eq!(
                    cards.len(),
                    2,
                    "Actual painted cards {scale}/{name}/{width}: {cards:?}"
                );
                assert_eq!(lines.len(), 1, "Actual painted connector count");
                assert!((cards[0].width() - cards[1].width()).abs() < 0.1);
                assert!((cards[0].height() - cards[1].height()).abs() < 0.1);
                let expected = if (cards[0].center().y - cards[1].center().y).abs() > 0.1 {
                    [cards[0].center_bottom(), cards[1].center_top()]
                } else {
                    [cards[0].right_center(), cards[1].left_center()]
                };
                assert!(
                    (lines[0][0] - expected[0]).length() < 0.1
                        && (lines[0][1] - expected[1]).length() < 0.1
                );
                assert!(a.metrics.ntp.evidence.iter().any(|(t, _)| {
                    t.contains("request delivered; reply not delivered; UDP123 allowed")
                }));
                let field = reveal_sized(&ctx, &mut a, width, 929., "command");
                click_sized(&ctx, &mut a, width, 929., field);
                let id = a.metrics.ntp.fields[0].2;
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                frame(
                    &ctx,
                    &mut a,
                    width,
                    929.,
                    vec![egui::Event::Paste("show ntp status".into())],
                );
                assert_eq!(session(&a).command, "show ntp status");
                assert_eq!(session(&a).current, before);
                assert_eq!(session(&a).example, example);
                let cursor = egui::text_edit::TextEditState::load(&ctx, id)
                    .unwrap()
                    .cursor
                    .char_range();
                let resized = if width == 760. { 1600. } else { 760. };
                for _ in 0..4 {
                    frame(&ctx, &mut a, resized, 929., vec![]);
                }
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                assert_eq!(
                    egui::text_edit::TextEditState::load(&ctx, id)
                        .unwrap()
                        .cursor
                        .char_range(),
                    cursor
                );
                assert_eq!(session(&a).command, "show ntp status");
                assert_eq!(session(&a).current, before);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                let r = reveal_sized(&ctx, &mut a, width, 929., "check");
                click_sized(&ctx, &mut a, width, 929., r);
                let bottom = a
                    .metrics
                    .ntp
                    .controls
                    .iter()
                    .filter(|c| c.0 == "run" || c.0 == "check")
                    .map(|c| c.1.bottom())
                    .fold(f32::MIN, f32::max);
                assert!(a.metrics.ntp.rows[0].1.top() <= bottom + 16.);
                assert_eq!(
                    a.metrics.ntp.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
                    vec![false, false, false, false, true]
                );
                assert!(a.metrics.ntp.inventory.is_none());
                assert_eq!(session(&a).current, before);
            }
        }
    }
}
#[test]
fn default_compact_evidence_exposes_remote_reply_failure_cycle_and_allowed_source_addresses() {
    for offset in [1, 2] {
        let (ctx, mut a, _) = opened_at(offset);
        tap(&ctx, &mut a, "assignment");
        let before = session(&a).current.clone();
        let current = serde_json::to_value(&a.progress).unwrap();
        assert!(a.metrics.ntp.inventory.is_none());
        assert!(
            a.metrics
                .ntp
                .evidence
                .iter()
                .any(|(t, _)| t.contains("Permitted upstreams")
                    && t.contains(if offset == 1 {
                        "10.72.0.101"
                    } else {
                        "10.73.0.102"
                    }))
        );
        if offset == 1 {
            assert!(
                a.metrics
                    .ntp
                    .evidence
                    .iter()
                    .any(|(t, _)| t.contains("10.72.0.1 -> 10.72.0.2 -> 10.72.0.1"))
            );
        } else {
            select_device(&ctx, &mut a, "10.73.0.3");
            assert!(a.metrics.ntp.evidence.iter().any(|(t, _)| {
                t.contains("10.73.0.3 -> 10.73.0.1: request delivered; reply delivered")
            }));
            assert!(a.metrics.ntp.evidence.iter().any(|(t, _)| {
                t.contains("10.73.0.1 -> 10.73.0.101: request delivered; reply not delivered")
            }));
            assert!(
                a.metrics
                    .ntp
                    .evidence
                    .iter()
                    .any(|(t, _)| t.contains("10.73.0.3 -> 10.73.0.1 -> 10.73.0.101"))
            );
            select_device(&ctx, &mut a, "10.73.0.1");
        }
        assert_eq!(session(&a).current, before);
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), current);
        tap(&ctx, &mut a, "inspection");
        assert!(a.metrics.ntp.inventory.is_some());
        assert_eq!(
            a.metrics
                .ntp
                .inspection_nodes
                .iter()
                .map(|n| n.0.clone())
                .collect::<Vec<_>>(),
            session(&a)
                .current
                .nodes
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        tap(&ctx, &mut a, "all-facts");
        assert_eq!(
            a.metrics.ntp.inspection_facts.len(),
            session(&a).current.deliveries.len()
        );
        assert_eq!(serde_json::to_value(&a.progress).unwrap(), current);
    }
}

#[test]
fn shared_next_refuses_same_frame_unapplied_text_then_navigates_to_real_unsolved_sibling() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    for c in a.scenarios[i].solution.clone() {
        if let Some(ip) = c.strip_prefix("SIM select ") {
            select_device(&ctx, &mut a, ip);
        } else {
            type_command(&ctx, &mut a, &c);
            tap(&ctx, &mut a, "run");
        }
    }
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let next = target(&ctx, &mut a, "next", false);
    let before = serde_json::to_value(&a.progress).unwrap();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(next.center())],
    );
    let mut events = vec![egui::Event::Paste("x".repeat(513))];
    for pressed in [true, false] {
        events.push(egui::Event::PointerButton {
            pos: next.center(),
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
    let old = &a.progress.attempts[&a.scenarios[i].id];
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        before["attempts"][&a.scenarios[i].id]
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i + 1].id],
        &a.scenarios[i + 1]
    ));
    assert_eq!(a.progress.clock, before["clock"].as_u64().unwrap() + 1);
}

fn ntp_report_paints_inside(
    shape: &egui::Shape,
    clip: egui::Rect,
    viewport: egui::Rect,
    expected: &str,
    tolerance: f32,
) -> bool {
    match shape {
        egui::Shape::Vec(items) => items
            .iter()
            .any(|s| ntp_report_paints_inside(s, clip, viewport, expected, tolerance)),
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

fn reveal_ntp_output_frame(ctx: &egui::Context, a: &mut LabApp, width: f32) -> egui::FullOutput {
    for _ in 0..20 {
        let output = sized_compact_frame(ctx, a, width, 929., vec![]);
        let target = a.metrics.output.unwrap();
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(target, root, ctx.pixels_per_point()) {
            return output;
        }
        sized_compact_frame(
            ctx,
            a,
            width,
            929.,
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
            sized_compact_frame(ctx, a, width, 929., vec![]);
        }
    }
    panic!("Bounded actual outer wheels must reveal NTP output");
}
#[test]
fn fullwidth_six_row_transcript_paints_current_reports_and_browses_real_history_at_all_geometry() {
    for practice in [false, true] {
        let (ctx, mut a, _) = opened();
        tap(
            &ctx,
            &mut a,
            if practice { "example" } else { "assignment" },
        );
        for _ in 0..4 {
            type_command(&ctx, &mut a, "show running-config");
            tap(&ctx, &mut a, "run");
        }
        type_command(
            &ctx,
            &mut a,
            if practice {
                "ntp server 10.81.0.102"
            } else {
                "ntp server 10.71.0.101"
            },
        );
        tap(&ctx, &mut a, "run");
        type_command(&ctx, &mut a, "show running-config");
        tap(&ctx, &mut a, "run");
        let history = if practice {
            &session(&a).example_transcript
        } else {
            &session(&a).transcript
        };
        assert_eq!(history.len(), 6);
        let older = history.first().unwrap().output.clone();
        let latest = history.last().unwrap().output.clone();
        assert_ne!(older, latest);
        assert!(!a.metrics.completion);
        let before = serde_json::to_value(&a.progress).unwrap();
        let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for theme_id in ["dlms-dark", "dlms-light"] {
                for width in [760., 1600.] {
                    a.scale = scale;
                    a.theme_id = theme_id.into();
                    theme::apply(&ctx, theme_id);
                    for _ in 0..6 {
                        sized_compact_frame(&ctx, &mut a, width, 929., vec![]);
                    }
                    let output = reveal_ntp_output_frame(&ctx, &mut a, width);
                    let viewport = a.metrics.output.unwrap();
                    let tolerance = 1. / ctx.pixels_per_point();
                    let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                    let row = ctx.fonts_mut(|f| f.row_height(&font));
                    assert!((viewport.height() - row * 6.).abs() <= tolerance);
                    // ScrollArea's inner_rect excludes its reserved scrollbar;
                    // these six reports overflow the six-row content viewport.
                    let scrollbar = ctx.style().spacing.scroll.allocated_width();
                    assert!(scrollbar > 0.);
                    assert!(
                        (viewport.left() - a.metrics.ntp.fields[0].1.left()).abs() <= tolerance
                            && (viewport.right() + scrollbar - a.metrics.ntp.fields[0].1.right())
                                .abs()
                                <= tolerance,
                        "Content plus reserved scrollbar fills command-field width: {practice}/{scale}/{theme_id}/{width}, viewport={viewport:?}, scrollbar={scrollbar}, field={:?}, tolerance={tolerance}",
                        a.metrics.ntp.fields[0].1
                    );
                    assert!(
                        output.shapes.iter().any(|s| ntp_report_paints_inside(
                            &s.shape,
                            s.clip_rect,
                            viewport,
                            &latest,
                            tolerance
                        )),
                        "Actual newest report glyphs remain visible within clip {practice}/{scale}/{theme_id}/{width}"
                    );
                    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
                    assert_eq!(
                        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                        receipts
                    );
                }
            }
        }
        for (expected, direction) in [(&older, 1.), (&latest, -1.)] {
            let mut painted = false;
            for _ in 0..20 {
                reveal_ntp_output_frame(&ctx, &mut a, 1600.);
                let viewport = a.metrics.output.unwrap();
                sized_compact_frame(
                    &ctx,
                    &mut a,
                    1600.,
                    929.,
                    vec![egui::Event::PointerMoved(viewport.center())],
                );
                for _ in 0..3 {
                    sized_compact_frame(&ctx, &mut a, 1600., 929., vec![]);
                }
                sized_compact_frame(
                    &ctx,
                    &mut a,
                    1600.,
                    929.,
                    vec![egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., direction * viewport.height() / 3.),
                        modifiers: Default::default(),
                    }],
                );
                for _ in 0..10 {
                    sized_compact_frame(&ctx, &mut a, 1600., 929., vec![]);
                }
                let output = reveal_ntp_output_frame(&ctx, &mut a, 1600.);
                let viewport = a.metrics.output.unwrap();
                if output.shapes.iter().any(|s| {
                    ntp_report_paints_inside(
                        &s.shape,
                        s.clip_rect,
                        viewport,
                        expected,
                        1. / ctx.pixels_per_point(),
                    )
                }) {
                    painted = true;
                    break;
                }
            }
            assert!(
                painted,
                "Real inner wheels must browse older and newest NTP reports"
            );
            assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
            assert_eq!(
                serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                receipts
            );
        }
    }
}

#[test]
fn actual_unsupported_command_is_concise_beside_run_full_help_is_requested_and_retry_clears_it() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    assert_eq!(
        a.metrics.ntp.simulation_label.as_deref(),
        Some("Simulated NTP configuration · no real clock, daemon or packets.")
    );
    assert!(
        !a.metrics
            .ntp
            .simulation_label
            .as_ref()
            .unwrap()
            .contains("SIM select")
    );
    tap(&ctx, &mut a, "check");
    let flags = a.metrics.ntp.rows.iter().map(|r| r.0).collect::<Vec<_>>();
    type_command(&ctx, &mut a, "ntp peer 192.0.2.1");
    let before = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    tap(&ctx, &mut a, "run");
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    assert_eq!(a.notice.as_deref(), Some(dlms_learning_lab::ntp::HELP)); // original cause retained
    assert_eq!(
        a.metrics.ntp.notice_text.as_deref(),
        Some(
            "Needs attention · Unsupported command. Open Supported commands and limits for the available subset."
        )
    );
    assert!(a.metrics.ntp.command_help_text.is_none());
    assert_eq!(
        a.metrics.ntp.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        flags
    );
    let run = a
        .metrics
        .ntp
        .controls
        .iter()
        .find(|r| r.0 == "run")
        .unwrap()
        .1;
    let notice = a.metrics.ntp.notice.unwrap();
    assert!(notice.top() >= run.bottom());
    assert!(notice.bottom() <= a.metrics.ntp.rows[0].1.top());
    tap(&ctx, &mut a, "command-help");
    assert_eq!(
        a.metrics.ntp.command_help_text.as_deref(),
        Some(dlms_learning_lab::ntp::HELP)
    );
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    type_command(&ctx, &mut a, "show ntp status");
    let world = session(&a).current.clone();
    tap(&ctx, &mut a, "run");
    assert!(
        a.notice.is_none() && a.metrics.ntp.notice.is_none() && a.metrics.ntp.notice_text.is_none()
    );
    assert_eq!(session(&a).current, world);
    assert_eq!(
        session(&a).transcript.last().unwrap().input,
        "show ntp status"
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
}

#[test]
fn original_unicode_source_chains_remain_exact_while_native_checks_and_worked_text_use_ascii() {
    let (ctx, mut a, i) = opened_at(1);
    let engine::Goal::Ntp { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let original = definition.lesson.worked[0].explanation.clone();
    assert!(original.contains("11→12→11"));
    assert!(
        a.metrics
            .ntp
            .worked_explanations
            .iter()
            .any(|s| s.contains("11 -> 12 -> 11"))
    );
    assert!(
        !a.metrics
            .ntp
            .worked_explanations
            .iter()
            .any(|s| s.contains('→'))
    );
    assert_eq!(
        dlms_learning_lab::developer_digest::hash(include_bytes!(
            "../../../scenarios/ntp-client-server-v1.json"
        )),
        "f024f11706a53096f6922e147c9a50f3d8dfbab00c42cedf85c7cc341dea6206"
    );
    tap(&ctx, &mut a, "example");
    assert!(
        a.metrics
            .ntp
            .worked_explanations
            .iter()
            .any(|s| s.contains("11 -> 12 -> 11"))
    );
    tap(&ctx, &mut a, "assignment");
    let before = serde_json::to_value(&a.progress).unwrap();
    tap(&ctx, &mut a, "check");
    let engine::Goal::Ntp { definition } = &a.scenarios[i].goal else {
        panic!()
    };
    let raw = definition.checks(session(&a));
    assert!(raw.iter().any(|r| r.1.contains('→')));
    assert_eq!(
        a.metrics.ntp.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        raw.iter().map(|r| r.0).collect::<Vec<_>>()
    );
    assert_eq!(a.metrics.ntp.check_texts.len(), raw.len());
    for (rendered, (_, original)) in a.metrics.ntp.check_texts.iter().zip(&raw) {
        assert_eq!(rendered.replace(" -> ", " → "), *original);
    }
    assert!(
        !a.metrics
            .ntp
            .check_texts
            .iter()
            .any(|s| s.contains('→') || s.contains('□'))
    );
    assert!(
        a.metrics
            .ntp
            .evidence
            .iter()
            .any(|(s, _)| s.contains("10.72.0.1 -> 10.72.0.2 -> 10.72.0.1"))
    );
    // Checking changes only the actual checked presentation flag; it cannot edit facts.
    let mut expected = before;
    expected["attempts"][&a.scenarios[i].id]["state"]["simulation"]["checked"] =
        serde_json::json!(true);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), expected);
    assert_eq!(definition.lesson.worked[0].explanation, original);
}

#[test]
fn requested_hints_persist_one_at_a_time_without_worked_answer_and_are_isolated_by_scope() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    let initial = serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap();
    assert!(
        a.metrics.ntp.requested_hints.is_empty()
            && a.metrics.ntp.worked_commands.is_empty()
            && a.metrics.ntp.reference_urls.is_empty()
    );
    tap(&ctx, &mut a, "optional-help");
    assert!(a.metrics.ntp.requested_hints.is_empty() && a.metrics.ntp.worked_commands.is_empty());
    let before = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    tap(&ctx, &mut a, "show-hint");
    let mut expected = before.clone();
    expected["attempts"][&a.scenarios[i].id]["hints"] = serde_json::json!(1);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), expected);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    assert_eq!(a.metrics.ntp.requested_hints, a.scenarios[i].hints[..1]);
    assert!(a.metrics.ntp.worked_commands.is_empty() && a.metrics.ntp.reference_urls.is_empty());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        initial
    );
    a.save();
    let raw = std::fs::read(&a.path).unwrap();
    let saved = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(saved.attempts[&a.scenarios[i].id].hints, 1);
    assert!(saved.migration_receipts.is_some());
    assert_eq!(std::fs::read(&a.path).unwrap(), raw);
    let before = serde_json::to_value(&a.progress).unwrap();
    tap(&ctx, &mut a, "worked-approach");
    let expected_steps = a.scenarios[i]
        .solution
        .iter()
        .map(|command| {
            command.strip_prefix("SIM select ").map_or_else(
                || command.clone(),
                |ip| format!("Choose device {ip} in Selected device."),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(a.metrics.ntp.worked_commands, expected_steps);
    assert!(
        !a.metrics
            .ntp
            .worked_commands
            .iter()
            .any(|s| s.starts_with("SIM select "))
    );
    assert!(
        a.metrics
            .ntp
            .worked_commands
            .iter()
            .any(|s| s.starts_with("Choose device "))
    );
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    tap(&ctx, &mut a, "references");
    assert_eq!(a.metrics.ntp.reference_urls, a.scenarios[i].sources);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    accounts_ui_click(&ctx, &mut a, |m| m.mode_challenge);
    draw(&ctx, &mut a);
    assert!(a.progress.attempts[&a.scenarios[i].id].challenge);
    assert!(a.metrics.ntp.goal.is_some());
    assert!(a.metrics.ntp.worked_commands.is_empty() && a.metrics.ntp.reference_urls.is_empty());
    tap(&ctx, &mut a, "optional-help");
    assert_eq!(a.metrics.ntp.requested_hints, a.scenarios[i].hints[..1]);
    assert!(a.metrics.ntp.worked_commands.is_empty());
    for count in 2..=a.scenarios[i].hints.len() {
        tap(&ctx, &mut a, "show-hint");
        assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, count);
    }
    assert!(!a.metrics.ntp.controls.iter().any(|r| r.0 == "show-hint"));
    let old = serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id]).unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.back);
    draw(&ctx, &mut a);
    assert!(a.selected.is_none());
    // Select the real normal sibling card; no launch/state installation replaces
    // the actual native Start gesture used by opened_at and this source route.
    let peer = i + 1;
    let mut start = None;
    for _ in 0..20 {
        let pos = a
            .metrics
            .card_verification
            .iter()
            .position(|r| r.0 == a.scenarios[peer].id)
            .unwrap();
        let r = a.metrics.card_actions[pos].0;
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, viewport, ctx.pixels_per_point()) {
            start = Some(r);
            break;
        }
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(viewport.left() + 1., viewport.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (viewport.center().y - r.center().y)
                            .clamp(-viewport.height() * 0.75, viewport.height() * 0.75),
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
        start.expect("Real sibling Start must become fully visible"),
    );
    assert_eq!(a.selected, Some(peer));
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "optional-help");
    assert_eq!(a.progress.attempts[&a.scenarios[peer].id].hints, 0);
    assert!(a.metrics.ntp.requested_hints.is_empty() && a.metrics.ntp.worked_commands.is_empty());
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id]).unwrap(),
        old
    );
    let peer_before =
        serde_json::to_value(&a.progress.attempts[&a.scenarios[peer].id].state).unwrap();
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(a.metrics.ntp.requested_hints, a.scenarios[peer].hints[..1]);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[peer].id].state).unwrap(),
        peer_before
    );
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id]).unwrap(),
        old
    );
}

#[test]
fn every_guidance_control_vetoes_same_frame_enter_or_invalid_paste_and_held_release_before_dispatch()
 {
    for paste in [false, true] {
        let (ctx, mut a, i) = opened();
        tap(&ctx, &mut a, "assignment");
        type_command(&ctx, &mut a, "show ntp status");
        tap(&ctx, &mut a, "optional-help");
        for key in [
            "show-hint",
            "worked-approach",
            "references",
            "command-help",
            "optional-help",
        ] {
            let command = target(&ctx, &mut a, "command", true);
            click_real(&ctx, &mut a, command);
            let id = a
                .metrics
                .ntp
                .fields
                .iter()
                .find(|r| r.0 == "command")
                .unwrap()
                .2;
            assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            let r = target(&ctx, &mut a, key, false);
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            let before = serde_json::to_value(&a.progress).unwrap();
            let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
            let mut events = vec![egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }];
            if paste {
                events.push(egui::Event::Paste("x".repeat(513)));
            } else {
                events.push(egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
            frame(&ctx, &mut a, 1200., 850., events);
            assert_eq!(
                serde_json::to_value(&a.progress).unwrap(),
                before,
                "{key} press"
            );
            let mut released = vec![egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }];
            if !paste {
                released.push(egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers: Default::default(),
                });
            }
            frame(&ctx, &mut a, 1200., 850., released);
            draw(&ctx, &mut a);
            assert_eq!(
                serde_json::to_value(&a.progress).unwrap(),
                before,
                "{key} held release"
            );
            assert_eq!(
                serde_json::to_value(&a.progress.migration_receipts).unwrap(),
                receipts
            );
            assert_eq!(a.selected, Some(i));
            assert!(session(&a).transcript.is_empty() && !session(&a).checked);
            assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, 0);
        }
        tap(&ctx, &mut a, "show-hint");
        assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, 1);
        type_command(&ctx, &mut a, "show ntp status");
        tap(&ctx, &mut a, "run");
        assert_eq!(session(&a).transcript.len(), 1);
    }
}

// One platform input event frame and one requested paint, not draw()/smoke/update
// polling. FullOutput and egui's source-specific repaint cause prove scheduling.
fn ntp_single_frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200. / a.scale, 850. / a.scale),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    )
}
#[test]
fn native_idle_run_check_and_hint_each_request_their_own_followup_feedback_frame() {
    for key in ["run", "check", "show-hint"] {
        let (ctx, mut a, i) = opened();
        assert!(!a.smoke && a.reopen.is_none());
        tap(&ctx, &mut a, "assignment");
        if key == "run" {
            type_command(&ctx, &mut a, "show ntp status");
        }
        if key == "show-hint" {
            tap(&ctx, &mut a, "optional-help");
        }
        let r = target(&ctx, &mut a, key, false);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        let output = ntp_single_frame(
            &ctx,
            &mut a,
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
        assert_eq!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            std::time::Duration::ZERO
        );
        // Honor precisely the requested next frame with no more input/timer.
        let _ = ntp_single_frame(&ctx, &mut a, vec![]);
        assert!(
            ctx.repaint_causes()
                .iter()
                .any(|c| c.file.ends_with("src/ui/ntp_workspace.rs")),
            "{key} must explicitly schedule its own feedback paint"
        );
        match key {
            "run" => assert_eq!(
                session(&a).transcript.last().unwrap().input,
                "show ntp status"
            ),
            "check" => {
                assert!(session(&a).checked);
                assert_eq!(
                    a.metrics.ntp.rows.iter().map(|r| r.0).collect::<Vec<_>>(),
                    vec![false, false, false, true]
                );
            }
            _ => {
                assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, 1);
                assert_eq!(a.metrics.ntp.requested_hints, a.scenarios[i].hints[..1]);
            }
        }
    }
}
