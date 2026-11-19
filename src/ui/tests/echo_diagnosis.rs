//! Current native host controls, new Echo diagnosis task and opt-in prerequisites.
use super::*;
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let current = LabApp::current_catalog_for_session(true, false, Some("echo-diagnosis-batch"));
    assert!(
        current,
        "The real Echo diagnosis capture must use the current catalog"
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
fn echo_diagnosis_teaches_then_uses_real_terminal_focus_and_state_based_checks() {
    let (ctx, mut a, i) = opened("lpic1-echo-path-diagnosis");
    let source = a.scenarios[i].clone();
    let engine::Goal::HostNetwork { definition } = source.goal.clone() else {
        panic!()
    };
    let entry = lesson_presentation::entry(&source).unwrap();
    assert_eq!(
        (
            entry.concepts.len(),
            entry.worked.len(),
            entry.practice.len()
        ),
        (4, 4, 3)
    );
    assert!(!a.metrics.host.fields.iter().any(|r| r.0 == "command"));
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "fill-example");
    let engine::State::HostNetwork { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    assert_eq!(simulation.practice.world, definition.example);
    assert_eq!(simulation.assignment.world, definition.initial);
    assert!(!simulation.practice.command.is_empty());
    for event in &definition.example_actions {
        action(&ctx, &mut a, event);
    }
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.host.result.unwrap().1, 0);
    assert!(!progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
    tap(&ctx, &mut a, "assignment");
    assert!(
        !a.metrics
            .host
            .controls
            .iter()
            .any(|r| matches!(r.0.as_str(), "probe-run" | "save-resolver" | "fill-example"))
    );
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.host.result.unwrap().1 > 0);
    for cmd in &source.solution {
        let event = serde_json::from_str(cmd.strip_prefix("host-action ").unwrap()).unwrap();
        action(&ctx, &mut a, &event);
    }
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.host.result.unwrap().1, 0);
    assert!(progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
    action(
        &ctx,
        &mut a,
        &dlms_learning_lab::host_workspace::Event::Command {
            input: "ping -c 1 172.22.40.99".into(),
        },
    );
    assert_eq!(
        a.metrics.host.result.unwrap().1,
        0,
        "A read-only query must keep current feedback visible"
    );
    assert!(progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
    action(
        &ctx,
        &mut a,
        &dlms_learning_lab::host_workspace::Event::Command {
            input: "ip link set dev ens3 down".into(),
        },
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
    tap(&ctx, &mut a, "check");
    let (total, failed, _) = a.metrics.host.result.unwrap();
    assert!(failed > 0 && failed < total);
}

#[test]
fn echo_diagnosis_reflows_all_three_views_at_supported_scales_and_both_themes() {
    let (ctx, mut a, i) = opened("lpic1-echo-path-diagnosis");
    let source = a.scenarios[i].clone();
    for light in [false, true] {
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
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [760., 1600.] {
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    let engine::State::HostNetwork { simulation } =
                        &mut a.progress.attempts.get_mut(&source.id).unwrap().state
                    else {
                        panic!()
                    };
                    simulation.view = view;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 7000., vec![]);
                    }
                    assert!(
                        (ctx.zoom_factor() - scale).abs() < 0.001,
                        "Measure the actual app preference, not a reset/default zoom"
                    );
                    let clip = a
                        .metrics
                        .main_viewport
                        .unwrap()
                        .intersect(ctx.content_rect());
                    for (key, rect, _) in &a.metrics.host.fields {
                        assert!(
                            rect.left() >= clip.left() - 1. && rect.right() <= clip.right() + 1.,
                            "{key} {view:?} {scale} width{width} {rect:?} {clip:?}"
                        );
                    }
                    assert!(
                        !a.metrics
                            .host
                            .controls
                            .iter()
                            .any(|r| matches!(r.0.as_str(), "probe-run" | "save-resolver"))
                    );
                }
            }
        }
    }
}

#[test]
fn echo_diagnosis_painted_terminal_and_facts_keep_the_shared_padding_gap() {
    let (ctx, mut a, i) = opened("lpic1-echo-path-diagnosis");
    let source = a.scenarios[i].clone();
    tap(&ctx, &mut a, "assignment");
    for text in &source.solution[..3] {
        action(
            &ctx,
            &mut a,
            &serde_json::from_str(text.strip_prefix("host-action ").unwrap()).unwrap(),
        );
    }
    tap(&ctx, &mut a, "check");
    a.scale = 1.;
    a.lesson_scroll_reset = true;
    fn find(shape: &egui::Shape, clip: egui::Rect, out: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::Shape::Text(text) => out.push((
                text.galley.job.text.clone(),
                shape.visual_bounding_rect().intersect(clip),
            )),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    find(shape, clip, out)
                }
            }
            _ => {}
        }
    }
    let mut labels = Vec::new();
    for n in 0..4 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920., 1080.),
                )),
                time: Some(100. + n as f64),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        labels.clear();
        for shape in &output.shapes {
            find(&shape.shape, shape.clip_rect, &mut labels);
        }
    }
    let left = labels
        .iter()
        .find(|(t, _)| t.starts_with("# ip -4 route replace default"))
        .expect("Actual painted terminal command")
        .1;
    let right = labels
        .iter()
        .find(|(t, _)| t.starts_with("Gateway 10.96.16.1"))
        .expect("Actual painted adjacent fact")
        .1;
    assert!(left.is_positive() && right.is_positive());
    let gap = right.left() - left.right();
    println!("ECHO_PAINT_GAP={gap} terminal={left:?} facts={right:?}");
    assert!(
        gap >= f32::from(controls::PAD_X) - 1.,
        "Painted transcript and facts need the shared padding, not just disjoint input rectangles: gap{gap}"
    );
}
