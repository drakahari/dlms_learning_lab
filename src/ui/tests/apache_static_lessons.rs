//! Source controller checks. Real input/Render/Save/SmokeTick, not PNG/native
//! evidence. Runtime tests require root's normal140 registration first.
use super::super::smoke::apache_static_lessons::{FIRST, phases_for_test};
use super::*;
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!(
        a.scenarios.len(),
        140,
        "normal140 registration required, no fake filler scenarios"
    );
    assert!(
        a.units
            .iter()
            .any(|u| u.id == "static-web-services" && u.labs.len() == 4)
    );
    a.smoke = true;
    a.smoke_stage = FIRST;
    (ctx, a)
}
fn tick(ctx: &egui::Context, a: &mut LabApp, width: f32, height: f32, density: f32, drive: bool) {
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
    // Acknowledge screenshot requests only; no synthetic image or model result.
    a.capture_pending = false;
    let scale = a.scale;
    let n = ctx
        .data(|d| d.get_temp::<u64>(egui::Id::new("apache-test-time")))
        .unwrap_or(0)
        + 1;
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("apache-test-time"), n));
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / (scale * density), height / (scale * density)),
        )),
        time: Some(n as f64 * 0.1),
        events,
        modifiers,
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&egui::ViewportId::ROOT)
        .unwrap()
        .native_pixels_per_point = Some(density);
    let _ = ctx.run(input, |ctx| {
        a.render(ctx);
        a.save();
        a.started = Instant::now() - std::time::Duration::from_secs(1);
        if drive {
            a.smoke_tick(ctx);
        }
    });
}
#[test]
fn apache_lessons_normal_plan_has_closed_four_scope_and_bounded_finish() {
    let data = pack::load(include_str!("../../../scenarios/apache-static-v1.json")).unwrap();
    assert_eq!(data.len(), 4);
    let (last, insert, count, shots) = phases_for_test(&data);
    assert_eq!(last, FIRST + count - 1);
    assert!(last < 37000 && insert > FIRST + 4);
    assert!(
        (40..=72).contains(&shots),
        "bounded representative requested images"
    );
    assert!(count < 1000);
}
#[test]
fn apache_lessons_actual_opening_geometry_and_insert_phase_use_real_context_without_execution() {
    let (ctx, mut a) = opened();
    let (_, insert, _, _) = phases_for_test(&a.scenarios);
    for _ in 0..1500 {
        tick(&ctx, &mut a, 1200., 850., 1., true);
        if a.smoke_stage > insert {
            break;
        }
    }
    assert_eq!(a.smoke_stage, insert + 1);
    let measured = ctx
        .data(|d| {
            d.get_temp::<(u8, u32, u32, u32, u32)>(egui::Id::new(("apache-geometry", FIRST + 1)))
        })
        .unwrap();
    assert!(
        measured.0 <= 20 && f32::from_bits(measured.3) > 0. && f32::from_bits(measured.4) == 1.
    );
    let i = a.selected.unwrap();
    assert_eq!(a.scenarios[i].id, "lpic2-web-routing");
    let attempt = &a.progress.attempts[&a.scenarios[i].id];
    assert!(
        serde_json::to_value(&attempt.state).unwrap()
            == serde_json::to_value(&a.scenarios[i].initial).unwrap()
    );
    let lesson = a.scenarios[i].apache_static_teaching.as_ref().unwrap();
    let l = attempt
        .workspace
        .as_ref()
        .unwrap()
        .apache_static_learning
        .as_ref()
        .unwrap();
    assert!(
        serde_json::to_value(&l.simulation).unwrap()
            == serde_json::to_value(&lesson.definition.initial).unwrap()
    );
    assert_ne!(l.draft.config_text, l.simulation.saved);
    assert!(!progress::current_solved(attempt, &a.scenarios[i]));
}
#[test]
fn apache_lessons_complete_real_input_plan_with_idle_autosave_and_held_finish() {
    let (ctx, mut a) = opened();
    let (last, _, _, _) = phases_for_test(&a.scenarios);
    for _ in 0..18000 {
        // The explicit wide/narrow requests are represented at measured density
        // in headless geometry; native must separately report its actual window.
        let (w, h) = if a.scale == 1.5 {
            (1920., 1080.)
        } else if a.scale == 2. {
            (946., 929.)
        } else {
            (1200., 850.)
        };
        tick(&ctx, &mut a, w, h, 1.15, true);
        // Native has idle render/autosave frames between controller ticks; these
        // must not continually force feedback scrolling or mutate saved records.
        for _ in 0..2 {
            tick(&ctx, &mut a, w, h, 1.15, false);
        }
        if a.smoke_stage == last {
            break;
        }
    }
    assert_eq!(
        a.smoke_stage, last,
        "bounded full real-input plan reaches Finish"
    );
    for _ in 0..2 {
        tick(&ctx, &mut a, 1200., 850., 1.15, true);
        assert_eq!(a.smoke_stage, last);
        assert!(a.pending_events.is_empty() && !a.capture_pending);
    }
    assert_eq!(a.filter.unit, "static-web-services");
    assert_eq!(a.metrics.card_actions.len(), 4);
}
#[test]
fn apache_lessons_unknown_phase_and_missing_normal_registration_fail_closed() {
    let (ctx, mut a) = app();
    a.smoke = true;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        a.apache_static_lessons_native(&ctx, 37000)
    }));
    assert!(result.is_err());
    // Removing one required registered ID must not silently run a developer lab.
    a.scenarios.retain(|s| s.id != "lpic2-web-routing");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        a.apache_static_lessons_native(&ctx, FIRST)
    }));
    assert!(result.is_err());
}
