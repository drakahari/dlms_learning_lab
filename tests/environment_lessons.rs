//! Authored environment tasks exercise actual parent/child state across varied inputs.
use dlms_learning_lab::{
    engine::{self, State},
    environment_workspace as e, pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/linux-files-v1.json"),
        include_str!("../scenarios/shell-unit-v1.json"),
        include_str!("../scenarios/environment-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "linux-shell-environment")
    .collect()
}
fn session(s: &pack::Scenario) -> e::Session {
    let State::Environment { simulation } = &s.initial else {
        panic!()
    };
    (**simulation).clone()
}
fn run(d: &mut e::Draft, t: &e::Task, source: &str) -> Vec<bool> {
    d.edit(source.into()).unwrap();
    assert!(d.store(e::run(t, source).unwrap(), t));
    e::checks(t, d).into_iter().map(|r| r.0).collect()
}
#[test]
fn environment_taught_solutions_alternatives_and_separate_worked_cases_pass() {
    let labs = labs();
    assert_eq!(labs.len(), 2);
    for s in labs {
        let mut x = session(&s);
        let initial = x.assignment.clone();
        assert_eq!(x.definition.assignment.cases.len(), 6);
        assert_eq!(x.definition.example.cases.len(), 2);
        let example = x.example.source.clone();
        assert!(
            run(&mut x.example, &x.definition.example, &example)
                .iter()
                .all(|p| *p)
        );
        assert_eq!(x.assignment, initial);
        assert!(!engine::solved(
            &State::Environment {
                simulation: Box::new(x.clone())
            },
            &s.goal
        ));
        let tail = if s.id.ends_with("parent-note") {
            "export -n NOTE\n"
        } else {
            ""
        };
        for source in [
            s.solution[0].clone(),
            format!("{tail}export MODE=\"$2\" SITE=\"$1\"\n"),
            format!("scratch=$1\nMODE=$2\nSITE=$scratch\nexport MODE\nexport SITE\n{tail}"),
        ] {
            assert!(
                run(&mut x.assignment, &x.definition.assignment, &source)
                    .iter()
                    .all(|p| *p),
                "{} / {source}",
                s.id
            );
            assert!(engine::solved(
                &State::Environment {
                    simulation: Box::new(x.clone())
                },
                &s.goal
            ));
        }
    }
}
#[test]
fn environment_common_mistakes_hardcoding_counterfeit_reports_and_files_fail() {
    for s in labs() {
        let mut x = session(&s);
        let t = x.definition.assignment.clone();
        let starter = x.assignment.source.clone();
        assert!(run(&mut x.assignment, &t, &starter).iter().any(|p| !*p));
        let tail = if s.id.ends_with("parent-note") {
            "export -n NOTE\n"
        } else {
            ""
        };
        for wrong in [
            "SITE=$1\nMODE=$2\n".into(),
            format!("{tail}SITE='North Hall'\nMODE=brief\nexport SITE MODE\n"),
            format!("{}export SCRATCH=yes\n", s.solution[0]),
            format!("{}printf changed > keep.txt\n", s.solution[0]),
            "printf 'Team: %s\\nSite: %s\\nMode: %s\\n' \"$TEAM\" \"$1\" \"$2\"\n".into(),
        ] {
            assert!(
                run(&mut x.assignment, &t, &wrong).iter().any(|p| !*p),
                "{} false pass: {wrong}",
                s.id
            );
        }
        let first = &t.cases[0].input.args[0];
        let hardcoded = format!(
            "{tail}SITE=\"$1\"\nMODE=\"$2\"\nexport MODE\nif test \"$SITE\" = {first:?}; then export SITE; fi\n"
        );
        let flags = run(&mut x.assignment, &t, &hardcoded);
        assert!(
            flags.iter().any(|p| *p) && flags.iter().any(|p| !*p),
            "partial completion should remain visible"
        );
        if s.id.ends_with("parent-note") {
            for wrong in [
                s.solution[0].replace("export -n NOTE", "NOTE=''"),
                s.solution[0].replace("export -n NOTE", "export -n TEAM"),
            ] {
                assert!(run(&mut x.assignment, &t, &wrong).iter().any(|p| !*p));
            }
        }
        for invalid in ["if true; then\n", "export -p\n"] {
            assert!(run(&mut x.assignment, &t, invalid).iter().all(|p| !*p));
            assert!(
                x.assignment
                    .result
                    .as_ref()
                    .unwrap()
                    .results
                    .iter()
                    .all(|r| r.parent.error.is_some())
            );
        }
    }
}
#[test]
fn environment_progress_drafts_reverification_reset_and_invalid_restore_keep_other_work() {
    let labs = labs();
    let s = &labs[0];
    let other = &labs[1];
    let mut p = Progress {
        schema_version: 1,
        clock: 4,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    for source in &labs {
        let mut a = Attempt::new(source, 1);
        workspace::prepare(&mut a);
        p.attempts.insert(source.id.clone(), a);
    }
    let untouched = serde_json::to_value(&p.attempts[&other.id]).unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    let example = "# separate example draft\nLOCATION=$1\n";
    x.example.edit(example.into()).unwrap();
    assert!(
        run(&mut x.assignment, &x.definition.assignment, &s.solution[0])
            .iter()
            .all(|p| *p)
    );
    assert!(progress::current_solved(a, s));
    let raw = serde_json::to_string(&p).unwrap();
    let mut restored = progress::decode(raw.as_bytes(), &labs).unwrap();
    assert!(
        !progress::current_solved(&restored.attempts[&s.id], s),
        "serialized observations alone are not current credit"
    );
    let a = restored.attempts.get_mut(&s.id).unwrap();
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    assert_eq!(x.example.source, example);
    assert!(x.assignment.needs_reverification(&x.definition.assignment));
    let text = x.assignment.source.clone();
    assert!(
        run(&mut x.assignment, &x.definition.assignment, &text)
            .iter()
            .all(|p| *p)
    );
    assert!(progress::current_solved(a, s));
    let State::Environment { simulation: x } = &mut a.state else {
        panic!()
    };
    x.assignment.edit("SITE=wrong\n".into()).unwrap();
    assert!(!progress::current_solved(a, s));
    let invalid = raw.replace("export SITE MODE", "export BAD MODE");
    assert_ne!(invalid, raw);
    assert!(
        progress::decode(invalid.as_bytes(), &labs).is_err(),
        "authored definition mutation must fail"
    );
    restored.attempts.insert(s.id.clone(), Attempt::new(s, 2));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&other.id]).unwrap(),
        untouched
    );
    assert_eq!(restored.favorites, p.favorites);
}
#[test]
fn environment_invalid_pack_rejects_inconsistent_child_and_source_identity() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/environment-lessons-v1.json")).unwrap();
    for path in [
        "/scenarios/0/goal/definition/assignment/instructions",
        "/scenarios/0/goal/definition/id",
    ] {
        let mut v = original.clone();
        *v.pointer_mut(path).unwrap() = serde_json::json!("changed");
        let raw = serde_json::to_string(&v).unwrap();
        assert!(
            pack::load_catalog(&[
                include_str!("../scenarios/starter-v1.json"),
                include_str!("../scenarios/linux-files-v1.json"),
                include_str!("../scenarios/shell-unit-v1.json"),
                &raw
            ])
            .is_err()
        );
    }
}
#[test]
fn environment_preserves257_labs74_units_and_registers_two_distinct_tasks() {
    use dlms_learning_lab::course;
    let prior=pack::accepted257().unwrap();let now=pack::accepted259().unwrap();
    assert_eq!((prior.len(),now.len()),(257,259));
    for (a,b) in prior.iter().zip(&now) {assert_eq!(serde_json::to_value(a).unwrap(),serde_json::to_value(b).unwrap(),"changed {}",a.id);}
    let old=course::accepted257(&prior).unwrap();let units=course::accepted259(&now).unwrap();
    assert_eq!((old.len(),units.len()),(74,75));
    let retained:Vec<_>=units.iter().filter(|u|u.id!="linux-shell-environment").collect();
    assert_eq!(serde_json::to_value(old).unwrap(),serde_json::to_value(retained).unwrap());
    let at=units.iter().position(|u|u.id=="linux-file-foundations").unwrap();
    assert_eq!(units[at+1].id,"linux-shell-environment");
    assert_eq!(units[at+1].labs,vec!["lpic1-env-child-settings","lpic1-env-parent-note"]);
    assert_eq!(now.iter().filter(|s|s.track=="LPIC-1").count(),76);
    assert_eq!(now.iter().filter(|s|s.track=="Bash/Shell").count(),68);
}
