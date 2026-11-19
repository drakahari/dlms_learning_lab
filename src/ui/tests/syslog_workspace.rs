//! Real egui private-fixture controls. Not Fedora, screen-reader or novice acceptance.
use super::*;
use dlms_learning_lab::syslog_workspace as work;
fn session(a: &LabApp) -> &work::Session {
    let State::Syslog { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn profile(a: &LabApp) -> serde_json::Value {
    serde_json::json!({"portable":a.progress,"receipts":a.progress.migration_receipts})
}
#[derive(Clone, Copy, Debug)]
enum Target<'a> {
    Control(&'a str),
    Field(&'a str),
    Choice(&'a str),
    Body(&'a str),
}
fn rect(a: &LabApp, t: Target<'_>) -> (egui::Rect, egui::Rect) {
    match t {
        Target::Control(k) => {
            let r = a
                .metrics
                .syslog
                .controls
                .iter()
                .find(|r| r.0 == k)
                .unwrap_or_else(|| panic!("Actual Syslog control {k}"));
            (r.1, r.2)
        }
        Target::Field(k) => {
            let r = a.metrics.syslog.fields.iter().find(|r| r.0 == k).unwrap();
            (r.1, r.3)
        }
        Target::Body(k) => {
            let r = a
                .metrics
                .syslog
                .help_bodies
                .iter()
                .find(|r| r.0 == k)
                .expect("Requested body response exists independently of paintculling");
            (r.1, r.2)
        }
        Target::Choice(k) => {
            let r = a
                .metrics
                .syslog
                .choices
                .iter()
                .find(|r| r.0 == k)
                .unwrap_or_else(|| panic!("Actual Syslog choice {k}"));
            (r.1, r.2)
        }
    }
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, t: Target<'_>, w: f32, h: f32) -> egui::Rect {
    for _ in 0..20 {
        let (r, clip) = rect(a, t);
        let clip = if matches!(t, Target::Choice(_)) {
            clip.intersect(ctx.content_rect())
        } else {
            clip.intersect(ctx.content_rect())
                .intersect(a.metrics.main_viewport.unwrap())
        };
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        let point = egui::pos2(
            if matches!(t, Target::Choice(_)) {
                r.center().x
            } else {
                clip.left() + 2.
            },
            clip.center().y,
        );
        frame(ctx, a, w, h, vec![egui::Event::PointerMoved(point)]);
        frame(
            ctx,
            a,
            w,
            h,
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(
                    0.,
                    (clip.center().y - r.center().y)
                        .clamp(-clip.height() * 0.75, clip.height() * 0.75),
                ),
                modifiers: Default::default(),
            }],
        );
        for _ in 0..10 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!(
        "Bounded real Syslog reveal must reach {t:?}: rect/clip={:?}; main={:?}",
        rect(a, t),
        a.metrics.main_viewport
    );
}
fn click_real(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect, w: f32, h: f32) {
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
    let r = reveal(ctx, a, Target::Control(key), 1200., 850.);
    click_real(ctx, a, r, 1200., 850.);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, choice: &str) {
    tap(ctx, a, key);
    let r = reveal(ctx, a, Target::Choice(choice), 1200., 850.);
    click_real(ctx, a, r, 1200., 850.);
}
fn type_field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = reveal(ctx, a, Target::Field(key), 1200., 850.);
    click_real(ctx, a, r, 1200., 850.);
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
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let source = pack::load(include_str!(
        "../../../tests/fixtures/syslog-workspace-preview.json"
    ))
    .unwrap()
    .remove(0);
    let title = source.title.clone();
    let i = a.scenarios.len();
    a.scenarios.push(source);
    a.filter.track = "CCNA".into();
    a.filter.search = title;
    draw(&ctx, &mut a);
    let at = a
        .metrics
        .card_verification
        .iter()
        .position(|r| r.0 == "dev-syslog-policy-workspace")
        .unwrap();
    let start = a.metrics.card_actions[at].0;
    assert!(smoke::smoke_control_visible(
        start,
        a.metrics.main_viewport.unwrap(),
        ctx.pixels_per_point()
    ));
    assert!(a.progress.attempts.is_empty());
    click_real(&ctx, &mut a, start, 1200., 850.);
    assert_eq!(a.selected, Some(i));
    assert_eq!(a.progress.attempts.len(), 1);
    (ctx, a, i)
}
fn solved(a: &LabApp) -> bool {
    progress::current_solved(
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id],
        &a.scenarios[a.selected.unwrap()],
    )
}
fn checked_rows(a: &LabApp, pass: bool) {
    let source = &a.scenarios[a.selected.unwrap()];
    let practice =
        !a.progress.attempts[&source.id].challenge && session(a).view == teaching::View::Example;
    assert!(if practice {
        session(a).example_checked
    } else {
        session(a).checked
    });
    let engine::Goal::Syslog { definition } = &source.goal else {
        panic!()
    };
    let expected = if practice {
        definition.practice_checks(session(a))
    } else {
        definition.checks(session(a))
    };
    assert!(!expected.is_empty());
    let passed = expected.iter().filter(|r| r.0).count();
    let summary = format!(
        "{} · {} of {} checks pass",
        if pass {
            "Configuration passes"
        } else {
            "Configuration needs attention"
        },
        passed,
        expected.len()
    );
    assert!(
        a.metrics.syslog.texts.contains(&summary),
        "Actual Check must paint the complete count/status summary"
    );
    if pass {
        assert!(expected.iter().all(|r| r.0));
    } else {
        assert!(expected.iter().any(|r| !r.0));
        if !a.metrics.syslog.disclosures.contains(&"check-details") {
            assert!(
                a.metrics
                    .syslog
                    .texts
                    .contains(&expected.iter().find(|r| !r.0).unwrap().1)
            );
        }
    }
    for (index, ok, _, _) in &a.metrics.syslog.painted_rows {
        assert_eq!(*ok, expected[*index].0);
    }
}
fn enter(pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    }
}

#[test]
fn syslog_native_intro_practice_fill_apply_and_assignment_are_separate() {
    let (ctx, mut a, _) = opened();
    assert_eq!(session(&a).view, teaching::View::Introduction);
    let assignment = session(&a).current.clone();
    let draft = session(&a).editor.clone();
    tap(&ctx, &mut a, "example");
    let old = session(&a).example.clone();
    tap(&ctx, &mut a, "fill-example");
    assert_eq!(session(&a).example, old);
    assert_eq!(session(&a).current, assignment);
    assert_eq!(session(&a).editor, draft);
    assert!(!solved(&a));
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    checked_rows(&a, true);
    assert!(!solved(&a));
    tap(&ctx, &mut a, "generate");
    assert_eq!(
        session(&a)
            .example
            .local_history("10.93.0.20".parse().unwrap())
            .unwrap()
            .len(),
        1
    );
    tap(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).current, assignment);
    assert_eq!(session(&a).editor, draft);
    tap(&ctx, &mut a, "check");
    checked_rows(&a, false);
}
#[test]
fn syslog_native_all_facilities_are_real_choices_and_actual_alternate_passes() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    let original = session(&a).current.clone();
    for n in 0..24 {
        choose(&ctx, &mut a, "facility", &format!("facility-{n}"));
        assert_eq!(
            session(&a)
                .editor
                .sources
                .values()
                .next()
                .unwrap()
                .facility
                .number(),
            n
        );
        assert_eq!(session(&a).current, original);
        assert!(!solved(&a));
    }
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(solved(&a));
    checked_rows(&a, true);
    choose(&ctx, &mut a, "buffer", "buffer-5");
    assert!(!solved(&a));
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(solved(&a));
    choose(&ctx, &mut a, "remote", "remote-off");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(!solved(&a));
    assert!(
        a.metrics
            .syslog
            .texts
            .iter()
            .any(|t| t.contains("disabled"))
    );
    choose(&ctx, &mut a, "remote", "remote-7");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(
        solved(&a),
        "Collector rejects debug; broad remote threshold remains a legitimate alternative"
    );
}
#[test]
fn syslog_native_specific_bad_address_retry_and_unapplied_feedback_preserve_policy() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    let applied = session(&a).current.clone();
    type_field(&ctx, &mut a, "host-0", "10.92.");
    assert_eq!(session(&a).current, applied);
    tap(&ctx, &mut a, "check");
    assert!(
        a.metrics
            .syslog
            .texts
            .iter()
            .any(|t| t.contains("Unapplied changes"))
    );
    tap(&ctx, &mut a, "apply");
    assert!(
        a.metrics
            .syslog
            .texts
            .iter()
            .any(|t| t.contains("complete IPv4"))
    );
    assert_eq!(session(&a).current, applied);
    type_field(&ctx, &mut a, "host-0", "10.92.0.100");
    choose(&ctx, &mut a, "facility", "facility-23");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(solved(&a));
}
#[test]
fn syslog_native_bad_aggregate_ignored_and_held_packets_never_apply_old_draft() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    let r = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
    click_real(&ctx, &mut a, r, 1200., 850.);
    let id = a.metrics.syslog.fields[0].2;
    let before = profile(&a);
    let cursor = egui::text_edit::TextEditState::load(&ctx, id)
        .unwrap()
        .cursor
        .char_range();
    let ctrl = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let mut aggregate = [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: ctrl,
        })
        .collect::<Vec<_>>();
    aggregate.extend([
        egui::Event::Text("1".repeat(8)),
        egui::Event::Text("2".repeat(8)),
        enter(true),
        enter(false),
    ]);
    for events in [
        vec![
            egui::Event::Paste("x".repeat(16)),
            enter(true),
            enter(false),
        ],
        vec![
            enter(true),
            enter(false),
            egui::Event::Paste("192.0.2.9".into()),
        ],
        vec![egui::Event::Paste("é".repeat(8))],
        aggregate,
    ] {
        frame(&ctx, &mut a, 1200., 850., events);
        draw(&ctx, &mut a);
        assert_eq!(profile(&a), before);
        assert_eq!(
            egui::text_edit::TextEditState::load(&ctx, id)
                .unwrap()
                .cursor
                .char_range(),
            cursor
        );
    }
    let apply = reveal(&ctx, &mut a, Target::Control("apply"), 1200., 850.);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(apply.center())],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerButton {
            pos: apply.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Paste("x".repeat(16))],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerButton {
            pos: apply.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    );
    draw(&ctx, &mut a);
    assert_eq!(profile(&a), before);
    tap(&ctx, &mut a, "check");
    assert!(session(&a).checked);
    assert!(!solved(&a));
}
#[test]
fn syslog_native_back_continue_and_selected_reset_keep_other_work_and_clear_old_undo() {
    let (ctx, mut a, i) = opened();
    let old = a.scenarios[0].clone();
    a.progress
        .attempts
        .insert(old.id.clone(), Attempt::new(&old, 0));
    tap(&ctx, &mut a, "assignment");
    type_field(&ctx, &mut a, "host-0", "192.0.");
    let state = serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap();
    let others = a
        .progress
        .attempts
        .iter()
        .filter(|(id, _)| **id != a.scenarios[i].id)
        .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let clock = a.progress.clock;
    // Use real outer wheels to reveal the existing native Back control.
    for _ in 0..20 {
        let root = a.metrics.main_viewport.unwrap();
        let back = a.metrics.back.unwrap();
        if smoke::smoke_control_visible(back, root, ctx.pixels_per_point()) {
            click_real(&ctx, &mut a, back, 1200., 850.);
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
                    delta: egui::vec2(0., root.height() * 0.75),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            draw(&ctx, &mut a);
        }
    }
    assert!(a.selected.is_none());
    draw(&ctx, &mut a);
    let at = a
        .metrics
        .card_verification
        .iter()
        .position(|r| r.0 == "dev-syslog-policy-workspace")
        .unwrap();
    let start = a.metrics.card_actions[at].0;
    click_real(&ctx, &mut a, start, 1200., 850.);
    assert_eq!(a.progress.clock, clock + 1);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        state
    );
    let reset = a.metrics.reset.unwrap();
    click_real(&ctx, &mut a, reset, 1200., 850.);
    let confirm = a.metrics.confirm_reset.unwrap();
    click_real(&ctx, &mut a, confirm, 1200., 850.);
    tap(&ctx, &mut a, "assignment");
    assert_eq!(
        session(&a).editor.sources.values().next().unwrap().hosts,
        ["10.92.0.100"]
    );
    let field = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
    click_real(&ctx, &mut a, field, 1200., 850.);
    let before = profile(&a);
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
    draw(&ctx, &mut a);
    assert_eq!(profile(&a), before);
    assert_eq!(
        a.progress
            .attempts
            .iter()
            .filter(|(id, _)| **id != a.scenarios[i].id)
            .map(|(id, a)| (id.clone(), serde_json::to_value(a).unwrap()))
            .collect::<std::collections::BTreeMap<_, _>>(),
        others
    );
}
fn painted<'a>(
    shape: &'a egui::Shape,
    clip: egui::Rect,
    out: &mut Vec<(egui::Rect, &'a egui::epaint::TextShape)>,
) {
    match shape {
        egui::Shape::Text(t) => out.push((clip, t)),
        egui::Shape::Vec(v) => {
            for s in v {
                painted(s, clip, out)
            }
        }
        _ => (),
    }
}
#[test]
fn syslog_native_alignment_actual_glyphs_placeholder_focus_and_challenge_at_supported_sizes() {
    let (ctx, mut a, _) = opened();
    for view in ["assignment", "example"] {
        a.scale = 1.;
        a.theme_id = "dlms-dark".into();
        theme::apply(&ctx, "dlms-dark");
        draw(&ctx, &mut a);
        tap(&ctx, &mut a, view);
        tap(&ctx, &mut a, "host-add");
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for theme in ["dlms-dark", "dlms-light"] {
                for width in [760., 1400.] {
                    a.scale = scale;
                    a.theme_id = theme.into();
                    assert_eq!(theme::find(theme).id, theme);
                    theme::apply(&ctx, theme);
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 1000., vec![]);
                    }
                    let field = reveal(&ctx, &mut a, Target::Field("host-1"), width, 1000.);
                    click_real(&ctx, &mut a, field, width, 1000.);
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width / a.scale, 1000. / a.scale),
                            )),
                            ..Default::default()
                        },
                        |ctx| a.render(ctx),
                    );
                    let mut texts = Vec::new();
                    for s in &out.shapes {
                        painted(&s.shape, s.clip_rect, &mut texts);
                    }
                    let f = a
                        .metrics
                        .syslog
                        .fields
                        .iter()
                        .find(|f| f.0 == "host-1")
                        .unwrap();
                    assert_eq!(ctx.memory(|m| m.focused()), Some(f.2));
                    let (tclip, t) = texts
                        .iter()
                        .find(|(_, t)| {
                            t.galley.text() == "192.0.2.100"
                                && f.1.intersects(t.visual_bounding_rect())
                        })
                        .unwrap();
                    let bounds = t.visual_bounding_rect();
                    assert!(
                        tclip
                            .expand(1. / out.pixels_per_point)
                            .contains_rect(bounds)
                    );
                    let layout = t.galley.rect.translate(t.pos.to_vec2());
                    assert!(
                        (layout.center().y - f.1.center().y).abs() <= 1. / out.pixels_per_point,
                        "Native singleline text layout must center by font metrics, independently of glyph ascenders"
                    );
                    assert!(
                        t.opacity_factor > 0.
                            && !t.galley.elided
                            && t.galley.rows.iter().any(|r| !r.visuals.mesh.vertices
                                [r.visuals.glyph_vertex_range.clone()]
                            .is_empty())
                    );
                    for (_, r, _, clip) in &a.metrics.syslog.fields {
                        assert!(r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.);
                    }
                    assert!(!solved(&a));
                }
            }
        }
    }
    a.scale = 1.;
    a.theme_id = "dlms-dark".into();
    theme::apply(&ctx, "dlms-dark");
    draw(&ctx, &mut a);
    let challenge = a.metrics.mode_challenge.unwrap();
    click_real(&ctx, &mut a, challenge, 1200., 850.);
    assert!(a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].challenge);
    assert!(
        !a.metrics
            .syslog
            .controls
            .iter()
            .any(|r| r.0 == "fill-example")
    );
    assert!(!a.metrics.syslog.controls.iter().any(|r| r.0 == "show-hint"));
}
#[test]
fn syslog_native_history_is_six_rows_and_real_right_edge_wheels_reach_old_and_latest() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    choose(&ctx, &mut a, "facility", "facility-23");
    tap(&ctx, &mut a, "apply");
    for _ in 0..10 {
        tap(&ctx, &mut a, "generate");
    }
    assert!(solved(&a));
    let history = session(&a)
        .current
        .local_history("10.92.0.10".parse().unwrap())
        .unwrap();
    assert_eq!(history.first().unwrap().sequence, 3);
    assert_eq!(history.last().unwrap().sequence, 10);
    let before = profile(&a);
    for direction in [1., -1.] {
        let mut saw = false;
        for _ in 0..20 {
            let (viewport, height) = a.metrics.syslog.history.unwrap();
            let root = a.metrics.main_viewport.unwrap();
            if !root.contains_rect(viewport) {
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
                                (root.center().y - viewport.center().y)
                                    .clamp(-root.height() * 0.75, root.height() * 0.75),
                            ),
                            modifiers: Default::default(),
                        },
                    ],
                );
                for _ in 0..10 {
                    draw(&ctx, &mut a);
                }
                continue;
            }
            assert!((viewport.height() - height).abs() < 1.);
            let point = egui::pos2(viewport.right() - 2., viewport.center().y);
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(point)],
            );
            for _ in 0..3 {
                draw(&ctx, &mut a);
            }
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., direction * height * 0.4),
                    modifiers: Default::default(),
                }],
            );
            for _ in 0..10 {
                draw(&ctx, &mut a);
            }
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1200., 850.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            let mut texts = Vec::new();
            for s in &out.shapes {
                painted(&s.shape, s.clip_rect, &mut texts);
            }
            let wanted = if direction > 0. { "#3 " } else { "#10 " };
            saw = texts.iter().any(|(clip, t)| {
                t.opacity_factor > 0.
                    && t.galley.rows.iter().any(|row| {
                        let chars = row.glyphs.iter().map(|g| g.chr).collect::<String>();
                        let vertices =
                            &row.visuals.mesh.vertices[row.visuals.glyph_vertex_range.clone()];
                        chars.starts_with(wanted)
                            && !vertices.is_empty()
                            && vertices.iter().all(|v| {
                                clip.intersect(viewport)
                                    .expand(1. / out.pixels_per_point)
                                    .contains(v.pos + row.pos.to_vec2() + t.pos.to_vec2())
                            })
                    })
            });
            if saw {
                break;
            }
        }
        assert!(
            saw,
            "Actual retained history must paint the requested older/latest row"
        );
        assert_eq!(profile(&a), before);
    }
}

#[test]
fn syslog_native_keyboard_and_requested_help_retain_meaningful_work_and_profile_law() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    let field = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
    click_real(&ctx, &mut a, field, 1200., 850.);
    let id = a.metrics.syslog.fields[0].2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    let tab = |shift| {
        [true, false]
            .into_iter()
            .map(move |pressed| egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers {
                    shift,
                    ..Default::default()
                },
            })
            .collect::<Vec<_>>()
    };
    frame(&ctx, &mut a, 1200., 850., tab(false));
    let focus = ctx.memory(|m| m.focused()).unwrap();
    let response = ctx.read_response(focus).unwrap();
    let remove = a
        .metrics
        .syslog
        .controls
        .iter()
        .find(|r| r.0 == "host-remove-0")
        .unwrap()
        .1;
    assert_eq!(response.rect, remove);
    frame(&ctx, &mut a, 1200., 850., tab(true));
    // egui queues Previous focus for the next pass (memory::Focus::interested_in_focus).
    // Process that native pass before asserting the unchanged reverse-Tab target.
    frame(&ctx, &mut a, 1200., 850., vec![]);
    assert_eq!(
        ctx.memory(|m| m.focused()),
        Some(id),
        "actual={:?}; field={:?}",
        ctx.memory(|m| m.focused())
            .and_then(|id| ctx.read_response(id))
            .map(|r| r.rect),
        a.metrics.syslog.fields
    );
    let mut expected = profile(&a);
    tap(&ctx, &mut a, "optional-help");
    tap(&ctx, &mut a, "show-hint");
    expected["portable"]["attempts"][&a.scenarios[i].id]["hints"] = 1.into();
    assert_eq!(profile(&a), expected);
    tap(&ctx, &mut a, "worked-approach");
    let text = a.scenarios[i].explanation.clone();
    let body = reveal(&ctx, &mut a, Target::Body(&text), 1200., 850.);
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
    let mut texts = Vec::new();
    for shape in &output.shapes {
        painted(&shape.shape, shape.clip_rect, &mut texts);
    }
    assert!(texts.iter().any(|(clip, t)| {
        t.galley.text() == text
            && t.opacity_factor > 0.
            && !t.galley.elided
            && clip
                .intersect(a.metrics.main_viewport.unwrap())
                .expand(1. / output.pixels_per_point)
                .contains_rect(t.visual_bounding_rect())
            && body.intersects(t.visual_bounding_rect())
    }));
    assert_eq!(profile(&a), expected);
    assert!(!solved(&a));
}
#[test]
fn syslog_native_idle_apply_generate_check_and_hint_request_their_own_repaint() {
    for key in ["apply", "generate", "check", "show-hint"] {
        let (ctx, mut a, i) = opened();
        assert!(!a.smoke && a.reopen.is_none());
        tap(&ctx, &mut a, "assignment");
        if key == "show-hint" {
            tap(&ctx, &mut a, "optional-help");
        }
        let r = reveal(&ctx, &mut a, Target::Control(key), 1200., 850.);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        a.dirty = false;
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
            |ctx| a.render(ctx),
        );
        assert_eq!(
            output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            std::time::Duration::ZERO
        );
        frame(&ctx, &mut a, 1200., 850., vec![]);
        assert!(
            ctx.repaint_causes()
                .iter()
                .any(|c| c.file.ends_with("src/ui/syslog_workspace.rs")),
            "{key} must schedule its own feedback frame without a smoke/timer tick"
        );
        match key {
            "generate" => assert_eq!(
                session(&a)
                    .current
                    .local_history("10.92.0.10".parse().unwrap())
                    .unwrap()
                    .len(),
                1
            ),
            "check" => assert!(session(&a).checked),
            "show-hint" => assert_eq!(a.progress.attempts[&a.scenarios[i].id].hints, 1),
            _ => assert!(a.metrics.syslog.texts.iter().any(|t| t.contains("applied"))),
        }
    }
}

#[path = "../../../tests/support/syslog_capacity.rs"]
mod capacity;
#[test]
fn syslog_native_whole_attempt_overflow_retains_success_draft_cursor_undo_and_hint_digit() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    choose(&ctx, &mut a, "facility", "facility-23");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(solved(&a));
    let source = capacity::padded_source(a.scenarios[i].clone());
    let id = source.id.clone();
    a.scenarios[i] = source.clone();
    let attempt = a.progress.attempts.get_mut(&id).unwrap();
    attempt.hints = 9;
    capacity::fill_to(&source, attempt, work::MAX_CONTEXT);
    a.dirty = false;
    draw(&ctx, &mut a);
    let r = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
    click_real(&ctx, &mut a, r, 1200., 850.);
    let field = a
        .metrics
        .syslog
        .fields
        .iter()
        .find(|r| r.0 == "host-0")
        .unwrap()
        .2;
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
                key: egui::Key::A,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers,
            })
            .collect(),
    );
    let native = egui::text_edit::TextEditState::load(&ctx, field).unwrap();
    let cursor = native.cursor.char_range();
    type UndoTrace = (
        Vec<(egui::text::CCursorRange, String)>,
        Vec<(egui::text::CCursorRange, String)>,
        bool,
    );
    fn undo_trace(state: &egui::text_edit::TextEditState, text: &str) -> UndoTrace {
        let current = (state.cursor.char_range().unwrap(), text.to_owned());
        let mut undos = state.undoer();
        let mut now = current.clone();
        let mut past = vec![];
        while let Some(v) = undos.undo(&now).cloned() {
            now = v.clone();
            past.push(v);
            assert!(past.len() <= 100);
        }
        let mut redos = state.undoer();
        let mut now = current;
        let mut future = vec![];
        while let Some(v) = redos.redo(&now).cloned() {
            now = v.clone();
            future.push(v);
            assert!(future.len() <= 100);
        }
        (past, future, state.undoer().is_in_flux())
    }
    let undo = undo_trace(&native, "10.92.0.100");
    let before = profile(&a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Paste("10.92.0.100x".into())],
    );
    draw(&ctx, &mut a);
    assert_eq!(profile(&a), before);
    assert!(solved(&a));
    assert!(session(&a).checked);
    assert!(!a.dirty);
    let native = egui::text_edit::TextEditState::load(&ctx, field).unwrap();
    assert_eq!(native.cursor.char_range(), cursor);
    assert_eq!(undo_trace(&native, "10.92.0.100"), undo);
    assert!(
        a.syslog_presentation
            .notice
            .as_ref()
            .unwrap()
            .1
            .contains("size limit")
    );
    tap(&ctx, &mut a, "optional-help");
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(profile(&a), before);
    assert_eq!(a.progress.attempts[&id].hints, 9);
    // The exact same real edit succeeds once legitimate saved room is available.
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .transcript
        .remove(0);
    type_field(&ctx, &mut a, "host-0", "10.92.0.100x");
    assert_eq!(
        session(&a).editor.sources[&"10.92.0.10".parse::<std::net::Ipv4Addr>().unwrap()].hosts[0],
        "10.92.0.100x"
    );
    assert!(!solved(&a));
    assert!(!session(&a).checked);
    assert!(a.dirty);
    tap(&ctx, &mut a, "show-hint");
    assert_eq!(a.progress.attempts[&id].hints, 10);
}
#[test]
fn syslog_native_unicode_line_separators_are_whole_packet_refusals() {
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    let r = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
    click_real(&ctx, &mut a, r, 1200., 850.);
    let before = profile(&a);
    for text in ["1\u{2028}2", "1\u{2029}2"] {
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Paste(text.into())],
        );
        draw(&ctx, &mut a);
        assert_eq!(profile(&a), before);
    }
}

#[test]
fn syslog_native_refused_mode_change_preserves_prior_dirty_and_complete_progress() {
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
                click_real(ctx, a, r, 1200., 850.);
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
        panic!("Actual mode header must be visible within20 real outer wheels");
    }

    for prior_dirty in [false, true] {
        let (ctx, mut a, i) = opened();
        tap(&ctx, &mut a, "assignment");
        choose(&ctx, &mut a, "facility", "facility-23");
        tap(&ctx, &mut a, "apply");
        tap(&ctx, &mut a, "check");
        mode(&ctx, &mut a, true);
        assert!(a.progress.attempts[&a.scenarios[i].id].challenge);
        assert!(solved(&a));
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
        let before = profile(&a);
        // Actual header click; mode must not be installed before admission.
        mode(&ctx, &mut a, false);
        assert_eq!(profile(&a), before);
        assert_eq!(a.dirty, prior_dirty);
        assert!(a.progress.attempts[&id].challenge);
        assert!(session(&a).checked && solved(&a));
        assert!(a.notice.as_ref().unwrap().contains("mode change"));
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .transcript
            .remove(0);
        let mut expected = profile(&a);
        expected["portable"]["attempts"][&id]["challenge"] = false.into();
        mode(&ctx, &mut a, false);
        assert_eq!(profile(&a), expected);
        assert!(!a.progress.attempts[&id].challenge);
        assert!(a.dirty && solved(&a));
    }
}

#[test]
fn syslog_native_all_disclosures_veto_real_enter_actions_and_held_release_without_edit_packets() {
    fn tab_events() -> Vec<egui::Event> {
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            })
            .collect()
    }
    fn focus(ctx: &egui::Context, a: &mut LabApp, key: &str) {
        for _ in 0..96 {
            let target = if key == "next" {
                a.metrics.next_unit.unwrap()
            } else {
                rect(a, Target::Control(key)).0
            };
            if ctx
                .memory(|m| m.focused())
                .and_then(|id| ctx.read_response(id))
                .is_some_and(|r| r.rect == target)
            {
                return;
            }
            frame(ctx, a, 1200., 850., tab_events());
            draw(ctx, a);
        }
        panic!("Actual Tab navigation must focus {key} within96 controls");
    }
    for header in ["optional-help", "worked-approach", "references"] {
        for action in ["enter-field", "generate", "check", "next"] {
            for held in [false, true] {
                let (ctx, mut a, i) = opened();
                tap(&ctx, &mut a, "assignment");
                choose(&ctx, &mut a, "facility", "facility-23");
                if action == "next" {
                    tap(&ctx, &mut a, "apply");
                    let id = a.scenarios[i].id.clone();
                    let unit = "dev-syslog-guidance-unit";
                    a.scenarios[i].unit = unit.into();
                    let mut sibling = a.scenarios[i].clone();
                    sibling.id = "dev-syslog-guidance-sibling".into();
                    let sibling_id = sibling.id.clone();
                    a.scenarios.push(sibling);
                    a.units.push(course::Unit {
                        id: unit.into(),
                        track: "CCNA".into(),
                        title: "Private guidance route".into(),
                        summary: "Private source-only regression".into(),
                        prerequisites: vec![],
                        labs: vec![id, sibling_id],
                    });
                    draw(&ctx, &mut a);
                    assert!(solved(&a));
                    assert!(a.metrics.next_unit.is_some());
                }
                if header != "optional-help" {
                    tap(&ctx, &mut a, "optional-help");
                }
                if action == "enter-field" {
                    let r = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
                    click_real(&ctx, &mut a, r, 1200., 850.);
                    let id = a
                        .metrics
                        .syslog
                        .fields
                        .iter()
                        .find(|r| r.0 == "host-0")
                        .unwrap()
                        .2;
                    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
                } else {
                    focus(&ctx, &mut a, action);
                }
                let target = reveal(&ctx, &mut a, Target::Control(header), 1200., 850.);
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    850.,
                    vec![egui::Event::PointerMoved(target.center())],
                );
                let before = profile(&a);
                let enter = |pressed| egui::Event::Key {
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
                let mut events = vec![enter(true), pointer(true)];
                if !held {
                    events.extend([pointer(false), enter(false)]);
                }
                assert!(
                    !events
                        .iter()
                        .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
                );
                frame(&ctx, &mut a, 1200., 850., events);
                assert!(
                    a.metrics.syslog.guidance,
                    "Actual header intent must be observed: {header}/{action}/{held}"
                );
                match action {
                    "enter-field" => assert!(
                        a.metrics.syslog.entered,
                        "A real focused-field Enter must reach the shared dispatch gate"
                    ),
                    "generate" => assert!(
                        a.metrics.syslog.intents.1,
                        "Actual Generate keyboard intent must reach the shared dispatch gate"
                    ),
                    "check" => assert!(
                        a.metrics.syslog.intents.2,
                        "Actual Check keyboard intent must reach the shared dispatch gate"
                    ),
                    _ => assert!(
                        a.metrics.next_unit.is_none(),
                        "Help must suppress the otherwise present real Next action"
                    ),
                }
                assert_eq!(
                    profile(&a),
                    before,
                    "Help cannot execute {action}: {header}/{held}"
                );
                assert_eq!(a.selected, Some(i));
                if held {
                    frame(
                        &ctx,
                        &mut a,
                        1200.,
                        850.,
                        vec![pointer(false), enter(false)],
                    );
                    assert_eq!(
                        profile(&a),
                        before,
                        "Held help release cannot execute old {action}"
                    );
                    assert_eq!(a.selected, Some(i));
                }
                draw(&ctx, &mut a);
                assert_eq!(profile(&a), before);
                // A later ordinary action still operates; the veto is not a lockout.
                if action == "enter-field" {
                    tap(&ctx, &mut a, "apply");
                    assert!(solved(&a));
                }
                if action == "generate" {
                    tap(&ctx, &mut a, "generate");
                    assert!(
                        !session(&a)
                            .current
                            .local_history("10.92.0.10".parse().unwrap())
                            .unwrap()
                            .is_empty()
                    );
                }
                if action == "check" {
                    tap(&ctx, &mut a, "check");
                    assert!(session(&a).checked);
                }
                if action == "next" {
                    assert!(a.metrics.next_unit.is_some());
                    assert_eq!(a.progress.attempts.len(), 1);
                }
            }
        }
    }
}

#[test]
fn syslog_ordinary_split_frame_hint_press_idle_release_awards_only_one_requested_hint() {
    let (ctx, mut a, i) = opened();
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "optional-help");
    let r = reveal(&ctx, &mut a, Target::Control("show-hint"), 1200., 850.);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    let before = profile(&a);
    let pointer = |pressed| egui::Event::PointerButton {
        pos: r.center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(&ctx, &mut a, 1200., 850., vec![pointer(true)]);
    assert_eq!(profile(&a), before);
    assert!(!a.syslog_presentation.held_veto);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![]);
        assert_eq!(profile(&a), before);
    }
    frame(&ctx, &mut a, 1200., 850., vec![pointer(false)]);
    draw(&ctx, &mut a);
    let mut expected = before;
    expected["portable"]["attempts"][&a.scenarios[i].id]["hints"] = 1.into();
    assert_eq!(profile(&a), expected);
    assert!(!solved(&a));
    assert!(!session(&a).checked);
}
#[test]
fn syslog_ordinary_disclosure_split_clicks_open_actual_bodies_without_mutating_work() {
    for key in ["optional-help", "worked-approach", "references"] {
        let (ctx, mut a, _) = opened();
        tap(&ctx, &mut a, "assignment");
        if key != "optional-help" {
            tap(&ctx, &mut a, "optional-help");
        }
        let r = reveal(&ctx, &mut a, Target::Control(key), 1200., 850.);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::PointerMoved(r.center())],
        );
        let before = profile(&a);
        let pointer = |pressed| egui::Event::PointerButton {
            pos: r.center(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(&ctx, &mut a, 1200., 850., vec![pointer(true)]);
        assert_eq!(profile(&a), before);
        assert!(!a.syslog_presentation.held_veto);
        for _ in 0..3 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
            assert_eq!(profile(&a), before);
        }
        frame(&ctx, &mut a, 1200., 850., vec![pointer(false)]);
        for _ in 0..3 {
            draw(&ctx, &mut a);
        }
        assert!(
            a.metrics.syslog.disclosures.contains(&key),
            "Actual {key} body must open after a normal release"
        );
        assert_eq!(profile(&a), before);
        assert!(!solved(&a));
    }
}

// Private authored source bootstrap only. No State/Outcome/progress is installed;
// the actual library Start creates the sole selected Attempt.
fn opened_harbor() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let extra: pack::Pack =
        serde_json::from_str(include_str!("../../../scenarios/syslog-policy-v1.json")).unwrap();
    assert_eq!(extra.scenarios.len(), 4);
    for s in &extra.scenarios {
        work::source(s).unwrap();
    }
    let id = "ccna-syslog-policy-handover";
    let source = extra.scenarios.iter().find(|s| s.id == id).unwrap();
    let title = source.title.clone();
    let ids = extra.scenarios.iter().map(|s| s.id.clone()).collect();
    let i = a.scenarios.len() + 3;
    a.scenarios.extend(extra.scenarios);
    a.units.push(course::Unit {
        id: "ccna-syslog-policy".into(),
        track: "CCNA".into(),
        title: "Private logging-policy review".into(),
        summary: "Unregistered authored source".into(),
        prerequisites: vec!["ipv4-foundations".into()],
        labs: ids,
    });
    a.filter.track = "CCNA".into();
    a.filter.search = title;
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
        a.metrics.main_viewport.unwrap(),
        ctx.pixels_per_point()
    ));
    assert!(a.progress.attempts.is_empty());
    click_real(&ctx, &mut a, start, 1200., 850.);
    assert_eq!(a.selected, Some(i));
    assert_eq!(a.progress.attempts.len(), 1);
    tap(&ctx, &mut a, "assignment");
    (ctx, a, i)
}
fn harbor_rows(a: &LabApp) -> Vec<(bool, String)> {
    let engine::Goal::Syslog { definition } = &a.scenarios[a.selected.unwrap()].goal else {
        panic!()
    };
    definition.checks(session(a))
}
fn solve_harbor_with_native_controls(ctx: &egui::Context, a: &mut LabApp) {
    choose(ctx, a, "component", "collector-10.217.0.110");
    tap(ctx, a, "selector-add");
    choose(ctx, a, "selector-facility-0", "selector-facility-0-22");
    choose(ctx, a, "selector-threshold-0", "selector-threshold-0-4");
    tap(ctx, a, "selector-add");
    choose(ctx, a, "selector-facility-1", "selector-facility-1-23");
    choose(ctx, a, "selector-threshold-1", "selector-threshold-1-3");
    tap(ctx, a, "apply");
    for ip in ["10.217.0.10", "10.217.0.20"] {
        choose(ctx, a, "component", &format!("source-{ip}"));
        type_field(ctx, a, "host-0", "10.217.0.110");
        tap(ctx, a, "apply");
    }
    assert!(solved(a));
}

fn assert_real_result_label(
    ctx: &egui::Context,
    a: &mut LabApp,
    width: f32,
    key: &str,
    label: &str,
) {
    reveal(ctx, a, Target::Control(key), width, 1000.);
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, 1000. / a.scale),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut texts = Vec::new();
    for shape in &out.shapes {
        painted(&shape.shape, shape.clip_rect, &mut texts);
    }
    assert!(
        texts.iter().any(|(clip, text)| text.galley.text() == label
            && text.opacity_factor > 0.
            && !text.galley.elided
            && text.galley.rows.iter().any(|r| !r.visuals.mesh.vertices
                [r.visuals.glyph_vertex_range.clone()]
            .is_empty())
            && clip
                .intersect(ctx.content_rect())
                .expand(1. / out.pixels_per_point)
                .contains_rect(text.visual_bounding_rect())),
        "Actual requested {key} must contain fully clipped nonempty glyphs at {width}/{}",
        a.scale
    );
}
#[test]
fn syslog_harbor52_default_summary_and_requested_full_rows_are_real_paint_at_all_supported_sizes() {
    let (ctx, mut a, _) = opened_harbor();
    tap(&ctx, &mut a, "check");
    let expected = harbor_rows(&a);
    assert_eq!(expected.len(), 52);
    assert!(expected.iter().any(|r| !r.0));
    let first = expected.iter().find(|r| !r.0).unwrap().1.clone();
    let before = profile(&a);
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
        for name in ["dlms-dark", "dlms-light"] {
            for width in [760., 1600.] {
                a.scale = scale;
                a.theme_id = name.into();
                theme::apply(&ctx, name);
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 1000., vec![]);
                }
                if a.metrics.syslog.disclosures.contains(&"check-details") {
                    let r = reveal(&ctx, &mut a, Target::Control("check-details"), width, 1000.);
                    click_real(&ctx, &mut a, r, width, 1000.);
                    for _ in 0..10 {
                        frame(&ctx, &mut a, width, 1000., vec![]);
                    }
                }
                assert!(!a.metrics.syslog.disclosures.contains(&"check-details"));
                assert!(a.metrics.syslog.rows.is_empty());
                assert!(a.metrics.syslog.painted_rows.is_empty());
                assert_eq!(
                    a.metrics
                        .syslog
                        .texts
                        .iter()
                        .filter(|t| **t == first)
                        .count(),
                    1
                );
                checked_rows(&a, false);
                let summary = format!(
                    "Configuration needs attention · {} of 52 checks pass",
                    expected.iter().filter(|r| r.0).count()
                );
                assert_real_result_label(&ctx, &mut a, width, "result-summary", &summary);
                assert_real_result_label(
                    &ctx,
                    &mut a,
                    width,
                    "first-issue",
                    &format!("Needs attention · {first}"),
                );
                let r = reveal(&ctx, &mut a, Target::Control("check-details"), width, 1000.);
                click_real(&ctx, &mut a, r, width, 1000.);
                let mut seen = std::collections::BTreeSet::new();
                for _ in 0..20 {
                    // Inspect original real egui text shapes. No resized image,
                    // forced offsets or model-generated result rectangles.
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width / a.scale, 1000. / a.scale),
                            )),
                            ..Default::default()
                        },
                        |ctx| a.render(ctx),
                    );
                    let mut texts = Vec::new();
                    for shape in &out.shapes {
                        painted(&shape.shape, shape.clip_rect, &mut texts);
                    }
                    assert!(a.metrics.syslog.disclosures.contains(&"check-details"));
                    assert!(
                        !a.metrics
                            .syslog
                            .controls
                            .iter()
                            .any(|r| r.0 == "first-issue")
                    );
                    for (n, ok, rect, clip) in &a.metrics.syslog.painted_rows {
                        assert_eq!(*ok, expected[*n].0);
                        assert!(rect.intersects(clip.intersect(ctx.content_rect())));
                        let label = format!(
                            "{} · {}",
                            if *ok { "Passed" } else { "Needs attention" },
                            expected[*n].1
                        );
                        if texts.iter().any(|(paint_clip, text)| {
                            text.galley.text() == label
                                && text.opacity_factor > 0.
                                && !text.galley.elided
                                && text.galley.rows.iter().any(|r| {
                                    !r.visuals.mesh.vertices[r.visuals.glyph_vertex_range.clone()]
                                        .is_empty()
                                })
                                && paint_clip
                                    .intersect(*clip)
                                    .intersect(ctx.content_rect())
                                    .expand(1. / out.pixels_per_point)
                                    .contains_rect(text.visual_bounding_rect())
                        }) {
                            seen.insert(*n);
                        }
                    }
                    assert_eq!(profile(&a), before);
                    if seen.len() == 52 {
                        break;
                    }
                    let clip = rect(&a, Target::Control("check-details"))
                        .1
                        .intersect(ctx.content_rect())
                        .intersect(a.metrics.main_viewport.unwrap());
                    let point = egui::pos2(clip.left() + 2., clip.center().y);
                    frame(
                        &ctx,
                        &mut a,
                        width,
                        1000.,
                        vec![egui::Event::PointerMoved(point)],
                    );
                    frame(
                        &ctx,
                        &mut a,
                        width,
                        1000.,
                        vec![egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0., -clip.height() * 0.65),
                            modifiers: Default::default(),
                        }],
                    );
                    for _ in 0..10 {
                        frame(&ctx, &mut a, width, 1000., vec![]);
                    }
                }
                assert_eq!(
                    seen,
                    (0..52).collect::<std::collections::BTreeSet<_>>(),
                    "Every original check must be reachable as actual full-clipped glyphs: {scale}/{name}/{width}"
                );
                assert_eq!(profile(&a), before);
            }
        }
    }
}

#[test]
fn syslog_harbor_native_completion_and_current_damage_keep_full_checks_and_compact_first_failure() {
    let (ctx, mut a, _) = opened_harbor();
    solve_harbor_with_native_controls(&ctx, &mut a);
    tap(&ctx, &mut a, "check");
    checked_rows(&a, true);
    assert_eq!(harbor_rows(&a).len(), 52);
    assert!(a.metrics.syslog.rows.is_empty());
    assert!(
        a.metrics
            .syslog
            .texts
            .contains(&"Configuration passes · 52 of 52 checks pass".into())
    );
    assert!(a.metrics.next_unit.is_some());
    let before = profile(&a);
    let r = reveal(&ctx, &mut a, Target::Control("check-details"), 1200., 850.);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(r.center())],
    );
    let pointer = |pressed| egui::Event::PointerButton {
        pos: r.center(),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    frame(&ctx, &mut a, 1200., 850., vec![pointer(true)]);
    assert_eq!(profile(&a), before);
    assert!(!a.syslog_presentation.held_veto);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![]);
    }
    frame(&ctx, &mut a, 1200., 850., vec![pointer(false)]);
    draw(&ctx, &mut a);
    assert!(a.metrics.syslog.disclosures.contains(&"check-details"));
    assert_eq!(profile(&a), before);
    tap(&ctx, &mut a, "check-details");
    choose(&ctx, &mut a, "component", "source-10.217.0.20");
    type_field(&ctx, &mut a, "host-0", "10.217.0.100");
    assert!(!solved(&a));
    assert!(!session(&a).checked);
    assert!(a.metrics.next_unit.is_none());
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    checked_rows(&a, false);
    let expected = harbor_rows(&a);
    assert_eq!(expected.len(), 52);
    let failure = &expected.iter().find(|r| !r.0).unwrap().1;
    assert_eq!(
        a.metrics
            .syslog
            .texts
            .iter()
            .filter(|t| *t == failure)
            .count(),
        1
    );
    assert!(a.metrics.syslog.rows.is_empty());
    assert!(a.metrics.next_unit.is_none());
    type_field(&ctx, &mut a, "host-0", "10.217.0.110");
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    checked_rows(&a, true);
    assert!(solved(&a));
}

#[test]
fn syslog_results_disclosure_vetoes_real_enter_actions_next_and_held_release_without_locking_clean_actions()
 {
    fn focus(ctx: &egui::Context, a: &mut LabApp, key: &str) {
        for _ in 0..96 {
            let target = if key == "next" {
                a.metrics.next_unit.unwrap()
            } else {
                rect(a, Target::Control(key)).0
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
        panic!("Real keyboard focus must reach {key}");
    }
    for action in ["enter-field", "generate", "check", "next"] {
        for held in [false, true] {
            let (ctx, mut a, i) = opened_harbor();
            if action == "next" {
                solve_harbor_with_native_controls(&ctx, &mut a);
            }
            if action == "enter-field" {
                type_field(&ctx, &mut a, "host-0", "10.217.0.110");
                assert!(
                    session(&a)
                        .has_unapplied(
                            match &a.scenarios[i].goal {
                                engine::Goal::Syslog { definition } => definition,
                                _ => panic!(),
                            },
                            false
                        )
                        .unwrap()
                );
            }
            tap(&ctx, &mut a, "check");
            if action == "enter-field" {
                let r = reveal(&ctx, &mut a, Target::Field("host-0"), 1200., 850.);
                click_real(&ctx, &mut a, r, 1200., 850.);
                let id = a
                    .metrics
                    .syslog
                    .fields
                    .iter()
                    .find(|r| r.0 == "host-0")
                    .unwrap()
                    .2;
                assert_eq!(ctx.memory(|m| m.focused()), Some(id));
            } else {
                focus(&ctx, &mut a, action);
            }
            let r = reveal(&ctx, &mut a, Target::Control("check-details"), 1200., 850.);
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            let before = profile(&a);
            let pointer = |pressed| egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            };
            let mut events = vec![enter(true), pointer(true)];
            if !held {
                events.extend([pointer(false), enter(false)]);
            }
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
            );
            frame(&ctx, &mut a, 1200., 850., events);
            assert!(a.metrics.syslog.guidance);
            match action {
                "enter-field" => assert!(a.metrics.syslog.entered),
                "generate" => assert!(a.metrics.syslog.intents.1),
                "check" => assert!(a.metrics.syslog.intents.2),
                _ => assert!(a.metrics.next_unit.is_none()),
            }
            assert_eq!(profile(&a), before);
            assert_eq!(a.selected, Some(i));
            if held {
                frame(
                    &ctx,
                    &mut a,
                    1200.,
                    850.,
                    vec![pointer(false), enter(false)],
                );
                assert_eq!(profile(&a), before);
            }
            draw(&ctx, &mut a);
            assert_eq!(profile(&a), before);
            match action {
                "enter-field" => {
                    tap(&ctx, &mut a, "apply");
                    assert_eq!(
                        session(&a)
                            .current
                            .device_policy("10.217.0.10".parse().unwrap())
                            .unwrap()
                            .hosts,
                        std::collections::BTreeSet::from(["10.217.0.110"
                            .parse::<std::net::Ipv4Addr>()
                            .unwrap()])
                    );
                    assert!(!session(&a).checked);
                }
                "generate" => {
                    tap(&ctx, &mut a, "generate");
                    assert!(
                        !session(&a)
                            .current
                            .local_history("10.217.0.10".parse().unwrap())
                            .unwrap()
                            .is_empty()
                    );
                }
                "check" => {
                    tap(&ctx, &mut a, "check");
                    assert!(session(&a).checked);
                }
                _ => assert!(a.metrics.next_unit.is_some()),
            }
        }
    }
}
