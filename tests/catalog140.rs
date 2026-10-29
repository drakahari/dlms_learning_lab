//! Verify the original140 registration prefix under current147.
//! Pins authored input metadata, never regenerated historical learner results.
#[path = "support/teaching_amendment.rs"]
mod teaching_amendment;
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    shell, workspace,
};
use std::collections::{BTreeMap, BTreeSet};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
use digest as amendment_sha;
const NEW: [&str; 7] = [
    "shell-preserve-lines",
    "shell-pipeline-status",
    "shell-safe-handover",
    "lpic2-web-routing",
    "lpic2-web-pages",
    "lpic2-web-access",
    "lpic2-web-handover",
];
const OLD_PACKS: [(&str, &[u8]); 31] = [
    (
        "scenarios/starter-v1.json",
        include_bytes!("../scenarios/starter-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-unit-v1.json",
        include_bytes!("../scenarios/shell-unit-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-composition-v1.json",
        include_bytes!("../scenarios/shell-composition-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-reusable-v1.json",
        include_bytes!("../scenarios/shell-reusable-v1.json").as_slice(),
    ),
    (
        "scenarios/linux-files-v1.json",
        include_bytes!("../scenarios/linux-files-v1.json").as_slice(),
    ),
    (
        "scenarios/ipv6-static-v1.json",
        include_bytes!("../scenarios/ipv6-static-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-scope-v1.json",
        include_bytes!("../scenarios/shell-scope-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-text-v1.json",
        include_bytes!("../scenarios/shell-text-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-counts-v1.json",
        include_bytes!("../scenarios/shell-counts-v1.json").as_slice(),
    ),
    (
        "scenarios/linux-access-v1.json",
        include_bytes!("../scenarios/linux-access-v1.json").as_slice(),
    ),
    (
        "scenarios/vlan-data-v1.json",
        include_bytes!("../scenarios/vlan-data-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-file-batches-v1.json",
        include_bytes!("../scenarios/shell-file-batches-v1.json").as_slice(),
    ),
    (
        "scenarios/ipv4-static-v1.json",
        include_bytes!("../scenarios/ipv4-static-v1.json").as_slice(),
    ),
    (
        "scenarios/dns-forward-v1.json",
        include_bytes!("../scenarios/dns-forward-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-arrays-v1.json",
        include_bytes!("../scenarios/shell-arrays-v1.json").as_slice(),
    ),
    (
        "scenarios/linux-processes-v1.json",
        include_bytes!("../scenarios/linux-processes-v1.json").as_slice(),
    ),
    (
        "scenarios/discovery-v1.json",
        include_bytes!("../scenarios/discovery-v1.json").as_slice(),
    ),
    (
        "scenarios/storage-v1.json",
        include_bytes!("../scenarios/storage-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-capture-v1.json",
        include_bytes!("../scenarios/shell-capture-v1.json").as_slice(),
    ),
    (
        "scenarios/acl-policy-v1.json",
        include_bytes!("../scenarios/acl-policy-v1.json").as_slice(),
    ),
    (
        "scenarios/accounts-lifecycle-v1.json",
        include_bytes!("../scenarios/accounts-lifecycle-v1.json").as_slice(),
    ),
    (
        "scenarios/ospf-p2p-v1.json",
        include_bytes!("../scenarios/ospf-p2p-v1.json").as_slice(),
    ),
    (
        "scenarios/unit-manager-v1.json",
        include_bytes!("../scenarios/unit-manager-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-case-v1.json",
        include_bytes!("../scenarios/shell-case-v1.json").as_slice(),
    ),
    (
        "scenarios/mac-learning-v1.json",
        include_bytes!("../scenarios/mac-learning-v1.json").as_slice(),
    ),
    (
        "scenarios/linux-find-v1.json",
        include_bytes!("../scenarios/linux-find-v1.json").as_slice(),
    ),
    (
        "scenarios/dns-reverse-v1.json",
        include_bytes!("../scenarios/dns-reverse-v1.json").as_slice(),
    ),
    (
        "scenarios/shell-getopts-v1.json",
        include_bytes!("../scenarios/shell-getopts-v1.json").as_slice(),
    ),
    (
        "scenarios/dhcpv4-client-v1.json",
        include_bytes!("../scenarios/dhcpv4-client-v1.json").as_slice(),
    ),
    (
        "scenarios/debian-packages-v1.json",
        include_bytes!("../scenarios/debian-packages-v1.json").as_slice(),
    ),
    (
        "scenarios/spanning-tree-v1.json",
        include_bytes!("../scenarios/spanning-tree-v1.json").as_slice(),
    ),
];
fn index() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/catalog133-original-index.json")).unwrap()
}
fn exact<A: serde::Serialize + ?Sized, B: serde::Serialize + ?Sized>(a: &A, b: &B, label: &str) {
    fn diff(a: &serde_json::Value, b: &serde_json::Value) -> Option<String> {
        if a == b {
            return None;
        }
        match (a, b) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                if a.len() != b.len() {
                    return Some("object key count".into());
                }
                for (k, v) in a {
                    let Some(other) = b.get(k) else {
                        return Some(format!("missing {:.60}", k));
                    };
                    if let Some(d) = diff(v, other) {
                        return Some(format!("{:.60}.{:.120}", k, d));
                    }
                }
                Some("object".into())
            }
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                if a.len() != b.len() {
                    return Some("array length".into());
                }
                a.iter()
                    .zip(b)
                    .enumerate()
                    .find_map(|(i, (a, b))| diff(a, b).map(|d| format!("[{i}].{d:.120}")))
            }
            _ => Some("scalar/type (value withheld)".into()),
        }
    }
    let a = serde_json::to_value(a).unwrap();
    let b = serde_json::to_value(b).unwrap();
    let d = diff(&a, &b);
    assert!(d.is_none(), "{label}: {}", d.unwrap_or_default());
}
#[test]
fn exact_frozen133_typed_specs_and33_units_are_unchanged_prefixes_of_current147() {
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 171);
    let old = index();
    let pins = old["packs"].as_array().unwrap();
    assert_eq!(pins.len(), 31);
    for ((name, bytes), pin) in OLD_PACKS.iter().zip(pins) {
        assert_eq!(*name, pin["path"].as_str().unwrap());
        let original_bytes = teaching_amendment::original_pack(name, bytes).unwrap();
        let bytes = original_bytes.as_slice();
        assert_eq!(bytes.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(
            digest::hash(bytes),
            pin["sha256"].as_str().unwrap(),
            "Frozen authored pack changed: {name}"
        );
    }
    let original_packs = OLD_PACKS
        .iter()
        .map(|(name, bytes)| teaching_amendment::original_pack(name, bytes).unwrap())
        .collect::<Vec<_>>();
    let texts = original_packs
        .iter()
        .map(|bytes| std::str::from_utf8(bytes).unwrap())
        .collect::<Vec<_>>();
    let original = pack::load_catalog(&texts).unwrap();
    assert_eq!(original.len(), 133);
    exact(
        &teaching_amendment::original_catalog(&all[..133]).unwrap(),
        &original,
        "Every typed original Scenario field except the exact closed teaching amendment",
    );
    let ordered = all[..133]
        .iter()
        .map(|s| serde_json::json!({"id":s.id,"version":s.version}))
        .collect::<Vec<_>>();
    exact(&ordered, &old["catalog"], "Frozen133 order/versions");
    assert_eq!(
        all[133..140]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        NEW
    );
    let units = course::bundled(&all).unwrap();
    assert_eq!(units.len(), 44);
    let old_units: Vec<course::Unit> =
        serde_json::from_value(old["units"]["units"].clone()).unwrap();
    assert_eq!(old_units.len(), 33);
    exact(&units[..33], &old_units, "Every original unit field/order");
    assert_eq!(units[33].id, "shell-safe-automation");
    assert_eq!(units[34].id, "static-web-services");
    assert_eq!(
        units[33]
            .labs
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        NEW[..3]
    );
    assert_eq!(
        units[34]
            .labs
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        NEW[3..]
    );
    let mut tracks = BTreeMap::new();
    for s in &all[..140] {
        *tracks.entry(s.track.as_str()).or_insert(0usize) += 1;
    }
    assert_eq!(
        tracks,
        BTreeMap::from([
            ("CCNA", 42),
            ("LPIC-1", 28),
            ("LPIC-2", 24),
            ("Bash/Shell", 46)
        ])
    );
    let scripts = all
        .iter()
        .filter_map(|s| {
            if let Goal::Script { cases } = &s.goal {
                Some((s, cases))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 55);
    assert_eq!(scripts.iter().map(|(_, c)| c.len()).sum::<usize>(), 420);
    assert_eq!(
        scripts.iter().filter(|(s, _)| s.teaching.is_some()).count(),
        55
    );
}
#[test]
fn new_unit_prerequisites_preserve_real_cross_unit_links_and_all_four_web_tasks() {
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    let unit = units
        .iter()
        .find(|u| u.id == "shell-safe-automation")
        .unwrap();
    let by_lab = units
        .iter()
        .flat_map(|u| u.labs.iter().map(move |id| (id.as_str(), u.id.as_str())))
        .collect::<BTreeMap<_, _>>();
    let mut needed = BTreeSet::new();
    for s in &all[133..136] {
        for prerequisite in &s.prerequisites {
            let prior = by_lab[prerequisite.as_str()];
            if prior != unit.id.as_str() {
                needed.insert(prior);
            }
        }
    }
    assert_eq!(
        unit.prerequisites
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        needed
    );
    for (id, s) in NEW.iter().zip(&all[133..140]) {
        assert_eq!(*id, s.id);
        assert!(!engine::solved(&s.initial, &s.goal));
    }
    assert!(
        units[34].prerequisites.is_empty(),
        "The web introduction supplies its new terminology; no hidden prior Apache task"
    );
}
#[test]
fn old_actual_profiles_keep_exact_contexts_and_do_not_receive_new_answers() {
    let all = pack::bundled().unwrap();
    for (raw, count) in [
        (
            include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json").as_slice(),
            102,
        ),
        (
            include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice(),
            116,
        ),
    ] {
        let mut p = progress::decode(raw, &all).unwrap();
        assert_eq!(p.attempts.len(), count);
        let old = serde_json::to_value(&p.attempts).unwrap();
        let receipts = serde_json::to_value(&p.migration_receipts).unwrap();
        for id in NEW {
            assert!(!p.attempts.contains_key(id));
            assert!(!p.favorites.contains(id));
        }
        let encoded = serde_json::to_vec(&p).unwrap();
        let loaded = progress::decode(&encoded, &all).unwrap();
        exact(
            &loaded.attempts,
            &p.attempts,
            "Old load/re-encode attempts stay exact",
        );
        exact(
            &p.migration_receipts,
            &receipts,
            "Reading never stamps current receipts",
        );
        // Only this explicit constructor loop adds missing contexts. New histories
        // remain empty and no current/historical result is fabricated for those10.
        for s in &all {
            p.attempts
                .entry(s.id.clone())
                .or_insert_with(|| Attempt::new(s, 1));
        }
        assert_eq!(p.attempts.len(), 171);
        for (id, attempt) in old.as_object().unwrap() {
            exact(
                &p.attempts[id],
                attempt,
                "Existing Attempt unaffected by explicit new constructor",
            );
        }
        for s in &all[133..] {
            let a = &p.attempts[&s.id];
            assert_eq!(a.state, s.initial);
            assert!(!progress::current_solved(a, s));
            assert!(!p.favorites.contains(&s.id));
            if let State::Script { run, .. } = &a.state {
                assert!(run.is_none());
            }
            assert!(
                a.workspace
                    .as_ref()
                    .is_none_or(|w| w.script_history.is_none() && w.script_history_more.is_empty())
            );
        }
    }
}
#[test]
fn original_seven_reference_states_are_derived_and_goal_aware_breakage_stays_resumable() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all[133..140] {
        let mut a = Attempt::new(s, 1);
        for line in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
        }
        if let (State::Script { source, run }, Goal::Script { cases }) = (&mut a.state, &s.goal) {
            let results = cases.iter().map(|c| shell::run(source, c)).collect();
            *run = Some(shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: source.clone(),
                results,
            });
        }
        assert!(engine::solved(&a.state, &s.goal), "{}", s.id);
        workspace::prepare(&mut a);
        if matches!(a.state, State::ApacheStatic { .. }) {
            let action = dlms_learning_lab::apache_static::Action::Stop {};
            engine::execute_for_goal(
                &mut a.state,
                &s.goal,
                &format!(
                    "{}{}",
                    dlms_learning_lab::apache_static_workspace::PREFIX,
                    serde_json::to_string(&action).unwrap()
                ),
            )
            .unwrap();
        } else if let State::Script { source, .. } = &mut a.state {
            source.push_str("\nprintf changed\n");
        }
        assert!(!progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    exact(
        &loaded.attempts,
        &p.attempts,
        "Every broken new Attempt saves and loads",
    );
}
