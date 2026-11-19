//! Current native host controls, new mixed-IP task and opt-in prerequisites.
use super::*;
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let current = LabApp::current_catalog_for_session(true, false, Some("mixed-ip-batch"));
    assert!(
        current,
        "The real mixed-IP capture must use the current catalog"
    );
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, current);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    let mut stable = None;
    for _ in 0..24 {
        // A preceding Enter can publish output after that frame's geometry.
        // Measure the newly painted field and wait for pointer arrival first.
        frame(ctx, a, 1200., 950., vec![]);
        let r = if field {
            a.metrics.host.fields.iter().find(|r| r.0 == key).unwrap().1
        } else {
            a.metrics
                .host
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if stable == Some((r, clip)) {
                return r;
            }
            stable = Some((r, clip));
            frame(
                ctx,
                a,
                1200.,
                950.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            continue;
        }
        stable = None;
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
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
    panic!("Host target never visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key, true);
    click(ctx, a, r);
    let id = a.metrics.host.fields.iter().find(|r| r.0 == key).unwrap().2;
    assert_eq!(
        ctx.memory(|m| m.focused()),
        Some(id),
        "Actual host field {key} at {r:?}"
    );
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
                modifiers: egui::Modifiers::CTRL,
            },
            egui::Event::Text(text.into()),
        ],
    );
}

fn action(ctx: &egui::Context, a: &mut LabApp, e: &dlms_learning_lab::host_workspace::Event) {
    use dlms_learning_lab::host_workspace::Event;
    match e {
        Event::Command { input } => {
            field(ctx, a, "command", input);
            tap(ctx, a, "run");
        }
        Event::Resolver { text } => {
            field(ctx, a, "resolver", text);
            tap(ctx, a, "save-resolver");
        }
    }
}

#[test]
fn mixed_ip_teaches_before_testing_and_retains_the_full_terminal_assignment() {
    let (ctx, mut a, i) = opened("lpic1-host-mixed-ip");
    let s = a.scenarios[i].clone();
    let entry = lesson_presentation::entry(&s).unwrap();
    assert_eq!(entry.concepts.len(), 4);
    assert_eq!(entry.worked.len(), 7);
    assert_eq!(entry.practice.len(), 3);
    assert_eq!(entry.requirements.join(" "), s.objective);
    assert!(!a.metrics.host.controls.iter().any(|r| r.0 == "command"));
    tap(&ctx, &mut a, "example");
    assert!(
        a.metrics
            .host
            .controls
            .iter()
            .any(|r| r.0 == "fill-example")
    );
    let engine::Goal::HostNetwork { definition } = &s.goal else {
        panic!()
    };
    let before = serde_json::to_value(&a.progress.attempts[&s.id].state).unwrap();
    tap(&ctx, &mut a, "fill-example");
    let engine::State::HostNetwork { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    assert_eq!(simulation.practice.world, definition.example);
    assert_eq!(simulation.assignment.world, definition.initial);
    assert_ne!(
        serde_json::to_value(&a.progress.attempts[&s.id].state).unwrap(),
        before
    );
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    for e in &definition.example_actions {
        action(&ctx, &mut a, e);
    }
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.host.result.unwrap().1, 0);
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "assignment");
    assert!(a.metrics.host.controls.iter().any(|r| r.0 == "command"));
    assert!(
        !a.metrics
            .host
            .controls
            .iter()
            .any(|r| r.0 == "fill-example")
    );
}
#[test]
fn mixed_ip_real_keyboard_partial_success_invalidation_and_saved_work() {
    let (ctx, mut a, i) = opened("lpic1-host-mixed-ip");
    let s = a.scenarios[i].clone();
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.host.result.unwrap().1 > 0);
    for (index, source) in s.solution.iter().enumerate() {
        let e = serde_json::from_str(source.strip_prefix("host-action ").unwrap()).unwrap();
        action(&ctx, &mut a, &e);
        if index == 2 {
            tap(&ctx, &mut a, "check");
            let result = a.metrics.host.result.unwrap();
            assert!(result.1 > 0 && result.1 < result.0);
            assert!(!a.metrics.completion);
        }
    }
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    action(
        &ctx,
        &mut a,
        &dlms_learning_lab::host_workspace::Event::Command {
            input: "ip -6 route replace default via fe80::1 dev ens4 metric 100".into(),
        },
    );
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.host.result.unwrap().1 > 0);
    a.save();
    assert!(a.storage_error.is_none());
    assert_eq!(
        serde_json::to_value(progress::load(&a.path, &a.scenarios).unwrap()).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
}
#[test]
fn foundation_prerequisites_are_explicitly_opted_in_without_altering_any_other_lesson() {
    let raw: Vec<lesson_presentation::Entry> =
        serde_json::from_str(include_str!("../../../scenarios/presentation/lpic1.json")).unwrap();
    for entry in raw {
        let got = lesson_presentation::entries().get(&entry.id).unwrap();
        let opted = matches!(
            entry.id.as_str(),
            "lpic1-host-address" | "lpic1-host-routing"
        );
        assert_eq!(
            got.concepts.len(),
            entry.concepts.len() + usize::from(opted)
        );
        assert_eq!(got.worked.len(), entry.worked.len() + usize::from(opted));
        assert_eq!(got.requirements, entry.requirements);
        assert_eq!(got.practice.len(), entry.practice.len());
        if opted {
            assert!(!got.prerequisite_sources.is_empty());
        } else {
            assert!(got.prerequisite_sources.is_empty());
        }
    }
    for id in ["lpic1-host-address", "lpic1-host-routing"] {
        let (ctx, mut a, i) = opened(id);
        let e = lesson_presentation::entry(&a.scenarios[i]).unwrap();
        assert_eq!(e.concepts.len(), 4);
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for light in [false, true] {
                let id = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &id);
                frame(&ctx, &mut a, 900., 900., vec![]);
                assert!(!a.metrics.completion);
                assert_eq!(a.metrics.lesson_tabs.len(), 3);
            }
        }
    }
}

#[test]
fn mixed_ip_workspace_guidance_follows_both_requirements_not_just_ipv6_presence() {
    let (_, a, _) = opened("lpic1-host-mixed-ip");
    for s in &a.scenarios {
        if let engine::Goal::HostNetwork { definition } = &s.goal {
            for practice in [false, true] {
                assert_eq!(
                    host_page::repairs_both_families(definition, practice),
                    s.id == "lpic1-host-mixed-ip",
                    "{} practice={practice}",
                    s.id
                );
            }
        }
    }
}
