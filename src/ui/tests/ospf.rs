use super::*;
use dlms_learning_lab::ospf_workspace::Draft;

fn fixture(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.add_ospf_fixture();
    a.launch(i);
    a.set_ospf_view(i, view);
    (ctx, a, i)
}

#[test]
fn ospf_new_attempt_opens_task_router_but_resumed_selections_stay_intact() {
    for (name, router) in [
        ("ccna-ospf-router-identity", "Branch"),
        ("ccna-ospf-p2p-parameters", "Office"),
        ("ccna-ospf-directed-ecmp", "Ingress"),
        ("ccna-ospf-passive-handover", "Site"),
    ] {
        let (ctx, mut a) = app();
        let i = a.scenarios.iter().position(|s| s.id == name).unwrap();
        a.launch(i);
        a.set_ospf_view(i, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let d = a.progress.attempts[name]
            .workspace
            .as_ref()
            .unwrap()
            .ospf_draft
            .as_ref()
            .unwrap();
        assert_eq!(d.selected.router, router);
        assert!(
            a.metrics.ospf_fields[0]
                .0
                .starts_with(&format!("{router}:"))
        );
        let mut selected = d.selected.clone();
        let other = d
            .routers
            .iter()
            .find(|(id, _)| id.as_str() != router)
            .unwrap();
        selected.router = other.0.clone();
        selected.interface = other.1.interfaces.keys().next().unwrap().clone();
        a.progress
            .attempts
            .get_mut(name)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .ospf_draft
            .as_mut()
            .unwrap()
            .selected = selected.clone();
        let bytes = serde_json::to_vec(&a.progress).unwrap();
        a.progress = progress::decode(&bytes, &a.scenarios).unwrap();
        a.launch(i);
        assert_eq!(
            a.progress.attempts[name]
                .workspace
                .as_ref()
                .unwrap()
                .ospf_draft
                .as_ref()
                .unwrap()
                .selected,
            selected
        );
        a.reset_attempt(i);
        assert_eq!(
            a.progress.attempts[name]
                .workspace
                .as_ref()
                .unwrap()
                .ospf_draft
                .as_ref()
                .unwrap()
                .selected
                .router,
            router
        );
    }
}
fn select_core(a: &mut LabApp, i: usize) {
    let d = a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .ospf_draft
        .as_mut()
        .unwrap();
    d.selected.router = "Core".into();
    d.selected.interface = "branch-core".into();
}
fn replace(ctx: &egui::Context, a: &mut LabApp, name: &str, text: &str) {
    draw(ctx, a);
    let id = a
        .metrics
        .ospf_fields
        .iter()
        .find(|(k, _, _)| k == name)
        .unwrap()
        .2;
    a.focus_input = false;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    draw(ctx, a);
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
}
fn check(ctx: &egui::Context, a: &mut LabApp) {
    frame(ctx, a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    draw(ctx, a);
}

#[test]
fn ospf_native_router_and_interface_pickers_are_operable() {
    let (ctx, mut a, i) = fixture(teaching::View::Assignment);
    draw(&ctx, &mut a);
    let target = a.metrics.ospf_router.unwrap().center();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::PointerMoved(target)],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::PointerButton {
                pos: target,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos: target,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
    );
    draw(&ctx, &mut a);
    let rect = a
        .metrics
        .ospf_router_items
        .iter()
        .find(|x| x.0 == "Core")
        .expect("open native router menu")
        .1;
    click(&ctx, &mut a, rect);
    let s = &a.scenarios[i];
    assert_eq!(
        a.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .ospf_draft
            .as_ref()
            .unwrap()
            .selected
            .router,
        "Core"
    );
    accounts_ui_click(&ctx, &mut a, |m| m.ospf_interface);
    let rect = a
        .metrics
        .ospf_interface_items
        .iter()
        .find(|x| x.0 == "core-edge")
        .expect("open native interface menu")
        .1;
    click(&ctx, &mut a, rect);
    assert_eq!(
        a.progress.attempts[&a.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap()
            .ospf_draft
            .as_ref()
            .unwrap()
            .selected
            .interface,
        "core-edge"
    );
}

#[test]
fn ospf_passive_lan_long_values_fit_and_text_focus_survives_reflow() {
    for scale in [1., 1.5, 2.] {
        for width in [1200., 720.] {
            let (ctx, mut a, i) = fixture(teaching::View::Assignment);
            a.scale = scale;
            let d = a
                .progress
                .attempts
                .get_mut(&a.scenarios[i].id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap()
                .ospf_draft
                .as_mut()
                .unwrap();
            d.selected.router = "Edge".into();
            d.selected.interface = "lan".into();
            d.routers.get_mut("Edge").unwrap().router_id = "x".repeat(64);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 560., vec![]);
            }
            for rect in a
                .metrics
                .ospf_fields
                .iter()
                .map(|x| &x.1)
                .chain(a.metrics.ospf_toggles.iter().map(|x| &x.1))
                .chain(a.metrics.ospf_type.iter())
            {
                assert!(
                    rect.left() >= 0. && rect.right() <= ctx.content_rect().right() + 1.,
                    "{width}/{scale}: {rect:?}"
                );
            }
        }
    }
    let (ctx, mut a, i) = fixture(teaching::View::Assignment);
    select_core(&mut a, i);
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "Core:router-id", "unfinished");
    let id = a
        .metrics
        .ospf_fields
        .iter()
        .find(|x| x.0 == "Core:router-id")
        .unwrap()
        .2;
    for _ in 0..3 {
        frame(&ctx, &mut a, 720., 560., vec![]);
    }
    assert_eq!(
        a.metrics
            .ospf_fields
            .iter()
            .find(|x| x.0 == "Core:router-id")
            .unwrap()
            .2,
        id
    );
    assert_eq!(ctx.memory(|m| m.focused()), Some(id));
    frame(&ctx, &mut a, 720., 560., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(id));
    assert!(ctx.memory(|m| m.focused()).is_some());
}

#[test]
fn ospf_disabled_background_does_not_apply_or_take_focus_from_backup_dialog() {
    let (ctx, mut a, i) = fixture(teaching::View::Assignment);
    select_core(&mut a, i);
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "Core/branch-core:hello", "10");
    let before = a.progress.attempts[&a.scenarios[i].id].state.clone();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("synthetic.json");
    a.smoke = true;
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let export = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, export);
    assert!(a.data_tools.modal_open());
    a.focus_input = true;
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&a.scenarios[i].id].state, before);
    assert!(!a.checked && !a.metrics.completion);
    assert!(
        !a.metrics
            .ospf_fields
            .iter()
            .any(|x| Some(x.2) == ctx.memory(|m| m.focused()))
    );
}

#[test]
fn ospf_authored_tasks_keep_configuration_controls_in_width_and_help_optional() {
    for name in [
        "ccna-ospf-router-identity",
        "ccna-ospf-p2p-parameters",
        "ccna-ospf-directed-ecmp",
        "ccna-ospf-passive-handover",
    ] {
        for (width, height) in [(1200., 850.), (720., 560.)] {
            for scale in [1., 1.5, 2.] {
                for (view, challenge) in [
                    (teaching::View::Introduction, false),
                    (teaching::View::Example, false),
                    (teaching::View::Assignment, false),
                    (teaching::View::Assignment, true),
                ] {
                    let (ctx, mut a) = app();
                    let i = a.scenarios.iter().position(|s| s.id == name).unwrap();
                    a.launch(i);
                    a.set_ospf_view(i, view);
                    a.scale = scale;
                    a.progress.attempts.get_mut(name).unwrap().challenge = challenge;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, height, vec![]);
                    }
                    assert!(!a.metrics.help_visible);
                    if challenge {
                        assert!(a.metrics.lesson_tabs.is_empty());
                    }
                    for rect in a
                        .metrics
                        .ospf_fields
                        .iter()
                        .map(|x| &x.1)
                        .chain(a.metrics.ospf_toggles.iter().map(|x| &x.1))
                        .chain(a.metrics.ospf_type.iter())
                        .chain(a.metrics.ospf_diagram.iter())
                        .chain(a.metrics.check.iter())
                    {
                        assert!(
                            rect.left() >= 0. && rect.right() <= ctx.content_rect().right() + 1.,
                            "{name}/{width}/{scale}/{view:?}: {rect:?}"
                        );
                    }
                    if view != teaching::View::Introduction || challenge {
                        assert_eq!(a.metrics.ospf_fields.len(), 8);
                        assert!(a.metrics.check.is_some());
                    }
                }
            }
        }
    }
}

#[test]
fn ospf_native_reflows_without_solution_help_at_all_scales_and_widths() {
    for (width, height) in [(1200., 850.), (720., 560.)] {
        for scale in [1., 1.5, 2.] {
            for (view, challenge) in [
                (teaching::View::Introduction, false),
                (teaching::View::Example, false),
                (teaching::View::Assignment, false),
                (teaching::View::Assignment, true),
            ] {
                let (ctx, mut a, i) = fixture(view);
                a.scale = scale;
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = challenge;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                assert!(!a.metrics.help_visible);
                if challenge {
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
                for rect in a
                    .metrics
                    .ospf_fields
                    .iter()
                    .map(|x| &x.1)
                    .chain(a.metrics.ospf_router.iter())
                    .chain(a.metrics.ospf_interface.iter())
                    .chain(a.metrics.ospf_diagram.iter())
                    .chain(a.metrics.ospf_type.iter())
                    .chain(a.metrics.ospf_toggles.iter().map(|x| &x.1))
                    .chain(a.metrics.check.iter())
                {
                    assert!(
                        rect.left() >= 0. && rect.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale} {view:?} challenge={challenge}: {rect:?}; fields={:?}, router={:?}, interface={:?}, diagram={:?}, check={:?}",
                        a.metrics.ospf_fields,
                        a.metrics.ospf_router,
                        a.metrics.ospf_interface,
                        a.metrics.ospf_diagram,
                        a.metrics.check
                    );
                }
                if view != teaching::View::Introduction || challenge {
                    assert_eq!(a.metrics.ospf_fields.len(), 8);
                    assert!(a.metrics.check.is_some());
                }
            }
        }
    }
}

#[test]
fn ospf_native_pending_errors_partial_alternate_retry_resume_and_reset_are_fresh() {
    let (ctx, mut a, i) = fixture(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    select_core(&mut a, i);
    draw(&ctx, &mut a);
    let initial = a.progress.attempts[&s.id].state.clone();
    replace(&ctx, &mut a, "Core/branch-core:hello", "not a number");
    check(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&s.id].state, initial);
    let w = a.progress.attempts[&s.id].workspace.as_ref().unwrap();
    assert!(w.checked);
    assert!(
        w.ospf_draft
            .as_ref()
            .unwrap()
            .notice
            .as_ref()
            .unwrap()
            .contains("Core/branch-core")
    );
    assert!(!a.metrics.completion);
    replace(&ctx, &mut a, "Core/branch-core:hello", "10");
    check(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(a.metrics.completion);
    // Merely inspecting a different interface must retain current success.
    a.progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .ospf_draft
        .as_mut()
        .unwrap()
        .selected
        .interface = "core-edge".into();
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    select_core(&mut a, i);
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "Core/branch-core:cost", "23");
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    check(&ctx, &mut a);
    assert!(a.metrics.completion, "alternate positive cost is permitted");
    let encoded = serde_json::to_vec(&a.progress).unwrap();
    a.progress = progress::decode(&encoded, &a.scenarios).unwrap();
    a.launch(i);
    a.set_ospf_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    replace(&ctx, &mut a, "Core/branch-core:hello", "11");
    assert!(!a.metrics.completion);
    check(&ctx, &mut a);
    let outcomes = engine::checks(&a.progress.attempts[&s.id].state, &s.goal);
    assert!(outcomes.iter().any(|c| c.0) && outcomes.iter().any(|c| !c.0));
    replace(&ctx, &mut a, "Core/branch-core:hello", "10");
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.launch(0);
    let keep = serde_json::to_vec(&a.progress.attempts[&a.scenarios[0].id]).unwrap();
    a.launch(i);
    a.reset_attempt(i);
    assert_eq!(a.progress.attempts[&s.id].state, initial);
    assert_eq!(
        serde_json::to_vec(&a.progress.attempts[&a.scenarios[0].id]).unwrap(),
        keep
    );
}

#[test]
fn ospf_example_fill_is_unapplied_ungraded_and_retains_assignment_draft() {
    let (ctx, mut a, i) = fixture(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    select_core(&mut a, i);
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "Core:router-id", "unfinished assignment");
    let assignment = serde_json::to_vec(
        a.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .ospf_draft
            .as_ref()
            .unwrap(),
    )
    .unwrap();
    let assignment_id = a
        .metrics
        .ospf_fields
        .iter()
        .find(|x| x.0 == "Core:router-id")
        .unwrap()
        .2;
    a.set_ospf_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.insert);
    let lesson = s.ospf_teaching.as_ref().unwrap();
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .ospf_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.simulation, lesson.fixture);
    assert!(!l.checked);
    assert!(!l.draft.matches(&l.simulation));
    assert!(!a.metrics.ospf_fields.iter().any(|x| x.2 == assignment_id));
    check(&ctx, &mut a);
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .ospf_learning
        .as_ref()
        .unwrap();
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    assert_eq!(
        serde_json::to_vec(
            a.progress.attempts[&s.id]
                .workspace
                .as_ref()
                .unwrap()
                .ospf_draft
                .as_ref()
                .unwrap()
        )
        .unwrap(),
        assignment
    );
    a.set_ospf_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(a.metrics.ospf_fields.iter().any(|x| x.2 == assignment_id));
    let State::Ospf { simulation } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    assert!(Draft::new(simulation).unwrap().matches(simulation));
}
