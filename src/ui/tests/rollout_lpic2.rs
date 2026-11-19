//! Presentation-only regressions; source/engine objectives remain authoritative.
use super::*;

#[test]
fn rollout_lpic2_all_registered_labs_have_specific_three_view_reviews_and_guidance() {
    let (catalog, _) = ccna_pilot::catalog();
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../../scenarios/presentation/lpic2.json")).unwrap();
    let labs = catalog
        .iter()
        .filter(|s| s.track == "LPIC-2")
        .collect::<Vec<_>>();
    assert_eq!(labs.len(), 45);
    assert_eq!(entries.len(), labs.len());
    for scenario in labs {
        let raw = entries.iter().find(|e| e["id"] == scenario.id).unwrap();
        assert_eq!(raw["status"], "updated");
        for view in ["introduction", "example", "assignment"] {
            assert!(
                raw["review"][view].as_str().unwrap().len() > 80,
                "missing concrete {view} review: {}",
                scenario.id
            );
        }
        let entry = lesson_presentation::entry(scenario).unwrap();
        assert_eq!(entry.practice.len(), 3, "{}", scenario.id);
        assert!(
            entry
                .practice
                .iter()
                .all(|p| !p.title.is_empty() && !p.action.is_empty() && !p.observe.is_empty())
        );
        assert!(
            entry.worked.len() >= 2,
            "worked demonstration missing: {}",
            scenario.id
        );
        assert!(
            entry
                .worked
                .iter()
                .all(|r| !r.label.is_empty() && !r.value.is_empty() && !r.explanation.is_empty())
        );
        let compact = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        assert_eq!(
            compact(&entry.requirements.join(" ")),
            compact(&scenario.objective),
            "presentation checklist dropped/changed an original condition: {}",
            scenario.id
        );
    }
}

#[test]
fn rollout_lpic2_worked_literals_keep_record_syntax_and_distinct_practice_values() {
    let entries = lesson_presentation::entries();
    let dual = &entries["lpic2-dns-dualstack"].worked;
    assert_eq!(
        dual[0].value,
        "files.dual-practice.test. 300 IN A 192.0.2.140"
    );
    assert_eq!(
        dual[1].value,
        "files.dual-practice.test. 300 IN AAAA 2001:db8::140"
    );
    let export = &entries["lpic2-nfs-reference"].worked[0].value;
    assert_eq!(export, "/srv/studio-manuals/reference 198.51.100.20(ro)");
    let page = &entries["lpic2-web-handover"].worked[0].value;
    assert_eq!(page, "Exhibit 2 ready + newline");
    let access = &entries["lpic2-web-access"].worked[0].value;
    assert!(access.ends_with("</Directory>"));
    assert!(access.contains("Require all denied"));
}
