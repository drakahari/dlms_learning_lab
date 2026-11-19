//! Actual native widgets; injected input is technical evidence, not a novice study.
use super::*;
use dlms_learning_lab::{ftp_identity_source as binding, ftp_identity_workspace as work};
const ID: &str = "lpic2-ftp-local-jail";
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
    let State::FtpIdentity { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::FtpIdentity { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .ftp_identity
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
        .ftp_identity
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
fn ftp_identity_repeated_failure_cause_is_painted_once_and_every_case_remains_inspectable() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    let id = "lpic2-ftp-upload-rights";
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    let State::FtpIdentity { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    settle(&ctx, &mut a);
    hit(&ctx, &mut a, "check");
    let definition = binding::source(&a.scenarios[i]).unwrap().clone();
    let State::FtpIdentity { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    let before = simulation.clone();
    let checks = simulation.assignment.checks(&definition.assignment, &definition.assignment_inputs);
    let failed = checks.iter().filter(|c| !c.passed).count();
    let groups = ftp_identity_page::failure_groups(&checks);
    assert!(groups.len() < failed, "Exercise the actual repeated login failure");
    assert_eq!(groups.iter().map(|(_, labels)| labels.len()).sum::<usize>(), failed);
    let text = visible_text(&ctx, &mut a);
    for (detail, _) in &groups {
        assert_eq!(text.iter().filter(|s| s.as_str() == *detail).count(), 1);
    }
    let pass = checks.len() - failed;
    assert!(text.contains(&format!("Not complete · {pass}/{} outcomes pass", checks.len())));
    assert!(!text.iter().any(|s| s == "Assignment complete"));
    let (index, (_, labels)) = groups.iter().enumerate().find(|(_, (_, l))| l.len() > 1).unwrap();
    hit(&ctx, &mut a, &format!("failed-outcomes-{index}"));
    let text = visible_text(&ctx, &mut a);
    for label in labels {
        assert!(text.contains(&format!("Not passed · {label}")), "Missing case: {label}");
    }
    let State::FtpIdentity { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    assert_eq!(simulation, &before, "Inspecting feedback must not change grading or work");
}
#[test]
fn ftp_identity_enter_uses_the_correct_terminal_and_keeps_example_assignment_isolated() {
    let (ctx, mut a, _) = open(teaching::View::Example);
    let original = session(&a).assignment.clone();
    edit(&ctx, &mut a, "admin-input", "cat /etc/vsftpd.conf");
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
    assert!(session(&a).example.admin_output.contains("local_enable"));
    assert!(session(&a).example.client_output.is_empty());
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
}
#[test]
fn ftp_identity_editor_save_and_check_button_preserve_real_state_and_invalidate_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    for raw in &s.solution {
        engine::execute_for_goal(
            &mut a.progress.attempts.get_mut(ID).unwrap().state,
            &s.goal,
            raw,
        )
        .unwrap();
    }
    settle(&ctx, &mut a);
    assert!(a.metrics.completion);
    edit(&ctx, &mut a, "editor", "local_enable=NO\n");
    assert!(!session(&a).assignment.checked);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "save");
    assert_eq!(session(&a).assignment.world.saved, "local_enable=NO\n");
    edit(&ctx, &mut a, "admin-input", "systemctl restart vsftpd");
    hit(&ctx, &mut a, "admin-run");
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(a.metrics.ftp_identity.result.is_some_and(|(p, n, _)| p < n));
}
#[test]
fn ftp_identity_challenge_keeps_meaningful_workspace_without_unsolicited_guidance() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in ["introduction", "example", "try-example"] {
        assert!(!a.metrics.ftp_identity.controls.iter().any(|v| v.0 == key));
    }
    for key in ["editor", "admin-input", "client-input", "connect", "check"] {
        assert!(a.metrics.ftp_identity.controls.iter().any(|v| v.0 == key));
    }
}
#[test]
fn ftp_identity_editor_footer_stays_fixed_and_fields_fit_across_scales_and_themes() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let d = binding::source(&a.scenarios[i]).unwrap().clone();
    for (width, scale) in [(1450., 1.), (760., 1.5), (760., 2.)] {
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
            let paint = |a: &mut LabApp| {
                for _ in 0..3 {
                    frame(&ctx, a, width, 16000., vec![]);
                }
            };
            let State::FtpIdentity { simulation } =
                &mut a.progress.attempts.get_mut(ID).unwrap().state
            else {
                panic!()
            };
            simulation
                .action(
                    &d,
                    &work::Action::Edit {
                        text: String::new(),
                    },
                    false,
                )
                .unwrap();
            paint(&mut a);
            let rect = |a: &LabApp, key: &str| {
                a.metrics
                    .ftp_identity
                    .controls
                    .iter()
                    .find(|x| x.0 == key)
                    .unwrap()
                    .1
            };
            let before = rect(&a, "save");
            if width > 1400. && scale == 1. {
                assert!(
                    rect(&a, "client-input").left() > rect(&a, "admin-input").right(),
                    "Wide workspace must use the approved split layout"
                );
            }
            let State::FtpIdentity { simulation } =
                &mut a.progress.attempts.get_mut(ID).unwrap().state
            else {
                panic!()
            };
            simulation
                .action(
                    &d,
                    &work::Action::Edit {
                        text: format!(
                            "{}# {}\n",
                            d.assignment.initial.saved,
                            "wide-comment ".repeat(80)
                        ),
                    },
                    false,
                )
                .unwrap();
            paint(&mut a);
            let after = rect(&a, "save");
            assert!(
                (before.top() - after.top()).abs() < 0.1,
                "Scrollbar shifted Save: {before:?} {after:?}"
            );
            let viewport = rect(&a, "editor-viewport");
            let field = a
                .metrics
                .ftp_identity
                .fields
                .iter()
                .find(|x| x.0 == "editor")
                .unwrap()
                .1;
            assert!(
                field.width() > viewport.width() * 2.,
                "Long line must scroll rather than soft-wrap"
            );
            for key in [
                "admin-input",
                "admin-run",
                "client-input",
                "client-run",
                "file-choice",
                "client-choice",
            ] {
                let r = rect(&a, key);
                assert!(
                    r.left() >= -1. && r.right() <= width + 1.,
                    "{key} {r:?} at {width}/{scale}"
                );
            }
        }
    }
}
