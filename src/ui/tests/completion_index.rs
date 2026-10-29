//! Seven technical sources test ordinary library/Start/Continue controls. They
//! are synthetic fixtures, not seven new authored curriculum scenarios.
use super::*;
use crate::ui::completion_index::Status;
use crate::ui::tests::physical_workspace as fixture;
fn native_frame(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1600., 1100., events);
}
fn tap(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    for pressed in [true, false] {
        native_frame(
            ctx,
            a,
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
    for _ in 0..3 {
        native_frame(ctx, a, vec![]);
    }
}
fn wheel(ctx: &egui::Context, a: &mut LabApp, delta: f32) {
    let center = a
        .metrics
        .main_viewport
        .unwrap_or(ctx.content_rect())
        .center();
    native_frame(
        ctx,
        a,
        vec![
            egui::Event::PointerMoved(center),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., delta),
                modifiers: Default::default(),
            },
        ],
    );
    for _ in 0..2 {
        native_frame(ctx, a, vec![]);
    }
}
fn control(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..70 {
        if let Some((_, r, _)) = a
            .metrics
            .physical
            .controls
            .iter()
            .find(|(id, _, _)| id == key)
        {
            let r = *r;
            let clip = a
                .metrics
                .main_viewport
                .unwrap_or(ctx.content_rect())
                .intersect(ctx.content_rect());
            if crate::ui::smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
                return r;
            }
            wheel(ctx, a, if r.top() < clip.top() { 240. } else { -240. });
        } else {
            wheel(ctx, a, -240.);
        }
    }
    panic!("Physical native control {key} unreachable");
}
fn click_control(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = control(ctx, a, key);
    tap(ctx, a, r);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    click_control(ctx, a, key);
    let r = a
        .metrics
        .physical
        .choices
        .iter()
        .find(|(id, v, _, enabled)| id == key && v == value && *enabled)
        .unwrap_or_else(|| panic!("No native choice {key}/{value}"))
        .2;
    tap(ctx, a, r);
}
fn back(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..40 {
        if let Some(r) = a.metrics.back
            && crate::ui::smoke::smoke_control_visible(
                r,
                a.metrics
                    .main_viewport
                    .unwrap_or(ctx.content_rect())
                    .intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            )
        {
            tap(ctx, a, r);
            return;
        }
        wheel(ctx, a, 260.);
    }
    panic!("Header Back action unreachable");
}
fn search(ctx: &egui::Context, a: &mut LabApp, n: usize) {
    for _ in 0..40 {
        if let Some(r) = a.metrics.library_search
            && crate::ui::smoke::smoke_control_visible(
                r,
                a.metrics
                    .main_viewport
                    .unwrap_or(ctx.content_rect())
                    .intersect(ctx.content_rect()),
                ctx.pixels_per_point(),
            )
        {
            tap(ctx, a, r);
            native_frame(
                ctx,
                a,
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
                    egui::Event::Paste(format!("Index technical workspace {n}")),
                ],
            );
            for _ in 0..4 {
                native_frame(ctx, a, vec![]);
            }
            assert_eq!(a.metrics.card_actions.len(), 1);
            return;
        }
        wheel(ctx, a, 260.);
    }
    panic!("Search field unreachable");
}
fn start(ctx: &egui::Context, a: &mut LabApp, n: usize) {
    search(ctx, a, n);
    for _ in 0..40 {
        let r = a.metrics.card_actions[0].0;
        let clip = a
            .metrics
            .main_viewport
            .unwrap_or(ctx.content_rect())
            .intersect(ctx.content_rect());
        if crate::ui::smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            tap(ctx, a, r);
            assert!(a.selected.is_some());
            return;
        }
        wheel(ctx, a, if r.top() < clip.top() { 240. } else { -240. });
    }
    panic!("Normal Start/Continue action unreachable");
}
fn solve_controls(ctx: &egui::Context, a: &mut LabApp) {
    click_control(ctx, a, "skip-intro");
    choose(ctx, a, "cable", "assignment-clean");
    choose(ctx, a, "mode:assignment-right", "Auto");
    click_control(ctx, a, "apply");
    click_control(ctx, a, "check");
    assert!(a.metrics.physical.complete);
}
fn seven_sources(a: &mut LabApp) -> Vec<String> {
    (0..7)
        .map(|n| {
            let mut s = fixture::source();
            s.id = format!("test-index-normal-{n}");
            s.title = format!("Index technical workspace {n}");
            let id = s.id.clone();
            a.scenarios.push(s);
            id
        })
        .collect()
}
#[test]
fn exact_index_seven_normal_start_solve_back_continue_and_idle_keep_evicted_completion_current() {
    let (ctx, mut a) = app();
    let ids = seven_sources(&mut a);
    native_frame(&ctx, &mut a, vec![]);
    assert_eq!(ids.len(), 7);
    for (n, id) in ids.iter().enumerate() {
        start(&ctx, &mut a, n);
        solve_controls(&ctx, &mut a);
        back(&ctx, &mut a);
        assert_eq!(
            a.completion_index.status(
                &a.progress.attempts[id],
                a.scenarios.iter().find(|s| &s.id == id).unwrap()
            ),
            Some(Status::Complete)
        );
    }
    assert!(a.physical_presentation.observed().3 <= 4);
    assert_eq!(a.completion_index.observed().0, 7);
    for n in 0..7 {
        search(&ctx, &mut a, n);
        assert_eq!(a.metrics.card_verification[0].1, Some(Status::Complete));
    }
    let before = a.completion_index.observed();
    let p = a.physical_presentation.observed();
    let json = dlms_learning_lab::native_observations::json_admission_calls();
    for _ in 0..80 {
        native_frame(&ctx, &mut a, vec![]);
    }
    let after = a.completion_index.observed();
    assert_eq!(
        (
            after.0,
            after.1,
            after.2.contexts,
            after.2.key_copies,
            after.2.certificates
        ),
        (
            before.0,
            before.1,
            before.2.contexts,
            before.2.key_copies,
            before.2.certificates
        )
    );
    assert_eq!(a.physical_presentation.observed(), p);
    assert_eq!(
        dlms_learning_lab::native_observations::json_admission_calls(),
        json
    );
    start(&ctx, &mut a, 0);
    assert!(a.metrics.physical.complete);
    click_control(&ctx, &mut a, "up:assignment-left");
    assert!(!a.metrics.physical.complete);
    click_control(&ctx, &mut a, "apply");
    click_control(&ctx, &mut a, "check");
    assert!(!a.metrics.physical.complete);
    back(&ctx, &mut a);
    assert_eq!(a.metrics.card_verification[0].1, Some(Status::Incomplete));
    for id in &ids[1..] {
        let s = a.scenarios.iter().find(|s| &s.id == id).unwrap();
        assert_eq!(
            a.completion_index.status(&a.progress.attempts[id], s),
            Some(Status::Complete)
        );
    }
}
#[test]
fn exact_index_normal_reset_preserves_other_certificates_and_real_restore_clears_old_authority() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    let ids = seven_sources(&mut a);
    native_frame(&ctx, &mut a, vec![]);
    for n in 0..2 {
        start(&ctx, &mut a, n);
        solve_controls(&ctx, &mut a);
        back(&ctx, &mut a);
    }
    let Progress {
        schema_version,
        clock,
        favorites,
        attempts,
        migration_receipts,
    } = &a.progress;
    let saved = Progress {
        schema_version: *schema_version,
        clock: *clock,
        favorites: favorites.clone(),
        attempts: attempts.clone(),
        migration_receipts: migration_receipts.clone(),
    };
    start(&ctx, &mut a, 0);
    let mut reset_opened = false;
    for _ in 0..30 {
        if let Some(r) = a.metrics.reset {
            let clip = a
                .metrics
                .main_viewport
                .unwrap_or(ctx.content_rect())
                .intersect(ctx.content_rect());
            let visible = crate::ui::smoke::smoke_control_visible(r, clip, ctx.pixels_per_point());
            println!(
                "Index Reset actual rect={r:?}, clip={clip:?}, visible={visible}, selected={:?}, pending={}, settings={}, theme={}, data_modal={}, rejected={}",
                a.selected,
                a.reset_pending,
                a.settings_open,
                a.theme_preview,
                a.data_tools.modal_open(),
                a.metrics.physical.rejected
            );
            if visible {
                tap(&ctx, &mut a, r);
                assert!(
                    a.reset_pending,
                    "Actual visible Reset click must open the confirmation"
                );
                assert!(a.metrics.confirm_reset.is_some());
                reset_opened = true;
                break;
            }
        }
        wheel(&ctx, &mut a, 260.);
    }
    assert!(
        reset_opened,
        "Reset must be genuinely visible and open within the unchanged bound"
    );
    let r = a.metrics.confirm_reset.unwrap();
    tap(&ctx, &mut a, r);
    let other = a.scenarios.iter().find(|s| s.id == ids[1]).unwrap();
    assert_eq!(
        a.completion_index
            .status(&a.progress.attempts[&ids[1]], other),
        Some(Status::Complete)
    );
    back(&ctx, &mut a);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        serde_json::to_value(&saved.migration_receipts).unwrap(),
        "Selected Reset preserves actual transition provenance"
    );
    dlms_learning_lab::backup::export(
        &dir.path().join("dlms-learning-backup.json"),
        &saved,
        &a.scenarios,
    )
    .unwrap();
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let generation = a.completion_index.generation();
    let r = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.completion_index.generation() > generation);
    assert_eq!(
        a.progress.attempts[&ids[0]].state,
        saved.attempts[&ids[0]].state
    );
    let other = a.scenarios.iter().find(|s| s.id == ids[1]).unwrap();
    assert_eq!(
        a.completion_index
            .status(&a.progress.attempts[&ids[1]], other),
        Some(Status::Pending)
    );
}
#[test]
fn exact_index_one_background_proof_per_root_frame_even_when_egui_discards_a_pass() {
    let (ctx, mut a) = app();
    let ids = seven_sources(&mut a);
    native_frame(&ctx, &mut a, vec![]);
    // Real Start/Back creates two uncompleted attempts. Only ephemeral caches are
    // then reset to represent a cold verifier; no passing State/Progress is injected.
    for n in 0..2 {
        start(&ctx, &mut a, n);
        back(&ctx, &mut a);
    }
    a.physical_presentation.reset_all();
    a.completion_index.clear();
    a.completion_index.page(ids[..2].iter().cloned());
    ctx.memory_mut(|m| m.options.max_passes = std::num::NonZeroUsize::new(2).unwrap());
    let before = a.completion_index.observed().2;
    let before_work = a.physical_presentation.observed();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1600., 1100.),
        )),
        ..Default::default()
    };
    let output=ctx.run(input,|ctx|{a.render(ctx);if ctx.current_pass_index()==0{ctx.request_discard("Exercise the actual second layout pass without resetting the root-frame work lease");}});
    assert_eq!(output.platform_output.num_completed_passes, 2);
    let after = a.completion_index.observed();
    assert_eq!(after.0, 1);
    assert_eq!(after.2.certificates - before.certificates, 1);
    assert_eq!(after.2.key_copies - before.key_copies, 1);
    let work = a.physical_presentation.observed();
    assert_eq!(work.0 - before_work.0, 1);
    assert_eq!(work.1 - before_work.1, 1);
    assert_eq!(
        work.2 - before_work.2,
        2,
        "One admitted proof computes two scope vectors"
    );
    native_frame(&ctx, &mut a, vec![]);
    let next = a.completion_index.observed();
    assert_eq!(next.0, 2);
    assert_eq!(next.2.certificates - before.certificates, 2);
    assert_eq!(next.2.key_copies - before.key_copies, 2);
    let work = a.physical_presentation.observed();
    assert_eq!(work.0 - before_work.0, 2);
    assert_eq!(work.1 - before_work.1, 2);
    assert_eq!(work.2 - before_work.2, 4);
    for id in &ids[..2] {
        let s = a.scenarios.iter().find(|s| &s.id == id).unwrap();
        assert_eq!(
            a.completion_index.status(&a.progress.attempts[id], s),
            Some(Status::Incomplete)
        );
    }
}
