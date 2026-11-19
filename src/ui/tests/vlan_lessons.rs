use super::debian_contents::painted;
use super::nm_lessons::{field, fill, opened, result, settle, tap};
use super::*;
const IDS: [&str; 2] = [
    "lpic2-vlan-profile-commission",
    "lpic2-vlan-return-handover",
];
#[test]
fn vlan_views_teach_parent_tag_and_both_directions_without_assignment_answers() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Worked example") && text.contains("parent") && text.contains("tag"));
        assert!(text.contains("802.1Q") && text.contains("next startup"));
        tap(&ctx, &mut a, "example");
        let text = painted(&ctx, &mut a);
        assert!(
            text.contains("Supplied VLAN attachments")
                && text.contains("native 99")
                && text.contains("access VLAN30")
        );
        tap(&ctx, &mut a, "insert");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((2, 2)));
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        tap(&ctx, &mut a, "assignment");
        let text = painted(&ctx, &mut a);
        assert!(text.contains("Requirements") && !text.contains("Insert worked profiles"));
        assert!(!text.contains("10.160.") && !text.contains("10.164."));
        a.progress.attempts.get_mut(id).unwrap().challenge = true;
        let text = painted(&ctx, &mut a);
        assert!(!text.contains("Worked example") && !text.contains("Insert worked profiles"));
    }
}
#[test]
fn vlan_native_editor_keyboard_wrong_correct_damage_retry_and_saved_state() {
    for id in IDS {
        let (ctx, mut a, i) = opened(id);
        let s = a.scenarios[i].clone();
        tap(&ctx, &mut a, "assignment");
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((0, 2)));
        let raw = s.solution[0]
            .strip_prefix(dlms_learning_lab::nm_workspace::PREFIX)
            .unwrap();
        fill(&ctx, &mut a, raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((2, 2)));
        assert!(progress::current_solved(&a.progress.attempts[id], &s));
        a.save();
        assert!(a.storage_error.is_none());
        let got = progress::load(&a.path, &a.scenarios).unwrap();
        assert_eq!(
            serde_json::to_value(&got).unwrap(),
            serde_json::to_value(&a.progress).unwrap()
        );
        tap(&ctx, &mut a, "file");
        tap(&ctx, &mut a, "file:reports.nmconnection");
        field(&ctx, &mut a, "editor", "[vlan]\nprotocol=802.1ad\n");
        assert!(!progress::current_solved(&a.progress.attempts[id], &s));
        assert!(result(&a).is_none());
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((0, 2)));
        fill(&ctx, &mut a, raw);
        tap(&ctx, &mut a, "run");
        settle(&ctx, &mut a);
        assert_eq!(result(&a), Some((2, 2)));
    }
}
#[test]
fn vlan_inputs_and_controls_reflow_at_supported_scales_and_light_dark() {
    let (ctx, mut a, _) = opened(IDS[1]);
    tap(&ctx, &mut a, "assignment");
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
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
            for (width, height) in [(1500., 1000.), (850., 1000.)] {
                for _ in 0..4 {
                    frame(&ctx, &mut a, width, height, vec![]);
                }
                for (key, rect, _) in &a.metrics.nm_profiles.controls {
                    assert!(rect.is_finite(), "{key}");
                    assert!(
                        rect.width() <= ctx.content_rect().width() + 1.,
                        "{key}: {rect:?}"
                    );
                }
                assert!(a.metrics.nm_profiles.editor.is_some());
            }
        }
    }
}
