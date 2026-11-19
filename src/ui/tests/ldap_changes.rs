use super::*;
use dlms_learning_lab::ldap_change_workspace as work;
const IDS: [&str; 2] = ["lpic2-ldap-person-import", "lpic2-ldap-contact-maintenance"];
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
            "editor" => {
                let (r, _, clip) = a.metrics.ldap_changes.editor.unwrap();
                (r, clip)
            }
            "challenge" => (a.metrics.mode_challenge.unwrap(), main),
            _ => {
                let (_, r, c) = a
                    .metrics
                    .ldap_changes
                    .controls
                    .iter()
                    .find(|c| c.0 == key)
                    .unwrap();
                (*r, *c)
            }
        };
        let clip = clip.intersect(main);
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if key == "editor" {
                assert!(
                    a.metrics
                        .ldap_changes
                        .editor_text_clip
                        .unwrap()
                        .contains(r.center()),
                    "The click must reach the actual text area, not a scrollbar or clipped content"
                );
            }
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
    let id = a.metrics.ldap_changes.editor.unwrap().1;
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
    let State::LdapChanges { simulation } = &a.progress.attempts[&source.id].state else {
        panic!()
    };
    let scope = if simulation.view == teaching::View::Example {
        &simulation.practice
    } else {
        &simulation.assignment
    };
    assert_eq!(&scope.draft, text);
}

#[test]
fn ldap_changes_actual_editor_apply_check_feedback_and_stale_edits() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let d = work::source(&a.scenarios[i]).unwrap().clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert_eq!(
            a.metrics.ldap_changes.result.map(|r| (r.0, r.1)),
            Some((0, 3))
        );
        assert!(!a.metrics.completion);
        let font = egui::TextStyle::Monospace.resolve(&ctx.style());
        let row_height = ctx.fonts_mut(|f| f.row_height(&font));
        assert!(
            a.metrics.ldap_changes.output.unwrap().height() >= row_height,
            "The empty-output explanation must have positive readable height, not a collapsed clip"
        );
        assert!(
            a.metrics.ldap_changes.output.unwrap().height() < row_height * 5.,
            "An empty output placeholder must not reserve a terminal-sized blank region"
        );
        edit(&ctx, &mut a, "editor", &d.initial.reference);
        tap(&ctx, &mut a, "apply");
        let State::LdapChanges { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert!(simulation.assignment.output.as_ref().unwrap().applied);
        assert!(!simulation.assignment.checked);
        assert!(
            a.metrics.ldap_changes.output.unwrap().height() >= row_height * 14.,
            "Actual request/state output must retain useful scrolling space"
        );
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert!(
            a.metrics.ldap_changes.output.unwrap().height() >= row_height,
            "Completion must leave the output explanation readable"
        );
        assert!(
            a.metrics.ldap_changes.output.unwrap().height() < row_height * 5.,
            "Checking must not leave a blank result panel above the completion feedback"
        );
        edit(&ctx, &mut a, "editor", "version: 1\n\ndn: unfinished");
        assert!(!a.metrics.completion);
        assert!(a.metrics.ldap_changes.result.is_none());
        tap(&ctx, &mut a, "apply");
        assert!(a.notice.as_ref().unwrap().contains("Syntax"));
        edit(&ctx, &mut a, "editor", &d.initial.reference);
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
    }
}
#[test]
fn ldap_changes_practice_and_challenge_keep_full_assignment_independent() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let d = work::source(&a.scenarios[i]).unwrap().clone();
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "worked");
    tap(&ctx, &mut a, "fill-example");
    let State::LdapChanges { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert_eq!(simulation.assignment, work::Scope::new(&d.initial).unwrap());
    assert!(!simulation.practice.checked);
    assert!(simulation.practice.output.is_none());
    tap(&ctx, &mut a, "apply");
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert_eq!(
        a.metrics.ldap_changes.result.map(|r| (r.0, r.1)),
        Some((3, 3))
    );
    tap(&ctx, &mut a, "challenge");
    assert!(
        !a.metrics
            .ldap_changes
            .controls
            .iter()
            .any(|c| matches!(c.0.as_str(), "fill-example" | "example" | "introduction"))
    );
    assert!(a.metrics.ldap_changes.editor.is_some());
}
#[test]
fn ldap_changes_five_scales_two_widths_light_dark_keep_editor_and_controls_readable() {
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
                let State::LdapChanges { simulation } =
                    &mut a.progress.attempts.get_mut(&id).unwrap().state
                else {
                    panic!()
                };
                simulation.view = teaching::View::Assignment;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                let m = &a.metrics.ldap_changes;
                let e = m.editor.unwrap().0;
                assert!(e.width() > 50.);
                assert!(e.min.x >= 0. && e.max.x <= width / scale + 1.);
                assert!(m.output.is_some());
                if width == 760. {
                    assert!(!m.side_by_side);
                }
                let font = egui::TextStyle::Monospace.resolve(&ctx.style());
                let expected = ctx.fonts_mut(|f| f.row_height(&font)).ceil() * 15.
                    + 2. * f32::from(controls::PAD_Y);
                assert!((e.height() - expected).abs() <= 2.);
                let apply = m.controls.iter().find(|c| c.0 == "apply").unwrap().1;
                let control = ctx.style().spacing.interact_size.y;
                assert!(apply.height() >= control - 1.);
            }
        }
    }
}
#[test]
fn ldap_changes_editor_enter_is_text_and_tab_focuses_a_real_control() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, "editor", "version: 1");
    let id = a.metrics.ldap_changes.editor.unwrap().1;
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
    let State::LdapChanges { simulation } = &a.progress.attempts[IDS[0]].state else {
        panic!()
    };
    assert!(simulation.assignment.draft.ends_with('\n'));
    assert!(simulation.assignment.output.is_none());
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
    let focused = ctx.memory(|m| m.focused());
    assert!(focused.is_some());
    assert_ne!(focused, Some(id));
}

#[test]
fn ldap_changes_scrollbars_keep_editor_viewport_and_apply_button_in_place() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let id = a.scenarios[i].id.clone();
    let State::LdapChanges { simulation } = &mut a.progress.attempts.get_mut(&id).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    // Settle ordinary panel/scrollbar animations before measuring either state.
    for _ in 0..24 {
        frame(&ctx, &mut a, 1500., 4000., vec![]);
    }
    let viewport = a.metrics.ldap_changes.editor.unwrap().0;
    let apply = a
        .metrics
        .ldap_changes
        .controls
        .iter()
        .find(|c| c.0 == "apply")
        .unwrap()
        .1;
    let State::LdapChanges { simulation } = &mut a.progress.attempts.get_mut(&id).unwrap().state
    else {
        panic!()
    };
    simulation
        .assignment
        .edit(("# ".to_owned() + &"long value ".repeat(50) + "\n").repeat(25))
        .unwrap();
    for _ in 0..24 {
        frame(&ctx, &mut a, 1500., 4000., vec![]);
    }
    let after = a.metrics.ldap_changes.editor.unwrap().0;
    let after_apply = a
        .metrics
        .ldap_changes
        .controls
        .iter()
        .find(|c| c.0 == "apply")
        .unwrap()
        .1;
    assert_eq!(after, viewport);
    assert_eq!(after_apply, apply);
    let text_clip = a.metrics.ldap_changes.editor_text_clip.unwrap();
    assert!(text_clip.contains(viewport.center()));
    assert!(text_clip.width() < viewport.width());
}

fn opened_utility_fixture() -> (egui::Context, LabApp, usize) {
    use work::utility::{Credential, Draft, Utility};
    let (ctx, mut a, old) = opened(IDS[0]);
    let mut source = a.scenarios[old].clone();
    let mut d = work::source(&source).unwrap().clone();
    source.id = "ldap-utility-synthetic-fixture".into();
    d.initial.id = source.id.clone();
    d.example.id = "ldap-utility-synthetic-practice".into();
    for t in [&mut d.initial, &mut d.example] {
        let u = Utility {
            input_kind: Default::default(),
            uri: "ldap://directory.example.test".into(),
            input_file: "/work/people.ldif".into(),
            credentials: vec![Credential {
                id: "operator".into(),
                label: "Supplied import operator".into(),
                principal: t.grant.principal.clone(),
                sample_password: "Fictional-only-53".into(),
            }],
        };
        let command = format!(
            "ldapadd -x -W -H {} -D '{}' -f {}",
            u.uri, t.grant.principal, u.input_file
        );
        t.starter = Draft {
            credential: "operator".into(),
            command: command.replacen("ldapadd", "ldapmodify", 1),
            ldif: t.reference.clone(),
        }
        .encode()
        .unwrap();
        t.reference = Draft {
            credential: "operator".into(),
            command,
            ldif: t.reference.clone(),
        }
        .encode()
        .unwrap();
        t.utility = Some(u);
    }
    source.objective = d.initial.instructions.clone();
    source.prerequisites.clear();
    source.solution = vec![format!("{}{}", work::PREFIX, d.initial.reference)];
    source.initial = State::LdapChanges {
        simulation: Box::new(work::Session::new(&d).unwrap()),
    };
    source.goal = engine::Goal::LdapChanges {
        definition: Box::new(d),
    };
    a.scenarios.push(source);
    let i = a.scenarios.len() - 1;
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
#[test]
fn ldap_utility_native_command_edits_run_check_and_keyboard_invalidate_current_results() {
    let (ctx, mut a, i) = opened_utility_fixture();
    let id = a.scenarios[i].id.clone();
    let d = work::source(&a.scenarios[i]).unwrap().clone();
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "apply");
    let State::LdapChanges { simulation } = &a.progress.attempts[&id].state else {
        panic!()
    };
    assert!(!simulation.assignment.output.as_ref().unwrap().applied);
    assert!(!simulation.assignment.checked);
    let command = work::utility::Draft::parse(&d.initial.reference)
        .unwrap()
        .command;
    let r = target(&ctx, &mut a, "command");
    super::debian_contents::point_click(&ctx, &mut a, 1200., 950., r);
    let command_id = a.metrics.ldap_changes.command.unwrap().1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(command_id));
    frame(
        &ctx,
        &mut a,
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
            egui::Event::Paste(command.clone()),
        ],
    );
    let State::LdapChanges { simulation } = &a.progress.attempts[&id].state else {
        panic!()
    };
    let draft = work::utility::Draft::parse(&simulation.assignment.draft).unwrap();
    assert_eq!(draft.command, command);
    assert_eq!(
        draft.ldif,
        work::utility::Draft::parse(&d.initial.starter)
            .unwrap()
            .ldif
    );
    assert!(simulation.assignment.output.is_none());
    tap(&ctx, &mut a, "apply");
    let State::LdapChanges { simulation } = &a.progress.attempts[&id].state else {
        panic!()
    };
    assert!(simulation.assignment.output.as_ref().unwrap().applied);
    assert!(!simulation.assignment.checked);
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    let r = target(&ctx, &mut a, "command");
    super::debian_contents::point_click(&ctx, &mut a, 1200., 950., r);
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Text(" unsupported".into())],
    );
    assert!(!a.metrics.completion);
    assert!(a.metrics.ldap_changes.result.is_none());
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
    let focused = ctx.memory(|m| m.focused());
    assert!(focused.is_some());
    assert_ne!(focused, Some(command_id));
}
#[test]
fn ldap_utility_native_fields_reflow_at_five_scales_and_keep_populated_values_inside_columns() {
    let (ctx, mut a, i) = opened_utility_fixture();
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
                let State::LdapChanges { simulation } =
                    &mut a.progress.attempts.get_mut(&id).unwrap().state
                else {
                    panic!()
                };
                simulation.view = teaching::View::Assignment;
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                let m = &a.metrics.ldap_changes;
                for key in ["command", "credential", "editor", "apply"] {
                    let r = m.controls.iter().find(|c| c.0 == key).unwrap().1;
                    assert!(
                        r.min.x >= 0. && r.max.x <= width / scale + 1.,
                        "{key}: {r:?}, width={width} scale={scale}"
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
                if width == 760. {
                    assert!(!m.side_by_side);
                }
            }
        }
    }
}

#[test]
fn ldap_utility_native_short_editor_fits_its_reserved_viewport_at_three_scales() {
    let (ctx, mut a, i) = opened_utility_fixture();
    let id = a.scenarios[i].id.clone();
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        let State::LdapChanges { simulation } =
            &mut a.progress.attempts.get_mut(&id).unwrap().state
        else {
            panic!()
        };
        simulation.view = teaching::View::Assignment;
        let mut draft = work::utility::Draft::parse(&simulation.assignment.draft).unwrap();
        draft.ldif = "version: 1\n".into();
        simulation.assignment.edit(draft.encode().unwrap()).unwrap();
        for _ in 0..6 {
            frame(&ctx, &mut a, 1500., 4000., vec![]);
        }
        let m = &a.metrics.ldap_changes;
        let viewport = m.editor.unwrap().0;
        let edit = m.controls.iter().find(|c| c.0 == "editor").unwrap().1;
        assert!(
            edit.height() <= viewport.height() + 1.,
            "A short input must fit its actual reserved field: scale={scale}, text field={edit:?}, viewport={viewport:?}"
        );
    }
}
