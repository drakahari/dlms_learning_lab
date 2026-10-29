//! Real-event component route; separate native run must prove actual window geometry.
use super::*;
use crate::ui::smoke::private_office;
const BASE: usize = 72000;
fn full(p: &Progress) -> serde_json::Value {
    let mut v = serde_json::to_value(p).unwrap();
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
#[test]
fn normal_private_office_real_controls_examples_mask_failure_alternatives_resume_reset_and_layout()
{
    let (ctx, mut a) = app();
    a.smoke = true;
    a.smoke_stage = BASE;
    assert_eq!(a.scenarios.len(), 176);
    assert_eq!(a.units.len(), 46);
    assert!(a.progress.attempts.is_empty());
    let mut fullscreen = false;
    let mut shots = Vec::new();
    let mut finished = false;
    let total = private_office::route_len();
    let mut time = 0.;
    for _ in 0..9000 {
        time += 0.05;
        let (w, h) = if fullscreen {
            (1920., 1080.)
        } else {
            (946., 929.)
        };
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
        let mut input = egui::RawInput {
            time: Some(time),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w / a.scale, h / a.scale),
            )),
            events,
            modifiers,
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .fullscreen = Some(fullscreen);
        let output = ctx.run(input, |ctx| {
            a.render(ctx);
            a.save();
            let stage = a.smoke_stage;
            let (advance, shot) = a.private_office_native(ctx, stage);
            if let Some(name) = shot {
                assert!(!shots.contains(&name));
                shots.push(name)
            }
            if advance {
                a.smoke_stage += 1;
            }
            if a.smoke_stage == BASE + total {
                finished = true;
            }
        });
        for viewport in output.viewport_output.values() {
            for command in &viewport.commands {
                if let egui::ViewportCommand::Fullscreen(value) = command {
                    fullscreen = *value;
                }
            }
        }
        if finished {
            break;
        }
    }
    assert!(
        finished,
        "Bounded real-event route stalled at {}",
        a.smoke_stage
    );
    assert!(
        shots
            .iter()
            .any(|x| x.ends_with("traffic-passes-masks-disagree"))
    );
    assert!(
        shots
            .iter()
            .any(|x| x.ends_with("unfinished-draft-resumed"))
    );
    assert_eq!(
        shots
            .iter()
            .filter(|x| x.ends_with("matrix-assignment"))
            .count(),
        20
    );
    assert_eq!(
        shots
            .iter()
            .filter(|x| x.ends_with("matrix-200-workspace"))
            .count(),
        4
    );
}
#[test]
fn private_office_reset_comparison_rejects_corruption_of_every_retained_progress_field() {
    let (_ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-private-office-plan")
        .unwrap();
    a.launch(i);
    let s = &a.scenarios[i];
    let before = full(&a.progress);
    let expected = private_office::reset_expected(&before, s);
    assert!(private_office::reset_preserved(&before, &expected, s));
    let mut edits = Vec::new();
    for key in ["clock", "schema_version"] {
        let mut v = expected.clone();
        v[key] = 999.into();
        edits.push(v);
    }
    let mut v = expected.clone();
    v["favorites"] = serde_json::json!(["ccna-ipv4"]);
    edits.push(v);
    let mut v = expected.clone();
    v["migration_receipts"] = serde_json::json!({"workspace":true,"shell":[12]});
    edits.push(v);
    for (key, value) in [
        ("visited", serde_json::json!(999)),
        ("step", serde_json::json!(1)),
        ("hints", serde_json::json!(1)),
        ("challenge", serde_json::json!(true)),
        ("transcript", serde_json::json!(["extra"])),
    ] {
        let mut v = expected.clone();
        v["attempts"][&s.id][key] = value;
        edits.push(v);
    }
    let mut v = expected.clone();
    v["attempts"][&s.id]["workspace"]["routing_v4_draft"]["selected_device"] = "Birch".into();
    edits.push(v);
    assert_eq!(edits.len(), 10);
    for (i, edited) in edits.iter().enumerate() {
        assert_ne!(edited, &expected);
        assert!(
            !private_office::reset_preserved(&before, edited, s),
            "reset corruption {i} must be detected"
        );
    }
}
