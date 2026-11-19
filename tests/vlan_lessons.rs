//! Authored VLAN interactions and additive catalog/progress compatibility.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    nm_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const IDS: [&str; 2] = [
    "lpic2-vlan-profile-commission",
    "lpic2-vlan-return-handover",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted340().unwrap())
}
fn labs() -> impl Iterator<Item = &'static pack::Scenario> {
    catalog().iter().filter(|s| IDS.contains(&s.id.as_str()))
}
fn d(s: &pack::Scenario) -> &w::Definition {
    let Goal::NmProfiles { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::NmProfiles { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solution(s: &pack::Scenario) -> w::Files {
    serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 8);
    workspace::prepare(&mut a);
    a
}
#[test]
fn all338_prior_scenarios_and117_units_remain_exact_with_independent_vlan_requirements() {
    let now = catalog();
    let old = pack::accepted338().unwrap();
    assert_eq!((old.len(), now.len()), (338, 340));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..338]).unwrap()
    );
    let prior = course::accepted338(&old).unwrap();
    let units = course::accepted340(now).unwrap();
    assert_eq!((prior.len(), units.len()), (117, 118));
    let retained = units
        .iter()
        .filter(|u| u.id != "linux-vlan-profiles")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let n = units
        .iter()
        .position(|u| u.id == "linux-vlan-profiles")
        .unwrap();
    assert_eq!(units[n - 1].id, "linux-persistent-networking");
    assert_eq!(units[n].labs, IDS);
    for (track, count) in [
        ("CCNA", 74),
        ("LPIC-1", 114),
        ("LPIC-2", 75),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn taught_repair_separate_practice_and_supported_alternatives_reach_all_cases() {
    for s in labs() {
        let d = d(s);
        d.validate().unwrap();
        let mut a = fresh(s);
        assert!(!progress::current_solved(&a, s));
        let start = sim(&mut a).assignment.clone();
        sim(&mut a).practice.replace(&d.example_worked).unwrap();
        sim(&mut a).practice.run(&d.example).unwrap();
        assert!(sim(&mut a).practice.passed(&d.example));
        assert_eq!(sim(&mut a).assignment, start);
        assert!(!progress::current_solved(&a, s));
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        let mut alt = solution(s);
        let t = alt.get_mut("reports.nmconnection").unwrap();
        *t = t.replace("protocol=802.1Q\n", "").replace('=', " = ");
        t.push_str("\n# Equivalent supported settings\n");
        if s.id == IDS[1] {
            *t = t.replace("10.154.0.20/24", "10.154.0.21/24");
        }
        assert!(
            d.initial
                .run(&alt)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
    }
}
#[test]
fn wrong_ip_tag_parent_and_partial_work_produce_specific_failure_rows() {
    for s in labs() {
        let d = d(s);
        let good = solution(s);
        let rows = d.initial.run(&d.initial.starter).unwrap();
        assert!(rows.iter().all(|r| !r.passed()));
        assert!(rows[0].checks.iter().any(|r| r.0));
        for (from, to) in [("id=30\n", "id=99\n"), ("parent=enp1s0", "parent=enp2s0")] {
            let mut fs = good.clone();
            let t = fs.get_mut("reports.nmconnection").unwrap();
            *t = t.replace(from, to);
            let rows = d.initial.run(&fs).unwrap();
            assert!(!rows[0].passed());
            if s.id == IDS[0] && from == "parent=enp1s0" {
                // The second supplied switch uses native88: tag30 returns
                // tagged there. Preserve this real per-case success while
                // the native30 case still prevents overall completion.
                assert!(rows[1].passed());
            } else {
                assert!(!rows[1].passed());
            }
            assert!(
                rows.iter()
                    .any(|r| r.checks.iter().any(|c| !c.0 && c.1.contains("outward")))
            );
        }
        let mut fs = good;
        let t = fs.get_mut("reports.nmconnection").unwrap();
        *t = t.replace("address1=10.", "address1=192.");
        assert!(d.initial.run(&fs).unwrap().iter().all(|r| !r.passed()));
    }
}
#[test]
fn unsupported_keyfiles_and_audit_edits_are_not_false_positive_success() {
    for s in labs() {
        for (file, from, to) in [
            (
                "reports.nmconnection",
                "protocol=802.1Q",
                "protocol=802.1ad",
            ),
            ("reports.nmconnection", "flags=1", "flags=0"),
            (
                "audit.nmconnection",
                "never-default=true",
                "never-default=false",
            ),
        ] {
            let mut fs = solution(s);
            let t = fs.get_mut(file).unwrap();
            *t = t.replace(from, to);
            let rows = d(s).initial.run(&fs).unwrap();
            assert!(rows.iter().all(|r| !r.passed()));
        }
    }
}
#[test]
fn successful_drafts_are_invalidated_on_edit_and_restoring_text_requires_recheck() {
    for s in labs() {
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
        sim(&mut a)
            .assignment
            .replace(&d(s).initial.starter)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        sim(&mut a).assignment.replace(&solution(s)).unwrap();
        assert!(!progress::current_solved(&a, s));
        sim(&mut a).assignment.run(&d(s).initial).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn progress_backup_resume_reset_and_original_records_remain_isolated() {
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    for s in labs() {
        let mut a = fresh(s);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        sim(&mut a)
            .practice
            .edit("reports.nmconnection", "# practice draft")
            .unwrap();
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let original = std::fs::read(&path).unwrap();
    let got = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&got).unwrap()
    );
    for s in labs() {
        assert!(progress::current_solved(&got.attempts[&s.id], s));
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, catalog()).unwrap();
    let inspected = backup::inspect(
        &export,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let receipt = backup::restore(&path, &inspected, catalog()).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    let kept = serde_json::to_value(&p.attempts[IDS[1]]).unwrap();
    let first = labs().next().unwrap();
    p.attempts.insert(first.id.clone(), fresh(first));
    assert_eq!(serde_json::to_value(&p.attempts[IDS[1]]).unwrap(), kept);
    assert!(!progress::current_solved(&p.attempts[IDS[0]], first));
}
#[test]
fn authored_source_and_progress_cannot_invent_capabilities_or_cross_task_identity() {
    for s in labs() {
        let d = d(s);
        let mut a = fresh(s);
        let mut forged = serde_json::to_value(sim(&mut a)).unwrap();
        forged["origin"] = "another-lab".into();
        assert!(w::Session::from_json(&forged.to_string(), d).is_err());
        let mut bad = d.clone();
        bad.initial.cases[1].environment.vlan = None;
        assert!(bad.validate().is_err());
        let mut fs = solution(s);
        fs.insert("../host.nmconnection".into(), "not read".into());
        assert!(d.initial.run(&fs).is_err());
    }
}

#[test]
fn individual_authored_definitions_and_native_starters_match_before_catalog_loading() {
    let value: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/linux-vlan-lessons-v1.json")).unwrap();
    for raw in value["scenarios"].as_array().unwrap() {
        let s: pack::Scenario = serde_json::from_value(raw.clone()).unwrap();
        w::source(&s).unwrap();
        assert!(
            d(&s)
                .initial
                .run(&solution(&s))
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let State::NmProfiles { simulation } = &s.initial else {
            panic!()
        };
        assert_eq!(simulation.assignment.selected, "reports.nmconnection");
        assert!(!d(&s).solved(simulation));
    }
}
