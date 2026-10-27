//! Actual egui events against unregistered teaching, not native-window acceptance.
use super::*;
use dlms_learning_lab::nat_teaching::Learning;
const ID: &str = "nat-teaching-preview";
fn opened() -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let s = pack::load(include_str!(
        "../../../tests/fixtures/nat-teaching-preview.json"
    ))
    .unwrap()
    .remove(0);
    let i = a.scenarios.len();
    a.scenarios.push(s);
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn practice(a: &LabApp) -> &Learning {
    a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .nat_learning
        .as_ref()
        .unwrap()
}
fn rect(a: &LabApp, key: &str, field: bool) -> (egui::Rect, egui::Rect) {
    if field {
        let r = a
            .metrics
            .nat_fields
            .iter()
            .find(|r| r.0 == key)
            .expect("field");
        (r.1, r.4)
    } else {
        let r = a
            .metrics
            .nat_controls
            .iter()
            .find(|r| r.0 == key)
            .expect("control");
        (r.1, r.3)
    }
}
fn reveal(ctx: &egui::Context, a: &mut LabApp, key: &str, field: bool) -> egui::Rect {
    for n in 0..=20 {
        let (r, clip) = rect(a, key, field);
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let child = if key.starts_with("static:") || key.starts_with("remove:") {
            a.metrics.nat_static_viewport
        } else {
            None
        };
        let clip = child.map_or(clip, |c| c.intersect(clip));
        let v = root.intersect(clip);
        if v.contains(r.center()) && v.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        assert!(n < 20, "actual target not revealed: {key} {r:?} {v:?}");
        let inner = child.is_some() && root.contains_rect(clip);
        let port = if inner { v } else { root };
        let y = if r.bottom() > port.bottom() {
            port.bottom() - r.bottom()
        } else {
            port.top() - r.top()
        };
        let pointer = if inner {
            port.center()
        } else {
            egui::pos2(root.left() + 1., root.center().y)
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., y.clamp(-250., 250.)),
                    modifiers: Default::default(),
                },
            ],
        );
        // Real idle frames allow scroll animation to settle; no forced offset or focus.
        for _ in 0..60 {
            frame(ctx, a, 1200., 850., vec![]);
        }
    }
    unreachable!()
}
fn tap(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false);
    click(ctx, a, r);
}
fn input(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let r = reveal(ctx, a, key, true);
    click(ctx, a, r);
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
                modifiers: egui::Modifiers::COMMAND,
            },
            egui::Event::Text(value.into()),
        ],
    );
    draw(ctx, a);
}
fn outer(ctx: &egui::Context, a: &mut LabApp, get: impl Fn(&Metrics) -> egui::Rect) -> egui::Rect {
    for n in 0..=20 {
        let r = get(&a.metrics);
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if root.contains(r.center()) && root.expand(1. / ctx.pixels_per_point()).contains_rect(r) {
            return r;
        }
        assert!(n < 20, "header not visible {r:?} {root:?}");
        let y = if r.bottom() > root.bottom() {
            root.bottom() - r.bottom()
        } else {
            root.top() - r.top()
        };
        frame(
            ctx,
            a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., y.clamp(-300., 300.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..60 {
            frame(ctx, a, 1200., 850., vec![]);
        }
    }
    unreachable!()
}
fn example(ctx: &egui::Context, a: &mut LabApp) {
    let r = outer(ctx, a, |m| m.lesson_tabs[1]);
    click(ctx, a, r);
    assert_eq!(practice(a).view, teaching::View::Example);
}
#[test]
fn introduction_real_tabs_insert_apply_probe_check_and_assignment_separation() {
    let (ctx, mut a, _) = opened();
    assert_eq!(practice(&a).view, teaching::View::Introduction);
    let initial = a.progress.attempts[ID].state.clone();
    example(&ctx, &mut a);
    let before = practice(&a).simulation.clone();
    let r = outer(&ctx, &mut a, |m| m.insert.unwrap());
    click(&ctx, &mut a, r);
    assert_eq!(practice(&a).simulation, before);
    assert!(!practice(&a).draft.matches(&before));
    tap(&ctx, &mut a, "apply");
    assert!(practice(&a).simulation.bindings().is_empty());
    tap(&ctx, &mut a, "outbound");
    assert_eq!(practice(&a).simulation.bindings().len(), 1);
    tap(&ctx, &mut a, "check");
    assert!(practice(&a).checked);
    assert_eq!(a.progress.attempts[ID].state, initial);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[a.scenarios.len() - 1]
    ));
}
#[test]
fn practice_raw_edits_and_checked_rows_persist_across_actual_navigation_and_challenge() {
    let (ctx, mut a, i) = opened();
    example(&ctx, &mut a);
    input(&ctx, &mut a, "network", "unfinished practice prefix");
    tap(&ctx, &mut a, "check");
    let expected = serde_json::to_value(practice(&a)).unwrap();
    a.selected = None;
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(serde_json::to_value(practice(&a)).unwrap(), expected);
    let r = outer(&ctx, &mut a, |m| m.lesson_tabs[2]);
    click(&ctx, &mut a, r);
    assert_eq!(practice(&a).view, teaching::View::Assignment);
    assert_eq!(
        serde_json::to_value(practice(&a)).unwrap()["draft"],
        expected["draft"]
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(a.metrics.nat_controls.iter().any(|r| r.0 == "check"));
    assert!(a.metrics.insert.is_none());
}
#[test]
fn all_scale_practice_typing_focus_undo_and_modal_background_actions() {
    for scale in [1., 1.5, 2.] {
        let (ctx, mut a, _) = opened();
        example(&ctx, &mut a);
        a.scale = scale;
        draw(&ctx, &mut a);
        let before = practice(&a).draft.network.clone();
        input(&ctx, &mut a, "network", "10.80.0.8");
        assert_eq!(practice(&a).draft.network, "10.80.0.8");
        let id = a
            .metrics
            .nat_fields
            .iter()
            .find(|r| r.0 == "network")
            .unwrap()
            .2;
        assert!(ctx.memory(|m| m.has_focus(id)));
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Key {
                key: egui::Key::Z,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }],
        );
        draw(&ctx, &mut a);
        assert_eq!(practice(&a).draft.network, before);
        let state = practice(&a).simulation.clone();
        a.theme_preview = true;
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::Text("blocked".into()),
                egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
            ],
        );
        assert_eq!(practice(&a).simulation, state);
        assert_eq!(practice(&a).draft.network, before);
        assert!(a.metrics.nat_controls.iter().all(|r| !r.2));
    }
}
#[test]
fn example_reset_is_scoped_and_legacy_missing_metadata_starts_assignment() {
    let (ctx, mut a, i) = opened();
    example(&ctx, &mut a);
    let assignment = serde_json::to_value(&a.progress.attempts[ID].state).unwrap();
    let r = outer(&ctx, &mut a, |m| m.insert.unwrap());
    click(&ctx, &mut a, r);
    tap(&ctx, &mut a, "apply");
    a.progress.favorites.insert(ID.into());
    a.lesson_scroll_reset = true;
    draw(&ctx, &mut a);
    let r = outer(&ctx, &mut a, |m| m.example_reset.unwrap());
    click(&ctx, &mut a, r);
    assert_eq!(
        practice(&a).simulation,
        a.scenarios[i].nat_teaching.as_ref().unwrap().fixture
    );
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[ID].state).unwrap(),
        assignment
    );
    assert!(a.progress.favorites.contains(ID));
    a.progress
        .attempts
        .get_mut(ID)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .nat_learning = None;
    a.selected = None;
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(practice(&a).view, teaching::View::Assignment);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
}
#[test]
fn practice_modal_guards_cover_reset_settings_appearance_and_real_file_chooser() {
    for modal in 0..4 {
        let (ctx, mut a, _) = opened();
        example(&ctx, &mut a);
        input(&ctx, &mut a, "network", "10.80.0.8");
        let before = serde_json::to_value(&a.progress.attempts[ID]).unwrap();
        match modal {
            0 => a.reset_pending = true,
            1 => a.settings_open = true,
            2 => a.theme_preview = true,
            _ => {
                a.settings_open = true;
                a.settings_data = true;
                draw(&ctx, &mut a);
                let r = a.metrics.data_export.unwrap();
                click(&ctx, &mut a, r);
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
                egui::Event::Text("hidden edit".into()),
                acl_ctrl(egui::Key::Enter),
            ],
        );
        draw(&ctx, &mut a);
        assert!(a.metrics.nat_fields.iter().all(|r| !r.3));
        assert!(a.metrics.nat_controls.iter().all(|r| !r.2));
        assert!(
            serde_json::to_value(&a.progress.attempts[ID]).unwrap() == before,
            "modal {modal} changed practice/assignment"
        );
    }
}
#[test]
fn practice_reflow_paste_undo_and_assignment_input_have_separate_actual_ids() {
    let (ctx, mut a, _) = opened();
    example(&ctx, &mut a);
    input(&ctx, &mut a, "network", "10.80.0.8");
    let previous = practice(&a).draft.network.clone();
    let id = a
        .metrics
        .nat_fields
        .iter()
        .find(|r| r.0 == "network")
        .unwrap()
        .2;
    let r = reveal(&ctx, &mut a, "network", true);
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Paste("é".repeat(40))],
    );
    assert_eq!(practice(&a).draft.network, previous);
    frame(&ctx, &mut a, 650., 850., vec![]);
    assert_eq!(
        a.metrics
            .nat_fields
            .iter()
            .find(|r| r.0 == "network")
            .unwrap()
            .2,
        id
    );
    assert!(ctx.memory(|m| m.has_focus(id)));
    draw(&ctx, &mut a);
    let r = outer(&ctx, &mut a, |m| m.lesson_tabs[2]);
    click(&ctx, &mut a, r);
    let assignment_id = a
        .metrics
        .nat_fields
        .iter()
        .find(|r| r.0 == "network")
        .unwrap()
        .2;
    assert_ne!(assignment_id, id);
    input(&ctx, &mut a, "network", "assignment unfinished");
    let assignment = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .nat_draft
        .as_ref()
        .unwrap()
        .network
        .clone();
    example(&ctx, &mut a);
    let r = reveal(&ctx, &mut a, "network", true);
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    assert_eq!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .nat_draft
            .as_ref()
            .unwrap()
            .network,
        assignment
    );
}
