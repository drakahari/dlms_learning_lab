//! Actual native widgets; injected input is technical evidence, not a novice study.
use super::*;
use dlms_learning_lab::{service_exposure_source as binding, service_exposure_workspace as work};
const ID: &str = "lpic2-exposure-client-policy";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 12000., events);
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
    let State::ServiceExposure { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::ServiceExposure { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .service_exposure
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
        .service_exposure
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
fn visible_text(ctx: &egui::Context, a: &mut LabApp) -> Vec<String> {
    fn collect(shape: &egui::epaint::Shape, text: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    collect(s, text);
                }
            }
            egui::epaint::Shape::Text(t) => text.push(t.galley.job.text.clone()),
            _ => {}
        }
    }
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1450., 12000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = vec![];
    for s in out.shapes {
        collect(&s.shape, &mut text);
    }
    text
}

#[test]
fn exposure_enter_runs_only_the_focused_context_and_never_grades_practice() {
    let (ctx, mut a, _) = open(teaching::View::Example);
    let original = session(&a).assignment.clone();
    edit(&ctx, &mut a, "admin-input", "ss -ltnp");
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
    assert!(session(&a).example.admin_output.contains("LISTEN"));
    assert!(session(&a).example.scan_output.is_empty());
    edit(
        &ctx,
        &mut a,
        "scan-input",
        "nmap -n -Pn -sT -p 22,80,443,9000 192.0.2.10",
    );
    hit(&ctx, &mut a, "scan-run");
    assert!(session(&a).example.scan_output.contains("filtered"));
    assert!(session(&a).example.last_scan.is_some());
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
}
#[test]
fn exposure_real_controls_check_then_repair_and_invalidate_broken_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "check");
    assert!(session(&a).assignment.checked);
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .service_exposure
            .result
            .is_some_and(|(n, total, _)| n < total)
    );
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let envelope: serde_json::Value =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        let action: work::Action = serde_json::from_value(envelope["action"].clone()).unwrap();
        match action {
            work::Action::Event {
                event: work::Event::Admin { input },
            } => {
                edit(&ctx, &mut a, "admin-input", &input);
                hit(&ctx, &mut a, "admin-run");
            }
            work::Action::Event {
                event: work::Event::Check,
            } => hit(&ctx, &mut a, "check"),
            other => panic!("Review actual authored solution action {other:?}"),
        }
    }
    assert!(a.metrics.completion);
    edit(&ctx, &mut a, "admin-input", "kill -STOP 500");
    hit(&ctx, &mut a, "admin-run");
    assert!(!session(&a).assignment.checked);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    edit(&ctx, &mut a, "admin-input", "kill -CONT 500");
    hit(&ctx, &mut a, "admin-run");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
}
#[test]
fn exposure_challenge_retains_the_full_workspace_without_unsolicited_worked_answers() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["introduction", "example", "try-example", "worked"] {
        assert!(
            !a.metrics
                .service_exposure
                .controls
                .iter()
                .any(|v| v.0 == key)
        );
    }
    for key in [
        "assignment-content",
        "admin-input",
        "scan-input",
        "observer-choice",
        "check",
    ] {
        assert!(
            a.metrics
                .service_exposure
                .controls
                .iter()
                .any(|v| v.0 == key)
        );
    }
    let text = visible_text(&ctx, &mut a);
    assert!(!text.iter().any(|s| s.contains("iptables -D INPUT 1")));
    assert!(!text.iter().any(|s| s.contains("Show worked answer")));
}
#[test]
fn exposure_fields_and_columns_reflow_with_large_text_and_themes() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    edit(&ctx, &mut a, "scan-input", "nc -nvz 198.51.100.20 80");
    hit(&ctx, &mut a, "scan-run");
    assert_eq!(session(&a).assignment.scan_status, Some(1));
    for (width, scale) in [
        (1450., 1.),
        (1450., 1.5),
        (760., 1.),
        (760., 1.5),
        (760., 2.),
    ] {
        ctx.set_zoom_factor(scale);
        a.scale = scale;
        for light in [false, true] {
            a.theme_id = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap()
                .id
                .clone();
            theme::apply(&ctx, &a.theme_id);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 16000., vec![]);
            }
            let rect = |key: &str| {
                a.metrics
                    .service_exposure
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap()
                    .1
            };
            let admin = rect("admin-input");
            let scan = rect("scan-input");
            if width == 1450. && scale == 1. {
                assert!(
                    scan.left() > admin.right(),
                    "Wide layout should split contexts"
                );
            } else if width == 760. {
                assert!(
                    scan.top() > admin.bottom(),
                    "Narrow layout should stack contexts"
                );
            }
            for key in [
                "admin-input",
                "admin-run",
                "scan-input",
                "scan-run",
                "observer-choice",
                "scan-status",
                "check",
            ] {
                let r = rect(key);
                assert!(
                    r.left() >= -1. && r.right() <= width + 1.,
                    "{key} {r:?} at {width}/{scale}"
                );
                assert!(r.height() > 15., "Control must remain a usable text target");
            }
        }
    }
}

#[test]
fn exposure_native_nc_uses_new_history_and_explains_negative_results_without_credit() {
    let (ctx, mut a, _) = open(teaching::View::Example);
    let original = session(&a).assignment.clone();
    for (command, status, observed, detail) in [
        ("nc -n -v -z 192.0.2.10 443", 0, true, "command completed"),
        (
            "nc -nvz 192.0.2.10 80",
            1,
            true,
            "connection refused · no listener accepts this port",
        ),
        (
            "nc -zvn 192.0.2.10 22",
            1,
            true,
            "modeled timeout · a request or reply is dropped",
        ),
        (
            "nc -lnvz 192.0.2.10 443",
            1,
            false,
            "command needs attention",
        ),
    ] {
        edit(&ctx, &mut a, "scan-input", command);
        hit(&ctx, &mut a, "scan-run");
        let scope = &session(&a).example;
        assert_eq!(scope.scan_status, Some(status), "{command}");
        assert_eq!(scope.last_scan.is_some(), observed, "{command}");
        assert!(
            matches!(scope.history.last(), Some(work::Event::ObserverV3 { input }) if input == command)
        );
        assert!(
            visible_text(&ctx, &mut a)
                .iter()
                .any(|t| t.contains(detail)),
            "{command}: {detail}"
        );
        assert!(!a.metrics.completion);
        assert_eq!(session(&a).assignment, original);
    }
    let restored: work::Session =
        serde_json::from_str(&serde_json::to_string(session(&a)).unwrap()).unwrap();
    let d = binding::source(&a.scenarios[a.selected.unwrap()]).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(&restored, session(&a));
}

#[test]
fn exposure_introduction_has_one_worked_heading_and_separate_expected_caption() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    let text = visible_text(&ctx, &mut a);
    assert_eq!(
        text.iter()
            .filter(|t| t.as_str() == "Worked example")
            .count(),
        1
    );
    assert_eq!(
        text.iter()
            .filter(|t| t.as_str() == "Expected observations and actions")
            .count(),
        1
    );
    assert!(!text.iter().any(|t| t.contains("Worked example · expected")));
    assert!(
        text.iter()
            .any(|t| t.contains("nc -n -v -z 192.0.2.10 443"))
    );
}

#[test]
fn exposure_telnet_controls_keep_a_live_client_then_escape_and_quit_without_practice_credit() {
    use dlms_learning_lab::telnet_client::Mode;
    let (ctx, mut a, i) = open(teaching::View::Example);
    let original = session(&a).assignment.clone();
    edit(&ctx, &mut a, "scan-input", "telnet -c 192.0.2.10 443");
    hit(&ctx, &mut a, "scan-run");
    assert_eq!(
        session(&a).example.telnet,
        Some(Mode::Connected { port: 443 })
    );
    assert_eq!(session(&a).example.scan_status, None);
    assert!(
        visible_text(&ctx, &mut a)
            .iter()
            .any(|s| s.contains("client still running"))
    );
    let before = session(&a).example.clone();
    hit(&ctx, &mut a, "scan-input");
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Text("GET / HTTP/1.0".into())],
    );
    settle(&ctx, &mut a);
    assert_eq!(
        session(&a).example,
        before,
        "Unsupported data entry is disabled, never sent or silently executed"
    );
    hit(&ctx, &mut a, "scan-escape");
    assert_eq!(
        session(&a).example.telnet,
        Some(Mode::Command { port: 443 })
    );
    edit(&ctx, &mut a, "scan-input", "qui");
    let restored: work::Session =
        serde_json::from_slice(&serde_json::to_vec(session(&a)).unwrap()).unwrap();
    binding::source(&a.scenarios[i])
        .unwrap()
        .bound(&restored)
        .unwrap();
    assert_eq!(restored.example.scan_input, "qui");
    edit(&ctx, &mut a, "scan-input", "quit");
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
    assert!(session(&a).example.telnet.is_none());
    assert_eq!(session(&a).example.scan_status, Some(0));
    assert!(session(&a).example.scan_output.contains("client exited"));
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
}

#[test]
fn exposure_telnet_live_and_local_controls_reflow_without_shrinking_or_clipping() {
    use dlms_learning_lab::telnet_client::Mode;
    let (ctx, mut a, _) = open(teaching::View::Example);
    edit(&ctx, &mut a, "scan-input", "telnet -c 192.0.2.10 443");
    hit(&ctx, &mut a, "scan-run");
    for local in [false, true] {
        if local {
            ctx.set_zoom_factor(1.);
            a.scale = 1.;
            settle(&ctx, &mut a);
            hit(&ctx, &mut a, "scan-escape");
            assert_eq!(
                session(&a).example.telnet,
                Some(Mode::Command { port: 443 })
            );
        }
        for (scale, light) in [(1., false), (1.5, true), (2., false)] {
            ctx.set_zoom_factor(scale);
            a.scale = scale;
            a.theme_id = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap()
                .id
                .clone();
            theme::apply(&ctx, &a.theme_id);
            for _ in 0..3 {
                frame(&ctx, &mut a, 760., 16000., vec![]);
            }
            let outer = ctx.content_rect();
            for key in [
                "scan-status",
                "scan-input",
                "scan-run",
                "observer-choice",
                "check",
            ] {
                let (_, rect, _) = a
                    .metrics
                    .service_exposure
                    .controls
                    .iter()
                    .find(|r| r.0 == key)
                    .unwrap();
                assert!(
                    rect.left() >= outer.left() - 1. && rect.right() <= outer.right() + 1.,
                    "{key} {rect:?} at {scale}/{local}; outer {outer:?}"
                );
                assert!(rect.height() > 15.);
            }
        }
    }
}
