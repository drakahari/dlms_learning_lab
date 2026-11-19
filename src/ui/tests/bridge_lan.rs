use super::debian_contents::painted;
use super::nm_lessons::{field, fill, opened, result, settle, tap};
use super::*;
const ID: &str = "lpic2-bridge-lan-commission";
#[test]
fn bridge_views_teach_port_forwarding_and_keep_independent_work_separate() {
    let (ctx, mut a, i) = opened(ID);
    let s = a.scenarios[i].clone();
    let text = painted(&ctx, &mut a);
    assert!(
        text.contains("A bridge joins segments")
            && text.contains("Worked example")
            && text.contains("Software bridge")
    );
    tap(&ctx, &mut a, "example");
    let text = painted(&ctx, &mut a);
    assert!(
        text.contains("Provided LAN peers")
            && text.contains("10.250.0.50")
            && text.contains("port-enp2s0.nmconnection")
    );
    tap(&ctx, &mut a, "insert");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((3, 3)));
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    tap(&ctx, &mut a, "assignment");
    let text = painted(&ctx, &mut a);
    assert!(
        text.contains("Requirements")
            && text.contains("10.240.0.50")
            && !text.contains("10.250.")
            && !text.contains("Insert worked profiles")
    );
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    let text = painted(&ctx, &mut a);
    assert!(!text.contains("Worked example") && !text.contains("Insert worked profiles"));
    assert!(text.contains("Requirements") && text.contains("Saved profile editor"));
}
#[test]
fn actual_editor_partial_repair_mask_failure_retry_and_safe_saved_drafts() {
    let (ctx, mut a, i) = opened(ID);
    let s = a.scenarios[i].clone();
    let engine::Goal::NmProfiles { definition } = &s.goal else {
        panic!()
    };
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((0, 3)));
    let raw = s.solution[0]
        .strip_prefix(dlms_learning_lab::nm_workspace::PREFIX)
        .unwrap();
    let good: dlms_learning_lab::nm_workspace::Files = serde_json::from_str(raw).unwrap();
    field(&ctx, &mut a, "editor", &good["bridge.nmconnection"]);
    assert!(result(&a).is_none());
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((1, 3)));
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Profiles need work") && text.contains("blocked"));
    fill(&ctx, &mut a, raw);
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((3, 3)));
    assert!(progress::current_solved(&a.progress.attempts[ID], &s));
    a.save();
    assert!(a.storage_error.is_none());
    assert_eq!(
        serde_json::to_value(progress::load(&a.path, &a.scenarios).unwrap()).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    tap(&ctx, &mut a, "file");
    tap(&ctx, &mut a, "file:bridge.nmconnection");
    field(
        &ctx,
        &mut a,
        "editor",
        &good["bridge.nmconnection"].replace("10.240.0.20/24", "10.240.0.20/25"),
    );
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    assert!(result(&a).is_none());
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((0, 3)));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("10.240.0.0/24") && text.contains("10.240.0.0/25"));
    fill(&ctx, &mut a, raw);
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((3, 3)));
    assert_eq!(definition.initial.starter.len(), 4);
}
#[test]
fn bridge_editor_case_and_actions_fit_all_supported_scales_and_light_dark_widths() {
    let (ctx, mut a, _) = opened(ID);
    tap(&ctx, &mut a, "assignment");
    for scale in [1., 1.15, 1.25, 1.5, 2.] {
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
                    a.metrics
                        .nm_profiles
                        .controls
                        .iter()
                        .find(|c| c.0 == "file")
                        .unwrap()
                        .1,
                ] {
                    assert!(
                        r.left() >= clip.left() - 2. && r.right() <= clip.right() + 2.,
                        "scale{scale}/width{width}: {r:?} {clip:?}"
                    );
                }
            }
        }
    }
}
