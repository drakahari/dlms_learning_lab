//! Contents teaching through real native widgets; no novice-acceptance claim.
use super::*;
use dlms_learning_lab::{debian_packages_terminal::Session, script_teaching::View};
const ID: &str = "lpic1-package-file-provider";
fn opened(view: View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    a.set_debian_packages_view(i, view);
    draw(&ctx, &mut a);
    (ctx, a, i)
}
fn terminal(a: &LabApp, practice: bool) -> &Session {
    let w = a.progress.attempts[ID].workspace.as_ref().unwrap();
    if practice {
        w.debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
    } else {
        w.debian_packages_terminal.as_ref().unwrap()
    }
}
pub(super) fn find(
    ctx: &egui::Context,
    a: &mut LabApp,
    w: f32,
    h: f32,
    f: fn(&Metrics) -> Option<egui::Rect>,
) -> egui::Rect {
    for _ in 0..30 {
        let r = f(&a.metrics).expect("actual package control exists");
        let clip = a
            .metrics
            .main_viewport
            .unwrap()
            .intersect(ctx.content_rect());
        let t = 1. / ctx.pixels_per_point();
        if clip.contains(r.center()) && clip.expand(t).contains_rect(r) {
            return r;
        }
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
                    delta: egui::vec2(0., dy),
                    modifiers: Default::default(),
                },
            ],
        );
        for _ in 0..4 {
            frame(ctx, a, w, h, vec![]);
        }
    }
    panic!("actual package control must become visible");
}
pub(super) fn point_click(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, r: egui::Rect) {
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
    for _ in 0..3 {
        frame(ctx, a, w, h, vec![]);
    }
}
pub(super) fn command(ctx: &egui::Context, a: &mut LabApp, w: f32, h: f32, text: &str) {
    a.focus_input = false;
    let r = find(ctx, a, w, h, |m| m.package_command.map(|(r, _, _)| r));
    point_click(ctx, a, w, h, r);
    let id = a.metrics.package_command.unwrap().1;
    assert!(ctx.memory(|m| m.has_focus(id)));
    frame(
        ctx,
        a,
        w,
        h,
        vec![acl_ctrl(egui::Key::A), egui::Event::Text(text.into())],
    );
    for _ in 0..3 {
        frame(ctx, a, w, h, vec![]);
    }
}
pub(super) fn run(ctx: &egui::Context, a: &mut LabApp, text: &str) {
    command(ctx, a, 1200., 850., text);
    frame(ctx, a, 1200., 850., vec![key(egui::Key::Enter, false)]);
    draw(ctx, a);
}
pub(super) fn check_work(ctx: &egui::Context, a: &mut LabApp) {
    let r = find(ctx, a, 1200., 850., |m| m.check);
    point_click(ctx, a, 1200., 850., r);
}
pub(super) fn painted(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn append(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for x in v {
                    append(x, out)
                }
            }
            _ => {}
        }
    }
    let o = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 2400.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut text = String::new();
    for s in o.shapes {
        append(&s.shape, &mut text);
    }
    text
}
#[test]
fn contents_three_views_teach_file_lookup_without_revealing_assignment_provider() {
    let (ctx, mut a, i) = opened(View::Introduction);
    let text = painted(&ctx, &mut a);
    assert!(text.contains("WHAT YOU’LL LEARN"));
    assert!(
        !text.contains("1. 1."),
        "Section numbering belongs to the shared renderer"
    );
    assert!(text.contains("apt-file"));
    assert!(text.contains("gallery-tools"));
    assert!(!text.contains("records-tools"));
    a.set_debian_packages_view(i, View::Example);
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Refresh and compare file matches"));
    assert!(text.contains("Narrow the path and install the provider"));
    assert!(a.metrics.package_command.is_some());
    a.set_debian_packages_view(i, View::Assignment);
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Requirements"));
    assert!(text.contains("/usr/bin/ledger-check"));
    assert!(!text.contains("records-tools"));
    assert!(!text.contains("gallery-tools"));
    assert!(terminal(&a, false).command.is_empty());
    check_work(&ctx, &mut a);
    let feedback = painted(&ctx, &mut a);
    assert!(
        !feedback.contains("records-tools"),
        "Early check must explain investigation, not give away its provider"
    );
    assert!(feedback.contains("Search the complete path"));
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    a.set_debian_packages_view(i, View::Introduction);
    let text = painted(&ctx, &mut a);
    assert!(a.metrics.lesson_tabs.is_empty());
    assert!(!text.contains("gallery-tools"));
    assert!(!text.contains("records-tools"));
}
#[test]
fn contents_keyboard_run_queries_install_wrong_provider_retry_and_current_credit() {
    let (ctx, mut a, _) = opened(View::Assignment);
    let original = a.progress.attempts[ID].state.clone();
    run(&ctx, &mut a, "apt-file search ledger-check");
    assert_eq!(
        terminal(&a, false)
            .entries
            .last()
            .unwrap()
            .stdout
            .lines()
            .count(),
        2
    );
    assert_eq!(a.progress.attempts[ID].state, original);
    run(&ctx, &mut a, "apt-get --error-on=any update");
    run(&ctx, &mut a, "apt-file -F search usr/bin/ledger-check");
    assert_eq!(
        terminal(&a, false).entries.last().unwrap().stdout,
        "records-tools: usr/bin/ledger-check\n"
    );
    check_work(&ctx, &mut a);
    assert!(!a.metrics.completion);
    run(&ctx, &mut a, "apt-get -y install records-tools");
    check_work(&ctx, &mut a);
    assert!(a.metrics.completion);
    run(&ctx, &mut a, "apt-get -y install records-private");
    check_work(&ctx, &mut a);
    assert!(!a.metrics.completion);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[a.selected.unwrap()]
    ));
    run(&ctx, &mut a, "dpkg -r records-private");
    check_work(&ctx, &mut a);
    assert!(a.metrics.completion);
    command(&ctx, &mut a, 1200., 850., "apt-file search 'unfinished");
    assert!(progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[a.selected.unwrap()]
    ));
    assert_eq!(terminal(&a, false).command, "apt-file search 'unfinished");
}
#[test]
fn contents_example_is_separate_insert_is_inert_and_saved_drafts_stay_separate() {
    let (ctx, mut a, i) = opened(View::Example);
    let original = a.progress.attempts[ID].state.clone();
    let before = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .as_ref()
        .unwrap()
        .simulation
        .clone();
    let r = find(&ctx, &mut a, 1200., 850., |m| m.command_insert);
    point_click(&ctx, &mut a, 1200., 850., r);
    assert_eq!(terminal(&a, true).command, "apt-get --error-on=any update");
    assert!(terminal(&a, true).entries.is_empty());
    assert_eq!(
        a.progress.attempts[ID]
            .workspace
            .as_ref()
            .unwrap()
            .debian_packages_learning
            .as_ref()
            .unwrap()
            .simulation,
        before
    );
    for c in [
        "apt-get --error-on=any update",
        "apt-file -F search usr/bin/exhibit-audit",
        "apt-get -y install gallery-tools",
    ] {
        run(&ctx, &mut a, c);
    }
    check_work(&ctx, &mut a);
    let l = a.progress.attempts[ID]
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .as_ref()
        .unwrap();
    assert!(
        l.checks(a.scenarios[i].debian_packages_teaching.as_ref().unwrap())
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(a.progress.attempts[ID].state, original);
    assert!(!progress::current_solved(
        &a.progress.attempts[ID],
        &a.scenarios[i]
    ));
    command(
        &ctx,
        &mut a,
        1200.,
        850.,
        "apt-file list 'unfinished example",
    );
    a.set_debian_packages_view(i, View::Assignment);
    draw(&ctx, &mut a);
    command(
        &ctx,
        &mut a,
        1200.,
        850.,
        "dpkg-query -W 'unfinished assignment",
    );
    a.set_debian_packages_view(i, View::Example);
    draw(&ctx, &mut a);
    assert_eq!(
        terminal(&a, true).command,
        "apt-file list 'unfinished example"
    );
    assert_eq!(
        terminal(&a, false).command,
        "dpkg-query -W 'unfinished assignment"
    );
}
#[test]
fn contents_shared_controls_fit_supported_scales_and_light_dark_reflow() {
    let (ctx, mut a, i) = opened(View::Assignment);
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
        for scale in [1., 1.15, 1.25, 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [760., 1600.] {
                for view in [View::Introduction, View::Example, View::Assignment] {
                    a.set_debian_packages_view(i, view);
                    for _ in 0..3 {
                        frame(&ctx, &mut a, width, 850., vec![]);
                    }
                    if view != View::Introduction {
                        let input = a.metrics.package_command.unwrap().0;
                        assert!(
                            input.width() > 0.
                                && input.left() >= 0.
                                && input.right() <= ctx.content_rect().right() + 1.,
                            "{scale} {width} {view:?} {input:?}"
                        );
                        assert!(a.metrics.package_run.unwrap().0.is_finite());
                        assert!(a.metrics.check.unwrap().is_finite());
                    }
                }
            }
        }
    }
}

#[test]
fn explicit_debian_submission_reveals_its_new_output_and_keeps_manual_scrollback() {
    let (ctx, mut a, _) = opened(View::Assignment);
    for _ in 0..8 {
        run(&ctx, &mut a, "apt-file search ledger-check");
    }
    let _ = find(&ctx, &mut a, 1200., 850., |m| m.output);
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    let (offset, max, _) = a.metrics.package_output_scroll.unwrap();
    assert!(max > 0.);
    assert!(
        (offset - max).abs() < 2.,
        "New submission should reveal the latest output: {offset}/{max}"
    );
    let rect = a
        .metrics
        .package_output_scroll
        .unwrap()
        .2
        .intersect(a.metrics.main_viewport.unwrap())
        .intersect(ctx.content_rect());
    frame(
        &ctx,
        &mut a,
        1200.,
        850.,
        vec![
            egui::Event::PointerMoved(rect.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0., 1000.),
                modifiers: Default::default(),
            },
        ],
    );
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    let (offset, max, _) = a.metrics.package_output_scroll.unwrap();
    assert!(
        offset < max - 20.,
        "Manual scrollback remains available: {offset}/{max}"
    );
    run(&ctx, &mut a, "apt-get --error-on=any update");
    run(&ctx, &mut a, "apt-file -F search usr/bin/ledger-check");
    for _ in 0..12 {
        draw(&ctx, &mut a);
    }
    let (offset, max, _) = a.metrics.package_output_scroll.unwrap();
    assert!(
        (offset - max).abs() < 2.,
        "Explicit Run returns to the actual new result: {offset}/{max}"
    );
    assert_eq!(
        terminal(&a, false).entries.last().unwrap().stdout,
        "records-tools: usr/bin/ledger-check\n"
    );
}
