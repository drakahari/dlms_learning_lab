//! Actual egui controls for the two authored map lessons; not a novice study.
use super::*;
use dlms_learning_lab::engine::Goal;
use dlms_learning_lab::raid_assembly_workspace as w;
const ID: &str = "lpic2-raid1-assembly-policy";
fn text(ctx: &egui::Context, a: &mut LabApp, width: f32, events: Vec<egui::Event>) -> String {
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width / a.scale, 9000. / a.scale),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    fn collect(shape: &egui::Shape, s: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                s.push_str(&t.galley.job.text);
                s.push('\n');
            }
            egui::Shape::Vec(v) => {
                for t in v {
                    collect(t, s)
                }
            }
            _ => {}
        }
    }
    let mut s = String::new();
    for shape in output.shapes {
        collect(&shape.shape, &mut s);
    }
    s
}
fn setup() -> (egui::Context, LabApp) {
    let (ctx, mut a, _) = debug_recovery::opened(ID);
    a.smoke = false;
    text(&ctx, &mut a, 1500., vec![]);
    (ctx, a)
}
fn sim(a: &LabApp) -> &w::Session {
    let State::RaidAssembly { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn sim_mut(a: &mut LabApp) -> &mut w::Session {
    let State::RaidAssembly { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation
}
fn control(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .raid_assembly
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .1
}
fn field(a: &LabApp, key: &str) -> (egui::Rect, egui::Id, egui::Rect) {
    let r = a
        .metrics
        .raid_assembly
        .fields
        .iter()
        .find(|r| r.0 == key)
        .unwrap();
    (r.1, r.2, r.3)
}
fn click_control(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let rect = control(a, key);
    for pressed in [true, false] {
        text(
            ctx,
            a,
            1500.,
            vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    text(ctx, a, 1500., vec![]);
}
fn view(ctx: &egui::Context, a: &mut LabApp, v: teaching::View) {
    sim_mut(a).view = v;
    text(ctx, a, 1500., vec![]);
}
fn keys(k: egui::Key, modifiers: egui::Modifiers) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key: k,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
        })
        .collect()
}
fn replace(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    let id = field(a, key).1;
    ctx.memory_mut(|m| m.request_focus(id));
    text(ctx, a, 1500., vec![]);
    let mut e = keys(
        egui::Key::A,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
    );
    e.push(egui::Event::Paste(value.into()));
    text(ctx, a, 1500., e);
    text(ctx, a, 1500., vec![]);
}
#[test]
fn all_views_teach_before_task_and_keep_independent_uuid_hidden_by_default() {
    let (ctx, mut a) = setup();
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Assemble existing members"));
    assert!(t.contains("abcd1000"));
    view(&ctx, &mut a, teaching::View::Example);
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Insert command"));
    assert!(t.contains("Actual example terminal"));
    view(&ctx, &mut a, teaching::View::Assignment);
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Configuration · /etc/mdadm.conf"));
    assert!(!t.contains("abcd1000"));
    assert!(!t.contains("73112222"));
    assert!(!t.contains("Worked approach"));
    assert!(t.contains("Not checked"));
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!t.contains("Example file"));
    assert!(!t.contains("abcd1000"));
}
#[test]
fn insert_and_fill_do_not_run_or_save_and_practice_cannot_earn_credit() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Example);
    let before = sim(&a).clone();
    click_control(&ctx, &mut a, "insert");
    assert!(sim(&a).practice.command.starts_with("mdadm --examine"));
    assert_eq!(sim(&a).practice.runtime, before.practice.runtime);
    assert!(sim(&a).practice.history.is_empty());
    assert_eq!(sim(&a).assignment, before.assignment);
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap().clone();
    let Goal::RaidAssembly { definition: d } = &source.goal else {
        panic!()
    };
    let n = d
        .example_steps
        .iter()
        .position(|s| matches!(s.operation, w::Operation::Save { .. }))
        .unwrap();
    sim_mut(&mut a).example_step = n;
    text(&ctx, &mut a, 1500., vec![]);
    click_control(&ctx, &mut a, "insert");
    assert_ne!(sim(&a).practice.draft, before.practice.draft);
    assert_eq!(sim(&a).practice.runtime, before.practice.runtime);
    click_control(&ctx, &mut a, "save");
    assert_ne!(sim(&a).practice.runtime, before.practice.runtime);
    assert_eq!(sim(&a).assignment, before.assignment);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
}
#[test]
fn real_editor_save_run_check_and_edit_invalidation_work() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap().clone();
    let Goal::RaidAssembly { definition: d } = &source.goal else {
        panic!()
    };
    let w::Operation::Save { text: correct } = &d
        .reference
        .iter()
        .find(|s| matches!(s.operation, w::Operation::Save { .. }))
        .unwrap()
        .operation
    else {
        panic!()
    };
    replace(&ctx, &mut a, "editor", &correct.0);
    assert_ne!(sim(&a).assignment.draft, sim(&a).assignment.runtime.saved);
    click_control(&ctx, &mut a, "save");
    click_control(&ctx, &mut a, "check");
    assert!(a.metrics.raid_assembly.result.unwrap().1 > 0);
    replace(&ctx, &mut a, "command", "mdadm --assemble --scan /dev/md10");
    click_control(&ctx, &mut a, "run");
    assert_eq!(
        sim(&a).assignment.transcript.last().unwrap().result.status,
        0
    );
    click_control(&ctx, &mut a, "check");
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    replace(&ctx, &mut a, "editor", "# unfinished draft\n");
    assert!(!sim(&a).assignment.checked);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    assert_eq!(sim(&a).assignment.runtime.saved.0, correct.0);
}
#[test]
fn feedback_persists_until_editing_and_bad_pastes_do_not_dispatch() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    replace(&ctx, &mut a, "command", "ls");
    let before = sim(&a).clone();
    click_control(&ctx, &mut a, "run");
    assert_eq!(sim(&a), &before);
    for _ in 0..3 {
        assert!(text(&ctx, &mut a, 1500., vec![]).contains("Unsupported command or path"));
    }
    replace(&ctx, &mut a, "command", "cat /proc/mdstat");
    assert!(!text(&ctx, &mut a, 1500., vec![]).contains("Unsupported command or path"));
    click_control(&ctx, &mut a, "run");
    assert_eq!(
        sim(&a).assignment.transcript.last().unwrap().result.status,
        0
    );
    let id = field(&a, "command").1;
    ctx.memory_mut(|m| m.request_focus(id));
    text(&ctx, &mut a, 1500., vec![]);
    let before = sim(&a).clone();
    text(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Paste(
            "cat /proc/mdstat\nmdadm --stop /dev/md11".into(),
        )],
    );
    assert_eq!(sim(&a), &before);
    let id = field(&a, "editor").1;
    ctx.memory_mut(|m| m.request_focus(id));
    text(&ctx, &mut a, 1500., vec![]);
    let before = sim(&a).clone();
    text(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Paste("\0bad".into())],
    );
    assert_eq!(sim(&a), &before);
}
#[test]
fn five_supported_scales_two_widths_light_dark_and_tab_keep_controls_accessible() {
    for theme_id in ["dlms-dark", "dlms-light"] {
        for scale in [1., 1.25, 1.5, 1.75, 2.] {
            let (ctx, mut a) = setup();
            a.scale = scale;
            a.theme_id = theme_id.into();
            theme::apply(&ctx, theme_id);
            sim_mut(&mut a).view = teaching::View::Assignment;
            for width in [1500., 900.] {
                text(&ctx, &mut a, width, vec![]);
                text(&ctx, &mut a, width, vec![]);
                for key in ["editor", "command"] {
                    let (r, _, clip) = field(&a, key);
                    assert!(r.is_finite() && r.width() > 0.);
                    assert!(
                        r.intersect(clip).width() > r.width() * 0.95,
                        "{theme_id}/{scale}/{width}/{key}"
                    );
                }
                for key in ["save", "run", "check"] {
                    let r = control(&a, key);
                    assert!(r.is_finite() && r.width() > 0.);
                }
            }
            let id = field(&a, "editor").1;
            ctx.memory_mut(|m| m.request_focus(id));
            text(&ctx, &mut a, 900., vec![]);
            text(
                &ctx,
                &mut a,
                900.,
                keys(egui::Key::Tab, egui::Modifiers::NONE),
            );
            assert_ne!(ctx.memory(|m| m.focused()), Some(id));
        }
    }
}

#[test]
fn empty_output_shrinks_to_instructions_then_reserves_space_for_actual_results() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    text(&ctx, &mut a, 1500., vec![]);
    let visible = text(&ctx, &mut a, 1500., vec![]);
    assert!(
        visible.contains("Inspect original members with mdadm --examine"),
        "Empty terminal guidance must be visible, not clipped to a zero-height scroll viewport"
    );
    let empty = a.metrics.output.unwrap().height();
    assert!(
        empty < 200.,
        "Empty output must not leave a large gap before input: {empty}"
    );
    replace(&ctx, &mut a, "command", "cat /proc/mdstat");
    click_control(&ctx, &mut a, "run");
    text(&ctx, &mut a, 1500., vec![]);
    let populated = a.metrics.output.unwrap().height();
    assert!(
        populated > empty + 80.,
        "Actual results need useful scrolling space: {empty}/{populated}"
    );
    click_control(&ctx, &mut a, "data");
    assert!(text(&ctx, &mut a, 1500., vec![]).contains("Operations audit dataset"));
}
