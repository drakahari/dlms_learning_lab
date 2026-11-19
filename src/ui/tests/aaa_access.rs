use super::debian_contents::painted;
use super::*;
use dlms_learning_lab::aaa_workspace as work;
const IDS: [&str; 2] = [
    "aaa-methods-synthetic-local-records",
    "aaa-authorization-synthetic-local-records",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let d = super::super::aaa_fixture::definition(id == IDS[1]);
    let mut source = a
        .scenarios
        .iter()
        .find(|s| s.id == "ccna-management-protocols")
        .unwrap()
        .clone();
    source.id = d.initial.id.clone();
    source.objective = d.initial.instructions.clone();
    source.prerequisites.clear();
    source.solution = vec![format!("{}{}", work::PREFIX, d.initial.reference)];
    source.initial = State::AaaAccess {
        simulation: Box::new(work::Session::new(&d).unwrap()),
    };
    source.goal = engine::Goal::AaaAccess {
        definition: Box::new(d),
    };
    a.scenarios.push(source);
    let i = a.scenarios.len() - 1;
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for attempt in 0..30 {
        frame(ctx, a, 1200., 950., vec![]);
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let (r, clip) = if key == "editor" {
            let (r, _, _) = a.metrics.aaa_access.editor.unwrap();
            // Reveal the full outer viewport, including its legitimate bars.
            // The click itself is checked against the actual editable clip below.
            (r, main)
        } else if key == "challenge" {
            (a.metrics.mode_challenge.unwrap(), main)
        } else {
            let (_, r, c) = a
                .metrics
                .aaa_access
                .controls
                .iter()
                .find(|v| v.0 == key)
                .unwrap();
            (*r, *c)
        };
        let popup_item = key.starts_with("case-") || key.starts_with("line-");
        // Popup rows use their own scroll area and may extend outside the main
        // lesson viewport. Require the full item inside its actual popup clip.
        let clip = if popup_item {
            ctx.content_rect().intersect(clip)
        } else {
            main.intersect(clip)
        };
        if popup_item {
            let wheel = egui::pos2(
                r.center().x.clamp(clip.left(), clip.right()),
                clip.center().y,
            );
            assert_eq!(
                ctx.layer_id_at(wheel).map(|l| l.order),
                Some(egui::Order::Foreground),
                "Popup wheel must land on the actual menu layer"
            );
        }
        if popup_item && matches!(attempt, 0 | 2 | 29) {
            eprintln!(
                "AAA_POPUP_GEOMETRY key={key} attempt={attempt} rect={r:?} clip={clip:?} main={main:?} center_layer={:?}",
                ctx.layer_id_at(clip.center())
            );
        }
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if key == "editor" {
                let (_, _, text_clip) = a.metrics.aaa_access.editor.unwrap();
                assert!(
                    text_clip
                        .intersect(a.metrics.aaa_access.editor_frame.unwrap())
                        .contains(r.center()),
                    "Editor click must land in actual editable content, not its scrollbar"
                );
            }
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(if popup_item {
                    // The popup's vertical clip can span the window horizontally.
                    // Keep x over the actual row, rather than over the background.
                    egui::pos2(
                        r.center().x.clamp(clip.left(), clip.right()),
                        clip.center().y,
                    )
                } else {
                    egui::pos2(main.left() + 1., main.center().y)
                }),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!(
        "AAA target not visible: {key}; editor={:?}; main={:?}; scroll={:?}",
        a.metrics.aaa_access.editor, a.metrics.main_viewport, a.metrics.main_scroll_offset
    )
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "editor");
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    let id = a.metrics.aaa_access.editor.unwrap().1;
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
    let s = &a.scenarios[a.selected.unwrap()];
    let State::AaaAccess { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    let scope = if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    };
    assert_eq!(
        scope.draft, text,
        "Actual Ctrl+A/Paste must replace the document"
    );
}
fn state(a: &LabApp) -> &work::Session {
    let id = &a.scenarios[a.selected.unwrap()].id;
    let State::AaaAccess { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
#[test]
fn aaa_access_editor_checks_specific_partial_cases_syntax_unsupported_and_stale_retry() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = work::source(&a.scenarios[i]).unwrap().clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        let (p, t, _) = a
            .metrics
            .aaa_access
            .result
            .expect("valid starter has partial case results");
        assert_eq!((p, t), if id == IDS[0] { (9, 15) } else { (13, 15) });
        assert!(!a.metrics.completion);
        edit(&ctx, &mut a, "username Admin privilege 15 secret");
        tap(&ctx, &mut a, "apply");
        assert!(a.notice.as_deref().unwrap().contains("Syntax error"));
        assert!(a.metrics.aaa_access.result.is_none());
        edit(&ctx, &mut a, "aaa authentication login default none");
        tap(&ctx, &mut a, "check");
        assert!(a.notice.as_deref().unwrap().contains("Unsupported"));
        edit(&ctx, &mut a, &d.initial.reference);
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        assert!(a.metrics.completion);
        edit(&ctx, &mut a, &d.initial.starter);
        assert!(!a.metrics.completion);
        assert!(a.metrics.aaa_access.result.is_none());
        tap(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
    }
}
#[test]
fn aaa_access_trial_requires_apply_and_selector_changes_clear_actual_output() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let d = work::source(&a.scenarios[i]).unwrap().clone();
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, &d.initial.reference);
    tap(&ctx, &mut a, "inspect");
    assert!(
        state(&a)
            .assignment
            .observation
            .as_deref()
            .unwrap()
            .starts_with("Trial not run: Apply")
    );
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "inspect");
    assert!(
        state(&a)
            .assignment
            .observation
            .as_deref()
            .unwrap()
            .contains("IPv4 forward/reply path")
    );
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "case");
    tap(&ctx, &mut a, "case-3");
    assert!(state(&a).assignment.observation.is_none());
    tap(&ctx, &mut a, "inspect");
    assert!(
        state(&a)
            .assignment
            .observation
            .as_deref()
            .unwrap()
            .contains("does not match")
    );
    assert!(painted(&ctx, &mut a).contains("Required outcome: credential rejection returned by training service Primary; no later method attempted"));
    // The last long popup row requires scrolling the popup, not the lesson.
    tap(&ctx, &mut a, "case");
    tap(&ctx, &mut a, "case-7");
    assert_eq!(state(&a).assignment.selected_case, 7);
    assert!(state(&a).assignment.observation.is_none());
    tap(&ctx, &mut a, "inspect");
    let actual = state(&a).assignment.observation.as_deref().unwrap();
    assert!(actual.contains("unavailable; try the next method"));
    assert!(actual.contains("Local EXEC authorization: Admin receives level 15"));
}
#[test]
fn aaa_access_practice_is_separate_and_challenge_does_not_render_worked_answers() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let original = state(&a).assignment.clone();
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "worked-example");
        tap(&ctx, &mut a, "fill-example");
        assert!(!state(&a).practice.checked);
        tap(&ctx, &mut a, "check");
        assert_eq!(state(&a).assignment, original);
        assert!(!progress::current_solved(
            &a.progress.attempts[id],
            &a.scenarios[i]
        ));
        let practice = state(&a).practice.clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "challenge");
        assert_eq!(state(&a).practice, practice);
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Worked local-account"));
        assert!(!text.contains("Sample account records"));
        assert!(!text.contains("Fill worked example"));
        assert!(!text.contains("Sample-"));
    }
}
#[test]
fn aaa_access_controls_reflow_and_editor_focus_stays_stable_at_all_supported_scales() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    target(&ctx, &mut a, "editor");
    let id = a.metrics.aaa_access.editor.unwrap().1;
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        for width in [760., 1600.] {
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
                theme::apply(&ctx, &a.theme_id);
                for disabled in [false, true] {
                    a.reset_pending = disabled;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 1100., vec![]);
                    }
                    let main = a.metrics.main_viewport.unwrap();
                    let (editor, current, _) = a.metrics.aaa_access.editor.unwrap();
                    assert_eq!(current, id);
                    if scale == 1. && width == 1600. {
                        assert!(
                            a.metrics.aaa_access.side_by_side,
                            "Wide workspaces must use the editor/trial split"
                        );
                    }
                    if width == 760. {
                        assert!(
                            !a.metrics.aaa_access.side_by_side,
                            "Narrow workspaces stack without shrinking text"
                        );
                    }
                    assert!(editor.width() > 100.);
                    assert!(editor.right() <= main.right() + 1.);
                    let frame_rect = a.metrics.aaa_access.editor_frame.unwrap();
                    assert!(
                        frame_rect.left() >= editor.left() - 1.
                            && frame_rect.right() <= editor.right() + 1.,
                        "Native editor frame must fit its allocated width: {frame_rect:?} {editor:?}"
                    );
                    for key in ["apply", "check", "case", "line", "inspect"] {
                        let (_, r, _) = a
                            .metrics
                            .aaa_access
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap();
                        assert!(
                            r.left() >= main.left() - 1. && r.right() <= main.right() + 1.,
                            "{key} {scale}/{width}/{light}: {r:?} {main:?}"
                        );
                    }
                    let out = a.metrics.aaa_access.output.unwrap();
                    assert!(out.right() <= main.right() + 1.);
                }
            }
        }
    }
    a.reset_pending = false;
    a.scale = 1.;
    ctx.set_zoom_factor(1.);
    frame(&ctx, &mut a, 1200., 950., vec![]);
    let draft = state(&a).assignment.draft.clone();
    edit(&ctx, &mut a, &draft);
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
}
