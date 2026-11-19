//! Complete authored tasks and preserved catalog, not native acceptance.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace::{
        self as w, device_reference_flow::Operation, device_reference_session as packet,
    },
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-lvm-reference-repair", "lpic2-lvm-reference-handover"];
fn labs() -> &'static Vec<Scenario> {
    static V: OnceLock<Vec<Scenario>> = OnceLock::new();
    V.get_or_init(|| {
        serde_json::from_str::<Pack>(include_str!("../scenarios/lvm-reference-lessons-v1.json"))
            .unwrap()
            .scenarios
    })
}
fn d(s: &Scenario) -> &w::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn apply(a: &mut Attempt, s: &Scenario, o: &Operation) {
    sim(a).apply_reference(d(s), o, false).unwrap();
}
fn save(k: &str, p: &str) -> Operation {
    Operation::SaveReference {
        caller: k.into(),
        path: p.into(),
    }
}
fn command(t: &str) -> Operation {
    Operation::Command { text: t.into() }
}
fn solve(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 0);
    for text in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
    }
    assert!(
        progress::current_solved(&a, s),
        "{} {:?}",
        s.id,
        d(s).checks(sim(&mut a))
    );
    a
}
#[test]
fn full_authored_tasks_and_separate_teaching_have_reachable_actual_outcomes() {
    assert_eq!(
        labs().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for s in labs() {
        for op in &d(s).device_reference_task.as_ref().unwrap().reference {
            if let Operation::SaveReference { path, .. } = op {
                assert!(
                    d(s).device_reference_task
                        .as_ref()
                        .unwrap()
                        .context
                        .initial
                        .callers
                        .values()
                        .all(|c| &c.path != path),
                    "A supplied reference must not contain a completed graded address"
                );
            }
        }
        w::source(s).unwrap();
        d(s).validate().unwrap();
        assert!(!progress::current_solved(&Attempt::new(s, 0), s));
        assert_eq!(
            s.solution,
            d(s).device_reference_task
                .as_ref()
                .unwrap()
                .reference
                .iter()
                .map(|o| packet::encode(o).unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(d(s).lesson.concepts.len(), 4);
        assert!(s.simplifications.contains("not a stable ABI"));
        let mut a = Attempt::new(s, 0);
        for step in &d(s)
            .lesson
            .device_reference_practice
            .as_ref()
            .unwrap()
            .worked
        {
            sim(&mut a)
                .apply_reference(d(s), &step.operation, true)
                .unwrap();
        }
        assert!(d(s).practice_checks(sim(&mut a)).iter().all(|r| r.0));
        assert!(!progress::current_solved(&a, s));
        solve(s);
    }
}
#[test]
fn equivalent_rename_forms_both_orders_and_independent_configuration_order_pass() {
    let first = &labs()[0];
    let mut a = Attempt::new(first, 0);
    for o in d(first)
        .device_reference_task
        .as_ref()
        .unwrap()
        .reference
        .iter()
        .rev()
    {
        apply(&mut a, first, o);
    }
    assert!(progress::current_solved(&a, first));
    let s = &labs()[1];
    for commands in [
        vec![
            "vgrename store-main records-main",
            "lvrename /dev/records-main/ledger-old ledger-live",
        ],
        vec![
            "lvrename store-main/ledger-old ledger-live",
            "vgrename store-main records-main",
        ],
        vec![
            "lvrename store-main ledger-old temporary",
            "lvrename store-main temporary ledger-live",
            "vgrename store-main records-main",
        ],
    ] {
        let mut a = Attempt::new(s, 0);
        for c in commands {
            apply(&mut a, s, &command(c));
        }
        apply(&mut a, s, &save("audit", "/dev/records-main/audit-copy"));
        apply(
            &mut a,
            s,
            &save("reporting", "/dev/records-main/ledger-live"),
        );
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn partial_wrong_object_internal_form_and_stale_names_have_specific_failure_and_retry() {
    let s = &labs()[0];
    let mut a = Attempt::new(s, 0);
    apply(
        &mut a,
        s,
        &save("reporting", "/dev/north-data/daily-report"),
    );
    assert!(!progress::current_solved(&a, s));
    assert!(
        d(s).checks(sim(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("another volume"))
    );
    apply(&mut a, s, &save("audit", "/dev/north-data/audit-copy"));
    assert!(progress::current_solved(&a, s));
    apply(
        &mut a,
        s,
        &save("reporting", "/dev/mapper/north--data-daily--report"),
    );
    assert!(!progress::current_solved(&a, s));
    assert!(
        d(s).checks(sim(&mut a))
            .iter()
            .any(|r| r.1.contains("internal mapper"))
    );
    apply(
        &mut a,
        s,
        &save("reporting", "/dev/north-data/daily-report"),
    );
    assert!(progress::current_solved(&a, s));
    let s = &labs()[1];
    let mut a = solve(s);
    apply(&mut a, s, &save("reporting", "/dev/store-main/ledger-old"));
    assert!(!progress::current_solved(&a, s));
    assert!(
        d(s).checks(sim(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("cannot read"))
    );
    apply(
        &mut a,
        s,
        &save("reporting", "/dev/records-main/ledger-live"),
    );
    assert!(progress::current_solved(&a, s));
}
#[test]
fn save_resume_unsaved_drafts_current_invalidation_selected_reset_and_favorites_preserve_truth() {
    let catalog = pack::ccna_pilot().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs() {
        let mut a = solve(s);
        sim(&mut a)
            .edit_reference(d(s), "audit", "/dev/no_group/no_volume".into(), false)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        sim(&mut a).set_command(d(s), "vgs".into(), false).unwrap();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let data = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&data, &catalog).unwrap();
    for s in labs() {
        let a = restored.attempts.get_mut(&s.id).unwrap();
        w::context(a, s).unwrap();
        assert!(!progress::current_solved(a, s));
        assert_eq!(sim(a).command, "vgs");
        assert_eq!(
            sim(a).reference_runtime.as_ref().unwrap().assignment.drafts["audit"].0,
            "/dev/no_group/no_volume"
        );
        assert!(restored.favorites.contains(&s.id));
    }
    restored
        .attempts
        .insert(IDS[0].into(), Attempt::new(&labs()[0], 1));
    assert!(
        sim(restored.attempts.get_mut(IDS[1]).unwrap())
            .reference_runtime
            .as_ref()
            .unwrap()
            .assignment
            .drafts["audit"]
            .0
            .contains("no_group")
    );
    assert_eq!(restored.attempts.len(), 2);
}
#[test]
fn current331_preserves_all329_scenario_objects_and112_units_without_legacy_credit() {
    let before = pack::accepted329().unwrap();
    let after = pack::accepted331().unwrap();
    assert_eq!((before.len(), after.len()), (329, 331));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&after[..329]).unwrap()
    );
    assert_eq!(
        after[329..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let old = dlms_learning_lab::course::accepted329(&before).unwrap();
    let new = dlms_learning_lab::course::accepted331(&after).unwrap();
    assert_eq!((old.len(), new.len()), (112, 113));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            new.iter()
                .filter(|u| u.id != "lvm-application-references")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = new
        .iter()
        .position(|u| u.id == "lvm-application-references")
        .unwrap();
    assert_eq!(new[at - 1].id, "lvm-storage-names");
    assert_eq!(new[at].labs, IDS);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| after.iter().filter(|s| s.track == t).count()),
        [74, 114, 66, 77]
    );
    for (i, s) in after.iter().enumerate() {
        for id in &s.prerequisites {
            assert!(after.iter().position(|p| &p.id == id).unwrap() < i);
        }
    }
}
#[test]
fn every_independent_requirement_and_specific_practice_instruction_is_present() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    let compact = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    for s in labs() {
        let r = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == s.id)
            .unwrap();
        let req = r["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap())
            .collect::<String>();
        assert_eq!(compact(&req), compact(&s.objective));
        assert_eq!(r["practice"].as_array().unwrap().len(), 3);
        assert!(r["practice"].to_string().contains("Save"));
        assert!(r["practice"].to_string().contains("actual"));
    }
}
