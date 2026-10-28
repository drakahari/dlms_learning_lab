//! Real egui events on the four normally registered authored RPM activities.
use super::*;
use dlms_learning_lab::{
    rpm_packages_teaching as learn, rpm_packages_terminal as terminal, script_teaching::View,
};
const IDS: [&str; 4] = [
    "lpic1-rpm-provider",
    "lpic1-rpm-epoch",
    "lpic1-rpm-payload",
    "lpic1-rpm-handover",
];
fn lessons() -> Vec<Scenario> {
    static C: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        pack::load_catalog(&[
            include_str!("../../../scenarios/linux-files-v1.json"),
            include_str!("../../../scenarios/rpm-packages-v1.json"),
        ])
        .unwrap()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
    })
    .clone()
}
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (154, 39));
    for source in lessons() {
        let current = a.scenarios.iter().find(|s| s.id == source.id).unwrap();
        assert!(serde_json::to_value(current).unwrap() == serde_json::to_value(source).unwrap());
    }
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn chosen(a: &LabApp) -> &Attempt {
    &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id]
}
fn console(a: &LabApp, practice: bool) -> &terminal::Session {
    let w = chosen(a).workspace.as_ref().unwrap();
    if practice {
        &w.rpm_learning.as_ref().unwrap().terminal
    } else {
        w.rpm_session.as_ref().unwrap()
    }
}
fn learning(a: &LabApp) -> &learn::Learning {
    chosen(a)
        .workspace
        .as_ref()
        .unwrap()
        .rpm_learning
        .as_ref()
        .unwrap()
}
fn reveal(
    ctx: &egui::Context,
    a: &mut LabApp,
    w: f32,
    h: f32,
    get: impl Fn(&Metrics) -> Option<egui::Rect>,
) -> egui::Rect {
    for n in 0..=20 {
        let r = get(&a.metrics).expect("Actual RPM widget");
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if clip.contains(r.center()) && clip.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        assert!(n < 20, "RPM widget must be revealed {r:?} {clip:?}");
        let dy = if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom()
        } else {
            clip.top() - r.top()
        };
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy.clamp(-300., 300.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..60 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    unreachable!()
}
fn tap(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, r: egui::Rect) {
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(r.center()),
                egui::Event::PointerButton {
                    pos: r.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    for _ in 0..4 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn tab(ctx: &egui::Context, a: &mut LabApp, v: View) {
    let r = reveal(ctx, a, 1200., 850., |m| {
        m.rpm.tabs.iter().find(|r| r.0 == v).map(|r| r.1)
    });
    tap(ctx, a, 1200., 850., r);
    assert_eq!(learning(a).view, v);
}
fn type_command(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, text: &str) {
    let r = reveal(ctx, a, w, h, |m| m.rpm.command.map(|r| r.0));
    tap(ctx, a, w, h, r);
    let id = a.metrics.rpm.command.unwrap().1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        w,
        h,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    frame(ctx, a, w, h, vec![]);
}
fn submit(ctx: &egui::Context, a: &mut LabApp, text: &str, enter: bool) {
    type_command(ctx, a, 1200., 850., text);
    if enter {
        frame(ctx, a, 1200., 850., vec![key(egui::Key::Enter, false)]);
        frame(ctx, a, 1200., 850., vec![]);
    } else {
        let r = reveal(ctx, a, 1200., 850., |m| m.rpm.run.map(|r| r.0));
        tap(ctx, a, 1200., 850., r);
    }
}
#[test]
fn rpm_real_intro_insert_only_example_run_experiment_repair_and_assignment() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        assert_eq!(learning(&a).view, View::Introduction);
        let initial = serde_json::to_value(&chosen(&a).state).unwrap();
        tab(&ctx, &mut a, View::Example);
        let r = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.insert);
        tap(&ctx, &mut a, 1200., 850., r);
        assert!(!console(&a, true).command.is_empty());
        assert!(console(&a, true).entries.is_empty());
        let source = a.scenarios[i]
            .rpm_packages_teaching
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(learning(&a).simulation, source.definition.initial);
        for c in &source.example_commands {
            submit(&ctx, &mut a, &c.command, true);
        }
        assert!(learning(&a).checks(&source).iter().all(|r| r.0));
        assert!(serde_json::to_value(&chosen(&a).state).unwrap() == initial);
        for c in &source.try_change.commands {
            submit(&ctx, &mut a, &c.command, false);
        }
        assert_eq!(
            learning(&a)
                .checks(&source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            source.try_change.expected
        );
        for c in &source.repair {
            submit(&ctx, &mut a, &c.command, true);
        }
        assert!(learning(&a).checks(&source).iter().all(|r| r.0));
        tab(&ctx, &mut a, View::Assignment);
        assert!(console(&a, false).entries.is_empty());
        assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
    }
}
#[test]
fn rpm_actual_runs_checks_queries_unsupported_and_current_invalidation() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        tab(&ctx, &mut a, View::Assignment);
        let solution = a.scenarios[i].solution.clone();
        for c in solution {
            submit(&ctx, &mut a, &c, true);
        }
        assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
        let r = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.check);
        tap(&ctx, &mut a, 1200., 850., r);
        assert!(a.metrics.completion);
        let good = chosen(&a).state.clone();
        submit(&ctx, &mut a, "rpm -V anything", false);
        assert_eq!(chosen(&a).state, good);
        assert!(matches!(
            &console(&a, false).entries.last().unwrap().output,
            terminal::Outcome::SimulatorUnsupported { .. }
        ));
        assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
        let name = match &chosen(&a).state {
            engine::State::RpmPackages { simulation } => simulation
                .runtime()
                .installed
                .iter()
                .find(|r| r.identity.name.ends_with("-client"))
                .unwrap()
                .identity
                .name
                .clone(),
            _ => panic!(),
        };
        submit(&ctx, &mut a, &format!("rpm -q {name}"), true);
        assert_eq!(chosen(&a).state, good);
        let n = console(&a, false).entries.len();
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
        draw(&ctx, &mut a);
        assert_eq!(
            console(&a, false).entries.len(),
            n,
            "Check shortcut cannot also Run"
        );
        submit(&ctx, &mut a, &format!("rpm -e {name}"), true);
        assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
        assert!(!a.rpm_presentation.memo.prepared(id).unwrap());
    }
}
#[test]
fn rpm_real_modal_guards_and_oversized_replacement_keep_full_sessions() {
    for view in [View::Example, View::Assignment] {
        for modal in 0..4 {
            let (ctx, mut a, _) = opened(IDS[0]);
            tab(&ctx, &mut a, view);
            type_command(&ctx, &mut a, 1200., 850., "rpm -q ledger-client");
            let id = a.metrics.rpm.command.unwrap().1;
            assert!(ctx.memory(|m| m.has_focus(id)));
            let before = serde_json::to_value(chosen(&a)).unwrap();
            match modal {
                0 => a.reset_pending = true,
                1 => a.settings_open = true,
                2 => a.theme_preview = true,
                _ => {
                    a.smoke = true;
                    a.settings_open = true;
                    a.settings_data = true;
                    draw(&ctx, &mut a);
                    let r = a.metrics.data_export.unwrap();
                    tap(&ctx, &mut a, 1200., 850., r);
                    assert!(a.data_tools.modal_open());
                    a.settings_open = false;
                }
            }
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    egui::Event::Text("bad".into()),
                    key(egui::Key::Enter, false),
                    acl_ctrl(egui::Key::Enter),
                ],
            );
            draw(&ctx, &mut a);
            assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
            assert!(!a.metrics.rpm.command.unwrap().2);
            assert!(!a.metrics.rpm.run.unwrap().1);
            assert!(!a.metrics.rpm.check_enabled);
        }
    }
    let (ctx, mut a, _) = opened(IDS[0]);
    tab(&ctx, &mut a, View::Assignment);
    type_command(&ctx, &mut a, 1200., 850., "rpm -q ledger-client");
    let old = console(&a, false).clone();
    type_command(&ctx, &mut a, 1200., 850., &"x".repeat(2049));
    assert_eq!(console(&a, false), &old);
    type_command(&ctx, &mut a, 1200., 850., "rpm -q\0x");
    assert_eq!(console(&a, false), &old);
}
#[test]
fn rpm_two_raw_sessions_normal_save_readback_resume_and_selected_reset() {
    let (ctx, mut a, i) = opened(IDS[2]);
    tab(&ctx, &mut a, View::Example);
    submit(
        &ctx,
        &mut a,
        "rpm --reinstall /museum-rpms/example-client.rpm",
        true,
    );
    type_command(&ctx, &mut a, 1200., 850., "unfinished practice '");
    tab(&ctx, &mut a, View::Assignment);
    submit(&ctx, &mut a, "rpm --reinstall /rpms/client.rpm", true);
    type_command(&ctx, &mut a, 1200., 850., "unfinished assignment '");
    let id = a.scenarios[i].id.clone();
    a.progress.favorites.insert(id.clone());
    a.dirty = true;
    a.save();
    let before = serde_json::to_value(chosen(&a)).unwrap();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&loaded.attempts[&id]).unwrap() == before);
    let other = a.progress.attempts.clone();
    a.selected = None;
    a.launch(i);
    draw(&ctx, &mut a);
    assert!(console(&a, true).command.starts_with("unfinished practice"));
    assert!(
        console(&a, false)
            .command
            .starts_with("unfinished assignment")
    );
    let reset = reveal(&ctx, &mut a, 1200., 850., |m| m.reset);
    tap(&ctx, &mut a, 1200., 850., reset);
    let cancel = reveal(&ctx, &mut a, 1200., 850., |m| m.reset_cancel);
    tap(&ctx, &mut a, 1200., 850., cancel);
    assert_eq!(console(&a, false).command, "unfinished assignment '");
    let reset = reveal(&ctx, &mut a, 1200., 850., |m| m.reset);
    tap(&ctx, &mut a, 1200., 850., reset);
    let confirm = reveal(&ctx, &mut a, 1200., 850., |m| m.confirm_reset);
    tap(&ctx, &mut a, 1200., 850., confirm);
    assert!(console(&a, false).entries.is_empty());
    assert!(a.progress.favorites.contains(&id));
    for (k, v) in other {
        if k != id {
            assert!(
                serde_json::to_value(&a.progress.attempts[&k]).unwrap()
                    == serde_json::to_value(v).unwrap()
            );
        }
    }
}
#[test]
fn rpm_cache_library_paths_hit_and_same_id_foreign_context_cannot_keep_success() {
    let (ctx, mut a, i) = opened(IDS[0]);
    tab(&ctx, &mut a, View::Assignment);
    for c in a.scenarios[i].solution.clone() {
        submit(&ctx, &mut a, &c, true);
    }
    a.selected = None;
    draw(&ctx, &mut a);
    let before = a.rpm_presentation.memo.counters;
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    assert_eq!(
        a.rpm_presentation.memo.counters.evaluations,
        before.evaluations
    );
    assert_eq!(a.rpm_presentation.memo.counters.meters, before.meters);
    assert_eq!(a.rpm_presentation.memo.counters.snapshots, before.snapshots);
    let nat = pack::load(include_str!(
        "../../../tests/fixtures/nat-teaching-preview.json"
    ))
    .unwrap()
    .remove(0);
    let foreign =
        dlms_learning_lab::nat_teaching::Learning::new(nat.nat_teaching.as_ref().unwrap()).unwrap();
    a.progress
        .attempts
        .get_mut(IDS[0])
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .nat_learning = Some(foreign);
    draw(&ctx, &mut a);
    assert_eq!(a.rpm_presentation.memo.prepared(IDS[0]), Some(false));
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
}
#[test]
fn rpm_text_control_bounds_focus_undo_and_large_scale_reflow() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tab(&ctx, &mut a, View::Assignment);
    type_command(&ctx, &mut a, 1200., 850., "rpm -q ledger-client");
    let id = a.metrics.rpm.command.unwrap().1;
    for (width, height, scale, light) in [
        (1920., 1080., 1., true),
        (946., 929., 1.5, true),
        (946., 929., 2., false),
    ] {
        a.scale = scale;
        let theme = theme::inventory()
            .iter()
            .find(|t| t.light == light)
            .unwrap();
        a.theme_id = theme.id.clone();
        theme::apply(&ctx, &theme.id);
        assert_eq!(theme::find(&a.theme_id).light, light);
        frame(&ctx, &mut a, width, height, vec![]);
        let r = reveal(&ctx, &mut a, width, height, |m| m.rpm.command.map(|r| r.0));
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        assert!(clip.expand(1. / ctx.pixels_per_point()).contains_rect(r));
        assert_eq!(a.metrics.rpm.command.unwrap().1, id);
        tap(&ctx, &mut a, width, height, r);
        assert!(ctx.memory(|m| m.has_focus(id)));
        reveal(&ctx, &mut a, width, height, |m| m.rpm.run.map(|r| r.0));
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width / scale, height / scale),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        fn texts(shape: &egui::Shape, out: &mut Vec<egui::Rect>) {
            match shape {
                egui::Shape::Text(t) if t.galley.text() == "Run command" => {
                    out.push(egui::Rect::from_min_size(t.pos, t.galley.size()))
                }
                egui::Shape::Vec(v) => {
                    for s in v {
                        texts(s, out)
                    }
                }
                _ => (),
            }
        }
        let mut text = Vec::new();
        for shape in &output.shapes {
            texts(&shape.shape, &mut text);
        }
        assert_eq!(text.len(), 1);
        assert!(a.metrics.rpm.run.unwrap().0.contains_rect(text[0]));
    }
}

fn choose_inspector(
    ctx: &egui::Context,
    a: &mut LabApp,
    key: &str,
    get: impl Fn(&Metrics) -> Option<egui::Rect>,
) {
    let header = reveal(ctx, a, 1200., 850., get);
    tap(ctx, a, 1200., 850., header);
    for n in 0..=20 {
        let target = a
            .metrics
            .rpm
            .inspect_choices
            .iter()
            .find(|r| r.0 == key)
            .map(|r| r.1)
            .expect("Actual inspector choice");
        let clip = a
            .metrics
            .rpm
            .inspect_popup
            .expect("Actual inspector popup")
            .intersect(ctx.content_rect());
        if clip.contains(target.center())
            && clip
                .expand(1. / ctx.pixels_per_point())
                .contains_rect(target)
        {
            tap(ctx, a, 1200., 850., target);
            return;
        }
        assert!(n < 20, "Actual popup choice must be visible");
        let dy = if target.bottom() > clip.bottom() {
            clip.bottom() - target.bottom()
        } else {
            clip.top() - target.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(clip.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy.clamp(-200., 200.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..15 {
            draw(ctx, a);
        }
    }
}
#[test]
fn rpm_readonly_header_and_payload_inspector_observes_missing_then_actual_repair() {
    let (ctx, mut a, i) = opened(IDS[2]);
    tab(&ctx, &mut a, View::Assignment);
    let before = serde_json::to_value(chosen(&a)).unwrap();
    let open = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.inspector);
    tap(&ctx, &mut a, 1200., 850., open);
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    choose_inspector(&ctx, &mut a, "client", |m| m.rpm.inspect_header);
    choose_inspector(&ctx, &mut a, "/usr/share/archive-client/manual", |m| {
        m.rpm.inspect_path
    });
    assert!(
        a.metrics
            .rpm
            .inspection_text
            .iter()
            .any(|s| s == "Current virtual file is missing")
    );
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    submit(&ctx, &mut a, "rpm --reinstall /rpms/client.rpm", true);
    assert!(
        a.metrics
            .rpm
            .inspection_text
            .iter()
            .any(|s| s == "Current virtual bytes and attributes match this declared payload")
    );
    assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
}
#[test]
fn rpm_available_archive_inspector_labels_use_the_active_canonical_paths() {
    for view in [View::Assignment, View::Example] {
        let (ctx, mut a, i) = opened(IDS[2]);
        tab(&ctx, &mut a, view);
        let practice = view == View::Example;
        let definition = if practice {
            &a.scenarios[i]
                .rpm_packages_teaching
                .as_ref()
                .unwrap()
                .definition
        } else {
            let engine::Goal::RpmPackages { definition } = &a.scenarios[i].goal else {
                panic!()
            };
            definition.as_ref()
        };
        let state = if practice {
            &learning(&a).simulation
        } else {
            let engine::State::RpmPackages { simulation } = &chosen(&a).state else {
                panic!()
            };
            simulation
        };
        let expected = definition
            .archive_paths
            .iter()
            .filter(|(_, id)| state.runtime().available.contains(*id))
            .map(|(p, id)| (p.clone(), id.clone()))
            .collect::<Vec<_>>();
        assert!(!expected.is_empty());
        let before = serde_json::to_value(chosen(&a)).unwrap();
        let open = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.inspector);
        tap(&ctx, &mut a, 1200., 850., open);
        let available = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.inspect_available);
        tap(&ctx, &mut a, 1200., 850., available);
        let header = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.inspect_header);
        tap(&ctx, &mut a, 1200., 850., header);
        assert_eq!(
            a.metrics
                .rpm
                .inspect_choices
                .iter()
                .map(|(p, _)| p.clone())
                .collect::<Vec<_>>(),
            expected.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>()
        );
        assert!(
            a.metrics
                .rpm
                .inspect_choices
                .iter()
                .all(|(p, _)| p.starts_with('/') && p.ends_with(".rpm"))
        );
        // Close the popup, then choose a real canonical path through its actual existing widget.
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Escape, false)],
        );
        draw(&ctx, &mut a);
        choose_inspector(&ctx, &mut a, &expected[0].0, |m| m.rpm.inspect_header);
        assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    }
}

fn full_display_session(command: &str) -> terminal::Session {
    let mut s = terminal::Session {
        command: command.into(),
        entries: (0..16)
            .map(|_| terminal::Entry {
                command: "x".repeat(terminal::MAX_COMMAND),
                output: terminal::Outcome::Utility {
                    stdout: String::new(),
                    stderr: String::new(),
                    status: 0,
                    summary: None,
                },
            })
            .collect(),
        ..Default::default()
    };
    for n in 0..16 {
        let left = terminal::MAX_SESSION - serde_json::to_vec(&s).unwrap().len();
        if let terminal::Outcome::Utility { stdout, .. } = &mut s.entries[n].output {
            *stdout = "x".repeat(left.min(terminal::MAX_OUTPUT));
        }
    }
    assert_eq!(s.entries.len(), 16);
    assert_eq!(serde_json::to_vec(&s).unwrap().len(), terminal::MAX_SESSION);
    assert!(s.valid());
    s
}
#[test]
fn rpm_same_frame_rejected_aggregate_edit_cannot_run_old_draft_and_accepted_edit_does_run() {
    for view in [View::Assignment, View::Example] {
        for reject in [true, false] {
            let (ctx, mut a, _) = opened(IDS[2]);
            tab(&ctx, &mut a, view);
            let practice = view == View::Example;
            let old = if practice {
                "rpm --reinstall /museum-rpms/example-client.rpm"
            } else {
                "rpm --reinstall /rpms/client.rpm"
            };
            let session = full_display_session(old);
            let w = a
                .progress
                .attempts
                .get_mut(IDS[2])
                .unwrap()
                .workspace
                .as_mut()
                .unwrap();
            if practice {
                w.rpm_learning.as_mut().unwrap().terminal = session
            } else {
                w.rpm_session = Some(session)
            }
            draw(&ctx, &mut a);
            let input = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.command.map(|r| r.0));
            tap(&ctx, &mut a, 1200., 850., input);
            let before = serde_json::to_value(chosen(&a)).unwrap();
            let before_model = if practice {
                serde_json::to_value(&learning(&a).simulation).unwrap()
            } else {
                serde_json::to_value(&chosen(&a).state).unwrap()
            };
            let new = if reject {
                "x".repeat(terminal::MAX_COMMAND)
            } else if practice {
                "rpm -q garden-client".into()
            } else {
                "rpm -q archive-client".into()
            };
            assert!(new.len() <= terminal::MAX_COMMAND);
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    acl_ctrl(egui::Key::A),
                    egui::Event::Text(new.clone()),
                    key(egui::Key::Enter, false),
                ],
            );
            draw(&ctx, &mut a);
            if reject {
                assert!(
                    serde_json::to_value(chosen(&a)).unwrap() == before,
                    "Rejected same-frame replacement+Enter keeps both sessions and models exact"
                );
                assert_eq!(console(&a, practice).command, old);
                // An explicit later button action still executes that retained draft normally.
                let run = reveal(&ctx, &mut a, 1200., 850., |m| m.rpm.run.map(|r| r.0));
                tap(&ctx, &mut a, 1200., 850., run);
                assert_eq!(console(&a, practice).entries.last().unwrap().command, old);
                let after = if practice {
                    serde_json::to_value(&learning(&a).simulation).unwrap()
                } else {
                    serde_json::to_value(&chosen(&a).state).unwrap()
                };
                assert!(
                    after != before_model,
                    "Positive control: the retained old draft is a real mutation when deliberately run later"
                );
            } else {
                assert_eq!(console(&a, practice).entries.last().unwrap().command, new);
                assert!(matches!(
                    console(&a, practice).entries.last().unwrap().output,
                    terminal::Outcome::Utility { status: 0, .. }
                ));
                let after = if practice {
                    serde_json::to_value(&learning(&a).simulation).unwrap()
                } else {
                    serde_json::to_value(&chosen(&a).state).unwrap()
                };
                assert!(
                    after == before_model,
                    "Accepted same-frame query executes the new draft, without running old reinstall"
                );
            }
        }
    }
}
fn settle_rpm_undo_point(ctx: &egui::Context, a: &mut LabApp, practice: bool, expected: &str) {
    let expected_view = if practice {
        View::Example
    } else {
        View::Assignment
    };
    assert_eq!(learning(a).view, expected_view);
    let field = reveal(ctx, a, 1200., 850., |m| m.rpm.command.map(|r| r.0));
    tap(ctx, a, 1200., 850., field);
    let id = a.metrics.rpm.command.unwrap().1;
    // Pinned egui saves stable Undo points after one second. Observe its real widget
    // state while rendering bounded idle frames; do not inject a point or clock.
    for _ in 0..120 {
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "Actual command field must have focus in {expected_view:?} for {expected:?}"
        );
        assert_eq!(console(a, practice).command, expected);
        let state =
            egui::TextEdit::load_state(ctx, id).expect("Actual focused command widget state");
        if !state.undoer().is_in_flux() {
            return;
        }
        frame(ctx, a, 1200., 850., vec![]);
    }
    panic!("Actual focused RPM Undo group did not stabilize within120 rendered idle frames");
}
#[test]
fn rpm_actual_challenge_and_independent_undo_keep_practice_data_separate() {
    let (ctx, mut a, _) = opened(IDS[0]);
    tab(&ctx, &mut a, View::Example);
    type_command(&ctx, &mut a, 1200., 850., "rpm -q gallery-client");
    settle_rpm_undo_point(&ctx, &mut a, true, "rpm -q gallery-client");
    let practice_id = a.metrics.rpm.command.unwrap().1;
    let mut practice = serde_json::to_value(learning(&a)).unwrap();
    let mut after_tab = serde_json::to_value(chosen(&a)).unwrap();
    tab(&ctx, &mut a, View::Assignment);
    practice["view"] = serde_json::to_value(View::Assignment).unwrap();
    after_tab["workspace"]["rpm_learning"]["view"] = practice["view"].clone();
    assert!(
        serde_json::to_value(chosen(&a)).unwrap() == after_tab,
        "The explicit Assignment tab changes only the known view leaf; both models and sessions stay exact"
    );
    assert!(serde_json::to_value(learning(&a)).unwrap() == practice);
    assert_ne!(a.metrics.rpm.command.unwrap().1, practice_id);
    type_command(&ctx, &mut a, 1200., 850., "rpm -q ledger-client");
    settle_rpm_undo_point(&ctx, &mut a, false, "rpm -q ledger-client");
    type_command(&ctx, &mut a, 1200., 850., "rpm -q ledger-note");
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
        frame(&ctx, &mut a, 1200., 850., vec![]);
        assert!(serde_json::to_value(learning(&a)).unwrap() == practice);
        if console(&a, false).command == "rpm -q ledger-client" {
            break;
        }
    }
    assert_eq!(console(&a, false).command, "rpm -q ledger-client");
    let mode = reveal(&ctx, &mut a, 1200., 850., |m| m.mode_challenge);
    tap(&ctx, &mut a, 1200., 850., mode);
    assert!(chosen(&a).challenge);
    assert!(a.metrics.rpm.tabs.is_empty());
    assert!(a.metrics.rpm.insert.is_none());
    assert!(serde_json::to_value(learning(&a)).unwrap() == practice);
    let mode = reveal(&ctx, &mut a, 1200., 850., |m| m.mode_learn);
    tap(&ctx, &mut a, 1200., 850., mode);
    assert!(!chosen(&a).challenge);
    assert!(serde_json::to_value(learning(&a)).unwrap() == practice);
    tab(&ctx, &mut a, View::Example);
    practice["view"] = serde_json::to_value(View::Example).unwrap();
    assert!(serde_json::to_value(learning(&a)).unwrap() == practice);
    assert_eq!(console(&a, true).command, "rpm -q gallery-client");
    assert_eq!(console(&a, false).command, "rpm -q ledger-client");
    settle_rpm_undo_point(&ctx, &mut a, true, "rpm -q gallery-client");
    let before_practice_undo = serde_json::to_value(chosen(&a)).unwrap();
    let assignment_session = console(&a, false).clone();
    type_command(&ctx, &mut a, 1200., 850., "rpm -q gallery-note");
    assert_eq!(console(&a, true).command, "rpm -q gallery-note");
    assert_eq!(console(&a, false), &assignment_session);
    assert!(serde_json::to_value(&chosen(&a).state).unwrap() == before_practice_undo["state"]);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    frame(&ctx, &mut a, 1200., 850., vec![]);
    assert_eq!(console(&a, true).command, "rpm -q gallery-client");
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before_practice_undo);
}
