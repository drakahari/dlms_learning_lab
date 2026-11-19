//! Native widgets with private synthetic progress; not a novice/device study.
use super::*;
use dlms_learning_lab::{linux_router_source as binding, linux_router_workspace as work};
const ID: &str = "lpic2-router-saved-policy-recovery";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 9000., events);
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::LinuxRouter { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::LinuxRouter { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .router
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            r,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}: {r:?} {clip:?}"
    );
    let p = r.center();
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    hit(ctx, a, key);
    let id = a
        .metrics
        .router
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .2;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    paint(
        ctx,
        a,
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
            egui::Event::Text(text.into()),
        ],
    );
    settle(ctx, a);
}
#[test]
fn router_insert_is_inert_enter_runs_only_the_practice_and_challenge_hides_guidance() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let before = session(&a).assignment.clone();
    hit(&ctx, &mut a, "insert-command");
    assert!(session(&a).practice.output.is_empty());
    assert!(!session(&a).practice.input.is_empty());
    assert!(!session(&a).practice.checked);
    edit(&ctx, &mut a, "command", "iptables -S");
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    assert!(session(&a).practice.output.contains("-P INPUT DROP"));
    assert_eq!(session(&a).assignment, before);
    assert!(!a.metrics.completion);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["example", "introduction", "insert-command", "worked"] {
        assert!(
            !a.metrics.router.controls.iter().any(|r| r.0 == key),
            "Unsolicited guidance {key}"
        );
    }
    assert!(
        a.metrics
            .router
            .controls
            .iter()
            .any(|r| r.0 == "assignment")
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn router_real_controls_repair_saved_policy_and_editing_clears_current_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    for raw in &source.solution {
        let action: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        match action {
            work::Action::Command { line } => {
                edit(&ctx, &mut a, "command", &line);
                hit(&ctx, &mut a, "run");
            }
            work::Action::File { file } => {
                hit(&ctx, &mut a, "file");
                hit(&ctx, &mut a, &format!("file:{}", file.path()));
            }
            work::Action::Edit { text } => edit(&ctx, &mut a, "editor", &text),
            work::Action::Save => hit(&ctx, &mut a, "save"),
            work::Action::Check => hit(&ctx, &mut a, "check"),
            _ => panic!("Unexpected recipe action"),
        }
    }
    assert!(a.metrics.completion);
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    edit(&ctx, &mut a, "editor", "net.ipv4.ip_forward=0\n");
    assert!(!a.metrics.completion);
    assert!(!session(&a).assignment.checked);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .router
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
    assert!(session(&a).assignment.state.forwarding);
}
#[test]
fn router_editor_scrollbars_leave_save_aligned_at_supported_scales_and_widths() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    for width in [1450., 760.] {
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                ctx.set_zoom_factor(scale);
                a.scale = scale;
                let selected = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &selected);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 9000., vec![]);
                }
                let save = a
                    .metrics
                    .router
                    .controls
                    .iter()
                    .find(|r| r.0 == "save")
                    .unwrap()
                    .1;
                let viewport = a
                    .metrics
                    .router
                    .controls
                    .iter()
                    .find(|r| r.0 == "editor-viewport")
                    .unwrap()
                    .1;
                let State::LinuxRouter { simulation } =
                    &mut a.progress.attempts.get_mut(ID).unwrap().state
                else {
                    panic!()
                };
                simulation.assignment.filter_draft =
                    format!("# {}\n", "long-visible-configuration-value ".repeat(30)).repeat(20);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 9000., vec![]);
                }
                let after = a
                    .metrics
                    .router
                    .controls
                    .iter()
                    .find(|r| r.0 == "save")
                    .unwrap()
                    .1;
                let after_view = a
                    .metrics
                    .router
                    .controls
                    .iter()
                    .find(|r| r.0 == "editor-viewport")
                    .unwrap()
                    .1;
                assert!(
                    (after.top() - save.top()).abs() < 1.,
                    "Save shifted at {width}/{scale}/{light}"
                );
                assert_eq!(viewport.size(), after_view.size());
                assert!(after.bottom() <= ctx.content_rect().bottom());
                assert!(after.right() <= ctx.content_rect().right() + 1.);
                let State::LinuxRouter { simulation } =
                    &mut a.progress.attempts.get_mut(ID).unwrap().state
                else {
                    panic!()
                };
                simulation.assignment.filter_draft =
                    simulation.assignment.state.filter_file.clone();
            }
        }
    }
}

#[test]
fn router_editor_fits_the_actual_short_viewport_without_reducing_font_size() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    ctx.set_zoom_factor(2.);
    a.scale = 2.;
    for _ in 0..4 {
        frame(&ctx, &mut a, 1268., 632., vec![]);
    }
    let viewport = a
        .metrics
        .router
        .controls
        .iter()
        .find(|r| r.0 == "editor-viewport")
        .unwrap()
        .1;
    let parent = a.metrics.main_viewport.unwrap();
    assert!(
        viewport.height() <= parent.height(),
        "editor {viewport:?} exceeds actual visible area {parent:?}"
    );
    let line = ctx.style().text_styles[&egui::TextStyle::Monospace].size;
    assert!(
        viewport.height() >= 4. * line,
        "Keep at least four readable rows; use internal scrolling."
    );
    assert_eq!(ctx.zoom_factor(), 2.);
    let save = a
        .metrics
        .router
        .controls
        .iter()
        .find(|r| r.0 == "save")
        .unwrap()
        .1;
    assert!(save.top() > viewport.bottom());
}

#[test]
fn router_tab_and_shift_tab_reach_actions_without_running_a_command() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "command");
    let before = session(&a).assignment.clone();
    for (key, shift) in [
        ("run", false),
        ("case", false),
        ("trace", false),
        ("check", false),
        ("trace", true),
    ] {
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    shift,
                    ..Default::default()
                },
            }],
        );
        settle(&ctx, &mut a);
        let expected = a
            .metrics
            .router
            .focus
            .iter()
            .find(|r| r.0 == key)
            .unwrap()
            .1;
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(expected),
            "Native focus should reach {key}"
        );
        assert_eq!(
            session(&a).assignment,
            before,
            "Tab navigation must not execute or grade work."
        );
    }
}
