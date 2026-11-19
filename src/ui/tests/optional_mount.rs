use super::*;
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    for _ in 0..24 {
        // A cooperative job can enable this widget on the next paint.
        frame(ctx, a, 1200., 950., vec![]);
        let r = if field {
            a.metrics.optional_mount.editor.expect("editor").0
        } else {
            a.metrics
                .optional_mount
                .controls
                .iter()
                .find(|r| r.0 == key)
                .unwrap()
                .1
        };
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            return r;
        }
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
    panic!("Mount policy target never visible: {key}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = target(ctx, a, key, false);
    click(ctx, a, r);
    for _ in 0..3 {
        frame(ctx, a, 1200., 950., vec![]);
    }
}
fn field(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    let r = target(ctx, a, key, true);
    click(ctx, a, r);
    let id = a.metrics.optional_mount.editor.unwrap().1;
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
    let id = &a.scenarios[a.selected.unwrap()].id;
    let State::RemovableMounts { simulation } = &a.progress.attempts[id].state else {
        panic!()
    };
    let draft = if simulation.view == teaching::View::Example {
        &simulation.practice.draft
    } else {
        &simulation.assignment.draft
    };
    assert_eq!(
        draft, text,
        "Ctrl+A and paste must replace the whole multiline editor"
    );
}

const IDS: [&str; 1] = ["lpic1-removable-startup-policy"];
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..70 {
        frame(ctx, a, 1200., 950., vec![]);
        if a.optional_mount_run.is_none() {
            // Paint the now-enabled editor before sending a new pointer event.
            frame(ctx, a, 1200., 950., vec![]);
            return;
        }
    }
    panic!("bounded Mount policy run did not finish");
}
fn result(a: &LabApp) -> Option<(usize, usize)> {
    a.metrics.optional_mount.result.as_ref().map(|r| (r.0, r.1))
}
fn definition(s: &Scenario) -> &dlms_learning_lab::removable_workspace::Definition {
    let engine::Goal::RemovableMounts { definition } = &s.goal else {
        panic!()
    };
    definition
}
#[test]
fn optional_mount_actual_editor_wrong_correct_stale_retry_and_save_read() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert!(result(&a).unwrap().0 < 3);
        let raw = s.solution[0]
            .strip_prefix(dlms_learning_lab::removable_workspace::PREFIX)
            .unwrap();
        field(&ctx, &mut a, "editor", raw);
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((3, 3)));
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let restored = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(restored).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        field(&ctx, &mut a, "editor", "subnet {");
        assert!(result(&a).is_none());
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((0, 3)));
        field(&ctx, &mut a, "editor", raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((3, 3)));
    }
}
#[test]
fn optional_mount_worked_insert_is_inert_and_examples_remain_separate() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let d = definition(&s);
        tap(&ctx, &mut a, "example");
        tap(&ctx, &mut a, "insert");
        let State::RemovableMounts { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        assert_eq!(simulation.practice.draft, d.example_worked);
        assert!(simulation.practice.tested.is_none());
        assert_eq!(simulation.assignment.draft, d.initial.starter);
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((3, 3)));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
    }
}
#[test]
fn optional_mount_stop_keyboard_run_escape_and_navigation_cancel_inflight_work() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    let s = a.scenarios[i].clone();
    let raw = s.solution[0]
        .strip_prefix(dlms_learning_lab::removable_workspace::PREFIX)
        .unwrap();
    field(&ctx, &mut a, "editor", raw);
    let ctrl_enter = egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::CTRL,
    };
    frame(&ctx, &mut a, 1200., 950., vec![ctrl_enter]);
    assert!(a.optional_mount_run.is_some());
    let r = target(&ctx, &mut a, "stop", false);
    click(&ctx, &mut a, r);
    assert!(a.optional_mount_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[IDS[0]], &s));
    let r = target(&ctx, &mut a, "editor", true);
    click(&ctx, &mut a, r);
    let id = a.metrics.optional_mount.editor.unwrap().1;
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(!ctx.memory(|m| m.has_focus(id)));
    let run = target(&ctx, &mut a, "run", false);
    click(&ctx, &mut a, run);
    assert!(a.optional_mount_run.is_some());
    a.selected = None;
    frame(&ctx, &mut a, 1200., 950., vec![]);
    assert!(a.optional_mount_run.is_none());
    assert!(!progress::current_solved(&a.progress.attempts[IDS[0]], &s));
}
#[test]
fn optional_mount_editor_and_run_fit_supported_scales_light_dark_wide_narrow() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
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
                let r = a.metrics.optional_mount.editor.unwrap().0;
                assert!(
                    r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                    "scale{scale} width{width} {r:?} {clip:?}"
                );
                let button = a
                    .metrics
                    .optional_mount
                    .controls
                    .iter()
                    .find(|r| r.0 == "run")
                    .unwrap()
                    .1;
                assert!(button.left() >= clip.left() - 2. && button.right() <= clip.right() + 2.);
            }
        }
    }
}
fn painted(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn collect(shape: &egui::Shape, text: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                if t.galley.rows.len() > 1 && t.galley.job.text.len() > 80 {
                    assert!(!t.galley.job.justify);
                }
                text.push_str(&t.galley.job.text);
                text.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect(s, text)
                }
            }
            _ => {}
        }
    }
    let mut text = String::new();
    for _ in 0..4 {
        let out = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500., 6000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        text.clear();
        for s in out.shapes {
            collect(&s.shape, &mut text);
        }
    }
    text
}
#[test]
fn optional_mount_expected_required_failure_stays_visible_when_all_task_checks_pass() {
    let (ctx, mut a, i) = opened(IDS[0]);
    let source = a.scenarios[i].clone();
    tap(&ctx, &mut a, "assignment");
    let raw = source.solution[0]
        .strip_prefix(dlms_learning_lab::removable_workspace::PREFIX)
        .unwrap();
    field(&ctx, &mut a, "editor", raw);
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    tap(&ctx, &mut a, "case");
    tap(&ctx, &mut a, "case:2");
    assert_eq!(result(&a), Some((3, 3)));
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &source
    ));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("3/3 samples meet task requirements"));
    assert!(text.contains("Actual mount attempts · Required project filesystem absent"));
    assert!(text.contains("Failed ·"));
    assert!(text.contains("Required-storage result · a required mount failed"));
    assert!(!text.contains("startup succeeded"));
    let wrong = raw.replace("000000000091", "000000000099");
    field(&ctx, &mut a, "editor", &wrong);
    assert!(result(&a).is_none());
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &source
    ));
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((1, 3)));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Policy needs work"));
    assert!(text.contains("Requirement checks met · Removable audit medium absent"));
    assert!(text.contains("Requirements need work · Both media available"));
    assert!(text.contains("Not mounted"));
}
#[test]
fn optional_mount_three_views_teach_full_workshop_but_hide_assignment_answers() {
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        let (ctx, mut a, i) = opened(IDS[0]);
        let source = a.scenarios[i].clone();
        let State::RemovableMounts { simulation } =
            &mut a.progress.attempts.get_mut(IDS[0]).unwrap().state
        else {
            panic!()
        };
        simulation.view = view;
        let text = painted(&ctx, &mut a);
        let entry = lesson_presentation::entry(&source).unwrap();
        match view {
            teaching::View::Introduction => {
                assert!(text.contains(&entry.concepts[0].title));
                assert!(text.contains(&definition(&source).example_worked));
                assert!(text.contains("Worked example"));
            }
            teaching::View::Example => {
                assert!(text.contains(&entry.practice[0].title));
                assert!(text.contains("Insert worked policy"));
                assert!(text.contains("Not checked"));
            }
            teaching::View::Assignment => {
                assert!(text.contains("Requirements"));
                assert!(!text.contains("Insert worked policy"));
                assert!(!text.contains("Worked example"));
                assert!(!text.contains("ro,auto,nofail 0 0"));
            }
        }
    }
}
