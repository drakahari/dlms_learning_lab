use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::fs_integrity_workspace as work;
const IDS: [&str; 2] = ["lpic1-fs-offline-accounting", "lpic1-fs-retained-recovery"];
pub(super) fn opened(id: &str) -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a)
}
pub(super) fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        frame(ctx, a, 1200., 950., vec![]);
        let rect = if key == "reset-header" {
            a.metrics.reset.unwrap()
        } else if key == "confirm-reset" {
            a.metrics.confirm_reset.unwrap()
        } else {
            a.metrics
                .fs_integrity
                .controls
                .iter()
                .find(|c| c.0 == key)
                .unwrap_or_else(|| panic!("Missing {key}"))
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(rect, clip, ctx.pixels_per_point()) {
            return rect;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - rect.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Could not reveal {key}")
}
pub(super) fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let rect = target(ctx, a, key);
    click(ctx, a, rect);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
pub(super) fn field(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let rect = target(ctx, a, "command");
    click(ctx, a, rect);
    let id = a.metrics.fs_integrity.command.unwrap().1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    frame(
        ctx,
        a,
        1200.,
        950.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    command: true,
                    ..egui::Modifiers::CTRL
                },
            },
            egui::Event::Paste(text.into()),
        ],
    );
    assert_eq!(scope(a).command, text);
}
pub(super) fn scope(a: &LabApp) -> &work::Scope {
    let s = &a.scenarios[a.selected.unwrap()];
    let State::FsIntegrity { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    if simulation.view == teaching::View::Example {
        &simulation.example
    } else {
        &simulation.assignment
    }
}
pub(super) fn command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    field(ctx, a, text);
    frame(
        ctx,
        a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    frame(
        ctx,
        a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_eq!(scope(a).transcript.last().unwrap().input, text);
}
pub(super) fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..work::MAX_TICKS + 5 {
        frame(ctx, a, 1200., 950., vec![]);
        if a.fs_integrity_run.is_none() {
            return;
        }
    }
    panic!("Cooperative check did not settle")
}
#[test]
fn integrity_three_screens_teach_actual_tools_and_keep_insertion_and_practice_separate() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        let s = a.scenarios[a.selected.unwrap()].clone();
        let text = painted(&ctx, &mut a);
        assert!(
            text.contains("inode")
                && text.contains("Worked example")
                && text.contains("e2fsck -fn")
        );
        tap(&ctx, &mut a, "example");
        let before = serde_json::to_value(&a.progress.attempts[id]).unwrap();
        tap(&ctx, &mut a, "insert");
        assert_eq!(scope(&a).command, "findmnt");
        assert!(scope(&a).history().is_empty() && scope(&a).transcript.is_empty());
        let after = serde_json::to_value(&a.progress.attempts[id]).unwrap();
        assert_eq!(
            before["state"]["simulation"]["assignment"],
            after["state"]["simulation"]["assignment"]
        );
        let engine::Goal::FsIntegrity { definition } = &s.goal else {
            panic!()
        };
        for c in &definition.example_commands {
            command(&ctx, &mut a, c);
        }
        tap(&ctx, &mut a, "check");
        settle(&ctx, &mut a);
        assert_eq!(
            a.metrics.fs_integrity.result.map(|r| (r.0, r.1)),
            Some((3, 3))
        );
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(
            text.contains("Requirements")
                && !text.contains("Insert command")
                && !text.contains("/dev/vdd1")
        );
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Worked example") && !text.contains("Insert command"));
    }
}
#[test]
fn integrity_keyboard_enter_diagnostics_alternatives_damage_and_retry_use_real_controls() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        let s = a.scenarios[a.selected.unwrap()].clone();
        tap(&ctx, &mut a, "assignment");
        command(&ctx, &mut a, "sh -c forbidden");
        assert_eq!(
            scope(&a).transcript.last().unwrap().diagnostic,
            Some(dlms_learning_lab::fs_integrity_terminal::DiagnosticKind::Unsupported)
        );
        command(&ctx, &mut a, "e2fsck -fy /dev/vdb1");
        assert_eq!(scope(&a).transcript.last().unwrap().status, 8);
        tap(&ctx, &mut a, "check");
        settle(&ctx, &mut a);
        assert_eq!(
            a.metrics.fs_integrity.result.map(|r| (r.0, r.1)),
            Some((0, 3))
        );
        for raw in &s.solution {
            if let work::Action::Command { input } =
                serde_json::from_str(raw.strip_prefix(work::PREFIX).unwrap()).unwrap()
            {
                command(&ctx, &mut a, &input);
            }
        }
        assert!(
            scope(&a)
                .transcript
                .iter()
                .any(|m| m.status == 1 && m.diagnostic.is_none())
        );
        tap(&ctx, &mut a, "check");
        settle(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        assert!(painted(&ctx, &mut a).contains("Assignment complete"));
        command(&ctx, &mut a, "findmnt");
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        command(&ctx, &mut a, "umount /mnt/data");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "check");
        settle(&ctx, &mut a);
        assert!(painted(&ctx, &mut a).contains("Needs work"));
        command(&ctx, &mut a, "mount /dev/vdb1 /mnt/data");
        tap(&ctx, &mut a, "check");
        settle(&ctx, &mut a);
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn integrity_stop_reset_reentry_and_loaded_verification_do_not_publish_stale_results() {
    let id = IDS[0];
    let (ctx, mut a) = opened(id);
    let s = a.scenarios[a.selected.unwrap()].clone();
    tap(&ctx, &mut a, "assignment");
    for raw in &s.solution {
        if let work::Action::Command { input } =
            serde_json::from_str(raw.strip_prefix(work::PREFIX).unwrap()).unwrap()
        {
            command(&ctx, &mut a, &input);
        }
    }
    tap(&ctx, &mut a, "check");
    tap(&ctx, &mut a, "stop");
    assert!(a.fs_integrity_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    tap(&ctx, &mut a, "check");
    settle(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[id], &s));
    field(&ctx, &mut a, "unfinished draft");
    a.save();
    assert!(a.storage_error.is_none());
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    settle(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[id], &s));
    assert_eq!(scope(&a).command, "unfinished draft");
    tap(&ctx, &mut a, "reset-header");
    tap(&ctx, &mut a, "confirm-reset");
    assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    assert!(scope(&a).history().is_empty());
    assert!(scope(&a).command.is_empty());
}
#[test]
fn integrity_shared_singleline_controls_reflow_with_text_scaling_and_themes() {
    for id in IDS {
        let (ctx, mut a) = opened(id);
        tap(&ctx, &mut a, "assignment");
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.theme_id = if light {
                    theme::inventory()
                        .iter()
                        .find(|t| t.light)
                        .unwrap()
                        .id
                        .clone()
                } else {
                    theme::Preferences::default().theme
                };
                for (w, h) in [(1400., 950.), (900., 750.)] {
                    for _ in 0..3 {
                        frame(&ctx, &mut a, w, h, vec![]);
                    }
                    let metrics = &a.metrics.fs_integrity;
                    for (key, rect, _) in &metrics.controls {
                        assert!(
                            rect.is_finite() && rect.width() > 0. && rect.height() > 0.,
                            "{key}"
                        );
                        assert!(
                            rect.right() <= ctx.content_rect().right() + 1.,
                            "{key} {rect:?}"
                        );
                    }
                    let rect = metrics.command.unwrap().0;
                    assert!(rect.height() >= 22.);
                    assert!(metrics.output.unwrap().height() > rect.height() * 3.);
                }
            }
        }
    }
}
