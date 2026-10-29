//! Registration is append-only; new curriculum must not fabricate learner work.
#[path = "../examples/support/automation_digest.rs"]
mod amendment_sha;
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::{course, engine::Goal, pack, progress};
use std::collections::BTreeMap;
const NEW: [&str; 8] = [
    "lpic1-package-install",
    "lpic1-package-metadata",
    "lpic1-package-payload",
    "lpic1-package-handover",
    "ccna-stp-root",
    "ccna-stp-local-cost",
    "ccna-stp-parallel",
    "ccna-stp-handover",
];

#[test]
fn old125_specs_and31_units_remain_exact_within_original133_prefix_of_current147() {
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 168);
    let all = &current[..133];
    assert_eq!(all.len(), 133);
    let baseline: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/spanning-tree-original125.json")).unwrap();
    let original: Vec<pack::Scenario> =
        serde_json::from_value(baseline["scenarios"].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(teaching_amendment::original_catalog(&all[..125]).unwrap()).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    assert_eq!(
        all[125..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        NEW
    );
    let current_units = course::bundled(&current).unwrap();
    assert_eq!(current_units.len(), 43);
    let units = &current_units[..33];
    assert_eq!(units.len(), 33);
    assert_eq!(
        serde_json::to_value(&units[..31]).unwrap(),
        baseline["units"]["units"]
    );
    assert_eq!(units[31].id, "debian-package-administration");
    assert_eq!(units[32].id, "spanning-tree-steady-state");
    assert_eq!(
        units[31]
            .labs
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        NEW[..4]
    );
    assert_eq!(
        units[32]
            .labs
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        NEW[4..]
    );
    let mut counts = BTreeMap::new();
    for s in all {
        *counts.entry(s.track.as_str()).or_insert(0) += 1;
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            ("CCNA", 42),
            ("LPIC-1", 28),
            ("LPIC-2", 20),
            ("Bash/Shell", 43)
        ])
    );
    let cases = all
        .iter()
        .filter_map(|s| {
            if let Goal::Script { cases } = &s.goal {
                Some(cases.len())
            } else {
                None
            }
        })
        .sum::<usize>();
    assert_eq!(cases, 282, "Bash evaluator9 lesson inputs are unchanged");
}

#[test]
fn six_real_example_annotations_validate_and_old_writer_history_creates_no_new_attempts() {
    let all = pack::bundled().unwrap();
    let mut annotations = 0;
    for s in &all[125..129] {
        let l = s.debian_packages_teaching.as_ref().unwrap();
        l.validate().unwrap();
        for a in &l.example_actions {
            assert!(
                a.command
                    .as_ref()
                    .is_some_and(|c| !c.starts_with("packages "))
            );
            annotations += 1;
        }
    }
    assert_eq!(annotations, 6);
    let bytes = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let p = progress::decode(bytes, &all).unwrap();
    for id in NEW {
        assert!(!p.attempts.contains_key(id));
        assert!(!p.favorites.contains(id));
    }
    let original = serde_json::to_value(&p.attempts).unwrap();
    let q = progress::decode(&serde_json::to_vec(&p).unwrap(), &all).unwrap();
    assert_eq!(serde_json::to_value(&q.attempts).unwrap(), original);
    for id in NEW {
        assert!(!q.attempts.contains_key(id));
    }
}
