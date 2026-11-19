use dlms_learning_lab::{
    backup, client_names_workspace as w, course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const RAW: &str = include_str!("../scenarios/client-name-lessons-v1.json");
const IDS: [&str; 2] = ["lpic1-local-host-alias", "lpic1-name-source-fallback"];
fn catalog() -> Vec<pack::Scenario> {
    pack::load_catalog(&[include_str!("../scenarios/linux-host-lessons-v1.json"), RAW])
        .unwrap()
        .into_iter()
        .filter(|s| matches!(s.initial, State::ClientNames { .. }))
        .collect()
}
fn definition(s: &pack::Scenario) -> &w::Definition {
    let Goal::ClientNames { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut w::Session {
    let State::ClientNames { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 5);
    workspace::prepare(&mut a);
    a
}
fn reference(s: &pack::Scenario) -> w::Files {
    serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap()
}
fn apply(s: &pack::Scenario, files: &w::Files) -> State {
    let mut state = s.initial.clone();
    engine::execute_for_goal(
        &mut state,
        &s.goal,
        &format!("{}{}", w::PREFIX, serde_json::to_string(files).unwrap()),
    )
    .unwrap();
    state
}
#[test]
fn preserve248_labs69_units_exactly_then_add_two_prerequisite_ordered_name_tasks() {
    let prior = pack::accepted248().unwrap();
    let now = pack::accepted250().unwrap();
    assert_eq!((prior.len(), now.len()), (248, 250));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..248]).unwrap()
    );
    let before = course::accepted248(&prior).unwrap();
    let units = course::accepted250(&now).unwrap();
    assert_eq!((before.len(), units.len()), (69, 70));
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(&units[..69]).unwrap()
    );
    assert_eq!(units.last().unwrap().labs, IDS);
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["linux-host-networking"]
    );
    assert_eq!(now[249].prerequisites, [IDS[0]]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [72, 69, 43, 66]
    );
}
#[test]
fn authored_taught_and_legitimate_alternatives_pass_derived_queries_and_source_traces() {
    for s in catalog() {
        let d = w::source(&s).unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let answer_files = reference(&s);
        let mut alt = answer_files.clone();
        alt.get_mut("hosts")
            .unwrap()
            .insert_str(0, "# Preserve names with equivalent whitespace\n");
        alt.insert("nsswitch.conf".into(),"hosts: files [NOTFOUND=continue UNAVAIL=continue TRYAGAIN=continue SUCCESS=return] dns\n".into());
        for files in [answer_files, alt] {
            let state = apply(&s, &files);
            assert!(
                engine::solved(&state, &s.goal),
                "{:?}",
                engine::checks(&state, &s.goal)
            );
            let rows = d.initial.run(&files).unwrap();
            assert!(rows.iter().all(w::CaseResult::passed));
            assert!(rows.iter().all(|r| !r.lookups.is_empty()));
        }
        let mut a = fresh(&s);
        a.state = apply(&s, &reference(&s));
        assert!(progress::current_solved(&a, &s));
        session(&mut a).assignment.edit("hosts", "").unwrap();
        assert!(!progress::current_solved(&a, &s));
        for challenge in [false, true] {
            a.challenge = challenge;
            assert!(!progress::current_solved(&a, &s));
        }
    }
}
#[test]
fn wrong_addresses_aliases_order_early_returns_hardcoded_dns_and_syntax_do_not_pass() {
    for s in catalog() {
        let reference = reference(&s);
        let d = definition(&s);
        for nss in [
            "hosts: dns [NOTFOUND=return] files",
            "hosts:",
            "hosts: resolve files",
        ] {
            let mut wrong = reference.clone();
            wrong.insert("nsswitch.conf".into(), nss.into());
            assert!(
                !engine::solved(&apply(&s, &wrong), &s.goal),
                "{} {nss}",
                s.id
            );
        }
        let mut wrong = reference.clone();
        wrong
            .get_mut("hosts")
            .unwrap()
            .push_str("192.0.2.9 duplicate\n192.0.2.10 duplicate\n");
        let rows = d.initial.run(&wrong).unwrap();
        assert!(rows.iter().all(|r| !r.passed() && r.diagnostic.is_some()));
        let mut wrong = reference.clone();
        wrong.insert("hosts".into(), String::new());
        assert!(!engine::solved(&apply(&s, &wrong), &s.goal));
        assert!(!engine::solved(&apply(&s, &d.initial.starter), &s.goal));
    }
    let s = &catalog()[1];
    let mut wrong = reference(s);
    wrong
        .get_mut("hosts")
        .unwrap()
        .push_str("198.51.100.70 catalog.office.lab.test\n");
    let rows = definition(s).initial.run(&wrong).unwrap();
    assert_eq!(
        rows.iter().map(w::CaseResult::passed).collect::<Vec<_>>(),
        [false, false, false, true]
    );
    assert!(
        rows[2]
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("catalog.office.lab.test"))
    );
    assert!(rows[0].checks.iter().any(|r| r.0));
}
#[test]
fn exact_practice_actions_damage_and_repair_without_overwriting_or_completing_assignments() {
    for (i, s) in catalog().into_iter().enumerate() {
        let d = definition(&s);
        let mut sim = w::Session::new(d).unwrap();
        let original = sim.assignment.clone();
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let mut wrong = d.example_worked.clone();
        if i == 0 {
            let text = wrong["hosts"].replace(
                "studio.workshop.lab.test studio",
                "studio.workshop.lab.test studio-old",
            );
            wrong.insert("hosts".into(), text);
        } else {
            wrong.insert(
                "nsswitch.conf".into(),
                "hosts: files [NOTFOUND=return] dns\n".into(),
            );
        }
        sim.practice.replace(&wrong).unwrap();
        let rows = sim.practice.run(&d.example).unwrap();
        assert!(rows.iter().any(|r| !r.passed()));
        assert!(rows.iter().flat_map(|r| &r.checks).any(|r| r.0));
        assert!(!d.solved(&sim));
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert_eq!(sim.assignment, original);
    }
}
#[test]
fn drafts_current_credit_backup_restore_and_selected_reset_survive_safe_writes() {
    let mut all = catalog();
    all.push(
        pack::load(include_str!("../scenarios/starter-v1.json"))
            .unwrap()
            .remove(0),
    );
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = fresh(&all[0]);
    a.state = apply(&all[0], &reference(&all[0]));
    let d = definition(&all[0]);
    session(&mut a).practice.replace(&d.example_worked).unwrap();
    session(&mut a).view = dlms_learning_lab::script_teaching::View::Example;
    p.attempts.insert(all[0].id.clone(), a);
    p.favorites.insert(all[0].id.clone());
    p.attempts.insert(all[2].id.clone(), fresh(&all[2]));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::current_solved(&got.attempts[&all[0].id], &all[0]));
    let export = dir.path().join("backup.json");
    backup::export(&export, &p, &all).unwrap();
    let restore =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    let receipt = backup::restore(&path, &restore, &all).unwrap();
    assert_eq!(
        std::fs::read(receipt.safety_copy.unwrap()).unwrap(),
        original
    );
    let kept = serde_json::to_value(&p.attempts[&all[2].id]).unwrap();
    p.attempts.insert(IDS[0].into(), fresh(&all[0]));
    assert_eq!(serde_json::to_value(&p.attempts[&all[2].id]).unwrap(), kept);
    assert!(p.favorites.contains(IDS[0]));
    assert!(!progress::current_solved(&p.attempts[IDS[0]], &all[0]));
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let decoded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&decoded).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
}
