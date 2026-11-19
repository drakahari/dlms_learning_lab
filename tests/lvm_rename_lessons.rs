//! Two authored naming tasks, separate worked worlds and behavior-based goals.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
use serde_json::{Value, json};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-lvm-rename-volume", "lpic2-lvm-rename-handover"];
#[test]
fn registered321_preserves_every319_scenario108_units_and_prerequisite_order() {
    let before = pack::accepted319().unwrap();
    let all = pack::accepted321().unwrap();
    assert_eq!((before.len(), all.len()), (319, 321));
    assert_eq!(
        serde_json::to_value(&all[..319]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        all[319..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 111, 59, 77]
    );
    for s in &all[319..] {
        let at = all.iter().position(|a| a.id == s.id).unwrap();
        for id in &s.prerequisites {
            assert!(all.iter().position(|a| &a.id == id).unwrap() < at);
        }
    }
    let old = dlms_learning_lab::course::accepted319(&before).unwrap();
    let units = dlms_learning_lab::course::accepted321(&all).unwrap();
    assert_eq!((old.len(), units.len()), (108, 109));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "lvm-storage-names")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 43, 20, 26]
    );
    let at = units
        .iter()
        .position(|u| u.id == "lvm-storage-names")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at - 1].id, "lvm-availability");
}
fn previous() -> &'static Vec<pack::Scenario> {
    static P: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    P.get_or_init(|| pack::accepted319().unwrap())
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
        combined(include_str!("../scenarios/lvm-rename-lessons-v1.json"))
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
fn each_introduction_taught_method_and_adverse_practice_are_real_and_separate() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        let d = def(s);
        d.validate().unwrap();
        assert!(d.renaming.is_some() && d.lesson.renaming.is_some());
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
fn supported_alternate_forms_and_namespace_orders_have_identical_outcomes() {
    let variants = [
        vec!["lvrename team draft reports"],
        vec!["lvrename depot/archive catalog", "vgrename depot warehouse"],
    ];
    for (s, commands) in labs().iter().zip(variants) {
        let mut a = Attempt::new(s, 2);
        for c in commands {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        assert_eq!(sim(&a).current, sim(&solve(s)).current);
    }
}
#[test]
fn specific_wrong_names_collisions_partial_work_and_current_invalidation_are_reported() {
    for (i, s) in labs().iter().enumerate() {
        let d = def(s);
        let mut a = Attempt::new(s, 3);
        sim_mut(&mut a).checked = true;
        sim_mut(&mut a).transcript.push(work::Row {
            input: "lvs".into(),
            output: "all correct".into(),
        });
        assert!(!progress::current_solved(&a, s));
        if i == 0 {
            let before = sim(&a).current.clone();
            let e =
                engine::execute_for_goal(&mut a.state, &s.goal, "lvrename team draft reference")
                    .unwrap_err();
            assert!(e.contains("exists"));
            assert_eq!(sim(&a).current, before);
            engine::execute_for_goal(&mut a.state, &s.goal, "lvrename team draft temporary")
                .unwrap();
            assert!(
                d.checks(sim(&a))
                    .iter()
                    .any(|r| !r.0 && r.1.contains("team/temporary"))
            );
        } else {
            engine::execute_for_goal(&mut a.state, &s.goal, "vgrename depot warehouse").unwrap();
            let rows = d.checks(sim(&a));
            assert!(
                rows.iter()
                    .any(|r| r.0 && r.1.contains("required name warehouse"))
            );
            assert!(
                rows.iter()
                    .any(|r| !r.0 && r.1.contains("warehouse/archive"))
            );
        }
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        sim_mut(&mut a).checked = true;
        let c = if i == 0 {
            "lvrename team reports temporary"
        } else {
            "vgrename warehouse temporary"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        assert!(!sim(&a).checked);
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        let c = if i == 0 {
            "vgchange -ay reserve"
        } else {
            "vgchange -ay warehouse"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        let (g, v) = if i == 0 {
            ("team", "reports")
        } else {
            ("warehouse", "catalog")
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
            .push_str("damaged");
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn save_resume_preserves_names_data_drafts_favorites_and_selected_reset() {
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
        sim_mut(&mut a).set_command(d, "lvs".into(), true).unwrap();
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
fn malformed_ids_identity_goals_and_unreachable_work_are_rejected() {
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    for mode in 0..7 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["scenarios"][0]["id"] = json!("Invalid ID"),
            1 => bad["scenarios"][1]["id"] = bad["scenarios"][0]["id"].clone(),
            2 => bad["scenarios"][0]["goal"]["definition"]["initial"]["identity"] = Value::Null,
            3 => bad["scenarios"][0]["solution"] = json!(["lvs"]),
            4 => {
                bad["scenarios"][0]["goal"]["definition"]["renaming"]["volumes"][0]["name"] =
                    json!("reference")
            }
            5 => {
                bad["scenarios"][0]["goal"]["definition"]["lesson"]["worked"][1]["text"] =
                    json!("lvrename --force studio notes journal")
            }
            _ => {
                bad["scenarios"][1]["goal"]["definition"]["renaming"]["groups"][0]["name"] =
                    json!("live")
            }
        }
        let error = combined(&bad.to_string())
            .err()
            .unwrap_or_else(|| panic!("mode{mode} accepted"));
        assert!(
            !error.contains("invalid prerequisites"),
            "mode{mode}:{error}"
        );
    }
}
