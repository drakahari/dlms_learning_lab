//! Prospective normal NTP190 closure and LVM/NTP source-authority composition.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::{Value, json};
#[path = "support/combined_catalog.rs"]
mod combined;
use dlms_learning_lab::developer_digest as digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use ere::original190::before_ntp as original;
use lvm_registration::ntp_registration as registration;
use one_off_boundary::loop_registration::lvm_registration;
use registration::ere_registration as ere;
fn value<T: serde::Serialize + ?Sized>(a: &T) -> Value {
    serde_json::to_value(a).unwrap()
}
fn prepared(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    a
}
#[test]
fn all186_complete_sources_49_units_and57scripts_remain_before_four_normal_ntp_tasks() {
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 193);
    let all = ere::before_scenarios(&current).to_vec();
    let old = original::load();
    assert_eq!((all.len(), old.len()), (190, 186));
    assert_eq!(value(&all[..186]), value(&old));
    assert_eq!(value(registration::before_scenarios(&all)), value(&old));
    let authored: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/ntp-client-server-v1.json")).unwrap();
    assert_eq!(value(&all[186..]), value(&authored.scenarios));
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| old.iter().filter(|s| s.track == t).count()),
        [53, 40, 36, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 40, 36, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [57, 40, 36, 60]
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
    for s in &all[186..] {
        assert_eq!(s.unit, "ntp-client-server");
        assert!(matches!(s.initial, State::Ntp { .. }) && matches!(s.goal, Goal::Ntp { .. }));
        assert!(!progress::current_solved(&prepared(s), s));
    }
}
#[test]
fn strict50_to49_full_raw_typed_inverse_composes_all_older_proofs_and_rejects_mutations() {
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 193);
    let all = ere::before_scenarios(&current).to_vec();
    assert_eq!(all.len(), 190);
    let old = original::load();
    let current_raw = include_bytes!("../scenarios/units-v1.json");
    let checked50 = ere::before_units(current_raw).unwrap();
    let raw = checked50.as_slice();
    let before = registration::before_units(raw).unwrap();
    assert_eq!(before, include_bytes!("fixtures/ntp-units-before186.json"));
    let r: Value =
        serde_json::from_str(include_str!("fixtures/catalog190-registration.json")).unwrap();
    assert_eq!(digest::hash(raw), r["after_units50_sha256"]);
    assert_eq!(digest::hash(&before), r["before_units49_sha256"]);
    let current_units = course::bundled(&current).unwrap();
    assert_eq!(current_units.len(), 51);
    let units = ere::before_typed_units(&current_units);
    let previous = course::load(std::str::from_utf8(&before).unwrap(), &old).unwrap();
    assert_eq!((units.len(), previous.len()), (50, 49));
    assert_eq!(value(&units[..49]), value(&previous));
    assert_eq!(value(&units[49]), r["ntp_unit"]);
    assert_eq!(units[49].labs, registration::IDS);
    assert_eq!(units[49].prerequisites, ["ipv4-foundations"]);
    assert_eq!(
        value(registration::before_typed_units(units)),
        value(&previous)
    );
    assert_eq!(
        combined::old_unit_bytes(raw).unwrap(),
        combined::old_unit_bytes(&before).unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&combined::old_unit_bytes(raw).unwrap()).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
    assert!(registration::before_units(&before).is_err());
    let mut whitespace = raw.to_vec();
    whitespace.push(b' ');
    assert!(registration::before_units(&whitespace).is_err());
    for field in [
        "title",
        "summary",
        "prerequisite",
        "lab",
        "old-unit",
        "extra",
    ] {
        let mut bad: Value = serde_json::from_slice(raw).unwrap();
        match field {
            "title" => bad["units"][49]["title"] = json!("changed"),
            "summary" => bad["units"][49]["summary"] = json!("changed"),
            "prerequisite" => bad["units"][49]["prerequisites"] = json!([]),
            "lab" => bad["units"][49]["labs"][0] = json!("ccna-ipv4"),
            "old-unit" => bad["units"][0]["summary"] = json!("changed"),
            _ => {
                let extra = bad["units"][49].clone();
                bad["units"].as_array_mut().unwrap().push(extra);
            }
        }
        assert!(registration::before_units(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert!(
            std::panic::catch_unwind(|| registration::before_values(
                bad["units"].as_array().unwrap()
            ))
            .is_err()
        );
    }
}
#[test]
fn source_inverse_never_masks_partial_extra_or_modified_new_tasks() {
    let current = pack::bundled().unwrap();
    assert_eq!(current.len(), 193);
    let all = ere::before_scenarios(&current).to_vec();
    for mode in 0..6 {
        let mut bad = all.clone();
        match mode {
            0 => {
                bad.pop().unwrap();
            }
            1 => bad.push(all[186].clone()),
            2 => bad[186].objective.push_str(" changed"),
            3 => {
                let Goal::Ntp { definition } = &mut bad[186].goal else {
                    panic!()
                };
                definition.targets[0].allowed_references.pop();
            }
            4 => {
                let Goal::Ntp { definition } = &mut bad[186].goal else {
                    panic!()
                };
                definition.lesson.worked[0].explanation.push_str(" changed");
            }
            _ => bad[182].version += 1,
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
fn retained_old_profiles_have_no_new_ntp_or_lvm_answers_favorites_or_completion() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let original: Progress = serde_json::from_slice(raw).unwrap();
    let loaded = progress::decode(raw, &all).unwrap();
    assert_eq!(value(&loaded.attempts), value(&original.attempts));
    assert_eq!(value(&loaded.favorites), value(&original.favorites));
    for id in registration::IDS.into_iter().chain(lvm_registration::IDS) {
        assert!(!loaded.attempts.contains_key(id) && !loaded.favorites.contains(id));
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert!(!progress::current_solved(&prepared(s), s));
    }
}
#[test]
fn shared_normal_save_checks_both_families_and_every_nonselected_attempt_before_writing() {
    let all = pack::bundled().unwrap();
    let mut good = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all[182..] {
        good.attempts.insert(s.id.clone(), prepared(s));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    let caller = value(&good);
    progress::save_with_catalog(&path, &good, &all).unwrap();
    assert_eq!(value(&good), caller);
    let raw = std::fs::read(&path).unwrap();
    let loaded = progress::decode(&raw, &all).unwrap();
    assert!(loaded.migration_receipts.is_some());
    for index in [182, 183, 184, 185, 186, 187, 188, 189] {
        for mode in 0..4 {
            let mut p: Progress = serde_json::from_value(value(&good)).unwrap();
            let mut sources = all.clone();
            let id = &all[index].id;
            match mode {
                0 => {
                    let a = p.attempts.get_mut(id).unwrap();
                    match &mut a.state {
                        State::Lvm { simulation } => {
                            for s in [&mut simulation.initial, &mut simulation.current] {
                                s.devices.values_mut().next().unwrap().bytes += 512;
                                s.validate().unwrap();
                            }
                            simulation.validate().unwrap();
                        }
                        State::Ntp { simulation } => {
                            for s in [&mut simulation.initial, &mut simulation.current] {
                                let node = s
                                    .nodes
                                    .values_mut()
                                    .find(|n| {
                                        matches!(
                                            n.role,
                                            dlms_learning_lab::ntp::Role::Reference { .. }
                                        )
                                    })
                                    .unwrap();
                                let dlms_learning_lab::ntp::Role::Reference { healthy, .. } =
                                    &mut node.role
                                else {
                                    panic!()
                                };
                                *healthy = !*healthy;
                            }
                            simulation.validate().unwrap();
                        }
                        _ => panic!(),
                    }
                }
                1 => sources.retain(|s| &s.id != id),
                2 => p.attempts.get_mut(id).unwrap().state = all[0].initial.clone(),
                _ => {
                    p.attempts
                        .get_mut(id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .command = "foreign generic command".into()
                }
            }
            let before = value(&p);
            assert!(progress::save_with_catalog(&path, &p, &sources).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), raw);
            assert_eq!(value(&p), before);
            assert!(!progress::current_solved(&p.attempts[id], &all[index]));
        }
    }
    assert!(progress::save(&path, &good).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), raw);
}
#[test]
fn one_family_drafts_and_retry_never_execute_or_replace_the_other_familys_world() {
    let all = pack::bundled().unwrap();
    let l = &all[182];
    let n = &all[186];
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(l.id.clone(), prepared(l));
    p.attempts.insert(n.id.clone(), prepared(n));
    let keep = value(&p.attempts[&l.id]);
    let Goal::Ntp { definition } = &n.goal else {
        panic!()
    };
    let State::Ntp { simulation } = &mut p.attempts.get_mut(&n.id).unwrap().state else {
        panic!()
    };
    let initial = simulation.current.clone();
    simulation
        .set_command(definition, "show ntp status".into(), false)
        .unwrap();
    simulation
        .set_command(definition, "show ntp associations".into(), true)
        .unwrap();
    assert_eq!(simulation.current, initial);
    let ntp = value(&p.attempts[&n.id]);
    assert_eq!(value(&p.attempts[&l.id]), keep);
    let Goal::Lvm { definition } = &l.goal else {
        panic!()
    };
    let State::Lvm { simulation } = &mut p.attempts.get_mut(&l.id).unwrap().state else {
        panic!()
    };
    let initial = simulation.current.clone();
    simulation
        .set_command(definition, "unfinished LVM assignment |".into(), false)
        .unwrap();
    simulation
        .set_command(definition, "separate LVM practice draft |".into(), true)
        .unwrap();
    assert_eq!(simulation.current, initial);
    assert_eq!(value(&p.attempts[&n.id]), ntp);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let again = progress::load(&path, &all).unwrap();
    assert_eq!(value(&again.attempts), value(&p.attempts));
    assert!(!progress::current_solved(&again.attempts[&n.id], n));
    assert!(!progress::current_solved(&again.attempts[&l.id], l));
}
#[test]
fn current_oneoff_source_and_saved_metadata_remain_foreign_to_each_new_typed_family() {
    let all = pack::bundled().unwrap();
    let one = all
        .iter()
        .find(|s| matches!(s.initial, State::OneOff { .. }))
        .unwrap();
    let mut oneoff = Attempt::new(one, 1);
    dlms_learning_lab::one_off_workspace::budget::prepare(&mut oneoff, one).unwrap();
    let w = oneoff.workspace.as_ref().unwrap();
    assert!(w.one_off_session.is_some() && w.one_off_learning.is_some());
    for index in [182, 186] {
        let source = &all[index];
        let a = prepared(source);
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(source.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profile.json");
        progress::save_with_catalog(&path, &p, &all).unwrap();
        let raw = std::fs::read(&path).unwrap();
        for field in 0..2 {
            let mut bad: Progress = serde_json::from_value(value(&p)).unwrap();
            let a = bad.attempts.get_mut(&source.id).unwrap();
            if field == 0 {
                a.workspace.as_mut().unwrap().one_off_session = w.one_off_session.clone();
            } else {
                a.workspace.as_mut().unwrap().one_off_learning = w.one_off_learning.clone();
            }
            let keep = value(a);
            workspace::prepare(a);
            assert_eq!(value(a), keep);
            assert!(!progress::current_solved(a, source));
            let before = value(&bad);
            assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
            assert!(progress::save_with_catalog(&path, &bad, &all).is_err());
            assert_eq!(std::fs::read(&path).unwrap(), raw);
            assert_eq!(value(&bad), before);
        }
        let mut foreign = source.clone();
        foreign.one_off_teaching = one.one_off_teaching.clone();
        assert!(foreign.one_off_teaching.is_some());
        match source.initial {
            State::Lvm { .. } => {
                assert!(dlms_learning_lab::lvm_workspace::source(&foreign).is_err())
            }
            State::Ntp { .. } => {
                assert!(dlms_learning_lab::ntp_workspace::source(&foreign).is_err())
            }
            _ => panic!(),
        }
    }
}
