//! Registered lesson content on the approved views, separate from capability fixtures.
use super::*;
fn texts(shape: &egui::Shape, out: &mut String) {
    match shape {
        egui::Shape::Text(t) => {
            assert!(
                !t.galley.job.justify,
                "stretched text: {}",
                t.galley.job.text
            );
            out.push_str(&t.galley.job.text);
            out.push('\n');
        }
        egui::Shape::Vec(shapes) => {
            for s in shapes {
                texts(s, out)
            }
        }
        _ => {}
    }
}
#[test]
fn registered_syslog_views_teach_worked_settings_and_keep_independent_answers_hidden() {
    for id in [
        "ccna-syslog-local-remote",
        "ccna-syslog-export-path",
        "ccna-syslog-facility-routing",
        "ccna-syslog-policy-handover",
    ] {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        let source = a.scenarios[i].clone();
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            if let State::Syslog { simulation } =
                &mut a.progress.attempts.get_mut(id).unwrap().state
            {
                simulation.view = view;
            }
            let mut text = String::new();
            for _ in 0..4 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1500., 6000.),
                        )),
                        ..Default::default()
                    },
                    |ctx| a.render(ctx),
                );
                text.clear();
                for s in output.shapes {
                    texts(&s.shape, &mut text);
                }
            }
            let entry = lesson_presentation::entry(&source).unwrap();
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("WHAT YOU’LL LEARN"));
                    assert!(text.contains("Worked example"));
                    assert!(text.contains(&entry.worked[0].value));
                }
                teaching::View::Example => {
                    assert!(text.contains(&entry.practice[0].title));
                    assert!(text.contains(&entry.practice[1].title));
                    assert!(text.contains(&entry.practice[2].title));
                    assert!(text.contains("Fill example settings"));
                }
                teaching::View::Assignment => {
                    assert!(text.contains("Requirements"));
                    assert!(!text.contains("Worked example"));
                    assert!(!text.contains("apply_source"));
                    assert!(!text.contains("Fill example settings"));
                }
            }
            assert!(!progress::current_solved(&a.progress.attempts[id], &source));
        }
    }
}
