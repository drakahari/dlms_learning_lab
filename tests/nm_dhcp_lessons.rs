//! Authored automatic-profile assignments, independent practice and retained work.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    nm_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const RAW: &str = include_str!("../scenarios/linux-nm-dhcp-lessons-v1.json");
const IDS: [&str; 2] = ["lpic1-nm-dhcp-startup", "lpic1-nm-dhcp-local-only"];
fn labs() -> Vec<pack::Scenario> {
    pack::load_catalog(&[include_str!("../scenarios/linux-nm-lessons-v1.json"), RAW])
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "linux-automatic-ethernet")
        .collect()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::NmProfiles { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn state(a: &mut Attempt) -> &mut w::Session {
    let State::NmProfiles { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solution(s: &pack::Scenario) -> w::Files {
    serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    a
}

#[test]
fn complete_assignments_use_each_lease_and_typed_outcomes_not_exact_keyfile_strings() {
    for s in labs() {
        let d = definition(&s);
        d.validate().unwrap();
        assert_eq!(d.initial.cases.len(), 3);
        let mut a = fresh(&s);
        assert!(!progress::current_solved(&a, &s));
        let results = state(&mut a).assignment.run(&d.initial).unwrap();
        assert!(results.iter().all(|r| !r.passed()));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, &s));
        let rows = d.initial.run(&solution(&s)).unwrap();
        for (c, r) in d.initial.cases.iter().zip(&rows) {
            assert!(r.passed(), "{}: {r:?}", s.id);
            let future = r.future.as_ref().unwrap();
            assert!(future.uses_dhcp("enp1s0"));
            let dlms_learning_lab::nm_dhcp::Outcome::Bound { address, router } =
                &c.environment.dhcp["enp1s0"]
            else {
                panic!()
            };
            let actual = future.for_device("enp1s0").unwrap();
            assert_eq!(&actual.address, address);
            assert_eq!(actual.gateway, if s.id == IDS[1] { None } else { *router });
        }
        let mut equivalent = solution(&s);
        for text in equivalent.values_mut() {
            *text = format!(
                "# equivalent supported keyfile\r\n{}",
                text.replace("type=ethernet", "type=802-3-ethernet")
                    .replace('=', " = ")
                    .replace('\n', "\r\n")
            );
        }
        assert!(
            d.initial
                .run(&equivalent)
                .unwrap()
                .iter()
                .all(|r| r.passed())
        );
    }
}

#[test]
fn plausible_wrong_repairs_partial_states_and_edits_fail_with_specific_feedback() {
    for s in labs() {
        let d = definition(&s);
        let good = solution(&s);
        let mut wrong = good.clone();
        let text = if s.id == IDS[0] {
            let dlms_learning_lab::nm_dhcp::Outcome::Bound { address, router } =
                &d.initial.cases[0].environment.dhcp["enp1s0"]
            else {
                panic!()
            };
            wrong["app.nmconnection"].replace(
                "method=auto",
                &format!(
                    "method=manual\naddress1={}/{}\ngateway={}",
                    address.address,
                    address.prefix,
                    router.unwrap()
                ),
            )
        } else {
            wrong["app.nmconnection"]
                .replace("never-default=true", "never-default=false")
                .replace("route-metric=100", "route-metric=400")
        };
        wrong.insert("app.nmconnection".into(), text);
        let failed = d.initial.run(&wrong).unwrap();
        assert!(failed.iter().all(|r| !r.passed()));
        assert!(
            failed
                .iter()
                .all(|r| r.checks.iter().any(|c| c.0) && r.checks.iter().any(|c| !c.0))
        );
        assert!(failed.iter().any(|r| r.checks.iter().any(|(ok, why)| !ok
            && why.contains(if s.id == IDS[0] {
                "manual address"
            } else {
                "Raising a metric"
            }))));
        let mut damaged = good.clone();
        let x = damaged["management.nmconnection"].replace("route-metric=200", "route-metric=201");
        damaged.insert("management.nmconnection".into(), x);
        assert!(d.initial.run(&damaged).unwrap().iter().all(|r| {
            r.checks
                .iter()
                .any(|(ok, why)| !ok && why.contains("protected profile"))
        }));
        let mut a = fresh(&s);
        state(&mut a).assignment.replace(&good).unwrap();
        state(&mut a).assignment.run(&d.initial).unwrap();
        assert!(progress::current_solved(&a, &s));
        state(&mut a).assignment.replace(&wrong).unwrap();
        assert!(!progress::current_solved(&a, &s));
        state(&mut a).assignment.replace(&good).unwrap();
        assert!(!progress::current_solved(&a, &s));
        state(&mut a).assignment.run(&d.initial).unwrap();
        assert!(progress::current_solved(&a, &s));
        let text = good["app.nmconnection"].replace("[ipv4]", "[ipv4]\ndns=10.1.0.53");
        state(&mut a)
            .assignment
            .edit("app.nmconnection", &text)
            .unwrap();
        let rows = state(&mut a).assignment.run(&d.initial).unwrap();
        assert!(
            rows.iter()
                .all(|r| r.diagnostic.as_ref().unwrap().contains("Unsupported"))
        );
        assert!(!progress::current_solved(&a, &s));
    }
}

#[test]
fn taught_workshops_and_experiments_are_real_separate_and_repairable() {
    for s in labs() {
        let d = definition(&s);
        let mut a = fresh(&s);
        let untouched = state(&mut a).assignment.clone();
        state(&mut a).practice.replace(&d.example_worked).unwrap();
        assert!(state(&mut a).practice.tested.is_none());
        assert!(!progress::current_solved(&a, &s));
        let rows = state(&mut a).practice.run(&d.example).unwrap();
        assert!(rows.iter().all(|r| r.passed()));
        assert_eq!(state(&mut a).assignment, untouched);
        assert!(!progress::current_solved(&a, &s));
        let mut experiment = d.example_worked.clone();
        let x = if s.id == IDS[0] {
            let dlms_learning_lab::nm_dhcp::Outcome::Bound { address, router } =
                &d.example.cases[0].environment.dhcp["enp1s0"]
            else {
                panic!()
            };
            experiment["app.nmconnection"].replace(
                "method=auto",
                &format!(
                    "method=manual\naddress1={}/{}\ngateway={}",
                    address.address,
                    address.prefix,
                    router.unwrap()
                ),
            )
        } else {
            experiment["app.nmconnection"]
                .replace("never-default=true", "never-default=false")
                .replace("route-metric=100", "route-metric=400")
        };
        experiment.insert("app.nmconnection".into(), x);
        state(&mut a).practice.replace(&experiment).unwrap();
        assert!(
            state(&mut a)
                .practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(|r| !r.passed())
        );
        state(&mut a).practice.replace(&d.example_worked).unwrap();
        assert!(
            state(&mut a)
                .practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(|r| r.passed())
        );
        assert_eq!(state(&mut a).assignment, untouched);
    }
}

#[test]
fn separate_drafts_results_favorites_backup_and_selected_reset_survive_safe_storage() {
    let labs = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &labs {
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let d = definition(s);
        state(&mut a).practice.replace(&d.example_worked).unwrap();
        state(&mut a)
            .practice
            .edit(
                "app.nmconnection",
                &(d.example_worked["app.nmconnection"].clone() + "# unfinished independent note\n"),
            )
            .unwrap();
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let got = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&got).unwrap()
    );
    assert!(
        labs.iter()
            .all(|s| progress::current_solved(&got.attempts[&s.id], s))
    );
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &labs).unwrap();
    let candidate = backup::inspect(
        &export,
        &labs,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    backup::restore(&path, &candidate, &labs).unwrap();
    let retained = serde_json::to_value(&p.attempts[IDS[1]]).unwrap();
    p.attempts.insert(IDS[0].into(), fresh(&labs[0]));
    assert_eq!(serde_json::to_value(&p.attempts[IDS[1]]).unwrap(), retained);
    assert!(p.favorites.contains(IDS[0]));
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    assert!(!progress::current_solved(
        &progress::load(&path, &labs).unwrap().attempts[IDS[0]],
        &labs[0]
    ));
}

#[test]
fn retain263_scenario_values_and77_units_then_insert_one_prerequisite_ordered_unit() {
    let prior = pack::accepted263().unwrap();
    let now = pack::accepted265().unwrap();
    assert_eq!((prior.len(), now.len()), (263, 265));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..263]).unwrap()
    );
    let old = course::accepted263(&prior).unwrap();
    let current = course::accepted265(&now).unwrap();
    assert_eq!((old.len(), current.len()), (77, 78));
    let filtered: Vec<_> = current
        .iter()
        .filter(|u| u.id != "linux-automatic-ethernet")
        .collect();
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(filtered).unwrap()
    );
    let before = current
        .iter()
        .position(|u| u.id == "linux-persistent-networking")
        .unwrap();
    assert_eq!(current[before + 1].id, "linux-automatic-ethernet");
    assert_eq!(current[before + 1].labs, IDS);
    assert_eq!(now[263].prerequisites, ["lpic1-nm-uplink-handover"]);
    assert_eq!(now[264].prerequisites, [IDS[0]]);
}
