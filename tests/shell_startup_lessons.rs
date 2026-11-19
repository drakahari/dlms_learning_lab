//! Authored startup-file lessons: actual sessions, transfer and retained progress.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    startup_workspace as w, workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic1-shell-startup-priority", "lpic1-shell-startup-quiet"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted269().unwrap())
}
fn labs() -> impl Iterator<Item = &'static pack::Scenario> {
    catalog().iter().filter(|s| IDS.contains(&s.id.as_str()))
}
fn def(s: &pack::Scenario) -> &w::Definition {
    let Goal::StartupFiles { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn solution(s: &pack::Scenario) -> dlms_learning_lab::shell::startup::Files {
    serde_json::from_str(s.solution[0].strip_prefix(w::PREFIX).unwrap()).unwrap()
}
#[test]
fn two_authored_tasks_retain_267_scenarios_and79_units_in_order() {
    let old = pack::accepted267().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (267, 269));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..267]).unwrap()
    );
    let prior = course::accepted267(&old).unwrap();
    let current = course::accepted269(now).unwrap();
    assert_eq!((prior.len(), current.len()), (79, 80));
    let filtered: Vec<_> = current
        .iter()
        .filter(|u| u.id != "linux-shell-startup-files")
        .collect();
    assert_eq!(
        serde_json::to_value(prior).unwrap(),
        serde_json::to_value(filtered).unwrap()
    );
    let at = current
        .iter()
        .position(|u| u.id == "linux-shell-startup-files")
        .unwrap();
    assert_eq!(current[at - 1].id, "linux-shell-environment");
    assert_eq!(current[at].labs, IDS);
}
#[test]
fn taught_solutions_and_equivalent_supported_exports_pass_all_varied_sessions() {
    for s in labs() {
        let d = def(s);
        d.validate().unwrap();
        assert_eq!(
            d.initial.cases.len(),
            if s.id.ends_with("quiet") { 5 } else { 6 }
        );
        let session = w::Session::new(d);
        assert_eq!(
            session.assignment.selected,
            if s.id.ends_with("quiet") {
                "~/.bashrc"
            } else {
                "~/.bash_profile"
            }
        );
        assert!(
            session.assignment.files.0[&session.assignment.selected]
                .text
                .contains("legacy"),
            "Start on an intentionally faulty selected file"
        );
        let rows = d.initial.run(&solution(s)).unwrap();
        assert!(rows.iter().all(w::CaseResult::passed), "{} {rows:?}", s.id);
        let mut alt = solution(s);
        for file in alt.0.values_mut() {
            file.text = file.text.replace(
                "APP_NAME=\"$PROJECT-app\"\nexport APP_NAME",
                "export APP_NAME=$PROJECT-app",
            );
            file.text.push_str("# equivalent settings\n");
        }
        assert!(
            d.initial
                .run(&alt)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        assert!(progress::current_solved(&a, s));
    }
}
#[test]
fn plausible_wrong_file_empty_winner_unexported_hardcoded_print_and_early_exit_fail() {
    for s in labs() {
        let d = def(s);
        let good = solution(s);
        for replacement in [
            "",
            "APP_NAME=\"$PROJECT-app\"",
            "export APP_NAME=harbor-app",
            "printf '%s\\n' harbor-app; exit 0",
            "for x in a; do :; done",
        ] {
            let mut wrong = good.clone();
            let file = if s.id.ends_with("priority") {
                "~/.bash_profile"
            } else {
                "~/.bashrc"
            };
            wrong.0.get_mut(file).unwrap().text = replacement.into();
            let rows = d.initial.run(&wrong).unwrap();
            assert!(rows.iter().any(|r| !r.passed()), "{} {replacement}", s.id);
            assert!(
                rows.iter()
                    .filter(|r| !r.passed())
                    .all(|r| r.checks.iter().any(|(ok, why)| !ok && !why.is_empty()))
            );
        }
        for (path, file) in &good.0 {
            let mut hardcoded = good.clone();
            let first = d
                .initial
                .cases
                .iter()
                .find(|c| {
                    c.present.contains(path) && match &c.invocation {
                        dlms_learning_lab::shell::startup::Invocation::InteractiveLogin {
                            ..
                        } => c
                            .present
                            .iter()
                            .find(|p| {
                                ["~/.bash_profile", "~/.bash_login", "~/.profile"]
                                    .contains(&p.as_str())
                            })
                            .is_some_and(|p| p == path),
                        dlms_learning_lab::shell::startup::Invocation::InteractiveNonLogin {
                            ..
                        } => path == "~/.bashrc",
                        dlms_learning_lab::shell::startup::Invocation::NonInteractive {
                            bash_env,
                        } => bash_env.as_deref() == Some(path.as_str()),
                    }
                })
                .unwrap();
            hardcoded.0.get_mut(path).unwrap().text =
                file.text.replace("$PROJECT", &first.input.env["PROJECT"]);
            assert!(
                d.initial
                    .run(&hardcoded)
                    .unwrap()
                    .iter()
                    .any(|r| !r.passed()),
                "{} hardcoded in {path}",
                s.id
            );
            if s.id.ends_with("priority") {
                let mut wrong_base = good.clone();
                wrong_base
                    .0
                    .get_mut(path)
                    .unwrap()
                    .text
                    .push_str(&format!("\nBASE={}\n", first.input.env["SITE"]));
                assert!(
                    d.initial
                        .run(&wrong_base)
                        .unwrap()
                        .iter()
                        .any(|r| !r.passed()),
                    "hardcoded BASE in {path}"
                );
            }
        }
        let initial = d.initial.run(&d.initial.starter).unwrap();
        assert!(initial.iter().any(w::CaseResult::passed) && initial.iter().any(|r| !r.passed()));
        if s.id.ends_with("priority") {
            let mut wrong = d.initial.starter.clone();
            wrong.0.get_mut("~/.profile").unwrap().text = good.0["~/.profile"].text.clone();
            assert!(d.initial.run(&wrong).unwrap().iter().any(|r| !r.passed()));
        }
    }
}
#[test]
fn worked_practice_has_distinct_variables_and_does_not_solve_assignment() {
    for s in labs() {
        let d = def(s);
        assert!(
            d.example
                .run(&d.example_worked)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        assert!(
            d.initial
                .run(&d.example_worked)
                .unwrap()
                .iter()
                .any(|r| !r.passed())
        );
        assert!(
            d.example_worked
                .0
                .values()
                .any(|f| f.text.contains("WORKSHOP"))
        );
        assert!(
            !d.example_worked
                .0
                .values()
                .any(|f| f.text.contains("APP_NAME"))
        );
        let mut sim = w::Session::new(d);
        sim.practice.replace(&d.example, &d.example_worked).unwrap();
        sim.practice.run(&d.example).unwrap();
        assert!(!d.solved(&sim));
        assert_eq!(sim.assignment.files, d.initial.starter);
    }
}
#[test]
fn source_bound_resume_reset_invalidates_stale_success_without_inventing_credit() {
    for s in labs() {
        let d = def(s);
        let mut a = Attempt::new(s, 4);
        workspace::prepare(&mut a);
        engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
        let State::StartupFiles { simulation } = &mut a.state else {
            panic!()
        };
        let file = simulation.practice.selected.clone();
        simulation
            .practice
            .edit(&d.example, &file, "VALUE='unfinished")
            .unwrap();
        let mut p = Progress {
            schema_version: 1,
            clock: 4,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, catalog()).unwrap();
        let mut loaded = progress::load(&path, catalog()).unwrap();
        let a = loaded.attempts.get_mut(&s.id).unwrap();
        assert!(!progress::current_solved(a, s));
        let State::StartupFiles { simulation } = &mut a.state else {
            panic!()
        };
        assert_eq!(simulation.practice.files.0[&file].text, "VALUE='unfinished");
        simulation.assignment.run(&d.initial).unwrap();
        assert!(d.solved(simulation));
        let file = simulation.assignment.selected.clone();
        simulation
            .assignment
            .edit(&d.initial, &file, "# changed")
            .unwrap();
        assert!(!d.solved(simulation));
        **simulation = w::Session::new(d);
        assert!(!d.solved(simulation));
        assert_eq!(simulation.assignment.files, d.initial.starter);
    }
}
