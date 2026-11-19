//! Real native widgets for endpoint pairing and reverse-policy repair.
use super::*;
use dlms_learning_lab::transport_workspace as work;
const IDS: [&str; 3] = [
    "ccna-transport-port-pairing",
    "ccna-transport-return-policy",
    "lpic1-protocol-round-trips",
];
fn setup(id: &str, view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = debug_recovery::opened(id);
    let State::Transport { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    (ctx, a, i)
}
fn full(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1600., 7000., vec![]);
    }
}
fn control(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .transport
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .1
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    full(ctx, a);
    let r = control(a, key);
    click(ctx, a, r);
    full(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    full(ctx, a);
    let (_, r, id) = a
        .metrics
        .transport
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap()
        .clone();
    let band = egui::Rect::from_min_size(
        r.min + egui::vec2(3., 3.),
        egui::vec2((r.width() - 6.).max(1.), 22.),
    );
    click(ctx, a, band);
    assert_eq!(
        ctx.memory(|m| m.focused()),
        Some(id),
        "field {key} target {r:?}, band {band:?}, viewport {:?}, notice {:?}",
        a.metrics.main_viewport,
        (a.notice.clone(), a.metrics.transport.editor_clip)
    );
    frame(
        ctx,
        a,
        1600.,
        7000.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    full(ctx, a);
}
fn session<'a>(a: &'a LabApp, id: &str) -> &'a work::Session {
    let State::Transport { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    simulation
}
#[test]
fn transport_actual_fields_check_retry_practice_isolation_and_save_resume() {
    for id in IDS {
        let (ctx, mut a, i) = setup(id, teaching::View::Example);
        let source = a.scenarios[i].clone();
        let engine::Goal::Transport { definition } = &source.goal else {
            panic!()
        };
        let original = session(&a, id).assignment.clone();
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.transport.result.is_some_and(|r| r.0 < r.1));
        tap(&ctx, &mut a, "fill-example");
        assert!(session(&a, id).practice.tested.is_none());
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.transport.result.is_some_and(|r| r.0 == r.1));
        assert_eq!(session(&a, id).assignment, original);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.transport.result.is_some_and(|r| r.0 < r.1));
        if id == IDS[0] {
            replace(&ctx, &mut a, "listener-0-port", "8443");
        } else {
            replace(
                &ctx,
                &mut a,
                "config-R1",
                &definition.worked_assignment.documents.routers["R1"],
            );
        }
        assert!(session(&a, id).assignment.tested.is_none());
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        if id == IDS[2] {
            assert!(a.metrics.unit_complete.is_some() && a.metrics.unit_back.is_some());
            assert!(
                a.metrics
                    .transport
                    .controls
                    .iter()
                    .any(|r| r.0 == "echo-client-echo")
            );
            let trace = a
                .metrics
                .transport
                .controls
                .iter()
                .find(|r| r.0 == "echo-client-echo")
                .unwrap()
                .1;
            assert!(a.metrics.transport.case_feedback.unwrap().bottom() <= trace.top());
        }
        a.save();
        assert!(a.storage_error.is_none());
        let loaded = progress::load(&a.path, &a.scenarios).unwrap();
        assert!(progress::current_solved(&loaded.attempts[id], &source));
        if id == IDS[0] {
            replace(&ctx, &mut a, "report-retry-server", "1");
        } else {
            replace(
                &ctx,
                &mut a,
                "config-R1",
                "ip access-list extended RETURN\n 10 permit ip any any\nexit\ninterface office\n ip access-group RETURN out\nexit\n",
            );
        }
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        if id == IDS[2] {
            assert!(a.metrics.unit_complete.is_none() && a.metrics.unit_back.is_none());
        }
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.transport.result.is_some_and(|r| r.0 < r.1));
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "reset-example");
        assert!(session(&a, id).practice.tested.is_none());
        assert_eq!(
            session(&a, id).practice,
            work::Scope::new(&definition.example).unwrap()
        );
    }
}
fn shape_text(shape: &egui::Shape, out: &mut String) {
    match shape {
        egui::Shape::Text(t) => {
            out.push_str(&t.galley.job.text);
            out.push('\n');
        }
        egui::Shape::Vec(v) => {
            for s in v {
                shape_text(s, out)
            }
        }
        _ => {}
    }
}
#[test]
fn transport_three_screens_remain_source_specific_and_assignment_hides_worked_repair() {
    for id in IDS {
        let (ctx, mut a, _) = setup(id, teaching::View::Introduction);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            let State::Transport { simulation } =
                &mut a.progress.attempts.get_mut(id).unwrap().state
            else {
                panic!()
            };
            simulation.view = view;
            full(&ctx, &mut a);
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1600., 7000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            let mut text = String::new();
            for s in out.shapes {
                shape_text(&s.shape, &mut text);
            }
            assert!(!text.contains("More detail · original explanations"));
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("SYN"));
                    if id == IDS[2] {
                        assert!(text.contains("9443") && text.contains("ICMP"));
                    } else {
                        assert!(text.contains("8088"));
                    }
                    assert!(a.metrics.example_action.is_some());
                }
                teaching::View::Example => {
                    assert!(text.contains("Not checked"));
                    assert!(text.contains("Check configuration"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("10 permit tcp host 198.51.100.20 eq 8443"));
                    assert!(
                        !text.contains(
                            "10 permit tcp host 10.80.20.20 eq 8443 host 10.80.10.10 range"
                        )
                    );
                }
            }
        }
    }
}
#[test]
fn transport_field_alignment_focus_and_control_bounds_at_supported_scales() {
    for id in IDS {
        for light in [false, true] {
            for scale in [1., 1.15, 1.25, 1.5, 2.] {
                for width in [760., 1600.] {
                    let (ctx, mut a, _) = setup(id, teaching::View::Example);
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
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 7000., vec![]);
                    }
                    let vp = a.metrics.main_viewport.unwrap();
                    if let Some(clip) = a.metrics.transport.editor_clip {
                        let editor = a
                            .metrics
                            .transport
                            .fields
                            .iter()
                            .find(|f| f.0 == "config-R1")
                            .unwrap()
                            .1;
                        assert!(editor.top() >= clip.top() && editor.bottom() <= clip.bottom());
                    }
                    for (key, r, _) in &a.metrics.transport.fields {
                        assert!(
                            r.left() >= vp.left() - 2. && r.right() <= vp.right() + 2.,
                            "{id} {key} {scale}/{width}: {r:?}/{vp:?}"
                        );
                    }
                    for (key, r, clip) in &a.metrics.transport.controls {
                        if key.ends_with("-protocol") {
                            assert!(
                                r.width() >= 80.,
                                "Unreadable protocol selector {key}: {r:?}"
                            );
                        }
                        assert!(
                            r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                            "{id} {key} {scale}/{width}: {r:?}/{clip:?}"
                        );
                    }
                }
            }
        }
    }
}
