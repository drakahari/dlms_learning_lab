//! Authored but not yet registered swap unit; no count or UI acceptance from tests alone.
use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    storage_teaching::{self, Learning},
    storage_workspace::{self as work, Draft},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic1-swap-prepared-activation",
    "lpic1-swap-replacement-handover",
];
fn labs() -> &'static Vec<pack::Scenario> {
    static L: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    L.get_or_init(|| {
        let old = pack::accepted310().unwrap();
        let mut texts = old
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":rows}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        texts.push(include_str!("../scenarios/swap-policy-lessons-v1.json").into());
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn storage(s: &State) -> &dlms_learning_lab::storage::State {
    let State::PersistentMounts { storage } = s else {
        panic!()
    };
    storage
}
fn storage_mut(s: &mut State) -> &mut dlms_learning_lab::storage::State {
    let State::PersistentMounts { storage } = s else {
        panic!()
    };
    storage
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    storage_teaching::prepare(&mut a, s.storage_teaching.as_ref().unwrap(), true);
    a
}
fn solve(s: &pack::Scenario) -> Attempt {
    let mut a = attempt(s);
    for c in &s.solution {
        engine::execute(&mut a.state, c).unwrap_or_else(|e| panic!("{} {c}: {e}", s.id));
    }
    a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn authored_references_and_separate_worked_contexts_reach_real_state() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        assert!(!engine::solved(&s.initial, &s.goal));
        let a = solve(s);
        assert!(engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
        let l = s.storage_teaching.as_ref().unwrap();
        l.validate().unwrap();
        let mut ex = Learning::new(l);
        let assignment = a.state.clone();
        let before = ex.state.clone();
        ex.fill_example(l).unwrap();
        assert_eq!(ex.state, before);
        assert!(!ex.checked);
        assert!(ex.draft.save(&mut ex.state));
        for c in &l.example_solution {
            work::adapter(&mut ex.state, c).unwrap();
        }
        ex.draft = Draft::new(&ex.state);
        ex.checked = true;
        assert!(ex.checks(l).iter().all(|r| r.0));
        assert_eq!(a.state, assignment);
        assert_ne!(storage(&a.state).inventory(), ex.state.inventory());
    }
}
#[test]
fn guided_saved_only_changes_expose_a_specific_misconception_and_retry() {
    for (n, s) in labs().iter().enumerate() {
        let l = s.storage_teaching.as_ref().unwrap();
        let mut ex = Learning::new(l);
        for c in &l.example_solution {
            work::adapter(&mut ex.state, c).unwrap();
        }
        ex.draft = Draft::new(&ex.state);
        ex.checked = true;
        let active = ex.state.active_swap().clone();
        for c in &l.try_actions {
            work::adapter(&mut ex.state, c).unwrap();
        }
        ex.draft = Draft::new(&ex.state);
        let rows = ex.checks(l);
        assert!(rows.iter().any(|r| !r.0));
        assert!(rows.iter().any(|r| r.0 && r.1.contains("Active workshop")));
        assert_eq!(ex.state.active_swap(), &active);
        assert!(rows.iter().any(|r| !r.0
            && r.1.contains(if n == 0 {
                "automatic activation"
            } else {
                "priority is"
            })));
        let save = l.example_solution[0].clone();
        work::adapter(&mut ex.state, &save).unwrap();
        ex.draft = Draft::new(&ex.state);
        assert!(ex.checks(l).iter().all(|r| r.0));
        let mut a = attempt(s);
        a.workspace.as_mut().unwrap().storage_learning = Some(ex);
        assert_eq!(a.state, s.initial);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn alternate_explicit_activation_orders_pass_without_matching_reference_strings() {
    for s in labs() {
        let mut a = attempt(s);
        let facts = storage(&a.state).inventory().swap().unwrap().clone();
        let target = facts
            .areas
            .iter()
            .find(|a| matches!(a.id.as_str(), "fast-swap" | "replacement"))
            .unwrap();
        let p = if target.id == "replacement" { 100 } else { 40 };
        if target.id == "replacement" {
            engine::execute(&mut a.state, &format!("swapoff -U {}", target.uuid)).unwrap();
            let old = facts.areas.iter().find(|a| a.id == "retired").unwrap();
            engine::execute(&mut a.state, &format!("swapoff -U {}", old.uuid)).unwrap();
        }
        engine::execute(&mut a.state, &format!("swapon -p {p} -U {}", target.uuid)).unwrap();
        engine::execute(&mut a.state, &s.solution[0]).unwrap();
        a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
        assert!(
            progress::current_solved(&a, s),
            "{:?}",
            engine::checks(&a.state, &s.goal)
        );
        assert!(s.solution.iter().all(|c| !c.contains("swapon -p")));
        engine::execute(&mut a.state, &format!("swapoff -U {}", target.uuid)).unwrap();
        assert!(!progress::current_solved(&a, s));
        engine::execute(&mut a.state, "swapon -a").unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn save_only_partial_wrong_identity_and_erased_table_cannot_award_completion() {
    for s in labs() {
        let mut a = attempt(s);
        engine::execute(&mut a.state, &s.solution[0]).unwrap();
        a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
        assert!(!progress::current_solved(&a, s));
        let rows = engine::checks(&a.state, &s.goal);
        assert!(rows.iter().any(|r| r.0));
        assert!(rows.iter().any(|r| !r.0 && r.1.contains("Active")));
        let mut a = solve(s);
        let source = storage(&a.state).inventory().swap().unwrap().areas[0]
            .uuid
            .clone();
        let text = storage(&a.state)
            .configuration()
            .replace(&source, "99999999-9999-4999-8999-999999999999");
        storage_mut(&mut a.state).save_configuration(text).unwrap();
        a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
        assert!(!progress::current_solved(&a, s));
        assert!(
            engine::checks(&a.state, &s.goal)
                .iter()
                .any(|r| !r.0 && r.1.contains("No supplied swap area"))
        );
        storage_mut(&mut a.state)
            .save_configuration(String::new())
            .unwrap();
        assert!(!engine::solved(&a.state, &s.goal));
    }
}
#[test]
fn actual_safe_progress_write_read_keeps_both_drafts_and_favorites_without_inventing_credit() {
    let all = labs().clone();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = solve(s);
        let w = a.workspace.as_mut().unwrap();
        w.storage_draft.as_mut().unwrap().text += "# unfinished assignment note\n";
        w.storage_draft.as_mut().unwrap().command = "swapon --show --bytes".into();
        let l = w.storage_learning.as_mut().unwrap();
        l.draft.text = "# independent example notes\n".into();
        l.draft.command = "blkid".into();
        assert!(!progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    let other = serde_json::to_value(&loaded.attempts[IDS[1]]).unwrap();
    let mut changed = loaded;
    changed.attempts.insert(IDS[0].into(), attempt(&all[0]));
    assert_eq!(
        serde_json::to_value(&changed.attempts[IDS[1]]).unwrap(),
        other
    );
    assert!(changed.favorites.contains(IDS[0]));
    assert!(!progress::current_solved(
        &changed.attempts[IDS[0]],
        &all[0]
    ));
}
#[test]
fn registered312_preserves_all310_scenarios104_units_and_prerequisite_order() {
    let old = pack::accepted310().unwrap();
    let all = pack::accepted312().unwrap();
    assert_eq!((old.len(), all.len()), (310, 312));
    assert_eq!(
        serde_json::to_value(&all[..310]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 111, 50, 77]
    );
    for s in labs() {
        let at = all.iter().position(|r| r.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(&all[at]).unwrap(),
            serde_json::to_value(s).unwrap()
        );
        for p in &s.prerequisites {
            assert!(all.iter().position(|r| &r.id == p).unwrap() < at);
        }
    }
    let before = dlms_learning_lab::course::accepted310(&old).unwrap();
    let units = dlms_learning_lab::course::accepted312(&all).unwrap();
    assert_eq!((before.len(), units.len()), (104, 105));
    let kept = units
        .iter()
        .filter(|u| u.id != "linux-swap-policy")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 43, 16, 26]
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-swap-policy")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    for id in ["linux-disk-preparation", "linux-removable-callers"] {
        assert!(units.iter().position(|u| u.id == id).unwrap() < at);
    }
}
