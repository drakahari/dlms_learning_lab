//! Reachability, useful refusals, alternatives and progress for authored conflicts.
use dlms_learning_lab::{
    debian_packages as core, debian_packages_teaching as teach, debian_packages_terminal as term,
    debian_packages_workspace as work,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/debian-packages-v1.json"),
        include_str!("../scenarios/debian-conflict-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.unit == "debian-package-conflicts")
    .collect()
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::DebianPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn simulation(s: &mut State) -> &mut core::State {
    let State::DebianPackages { simulation } = s else {
        panic!()
    };
    simulation
}
fn run(s: &pack::Scenario, state: &mut State, c: &str) -> Result<term::Output, term::Diagnostic> {
    term::execute(definition(s), simulation(state), c)
}
fn names(s: &pack::Scenario) -> (&str, &str) {
    if s.id.ends_with("retirement") {
        ("ops-legacy", "ops-delivery")
    } else {
        ("branch-relay", "branch-dashboard")
    }
}
#[test]
fn authored_methods_metadata_queries_and_supported_alternatives_reach_outcomes() {
    let labs = labs();
    assert_eq!(labs.len(), 2);
    for s in labs {
        let (old, new) = names(&s);
        let mut state = s.initial.clone();
        assert!(!engine::solved(&state, &s.goal));
        let before = serde_json::to_vec(&state).unwrap();
        let out = run(&s, &mut state, &format!("apt-cache show {new}")).unwrap();
        assert!(out.stdout.contains("Conflicts: "));
        assert!(!out.changed);
        let out = run(&s, &mut state, &format!("dpkg-query -W {old}")).unwrap();
        assert_eq!(out.status, 0);
        assert_eq!(serde_json::to_vec(&state).unwrap(), before);
        let blocked = run(&s, &mut state, &format!("apt-get -y install {new}")).unwrap_err();
        assert_eq!(blocked.kind, term::DiagnosticKind::Model);
        assert_eq!(blocked.status, 100);
        assert!(blocked.message.contains(old));
        assert_eq!(serde_json::to_vec(&state).unwrap(), before);
        for c in &s.solution {
            run(&s, &mut state, c).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        let mut alternate = s.initial.clone();
        let commands = if s.id.ends_with("retirement") {
            vec![
                format!("dpkg -r {old}"),
                format!("apt-get -y install {new} ops-lib"),
            ]
        } else {
            vec![format!("apt-get -y install {new} {old}")]
        };
        for c in commands {
            run(&s, &mut alternate, &c).unwrap();
        }
        assert!(engine::solved(&alternate, &s.goal));
        assert_eq!(simulation(&mut alternate), simulation(&mut state));
        run(&s, &mut state, &format!("dpkg --remove {new}")).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        run(&s, &mut state, &format!("apt-get -y install {new}")).unwrap();
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn plausible_wrong_retirements_partial_repairs_and_lost_settings_fail_usefully() {
    for s in labs() {
        let (old, new) = names(&s);
        let mut partial = s.initial.clone();
        run(&s, &mut partial, &s.solution[0]).unwrap();
        assert!(!engine::solved(&partial, &s.goal));
        let checks = definition(&s).checks(simulation(&mut partial));
        assert!(checks.iter().any(|r| r.0) && checks.iter().any(|r| !r.0));
        let mut wrong = s.initial.clone();
        run(
            &s,
            &mut wrong,
            &format!(
                "dpkg {} {old}",
                if s.id.ends_with("retirement") {
                    "--purge"
                } else {
                    "--remove"
                }
            ),
        )
        .unwrap();
        run(&s, &mut wrong, &format!("apt-get -y install {new}")).unwrap();
        assert!(!engine::solved(&wrong, &s.goal));
        let checks = definition(&s).checks(simulation(&mut wrong));
        assert!(checks.iter().any(|r| !r.0
            && r.1.contains(if s.id.ends_with("retirement") {
                "configuration-only"
            } else {
                "installed record must meet"
            })));
        let mut solved = s.initial.clone();
        for c in &s.solution {
            run(&s, &mut solved, c).unwrap();
        }
        let State::DebianPackages { simulation: good } = &solved else {
            panic!()
        };
        let mut rt = good.runtime().clone();
        let path = if s.id.ends_with("retirement") {
            "/etc/ops/relay.conf"
        } else {
            "/etc/branch/relay.conf"
        };
        rt.files.insert(path.into(), "mode=discarded\n".into());
        let damaged = core::State::new(good.catalog().clone(), rt).unwrap();
        assert!(!definition(&s).checks(&damaged).iter().all(|r| r.0));
        assert!(run(&s, &mut solved, "printf done").is_err());
    }
}
#[test]
fn separate_workshops_teach_refusal_repair_and_expected_partial_results() {
    for s in labs() {
        let l = s.debian_packages_teaching.as_ref().unwrap();
        l.validate().unwrap();
        let assignment = s.initial.clone();
        let mut x = teach::Learning::new(l).unwrap();
        x.initialize_terminal(l).unwrap();
        let new = if s.id.ends_with("retirement") {
            "gallery-delivery"
        } else {
            "studio-dashboard"
        };
        x.insert_command(l, &format!("apt-cache show {new}"))
            .unwrap();
        x.submit_command(l).unwrap();
        let untouched = x.simulation.clone();
        x.insert_command(l, &format!("apt-get -y install {new}"))
            .unwrap();
        x.submit_command(l).unwrap();
        assert_eq!(x.simulation, untouched);
        assert_eq!(
            x.terminal
                .as_ref()
                .unwrap()
                .entries
                .last()
                .unwrap()
                .diagnostic,
            Some(term::DiagnosticKind::Model)
        );
        for a in &l.example_actions {
            x.insert_command(l, a.command.as_ref().unwrap()).unwrap();
            x.submit_command(l).unwrap();
        }
        assert!(x.checks(l).iter().all(|r| r.0));
        for a in &l.try_change.actions {
            x.execute(l, a).unwrap();
        }
        assert_eq!(
            x.checks(l).iter().map(|r| r.0).collect::<Vec<_>>(),
            l.try_change.expected
        );
        for a in &l.repair {
            x.execute(l, a).unwrap();
        }
        assert!(x.checks(l).iter().all(|r| r.0));
        assert_eq!(s.initial, assignment);
        assert!(!engine::solved(&assignment, &s.goal));
    }
}
#[test]
fn terminal_history_independent_command_drafts_and_current_progress_resume() {
    let labs = labs();
    let s = &labs[0];
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    let lesson = s.debian_packages_teaching.as_ref().unwrap();
    teach::prepare(&mut a, lesson, false).unwrap();
    for c in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, c).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_learning
        .as_mut()
        .unwrap();
    learning.initialize_terminal(lesson).unwrap();
    learning
        .insert_command(lesson, "apt-cache show unfinished-gallery")
        .unwrap();
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let loaded = progress::load(&path, &labs).unwrap();
    assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    let resumed = &loaded.attempts[&s.id];
    assert_eq!(
        resumed
            .workspace
            .as_ref()
            .unwrap()
            .debian_packages_learning
            .as_ref()
            .unwrap()
            .terminal
            .as_ref()
            .unwrap()
            .command,
        "apt-cache show unfinished-gallery"
    );
    assert!(loaded.favorites.contains(&s.id));
    assert!(!progress::current_solved(&Attempt::new(s, 2), s));
}
#[test]
fn preceding261_catalog_and76units_are_exact_with_conflicts_after_foundations() {
    use dlms_learning_lab::course;
    let prior = pack::accepted261().unwrap();
    let now = pack::accepted263().unwrap();
    assert_eq!((prior.len(), now.len()), (261, 263));
    for old in &prior {
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(now.iter().find(|s| s.id == old.id).unwrap()).unwrap(),
            "{} changed",
            old.id
        );
    }
    let old = course::accepted261(&prior).unwrap();
    let units = course::accepted263(&now).unwrap();
    assert_eq!((old.len(), units.len()), (76, 77));
    let retained = units
        .iter()
        .filter(|u| u.id != "debian-package-conflicts")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "debian-package-conflicts")
        .unwrap();
    assert_eq!(units[at - 1].id, "debian-package-administration");
}
