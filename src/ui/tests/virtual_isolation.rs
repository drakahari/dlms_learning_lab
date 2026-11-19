//! Actual synthetic widgets; no novice/physical-input/native-platform acceptance.
use super::debug_recovery::{paint, settle, tap};
use super::*;
use dlms_learning_lab::{virtual_isolation_source as binding, virtual_isolation_workspace as work};
#[path = "../../../tests/support/virtual_isolation_tasks.rs"]
mod setup;
fn opened(routing: bool) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let mut s = a
        .scenarios
        .iter()
        .find(|s| s.id == "ccna-leaf-spine-rack-expansion")
        .unwrap()
        .clone();
    let d = setup::definition(routing);
    s.id = d.id.clone();
    s.title = if routing {
        "Repair separate routing contexts"
    } else {
        "Place resilient virtual workloads"
    }
    .into();
    s.goal = engine::Goal::VirtualIsolation {
        definition: Box::new(d.clone()),
    };
    s.initial = State::VirtualIsolation {
        simulation: Box::new(d.new_session().unwrap()),
    };
    let i = a.scenarios.len();
    a.scenarios.push(s);
    a.launch(i);
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::VirtualIsolation { simulation } =
        &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id].state
    else {
        panic!()
    };
    simulation
}
fn click(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    for _ in 0..20 {
        settle(ctx, a);
        let (_, r, clip) = a
            .metrics
            .virtual_isolation
            .controls
            .iter()
            .find(|r| r.0 == key)
            .unwrap_or_else(|| panic!("Missing {key}"))
            .clone();
        let clip = clip.intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            tap(ctx, a, r);
            return;
        }
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
    }
    panic!("Cannot reveal {key}")
}
#[test]
fn virtual_isolation_native_check_example_isolation_and_configuration_invalidation() {
    for routing in [false, true] {
        let (ctx, mut a, i) = opened(routing);
        let s = a.scenarios[i].clone();
        let d = binding::source(&s).unwrap();
        let original = session(&a).assignment.clone();
        click(&ctx, &mut a, "example");
        click(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        assert_eq!(session(&a).assignment, original);
        assert!(
            a.metrics
                .virtual_isolation
                .controls
                .iter()
                .any(|r| r.0 == "failed-case")
        );
        click(&ctx, &mut a, "assignment");
        let State::VirtualIsolation { simulation } =
            &mut a.progress.attempts.get_mut(&s.id).unwrap().state
        else {
            panic!()
        };
        simulation
            .assignment
            .edit(&d.assignment, d.assignment.reference())
            .unwrap();
        simulation.assignment.check(&d.assignment).unwrap();
        a.completion_index.forget(&s.id);
        settle(&ctx, &mut a);
        assert!(a.metrics.completion);
        click(
            &ctx,
            &mut a,
            if routing { "router-up" } else { "up:db-one" },
        );
        assert!(!a.metrics.completion && !session(&a).assignment.checked);
        assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    }
}
#[test]
fn virtual_isolation_controls_reflow_without_shrinking_fonts_or_cards() {
    let (ctx, mut a, _) = opened(false);
    click(&ctx, &mut a, "assignment");
    for (width, scale, theme) in [
        (1600., 1., "lab-navy"),
        (820., 1.5, "dlms-light"),
        (820., 2., "lab-navy"),
    ] {
        a.scale = scale;
        a.theme_id = theme.into();
        theme::apply(&ctx, &a.theme_id);
        ctx.set_zoom_factor(scale);
        for _ in 0..3 {
            frame(&ctx, &mut a, width, 1000., vec![]);
        }
        assert_eq!(a.scale, scale);
        assert_eq!(a.metrics.virtual_isolation.cards.len(), 3);
        let first = a.metrics.virtual_isolation.cards[0];
        for r in &a.metrics.virtual_isolation.cards {
            assert!((r.width() - first.width()).abs() < 1.);
            assert!((r.height() - first.height()).abs() < 1.);
        }
        for (_, r, clip) in &a.metrics.virtual_isolation.controls {
            assert!(r.width() > 0. && r.height() > 0.);
            assert!(r.left() >= clip.left() - 1.);
            assert!(r.right() <= clip.right() + 1., "{r:?} {clip:?}");
        }
    }
}
#[test]
fn virtual_isolation_dropdown_escape_and_tab_keep_keyboard_focus_visible() {
    for routing in [false, true] {
        let (ctx, mut a, _) = opened(routing);
        click(&ctx, &mut a, "assignment");
        let key = if routing {
            "context:a-client"
        } else {
            "placement:db-one"
        };
        click(&ctx, &mut a, key);
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        settle(&ctx, &mut a);
        let id = a
            .metrics
            .virtual_isolation
            .focus
            .iter()
            .find(|(k, _)| k == key)
            .unwrap()
            .1;
        assert!(!egui::ComboBox::is_open(&ctx, id));
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        settle(&ctx, &mut a);
        assert!(ctx.memory(|m| m.focused()).is_some());
        assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    }
}
