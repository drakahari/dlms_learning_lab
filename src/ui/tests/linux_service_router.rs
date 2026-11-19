//! Native widgets with private synthetic progress; not a novice/device study.
use super::*;
use dlms_learning_lab::{linux_service_router as work, linux_service_router_source as binding};
const ID: &str = "lpic2-router-service-name-recovery";
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1450., 9000., events);
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
    let State::LinuxServiceRouter { simulation } =
        &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::LinuxServiceRouter { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .services_router
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
        .services_router
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

#[test]
fn services_router_example_file_fill_and_command_insert_are_inert_and_isolated() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let assignment = session(&a).assignment.clone();
    hit(&ctx, &mut a, "worked-file-toggle");
    for _ in 0..15 {
        paint(&ctx, &mut a, vec![]);
    }
    hit(&ctx, &mut a, "fill-file");
    let d = binding::source(&a.scenarios[i]).unwrap();
    assert_eq!(session(&a).practice.editor, d.example.required_services);
    assert_eq!(
        session(&a).practice.saved_services,
        d.example.initial_services
    );
    assert!(session(&a).practice.router.output.is_empty());
    assert_eq!(session(&a).assignment, assignment);
    assert!(!a.metrics.completion);
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "insert-command");
    assert!(!session(&a).practice.router.input.is_empty());
    assert!(session(&a).practice.router.output.is_empty());
    edit(&ctx, &mut a, "command", "getent services previews/tcp");
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
    assert!(session(&a).practice.router.output.contains("9443/tcp"));
    assert_eq!(session(&a).assignment, assignment);
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    for key in [
        "example",
        "introduction",
        "insert-command",
        "worked",
        "fill-file",
        "worked-file-toggle",
    ] {
        assert!(
            !a.metrics
                .services_router
                .controls
                .iter()
                .any(|r| r.0 == key)
        );
    }
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn services_router_failed_save_is_local_and_retry_clears_stale_feedback() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let saved = session(&a).assignment.saved_services.clone();
    edit(
        &ctx,
        &mut a,
        "editor",
        "report not-a-port/tcp reports\nclock 123/udp time\n",
    );
    hit(&ctx, &mut a, "save");
    assert_eq!(session(&a).assignment.saved_services, saved);
    assert!(
        a.metrics
            .services_router
            .controls
            .iter()
            .any(|r| r.0 == "save-error")
    );
    let key = egui::Id::new(("router-services-save-error", ID, false));
    let (_, error) = ctx.data(|d| d.get_temp::<(String, String)>(key)).unwrap();
    assert!(error.contains("File not saved"));
    assert!(error.contains("Line 1"));
    let correct = binding::source(&a.scenarios[i])
        .unwrap()
        .assignment
        .required_services
        .clone();
    edit(&ctx, &mut a, "editor", &correct);
    assert!(
        !a.metrics
            .services_router
            .controls
            .iter()
            .any(|r| r.0 == "save-error")
    );
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    assert!(
        a.metrics
            .services_router
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
    hit(&ctx, &mut a, "save");
    assert_eq!(session(&a).assignment.saved_services, correct);
    hit(&ctx, &mut a, "check");
    assert!(
        !a.metrics.completion,
        "Saving alone cannot repair installed numbers."
    );
}
#[test]
fn services_router_native_edit_save_run_check_and_invalidation_work_end_to_end() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    for raw in &source.solution {
        let action: work::Action =
            serde_json::from_str(raw.strip_prefix(binding::PREFIX).unwrap()).unwrap();
        match action {
            work::Action::Edit { text } => edit(&ctx, &mut a, "editor", &text),
            work::Action::Save => hit(&ctx, &mut a, "save"),
            work::Action::Router {
                action: dlms_learning_lab::linux_v6_workspace::Action::Command { line },
            } => {
                edit(&ctx, &mut a, "command", &line);
                hit(&ctx, &mut a, "run");
            }
            work::Action::Router {
                action: dlms_learning_lab::linux_v6_workspace::Action::Check,
            } => hit(&ctx, &mut a, "check"),
            _ => panic!("Unexpected authored recipe action"),
        }
    }
    assert!(a.metrics.completion);
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    hit(&ctx, &mut a, "run-traffic");
    assert!(a.metrics.completion);
    assert!(session(&a).assignment.router.last_trial.is_some());
    let comments = format!(
        "# Keep equivalent meanings\n{}",
        session(&a).assignment.editor
    );
    edit(&ctx, &mut a, "editor", &comments);
    assert!(!a.metrics.completion);
    assert!(session(&a).assignment.router.last_trial.is_none());
    hit(&ctx, &mut a, "save");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    hit(&ctx, &mut a, "command");
    let before = session(&a).assignment.clone();
    for key in ["run", "case"] {
        paint(
            &ctx,
            &mut a,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        settle(&ctx, &mut a);
        let expected = a
            .metrics
            .services_router
            .focus
            .iter()
            .find(|r| r.0 == key)
            .unwrap()
            .1;
        assert_eq!(ctx.memory(|m| m.focused()), Some(expected));
        assert_eq!(session(&a).assignment, before);
    }
}
#[test]
fn services_router_save_footer_and_controls_fit_scales_widths_and_themes() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    let source = a.scenarios[i].clone();
    let definition = binding::source(&source).unwrap();
    for width in [1450., 760.] {
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                ctx.set_zoom_factor(scale);
                a.scale = scale;
                let selected = theme::inventory()
                    .iter()
                    .find(|t| t.light == light)
                    .unwrap()
                    .id
                    .clone();
                theme::apply(&ctx, &selected);
                for long in [false, true] {
                    let State::LinuxServiceRouter { simulation } =
                        &mut a.progress.attempts.get_mut(ID).unwrap().state
                    else {
                        panic!()
                    };
                    simulation.action(definition,&work::Action::Edit{text:if long{format!("{}{}",definition.assignment.initial_services,"# A long explanatory comment with spaces that stays inside the virtual editor\n".repeat(80))}else{definition.assignment.initial_services.clone()}},false).unwrap();
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 9000., vec![]);
                    }
                    for key in ["save", "command", "run", "case", "run-traffic", "check"] {
                        let (_, r, clip) = a
                            .metrics
                            .services_router
                            .controls
                            .iter()
                            .find(|r| r.0 == key)
                            .unwrap();
                        assert!(
                            r.left() >= clip.left() - 1.,
                            "{width}/{scale}/{light}/{key}"
                        );
                        assert!(
                            r.right() <= ctx.content_rect().right() + 1.,
                            "{width}/{scale}/{light}/{key}: {r:?}"
                        );
                        assert!(r.height() >= 20.);
                    }
                    let save = a
                        .metrics
                        .services_router
                        .controls
                        .iter()
                        .find(|r| r.0 == "save")
                        .unwrap()
                        .1;
                    let marker = egui::Id::new((
                        "services-footer-baseline",
                        width as u32,
                        (scale * 100.) as u32,
                        light,
                    ));
                    if long {
                        let before = ctx.data(|d| d.get_temp::<egui::Rect>(marker)).unwrap();
                        assert!(
                            (save.top() - before.top()).abs() < 1.,
                            "Scrollbar/content must not shift Save: {save:?} versus {before:?}"
                        );
                    } else {
                        ctx.data_mut(|d| d.insert_temp(marker, save));
                    }
                }
                assert_eq!(ctx.zoom_factor(), scale);
            }
        }
    }
}
