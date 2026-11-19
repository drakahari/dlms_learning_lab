//! Real egui event paths; representative rendering, not an independent novice study.
use super::*;
use dlms_learning_lab::{engine::State, raid1_workspace as work};
const IDS: [&str; 2] = ["lpic2-raid1-replace", "lpic2-raid1-handover"];
fn opened(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = super::debug_recovery::opened(id);
    set_view(&mut a, i, view);
    frame(&ctx, &mut a, 1200., 850., vec![]);
    (ctx, a, i)
}
fn set_view(a: &mut LabApp, i: usize, view: teaching::View) {
    let s = a.scenarios[i].clone();
    let engine::Goal::Raid1 { definition } = &s.goal else {
        panic!()
    };
    let State::Raid1 { simulation } = &mut a.progress.attempts.get_mut(&s.id).unwrap().state else {
        panic!()
    };
    simulation.set_view(definition, view).unwrap();
}
fn sim<'a>(a: &'a LabApp, id: &str) -> &'a work::Session {
    let State::Raid1 { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
fn settle(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32) {
    for _ in 0..4 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn click(ctx: &egui::Context, a: &mut LabApp, name: &str, w: f32, h: f32) {
    for _ in 0..30 {
        let r = a
            .metrics
            .raid1
            .controls
            .iter()
            .find(|r| r.0 == name)
            .unwrap_or_else(|| panic!("Missing {name}"))
            .1;
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if crate::ui::smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            super::debian_contents::point_click(ctx, a, w, h, r);
            return;
        }
        let dy = if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom()
        } else {
            clip.top() - r.top()
        };
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        settle(ctx, a, w, h);
    }
    panic!(
        "{name} did not become visible; scale={} rect={:?} viewport={:?} content={:?} offset={:?}",
        a.scale,
        a.metrics.raid1.controls.iter().find(|c| c.0 == name),
        a.metrics.main_viewport,
        a.metrics.main_content_size,
        a.metrics.main_scroll_offset
    )
}
fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    click(ctx, a, "command", 1200., 850.);
    let id = a.metrics.raid1.command.unwrap().1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    settle(ctx, a, 1200., 850.);
    frame(ctx, a, 1200., 850., vec![key(egui::Key::Enter, false)]);
    settle(ctx, a, 1200., 850.);
}
#[test]
fn raid1_three_views_teach_before_work_and_challenge_does_not_inherit_revealed_answers() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Introduction);
        let text = super::debian_contents::painted(&ctx, &mut a);
        assert!(text.contains("Worked example") && text.contains("usable"));
        let example = if id.ends_with("replace") {
            "/dev/md30"
        } else {
            "/dev/md32"
        };
        assert!(text.contains(example));
        set_view(&mut a, i, teaching::View::Example);
        let text = super::debian_contents::painted(&ctx, &mut a);
        assert!(text.contains("Replacement inventory") && text.contains("Insert command"));
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        set_view(&mut a, i, teaching::View::Assignment);
        let text = super::debian_contents::painted(&ctx, &mut a);
        assert!(text.contains("Requirements"));
        assert!(!text.contains(example));
        for c in &a.scenarios[i].solution {
            assert!(!text.contains(c), "independent solution exposed: {c}");
        }
        click(&ctx, &mut a, "help", 1200., 850.);
        click(&ctx, &mut a, "worked-help", 1200., 850.);
        let revealed = super::debian_contents::painted(&ctx, &mut a);
        assert!(revealed.contains(&a.scenarios[i].solution[0]));
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        set_view(&mut a, i, teaching::View::Introduction);
        let text = super::debian_contents::painted(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!text.contains(example));
        for c in a.scenarios[i]
            .solution
            .iter()
            .filter(|c| !c.starts_with(work::PREFIX))
        {
            assert!(
                !text.contains(c),
                "Challenge inherited a revealed solution: {c}"
            );
        }
        assert!(text.contains("Run command"));
    }
}
#[test]
fn raid1_real_terminal_and_advance_controls_grade_partial_wrong_correct_and_broken_state() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id, teaching::View::Assignment);
        settle(&ctx, &mut a, 1200., 850.);
        click(&ctx, &mut a, "check", 1200., 850.);
        assert!(!a.metrics.completion);
        let s = a.scenarios[i].clone();
        let small = if id.ends_with("replace") {
            "mdadm /dev/md10 --add /dev/vdd1"
        } else {
            "mdadm /dev/md12 --add /dev/sdd1"
        };
        command(&ctx, &mut a, small);
        assert_eq!(sim(&a, id).transcript.last().unwrap().status, 1);
        assert!(
            sim(&a, id)
                .transcript
                .last()
                .unwrap()
                .output
                .contains("usable MD capacity")
        );
        for c in s.solution.iter().take(s.solution.len() - 1) {
            command(&ctx, &mut a, c);
        }
        click(&ctx, &mut a, "advance", 1200., 850.);
        click(&ctx, &mut a, "check", 1200., 850.);
        let (passed, total, _) = a.metrics.raid1.result.unwrap();
        assert!(passed < total);
        assert!(!a.metrics.completion);
        click(&ctx, &mut a, "advance", 1200., 850.);
        click(&ctx, &mut a, "check", 1200., 850.);
        assert!(a.metrics.completion);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        let broken = if id.ends_with("replace") {
            "mdadm /dev/md10 --fail /dev/vdc1"
        } else {
            "mdadm /dev/md12 --fail /dev/sdc1"
        };
        command(&ctx, &mut a, broken);
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let loaded = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded.attempts[id]).unwrap(),
            serde_json::to_value(&a.progress.attempts[id]).unwrap()
        );
    }
}
#[test]
fn raid1_practice_insert_is_inert_and_a_run_cannot_change_the_assignment_draft() {
    let id = IDS[0];
    let (ctx, mut a, i) = opened(id, teaching::View::Assignment);
    click(&ctx, &mut a, "command", 1200., 850.);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("mdadm --detail /dev/md10".into())],
    );
    settle(&ctx, &mut a, 1200., 850.);
    let expected = sim(&a, id).current.clone();
    let draft = sim(&a, id).command.clone();
    set_view(&mut a, i, teaching::View::Example);
    settle(&ctx, &mut a, 1200., 850.);
    click(&ctx, &mut a, "insert", 1200., 850.);
    assert_eq!(sim(&a, id).current, expected);
    assert_eq!(sim(&a, id).command, draft);
    assert!(sim(&a, id).example_transcript.is_empty());
    click(&ctx, &mut a, "run", 1200., 850.);
    assert_eq!(sim(&a, id).current, expected);
    assert_eq!(sim(&a, id).command, draft);
    assert!(!progress::current_solved(
        &a.progress.attempts[id],
        &a.scenarios[i]
    ));
}
#[test]
fn raid1_invalid_packets_preserve_the_exact_draft_without_running_its_old_text() {
    let id = IDS[0];
    let (ctx, mut a, _) = opened(id, teaching::View::Assignment);
    click(&ctx, &mut a, "command", 1200., 850.);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text(
            "mdadm /dev/md10 --remove /dev/vdb1".into(),
        )],
    );
    settle(&ctx, &mut a, 1200., 850.);
    let old = sim(&a, id).clone();
    for events in [
        vec![
            egui::Event::Paste("x".repeat(513)),
            key(egui::Key::Enter, false),
        ],
        vec![
            egui::Event::Paste("a\nb".into()),
            key(egui::Key::Enter, false),
        ],
        vec![
            key(egui::Key::Enter, false),
            egui::Event::Text("ignored after Enter".into()),
        ],
    ] {
        click(&ctx, &mut a, "command", 1200., 850.);
        frame(&ctx, &mut a, 1200., 850., events);
        settle(&ctx, &mut a, 1200., 850.);
        assert_eq!(sim(&a, id), &old);
    }
}
#[test]
fn raid1_reflows_populated_command_and_visible_controls_in_light_dark_and_supported_scales() {
    let (ctx, mut a, i) = opened(IDS[1], teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    let engine::Goal::Raid1 { definition } = &s.goal else {
        panic!()
    };
    let State::Raid1 { simulation } = &mut a.progress.attempts.get_mut(IDS[1]).unwrap().state
    else {
        panic!()
    };
    simulation
        .set_command(definition, "mdadm --detail /dev/md12".into(), false)
        .unwrap();
    for light in [false, true] {
        let t = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap();
        a.theme_id = t.id.clone();
        theme::apply(&ctx, &t.id);
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [760., 1280.] {
                settle(&ctx, &mut a, width, 900.);
                for (key, r, _) in &a.metrics.raid1.controls {
                    assert!(
                        r.left() >= -1. && r.right() <= ctx.content_rect().right() + 1.,
                        "{}/{scale}/{width}: {key}: {r:?}",
                        t.id
                    );
                }
                let (_, id) = a.metrics.raid1.command.unwrap();
                ctx.memory_mut(|m| m.request_focus(id));
                settle(&ctx, &mut a, width, 900.);
                assert!(ctx.memory(|m| m.has_focus(id)));
                assert_eq!(sim(&a, IDS[1]).command, "mdadm --detail /dev/md12");
            }
        }
    }
}
#[test]
fn raid1_last_guided_menu_action_is_reachable_in_narrow_enlarged_native_popup() {
    for scale in [1., 1.5, 2.] {
        let (ctx, mut a, _) = opened(IDS[0], teaching::View::Example);
        a.scale = scale;
        settle(&ctx, &mut a, 760., 632.);
        let before = sim(&a, IDS[0]).clone();
        click(&ctx, &mut a, "example-picker", 760., 632.);
        a.smoke_stage = 945001;
        let mut picked = false;
        for n in 0..200 {
            let events = std::mem::take(&mut a.pending_events);
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760. / a.scale, 632. / a.scale),
                    )),
                    time: Some(50. + n as f64 * 0.05),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    a.render(ctx);
                    picked = a.raid1_popup_choice(ctx, 5);
                },
            );
            if picked {
                break;
            }
        }
        assert!(picked, "Last guided action must be clickable at{scale}");
        assert_eq!(sim(&a, IDS[0]).selected_example, 5);
        assert_eq!(sim(&a, IDS[0]).current, before.current);
        assert_eq!(sim(&a, IDS[0]).example, before.example);
        assert!(sim(&a, IDS[0]).example_transcript.is_empty());
    }
}
