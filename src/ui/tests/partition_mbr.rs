//! Real egui controls over registered authored data; not native/novice acceptance.
use super::*;
use dlms_learning_lab::{
    engine::{Goal, State},
    partition_fs_workspace as work,
    script_teaching::View,
};
const IDS: [&str; 3] = [
    "lpic1-mbr-primary-plan",
    "lpic1-mbr-logical-space",
    "lpic1-mbr-storage-handover",
];
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    // This is the live review catalog, not a frozen346fixture. The exact
    // original three-lab unit remains asserted immediately below.
    assert_eq!(a.scenarios.len(), 380);
    assert!(
        a.units
            .iter()
            .any(|u| u.id == "mbr-layouts" && u.labs == IDS)
    );
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    assert_eq!(a.selected, Some(i));
    (ctx, a, i)
}
fn sim(a: &LabApp, i: usize) -> &work::Session {
    let State::PartitionFs { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn view(a: &mut LabApp, i: usize, v: View) {
    let s = a.scenarios[i].clone();
    work::binding::set_view(a.progress.attempts.get_mut(&s.id).unwrap(), &s, v).unwrap();
    a.lesson_scroll_reset = true;
}
fn shape(s: &egui::Shape, text: &mut String) {
    match s {
        egui::Shape::Text(t) => {
            assert!(!t.galley.job.justify, "Stretched lesson text");
            text.push_str(&t.galley.job.text);
            text.push('\n');
        }
        egui::Shape::Vec(v) => {
            for s in v {
                shape(s, text)
            }
        }
        _ => {}
    }
}
fn text(ctx: &egui::Context, a: &mut LabApp) -> String {
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
        shape(&s.shape, &mut t);
    }
    t
}
fn rect(a: &LabApp, key: &str, field: bool) -> (egui::Rect, egui::Rect) {
    let m = &a.metrics.partition_fs;
    if field {
        let r = m.fields.iter().find(|r| r.0 == key).unwrap();
        (r.1, r.3)
    } else {
        let r = m.controls.iter().find(|r| r.0 == key).unwrap();
        (r.1, r.2)
    }
}
fn reveal(
    ctx: &egui::Context,
    a: &mut LabApp,
    key: &str,
    field: bool,
    w: f32,
    h: f32,
) -> egui::Rect {
    for _ in 0..40 {
        frame(ctx, a, w, h, vec![]);
        let (r, clip) = rect(a, key, field);
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip.intersect(root), ctx.pixels_per_point()) {
            return r;
        }
        let pos = egui::pos2(root.left() + 2., root.center().y);
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (root.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("{key} did not reflow/scroll at {w}/{h}")
}
fn click(ctx: &egui::Context, a: &mut LabApp, r: egui::Rect, w: f32, h: f32) {
    frame(ctx, a, w, h, vec![egui::Event::PointerMoved(r.center())]);
    frame(
        ctx,
        a,
        w,
        h,
        [true, false]
            .map(|pressed| egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            })
            .to_vec(),
    );
    for _ in 0..4 {
        frame(ctx, a, w, h, vec![]);
    }
}
fn press(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let r = reveal(ctx, a, key, false, 1400., 1000.);
    click(ctx, a, r, 1400., 1000.);
}
#[test]
fn all_authored_screens_teach_before_action_and_hide_worked_values_in_assignment_and_challenge() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        for v in [View::Introduction, View::Example, View::Assignment] {
            view(&mut a, i, v);
            let t = text(&ctx, &mut a);
            assert!(t.contains("MBR"));
            match v {
                View::Introduction => {
                    assert!(t.contains("Worked example"));
                    assert!(
                        t.contains("A table entry")
                            || t.contains("Free space")
                            || t.contains("Investigate")
                    );
                }
                View::Example => {
                    assert!(t.contains("Fill selected step"));
                    assert!(t.contains("Add partition using draft"));
                    assert!(
                        a.metrics
                            .partition_fs
                            .controls
                            .iter()
                            .any(|r| r.0 == "mbr-role")
                    );
                    assert!(!a.metrics.partition_fs.fields.iter().any(|r| r.0 == "name"));
                }
                View::Assignment => {
                    assert!(t.contains("Requirements") || t.contains("YOUR TASK"));
                    assert!(!t.contains("/dev/vde"), "Worked world exposed by default");
                    assert!(!t.contains("Worked example"));
                    assert!(
                        !t.contains("mbr1"),
                        "Internal key was taught as an MBR name"
                    );
                }
            }
        }
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let t = text(&ctx, &mut a);
        assert!(!t.contains("Worked example"));
        assert!(!t.contains("Fill selected step"));
        assert!(!t.contains("/dev/vde"));
    }
}
#[test]
fn actual_example_fill_create_add_initialize_and_check_are_separate_native_actions() {
    let (ctx, mut a, i) = opened(IDS[0]);
    view(&mut a, i, View::Example);
    let s = a.scenarios[i].clone();
    let Goal::PartitionFs { definition } = &s.goal else {
        panic!()
    };
    let original =
        serde_json::to_value((&sim(&a, i).current, &sim(&a, i).assignment_editor)).unwrap();
    for (n, step) in definition.lesson.worked.iter().enumerate() {
        work::binding::select_example(a.progress.attempts.get_mut(&s.id).unwrap(), &s, n).unwrap();
        draw(&ctx, &mut a);
        let before = sim(&a, i).example.clone();
        press(&ctx, &mut a, "fill-example");
        assert_eq!(sim(&a, i).example, before, "Fill applied an operation");
        let key = match &step.operation {
            work::Operation::Mbr {
                operation: dlms_learning_lab::partition_fs::mbr::Action::Create { .. },
            } => "create-mbr",
            work::Operation::Mbr { .. } => "add",
            work::Operation::Format { .. } => "format",
            _ => panic!("Unexpected legacy operation"),
        };
        press(&ctx, &mut a, key);
        assert!(a.notice.is_none(), "{:?}", a.notice);
    }
    press(&ctx, &mut a, "check");
    assert!(definition.practice_checks(sim(&a, i)).iter().all(|r| r.0));
    assert_eq!(
        serde_json::to_value((&sim(&a, i).current, &sim(&a, i).assignment_editor)).unwrap(),
        original
    );
    assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
}
#[test]
fn container_selection_explains_and_disables_formatting_without_changing_state() {
    let (ctx, mut a, i) = opened(IDS[2]);
    view(&mut a, i, View::Assignment);
    let s = a.scenarios[i].clone();
    let mut draft = sim(&a, i).assignment_editor.drafts["/dev/vda"].clone();
    draft.selected_partition = Some(2);
    draft.mbr_role = Some(dlms_learning_lab::partition_fs::mbr::Kind::Extended);
    work::binding::set_draft(
        a.progress.attempts.get_mut(&s.id).unwrap(),
        &s,
        "/dev/vda",
        draft,
        false,
    )
    .unwrap();
    let t = text(&ctx, &mut a);
    assert!(t.contains("extended container"));
    assert!(t.contains("container cannot hold a filesystem"));
    assert!(t.contains("Container only"));
    let before = serde_json::to_value(&a.progress).unwrap();
    press(&ctx, &mut a, "format");
    assert_eq!(serde_json::to_value(&a.progress).unwrap(), before);
    assert!(!sim(&a, i).checked);
}
#[test]
fn populated_single_line_fields_roles_buttons_and_scrolling_survive_widths_scaling_and_themes() {
    let (ctx, mut a, i) = opened(IDS[2]);
    view(&mut a, i, View::Assignment);
    let s = a.scenarios[i].clone();
    let mut draft = sim(&a, i).assignment_editor.drafts["/dev/vda"].clone();
    draft.start = "12345678901234567890".into();
    draft.end = "18446744073709551615".into();
    work::binding::set_draft(
        a.progress.attempts.get_mut(&s.id).unwrap(),
        &s,
        "/dev/vda",
        draft,
        false,
    )
    .unwrap();
    for light in [false, true] {
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
        for w in [760., 1500.] {
            for scale in [1., 1.15, 1.25, 1.5, 2.] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.lesson_scroll_reset = true;
                frame(&ctx, &mut a, w, 1000., vec![]);
                let start = reveal(&ctx, &mut a, "start", true, w, 1000.);
                let glyph = a
                    .metrics
                    .partition_fs
                    .field_text
                    .iter()
                    .find(|r| r.0 == "start")
                    .unwrap();
                assert!(
                    (glyph.2.center().y - start.center().y).abs() < 1.2,
                    "Uncentered populated field at {w}/{scale}"
                );
                let role = rect(&a, "mbr-role", false).0;
                let button = rect(&a, "create-mbr", false).0;
                assert!((start.height() - role.height()).abs() < 1.2);
                assert!((start.height() - button.height()).abs() < 1.2);
                let root = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                assert!(start.left() >= root.left() - 1. && start.right() <= root.right() + 1.);
                let _ = reveal(&ctx, &mut a, "check", false, w, 1000.);
            }
        }
    }
}

#[test]
fn keyboard_editing_and_tab_keep_drafts_separate_from_applied_disk_state() {
    let (ctx, mut a, i) = opened(IDS[0]);
    view(&mut a, i, View::Assignment);
    draw(&ctx, &mut a);
    let before = sim(&a, i).current.clone();
    let r = reveal(&ctx, &mut a, "start", true, 1400., 1000.);
    click(&ctx, &mut a, r, 1400., 1000.);
    frame(
        &ctx,
        &mut a,
        1400.,
        1000.,
        vec![egui::Event::Text("2048".into())],
    );
    frame(&ctx, &mut a, 1400., 1000., vec![]);
    assert_eq!(
        sim(&a, i).assignment_editor.drafts["/dev/vda"].start,
        "2048"
    );
    assert_eq!(sim(&a, i).current, before);
    let end = a
        .metrics
        .partition_fs
        .fields
        .iter()
        .find(|r| r.0 == "end")
        .unwrap()
        .2;
    frame(
        &ctx,
        &mut a,
        1400.,
        1000.,
        [true, false]
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Default::default(),
            })
            .to_vec(),
    );
    assert_eq!(ctx.memory(|m| m.focused()), Some(end));
    frame(
        &ctx,
        &mut a,
        1400.,
        1000.,
        vec![egui::Event::Text("18431".into())],
    );
    frame(&ctx, &mut a, 1400., 1000., vec![]);
    assert_eq!(sim(&a, i).assignment_editor.drafts["/dev/vda"].end, "18431");
    assert_eq!(sim(&a, i).current, before);
    assert!(!sim(&a, i).checked);
}

#[test]
fn expanded_mbr_reader_can_be_reached_below_the_native_configuration_controls() {
    let (ctx, mut a, i) = opened(IDS[0]);
    a.smoke = true; // Capture actual rendered source text, without auto-running smoke.
    view(&mut a, i, View::Assignment);
    press(&ctx, &mut a, "inspect");
    let mut visible = false;
    for _ in 0..40 {
        frame(&ctx, &mut a, 1400., 1000., vec![]);
        let viewport = a
            .metrics
            .partition_fs
            .files_viewport
            .expect("Expanded reader");
        let root = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(viewport, root, ctx.pixels_per_point()) {
            visible = true;
            break;
        }
        frame(
            &ctx,
            &mut a,
            1400.,
            1000.,
            vec![
                egui::Event::PointerMoved(egui::pos2(root.left() + 1., root.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(
                        0.,
                        (root.center().y - viewport.center().y).clamp(-400., 400.),
                    ),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(&ctx, &mut a, 1400., 1000., vec![]);
        }
    }
    assert!(
        visible,
        "Reader must be revealed through the outer scroller first"
    );
    let State::PartitionFs { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    let expected = simulation
        .origin
        .initial
        .disks
        .iter()
        .find_map(|(path, d)| {
            d.partitions().find_map(|p| {
                if let dlms_learning_lab::partition_fs::Content::Ext4 { files, .. } = &p.content {
                    files
                        .iter()
                        .next()
                        .map(|(name, f)| format!("{path}{} · {name}: {}", p.number, f.text))
                } else {
                    None
                }
            })
        })
        .unwrap();
    assert!(
        a.metrics
            .partition_fs
            .text_rects
            .iter()
            .any(|(text, rect, clip)| text == &expected
                && smoke::smoke_control_visible(*rect, *clip, ctx.pixels_per_point())),
        "Actual retained source record must be visible"
    );
    assert_eq!(simulation.current, simulation.origin.initial);
}

#[test]
fn optional_mbr_reference_describes_the_actual_profile_and_keeps_worked_answers_closed() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        view(&mut a, i, View::Assignment);
        press(&ctx, &mut a, "help");
        press(&ctx, &mut a, "references");
        let t = text(&ctx, &mut a);
        assert!(t.contains("Virtual MBR preparation only"));
        assert!(t.contains("raw EBR chains"));
        assert!(!t.contains("The pure subset has no deletion, resize, MBR/logical partitions"));
        assert!(
            !t.contains("/dev/vde"),
            "Opening references must not reveal the worked assignment"
        );
        assert_eq!(sim(&a, i).current, sim(&a, i).origin.initial);
    }
}
