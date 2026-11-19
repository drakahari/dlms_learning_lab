//! Real controls over the registered review unit; reuse the HSRP input guards.
use super::*;
use dlms_learning_lab::hsrp as core;
const IDS: [&str; 4] = [
    "ccna-hsrp-virtual-gateway",
    "ccna-hsrp-incumbent-return",
    "ccna-hsrp-wan-tracking",
    "ccna-hsrp-resilient-handover",
];
fn opened_gateway(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn apply_action(ctx: &egui::Context, a: &mut LabApp, action: &work::Action) {
    match action {
        work::Action::Gateway { client, gateway } => text(
            ctx,
            a,
            &format!("client-{client}"),
            &gateway.to_string(),
            1200.,
            929.,
        ),
        work::Action::Router { router, config } => {
            let index = usize::from(*router == core::RouterId::B);
            let old = session(a).draft.routers[index].clone();
            if old.participating != config.participating {
                tap(ctx, a, &format!("router-{router:?}-participating"));
            }
            text(
                ctx,
                a,
                &format!("router-{router:?}-priority"),
                &config.priority.to_string(),
                1200.,
                929.,
            );
            if old.preempt != config.preempt {
                tap(ctx, a, &format!("router-{router:?}-preempt"));
            }
            if old.track != config.track_wan_decrement.is_some() {
                tap(ctx, a, &format!("router-{router:?}-track"));
            }
            if let Some(n) = config.track_wan_decrement {
                text(
                    ctx,
                    a,
                    &format!("router-{router:?}-decrement"),
                    &n.to_string(),
                    1200.,
                    929.,
                );
            }
        }
    }
    tap(ctx, a, "apply");
}
#[test]
fn registered_gateway_batch_real_controls_grade_partial_success_damage_retry_and_save_resume() {
    for id in IDS {
        let (ctx, mut a, i) = opened_gateway(id);
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.hsrp.summary.unwrap().1 > 0, "{id}");
        let source = a.scenarios[i].clone();
        let actions = source
            .solution
            .iter()
            .map(|s| {
                serde_json::from_str::<work::Action>(s.strip_prefix(work::PREFIX).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();
        for action in &actions {
            apply_action(&ctx, &mut a, action);
        }
        tap(&ctx, &mut a, "check");
        assert_eq!(a.metrics.hsrp.summary.unwrap().1, 0, "{id} {:?}", a.notice);
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        let before = portable(&a);
        a.save();
        assert!(a.storage_error.is_none(), "{:?}", a.storage_error);
        let reopened = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&reopened).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        assert_eq!(portable(&a), before);
        let Goal::Hsrp { definition } = &source.goal else {
            panic!()
        };
        let damage = if let Some(client) = definition.policy.editable_clients.first() {
            work::Action::Gateway {
                client: *client,
                gateway: definition.policy.initial.router(core::RouterId::A).address,
            }
        } else {
            let mut config = session(&a).current.router(core::RouterId::A).config.clone();
            config.preempt = false;
            work::Action::Router {
                router: core::RouterId::A,
                config,
            }
        };
        apply_action(&ctx, &mut a, &damage);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "check");
        assert!(a.metrics.hsrp.summary.unwrap().1 > 0);
        assert!(
            crate::ui::hsrp_workspace::coaching(&definition.policy, &session(&a).current).is_some()
        );
        for action in &actions {
            apply_action(&ctx, &mut a, action);
        }
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
    }
}
#[test]
fn registered_gateway_practice_is_separate_and_draft_edits_clear_success_before_apply() {
    let (ctx, mut a, i) = opened_gateway(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    let source = a.scenarios[i].clone();
    for cmd in &source.solution {
        engine::execute_for_goal(
            &mut a.progress.attempts.get_mut(&source.id).unwrap().state,
            &source.goal,
            cmd,
        )
        .unwrap();
    }
    tap(&ctx, &mut a, "check");
    assert!(progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
    let saved = session(&a).clone();
    tap(&ctx, &mut a, "example");
    for n in 0..3 {
        // The example picker is source-owned and setting insertion remains an inert draft edit.
        work::binding::change(
            a.progress.attempts.get_mut(&source.id).unwrap(),
            &source,
            work::binding::Change::SelectExample(n),
        )
        .unwrap();
        draw(&ctx, &mut a);
        tap(&ctx, &mut a, "insert");
        tap(&ctx, &mut a, "apply");
    }
    tap(&ctx, &mut a, "check");
    assert_eq!(a.metrics.hsrp.summary.unwrap().1, 0);
    assert_eq!(session(&a).current, saved.current);
    assert_eq!(session(&a).history, saved.history);
    assert_eq!(session(&a).draft, saved.draft);
    tap(&ctx, &mut a, "assignment");
    text(&ctx, &mut a, "client-1", "192.", 1200., 929.);
    assert!(!progress::current_solved(
        &a.progress.attempts[&source.id],
        &source
    ));
    assert_eq!(session(&a).current, saved.current);
    tap(&ctx, &mut a, "apply");
    assert!(
        a.notice
            .as_ref()
            .is_some_and(|s| s.contains("gateway") || s.contains("IPv4"))
    );
    assert_eq!(session(&a).current, saved.current);
}
#[test]
fn registered_gateway_all_three_views_use_specific_teaching_and_preserve_assignment_visibility() {
    fn text_shape(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    text_shape(s, out)
                }
            }
            _ => {}
        }
    }
    for id in IDS {
        let (ctx, mut a, i) = opened_gateway(id);
        let source = a.scenarios[i].clone();
        for (view, key) in [
            (teaching::View::Introduction, "introduction"),
            (teaching::View::Example, "example"),
            (teaching::View::Assignment, "assignment"),
        ] {
            work::binding::change(
                a.progress.attempts.get_mut(id).unwrap(),
                &source,
                work::binding::Change::View(view),
            )
            .unwrap();
            let mut painted = String::new();
            for _ in 0..4 {
                let o = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1500., 6000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                painted.clear();
                for s in o.shapes {
                    text_shape(&s.shape, &mut painted);
                }
            }
            match key {
                "introduction" => {
                    assert!(painted.contains("WHAT YOU’LL LEARN"));
                    assert!(painted.contains("Worked example"));
                    assert!(
                        painted.contains(
                            &lesson_presentation::entry(&source).unwrap().worked[0].value
                        )
                    );
                }
                "example" => {
                    assert!(painted.contains("Inspect the practice network"));
                    assert!(painted.contains("Apply and verify each setting"));
                    assert!(painted.contains("Not checked."));
                }
                _ => {
                    assert!(painted.contains("Requirements"));
                    assert!(!painted.contains("Worked example"));
                    assert!(!painted.contains("SIM HSRP"));
                }
            }
            assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        }
    }
}

#[test]
fn registered_gateway_failed_return_explains_the_mistake_without_default_rust_fault_records() {
    fn painted(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for shape in v {
                    painted(shape, text);
                }
            }
            _ => {}
        }
    }
    let (ctx, mut app, i) = opened_gateway("ccna-hsrp-incumbent-return");
    let source = app.scenarios[i].clone();
    for command in &source.solution {
        engine::execute_for_goal(
            &mut app.progress.attempts.get_mut(&source.id).unwrap().state,
            &source.goal,
            command,
        )
        .unwrap();
    }
    let Goal::Hsrp { definition } = &source.goal else {
        panic!()
    };
    let mut config = session(&app)
        .current
        .router(core::RouterId::A)
        .config
        .clone();
    config.preempt = false;
    let a = app.progress.attempts.get_mut(&source.id).unwrap();
    let State::Hsrp { simulation } = &mut a.state else {
        panic!()
    };
    simulation
        .apply_action(
            definition,
            &work::Action::Router {
                router: core::RouterId::A,
                config,
            },
            false,
        )
        .unwrap();
    for request in [
        work::binding::Change::View(teaching::View::Assignment),
        work::binding::Change::Check { practice: false },
    ] {
        work::binding::change(a, &source, request).unwrap();
    }
    let mut text = String::new();
    for _ in 0..4 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 5000.),
                )),
                ..Default::default()
            },
            |ctx| app.render(ctx),
        );
        text.clear();
        for shape in output.shapes {
            painted(&shape.shape, &mut text);
        }
    }
    assert!(text.contains("A power failure and return"));
    assert!(text.contains("the router that should take over needs its own Preempt setting"));
    assert!(!text.contains("Power {"));
    assert!(!text.contains("settled Active Some("));
    assert_eq!(app.metrics.hsrp.summary.unwrap().1, 1);
    assert!(!progress::current_solved(
        &app.progress.attempts[&source.id],
        &source
    ));
}
