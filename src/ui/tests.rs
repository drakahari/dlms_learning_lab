use super::*;
use engine::State;
mod apache_static;
mod apache_static_lessons;
mod client_learning;
mod command_vectors;
mod debian_packages;
mod dhcpv4;
mod dhcpv4_lessons;
mod getopts_history;
mod getopts_lessons;
mod mac_learning;
mod nat;
mod nat_lessons;
mod nat_lessons_native;
mod nat_native;
mod nat_teaching;
mod nfs_lessons_native;
pub(in crate::ui) mod nfs_workspace;
mod ospf;
mod safe_automation_lessons;
mod seed_learning;
mod spanning_tree;
mod ui_consistency;
#[test]
fn opening_terminal_assignment_keeps_task_visible_and_scrolling_does_not_grow_output() {
    for scale in [1., 1.5, 2.] {
        let (ctx, mut a) = app();
        a.scale = scale;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-search-release-set")
            .unwrap();
        a.launch(i);
        a.set_terminal_view(i, teaching::View::Assignment);
        for _ in 0..6 {
            frame(&ctx, &mut a, 1268., 632., vec![]);
        }
        assert!(
            a.metrics.main_scroll_offset.y < 1.,
            "opening must show the beginning of the task at {scale}: {:?}",
            a.metrics.main_scroll_offset
        );
        let before = a.metrics.output.unwrap().height();
        let viewport = a.metrics.main_viewport.unwrap();
        frame(
            &ctx,
            &mut a,
            1268.,
            632.,
            vec![
                egui::Event::PointerMoved(egui::pos2(viewport.left() + 1., viewport.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -220.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..15 {
            frame(&ctx, &mut a, 1268., 632., vec![]);
        }
        assert!(
            a.metrics.main_scroll_offset.y > 1.,
            "test must actually scroll"
        );
        assert!(
            (a.metrics.output.unwrap().height() - before).abs() < 1.,
            "scrolling must not resize the terminal at {scale}"
        );
    }
}
// Cache only trusted compile-time catalog validation. Every app receives owned
// clones; contexts, profiles, progress, workspaces and fixture injection stay fresh.
pub(super) fn validated_ui_catalog() -> (Vec<Scenario>, Vec<course::Unit>) {
    static CATALOG: std::sync::OnceLock<(Vec<Scenario>, Vec<course::Unit>)> =
        std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let scenarios = pack::bundled().expect("Bundled pack validates");
            let units = course::bundled(&scenarios).expect("Bundled units validate");
            (scenarios, units)
        })
        .clone()
}
fn app() -> (egui::Context, LabApp) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context(&ctx, true, false);
    a.smoke = false;
    (ctx, a)
}
#[test]
fn ui_catalog_cache_clones_do_not_share_scenarios_units_or_app_state() {
    let baseline = serde_json::to_vec(&validated_ui_catalog()).unwrap();
    let (first_ctx, mut first) = app();
    let (second_ctx, mut second) = app();
    assert_ne!(
        first.path, second.path,
        "each app needs its own profile path"
    );
    assert_ne!(
        first._smoke_profile.as_ref().unwrap().path(),
        second._smoke_profile.as_ref().unwrap().path()
    );
    assert!(first.progress.attempts.is_empty() && second.progress.attempts.is_empty());
    assert!(first.progress.favorites.is_empty() && second.progress.favorites.is_empty());
    let i = first
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-gateway")
        .unwrap();
    let id = first.scenarios[i].id.clone();
    first.launch(i);
    first.progress.favorites.insert(id.clone());
    first
        .progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .client_draft
        .as_mut()
        .unwrap()
        .address = "unfinished first app".into();
    assert!(second.progress.attempts.is_empty() && second.progress.favorites.is_empty());
    second.launch(i);
    assert_ne!(
        first.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .client_draft
            .as_ref()
            .unwrap()
            .address,
        second.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .client_draft
            .as_ref()
            .unwrap()
            .address
    );
    first.scenarios[i].title.push_str(" first app edit");
    first.scenarios[i].solution.push("first app only".into());
    let State::Network { address, .. } = &mut first.scenarios[i].initial else {
        panic!("supplied gateway scenario kind");
    };
    *address = "192.0.2.222".parse().unwrap();
    first.units[0].title.push_str(" first app edit");
    first.units[0].labs.clear();
    first.units[0].prerequisites.push("first-app-only".into());
    assert!(
        serde_json::to_vec(&(&first.scenarios, &first.units)).unwrap() != baseline,
        "mutation setup must differ from the trusted catalog"
    );
    assert!(
        serde_json::to_vec(&(&second.scenarios, &second.units)).unwrap() == baseline,
        "scenario/initial/solution or unit mutations leaked into another app"
    );
    assert!(
        serde_json::to_vec(&validated_ui_catalog()).unwrap() == baseline,
        "app mutations leaked into the retained trusted base"
    );
    let second_spacing = second_ctx.style().spacing.item_spacing;
    first_ctx.style_mut(|s| s.spacing.item_spacing.x += 7.);
    assert_eq!(second_ctx.style().spacing.item_spacing, second_spacing);
    assert!(second.progress.favorites.is_empty());
}
fn accounts_fixture_app(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let (ctx, mut a) = app();
    let i = a.add_accounts_fixture();
    a.launch(i);
    a.set_accounts_view(i, view);
    (ctx, a, i)
}
fn accounts_ui_click(
    ctx: &egui::Context,
    a: &mut LabApp,
    target: fn(&Metrics) -> Option<egui::Rect>,
) {
    for _ in 0..20 {
        let r = target(&a.metrics).expect("account control present");
        let viewport = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if r.top() >= viewport.top() && r.bottom() <= viewport.bottom() {
            click(ctx, a, r);
            return;
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
                egui::Event::PointerMoved(egui::pos2(viewport.left() + 1., viewport.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            frame(ctx, a, 1200., 850., vec![]);
        }
    }
    panic!("Account control did not become visible");
}
fn accounts_ui_command(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    a.focus_input = true;
    draw(ctx, a);
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    frame(ctx, a, 1200., 850., vec![key(egui::Key::Enter, false)]);
    draw(ctx, a);
}
#[test]
fn accounts_native_intro_practice_insert_execute_try_repair_and_assignment_are_isolated() {
    let (ctx, mut a, i) = accounts_fixture_app(teaching::View::Introduction);
    let s = a.scenarios[i].clone();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    draw(&ctx, &mut a);
    assert!(a.metrics.terminal.is_none());
    assert_eq!(a.metrics.lesson_tabs.len(), 3);
    accounts_ui_click(&ctx, &mut a, |m| m.example_action);
    assert_eq!(a.accounts_view(i), teaching::View::Example);
    let input_id = a.metrics.focus.unwrap();
    accounts_ui_click(&ctx, &mut a, |m| m.insert);
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .accounts_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.command, lesson.example[0]);
    assert_eq!(l.simulation, lesson.fixture);
    assert!(l.transcript.is_empty());
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    assert_eq!(ctx.memory(|m| m.focused()), Some(input_id));
    for c in &lesson.example {
        accounts_ui_command(&ctx, &mut a, c);
    }
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .accounts_learning
        .as_ref()
        .unwrap();
    assert!(l.checked);
    assert!(l.checks(lesson).iter().all(|c| c.0));
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    for c in &lesson.try_actions {
        accounts_ui_command(&ctx, &mut a, c);
    }
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .accounts_learning
        .as_ref()
        .unwrap();
    assert!(l.checks(lesson).iter().any(|c| !c.0));
    for c in &lesson.repair {
        accounts_ui_command(&ctx, &mut a, c);
    }
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(
        a.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .accounts_learning
            .as_ref()
            .unwrap()
            .checks(lesson)
            .iter()
            .all(|c| c.0)
    );
    accounts_ui_click(&ctx, &mut a, |m| m.example_reset);
    let l = a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .accounts_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.simulation, lesson.fixture);
    assert!(l.command.is_empty() && l.transcript.is_empty() && !l.checked);
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    a.set_accounts_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_ne!(a.metrics.focus, Some(input_id));
    assert!(a.input.is_empty());
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
}
#[test]
fn accounts_native_alternate_partial_wrong_repair_and_modes_use_current_outcomes() {
    let (ctx, mut a, i) = accounts_fixture_app(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(!a.metrics.completion);
    accounts_ui_command(&ctx, &mut a, "usermod -g review mira");
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(!a.metrics.completion);
    for c in [
        "groupadd --gid 2200 reports",
        "usermod -G reports,audit,reports mira",
        "id mira",
    ] {
        accounts_ui_command(&ctx, &mut a, c);
    }
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(a.metrics.completion);
    a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
    accounts_ui_command(&ctx, &mut a, "usermod -G reports mira");
    assert!(!a.metrics.completion);
    accounts_ui_command(&ctx, &mut a, "usermod -G audit,reports mira");
    assert!(a.metrics.completion);
    let before = a.progress.attempts[&s.id].state.clone();
    accounts_ui_command(&ctx, &mut a, "userdel root");
    assert_eq!(a.progress.attempts[&s.id].state, before);
    assert!(a.metrics.completion);
    accounts_ui_command(&ctx, &mut a, "usermod -G");
    assert_eq!(a.progress.attempts[&s.id].state, before);
    let diagnostic = a.progress.attempts[&s.id].transcript.last().unwrap();
    assert!(diagnostic.len() > "Virtual administrator # usermod -G".len() + 10);
    assert!(a.metrics.completion);
    a.progress.attempts.get_mut(&s.id).unwrap().challenge = false;
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn accounts_native_input_limits_focus_and_undo_are_separate_between_contexts_and_labs() {
    let (ctx, mut a, i) = accounts_fixture_app(teaching::View::Assignment);
    draw(&ctx, &mut a);
    let assignment_id = a.metrics.focus.unwrap();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("é".repeat(1100))],
    );
    draw(&ctx, &mut a);
    assert_eq!(a.input.len(), 2048);
    assert!(a.input.is_char_boundary(a.input.len()));
    let id = a.scenarios[i].id.clone();
    assert_eq!(
        a.progress.attempts[&id].workspace.as_ref().unwrap().command,
        a.input
    );
    a.set_accounts_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let example_id = a.metrics.focus.unwrap();
    assert_ne!(example_id, assignment_id);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("id tess".into())],
    );
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(example_id));
    a.set_accounts_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.focus, Some(assignment_id));
    assert_eq!(a.input.len(), 2048);
    let old = a
        .scenarios
        .iter()
        .position(|s| matches!(s.initial, State::Filesystem { .. }))
        .unwrap();
    open_assignment(&mut a, old);
    draw(&ctx, &mut a);
    assert_ne!(a.metrics.focus, Some(assignment_id));
    assert_ne!(a.metrics.focus, Some(example_id));
    assert!(a.input.is_empty());
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    draw(&ctx, &mut a);
    assert!(a.input.is_empty());
    a.launch(i);
    a.set_accounts_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .accounts_learning
            .as_ref()
            .unwrap()
            .command,
        "id tess"
    );
}
#[test]
fn accounts_native_exact_resume_and_reset_only_selected_activity() {
    let (ctx, mut a, i) = accounts_fixture_app(teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    solve(&mut a, i);
    draw(&ctx, &mut a);
    accounts_ui_click(&ctx, &mut a, |m| m.check);
    assert!(a.metrics.completion);
    a.input = "usermod -G ".into();
    a.progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = a.input.clone();
    let l = a
        .progress
        .attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .accounts_learning
        .as_mut()
        .unwrap();
    l.command = "getent group ".into();
    let old = a
        .scenarios
        .iter()
        .find(|s| matches!(s.initial, State::Filesystem { .. }))
        .unwrap()
        .clone();
    a.progress
        .attempts
        .insert(old.id.clone(), Attempt::new(&old, 5));
    a.progress.favorites.insert(s.id.clone());
    let other = serde_json::to_vec(&a.progress.attempts[&old.id]).unwrap();
    let mut expected = serde_json::to_value(&a.progress.attempts[&s.id]).unwrap();
    let bytes = serde_json::to_vec(&a.progress).unwrap();
    a.progress = progress::decode(&bytes, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&s.id]).unwrap(),
        expected
    );
    a.launch(i);
    expected["visited"] = serde_json::json!(a.progress.clock);
    draw(&ctx, &mut a);
    assert_eq!(
        serde_json::to_value(&a.progress.attempts[&s.id]).unwrap(),
        expected
    );
    assert_eq!(a.input, "usermod -G ");
    assert!(a.metrics.completion);
    a.reset_attempt(i);
    draw(&ctx, &mut a);
    assert_eq!(a.accounts_view(i), teaching::View::Introduction);
    assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    assert!(!a.metrics.completion);
    assert!(a.input.is_empty());
    assert_eq!(
        serde_json::to_vec(&a.progress.attempts[&old.id]).unwrap(),
        other
    );
    assert!(a.progress.favorites.contains(&s.id));
}
#[test]
fn accounts_native_reflows_at_all_scales_with_optional_challenge_help() {
    for (width, height) in [(1200., 850.), (720., 560.)] {
        for scale in [1., 1.5, 2.] {
            for (view, challenge) in [
                (teaching::View::Introduction, false),
                (teaching::View::Example, false),
                (teaching::View::Assignment, false),
                (teaching::View::Assignment, true),
            ] {
                let (ctx, mut a, i) = accounts_fixture_app(view);
                a.scale = scale;
                let id = a.scenarios[i].id.clone();
                a.progress.attempts.get_mut(&id).unwrap().challenge = challenge;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                for rect in a
                    .metrics
                    .lesson_tabs
                    .iter()
                    .chain(a.metrics.terminal.iter())
                    .chain(a.metrics.output.iter())
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.accounts_run.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        rect.left() >= 0. && rect.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale} {view:?} challenge={challenge}: {rect:?}"
                    );
                }
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.guide);
                if challenge {
                    assert!(a.metrics.lesson_tabs.is_empty());
                    assert!(a.metrics.terminal.is_some());
                }
            }
        }
    }
}
#[test]
fn accounts_native_long_transcripts_follow_the_latest_error_in_both_terminals() {
    for view in [teaching::View::Assignment, teaching::View::Example] {
        let (ctx, mut a, i) = accounts_fixture_app(view);
        for _ in 0..10 {
            accounts_ui_command(&ctx, &mut a, "getent passwd");
        }
        let (offset, _) = a.metrics.accounts_output_scroll.unwrap();
        assert!(offset > 0., "long output must scroll in {view:?}");
        accounts_ui_command(&ctx, &mut a, "usermod -G");
        let (offset, viewport) = a.metrics.accounts_output_scroll.unwrap();
        let latest = a.metrics.accounts_latest_output.unwrap();
        assert!(
            offset > 0. && viewport.expand(1.).contains_rect(latest),
            "latest error is outside {view:?} viewport: offset={offset} viewport={viewport:?} row={latest:?}"
        );
        let attempt = &a.progress.attempts[&a.scenarios[i].id];
        let transcript = if view == teaching::View::Assignment {
            &attempt.transcript
        } else {
            &attempt
                .workspace
                .as_ref()
                .unwrap()
                .accounts_learning
                .as_ref()
                .unwrap()
                .transcript
        };
        assert!(transcript.last().unwrap().contains("usermod -G"));
        assert!(transcript.last().unwrap().lines().count() > 1);
        assert_eq!(attempt.state, a.scenarios[i].initial);
    }
}
fn open_assignment(a: &mut LabApp, i: usize) {
    a.launch(i);
    if a.scenarios[i].dhcpv4_teaching.is_some() {
        a.set_dhcpv4_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].unit_manager_teaching.is_some() {
        a.set_unit_manager_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].ospf_teaching.is_some() {
        a.set_ospf_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].acl_teaching.is_some() {
        a.set_acl_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].accounts_teaching.is_some() {
        a.set_accounts_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].storage_teaching.is_some() {
        a.set_storage_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].discovery_teaching.is_some() {
        a.set_discovery_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].dns_teaching.is_some() {
        a.set_dns_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].address_teaching.is_some() {
        a.set_address_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].service_teaching.is_some() {
        a.set_service_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].legacy_access_teaching.is_some()
        || a.scenarios[i].legacy_logs_teaching.is_some()
    {
        a.set_legacy_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].seed_dns_teaching.is_some() || a.scenarios[i].seed_mount_teaching.is_some() {
        a.set_seed_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].client_teaching.is_some() {
        a.set_client_view(i, teaching::View::Assignment);
    }
    if a.scenarios[i].process_teaching.is_some() {
        a.set_process_view(i, teaching::View::Assignment);
    }
}
fn frame(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, events: Vec<egui::Event>) {
    let modifiers = events
        .iter()
        .rev()
        .find_map(|e| {
            if let egui::Event::Key { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(w / a.scale, h / a.scale),
            )),
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
}
fn draw(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        frame(ctx, a, 1200., 850., vec![]);
    }
}
fn key(key: egui::Key, shift: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            shift,
            ..Default::default()
        },
    }
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect) {
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
    draw(ctx, a);
}
fn check(ctx: &egui::Context, a: &mut LabApp) {
    let r = a.metrics.check.unwrap();
    click(ctx, a, r);
}
fn solve(a: &mut LabApp, i: usize) {
    for command in a.scenarios[i].solution.clone() {
        a.input = command;
        a.submit(i);
    }
}
#[test]
fn all_nine_keep_workspace_prominent_and_controls_inside_width_at_both_modes_and_scales() {
    for i in 0..9 {
        for challenge in [false, true] {
            for (width, height, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
                let (ctx, mut a) = app();
                open_assignment(&mut a, i);
                a.scale = scale;
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = challenge;
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                assert!(!a.metrics.guide);
                assert!(!a.metrics.help_visible);
                let bounds = ctx.input(|input| input.content_rect());
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.terminal.iter())
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.editor_save.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= bounds.right() + 1.,
                        "{i} clips at {width}/{scale}: {r:?} vs {bounds:?}"
                    );
                }
                if width == 1200. {
                    assert!(
                        a.metrics.check.unwrap().bottom() < 850.,
                        "Check hidden on initial desktop lab {i}: {:?}",
                        a.metrics.check
                    );
                }
                if i == 0 {
                    assert_eq!(a.metrics.inputs.len(), 5);
                    for r in &a.metrics.inputs {
                        assert!((r.left() - a.metrics.inputs[0].left()).abs() < 1.);
                    }
                }
            }
        }
    }
}
#[test]
fn library_topic_is_visible_extra_filters_are_optional_and_catalog_navigation_survives_return() {
    let (ctx, mut a) = app();
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.filters.len(), 2);
    assert!(a.metrics.pagination);
    a.filter.track = "CCNA".into();
    draw(&ctx, &mut a);
    assert_eq!(
        a.metrics.pagination,
        a.scenarios.iter().filter(|s| s.track == "CCNA").count() > 12
    );
    a.filter.unit = "ipv4-foundations".into();
    draw(&ctx, &mut a);
    assert!(!a.metrics.pagination);
    let r = a.metrics.extras.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.metrics.filters.len(), 6);
    a.filter.track = "LPIC-1".into();
    a.filter.unit.clear();
    a.filter.topic = "Permissions".into();
    a.filter.search = "report".into();
    a.progress.favorites.insert("lpic1-permissions".into());
    a.filter.favorites = true;
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    open_assignment(&mut a, 3);
    a.selected = None;
    draw(&ctx, &mut a);
    assert_eq!(a.filter.search, "report");
    assert_eq!(a.metrics.card_actions.len(), 1);
    a.filter = Filter::default();
    a.developer = true;
    a.synthetic = true;
    draw(&ctx, &mut a);
    assert_eq!(a.synthetic_catalog.len(), 900);
    assert!(a.metrics.pagination);
    assert_eq!(a.metrics.card_actions.len(), 12);
    for width in [720., 1200.] {
        for scale in [1., 1.5] {
            a.scale = scale;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, 850., vec![]);
            }
            let bounds = ctx.input(|input| input.content_rect());
            for r in a.metrics.header.iter().chain(a.metrics.filters.iter()) {
                assert!(
                    r.right() <= bounds.right() + 1.,
                    "library overflow {width}/{scale}: {r:?}"
                );
            }
        }
    }
}
#[test]
fn ipv4_native_form_checks_partial_answers_and_clears_stale_topology_success() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 0);
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.inputs.len(), 5);
    let first = a.metrics.focus.unwrap();
    assert_eq!(ctx.memory(|m| m.focused()), Some(first));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("192.168.10.77".into())],
    );
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(first));
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let view = a.ipv4_view.as_mut().unwrap();
    view.exercise.answers = [
        "192.168.10.64",
        "192.168.10.95",
        "192.168.10.65",
        "192.168.10.94",
    ]
    .map(String::from);
    view.exercise.second = Some("192.168.10.80".into());
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.81".into());
    draw(&ctx, &mut a);
    assert!(
        !a.metrics.completion,
        "a different valid host still needs a fresh check"
    );
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.77".into());
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!engine::solved(
        &a.progress.attempts["ccna-ipv4"].state,
        &a.scenarios[0].goal
    ));
}
#[test]
fn every_lab_grades_state_and_invalidates_after_breakage_without_walkthrough_gates() {
    for i in 0..9 {
        let (ctx, mut a) = app();
        open_assignment(&mut a, i);
        if i == 0 {
            solve(&mut a, i);
            a.load_ipv4_view(i);
        } else {
            solve(&mut a, i);
        }
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        assert!(a.metrics.completion, "solution not complete {i}");
        assert_eq!(a.progress.attempts[&a.scenarios[i].id].step, 0);
        match &mut a
            .progress
            .attempts
            .get_mut(&a.scenarios[i].id)
            .unwrap()
            .state
        {
            State::PersistentMounts { storage } => {
                storage.save_configuration("broken".into()).unwrap()
            }
            State::Processes { table } => {
                let State::Processes { table: initial } = &a.scenarios[i].initial else {
                    panic!()
                };
                *table = initial.clone();
            }
            State::Subnet { second, .. } => *second = Some("192.168.10.77".into()),
            State::Vlsm { allocations, .. } => allocations[0].cidr = "10.20.0.1/26".into(),
            State::Network { gateway, .. } => *gateway = "192.168.40.1".parse().unwrap(),
            State::Unix { nodes, .. } => {
                let p = if i == 3 {
                    "/srv/report.txt"
                } else {
                    "/srv/project/notes.txt"
                };
                nodes.get_mut(p).unwrap().mode = 0;
            }
            State::Logs { result, .. } => result.clear(),
            State::Service { enabled, .. } => *enabled = false,
            State::Dns { loaded, .. } => {
                loaded.insert("portal.lab.test".into(), "192.0.2.8".into());
            }
            State::Mount { writable, .. } => *writable = false,
            State::Script { run, .. } => *run = None,
            state @ State::ApacheStatic { .. } => {
                let action = dlms_learning_lab::apache_static::Action::Stop {};
                engine::execute_for_goal(
                    state,
                    &a.scenarios[i].goal,
                    &format!(
                        "{}{}",
                        dlms_learning_lab::apache_static_workspace::PREFIX,
                        serde_json::to_string(&action).unwrap()
                    ),
                )
                .unwrap();
            }
            State::Nfs { .. }
            | State::RpmPackages { .. }
            | State::BasicNat { .. }
            | State::DebianPackages { .. }
            | State::SpanningTree { .. }
            | State::DhcpV4 { .. }
            | State::Ipv4Acl { .. }
            | State::Ospf { .. }
            | State::UnitManager { .. }
            | State::Accounts { .. }
            | State::Discovery { .. }
            | State::AuthoritativeDns { .. }
            | State::MacLearning { .. }
            | State::Switching { .. }
            | State::Ipv6 { .. }
            | State::Ipv4Routing { .. }
            | State::Filesystem { .. } => {
                unreachable!("This regression targets the original nine workspaces")
            }
        }
        if i == 0 {
            a.load_ipv4_view(i);
        }
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion, "success remained after breakage {i}");
    }
}
#[test]
fn service_editor_saves_without_starting_and_resume_preserves_unsaved_draft() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 6);
    draw(&ctx, &mut a);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_port = Some(" 8080 ".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    assert!(
        matches!(&a.progress.attempts["lpic2-service"].state,State::Service{port,running:false,enabled:false,..}if port=="8080")
    );
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_port
            .as_deref(),
        Some("8080")
    );
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    for c in ["systemctl restart atlas", "systemctl enable atlas"] {
        a.input = c.into();
        a.submit(6);
    }
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_port = Some("unfinished draft".into());
    a.selected = None;
    open_assignment(&mut a, 6);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_port
            .as_deref(),
        Some("unfinished draft")
    );
    a.input = "systemctl stop atlas".into();
    a.submit(6);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
}
#[test]
fn dns_and_mount_editors_stage_configuration_then_terminal_verifies_application() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 7);
    draw(&ctx, &mut a);
    let original = a.progress.attempts["lpic2-dns"].state.clone();
    let w = a
        .progress
        .attempts
        .get_mut("lpic2-dns")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap();
    w.dns_address = Some("192.0.2.80".into());
    w.dns_serial = Some("bad".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.progress.attempts["lpic2-dns"].state, original);
    a.progress
        .attempts
        .get_mut("lpic2-dns")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_serial = Some("2026102102".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    a.input = "systemctl reload named".into();
    a.submit(7);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    let served = a.progress.attempts["lpic2-dns"].state.clone();
    a.progress
        .attempts
        .get_mut("lpic2-dns")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_serial = Some("unfinished serial".into());
    draw(&ctx, &mut a);
    assert!(
        !a.metrics.completion,
        "An unsaved DNS edit must hide completion"
    );
    assert_eq!(a.progress.attempts["lpic2-dns"].state, served);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.progress.attempts["lpic2-dns"].state, served);
    assert!(
        !a.metrics.completion,
        "An invalid Save must retain the pending draft"
    );
    a.progress
        .attempts
        .get_mut("lpic2-dns")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_serial = Some("2026102102".into());
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    open_assignment(&mut a, 8);
    draw(&ctx, &mut a);
    a.progress
        .attempts
        .get_mut("lpic2-storage")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mount_device = Some("/dev/vdb1".into());
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    for c in [
        "mount -o remount,rw /data",
        "rm -r /data/cache",
        "umount /data",
        "mount /data",
    ] {
        a.input = c.into();
        a.submit(8);
    }
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    let mounted = a.progress.attempts["lpic2-storage"].state.clone();
    a.progress
        .attempts
        .get_mut("lpic2-storage")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mount_device = Some("unfinished device".into());
    draw(&ctx, &mut a);
    assert!(
        !a.metrics.completion,
        "An unsaved mount edit must hide completion"
    );
    assert_eq!(a.progress.attempts["lpic2-storage"].state, mounted);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.progress.attempts["lpic2-storage"].state, mounted);
    assert!(!a.metrics.completion);
    a.selected = None;
    open_assignment(&mut a, 8);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert_eq!(
        a.progress.attempts["lpic2-storage"]
            .workspace
            .as_ref()
            .unwrap()
            .mount_device
            .as_deref(),
        Some("unfinished device")
    );
    a.progress
        .attempts
        .get_mut("lpic2-storage")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .mount_device = Some("/dev/vdb1".into());
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
}
#[test]
fn terminal_enter_focus_and_query_invalidation_work_in_normal_app() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 5);
    draw(&ctx, &mut a);
    let focus = a.metrics.focus.unwrap();
    assert_eq!(ctx.memory(|m| m.focused()), Some(focus));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text(
            "cat /var/log/app.log | grep ERROR".into(),
        )],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), Some(focus));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, true)]);
    draw(&ctx, &mut a);
    assert_eq!(ctx.memory(|m| m.focused()), Some(focus));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("cat /var/log/app.log".into())],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
}
#[test]
fn reset_resume_and_numerical_practice_preserve_other_attempts_and_original_answers() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 0);
    solve(&mut a, 0);
    a.load_ipv4_view(0);
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    let original = a.progress.attempts["ccna-ipv4"].state.clone();
    open_assignment(&mut a, 5);
    a.input = "unfinished command".into();
    a.progress
        .attempts
        .get_mut("lpic1-logs")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = a.input.clone();
    open_assignment(&mut a, 0);
    let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
    let e = ipv4::Exercise::from_state(&attempt.state).unwrap();
    let mut l = ipv4::Lesson::from_legacy(e.ip, e.prefix, 0);
    l.variant = Some(ipv4::generated(2));
    l.use_variant = true;
    attempt.ipv4 = Some(l);
    a.load_ipv4_view(0);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, original);
    assert!(!a.metrics.completion);
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.dirty = true;
    a.save();
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(p.attempts["ccna-ipv4"].state, original);
    a.progress = p;
    open_assignment(&mut a, 5);
    assert_eq!(a.input, "unfinished command");
    a.progress.favorites.insert("lpic1-logs".into());
    open_assignment(&mut a, 0);
    draw(&ctx, &mut a);
    let before_reset = a.progress.attempts["ccna-ipv4"].state.clone();
    let r = a.metrics.reset.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.reset_pending);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, before_reset);
    let r = a.metrics.confirm_reset.unwrap();
    click(&ctx, &mut a, r);
    assert!(!a.reset_pending);
    assert_eq!(
        a.progress.attempts["lpic1-logs"]
            .workspace
            .as_ref()
            .unwrap()
            .command,
        "unfinished command"
    );
    assert!(a.progress.favorites.contains("lpic1-logs"));
    assert!(a.progress.attempts["ccna-ipv4"].ipv4.is_none());
}

#[test]
fn assistance_is_optional_and_inserting_a_command_never_executes_it() {
    for i in [3, 6] {
        let (ctx, mut a) = app();
        open_assignment(&mut a, i);
        draw(&ctx, &mut a);
        let id = a.scenarios[i].id.clone();
        let original = a.progress.attempts[&id].state.clone();
        assert!(!a.metrics.guide);
        assert!(a.metrics.insert.is_none());
        let r = a.metrics.help.unwrap();
        click(&ctx, &mut a, r);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(700., 700.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -400.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
        let r = a.metrics.walkthrough.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.metrics.guide);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                egui::Event::PointerMoved(egui::pos2(700., 700.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -500.),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..12 {
            frame(&ctx, &mut a, 1200., 850., vec![]);
        }
        let r = a.metrics.insert.unwrap();
        assert!(ctx.input(|i| i.content_rect()).contains(r.center()));
        click(&ctx, &mut a, r);
        assert!(!a.input.is_empty());
        assert_eq!(a.progress.attempts[&id].state, original);
        assert!(a.progress.attempts[&id].transcript.is_empty());
        assert_eq!(
            ctx.memory(|m| m.focused()),
            a.metrics.focus,
            "Insert focuses the terminal even beside an editor"
        );
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        draw(&ctx, &mut a);
        assert!(!a.metrics.guide);
        assert!(a.metrics.insert.is_none());
    }
}

#[test]
fn scripting_workspaces_keep_tasks_and_controls_at_sizes_scales_and_modes() {
    for (i, _) in pack::bundled()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.teaching.is_some())
    {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                for challenge in [false, true] {
                    let (ctx, mut a) = app();
                    open_assignment(&mut a, i);
                    a.set_script_view(i, teaching::View::Assignment);
                    a.scale = scale;
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = challenge;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    assert!(
                        a.metrics.editor_viewport.unwrap().height() >= 175.,
                        "usable script editor collapsed"
                    );
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    for r in a
                        .metrics
                        .header
                        .iter()
                        .chain(a.metrics.inputs.iter())
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.stop.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "script{i} {width}/{scale} {r:?}"
                        );
                    }
                    assert!(a.metrics.stop.is_some());
                    assert!(a.metrics.check.is_some());
                }
            }
        }
    }
}
#[test]
fn scripting_native_run_stop_focus_resume_and_success_invalidation() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    a.set_script_view(9, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let editor = a.metrics.focus.unwrap();
    let solution = a.scenarios[9].solution[0].clone();
    assert!(ctx.memory(|m| m.has_focus(editor)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text(solution),
        ],
    );
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        }],
    );
    for _ in 0..20 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Escape, false)],
    );
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(editor)));
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text("while true; do :; done".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    check(&ctx, &mut a);
    assert!(a.script_job.is_some());
    let stop = a.metrics.stop.unwrap();
    click(&ctx, &mut a, stop);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
    let draft = a.progress.attempts["shell-handover"].state.clone();
    open_assignment(&mut a, 0);
    open_assignment(&mut a, 9);
    assert_eq!(a.progress.attempts["shell-handover"].state, draft);
    a.reset_attempt(9);
    assert_eq!(
        a.progress.attempts["shell-handover"].state,
        a.scenarios[9].initial
    );
}
#[test]
fn every_theme_renders_focus_selection_and_disabled_samples() {
    for t in theme::inventory() {
        let (ctx, mut a) = app();
        a.theme_id = t.id.clone();
        theme::apply(&ctx, &t.id);
        a.theme_preview = true;
        draw(&ctx, &mut a);
        assert_eq!(theme::roles(&ctx).text, t.roles.text);
        assert!(ctx.style().visuals.selection.bg_fill == t.roles.accent);
        assert_eq!(
            ctx.style().visuals.selection.stroke.color,
            t.roles.on_accent
        );
        assert_eq!(ctx.style().visuals.disabled_alpha, 1.);
    }
}

#[test]
fn learn_examples_are_editable_ungraded_and_drafts_survive_navigation_save_and_retry() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    open_assignment(&mut a, 9);
    draw(&ctx, &mut a);
    assert_eq!(a.script_view(9), teaching::View::Introduction);
    assert!(a.metrics.check.is_none());
    assert!(!a.metrics.help_visible);
    let before = a.progress.attempts["shell-handover"].state.clone();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.script_view(9), teaching::View::Example);
    let editor = a.metrics.focus.unwrap();
    assert_eq!(ctx.memory(|m| m.focused()), Some(editor));
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    let l = a.progress.attempts["shell-handover"]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap();
    assert_eq!(
        l.example_result.as_ref().unwrap().stdout,
        "Welcome\nHello, Mina!\nRoom: Library\n"
    );
    assert_eq!(a.progress.attempts["shell-handover"].state, before);
    assert!(!a.metrics.completion);
    let edited = l.example_source.replace("Mina", "Rowan");
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
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
            egui::Event::Text(edited.clone()),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert!(
        a.progress.attempts["shell-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .as_ref()
            .unwrap()
            .stdout
            .contains("Rowan")
    );
    let r = a.metrics.assignment_action.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.script_view(9), teaching::View::Assignment);
    assert_eq!(a.progress.attempts["shell-handover"].state, before);
    let solution = a.scenarios[9].solution[0].clone();
    if let State::Script { source, run } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = solution;
        *run = None;
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    let solved = a.progress.attempts["shell-handover"].state.clone();
    a.set_script_view(9, teaching::View::Example);
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert_eq!(a.progress.attempts["shell-handover"].state, solved);
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
    assert_eq!(a.script_view(9), teaching::View::Example);
    assert_eq!(
        a.progress.attempts["shell-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source,
        edited
    );
    a.set_script_view(9, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.reset_attempt(9);
    draw(&ctx, &mut a);
    assert_eq!(a.script_view(9), teaching::View::Introduction);
    assert_eq!(a.progress.attempts["shell-handover"].state, before);
}

#[test]
fn all_introductions_and_examples_wrap_at_large_text_and_challenge_stays_independent() {
    for (i, _) in pack::bundled()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, s)| s.teaching.is_some())
    {
        for view in [teaching::View::Introduction, teaching::View::Example] {
            for scale in [1., 1.5] {
                for width in [720., 1200.] {
                    let (ctx, mut a) = app();
                    open_assignment(&mut a, i);
                    a.set_script_view(i, view);
                    a.scale = scale;
                    if view == teaching::View::Example {
                        let lesson = a.scenarios[i].teaching.as_ref().unwrap();
                        let result = shell::run(&lesson.example, &lesson.fixture);
                        a.progress
                            .attempts
                            .get_mut(&a.scenarios[i].id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .script_learning
                            .as_mut()
                            .unwrap()
                            .example_result = Some(result);
                    }
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    for r in a
                        .metrics
                        .header
                        .iter()
                        .chain(a.metrics.lesson_tabs.iter())
                        .chain(a.metrics.inputs.iter())
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.assignment_action.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "lab {i} {view:?} {width}/{scale} {r:?}"
                        );
                    }
                    if view == teaching::View::Example {
                        assert!(
                            a.metrics.editor_viewport.unwrap().height() >= 175.,
                            "example editor collapsed"
                        );
                    }
                    if view == teaching::View::Example {
                        assert!(
                            a.metrics.output.unwrap().height() >= 180.,
                            "example output collapsed"
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.completion);
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.script_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                    assert!(a.metrics.check.is_some());
                    assert!(a.metrics.example_action.is_none());
                }
            }
        }
    }
}

#[test]
fn settings_moves_samples_off_toolbar_and_theme_changes_persist() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    draw(&ctx, &mut a);
    assert!(!a.settings_open);
    assert!(!a.theme_preview);
    let r = a.metrics.settings.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.settings_open);
    a.theme_id = "dlms-light".into();
    a.sync_appearance(&ctx, ("lab-navy".into(), a.scale));
    let prefs = theme::Preferences::load(&a.path.with_file_name("preferences-v1.json")).unwrap();
    assert_eq!(prefs.theme, "dlms-light");
    assert_eq!(theme::inventory().len(), 28);
}

#[test]
fn example_keyboard_run_syntax_retry_and_stop_do_not_grade_the_assignment() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    a.set_script_view(9, teaching::View::Example);
    draw(&ctx, &mut a);
    let original = a.progress.attempts["shell-handover"].state.clone();
    let editor = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(editor)));
    let replace = |a: &mut LabApp, text: &str| {
        a.focus_input = true;
        draw(&ctx, a);
        frame(
            &ctx,
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
                egui::Event::Text(text.into()),
            ],
        );
        draw(&ctx, a);
    };
    let run = |a: &mut LabApp| {
        frame(
            &ctx,
            a,
            1200.,
            850.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            }],
        );
    };
    replace(&mut a, "printf 'not closed");
    run(&mut a);
    for _ in 0..5 {
        draw(&ctx, &mut a);
    }
    let error = a.progress.attempts["shell-handover"]
        .workspace
        .as_ref()
        .unwrap()
        .script_learning
        .as_ref()
        .unwrap()
        .example_result
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    assert_eq!(teaching::diagnostic(error), teaching::Diagnostic::Syntax);
    let example = a.scenarios[9].teaching.as_ref().unwrap().example.clone();
    replace(&mut a, &example);
    run(&mut a);
    for _ in 0..5 {
        draw(&ctx, &mut a);
    }
    assert!(
        a.progress.attempts["shell-handover"]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_result
            .as_ref()
            .unwrap()
            .error
            .is_none()
    );
    a.focus_input = true;
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Escape, false)],
    );
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(editor)));
    replace(&mut a, "while true; do :; done");
    run(&mut a);
    assert!(a.script_job.is_some());
    draw(&ctx, &mut a);
    let r = a.metrics.stop.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts["shell-handover"].state, original);
}

#[test]
fn legacy_subnet_credit_does_not_invent_a_host_and_requires_new_check() {
    let (ctx, mut a) = app();
    let s = &a.scenarios[0];
    let mut old = Attempt::new(s, 1);
    if let State::Subnet {
        answers, second, ..
    } = &mut old.state
    {
        *answers = [
            "192.168.10.64",
            "192.168.10.95",
            "192.168.10.65",
            "192.168.10.94",
        ]
        .map(String::from);
        *second = None;
    }
    a.progress.attempts.insert(s.id.clone(), old);
    open_assignment(&mut a, 0);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(progress::legacy_credit(&a.progress.attempts["ccna-ipv4"]));
    assert!(!progress::current_solved(
        &a.progress.attempts["ccna-ipv4"],
        &a.scenarios[0]
    ));
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.81".into());
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert!(
        a.progress.attempts["ccna-ipv4"]
            .workspace
            .as_ref()
            .unwrap()
            .legacy_host_reviewed
    );
    assert!(progress::current_solved(
        &a.progress.attempts["ccna-ipv4"],
        &a.scenarios[0]
    ));
}

#[test]
fn historical_auto_host_snapshot_is_reviewed_resumed_and_invalidated() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    open_assignment(&mut a, 0);
    solve(&mut a, 0);
    let attempt = a.progress.attempts.get_mut("ccna-ipv4").unwrap();
    attempt.workspace.as_mut().unwrap().migrated_host = true;
    attempt.workspace.as_mut().unwrap().ipv4_check = Some(model::snapshot(
        &ipv4::Exercise::from_state(&attempt.state).unwrap(),
    ));
    let before = attempt.state.clone();
    open_assignment(&mut a, 0);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts["ccna-ipv4"].state, before);
    assert!(!a.metrics.completion);
    assert!(a.ipv4_view.as_ref().unwrap().checked_answers.is_none());
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    open_assignment(&mut a, 0);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.ipv4_view.as_mut().unwrap().exercise.second = Some("192.168.10.77".into());
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts["ccna-ipv4"],
        &a.scenarios[0]
    ));
}

fn data_preview(ctx: &egui::Context, a: &mut LabApp) {
    let r = a.metrics.data_restore.unwrap();
    click(ctx, a, r);
    let r = a.metrics.data_filename.unwrap();
    click(ctx, a, r);
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![egui::Event::Text("dlms-learning-backup.json".into())],
    );
    draw(ctx, a);
    let r = a.metrics.data_file_action.unwrap();
    click(ctx, a, r);
    for _ in 0..100 {
        draw(ctx, a);
        if a.metrics.data_confirm.is_some() {
            // Async inspection can first show the window on this frame. Let egui
            // settle its measured geometry before using a synthetic pointer target.
            draw(ctx, a);
            draw(ctx, a);
            assert!(a.metrics.data_confirm.is_some());
            return;
        }
        std::thread::yield_now();
    }
    panic!("Restore preview did not finish");
}
#[test]
fn data_ui_exports_previews_cancels_then_restores_without_losing_current_drafts() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    open_assignment(&mut a, 9);
    if let State::Script { source, .. } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = "saved='draft'".into();
    }
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let r = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, r);
    let r = a.metrics.data_file_action.unwrap();
    click(&ctx, &mut a, r);
    assert!(dir.path().join("dlms-learning-backup.json").exists());
    a.reset_attempt(9);
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_cancel.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(
        a.progress.attempts["shell-handover"].state,
        a.scenarios[9].initial
    );
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, r);
    assert!(
        matches!(&a.progress.attempts["shell-handover"].state,State::Script{source,..}if source=="saved='draft'")
    );
    let safety = progress::load(
        &dir.path().join("progress-before-restore-1.json"),
        &a.scenarios,
    )
    .unwrap();
    assert_eq!(
        safety.attempts["shell-handover"].state,
        a.scenarios[9].initial
    );
    assert!(!a.dirty);
}
#[test]
fn restoring_corrupt_profile_protects_unsaved_read_only_session_work() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    open_assignment(&mut a, 9);
    dlms_learning_lab::backup::export(
        &dir.path().join("dlms-learning-backup.json"),
        &a.progress,
        &a.scenarios,
    )
    .unwrap();
    std::fs::write(&a.path, b"corrupt original").unwrap();
    a.read_only = true;
    a.storage_error = Some("controlled invalid profile".into());
    if let State::Script { source, .. } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = "keep='unsaved session'".into();
    }
    a.dirty = true;
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let r = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, r);
    assert!(!a.read_only, "{}", a.data_tools.restore_diagnostic());
    assert_eq!(
        std::fs::read(dir.path().join("progress-before-restore-1.json")).unwrap(),
        b"corrupt original"
    );
    let session = dlms_learning_lab::backup::inspect(
        &dir.path().join("session-before-restore-1.json"),
        &a.scenarios,
        &std::sync::atomic::AtomicBool::new(false),
        &std::sync::atomic::AtomicUsize::new(0),
    )
    .unwrap();
    assert!(
        matches!(&session.progress.attempts["shell-handover"].state,State::Script{source,..}if source=="keep='unsaved session'")
    );
}

#[test]
fn backup_dialog_does_not_allow_background_editor_shortcuts_to_run_labs() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.smoke = true;
    open_assignment(&mut a, 9);
    a.set_script_view(9, teaching::View::Assignment);
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let r = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        }],
    );
    draw(&ctx, &mut a);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
}

#[test]
fn course_start_and_unit_filters_keep_search_favorites_and_resume_navigation() {
    let (ctx, mut a) = app();
    a.filter.track = "Bash/Shell".into();
    draw(&ctx, &mut a);
    let r = a.metrics.course_start.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.selected, Some(9));
    assert_eq!(a.script_view(9), teaching::View::Introduction);
    let r = a.metrics.back.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.filter.track, "Bash/Shell");
    a.filter.unit = "shell-foundations".into();
    a.filter.search = "handover".into();
    a.progress.favorites.insert("shell-handover".into());
    a.filter.favorites = true;
    draw(&ctx, &mut a);
    assert_eq!(a.metrics.card_actions.len(), 1);
    open_assignment(&mut a, 9);
    draw(&ctx, &mut a);
    let r = a.metrics.back.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.filter.unit, "shell-foundations");
    assert_eq!(a.filter.search, "handover");
    assert!(a.filter.favorites);
}
#[test]
fn unit_filter_controls_wrap_at_narrow_widths_and_large_text() {
    for (track, unit) in [
        ("LPIC-2", "linux-service-recovery"),
        ("LPIC-1", "linux-access-policy"),
    ] {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                let (ctx, mut a) = app();
                a.filter.track = track.into();
                a.filter.unit = unit.into();
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, 850., vec![]);
                }
                for r in &a.metrics.filters {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale}: {r:?}, screen {:?}",
                        ctx.content_rect()
                    );
                }
            }
        }
    }
}
#[test]
fn function_scripts_run_cooperatively_in_the_existing_workspace_and_preserve_drafts() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    a.set_script_view(9, teaching::View::Assignment);
    let source = "handover() { printf 'Site: %s\\nTask: %s\\n' \"$1\" \"$2\"; }\nhandover \"$SITE\" \"$TASK\"";
    if let State::Script { source: draft, run } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *draft = source.into();
        *run = None;
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    if let State::Script { source, run } =
        &mut a.progress.attempts.get_mut("shell-handover").unwrap().state
    {
        *source = "spin() { while true; do :; done; }\nspin".into();
        *run = None;
    }
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    check(&ctx, &mut a);
    assert!(a.script_job.is_some());
    let stop = a.metrics.stop.unwrap();
    click(&ctx, &mut a, stop);
    assert!(a.script_job.is_none());
    let draft = a.progress.attempts["shell-handover"].state.clone();
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
    assert_eq!(a.progress.attempts["shell-handover"].state, draft);
    assert!(!a.metrics.completion);
}

#[test]
fn terminal_introduction_practice_and_assignment_keep_drafts_focus_and_credit_separate() {
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-file-navigation")
        .unwrap();
    open_assignment(&mut a, i);
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Introduction);
    assert!(a.metrics.terminal.is_none());
    let original = a.progress.attempts["lpic1-file-navigation"].state.clone();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Example);
    assert!(ctx.memory(|m| m.has_focus(a.metrics.focus.unwrap())));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Text("pwd".into()),
            key(egui::Key::Enter, false),
        ],
    );
    draw(&ctx, &mut a);
    let l = a.progress.attempts["lpic1-file-navigation"]
        .workspace
        .as_ref()
        .unwrap()
        .terminal_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.transcript.len(), 1);
    assert!(l.transcript[0].contains("/home/alex"));
    assert_eq!(a.progress.attempts["lpic1-file-navigation"].state, original);
    assert!(!a.metrics.completion);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("practice draft".into())],
    );
    draw(&ctx, &mut a);
    a.set_terminal_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("assignment draft".into())],
    );
    draw(&ctx, &mut a);
    let assignment = a.input.clone();
    a.set_terminal_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts["lpic1-file-navigation"]
            .workspace
            .as_ref()
            .unwrap()
            .terminal_learning
            .as_ref()
            .unwrap()
            .command,
        "practice draft"
    );
    assert_eq!(a.input, assignment);
    let directory = tempfile::tempdir().unwrap();
    a.path = directory.path().join("progress.json");
    a.save();
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    a.progress
        .attempts
        .get_mut("lpic1-file-navigation")
        .unwrap()
        .challenge = true;
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Assignment);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
    a.reset_attempt(i);
    draw(&ctx, &mut a);
    assert_eq!(a.terminal_view(i), teaching::View::Introduction);
    assert_eq!(a.progress.attempts["lpic1-file-navigation"].state, original);
}
#[test]
fn file_lesson_layouts_wrap_and_challenge_keeps_full_assignment_at_large_text() {
    let labs = pack::bundled().unwrap();
    for (i, s) in labs
        .iter()
        .enumerate()
        .filter(|(_, s)| s.terminal_teaching.is_some())
    {
        for width in [720., 1200.] {
            for scale in [1., 1.5] {
                for view in [
                    teaching::View::Introduction,
                    teaching::View::Example,
                    teaching::View::Assignment,
                ] {
                    let (ctx, mut a) = app();
                    open_assignment(&mut a, i);
                    a.scale = scale;
                    a.set_terminal_view(i, view);
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    for r in a
                        .metrics
                        .header
                        .iter()
                        .chain(a.metrics.lesson_tabs.iter())
                        .chain(a.metrics.terminal.iter())
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.assignment_action.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "{} {width}/{scale} {view:?} {r:?}",
                            s.id
                        );
                    }
                    if view != teaching::View::Introduction {
                        assert!(a.metrics.output.unwrap().height() >= 140.);
                        assert!(a.metrics.terminal.is_some());
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.terminal_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}
#[test]
fn file_assignments_grade_then_invalidate_and_selected_reset_preserves_other_labs() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    let old = a.progress.attempts["shell-handover"].state.clone();
    let indexes: Vec<_> = a
        .scenarios
        .iter()
        .enumerate()
        .filter(|(_, s)| s.terminal_teaching.is_some())
        .map(|(i, _)| i)
        .collect();
    for i in indexes {
        open_assignment(&mut a, i);
        a.set_terminal_view(i, teaching::View::Assignment);
        for cmd in a.scenarios[i].solution.clone() {
            a.input = cmd;
            a.submit(i);
        }
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        draw(&ctx, &mut a);
        assert!(a.metrics.completion, "{}", a.scenarios[i].id);
        let id = a.scenarios[i].id.clone();
        let State::Filesystem { tree } = &mut a.progress.attempts.get_mut(&id).unwrap().state
        else {
            panic!()
        };
        // Each assignment protects at least one supplied source; change all for this UI regression.
        for text in tree.files.values_mut() {
            text.push_str("changed");
        }
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion, "{id}");
        a.reset_attempt(i);
        assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
        assert_eq!(a.progress.attempts["shell-handover"].state, old);
    }
}

#[test]
fn new_linux_learners_start_with_file_foundations_without_reordering_saved_ids() {
    let (ctx, mut a) = app();
    a.filter.track = "LPIC-1".into();
    draw(&ctx, &mut a);
    let r = a.metrics.course_start.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.scenarios[a.selected.unwrap()].id, "lpic1-file-navigation");
    assert_eq!(
        a.terminal_view(a.selected.unwrap()),
        teaching::View::Introduction
    );
    assert_eq!(a.scenarios[3].id, "lpic1-permissions");
}

#[test]
fn terminal_example_insert_only_populates_input_and_tab_releases_the_field() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 24);
    a.set_terminal_view(24, teaching::View::Example);
    draw(&ctx, &mut a);
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let l = a.progress.attempts["lpic1-file-navigation"]
        .workspace
        .as_ref()
        .unwrap()
        .terminal_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.command, "pwd");
    assert!(l.transcript.is_empty());
    assert_eq!(
        l.tree,
        a.scenarios[24].terminal_teaching.as_ref().unwrap().fixture
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(
        !ctx.memory(|m| m.has_focus(field)),
        "Tab leaves the one-line terminal input"
    );
}

#[test]
fn ipv6_native_typing_check_retry_focus_and_selected_reset() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    let earlier = a.progress.attempts["shell-handover"].clone();
    open_assignment(&mut a, 30);
    assert_eq!(a.ipv6_view(30), teaching::View::Introduction);
    draw(&ctx, &mut a);
    let r = a.metrics.assignment_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let id = a.scenarios[30].id.clone();
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .as_ref()
            .unwrap()
            .selected_device,
        "NewPC"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("2001:db8:44::23/64".into())],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(
        a.metrics.completion,
        "Manual entry should work without advanced settings"
    );
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text("malformed".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[30]
    ));
    check(&ctx, &mut a);
    assert!(a.notice.as_ref().unwrap().contains("Malformed"));
    open_assignment(&mut a, 31);
    open_assignment(&mut a, 30);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .as_ref()
            .unwrap()
            .devices["NewPC"]
            .interfaces["lan"]
            .address,
        "malformed"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    a.reset_attempt(30);
    assert_eq!(a.progress.attempts[&id].state, a.scenarios[30].initial);
    assert_eq!(a.progress.attempts["shell-handover"].state, earlier.state);
}

#[test]
fn ipv6_examples_apply_without_credit_and_open_assignment_without_overwriting_draft() {
    let (ctx, mut a) = app();
    for i in 30..36 {
        open_assignment(&mut a, i);
        let id = a.scenarios[i].id.clone();
        let assignment = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_draft
            .clone();
        a.set_ipv6_view(i, teaching::View::Example);
        let lesson = a.scenarios[i].ipv6_teaching.clone().unwrap();
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .ipv6_learning
            .as_mut()
            .unwrap()
            .fill_example(&lesson)
            .unwrap();
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        let attempt = &a.progress.attempts[&id];
        let learning = attempt
            .workspace
            .as_ref()
            .unwrap()
            .ipv6_learning
            .as_ref()
            .unwrap();
        assert!(learning.checks(&lesson).iter().all(|(pass, _)| *pass));
        assert_eq!(attempt.state, a.scenarios[i].initial);
        assert_eq!(attempt.workspace.as_ref().unwrap().ipv6_draft, assignment);
        assert!(!progress::current_solved(attempt, &a.scenarios[i]));
        let r = a.metrics.lesson_tabs[2];
        click(&ctx, &mut a, r);
        assert_eq!(a.ipv6_view(i), teaching::View::Assignment);
    }
}

#[test]
fn ipv6_native_views_keep_controls_in_width_with_large_text_and_challenge_is_independent() {
    for i in 30..36 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for routes in [false, true] {
                for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
                    let (ctx, mut a) = app();
                    open_assignment(&mut a, i);
                    a.scale = scale;
                    a.set_ipv6_view(i, view);
                    let workspace = a
                        .progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap();
                    workspace.ipv6_draft.as_mut().unwrap().show_routes = routes;
                    workspace.ipv6_learning.as_mut().unwrap().draft.show_routes = routes;
                    for _ in 0..4 {
                        frame(&ctx, &mut a, w, h, vec![]);
                    }
                    for r in a
                        .metrics
                        .inputs
                        .iter()
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.lesson_tabs.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "lab{i} {view:?} {w}/{scale} {r:?}"
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    if w == 1200. && view == teaching::View::Assignment && !routes {
                        assert!(
                            a.metrics.check.unwrap().bottom() < 850.,
                            "check below desktop lab{i}"
                        );
                    }
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.ipv6_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}

#[test]
fn earlier_script_runs_are_labeled_legacy_until_a_current_native_run_and_remain_saved() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    let lab = a.scenarios[9].clone();
    let engine::Goal::Script { cases } = &lab.goal else {
        panic!()
    };
    let original = lab.solution[0].clone();
    let source = format!("IFS=,\n{original}");
    a.progress.attempts.get_mut(&lab.id).unwrap().state = State::Script {
        source: source.clone(),
        run: Some(shell::Record {
            engine_revision: 0,
            source,
            results: cases.iter().map(|c| shell::run(&original, c)).collect(),
        }),
    };
    open_assignment(&mut a, 9);
    a.set_script_view(9, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(progress::legacy_script_credit(
        &a.progress.attempts[&lab.id],
        &lab
    ));
    check(&ctx, &mut a);
    for _ in 0..10 {
        draw(&ctx, &mut a);
    }
    assert!(!a.metrics.completion);
    assert!(progress::legacy_script_credit(
        &a.progress.attempts[&lab.id],
        &lab
    ));
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text(original),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    for _ in 0..15 {
        draw(&ctx, &mut a);
    }
    assert!(a.metrics.completion);
    assert!(progress::legacy_script_credit(
        &a.progress.attempts[&lab.id],
        &lab
    ));
    let history = a.progress.attempts[&lab.id]
        .workspace
        .as_ref()
        .unwrap()
        .script_history
        .clone();
    open_assignment(&mut a, 10);
    open_assignment(&mut a, 9);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert_eq!(
        a.progress.attempts[&lab.id]
            .workspace
            .as_ref()
            .unwrap()
            .script_history,
        history
    );
    a.reset_attempt(9);
    assert!(
        a.progress.attempts[&lab.id]
            .workspace
            .as_ref()
            .unwrap()
            .script_history
            .is_none()
    );
}

#[test]
fn count_assignments_keep_the_editor_and_actions_in_width_and_stop_file_loops() {
    for i in 43..47 {
        for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
            let (ctx, mut a) = app();
            open_assignment(&mut a, i);
            a.set_script_view(i, teaching::View::Assignment);
            a.scale = scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a.metrics.inputs.iter().chain(a.metrics.check.iter()) {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "lab{i} {w}/{scale}: {r:?}"
                );
            }
            assert!(a.metrics.editor_viewport.unwrap().height() >= 175.);
            assert!(!a.metrics.help_visible);
        }
    }
    let (ctx, mut a) = app();
    open_assignment(&mut a, 46);
    a.set_script_view(46, teaching::View::Assignment);
    if let State::Script { source, run } = &mut a
        .progress
        .attempts
        .get_mut("shell-count-capstone")
        .unwrap()
        .state
    {
        *source = "while true; do :; done < alpha.txt".into();
        *run = None;
    }
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.script_job.is_some());
    let r = a.metrics.stop.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.script_job.is_none());
    assert!(!a.metrics.completion);
}

#[test]
fn switching_fixture_keeps_practice_separate_and_wraps_with_visible_keyboard_focus() {
    for (width, height, scale) in [
        (1200., 850., 1.),
        (1120., 850., 1.),
        (1080., 850., 1.),
        (720., 560., 1.),
        (720., 560., 1.5),
    ] {
        let (ctx, mut a) = app();
        a.scenarios.push(
            pack::load(include_str!("../../tests/fixtures/switching-preview.json"))
                .unwrap()
                .remove(0),
        );
        let i = a.scenarios.len() - 1;
        open_assignment(&mut a, i);
        a.scale = scale;
        assert_eq!(a.switching_view(i), teaching::View::Introduction);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            a.set_switching_view(i, view);
            for _ in 0..3 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.check.iter())
                .chain(a.metrics.lesson_tabs.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "{view:?} {width}/{scale}: {r:?}"
                );
            }
            assert!(!a.metrics.help_visible);
            assert!(!a.metrics.completion);
        }
        assert!(a.metrics.inputs.len() >= 5);
        let input = a.metrics.inputs[0];
        if ctx.content_rect().contains(input.center()) {
            for pressed in [true, false] {
                frame(
                    &ctx,
                    &mut a,
                    width,
                    height,
                    vec![
                        egui::Event::PointerMoved(input.center()),
                        egui::Event::PointerButton {
                            pos: input.center(),
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
            assert!(ctx.memory(|m| m.focused()).is_some());
        }
        frame(
            &ctx,
            &mut a,
            width,
            height,
            vec![key(egui::Key::Tab, false)],
        );
        assert!(ctx.memory(|m| m.focused()).is_some());
        let s = &a.scenarios[i];
        a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
        frame(&ctx, &mut a, width, height, vec![]);
        assert_eq!(a.switching_view(i), teaching::View::Assignment);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(!a.metrics.help_visible);
    }
}

#[test]
fn authored_vlan_assignments_keep_tools_in_width_and_guidance_optional() {
    for name in [
        "ccna-vlan-access",
        "ccna-vlan-trunk",
        "ccna-vlan-native",
        "ccna-vlan-handover",
    ] {
        for (width, height, scale) in [(1200., 850., 1.), (1080., 850., 1.), (720., 560., 1.5)] {
            let (ctx, mut a) = app();
            let i = a.scenarios.iter().position(|s| s.id == name).unwrap();
            open_assignment(&mut a, i);
            a.scale = scale;
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                a.set_switching_view(i, view);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.lesson_tabs.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{name} {view:?} {width}/{scale}: {r:?}"
                    );
                }
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.completion);
            }
            a.progress.attempts.get_mut(name).unwrap().challenge = true;
            frame(&ctx, &mut a, width, height, vec![]);
            assert_eq!(a.switching_view(i), teaching::View::Assignment);
            assert!(!a.metrics.inputs.is_empty());
            assert!(a.metrics.lesson_tabs.is_empty());
        }
    }
}

#[test]
fn resumed_switching_draft_never_shows_applied_state_as_current_completion() {
    fn shape_text(shape: &egui::epaint::Shape, result: &mut String) {
        match shape {
            egui::epaint::Shape::Text(t) => {
                result.push_str(&t.galley.job.text);
                result.push('\n');
            }
            egui::epaint::Shape::Vec(shapes) => {
                for s in shapes {
                    shape_text(s, result);
                }
            }
            _ => {}
        }
    }
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-vlan-access")
        .unwrap();
    open_assignment(&mut a, i);
    a.set_switching_view(i, teaching::View::Assignment);
    let s = &a.scenarios[i];
    let mut state = s.initial.clone();
    for cmd in &s.solution {
        engine::execute(&mut state, cmd).unwrap();
    }
    let attempt = a.progress.attempts.get_mut(&s.id).unwrap();
    attempt.state = state;
    let w = attempt.workspace.as_mut().unwrap();
    w.checked = true;
    w.switching_draft
        .as_mut()
        .unwrap()
        .switches
        .get_mut("office")
        .unwrap()
        .ports
        .get_mut("staff-b")
        .unwrap()
        .access = "bad".into();
    assert!(engine::solved(&attempt.state, &s.goal));
    assert!(!progress::current_solved(attempt, s));
    let raw = serde_json::to_vec(&a.progress).unwrap();
    a.progress = progress::decode(&raw, &a.scenarios).unwrap();
    a.checked = true;
    draw(&ctx, &mut a);
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 850.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for shape in output.shapes {
        shape_text(&shape.shape, &mut text);
    }
    assert!(!a.metrics.completion);
    assert!(!text.contains("Objective complete"), "{text}");
    assert!(text.contains("Draft not verified"), "{text}");
}

#[test]
fn file_batch_lessons_keep_native_tools_in_width_and_challenge_guidance_optional() {
    for i in 55..58 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.5)] {
                let (ctx, mut a) = app();
                open_assignment(&mut a, i);
                a.set_script_view(i, view);
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{i} {view:?} {w}/{scale}: {r:?}"
                    );
                }
                if view != teaching::View::Introduction {
                    assert!(a.metrics.editor_viewport.unwrap().height() >= 175.);
                }
                assert!(!a.metrics.help_visible);
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                for _ in 0..3 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert!(a.metrics.lesson_tabs.is_empty());
                assert_eq!(a.script_view(i), teaching::View::Assignment);
                assert!(!a.metrics.help_visible);
            }
        }
    }
}

#[test]
fn file_batch_editors_support_keyboard_run_and_release_focus_for_navigation() {
    for i in 55..58 {
        let (ctx, mut a) = app();
        open_assignment(&mut a, i);
        a.set_script_view(i, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let editor = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(editor)));
        let source = a.scenarios[i].solution[0].clone();
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
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
                egui::Event::Text(source),
            ],
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            }],
        );
        for _ in 0..40 {
            draw(&ctx, &mut a);
            if a.script_job.is_none() {
                break;
            }
        }
        assert!(a.metrics.completion, "{}", a.scenarios[i].id);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Escape, false)],
        );
        draw(&ctx, &mut a);
        frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
        draw(&ctx, &mut a);
        assert!(!ctx.memory(|m| m.has_focus(editor)));
        assert!(ctx.memory(|m| m.focused()).is_some());
    }
}

#[test]
fn routing_v4_native_typing_check_retry_focus_and_selected_reset() {
    let (ctx, mut a) = app();
    open_assignment(&mut a, 9);
    let earlier = a.progress.attempts["shell-handover"].clone();
    let i = a.add_routing_fixture();
    open_assignment(&mut a, i);
    assert_eq!(a.routing_v4_view(i), teaching::View::Introduction);
    draw(&ctx, &mut a);
    let r = a.metrics.assignment_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let id = a.scenarios[i].id.clone();
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap()
            .selected_device,
        "Client"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text("192.0.2.99/24".into()),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(
        a.metrics.completion,
        "Manual entry should work without advanced settings"
    );
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text("malformed".into()),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    check(&ctx, &mut a);
    assert!(a.notice.as_ref().unwrap().contains("Malformed"));
    open_assignment(&mut a, 31);
    open_assignment(&mut a, i);
    draw(&ctx, &mut a);
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap()
            .devices["Client"]
            .interfaces["lan"]
            .address,
        "malformed"
    );
    let field = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(field)));
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(field)));
    a.reset_attempt(i);
    assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
    assert_eq!(a.progress.attempts["shell-handover"].state, earlier.state);
}

#[test]
fn routing_v4_native_views_keep_controls_in_width_with_large_text_and_challenge_is_independent() {
    for _ in 0..1 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for routes in [false, true] {
                for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.), (720., 560., 1.5)] {
                    let (ctx, mut a) = app();
                    let i = a.add_routing_fixture();
                    open_assignment(&mut a, i);
                    a.scale = scale;
                    a.set_routing_v4_view(i, view);
                    let workspace = a
                        .progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap();
                    workspace.routing_v4_draft.as_mut().unwrap().show_routes = routes;
                    workspace
                        .routing_v4_learning
                        .as_mut()
                        .unwrap()
                        .draft
                        .show_routes = routes;
                    for _ in 0..4 {
                        frame(&ctx, &mut a, w, h, vec![]);
                    }
                    for r in a
                        .metrics
                        .inputs
                        .iter()
                        .chain(a.metrics.check.iter())
                        .chain(a.metrics.lesson_tabs.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "lab{i} {view:?} {w}/{scale} {r:?}"
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    assert!(!a.metrics.guide);
                    if w == 1200. && view == teaching::View::Assignment && !routes {
                        assert!(
                            a.metrics.check.unwrap().bottom() < 850.,
                            "check below desktop lab{i}"
                        );
                    }
                    a.progress
                        .attempts
                        .get_mut(&a.scenarios[i].id)
                        .unwrap()
                        .challenge = true;
                    draw(&ctx, &mut a);
                    assert_eq!(a.routing_v4_view(i), teaching::View::Assignment);
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}

#[test]
fn routing_v4_single_interface_router_is_editable_and_guideless_help_is_safe() {
    let (ctx, mut a) = app();
    let i = a.add_routing_fixture();
    if let engine::Goal::Ipv4Routing { requirements } = &mut a.scenarios[i].goal {
        requirements.push(dlms_learning_lab::ipv4_routing::Requirement::Forwarding {
            device: "Client".into(),
            enabled: true,
        });
    }
    open_assignment(&mut a, i);
    a.set_routing_v4_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let r = a.metrics.routing_router_options.unwrap();
    click(&ctx, &mut a, r);
    let r = a.metrics.routing_forward.unwrap();
    click(&ctx, &mut a, r);
    assert!(
        a.progress.attempts[&a.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap()
            .devices["Client"]
            .forwarding
    );
    let r = a.metrics.help.unwrap();
    click(&ctx, &mut a, r);
    assert!(a.help_open);
    assert!(!a.metrics.guide); // Empty optional walkthrough is valid authoring data.
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
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
            egui::Event::Text("192.0.2.99/24".into()),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
}

#[test]
fn ipv4_authored_route_entry_supports_keyboard_tab_and_default_distance_without_reference_fill() {
    for keyboard_open in [true, false] {
        let (ctx, mut a) = app();
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-ipv4-network-route")
            .unwrap();
        open_assignment(&mut a, i);
        draw(&ctx, &mut a);
        let r = a.metrics.assignment_action.unwrap();
        click(&ctx, &mut a, r);
        assert!(a.metrics.inputs.is_empty());
        let r = a.metrics.routing_add.unwrap();
        click(&ctx, &mut a, r);
        let focus = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(focus)));
        let select = egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![select, egui::Event::Text("10.40.1.0/24".into())],
        );
        frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("10.0.0.2".into())],
        );
        draw(&ctx, &mut a);
        if keyboard_open {
            frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
            assert!(ctx.memory(|m| m.has_focus(a.metrics.routing_picker_id.unwrap())));
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![key(egui::Key::Enter, false)],
            );
            draw(&ctx, &mut a);
        } else {
            let r = a.metrics.routing_route_interface.unwrap();
            click(&ctx, &mut a, r);
        }
        let target = a
            .metrics
            .routing_port_options
            .iter()
            .find(|(port, _)| port == "transit")
            .unwrap()
            .1;
        let mut tabs = 0;
        while !ctx.memory(|m| m.has_focus(target)) && tabs < 8 {
            frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
            tabs += 1;
            let mut release = key(egui::Key::Tab, false);
            if let egui::Event::Key { pressed, .. } = &mut release {
                *pressed = false;
            }
            frame(&ctx, &mut a, 1200., 850., vec![release]);
        }
        assert!(
            ctx.memory(|m| m.has_focus(target)),
            "Outgoing interface option reachable by Tab"
        );
        assert_eq!(tabs, 1, "Selected menu option receives focus when opened");
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        assert!(
            a.metrics.completion,
            "notice {:?} draft {:?}",
            a.notice,
            a.progress.attempts[&a.scenarios[i].id]
                .workspace
                .as_ref()
                .unwrap()
                .routing_v4_draft
        );
        let id = a.scenarios[i].id.clone();
        let draft = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .routing_v4_draft
            .as_ref()
            .unwrap();
        assert_eq!(draft.devices["R1"].routes[0].distance, "1");
        let r = a.metrics.inputs[1];
        click(&ctx, &mut a, r);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("x".into())],
        );
        draw(&ctx, &mut a);
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
    }
}
#[test]
fn six_authored_ipv4_workspaces_keep_full_tasks_tools_and_guidance_separate_at_each_size() {
    for id in [
        "ccna-ipv4-network-route",
        "ccna-ipv4-return-route",
        "ccna-ipv4-specific-route",
        "ccna-ipv4-host-exception",
        "ccna-ipv4-floating-route",
        "ccna-ipv4-branch-handover",
    ] {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [
                (1200., 850., 1.),
                (1200., 850., 1.5),
                (1000., 800., 1.),
                (940., 700., 1.),
                (960., 700., 1.5),
                (720., 560., 1.),
                (720., 560., 1.5),
            ] {
                let (ctx, mut a) = app();
                let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
                open_assignment(&mut a, i);
                a.set_routing_v4_view(i, view);
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.guide);
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.lesson_tabs.iter())
                    .chain(a.metrics.routing_add.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{id} {view:?} {w}/{scale}: {r:?}"
                    );
                }
                if w == 1200. && scale == 1. && view == teaching::View::Assignment {
                    assert!(
                        a.metrics.check.unwrap().bottom() < 850.,
                        "Desktop Check below workspace {id}: {:?}",
                        a.metrics.check
                    );
                }
                a.progress.attempts.get_mut(id).unwrap().challenge = true;
                draw(&ctx, &mut a);
                assert_eq!(a.routing_v4_view(i), teaching::View::Assignment);
                assert!(a.metrics.lesson_tabs.is_empty());
                assert!(!a.metrics.help_visible);
            }
        }
    }
}

#[test]
fn addressing_introduction_examples_and_assignments_preserve_drafts_and_credit() {
    for i in [0, 1] {
        let (ctx, mut a) = app();
        let dir = tempfile::tempdir().unwrap();
        a.path = dir.path().join("progress.json");
        a.launch(i);
        draw(&ctx, &mut a);
        assert_eq!(a.address_view(i), teaching::View::Introduction);
        assert!(a.metrics.check.is_none());
        assert_eq!(a.metrics.lesson_tabs.len(), 3);
        assert!(!a.metrics.help_visible);
        let id = a.scenarios[i].id.clone();
        let original = a.progress.attempts[&id].state.clone();
        let r = a.metrics.example_action.unwrap();
        click(&ctx, &mut a, r);
        assert_eq!(a.address_view(i), teaching::View::Example);
        let example_focus = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(example_focus)));
        let r = a.metrics.insert.unwrap();
        click(&ctx, &mut a, r);
        check(&ctx, &mut a);
        draw(&ctx, &mut a);
        let example = &a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .address_learning
            .as_ref()
            .unwrap();
        assert!(example.solved());
        assert!(!a.metrics.completion);
        assert!(!progress::current_solved(
            &a.progress.attempts[&id],
            &a.scenarios[i]
        ));
        assert_eq!(a.progress.attempts[&id].state, original);
        let saved_example = serde_json::to_value(example).unwrap();
        // Requested worked help can extend a shorter viewport; use the real scroll path.
        for _ in 0..8 {
            if a.metrics.assignment_action.unwrap().bottom() < ctx.content_rect().bottom() {
                break;
            }
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    egui::Event::PointerMoved(ctx.content_rect().center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., -180.),
                        modifiers: Default::default(),
                    },
                ],
            );
        }
        let r = a.metrics.assignment_action.unwrap();
        click(&ctx, &mut a, r);
        assert_eq!(
            a.address_view(i),
            teaching::View::Assignment,
            "lab{i} action {r:?}"
        );
        assert_eq!(a.progress.attempts[&id].state, original);
        let actions = a.scenarios[i].solution.clone();
        for action in actions {
            engine::execute(
                &mut a.progress.attempts.get_mut(&id).unwrap().state,
                &action,
            )
            .unwrap();
        }
        a.load_ipv4_view(i);
        draw(&ctx, &mut a);
        check(&ctx, &mut a);
        assert!(a.metrics.completion);
        let solved = a.progress.attempts[&id].state.clone();
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        assert_eq!(a.progress.attempts[&id].state, solved);
        assert_eq!(
            serde_json::to_value(
                a.progress.attempts[&id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .address_learning
                    .as_ref()
                    .unwrap()
            )
            .unwrap()["state"],
            saved_example["state"]
        );
        let r = a.metrics.example_reset.unwrap();
        click(&ctx, &mut a, r);
        assert_eq!(a.progress.attempts[&id].state, solved);
        assert!(
            !a.progress.attempts[&id]
                .workspace
                .as_ref()
                .unwrap()
                .address_learning
                .as_ref()
                .unwrap()
                .solved()
        );
        a.save();
        a.progress = progress::load(&a.path, &a.scenarios).unwrap();
        a.launch(i);
        assert_eq!(a.address_view(i), teaching::View::Example);
        assert_eq!(a.progress.attempts[&id].state, solved);
        a.reset_attempt(i);
        assert_eq!(a.address_view(i), teaching::View::Introduction);
        assert_eq!(a.progress.attempts[&id].state, original);
        // Pre-teaching attempts reopen their intact assignment rather than forcing a lesson.
        a.progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .address_learning = None;
        a.launch(i);
        assert_eq!(a.address_view(i), teaching::View::Assignment);
    }
}
#[test]
fn addressing_fields_namespace_keyboard_undo_between_example_and_assignment() {
    for i in [0, 1] {
        let (ctx, mut a) = app();
        a.launch(i);
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        let example_id = a.metrics.focus.unwrap();
        let command = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        };
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("example-draft".into())],
        );
        draw(&ctx, &mut a);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                command(egui::Key::A),
                egui::Event::Text("example-edit".into()),
            ],
        );
        draw(&ctx, &mut a);
        let id = a.scenarios[i].id.clone();
        let example = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .address_learning
            .as_ref()
            .unwrap()
            .state
            .clone();
        a.set_address_view(i, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let assignment_id = a.metrics.focus.unwrap();
        assert_ne!(assignment_id, example_id);
        assert!(ctx.memory(|m| m.has_focus(assignment_id)));
        let original = a.progress.attempts[&id].state.clone();
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(a.progress.attempts[&id].state, original);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text("assignment-draft".into())],
        );
        draw(&ctx, &mut a);
        let draft = a.progress.attempts[&id].state.clone();
        assert_ne!(draft, original);
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        assert_eq!(
            a.progress.attempts[&id]
                .workspace
                .as_ref()
                .unwrap()
                .address_learning
                .as_ref()
                .unwrap()
                .state,
            example
        );
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(a.progress.attempts[&id].state, draft);
    }
}
#[test]
fn addressing_views_wrap_and_challenge_keeps_complete_independent_assignments() {
    for i in [0, 1] {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [
                (1200., 850., 1.),
                (960., 740., 1.5),
                (720., 560., 1.),
                (720., 560., 1.5),
            ] {
                let (ctx, mut a) = app();
                a.launch(i);
                a.scale = scale;
                a.set_address_view(i, view);
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.lesson_tabs.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{i} {view:?} {w}/{scale} {r:?}"
                    );
                }
                if let Some(r) = a.metrics.address_diagram {
                    assert!(
                        r.height() < 400. && r.right() <= ctx.content_rect().right() + 1.,
                        "diagram {w}/{scale} {r:?}"
                    );
                }
                if w == 1200. && view != teaching::View::Introduction {
                    assert!(a.metrics.check.unwrap().bottom() < 850.);
                }
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert_eq!(a.address_view(i), teaching::View::Assignment);
                assert!(a.metrics.lesson_tabs.is_empty());
                assert!(a.metrics.insert.is_none());
                assert_eq!(a.metrics.inputs.len(), if i == 0 { 5 } else { 3 });
                assert!(!a.metrics.help_visible);
                assert!(!a.metrics.help.is_none());
            }
        }
    }
}

#[test]
fn service_fresh_learning_teaches_runs_and_preserves_both_editor_terminal_drafts() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.launch(6);
    draw(&ctx, &mut a);
    assert_eq!(a.service_view(6), teaching::View::Introduction);
    assert!(a.metrics.check.is_none());
    let original = a.progress.attempts["lpic2-service"].state.clone();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(a.service_view(6), teaching::View::Example);
    let example_focus = a.metrics.focus.unwrap();
    assert!(ctx.memory(|m| m.has_focus(example_focus)));
    let r = a.metrics.command_insert.unwrap();
    click(&ctx, &mut a, r);
    let l = a.progress.attempts["lpic2-service"]
        .workspace
        .as_ref()
        .unwrap()
        .service_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.command, "systemctl status atlas");
    assert!(l.transcript.is_empty());
    assert_eq!(
        l.state,
        a.scenarios[6].service_teaching.as_ref().unwrap().fixture
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    assert!(
        !a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .transcript
            .is_empty()
    );
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .state,
        a.scenarios[6].service_teaching.as_ref().unwrap().fixture
    );
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    check(&ctx, &mut a);
    assert!(!a.metrics.completion);
    for command in ["systemctl restart atlas", "systemctl enable atlas"] {
        a.focus_input = true;
        draw(&ctx, &mut a);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Text(command.into())],
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Enter, false)],
        );
        draw(&ctx, &mut a);
    }
    check(&ctx, &mut a);
    assert!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .solved(a.scenarios[6].service_teaching.as_ref().unwrap())
    );
    assert!(!a.metrics.completion);
    assert_eq!(a.progress.attempts["lpic2-service"].state, original);
    a.set_service_view(6, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_ne!(example_focus, a.metrics.focus.unwrap());
    a.input = "original terminal draft".into();
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = a.input.clone();
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_port = Some("original file draft".into());
    a.set_service_view(6, teaching::View::Example);
    draw(&ctx, &mut a);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_learning
        .as_mut()
        .unwrap()
        .port_draft = "9091".into();
    draw(&ctx, &mut a);
    assert!(
        !a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .solved(a.scenarios[6].service_teaching.as_ref().unwrap())
    );
    a.save();
    a.progress = progress::load(&a.path, &a.scenarios).unwrap();
    a.launch(6);
    assert_eq!(a.service_view(6), teaching::View::Example);
    assert_eq!(a.input, "original terminal draft");
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_port
            .as_deref(),
        Some("original file draft")
    );
    assert_eq!(
        a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .as_ref()
            .unwrap()
            .port_draft,
        "9091"
    );
    a.reset_attempt(6);
    assert_eq!(a.service_view(6), teaching::View::Introduction);
    assert_eq!(a.progress.attempts["lpic2-service"].state, original);
    a.progress
        .attempts
        .get_mut("lpic2-service")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .service_learning = None;
    a.launch(6);
    assert_eq!(a.service_view(6), teaching::View::Assignment);
}
#[test]
fn service_views_wrap_and_challenge_keeps_the_full_editor_and_terminal() {
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        for (w, h, scale) in [
            (1200., 850., 1.),
            (960., 740., 1.5),
            (720., 560., 1.),
            (720., 560., 1.5),
        ] {
            let (ctx, mut a) = app();
            a.launch(6);
            a.scale = scale;
            a.set_service_view(6, view);
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.check.iter())
                .chain(a.metrics.terminal.iter())
                .chain(a.metrics.lesson_tabs.iter())
                .chain(a.metrics.example_action.iter())
                .chain(a.metrics.assignment_action.iter())
                .chain(a.metrics.command_insert.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "{view:?} {w}/{scale} {r:?}"
                );
            }
            if w == 1200. {
                if view == teaching::View::Introduction {
                    assert!(a.metrics.example_action.unwrap().bottom() < 850.);
                } else {
                    assert!(a.metrics.check.unwrap().bottom() < 850.);
                }
            }
            a.progress
                .attempts
                .get_mut("lpic2-service")
                .unwrap()
                .challenge = true;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            assert_eq!(a.service_view(6), teaching::View::Assignment);
            assert!(a.metrics.lesson_tabs.is_empty());
            assert!(a.metrics.command_insert.is_none());
            assert!(a.metrics.terminal.is_some());
            assert!(a.metrics.editor_save.is_some());
            assert!(!a.metrics.help_visible);
        }
    }
}
#[test]
fn service_example_and_assignment_editor_terminal_undo_histories_are_isolated() {
    for terminal in [false, true] {
        let (ctx, mut a) = app();
        a.launch(6);
        a.set_service_view(6, teaching::View::Example);
        draw(&ctx, &mut a);
        let r = if terminal {
            a.metrics.terminal.unwrap()
        } else {
            a.metrics.inputs[0]
        };
        click(&ctx, &mut a, r);
        let command = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                command: true,
                ..egui::Modifiers::CTRL
            },
        };
        for text in ["example-draft", "example-edit"] {
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![command(egui::Key::A), egui::Event::Text(text.into())],
            );
            draw(&ctx, &mut a);
        }
        let example = a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap()
            .service_learning
            .clone()
            .unwrap();
        assert_eq!(
            if terminal {
                &example.command
            } else {
                &example.port_draft
            },
            "example-edit"
        );
        a.set_service_view(6, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let r = if terminal {
            a.metrics.terminal.unwrap()
        } else {
            a.metrics.inputs[0]
        };
        click(&ctx, &mut a, r);
        let before = serde_json::to_value(&a.progress.attempts["lpic2-service"]).unwrap();
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        assert_eq!(
            serde_json::to_value(&a.progress.attempts["lpic2-service"]).unwrap(),
            before
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![
                command(egui::Key::A),
                egui::Event::Text("assignment-draft".into()),
            ],
        );
        draw(&ctx, &mut a);
        let draft = a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap();
        let draft = if terminal {
            draft.command.clone()
        } else {
            draft.service_port.clone().unwrap()
        };
        assert_eq!(draft, "assignment-draft");
        a.set_service_view(6, teaching::View::Example);
        draw(&ctx, &mut a);
        let r = if terminal {
            a.metrics.terminal.unwrap()
        } else {
            a.metrics.inputs[0]
        };
        click(&ctx, &mut a, r);
        frame(&ctx, &mut a, 1200., 850., vec![command(egui::Key::Z)]);
        draw(&ctx, &mut a);
        let w = a.progress.attempts["lpic2-service"]
            .workspace
            .as_ref()
            .unwrap();
        assert_eq!(
            if terminal {
                &w.command
            } else {
                w.service_port.as_ref().unwrap()
            },
            "assignment-draft"
        );
    }
}

fn accessibility_tree(
    ctx: &egui::Context,
    a: &mut LabApp,
    w: f32,
    h: f32,
) -> egui::accesskit::TreeUpdate {
    ctx.enable_accesskit();
    let mut result = None;
    for _ in 0..3 {
        result = ctx
            .run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(w / a.scale, h / a.scale),
                    )),
                    ..Default::default()
                },
                |ctx| a.render(ctx),
            )
            .platform_output
            .accesskit_update;
    }
    result.expect("native accessibility tree enabled")
}
fn accessible_named<'a>(
    tree: &'a egui::accesskit::TreeUpdate,
    name: &str,
) -> &'a egui::accesskit::Node {
    &tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(name))
        .unwrap_or_else(|| {
            panic!(
                "Missing native name {name}; names: {:?}",
                tree.nodes
                    .iter()
                    .filter_map(|(_, n)| n.label())
                    .collect::<Vec<_>>()
            )
        })
        .1
}
#[test]
fn native_accessibility_shared_assignments_expose_names_without_losing_values_or_roles() {
    use egui::accesskit::Role;
    let (_, inventory) = app();
    let labs: Vec<_> = inventory
        .scenarios
        .iter()
        .enumerate()
        .filter(|(i, s)| {
            *i < 9 || matches!(s.initial, State::Script { .. } | State::Filesystem { .. })
        })
        .map(|(i, _)| i)
        .collect();
    for i in labs {
        for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.5)] {
            let (ctx, mut a) = app();
            a.launch(i);
            a.scale = scale;
            let id = a.scenarios[i].id.clone();
            a.progress.attempts.get_mut(&id).unwrap().challenge = true;
            if let State::Script { source, .. } =
                &mut a.progress.attempts.get_mut(&id).unwrap().state
            {
                *source = "printf '%s\\n' 'Editable source'\n".into();
            }
            let tree = accessibility_tree(&ctx, &mut a, w, h);
            let mut count = 0;
            for (_, node) in &tree.nodes {
                if matches!(
                    node.role(),
                    Role::TextInput | Role::MultilineTextInput | Role::ComboBox
                ) {
                    assert!(
                        node.label().is_some_and(|s| !s.trim().is_empty()),
                        "lab{i} {w}/{scale}: unnamed {:?},value={:?},placeholder={:?}",
                        node.role(),
                        node.value(),
                        node.placeholder()
                    );
                    if matches!(node.role(), Role::TextInput | Role::MultilineTextInput) {
                        count += 1;
                    }
                }
            }
            assert!(count > 0, "lab{i} fields absent");
            let size = accessible_named(&tree, "Text size");
            assert_eq!(size.role(), Role::ComboBox);
            assert_eq!(
                size.value(),
                Some(if scale == 1. { "100%" } else { "150%" })
            );
            if matches!(a.scenarios[i].initial, State::Script { .. }) {
                let source = accessible_named(&tree, "Assignment script source");
                assert_eq!(source.role(), Role::MultilineTextInput);
                assert_eq!(source.value(), Some("printf '%s\\n' 'Editable source'\n"));
                assert_eq!(
                    accessible_named(&tree, "Inspect script test inputs").role(),
                    Role::ComboBox
                );
            }
            if i == 1 {
                for (name, hosts) in [("design", 60), ("support", 28), ("lab", 12)] {
                    assert_eq!(
                        accessible_named(
                            &tree,
                            &format!("{name} — network/prefix for {hosts} required hosts")
                        )
                        .role(),
                        Role::TextInput
                    );
                }
            }
        }
    }
}
#[test]
fn native_accessibility_examples_name_their_scope_and_preserve_editor_and_terminal_values() {
    use egui::accesskit::Role;
    for i in [1, 6, 9, 24] {
        let (ctx, mut a) = app();
        a.launch(i);
        match i {
            1 => a.set_address_view(i, teaching::View::Example),
            6 => a.set_service_view(i, teaching::View::Example),
            9 => a.set_script_view(i, teaching::View::Example),
            24 => a.set_terminal_view(i, teaching::View::Example),
            _ => unreachable!(),
        }
        let id = a.scenarios[i].id.clone();
        let w = a
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.service_learning {
            l.port_draft = "9091".into();
            l.command = "systemctl status atlas".into();
        }
        if let Some(l) = &mut w.script_learning {
            l.example_source = "printf '%s\\n' 'Example source'\n".into();
        }
        if let Some(l) = &mut w.terminal_learning {
            l.command = "pwd".into();
        }
        let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
        let checks = match i {
            1 => vec![("staff — network/prefix for 30 required hosts", "")],
            6 => vec![
                (
                    "Practice example service configuration port — ungraded",
                    "9091",
                ),
                (
                    "Service practice terminal command — ungraded",
                    "systemctl status atlas",
                ),
            ],
            9 => vec![(
                "Practice example script source — ungraded",
                "printf '%s\\n' 'Example source'\n",
            )],
            24 => vec![("Practice example terminal command — ungraded", "pwd")],
            _ => unreachable!(),
        };
        for (name, value) in checks {
            let n = accessible_named(&tree, name);
            assert_eq!(
                n.role(),
                if i == 9 {
                    Role::MultilineTextInput
                } else {
                    Role::TextInput
                }
            );
            assert_eq!(n.value(), Some(value));
        }
        for (_, n) in &tree.nodes {
            if n.role() == Role::ComboBox {
                assert!(
                    n.label().is_some(),
                    "example{i} unnamed picker {:?}",
                    n.value()
                );
            }
        }
    }
}
#[test]
fn native_accessibility_library_and_appearance_keep_filter_values_card_context_and_disabled_states()
{
    use egui::accesskit::Role;
    let (ctx, mut a) = app();
    let mut tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    assert!(tree.nodes.iter().any(|(_, n)| n.role() == Role::Button
        && n.label().is_some_and(|s| s.starts_with("Start lab: "))));
    assert!(tree.nodes.iter().any(|(_, n)| n.role() == Role::Button
        && n.label().is_some_and(|s| s.starts_with("Add favorite: "))));
    a.filter.search = "repair".into();
    a.filter.topic = "Services".into();
    a.filter.unit = "linux-service-recovery".into();
    a.filter.track = "LPIC-2".into();
    a.filter.difficulty = "Beginner".into();
    a.filter.status = "Complete".into();
    a.filter.max_minutes = 8;
    a.extra_filters = true;
    a.scale = 1.5;
    tree = accessibility_tree(&ctx, &mut a, 960., 740.);
    for (name, value) in [
        ("Search labs by title, goal or topic", "repair"),
        ("Topic filter", "Services"),
        ("Difficulty filter", "Beginner"),
        ("Completion status filter", "Complete"),
        ("Estimated time filter", "Up to 8 min"),
    ] {
        let n = accessible_named(&tree, name);
        assert_eq!(n.value(), Some(value));
    }
    assert_eq!(
        accessible_named(&tree, "Course unit filter").role(),
        Role::ComboBox
    );
    assert_eq!(
        accessible_named(&tree, "Application theme").role(),
        Role::ComboBox
    );
    a.theme_preview = true;
    tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    assert_eq!(
        accessible_named(&tree, "Theme sample editor — appearance example only").value(),
        Some("Select this editor text")
    );
    assert!(accessible_named(&tree, "Unavailable (disabled)").is_disabled());
}
#[test]
fn native_accessibility_backup_filename_has_a_name_and_retains_the_proposed_value() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    let r = a.metrics.data_export.unwrap();
    click(&ctx, &mut a, r);
    let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    let filename = accessible_named(&tree, "Progress backup filename");
    assert_eq!(filename.role(), egui::accesskit::Role::TextInput);
    assert_eq!(filename.value(), Some("dlms-learning-backup.json"));
    assert!(!filename.is_disabled());
}

#[test]
fn shared_singleline_centers_real_galleys_and_placeholders_at_font_metric_heights() {
    for body_size in [16., 24., 32.] {
        for text in [
            "",
            "192.168.10.77",
            "a very long value that scrolls within this bounded single-line field",
        ] {
            let (ctx, _) = app();
            ctx.style_mut(|s| {
                s.text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(body_size));
            });
            let mut value = text.to_owned();
            let mut geometry = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800., 600.),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    controls::configure(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            let h = controls::height(ui);
                            let field = controls::singleline(ui, &mut value)
                                .hint_text("IPv4 address")
                                .desired_width(240.)
                                .show(ui);
                            let button = ui.add(button("Check"));
                            let combo = egui::ComboBox::from_id_salt("alignment-test")
                                .selected_text("Selected")
                                .show_ui(ui, |_| {})
                                .response;
                            assert!((field.response.rect.height() - h).abs() < 1.);
                            assert!((button.rect.height() - h).abs() < 1.);
                            assert!((combo.rect.height() - h).abs() < 1.);
                            assert!(
                                (field.response.rect.center().y - button.rect.center().y).abs()
                                    < 1.
                            );
                            assert!(
                                (field.response.rect.center().y - combo.rect.center().y).abs() < 1.
                            );
                            let galley =
                                egui::Rect::from_min_size(field.galley_pos, field.galley.size());
                            assert!(
                                (galley.center().y - field.response.rect.center().y).abs() < 1.
                            );
                            assert!(field.text_clip_rect.left() >= field.response.rect.left());
                            assert!(field.text_clip_rect.right() <= field.response.rect.right());
                            geometry = Some(field.response.rect);
                        });
                    });
                },
            );
            fn find_hint(s: &egui::Shape) -> Option<egui::Rect> {
                match s {
                    egui::Shape::Text(t) if t.galley.job.text == "IPv4 address" => {
                        Some(egui::Rect::from_min_size(t.pos, t.galley.size()))
                    }
                    egui::Shape::Vec(v) => v.iter().find_map(find_hint),
                    _ => None,
                }
            }
            if text.is_empty() {
                let hint = output
                    .shapes
                    .iter()
                    .find_map(|s| find_hint(&s.shape))
                    .expect("real painted placeholder");
                assert!((hint.center().y - geometry.unwrap().center().y).abs() < 1.);
            }
            assert_eq!(value, text);
        }
    }
}

#[test]
fn all_authored_assignment_controls_fit_at_two_hundred_percent_and_vlandrafts_survive_reflow() {
    let (_, inventory) = app();
    for i in 0..inventory.scenarios.len() {
        for (w, h) in [(1920., 1080.), (720., 700.)] {
            let (ctx, mut a) = app();
            open_assignment(&mut a, i);
            a.scale = 2.;
            a.progress
                .attempts
                .get_mut(&a.scenarios[i].id)
                .unwrap()
                .challenge = true;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            // A no-wrap configuration editor can have wider scroll contents;
            // its visible viewport, rather than its child text rectangle, must fit.
            let terminal = if matches!(a.scenarios[i].initial, State::PersistentMounts { .. }) {
                a.metrics.editor_viewport.iter()
            } else {
                a.metrics.terminal.iter()
            };
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(terminal)
                .chain(a.metrics.check.iter())
                .chain(a.metrics.editor_save.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "lab{i} {w}/200%: {r:?} vs {:?}",
                    ctx.content_rect()
                );
            }
        }
    }
    let (ctx, mut a) = app();
    let i = a
        .scenarios
        .iter()
        .position(|s| matches!(s.initial, State::Switching { .. }))
        .unwrap();
    open_assignment(&mut a, i);
    a.progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .challenge = true;
    for _ in 0..4 {
        frame(&ctx, &mut a, 1920., 1080., vec![]);
    }
    let id = a.metrics.vlan_fields[0].0;
    ctx.memory_mut(|m| m.request_focus(id));
    frame(
        &ctx,
        &mut a,
        1920.,
        1080.,
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
            egui::Event::Text("30".into()),
        ],
    );
    for _ in 0..4 {
        frame(&ctx, &mut a, 720., 700., vec![]);
    }
    assert_eq!(
        a.metrics.vlan_fields[0].0, id,
        "same actual focused editor across wide/stacked grid"
    );
    assert!(ctx.memory(|m| m.has_focus(id)));
    let draft = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap()
        .switching_draft
        .as_ref()
        .unwrap();
    assert_eq!(
        draft.switches[&draft.selected_switch]
            .ports
            .values()
            .next()
            .unwrap()
            .access,
        "30"
    );
    frame(
        &ctx,
        &mut a,
        720.,
        700.,
        vec![egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
        }],
    );
    let draft = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap()
        .switching_draft
        .as_ref()
        .unwrap();
    assert_eq!(
        draft.switches[&draft.selected_switch]
            .ports
            .values()
            .next()
            .unwrap()
            .access,
        "20"
    );
}

#[test]
fn two_hundred_percent_appearance_selection_round_trips_without_changing_progress() {
    let temp = tempfile::tempdir().unwrap();
    let p = temp.path().join("preferences-v1.json");
    let prefs = theme::Preferences {
        version: 1,
        theme: "dlms-light".into(),
        text_scale: 2.,
    };
    prefs.save(&p).unwrap();
    assert_eq!(theme::Preferences::load(&p).unwrap().text_scale, 2.);
}

#[test]
fn shared_native_field_clicks_select_and_edit_the_character_under_the_cursor() {
    fn edit_frame(
        ctx: &egui::Context,
        value: &mut String,
        events: Vec<egui::Event>,
    ) -> egui::text_edit::TextEditOutput {
        let mut output = None;
        let modifiers = events
            .iter()
            .find_map(|e| {
                if let egui::Event::Key { modifiers, .. } = e {
                    Some(*modifiers)
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800., 600.),
                )),
                events,
                modifiers,
                ..Default::default()
            },
            |ctx| {
                controls::configure(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    output = Some(
                        controls::singleline(ui, value)
                            .id(egui::Id::new("aligned-hit-editor"))
                            .desired_width(240.)
                            .show(ui),
                    );
                });
            },
        );
        output.unwrap()
    }
    for size in [16., 32.] {
        let (ctx, _) = app();
        ctx.style_mut(|s| {
            s.text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(size));
        });
        let mut value = "ABCD".to_owned();
        let o = edit_frame(&ctx, &mut value, vec![]);
        let cursor = o.galley.pos_from_cursor(egui::text::CCursor::new(2));
        let point = egui::pos2(
            o.galley_pos.x + cursor.center().x,
            o.response.rect.center().y,
        );
        for pressed in [true, false] {
            let o = edit_frame(
                &ctx,
                &mut value,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
            assert_eq!(o.cursor_range.unwrap().primary.index, 2);
        }
        let o = edit_frame(
            &ctx,
            &mut value,
            vec![egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::SHIFT,
            }],
        );
        let selection = o.cursor_range.unwrap();
        assert_eq!(
            selection.primary.index.abs_diff(selection.secondary.index),
            1
        );
        edit_frame(&ctx, &mut value, vec![egui::Event::Text("X".into())]);
        assert_eq!(value, "ABXD");
    }
}

#[test]
fn explicit_worked_fill_explains_only_the_example_and_reset_hides_the_help() {
    for i in [0, 1] {
        let (ctx, mut a) = app();
        a.launch(i);
        let id = a.scenarios[i].id.clone();
        let original = a.progress.attempts[&id].state.clone();
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
        assert!(
            tree.nodes
                .iter()
                .any(|(_, n)| n.label().or(n.value()) == Some("Fill worked example"))
        );
        assert!(!tree.nodes.iter().any(|(_, n)| {
            n.label()
                .or(n.value())
                .is_some_and(|s| s.starts_with("Worked example:"))
        }));
        let fill = a.metrics.insert.unwrap();
        click(&ctx, &mut a, fill);
        let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
        assert!(
            tree.nodes.iter().any(|(_, n)| {
                n.label()
                    .or(n.value())
                    .is_some_and(|s| s.starts_with("Worked example:") && s.contains("ungraded"))
            }),
            "tree text: {:?}",
            tree.nodes
                .iter()
                .filter_map(|(_, n)| n.label().or(n.value()).or(n.value()))
                .collect::<Vec<_>>()
        );
        assert_eq!(a.progress.attempts[&id].state, original);
        let learning = a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .address_learning
            .as_ref()
            .unwrap();
        assert!(learning.checked.is_none());
        assert!(!learning.solved());
        let reset = a.metrics.example_reset.unwrap();
        click(&ctx, &mut a, reset);
        let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
        assert!(!tree.nodes.iter().any(|(_, n)| {
            n.label()
                .or(n.value())
                .is_some_and(|s| s.starts_with("Worked example:"))
        }));
        let fill = a.metrics.insert.unwrap();
        click(&ctx, &mut a, fill);
        assert!(a.address_worked.contains_key(&id));
        let reset = a.metrics.reset.unwrap();
        click(&ctx, &mut a, reset);
        let confirm = a.metrics.confirm_reset.unwrap();
        assert!(
            confirm.top() >= 0. && confirm.bottom() < ctx.content_rect().bottom(),
            "{id}: confirm {confirm:?}"
        );
        click(&ctx, &mut a, confirm);
        assert!(
            !a.address_worked.contains_key(&id),
            "{id}: reset_pending={}, confirm={confirm:?}",
            a.reset_pending
        );
        a.set_address_view(i, teaching::View::Example);
        draw(&ctx, &mut a);
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
        assert!(!tree.nodes.iter().any(|(_, n)| {
            n.label().or(n.value()) == Some("Fill worked example")
                || n.label()
                    .or(n.value())
                    .is_some_and(|s| s.starts_with("Worked example:"))
        }));
        assert_eq!(a.metrics.inputs.len(), if i == 0 { 5 } else { 3 });
    }
}

#[test]
fn validated_restore_clears_requested_worked_help_without_losing_imported_example_drafts() {
    let (ctx, mut a) = app();
    let temp = tempfile::tempdir().unwrap();
    a.path = temp.path().join("progress.json");
    a.smoke = true;
    a.launch(0);
    a.set_address_view(0, teaching::View::Example);
    draw(&ctx, &mut a);
    dlms_learning_lab::backup::export(
        &temp.path().join("dlms-learning-backup.json"),
        &a.progress,
        &a.scenarios,
    )
    .unwrap();
    let fill = a.metrics.insert.unwrap();
    click(&ctx, &mut a, fill);
    assert!(a.address_worked.contains_key("ccna-ipv4"));
    a.settings_open = true;
    a.settings_data = true;
    draw(&ctx, &mut a);
    data_preview(&ctx, &mut a);
    let confirm = a.metrics.data_confirm.unwrap();
    click(&ctx, &mut a, confirm);
    assert!(a.address_worked.is_empty());
    assert!(a.address_example.is_none());
    a.settings_open = false;
    a.launch(0);
    a.set_address_view(0, teaching::View::Example);
    draw(&ctx, &mut a);
    assert!(
        a.address_example
            .as_ref()
            .unwrap()
            .exercise
            .answers
            .iter()
            .all(String::is_empty)
    );
    let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    assert!(!tree.nodes.iter().any(|(_, n)| {
        n.label()
            .or(n.value())
            .is_some_and(|s| s.starts_with("Worked example:"))
    }));
}

#[test]
fn dns_workspace_examples_keep_drafts_credit_and_keyboard_focus_separate() {
    let (ctx, mut a) = app();
    let i = a.add_dns_fixture();
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(a.dns_view(i), teaching::View::Introduction);
    assert_eq!(a.metrics.lesson_tabs.len(), 3);
    let assignment = serde_json::to_vec(&a.progress.attempts[&a.scenarios[i].id].state).unwrap();
    let r = a.metrics.example_action.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.dns_view(i), teaching::View::Example);
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let l = a.progress.attempts[&a.scenarios[i].id]
        .workspace
        .as_ref()
        .unwrap()
        .dns_learning
        .as_ref()
        .unwrap();
    assert!(!l.checked);
    assert!(!l.draft.matches(&l.server));
    assert_eq!(
        serde_json::to_vec(&a.progress.attempts[&a.scenarios[i].id].state).unwrap(),
        assignment
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[&a.scenarios[i].id],
        &a.scenarios[i]
    ));
    let id = a.scenarios[i].id.clone();
    a.set_dns_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let editor = a.metrics.terminal.unwrap();
    click(&ctx, &mut a, editor);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Text("; My independent draft\n".into())],
    );
    draw(&ctx, &mut a);
    let w = a.progress.attempts[&id].workspace.as_ref().unwrap();
    assert!(
        w.dns_draft
            .as_ref()
            .unwrap()
            .text
            .contains("My independent draft")
    );
    assert!(
        !w.dns_learning
            .as_ref()
            .unwrap()
            .draft
            .text
            .contains("My independent draft")
    );
    let focused = a.metrics.focus.unwrap();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    draw(&ctx, &mut a);
    assert_ne!(a.metrics.focus, Some(focused));
    a.set_dns_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    a.set_dns_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .dns_draft
            .as_ref()
            .unwrap()
            .text
            .contains("My independent draft")
    );
    a.progress.attempts.get_mut(&id).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(a.metrics.insert.is_none());
    assert!(a.metrics.editor_save.is_some());
    assert!(a.metrics.dns_reload.is_some());
}
#[test]
fn dns_workspace_save_reload_update_checks_and_unsaved_changes_invalidate_current_success() {
    use dlms_learning_lab::dns_workspace::Draft;
    let (ctx, mut a) = app();
    let i = a.add_dns_fixture();
    a.launch(i);
    a.set_dns_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let id = a.scenarios[i].id.clone();
    let mut reference = a.scenarios[i].initial.clone();
    for action in &a.scenarios[i].solution {
        engine::execute(&mut reference, action).unwrap();
    }
    let State::AuthoritativeDns { server } = &reference else {
        panic!()
    };
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_draft = Some(Draft::new(server));
    draw(&ctx, &mut a);
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = a.metrics.check.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let r = a.metrics.dns_reload.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = a.metrics.check.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    let r = a.metrics.dns_update.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    let r = a.metrics.check.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .dns_draft
        .as_mut()
        .unwrap()
        .text
        .push_str("\nmalformed draft");
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    let before = a.progress.attempts[&id].state.clone();
    let r = a.metrics.editor_save.unwrap();
    click(&ctx, &mut a, r);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&id].state, before);
    assert!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .dns_draft
            .as_ref()
            .unwrap()
            .notice
            .as_ref()
            .unwrap()
            .contains("Line ")
    );
    a.reset_attempt(i);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
    assert_eq!(a.dns_view(i), teaching::View::Introduction);
}
#[test]
fn dns_workspace_reflows_full_editor_lookup_and_check_at_large_text_and_narrow_widths() {
    let (ctx, mut a) = app();
    let i = a.add_dns_fixture();
    a.launch(i);
    a.set_dns_view(i, teaching::View::Assignment);
    for (w, h, scale) in [
        (1920., 1080., 1.),
        (1200., 900., 1.5),
        (960., 900., 2.),
        (720., 900., 1.5),
    ] {
        a.scale = scale;
        for _ in 0..4 {
            frame(&ctx, &mut a, w, h, vec![]);
        }
        let bounds = ctx.content_rect();
        if w / scale >= 1000. {
            assert!(
                a.metrics.inputs[1].width() >= 295.,
                "Desktop lookup name width must be reserved"
            );
            assert!(
                !a.metrics.dns_editor_overflow,
                "Short record lines need no horizontal scrolling on desktop"
            );
        }
        for r in a
            .metrics
            .inputs
            .iter()
            .chain(a.metrics.editor_viewport.iter())
            .chain(a.metrics.check.iter())
            .chain(a.metrics.dns_reload.iter())
            .chain(a.metrics.dns_update.iter())
            .chain(a.metrics.dns_pickers.iter())
            .chain(a.metrics.probe.iter())
        {
            assert!(
                r.left() >= bounds.left() - 1. && r.right() <= bounds.right() + 1.,
                "control {r:?} outside {bounds:?} at{scale}"
            );
        }
        // Check may scroll vertically at larger text. It must remain reachable by real wheel input.
        for _ in 0..12 {
            let r = a.metrics.check.unwrap();
            if r.bottom() <= bounds.bottom() - 20. && r.top() > 80. {
                break;
            }
            let delta = (bounds.bottom() - r.bottom() - 35.).min(-30.);
            frame(
                &ctx,
                &mut a,
                w,
                h,
                vec![
                    egui::Event::PointerMoved(bounds.center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., delta),
                        modifiers: Default::default(),
                    },
                ],
            );
        }
        for _ in 0..4 {
            frame(&ctx, &mut a, w, h, vec![]);
        }
        assert!(
            a.metrics.check.unwrap().bottom() <= bounds.bottom(),
            "Check not reachable at{scale}"
        );
    }
}

#[test]
fn dns_secondary_alias_outcome_keeps_secondary_controls_available() {
    use dlms_learning_lab::{
        dns::QueryType,
        dns_workspace::{Requirement, Snapshot},
    };
    let (ctx, mut a) = app();
    let i = a.add_dns_fixture();
    let engine::Goal::AuthoritativeDns { requirements } = &mut a.scenarios[i].goal else {
        panic!()
    };
    requirements.retain(|r| {
        matches!(
            r,
            Requirement::Resolves {
                copy: Snapshot::Primary,
                ..
            }
        )
    });
    requirements.push(Requirement::Published { secondary: false });
    requirements.push(Requirement::AliasTo {
        copy: Snapshot::Secondary,
        owner: "portal.preview.test.".into(),
        record_type: QueryType::A,
        target: "app.preview.test.".into(),
    });
    a.launch(i);
    a.set_dns_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(a.metrics.dns_update.is_some());
}

#[test]
fn dns_record_editor_accepts_long_ascii_and_bounds_unicode_without_losing_progress() {
    let (ctx, mut a) = app();
    let dir = tempfile::tempdir().unwrap();
    a.path = dir.path().join("progress.json");
    let i = a.add_dns_fixture();
    a.launch(i);
    a.set_dns_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let r = a.metrics.editor_viewport.unwrap();
    click(&ctx, &mut a, r);
    let ctrl = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let select = || egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: ctrl,
    };
    let text = format!(";{}", "x".repeat(14_000));
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![select(), egui::Event::Text(text.clone())],
    );
    draw(&ctx, &mut a);
    let id = a.scenarios[i].id.clone();
    assert_eq!(
        a.progress.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .dns_draft
            .as_ref()
            .unwrap()
            .text,
        text
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![select(), egui::Event::Text("é".repeat(24_001))],
    );
    draw(&ctx, &mut a);
    let draft = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .dns_draft
        .as_ref()
        .unwrap();
    assert_eq!(draft.text.len(), 48_000);
    assert!(draft.valid());
    assert!(draft.text.ends_with('é'));
    progress::save(&a.path, &a.progress).unwrap();
    let p = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        p.attempts[&id]
            .workspace
            .as_ref()
            .unwrap()
            .dns_draft
            .as_ref()
            .unwrap()
            .text,
        draft.text
    );
}
#[test]
fn dns_example_and_assignment_keep_independent_native_undo_histories() {
    let (ctx, mut a) = app();
    let i = a.add_dns_fixture();
    a.launch(i);
    let id = a.scenarios[i].id.clone();
    let ctrl = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    let key = |key| egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: ctrl,
    };
    a.set_dns_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let r = a.metrics.editor_viewport.unwrap();
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            key(egui::Key::A),
            egui::Event::Text("; practice-only edit".into()),
        ],
    );
    draw(&ctx, &mut a);
    let practice = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .dns_learning
        .as_ref()
        .unwrap()
        .draft
        .text
        .clone();
    assert!(practice.contains("practice-only"));
    a.set_dns_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let r = a.metrics.editor_viewport.unwrap();
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            key(egui::Key::A),
            egui::Event::Text("; assignment-only edit".into()),
        ],
    );
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Z)]);
    draw(&ctx, &mut a);
    let w = a.progress.attempts[&id].workspace.as_ref().unwrap();
    assert!(!w.dns_draft.as_ref().unwrap().text.contains("practice-only"));
    assert!(
        !w.dns_draft
            .as_ref()
            .unwrap()
            .text
            .contains("assignment-only")
    );
    assert_eq!(w.dns_learning.as_ref().unwrap().draft.text, practice);
    let assignment = w.dns_draft.as_ref().unwrap().text.clone();
    a.set_dns_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let r = a.metrics.editor_viewport.unwrap();
    click(&ctx, &mut a, r);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Z)]);
    draw(&ctx, &mut a);
    let w = a.progress.attempts[&id].workspace.as_ref().unwrap();
    assert!(
        !w.dns_learning
            .as_ref()
            .unwrap()
            .draft
            .text
            .contains("assignment-only")
    );
    assert_ne!(w.dns_learning.as_ref().unwrap().draft.text, practice);
    assert_eq!(w.dns_draft.as_ref().unwrap().text, assignment);
}

#[test]
fn authored_dns_lessons_keep_complete_tasks_and_tools_in_both_modes_at_scaled_sizes() {
    let (ctx, mut a) = app();
    let indices = a
        .scenarios
        .iter()
        .enumerate()
        .filter(|(_, s)| s.unit == "dns-forward")
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 6);
    for i in indices {
        a.launch(i);
        draw(&ctx, &mut a);
        assert_eq!(a.dns_view(i), teaching::View::Introduction);
        assert!(a.metrics.example_action.is_some());
        a.set_dns_view(i, teaching::View::Assignment);
        for (w, h, scale) in [(1920., 1080., 1.), (1200., 900., 1.5), (960., 900., 2.)] {
            a.scale = scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            let bounds = ctx.content_rect();
            assert!(a.metrics.editor_save.is_some());
            assert!(a.metrics.dns_reload.is_some());
            assert!(a.metrics.check.is_some());
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.dns_pickers.iter())
                .chain(a.metrics.probe.iter())
                .chain(a.metrics.editor_viewport.iter())
            {
                assert!(
                    r.left() >= bounds.left() - 1. && r.right() <= bounds.right() + 1.,
                    "{} {r:?} at{scale}",
                    a.scenarios[i].id
                );
            }
        }
        let id = a.scenarios[i].id.clone();
        a.progress.attempts.get_mut(&id).unwrap().challenge = true;
        a.set_dns_view(i, teaching::View::Introduction);
        for _ in 0..4 {
            frame(&ctx, &mut a, 960., 900., vec![]);
        }
        assert_eq!(a.dns_view(i), teaching::View::Assignment);
        assert!(a.metrics.lesson_tabs.is_empty());
        assert!(a.metrics.insert.is_none());
        assert!(a.metrics.editor_save.is_some());
        assert!(a.metrics.check.is_some());
        a.reset_attempt(i);
        assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
        assert_eq!(a.dns_view(i), teaching::View::Introduction);
    }
}

#[test]
fn synthetic_native_profiles_are_private_distinct_and_cleaned_without_touching_normal_data() {
    let (_ctx, first) = app();
    let (_ctx, second) = app();
    let first_dir = first.path.parent().unwrap().to_path_buf();
    let second_dir = second.path.parent().unwrap().to_path_buf();
    assert_ne!(first_dir, second_dir);
    assert!(first._smoke_profile.is_some());
    assert!(second._smoke_profile.is_some());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&first_dir).unwrap().permissions().mode() & 0o077,
            0
        );
    }
    std::fs::write(&first.path, b"synthetic fixture").unwrap();
    drop(first);
    assert!(!first_dir.exists());
    assert!(second_dir.exists());
    drop(second);
    assert!(!second_dir.exists());
}

#[test]
fn array_lessons_keep_native_tools_in_width_and_challenge_guidance_optional() {
    for i in 70..74 {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [(1200., 850., 1.), (720., 560., 1.5), (720., 700., 2.)] {
                let (ctx, mut a) = app();
                open_assignment(&mut a, i);
                a.set_script_view(i, view);
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{i} {view:?} {w}/{scale}: {r:?}"
                    );
                }
                if view != teaching::View::Introduction {
                    assert!(a.metrics.editor_viewport.unwrap().height() >= 175.);
                }
                assert!(!a.metrics.help_visible);
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                for _ in 0..3 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert!(a.metrics.lesson_tabs.is_empty());
                assert_eq!(a.script_view(i), teaching::View::Assignment);
                assert!(!a.metrics.help_visible);
            }
        }
    }
}

#[test]
fn array_editors_support_keyboard_run_and_release_focus_for_navigation() {
    for i in 70..74 {
        let (ctx, mut a) = app();
        open_assignment(&mut a, i);
        a.set_script_view(i, teaching::View::Assignment);
        draw(&ctx, &mut a);
        let editor = a.metrics.focus.unwrap();
        assert!(ctx.memory(|m| m.has_focus(editor)));
        let source = a.scenarios[i].solution[0].clone();
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
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
                egui::Event::Text(source),
            ],
        );
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::CTRL,
            }],
        );
        for _ in 0..40 {
            draw(&ctx, &mut a);
            if a.script_job.is_none() {
                break;
            }
        }
        assert!(a.metrics.completion, "{}", a.scenarios[i].id);
        frame(
            &ctx,
            &mut a,
            1200.,
            850.,
            vec![key(egui::Key::Escape, false)],
        );
        draw(&ctx, &mut a);
        frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
        draw(&ctx, &mut a);
        assert!(!ctx.memory(|m| m.has_focus(editor)));
        assert!(ctx.memory(|m| m.focused()).is_some());
    }
}

#[test]
fn long_array_script_clicks_use_visible_viewport_and_edits_clear_current_success() {
    let (ctx, mut a) = app();
    let i = 73;
    open_assignment(&mut a, i);
    a.set_script_view(i, teaching::View::Assignment);
    let s = &a.scenarios[i];
    let id = s.id.clone();
    let source = s.solution[0].clone();
    let engine::Goal::Script { cases } = &s.goal else {
        panic!()
    };
    a.progress.attempts.get_mut(&id).unwrap().state = State::Script {
        source: source.clone(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(&source, c)).collect(),
        }),
    };
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    let editor = a.metrics.focus.unwrap();
    ctx.memory_mut(|m| m.surrender_focus(editor));
    let viewport = a.metrics.editor_viewport.unwrap();
    assert!(
        a.metrics.inputs[0].height() > viewport.height(),
        "the full text rectangle extends outside its scroll viewport"
    );
    click(&ctx, &mut a, viewport.shrink(4.));
    assert!(ctx.memory(|m| m.has_focus(editor)));
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Text("printf changed".into()),
        ],
    );
    draw(&ctx, &mut a);
    let State::Script { source, run } = &a.progress.attempts[&id].state else {
        panic!()
    };
    assert_eq!(source, "printf changed");
    assert!(run.is_none());
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
}

#[test]
fn process_workspace_views_keep_tasks_tools_and_guidance_separate_at_responsive_sizes() {
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        for (w, h, scale) in [(1200., 850., 1.), (720., 700., 1.5), (720., 700., 2.)] {
            let (ctx, mut a) = app();
            let i = a.add_process_fixture();
            a.launch(i);
            a.set_process_view(i, view);
            a.scale = scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.check.iter())
                .chain(a.metrics.example_action.iter())
                .chain(a.metrics.assignment_action.iter())
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "{view:?}/{scale}: {r:?}"
                );
            }
            assert!(!a.metrics.help_visible);
            a.progress
                .attempts
                .get_mut(&a.scenarios[i].id)
                .unwrap()
                .challenge = true;
            for _ in 0..3 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            assert_eq!(a.process_view(i), teaching::View::Assignment);
            assert!(a.metrics.lesson_tabs.is_empty());
            assert!(!a.metrics.help_visible);
        }
    }
}
#[test]
fn process_terminal_keyboard_check_invalidation_and_focus_remain_native() {
    let (ctx, mut a) = app();
    let i = a.add_process_fixture();
    a.launch(i);
    a.set_process_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    let r = a.metrics.terminal.unwrap();
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Text("kill -s SIGSTOP 101".into()),
            key(egui::Key::Enter, false),
        ],
    );
    draw(&ctx, &mut a);
    check(&ctx, &mut a);
    assert!(a.metrics.completion);
    let id = a.scenarios[i].id.clone();
    assert!(progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    let r = a.metrics.terminal.unwrap();
    click(&ctx, &mut a, r);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Text("kill -CONT 101".into()),
            key(egui::Key::Enter, false),
        ],
    );
    draw(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    let focus = a.metrics.focus.unwrap();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Escape, false)],
    );
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    draw(&ctx, &mut a);
    assert!(!ctx.memory(|m| m.has_focus(focus)));
    assert!(ctx.memory(|m| m.focused()).is_some());
}
#[test]
fn process_insert_does_not_execute_and_example_cannot_overwrite_assignment() {
    let (ctx, mut a) = app();
    let i = a.add_process_fixture();
    a.launch(i);
    let id = a.scenarios[i].id.clone();
    let initial = a.progress.attempts[&id].state.clone();
    a.set_process_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let r = a.metrics.insert.unwrap();
    click(&ctx, &mut a, r);
    let l = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .process_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.command, "ps -eo pid,ppid,user,stat,ni,comm");
    assert!(l.transcript.is_empty());
    assert_eq!(a.progress.attempts[&id].state, initial);
    let r = a.metrics.terminal.unwrap();
    click(&ctx, &mut a, r);
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Text("kill -STOP 610".into()),
        ],
    );
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![key(egui::Key::Enter, false)],
    );
    draw(&ctx, &mut a);
    let l = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .process_learning
        .as_ref()
        .unwrap();
    assert!(matches!(
        l.table.processes[&610].phase,
        dlms_learning_lab::processes::Phase::Stopped { .. }
    ));
    assert_eq!(a.progress.attempts[&id].state, initial);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    a.set_process_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&id].state, initial);
}

#[test]
fn discovery_native_views_reflow_without_clipping_fields_or_revealing_challenge_guidance() {
    for view in [
        teaching::View::Introduction,
        teaching::View::Example,
        teaching::View::Assignment,
    ] {
        for (w, h, scale) in [
            (1920., 1080., 1.),
            (1200., 850., 1.),
            (720., 700., 1.5),
            (720., 700., 2.),
        ] {
            let (ctx, mut a) = app();
            let i = a.add_discovery_fixture();
            a.launch(i);
            a.set_discovery_view(i, view);
            a.scale = scale;
            for _ in 0..4 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            for r in a
                .metrics
                .inputs
                .iter()
                .chain(a.metrics.editor_save.iter())
                .chain(a.metrics.check.iter())
                .chain(a.metrics.discovery_send.iter().map(|(_, r)| r))
            {
                assert!(
                    r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                    "{view:?} {w}/{scale}: {r:?} vs {:?}; fields {:?}",
                    ctx.content_rect(),
                    a.metrics.discovery_fields
                );
            }
            a.progress
                .attempts
                .get_mut(&a.scenarios[i].id)
                .unwrap()
                .challenge = true;
            for _ in 0..3 {
                frame(&ctx, &mut a, w, h, vec![]);
            }
            assert_eq!(a.discovery_view(i), teaching::View::Assignment);
            assert!(a.metrics.lesson_tabs.is_empty());
            assert!(!a.metrics.help_visible);
        }
    }
}
#[test]
fn discovery_example_fill_is_nonexecuting_and_keeps_assignment_and_drafts_independent() {
    let (ctx, mut a) = app();
    let i = a.add_discovery_fixture();
    a.launch(i);
    let id = a.scenarios[i].id.clone();
    let original = a.progress.attempts[&id].state.clone();
    a.set_discovery_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let before = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .discovery_learning
        .as_ref()
        .unwrap()
        .simulation
        .clone();
    let r = a.metrics.command_insert.unwrap();
    click(&ctx, &mut a, r);
    let l = a.progress.attempts[&id]
        .workspace
        .as_ref()
        .unwrap()
        .discovery_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.simulation, before);
    assert!(!l.draft.matches(&before));
    assert!(!l.checked);
    assert_eq!(a.progress.attempts[&id].state, original);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
    a.set_discovery_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&id].state, original);
}
#[test]
fn discovery_single_line_editing_uses_real_focus_and_invalidates_success_before_apply() {
    let (ctx, mut a) = app();
    let i = a.add_discovery_fixture();
    a.launch(i);
    let id = a.scenarios[i].id.clone();
    for action in a.scenarios[i].solution.clone() {
        engine::execute(
            &mut a.progress.attempts.get_mut(&id).unwrap().state,
            &action,
        )
        .unwrap();
    }
    let engine::State::Discovery { simulation } = &a.progress.attempts[&id].state else {
        panic!()
    };
    let d = dlms_learning_lab::discovery_workspace::Draft::new(simulation);
    a.progress
        .attempts
        .get_mut(&id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .discovery_draft = Some(d);
    a.set_discovery_view(i, teaching::View::Assignment);
    a.checked = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    let r = a.metrics.inputs[0];
    click(&ctx, &mut a, r);
    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
            egui::Event::Text("invalid".into()),
        ],
    );
    draw(&ctx, &mut a);
    let w = a.progress.attempts[&id].workspace.as_ref().unwrap();
    let d = w.discovery_draft.as_ref().unwrap();
    assert_eq!(d.devices[&d.selected_device].ttl, "invalid");
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[&id],
        &a.scenarios[i]
    ));
}

#[test]
fn shared_topology_cards_wrap_long_values_and_align_equal_rows_without_clipping_text() {
    for width in [310., 470., 1100.] {
        for star in [false, true] {
            let ctx = egui::Context::default();
            theme::apply(&ctx, "dlms-dark");
            let long = "long-device-interface-identifier-with-no-whitespace-01234567890123";
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 1800.),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    controls::configure(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let hub = [
                            long,
                            "An optional advertised name that should wrap safely",
                            "LLDP enabled",
                        ];
                        let peer = ["short/port", "Short name", "LLDP disabled"];
                        if !star
                            || !controls::topology_star(
                                ui,
                                hub,
                                &[peer, hub, peer],
                                "Wrapped caption about the direct physical connection",
                            )
                        {
                            controls::topology_pair(
                                ui,
                                hub,
                                peer,
                                palette(ui).muted,
                                "Wrapped caption about the direct physical connection",
                            );
                        }
                    });
                },
            );
            let cards: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|s| {
                    if let egui::Shape::Rect(r) = &s.shape {
                        (r.stroke.width == 1.).then_some(r.rect)
                    } else {
                        None
                    }
                })
                .collect();
            let count = if star && width > 400. { 4 } else { 2 };
            assert_eq!(cards.len(), count);
            assert!(
                cards
                    .iter()
                    .all(|c| (c.height() - cards[0].height()).abs() < 0.1)
            );
            let mut labels = 0;
            for s in &output.shapes {
                if let egui::Shape::Text(t) = &s.shape {
                    if t.galley.text().starts_with("Wrapped caption") {
                        continue;
                    }
                    let r = egui::Rect::from_min_size(t.pos, t.galley.size());
                    assert!(
                        cards.iter().any(|c| c.shrink(3.).contains_rect(r)),
                        "{width}: text {:?} at {r:?} outside {cards:?}",
                        t.galley.text()
                    );
                    labels += 1;
                }
                if count == 4
                    && let egui::Shape::LineSegment { points, .. } = &s.shape
                {
                    assert_eq!(points[0], cards[0].center_bottom());
                    assert!(cards[1..].iter().any(|c| points[1] == c.center_top()));
                }
            }
            assert_eq!(labels, count * 3, "all original labels remain painted");
        }
    }
}

#[test]
fn authored_discovery_lessons_keep_all_tools_in_width_and_guidance_optional_at_large_text() {
    let (_, inventory) = app();
    let labs: Vec<_> = inventory
        .scenarios
        .iter()
        .enumerate()
        .filter(|(_, s)| s.unit == "lldp-discovery")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(labs.len(), 4);
    for i in labs {
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            for (w, h, scale) in [
                (1200., 850., 1.),
                (720., 700., 1.5),
                (720., 700., 2.),
                (1920., 1080., 2.),
            ] {
                let (ctx, mut a) = app();
                a.launch(i);
                a.set_discovery_view(i, view);
                a.scale = scale;
                for _ in 0..4 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.discovery_send.iter().map(|(_, r)| r))
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.editor_save.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{} {view:?} {w}/{scale} {r:?}",
                        a.scenarios[i].id
                    );
                }
                a.progress
                    .attempts
                    .get_mut(&a.scenarios[i].id)
                    .unwrap()
                    .challenge = true;
                for _ in 0..3 {
                    frame(&ctx, &mut a, w, h, vec![]);
                }
                assert!(a.metrics.lesson_tabs.is_empty() && !a.metrics.help_visible);
                assert_eq!(a.discovery_view(i), teaching::View::Assignment);
            }
        }
    }
}

#[test]
fn native_accessibility_topology_images_expose_addresses_connections_and_updated_states() {
    use egui::accesskit::Role;
    let (ctx, mut a) = app();
    open_assignment(&mut a, 0);
    let tree = accessibility_tree(&ctx, &mut a, 1200., 850.);
    let diagram = tree
        .nodes
        .iter()
        .map(|(_, n)| n)
        .find(|n| {
            n.role() == Role::Image && n.label().is_some_and(|s| s.starts_with("Network diagram."))
        })
        .expect("painted topology has a native image description");
    let label = diagram.label().unwrap();
    assert!(
        label.contains("192.168.10.77") && label.contains("Workstation 2") && label.contains("/27")
    );
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "ccna-lldp-handover")
        .unwrap();
    open_assignment(&mut a, i);
    let summary = |tree: &egui::accesskit::TreeUpdate| {
        tree.nodes
            .iter()
            .filter_map(|(_, n)| (n.role() == Role::Image).then(|| n.label()).flatten())
            .find(|s| s.starts_with("Network diagram."))
            .unwrap()
            .to_owned()
    };
    let before = summary(&accessibility_tree(&ctx, &mut a, 1200., 850.));
    assert!(
        before.contains("finance; Finance; LLDP disabled") && before.contains("relay/finance"),
        "{before}"
    );
    let engine::State::Discovery { simulation } = &mut a
        .progress
        .attempts
        .get_mut("ccna-lldp-handover")
        .unwrap()
        .state
    else {
        panic!()
    };
    let mut topology = simulation.topology().clone();
    topology.devices.get_mut("finance").unwrap().enabled = true;
    simulation.reconfigure(topology).unwrap();
    let after = summary(&accessibility_tree(&ctx, &mut a, 1200., 850.));
    assert!(after.contains("finance; Finance; LLDP enabled"), "{after}");
    assert_ne!(before, after);
}

#[test]
fn explicit_configuration_check_reveals_success_or_retry_feedback_below_the_fold() {
    for solved in [false, true] {
        let (ctx, mut a) = app();
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "ccna-lldp-handover")
            .unwrap();
        open_assignment(&mut a, i);
        let id = a.scenarios[i].id.clone();
        if solved {
            for command in a.scenarios[i].solution.clone() {
                engine::execute(
                    &mut a.progress.attempts.get_mut(&id).unwrap().state,
                    &command,
                )
                .unwrap();
            }
            let State::Discovery { simulation } = &a.progress.attempts[&id].state else {
                panic!()
            };
            let draft = dlms_learning_lab::discovery_workspace::Draft::new(simulation);
            a.progress
                .attempts
                .get_mut(&id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap()
                .discovery_draft = Some(draft);
        }
        draw(&ctx, &mut a);
        for _ in 0..20 {
            let r = a.metrics.check.unwrap();
            if r.bottom() <= 825. && r.top() > 85. {
                break;
            }
            frame(
                &ctx,
                &mut a,
                1200.,
                850.,
                vec![
                    egui::Event::PointerMoved(egui::pos2(600., 500.)),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0., (825. - r.bottom()).min(-25.)),
                        modifiers: Default::default(),
                    },
                ],
            );
        }
        for _ in 0..20 {
            draw(&ctx, &mut a);
        }
        let state_before = a.progress.attempts[&id].state.clone();
        let draft_before = serde_json::to_vec(
            &a.progress.attempts[&id]
                .workspace
                .as_ref()
                .unwrap()
                .discovery_draft,
        )
        .unwrap();
        let r = a.metrics.check.unwrap();
        assert!(r.bottom() <= 850.);
        click(&ctx, &mut a, r);
        for _ in 0..20 {
            draw(&ctx, &mut a);
        }
        let feedback = a.metrics.feedback.unwrap();
        assert!(feedback.height() > 0.);
        assert!(
            feedback.top() >= 60. && feedback.bottom() <= 850.,
            "solved{solved} feedback{feedback:?}"
        );
        assert_eq!(a.metrics.completion, solved);
        assert_eq!(a.progress.attempts[&id].state, state_before);
        assert_eq!(
            serde_json::to_vec(
                &a.progress.attempts[&id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .discovery_draft
            )
            .unwrap(),
            draft_before
        );
    }
}

#[test]
fn native_storage_fixture_reflows_editors_terminal_and_actions_without_font_locking() {
    for (width, height) in [(1200., 850.), (720., 560.)] {
        for scale in [1., 1.5, 2.] {
            for (view, challenge) in [
                (teaching::View::Introduction, false),
                (teaching::View::Example, false),
                (teaching::View::Assignment, false),
                (teaching::View::Assignment, true),
            ] {
                let (ctx, mut a) = app();
                let i = a.add_storage_fixture();
                a.launch(i);
                a.set_storage_view(i, view);
                a.scale = scale;
                a.progress
                    .attempts
                    .get_mut("test-storage-workspace")
                    .unwrap()
                    .challenge = challenge;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                for r in a
                    .metrics
                    .inputs
                    .iter()
                    .chain(a.metrics.editor_viewport.iter())
                    .chain(a.metrics.output.iter())
                    .chain(a.metrics.editor_save.iter())
                    .chain(a.metrics.storage_run.iter())
                    .chain(a.metrics.storage_validate.iter())
                    .chain(a.metrics.storage_test.iter())
                    .chain(a.metrics.check.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale} {view:?} challenge{challenge}: {r:?}"
                    );
                }
                assert!(!a.metrics.help_visible);
                if challenge {
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
            }
        }
    }
}

#[test]
fn storage_fresh_output_shows_actual_identity_including_skipped_manual_sources() {
    let (_, mut a) = app();
    let i = a.add_storage_fixture();
    let s = a.scenarios[i].clone();
    let mut state = s.initial.clone();
    for command in &s.solution {
        engine::execute(&mut state, command).unwrap();
    }
    let State::PersistentMounts { storage } = &mut state else {
        unreachable!()
    };
    storage
        .save_configuration(
            storage
                .configuration()
                .replace("UUID=22222222-2222-4222-8222-222222222222", "/dev/vdc1")
                .replace("/reports ext4 ro", "/reports ext4 ro,noauto"),
        )
        .unwrap();
    let engine::Goal::PersistentMounts { requirements, .. } = &s.goal else {
        unreachable!()
    };
    let before = storage.clone();
    let output = storage_workspace::fresh_report(storage, requirements);
    assert!(output.contains("Saved manual /reports: resolves to volume reports"));
    assert!(output.contains("Saved manual /reports: resolves to volume work"));
    assert!(output.contains("UUID=11111111-1111-4111-8111-111111111111"));
    assert_eq!(*storage, before);
}

#[test]
fn storage_large_text_outer_wheel_scroll_reaches_the_check_below_nested_editors() {
    let (ctx, mut a) = app();
    let i = a.add_storage_fixture();
    open_assignment(&mut a, i);
    a.scale = 2.;
    a.theme_id = "dlms-dark".into();
    theme::apply(&ctx, &a.theme_id);
    for _ in 0..4 {
        frame(&ctx, &mut a, 822., 382., vec![]);
    }
    let before = a.metrics.main_scroll_offset;
    let theme = a.metrics.header[3];
    let settings = a.metrics.settings.unwrap();
    assert!(
        (theme.center().y - settings.center().y).abs() < 1.,
        "related toolbar controls should share their row when measured widths fit: theme{theme:?} settings{settings:?}"
    );
    for _ in 0..20 {
        let viewport = a.metrics.main_viewport.unwrap();
        let target = a.metrics.check.unwrap();
        if target.bottom() <= viewport.bottom() && target.top() >= viewport.top() {
            break;
        }
        let point = egui::pos2(viewport.left() + 1., viewport.center().y);
        let delta = if target.bottom() > viewport.bottom() {
            viewport.bottom() - target.bottom()
        } else {
            viewport.top() - target.top()
        };
        frame(
            &ctx,
            &mut a,
            822.,
            382.,
            vec![
                egui::Event::PointerMoved(point),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., delta),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..10 {
            frame(&ctx, &mut a, 822., 382., vec![]);
        }
    }
    assert!(
        a.metrics.main_scroll_offset.y > before.y,
        "before{before:?} after{:?} viewport{:?} size{:?} pointer{:?}",
        a.metrics.main_scroll_offset,
        a.metrics.main_viewport,
        a.metrics.main_content_size,
        ctx.input(|i| i.pointer.hover_pos())
    );
    let r = a.metrics.check.unwrap();
    let vp = a.metrics.main_viewport.unwrap();
    assert!(
        r.bottom() <= vp.bottom() + 1. && r.top() >= vp.top() - 1.,
        "check{r:?} viewport{vp:?} offset{:?}",
        a.metrics.main_scroll_offset
    );
}

#[test]
fn four_authored_storage_lessons_keep_full_tasks_editors_and_terminal_at_all_supported_scales() {
    let (_, base) = app();
    let indices = base
        .scenarios
        .iter()
        .enumerate()
        .filter_map(|(i, s)| (s.unit == "persistent-mounts").then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 4);
    for i in indices {
        for (width, height) in [(1200., 850.), (720., 560.)] {
            for scale in [1., 1.5, 2.] {
                for (view, challenge) in [
                    (teaching::View::Introduction, false),
                    (teaching::View::Example, false),
                    (teaching::View::Assignment, false),
                    (teaching::View::Assignment, true),
                ] {
                    let (ctx, mut a) = app();
                    a.launch(i);
                    a.set_storage_view(i, view);
                    a.scale = scale;
                    let id = a.scenarios[i].id.clone();
                    a.progress.attempts.get_mut(&id).unwrap().challenge = challenge;
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, height, vec![]);
                    }
                    for r in a
                        .metrics
                        .inputs
                        .iter()
                        .chain(a.metrics.editor_viewport.iter())
                        .chain(a.metrics.output.iter())
                        .chain(a.metrics.editor_save.iter())
                        .chain(a.metrics.storage_run.iter())
                        .chain(a.metrics.storage_validate.iter())
                        .chain(a.metrics.storage_test.iter())
                        .chain(a.metrics.check.iter())
                    {
                        assert!(
                            r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                            "{id}/{width}/{scale}/{view:?}: {r:?} vs{:?}",
                            ctx.content_rect()
                        );
                    }
                    assert!(!a.metrics.help_visible);
                    if challenge {
                        assert!(a.metrics.lesson_tabs.is_empty());
                    }
                }
            }
        }
    }
}

#[test]
fn storage_editor_keyboard_changes_preserve_applied_state_and_keep_separate_undo() {
    let command_key = |key| egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    };
    let (ctx, mut a) = app();
    let i = a.add_storage_fixture();
    open_assignment(&mut a, i);
    draw(&ctx, &mut a);
    let original = a.progress.attempts["test-storage-workspace"].state.clone();
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            command_key(egui::Key::A),
            egui::Event::Text("broken editor".into()),
        ],
    );
    assert_eq!(
        a.progress.attempts["test-storage-workspace"]
            .workspace
            .as_ref()
            .unwrap()
            .storage_draft
            .as_ref()
            .unwrap()
            .text,
        "broken editor"
    );
    assert_eq!(
        a.progress.attempts["test-storage-workspace"].state,
        original
    );
    frame(&ctx, &mut a, 1200., 850., vec![command_key(egui::Key::Z)]);
    assert_eq!(
        a.progress.attempts["test-storage-workspace"]
            .workspace
            .as_ref()
            .unwrap()
            .storage_draft
            .as_ref()
            .unwrap()
            .text,
        if let State::PersistentMounts { storage } = &original {
            storage.configuration()
        } else {
            unreachable!()
        }
    );
    a.set_storage_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            command_key(egui::Key::A),
            egui::Event::Text("separate example edit".into()),
        ],
    );
    assert_eq!(
        a.progress.attempts["test-storage-workspace"].state,
        original
    );
    a.set_storage_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert_ne!(
        a.progress.attempts["test-storage-workspace"]
            .workspace
            .as_ref()
            .unwrap()
            .storage_draft
            .as_ref()
            .unwrap()
            .text,
        "separate example edit"
    );
    let before = ctx.memory(|m| m.focused());
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), before);
}

#[test]
fn backup_chooser_reserves_native_footer_for_recovery_files_and_long_paths() {
    for scale in [1., 1.5, 2.] {
        for (width, height) in [(1920., 1080.), (960., 900.)] {
            let temp = tempfile::tempdir().unwrap();
            let folder = temp
                .path()
                .join("a-long-local-recovery-folder-with-many-existing-safety-copies");
            std::fs::create_dir(&folder).unwrap();
            for n in 0..24 {
                std::fs::write(
                    folder.join(format!("progress-before-restore-{n}.json")),
                    b"{}",
                )
                .unwrap();
            }
            let (ctx, mut a) = app();
            a.smoke = true;
            a.path = folder.join("progress.json");
            a.scale = scale;
            // Settle the application's scale before opening a new floating area;
            // egui rescales existing window memory when the zoom changes.
            for _ in 0..8 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            a.settings_open = true;
            a.settings_data = true;
            for _ in 0..8 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            let r = a.metrics.data_restore.unwrap();
            frame(
                &ctx,
                &mut a,
                width,
                height,
                vec![egui::Event::PointerMoved(r.center())],
            );
            for pressed in [true, false] {
                frame(
                    &ctx,
                    &mut a,
                    width,
                    height,
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
            for _ in 0..8 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            let clip = a.metrics.data_file_viewport.unwrap_or_else(|| panic!("chooser did not open at {width}x{height}/{scale}, restore={r:?}, viewport={:?}, settings={}: {}", ctx.content_rect(), a.settings_open, a.data_tools.restore_diagnostic())).intersect(ctx.content_rect()).expand(1.);
            for control in [
                a.metrics.data_filename.unwrap(),
                a.metrics.data_file_action.unwrap(),
            ] {
                assert!(
                    clip.contains_rect(control),
                    "{width}x{height} at {scale}: {control:?} outside {clip:?}"
                );
            }
            assert!(a.data_tools.modal_open());
            assert!(a.progress.attempts.is_empty());
        }
    }
}

fn capture_lesson_indexes(a: &LabApp) -> Vec<usize> {
    [
        "shell-capture-preview",
        "shell-capture-evidence",
        "shell-capture-decision",
        "shell-capture-handover",
    ]
    .iter()
    .map(|id| {
        a.scenarios
            .iter()
            .position(|s| &s.id == id)
            .expect("authored capture lesson bundled in the normal app")
    })
    .collect()
}
#[test]
fn capture_lessons_keep_full_tasks_and_separate_examples_at_large_text_without_exposing_challenge_help()
 {
    for ordinal in 0..4 {
        for (width, height, scale) in [(1920., 1080., 1.), (1200., 850., 1.5), (960., 900., 2.)] {
            let (ctx, mut a) = app();
            let indexes = capture_lesson_indexes(&a);
            let i = indexes[ordinal];
            a.scale = scale;
            a.launch(i);
            let id = a.scenarios[i].id.clone();
            for view in [
                teaching::View::Introduction,
                teaching::View::Example,
                teaching::View::Assignment,
            ] {
                a.set_script_view(i, view);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                assert_eq!(a.script_view(i), view);
                assert!(!a.metrics.help_visible);
                assert_eq!(a.progress.attempts[&id].state, a.scenarios[i].initial);
                assert!(!progress::current_solved(
                    &a.progress.attempts[&id],
                    &a.scenarios[i]
                ));
                for rect in a
                    .metrics
                    .lesson_tabs
                    .iter()
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                    .chain(a.metrics.editor_viewport.iter())
                {
                    assert!(
                        rect.left() >= 0. && rect.right() <= ctx.content_rect().right() + 1.,
                        "{id} {view:?} {width}/{scale}: {rect:?}"
                    );
                }
            }
            a.progress.attempts.get_mut(&id).unwrap().challenge = true;
            for _ in 0..3 {
                frame(&ctx, &mut a, width, height, vec![]);
            }
            assert_eq!(a.script_view(i), teaching::View::Assignment);
            assert!(a.metrics.lesson_tabs.is_empty());
            assert!(!a.metrics.help_visible);
            assert!(a.metrics.editor_viewport.is_some() && a.metrics.check.is_some());
        }
    }
}
#[test]
fn status_feedback_accepts_meaningful_nonzero_results_and_explains_a_wrong_zero() {
    let (ctx, mut a) = app();
    let indexes = capture_lesson_indexes(&a);
    let i = indexes[2];
    a.launch(i);
    a.set_script_view(i, teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    let engine::Goal::Script { cases } = &s.goal else {
        panic!()
    };
    assert_eq!(cases[0].status, 1);
    let source = s.solution[0].replace("exit 1", "exit 0");
    let results: Vec<_> = cases.iter().map(|c| shell::run(&source, c)).collect();
    assert_eq!(results[0].error, None);
    assert_eq!(results[0].stdout, cases[0].stdout);
    assert_eq!(results[0].stderr, cases[0].stderr);
    assert_eq!(results[0].status, 0);
    a.progress.attempts.get_mut(&s.id).unwrap().state = State::Script {
        source: source.clone(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source,
            results,
        }),
    };
    let mut text = String::new();
    for _ in 0..3 {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920., 1440.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        fn collect(s: &egui::Shape, text: &mut String) {
            match s {
                egui::Shape::Text(t) => {
                    text.push_str(t.galley.text());
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
        for s in output.shapes {
            collect(&s.shape, &mut text);
        }
    }
    assert!(!a.metrics.completion);
    assert!(
        text.contains("Expected exit status: 1 · Actual: 0"),
        "{text}"
    );
    assert!(text.contains("a nonzero status can be the correct result"));
    assert!(!text.contains("A successful task uses 0"));
    let results = cases
        .iter()
        .map(|c| shell::run(&s.solution[0], c))
        .collect();
    a.progress.attempts.get_mut(&s.id).unwrap().state = State::Script {
        source: s.solution[0].clone(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: s.solution[0].clone(),
            results,
        }),
    };
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
}

fn acl_ctrl(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    }
}
fn acl_replace(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    a.focus_input = true;
    draw(ctx, a);
    frame(
        ctx,
        a,
        1200.,
        850.,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
}
#[test]
fn acl_native_workspace_reflows_at_all_scales_and_challenge_keeps_guidance_optional() {
    for (width, height) in [(1200., 850.), (720., 560.)] {
        for scale in [1., 1.5, 2.] {
            for (view, challenge) in [
                (teaching::View::Introduction, false),
                (teaching::View::Example, false),
                (teaching::View::Assignment, false),
                (teaching::View::Assignment, true),
            ] {
                let (ctx, mut a) = app();
                let i = a.add_acl_fixture();
                a.launch(i);
                a.set_acl_view(i, view);
                a.scale = scale;
                a.progress
                    .attempts
                    .get_mut("test-acl-workspace")
                    .unwrap()
                    .challenge = challenge;
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                for rect in a
                    .metrics
                    .lesson_tabs
                    .iter()
                    .chain(a.metrics.acl_router.iter())
                    .chain(a.metrics.acl_diagram.iter())
                    .chain(a.metrics.editor_viewport.iter())
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        rect.left() >= 0. && rect.right() <= ctx.content_rect().right() + 1.,
                        "{width}/{scale} {view:?} challenge={challenge}: {rect:?} vs {:?}",
                        ctx.content_rect()
                    );
                }
                assert!(!a.metrics.help_visible);
                if challenge {
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
                if view != teaching::View::Introduction || challenge {
                    assert!(a.metrics.editor_viewport.is_some());
                }
            }
        }
    }
}
#[test]
fn acl_native_check_is_fresh_keyboard_edits_allow_retry_and_reset_resume_is_exact() {
    let (ctx, mut a) = app();
    let i = a.add_acl_fixture();
    a.launch(i);
    a.set_acl_view(i, teaching::View::Assignment);
    let s = a.scenarios[i].clone();
    draw(&ctx, &mut a);
    let initial = s.initial.clone();
    // Native editor input and Ctrl+Enter traverse the same callbacks as the button.
    acl_replace(&ctx, &mut a, "not supported IOS syntax");
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    draw(&ctx, &mut a);
    assert_eq!(a.progress.attempts[&s.id].state, initial);
    assert!(
        a.progress.attempts[&s.id]
            .workspace
            .as_ref()
            .unwrap()
            .acl_draft
            .as_ref()
            .unwrap()
            .notice
            .is_some()
    );
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    let partial = "ip access-list extended partial\n permit tcp any any eq 443\nexit\ninterface office\n ip access-group partial in";
    acl_replace(&ctx, &mut a, partial);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    draw(&ctx, &mut a);
    let engine::State::Ipv4Acl { network } = &a.progress.attempts[&s.id].state else {
        panic!()
    };
    let engine::Goal::Ipv4Acl { definition } = &s.goal else {
        panic!()
    };
    let d = &a.progress.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .acl_draft
        .as_ref()
        .unwrap()
        .documents;
    let checks = dlms_learning_lab::acl_workspace::checks(network, definition, d);
    assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
    let alternate = "ip access-list extended alternate\n deny tcp any any eq 22\n permit ip any any\nexit\ninterface office\n ip access-group alternate in";
    acl_replace(&ctx, &mut a, alternate);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Enter)]);
    draw(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(a.metrics.completion);
    a.set_acl_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    a.progress.attempts.get_mut(&s.id).unwrap().challenge = true;
    draw(&ctx, &mut a);
    assert!(a.metrics.completion);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!a.metrics.help_visible);
    a.progress.attempts.get_mut(&s.id).unwrap().challenge = false;
    a.set_acl_view(i, teaching::View::Assignment);
    let encoded = serde_json::to_vec(&a.progress).unwrap();
    a.progress = progress::decode(&encoded, &a.scenarios).unwrap();
    a.launch(i);
    a.set_acl_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    assert!(progress::current_solved(&a.progress.attempts[&s.id], &s));
    acl_replace(&ctx, &mut a, "changed pending draft");
    draw(&ctx, &mut a);
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
    assert!(!a.metrics.completion);
    assert!(!a.checked);
    let other = a.scenarios[0].id.clone();
    a.launch(0);
    let keep = serde_json::to_vec(&a.progress.attempts[&other]).unwrap();
    a.launch(i);
    a.reset_attempt(i);
    assert_eq!(a.progress.attempts[&s.id].state, initial);
    assert_eq!(
        serde_json::to_vec(&a.progress.attempts[&other]).unwrap(),
        keep
    );
}
#[test]
fn acl_example_fill_is_unapplied_ungraded_and_uses_separate_native_undo() {
    let (ctx, mut a) = app();
    let i = a.add_acl_fixture();
    a.launch(i);
    a.set_acl_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    acl_replace(&ctx, &mut a, "assignment draft retained");
    let assignment = serde_json::to_vec(
        &a.progress.attempts["test-acl-workspace"]
            .workspace
            .as_ref()
            .unwrap()
            .acl_draft,
    )
    .unwrap();
    a.set_acl_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let before = a.progress.attempts["test-acl-workspace"]
        .workspace
        .as_ref()
        .unwrap()
        .acl_learning
        .as_ref()
        .unwrap()
        .network
        .clone();
    let fill = a.metrics.insert.unwrap();
    click(&ctx, &mut a, fill);
    let l = a.progress.attempts["test-acl-workspace"]
        .workspace
        .as_ref()
        .unwrap()
        .acl_learning
        .as_ref()
        .unwrap();
    assert_eq!(l.network, before);
    assert!(!l.checked);
    assert_eq!(
        l.draft.documents.routers,
        a.scenarios[i]
            .acl_teaching
            .as_ref()
            .unwrap()
            .example_documents
            .routers
    );
    assert!(!progress::current_solved(
        &a.progress.attempts["test-acl-workspace"],
        &a.scenarios[i]
    ));
    acl_replace(&ctx, &mut a, "separate example draft");
    assert_eq!(
        serde_json::to_vec(
            &a.progress.attempts["test-acl-workspace"]
                .workspace
                .as_ref()
                .unwrap()
                .acl_draft
        )
        .unwrap(),
        assignment
    );
    let focused = ctx.memory(|m| m.focused());
    frame(&ctx, &mut a, 1200., 850., vec![key(egui::Key::Tab, false)]);
    assert_ne!(ctx.memory(|m| m.focused()), focused);
    a.set_acl_view(i, teaching::View::Assignment);
    draw(&ctx, &mut a);
    frame(&ctx, &mut a, 1200., 850., vec![acl_ctrl(egui::Key::Z)]);
    assert!(
        !a.progress.attempts["test-acl-workspace"]
            .workspace
            .as_ref()
            .unwrap()
            .acl_draft
            .as_ref()
            .unwrap()
            .documents
            .routers
            .values()
            .any(|s| s == "separate example draft")
    );
}

#[test]
fn acl_native_router_documents_preserve_the_aggregate_draft_bound_before_saving() {
    let (ctx, mut a) = app();
    let i = a.add_acl_fixture();
    let State::Ipv4Acl { network } = &mut a.scenarios[i].initial else {
        panic!()
    };
    let mut spare = network.topology.devices["Gateway"].clone();
    for port in spare.interfaces.values_mut() {
        port.up = false;
        port.address = None;
    }
    spare.routes.clear();
    for name in ["SpareA", "SpareB", "SpareC", "SpareD"] {
        network.topology.devices.insert(name.into(), spare.clone());
    }
    let initial_network = network.clone();
    let engine::Goal::Ipv4Acl { definition } = &mut a.scenarios[i].goal else {
        panic!()
    };
    definition.initial = initial_network;
    a.launch(i);
    a.set_acl_view(i, teaching::View::Assignment);
    let draft = a
        .progress
        .attempts
        .get_mut("test-acl-workspace")
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .acl_draft
        .as_mut()
        .unwrap();
    for (name, text) in &mut draft.documents.routers {
        *text = "#".repeat(if name == "Gateway" { 20_000 } else { 60_000 });
    }
    assert!(draft.valid());
    let before = draft.documents.routers["Gateway"].clone();
    acl_replace(&ctx, &mut a, &"#".repeat(30_000));
    let draft = a.progress.attempts["test-acl-workspace"]
        .workspace
        .as_ref()
        .unwrap()
        .acl_draft
        .as_ref()
        .unwrap();
    assert!(draft.valid());
    assert_eq!(draft.documents.routers["Gateway"], before);
    assert!(draft.notice.as_ref().unwrap().contains("256 KiB"));
    progress::decode(&serde_json::to_vec(&a.progress).unwrap(), &a.scenarios).unwrap();
}
#[test]
fn acl_native_tab_change_focuses_the_new_editor_in_the_same_rendered_frame() {
    let (ctx, mut a) = app();
    let i = a.add_acl_fixture();
    a.launch(i);
    a.set_acl_view(i, teaching::View::Example);
    draw(&ctx, &mut a);
    let old = a.metrics.focus.unwrap();
    let assignment = a.metrics.lesson_tabs[2];
    click(&ctx, &mut a, assignment);
    assert_eq!(a.acl_view(i), teaching::View::Assignment);
    assert_ne!(a.metrics.focus, Some(old));
    assert_eq!(ctx.memory(|m| m.focused()), a.metrics.focus);
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            acl_ctrl(egui::Key::A),
            egui::Event::Text("new assignment only".into()),
        ],
    );
    let w = a.progress.attempts["test-acl-workspace"]
        .workspace
        .as_ref()
        .unwrap();
    assert_eq!(
        w.acl_draft.as_ref().unwrap().documents.routers["Gateway"],
        "new assignment only"
    );
    assert!(
        !w.acl_learning
            .as_ref()
            .unwrap()
            .draft
            .documents
            .routers
            .values()
            .any(|s| s == "new assignment only")
    );
}

#[test]
fn acl_authored_units_keep_full_assignments_tables_editors_and_optional_challenge_at_all_scales() {
    for id in [
        "ccna-acl-source-policy",
        "ccna-acl-service-order",
        "ccna-acl-direction-repair",
        "ccna-acl-policy-handover",
    ] {
        for (width, height, scale) in [(1200., 850., 1.), (720., 560., 1.5), (960., 900., 2.)] {
            let (ctx, mut a) = app();
            let i = a
                .scenarios
                .iter()
                .position(|s| s.id == id)
                .expect("authored ACL lesson registered");
            a.scale = scale;
            a.launch(i);
            for (view, challenge) in [
                (teaching::View::Introduction, false),
                (teaching::View::Example, false),
                (teaching::View::Assignment, false),
                (teaching::View::Assignment, true),
            ] {
                a.progress.attempts.get_mut(id).unwrap().challenge = challenge;
                a.set_acl_view(i, view);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                assert!(!a.metrics.help_visible);
                if challenge {
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
                for r in a
                    .metrics
                    .acl_diagram
                    .iter()
                    .chain(a.metrics.acl_router.iter())
                    .chain(a.metrics.editor_viewport.iter())
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{id} {view:?} {width}/{scale}: {r:?} vs {:?}",
                        ctx.content_rect()
                    );
                }
                if view != teaching::View::Introduction || challenge {
                    assert!(a.metrics.editor_viewport.is_some() && a.metrics.check.is_some());
                }
                assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
                assert!(!progress::current_solved(
                    &a.progress.attempts[id],
                    &a.scenarios[i]
                ));
            }
        }
    }
}

#[test]
fn accounts_authored_unit_keeps_full_terminal_tasks_and_separate_guidance_at_all_scales() {
    for id in [
        "lpic1-account-onboarding",
        "lpic1-account-membership",
        "lpic1-account-retirement",
        "lpic1-account-team-handover",
    ] {
        for (width, height, scale) in [(1200., 850., 1.), (720., 560., 1.5), (960., 900., 2.)] {
            let (ctx, mut a) = app();
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            a.scale = scale;
            a.launch(i);
            for (view, challenge) in [
                (teaching::View::Introduction, false),
                (teaching::View::Example, false),
                (teaching::View::Assignment, false),
                (teaching::View::Assignment, true),
            ] {
                a.progress.attempts.get_mut(id).unwrap().challenge = challenge;
                a.set_accounts_view(i, view);
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                assert!(!a.metrics.help_visible && !a.metrics.guide);
                if challenge {
                    assert!(a.metrics.lesson_tabs.is_empty());
                }
                for r in a
                    .metrics
                    .terminal
                    .iter()
                    .chain(a.metrics.output.iter())
                    .chain(a.metrics.check.iter())
                    .chain(a.metrics.accounts_run.iter())
                    .chain(a.metrics.example_action.iter())
                    .chain(a.metrics.assignment_action.iter())
                {
                    assert!(
                        r.left() >= 0. && r.right() <= ctx.content_rect().right() + 1.,
                        "{id} {view:?} {width}/{scale}: {r:?} vs {:?}",
                        ctx.content_rect()
                    );
                }
                if view != teaching::View::Introduction || challenge {
                    assert!(a.metrics.terminal.is_some() && a.metrics.check.is_some());
                }
                assert_eq!(a.progress.attempts[id].state, a.scenarios[i].initial);
                assert!(!progress::current_solved(
                    &a.progress.attempts[id],
                    &a.scenarios[i]
                ));
            }
        }
    }
}

#[path = "tests/unit_manager.rs"]
mod unit_manager;

#[path = "tests/legacy_learning.rs"]
mod legacy_learning;

#[test]
fn scripting_capture_reveals_a_tall_introduction_before_clicking_its_example_action() {
    for scale in [1., 1.5, 2.] {
        let (ctx, mut a) = app();
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "shell-bundle")
            .unwrap();
        assert_eq!(
            i, 23,
            "The retained reusable-unit capture uses this original position"
        );
        a.launch(i);
        a.scale = scale;
        a.smoke = true;
        a.smoke_stage = 900 + 3 * 18 + 2;
        frame(&ctx, &mut a, 1200., 850., vec![]);
        let initial = a.metrics.example_action.unwrap();
        assert!(
            initial.bottom() > a.metrics.main_viewport.unwrap().bottom(),
            "The regression must reproduce an offscreen action at {scale}"
        );
        let mut wheel = false;
        for _ in 0..100 {
            let events = std::mem::take(&mut a.pending_events);
            wheel |= events
                .iter()
                .any(|e| matches!(e, egui::Event::MouseWheel { .. }));
            frame(&ctx, &mut a, 1200., 850., events);
            a.started = Instant::now() - std::time::Duration::from_secs(1);
            a.smoke_tick(&ctx);
            if a.smoke_stage == 900 + 3 * 18 + 3 {
                break;
            }
        }
        assert!(wheel, "The actual outer scroll must reveal the action");
        assert_eq!(a.smoke_stage, 900 + 3 * 18 + 3);
        assert_eq!(a.script_view(i), teaching::View::Example);
        assert_eq!(
            a.progress.attempts["shell-bundle"].state,
            a.scenarios[i].initial
        );
        assert!(!progress::current_solved(
            &a.progress.attempts["shell-bundle"],
            &a.scenarios[i]
        ));
    }
}

mod rpm_packages;

mod rpm_lessons_native;
