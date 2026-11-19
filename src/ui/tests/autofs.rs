//! Actual egui controls for the two authored map lessons; not a novice study.
use super::*;
use dlms_learning_lab::engine::Goal;
use dlms_learning_lab::{autofs::runtime::Name, autofs_workspace as w};
const ID: &str = "lpic2-autofs-indirect-repair";
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
    let State::Autofs { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn sim_mut(a: &mut LabApp) -> &mut w::Session {
    let State::Autofs { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state else {
        panic!()
    };
    simulation
}
fn control(a: &LabApp, key: &str) -> egui::Rect {
    a.metrics
        .autofs
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .1
}
fn field(a: &LabApp, key: &str) -> (egui::Rect, egui::Id, egui::Rect) {
    let r = a.metrics.autofs.fields.iter().find(|r| r.0 == key).unwrap();
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
fn three_views_teach_before_independent_work_without_exposing_default_solution() {
    let (ctx, mut a) = setup();
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Read the master and the map together"));
    assert!(t.contains("/training/team"));
    assert!(t.contains(":/dev/demo-indirect/reports"));
    assert!(!t.contains("Configuration · simulated files"));
    view(&ctx, &mut a, teaching::View::Example);
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Fill selected example"));
    assert!(t.contains("Actual example terminal"));
    view(&ctx, &mut a, teaching::View::Assignment);
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Configuration · simulated files"));
    assert!(t.contains("Not checked"));
    assert!(!t.contains("/dev/demo-indirect"));
    assert!(!t.contains("Fill selected example"));
    assert!(!t.contains("Worked approach"));
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!t.contains("/training/team"));
    assert!(!t.contains("Example file"));
}
#[test]
fn fill_requires_explicit_save_and_never_changes_assignment_or_awards_credit() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Example);
    let before = sim(&a).clone();
    click_control(&ctx, &mut a, "insert");
    assert_ne!(sim(&a).practice.drafts, before.practice.drafts);
    assert_eq!(sim(&a).practice.runtime, before.practice.runtime);
    assert_eq!(sim(&a).assignment, before.assignment);
    click_control(&ctx, &mut a, "save");
    assert_ne!(sim(&a).practice.runtime, before.practice.runtime);
    assert_eq!(sim(&a).assignment, before.assignment);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        a.scenarios.iter().find(|s| s.id == ID).unwrap()
    ));
}
#[test]
fn actual_editor_terminal_idle_check_and_retry_preserve_meaningful_state_grading() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    sim_mut(&mut a).assignment.selected = Name("/etc/auto.teams".into());
    text(&ctx, &mut a, 1500., vec![]);
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap().clone();
    let Goal::Autofs { definition: d } = &source.goal else {
        panic!()
    };
    let w::Operation::Save { text: correct, .. } = &d.reference[0].operation else {
        panic!()
    };
    replace(&ctx, &mut a, "editor", &correct.0);
    assert_ne!(sim(&a).assignment.drafts, sim(&a).assignment.runtime.saved);
    click_control(&ctx, &mut a, "save");
    click_control(&ctx, &mut a, "check");
    assert!(a.metrics.autofs.result.unwrap().1 > 0);
    click_control(&ctx, &mut a, "idle");
    for target in &d.assignment.targets {
        let cmd = format!("cat {}{}", target.point.0, target.file.0);
        replace(&ctx, &mut a, "command", &cmd);
        assert_eq!(sim(&a).assignment.command, cmd);
        click_control(&ctx, &mut a, "run");
        assert_eq!(
            sim(&a).assignment.transcript.last().unwrap().result.status,
            0
        );
    }
    click_control(&ctx, &mut a, "check");
    assert!(progress::current_solved(&a.progress.attempts[ID], &source));
    replace(&ctx, &mut a, "editor", "# unfinished change\n");
    assert!(!sim(&a).assignment.checked);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &source));
    assert_eq!(sim(&a).assignment.runtime.saved.maps[0].text.0, correct.0);
}
#[test]
fn malformed_configuration_feedback_and_bad_paste_do_not_dispatch_stale_actions() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    sim_mut(&mut a).assignment.selected = Name("/etc/auto.teams".into());
    text(&ctx, &mut a, 1500., vec![]);
    replace(
        &ctx,
        &mut a,
        "editor",
        "reports -fstype=ext4 /dev/ledger/reports\n",
    );
    click_control(&ctx, &mut a, "save");
    click_control(&ctx, &mut a, "idle");
    replace(&ctx, &mut a, "command", "cat /srv/teams/reports/keep.txt");
    click_control(&ctx, &mut a, "run");
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert!(t.contains("Map syntax error"));
    assert!(t.contains("colon"));
    let id = field(&a, "command").1;
    ctx.memory_mut(|m| m.request_focus(id));
    text(&ctx, &mut a, 1500., vec![]);
    let before = sim(&a).clone();
    text(
        &ctx,
        &mut a,
        1500.,
        vec![egui::Event::Paste(
            "cat /etc/auto.master\nsystemctl reload autofs".into(),
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
fn five_text_scales_light_dark_and_narrow_fields_use_native_geometry_and_tab_focus() {
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
                        "{theme_id}/{scale}/{width}/{key}: {r:?}/{clip:?}"
                    );
                }
                for key in ["save", "idle", "run", "check"] {
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
fn repeated_policy_diagnostic_is_shown_once_without_removing_any_objective_result() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    let source = a.scenarios.iter().find(|s| s.id == ID).unwrap().clone();
    let Goal::Autofs { definition: d } = &source.goal else {
        panic!()
    };
    sim_mut(&mut a).assignment.selected = Name("/etc/auto.teams".into());
    sim_mut(&mut a)
        .edit(
            d,
            "reports -fstype=ext4 /dev/ledger/reports\n".into(),
            false,
        )
        .unwrap();
    sim_mut(&mut a).save(d, false).unwrap();
    sim_mut(&mut a).run(d, w::CHECK, false).unwrap();
    let before = d.checks(sim(&a));
    let t = text(&ctx, &mut a, 1500., vec![]);
    assert_eq!(t.matches("Put a colon before").count(), 1);
    assert_eq!(a.metrics.autofs.result.unwrap().0, before.len());
    assert_eq!(
        a.metrics.autofs.result.unwrap().1,
        before.iter().filter(|r| !r.0).count()
    );
    assert!(t.contains("All objective results"));
    assert_eq!(d.checks(sim(&a)), before);
}

#[test]
fn unsupported_command_feedback_survives_frames_and_clears_on_valid_edit_and_retry() {
    let (ctx, mut a) = setup();
    view(&ctx, &mut a, teaching::View::Assignment);
    replace(&ctx, &mut a, "command", "ls");
    let before = sim(&a).clone();
    click_control(&ctx, &mut a, "run");
    assert_eq!(
        sim(&a),
        &before,
        "An unsupported command must not execute or create simulated effects"
    );
    for _ in 0..3 {
        let rendered = text(&ctx, &mut a, 1500., vec![]);
        assert!(
            rendered.contains("Unsupported command or path"),
            "The learner must have time to read why nothing ran"
        );
    }
    click_control(&ctx, &mut a, "example");
    assert!(!text(&ctx, &mut a, 1500., vec![]).contains("Unsupported command or path"));
    click_control(&ctx, &mut a, "assignment");
    assert!(!text(&ctx, &mut a, 1500., vec![]).contains("Unsupported command or path"));
    click_control(&ctx, &mut a, "run");
    assert!(text(&ctx, &mut a, 1500., vec![]).contains("Unsupported command or path"));
    replace(&ctx, &mut a, "command", "cat /etc/auto.master");
    let rendered = text(&ctx, &mut a, 1500., vec![]);
    assert!(
        !rendered.contains("Unsupported command or path"),
        "Edited input must clear stale action feedback"
    );
    click_control(&ctx, &mut a, "run");
    assert_eq!(
        sim(&a).assignment.transcript.last().unwrap().result.status,
        0
    );
    assert!(!text(&ctx, &mut a, 1500., vec![]).contains("Unsupported command or path"));
}
