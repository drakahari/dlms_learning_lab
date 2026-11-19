//! Authored hands-on integration and exact preceding-catalog preservation.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    raid_assembly as c, raid_assembly_source as source, raid_assembly_workspace as w, workspace,
};
use std::sync::OnceLock;
const ID: &str = "lpic2-raid1-assembly-policy";
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted338().unwrap())
}
fn lab() -> &'static pack::Scenario {
    catalog().iter().find(|s| s.id == ID).unwrap()
}
fn fresh() -> Attempt {
    let mut a = Attempt::new(lab(), 1);
    workspace::prepare(&mut a);
    source::context(&a, lab()).unwrap();
    a
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::RaidAssembly { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve() -> Attempt {
    let mut a = fresh();
    let d = source::source(lab()).unwrap();
    for step in &d.reference {
        assert_eq!(
            session(&mut a)
                .apply(d, &step.operation, false)
                .unwrap()
                .status,
            step.status
        );
    }
    session(&mut a).run(d, w::CHECK, false).unwrap();
    assert!(progress::current_solved(&a, lab()));
    a
}
#[test]
fn catalog_and_course_preserve_all337_previous_labs_and116_units_exactly() {
    let now = catalog();
    assert_eq!(now.len(), 338);
    let old = pack::accepted337().unwrap();
    assert_eq!(old.len(), 337);
    assert_eq!(
        now.iter()
            .filter(|s| s.id != ID)
            .map(|s| serde_json::to_value(s).unwrap())
            .collect::<Vec<_>>(),
        old.iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect::<Vec<_>>()
    );
    let units = course::accepted338(now).unwrap();
    assert_eq!(units.len(), 117);
    let prior = course::accepted337(&old).unwrap();
    assert_eq!(prior.len(), 116);
    assert_eq!(
        units
            .iter()
            .filter(|u| u.id != "raid1-assembly-policy")
            .map(|u| serde_json::to_value(u).unwrap())
            .collect::<Vec<_>>(),
        prior
            .iter()
            .map(|u| serde_json::to_value(u).unwrap())
            .collect::<Vec<_>>()
    );
    let n = units
        .iter()
        .position(|u| u.id == "raid1-assembly-policy")
        .unwrap();
    assert_eq!(units[n - 1].id, "linux-raid1-recovery");
    assert_eq!(lab().prerequisites, vec!["lpic2-raid1-handover"]);
    for (track, count) in [
        ("CCNA", 74),
        ("LPIC-1", 114),
        ("LPIC-2", 73),
        ("Bash/Shell", 77),
    ] {
        assert_eq!(now.iter().filter(|s| s.track == track).count(), count);
    }
}
#[test]
fn taught_method_and_practice_mistake_retry_are_reachable_without_example_credit() {
    let s = lab();
    let d = source::source(s).unwrap();
    d.validate_reachability().unwrap();
    assert!(!progress::current_solved(&fresh(), s));
    solve();
    let mut a = fresh();
    let initial = session(&mut a).assignment.clone();
    for step in &d.example_steps {
        assert_eq!(
            session(&mut a)
                .apply(d, &step.operation, true)
                .unwrap()
                .status,
            step.status
        );
    }
    session(&mut a).run(d, w::CHECK, true).unwrap();
    assert!(
        session(&mut a)
            .practice
            .checks(&d.example)
            .iter()
            .all(|r| r.0)
    );
    assert_eq!(session(&mut a).assignment, initial);
    assert!(!progress::current_solved(&a, s));
    assert_eq!(d.reference.iter().filter(|s| s.status == 1).count(), 1);
}
#[test]
fn legitimate_alternate_policy_and_explicit_reversed_members_pass() {
    let d = source::source(lab()).unwrap();
    let mut a = fresh();
    let correct = d
        .reference
        .iter()
        .find_map(|s| {
            if let w::Operation::Save { text } = &s.operation {
                Some(text.0.clone())
            } else {
                None
            }
        })
        .unwrap();
    let rows = correct
        .lines()
        .filter(|line| !line.starts_with("DEVICE"))
        .rev()
        .map(|s| s.replace("ARRAY", "arr").replace(' ', "\t"))
        .collect::<Vec<_>>()
        .join("\n");
    session(&mut a)
        .edit(
            d,
            format!("# supported default candidates\n{rows}\n"),
            false,
        )
        .unwrap();
    session(&mut a).save(d, false).unwrap();
    let u = d.assignment.requirements[0].uuid.display();
    session(&mut a)
        .run(
            d,
            &format!("mdadm -A /dev/md10 --uuid={u} /dev/vdb1 /dev/vda1"),
            false,
        )
        .unwrap();
    session(&mut a).run(d, w::CHECK, false).unwrap();
    assert!(progress::current_solved(&a, lab()));
}
#[test]
fn partial_wrong_saved_policy_and_stopped_peer_do_not_keep_success() {
    let d = source::source(lab()).unwrap();
    let mut a = fresh();
    let op = &d
        .reference
        .iter()
        .find(|s| matches!(s.operation, w::Operation::Save { .. }))
        .unwrap()
        .operation;
    session(&mut a).apply(d, op, false).unwrap();
    session(&mut a).run(d, w::CHECK, false).unwrap();
    let checks = d.checks(session(&mut a));
    assert!(checks.iter().any(|r| r.0));
    assert!(checks.iter().any(|r| !r.0));
    assert!(!progress::current_solved(&a, lab()));
    let mut a = solve();
    session(&mut a)
        .run(d, "mdadm --stop /dev/md11", false)
        .unwrap();
    assert!(!progress::current_solved(&a, lab()));
    session(&mut a).run(d, w::CHECK, false).unwrap();
    assert!(
        d.checks(session(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("/dev/md11"))
    );
}
#[test]
fn syntax_error_feedback_preserves_data_and_repair_remains_possible() {
    let d = source::source(lab()).unwrap();
    let mut a = fresh();
    session(&mut a)
        .edit(d, "ARRAY /dev/md10 name=unsupported\n".into(), false)
        .unwrap();
    session(&mut a).save(d, false).unwrap();
    let old = session(&mut a).assignment.runtime.live.clone();
    let result = session(&mut a)
        .run(d, "mdadm --assemble --scan /dev/md10", false)
        .unwrap();
    assert!(result.contains("Configuration line 1"));
    assert!(result.contains("UUID"));
    assert_eq!(session(&mut a).assignment.runtime.live, old);
    for step in &d.reference {
        session(&mut a).apply(d, &step.operation, false).unwrap();
    }
    session(&mut a).run(d, w::CHECK, false).unwrap();
    assert!(progress::current_solved(&a, lab()));
}
#[test]
fn progress_preserves_old_work_two_new_drafts_favorites_and_reset_scope() {
    let old = catalog()
        .iter()
        .find(|s| s.id == "lpic2-autofs-indirect-repair")
        .unwrap();
    let mut old_attempt = Attempt::new(old, 1);
    workspace::prepare(&mut old_attempt);
    let old_value = serde_json::to_value(&old_attempt).unwrap();
    let d = source::source(lab()).unwrap();
    let mut a = solve();
    session(&mut a)
        .edit(d, "# unfinished assignment\n".into(), false)
        .unwrap();
    session(&mut a)
        .edit(d, "# separate practice\n".into(), true)
        .unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.favorites.insert(ID.into());
    p.attempts.insert(old.id.clone(), old_attempt);
    p.attempts.insert(ID.into(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&q.attempts[&old.id]).unwrap(),
        old_value
    );
    assert!(!progress::current_solved(&q.attempts[ID], lab()));
    let w = session(q.attempts.get_mut(ID).unwrap());
    assert_eq!(w.assignment.draft.0, "# unfinished assignment\n");
    assert_eq!(w.practice.draft.0, "# separate practice\n");
    q.attempts.insert(ID.into(), fresh());
    assert_eq!(
        serde_json::to_value(&q.attempts[&old.id]).unwrap(),
        old_value
    );
    let original = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
#[test]
fn invalid_packs_mismatched_sources_and_invented_success_are_rejected() {
    let s = lab();
    let good = solve();
    let mut bad = good.clone();
    session(&mut bad).assignment.history.clear();
    assert!(source::context(&bad, s).is_err());
    assert!(!progress::current_solved(&bad, s));
    for variant in 0..4 {
        let mut s = s.clone();
        match variant {
            0 => s.id = "other-id".into(),
            1 => s.solution = vec!["pretend success".into()],
            2 => {
                let Goal::RaidAssembly { definition } = &mut s.goal else {
                    panic!()
                };
                definition.reference.clear();
            }
            _ => {
                let State::RaidAssembly { simulation } = &mut s.initial else {
                    panic!()
                };
                simulation.assignment.checked = true;
            }
        }
        let pack = pack::Pack {
            schema_version: 1,
            scenarios: vec![s],
        };
        assert!(pack::load(&serde_json::to_string(&pack).unwrap()).is_err());
    }
}
#[test]
fn naked_cross_family_and_unsupported_execution_cannot_escape_or_change_state() {
    let s = lab();
    let mut a = fresh();
    let original = a.state.clone();
    assert!(engine::execute(&mut a.state, "mdadm --assemble --scan /dev/md10").is_err());
    assert_eq!(a.state, original);
    let other = catalog()
        .iter()
        .find(|s| s.id == "lpic2-autofs-indirect-repair")
        .unwrap();
    assert!(engine::execute_for_goal(&mut a.state, &other.goal, "help").is_err());
    assert_eq!(a.state, original);
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "sh -c echo").is_err());
    assert_eq!(a.state, original);
    assert!(engine::execute_for_goal(&mut a.state, &s.goal, "cat /proc/mdstat").is_ok());
    let _ = c::CONFIG;
}
