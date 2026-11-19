//! Prospective complete source, behavior, alternatives and persisted-work gates.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace::{
        self as work, Session,
        snapshot_flow::{self as flow, Operation},
    },
    pack::{Pack, Scenario},
    progress::{self, Attempt, Progress},
};
use std::sync::OnceLock;
const DATA: &str = include_str!("../scenarios/lvm-cow-lessons-v1.json");
const IDS: [&str; 3] = [
    "lpic2-lvm-saved-view",
    "lpic2-lvm-cow-budget",
    "lpic2-lvm-record-recovery",
];
fn labs() -> &'static Vec<Scenario> {
    static LABS: OnceLock<Vec<Scenario>> = OnceLock::new();
    LABS.get_or_init(|| serde_json::from_str::<Pack>(DATA).unwrap().scenarios)
}
fn d(s: &Scenario) -> &work::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn command(text: &str) -> Operation {
    Operation::Command { text: text.into() }
}
fn solve(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 0);
    for text in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
    }
    assert!(
        progress::current_solved(&a, s),
        "{}: {:?}",
        s.id,
        d(s).checks(sim(&mut a))
    );
    a
}
#[test]
fn prospective_sources_have_reachable_assignments_and_meaningful_separate_practice() {
    assert_eq!(
        labs().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for s in labs() {
        work::source(s).unwrap();
        d(s).validate().unwrap();
        assert!(!progress::current_solved(&Attempt::new(s, 0), s));
        assert_eq!(
            d(s).snapshot_task
                .as_ref()
                .unwrap()
                .reference
                .iter()
                .map(|op| flow::encode(op).unwrap())
                .collect::<Vec<_>>(),
            s.solution
        );
        assert!(
            d(s).lesson
                .concepts
                .iter()
                .any(|c| c.text.contains("simulator") || c.text.contains("COW"))
        );
        assert!(
            s.simplifications.contains("metadata")
                && s.simplifications.contains("no mounted filesystem")
        );
        solve(s);
    }
}
#[test]
fn allocation_alternatives_and_reordered_independent_recovery_operations_pass_actual_outcomes() {
    for s in labs() {
        let mut a = Attempt::new(s, 0);
        let mut ops = d(s).snapshot_task.as_ref().unwrap().reference.clone();
        match s.id.as_str() {
            "lpic2-lvm-saved-view" => {
                ops[0] = command(
                    "lvcreate --chunksize 128 --name before_run --size 2m --snapshot /dev/report/report",
                )
            }
            "lpic2-lvm-cow-budget" => ops[0] = command("lvextend --size 3m /dev/budget/rolling"),
            _ => {
                ops.swap(0, 1);
                ops.swap(1, 2);
                *ops.last_mut().unwrap() = command(
                    "lvcreate --name handover --snapshot --chunksize 128K --size 2M recovery/report",
                );
            }
        }
        for op in ops {
            sim(&mut a).apply_snapshot(d(s), &op, false).unwrap();
        }
        assert!(progress::current_solved(&a, s), "{}", s.id);
    }
}
#[test]
fn partial_work_wrong_time_and_too_small_store_fail_specifically_and_invalid_view_never_revives() {
    let s = &labs()[0];
    let mut a = Attempt::new(s, 0);
    let def = d(s);
    sim(&mut a)
        .apply_snapshot(def, &Operation::ApplyUpdate { index: 0 }, false)
        .unwrap();
    sim(&mut a)
        .execute(
            def,
            "lvcreate -s -L 1M -n before_run -c 128 report/report",
            false,
        )
        .unwrap();
    let rows = def.checks(sim(&mut a));
    assert!(rows.iter().any(|r| !r.0 && r.1.contains("point-in-time")));
    assert!(!progress::current_solved(&a, s));
    let s = &labs()[1];
    let mut a = Attempt::new(s, 0);
    let def = d(s);
    let output = sim(&mut a)
        .apply_snapshot(def, &Operation::ApplyUpdate { index: 0 }, false)
        .unwrap();
    assert!(output.contains("invalid"));
    let before = serde_json::to_value(&a).unwrap();
    assert!(
        sim(&mut a)
            .execute(def, "lvextend -L +1M budget/rolling", false)
            .is_err()
    );
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
    assert!(
        def.checks(sim(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("invalid"))
    );
    let s = &labs()[2];
    let mut a = Attempt::new(s, 0);
    let def = d(s);
    sim(&mut a)
        .execute(
            def,
            "lvcreate -s -L 1M -n handover -c 128 recovery/report",
            false,
        )
        .unwrap();
    for op in &def.snapshot_task.as_ref().unwrap().reference[..3] {
        sim(&mut a).apply_snapshot(def, op, false).unwrap();
    }
    assert!(!progress::current_solved(&a, s));
    assert!(
        def.checks(sim(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("point-in-time"))
    );
}
#[test]
fn examples_do_not_earn_assignment_credit_experiments_invalidate_and_repairs_or_reset_work() {
    for s in labs() {
        let def = d(s);
        let p = def.lesson.snapshot_practice.as_ref().unwrap();
        let mut a = Attempt::new(s, 0);
        let original = sim(&mut a).current.clone();
        for step in &p.worked {
            sim(&mut a)
                .apply_snapshot(def, &step.operation, true)
                .unwrap();
        }
        assert!(def.practice_checks(sim(&mut a)).iter().all(|r| r.0));
        assert!(!progress::current_solved(&a, s));
        sim(&mut a).example_checked = true;
        sim(&mut a)
            .apply_snapshot(def, &p.experiment, true)
            .unwrap();
        assert!(!sim(&mut a).example_checked);
        assert!(def.practice_checks(sim(&mut a)).iter().any(|r| !r.0));
        for op in &p.repair {
            sim(&mut a).apply_snapshot(def, op, true).unwrap();
        }
        if !p.repair.is_empty() {
            assert!(def.practice_checks(sim(&mut a)).iter().all(|r| r.0));
        }
        sim(&mut a).reset_example(def).unwrap();
        assert_eq!(sim(&mut a).example, def.lesson.fixture);
        assert_eq!(sim(&mut a).current, original);
    }
}
#[test]
fn persist_real_drafts_and_partial_or_passed_state_without_transcript_or_checked_credit() {
    let catalog = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    for s in catalog {
        let mut a = solve(s);
        sim(&mut a).set_command(d(s), "vgs".into(), false).unwrap();
        sim(&mut a).set_command(d(s), "lvs".into(), true).unwrap();
        sim(&mut a).checked = true;
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&bytes, catalog).unwrap();
    for s in catalog {
        let a = restored.attempts.get_mut(&s.id).unwrap();
        assert!(progress::current_solved(a, s));
        assert_eq!(sim(a).command, "vgs");
        assert_eq!(sim(a).example_command, "lvs");
        let extra = format!(
            "lvcreate -s -L 1M -n extra -c 128 {}/report",
            ["report", "budget", "recovery"][IDS.iter().position(|id| *id == s.id).unwrap()]
        );
        sim(a).execute(d(s), &extra, false).unwrap();
        assert!(!sim(a).checked);
        assert!(!progress::current_solved(a, s));
        assert!(restored.favorites.contains(&s.id));
    }
    let s = &catalog[0];
    restored.attempts.insert(s.id.clone(), Attempt::new(s, 8));
    assert!(!progress::current_solved(&restored.attempts[&s.id], s));
    assert_eq!(restored.attempts.len(), 3);
}
#[test]
fn source_corruption_injected_checked_flags_and_matching_names_cannot_fake_outcomes() {
    for s in labs() {
        let mut a = Attempt::new(s, 0);
        sim(&mut a).checked = true;
        assert!(!progress::current_solved(&a, s));
        let def = d(s);
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            sim(&mut a)
                .apply_snapshot(def, &Operation::ApplyUpdate { index: 99 }, false)
                .is_err()
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        let mut bad = def.clone();
        bad.snapshot_task.as_mut().unwrap().context.plan.snapshots[0].minimum_bytes = 100;
        assert!(bad.validate().is_err());
        let mut bad = def.clone();
        bad.lesson.fixture = def.initial.clone();
        assert!(bad.validate().is_err());
    }
    let s = &labs()[2];
    let mut a = solve(s);
    let def = d(s);
    sim(&mut a)
        .execute(def, "lvremove -y recovery/trusted", false)
        .unwrap();
    sim(&mut a)
        .execute(
            def,
            "lvcreate -s -L 2M -n trusted -c 128 recovery/report",
            false,
        )
        .unwrap();
    assert!(!progress::current_solved(&a, s));
}

#[test]
fn current329_preserves_every326_scenario_and111_unit_object_and_prerequisite_order() {
    let old = dlms_learning_lab::pack::accepted326().unwrap();
    let now = dlms_learning_lab::pack::accepted329().unwrap();
    assert_eq!((old.len(), now.len()), (326, 329));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..326]).unwrap()
    );
    assert_eq!(
        now[326..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for (i, s) in now.iter().enumerate() {
        for prerequisite in &s.prerequisites {
            assert!(now.iter().position(|p| &p.id == prerequisite).unwrap() < i);
        }
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [74, 114, 64, 77]
    );
    let before = dlms_learning_lab::course::accepted326(&old).unwrap();
    let after = dlms_learning_lab::course::accepted329(&now).unwrap();
    assert_eq!((before.len(), after.len()), (111, 112));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(
            after
                .iter()
                .filter(|u| u.id != "lvm-cow-recovery")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = after
        .iter()
        .position(|u| u.id == "lvm-cow-recovery")
        .unwrap();
    assert_eq!(after[at - 1].id, "lvm-pv-capacity");
    assert_eq!(after[at].labs, IDS);
}

#[test]
fn reviewed_presentation_keeps_every_requirement_and_uses_lesson_specific_guidance() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    let compact = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    for s in labs() {
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == s.id)
            .unwrap();
        let requirements = row["requirements"].as_array().unwrap();
        assert!(requirements.len() >= 4);
        assert_eq!(
            compact(
                &requirements
                    .iter()
                    .map(|s| s.as_str().unwrap())
                    .collect::<String>()
            ),
            compact(&s.objective)
        );
        let steps = row["practice"].as_array().unwrap();
        assert_eq!(steps.len(), 3);
        assert!(
            steps
                .iter()
                .any(|s| s["action"].as_str().unwrap().contains("Record"))
        );
        assert!(steps.iter().any(|s| {
            s["action"]
                .as_str()
                .unwrap()
                .contains("Check separate example")
        }));
        assert!(!steps.iter().any(|s| {
            s["action"]
                .as_str()
                .unwrap()
                .contains("Choose each annotated step in order")
        }));
    }
}
