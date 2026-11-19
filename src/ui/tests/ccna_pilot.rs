use super::*;
pub(in crate::ui) fn catalog() -> (Vec<Scenario>, Vec<course::Unit>) {
    static C: std::sync::OnceLock<(Vec<Scenario>, Vec<course::Unit>)> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let s = pack::ccna_pilot().unwrap();
        let u = course::ccna_pilot(&s).unwrap();
        (s, u)
    })
    .clone()
}
fn app() -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    (ctx, a)
}

fn session(a: &mut LabApp, i: usize) -> &mut dlms_learning_lab::magic_subnet::Session {
    let engine::State::MagicSubnet { session } = &mut a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .state
    else {
        panic!("worksheet")
    };
    session
}
#[test]
fn ccna_pilot_views_reflow_and_keep_assignment_and_example_drafts_separate() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-magic-blocks")
        .unwrap();
    a.launch(i);
    for scale in [1., 1.5, 2.] {
        for width in [780., 1600.] {
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                session(&mut a, i).view = view;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let viewport = a.metrics.main_viewport.unwrap();
                for r in &a.metrics.inputs {
                    assert!(
                        r.left() >= viewport.left() - 2. && r.right() <= viewport.right() + 2.,
                        "field clipped {scale} {width}: {r:?} {viewport:?}"
                    );
                }
                assert_eq!(a.metrics.lesson_tabs.len(), 3);
            }
        }
    }
    a.scale = 1.;
    ctx.set_zoom_factor(1.);
    session(&mut a, i).view = teaching::View::Example;
    session(&mut a, i).example.answers[2] = "64".into();
    draw(&ctx, &mut a);
    assert!(
        a.metrics.check.unwrap().bottom() < 850.,
        "normal desktop practice keeps Check visible"
    );
    let draft = session(&mut a, i).example.clone();
    session(&mut a, i).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    assert!(session(&mut a, i).answers.iter().all(String::is_empty));
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    session(&mut a, i).view = teaching::View::Example;
    draw(&ctx, &mut a);
    assert_eq!(session(&mut a, i).example, draft);
}
#[test]
fn ccna_pilot_actual_check_button_keyboard_edit_and_stale_feedback() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-magic-blocks")
        .unwrap();
    a.launch(i);
    session(&mut a, i).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(session(&mut a, i).checked.is_some());
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    let fields = session(&mut a, i)
        .stage
        .fields(session(&mut a, i).problem)
        .unwrap();
    for (n, f) in fields.iter().enumerate() {
        session(&mut a, i).answers[n] = f.expected.clone();
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
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
                modifiers: egui::Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
            },
            egui::Event::Text("bad".into()),
        ],
    );
    assert_eq!(session(&mut a, i).answers[0], "bad");
    let current = session(&mut a, i).answers.clone();
    assert_ne!(session(&mut a, i).checked.as_ref(), Some(&current));
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
}
#[test]
fn ccna_pilot_opt_in_does_not_include_other_tracks_and_preview_phase_is_context_safe() {
    let (ctx, mut a) = app();
    assert_eq!(
        a.scenarios
            .iter()
            .filter(|s| ccna_presentation::enabled(s))
            .count(),
        74
    );
    assert!(
        a.scenarios
            .iter()
            .filter(|s| s.track != "CCNA")
            .all(|s| !ccna_presentation::enabled(s))
    );
    for _ in 0..15 {
        let _ = ctx.run(Default::default(), |ctx| a.ccna_preview_tick(ctx));
    }
    assert!(
        a.capture_pending,
        "exercise actual capture phase, not merely a case list"
    );
}
#[test]
fn ccna_pilot_native_edit_histories_reset_and_challenge_help_are_isolated() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-magic-blocks")
        .unwrap();
    a.launch(i);
    session(&mut a, i).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    let assignment_id = a.metrics.focus.unwrap();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("assignment-only-draft".into())],
    );
    assert_eq!(session(&mut a, i).answers[0], "assignment-only-draft");
    let tab = a.metrics.lesson_tabs[1];
    click(&ctx, &mut a, tab);
    let field = a.metrics.inputs[0];
    click(&ctx, &mut a, field);
    assert_ne!(ctx.memory(|m| m.focused()).unwrap(), assignment_id);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        }],
    );
    assert_ne!(
        session(&mut a, i).example.answers[0],
        "assignment-only-draft"
    );
    let tab = a.metrics.lesson_tabs[2];
    click(&ctx, &mut a, tab);
    assert_eq!(session(&mut a, i).answers[0], "assignment-only-draft");
    let help = a.metrics.help.unwrap();
    click(&ctx, &mut a, help);
    assert!(a.metrics.help_visible);
    a.progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .challenge = true;
    draw(&ctx, &mut a);
    assert!(
        !a.metrics.help_visible,
        "Challenge starts with worked bit help closed"
    );
    assert!(a.metrics.lesson_tabs.is_empty());
    a.reset_attempt(i);
    session(&mut a, i).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    assert_ne!(a.metrics.focus.unwrap(), assignment_id);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        }],
    );
    assert!(session(&mut a, i).answers.iter().all(String::is_empty));
}
#[test]
fn ccna_revision_library_starts_at_foundations_and_intro_preserves_saved_work() {
    let (ctx, mut a) = app();
    assert_eq!(a.filter.track, "CCNA");
    assert_eq!(a.filter.unit, "ccna-magic-number");
    assert_eq!(a.units[0].id, "ccna-magic-number");
    assert_eq!(a.units[1].id, "ipv4-foundations");
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics
            .card_verification
            .iter()
            .map(|x| x.0.as_str())
            .collect::<Vec<_>>(),
        [
            "ccna-magic-blocks",
            "ccna-magic-boundaries",
            "ccna-magic-cross-octet",
            "ccna-magic-octet-aligned"
        ]
    );
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-magic-blocks")
        .unwrap();
    a.launch(i);
    session(&mut a, i).answers[2] = "draft kept".into();
    session(&mut a, i).view = teaching::View::Assignment;
    a.progress.favorites.insert("ccna-magic-blocks".into());
    a.progress
        .attempts
        .get_mut("ccna-magic-blocks")
        .unwrap()
        .challenge = true;
    a.selected = None;
    draw(&ctx, &mut a);
    let start = a.metrics.course_start.unwrap();
    click(&ctx, &mut a, start);
    assert_eq!(a.selected, Some(i));
    assert_eq!(session(&mut a, i).view, teaching::View::Introduction);
    assert_eq!(session(&mut a, i).answers[2], "draft kept");
    assert!(a.progress.favorites.contains("ccna-magic-blocks"));
    assert!(!a.progress.attempts["ccna-magic-blocks"].challenge);
    a.selected = None;
    a.filter.unit.clear();
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_verification[0].0, "ccna-magic-blocks");
}
#[test]
fn ccna_revision_ipv4_uses_left_aligned_steps_table_and_specific_practice_actions() {
    fn text(shape: &egui::Shape, out: &mut Vec<(String, bool)>) {
        match shape {
            egui::Shape::Text(t) => out.push((t.galley.job.text.clone(), t.galley.job.justify)),
            egui::Shape::Vec(v) => {
                for s in v {
                    text(s, out)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-ipv4")
        .unwrap();
    a.launch(i);
    for (view, required) in [
        (
            teaching::View::Introduction,
            vec![
                "1. Read the prefix",
                "2. Calculate the block size",
                "139 is here",
                "192.168.5.128",
                "192.168.5.191",
                "192.168.5.129",
            ],
        ),
        (
            teaching::View::Example,
            vec![
                "What to do",
                "What to observe",
                "1. Select Fill worked example.",
                "broadcast address, 192.168.5.191",
            ],
        ),
        (
            teaching::View::Assignment,
            vec![
                "Find the network and broadcast addresses.",
                "different from Workstation 1.",
            ],
        ),
    ] {
        a.set_address_view(i, view);
        draw(&ctx, &mut a);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200., 1300.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        let mut rows = vec![];
        for s in output.shapes {
            text(&s.shape, &mut rows)
        }
        let all = rows
            .iter()
            .map(|r| r.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        for want in required {
            assert!(all.contains(want), "missing {want}: {all}");
        }
        assert!(!all.contains("Expected behavior"));
        assert!(!all.contains("assignment remains"));
        for (row, justify) in rows {
            if row.contains("The prefix ends")
                || row.contains("Select Fill worked")
                || row.contains("different from Workstation 1")
            {
                assert!(!justify, "stretched text: {row}");
            }
        }
    }
}
#[test]
fn ccna_revision_first_intro_teaches_the_whole_worked_method_without_opening_help() {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a) = app();
    a.open_magic_introduction();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 2000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for s in output.shapes {
        collect(&s.shape, &mut text);
    }
    for required in [
        "IPv4 foundations: the magic-number method",
        "Lesson 1 of 4",
        "Identify the relevant mask octet",
        "256 − 192 = 64",
        "0, 64, 128, 192",
        "64–127",
        "192.168.10.64",
        "192.168.10.127",
        "192.168.10.65",
        "192.168.10.126",
    ] {
        assert!(
            text.contains(required),
            "essential teaching hidden or absent: {required}"
        );
    }
    assert!(!a.metrics.help_visible);
}

fn magic_text(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn collect(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    let o = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 2200.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for s in o.shapes {
        collect(&s.shape, &mut text);
    }
    text
}
#[test]
fn ccna_magic_guidance_is_adjacent_dynamic_and_hides_the_containing_answer() {
    let (ctx, mut a) = app();
    a.open_magic_introduction();
    let i = a.selected.unwrap();
    session(&mut a, i).view = teaching::View::Example;
    for (prefix, octet, value, calc) in [
        (26, 4, 192, "256 − 192 ="),
        (20, 3, 240, "256 − 240 ="),
        (24, 3, 255, "256 − 255 ="),
    ] {
        session(&mut a, i).example.prefix = prefix.to_string();
        session(&mut a, i).example_checked = None;
        let text = magic_text(&ctx, &mut a);
        for want in [
            format!("/{prefix} ends in octet {octet}"),
            format!("Mask value in octet {octet}"),
            format!("IP octet {octet} · block start"),
            calc.into(),
            "Check answers".into(),
        ] {
            assert!(text.contains(&want), "missing {want}: {text}");
        }
        assert!(
            !text.contains("belongs in"),
            "unchecked diagram must not highlight the answer"
        );
        assert!(
            !text.contains("What to do"),
            "dense left-column instructions removed"
        );
        if prefix != 26 {
            assert!(!text.contains("256 − 192"));
        }
        let p = session(&mut a, i).example_problem().unwrap();
        assert_eq!(p.calculate().unwrap().mask_octet, value);
    }
    session(&mut a, i).example.prefix = "31".into();
    let text = magic_text(&ctx, &mut a);
    assert!(text.contains("special cases"));
    assert!(!text.contains("Calculate the block size"));
}
#[test]
fn ccna_magic_reveal_requires_correct_check_or_explicit_worked_answer() {
    let (ctx, mut a) = app();
    a.open_magic_introduction();
    let i = a.selected.unwrap();
    session(&mut a, i).view = teaching::View::Example;
    let p = session(&mut a, i).example_problem().unwrap();
    for (n, f) in session(&mut a, i)
        .stage
        .fields(p)
        .unwrap()
        .iter()
        .enumerate()
    {
        session(&mut a, i).example.answers[n] = f.expected.clone();
    }
    assert!(
        !magic_text(&ctx, &mut a).contains("141 belongs in 128–191"),
        "typing correct values alone must not reveal"
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(magic_text(&ctx, &mut a).contains("141 belongs in 128–191"));
    session(&mut a, i).example.answers[2] = "192".into();
    assert!(!magic_text(&ctx, &mut a).contains("141 belongs in"));
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    let text = magic_text(&ctx, &mut a);
    assert!(text.contains("Subtract the relevant mask value from 256"));
    assert!(!text.contains("141 belongs in"));
    draw(&ctx, &mut a);
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    assert!(magic_text(&ctx, &mut a).contains("141 belongs in 128–191"));
    assert!(session(&mut a, i).answers.iter().all(String::is_empty));
    assert!(!engine::solved(
        &a.progress.attempts[&a.scenarios[i].id].state,
        &a.scenarios[i].goal
    ));
    // Exercise a real native TextEdit event, not just changing model strings.
    draw(&ctx, &mut a);
    let r = a.metrics.magic_problem_inputs[1];
    click(&ctx, &mut a, r);
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
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            egui::Event::Text("20".into()),
        ],
    );
    assert_eq!(session(&mut a, i).example.prefix, "20");
    assert!(session(&mut a, i).example_checked.is_none());
    assert_eq!(session(&mut a, i).example.answers[0], "255.255.240.0");
    assert_eq!(session(&mut a, i).example.answers[1], "240");
    let text = magic_text(&ctx, &mut a);
    assert!(text.contains("/20 ends in octet 3"));
    assert!(!text.contains("belongs in"));
}
#[test]
fn ccna_magic_labels_formats_and_assignment_visibility_cover_all_stages() {
    let (ctx, mut a) = app();
    for (id, required, hidden) in [
        (
            "ccna-magic-blocks",
            "Mask value in octet 4",
            "192.168.50.128",
        ),
        (
            "ccna-magic-cross-octet",
            "Mask octet position (1–4)",
            "172.19.128.0",
        ),
        (
            "ccna-magic-octet-aligned",
            "Mask octet position (1–4)",
            "10.24.255.0",
        ),
    ] {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        session(&mut a, i).view = teaching::View::Assignment;
        let text = magic_text(&ctx, &mut a);
        assert!(text.contains(required));
        assert!(text.contains("Check answers"));
        assert!(!text.contains(hidden));
        assert!(!text.contains("Your address"));
        if id == "ccna-magic-blocks" {
            assert!(text.contains("Dotted-decimal mask"));
            assert!(text.contains("IP octet 4 · block end"));
        }
    }
    a.open_magic_introduction();
    let text = magic_text(&ctx, &mut a);
    assert!(text.contains("77 belongs in 64–127"));
    assert!(
        !text.contains("128 + 64 + 32"),
        "detailed arithmetic belongs in optional help"
    );
}
