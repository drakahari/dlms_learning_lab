use super::*;
use dlms_learning_lab::ldap_workspace as work;
const IDS: [&str; 2] = ["lpic2-ldap-client-base", "lpic2-ldap-contact-query"];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    app.smoke = false;
    let i = app.scenarios.iter().position(|s| s.id == id).unwrap();
    app.launch(i);
    draw(&ctx, &mut app);
    (ctx, app, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..30 {
        frame(ctx, a, 1200., 950., vec![]);
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let (r, clip) = match key {
            "config" => {
                let (r, _, clip) = a.metrics.ldap_client.config.unwrap();
                (r, clip)
            }
            "command" => {
                let (r, _, clip) = a.metrics.ldap_client.command.unwrap();
                (r, clip)
            }
            "challenge" => (a.metrics.mode_challenge.unwrap(), main),
            _ => {
                let (_, r, c) = a
                    .metrics
                    .ldap_client
                    .controls
                    .iter()
                    .find(|c| c.0 == key)
                    .unwrap();
                (*r, *c)
            }
        };
        let clip = clip.intersect(main);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(egui::pos2(main.left() + 1., main.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (main.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..6 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Directory target did not become visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 950., r);
    let id = if key == "config" {
        a.metrics.ldap_client.config.unwrap().1
    } else {
        a.metrics.ldap_client.command.unwrap().1
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
            egui::Event::Paste(text.into()),
        ],
    );
    let source = &a.scenarios[a.selected.unwrap()];
    let State::LdapClient { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    let scope = if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    };
    assert_eq!(
        if key == "config" {
            &scope.config
        } else {
            &scope.command
        },
        text
    );
}
#[test]
fn ldap_real_fields_apply_run_check_partial_feedback_and_stale_edits() {
    for (id, start) in IDS.into_iter().zip([0, 2]) {
        let (ctx, mut a, i) = opened(id);
        let d = work::source(&a.scenarios[i]).unwrap().clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.ldap_client.result.map(|r| r.0), Some(start));
        assert!(!a.metrics.completion);
        edit(&ctx, &mut a, "config", &d.initial.reference_config);
        edit(&ctx, &mut a, "command", &d.initial.reference_query);
        tap(&ctx, &mut a, "apply");
        tap(&ctx, &mut a, "run");
        let State::LdapClient { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(
            simulation
                .assignment
                .output
                .as_ref()
                .unwrap()
                .contains("dn:")
        );
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        edit(
            &ctx,
            &mut a,
            "command",
            "ldapsearch -x -LLL '(uid=alex' uid",
        );
        assert!(!a.metrics.completion);
        assert!(a.metrics.ldap_client.result.is_none());
        tap(&ctx, &mut a, "run");
        assert!(a.notice.is_none());
        let State::LdapClient { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(
            simulation
                .assignment
                .output
                .as_ref()
                .unwrap()
                .starts_with("Search not run: Syntax:")
        );
        edit(&ctx, &mut a, "command", &d.initial.reference_query);
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
    }
}
#[test]
fn ldap_actual_practice_fill_and_challenge_keep_assignment_independent() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let d = work::source(&a.scenarios[i]).unwrap().clone();
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "worked");
    tap(&ctx, &mut a, "fill-example");
    let State::LdapClient { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment, work::Scope::new(&d.initial).unwrap());
    assert!(!simulation.practice.checked);
    assert!(simulation.practice.output.is_none());
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "challenge");
    assert!(
        !a.metrics
            .ldap_client
            .controls
            .iter()
            .any(|c| matches!(c.0.as_str(), "fill-example" | "example" | "introduction"))
    );
    assert!(a.metrics.ldap_client.config.is_some());
    assert!(a.metrics.ldap_client.command.is_some());
}
#[test]
fn ldap_all_supported_scales_themes_and_widths_keep_fields_and_scrolls_readable() {
    let (ctx, mut a, i) = opened(IDS[1]);
    let id = a.scenarios[i].id.clone();
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
            for (width, height) in [(1500., 1600.), (760., 1200.)] {
                let State::LdapClient { simulation } =
                    &mut a.progress.attempts.get_mut(&id).unwrap().state
                else {
                    panic!()
                };
                simulation.view = teaching::View::Assignment;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                let m = &a.metrics.ldap_client;
                let (c, _, _) = m.command.unwrap();
                let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                let expected =
                    ctx.fonts_mut(|f| f.row_height(&font)).ceil() + 2. * f32::from(controls::PAD_Y);
                assert!((c.height() - expected).abs() <= 2.);
                assert!(c.width() > 50.);
                let cfg = m.config.unwrap().0;
                assert!(cfg.width() > 50.);
                assert!(cfg.min.x >= 0. && c.min.x >= 0.);
                assert!(cfg.max.x <= width / scale + 1. && c.max.x <= width / scale + 1.);
                if width == 760. {
                    assert!(!m.side_by_side);
                }
                assert!(m.output.is_some());
            }
        }
    }
}

#[test]
fn ldap_enter_runs_the_literal_query_and_tab_moves_focus() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let d = work::source(&a.scenarios[i]).unwrap().clone();
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, "config", &d.initial.reference_config);
    tap(&ctx, &mut a, "apply");
    edit(&ctx, &mut a, "command", &d.initial.reference_query);
    let id = a.metrics.ldap_client.command.unwrap().1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
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
    let State::LdapClient { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert!(
        simulation
            .assignment
            .output
            .as_ref()
            .unwrap()
            .contains("uid: lee")
    );
    assert!(!simulation.assignment.checked);
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
    let moved = ctx.memory(|m| m.focused());
    assert!(
        moved.is_some(),
        "Tab must focus a real control, not merely leave the field unfocused"
    );
    assert_ne!(moved, Some(id));
}
