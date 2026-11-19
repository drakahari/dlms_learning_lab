//! Actual authored filesystem tasks; outcomes, preservation and safe progress.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    fs_integrity_workspace as w, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const IDS: [&str; 2] = ["lpic1-fs-offline-accounting", "lpic1-fs-retained-recovery"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn labs() -> impl Iterator<Item = &'static pack::Scenario> {
    catalog().iter().filter(|s| IDS.contains(&s.id.as_str()))
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::FsIntegrity { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::FsIntegrity { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 7);
    workspace::prepare(&mut a);
    a
}
fn commands(s: &pack::Scenario) -> Vec<String> {
    s.solution
        .iter()
        .filter_map(|raw| {
            match serde_json::from_str(raw.strip_prefix(w::PREFIX).unwrap()).unwrap() {
                w::Action::Command { input } => Some(input),
                w::Action::Check {} => None,
            }
        })
        .collect()
}
#[test]
fn authored_solutions_and_alternate_spellings_preserve_actual_data_across_three_cases() {
    for s in labs() {
        let d = definition(s);
        d.validate().unwrap();
        assert_eq!(d.assignment.cases.len(), 3);
        let recipe = commands(s);
        let rows = d.assignment.run(&recipe).unwrap();
        assert!(rows.iter().all(w::CaseResult::passed), "{} {rows:?}", s.id);
        for (c, r) in d.assignment.cases.iter().zip(&rows) {
            let actual = r.actual.as_ref().unwrap();
            assert_eq!(actual.disks[1], c.initial.disks[1]);
            assert!(actual.disks[0].volume.findings().unwrap().is_empty());
            assert_eq!(
                actual.disks[0].volume.files[&12].content,
                c.initial.disks[0].volume.files[&12].content
            );
        }
        let alt: Vec<_> = recipe
            .iter()
            .map(|c| {
                c.replace("e2fsck -fp", "fsck.ext4 -p -f")
                    .replace("e2fsck -fy", "e2fsck -y -f")
                    .replace("e2fsck -fn", "fsck.ext4 -n -f")
            })
            .collect();
        assert!(
            d.assignment
                .run(&alt)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let mut a = fresh(s);
        assert!(!progress::current_solved(&a, s));
        for line in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
        }
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn diagnostic_only_wrong_device_mounted_repair_and_partial_recovery_do_not_pass() {
    for s in labs() {
        let d = definition(s);
        for recipe in [
            vec!["findmnt".into()],
            vec![
                "umount /mnt/data".into(),
                "e2fsck -fn /dev/vdb1".into(),
                "mount /dev/vdb1 /mnt/data".into(),
            ],
            vec!["e2fsck -fy /dev/vdb1".into()],
            vec![
                "umount /mnt/notes".into(),
                "e2fsck -fp /dev/vdc1".into(),
                "mount /dev/vdc1 /mnt/notes".into(),
            ],
        ] {
            let rows = d.assignment.run(&recipe).unwrap();
            assert!(rows.iter().all(|r| !r.passed()), "{} {rows:?}", s.id);
        }
        let mut good = commands(s);
        good.pop();
        let rows = d.assignment.run(&good).unwrap();
        assert!(rows.iter().all(|r| !r.passed()
            && r.checks.iter().any(|c| c.0)
            && r.checks.iter().any(|c| !c.0 && c.1.contains("mount"))));
        if s.id == IDS[1] {
            let preen: Vec<_> = commands(s)
                .iter()
                .map(|c| c.replace("-fy", "-fp"))
                .collect();
            let rows = d.assignment.run(&preen).unwrap();
            assert!(rows.iter().all(|r| {
                !r.passed()
                    && r.checks
                        .iter()
                        .any(|c| !c.0 && c.1.contains("recovery decision"))
            }));
        }
        let mut good = commands(s);
        good.push("umount /mnt/notes".into());
        assert!(d.assignment.run(&good).unwrap().iter().all(|r| !r.passed()));
    }
}
#[test]
fn separate_examples_drafts_resume_backup_restore_reset_and_stale_success_are_honest() {
    let ls: Vec<_> = labs().cloned().collect();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &ls {
        let mut a = fresh(s);
        let d = definition(s);
        let original = serde_json::to_value(&session(&mut a).assignment).unwrap();
        for command in &d.example_commands {
            session(&mut a)
                .example
                .execute_recorded(&d.example, command)
                .unwrap();
        }
        assert!(
            session(&mut a)
                .example
                .run_checked(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert!(!progress::current_solved(&a, s));
        assert_eq!(
            serde_json::to_value(&session(&mut a).assignment).unwrap(),
            original
        );
        for line in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, line).unwrap();
        }
        session(&mut a).assignment.command = "cat /mnt/data/report.txt".into();
        session(&mut a).example.command = "ls -li /mnt/workshop/lost+found".into();
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &ls).unwrap();
    let mut restored = progress::load(&path, &ls).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for s in &ls {
        let a = restored.attempts.get_mut(&s.id).unwrap();
        assert!(
            !progress::current_solved(a, s),
            "Serialized stamp cannot grant mastery"
        );
        let d = definition(s);
        assert!(session(a).assignment.pending_recheck());
        session(a).assignment.run_checked(&d.assignment).unwrap();
        assert!(progress::current_solved(a, s));
        session(a)
            .assignment
            .execute_recorded(&d.assignment, "findmnt")
            .unwrap();
        assert!(progress::current_solved(a, s));
        session(a)
            .assignment
            .execute_recorded(&d.assignment, "umount /mnt/data")
            .unwrap();
        assert!(!progress::current_solved(a, s));
        session(a).assignment.run_checked(&d.assignment).unwrap();
        assert!(!progress::current_solved(a, s));
        session(a)
            .assignment
            .execute_recorded(&d.assignment, "mount /dev/vdb1 /mnt/data")
            .unwrap();
        session(a).assignment.run_checked(&d.assignment).unwrap();
        assert!(progress::current_solved(a, s));
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &ls).unwrap();
    let inspected =
        backup::inspect(&export, &ls, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    backup::restore(&path, &inspected, &ls).unwrap();
    let retained = serde_json::to_value(&p.attempts[IDS[1]]).unwrap();
    p.attempts.insert(IDS[0].into(), fresh(&ls[0]));
    assert_eq!(serde_json::to_value(&p.attempts[IDS[1]]).unwrap(), retained);
    assert!(p.favorites.contains(IDS[0]));
    progress::save_with_catalog(&path, &p, &ls).unwrap();
    assert!(!progress::current_solved(
        &progress::load(&path, &ls).unwrap().attempts[IDS[0]],
        &ls[0]
    ));
}
#[test]
fn retain_all265_values_and78_units_and_insert_after_real_mount_prerequisites() {
    let prior = pack::accepted265().unwrap();
    let snapshot = pack::accepted267().unwrap();
    let now = &snapshot;
    assert_eq!((prior.len(), now.len()), (265, 267));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..265]).unwrap()
    );
    let old = course::accepted265(&prior).unwrap();
    let current = course::accepted267(now).unwrap();
    assert_eq!((old.len(), current.len()), (78, 79));
    let filtered: Vec<_> = current
        .iter()
        .filter(|u| u.id != "linux-filesystem-consistency")
        .collect();
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(filtered).unwrap()
    );
    let at = current
        .iter()
        .position(|u| u.id == "linux-removable-callers")
        .unwrap();
    let links = current
        .iter()
        .position(|u| u.id == "linux-link-handover")
        .unwrap();
    let new = current
        .iter()
        .position(|u| u.id == "linux-filesystem-consistency")
        .unwrap();
    assert_eq!(new, at.max(links) + 1);
    assert_eq!(
        current[new].prerequisites,
        ["linux-removable-callers", "linux-link-handover"]
    );
    assert_eq!(current[new].labs, IDS);
    assert_eq!(
        now[265].prerequisites,
        ["lpic1-links-handover", "lpic1-removable-shared-handover"]
    );
    assert_eq!(now[266].prerequisites, [IDS[0]]);
}
