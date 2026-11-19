//! New authored content; registration/native acceptance tracked separately.
use dlms_learning_lab::{
    debian_packages as core, debian_packages_teaching as teach, debian_packages_terminal as term,
    debian_packages_workspace as work,
    engine::{self, Goal, State},
    pack,
};
use std::sync::OnceLock;
fn labs() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| {
        pack::load_catalog(&[
            include_str!("../scenarios/debian-packages-v1.json"),
            include_str!("../scenarios/debian-conflict-lessons-v1.json"),
            include_str!("../scenarios/debian-conffile-lessons-v1.json"),
        ])
        .unwrap()
        .into_iter()
        .filter(|s| s.unit == "debian-conffile-upgrades")
        .collect()
    })
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
#[test]
fn taught_native_upgrades_and_separate_guided_preservation_experiments_reach_their_outcomes() {
    assert_eq!(labs().len(), 3);
    for s in labs() {
        let d = definition(s);
        d.validate().unwrap();
        let l = s.debian_packages_teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert!(l.separate_from(&d.initial));
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for command in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, command).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{} {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        let mut example = teach::Learning::new(l).unwrap();
        for a in &l.example_actions {
            example.execute(l, &a.action).unwrap();
        }
        assert!(example.checks(l).iter().all(|r| r.0));
        for a in &l.try_change.actions {
            example.execute(l, a).unwrap();
        }
        assert_eq!(
            example.checks(l).iter().map(|r| r.0).collect::<Vec<_>>(),
            l.try_change.expected
        );
        for a in &l.repair {
            example.execute(l, a).unwrap();
        }
        assert!(example.checks(l).iter().all(|r| r.0));
        assert!(!engine::solved(&s.initial, &s.goal));
    }
}
#[test]
fn wrong_native_choices_and_metadata_or_version_only_shortcuts_fail_specific_results() {
    for s in labs() {
        let d = definition(s);
        let mut state = s.initial.clone();
        engine::execute_for_goal(&mut state, &s.goal, &s.solution[0]).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        let work::Requirement::ConfigurationUpgrade { outcome } = &d.requirements[0] else {
            panic!()
        };
        let pkg = &outcome.package;
        let before = simulation(&mut state).clone();
        let err = term::execute(
            d,
            simulation(&mut state),
            &format!("apt-get -y install {pkg}"),
        )
        .unwrap_err();
        assert_eq!(err.status, 100);
        assert_eq!(simulation(&mut state), &before);
        let mut u = simulation(&mut state).review_conffiles(pkg).unwrap();
        for row in &mut u.files {
            if let Some(required) = outcome.choices.iter().find(|p| p.path == row.path) {
                row.decision = Some(match required.decision {
                    core::ConffileDecision::KeepLocal => core::ConffileDecision::InstallNew,
                    core::ConffileDecision::InstallNew => core::ConffileDecision::KeepLocal,
                });
            }
        }
        work::apply(
            d,
            simulation(&mut state),
            &core::Action::UpgradeConffiles { upgrade: u },
        )
        .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        let checks = engine::checks(&state, &s.goal);
        assert!(!checks[0].0 && checks[0].1.contains(&outcome.choices[0].path));
        assert!(checks.iter().skip(1).any(|r| r.0));
    }
}
#[test]
fn registered_unit_preserves_all272_prior_labs_and81_units_in_prerequisite_order() {
    use dlms_learning_lab::course;
    let before = pack::accepted272().unwrap();
    let now = pack::accepted275().unwrap();
    assert_eq!((before.len(), now.len()), (272, 275));
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&now[..272]).unwrap()
    );
    let old_units = course::accepted272(&before).unwrap();
    let units = course::accepted275(&now).unwrap();
    assert_eq!((old_units.len(), units.len()), (81, 82));
    let filtered: Vec<_> = units
        .iter()
        .filter(|u| u.id != "debian-conffile-upgrades")
        .collect();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(filtered).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "debian-conffile-upgrades")
        .unwrap();
    assert_eq!(units[at - 1].id, "debian-package-conflicts");
    assert_eq!(
        units[at].labs,
        vec![
            "lpic1-conffile-retain",
            "lpic1-conffile-adopt",
            "lpic1-conffile-handover"
        ]
    );
}
#[test]
fn explicit_dependency_first_is_an_equivalent_supported_upgrade_order() {
    for s in labs() {
        let d = definition(s);
        let work::Requirement::ConfigurationUpgrade { outcome } = &d.requirements[0] else {
            panic!()
        };
        let mut expected = s.initial.clone();
        for c in &s.solution {
            engine::execute_for_goal(&mut expected, &s.goal, c).unwrap();
        }
        let mut alternate = s.initial.clone();
        engine::execute_for_goal(&mut alternate, &s.goal, &s.solution[0]).unwrap();
        let lib = outcome.package.trim_end_matches("-report").to_owned() + "-lib";
        term::execute(
            d,
            simulation(&mut alternate),
            &format!("apt-get -y install {lib}"),
        )
        .unwrap();
        let current = simulation(&mut alternate);
        let mut draft = work::Draft::new(current).unwrap();
        work::prepare_configuration(d, current, &mut draft, &outcome.package).unwrap();
        for row in &outcome.choices {
            work::choose_configuration(d, current, &mut draft, &row.path, Some(row.decision))
                .unwrap();
        }
        work::apply(d, current, &draft.action().unwrap()).unwrap();
        assert!(engine::solved(&alternate, &s.goal));
        assert_eq!(alternate, expected);
    }
}
#[test]
fn pending_review_resume_practice_isolation_retries_and_selected_reset_preserve_other_work() {
    use dlms_learning_lab::{
        progress::{self, Attempt, Progress},
        workspace,
    };
    let catalog = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 7,
        ..Default::default()
    };
    for (n, s) in catalog.iter().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        workspace::prepare(&mut a);
        teach::prepare(&mut a, s.debian_packages_teaching.as_ref().unwrap(), true).unwrap();
        p.attempts.insert(s.id.clone(), a);
    }
    let s = &catalog[0];
    let d = definition(s);
    let work::Requirement::ConfigurationUpgrade { outcome } = &d.requirements[0] else {
        panic!()
    };
    let a = p.attempts.get_mut(&s.id).unwrap();
    let assignment = a.state.clone();
    let lesson = s.debian_packages_teaching.as_ref().unwrap();
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_learning
        .as_mut()
        .unwrap();
    for action in &lesson.example_actions {
        learning.execute(lesson, &action.action).unwrap();
    }
    assert!(learning.checks(lesson).iter().all(|r| r.0));
    learning.terminal = Some(Default::default());
    learning.terminal.as_mut().unwrap().command = "dpkg-query -W gallery-report".into();
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).unwrap();
    let state = simulation(&mut a.state);
    let draft = a
        .workspace
        .as_mut()
        .unwrap()
        .debian_packages_draft
        .as_mut()
        .unwrap();
    work::prepare_configuration(d, state, draft, &outcome.package).unwrap();
    for row in &outcome.choices {
        work::choose_configuration(d, state, draft, &row.path, Some(row.decision)).unwrap();
    }
    assert!(!progress::current_solved(a, s));
    p.favorites.insert(catalog[2].id.clone());
    let untouched = serde_json::to_value(&p.attempts[&catalog[2].id]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog).unwrap();
    let mut q = progress::load(&path, catalog).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    let a = q.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, s));
    let action = a
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_draft
        .as_ref()
        .unwrap()
        .action()
        .unwrap();
    work::apply(d, simulation(&mut a.state), &action).unwrap();
    assert!(progress::current_solved(a, s));
    let viewer = outcome.package.trim_end_matches("-report").to_owned() + "-viewer";
    term::execute(d, simulation(&mut a.state), &format!("dpkg -r {viewer}")).unwrap();
    assert!(!progress::current_solved(a, s));
    term::execute(
        d,
        simulation(&mut a.state),
        &format!("apt-get -y install {viewer}"),
    )
    .unwrap();
    assert!(progress::current_solved(a, s));
    let example = a
        .workspace
        .as_ref()
        .unwrap()
        .debian_packages_learning
        .clone();
    progress::save_with_catalog(&path, &q, catalog).unwrap();
    let mut q = progress::load(&path, catalog).unwrap();
    let a = q.attempts.get_mut(&s.id).unwrap();
    assert!(progress::current_solved(a, s));
    assert_eq!(
        a.workspace.as_ref().unwrap().debian_packages_learning,
        example
    );
    let visited = a.visited;
    *a = Attempt::new(s, visited);
    workspace::prepare(a);
    teach::prepare(a, lesson, true).unwrap();
    assert!(!progress::current_solved(a, s));
    assert_eq!(a.state, s.initial);
    assert_eq!(
        serde_json::to_value(&q.attempts[&catalog[2].id]).unwrap(),
        untouched
    );
    assert!(q.favorites.contains(&catalog[2].id));
}
