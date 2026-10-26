//! Actual egui controls, not native/device or novice acceptance. Reference text
//! is trusted fixture input; it is never installed as a solved State by a flow.
use super::*;
use dlms_learning_lab::{
    apache_static::{self as web, Action},
    apache_static_teaching::Learning,
    apache_static_workspace::{self as work, Draft, EditorKind},
};
const ID: &str = "test-apache-static-workspace";
fn same(a: &impl serde::Serialize, b: &impl serde::Serialize, label: &'static str) {
    let left = serde_json::to_value(a).unwrap();
    let right = serde_json::to_value(b).unwrap();
    assert!(
        left == right,
        "{label}: complete serialized fields differ (values withheld)"
    );
}
fn opened(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let fixture = pack::load(include_str!(
        "../../../tests/fixtures/apache-static-workspace-preview.json"
    ))
    .unwrap()
    .remove(0);
    assert!(!a.scenarios.iter().any(|s| s.id == ID));
    let i = a.scenarios.len();
    a.scenarios.push(fixture);
    a.launch(i);
    a.set_apache_static_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn draft(a: &LabApp) -> &Draft {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .apache_static_draft
        .as_ref()
        .unwrap()
}
fn learning(a: &LabApp) -> &Learning {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .apache_static_learning
        .as_ref()
        .unwrap()
}
fn simulation(a: &LabApp) -> &web::State {
    let State::ApacheStatic { simulation } = &a.progress.attempts[ID].state else {
        panic!("Apache state")
    };
    simulation
}
fn correct(a: &LabApp, i: usize) -> String {
    let command = &a.scenarios[i].solution[0];
    let Action::SaveConfig { text } =
        serde_json::from_str(command.strip_prefix(work::PREFIX).unwrap()).unwrap()
    else {
        panic!("fixture SaveConfig")
    };
    text
}
fn locate(
    ctx: &egui::Context,
    a: &mut LabApp,
    find: impl Fn(&Metrics) -> Option<egui::Rect>,
) -> egui::Rect {
    for _ in 0..30 {
        let r = find(&a.metrics).expect("actual Apache control exists");
        let v = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if v.contains(r.center()) && v.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        let delta = if r.bottom() > v.bottom() {
            v.bottom() - r.bottom() - 8.
        } else {
            v.top() - r.top() + 8.
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(
                    r.center().x.clamp(v.left() + 1., v.right() - 1.),
                    v.center().y,
                )),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta.clamp(-400., 400.)),
                    modifiers: Default::default(),
                },
            ],
        );
        draw(ctx, a);
    }
    panic!("Apache control remained clipped after bounded actual scrolling");
}
fn control(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = locate(ctx, a, |m| {
        m.apache_controls
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, r, _)| *r)
    });
    click(ctx, a, r);
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, text: &str) {
    a.focus_input = false;
    let r = locate(ctx, a, |m| {
        if key == "config" || key == "file" {
            let (_, v, _, _) = m.apache_editor?;
            // Reveal a real control-height band, not an unfulfillable full tall editor.
            Some(egui::Rect::from_center_size(
                v.center(),
                egui::vec2(v.width(), 32.0_f32.min(v.height())),
            ))
        } else {
            m.apache_fields
                .iter()
                .find(|(k, _, _, _)| k == key)
                .map(|(_, r, _, _)| *r)
        }
    });
    click(ctx, a, r);
    let id = a
        .metrics
        .apache_fields
        .iter()
        .find(|(k, _, _, _)| k == key)
        .unwrap()
        .2;
    assert!(ctx.memory(|m| m.has_focus(id)), "actual {key} text focus");
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    draw(ctx, a);
}
fn check(ctx: &egui::Context, a: &mut LabApp) {
    control(ctx, a, "check");
}
fn repair(ctx: &egui::Context, a: &mut LabApp, i: usize) {
    let text = correct(a, i);
    replace(ctx, a, "config", &text);
    control(ctx, a, "save-config");
    assert!(
        !progress::current_solved(&a.progress.attempts[ID], &a.scenarios[i]),
        "Save alone leaves active old document"
    );
    control(ctx, a, "reload");
    check(ctx, a);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    assert!(a.metrics.completion);
}
#[test]
fn apache_native_intro_example_insert_save_reload_experiment_repair_is_separate() {
    let (ctx, mut a, i) = opened(teaching::View::Introduction);
    let assignment = a.progress.attempts[ID].state.clone();
    assert!(a.metrics.apache_fields.is_empty());
    control(&ctx, &mut a, "try-example");
    assert_eq!(a.apache_static_view(i), teaching::View::Example);
    let initial = learning(&a).simulation.clone();
    control(&ctx, &mut a, "insert");
    assert_eq!(
        learning(&a).simulation,
        initial,
        "Insert does not save or activate"
    );
    assert_ne!(learning(&a).draft.config_text, initial.saved);
    control(&ctx, &mut a, "save-config");
    assert!(
        learning(&a)
            .checks(a.scenarios[i].apache_static_teaching.as_ref().unwrap())
            .iter()
            .any(|r| !r.0)
    );
    control(&ctx, &mut a, "reload");
    check(&ctx, &mut a);
    assert!(
        learning(&a)
            .checks(a.scenarios[i].apache_static_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert!(!a.metrics.completion);
    let good = learning(&a).draft.config_text.clone();
    replace(&ctx, &mut a, "config", &initial.saved);
    control(&ctx, &mut a, "save-config");
    check(&ctx, &mut a);
    assert_eq!(
        learning(&a).simulation.active.as_ref().unwrap().source,
        good
    );
    assert!(
        learning(&a)
            .checks(a.scenarios[i].apache_static_teaching.as_ref().unwrap())
            .iter()
            .any(|r| !r.0)
    );
    replace(&ctx, &mut a, "config", &good);
    control(&ctx, &mut a, "save-config");
    check(&ctx, &mut a);
    assert!(
        learning(&a)
            .checks(a.scenarios[i].apache_static_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, assignment);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn apache_native_assignment_save_reload_query_pending_and_malformed_retry() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    repair(&ctx, &mut a, i);
    let applied = a.progress.attempts[ID].state.clone();
    replace(&ctx, &mut a, "port", "bad!");
    control(&ctx, &mut a, "send-request");
    assert!(draft(&a).query(simulation(&a)).is_err());
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    assert!(a.metrics.completion && !a.metrics.apache_check_evaluated);
    replace(&ctx, &mut a, "config", "UnsupportedDirective yes\n");
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    control(&ctx, &mut a, "save-config");
    let State::ApacheStatic { simulation: old } = &applied else {
        panic!("Apache state")
    };
    assert_eq!(
        simulation(&a).saved,
        "UnsupportedDirective yes\n",
        "Save retains bounded raw text without parsing"
    );
    assert_eq!(
        simulation(&a).active,
        old.active,
        "Save never activates the raw document"
    );
    assert_eq!(draft(&a).config_text, "UnsupportedDirective yes\n");
    let engine::Goal::ApacheStatic { definition } = &a.scenarios[i].goal else {
        panic!("Apache goal")
    };
    let rows = definition.checks(simulation(&a));
    assert!(
        rows[0].0 && !rows[1].0,
        "active positive request still passes; next start of malformed saved text fails"
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    control(&ctx, &mut a, "reload");
    assert_eq!(
        simulation(&a).active,
        old.active,
        "invalid Reload keeps the active copy"
    );
    assert_eq!(simulation(&a).saved, "UnsupportedDirective yes\n");
    let good = correct(&a, i);
    replace(&ctx, &mut a, "config", &good);
    control(&ctx, &mut a, "save-config");
    control(&ctx, &mut a, "reload");
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert_eq!(
        draft(&a).request_port,
        "bad!",
        "read-only query draft is independent"
    );
}
#[test]
fn apache_native_live_file_save_affects_response_without_reload() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    repair(&ctx, &mut a, i);
    control(&ctx, &mut a, "file-tab");
    control(&ctx, &mut a, "file-picker");
    let path = "/srv/report/index.html";
    let r = a
        .metrics
        .apache_file_items
        .iter()
        .find(|(p, _)| p == path)
        .unwrap()
        .1;
    let pane = a
        .metrics
        .apache_popup
        .as_ref()
        .unwrap()
        .1
        .intersect(ctx.content_rect());
    assert!(pane.expand(1. / ctx.pixels_per_point()).contains_rect(r));
    click(&ctx, &mut a, r);
    let active = simulation(&a).active.clone();
    replace(&ctx, &mut a, "file", "Changed live page\n");
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    control(&ctx, &mut a, "save-file");
    assert_eq!(simulation(&a).active, active);
    assert_eq!(simulation(&a).files[path], "Changed live page\n");
    replace(&ctx, &mut a, "port", "8080");
    replace(&ctx, &mut a, "host-0", "report.example.test");
    let web::Outcome::Http(response) = draft(&a).query(simulation(&a)).unwrap() else {
        panic!("active listener")
    };
    assert_eq!(response.body, "Changed live page\n");
    replace(&ctx, &mut a, "file", "Report ready\n");
    control(&ctx, &mut a, "save-file");
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn apache_native_check_is_once_per_event_resumed_once_and_manual_scroll_is_not_revealed_again() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    repair(&ctx, &mut a, i);
    let before = a.progress.attempts[ID].clone();
    for _ in 0..12 {
        frame(&ctx, &mut a, 1200., 850., vec![]);
        assert!(!a.metrics.apache_check_evaluated);
    }
    // The actual editor can be revealed after feedback across multiple idle frames.
    let text = correct(&a, i);
    replace(&ctx, &mut a, "config", &text);
    for _ in 0..8 {
        draw(&ctx, &mut a);
    }
    assert!(
        a.metrics.main_scroll_offset.y < 200.,
        "feedback must not force idle scrolling to the bottom"
    );
    assert_eq!(a.progress.attempts[ID].state, before.state);
    a.invalidate_apache_presentation();
    frame(&ctx, &mut a, 1200., 850., vec![]);
    assert!(
        a.metrics.apache_check_evaluated,
        "resumed checked view recomputes from actual context once"
    );
    frame(&ctx, &mut a, 1200., 850., vec![]);
    assert!(!a.metrics.apache_check_evaluated);
    let r = locate(&ctx, &mut a, |m| m.mode_challenge);
    click(&ctx, &mut a, r);
    assert!(a.progress.attempts[ID].challenge && a.metrics.lesson_tabs.is_empty() && !a.help_open);
    assert!(a.metrics.apache_action_items.is_empty() && a.metrics.insert.is_none());
    let r = locate(&ctx, &mut a, |m| m.help);
    click(&ctx, &mut a, r);
    assert!(a.help_open);
}
#[test]
fn apache_native_raw_drafts_editor_kind_save_resume_and_selected_reset_preserve_other_work() {
    let (ctx, mut a, i) = opened(teaching::View::Example);
    control(&ctx, &mut a, "insert");
    replace(&ctx, &mut a, "port", "example");
    let practice = learning(&a).clone();
    a.set_apache_static_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "config", "unfinished assignment config");
    control(&ctx, &mut a, "file-tab");
    replace(&ctx, &mut a, "port", "bad!");
    let other = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-gateway")
        .unwrap();
    a.launch(other);
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(draft(&a).editor_kind, Some(EditorKind::File));
    assert_eq!(learning(&a).draft, practice.draft);
    a.progress.favorites.insert(ID.into());
    a.dirty = true;
    a.save();
    assert!(a.storage_error.is_none());
    let raw = progress::read_bounded(&a.path, 16_000_000).unwrap();
    let loaded = progress::load(&a.path, &a.scenarios).unwrap();
    same(
        &loaded.attempts[ID],
        &a.progress.attempts[ID],
        "loaded selected Attempt",
    );
    assert_eq!(loaded.favorites, a.progress.favorites);
    assert!(
        progress::read_bounded(&a.path, 16_000_000).unwrap() == raw,
        "profile raw bytes changed during read-only load"
    );
    let other_before = loaded.attempts[&a.scenarios[other].id].clone();
    let selected_before = a.progress.attempts[ID].clone();
    let r = locate(&ctx, &mut a, |m| m.reset);
    click(&ctx, &mut a, r);
    let r = a.metrics.reset_cancel.unwrap();
    click(&ctx, &mut a, r);
    same(
        &a.progress.attempts[ID],
        &selected_before,
        "Cancel retains selected Attempt",
    );
    let r = locate(&ctx, &mut a, |m| m.reset);
    click(&ctx, &mut a, r);
    let r = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.progress.attempts[ID].state, a.scenarios[i].initial);
    same(
        &a.progress.attempts[&a.scenarios[other].id],
        &other_before,
        "Reset retains other Attempt",
    );
    assert!(a.progress.favorites.contains(ID));
}
#[test]
fn apache_native_all_modals_block_actual_focused_edits_shortcut_and_server_controls() {
    for view in [teaching::View::Example, teaching::View::Assignment] {
        for modal in ["reset", "settings", "theme", "data"] {
            let (ctx, mut a, _) = opened(view);
            let id = a
                .metrics
                .apache_fields
                .iter()
                .find(|(k, _, _, _)| k == "config")
                .unwrap()
                .2;
            if modal == "data" {
                a.settings_open = true;
                a.settings_data = true;
                draw(&ctx, &mut a);
                let r = a.metrics.data_export.unwrap();
                click(&ctx, &mut a, r);
                assert!(a.data_tools.modal_open());
                a.settings_open = false;
            }
            a.reset_pending = modal == "reset";
            a.settings_open = modal == "settings";
            a.theme_preview = modal == "theme";
            let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
            ctx.memory_mut(|m| m.request_focus(id));
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    acl_ctrl(egui::Key::A),
                    egui::Event::Text("behind modal".into()),
                    acl_ctrl(egui::Key::Enter),
                ],
            );
            same(
                &a.progress.attempts[ID],
                &before,
                "modal retains complete Attempt",
            );
            let host_id = a
                .metrics
                .apache_fields
                .iter()
                .find(|(key, _, _, _)| key == "host-0")
                .unwrap()
                .2;
            ctx.memory_mut(|m| m.request_focus(host_id));
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![egui::Event::Text("behind-modal.example.test".into())],
            );
            same(
                &a.progress.attempts[ID],
                &before,
                "modal retains absent first Host as well as config",
            );

            assert!(
                a.metrics
                    .apache_fields
                    .iter()
                    .all(|(_, _, _, enabled)| !*enabled)
            );
            assert!(
                a.metrics
                    .apache_controls
                    .iter()
                    .filter(|(k, _, _)| [
                        "save-config",
                        "start",
                        "reload",
                        "stop",
                        "check",
                        "send-request"
                    ]
                    .contains(&k.as_str()))
                    .all(|(_, _, enabled)| !*enabled)
            );
            assert!(!a.metrics.apache_check_evaluated);
        }
    }
}
#[test]
fn apache_native_editor_and_request_ids_bounds_and_focus_survive_scale_and_reflow() {
    for view in [teaching::View::Example, teaching::View::Assignment] {
        for scale in [1., 1.5, 2.] {
            let (ctx, mut a, _) = opened(view);
            a.scale = scale;
            let mut ids = None;
            for width in [1600., 720., 1600.] {
                for _ in 0..6 {
                    frame(&ctx, &mut a, width, 929., vec![]);
                }
                let clip = a.metrics.main_viewport.unwrap();
                for r in &a.metrics.inputs {
                    assert!(
                        r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                        "input {r:?} in {clip:?} at {width}/{scale}"
                    );
                }
                for (key, r, _) in &a.metrics.apache_controls {
                    assert!(
                        r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                        "{key}: {r:?} in {clip:?}"
                    );
                }
                let now = a
                    .metrics
                    .apache_fields
                    .iter()
                    .map(|(k, _, id, _)| (k.clone(), *id))
                    .collect::<Vec<_>>();
                if let Some(before) = &ids {
                    assert_eq!(&now, before, "field identity crosses responsive containers");
                } else {
                    ids = Some(now);
                }
                let port = a
                    .metrics
                    .apache_fields
                    .iter()
                    .find(|(k, _, _, _)| k == "port")
                    .unwrap();
                let path = a
                    .metrics
                    .apache_fields
                    .iter()
                    .find(|(k, _, _, _)| k == "path")
                    .unwrap();
                assert!((port.1.left() - path.1.left()).abs() < 1.);
            }
        }
    }
}
#[test]
fn apache_native_header_ids_and_order_remain_stable_and_legacy_workspace_unchanged() {
    for id in ["ccna-gateway", ID] {
        let (ctx, mut a, i) = opened(teaching::View::Assignment);
        let i = if id == ID {
            i
        } else {
            a.scenarios.iter().position(|s| s.id == id).unwrap()
        };
        a.launch(i);
        draw(&ctx, &mut a);
        let first = a.metrics.workspace_header_controls.clone();
        assert_eq!(
            first.iter().map(|(k, _, _)| k.as_str()).collect::<Vec<_>>(),
            vec!["back", "learn", "challenge", "reset"]
        );
        for _ in 0..8 {
            draw(&ctx, &mut a);
            assert_eq!(a.metrics.workspace_header_controls, first);
        }
        assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
    }
}

#[test]
fn apache_native_actual_restore_discards_old_check_display_and_keeps_imported_drafts() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    a.smoke = true;
    // Export the real unsolved attempt before solving through controls.
    let backup_path = a.path.parent().unwrap().join("dlms-learning-backup.json");
    dlms_learning_lab::backup::export(&backup_path, &a.progress, &a.scenarios).unwrap();
    let imported = a.progress.attempts[ID].clone();
    repair(&ctx, &mut a, i);
    assert!(a.metrics.completion);
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, r);
    same(
        &a.progress.attempts[ID],
        &imported,
        "restored selected Attempt",
    );
    assert!(a.selected.is_none());
    a.settings_open = false;
    a.launch(i);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion && !a.metrics.apache_check_evaluated);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn apache_native_mode_focus_undo_and_oversize_paste_are_isolated() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    let initial = draft(&a).config_text.clone();
    replace(&ctx, &mut a, "config", "assignment raw");
    let assignment_id = a
        .metrics
        .apache_fields
        .iter()
        .find(|(k, _, _, _)| k == "config")
        .unwrap()
        .2;
    let state = a.progress.attempts[ID].state.clone();
    replace(&ctx, &mut a, "config", &"é".repeat(web::MAX_CONFIG));
    assert_eq!(
        draft(&a).config_text,
        "assignment raw",
        "whole oversized paste rolls back selection deletion"
    );
    assert_eq!(a.progress.attempts[ID].state, state);
    a.set_apache_static_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    replace(&ctx, &mut a, "config", "example raw");
    let example_id = a
        .metrics
        .apache_fields
        .iter()
        .find(|(k, _, _, _)| k == "config")
        .unwrap()
        .2;
    assert_ne!(example_id, assignment_id);
    for _ in 0..3 {
        frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(draft(&a).config_text, "assignment raw");
        if learning(&a).draft.config_text != "example raw" {
            break;
        }
    }
    assert_ne!(
        learning(&a).draft.config_text,
        "example raw",
        "real Undo restores practice only"
    );
    assert_ne!(initial, "assignment raw");
    a.set_apache_static_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics
            .apache_fields
            .iter()
            .find(|(k, _, _, _)| k == "config")
            .unwrap()
            .2,
        assignment_id
    );
    assert_eq!(draft(&a).config_text, "assignment raw");
}

#[test]
fn apache_native_main_host_is_edit_only_and_advanced_retains_exact_header_shapes() {
    let (ctx, mut a, i) = opened(teaching::View::Assignment);
    assert!(draft(&a).request_hosts.is_empty());
    assert!(
        a.metrics
            .apache_fields
            .iter()
            .any(|(key, _, _, _)| key == "host-0")
    );
    assert!(
        !a.metrics
            .apache_fields
            .iter()
            .any(|(key, _, _, _)| key == "host-1")
    );
    assert!(
        !a.metrics
            .apache_controls
            .iter()
            .any(|(key, _, _)| key == "host-add" || key == "host-remove")
    );
    let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
    for _ in 0..8 {
        draw(&ctx, &mut a);
        same(
            &a.progress.attempts[ID],
            &before,
            "idle must not synthesize a Host header",
        );
    }
    repair(&ctx, &mut a, i);
    let state = a.progress.attempts[ID].state.clone();
    let id = a
        .metrics
        .apache_fields
        .iter()
        .find(|(key, _, _, _)| key == "host-0")
        .unwrap()
        .2;
    replace(&ctx, &mut a, "host-0", &"é".repeat(261));
    assert!(
        draft(&a).request_hosts.is_empty(),
        "oversized paste cannot synthesize/truncate the first header"
    );
    same(
        &a.progress.attempts[ID].state,
        &state,
        "read-only paste failure keeps applied state",
    );
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    assert!(
        a.metrics.completion,
        "read-only request error retains last Check display"
    );
    replace(&ctx, &mut a, "host-0", "report.example.test");
    assert_eq!(draft(&a).request_hosts, vec!["report.example.test"]);
    assert_eq!(
        a.metrics
            .apache_fields
            .iter()
            .find(|(key, _, _, _)| key == "host-0")
            .unwrap()
            .2,
        id
    );
    let web::Outcome::Http(response) = draft(&a).query(simulation(&a)).unwrap() else {
        panic!("active server")
    };
    assert_eq!(response.status.code(), 200);
    control(&ctx, &mut a, "advanced-hosts");
    control(&ctx, &mut a, "host-add");
    replace(&ctx, &mut a, "host-1", "observer.example.test");
    assert_eq!(
        draft(&a).request_hosts,
        vec!["report.example.test", "observer.example.test"]
    );
    control(&ctx, &mut a, "advanced-hosts");
    // egui keeps the closing body during its ordinary animation. Observe the
    // collapsed steady state after advancing frames; do not disable animation.
    for _ in 0..12 {
        frame(&ctx, &mut a, 1200., 850., vec![]);
    }
    assert!(
        !a.metrics
            .apache_fields
            .iter()
            .any(|(key, _, _, _)| key == "host-1")
    );
    assert!(
        !a.metrics
            .apache_controls
            .iter()
            .any(|(key, _, _)| key == "host-add" || key == "host-remove")
    );
    let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    assert!(
        // egui exposes a static Label's text as its AccessKit value.
        tree.nodes.iter().any(|(_, node)| node.value()
            == Some("Two Host fields are active · inspect Advanced request fields.")),
        "retained duplicate headers must be disclosed while advanced controls are collapsed"
    );
    assert_eq!(draft(&a).request_hosts.len(), 2);
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    control(&ctx, &mut a, "advanced-hosts");
    control(&ctx, &mut a, "host-remove");
    control(&ctx, &mut a, "host-remove");
    assert!(draft(&a).request_hosts.is_empty());
    control(&ctx, &mut a, "host-add");
    assert_eq!(
        draft(&a).request_hosts,
        vec![String::new()],
        "one empty header is not normalized to no header"
    );
    assert!(
        draft(&a)
            .query(simulation(&a))
            .unwrap_err()
            .contains("outside the literal DNS subset"),
        "an empty Host value is an explicit excluded form, not a fabricated HTTP400"
    );
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
