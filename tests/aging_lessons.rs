use dlms_learning_lab::{
    aging_workspace as w, backup, course,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};
const IDS: [&str; 3] = [
    "lpic1-aging-rotation",
    "lpic1-aging-grace",
    "lpic1-aging-contract-reset",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted346().unwrap())
}
fn lab(id: &str) -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    w::context(&a, s).unwrap();
    a
}
#[test]
fn all343_previous_labs_and121_units_are_exact_and_new_policy_tasks_follow_prerequisites() {
    let old = pack::accepted343().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (343, 346));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(
            now.iter()
                .filter(|s| !IDS.contains(&s.id.as_str()))
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let before = course::accepted343(&old).unwrap();
    let units = course::accepted346(now).unwrap();
    assert_eq!((before.len(), units.len()), (121, 122));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(
            units
                .iter()
                .filter(|u| u.id != "linux-password-aging")
                .collect::<Vec<_>>()
        )
        .unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-password-aging")
        .unwrap();
    assert_eq!(units[at - 1].id, "local-account-lifecycle");
    assert_eq!(units[at].labs, IDS);
    assert_eq!(units[at + 1].id, "linux-authentication-stacks");
    for (track, n, u) in [
        ("CCNA", 74, 20),
        ("LPIC-1", 117, 45),
        ("LPIC-2", 78, 31),
        ("Bash/Shell", 77, 26),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), n);
        assert_eq!(units.iter().filter(|x| x.track == track).count(), u);
    }
    assert_eq!(lab(IDS[0]).prerequisites, ["lpic1-account-team-handover"]);
    assert_eq!(lab(IDS[1]).prerequisites, [IDS[0]]);
    assert_eq!(lab(IDS[2]).prerequisites, [IDS[1]]);
}
#[test]
fn taught_and_alternate_methods_pass_while_plausible_breakages_and_partial_work_fail() {
    for (id, valid, wrong, partial) in [
        (
            IDS[0],
            "chage --maxdays=45 --warndays=10 morgan",
            "chage -d 2026-02-01 morgan",
            "chage -M 45 morgan",
        ),
        (
            IDS[1],
            "chage --inactive=7 river",
            "chage -M -1 river",
            "chage -I 6 river",
        ),
        (
            IDS[2],
            "chage --expiredate=2026-10-20 sage",
            "chage -d -1 sage",
            "chage -E 2026-10-21 sage",
        ),
    ] {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        assert!(!progress::current_solved(&a, s));
        let State::Aging { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.edit_input(partial).unwrap();
        simulation.assignment.execute(&d.initial).unwrap();
        let rows = simulation.assignment.check(&d.initial).unwrap();
        assert!(rows.iter().any(|r| r.0));
        assert!(rows.iter().any(|r| !r.0));
        assert!(!d.solved(simulation));
        for command in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
        }
        assert!(progress::current_solved(&a, s));
        let State::Aging { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.edit_input(wrong).unwrap();
        simulation.assignment.execute(&d.initial).unwrap();
        assert!(!d.solved(simulation));
        simulation.assignment.check(&d.initial).unwrap();
        assert!(!d.solved(simulation));
        a = fresh(s);
        let State::Aging { simulation } = &mut a.state else {
            panic!()
        };
        simulation.assignment.edit_input(valid).unwrap();
        simulation.assignment.execute(&d.initial).unwrap();
        simulation.assignment.check(&d.initial).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn all_new_drafts_practice_favorites_and_old_pam_work_survive_save_export_restore_and_selected_reset()
 {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for id in IDS {
        let s = lab(id);
        let d = w::source(s).unwrap();
        let mut a = fresh(s);
        for step in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, step).unwrap();
        }
        let State::Aging { simulation } = &mut a.state else {
            panic!()
        };
        simulation.fill_example(d).unwrap();
        simulation.practice.check(&d.example).unwrap();
        simulation
            .assignment
            .edit_input("chage -l retained_draft")
            .unwrap();
        p.attempts.insert(id.into(), a);
        p.favorites.insert(id.into());
    }
    let pam = lab("lpic2-pam-auth-bypass");
    let mut old = Attempt::new(pam, 5);
    workspace::prepare(&mut old);
    let State::PamStack { simulation } = &mut old.state else {
        panic!()
    };
    simulation
        .assignment
        .edit("# unfinished retained PAM draft")
        .unwrap();
    let old_value = serde_json::to_value(&old).unwrap();
    p.attempts.insert(pam.id.clone(), old);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let original = std::fs::read(&path).unwrap();
    let restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    for id in IDS {
        assert!(progress::current_solved(&restored.attempts[id], lab(id)));
    }
    let exported = dir.path().join("backup.json");
    backup::export(&exported, &p, catalog()).unwrap();
    let checked = backup::inspect(
        &exported,
        catalog(),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    let receipt = backup::restore(&path, &checked, catalog()).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    p.attempts.insert(IDS[0].into(), fresh(lab(IDS[0])));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], lab(IDS[0])));
    assert!(progress::current_solved(&p.attempts[IDS[1]], lab(IDS[1])));
    assert_eq!(
        serde_json::to_value(&p.attempts[&pam.id]).unwrap(),
        old_value
    );
    assert!(p.favorites.contains(IDS[0]));
}
