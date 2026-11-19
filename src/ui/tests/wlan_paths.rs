//! Actual normal426catalog/native AP-path interactions.
use super::*;
use dlms_learning_lab::{engine::Goal, wlan_paths_workspace as work};
const IDS: [&str; 2] = ["ccna-wlc-campus-forwarding", "ccna-flexconnect-local-vlan"];
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
    let State::WlanPaths { simulation } = &mut a.progress.attempts.get_mut(id).unwrap().state
    else {
        panic!("AP-path state")
    };
    simulation
}
fn definition(a: &LabApp) -> work::Definition {
    let Goal::WlanPaths { definition } = &a.scenarios[a.selected.unwrap()].goal else {
        panic!("AP-path goal")
    };
    (**definition).clone()
}
fn rect(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .wlan_paths
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}; notice {:?}", a.notice))
        .1
}
fn has(a: &LabApp, key: &str) -> bool {
    a.metrics.wlan_paths.controls.iter().any(|r| r.0 == key)
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
    panic!("WLAN control {key} did not become visible")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key);
    click(ctx, a, r);
    draw(ctx, a);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, field_key: &str, text: &str) {
    tap(ctx, a, field_key);
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
        .wlan_paths
        .choices
        .iter()
        .find(|r| r.0 == choice)
        .unwrap_or_else(|| panic!("Missing choice {choice}"))
        .1;
    click(ctx, a, r);
    draw(ctx, a);
}
#[test]
fn staged_real_lessons_have_taught_views_full_tasks_and_no_default_worked_answer() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(!has(&a, "check"));
        let source = &a.scenarios[a.selected.unwrap()];
        assert_eq!(
            lesson_presentation::entry(source).unwrap().concepts.len(),
            4
        );
        session(&mut a, &id).view = teaching::View::Example;
        draw(&ctx, &mut a);
        assert!(has(&a, "fill") && has(&a, "check") && has(&a, "topology"));
        session(&mut a, &id).view = teaching::View::Assignment;
        draw(&ctx, &mut a);
        assert!(has(&a, "check") && !has(&a, "fill"));
        assert!(!a.metrics.wlan_paths.worked_answer_visible);
        assert_eq!(has(&a, "native"), n == 1);
        assert_eq!(has(&a, "local-staff-wlan"), n == 1);
        assert_eq!(has(&a, "forwarding-staff-wlan"), n == 0);
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        session(&mut a, &id).view = teaching::View::Introduction;
        draw(&ctx, &mut a);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(has(&a, "check") && !has(&a, "fill"));
        assert!(!a.metrics.wlan_paths.worked_answer_visible);
    }
}
#[test]
fn actual_practice_fill_and_check_never_award_independent_credit_or_replace_its_draft() {
    for n in 0..2 {
        let (ctx, mut a, id) = opened(n);
        let def = definition(&a);
        let assignment = serde_json::to_vec(&session(&mut a, &id).assignment).unwrap();
        session(&mut a, &id).view = teaching::View::Example;
        draw(&ctx, &mut a);
        tap(&ctx, &mut a, "fill");
        assert!(!session(&mut a, &id).practice.checked());
        tap(&ctx, &mut a, "check");
        assert!(session(&mut a, &id).practice.solved(&def.example));
        assert_eq!(
            serde_json::to_vec(&session(&mut a, &id).assignment).unwrap(),
            assignment
        );
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[a.selected.unwrap()]
        ));
    }
}
#[test]
fn native_campus_mode_and_allowed_list_repair_then_breaking_edit_clears_success() {
    let (ctx, mut a, id) = opened(0);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    choose(&ctx, &mut a, "mode", "mode-Local");
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.wlan_paths.summary.as_deref(), Some("fail"));
    replace(
        &ctx,
        &mut a,
        "port-allowed-access-controller-port",
        "10,20,30",
    );
    assert_eq!(a.metrics.wlan_paths.summary.as_deref(), Some("unchecked"));
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert_eq!(a.metrics.wlan_paths.summary.as_deref(), Some("pass"));
    replace(&ctx, &mut a, "port-allowed-access-controller-port", "10,30");
    assert!(!a.metrics.completion);
    assert!(!session(&mut a, &id).assignment.checked());
    tap(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert_eq!(a.metrics.wlan_paths.summary.as_deref(), Some("fail"));
}
#[test]
fn native_branch_numeric_errors_remain_editable_and_the_complete_repair_passes() {
    let (ctx, mut a, id) = opened(1);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "native", "abc");
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.wlan_paths.summary.as_deref(), Some("fail"));
    assert_eq!(session(&mut a, &id).assignment.fields.native, "abc");
    let def = definition(&a);
    assert!(
        def.initial
            .checks(&session(&mut a, &id).assignment.fields)
            .iter()
            .any(|r| r.feedback.contains("whole VLAN"))
    );
    replace(&ctx, &mut a, "native", "40");
    choose(&ctx, &mut a, "mode", "mode-FlexConnect");
    replace(&ctx, &mut a, "local-staff-wlan", "50");
    replace(&ctx, &mut a, "port-allowed-access-ap-port", "40,50,60");
    tap(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    assert!(session(&mut a, &id).assignment.solved(&def.initial));
}
#[test]
fn shared_fields_port_columns_and_topology_stay_within_wide_narrow_enlarged_bounds() {
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
            for _ in 0..4 {
                frame(&ctx, &mut a, width, 1080., vec![]);
            }
            let main = a.metrics.main_viewport.unwrap();
            for (_, r, _, _) in &a.metrics.wlan_paths.fields {
                assert!(
                    r.left() >= main.left() - 1. && r.right() <= main.right() + 1.,
                    "{n}/{scale}/{width}: {r:?} outside {main:?}"
                );
                assert!(r.height() >= 30.);
            }
            assert!(!a.metrics.wlan_paths.worked_answer_visible);
        }
    }
}
#[test]
fn focused_numeric_input_accepts_keyboard_and_tab_without_executing_or_mutating_another_draft() {
    let (ctx, mut a, id) = opened(1);
    session(&mut a, &id).view = teaching::View::Assignment;
    draw(&ctx, &mut a);
    let practice = serde_json::to_vec(&session(&mut a, &id).practice).unwrap();
    replace(&ctx, &mut a, "native", "4");
    let field = a
        .metrics
        .wlan_paths
        .fields
        .iter()
        .find(|r| r.0 == "native")
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    assert_eq!(session(&mut a, &id).assignment.fields.native, "4");
    assert!(!session(&mut a, &id).assignment.checked());
    assert_eq!(
        serde_json::to_vec(&session(&mut a, &id).practice).unwrap(),
        practice
    );
}
