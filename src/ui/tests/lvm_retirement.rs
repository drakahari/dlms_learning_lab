//! Actual egui layouts/inputs; separate from native or novice acceptance.
use super::*;
use dlms_learning_lab::lvm_workspace as work;
const IDS: [&str; 2] = smoke::lvm_provisioning::RETIRE_IDS;
fn opened(id: &str) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
    a.launch(i);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn view(a: &mut LabApp, i: usize, v: teaching::View) {
    let engine::Goal::Lvm { definition } = a.scenarios[i].goal.clone() else {
        panic!()
    };
    let engine::State::Lvm { simulation } = &mut a
        .progress
        .attempts
        .get_mut(&a.scenarios[i].id)
        .unwrap()
        .state
    else {
        panic!()
    };
    simulation.set_view(&definition, v).unwrap();
    a.lesson_scroll_reset = true;
}
fn sim(a: &LabApp, i: usize) -> &work::Session {
    let engine::State::Lvm { simulation } = &a.progress.attempts[&a.scenarios[i].id].state else {
        panic!()
    };
    simulation
}
fn text(shape: &egui::Shape, out: &mut String) {
    match shape {
        egui::Shape::Text(t) => {
            assert!(!t.galley.job.justify, "Stretched lesson text");
            out.push_str(&t.galley.job.text);
            out.push('\n');
        }
        egui::Shape::Vec(v) => {
            for s in v {
                text(s, out)
            }
        }
        _ => {}
    }
}
fn rendered(ctx: &egui::Context, a: &mut LabApp) -> String {
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
    let mut s = String::new();
    for shape in out.shapes {
        text(&shape.shape, &mut s);
    }
    s
}
fn target(ctx: &egui::Context, a: &mut LabApp, key: &str, w: f32, h: f32) -> egui::Rect {
    let mut stable = None;
    for _ in 0..40 {
        frame(ctx, a, w, h, vec![]);
        let r = smoke::lvm_provisioning::control(a, key).unwrap_or_else(|| panic!("Missing {key}"));
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        if smoke::smoke_control_visible(r, clip, ctx.pixels_per_point()) {
            if stable == Some((r, clip)) {
                return r;
            }
            stable = Some((r, clip));
            frame(ctx, a, w, h, vec![egui::Event::PointerMoved(r.center())]);
            continue;
        }
        stable = None;
        frame(
            ctx,
            a,
            w,
            h,
            vec![
                egui::Event::PointerMoved(egui::pos2(clip.left() + 1., clip.center().y)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., (clip.center().y - r.center().y).clamp(-500., 500.)),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..8 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("{key} failed to reflow/scroll at {w}x{h}")
}
#[test]
fn retired_lvm_all_three_views_teach_before_practice_and_hide_independent_solution_commands() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        for v in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            view(&mut a, i, v);
            let s = rendered(&ctx, &mut a);
            assert!(a.preview_view_ready(i, v));
            assert!(a.preview_action_rect().is_some());
            match v {
                teaching::View::Introduction => {
                    assert!(s.contains("Worked example"));
                    assert!(s.contains("Try this example"));
                    assert!(s.contains("filesystem"));
                    if id == IDS[0] {
                        assert!(s.contains("extents"));
                        assert!(s.contains("Inspect before"));
                    } else {
                        assert!(s.contains("PV label"));
                        assert!(s.contains("secure erasure"));
                    }
                }
                teaching::View::Example => {
                    assert!(s.contains("Insert selected command"));
                    assert!(s.contains("Check separate example"));
                    assert!(s.contains("Observe:"));
                    assert!(s.contains("Supplied device facts"));
                }
                teaching::View::Assignment => {
                    assert!(s.contains("Requirements"));
                    assert!(s.contains("Check work"));
                    assert!(s.contains("Supplied device facts"));
                    let engine::Goal::Lvm { definition } = &a.scenarios[i].goal else {
                        panic!()
                    };
                    assert!(!s.contains("Worked example"));
                    for c in &definition.lesson.worked {
                        if c.text.starts_with("lvremove")
                            || c.text.starts_with("vgremove")
                            || c.text.starts_with("pvremove")
                        {
                            assert!(!s.contains(&c.text), "Assignment exposed a worked command");
                        }
                    }
                }
            }
        }
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let s = rendered(&ctx, &mut a);
        assert!(!s.contains("Worked example"));
        assert!(!s.contains("Try this example"));
        assert!(s.contains("Requirements"));
    }
}
#[test]
fn retired_lvm_command_height_and_visible_controls_survive_widths_scales_and_both_theme_types() {
    let (ctx, mut a, i) = opened(IDS[1]);
    view(&mut a, i, teaching::View::Assignment);
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
        for width in [760., 1600.] {
            for scale in [1., 1.15, 1.25, 1.5, 2.] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
                a.lesson_scroll_reset = true;
                let r = target(&ctx, &mut a, "command", width, 900.);
                let run = smoke::lvm_provisioning::control(&a, "run").unwrap();
                assert!(
                    (r.height() - run.height()).abs() < 1.1,
                    "Control heights {width}/{scale}"
                );
                let clip = a
                    .metrics
                    .main_viewport
                    .unwrap()
                    .intersect(ctx.content_rect());
                assert!(
                    r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                    "Command clipped {width}/{scale}"
                );
                let check = target(&ctx, &mut a, "check", width, 900.);
                assert!(check.width() > 0.);
            }
        }
    }
}
#[test]
fn retired_lvm_real_keyboard_edit_enter_retry_and_tab_keep_grading_and_focus_honest() {
    let (ctx, mut a, i) = opened(IDS[0]);
    view(&mut a, i, teaching::View::Assignment);
    let edit = |a: &mut LabApp, value: &str| {
        let r = target(&ctx, a, "command", 1200., 950.);
        click(&ctx, a, r);
        let id = a.metrics.lvm.fields[0].2;
        assert_eq!(ctx.memory(|m| m.focused()), Some(id));
        let modifiers = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
        frame(
            &ctx,
            a,
            1200.,
            950.,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                },
                egui::Event::Paste(value.into()),
            ],
        );
    };
    let engine::Goal::Lvm { definition } = a.scenarios[i].goal.clone() else {
        panic!()
    };
    let engine::State::Lvm { simulation } = &mut a.progress.attempts.get_mut(IDS[0]).unwrap().state
    else {
        panic!()
    };
    simulation
        .execute(&definition, "lvremove -y records/cache", false)
        .unwrap();
    let before = sim(&a, i).current.clone();
    edit(&mut a, "lvremove -y records/scratch");
    assert_eq!(sim(&a, i).current, before);
    assert_eq!(sim(&a, i).command, "lvremove -y records/scratch");
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    edit(&mut a, "lvcreate -l 1 -n scratch records");
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(!progress::current_solved(
        &a.progress.attempts[IDS[0]],
        &a.scenarios[i]
    ));
    assert!(!sim(&a, i).checked);
    edit(&mut a, "lvs");
    let field = a.metrics.lvm.fields[0].2;
    frame(
        &ctx,
        &mut a,
        1200.,
        950.,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_ne!(ctx.memory(|m| m.focused()), Some(field));
    assert!(ctx.memory(|m| m.focused()).is_some());
}
