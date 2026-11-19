use super::debian_contents::painted;
use super::nm_lessons::{field, fill, opened, result, settle, tap};
use super::*;
const ID: &str = "lpic2-bonded-vlan-handover";
#[test]
fn bond_views_teach_forced_port_startup_and_keep_independent_answers_separate() {
    let (ctx, mut a, i) = opened(ID);
    let s = a.scenarios[i].clone();
    let text = painted(&ctx, &mut a);
    assert!(text.contains("autoconnect-ports=1") && text.contains("autoconnect=false"));
    assert!(text.contains("Worked example") && text.contains("bound"));
    tap(&ctx, &mut a, "example");
    let text = painted(&ctx, &mut a);
    assert!(
        text.contains("Supplied bond and physical candidates")
            && text.contains("port-enp2s0.nmconnection")
    );
    tap(&ctx, &mut a, "insert");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((3, 3)));
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    tap(&ctx, &mut a, "assignment");
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Requirements") && !text.contains("Insert worked profiles"));
    assert!(!text.contains("10.190.") && !text.contains("10.191."));
    a.progress.attempts.get_mut(ID).unwrap().challenge = true;
    let text = painted(&ctx, &mut a);
    assert!(!text.contains("Worked example") && !text.contains("Insert worked profiles"));
}
#[test]
fn bond_editor_partial_correct_damage_retry_and_saved_drafts_use_real_widgets() {
    let (ctx, mut a, i) = opened(ID);
    let s = a.scenarios[i].clone();
    let engine::Goal::NmProfiles { definition } = &s.goal else {
        panic!()
    };
    tap(&ctx, &mut a, "assignment");
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((0, 3)));
    let partial = definition.initial.starter["bond.nmconnection"]
        .replace("autoconnect-ports=0", "autoconnect-ports=1");
    field(&ctx, &mut a, "editor", &partial);
    assert!(result(&a).is_none());
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((2, 3)));
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    let text = painted(&ctx, &mut a);
    assert!(text.contains("Profiles need work") && text.contains("enp1s0 has no carrier"));
    let raw = s.solution[0]
        .strip_prefix(dlms_learning_lab::nm_workspace::PREFIX)
        .unwrap();
    fill(&ctx, &mut a, raw);
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((3, 3)));
    assert!(progress::current_solved(&a.progress.attempts[ID], &s));
    a.save();
    assert!(a.storage_error.is_none());
    let got = progress::load(&a.path, &a.scenarios).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&a.progress).unwrap()
    );
    tap(&ctx, &mut a, "file");
    tap(&ctx, &mut a, "file:bond.nmconnection");
    let broken = partial.replace("mode=active-backup", "mode=802.3ad");
    assert_ne!(broken, partial);
    field(&ctx, &mut a, "editor", &broken);
    assert!(!progress::current_solved(&a.progress.attempts[ID], &s));
    assert!(result(&a).is_none());
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((0, 3)));
    fill(&ctx, &mut a, raw);
    tap(&ctx, &mut a, "run");
    settle(&ctx, &mut a);
    assert_eq!(result(&a), Some((3, 3)));
}
#[test]
fn bond_five_file_editor_and_source_facts_fit_supported_scales_widths_and_themes() {
    let (ctx, mut a, _) = opened(ID);
    tap(&ctx, &mut a, "assignment");
    for scale in [1., 1.15, 1.25, 1.5, 1.75, 2.] {
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
                        "scale{scale} width{width}: {r:?} {clip:?}"
                    );
                }
            }
        }
    }
}
