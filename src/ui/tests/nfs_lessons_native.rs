//! Actual smoke_tick and egui events with screenshot-command acknowledgement.
//! These tests are not a Fedora-window, novice or assistive-device acceptance claim.
use super::super::smoke::nfs_lessons::{FIRST, IDS, checkpoints, phase_bounds};
use super::*;
#[derive(Default)]
struct Witness {
    frames: usize,
    shots: Vec<String>,
    focused: usize,
    reads: usize,
    close: usize,
    fullscreen: Vec<bool>,
}
fn opened() -> (egui::Context, LabApp) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (168, 43));
    assert!(a.progress.attempts.is_empty());
    a.smoke = true;
    a.smoke_stage = FIRST;
    (ctx, a)
}
fn painted(shape: &egui::epaint::Shape, text: &str) -> bool {
    match shape {
        egui::epaint::Shape::Text(t) => t.galley.job.text.contains(text),
        egui::epaint::Shape::Vec(v) => v.iter().any(|s| painted(s, text)),
        _ => false,
    }
}
fn frame(
    ctx: &egui::Context,
    a: &mut LabApp,
    age: bool,
    w: &mut Witness,
    drive: bool,
) -> egui::FullOutput {
    let events = std::mem::take(&mut a.pending_events);
    w.frames += 1;
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
    let (width, height) = if a.scale == 1. {
        (1920., 1080.)
    } else {
        (946., 929.)
    };
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width / (a.scale * 1.15), height / (a.scale * 1.15)),
        )),
        time: Some(w.frames as f64 / 20.),
        events,
        modifiers,
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
        if age {
            a.started = Instant::now() - std::time::Duration::from_secs(1);
        }
        if drive {
            a.smoke_tick(ctx);
        }
    });
    let mut screenshots = 0;
    for viewport in out.viewport_output.values() {
        for command in &viewport.commands {
            match command {
                egui::ViewportCommand::Fullscreen(b) => w.fullscreen.push(*b),
                egui::ViewportCommand::Close => w.close += 1,
                egui::ViewportCommand::Screenshot(data) => {
                    screenshots += 1;
                    let label = data
                        .data
                        .as_ref()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    let root = a
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    if label.contains("focused-relative") || label.contains("resumed-example-draft")
                    {
                        let r = a
                            .metrics
                            .nfs
                            .fields
                            .iter()
                            .find(|r| r.0 == "relative")
                            .unwrap();
                        assert!(r.3 && ctx.memory(|m| m.has_focus(r.2)));
                        assert!(super::super::smoke::smoke_control_visible(
                            r.1,
                            r.4.intersect(root),
                            ctx.pixels_per_point()
                        ));
                        let s = &a.scenarios[a.selected.unwrap()];
                        let a = &a.progress.attempts[&s.id];
                        let l = a.workspace.as_ref().unwrap().nfs_learning.as_ref().unwrap();
                        let session = if l.view == teaching::View::Example && !a.challenge {
                            &l.session
                        } else {
                            a.workspace.as_ref().unwrap().nfs_session.as_ref().unwrap()
                        };
                        assert!(!session.relative.is_empty());
                        w.focused += 1;
                    }
                    if label.contains("read-output") || label.contains("actual-read") {
                        let (r, v) = a
                            .metrics
                            .nfs
                            .read_output
                            .expect("Actual rendered read text geometry");
                        assert!(
                            a.metrics.nfs.read_shown
                                && super::super::smoke::smoke_control_visible(
                                    r,
                                    v.intersect(root),
                                    ctx.pixels_per_point()
                                )
                        );
                        let s = &a.scenarios[a.selected.unwrap()];
                        let attempt = &a.progress.attempts[&s.id];
                        let entry = attempt
                            .workspace
                            .as_ref()
                            .unwrap()
                            .nfs_session
                            .as_ref()
                            .unwrap()
                            .entries
                            .last()
                            .unwrap();
                        assert_eq!(entry.operation, "Read existing file");
                        let (_, expected) = super::super::smoke::nfs_lessons::completed_read(s);
                        assert_eq!(
                            entry.detail,
                            expected.strip_prefix("read:").unwrap(),
                            "Actual painted read has the exact current-goal text"
                        );
                        assert!(
                            out.shapes.iter().any(|s| painted(&s.shape, &entry.detail)),
                            "Capture phase actually paints the observed read text"
                        );
                        w.reads += 1;
                    }
                    w.shots.push(label);
                }
                _ => {}
            }
        }
    }
    if a.capture_pending {
        assert!(
            screenshots > 0,
            "Only acknowledge an actual Screenshot command"
        );
        a.capture_pending = false;
    }
    out
}
fn until(ctx: &egui::Context, a: &mut LabApp, target: usize, w: &mut Witness) {
    let mut previous = a.smoke_stage;
    let mut stalled = 0;
    for _ in 0..100_000 {
        if a.smoke_stage == target {
            return;
        }
        assert!(a.smoke_stage < target);
        if previous == a.smoke_stage {
            stalled += 1;
        } else {
            previous = a.smoke_stage;
            stalled = 0;
        }
        assert!(stalled <= 400, "Actual NFS step{} stalled", a.smoke_stage);
        frame(ctx, a, true, w, true);
    }
    panic!("Bounded four-task NFS event plan");
}
#[test]
fn normal_nfs_plan_has_exact_registered_scope_and_closed_phase_capture_budget() {
    let (_, a) = opened();
    assert_eq!(LabApp::nfs_lessons_stage(false, Some("nfs-lessons")), None);
    assert_eq!(
        LabApp::nfs_lessons_stage(true, Some("nfs-lessons-extra")),
        None
    );
    assert_eq!(
        LabApp::nfs_lessons_stage(true, Some("nfs-lessons")),
        Some(FIRST)
    );
    let (first, last) = phase_bounds(&a.scenarios);
    assert_eq!(first, 45000);
    assert!(last < 46000 && last > FIRST);
    assert_eq!(last - first + 1, 972, "Closed four-task phase expansion");
    let marks = checkpoints(&a.scenarios);
    assert_eq!(
        marks
            .iter()
            .filter(|(n, _)| n.starts_with("select-"))
            .count(),
        4
    );
    assert_eq!(marks.last().unwrap().0, "finish");
    assert_eq!(marks.last().unwrap().1, last);
    assert_eq!(
        marks.iter().filter(|(n, _)| n.starts_with("nfs-")).count(),
        106,
        "Every original capture retained within the128-capture budget",
    );
    assert_eq!(
        a.scenarios[150..154]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
}
#[test]
fn completed_reads_use_current_goal_clients_and_exact_authored_text() {
    let (_, a) = opened();
    let expected = [
        (
            "c-manuals-research",
            "/mnt/manuals/reference",
            "p-manuals-owner",
            "notice.txt",
            "Operations handbook: revision blue\n",
        ),
        (
            "c-dispatch-editor",
            "/mnt/dispatch/work",
            "p-dispatch-owner",
            "result.txt",
            "Dispatch approved for distribution\n",
        ),
        (
            "c-handoff-reader",
            "/mnt/handoff/work",
            "p-handoff-owner",
            "result.txt",
            "Draft: publication has not happened\n",
        ),
        (
            "c-pressroom-reader",
            "/mnt/pressroom/reports",
            "p-pressroom-owner",
            "notice.txt",
            "Daily reports: retained original\n",
        ),
    ];
    for (id, (client, mountpoint, profile, relative, text)) in IDS.into_iter().zip(expected) {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        let (action, kind) = super::super::smoke::nfs_lessons::completed_read(s);
        assert_eq!(
            action,
            dlms_learning_lab::nfs::Action::ReadExisting {
                client: client.into(),
                mountpoint: mountpoint.into(),
                profile: profile.into(),
                relative: relative.into(),
            }
        );
        assert_eq!(kind, format!("read:{text}"));
    }
    let review: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/nfs-review.json")).unwrap();
    assert_eq!(
        review["lessons"][2]["read_only"]["actions"][0]["client"], "c-handoff-editor",
        "Canonical initial-only control stays unchanged"
    );
}
#[test]
fn first_normal_nfs_flow_runs_real_editors_actions_two_alternates_and_readback() {
    let (ctx, mut a) = opened();
    let mut witness = Witness::default();
    let next = checkpoints(&a.scenarios)
        .iter()
        .find(|(name, _)| name == "select-1")
        .unwrap()
        .1;
    until(&ctx, &mut a, next, &mut witness);
    let s = a.scenarios.iter().find(|s| s.id == IDS[0]).unwrap();
    assert!(progress::current_solved(&a.progress.attempts[IDS[0]], s));
    assert!(a.progress.favorites.contains(IDS[0]));
    assert!(witness.reads >= 3 && witness.focused >= 3);
    assert!(
        witness.shots.iter().any(|s| s.contains("wrong"))
            && witness.shots.iter().any(|s| s.contains("invalidated"))
    );
    assert!(a.selected.is_none());
    // Exercise the same receipt guard reached by real confirmed Reset events.
    // Mutate only the comparison snapshot, never live Progress or saved files.
    let progress_before = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    super::super::smoke::nfs_lessons::verify_reset_receipts(
        &a.progress.migration_receipts,
        &receipts,
    );
    let changed_receipts = if receipts.is_null() {
        serde_json::json!({"workspace": false, "shell": []})
    } else {
        serde_json::Value::Null
    };
    assert!(
        std::panic::catch_unwind(|| {
            super::super::smoke::nfs_lessons::verify_reset_receipts(
                &a.progress.migration_receipts,
                &changed_receipts,
            );
        })
        .is_err(),
        "Complete receipt guard rejects a changed comparison snapshot"
    );
    let visited = a.progress.attempts[IDS[0]].visited;
    super::super::smoke::nfs_lessons::verify_reset_recency(
        a.progress.clock,
        visited,
        &progress_before,
        IDS[0],
    );
    for field in ["clock", "visited"] {
        let mut changed = progress_before.clone();
        if field == "clock" {
            changed["clock"] = (a.progress.clock + 1).into();
        } else {
            changed["attempts"][IDS[0]]["visited"] = (visited + 1).into();
        }
        assert!(
            std::panic::catch_unwind(|| {
                super::super::smoke::nfs_lessons::verify_reset_recency(
                    a.progress.clock,
                    visited,
                    &changed,
                    IDS[0],
                );
            })
            .is_err(),
            "Reset recency guard rejects changed {field} comparison snapshot"
        );
    }
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), progress_before);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
}
#[test]
fn all_four_normal_nfs_flows_paint_measured_outputs_and_hold_delayed_finish_without_resetting() {
    let (ctx, mut a) = opened();
    let mut witness = Witness::default();
    let finish = phase_bounds(&a.scenarios).1;
    until(&ctx, &mut a, finish, &mut witness);
    assert_eq!(a.progress.attempts.len(), 4);
    assert!(witness.shots.len() <= 128);
    assert_eq!(witness.focused, 12);
    assert_eq!(witness.reads, 12);
    assert!(witness.fullscreen.contains(&true) && witness.fullscreen.contains(&false));
    for id in IDS {
        let s = a.scenarios.iter().find(|s| s.id == id).unwrap();
        assert!(progress::current_solved(&a.progress.attempts[id], s));
    }
    let before = serde_json::to_value(&a.progress).unwrap();
    let receipts = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    frame(&ctx, &mut a, true, &mut witness, true);
    assert_eq!(a.smoke_stage, finish);
    for _ in 0..3 {
        frame(&ctx, &mut a, false, &mut witness, true);
    }
    assert_eq!(a.smoke_stage, finish);
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert_eq!(
        serde_json::to_value(&a.progress.migration_receipts).unwrap(),
        receipts
    );
    assert!(witness.close >= 2);
}
#[test]
fn controller_has_no_model_profile_or_solved_result_installation_path() {
    let source = include_str!("../smoke/nfs_lessons.rs");
    for forbidden in [
        "perform_attempt(",
        "perform_practice(",
        "execute_for_goal(",
        "engine::execute(",
        ".progress =",
        ".state =",
        "expected_state",
        "std::process",
        "Command::new",
        "from_value::<Progress>",
    ] {
        assert!(
            !source.contains(forbidden),
            "Controller contains forbidden shortcut {forbidden}"
        );
    }
    assert!(
        source.contains("self.pending_events")
            && source.contains("self.nf_click")
            && source.contains("progress::load")
    );
    let scenarios = pack::bundled().unwrap();
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/nfs-review.json")).unwrap();
    for (s, review) in scenarios[150..154]
        .iter()
        .zip(raw["lessons"].as_array().unwrap())
    {
        assert_eq!(s.id, review["id"]);
        let actions = s
            .solution
            .iter()
            .map(|text| {
                let action = dlms_learning_lab::nfs::Action::from_json(
                    text.strip_prefix(dlms_learning_lab::nfs_workspace::PREFIX)
                        .unwrap(),
                )
                .unwrap();
                serde_json::to_value(action).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(actions, *review["reference"]["actions"].as_array().unwrap());
    }
}
