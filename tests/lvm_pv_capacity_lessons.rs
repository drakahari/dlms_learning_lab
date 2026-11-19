//! Authored capacity-layer and retained-placement tasks. No host LVM execution.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm::MIB,
    lvm_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
use serde_json::{Value, json};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-lvm-pv-capacity", "lpic2-lvm-pv-end"];
#[test]
fn registered323_keeps_every321_scenario109_units_and_prerequisite_order() {
    let before = pack::accepted321().unwrap();
    let all = pack::accepted323().unwrap();
    assert_eq!((before.len(), all.len()), (321, 323));
    assert_eq!(
        serde_json::to_value(&all[..321]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        all[321..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 111, 61, 77]
    );
    for s in &all[321..] {
        let at = all.iter().position(|a| a.id == s.id).unwrap();
        for id in &s.prerequisites {
            assert!(all.iter().position(|a| &a.id == id).unwrap() < at);
        }
    }
    let old = dlms_learning_lab::course::accepted321(&before).unwrap();
    let units = dlms_learning_lab::course::accepted323(&all).unwrap();
    assert_eq!((old.len(), units.len()), (109, 110));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "lvm-pv-capacity")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 43, 21, 26]
    );
    let at = units
        .iter()
        .position(|u| u.id == "lvm-pv-capacity")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at - 1].id, "lvm-storage-names");
}
fn previous() -> &'static Vec<pack::Scenario> {
    static P: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    P.get_or_init(|| pack::accepted321().unwrap())
}
fn combined(text: &str) -> Result<Vec<pack::Scenario>, String> {
    let mut texts = vec![text.to_owned()];
    texts.extend(
        previous().chunks(8).map(|rows| {
            serde_json::to_string(&json!({"schema_version":1,"scenarios":rows})).unwrap()
        }),
    );
    pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
}
fn labs() -> &'static Vec<pack::Scenario> {
    static L: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    L.get_or_init(|| {
        combined(include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json"))
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn def(s: &pack::Scenario) -> &work::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &Attempt) -> &work::Session {
    let State::Lvm { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn sim_mut(a: &mut Attempt) -> &mut work::Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c)
            .unwrap_or_else(|e| panic!("{} {c}: {e}", s.id));
    }
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn taught_method_and_separate_adverse_practice_work_for_both_tasks() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        let d = def(s);
        d.validate().unwrap();
        assert!(d.initial.pv_capacity.is_some());
        assert!(d.lesson.concepts.len() >= 4);
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut a = Attempt::new(s, 2);
        let original = sim(&a).current.clone();
        for (i, c) in d.lesson.worked.iter().enumerate() {
            sim_mut(&mut a).insert(d, i).unwrap();
            assert_eq!(sim(&a).current, original);
            sim_mut(&mut a).execute(d, &c.text, true).unwrap();
        }
        assert!(d.practice_checks(sim(&a)).iter().all(|r| r.0));
        assert!(!progress::current_solved(&a, s));
        assert_eq!(sim(&a).current, original);
        sim_mut(&mut a)
            .execute(d, &d.lesson.experiment, true)
            .unwrap();
        assert!(d.practice_checks(sim(&a)).iter().any(|r| !r.0));
        sim_mut(&mut a).execute(d, &d.lesson.repair, true).unwrap();
        assert!(d.practice_checks(sim(&a)).iter().all(|r| r.0));
        let _ = solve(s);
    }
}
#[test]
fn alternate_supported_forms_and_larger_legitimate_application_capacity_pass() {
    for (s, commands) in labs().iter().zip([
        vec![
            "pvresize --setphysicalvolumesize 65m /dev/vaulta",
            "lvextend --extents 6 /dev/vault/index",
            "resize2fs /dev/vault/index 6144",
        ],
        vec!["pvresize --setphysicalvolumesize 25m /dev/storea"],
    ]) {
        let mut a = Attempt::new(s, 3);
        for c in commands {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        assert_eq!(sim(&a).current, sim(&solve(s)).current);
    }
    let s = &labs()[0];
    let mut a = Attempt::new(s, 4);
    for c in [
        "pvresize /dev/vaulta",
        "lvextend -L 32M vault/index",
        "resize2fs /dev/vault/index",
    ] {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    assert_eq!(sim(&a).current.lv_bytes("vault/index").unwrap(), 32 * MIB);
}
#[test]
fn wrong_layers_unsafe_end_partial_work_forged_output_and_current_invalidation_fail() {
    for (i, s) in labs().iter().enumerate() {
        let d = def(s);
        let mut a = Attempt::new(s, 3);
        sim_mut(&mut a).checked = true;
        sim_mut(&mut a).transcript.push(work::Row {
            input: "pvs".into(),
            output: "all correct".into(),
        });
        assert!(!progress::current_solved(&a, s));
        let before = sim(&a).current.clone();
        let (bad, word) = if i == 0 {
            ("lvextend -L 24M vault/index", "free")
        } else {
            (
                "pvresize --setphysicalvolumesize 17M /dev/storea",
                "extent 4",
            )
        };
        let error = engine::execute_for_goal(&mut a.state, &s.goal, bad).unwrap_err();
        assert!(error.contains(word), "{error}");
        assert_eq!(sim(&a).current, before);
        let partial = if i == 0 {
            "pvresize /dev/vaulta"
        } else {
            "pvresize --setphysicalvolumesize 29M /dev/storea"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, partial).unwrap();
        let rows = d.checks(sim(&a));
        assert!(rows.iter().any(|r| r.0));
        assert!(rows.iter().any(|r| !r.0));
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        sim_mut(&mut a).checked = true;
        let adverse = if i == 0 {
            "pvresize --setphysicalvolumesize 33M /dev/vaulta"
        } else {
            "pvresize /dev/storea"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, adverse).unwrap();
        assert!(!sim(&a).checked);
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        let peer = if i == 0 {
            "/dev/holda"
        } else {
            "/dev/reservea"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, &format!("pvresize {peer}")).unwrap();
        assert!(
            d.checks(sim(&a))
                .iter()
                .any(|r| !r.0 && r.1.contains(&format!("Unrelated {peer}")))
        );
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        let (g, v) = if i == 0 {
            ("vault", "index")
        } else {
            ("store", "report")
        };
        sim_mut(&mut a)
            .current
            .groups
            .get_mut(g)
            .unwrap()
            .volumes
            .get_mut(v)
            .unwrap()
            .filesystem
            .as_mut()
            .unwrap()
            .files
            .get_mut("/keep.txt")
            .unwrap()
            .contents
            .push_str("changed");
        assert!(!progress::current_solved(&a, s));
    }
    let s = &labs()[1];
    let mut a = solve(s);
    for c in ["lvextend -l +1 store/report", "resize2fs /dev/store/report"] {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(
        def(s)
            .checks(sim(&a))
            .iter()
            .any(|r| !r.0 && r.1.contains("preserve the original LV"))
    );
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn safe_save_resume_keeps_capacity_layers_drafts_favorites_and_selected_reset() {
    let all = labs().clone();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = solve(s);
        let d = def(s);
        sim_mut(&mut a)
            .set_command(d, "unfinished assignment".into(), false)
            .unwrap();
        sim_mut(&mut a).set_command(d, "pvs".into(), true).unwrap();
        for c in &d.lesson.worked {
            sim_mut(&mut a).execute(d, &c.text, true).unwrap();
        }
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
    let other = serde_json::to_value(&loaded.attempts[IDS[1]]).unwrap();
    loaded
        .attempts
        .insert(IDS[0].into(), Attempt::new(&all[0], 9));
    assert_eq!(
        serde_json::to_value(&loaded.attempts[IDS[1]]).unwrap(),
        other
    );
    assert!(!progress::current_solved(&loaded.attempts[IDS[0]], &all[0]));
    assert!(loaded.favorites.contains(IDS[0]));
}
#[test]
fn malformed_source_goals_ids_and_unreachable_solutions_are_rejected() {
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-pv-capacity-lessons-v1.json")).unwrap();
    for mode in 0..8 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["scenarios"][0]["id"] = json!("Invalid ID"),
            1 => bad["scenarios"][1]["id"] = bad["scenarios"][0]["id"].clone(),
            2 => bad["scenarios"][0]["goal"]["definition"]["initial"]["pv_capacity"] = Value::Null,
            3 => bad["scenarios"][0]["solution"] = json!(["pvs"]),
            4 => {
                bad["scenarios"][0]["goal"]["definition"]["targets"][0]["pv_sizes"][0]["bytes"] =
                    json!(66 * MIB)
            }
            5 => {
                bad["scenarios"][1]["goal"]["definition"]["targets"][0]["pv_sizes"][0]["bytes"] =
                    json!(17 * MIB)
            }
            6 => {
                bad["scenarios"][0]["goal"]["definition"]["lesson"]["worked"][1]["text"] =
                    json!("pvresize --force /dev/aspena")
            }
            _ => {
                bad["scenarios"][1]["goal"]["definition"]["targets"][0]["minimum_lv"] =
                    json!(16 * MIB)
            }
        }
        let e = combined(&bad.to_string())
            .err()
            .unwrap_or_else(|| panic!("mode{mode} accepted"));
        assert!(!e.contains("invalid prerequisites"), "mode{mode}:{e}");
    }
}
