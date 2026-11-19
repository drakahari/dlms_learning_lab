//! Registered native configuration page checks.
use super::*;
use dlms_learning_lab::ldap_access_workspace as work;
const ID: &str = "lpic2-ldap-config-deployment";
fn frame(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, events: Vec<egui::Event>) {
    let mut delivered = std::mem::take(&mut a.pending_events);
    delivered.extend(events);
    super::frame(ctx, a, w, h, delivered);
}
fn opened() -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == ID)
        .expect("Configuration deployment must be present in the normal review catalog");
    a.launch(i);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 1000., vec![]);
    }
    (ctx, a, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..45 {
        frame(ctx, a, 1200., 1000., vec![]);
        let main = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let (r, clip) = match key {
            "editor" => {
                let (r, _, c) = a.metrics.ldap_access.editor.unwrap();
                (r, c)
            }
            "challenge" => (a.metrics.mode_challenge.unwrap(), main),
            _ => {
                let (_, r, c) = a
                    .metrics
                    .ldap_access
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
    panic!("ACL target did not become visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 1000., r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 1000., vec![]);
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    let r = target(ctx, a, "editor");
    super::debian_contents::point_click(ctx, a, 1200., 1000., r);
    let id = a.metrics.ldap_access.editor.unwrap().1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    frame(
        ctx,
        a,
        1200.,
        1000.,
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
    for _ in 0..3 {
        frame(ctx, a, 1200., 1000., vec![]);
    }
}
#[test]
fn configuration_native_practice_fill_is_inert_runs_real_changes_and_cannot_award_assignment_credit()
 {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    tap(&ctx, &mut a, "example");
    let State::LdapAccess { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    let assignment = simulation.assignment.clone();
    tap(&ctx, &mut a, "worked");
    tap(&ctx, &mut a, "fill-example");
    let State::LdapAccess { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.practice.draft, d.example.reference);
    assert!(simulation.practice.output.is_none());
    assert_eq!(simulation.assignment, assignment);
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ldap_access.result.map(|r| (r.0, r.1)),
        Some((3, 3))
    );
    let State::LdapAccess { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert!(
        simulation
            .practice
            .output
            .as_ref()
            .is_some_and(|r| r.allowed && r.passed && r.before != r.after)
    );
    assert_eq!(simulation.assignment, assignment);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "challenge");
    assert!(!a.metrics.ldap_access.controls.iter().any(|c| matches!(
        c.0.as_str(),
        "introduction" | "example" | "worked" | "fill-example"
    )));
}
#[test]
fn configuration_native_keyboard_editing_and_tab_clear_stale_success_without_automatic_execution() {
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
    frame(&ctx, &mut a, 1200., 1000., vec![]);
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    edit(&ctx, &mut a, "access to * by * read");
    let State::LdapAccess { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.assignment.draft, "access to * by * read");
    assert!(!simulation.assignment.checked && simulation.assignment.output.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    let id = a.metrics.ldap_access.editor.unwrap().1;
    frame(
        &ctx,
        &mut a,
        1200.,
        1000.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    frame(&ctx, &mut a, 1200., 1000., vec![]);
    assert!(ctx.memory(|m| m.focused()).is_some());
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    edit(&ctx, &mut a, &d.initial.reference);
    let State::LdapAccess { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.assignment.output.is_none());
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ldap_access.result.map(|r| (r.0, r.1)),
        Some((3, 3))
    );
}
#[test]
fn configuration_native_editor_controls_and_long_actual_values_fit_two_widths_five_scales_and_both_themes()
 {
    let (ctx, mut a, i) = opened();
    if let engine::Goal::LdapAccess { definition } = &mut a.scenarios[i].goal {
        definition.initial.cases[0].probes[0].fields[0].value =
            format!("{}@fixture.test", "avery-long-".repeat(20));
    }
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    let State::LdapAccess { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
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
                let m = &a.metrics.ldap_access;
                for key in ["editor", "case", "probe", "run", "check"] {
                    let (_, r, _) = m.controls.iter().find(|c| c.0 == key).unwrap();
                    assert!(
                        r.min.x >= 0. && r.max.x <= width / scale + 1.,
                        "{key} overflow {r:?} scale={scale} width={width}"
                    );
                }
                let output = m.output.unwrap();
                assert!(
                    output.max.x <= width / scale + 1.,
                    "Actual output overflow {output:?} scale={scale} width={width}"
                );
                // Long policy headers are logical lines. Soft wrapping would
                // falsely teach newlines in this line-oriented configuration.
                assert_eq!(m.editor_rows, Some(d.initial.reference.split('\n').count()));
                let text_clip = m.editor_text_clip.unwrap();
                assert!(text_clip.width() <= m.editor.unwrap().0.width() + 1.);
                let inventory = m
                    .configuration_inventory
                    .expect("The supplied backend inventory is essential");
                assert!(
                    inventory.min.x >= 0. && inventory.max.x <= width / scale + 1.,
                    "Inventory overflow {inventory:?} scale={scale} width={width}"
                );
                let editor = m.editor.unwrap().0;
                let style = ctx.style();
                let row =
                    ctx.fonts_mut(|f| f.row_height(&egui::TextStyle::Monospace.resolve(&style)));
                assert!(editor.height() >= row * 10.);
            }
        }
    }
}
