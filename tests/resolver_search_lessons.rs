use dlms_learning_lab::{
    backup, client_names_workspace as w, course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
const RAW: &str = include_str!("../scenarios/resolver-search-lessons-v1.json");
const IDS: [&str; 2] = ["lpic1-dns-search-suffixes", "lpic1-dns-absolute-precedence"];
fn catalog() -> Vec<pack::Scenario> {
    pack::load_catalog(&[include_str!("../scenarios/linux-host-lessons-v1.json"),
        include_str!("../scenarios/client-name-lessons-v1.json"), RAW])
        .unwrap()
        .into_iter()
        .filter(|s| matches!(&s.goal, Goal::ClientNames {definition} if definition.initial.mode==w::Mode::Search))
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
fn preserve250_labs70_units_exactly_and_integrate_two_search_assignments() {
    let prior = pack::accepted250().unwrap();
    let now = pack::accepted252().unwrap();
    assert_eq!((prior.len(), now.len()), (250, 252));
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..250]).unwrap()
    );
    let old = course::accepted250(&prior).unwrap();
    let units = course::accepted252(&now).unwrap();
    assert_eq!((old.len(), units.len()), (70, 71));
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(&units[..70]).unwrap()
    );
    assert_eq!(units.last().unwrap().labs, IDS);
    assert_eq!(
        units.last().unwrap().prerequisites,
        ["linux-client-name-sources"]
    );
    assert_eq!(now[251].prerequisites, [IDS[0]]);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| now.iter().filter(|s| s.track == t).count()),
        [72, 71, 43, 66]
    );
}
#[test]
fn taught_and_supported_alternatives_use_real_candidate_attempts_not_exact_strings() {
    for s in catalog() {
        let d = w::source(&s).unwrap();
        assert_eq!(d.initial.mode, w::Mode::Search);
        let solution_files = reference(&s);
        let mut alternatives = vec![solution_files.clone()];
        let mut alt = solution_files.clone();
        alt.get_mut("resolv.conf")
            .unwrap()
            .insert_str(0, "# supplied virtual excerpt\n");
        let text = alt["resolv.conf"].replace("ndots:1", "ndots:2");
        alt.insert("resolv.conf".into(), text);
        alternatives.push(alt);
        if s.id == IDS[1] {
            let mut alt = solution_files.clone();
            let text = alt["resolv.conf"].replace("search ", "domain ");
            alt.insert("resolv.conf".into(), text);
            alternatives.push(alt);
        }
        for files in alternatives {
            let state = apply(&s, &files);
            assert!(
                engine::solved(&state, &s.goal),
                "{:?}",
                engine::checks(&state, &s.goal)
            );
            let rows = d.initial.run(&files).unwrap();
            assert!(rows.iter().all(w::CaseResult::passed));
            assert!(rows.iter().all(|r| !r.searches.is_empty()));
        }
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut a = fresh(&s);
        a.state = apply(&s, &solution_files);
        assert!(progress::current_solved(&a, &s));
        session(&mut a)
            .assignment
            .edit("resolv.conf", "search wrong.lab.test")
            .unwrap();
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn wrong_site_incomplete_fallback_default_domain_and_ndots_extremes_have_specific_failures() {
    for s in catalog() {
        let d = definition(&s);
        let solution_files = reference(&s);
        for wrong in [
            d.initial.starter.clone(),
            w::Files::from([("resolv.conf".into(), String::new())]),
            w::Files::from([("resolv.conf".into(), "nameserver 192.0.2.53".into())]),
            w::Files::from([("resolv.conf".into(), "options ndots:-1".into())]),
        ] {
            assert!(!engine::solved(&apply(&s, &wrong), &s.goal));
        }
        let wrong = if s.id == IDS[0] {
            solution_files["resolv.conf"].replace(" south.office.lab.test", "")
        } else {
            solution_files["resolv.conf"].replace("ndots:1", "ndots:0")
        };
        let rows = d
            .initial
            .run(&w::Files::from([("resolv.conf".into(), wrong)]))
            .unwrap();
        assert!(rows.iter().all(|r| !r.passed()));
        assert!(rows.iter().all(|r| r.checks.iter().any(|c| c.0)));
        assert!(
            rows.iter()
                .flat_map(|r| &r.checks)
                .any(|r| !r.0 && r.1.contains("candidate"))
        );
    }
}
#[test]
fn exact_example_experiments_repair_and_keep_assignment_drafts_and_credit_separate() {
    for (i, s) in catalog().into_iter().enumerate() {
        let d = definition(&s);
        let mut sim = w::Session::new(d).unwrap();
        let assignment = sim.assignment.clone();
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let text = if i == 0 {
            d.example_worked["resolv.conf"].replace(" archive.workshop.lab.test", "")
        } else {
            d.example_worked["resolv.conf"].replace("ndots:1", "ndots:0")
        };
        sim.practice.edit("resolv.conf", &text).unwrap();
        assert!(sim.practice.tested.is_none());
        let rows = sim.practice.run(&d.example).unwrap();
        assert!(rows.iter().all(|r| !r.passed()));
        assert!(rows.iter().all(|r| r.checks.iter().any(|c| c.0)));
        sim.practice.replace(&d.example_worked).unwrap();
        assert!(
            sim.practice
                .run(&d.example)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert_eq!(sim.assignment, assignment);
        assert!(!d.solved(&sim));
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
