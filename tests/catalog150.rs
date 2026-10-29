//! Combined154 Bash boundary: exact original147 plus Bash3, then NFS4.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt},
    script_teaching, shell, workspace,
};
use serde_json::Value;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "support/cron_registration.rs"]
mod cron_registration;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
const IDS: [&str; 3] = [
    "shell-command-options",
    "shell-command-dispatch",
    "shell-command-plan",
];
fn review() -> Value {
    serde_json::from_str(include_str!("fixtures/shell-command-vectors-review.json")).unwrap()
}
fn existing() -> Vec<Scenario> {
    let r = review();
    let mut texts = Vec::new();
    for pin in r["original_source_pins"].as_array().unwrap() {
        let name = pin["path"].as_str().unwrap();
        if name == "scenarios/units-v1.json" {
            continue;
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(digest::hash(&bytes), pin["sha256"].as_str().unwrap());
        texts.push(String::from_utf8(bytes).unwrap());
    }
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap()
}
fn old_units(raw: &[u8]) -> Result<Vec<u8>, String> {
    combined_catalog::old_unit_bytes(raw)
}

#[test]
fn exact_original147_typed_specifications_and_order_are_the_combined154_prefix() {
    let old = existing();
    assert_eq!(old.len(), 147);
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 176);
    assert_eq!(
        serde_json::to_value(&all[..147]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        all[147..150]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let new: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/shell-command-vectors-v1.json")).unwrap();
    assert_eq!(
        serde_json::to_value(&all[147..150]).unwrap(),
        serde_json::to_value(new.scenarios).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [53, 36, 32, 55]
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
}
#[test]
fn exact_closed_unit_append_reconstructs_the_complete_original37_unit_bytes() {
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let raw39 = cron_registration::previous_units(
        &cron_registration::private_cron_units(current_raw).unwrap(),
    )
    .unwrap();
    let raw = raw39.as_slice();
    let old = old_units(raw).unwrap();
    let original = course::load(std::str::from_utf8(&old).unwrap(), &existing()).unwrap();
    assert_eq!(original.len(), 37);
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!(units.len(), 46);
    assert_eq!(
        serde_json::to_value(&units[..37]).unwrap(),
        serde_json::to_value(original).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&units[37]).unwrap(),
        review()["unit_proposal"]
    );
    let mut altered = raw.to_vec();
    altered.push(b' ');
    assert!(old_units(&altered).is_err());
    let mut v: Value = serde_json::from_slice(raw).unwrap();
    v["units"][0]["title"] = "Unreviewed old unit".into();
    assert!(old_units(&serde_json::to_vec(&v).unwrap()).is_err());
    v["units"][0]["title"] = "Another unreviewed title".into();
    assert!(old_units(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn normal_new_labs_have_real_prerequisites_incomplete_attempts_and_separate_teaching() {
    let all = pack::bundled().unwrap();
    for s in &all[147..150] {
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        assert_eq!(s.version, 1);
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        assert!(!progress::current_solved(&a, s));
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .view,
            script_teaching::View::Introduction
        );
        let before = a.state.clone();
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_result = Some(shell::run(
            &l.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn genuine_old_work_remains_exact_without_new_attempts_answers_or_mastery() {
    let all = pack::bundled().unwrap();
    for bytes in [
        include_bytes!("fixtures/shell-v7-synthetic-writer-progress.json").as_slice(),
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json").as_slice(),
    ] {
        let before: Value = serde_json::from_slice(bytes).unwrap();
        let loaded = progress::decode(bytes, &all).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded.attempts).unwrap(),
            before["attempts"]
        );
        assert_eq!(
            serde_json::to_value(&loaded.migration_receipts).unwrap(),
            before["migration_receipts"]
        );
        assert!(
            IDS.iter()
                .all(|id| !loaded.attempts.contains_key(*id) && !loaded.favorites.contains(*id))
        );
        for a in loaded.attempts.values() {
            if let State::Script {
                run: Some(record), ..
            } = &a.state
            {
                assert_ne!(record.engine_revision, shell::EVALUATOR_REVISION);
            }
        }
    }
}
#[test]
fn current_full_assignment_truth_comes_from_behavior_and_new_source_recheck() {
    let all = pack::bundled().unwrap();
    for s in &all[147..150] {
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        let source = &s.solution[0];
        let record = shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(source, c)).collect(),
        };
        let mut state = State::Script {
            source: source.clone(),
            run: Some(record),
        };
        assert!(engine::solved(&state, &s.goal));
        let State::Script { source, .. } = &mut state else {
            panic!()
        };
        source.push_str("\n# current edit\n");
        assert!(!engine::solved(&state, &s.goal));
    }
}
