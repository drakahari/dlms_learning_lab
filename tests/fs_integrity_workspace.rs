#[path = "support/fs_integrity.rs"]
mod fixture;
use dlms_learning_lab::fs_integrity_workspace::{
    CaseResult, Definition, MAX_TICKS, Requirement, Run,
};
#[test]
fn real_multi_case_repairs_and_supported_alternatives_pass_state_outcomes() {
    for recovery in [false, true] {
        let d = fixture::definition(recovery);
        d.validate().unwrap();
        let commands = fixture::recipe(&d.assignment, recovery);
        let rows = d.assignment.run(&commands).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(CaseResult::passed), "{rows:?}");
        for (case, row) in d.assignment.cases.iter().zip(&rows) {
            let state = row.actual.as_ref().unwrap();
            assert!(state.disks[0].volume.findings().unwrap().is_empty());
            assert_eq!(state.disks[1], case.initial.disks[1]);
        }
        let alternate: Vec<_> = commands
            .iter()
            .map(|c| {
                c.replace("e2fsck -fn", "fsck.ext4 -n -f")
                    .replace("e2fsck -fp", "e2fsck -p -f")
                    .replace("e2fsck -fy", "fsck.ext4 -y -f")
            })
            .collect();
        assert!(
            d.assignment
                .run(&alternate)
                .unwrap()
                .iter()
                .all(CaseResult::passed)
        );
    }
}
#[test]
fn queries_mounting_alone_and_preening_an_orphan_fail_with_partial_specific_feedback() {
    for recovery in [false, true] {
        let d = fixture::definition(recovery);
        let first = &d.assignment.cases[0].initial.disks[0];
        for commands in [
            vec!["findmnt".into()],
            vec![
                format!("umount {}", first.mountpoint),
                format!("mount {} {}", first.device, first.mountpoint),
            ],
        ] {
            let rows = d.assignment.run(&commands).unwrap();
            assert!(rows.iter().all(|r| !r.passed()));
            assert!(
                rows.iter()
                    .all(|r| r.checks.iter().any(|c| c.0) && r.checks.iter().any(|c| !c.0))
            );
        }
        if recovery {
            let rows = d
                .assignment
                .run(&fixture::recipe(&d.assignment, false))
                .unwrap();
            assert!(rows.iter().all(|r| !r.passed()));
            assert!(rows.iter().all(|r| {
                r.checks
                    .iter()
                    .any(|(ok, why)| !ok && why.contains("recovery decision"))
            }));
        }
        let mut commands = fixture::recipe(&d.assignment, recovery);
        commands.push("umount /mnt/notes".into());
        let rows = d.assignment.run(&commands).unwrap();
        assert!(rows.iter().all(|r| {
            r.checks
                .iter()
                .any(|(ok, why)| !ok && why.contains("separate volume"))
        }));
    }
}
#[test]
fn cooperative_cancel_unfinished_jobs_and_bad_commands_cannot_publish_success() {
    let d = fixture::definition(true);
    let history = fixture::recipe(&d.assignment, true);
    let mut run = Run::new(&d.assignment, &history).unwrap();
    assert!(run.results().is_none());
    for _ in 0..3 {
        assert!(!run.tick());
    }
    run.cancel();
    for _ in 0..MAX_TICKS {
        assert!(!run.tick());
    }
    assert!(!run.done());
    assert!(run.results().is_none());
    let mut fresh = Run::new(&d.assignment, &history).unwrap();
    let mut ticks = 0;
    while ticks < MAX_TICKS && !fresh.tick() {
        ticks += 1;
    }
    assert!(fresh.done());
    assert!(fresh.results().unwrap().iter().all(CaseResult::passed));
    assert_eq!(fresh.history(), history);
    assert_eq!(fresh.task(), &d.assignment);
    assert!(Run::new(&d.assignment, &["sh -c anything".into()]).is_err());
    let rows = d.assignment.run(&["e2fsck -fy /dev/vdb1".into()]).unwrap();
    assert!(
        rows.iter()
            .all(|r| !r.passed() && r.diagnostic.as_ref().unwrap().contains("Unmount"))
    );
}
#[test]
fn source_definitions_refuse_unknown_goals_overlapping_practice_and_unreachable_examples() {
    let d = fixture::definition(false);
    d.validate().unwrap();
    let mut bad = d.clone();
    bad.assignment.cases[0]
        .requirements
        .push(Requirement::Healthy {
            device: "/dev/not-supplied".into(),
        });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    let duplicate = bad.assignment.cases[0].requirements[0].clone();
    bad.assignment.cases[0].requirements.push(duplicate);
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example_commands = vec!["findmnt".into()];
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example = d.assignment.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.assignment.cases[1].initial.disks[0].mountpoint = "/mnt/different".into();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    for c in &mut bad.assignment.cases {
        c.initial.disks[0].volume.mounted = false;
        c.initial.disks[0]
            .volume
            .check(dlms_learning_lab::fs_integrity::Mode::Preen)
            .unwrap();
        c.initial.disks[0].volume.mounted = true;
    }
    assert!(bad.validate().is_err());
    let raw = serde_json::to_string(&d).unwrap();
    assert_eq!(serde_json::from_str::<Definition>(&raw).unwrap(), d);
    let bad = raw.replacen("\"assignment\":{", "\"assignment\":{\"run_host\":true,", 1);
    assert!(serde_json::from_str::<Definition>(&bad).is_err());
}
