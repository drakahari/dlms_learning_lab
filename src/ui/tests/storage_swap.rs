use super::*;
const IDS: [&str; 2] = [
    "lpic1-swap-prepared-activation",
    "lpic1-swap-replacement-handover",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn rect(a: &LabApp, key: &str) -> Option<egui::Rect> {
    smoke::swap_policy::control(a, key)
}

fn target(ctx: &egui::Context, a: &mut LabApp, key: &str) -> egui::Rect {
    let mut last = None;
    for _ in 0..32 {
        frame(ctx, a, 1200., 950., vec![]);
        let r = rect(a, key).unwrap_or_else(|| panic!("Missing {key}"));
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if last == Some((r, clip)) {
                return r;
            }
            last = Some((r, clip));
            frame(
                ctx,
                a,
                1200.,
                950.,
                vec![egui::Event::PointerMoved(r.center())],
            );
            continue;
        }
        last = None;
        frame(
            ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, 1200., 950., vec![]);
        }
    }
    panic!("Swap widget {key} did not become visible")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn draft(a: &LabApp) -> &dlms_learning_lab::storage_workspace::Draft {
    let i = a.selected.unwrap();
    let w = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap();
    if a.storage_view(i) == teaching::View::Example {
        &w.storage_learning.as_ref().unwrap().draft
    } else {
        w.storage_draft.as_ref().unwrap()
    }
}
fn edit(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = target(ctx, a, key);
    click(ctx, a, r);
    assert!(ctx.memory(|m| m.focused()).is_some());
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
    assert_eq!(
        if key == "editor" {
            &draft(a).text
        } else {
            &draft(a).command
        },
        value
    );
}
fn apply(ctx: &egui::Context, a: &mut LabApp, action: &str) {
    if let Some(raw) = action.strip_prefix("storage ") {
        let dlms_learning_lab::storage_workspace::Action::Save { text } =
            serde_json::from_str(raw).unwrap()
        else {
            panic!()
        };
        edit(ctx, a, "editor", &text);
        tap(ctx, a, "save");
    } else {
        edit(ctx, a, "command", action);
        tap(ctx, a, "run");
    }
}
#[test]
fn swap_native_assignment_controls_handle_partial_alternatives_current_invalidation_and_tab() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        tap(&ctx, &mut a, "assignment");
        let source = a.scenarios[i].clone();
        tap(&ctx, &mut a, "check");
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        for (n, command) in source.solution.iter().enumerate() {
            apply(&ctx, &mut a, command);
            if n == 0 {
                tap(&ctx, &mut a, "check");
                assert!(!progress::current_solved(&a.progress.attempts[id], &source));
            }
        }
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        tap(&ctx, &mut a, "fresh");
        assert!(draft(&a).output.contains("Expected automatic areas:"));
        assert!(draft(&a).output.contains("Actual active areas:"));
        assert!(!draft(&a).output.contains("Some("));
        apply(&ctx, &mut a, "swapoff /dev/vdc1");
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        apply(&ctx, &mut a, "swapon -a");
        tap(&ctx, &mut a, "check");
        assert!(progress::current_solved(&a.progress.attempts[id], &source));
        edit(&ctx, &mut a, "command", "swapon --show --bytes");
        let focused = ctx.memory(|m| m.focused());
        frame(&ctx, &mut a, 1200., 950., vec![key(egui::Key::Tab, false)]);
        assert_ne!(ctx.memory(|m| m.focused()), focused);
    }
}
#[test]
fn swap_native_worked_fill_and_actual_practice_run_never_touch_assignment_drafts() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        let original = a.progress.attempts[id].state.clone();
        edit(&ctx, &mut a, "editor", "# assignment notes\n");
        tap(&ctx, &mut a, "example");
        let before = a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .storage_learning
            .as_ref()
            .unwrap()
            .state
            .clone();
        tap(&ctx, &mut a, "fill");
        let learning = a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .storage_learning
            .as_ref()
            .unwrap();
        assert_eq!(learning.state, before);
        assert!(!learning.checked);
        assert_ne!(learning.draft.text, learning.state.configuration());
        let lesson = source.storage_teaching.as_ref().unwrap();
        for action in &lesson.example_solution {
            apply(&ctx, &mut a, action);
        }
        tap(&ctx, &mut a, "check");
        let learning = a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .storage_learning
            .as_ref()
            .unwrap();
        assert!(learning.checks(lesson).iter().all(|r| r.0));
        assert_eq!(a.progress.attempts[id].state, original);
        assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        for action in &lesson.try_actions {
            apply(&ctx, &mut a, action);
        }
        tap(&ctx, &mut a, "check");
        let learning = a.progress.attempts[id]
            .workspace
            .as_ref()
            .unwrap()
            .storage_learning
            .as_ref()
            .unwrap();
        assert!(learning.checks(lesson).iter().any(|r| !r.0));
        apply(&ctx, &mut a, &lesson.example_solution[0]);
        tap(&ctx, &mut a, "check");
        assert!(
            a.progress.attempts[id]
                .workspace
                .as_ref()
                .unwrap()
                .storage_learning
                .as_ref()
                .unwrap()
                .checks(lesson)
                .iter()
                .all(|r| r.0)
        );
        tap(&ctx, &mut a, "assignment");
        assert_eq!(draft(&a).text, "# assignment notes\n");
    }
}
#[test]
fn swap_native_fields_and_editor_reflow_at_every_supported_scale_light_dark() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        tap(&ctx, &mut a, "assignment");
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            for light in [false, true] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
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
                for width in [760., 1600.] {
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 950., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for r in a
                        .metrics
                        .storage_command
                        .iter()
                        .chain(a.metrics.editor_save.iter())
                        .chain(a.metrics.storage_run.iter())
                        .chain(a.metrics.storage_test.iter())
                        .chain(a.metrics.storage_validate.iter())
                    {
                        assert!(
                            r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                            "{id} scale{scale} width{width} {r:?} {clip:?}"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn swap_native_three_views_teach_and_show_unchecked_state_without_assignment_answers() {
    fn collect(s: &egui::Shape, t: &mut String) {
        match s {
            egui::Shape::Text(x) => {
                assert!(
                    !x.galley.job.justify,
                    "Stretched text: {}",
                    x.galley.job.text
                );
                t.push_str(&x.galley.job.text);
                t.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, t)
                }
            }
            _ => {}
        }
    }
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let source = a.scenarios[i].clone();
        let example_uuid = source
            .storage_teaching
            .as_ref()
            .unwrap()
            .fixture
            .inventory()
            .swap()
            .unwrap()
            .areas[0]
            .uuid
            .clone();
        for (view, button) in [
            (teaching::View::Introduction, "introduction"),
            (teaching::View::Example, "example"),
            (teaching::View::Assignment, "assignment"),
        ] {
            tap(&ctx, &mut a, button);
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1500., 7000.),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            );
            let mut t = String::new();
            for s in out.shapes {
                collect(&s.shape, &mut t);
            }
            match view {
                teaching::View::Introduction => {
                    if id == IDS[0] {
                        assert!(t.contains("Prepared, saved and active"));
                    } else {
                        assert!(t.contains("Read the handover as three checks"));
                        assert!(t.contains("saved file can be ahead"));
                    }
                    assert!(t.contains("Worked example"));
                    assert!(t.contains("none"));
                    assert!(t.contains("priority"));
                }
                teaching::View::Example => {
                    assert!(t.contains("Fill worked example"));
                    assert!(t.contains("Save configuration"));
                    assert!(t.contains("Check example"));
                    assert!(t.contains("Observe:"));
                }
                teaching::View::Assignment => {
                    assert!(t.contains("Requirements"));
                    assert!(t.contains("Supplied swap areas"));
                    assert!(t.contains("Check configuration"));
                    assert!(t.contains("Test saved policy"));
                    assert!(!t.contains(&example_uuid));
                    assert!(!t.contains("swapon -p "));
                    assert!(!t.contains("storage {"));
                }
            }
        }
    }
}

#[test]
fn swap_native_preview_target_follows_all_three_views_at_large_text() {
    let (ctx, mut a, i) = opened(IDS[0]);
    a.scale = 2.;
    ctx.set_zoom_factor(2.);
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        a.set_storage_view(i, view);
        for _ in 0..4 {
            frame(&ctx, &mut a, 1268., 632., vec![]);
        }
        assert!(a.preview_view_ready(i, view));
        assert_eq!(
            a.preview_action_rect(),
            if view == teaching::View::Introduction {
                a.metrics.example_action
            } else {
                a.metrics.check
            }
        );
        assert!(a.preview_action_rect().is_some());
    }
}
