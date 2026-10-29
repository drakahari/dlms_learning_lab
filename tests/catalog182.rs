//! Normal two-task registration proof; sourceproposal until actual scoped/full/native gates.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt},
    shell,
};
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use one_off_boundary::loop_registration as registration;
#[path = "support/combined_catalog.rs"]
mod combined;
use serde_json::{Value, json};
fn v<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn recipe() -> Value {
    serde_json::from_str(include_str!("fixtures/catalog182-registration.json")).unwrap()
}
#[test]
fn old180_fullspecs_all55scripts420cases55examples_and_original45packs_are_immutable() {
    assert_eq!(shell::EVALUATOR_REVISION, 13);
    let all = pack::bundled().unwrap();
    let old = registration::before_scenarios(&all);
    assert_eq!(old.len(), 180);
    let r = recipe();
    let mut original = Vec::new();
    for pin in r["before_packs"].as_array().unwrap() {
        let p = pin["path"].as_str().unwrap();
        assert!(p.starts_with("scenarios/") && std::path::Path::new(p).components().count() == 2);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(p);
        let meta = std::fs::symlink_metadata(&path).unwrap();
        assert!(
            meta.is_file()
                && !meta.file_type().is_symlink()
                && meta.len() <= pack::MAX_PACK_BYTES as u64
        );
        let raw = std::fs::read(path).unwrap();
        assert_eq!(raw.len() as u64, pin["bytes"].as_u64().unwrap());
        assert_eq!(digest::hash(&raw), pin["sha256"]);
        let p: pack::Pack = serde_json::from_slice(&raw).unwrap();
        original.extend(p.scenarios);
    }
    assert_eq!(r["before_packs"].as_array().unwrap().len(), 45);
    assert_eq!(original.len(), 180);
    assert_eq!(v(old), v(&original));
    let old_scripts = old
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(old_scripts.len(), 55);
    assert_eq!(
        old_scripts
            .iter()
            .map(|s| if let Goal::Script { cases } = &s.goal {
                cases.len()
            } else {
                0
            })
            .sum::<usize>(),
        420
    );
    assert_eq!(
        old_scripts.iter().filter(|s| s.teaching.is_some()).count(),
        55
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| old.iter().filter(|s| s.track == t).count()),
        [53, 40, 32, 55]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|t| {
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all)
                .iter()
                .filter(|s| s.track == t)
                .count()
        }),
        [53, 40, 32, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [53, 40, 36, 57]
    );
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.goal, Goal::Script { .. }))
            .count(),
        57
    );
    assert_eq!(
        all.iter()
            .filter_map(|s| if let Goal::Script { cases } = &s.goal {
                Some(cases.len())
            } else {
                None
            })
            .sum::<usize>(),
        442
    );
    assert_eq!(all.iter().filter(|s| s.teaching.is_some()).count(), 57);
    for s in &all[180..182] {
        assert_eq!(s.version, 1);
        assert_eq!(s.unit, "shell-loop-recovery");
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        assert!(!progress::current_solved(&Attempt::new(s, 1), s));
    }
}
#[test]
fn exact48_to47_raw_and_typed_inverse_preserves_all_priorunits_and_historicalrecipes() {
    let all = pack::bundled().unwrap();
    let old = registration::before_scenarios(&all);
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let checked48 = registration::lvm_registration::before_units(current_raw).unwrap();
    let raw = checked48.as_slice();
    let before = registration::before_units(raw).unwrap();
    let r = recipe();
    assert_eq!(digest::hash(raw), r["after_units48_sha256"]);
    assert_eq!(digest::hash(&before), r["before_units47_sha256"]);
    let current_units = course::bundled(&all).unwrap();
    assert_eq!(current_units.len(), 49);
    let units = registration::lvm_registration::before_typed_units(&current_units);
    let old_units = course::load(std::str::from_utf8(&before).unwrap(), old).unwrap();
    assert_eq!((units.len(), old_units.len()), (48, 47));
    assert_eq!(v(&units[..47]), v(&old_units));
    assert_eq!(v(&units[47]), r["loop_unit"]);
    assert_eq!(units[47].labs, registration::IDS);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [14, 10, 8, 16]
    );
    let prior37 = combined::old_unit_bytes(raw).unwrap();
    assert_eq!(prior37, combined::old_unit_bytes(&before).unwrap());
    assert_eq!(
        serde_json::from_slice::<Value>(&prior37).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
    assert!(registration::before_units(&before).is_err());
    let mut spaced = raw.to_vec();
    spaced.push(b' ');
    assert!(registration::before_units(&spaced).is_err());
    for key in ["first-unit", "new-prereq", "new-id", "new-lab"] {
        let mut changed: Value = serde_json::from_slice(raw).unwrap();
        match key {
            "first-unit" => changed["units"][0]["summary"] = json!("unapproved old edit"),
            "new-prereq" => changed["units"][47]["prerequisites"][0] = json!("missing-unit"),
            "new-id" => changed["units"][47]["id"] = json!("linux-one-off-jobs"),
            _ => changed["units"][47]["labs"][0] = json!("shell-report"),
        };
        let text = serde_json::to_vec(&changed).unwrap();
        assert!(registration::before_units(&text).is_err());
    }
}
#[test]
fn exact_newappend_checks_do_not_ignore_changedtasks_or_edited_predecessor_identities() {
    let all = pack::bundled().unwrap();
    for (which, index) in [("new", 180), ("old", 0)] {
        let mut changed = all.clone();
        if which == "new" {
            changed[index].objective.push_str(" unapproved")
        } else {
            changed[index].version += 1;
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration::before_scenarios(&changed)
            }))
            .is_err()
        );
    }
    let mut extra = all.clone();
    extra.push(all[180].clone());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            registration::before_scenarios(&extra)
        }))
        .is_err()
    );
    let old = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let p = progress::decode(old, &all).unwrap();
    for id in registration::IDS {
        assert!(!p.attempts.contains_key(id));
        assert!(!p.favorites.contains(id));
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert!(matches!(s.initial, State::Script { run: None, .. }));
    }
}

#[test]
fn nonempty_authored_guides_are_preserved_and_modified_guidance_is_refused() {
    let all = pack::bundled().unwrap();
    for index in [180, 181] {
        assert_eq!(all[index].guide.len(), 3);
        for field in ["title", "why", "action", "commands"] {
            let mut changed = all.clone();
            let step = &mut changed[index].guide[0];
            match field {
                "title" => step.title.push_str(" altered"),
                "why" => step.why.push_str(" altered"),
                "action" => step.action.push_str(" altered"),
                "commands" => step.commands.push(pack::CommandExample {
                    command: "printf 'altered'".into(),
                    explanation: "Different guidance".into(),
                    simulator_only: false,
                }),
                _ => unreachable!(),
            }
            assert!(std::panic::catch_unwind(|| registration::before_scenarios(&changed)).is_err());
        }
    }
}
