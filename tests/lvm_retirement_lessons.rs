//! Two authored raw-retirement tasks. No host LVM commands or private progress.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
use serde_json::{Value, json};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-lvm-retire-raw", "lpic2-lvm-retire-pool"];
fn previous() -> &'static Vec<pack::Scenario> {
    static P: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    P.get_or_init(|| pack::accepted315().unwrap())
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
        combined(include_str!("../scenarios/lvm-retirement-lessons-v1.json"))
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
fn every_taught_example_is_reachable_and_practice_never_awards_assignment_credit() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        let d = def(s);
        d.validate().unwrap();
        assert!(d.retirement.is_some() && d.lesson.retirement.is_some());
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(d.lesson.concepts.len() >= 3);
        assert!(
            s.sources
                .iter()
                .any(|s| s.contains("pvremove") || s.contains("lvremove"))
        );
        let mut a = Attempt::new(s, 2);
        let original = sim(&a).current.clone();
        for c in &d.lesson.worked {
            sim_mut(&mut a).execute(d, &c.text, true).unwrap();
        }
        assert!(d.practice_checks(sim(&a)).iter().all(|r| r.0));
        assert_eq!(sim(&a).current, original);
        assert!(!progress::current_solved(&a, s));
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
fn alternate_orders_spellings_and_individual_pv_removals_pass() {
    let variants = [
        vec![
            "lvremove --yes /dev/records/cache",
            "lvremove -y records/scratch",
        ],
        vec![
            "lvremove --yes /dev/retired/queue",
            "lvremove -y retired/scratch",
            "vgremove --yes retired",
            "pvremove /dev/retireb",
            "pvremove /dev/retirea",
        ],
    ];
    for (s, commands) in labs().iter().zip(variants) {
        let mut a = Attempt::new(s, 3);
        for c in commands {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn partial_forged_reports_wrong_approaches_data_damage_and_stale_success_fail() {
    for (i, s) in labs().iter().enumerate() {
        let d = def(s);
        let mut a = Attempt::new(s, 3);
        sim_mut(&mut a).checked = true;
        sim_mut(&mut a).transcript.push(work::Row {
            input: "lvs".into(),
            output: "all retired".into(),
        });
        assert!(!progress::current_solved(&a, s));
        let wrong = if i == 0 {
            "lvextend -l +1 records/ledger"
        } else {
            "vgremove retired"
        };
        let before = sim(&a).current.clone();
        let result = engine::execute_for_goal(&mut a.state, &s.goal, wrong);
        if i == 1 {
            assert!(result.is_err());
            assert_eq!(sim(&a).current, before);
        }
        assert!(!progress::current_solved(&a, s));
        let partial = if i == 0 {
            "lvremove -y records/scratch"
        } else {
            "lvremove -y retired/scratch"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, partial).unwrap();
        let rows = d.checks(sim(&a));
        assert!(rows.iter().any(|r| r.0));
        assert!(rows.iter().any(|r| !r.0));
        let mut a = solve(s);
        a.step = s.steps.len();
        sim_mut(&mut a).checked = true;
        let reintroduce = if i == 0 {
            "lvcreate -l 1 -n scratch records"
        } else {
            "pvcreate /dev/retirea"
        };
        engine::execute_for_goal(&mut a.state, &s.goal, reintroduce).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert!(!sim(&a).checked);
        assert_eq!(a.step, s.steps.len());
        let mut a = solve(s);
        sim_mut(&mut a)
            .current
            .groups
            .get_mut("records")
            .unwrap()
            .volumes
            .get_mut("ledger")
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
fn saves_keep_drafts_favorites_prior_work_and_selected_reset() {
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
fn invalid_ids_mixed_plans_injected_profiles_and_unreachable_work_are_rejected() {
    let raw: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-retirement-lessons-v1.json")).unwrap();
    for mode in 0..7 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["scenarios"][0]["id"] = json!("Invalid ID"),
            1 => bad["scenarios"][1]["id"] = bad["scenarios"][0]["id"].clone(),
            2 => {
                bad["scenarios"][0]["goal"]["definition"]["initial"]["provisioning"] =
                    json!("Linear")
            }
            3 => bad["scenarios"][0]["solution"] = json!(["lvs"]),
            4 => {
                bad["scenarios"][0]["goal"]["definition"]["retirement"]["retired_volumes"] =
                    json!(["records/ledger"])
            }
            5 => {
                bad["scenarios"][0]["goal"]["definition"]["lesson"]["worked"][0]["text"] =
                    json!("lvremove -ff workshop/ledger")
            }
            _ => {
                bad["scenarios"][1]["goal"]["definition"]["retirement"]["cleared_pvs"] =
                    json!(["/dev/orphan"])
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
#[test]
fn registered317_keeps_all315_scenarios106_units_and_prerequisite_sequence() {
    let before = pack::accepted315().unwrap();
    let all = pack::accepted317().unwrap();
    assert_eq!((before.len(), all.len()), (315, 317));
    assert_eq!(
        serde_json::to_value(&all[..315]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        all[315..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 111, 55, 77]
    );
    for s in &all[315..] {
        let at = all.iter().position(|a| a.id == s.id).unwrap();
        for id in &s.prerequisites {
            assert!(all.iter().position(|a| &a.id == id).unwrap() < at);
        }
    }
    let old = dlms_learning_lab::course::accepted315(&before).unwrap();
    let units = dlms_learning_lab::course::accepted317(&all).unwrap();
    assert_eq!((old.len(), units.len()), (106, 107));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "lvm-raw-retirement")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 43, 18, 26]
    );
    let at = units
        .iter()
        .position(|u| u.id == "lvm-raw-retirement")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at - 1].id, "lvm-linear-provisioning");
}
