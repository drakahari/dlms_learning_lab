use super::debian_contents::painted;
use super::nm_lessons::{field, fill, opened, result, settle, tap};
use super::*;
const IDS: [&str; 2] = ["lpic1-nm-dhcp-startup", "lpic1-nm-dhcp-local-only"];

#[test]
fn nm_dhcp_three_views_teach_acquisition_and_keep_independent_work_unsolved() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("method=auto") && text.contains("autoconnect=true"));
        assert!(text.contains("lease") && text.contains("Worked example"));
        tap(&ctx, &mut a, "example");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Test startup profiles") && text.contains("Supplied DHCP"));
        tap(&ctx, &mut a, "insert");
        let State::NmProfiles { simulation } = &a.progress.attempts[id].state else {
            panic!()
        };
        let engine::Goal::NmProfiles { definition } = &s.goal else {
            panic!()
        };
        assert_eq!(simulation.practice.draft, definition.example_worked);
        assert!(simulation.practice.tested.is_none());
        assert_eq!(simulation.assignment.draft, definition.initial.starter);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((3, 3)));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements") && !text.contains("Insert worked profiles"));
        assert!(!text.contains("10.150.") && !text.contains("10.152."));
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Insert worked profiles") && !text.contains("Worked example"));
    }
}

#[test]
fn nm_dhcp_editor_keyboard_run_wrong_correct_damage_retry_and_saved_drafts() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((0, 3)));
        let raw = s.solution[0]
            .strip_prefix(dlms_learning_lab::nm_workspace::PREFIX)
            .unwrap();
        fill(&ctx, &mut a, raw);
        assert!(result(&a).is_none());
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
                modifiers: egui::Modifiers::CTRL,
            }],
        );
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((3, 3)));
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let restored = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        tap(&ctx, &mut a, "file");
        tap(&ctx, &mut a, "file:app.nmconnection");
        field(&ctx, &mut a, "editor", "[ipv4]\nmethod=shared\n");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((0, 3)));
        fill(&ctx, &mut a, raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((3, 3)));
    }
}

#[test]
fn nm_dhcp_new_lease_facts_and_editor_controls_fit_wide_narrow_and_larger_text() {
    for id in IDS {
        let (ctx, mut a, _) = opened(id);
        tap(&ctx, &mut a, "assignment");
        for scale in [1., 1.5, 2.] {
            for light in [false, true] {
                a.scale = scale;
                ctx.set_zoom_factor(scale);
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
                    for _ in 0..4 {
                        frame(&ctx, &mut a, width, 950., vec![]);
                    }
                    let clip = a.metrics.main_viewport.unwrap();
                    for r in [
                        a.metrics.nm_profiles.editor.unwrap().0,
                        a.metrics
                            .nm_profiles
                            .controls
                            .iter()
                            .find(|c| c.0 == "run")
                            .unwrap()
                            .1,
                        a.metrics
                            .nm_profiles
                            .controls
                            .iter()
                            .find(|c| c.0 == "case")
                            .unwrap()
                            .1,
                    ] {
                        assert!(
                            r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                            "{id} scale{scale} width{width}: {r:?} clip{clip:?}"
                        );
                    }
                }
            }
        }
    }
}
