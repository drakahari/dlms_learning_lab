//! Prospective normal OneOff registration; no native/installed acceptance claim.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    one_off_workspace as work, pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
};
use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as disk_registration;
#[path = "support/combined_catalog.rs"]
mod combined;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "support/one_off_registration.rs"]
mod registration;
fn value<T: serde::Serialize + ?Sized>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap()
}
#[test]
fn full176_prefix_all_old_scripts_examples_cases_and_exact_new_four_remain_distinct() {
    let all = pack::bundled().unwrap();
    let old = registration::original_catalog(&all);
    assert_eq!(old.len(), 176);
    assert_eq!(
        one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(
            &all
        )
        .len(),
        186
    );
    assert_eq!(one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&all).len(),190);
    assert_eq!(disk_registration::before_scenarios(&all).len(), 193);
    assert_eq!(all.len(), 197);
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
        one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(
            &all
        )
        .len(),
        186
    );
    assert_eq!(one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&all).len(),190);
    assert_eq!(disk_registration::before_scenarios(&all).len(), 193);
    assert_eq!(all.len(), 197);
    assert_eq!(["CCNA","LPIC-1","LPIC-2","Bash/Shell"].map(|t|one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_scenarios(&all).iter().filter(|s|s.track==t).count()),[53,40,36,57]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|t| disk_registration::before_scenarios(
            &all
        )
        .iter()
        .filter(|s| s.track == t)
        .count()),
        [57, 40, 36, 60]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [57, 44, 36, 60]
    );
    let scripts = one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_scenarios(&all)
        .iter()
        .filter(|s| matches!(s.goal, Goal::Script { .. }))
        .collect::<Vec<_>>();
    assert_eq!(scripts.len(), 57);
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
    assert_eq!(scripts.iter().filter(|s| s.teaching.is_some()).count(), 57);
    assert_eq!(dlms_learning_lab::shell::EVALUATOR_REVISION, 15);
    for s in &all[176..180] {
        assert_eq!(s.version, 1);
        assert_eq!(s.unit, "linux-one-off-jobs");
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for input in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn exact47_to46_inverse_keeps_both_network_appends_and_every_old37_byte() {
    let all = pack::bundled().unwrap();
    let old = registration::original_catalog(&all);
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    let current_units = course::bundled(&all).unwrap();
    assert_eq!(
        disk_registration::before_typed_units(&current_units).len(),
        51
    );
    assert_eq!(current_units.len(), 52);
    let units = one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::before_typed_units(&current_units).to_vec();
    let old_units = course::load(std::str::from_utf8(&before).unwrap(), &old).unwrap();
    assert_eq!(
        (
            one_off_boundary::loop_registration::lvm_registration::before_typed_units(&units).len(),
            old_units.len()
        ),
        (48, 46)
    );
    assert_eq!((one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_typed_units(&units).len(),old_units.len()),(49,46));
    assert_eq!((units.len(), old_units.len()), (50, 46));
    assert_eq!(value(&units[..46]), value(&old_units));
    assert_eq!(units[46].id, "linux-one-off-jobs");
    assert_eq!(units[46].labs, registration::IDS.map(str::to_owned));
    assert_eq!(
        units[46].prerequisites,
        ["linux-file-foundations", "shell-foundations"]
    );
    let checked47 = one_off_boundary::loop_registration::before_typed_units(&units);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| checked47.iter().filter(|u| u.track == t).count()),
        [14, 10, 8, 15]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|t| {
            one_off_boundary::loop_registration::lvm_registration::before_typed_units(&units)
                .iter()
                .filter(|u| u.track == t)
                .count()
        }),
        [14, 10, 8, 16]
    );
    assert_eq!(["CCNA","LPIC-1","LPIC-2","Bash/Shell"].map(|t|one_off_boundary::loop_registration::lvm_registration::ntp_registration::before_typed_units(&units).iter().filter(|u|u.track==t).count()),[14,10,9,16]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [15, 10, 9, 16]
    );
    let before44 = combined::before_physical_units(&before).unwrap();
    let before37 = combined::old_unit_bytes(&before).unwrap();
    assert_eq!(before37, combined::old_unit_bytes(&before44).unwrap());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&before37).unwrap()["units"]
            .as_array()
            .unwrap()
            .len(),
        37
    );
}
#[test]
fn changed_raw_or_typed_appends_wrong_prerequisites_and_duplicate_units_are_refused() {
    let all = pack::bundled().unwrap();
    let raw = include_bytes!("../scenarios/units-v1.json");
    let before = registration::before_units(raw).unwrap();
    assert!(registration::before_units(&before).is_err());
    let mut extra = raw.to_vec();
    extra.push(b' ');
    assert!(registration::before_units(&extra).is_err());
    for mutation in 0..4 {
        let mut v: serde_json::Value = serde_json::from_slice(raw).unwrap();
        match mutation {
            0 => v["units"][0]["summary"] = "unauthorized predecessor edit".into(),
            1 => v["units"][46]["labs"][0] = "ccna-private-office-plan".into(),
            2 => v["units"][46]["prerequisites"][0] = "not-a-real-unit".into(),
            _ => v["units"][46]["id"] = "physical-uplink-diagnosis".into(),
        };
        let changed = serde_json::to_vec(&v).unwrap();
        assert!(registration::before_units(&changed).is_err());
        if mutation != 0 {
            assert!(course::load(std::str::from_utf8(&changed).unwrap(), &all).is_err());
        }
    }
}
#[test]
fn old_profile_never_invents_one_off_answers_and_fresh_drafts_resume_reset_only_selected() {
    let all = pack::bundled().unwrap();
    let original = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let decoded = progress::decode(original, &all).unwrap();
    for id in registration::IDS {
        assert!(!decoded.attempts.contains_key(id));
        assert!(!decoded.favorites.contains(id));
    }
    let mut p = Progress {
        schema_version: 1,
        clock: 91,
        ..Default::default()
    };
    for (n, s) in all[176..180].iter().enumerate() {
        let mut a = Attempt::new(s, (n + 1) as u64);
        work::budget::prepare(&mut a, s).unwrap();
        let Goal::OneOff { definition } = &s.goal else {
            panic!()
        };
        work::budget::apply_attempt(
            &mut a,
            s,
            &work::Action::Edit {
                path: definition.editable[0].clone(),
                text: format!("# unfinished assignment {n}\n"),
            },
            false,
        )
        .unwrap();
        let lesson = s.one_off_teaching.as_ref().unwrap();
        work::budget::apply_attempt(
            &mut a,
            s,
            &work::Action::Edit {
                path: lesson.definition.editable[0].clone(),
                text: format!("# independent example {n}\n"),
            },
            true,
        )
        .unwrap();
        work::budget::set_view(&mut a, s, View::Assignment, 0).unwrap();
        assert!(!progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("registration-progress.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::decode(&std::fs::read(&path).unwrap(), &all).unwrap();
    assert_eq!(value(&restored), value(&p));
    assert!(restored.migration_receipts.is_some());
    let receipts = value(&restored.migration_receipts);
    let before = value(&restored);
    let s = &all[176];
    let visited = restored.attempts[&s.id].visited;
    work::budget::reset_attempt(restored.attempts.get_mut(&s.id).unwrap(), s).unwrap();
    assert_eq!(restored.clock, 91);
    assert_eq!(value(&restored.favorites), before["favorites"]);
    assert_eq!(value(&restored.migration_receipts), receipts);
    assert_eq!(restored.attempts[&s.id].visited, visited);
    assert_eq!(restored.attempts[&s.id].state, s.initial);
    for source in &all[177..180] {
        assert_eq!(
            value(&restored.attempts[&source.id]),
            before["attempts"][&source.id]
        );
    }
    // The newly registered Loop tasks were absent from this retained old profile
    // and from the four edited OneOff attempts. Never fabricate their state.
    assert_eq!(
        all[180..182]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ["shell-loop-intake", "shell-loop-fallback"]
    );
    for source in &all[180..182] {
        assert!(
            matches!(source.initial, State::Script { run: None, .. })
                && matches!(source.goal, Goal::Script { .. })
        );
        assert!(
            !decoded.attempts.contains_key(&source.id) && !decoded.favorites.contains(&source.id)
        );
        assert!(!p.attempts.contains_key(&source.id) && !p.favorites.contains(&source.id));
        assert!(
            !restored.attempts.contains_key(&source.id) && !restored.favorites.contains(&source.id)
        );
        assert!(!progress::current_solved(&Attempt::new(source, 1), source));
    }
    for source in &all[182..186] {
        assert!(
            matches!(source.initial, State::Lvm { .. }) && matches!(source.goal, Goal::Lvm { .. })
        );
        assert!(
            !decoded.attempts.contains_key(&source.id) && !decoded.favorites.contains(&source.id)
        );
        assert!(
            !p.attempts.contains_key(&source.id) && !restored.attempts.contains_key(&source.id)
        );
        assert!(!p.favorites.contains(&source.id) && !restored.favorites.contains(&source.id));
        assert!(!progress::current_solved(&Attempt::new(source, 1), source));
    }
    for source in &all[186..190] {
        assert!(
            matches!(source.initial, State::Ntp { .. }) && matches!(source.goal, Goal::Ntp { .. })
        );
        assert!(
            !decoded.attempts.contains_key(&source.id) && !decoded.favorites.contains(&source.id)
        );
        assert!(
            !p.attempts.contains_key(&source.id) && !restored.attempts.contains_key(&source.id)
        );
        assert!(!p.favorites.contains(&source.id) && !restored.favorites.contains(&source.id));
        assert!(!progress::current_solved(&Attempt::new(source, 1), source));
    }
    for source in &all[190..193] {
        assert!(
            matches!(source.initial, State::Script { .. })
                && matches!(source.goal, Goal::Script { .. })
        );
        for profile in [&decoded, &p, &restored] {
            assert!(!profile.attempts.contains_key(&source.id));
            assert!(!profile.favorites.contains(&source.id));
        }
        assert!(!progress::current_solved(&Attempt::new(source, 1), source));
    }
    assert!(matches!(
        restored.attempts[&s.id].state,
        State::OneOff { .. }
    ));
}

#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[test]
fn closed_current_count_boundary_preserves176_and46_only_after_exact_append_checks() {
    let all = pack::bundled().unwrap();
    let units = course::bundled(&all).unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    assert_eq!(one_off_boundary::before_len(&units), 46);
    let mut changed = all.clone();
    changed[176].summary.push_str(" misleading altered task");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            one_off_boundary::before_len(&changed)
        }))
        .is_err()
    );
    let mut extra = all.clone();
    extra.push(all[176].clone());
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            one_off_boundary::before_len(&extra)
        }))
        .is_err()
    );
    let mut changed = units.clone();
    changed[0].summary.push_str(" changed original");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            one_off_boundary::before_len(&changed)
        }))
        .is_err()
    );
    changed = units.clone();
    changed[46].prerequisites.clear();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            one_off_boundary::before_len(&changed)
        }))
        .is_err()
    );
    let source = value(&all);
    let index = value(&units);
    for _ in 0..20 {
        assert_eq!(one_off_boundary::before_len(&all), 176);
        assert_eq!(one_off_boundary::before_len(&units), 46);
    }
    assert_eq!(value(&all), source);
    assert_eq!(value(&units), index);
}
