use one_off_boundary::loop_registration::lvm_registration::ntp_registration::ere_registration::partition_fs_registration as disk_registration;
#[path = "support/combined_catalog.rs"]
mod combined_catalog;
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/one_off_registration.rs"]
mod one_off_registration;
use dlms_learning_lab::{
    engine::{self, State},
    nfs as core, nfs_teaching as teach, nfs_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
use serde_json::json;
fn scenario(index: usize) -> pack::Scenario {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nfs-workbench-v1.json")).unwrap();
    let c = &raw["cases"][index];
    let solution = c["reference"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| format!("{}{}", work::PREFIX, serde_json::to_string(a).unwrap()))
        .collect::<Vec<_>>();
    let v = json!({"schema_version":1,"scenarios":[{"id":format!("nfs-technical-{index}"),"version":1,"track":"LPIC-2","topic":"Virtual NFS","unit":"","title":"Technical NFS adapter","summary":"Inspect supplied NFS layers","objective":"Complete actual current and new-mount outcomes","retry_guidance":"Inspect saved, applied and live values","difficulty":"Intermediate","minutes":15,"steps":["Inspect","Edit","Check"],"guide":[],"explanation":"Virtual existing files only","hints":["Save differs from Apply"],"sources":["https://www.rfc-editor.org/rfc/rfc1813.html"],"simplifications":"Supplied virtual NFSv3 ACCESS-style model; no host mount or RPC","initial":{"kind":"Nfs","simulation":c["definition"]["initial"]},"goal":{"kind":"Nfs","definition":c["definition"]},"solution":solution,"prerequisites":[],"nfs_teaching":c["lesson"]}]});
    pack::load(&v.to_string()).unwrap().remove(0)
}
fn exact<T: serde::Serialize>(a: &T, b: &serde_json::Value) {
    assert!(
        serde_json::to_value(a).unwrap() == *b,
        "complete caller differs"
    );
}
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    teach::prepare(&mut a, s, true).unwrap();
    for command in &s.solution {
        let action = core::Action::from_json(command.strip_prefix(work::PREFIX).unwrap()).unwrap();
        work::perform_attempt(&mut a, s, &action).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn all_four_loaded_references_current_invalidation_and_raw_request_drafts() {
    for n in 0..4 {
        let s = scenario(n);
        let mut a = solved(&s);
        let before = a.state.clone();
        let w = a.workspace.as_mut().unwrap();
        let session = w.nfs_session.as_mut().unwrap();
        session.relative = "../unfinished".into();
        session.write_text = "unsent replacement".into();
        assert!(progress::current_solved(&a, &s));
        assert_eq!(a.state, before);
        let dlms_learning_lab::engine::Goal::Nfs { definition } = &s.goal else {
            panic!()
        };
        let (client, specification) = definition
            .requirements
            .iter()
            .find_map(|r| match r {
                work::Requirement::CurrentMount {
                    client,
                    specification,
                } => Some((client, specification)),
                _ => None,
            })
            .unwrap();
        let action = core::Action::Unmount {
            client: client.clone(),
            mountpoint: specification.mountpoint.clone(),
        };
        work::perform_attempt(&mut a, &s, &action).unwrap();
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn full_source_version_kind_initial_and_lesson_pins_reject_before_mutation() {
    let s = scenario(0);
    let a = solved(&s);
    let mut wrong = s.clone();
    wrong.version += 1;
    assert!(!progress::current_solved(&a, &wrong));
    let mut attempt = a.clone();
    let before = serde_json::to_value(&attempt).unwrap();
    assert!(teach::prepare(&mut attempt, &wrong, false).is_err());
    exact(&attempt, &before);
    let mut wrong = s.clone();
    let other = scenario(1);
    wrong.nfs_teaching = other.nfs_teaching;
    assert!(!progress::current_solved(&a, &wrong));
    assert!(teach::prepare(&mut attempt, &wrong, false).is_err());
    exact(&attempt, &before);
    let mut wrong = s.clone();
    if let State::Nfs { simulation } = &mut wrong.initial {
        let mut inventory = simulation.inventory().clone();
        inventory.profiles[0].uid += 1;
        *simulation = core::State::new(inventory, simulation.runtime().clone()).unwrap();
    }
    assert!(!progress::current_solved(&a, &wrong));
    assert!(teach::prepare(&mut attempt, &wrong, false).is_err());
    exact(&attempt, &before);
    assert!(engine::execute(&mut attempt.state, &s.solution[0]).is_err());
    exact(&attempt, &before);
    assert!(
        engine::execute_for_goal(
            &mut attempt.state,
            &s.goal,
            &" ".repeat(work::PREFIX.len() + core::MAX_ACTION + 1)
        )
        .is_err()
    );
    exact(&attempt, &before);
}
#[test]
fn old147_wire_none_fields_and_existing_solved_nfs_without_metadata_remain_truthful() {
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_registration::original_catalog(&all).len(), 176);
    let raw46 =
        one_off_registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let units =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw46).unwrap(), &all[..176]).unwrap();
    assert_eq!(units.len(), 46);
    let raw44 = original44_unit_bytes();
    let original44 =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw44).unwrap(), &all[..171]).unwrap();
    assert_eq!(original44.len(), 44);
    assert_eq!(
        serde_json::to_value(&units[..44]).unwrap(),
        serde_json::to_value(&original44).unwrap()
    );
    assert_eq!(original44.last().unwrap().id, "shell-numeric-validation");
    for s in &all[..147] {
        assert!(s.nfs_teaching.is_none());
        assert!(
            !serde_json::to_string(s)
                .unwrap()
                .contains("\"nfs_teaching\"")
        );
        let a = Attempt::new(s, 1);
        assert!(
            !serde_json::to_string(&a)
                .unwrap()
                .contains("\"nfs_session\"")
        );
    }
    assert_eq!(
        all[147..150]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-command-options",
            "shell-command-dispatch",
            "shell-command-plan",
        ]
    );
    for s in &all[147..150] {
        assert!(matches!(s.initial, State::Script { .. }));
        assert!(matches!(s.goal, engine::Goal::Script { .. }));
        assert!(s.nfs_teaching.is_none());
    }
    const IDS: [&str; 4] = [
        "lpic2-nfs-reference",
        "lpic2-nfs-publication",
        "lpic2-nfs-handover",
        "lpic2-nfs-capstone",
    ];
    assert_eq!(
        all[150..154]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    for s in &all[150..154] {
        assert!(matches!(s.initial, State::Nfs { .. }));
        assert!(matches!(s.goal, engine::Goal::Nfs { .. }));
        assert!(s.nfs_teaching.is_some());
        let mut fresh = Attempt::new(s, 1);
        teach::prepare(&mut fresh, s, true).unwrap();
        assert_eq!(fresh.state, s.initial);
        let w = fresh.workspace.as_ref().unwrap();
        assert_eq!(*w.nfs_session.as_ref().unwrap(), work::Session::default());
        let l = w.nfs_learning.as_ref().unwrap();
        assert_eq!(
            l.view,
            dlms_learning_lab::script_teaching::View::Introduction
        );
        assert_eq!(l.session, work::Session::default());
        assert!(!progress::current_solved(&fresh, s));
    }
    let s = scenario(0);
    let mut a = solved(&s);
    a.workspace = None;
    assert!(progress::current_solved(&a, &s));
    let state = a.state.clone();
    teach::prepare(&mut a, &s, false).unwrap();
    assert_eq!(a.state, state);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .nfs_learning
            .as_ref()
            .unwrap()
            .view,
        dlms_learning_lab::script_teaching::View::Assignment
    );
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn actual_save_readonly_load_export_inspect_restore_preserves_all_records_before_reset() {
    use dlms_learning_lab::backup;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let raw = include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json");
    let full201=pack::bundled().unwrap();
    let current199=disk_registration::literal_registration::snapshot_registration::before_scenarios(&full201).to_vec();
    assert_eq!((full201.len(),current199.len()),(201,199));
    let full54=dlms_learning_lab::course::bundled(&full201).unwrap();
    let snapshot_raw53=disk_registration::literal_registration::snapshot_registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    assert_eq!(snapshot_raw53.as_slice(),include_bytes!("fixtures/snapshot-units-before199.json"));
    let checked53=dlms_learning_lab::course::load(std::str::from_utf8(&snapshot_raw53).unwrap(),&current199).unwrap();
    assert_eq!((full54.len(),checked53.len()),(54,53));
    assert_eq!(serde_json::to_value(disk_registration::literal_registration::snapshot_registration::before_typed_units(&full54)).unwrap(),serde_json::to_value(&checked53).unwrap());
    let previous197 = disk_registration::literal_registration::before_scenarios(&current199);
    assert_eq!(current199.len(), 199);
    let mut all = previous197.to_vec();
    assert_eq!(disk_registration::before_scenarios(&all).len(), 193);
    assert_eq!(all.len(), 197);
    let s = scenario(0);
    all.push(s.clone());
    let mut p = progress::decode(raw, &all).unwrap();
    // Missing later activities get only their normal initial Attempt, never a run/history.
    for source in all.iter().filter(|source| source.id != s.id) {
        p.attempts
            .entry(source.id.clone())
            .or_insert_with(|| Attempt::new(source, 7));
    }
    assert_eq!(disk_registration::before_attempts_len(&p.attempts), 193);
    assert_eq!(p.attempts.len(), 197);
    for id in disk_registration::IDS {
        let source = all.iter().find(|s| s.id == id).unwrap();
        let a = &p.attempts[id];
        assert!(matches!(a.state, State::PartitionFs { .. }));
        assert_eq!(a.state, source.initial);
        assert!(a.transcript.is_empty());
        dlms_learning_lab::partition_fs_workspace::context(a, source).unwrap();
        assert!(!progress::current_solved(a, source));
    }
    for id in [
        "shell-ere-event-view",
        "shell-ere-inventory-review",
        "shell-ere-incident-report",
    ] {
        let source = all.iter().find(|s| s.id == id).unwrap();
        let a = &p.attempts[id];
        assert_eq!(a.state, source.initial);
        assert!(!progress::current_solved(a, source));
    }
    assert_eq!(
        p.attempts
            .keys()
            .filter(|id| !disk_registration::IDS.contains(&id.as_str())
                && ![
                    "shell-ere-event-view",
                    "shell-ere-inventory-review",
                    "shell-ere-incident-report"
                ]
                .contains(&id.as_str())
                && !one_off_registration::IDS.contains(&id.as_str())
                && !["shell-loop-intake", "shell-loop-fallback"].contains(&id.as_str())
                && ![
                    "ccna-ntp-client-chain",
                    "ccna-ntp-cycle-repair",
                    "ccna-ntp-asymmetric-delivery",
                    "ccna-ntp-policy-handover"
                ]
                .contains(&id.as_str())
                && ![
                    "lpic2-lvm-free-space",
                    "lpic2-lvm-add-pv",
                    "lpic2-lvm-filesystem-gap",
                    "lpic2-lvm-handover"
                ]
                .contains(&id.as_str()))
            .count(),
        176
    );
    for id in one_off_registration::IDS {
        let source = all.iter().find(|s| s.id == id).unwrap();
        let new = &p.attempts[id];
        assert_eq!(new.state, source.initial);
        assert_eq!(new.step, 0);
        assert!(new.transcript.is_empty());
        assert!(!progress::current_solved(new, source));
    }
    let old = serde_json::to_value(&p.attempts).unwrap();
    let mut a = solved(&s);
    let source = s.nfs_teaching.as_ref().unwrap();
    for step in &source.example_actions {
        work::perform_practice(&mut a, &s, &step.action).unwrap();
    }
    let w = a.workspace.as_mut().unwrap();
    w.nfs_session.as_mut().unwrap().relative = "/unfinished assigned query".into();
    let l = w.nfs_learning.as_mut().unwrap();
    l.session.relative = "../unfinished example query".into();
    l.session.write_text = "unsubmitted example replacement".into();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    p.clock = 91;
    let favorites = p.favorites.clone();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    std::fs::write(&path, raw).unwrap();
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(
        std::fs::read(path.with_extension(format!(
            "before-shell-v{}.json",
            dlms_learning_lab::shell::EVALUATOR_REVISION
        )))
        .unwrap(),
        raw
    );
    let saved = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    for (id, value) in old.as_object().unwrap() {
        exact(&loaded.attempts[id], value);
    }
    let selected = serde_json::to_value(&loaded.attempts[&s.id]).unwrap();
    exact(&p.attempts[&s.id], &selected);
    let export = dir.path().join("portable.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    exact(&candidate.progress.attempts[&s.id], &selected);
    // Independent old-only inspection accounts for the existing explicit shell rechecks.
    let mut old_only = progress::decode(&saved, &all).unwrap();
    old_only.attempts.remove(&s.id);
    old_only.favorites.remove(&s.id);
    let old_export = dir.path().join("old-only.json");
    backup::export(&old_export, &old_only, &all).unwrap();
    let control = backup::inspect(
        &old_export,
        &all,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let expected_old = serde_json::to_value(&control.progress.attempts).unwrap();
    for (id, value) in expected_old.as_object().unwrap() {
        exact(&candidate.progress.attempts[id], value);
    }
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(restored.favorites, favorites);
    assert_eq!(restored.clock, 91);
    exact(&restored.attempts[&s.id], &selected);
    for (id, value) in expected_old.as_object().unwrap() {
        exact(&restored.attempts[id], value);
    }
    let restored_before_reset = json!({"portable":restored,"receipts":restored.migration_receipts});
    restored.attempts.insert(s.id.clone(), Attempt::new(&s, 7));
    teach::prepare(restored.attempts.get_mut(&s.id).unwrap(), &s, true).unwrap();
    for (id, value) in expected_old.as_object().unwrap() {
        exact(&restored.attempts[id], value);
    }
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    assert!(restored.favorites.contains(&s.id));
    // A separate full-current roundtrip preserves the actual decoded/exported
    // history baseline; the two new Scripts remain absent rather than solved.
    let current53=dlms_learning_lab::course::load(std::str::from_utf8(&snapshot_raw53).unwrap(),&current199).unwrap();
    let raw52 = disk_registration::literal_registration::before_units(include_bytes!(
        "../scenarios/units-v1.json"
    ))
    .unwrap();
    assert_eq!(
        raw52.as_slice(),
        include_bytes!("fixtures/literal-units-before197.json")
    );
    let previous52 =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw52).unwrap(), previous197).unwrap();
    assert_eq!((current53.len(), previous52.len()), (53, 52));
    assert_eq!(
        serde_json::to_value(disk_registration::literal_registration::before_typed_units(
            &current53
        ))
        .unwrap(),
        serde_json::to_value(&previous52).unwrap()
    );
    let normal = serde_json::to_value(&current199).unwrap();
    let mut current200 = current199;
    current200.push(s.clone());
    assert_eq!(current200.len(), 200);
    assert_eq!(serde_json::to_value(&current200[..199]).unwrap(), normal);
    assert_eq!(
        serde_json::to_value(&current200[199]).unwrap(),
        serde_json::to_value(&s).unwrap()
    );
    let caller = json!({"portable":loaded,"receipts":loaded.migration_receipts});
    let current_path = dir.path().join("current-synthetic.json");
    std::fs::write(&current_path, &saved).unwrap();
    progress::save_with_catalog(&current_path, &loaded, &current200).unwrap();
    assert_eq!(
        json!({"portable":loaded,"receipts":loaded.migration_receipts}),
        caller
    );
    let current_saved = std::fs::read(&current_path).unwrap();
    let current_loaded = progress::load(&current_path, &current200).unwrap();
    assert_eq!(std::fs::read(&current_path).unwrap(), current_saved);
    assert_eq!(
        json!({"portable":current_loaded,"receipts":current_loaded.migration_receipts}),
        caller
    );
    let current_export = dir.path().join("current-portable.json");
    backup::export(&current_export, &current_loaded, &current200).unwrap();
    let current_candidate = backup::inspect(
        &current_export,
        &current200,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert_eq!(
        json!({"portable":current_candidate.progress,"receipts":current_candidate.progress.migration_receipts}),
        json!({"portable":candidate.progress,"receipts":candidate.progress.migration_receipts})
    );
    backup::restore(&current_path, &current_candidate, &current200).unwrap();
    let mut current_restored = progress::load(&current_path, &current200).unwrap();
    assert_eq!(
        json!({"portable":current_restored,"receipts":current_restored.migration_receipts}),
        restored_before_reset
    );
    for (id, value) in expected_old.as_object().unwrap() {
        exact(&current_restored.attempts[id], value);
    }
    exact(&current_restored.attempts[&s.id], &selected);
    assert_eq!(
        current200[197..199]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        disk_registration::literal_registration::IDS
    );
    for source in &current200[197..199] {
        assert!(matches!(source.initial, State::Script { run: None, .. }));
        assert!(matches!(source.goal, engine::Goal::Script { .. }));
        assert!(!current_restored.attempts.contains_key(&source.id));
        assert!(!current_restored.favorites.contains(&source.id));
    }
    current_restored
        .attempts
        .insert(s.id.clone(), Attempt::new(&s, 7));
    teach::prepare(current_restored.attempts.get_mut(&s.id).unwrap(), &s, true).unwrap();
    for (id, value) in expected_old.as_object().unwrap() {
        exact(&current_restored.attempts[id], value);
    }
    assert!(!progress::current_solved(
        &current_restored.attempts[&s.id],
        &s
    ));
    assert!(current_restored.favorites.contains(&s.id));
    let mut current202=full201;
    current202.push(s.clone());
    assert_eq!(current202.len(),202);
    let path202=dir.path().join("current201-synthetic.json");
    std::fs::write(&path202,&current_saved).unwrap();
    let expected202=json!({"portable":current_loaded,"receipts":current_loaded.migration_receipts});
    progress::save_with_catalog(&path202,&current_loaded,&current202).unwrap();
    assert_eq!(json!({"portable":current_loaded,"receipts":current_loaded.migration_receipts}),expected202);
    let saved202=std::fs::read(&path202).unwrap();
    let loaded202=progress::load(&path202,&current202).unwrap();
    assert_eq!(std::fs::read(&path202).unwrap(),saved202);
    assert_eq!(json!({"portable":loaded202,"receipts":loaded202.migration_receipts}),expected202);
    let export202=dir.path().join("current201-portable.json");
    backup::export(&export202,&loaded202,&current202).unwrap();
    let candidate202=backup::inspect(&export202,&current202,&AtomicBool::new(false),&AtomicUsize::new(0)).unwrap();
    assert_eq!(json!({"portable":candidate202.progress,"receipts":candidate202.progress.migration_receipts}),json!({"portable":current_candidate.progress,"receipts":current_candidate.progress.migration_receipts}));
    backup::restore(&path202,&candidate202,&current202).unwrap();
    let mut restored202=progress::load(&path202,&current202).unwrap();
    assert_eq!(json!({"portable":restored202,"receipts":restored202.migration_receipts}),restored_before_reset);
    for (id,value) in expected_old.as_object().unwrap() { exact(&restored202.attempts[id],value); }
    exact(&restored202.attempts[&s.id],&selected);
    for id in ["shell-trim-options","shell-trim-manifest","shell-snapshot-audit","shell-conditional-rollout"] {
        assert!(!restored202.attempts.contains_key(id)&&!restored202.favorites.contains(id));
    }
    restored202.attempts.insert(s.id.clone(),Attempt::new(&s,7));
    teach::prepare(restored202.attempts.get_mut(&s.id).unwrap(),&s,true).unwrap();
    for (id,value) in expected_old.as_object().unwrap() { exact(&restored202.attempts[id],value); }
    assert!(!progress::current_solved(&restored202.attempts[&s.id],&s));
    assert!(restored202.favorites.contains(&s.id));

}
#[test]
fn whole_attempt_escaped_capacity_refusal_has_same_action_positive_and_exact_target() {
    let s = scenario(0);
    let mut low = Attempt::new(&s, 1);
    teach::prepare(&mut low, &s, true).unwrap();
    let server = if let State::Nfs { simulation } = &low.state {
        simulation.inventory().servers[0].id.clone()
    } else {
        panic!()
    };
    let action = core::Action::SetExportsDraft {
        server,
        text: ("\t".repeat(511) + "\n").repeat(16),
    };
    let mut positive = low.clone();
    work::perform_attempt(&mut positive, &s, &action).unwrap();
    assert_ne!(positive.state, low.state);
    let mut lo = 0usize;
    let mut hi = work::MAX_ATTEMPT;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        low.transcript = vec!["x".repeat(mid)];
        if serde_json::to_vec(&low).unwrap().len() <= work::MAX_ATTEMPT {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    low.transcript = vec!["x".repeat(lo)];
    assert_eq!(serde_json::to_vec(&low).unwrap().len(), work::MAX_ATTEMPT);
    let before = serde_json::to_value(&low).unwrap();
    let e = work::perform_attempt(&mut low, &s, &action).unwrap_err();
    assert!(e.contains("Attempt"));
    exact(&low, &before);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), low);
    progress::save(&path, &p).unwrap();
    let saved = std::fs::read(&path).unwrap();
    let mut oversized = p.attempts[&s.id].clone();
    oversized.transcript[0].push('x');
    p.attempts.insert(s.id.clone(), oversized);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), saved);
}

#[test]
fn saved_pending_publication_and_faults_do_not_masquerade_as_current_or_denied() {
    let s = scenario(0);
    let mut a = solved(&s);
    let dlms_learning_lab::engine::Goal::Nfs { definition } = &s.goal else {
        panic!()
    };
    let State::Nfs { simulation } = &a.state else {
        panic!()
    };
    let live = simulation.runtime().mounts.clone();
    let server = simulation.inventory().servers[0].id.clone();
    work::perform_attempt(
        &mut a,
        &s,
        &core::Action::SetExportsDraft {
            server: server.clone(),
            text: "unfinished unsupported text".into(),
        },
    )
    .unwrap();
    work::perform_attempt(
        &mut a,
        &s,
        &core::Action::SaveExportsDraft {
            server: server.clone(),
        },
    )
    .unwrap();
    let rows = progress::current_nfs_checks(&a, &s);
    assert_eq!(
        rows.len(),
        1 + definition.requirements.len() + definition.fresh.len()
    );
    assert!(!progress::current_solved(&a, &s));
    for (i, r) in definition.requirements.iter().enumerate() {
        if matches!(
            r,
            work::Requirement::CurrentMount { .. }
                | work::Requirement::CurrentAccess { .. }
                | work::Requirement::CurrentFileText { .. }
        ) {
            assert!(
                rows[i + 1].0,
                "held current mount/file/access remains truthful"
            );
        }
    }
    let State::Nfs { simulation } = &a.state else {
        panic!()
    };
    assert_eq!(simulation.runtime().mounts, live);
    let before = serde_json::to_value(&a).unwrap();
    assert!(
        work::perform_attempt(&mut a, &s, &core::Action::ApplySavedExports { server }).is_err()
    );
    exact(&a, &before);
    // Practice cannot be installed as the assignment, and a wrong supplied action is atomic.
    let source = s.nfs_teaching.as_ref().unwrap();
    for step in &source.example_actions {
        work::perform_practice(&mut a, &s, &step.action).unwrap();
    }
    exact(&a.state, &before["state"]);
    let before = serde_json::to_value(&a).unwrap();
    assert!(
        work::perform_attempt(
            &mut a,
            &s,
            &core::Action::Unmount {
                client: "not-a-supplied-client".into(),
                mountpoint: "/mnt/absent".into()
            }
        )
        .is_err()
    );
    exact(&a, &before);
}

// Recover the complete reviewed44 predecessor through both closed raw inverses.
// Canonical physical/private append recipes and all historical37 bytes stay fixed.
fn original44_unit_bytes() -> Vec<u8> {
    let raw47 = include_bytes!("../scenarios/units-v1.json");
    let original46 = one_off_registration::before_units(raw47).unwrap();
    let raw = original46.as_slice();
    let before = combined_catalog::before_physical_units(raw).unwrap();
    let current: serde_json::Value = serde_json::from_slice(raw).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(current["units"].as_array().unwrap().len(), 46);
    assert_eq!(original["units"].as_array().unwrap().len(), 44);
    assert_eq!(
        &current["units"].as_array().unwrap()[..44],
        original["units"].as_array().unwrap().as_slice()
    );
    let physical: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/catalog175-registration.json")).unwrap();
    let private: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/catalog176-registration.json")).unwrap();
    assert_eq!(current["units"][44], physical["physical_unit"]);
    assert_eq!(current["units"][45], private["private_unit"]);
    // Existing pinned helper validates original raw37 plus every prior recipe.
    assert_eq!(
        combined_catalog::old_unit_bytes(raw).unwrap(),
        combined_catalog::old_unit_bytes(&before).unwrap()
    );
    before
}
