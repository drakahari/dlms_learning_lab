//! Real widget events on four explicitly unregistered technical contexts.
use super::*;
use dlms_learning_lab::{
    nfs as core, nfs_teaching as learn, nfs_workspace as work, script_teaching::View,
};
use serde_json::json;
fn scenario(index: usize) -> pack::Scenario {
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/nfs-workbench-v1.json"
    ))
    .unwrap();
    let c = &raw["cases"][index];
    let solution = c["reference"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| format!("{}{}", work::PREFIX, serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    let v = json!({"schema_version":1,"scenarios":[{"id":format!("nfs-technical-{index}"),"version":1,"track":"LPIC-2","topic":"Virtual NFS","unit":"","title":"Technical NFS adapter","summary":"Inspect supplied NFS layers","objective":"Complete actual current and new-mount outcomes","retry_guidance":"Inspect saved, applied and live values","difficulty":"Intermediate","minutes":15,"steps":["Inspect","Edit","Check"],"guide":[],"explanation":"Virtual existing files only","hints":["Save differs from Apply"],"sources":["https://www.rfc-editor.org/rfc/rfc1813.html"],"simplifications":"Supplied virtual NFSv3 ACCESS-style model; no host mount or RPC","initial":{"kind":"Nfs","simulation":c["definition"]["initial"]},"goal":{"kind":"Nfs","definition":c["definition"]},"solution":solution,"prerequisites":[],"nfs_teaching":c["lesson"]}]});
    pack::load(&v.to_string()).unwrap().remove(0)
}

pub(in crate::ui) fn sources() -> Vec<Scenario> {
    (0..4).map(scenario).collect()
}
fn opened(n: usize) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (168, 43));
    let at = a.scenarios.len();
    a.scenarios.extend(sources());
    a.launch(at + n);
    draw(&ctx, &mut a);
    (ctx, a, at + n)
}
fn chosen(a: &LabApp) -> &Attempt {
    &a.progress.attempts[&a.scenarios[a.selected.unwrap()].id]
}
fn learning(a: &LabApp) -> &learn::Learning {
    chosen(a)
        .workspace
        .as_ref()
        .unwrap()
        .nfs_learning
        .as_ref()
        .unwrap()
}
fn live(a: &LabApp, practice: bool) -> &core::State {
    if practice {
        &learning(a).simulation
    } else {
        let State::Nfs { simulation } = &chosen(a).state else {
            panic!()
        };
        simulation
    }
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    let mut last = Vec::new();
    for _ in 0..40 {
        let (r, clip) = if field {
            let r = a
                .metrics
                .nfs
                .fields
                .iter()
                .find(|r| r.0 == key)
                .expect("Actual NFS editor");
            (r.1, r.4)
        } else {
            let r = a
                .metrics
                .nfs
                .controls
                .iter()
                .find(|r| r.0 == key)
                .expect("Actual NFS control");
            (r.1, r.3)
        };
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let v = root.intersect(clip);
        if v.contains(r.center()) && v.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        if last.len() == 3 {
            last.remove(0);
        }
        last.push((r, v));
        let dy = if r.bottom() > root.bottom() {
            root.bottom() - r.bottom() - 8.
        } else {
            root.top() - r.top() + 8.
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(
                    r.center().x.clamp(root.left() + 1., root.right() - 1.),
                    root.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., dy.clamp(-300., 300.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            draw(ctx, a);
        }
    }
    panic!("NFS strict visibility exhausted: {key} {last:?}")
}
fn tap(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
    for pressed in [true, false] {
        frame(
            ctx,
            a,
            1200.,
            850.,
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
    for _ in 0..12 {
        draw(ctx, a);
    }
}
fn control(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false);
    tap(ctx, a, r);
}
fn tab(ctx: &egui::Context, a: &mut LabApp, v: View) {
    for _ in 0..40 {
        let r = a.metrics.nfs.tabs.iter().find(|r| r.0 == v).unwrap().1;
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if root.contains_rect(r) {
            tap(ctx, a, r);
            assert_eq!(learning(a).view, v);
            return;
        }
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(root.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., 300.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            draw(ctx, a);
        }
    }
    panic!("NFS tab remained hidden")
}
fn choose(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    control(ctx, a, key);
    let target = format!("{key}:{value}");
    for _ in 0..20 {
        let r = a
            .metrics
            .nfs
            .choices
            .iter()
            .find(|r| r.0 == target)
            .expect("Authored supplied NFS option")
            .1;
        let clip = a
            .metrics
            .nfs
            .popup
            .as_ref()
            .unwrap()
            .1
            .intersect(ctx.content_rect());
        if clip.contains(r.center()) && clip.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            tap(ctx, a, r);
            assert!(
                a.metrics
                    .nfs
                    .selections
                    .iter()
                    .any(|r| r.0 == key && r.1 == value),
                "Actual popup choice failed: scenario={} challenge={} view={:?} key={key} requested={value} target={r:?} popup_clip={clip:?} root={:?} offset={:?} presented={:?}",
                a.scenarios[a.selected.unwrap()].id,
                chosen(a).challenge,
                learning(a).view,
                a.metrics.main_viewport,
                a.metrics.main_scroll_offset,
                a.nfs_presentation.observed_request_selection()
            );
            return;
        }
        let dy = if r.bottom() > clip.bottom() {
            clip.bottom() - r.bottom()
        } else {
            clip.top() - r.top()
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
                    delta: egui::vec2(0., dy.clamp(-250., 250.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            draw(ctx, a);
        }
    }
    panic!("NFS supplied option clipped")
}
fn text(ctx: &egui::Context, a: &mut LabApp, field_key: &str, value: &str) {
    let r = reveal(ctx, a, field_key, true);
    tap(ctx, a, r);
    let id = a
        .metrics
        .nfs
        .fields
        .iter()
        .find(|r| r.0 == field_key)
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(id)));
    let edit = if value.is_empty() {
        key(egui::Key::Backspace, false)
    } else {
        egui::Event::Text(value.into())
    };
    frame(ctx, a, 1200., 850., vec![acl_ctrl(egui::Key::A), edit]);
    draw(ctx, a);
}
fn settled_document_undo(ctx: &egui::Context, a: &mut LabApp, expected: &str) {
    let id = a
        .metrics
        .nfs
        .fields
        .iter()
        .find(|r| r.0 == "exports")
        .unwrap()
        .2;
    // Pinned egui records a stable point after one second. Observe actual idle
    // frames and native Undo state; never inject a point, focus or a clock.
    for _ in 0..120 {
        assert!(ctx.memory(|m| m.has_focus(id)));
        assert_eq!(live(a, false).runtime().servers[0].draft, expected);
        let state = egui::TextEdit::load_state(ctx, id).unwrap();
        if !state.undoer().is_in_flux() {
            return;
        }
        frame(ctx, a, 1200., 850., vec![]);
    }
    panic!("NFS document did not reach a native stable Undo checkpoint");
}
fn expanded(ctx: &egui::Context, a: &mut LabApp, toggle: &str, field: &str) {
    if !a.metrics.nfs.fields.iter().any(|r| r.0 == field)
        && !a.metrics.nfs.controls.iter().any(|r| r.0 == field)
    {
        control(ctx, a, toggle);
    }
}
fn request(ctx: &egui::Context, a: &mut LabApp, client: &str, mountpoint: &str) {
    choose(ctx, a, "client", client);
    choose(ctx, a, "mountpoint", mountpoint);
}
fn action(ctx: &egui::Context, a: &mut LabApp, step: &core::Action) {
    match step {
        core::Action::SetExportsDraft { server, text: body } => {
            choose(ctx, a, "server", server);
            text(ctx, a, "exports", body)
        }
        core::Action::SaveExportsDraft { server } => {
            choose(ctx, a, "server", server);
            control(ctx, a, "save-exports")
        }
        core::Action::ApplySavedExports { server } => {
            choose(ctx, a, "server", server);
            control(ctx, a, "apply-exports")
        }
        core::Action::SetFstabDraft { client, text: body } => {
            choose(ctx, a, "client", client);
            text(ctx, a, "fstab", body)
        }
        core::Action::SaveFstabDraft { client } => {
            choose(ctx, a, "client", client);
            control(ctx, a, "save-fstab")
        }
        core::Action::MountSaved { client, mountpoint } => {
            request(ctx, a, client, mountpoint);
            control(ctx, a, "mount-saved")
        }
        core::Action::Unmount { client, mountpoint } => {
            request(ctx, a, client, mountpoint);
            control(ctx, a, "unmount")
        }
        core::Action::MountExplicit {
            client,
            specification,
        } => {
            request(ctx, a, client, &specification.mountpoint);
            expanded(ctx, a, "manual-toggle", "root");
            choose(ctx, a, "manual-server", &specification.server);
            choose(ctx, a, "root", &specification.root);
            control(
                ctx,
                a,
                if specification.access == core::Access::Ro {
                    "access-ro"
                } else {
                    "access-rw"
                },
            );
            control(ctx, a, "mount-explicit")
        }
        core::Action::ReadExisting {
            client,
            mountpoint,
            profile,
            relative,
        } => {
            request(ctx, a, client, mountpoint);
            choose(ctx, a, "profile", profile);
            text(ctx, a, "relative", relative);
            control(ctx, a, "read")
        }
        core::Action::WriteExisting {
            client,
            mountpoint,
            profile,
            relative,
            text: body,
        } => {
            request(ctx, a, client, mountpoint);
            choose(ctx, a, "profile", profile);
            text(ctx, a, "relative", relative);
            expanded(ctx, a, "write-toggle", "write-text");
            text(ctx, a, "write-text", body);
            control(ctx, a, "write")
        }
    }
}
fn reference(a: &LabApp) -> Vec<core::Action> {
    a.scenarios[a.selected.unwrap()]
        .solution
        .iter()
        .map(|s| core::Action::from_json(s.strip_prefix(work::PREFIX).unwrap()).unwrap())
        .collect()
}
fn reset(ctx: &egui::Context, a: &mut LabApp, confirm: bool) {
    for _ in 0..40 {
        let r = a.metrics.reset.unwrap();
        let root = a.metrics.main_viewport.unwrap();
        if root.contains_rect(r) {
            tap(ctx, a, r);
            break;
        }
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(root.center()),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., 300.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            draw(ctx, a);
        }
    }
    let r = if confirm {
        a.metrics.confirm_reset.unwrap()
    } else {
        a.metrics.reset_cancel.unwrap()
    };
    assert!(a.metrics.main_viewport.unwrap().contains_rect(r));
    tap(ctx, a, r);
}
#[test]
fn nfs_four_real_introductions_isolated_fill_examples_experiments_and_repairs() {
    for n in 0..4 {
        let (ctx, mut a, i) = opened(n);
        assert_eq!(learning(&a).view, View::Introduction);
        let assigned = serde_json::to_value(&chosen(&a).state).unwrap();
        tab(&ctx, &mut a, View::Example);
        let before = learning(&a).simulation.clone();
        control(&ctx, &mut a, "fill");
        assert_eq!(
            learning(&a).simulation.runtime().servers[0].saved,
            before.runtime().servers[0].saved
        );
        assert_eq!(
            learning(&a).simulation.runtime().servers[0].applied,
            before.runtime().servers[0].applied
        );
        assert!(learning(&a).session.entries.is_empty());
        assert!(serde_json::to_value(&chosen(&a).state).unwrap() == assigned);
        let source = a.scenarios[i].nfs_teaching.as_ref().unwrap().clone();
        for step in &source.example_actions {
            action(&ctx, &mut a, &step.action);
        }
        control(&ctx, &mut a, "check");
        assert!(learning(&a).checks(&source).iter().all(|r| r.0));
        assert!(!a.metrics.completion);
        for step in &source.try_change.actions {
            action(&ctx, &mut a, step);
        }
        assert_eq!(
            learning(&a)
                .checks(&source)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            source.try_change.expected
        );
        for step in &source.repair {
            action(&ctx, &mut a, step);
        }
        assert!(learning(&a).checks(&source).iter().all(|r| r.0));
        assert!(serde_json::to_value(&chosen(&a).state).unwrap() == assigned);
        tab(&ctx, &mut a, View::Assignment);
        assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
    }
}
#[test]
fn nfs_four_actual_assignment_wrong_partial_reference_invalidation_repair_and_two_alternates() {
    let raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/nfs-workbench-v1.json"
    ))
    .unwrap();
    for n in 0..4 {
        let (ctx, mut a, i) = opened(n);
        tab(&ctx, &mut a, View::Assignment);
        control(&ctx, &mut a, "check");
        assert!(!a.metrics.completion);
        let actions = reference(&a);
        action(&ctx, &mut a, &actions[0]);
        control(&ctx, &mut a, "check");
        assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
        for step in &actions[1..] {
            action(&ctx, &mut a, step);
        }
        control(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
        let engine::Goal::Nfs { definition } = &a.scenarios[i].goal else {
            panic!()
        };
        let (client, spec) = definition
            .requirements
            .iter()
            .find_map(|r| {
                if let work::Requirement::CurrentMount {
                    client,
                    specification,
                } = r
                {
                    Some((client.clone(), specification.clone()))
                } else {
                    None
                }
            })
            .unwrap();
        action(
            &ctx,
            &mut a,
            &core::Action::Unmount {
                client: client.clone(),
                mountpoint: spec.mountpoint.clone(),
            },
        );
        assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
        action(
            &ctx,
            &mut a,
            &core::Action::MountExplicit {
                client,
                specification: spec,
            },
        );
        control(&ctx, &mut a, "check");
        assert!(a.metrics.completion);
        for alternate in raw["cases"][n]["alternatives"].as_array().unwrap() {
            reset(&ctx, &mut a, true);
            tab(&ctx, &mut a, View::Assignment);
            for step in alternate.as_array().unwrap() {
                action(&ctx, &mut a, &serde_json::from_value(step.clone()).unwrap());
            }
            control(&ctx, &mut a, "check");
            assert!(
                a.metrics.completion,
                "actual independently authored alternate"
            );
        }
    }
}
#[test]
fn nfs_readonly_unrun_queries_preserve_credit_while_mutations_refresh_same_frame() {
    let (ctx, mut a, i) = opened(1);
    tab(&ctx, &mut a, View::Assignment);
    for step in reference(&a) {
        action(&ctx, &mut a, &step);
    }
    control(&ctx, &mut a, "check");
    let state = chosen(&a).state.clone();
    text(&ctx, &mut a, "relative", "../unfinished");
    expanded(&ctx, &mut a, "write-toggle", "write-text");
    text(&ctx, &mut a, "write-text", "not yet written\n");
    assert_eq!(chosen(&a).state, state);
    assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
    assert!(a.metrics.completion);
    let required = if let engine::Goal::Nfs { definition } = &a.scenarios[i].goal {
        definition
            .requirements
            .iter()
            .find_map(|r| {
                if let work::Requirement::CurrentMount {
                    client,
                    specification,
                } = r
                {
                    Some((client.clone(), specification.mountpoint.clone()))
                } else {
                    None
                }
            })
            .unwrap()
    } else {
        panic!()
    };
    request(&ctx, &mut a, &required.0, &required.1);
    control(&ctx, &mut a, "unmount");
    assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
    assert!(!a.metrics.completion);
    assert!(
        !a.metrics
            .nfs
            .state_text
            .iter()
            .any(|t| t.contains(&format!("to {}", required.1)))
    );
}
#[test]
fn nfs_modals_and_rejected_paste_cannot_submit_old_fields() {
    for view in [View::Example, View::Assignment] {
        for modal in 0..4 {
            let (ctx, mut a, _) = opened(0);
            tab(&ctx, &mut a, view);
            text(&ctx, &mut a, "exports", "# unfinished\n");
            let before = serde_json::to_value(chosen(&a)).unwrap();
            match modal {
                0 => a.reset_pending = true,
                1 => a.settings_open = true,
                2 => a.theme_preview = true,
                _ => {
                    a.settings_open = true;
                    a.settings_data = true;
                    draw(&ctx, &mut a);
                    let r = a.metrics.data_export.unwrap();
                    tap(&ctx, &mut a, r);
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
                    egui::Event::Text("background".into()),
                    acl_ctrl(egui::Key::Enter),
                ],
            );
            draw(&ctx, &mut a);
            assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
            assert!(a.metrics.nfs.fields.iter().all(|r| !r.3));
            assert!(a.metrics.nfs.controls.iter().all(|r| !r.2));
        }
    }
    // The identical focused edit+Check sequence is delivered in both cases.
    for too_big in [false, true] {
        let (ctx, mut a, _) = opened(0);
        tab(&ctx, &mut a, View::Assignment);
        let r = reveal(&ctx, &mut a, "exports", true);
        tap(&ctx, &mut a, r);
        let before = serde_json::to_value(chosen(&a)).unwrap();
        let input = if too_big {
            "x".repeat(core::MAX_DOCUMENT + 1)
        } else {
            "# accepted replacement\n".into()
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                acl_ctrl(egui::Key::A),
                egui::Event::Text(input),
                acl_ctrl(egui::Key::Enter),
            ],
        );
        if too_big {
            assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
            assert!(!a.metrics.nfs.check_evaluated);
        } else {
            assert!(serde_json::to_value(chosen(&a)).unwrap() != before);
            assert!(a.metrics.nfs.check_evaluated);
        }
    }
}
#[test]
fn nfs_editor_ids_focus_undo_and_both_layouts_survive_font_theme_and_width_reflow() {
    let (ctx, mut a, _) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    text(&ctx, &mut a, "exports", "# first\n");
    let id = a
        .metrics
        .nfs
        .fields
        .iter()
        .find(|r| r.0 == "exports")
        .unwrap()
        .2;
    let before = live(&a, false).runtime().servers[0].draft.clone();
    settled_document_undo(&ctx, &mut a, &before);
    text(&ctx, &mut a, "exports", "# second\n");
    settled_document_undo(&ctx, &mut a, "# second\n");
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        for light in [true, false] {
            let selected = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap();
            a.theme_id = selected.id.clone();
            theme::apply(&ctx, &selected.id);
            assert_eq!(theme::find(&a.theme_id).light, light);
            for (w, h) in [(1920., 1080.), (946., 929.)] {
                frame(&ctx, &mut a, w, h, vec![]);
                assert_eq!(
                    a.metrics
                        .nfs
                        .fields
                        .iter()
                        .find(|r| r.0 == "exports")
                        .unwrap()
                        .2,
                    id
                );
                assert!(ctx.memory(|m| m.has_focus(id)));
                for r in &a.metrics.nfs.fields {
                    assert!(
                        r.1.width() > 0.
                            && r.1.right()
                                <= ctx.content_rect().right() + 1. / ctx.pixels_per_point()
                    );
                }
                if w == 946. && scale == 2. {
                    assert!(!a.metrics.nfs.columns);
                }
            }
        }
    }
    a.scale = 1.;
    draw(&ctx, &mut a);
    let mut restored = false;
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
        draw(&ctx, &mut a);
        if live(&a, false).runtime().servers[0].draft == before {
            restored = true;
            break;
        }
    }
    assert!(
        restored,
        "native Undo restores the accepted previous document"
    );
}
#[test]
fn nfs_warm_save_full_raw_drafts_cancel_reset_challenge_and_plain_load_preserve_records() {
    let (ctx, mut a, i) = opened(0);
    tab(&ctx, &mut a, View::Example);
    text(&ctx, &mut a, "relative", "example unfinished");
    tab(&ctx, &mut a, View::Assignment);
    for step in reference(&a) {
        action(&ctx, &mut a, &step);
    }
    text(&ctx, &mut a, "relative", "assigned unfinished");
    let before = serde_json::to_value(chosen(&a)).unwrap();
    reset(&ctx, &mut a, false);
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    a.progress.favorites.insert(a.scenarios[i].id.clone());
    a.dirty = true;
    a.save();
    let bytes = std::fs::read(&a.path).unwrap();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&loaded.attempts[&a.scenarios[i].id]).unwrap() == before);
    assert_eq!(std::fs::read(&a.path).unwrap(), bytes);
    let r = a.metrics.mode_challenge.unwrap(); // Reveal the actual common header before changing mode.
    let root = a.metrics.main_viewport.unwrap();
    if !root.contains_rect(r) {
        for _ in 0..12 {
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    egui::Event::PointerMoved(root.center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., 300.),
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    }
    let r = a.metrics.mode_challenge.unwrap();
    assert!(a.metrics.main_viewport.unwrap().contains_rect(r));
    tap(&ctx, &mut a, r);
    assert!(chosen(&a).challenge);
    assert!(a.metrics.nfs.tabs.is_empty());
    assert!(!a.metrics.nfs.controls.iter().any(|r| r.0 == "fill"));
    assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
    reset(&ctx, &mut a, true);
    assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
    assert!(a.progress.favorites.contains(&a.scenarios[i].id));
}

const OLD_WRITE_DRAFT: &str = "old draft must not be submitted\n";
fn ready_write_fixture(full: bool) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = opened(1);
    tab(&ctx, &mut a, View::Assignment);
    for step in reference(&a) {
        action(&ctx, &mut a, &step);
    }
    let write = reference(&a)
        .into_iter()
        .find(|a| matches!(a, core::Action::WriteExisting { .. }))
        .unwrap();
    let core::Action::WriteExisting {
        client,
        mountpoint,
        profile,
        relative,
        ..
    } = write
    else {
        panic!()
    };
    request(&ctx, &mut a, &client, &mountpoint);
    choose(&ctx, &mut a, "profile", &profile);
    text(&ctx, &mut a, "relative", &relative);
    expanded(&ctx, &mut a, "write-toggle", "write-text");
    text(&ctx, &mut a, "write-text", OLD_WRITE_DRAFT);
    if full {
        let attempt = a.progress.attempts.get_mut(&a.scenarios[i].id).unwrap();
        let mut lo = 0usize;
        let mut hi = work::MAX_ATTEMPT;
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            attempt.transcript = vec!["x".repeat(mid)];
            if serde_json::to_vec(attempt).unwrap().len() <= work::MAX_ATTEMPT {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        attempt.transcript = vec!["x".repeat(lo)];
        assert_eq!(
            serde_json::to_vec(attempt).unwrap().len(),
            work::MAX_ATTEMPT
        );
    }
    draw(&ctx, &mut a);
    let r = reveal(&ctx, &mut a, "write-text", true);
    tap(&ctx, &mut a, r);
    (ctx, a, i)
}
fn written_result(a: &LabApp) -> &str {
    &live(a, false)
        .runtime()
        .contents
        .iter()
        .find(|c| c.path == "/srv/assignment1/work/result.txt")
        .unwrap()
        .text
}
fn pending_write_text(a: &LabApp) -> &str {
    &chosen(a)
        .workspace
        .as_ref()
        .unwrap()
        .nfs_session
        .as_ref()
        .unwrap()
        .write_text
}
#[test]
fn nfs_dropped_paste_vetoes_write_at_both_capacities_but_ordinary_and_earlier_edited_writes_work() {
    for full in [false, true] {
        let (ctx, mut a, i) = ready_write_fixture(full);
        let write = reveal(&ctx, &mut a, "write", false);
        // Actual held-button release packet already observed to drop Paste under
        // pinned egui's default outside-click focus policy. Do not invent delivery.
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(write.center()),
                egui::Event::PointerButton {
                    pos: write.center(),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        );
        let before = serde_json::to_value(chosen(&a)).unwrap();
        let replacement = "\u{1}".repeat(4096);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                acl_ctrl(egui::Key::A),
                egui::Event::Paste(replacement.clone()),
                egui::Event::PointerButton {
                    pos: write.center(),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                },
            ],
        );
        assert_eq!(
            a.metrics.nfs.queued,
            Some("write"),
            "Actual pointer release targets Write"
        );
        assert!(
            a.metrics.nfs.pending_text
                && !a.metrics.nfs.received_text
                && a.metrics.nfs.dropped_text,
            "The real dropped Paste is distinguished from an accepted edit"
        );
        assert_eq!(
            a.metrics.nfs.submitted, None,
            "Dropped text vetoes dispatch at either capacity"
        );
        assert!(
            serde_json::to_value(chosen(&a)).unwrap() == before,
            "Dropped edit+click retains complete Attempt, including genuine histories"
        );
        assert_eq!(pending_write_text(&a), OLD_WRITE_DRAFT);
        assert!(progress::current_solved(chosen(&a), &a.scenarios[i]));
        let (_, notice) = a.nfs_presentation.observed_write_request();
        assert!(
            notice.is_some_and(|s| s.contains("was not applied") && s.contains("Enter it again")),
            "Explain dropped text, rather than a capacity failure"
        );
        if !full {
            // No pending text: the normal later button genuinely writes its
            // retained, documented old request. The guard does not disable Write.
            control(&ctx, &mut a, "write");
            assert_eq!(written_result(&a), OLD_WRITE_DRAFT);
            let input = reveal(&ctx, &mut a, "write-text", true);
            tap(&ctx, &mut a, input);
            let before_state = chosen(&a).state.clone();
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    acl_ctrl(egui::Key::A),
                    egui::Event::Paste(replacement.clone()),
                ],
            );
            assert!(
                a.metrics.nfs.pending_text
                    && a.metrics.nfs.received_text
                    && !a.metrics.nfs.dropped_text
            );
            assert_eq!(
                a.metrics.nfs.submitted, None,
                "An earlier accepted Paste never writes automatically"
            );
            assert_eq!(chosen(&a).state, before_state);
            assert_eq!(pending_write_text(&a), replacement);
            assert_eq!(a.nfs_presentation.observed_write_request().0, replacement);
            control(&ctx, &mut a, "write");
            assert_eq!(
                written_result(&a),
                replacement,
                "The real later Write uses newly accepted text"
            );
        }
    }
}
#[test]
fn nfs_same_frame_paste_check_admits_new_or_identical_text_and_atomically_rejects_full_attempt() {
    for full in [false, true] {
        let (ctx, mut a, _) = ready_write_fixture(full);
        let before = serde_json::to_value(chosen(&a)).unwrap();
        let state = chosen(&a).state.clone();
        let replacement = "\u{1}".repeat(4096);
        // The existing Ctrl+Enter Check shortcut preserves actual editor focus.
        // Both capacities receive the exact same real focused Paste+Check packet.
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                acl_ctrl(egui::Key::A),
                egui::Event::Paste(replacement.clone()),
                acl_ctrl(egui::Key::Enter),
            ],
        );
        assert!(
            a.metrics.nfs.pending_text
                && a.metrics.nfs.received_text
                && !a.metrics.nfs.dropped_text,
            "Both capacities genuinely receive the Paste, independently of admission"
        );
        assert_eq!(a.metrics.nfs.queued, Some("check"));
        if full {
            assert_eq!(a.metrics.nfs.submitted, None);
            assert!(!a.metrics.nfs.check_evaluated);
            assert!(
                serde_json::to_value(chosen(&a)).unwrap() == before,
                "Whole Attempt and both separate workspaces stay exact on capacity refusal"
            );
        } else {
            assert_eq!(a.metrics.nfs.submitted, Some("check"));
            assert!(a.metrics.nfs.check_evaluated);
            assert_eq!(pending_write_text(&a), replacement);
            assert_eq!(a.nfs_presentation.observed_write_request().0, replacement);
            assert_eq!(
                chosen(&a).state,
                state,
                "Check does not write the pending replacement"
            );
        }
    }
    let (ctx, mut a, _) = ready_write_fixture(false);
    let before = serde_json::to_value(chosen(&a)).unwrap();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Paste(OLD_WRITE_DRAFT.into()),
            acl_ctrl(egui::Key::Enter),
        ],
    );
    assert!(
        a.metrics.nfs.pending_text && a.metrics.nfs.received_text && !a.metrics.nfs.dropped_text
    );
    assert_eq!(
        a.metrics.nfs.submitted,
        Some("check"),
        "Identical delivered Paste is not a dropped event"
    );
    assert!(a.metrics.nfs.check_evaluated);
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    for ignored in ["\n", "\r"] {
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::Text(ignored.into()),
                acl_ctrl(egui::Key::Enter),
            ],
        );
        assert!(!a.metrics.nfs.pending_text && !a.metrics.nfs.dropped_text);
        assert_eq!(
            a.metrics.nfs.submitted,
            Some("check"),
            "Pinned egui ignores exact newline Text events"
        );
        assert!(a.metrics.nfs.check_evaluated);
        assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    }
}
#[test]
fn nfs_actual_backup_review_cancel_restore_and_reopen_keep_full_selected_and_genuine_history() {
    use dlms_learning_lab::backup;
    let (ctx, mut a, i) = opened(0);
    a.smoke = true;
    let raw = include_bytes!("../../../tests/fixtures/shell-v8-synthetic-writer-progress.json");
    a.progress = progress::decode(raw, &a.scenarios).unwrap();
    std::fs::write(&a.path, raw).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    tab(&ctx, &mut a, View::Example);
    text(&ctx, &mut a, "relative", "gallery pending request");
    tab(&ctx, &mut a, View::Assignment);
    for step in reference(&a) {
        action(&ctx, &mut a, &step);
    }
    text(&ctx, &mut a, "relative", "assignment pending request");
    let id = a.scenarios[i].id.clone();
    a.progress.favorites.insert(id.clone());
    a.dirty = true;
    a.save();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    let selected = serde_json::to_value(&loaded.attempts[&id]).unwrap();
    let raw_saved = std::fs::read(&a.path).unwrap();
    assert_eq!(
        std::fs::read(a.path.with_extension(format!(
            "before-shell-v{}.json",
            dlms_learning_lab::shell::EVALUATOR_REVISION
        )))
        .unwrap(),
        raw
    );
    let export = a.path.parent().unwrap().join("dlms-learning-backup.json");
    backup::export(&export, &loaded, &a.scenarios).unwrap();
    assert_eq!(std::fs::read(&a.path).unwrap(), raw_saved);
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let before = serde_json::to_value((&a.progress, &a.progress.migration_receipts)).unwrap();
    let r = a.metrics.data_cancel.unwrap();
    tap(&ctx, &mut a, r);
    assert!(serde_json::to_value((&a.progress, &a.progress.migration_receipts)).unwrap() == before);
    assert_eq!(std::fs::read(&a.path).unwrap(), raw_saved);
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_confirm.unwrap();
    tap(&ctx, &mut a, r);
    assert!(a.selected.is_none());
    assert!(serde_json::to_value(&a.progress.attempts[&id]).unwrap() == selected);
    assert!(a.progress.favorites.contains(&id));
    let restored = progress::load(&a.path, &a.scenarios).unwrap();
    assert!(serde_json::to_value(&restored.attempts[&id]).unwrap() == selected);
    // The existing inspection intentionally rechecks scripts; its real old8 records must remain as historical evidence.
    let old: serde_json::Value = serde_json::from_slice(raw).unwrap();
    for (old_id, attempt) in old["attempts"].as_object().unwrap() {
        if !attempt["state"]["run"].is_null() {
            let current = serde_json::to_value(&restored.attempts[old_id]).unwrap();
            let w = &current["workspace"];
            assert!(
                w["script_history"] == attempt["state"]["run"]
                    || w["script_history_more"]
                        .as_array()
                        .is_some_and(|rows| rows.contains(&attempt["state"]["run"]))
            );
        }
    }
    a.settings_open = false;
    let mut continued = selected.clone();
    continued["visited"] = json!(a.progress.clock.saturating_add(1));
    a.launch(i);
    draw(&ctx, &mut a);
    assert!(
        serde_json::to_value(chosen(&a)).unwrap() == continued,
        "Continue changes only recency; full sessions and separate Learning retained before reset"
    );
}

#[test]
fn nfs_launch_capacity_refusal_keeps_old_visit_and_missing_metadata_without_panicking() {
    let (ctx, mut a, i) = opened(0);
    let source = a.scenarios[i].clone();
    let mut initial = Attempt::new(&source, 9);
    let mut lo = 0usize;
    let mut hi = work::MAX_ATTEMPT;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        initial.transcript = vec!["x".repeat(mid)];
        if serde_json::to_vec(&initial).unwrap().len() <= work::MAX_ATTEMPT {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    initial.transcript = vec!["x".repeat(lo)];
    let before = serde_json::to_value(&initial).unwrap();
    assert!(initial.workspace.is_none());
    a.progress.attempts.insert(source.id.clone(), initial);
    a.progress.clock = 9;
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.clock, 10);
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    assert!(a.notice.as_ref().is_some_and(|n| n.contains("Recency")));
    assert!(a.metrics.nfs.fields.is_empty());
}

#[test]
fn nfs_plain_enter_edits_multiline_and_ctrl_enter_checks_without_another_edit() {
    let (ctx, mut a, _) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    text(&ctx, &mut a, "exports", "# line");
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert_eq!(live(&a, false).runtime().servers[0].draft, "# line\n");
    let before = serde_json::to_value(chosen(&a)).unwrap();
    let document_id = a
        .metrics
        .nfs
        .fields
        .iter()
        .find(|r| r.0 == "exports")
        .unwrap()
        .2;
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(ctx.memory(|m| m.focused()).is_some());
    assert!(
        !ctx.memory(|m| m.has_focus(document_id)),
        "Tab moves to the next control"
    );
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, true)]);
    draw(&ctx, &mut a);
    assert!(
        ctx.memory(|m| m.has_focus(document_id)),
        "Shift+Tab returns to the document"
    );
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    assert!(a.metrics.nfs.check_evaluated);
    assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
}

#[test]
fn nfs_external_document_replacement_and_example_reset_cannot_replay_old_undo() {
    for practice in [false, true] {
        let (ctx, mut a, i) = opened(0);
        tab(
            &ctx,
            &mut a,
            if practice {
                View::Example
            } else {
                View::Assignment
            },
        );
        text(&ctx, &mut a, "exports", "# first accepted document\n");
        for _ in 0..24 {
            draw(&ctx, &mut a);
        }
        text(&ctx, &mut a, "exports", "# second accepted document\n");
        for _ in 0..24 {
            draw(&ctx, &mut a);
        }
        let source = a.scenarios[i].clone();
        let server = live(&a, practice).inventory().servers[0].id.clone();
        let replacement = core::Action::SetExportsDraft {
            server,
            text: "# restored document\n".into(),
        };
        let attempt = a.progress.attempts.get_mut(&source.id).unwrap();
        // A validated external source update, like restore, is distinct from local typing.
        if practice {
            work::perform_practice(attempt, &source, &replacement).unwrap();
        } else {
            work::perform_attempt(attempt, &source, &replacement).unwrap();
        }
        draw(&ctx, &mut a);
        let before = serde_json::to_value(chosen(&a)).unwrap();
        for _ in 0..3 {
            frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
            draw(&ctx, &mut a);
        }
        assert!(
            serde_json::to_value(chosen(&a)).unwrap() == before,
            "Undo cannot recover a document from before an external replacement"
        );
        if practice {
            text(&ctx, &mut a, "exports", "# before example reset\n");
            for _ in 0..24 {
                draw(&ctx, &mut a);
            }
            let old_id = a
                .metrics
                .nfs
                .fields
                .iter()
                .find(|r| r.0 == "exports")
                .unwrap()
                .2;
            control(&ctx, &mut a, "reset-example");
            let r = reveal(&ctx, &mut a, "exports", true);
            tap(&ctx, &mut a, r);
            let new_id = a
                .metrics
                .nfs
                .fields
                .iter()
                .find(|r| r.0 == "exports")
                .unwrap()
                .2;
            assert_ne!(old_id, new_id, "Reset starts a new edit generation");
            let before = serde_json::to_value(chosen(&a)).unwrap();
            frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
            draw(&ctx, &mut a);
            assert!(serde_json::to_value(chosen(&a)).unwrap() == before);
        }
    }
}

fn read_held_archive(ctx: &egui::Context, a: &mut LabApp, practice: bool) {
    let mount = live(a, practice)
        .runtime()
        .mounts
        .iter()
        .find(|m| m.specification.root.ends_with("/archive"))
        .unwrap()
        .clone();
    let profile = live(a, practice)
        .inventory()
        .clients
        .iter()
        .find(|c| c.id == mount.client)
        .unwrap()
        .profiles[0]
        .clone();
    action(
        ctx,
        a,
        &core::Action::ReadExisting {
            client: mount.client,
            mountpoint: mount.specification.mountpoint,
            profile,
            relative: "notice.txt".into(),
        },
    );
    assert!(
        a.metrics.nfs.read_shown,
        "An actual successful read is displayed in its own view"
    );
    assert!(a.metrics.nfs.notice_shown);
}
#[test]
fn nfs_real_read_outputs_and_status_do_not_cross_assignment_example_or_example_reset() {
    let (ctx, mut a, _) = opened(0);
    tab(&ctx, &mut a, View::Assignment);
    read_held_archive(&ctx, &mut a, false);
    tab(&ctx, &mut a, View::Example);
    assert!(
        !a.metrics.nfs.read_shown && !a.metrics.nfs.notice_shown,
        "Assignment read/status cannot appear as example output"
    );
    read_held_archive(&ctx, &mut a, true);
    tab(&ctx, &mut a, View::Assignment);
    assert!(
        !a.metrics.nfs.read_shown && !a.metrics.nfs.notice_shown,
        "Example read/status cannot appear as assignment output"
    );
    read_held_archive(&ctx, &mut a, false);
    tab(&ctx, &mut a, View::Example);
    read_held_archive(&ctx, &mut a, true);
    let assignment = serde_json::to_value(&chosen(&a).state).unwrap();
    control(&ctx, &mut a, "reset-example");
    assert!(
        !a.metrics.nfs.read_shown && !a.metrics.nfs.notice_shown,
        "Reset removes old successful read and operation status"
    );
    assert!(serde_json::to_value(&chosen(&a).state).unwrap() == assignment);
}

fn exact_rendered_rows(a: &LabApp, expected: &[bool], reason: &str) {
    assert_eq!(
        a.metrics.nfs.rows.iter().map(|r| r.1).collect::<Vec<_>>(),
        expected,
        "Full native Check vector: {reason}"
    );
    assert!(!a.metrics.completion);
}
fn plausible_wrong_plan(n: usize, reference: &[core::Action]) -> Vec<core::Action> {
    let mut wrong = reference.to_vec();
    match n {
        0 => {
            let core::Action::SetExportsDraft { text, .. } = &mut wrong[0] else {
                panic!()
            };
            // Mistakenly grant the replacement address, rather than this task's reader.
            *text = text.replace("192.0.2.20(ro)", "192.0.2.21(ro)");
        }
        1 => {
            let core::Action::SetFstabDraft { text, .. } = &mut wrong[0] else {
                panic!()
            };
            *text = text.replace(",rw", ",ro");
            let core::Action::MountSaved { client, mountpoint } = &wrong[3] else {
                panic!()
            };
            // A manual current repair does not fix the still-read-only saved mount.
            wrong[3] = core::Action::MountExplicit {
                client: client.clone(),
                specification: core::MountSpec {
                    server: "s-assignment1".into(),
                    root: "/srv/assignment1/work".into(),
                    mountpoint: mountpoint.clone(),
                    access: core::Access::Rw,
                },
            };
        }
        2 => {
            let core::Action::SetExportsDraft { text, .. } = &mut wrong[1] else {
                panic!()
            };
            // Retiring current/saved mounts is insufficient if fresh old-client access remains.
            *text = text.replace("192.0.2.21(rw)", "192.0.2.21(rw) 192.0.2.20(rw)");
        }
        3 => {
            let core::Action::SetFstabDraft { text, .. } = &mut wrong[6] else {
                panic!()
            };
            // A server read-only export does not mean the client mounted read-only.
            *text = text.replace(",ro", ",rw");
        }
        _ => unreachable!(),
    }
    wrong
}
#[test]
fn nfs_four_plausible_wrong_configs_and_meaningful_partial_repairs_have_literal_full_vectors() {
    // Manually reasoned from the fixture's ordered requirements; one final row
    // per independent new-mount TRIAL, not per step within that trial.
    let initial: [&[bool]; 4] = [
        &[true, false, false, true, true, true, true, false, false],
        &[
            true, false, false, false, true, false, true, true, false, true,
        ],
        &[
            true, false, false, false, true, false, false, true, true, false, false, true,
        ],
        &[
            true, false, false, false, false, true, false, false, true, true, true, false, false,
            true,
        ],
    ];
    let partial: [&[bool]; 4] = [
        &[true, false, false, true, true, true, true, true, true],
        &[
            true, true, false, false, true, true, true, true, false, true,
        ],
        &[
            true, false, true, false, true, true, true, true, true, true, true, true,
        ],
        &[
            true, true, false, false, false, true, true, true, true, true, true, false, true, true,
        ],
    ];
    let wrong: [&[bool]; 4] = [
        &[true, false, false, true, true, true, true, false, true],
        &[true, true, true, true, true, false, true, true, true, true],
        &[
            true, true, true, true, true, true, true, true, true, true, false, true,
        ],
        &[
            true, true, false, true, true, true, true, false, true, true, true, true, true, true,
        ],
    ];
    for n in 0..4 {
        let (ctx, mut a, _) = opened(n);
        tab(&ctx, &mut a, View::Assignment);
        control(&ctx, &mut a, "check");
        exact_rendered_rows(&a, initial[n], "untouched initial work");
        let plan = reference(&a);
        let prefix = [3, 4, 8, 9][n];
        for step in &plan[..prefix] {
            action(&ctx, &mut a, step);
        }
        control(&ctx, &mut a, "check");
        exact_rendered_rows(
            &a,
            partial[n],
            "real policy/mount repairs with remaining outcomes",
        );
        assert_ne!(initial[n], partial[n]);
        reset(&ctx, &mut a, true);
        tab(&ctx, &mut a, View::Assignment);
        for step in plausible_wrong_plan(n, &plan) {
            action(&ctx, &mut a, &step);
        }
        control(&ctx, &mut a, "check");
        exact_rendered_rows(
            &a,
            wrong[n],
            "plausible client/persistence/revocation/mode mistake",
        );
        let id = a.scenarios[a.selected.unwrap()].id.clone();
        let rows = a.nfs_presentation.memo.rows(&id, false).unwrap();
        match n {
            0 => assert!(
                !rows[1].0
                    && rows[1].1.contains("Required current mount")
                    && !rows[7].0
                    && rows[7].1.contains("New-mount test 1")
            ),
            1 => assert!(!rows[5].0 && rows[5].1.contains("Saved mount") && rows[1].0 && rows[2].0),
            2 => assert!(
                !rows[10].0
                    && rows[10].1.contains("New-mount test 2")
                    && rows[1..9].iter().all(|r| r.0)
            ),
            3 => assert!(
                !rows[2].0
                    && rows[2].1.contains("Required current mount")
                    && !rows[7].0
                    && rows[7].1.contains("Saved mount")
                    && rows[4].0
            ),
            _ => unreachable!(),
        }
    }
}

fn normal_nfs_opened(id: &str) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    assert_eq!((a.scenarios.len(), a.units.len()), (168, 43));
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(
        a.scenarios.len(),
        168,
        "Use the normal authored catalog only"
    );
    assert_eq!(chosen(&a).state, a.scenarios[i].initial);
    (ctx, a, i)
}
fn painted_nfs_text(shape: &egui::epaint::Shape, text: &str) -> bool {
    match shape {
        egui::epaint::Shape::Text(t) => t.galley.job.text == text,
        egui::epaint::Shape::Vec(v) => v.iter().any(|s| painted_nfs_text(s, text)),
        _ => false,
    }
}
fn assert_normal_write_presentation(ctx: &egui::Context, a: &mut LabApp, required: bool) {
    assert_eq!(
        a.metrics.nfs.fields.iter().any(|r| r.0 == "write-text"),
        required,
        "A required editor opens by default; optional exploration stays collapsed"
    );
    assert_eq!(
        a.metrics.nfs.controls.iter().any(|r| r.0 == "write"),
        required
    );
    let _ = reveal(ctx, a, "write-toggle", false);
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 850.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let label = if required {
        "Replace existing file text"
    } else {
        "Replace existing file text · Optional"
    };
    let other = if required {
        "Replace existing file text · Optional"
    } else {
        "Replace existing file text"
    };
    assert!(
        out.shapes.iter().any(|s| painted_nfs_text(&s.shape, label)),
        "Actual visible header has the correct task-specific label"
    );
    assert!(!out.shapes.iter().any(|s| painted_nfs_text(&s.shape, other)));
    if required {
        assert!(
            a.metrics
                .nfs
                .fields
                .iter()
                .find(|r| r.0 == "write-text")
                .unwrap()
                .3
        );
        assert!(
            a.metrics
                .nfs
                .controls
                .iter()
                .find(|r| r.0 == "write")
                .unwrap()
                .2
        );
    }
}
const NORMAL_WRITE_TASKS: [(&str, bool); 4] = [
    ("lpic2-nfs-reference", false),
    ("lpic2-nfs-publication", true),
    ("lpic2-nfs-handover", false),
    ("lpic2-nfs-capstone", true),
];
#[test]
fn normal_nfs_required_write_editor_is_open_and_honestly_labeled_in_learn_and_challenge() {
    for (id, required) in NORMAL_WRITE_TASKS {
        for challenge in [false, true] {
            let (ctx, mut a, i) = normal_nfs_opened(id);
            if challenge {
                let r = a.metrics.mode_challenge.unwrap();
                assert!(a.metrics.main_viewport.unwrap().contains_rect(r));
                tap(&ctx, &mut a, r);
                assert!(chosen(&a).challenge);
                assert!(a.metrics.nfs.tabs.is_empty());
                assert!(!a.metrics.nfs.controls.iter().any(|r| r.0 == "fill"));
            } else {
                tab(&ctx, &mut a, View::Assignment);
            }
            let before = serde_json::to_value(&a.progress).unwrap();
            assert_normal_write_presentation(&ctx, &mut a, required);
            assert_eq!(
                serde_json::to_value(&a.progress).unwrap(),
                before,
                "Editor presentation adds no stored flag, action or completion"
            );
            assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
        }
    }
}
#[test]
fn normal_nfs_assignment_and_practice_write_drafts_are_separate_without_running_write() {
    for (id, required) in NORMAL_WRITE_TASKS {
        let (ctx, mut a, i) = normal_nfs_opened(id);
        tab(&ctx, &mut a, View::Assignment);
        assert_normal_write_presentation(&ctx, &mut a, required);
        expanded(&ctx, &mut a, "write-toggle", "write-text");
        let mut assignment_before = serde_json::to_value(&a.progress).unwrap();
        text(
            &ctx,
            &mut a,
            "write-text",
            "Assignment draft stays separate\n",
        );
        let session = chosen(&a)
            .workspace
            .as_ref()
            .unwrap()
            .nfs_session
            .as_ref()
            .unwrap();
        assert_eq!(session.write_text, "Assignment draft stays separate\n");
        assert!(session.entries.is_empty(), "Typing never executes Write");
        assignment_before["attempts"][id]["workspace"]["nfs_session"] =
            serde_json::to_value(session).unwrap();
        assert_eq!(
            serde_json::to_value(&a.progress).unwrap(),
            assignment_before,
            "Assignment typing changes only its saved request draft"
        );
        tab(&ctx, &mut a, View::Example);
        assert_normal_write_presentation(&ctx, &mut a, required);
        expanded(&ctx, &mut a, "write-toggle", "write-text");
        let mut practice_before = serde_json::to_value(&a.progress).unwrap();
        text(
            &ctx,
            &mut a,
            "write-text",
            "Different ungraded practice draft\n",
        );
        assert_eq!(
            learning(&a).session.write_text,
            "Different ungraded practice draft\n"
        );
        assert!(
            learning(&a).session.entries.is_empty(),
            "Practice typing never executes Write"
        );
        practice_before["attempts"][id]["workspace"]["nfs_learning"]["session"] =
            serde_json::to_value(&learning(&a).session).unwrap();
        assert_eq!(
            serde_json::to_value(&a.progress).unwrap(),
            practice_before,
            "Practice typing changes only its separate draft, never assignment State or credit"
        );
        assert_eq!(
            a.nfs_presentation.observed_write_request().0,
            "Different ungraded practice draft\n"
        );
        tab(&ctx, &mut a, View::Assignment);
        draw(&ctx, &mut a);
        assert_eq!(
            a.nfs_presentation.observed_write_request().0,
            "Assignment draft stays separate\n"
        );
        assert_eq!(
            chosen(&a)
                .workspace
                .as_ref()
                .unwrap()
                .nfs_session
                .as_ref()
                .unwrap()
                .write_text,
            "Assignment draft stays separate\n"
        );
        assert_eq!(
            learning(&a).session.write_text,
            "Different ungraded practice draft\n"
        );
        assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
    }
}

fn complete_nfs_progress(a: &LabApp) -> serde_json::Value {
    let mut value = serde_json::to_value(&a.progress).unwrap();
    value["migration_receipts"] = serde_json::to_value(&a.progress.migration_receipts).unwrap();
    value
}
fn normal_nfs_given_mode(id: &str, mode: usize) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a, i) = normal_nfs_opened(id);
    match mode {
        0 => tab(&ctx, &mut a, View::Assignment),
        1 => {
            let r = a.metrics.mode_challenge.unwrap();
            assert!(a.metrics.main_viewport.unwrap().contains_rect(r));
            tap(&ctx, &mut a, r);
            assert!(chosen(&a).challenge && a.metrics.nfs.tabs.is_empty());
        }
        2 => tab(&ctx, &mut a, View::Example),
        _ => unreachable!(),
    }
    (ctx, a, i)
}
fn given_capture(ctx: &egui::Context, a: &mut LabApp) -> egui::FullOutput {
    let before = complete_nfs_progress(a);
    let goal = serde_json::to_value(&a.scenarios[a.selected.unwrap()].goal).unwrap();
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200. / a.scale, 850. / a.scale),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    assert_eq!(
        complete_nfs_progress(a),
        before,
        "Reading givens must retain every progress field and exact receipts"
    );
    assert_eq!(
        serde_json::to_value(&a.scenarios[a.selected.unwrap()].goal).unwrap(),
        goal
    );
    out
}
fn given_text_geometry(
    shape: &egui::epaint::Shape,
    text: &str,
) -> Option<(egui::Rect, egui::FontId)> {
    match shape {
        egui::epaint::Shape::Text(t) if painted_nfs_text(shape, text) => Some((
            egui::Rect::from_min_size(t.pos, t.galley.size()),
            t.galley.job.sections[0].format.font_id.clone(),
        )),
        egui::epaint::Shape::Vec(v) => v.iter().find_map(|s| given_text_geometry(s, text)),
        _ => None,
    }
}
fn visible_given_text(
    out: &egui::FullOutput,
    ctx: &egui::Context,
    a: &LabApp,
    text: &str,
) -> Option<egui::Rect> {
    let root = a
        .metrics
        .main_viewport
        .unwrap()
        .intersect(ctx.content_rect());
    out.shapes.iter().find_map(|s| {
        let (r, font) = given_text_geometry(&s.shape, text)?;
        let clip = s.clip_rect.intersect(root);
        if clip.contains(r.center()) && clip.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            assert_eq!(
                font,
                egui::TextStyle::Body.resolve(&ctx.style()),
                "Givens stay in normal readable body text"
            );
            Some(r)
        } else {
            None
        }
    })
}
fn relevant_given_paint(
    shape: &egui::epaint::Shape,
    target: &str,
    found: &mut Vec<(String, egui::Rect, egui::FontId)>,
) {
    if found.len() >= 16 {
        return;
    }
    match shape {
        egui::epaint::Shape::Text(t) => {
            let text = &t.galley.job.text;
            if (text == target
                || text.contains("IPv4")
                || text.parse::<std::net::Ipv4Addr>().is_ok())
                && let Some((r, font)) = given_text_geometry(shape, text)
            {
                found.push((text.chars().take(256).collect(), r, font));
            }
        }
        egui::epaint::Shape::Vec(v) => {
            for child in v {
                relevant_given_paint(child, target, found);
            }
        }
        _ => {}
    }
}
fn seek_given_text(
    ctx: &egui::Context,
    a: &mut LabApp,
    text: &str,
) -> (egui::FullOutput, egui::Rect) {
    let mut last = std::collections::VecDeque::new();
    for tick in 0..20 {
        let out = given_capture(ctx, a);
        if let Some(r) = visible_given_text(&out, ctx, a, text) {
            return (out, r);
        }
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let mut paint = Vec::new();
        for clipped in &out.shapes {
            let mut rows = Vec::new();
            relevant_given_paint(&clipped.shape, text, &mut rows);
            for row in rows {
                if paint.len() < 16 {
                    paint.push((row, clipped.clip_rect));
                }
            }
        }
        if last.len() == 3 {
            last.pop_front();
        }
        let session = if learning(a).view == View::Example && !chosen(a).challenge {
            &learning(a).session
        } else {
            chosen(a)
                .workspace
                .as_ref()
                .unwrap()
                .nfs_session
                .as_ref()
                .unwrap()
        };
        let raw_request = (
            &session.server,
            &session.client,
            &session.mountpoint,
            &session.profile,
            session.relative.len(),
            session.write_text.len(),
            session.entries.len(),
        );
        last.push_back(format!("tick={tick} scenario={} challenge={} view={:?} scale={} root={root:?} content={:?} offset={:?} size={:?} presented={:?} selections={:?} popup={:?} session={:?} paint={paint:?}", a.scenarios[a.selected.unwrap()].id, chosen(a).challenge, learning(a).view, a.scale, ctx.content_rect(), a.metrics.main_scroll_offset, a.metrics.main_content_size, a.nfs_presentation.observed_request_selection(), a.metrics.nfs.selections, a.metrics.nfs.popup, raw_request));
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -240.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            draw(ctx, a);
        }
    }
    panic!(
        "Actual supplied label did not become fully visible within20 scrolls: {text}; last3actual observations={last:?}"
    );
}
fn choose_given(ctx: &egui::Context, a: &mut LabApp, key: &str, id: &str, practice: bool) {
    let selected = a.scenarios[a.selected.unwrap()].id.clone();
    let mut before = complete_nfs_progress(a);
    let (server, client, mountpoint, profile) = a.nfs_presentation.observed_request_selection();
    let mut proposed = [
        server.to_owned(),
        client.to_owned(),
        mountpoint.to_owned(),
        profile.to_owned(),
    ];
    let field = match key {
        "server" => 0,
        "client" => 1,
        _ => unreachable!(),
    };
    let changed = proposed[field] != id;
    if changed {
        proposed[field] = id.into();
        if key == "client" {
            let client = live(a, practice)
                .inventory()
                .clients
                .iter()
                .find(|c| c.id == id)
                .unwrap();
            if !client.mountpoints.contains(&proposed[2]) {
                proposed[2].clear();
            }
            if !client.profiles.contains(&proposed[3]) {
                proposed[3].clear();
            }
        }
    }
    choose(ctx, a, key, id);
    let presented = a.nfs_presentation.observed_request_selection();
    assert_eq!(
        if key == "server" {
            presented.0
        } else {
            presented.1
        },
        id,
        "Actual presented request after real choice must match requested {key}; scenario={selected} practice={practice} selected={presented:?} popup={:?} choices={:?}",
        a.metrics.nfs.popup,
        a.metrics.nfs.choices
    );
    let session = if practice {
        &learning(a).session
    } else {
        chosen(a)
            .workspace
            .as_ref()
            .unwrap()
            .nfs_session
            .as_ref()
            .unwrap()
    };
    if changed {
        assert_eq!(
            (
                &session.server,
                &session.client,
                &session.mountpoint,
                &session.profile
            ),
            (&proposed[0], &proposed[1], &proposed[2], &proposed[3]),
            "Actual persisted selection/reconcile must match the requested choice and known givens"
        );
        let mut expected_session = if practice {
            before["attempts"][&selected]["workspace"]["nfs_learning"]["session"].clone()
        } else {
            before["attempts"][&selected]["workspace"]["nfs_session"].clone()
        };
        for (name, value) in ["server", "client", "mountpoint", "profile"]
            .into_iter()
            .zip(proposed)
        {
            expected_session[name] = value.into();
        }
        if practice {
            before["attempts"][&selected]["workspace"]["nfs_learning"]["session"] =
                expected_session;
        } else {
            before["attempts"][&selected]["workspace"]["nfs_session"] = expected_session;
        }
    }
    // Popup selection can scroll the containing workspace. Return to the real
    // selector before seeking its immediately following read-only address row.
    let _ = reveal(ctx, a, key, false);
    let presented = a.nfs_presentation.observed_request_selection();
    assert_eq!(
        if key == "server" {
            presented.0
        } else {
            presented.1
        },
        id
    );
    assert_eq!(
        complete_nfs_progress(a),
        before,
        "Same-option clicks retain the entire Session; genuine selections and actual selector reveal change only independently predicted selection/reconcile fields, never relative/write text, histories, model, documents or receipts"
    );
}

#[test]
fn normal_nfs_given_addresses_follow_actual_selection_in_both_modes_and_separate_practice() {
    for (id, _) in NORMAL_WRITE_TASKS {
        for mode in 0..3 {
            let (ctx, mut a, i) = normal_nfs_given_mode(id, mode);
            let practice = mode == 2;
            let prefix = if practice { "198.51.100" } else { "192.0.2" };
            let servers = live(&a, practice)
                .inventory()
                .servers
                .iter()
                .map(|s| (s.id.clone(), s.address.to_string()))
                .collect::<Vec<_>>();
            let clients = live(&a, practice)
                .inventory()
                .clients
                .iter()
                .map(|s| (s.id.clone(), s.address.to_string()))
                .collect::<Vec<_>>();
            for (n, (server, address)) in servers.iter().enumerate() {
                assert_eq!(*address, format!("{prefix}.{}", 10 + n));
                choose_given(&ctx, &mut a, "server", server, practice);
                let (out, value) = seek_given_text(&ctx, &mut a, address);
                let label = visible_given_text(&out, &ctx, &a, "Server IPv4").unwrap();
                assert!(
                    (label.center().y - value.center().y).abs() < 1.,
                    "Shared label/value row centers match"
                );
            }
            for (client, address) in clients {
                let octet = if client.ends_with("-reader") {
                    21
                } else if client.ends_with("-vault") {
                    40
                } else if client.ends_with("-outside") {
                    90
                } else {
                    20
                };
                assert_eq!(address, format!("{prefix}.{octet}"));
                choose_given(&ctx, &mut a, "client", &client, practice);
                let (out, value) = seek_given_text(&ctx, &mut a, &address);
                let label = visible_given_text(&out, &ctx, &a, "Client IPv4").unwrap();
                assert!((label.center().y - value.center().y).abs() < 1.);
                if a.metrics.nfs.columns {
                    let server = visible_given_text(&out, &ctx, &a, "Server IPv4").unwrap();
                    assert!(
                        (server.center().y - label.center().y).abs() < 1.,
                        "Both Docs columns reserve the same given row"
                    );
                }
            }
            assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
        }
    }
}
#[test]
fn normal_nfs_optional_inventory_reveals_every_supplied_address_and_numeric_profile_id() {
    for (id, _) in NORMAL_WRITE_TASKS {
        for mode in 0..3 {
            let (ctx, mut a, i) = normal_nfs_given_mode(id, mode);
            let practice = mode == 2;
            a.scale = [1., 1.5, 2.][mode];
            for _ in 0..3 {
                given_capture(&ctx, &mut a);
            }
            let mut expected = Vec::new();
            let inventory = live(&a, practice).inventory();
            for s in &inventory.servers {
                expected.push(format!("Server {} · IPv4 {}", s.id, s.address));
            }
            for c in &inventory.clients {
                expected.push(format!("Client {} · IPv4 {}", c.id, c.address));
                expected.push(format!(
                    "Mountpoints: {}; numeric profiles: {}",
                    c.mountpoints.join(", "),
                    c.profiles.join(", ")
                ));
            }
            for p in &inventory.profiles {
                expected.push(format!(
                    "{} ({}) · UID {}, GID {}, additional groups {:?}",
                    p.id, p.label, p.uid, p.gid, p.groups
                ));
            }
            let before = complete_nfs_progress(&a);
            let (_, header) = seek_given_text(&ctx, &mut a, "Inventory and identities · Optional");
            tap(&ctx, &mut a, header);
            for text in expected {
                let _ = seek_given_text(&ctx, &mut a, &text);
            }
            assert_eq!(
                complete_nfs_progress(&a),
                before,
                "Opening and reading optional inventory does not edit anything"
            );
            assert!(!progress::current_solved(chosen(&a), &a.scenarios[i]));
        }
    }
}
