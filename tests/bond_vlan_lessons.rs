//! Authored bonded-VLAN capstone, source preservation and concrete progress.
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
const ID: &str = "lpic2-bonded-vlan-handover";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted341().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
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
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 7);
    workspace::prepare(&mut a);
    w::context(&a, lab()).unwrap();
    a
}
#[test]
fn standalone_authored_source_and_taught_solution_are_reachable_before_catalog_loading() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/bond-vlan-lessons-v1.json")).unwrap();
    let s: pack::Scenario = serde_json::from_value(data["scenarios"][0].clone()).unwrap();
    let d = w::source(&s).unwrap();
    assert_eq!(d.initial.cases.len(), 3);
    assert!(
        d.initial
            .run(&solution(&s))
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
    assert_eq!(d.example_worked.len(), 5);
    assert_eq!(
        w::Session::new(d).unwrap().assignment.selected,
        "bond.nmconnection"
    );
}
#[test]
fn all340_existing_scenarios_and118_units_remain_exact_with_one_distinct_capstone() {
    let now = catalog();
    let old = pack::accepted340().unwrap();
    assert_eq!((old.len(), now.len()), (340, 341));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..340]).unwrap()
    );
    let prior = course::accepted340(&old).unwrap();
    let units = course::accepted341(now).unwrap();
    assert_eq!((prior.len(), units.len()), (118, 119));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-bonded-vlan")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let n = units
        .iter()
        .position(|u| u.id == "linux-bonded-vlan")
        .unwrap();
    assert_eq!(units[n - 1].id, "linux-vlan-profiles");
    assert_eq!(units[n].labs, [ID]);
    assert_eq!(lab().prerequisites, ["lpic2-vlan-return-handover"]);
    for (track, count) in [
        ("CCNA", 74),
        ("LPIC-1", 114),
        ("LPIC-2", 76),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn genuine_zero_partial_and_complete_cases_explain_binding_and_preserve_current_snapshots() {
    let s = lab();
    let d = definition(s);
    let rows = d.initial.run(&d.initial.starter).unwrap();
    assert_eq!(rows.iter().filter(|r| r.passed()).count(), 0);
    assert!(rows.iter().all(|r| r.checks.iter().any(|c| c.0)));
    let mut partial = d.initial.starter.clone();
    let text = partial.get_mut("bond.nmconnection").unwrap();
    *text = text.replace("autoconnect-ports=0", "autoconnect-ports=1");
    let rows = d.initial.run(&partial).unwrap();
    assert_eq!(
        rows.iter().map(w::CaseResult::passed).collect::<Vec<_>>(),
        [true, false, true]
    );
    assert!(
        rows[1]
            .checks
            .iter()
            .any(|c| !c.0 && c.1.contains("no selected monitored member"))
    );
    for row in rows {
        let current = row.current.unwrap();
        assert_eq!(current.controls().unwrap().current.members.len(), 2);
    }
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    assert!(progress::current_solved(&a, s));
}
#[test]
fn supported_alternative_primary_port_flags_and_report_address_are_graded_by_behavior() {
    let s = lab();
    let d = definition(s);
    let mut alt = solution(s);
    let text = alt.get_mut("bond.nmconnection").unwrap();
    *text = text
        .replace("primary=enp1s0", "primary=enp2s0")
        .replace("mode=active-backup", "mode=1")
        .replace("primary_reselect=always", "primary_reselect=0");
    let text = alt.get_mut("port-enp2s0.nmconnection").unwrap();
    *text = text.replace("autoconnect=false", "autoconnect=true");
    let text = alt.get_mut("reports.nmconnection").unwrap();
    *text = text.replace("10.180.0.20/24", "10.180.0.21/24");
    for text in alt.values_mut() {
        *text = format!(
            "# equivalent supported configuration\n{}",
            text.replace('=', " = ")
        );
    }
    assert!(
        d.initial
            .run(&alt)
            .unwrap()
            .iter()
            .all(w::CaseResult::passed)
    );
}
#[test]
fn wrong_tags_duplicate_peer_address_missing_binding_and_audit_changes_fail() {
    let s = lab();
    let d = definition(s);
    for (file, from, to) in [
        ("reports.nmconnection", "id=30\n", "id=99\n"),
        ("reports.nmconnection", "10.180.0.20/24", "10.180.0.50/24"),
        ("reports.nmconnection", "10.180.0.20/24", "10.99.0.20/24"),
        (
            "port-enp2s0.nmconnection",
            "controller=bond0\nport-type=bond\n",
            "",
        ),
        (
            "audit.nmconnection",
            "never-default=true",
            "never-default=false",
        ),
    ] {
        let mut fs = solution(s);
        let text = fs.get_mut(file).unwrap();
        *text = text.replace(from, to);
        assert!(
            d.initial.run(&fs).unwrap().iter().any(|r| !r.passed()),
            "{file} {to}"
        );
    }
    let mut fs = solution(s);
    let text = fs.get_mut("reports.nmconnection").unwrap();
    *text = text.replace("10.180.0.20/24", "10.180.0.50/24");
    assert!(
        d.initial
            .run(&fs)
            .unwrap()
            .iter()
            .all(|r| r.checks.iter().any(|c| !c.0 && c.1.contains("conflicts")))
    );
}
#[test]
fn independent_port_startup_and_unmodeled_syntax_remain_explicit_diagnostics() {
    let d = definition(lab());
    for (file, text) in [
        ("bond.nmconnection", "[bond\nmode=active-backup"),
        ("bond.nmconnection", "[bond]\nmode=802.3ad\n"),
    ] {
        let mut fs = solution(lab());
        fs.insert(file.into(), text.into());
        assert!(
            d.initial
                .run(&fs)
                .unwrap()
                .iter()
                .all(|r| !r.passed() && r.diagnostic.is_some())
        );
    }
    let mut fs = d.initial.starter.clone();
    let text = fs.get_mut("port-enp2s0.nmconnection").unwrap();
    *text = text.replace("autoconnect=false", "autoconnect=true");
    assert!(d.initial.run(&fs).unwrap().iter().all(|r| {
        r.diagnostic
            .as_ref()
            .is_some_and(|s| s.contains("Independent port-driven"))
    }));
}
#[test]
fn separate_example_immutable_assignment_and_stale_success_have_real_drafts() {
    let s = lab();
    let d = definition(s);
    let mut a = fresh();
    let before = sim(&mut a).assignment.clone();
    sim(&mut a).practice.replace(&d.example_worked).unwrap();
    sim(&mut a).practice.run(&d.example).unwrap();
    assert!(sim(&mut a).practice.passed(&d.example));
    assert_eq!(sim(&mut a).assignment, before);
    assert!(!progress::current_solved(&a, s));
    sim(&mut a).assignment.replace(&solution(s)).unwrap();
    sim(&mut a).assignment.run(&d.initial).unwrap();
    assert!(progress::current_solved(&a, s));
    sim(&mut a).assignment.replace(&d.initial.starter).unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(sim(&mut a).assignment.tested.is_none());
    sim(&mut a).assignment.replace(&solution(s)).unwrap();
    assert!(!progress::current_solved(&a, s));
    sim(&mut a).assignment.run(&d.initial).unwrap();
    assert!(progress::current_solved(&a, s));
}
#[test]
fn safe_storage_backup_reset_and_resume_keep_all_five_files_and_other_records() {
    let s = lab();
    let mut a = fresh();
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    sim(&mut a)
        .practice
        .select("port-enp2s0.nmconnection")
        .unwrap();
    sim(&mut a)
        .practice
        .edit("port-enp2s0.nmconnection", "# unfinished example")
        .unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    p.attempts.insert(ID.into(), a);
    p.favorites.insert(ID.into());
    let old_source = catalog()
        .iter()
        .find(|s| s.id == "lpic2-vlan-return-handover")
        .unwrap();
    let mut old = Attempt::new(old_source, 5);
    workspace::prepare(&mut old);
    sim(&mut old)
        .assignment
        .edit(
            "reports.nmconnection",
            "# preserved unfinished pre-bond draft",
        )
        .unwrap();
    assert!(!progress::current_solved(&old, old_source));
    let old_record = serde_json::to_value(&old).unwrap();
    p.attempts.insert(old_source.id.clone(), old);
    p.favorites.insert(old_source.id.clone());
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let raw = std::fs::read(&path).unwrap();
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert!(progress::current_solved(&restored.attempts[ID], s));
    let backup_path = dir.path().join("backup.json");
    backup::export(&backup_path, &p, catalog()).unwrap();
    let inspected = backup::inspect(
        &backup_path,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let receipt = backup::restore(&path, &inspected, catalog()).unwrap();
    assert_eq!(std::fs::read(receipt.safety_copy.unwrap()).unwrap(), raw);
    assert!(!progress::current_solved(&fresh(), s));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&old_source.id]).unwrap(),
        old_record
    );
    p.attempts.insert(ID.into(), fresh());
    assert_eq!(
        serde_json::to_value(&p.attempts[&old_source.id]).unwrap(),
        old_record
    );
    assert!(p.favorites.contains(ID) && p.favorites.contains(&old_source.id));
}
