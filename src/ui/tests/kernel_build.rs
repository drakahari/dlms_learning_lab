//! Actual native widgets in a headless frame; Fedora image evidence is separate.
use super::*;
use dlms_learning_lab::{kernel_source as binding, kernel_workspace as work};
const ID: &str = "lpic2-kernel-root-build";
fn open(view: teaching::View) -> (egui::Context, LabApp, usize) {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_catalog(&ctx, true, false, false, true);
    a.smoke = false;
    let i = a.scenarios.iter().position(|s| s.id == ID).unwrap();
    a.launch(i);
    let State::KernelBuild { simulation } = &mut a.progress.attempts.get_mut(ID).unwrap().state
    else {
        panic!()
    };
    simulation.view = view;
    settle(&ctx, &mut a);
    (ctx, a, i)
}
fn session(a: &LabApp) -> &work::Session {
    let State::KernelBuild { simulation } = &a.progress.attempts[ID].state else {
        panic!()
    };
    simulation
}
fn paint(ctx: &egui::Context, a: &mut LabApp, events: Vec<egui::Event>) {
    frame(ctx, a, 1500., 16000., events)
}
fn settle(ctx: &egui::Context, a: &mut LabApp) {
    for _ in 0..3 {
        paint(ctx, a, vec![]);
    }
}
fn hit(ctx: &egui::Context, a: &mut LabApp, key: &str) {
    let (_, r, clip) = a
        .metrics
        .kernel_build
        .controls
        .iter()
        .find(|r| r.0 == key)
        .unwrap_or_else(|| panic!("Missing {key}"))
        .clone();
    assert!(
        smoke::smoke_control_visible(
            r,
            clip.intersect(ctx.content_rect()),
            ctx.pixels_per_point()
        ),
        "Clipped {key}"
    );
    for pressed in [true, false] {
        paint(
            ctx,
            a,
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
    settle(ctx, a);
}
fn input(ctx: &egui::Context, a: &mut LabApp, key: &str, value: &str) {
    hit(ctx, a, key);
    paint(
        ctx,
        a,
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
            egui::Event::Paste(value.into()),
        ],
    );
    settle(ctx, a);
}
#[test]
fn kernel_build_intro_can_be_skipped_and_challenge_does_not_reveal_example() {
    let (ctx, mut a, _) = open(teaching::View::Introduction);
    let initial = session(&a).assignment.clone();
    assert!(
        a.metrics
            .kernel_build
            .controls
            .iter()
            .any(|r| r.0 == "worked")
    );
    assert!(
        !a.metrics
            .kernel_build
            .controls
            .iter()
            .any(|r| r.0 == "config")
    );
    hit(&ctx, &mut a, "assignment");
    assert_eq!(session(&a).assignment, initial);
    assert!(
        !a.metrics
            .kernel_build
            .controls
            .iter()
            .any(|r| r.0 == "fill-example" || r.0 == "worked-answer")
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    settle(&ctx, &mut a);
    assert!(
        !a.metrics
            .kernel_build
            .controls
            .iter()
            .any(|r| r.0 == "example" || r.0 == "worked")
    );
}
#[test]
fn kernel_build_practice_is_explicit_and_preserves_assignment_work() {
    let (ctx, mut a, i) = open(teaching::View::Example);
    let original = session(&a).assignment.clone();
    assert!(
        !a.metrics
            .kernel_build
            .controls
            .iter()
            .any(|r| r.0 == "fill-example")
    );
    hit(&ctx, &mut a, "worked-answer");
    hit(&ctx, &mut a, "fill-example");
    assert!(!session(&a).example.work.checked);
    hit(&ctx, &mut a, "check");
    let d = binding::source(&a.scenarios[i]).unwrap();
    assert!(d.example.solved(&session(&a).example.work));
    assert_eq!(session(&a).assignment, original);
    assert!(!a.metrics.completion);
}
#[test]
fn kernel_build_config_console_and_assembly_produce_checked_outcomes_and_clear_stale_success() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "check");
    assert!(!a.metrics.completion);
    let d = binding::source(&a.scenarios[i]).unwrap();
    let config = d
        .assignment
        .reference
        .iter()
        .find_map(|op| {
            if let work::Action::Config { text } = op {
                Some(text.clone())
            } else {
                None
            }
        })
        .unwrap();
    input(&ctx, &mut a, "config", &config);
    hit(&ctx, &mut a, "save-config");
    input(&ctx, &mut a, "command", "make all");
    hit(&ctx, &mut a, "run");
    hit(&ctx, &mut a, "build-initramfs");
    hit(&ctx, &mut a, "check");
    assert!(a.metrics.completion);
    input(&ctx, &mut a, "config", "# unfinished draft\n");
    assert!(!a.metrics.completion);
    assert!(!session(&a).assignment.work.checked);
    hit(&ctx, &mut a, "check");
    assert!(
        a.metrics
            .kernel_build
            .controls
            .iter()
            .any(|r| r.0 == "failed-case")
    );
}
#[test]
fn kernel_build_single_line_focus_and_layout_survive_text_scaling() {
    let (ctx, mut a, _) = open(teaching::View::Assignment);
    hit(&ctx, &mut a, "config");
    paint(
        &ctx,
        &mut a,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    settle(&ctx, &mut a);
    let save = a
        .metrics
        .kernel_build
        .focus
        .iter()
        .find(|r| r.0 == "save-config")
        .unwrap()
        .1;
    assert_eq!(ctx.memory(|m| m.focused()), Some(save));
    for scale in [1., 1.5, 2.] {
        a.scale = scale;
        ctx.set_zoom_factor(scale);
        for width in [760., 1450.] {
            frame(&ctx, &mut a, width, 16000., vec![]);
            if scale == 1. && width == 1450. {
                let field = |key: &str| {
                    a.metrics
                        .kernel_build
                        .fields
                        .iter()
                        .find(|r| r.0 == key)
                        .unwrap()
                        .1
                };
                assert!(
                    (field("config").top() - field("command").top()).abs()
                        < ctx.style().text_styles[&egui::TextStyle::Body].size * 1.5,
                    "Wide workspace must put configuration and build console beside each other: config={:?}, command={:?}",
                    field("config"),
                    field("command")
                );
            }
            for (key, r, clip) in &a.metrics.kernel_build.controls {
                if ["command", "save-config", "run", "check"].contains(&key.as_str()) {
                    assert!(
                        r.left() >= clip.left() - 1. && r.right() <= clip.right() + 1.,
                        "Clipped {key} at {width}/{scale}: {r:?} {clip:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn kernel_build_output_labels_and_values_have_distinct_aligned_space() {
    let (ctx, mut a, i) = open(teaching::View::Assignment);
    for populated in [false, true] {
        if populated {
            solve(&mut a, i);
        }
        for scale in [1., 1.5, 2.] {
            a.scale = scale;
            ctx.set_zoom_factor(scale);
            for width in [760., 1450.] {
                for _ in 0..3 {
                    frame(&ctx, &mut a, width, 16000., vec![]);
                }
                let mut previous: Option<f32> = None;
                for name in ["Image build", "Module build", "Build initramfs"] {
                    let find = |prefix: &str| {
                        a.metrics
                            .kernel_build
                            .controls
                            .iter()
                            .find(|r| r.0 == format!("{prefix}:{name}"))
                            .unwrap()
                            .1
                    };
                    let label = find("artifact-label");
                    let value = find("artifact-value");
                    assert!(
                        !label.intersects(value),
                        "Overlapping {name} at {width}/{scale}: {label:?} {value:?}"
                    );
                    if label.center().y <= value.bottom() && value.top() < label.bottom() {
                        if let Some(x) = previous {
                            assert!((value.left() - x).abs() < 1., "Value columns shifted");
                        }
                        previous = Some(value.left());
                    }
                }
            }
        }
    }
}
