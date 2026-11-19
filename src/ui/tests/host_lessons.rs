use super::*;
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
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
const IDS: [&str; 4] = [
    "lpic1-host-address",
    "lpic1-host-routing",
    "lpic1-host-resolver",
    "lpic1-host-handover",
];
#[test]
fn host_all_native_assignment_flows_feedback_partial_alternates_and_invalidation() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.host.result.unwrap().1 > 0);
        for (cmd_i, cmd) in source.solution.iter().enumerate() {
            let e = serde_json::from_str(cmd.strip_prefix("host-action ").unwrap()).unwrap();
            action(&ctx, &mut a, &e);
            if cmd_i == 0 && source.solution.len() > 1 {
                tap(&ctx, &mut a, "check");
                assert!(a.metrics.host.result.unwrap().1 > 0);
            }
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.host.result.unwrap().1, 0, "{id} {:?}", a.notice);
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let got = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&got).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        if id.ends_with("resolver") || id.ends_with("handover") {
            field(&ctx, &mut a, "resolver", "nameserver 203.0.113.54\n");
            assert!(!progress::current_solved(&a.progress.attempts[id], &source));
            assert!(a.metrics.host.result.is_none());
            tap(&ctx, &mut a, "save-resolver");
        } else {
            action(
                &ctx,
                &mut a,
                &dlms_learning_lab::host_workspace::Event::Command {
                    input: "ip link set dev ens4 down".into(),
                },
            );
        }
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.host.result.unwrap().1 > 0);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
    }
}
#[test]
fn host_worked_insert_does_not_execute_separate_examples_pass_and_commands_get_focus() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let engine::Goal::HostNetwork { definition } = &source.goal else {
            panic!()
        };
        tap(&ctx, &mut a, "example");
        let before = a.progress.attempts[id].state.clone();
        tap(&ctx, &mut a, "fill-example");
        let State::HostNetwork { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let State::HostNetwork { simulation: old } = &before else {
            panic!()
        };
        assert_eq!(simulation.practice.world, old.practice.world);
        assert_eq!(simulation.practice.history, old.practice.history);
        assert_eq!(simulation.assignment, old.assignment);
        tap(&ctx, &mut a, "reset-example");
        for e in &definition.example_actions {
            action(&ctx, &mut a, e);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.host.result.unwrap().1, 0, "{id} {:?}", a.notice);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "assignment");
        let State::HostNetwork { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment, old.assignment);
    }
}
#[test]
fn host_fields_fit_at_supported_scales_and_widths_in_light_dark() {
    let (ctx, mut a, _) = opened("lpic1-host-handover");
    tap(&ctx, &mut a, "assignment");
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
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
            for width in [760., 1600.] {
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 950., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                for (name, r, _) in &a.metrics.host.fields {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "{name} scale{scale} width{width} {r:?} {clip:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn host_painted_learning_states_teach_before_assignment_and_hide_solutions() {
    fn collect(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                assert!(!t.galley.job.justify);
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            if let State::HostNetwork { simulation } =
                &mut a.progress.attempts.get_mut(id).unwrap().state
            {
                simulation.view = view;
            }
            let mut text = String::new();
            for _ in 0..4 {
                let o = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1500., 6000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                text.clear();
                for s in o.shapes {
                    collect(&s.shape, &mut text);
                }
            }
            let entry = lesson_presentation::entry(&source).unwrap();
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("WHAT YOU’LL LEARN"));
                    assert!(text.contains("Worked example"));
                    assert!(text.contains(&entry.worked[0].value));
                    assert!(!text.contains("1. 1."));
                }
                teaching::View::Example => {
                    assert!(text.contains(&entry.practice[0].title));
                    assert!(text.contains("Not checked"));
                    assert!(text.contains("Simulated administrator terminal"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("One valid approach"));
                    assert!(!text.contains("Worked example"));
                    for cmd in &source.solution {
                        assert!(!text.contains(cmd));
                    }
                }
            }
        }
    }
}
#[test]
fn host_command_enter_runs_in_simulator_and_unsupported_is_truthful() {
    let (ctx, mut a, i) = opened("lpic1-host-handover");
    tap(&ctx, &mut a, "assignment");
    field(&ctx, &mut a, "command", "ip link show");
    let source = a.scenarios[i].clone();
    frame(
        &ctx,
        &mut a,
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
    let State::HostNetwork { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    assert!(
        simulation
            .assignment
            .transcript
            .last()
            .unwrap()
            .contains("ens3")
    );
    let before = simulation.assignment.world.clone();
    field(&ctx, &mut a, "command", "sudo cat /etc/passwd");
    tap(&ctx, &mut a, "run");
    let State::HostNetwork { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.world, before);
    assert!(
        simulation
            .assignment
            .transcript
            .last()
            .unwrap()
            .contains("Not applied")
    );
}

const IPV6_IDS: [&str; 3] = [
    "lpic2-host-ipv6-addition",
    "lpic2-host-ipv6-gateway-scope",
    "lpic2-host-dual-stack-handover",
];
#[test]
fn ipv6_host_real_controls_keep_examples_ungraded_and_current_success_honest() {
    for id in IPV6_IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let engine::Goal::HostNetwork { definition } = &source.goal else {
            panic!()
        };
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "fill-example");
        let State::HostNetwork { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(simulation.practice.history.is_empty());
        assert_eq!(simulation.practice.world, definition.example);
        assert_eq!(
            simulation.assignment,
            dlms_learning_lab::host_workspace::Session::new(definition).assignment
        );
        for e in &definition.example_actions {
            action(&ctx, &mut a, e);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.host.result.unwrap().1, 0);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "assignment");
        let State::HostNetwork { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(simulation.assignment.command.is_empty());
        assert!(simulation.assignment.probe_ipv6_input.is_empty());
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.host.result.unwrap().1 > 0);
        for (n, c) in source.solution.iter().enumerate() {
            let e = serde_json::from_str(c.strip_prefix("host-action ").unwrap()).unwrap();
            action(&ctx, &mut a, &e);
            if n == 0 && source.solution.len() > 1 {
                tap(&ctx, &mut a, "check");
                let (total, failed, _) = a.metrics.host.result.unwrap();
                assert_eq!(total, definition.requirements.len() + 1);
                assert!(
                    failed > 0 && failed < total,
                    "Partial work retains correct outcomes"
                );
            }
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.host.result.unwrap().1, 0);
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        let long = "2001:db8:110:0:ffff:ffff:ffff:ffff";
        field(&ctx, &mut a, "probe", long);
        tap(&ctx, &mut a, "probe-run");
        let State::HostNetwork { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.assignment.probe_ipv6_input, long);
        assert!(
            simulation
                .assignment
                .probe_result
                .contains("No available supplied IPv6 destination")
        );
        assert!(
            progress::current_solved(&a.progress.attempts[id], &source),
            "Read-only exploration cannot change configuration"
        );
        action(
            &ctx,
            &mut a,
            &dlms_learning_lab::host_workspace::Event::Command {
                input: "ip -6 route del 2001:db8:140::/64 via fe80::1 dev ens4 metric 50".into(),
            },
        );
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        assert!(a.metrics.host.result.is_none());
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.host.result.unwrap().1, 1);
        action(
            &ctx,
            &mut a,
            &dlms_learning_lab::host_workspace::Event::Command {
                input: "ip -6 route add 2001:db8:140::/64 via fe80::1 dev ens4 metric 50".into(),
            },
        );
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.host.result.unwrap().1, 0);
        a.save();
        assert!(a.storage_error.is_none());
        let read = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(read).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
    }
}
#[test]
fn ipv6_host_fields_keep_full_text_and_alignment_at_all_supported_scales_and_widths() {
    for id in IPV6_IDS {
        let (ctx, mut a, _) = opened(id);
        tap(&ctx, &mut a, "assignment");
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
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
                for width in [760., 1600.] {
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 950., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for (name, r, _) in &a.metrics.host.fields {
                        assert!(
                            r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                            "{id} {name} scale{scale} width{width} {r:?} {clip:?}"
                        );
                    }
                }
            }
        }
        a.scale = 1.;
        ctx.set_zoom_factor(1.);
        field(&ctx, &mut a, "probe", "2001:db8:110:0:ffff:ffff:ffff:ffff");
        let State::HostNetwork { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(
            simulation.assignment.probe_ipv6_input,
            "2001:db8:110:0:ffff:ffff:ffff:ffff"
        );
        assert!(simulation.assignment.probe_input.is_empty());
    }
}
#[test]
fn ipv6_host_learning_views_are_left_aligned_and_independent_assignment_hides_repairs() {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                assert!(!t.galley.job.justify);
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out)
                }
            }
            _ => {}
        }
    }
    for id in IPV6_IDS {
        let (ctx, mut a, _) = opened(id);
        for (view, key) in [
            (teaching::View::Introduction, "introduction"),
            (teaching::View::Example, "example"),
            (teaching::View::Assignment, "assignment"),
        ] {
            tap(&ctx, &mut a, key);
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 7000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            let mut text = String::new();
            for shape in out.shapes {
                collect(&shape.shape, &mut text);
            }
            assert!(
                !text.contains("save resolver changes"),
                "IPv6 has no resolver editor"
            );
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("Worked example"));
                    assert!(text.contains("IPv6"));
                    assert!(text.contains("fe80::") || id == IPV6_IDS[0]);
                }
                teaching::View::Example => {
                    assert!(text.contains("Insert worked action"));
                    assert!(text.contains("Next worked action"));
                    assert!(text.contains("Run command"));
                    assert!(text.contains("Observe:"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(text.contains("Check network"));
                    assert!(!text.contains("host-action "));
                    assert!(!text.contains("ip -6 address add 2001:db8:110::41"));
                    assert!(!text.contains("ip -6 route replace default"));
                }
            }
        }
    }
}

#[test]
fn ipv6_host_preview_target_tracks_intro_practice_and_assignment_at_enlarged_text() {
    let (ctx, mut a, _) = opened(IPV6_IDS[0]);
    a.scale = 2.;
    ctx.set_zoom_factor(2.);
    for (tab, key) in [
        ("introduction", "try"),
        ("example", "check"),
        ("assignment", "check"),
    ] {
        tap(&ctx, &mut a, tab);
        let expected = a
            .metrics
            .host
            .controls
            .iter()
            .find(|c| c.0 == key)
            .unwrap()
            .1;
        assert_eq!(a.preview_action_rect(), Some(expected));
    }
}
