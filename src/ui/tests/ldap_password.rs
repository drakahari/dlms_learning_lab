use super::*;
use dlms_learning_lab::ldap_password_workspace as work;
const ID: &str = "lpic2-ldap-protected-password";
fn frame(ctx: &egui::Context, a: &mut LabApp, width: f32, height: f32, events: Vec<egui::Event>) {
    let mut delivered = std::mem::take(&mut a.pending_events);
    delivered.extend(events);
    super::frame(ctx, a, width, height, delivered);
}

fn opened() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 950., vec![]);
    }
    (ctx, a, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..40 {
        frame(ctx, a, 1200., 950., vec![]);
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let (r, clip) = match key {
            "config" => {
                let (r, _, c) = a.metrics.ldap_password.config.unwrap();
                (r, c)
            }
            "command" => {
                let (r, _, c) = a.metrics.ldap_password.command.unwrap();
                (r, c)
            }
            "challenge" => (a.metrics.mode_challenge.unwrap(), main),
            _ => {
                let (_, r, c) = a
                    .metrics
                    .ldap_password
                    .controls
                    .iter()
                    .find(|c| c.0 == key)
                    .unwrap();
                (*r, *c)
            }
        };
        if smoke::smoke_control_visible(r, clip.intersect(main), ctx.pixels_per_point()) {
            return r;
        }
        a.reveal_smoke_control(ctx, r);
    }
    panic!("Password target did not become visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    let id = if key == "command" {
        a.metrics.ldap_password.command.unwrap().1
    } else {
        a.metrics.ldap_password.config.unwrap().1
    };
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
            egui::Event::Paste(value.into()),
        ],
    );
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
#[test]
fn password_native_practice_fill_run_check_and_challenge_preserve_independent_work() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    tap(&ctx, &mut a, "example");
    let State::LdapPassword { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    let before = simulation.assignment.clone();
    tap(&ctx, &mut a, "worked");
    tap(&ctx, &mut a, "fill-example");
    let State::LdapPassword { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.practice.draft, d.example.reference);
    assert!(simulation.practice.output.is_none());
    assert_eq!(simulation.assignment, before);
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "check");
    let State::LdapPassword { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.practice.checks(&d.example).iter().all(|r| r.0));
    assert_eq!(simulation.assignment, before);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    tap(&ctx, &mut a, "assignment");
    let State::LdapPassword { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.draft, d.initial.starter);
    tap(&ctx, &mut a, "challenge");
    assert!(!a.metrics.ldap_password.controls.iter().any(|c| matches!(
        c.0.as_str(),
        "introduction" | "example" | "worked" | "fill-example"
    )));
}
#[test]
fn password_native_command_and_config_editing_have_focus_and_clear_stale_success_without_autorun() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    tap(&ctx, &mut a, "assignment");
    engine::execute_for_goal(
        &mut a.progress.attempts.get_mut(ID).unwrap().state,
        &source.goal,
        &source.solution[0],
    )
    .unwrap();
    frame(&ctx, &mut a, 1200., 950., vec![]);
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    edit(&ctx, &mut a, "command", "ldappasswd -x");
    let State::LdapPassword { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.draft.command, "ldappasswd -x");
    assert!(!simulation.assignment.checked && simulation.assignment.output.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    let id = a.metrics.ldap_password.command.unwrap().1;
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
            modifiers: egui::Modifiers::NONE,
        }],
    );
    frame(&ctx, &mut a, 1200., 950., vec![]);
    let focused = ctx.memory(|m| m.focused());
    assert!(focused.is_some());
    assert_ne!(focused, Some(id));
    edit(&ctx, &mut a, "command", &d.initial.reference.command);
    edit(&ctx, &mut a, "config", &d.initial.reference.config);
    let State::LdapPassword { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(
        simulation.assignment.draft.config,
        d.initial.reference.config
    );
    assert!(simulation.assignment.output.is_none());
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.ldap_password.result.unwrap().0, 3);
}
#[test]
fn password_native_controls_and_actual_probe_table_fit_two_widths_five_scales_and_dark_light() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    let State::LdapPassword { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    simulation
        .assignment
        .edit(d.initial.reference.clone())
        .unwrap();
    simulation.assignment.run(&d.initial).unwrap();
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
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for (width, height) in [(1500., 1800.), (760., 1600.)] {
                for _ in 0..5 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                let m = &a.metrics.ldap_password;
                for key in [
                    "command",
                    "config",
                    "bind",
                    "old",
                    "replacement",
                    "case",
                    "run",
                ] {
                    let (_, r, _) = m.controls.iter().find(|c| c.0 == key).unwrap();
                    assert!(
                        r.min.x >= 0. && r.max.x <= width / scale + 1.,
                        "{key}: {r:?} scale={scale} width={width}"
                    );
                }
                let command = m.command.unwrap().0;
                let style = ctx.style();
                let expected = [
                    egui::TextStyle::Body,
                    egui::TextStyle::Button,
                    egui::TextStyle::Monospace,
                ]
                .iter()
                .map(|kind| ctx.fonts_mut(|f| f.row_height(&kind.resolve(&style))))
                .fold(0_f32, f32::max)
                .ceil()
                    + 2. * f32::from(controls::PAD_Y);
                assert!(command.height() >= expected - 1.);
                let output = m.output.unwrap();
                assert!(
                    output.max.x <= width / scale + 1.,
                    "probe output overflow {output:?} scale={scale} width={width}"
                );
            }
        }
    }
}

#[test]
fn password_probe_labels_wrap_naturally_without_stretching_word_spaces() {
    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for width in [100., 150., 200.] {
                let text = "Unrelated identity: uid=sam,ou=People,dc=accounts,dc=test";
                let galley =
                    ldap_password_page::probe_text(ui, text, width, false, palette(ui).text);
                assert!(!galley.job.justify);
                assert_eq!(galley.job.halign, egui::Align::LEFT);
                assert!(galley.rows.len() > 1);
                assert!(galley.rows.iter().all(|row| row.size.x <= width + 1.));
                assert_eq!(galley.job.text, text);
            }
        });
    });
}
