//! Normal four-task LVM registration and source/restore boundaries. Prospective.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    lvm_workspace as work, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::{Value, json};
#[path = "support/combined_catalog.rs"]
mod combined;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/lvm_original_catalog.rs"]
mod original;
use one_off_boundary::loop_registration::lvm_registration as registration;
fn v<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    a
}
#[test]
fn normal186_retains_full182_objects_all57scripts442cases57examples_and_exact_four_lvm_sources() {
    let all = pack::bundled().unwrap();
    let old = original::load();
    assert_eq!(old.len(), 182);
    assert_eq!(all.len(), 186);
    assert_eq!(v(&all[..182]), v(&old));
    let before = registration::before_scenarios(&all);
    assert_eq!(v(before), v(&old));
    let source: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    assert_eq!(v(&all[182..]), v(&source.scenarios));
    assert_eq!(source.scenarios.len(), 4);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| old.iter().filter(|s| s.track == t).count()),
        [53, 40, 32, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [53, 40, 36, 57]
    );
    let scripts = all
        .iter()
        .filter(|s| matches!(s.initial, State::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 57);
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 57);
    assert_eq!(
        scripts
            .iter()
            .map(|s| if let Goal::Script { cases } = &s.goal {
                cases.len()
            } else {
                0
            })
            .sum::<usize>(),
        442
    );
    for s in &all[182..] {
        assert_eq!(s.unit, "lvm-volume-growth");
        assert!(matches!(s.initial, State::Lvm { .. }) && matches!(s.goal, Goal::Lvm { .. }));
        assert!(!progress::current_solved(&fresh(s), s));
        work::source(s).unwrap();
    }
}
#[test]
fn strict49_to48_raw_and_complete_typed_inverse_retains_every_old_recipe_without_tail_acceptance() {
    let all = pack::bundled().unwrap();
    let old = original::load();
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    assert_eq!(before, include_bytes!("fixtures/lvm-units-before182.json"));
    let r: Value =
        serde_json::from_str(include_str!("fixtures/catalog186-registration.json")).unwrap();
    assert_eq!(digest::hash(raw), r["after_units49_sha256"]);
    assert_eq!(digest::hash(&before), r["before_units48_sha256"]);
    let units = course::bundled(&all).unwrap();
    let prior = course::load(std::str::from_utf8(&before).unwrap(), &old).unwrap();
    assert_eq!((units.len(), prior.len()), (49, 48));
    assert_eq!(v(&units[..48]), v(&prior));
    assert_eq!(v(&units[48]), r["lvm_unit"]);
    assert_eq!(units[48].labs, registration::IDS);
    assert_eq!(units[48].prerequisites, ["persistent-mounts"]);
    assert_eq!(v(registration::before_typed_units(&units)), v(&prior));
    let old37 = combined::old_unit_bytes(raw).unwrap();
    assert_eq!(old37, combined::old_unit_bytes(&before).unwrap());
    assert_eq!(
        serde_json::from_slice::<Value>(&old37).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
    assert!(registration::before_units(&before).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(registration::before_units(&extra).is_err());
    for field in [
        "predecessor",
        "new-id",
        "new-lab",
        "new-prerequisite",
        "extra-unit",
    ] {
        let mut bad: Value = serde_json::from_slice(raw).unwrap();
        match field {
            "predecessor" => bad["units"][0]["summary"] = json!("unapproved previous edit"),
            "new-id" => bad["units"][48]["id"] = json!("shell-loop-recovery"),
            "new-lab" => bad["units"][48]["labs"][0] = json!("shell-loop-intake"),
            "new-prerequisite" => bad["units"][48]["prerequisites"][0] = json!("missing-unit"),
            _ => {
                let last = bad["units"][48].clone();
                bad["units"].as_array_mut().unwrap().push(last);
            }
        }
        assert!(registration::before_units(&serde_json::to_vec(&bad).unwrap()).is_err());
        let bad = bad["units"].as_array().unwrap();
        assert!(std::panic::catch_unwind(|| registration::before_values(bad)).is_err());
    }
}
#[test]
fn source_boundaries_reject_partial_or_extra_lvm_and_source_mutations_without_broadening_old_authority()
 {
    let all = pack::bundled().unwrap();
    for mode in 0..5 {
        let mut bad = all.clone();
        match mode {
            0 => {
                bad.pop().unwrap();
            }
            1 => bad.push(all[182].clone()),
            2 => bad[182].objective.push_str(" altered"),
            3 => {
                let Goal::Lvm { definition } = &mut bad[182].goal else {
                    panic!()
                };
                definition.targets[0].minimum_filesystem += 4096;
            }
            _ => bad[0].version += 1,
        }
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                registration::before_scenarios(&bad)
            }))
            .is_err()
        );
    }
}
#[test]
fn current_catalog_never_invents_lvm_work_or_credit_in_a_retained_old_profile() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let old: Progress = serde_json::from_slice(raw).unwrap();
    let loaded = progress::decode(raw, &all).unwrap();
    for id in registration::IDS {
        assert!(!old.attempts.contains_key(id) && !loaded.attempts.contains_key(id));
        assert!(!old.favorites.contains(id) && !loaded.favorites.contains(id));
        let source = all.iter().find(|s| s.id == id).unwrap();
        assert!(!progress::current_solved(&fresh(source), source));
    }
    assert_eq!(v(&old.attempts), v(&loaded.attempts));
    assert_eq!(v(&old.favorites), v(&loaded.favorites));
}
#[test]
fn all_new_lvm_sources_require_authored_save_context_even_when_current_and_initial_are_jointly_forged()
 {
    let all = pack::bundled().unwrap();
    let s = &all[182];
    let peer = &all[183];
    let mut good = Progress {
        schema_version: 1,
        ..Default::default()
    };
    good.attempts.insert(s.id.clone(), fresh(s));
    good.attempts.insert(peer.id.clone(), fresh(peer));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    let caller = v(&good);
    let caller_receipt = v(&good.migration_receipts);
    progress::save_with_catalog(&path, &good, &all).unwrap();
    assert_eq!(v(&good), caller);
    assert_eq!(v(&good.migration_receipts), caller_receipt);
    let raw = std::fs::read(&path).unwrap();
    let observed = progress::decode(&raw, &all).unwrap();
    assert!(observed.migration_receipts.is_some());
    for mode in 0..4 {
        let mut p: Progress = serde_json::from_value(v(&good)).unwrap();
        let mut sources = all.clone();
        match mode {
            0 => {
                let State::Lvm { simulation } = &mut p.attempts.get_mut(&peer.id).unwrap().state
                else {
                    panic!()
                };
                for model in [&mut simulation.initial, &mut simulation.current] {
                    // Preserve a valid sector-aligned model; test source authority,
                    // rather than failing earlier on malformed device geometry.
                    model.devices.values_mut().next().unwrap().bytes += 512;
                    model.validate().unwrap();
                }
                simulation.validate().unwrap();
            }
            1 => sources.retain(|s| s.id != peer.id),
            2 => {
                let mut foreign = s.clone();
                foreign.id = peer.id.clone();
                sources[183] = foreign;
            }
            _ => p.attempts.get_mut(&peer.id).unwrap().state = all[0].initial.clone(),
        }
        let before = v(&p);
        let receipt = v(&p.migration_receipts);
        assert!(progress::save_with_catalog(&path, &p, &sources).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        assert_eq!(v(&p), before);
        assert_eq!(v(&p.migration_receipts), receipt);
    }
    assert!(progress::save(&path, &good).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw);
}
#[test]
fn copied_example_names_display_text_and_foreign_oneoff_metadata_cannot_satisfy_lvm() {
    let all = pack::bundled().unwrap();
    for s in &all[182..] {
        let Goal::Lvm { definition } = &s.goal else {
            panic!()
        };
        let mut a = fresh(s);
        let before = v(&a);
        let State::Lvm { simulation } = &mut a.state else {
            panic!()
        };
        let wrong = definition
            .lesson
            .worked
            .iter()
            .find(|c| !["pvs", "vgs", "lvs"].contains(&c.text.as_str()))
            .unwrap();
        assert!(simulation.execute(definition, &wrong.text, false).is_err());
        assert_eq!(v(&a), before);
        let State::Lvm { simulation } = &mut a.state else {
            panic!()
        };
        simulation.checked = true;
        simulation.transcript.push(work::Row {
            input: "lvs".into(),
            output: "all checks pass; filesystem is large".into(),
        });
        assert!(!progress::current_solved(&a, s));
        let source = all
            .iter()
            .find(|s| matches!(s.initial, State::OneOff { .. }))
            .unwrap();
        let mut own = Attempt::new(source, 1);
        dlms_learning_lab::one_off_workspace::budget::prepare(&mut own, source).unwrap();
        a.workspace.as_mut().unwrap().one_off_session =
            own.workspace.as_mut().unwrap().one_off_session.take();
        assert!(work::context(&a, s).is_err());
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn joint_forgery_cannot_supply_authority_to_load_export_inspect_restore_or_completion() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let all = pack::bundled().unwrap();
    let s = &all[182];
    let mut own = fresh(s);
    for command in &s.solution {
        engine::execute_for_goal(&mut own.state, &s.goal, command).unwrap();
    }
    assert!(progress::current_solved(&own, s));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), own);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let portable = dir.path().join("good.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let raw = std::fs::read(&path).unwrap();
    backup::export(&portable, &p, &all).unwrap();
    let cancel = AtomicBool::new(false);
    let checked = AtomicUsize::new(0);
    let good = backup::inspect(&portable, &all, &cancel, &checked).unwrap();
    assert!(progress::current_solved(&good.progress.attempts[&s.id], s));
    let State::Lvm { simulation } = &mut p.attempts.get_mut(&s.id).unwrap().state else {
        panic!()
    };
    // Both self-carried markers agree; only the app's authored source is authority.
    for model in [&mut simulation.initial, &mut simulation.current] {
        model.devices.values_mut().next().unwrap().bytes += 512;
        model.validate().unwrap();
    }
    simulation.validate().unwrap();
    let before = v(&p);
    let badraw = serde_json::to_vec(&p).unwrap();
    assert!(!progress::current_solved(&p.attempts[&s.id], s));
    assert!(progress::decode(&badraw, &all).is_err());
    let badprofile = dir.path().join("bad-profile.json");
    std::fs::write(&badprofile, &badraw).unwrap();
    assert!(progress::load(&badprofile, &all).is_err());
    assert_eq!(std::fs::read(&badprofile).unwrap(), badraw);
    let badexport = dir.path().join("bad-export.json");
    assert!(backup::export(&badexport, &p, &all).is_err());
    assert!(!badexport.exists());
    let mut envelope: Value = serde_json::from_slice(&std::fs::read(&portable).unwrap()).unwrap();
    envelope["progress"] = v(&p);
    let forged = dir.path().join("forged-portable.json");
    let forgedraw = serde_json::to_vec(&envelope).unwrap();
    std::fs::write(&forged, &forgedraw).unwrap();
    assert!(backup::inspect(&forged, &all, &cancel, &checked).is_err());
    assert_eq!(std::fs::read(&forged).unwrap(), forgedraw);
    let candidate = backup::Candidate {
        progress: p,
        rechecked: 0,
    };
    assert!(backup::restore(&path, &candidate, &all).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw);
    assert_eq!(v(&candidate.progress), before);
    assert!(!dir.path().join("progress-before-restore-1.json").exists());
    let valid = progress::load(&path, &all).unwrap();
    assert!(progress::current_solved(&valid.attempts[&s.id], s));
}
