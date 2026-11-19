//! Three authored practical allocations. No host LVM commands or private progress.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
use serde_json::{Value, json};
use std::sync::OnceLock;
const IDS: [&str; 3] = [
    "lpic2-lvm-provision-staging",
    "lpic2-lvm-provision-group",
    "lpic2-lvm-provision-handover",
];
fn previous() -> &'static Vec<pack::Scenario> {
    static P: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    P.get_or_init(|| pack::accepted312().unwrap())
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
        combined(include_str!(
            "../scenarios/lvm-provisioning-lessons-v1.json"
        ))
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
fn every_taught_workshop_runs_then_exposes_and_repairs_its_real_adverse_change() {
    assert_eq!(
        labs().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for s in labs() {
        let d = def(s);
        d.validate().unwrap();
        assert!(d.plan.is_some() && d.lesson.plan.is_some());
        assert!(!engine::solved(&s.initial, &s.goal));
        if d.plan
            .as_ref()
            .unwrap()
            .groups
            .iter()
            .any(|g| !d.initial.groups.contains_key(&g.name))
        {
            let session = work::Session::new(d).unwrap();
            assert!(d.checks(&session).iter().any(|r| {
                !r.0 && r
                    .1
                    .contains("group missing. Create it on the supplied devices")
            }));
        }
        assert!((4..=6).contains(&d.lesson.concepts.len()));
        assert!(s.sources.iter().any(|s| s.contains("lpi.org")));
        let mut a = Attempt::new(s, 2);
        let assignment = serde_json::to_value(&sim(&a).current).unwrap();
        for c in &d.lesson.worked {
            sim_mut(&mut a).execute(d, &c.text, true).unwrap();
        }
        assert!(d.practice_checks(sim(&a)).iter().all(|r| r.0));
        assert!(!progress::current_solved(&a, s));
        assert_eq!(serde_json::to_value(&sim(&a).current).unwrap(), assignment);
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
fn alternate_size_forms_explicit_pv_initialization_and_allocation_orders_pass() {
    let variants = [
        vec!["lvcreate --name staging --extents 6 harbor"],
        vec![
            "pvcreate /dev/buildb /dev/builda",
            "vgcreate --physicalextentsize 2M intake /dev/buildb /dev/builda",
            "lvcreate --name buffer --size 20M intake",
        ],
        vec![
            "vgcreate -s 4M delivery /dev/deliverya",
            "lvcreate -n stage -l 8 delivery",
            "lvcreate --name queue --size 16M service",
        ],
    ];
    for (s, commands) in labs().iter().zip(variants) {
        let mut a = Attempt::new(s, 3);
        for c in commands {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        assert!(progress::current_solved(&a, s), "{}", s.id);
        assert_eq!(
            sim(&a)
                .current
                .groups
                .values()
                .flat_map(|g| g.volumes.values())
                .filter(|v| v.filesystem.is_none())
                .count(),
            def(s).plan.as_ref().unwrap().volumes.len()
        );
    }
}
#[test]
fn common_wrong_plans_partial_work_forged_output_and_data_damage_cannot_pass() {
    for (i, s) in labs().iter().enumerate() {
        let mut a = Attempt::new(s, 4);
        assert!(!progress::current_solved(&a, s));
        sim_mut(&mut a).transcript.push(work::Row {
            input: "lvs".into(),
            output: "All requested allocations pass".into(),
        });
        sim_mut(&mut a).checked = true;
        assert!(!progress::current_solved(&a, s));
        let wrong = match i {
            0 => vec!["lvcreate -l 100%FREE -n staging harbor"],
            1 => vec![
                "vgcreate intake /dev/builda /dev/buildb",
                "lvcreate -L 20M -n buffer intake",
            ],
            _ => vec!["lvcreate -l 4 -n queue service"],
        };
        for c in wrong {
            engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
        }
        let rows = def(s).checks(sim(&a));
        assert!(rows.iter().any(|r| r.0));
        assert!(rows.iter().any(|r| !r.0));
        assert!(!progress::current_solved(&a, s));
        let mut a = solve(s);
        a.step = s.steps.len();
        sim_mut(&mut a).checked = true;
        let target = def(s).plan.as_ref().unwrap().volumes[0].id.clone();
        engine::execute_for_goal(&mut a.state, &s.goal, &format!("lvremove -y {target}")).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert_eq!(
            a.step,
            s.steps.len(),
            "Walkthrough position is retained but cannot manufacture current success"
        );
        assert!(!sim(&a).checked);
        let mut a = solve(s);
        let original = def(s).initial.groups.iter().next().unwrap();
        let lv = original.1.volumes.keys().next().unwrap();
        let fs = sim_mut(&mut a)
            .current
            .groups
            .get_mut(original.0)
            .unwrap()
            .volumes
            .get_mut(lv)
            .unwrap()
            .filesystem
            .as_mut()
            .unwrap();
        fs.files
            .values_mut()
            .next()
            .unwrap()
            .contents
            .push_str("damaged");
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn saved_progress_retains_separate_drafts_allocations_favorites_and_selected_reset() {
    let all = labs().clone();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = solve(s);
        let d = def(s);
        sim_mut(&mut a)
            .set_command(d, "unfinished independent command".into(), false)
            .unwrap();
        sim_mut(&mut a).set_command(d, "vgs".into(), true).unwrap();
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
fn invalid_identifiers_source_facts_goals_and_unreachable_solutions_are_rejected() {
    let raw: Value = serde_json::from_str(include_str!(
        "../scenarios/lvm-provisioning-lessons-v1.json"
    ))
    .unwrap();
    for mode in 0..7 {
        let mut bad = raw.clone();
        match mode {
            0 => bad["scenarios"][0]["id"] = json!("Invalid ID"),
            1 => bad["scenarios"][1]["id"] = bad["scenarios"][0]["id"].clone(),
            2 => bad["scenarios"][0]["goal"]["definition"]["initial"]["provisioning"] = Value::Null,
            3 => bad["scenarios"][0]["solution"] = json!(["lvs"]),
            4 => {
                bad["scenarios"][0]["goal"]["definition"]["plan"]["groups"][0]["devices"] =
                    json!(["/dev/shareda"])
            }
            5 => {
                bad["scenarios"][0]["goal"]["definition"]["lesson"]["worked"][1]["text"] =
                    json!("lvcreate --type thin -L 1M -n unsupported workshop")
            }
            _ => bad["scenarios"][0]["initial"]["simulation"]["view"] = json!("Introduction"),
        }
        let error = combined(&bad.to_string())
            .err()
            .unwrap_or_else(|| panic!("mode{mode} accepted"));
        assert!(
            !error.contains("invalid prerequisites"),
            "mode{mode} must fail for its actual defect, not a missing test prerequisite: {error}"
        );
    }
}

#[test]
fn registered315_preserves_every312_scenario105_units_and_prerequisite_order() {
    let before = pack::accepted312().unwrap();
    let all = pack::accepted315().unwrap();
    assert_eq!((before.len(), all.len()), (312, 315));
    assert_eq!(
        serde_json::to_value(&all[..312]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(
        all[312..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 111, 53, 77]
    );
    for s in &all[312..] {
        let at = all.iter().position(|x| x.id == s.id).unwrap();
        for id in &s.prerequisites {
            assert!(all.iter().position(|x| &x.id == id).unwrap() < at);
        }
    }
    let old_units = dlms_learning_lab::course::accepted312(&before).unwrap();
    let units = dlms_learning_lab::course::accepted315(&all).unwrap();
    assert_eq!((old_units.len(), units.len()), (105, 106));
    assert_eq!(
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "lvm-linear-provisioning")
                .collect::<Vec<_>>()
        )
        .unwrap(),
        serde_json::to_value(&old_units).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| units.iter().filter(|u| u.track == t).count()),
        [20, 43, 17, 26]
    );
    let at = units
        .iter()
        .position(|u| u.id == "lvm-linear-provisioning")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    assert!(
        units
            .iter()
            .position(|u| u.id == "lvm-volume-growth")
            .unwrap()
            < at
    );
}
