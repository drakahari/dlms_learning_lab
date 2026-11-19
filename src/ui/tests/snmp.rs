//! Actual normal430 lesson definitions and native widgets; no profile/host activity.
use super::*;
use dlms_learning_lab::snmp_workspace as work;
const IDS: [&str; 2] = ["ccna-snmp-link-monitor", "ccna-snmp-counter-monitor"];
fn opened(n: usize) -> (egui::Context, LabApp, String) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    assert!(a._smoke_profile.is_some());
    assert_eq!((a.scenarios.len(), a.units.len()), (439, 172));
    a.launch(a.scenarios.iter().position(|s| s.id == IDS[n]).unwrap());
    draw(&ctx, &mut a);
    (ctx, a, IDS[n].into())
}
fn session<'a>(a: &'a mut LabApp, id: &str) -> &'a mut work::Session {
    let State::SnmpMonitor { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!("MIB state")
    };
    simulation
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .snmp
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; notice {:?}", a.notice))
        .1
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    for _ in 0..20 {
        let r = rect(a, key);
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if r.top() >= viewport.top() && r.bottom() <= viewport.bottom() {
            return r;
        }
        let delta = if r.bottom() > viewport.bottom() {
            viewport.bottom() - r.bottom()
        } else {
            viewport.top() - r.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(viewport.right() - 3., viewport.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            draw(ctx, a);
        }
    }
    panic!("Monitor control {key} did not become visible")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key);
    click(ctx, a, r);
    draw(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key_name: &str, text: &str) {
    tap(ctx, a, key_name);
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            },
            key(egui::Key::Delete, false),
            egui::Event::Text(text.into()),
        ],
    );
    draw(ctx, a);
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, choice: &str) {
    tap(ctx, a, key);
    draw(ctx, a);
    let r = a
        .metrics
        .snmp
        .choices
        .iter()
        .find(|r| r.0 == choice)
        .unwrap_or_else(|| panic!("Missing choice {choice}"))
        .1;
    click(ctx, a, r);
    draw(ctx, a);
}
#[test]
fn registered_lessons_have_taught_views_full_independent_tasks_and_no_default_solutions() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        let s = &a.scenarios[a.selected.unwrap()];
        assert_eq!(lesson_presentation::entry(s).unwrap().concepts.len(), 4);
        assert!(!a.metrics.snmp.controls.iter().any(|r| r.0 == "check"));
        session(&mut a, &id).view = teaching::View::Example;
        draw(&ctx, &mut a);
        assert!(a.metrics.snmp.controls.iter().any(|r| r.0 == "fill"));
        assert!(a.metrics.snmp.controls.iter().any(|r| r.0 == "poll"));
        session(&mut a, &id).view = teaching::View::Assignment;
        draw(&ctx, &mut a);
        assert!(a.metrics.snmp.controls.iter().any(|r| r.0 == "check"));
        assert!(!a.metrics.snmp.controls.iter().any(|r| r.0 == "fill"));
        assert!(!a.metrics.snmp.worked_answer_visible);
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        session(&mut a, &id).view = teaching::View::Introduction;
        draw(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.snmp.worked_answer_visible);
        assert!(a.metrics.snmp.controls.iter().any(|r| r.0 == "poll"));
    }
}
#[test]
fn practice_presets_poll_real_values_and_filling_does_not_execute_or_award_assignment_credit() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        let before = serde_json::to_value(&session(&mut a, &id).assignment).unwrap();
        session(&mut a, &id).view = teaching::View::Example;
        draw(&ctx, &mut a);
        tap(&ctx, &mut a, "names");
        assert!(a.metrics.snmp.query_text.is_none());
        tap(&ctx, &mut a, "poll");
        assert!(
            a.metrics
                .snmp
                .query_text
                .as_ref()
                .unwrap()
                .contains(if n == 0 {
                    "library-uplink"
                } else {
                    "workshop-uplink"
                })
        );
        tap(&ctx, &mut a, "fill");
        assert!(!session(&mut a, &id).practice.checked());
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.snmp.summary.as_deref(), Some("pass"));
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[a.selected.unwrap()]
        ));
        assert_eq!(
            serde_json::to_value(&session(&mut a, &id).assignment).unwrap(),
            before
        );
    }
}
#[test]
fn actual_link_fields_report_maintenance_failure_then_success_and_invalidation() {
    let (ctx, mut a, id) = opened(0);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "description", "staff-uplink");
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.snmp.summary.as_deref(), Some("fail"));
    assert!(
        a.metrics
            .snmp
            .case_results
            .iter()
            .any(|r| r.0 == "unexpected-down" && r.1)
    );
    assert!(
        a.metrics
            .snmp
            .case_results
            .iter()
            .any(|r| r.0 == "maintenance" && !r.1)
    );
    choose(&ctx, &mut a, "rule", "rule-UnplannedDown");
    assert_eq!(a.metrics.snmp.summary.as_deref(), Some("unchecked"));
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert_eq!(a.metrics.snmp.summary.as_deref(), Some("pass"));
    replace(&ctx, &mut a, "description", "visitor-uplink");
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.snmp.summary.as_deref(), Some("fail"));
    assert!(!a.metrics.completion);
}
#[test]
fn actual_counter_fields_retain_wrong_input_accept_equivalent_threshold_and_do_not_trust_query_output()
 {
    let (ctx, mut a, id) = opened(1);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "threshold", "abc");
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.snmp.summary.as_deref(), Some("fail"));
    assert!(
        a.metrics
            .snmp
            .controls
            .iter()
            .any(|r| r.0 == "configuration-feedback")
    );
    assert_eq!(session(&mut a, &id).assignment.fields.threshold, "abc");
    choose(&ctx, &mut a, "direction", "direction-Incoming");
    choose(&ctx, &mut a, "width", "width-Counter64");
    replace(&ctx, &mut a, "threshold", "70");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    tap(&ctx, &mut a, "names");
    tap(&ctx, &mut a, "poll");
    assert!(a.metrics.completion);
    replace(&ctx, &mut a, "threshold", "99");
    assert!(!a.metrics.completion);
    tap(&ctx, &mut a, "check");
    assert!(
        a.metrics
            .snmp
            .case_results
            .iter()
            .any(|r| r.0 == "busy" && !r.1)
    );
    assert!(
        a.metrics
            .snmp
            .case_results
            .iter()
            .any(|r| r.0 == "ordinary" && r.1)
    );
}
#[test]
fn malformed_query_feedback_persists_after_repaint_and_editing_clears_it_before_retry() {
    let (ctx, mut a, id) = opened(0);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "query-oid", "not-an-oid");
    tap(&ctx, &mut a, "poll");
    draw(&ctx, &mut a);
    assert!(
        a.metrics
            .snmp
            .query_text
            .as_ref()
            .unwrap()
            .starts_with("Query error:")
    );
    tap(&ctx, &mut a, "names");
    assert!(a.metrics.snmp.query_text.is_none());
    tap(&ctx, &mut a, "poll");
    assert!(
        a.metrics
            .snmp
            .query_text
            .as_ref()
            .unwrap()
            .contains("staff-uplink")
    );
    assert!(!a.metrics.completion);
}
#[test]
fn focused_text_and_tab_do_not_run_queries_or_modify_the_other_draft() {
    let (ctx, mut a, id) = opened(1);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    let before = serde_json::to_value(&session(&mut a, &id).practice).unwrap();
    replace(&ctx, &mut a, "threshold", "7");
    let field = a
        .metrics
        .snmp
        .fields
        .iter()
        .find(|r| r.0 == "threshold")
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    assert!(!session(&mut a, &id).assignment.checked());
    assert!(a.metrics.snmp.query_text.is_none());
    assert_eq!(
        serde_json::to_value(&session(&mut a, &id).practice).unwrap(),
        before
    );
}
#[test]
fn shared_controls_reflow_and_keep_populated_and_placeholder_fields_within_enlarged_light_dark_bounds()
 {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        session(&mut a, &id).view = teaching::View::Assignment;
        for (theme_id, scale, width) in [
            ("lab-navy", 1., 1920.),
            ("lab-navy", 1.5, 1100.),
            ("light", 2., 946.),
        ] {
            a.theme_id = theme_id.into();
            theme::apply(&ctx, theme_id);
            a.scale = scale;
            session(&mut a, &id).assignment.fields.description.clear();
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 1080., vec![]);
            }
            let main = a.metrics.main_viewport.unwrap();
            for (_, r, _, _) in &a.metrics.snmp.fields {
                assert!(
                    r.left() >= main.left() - 1. && r.right() <= main.right() + 1.,
                    "{n}/{scale}/{width}: {r:?} outside {main:?}"
                );
                assert!(r.height() >= 30.);
            }
            assert!(!a.metrics.snmp.worked_answer_visible);
        }
    }
}
#[test]
fn failed_cases_explain_the_misconception_and_name_wrong_targets_without_failing_correct_cases() {
    let (ctx, mut a, id) = opened(0);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    tap(&ctx, &mut a, "check");
    let target = &a
        .metrics
        .snmp
        .case_feedback
        .iter()
        .find(|c| c.0 == "unexpected-down")
        .unwrap()
        .1;
    assert!(target.contains("visitor-uplink") && target.contains("staff-uplink"));
    replace(&ctx, &mut a, "description", "staff-uplink");
    tap(&ctx, &mut a, "check");
    let maintenance = &a
        .metrics
        .snmp
        .case_feedback
        .iter()
        .find(|c| c.0 == "maintenance")
        .unwrap()
        .1;
    assert!(
        maintenance.contains("administratively Down")
            && maintenance.contains("planned maintenance")
    );
    assert!(
        !a.metrics
            .snmp
            .case_feedback
            .iter()
            .any(|c| c.0 == "unexpected-down")
    );
    assert!(
        a.metrics
            .snmp
            .case_results
            .iter()
            .any(|c| c.0 == "unexpected-down" && c.1)
    );
}
