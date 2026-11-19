//! Registered native schema page checks.
use super::*;
use dlms_learning_lab::ldap_schema_query_workspace as work;
const ID: &str = "lpic2-ldap-schema-consumer";
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
        .expect("Schema extension must be present in the normal review catalog");
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
                let (r, _, c) = a.metrics.ldap_schema_query.editor.unwrap();
                (r, c)
            }
            "challenge" => (a.metrics.mode_challenge.unwrap(), main),
            _ => {
                let (_, r, c) = a
                    .metrics
                    .ldap_schema_query
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
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key);
    super::debian_contents::point_click(ctx, a, 1200., 1000., r);
    let id = if key == "editor" {
        a.metrics.ldap_schema_query.editor.unwrap().1
    } else {
        a.metrics.ldap_schema_query.filter.unwrap().1
    };
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
fn consumer_worked_fill_and_real_run_remain_separate_from_assignment() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    tap(&ctx, &mut a, "example");
    tap(&ctx, &mut a, "worked");
    tap(&ctx, &mut a, "fill-example");
    let State::LdapSchemaQuery { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.practice.schema, d.example.reference_schema);
    assert_eq!(simulation.practice.filter, d.example.reference_filter);
    assert!(!simulation.practice.checked);
    assert!(simulation.practice.output.is_none());
    assert_eq!(simulation.assignment, work::Scope::new(&d.initial));
    tap(&ctx, &mut a, "run");
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ldap_schema_query.result.map(|r| (r.0, r.1)),
        Some((4, 4))
    );
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    tap(&ctx, &mut a, "challenge");
    assert!(
        !a.metrics
            .ldap_schema_query
            .controls
            .iter()
            .any(|c| matches!(c.0.as_str(), "worked" | "fill-example" | "introduction"))
    );
    let State::LdapSchemaQuery { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert_eq!(simulation.assignment, work::Scope::new(&d.initial));
}
#[test]
fn consumer_keyboard_edits_both_fields_and_invalidates_success_without_running() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, "editor", &d.initial.reference_schema);
    edit(&ctx, &mut a, "filter", &d.initial.reference_filter);
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    edit(&ctx, &mut a, "filter", "(accessTeam=*)");
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    let State::LdapSchemaQuery { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    assert!(simulation.assignment.output.is_none());
    assert!(!simulation.assignment.checked);
    assert_eq!(simulation.assignment.schema, d.initial.reference_schema);
    let r = target(&ctx, &mut a, "filter");
    super::debian_contents::point_click(&ctx, &mut a, 1200., 1000., r);
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
            modifiers: Default::default(),
        }],
    );
    assert_ne!(
        ctx.memory(|m| m.focused()),
        Some(a.metrics.ldap_schema_query.filter.unwrap().1)
    );
}
#[test]
fn consumer_partial_global_result_retains_the_passing_roster_and_correct_rule_counts() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, "editor", &d.initial.reference_schema);
    edit(&ctx, &mut a, "filter", "(|(cn=Iris Lane)(cn=Sam Lake))");
    tap(&ctx, &mut a, "check");
    assert_eq!(
        a.metrics.ldap_schema_query.result.map(|r| (r.0, r.1)),
        Some((1, 4))
    );
    assert_eq!(
        a.metrics.ldap_schema_query.case_results,
        vec![
            (true, true, 5, 5),
            (false, false, 5, 5),
            (false, false, 5, 5),
            (false, false, 5, 5)
        ]
    );
}
#[test]
fn consumer_fields_and_scrollable_editor_fit_two_widths_five_scales_light_and_dark() {
    let (ctx, mut a, i) = opened();
    let source = a.scenarios[i].clone();
    let d = work::source(&source).unwrap();
    let State::LdapSchemaQuery { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = teaching::View::Assignment;
    simulation
        .assignment
        .edit_schema(d.initial.reference_schema.clone())
        .unwrap();
    simulation
        .assignment
        .edit_filter(d.initial.reference_filter.clone())
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
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                let m = &a.metrics.ldap_schema_query;
                for key in ["editor", "case", "filter", "run", "check"] {
                    let (_, r, _) = m.controls.iter().find(|c| c.0 == key).unwrap();
                    assert!(
                        r.min.x >= 0. && r.max.x <= width / scale + 1.,
                        "{key} overflow at {scale}/{width}: {r:?}"
                    );
                }
                let field = m.filter.unwrap().0;
                let case = m.controls.iter().find(|c| c.0 == "case").unwrap().1;
                let run = m.controls.iter().find(|c| c.0 == "run").unwrap().1;
                assert!(
                    (field.width() - run.width()).abs() < 1.,
                    "Filter uses the whole action column: {field:?} vs {run:?}, scale {scale}, width {width}"
                );
                assert!(
                    (field.left() - run.left()).abs() < 1.,
                    "Filter and Run align"
                );
                assert!(
                    field.width() >= case.width(),
                    "Code input needs at least the dropdown value width"
                );
                assert!(
                    (field.height() - case.height()).abs() < 2.,
                    "Related single-line controls must share height"
                );
                assert_eq!(
                    m.editor_rows,
                    Some(d.initial.reference_schema.split('\n').count())
                );
                assert!(m.editor_text_clip.unwrap().width() <= m.editor.unwrap().0.width() + 1.);
                assert!(m.output.unwrap().max.x <= width / scale + 1.);
            }
        }
    }
}

#[test]
fn consumer_run_errors_are_painted_beside_the_input_and_clear_on_retry() {
    fn collect(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                out.push_str(t.galley.text());
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, out);
                }
            }
            _ => (),
        }
    }
    let (ctx, mut a, _) = opened();
    tap(&ctx, &mut a, "assignment");
    edit(&ctx, &mut a, "filter", "(accessTeam=A*)");
    tap(&ctx, &mut a, "run");
    let _ = target(&ctx, &mut a, "run-error");
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 1000.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut painted = String::new();
    for s in output.shapes {
        collect(&s.shape, &mut painted);
    }
    assert!(painted.contains("Input needs attention"));
    assert!(painted.contains("Unsupported"));
    assert!(painted.contains("Substring filters"));
    edit(&ctx, &mut a, "filter", "(accessTeam=Aster)");
    assert!(a.notice.is_none());
    assert!(
        !a.metrics
            .ldap_schema_query
            .controls
            .iter()
            .any(|c| c.0 == "run-error")
    );
}
