//! Normally registered authored RPM outcomes. No host utility, guessed observation, or history grading.
use dlms_learning_lab::{
    backup, engine, pack,
    progress::{self, Attempt, Progress},
    rpm_packages as core, rpm_packages_teaching as teaching,
    rpm_packages_terminal::{self as terminal, Outcome},
    rpm_packages_workspace as work,
    script_teaching::View,
    workspace,
};
use serde::Deserialize;
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicUsize},
};

const IDS: [&str; 4] = [
    "lpic1-rpm-provider",
    "lpic1-rpm-epoch",
    "lpic1-rpm-payload",
    "lpic1-rpm-handover",
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    name: String,
    commands: Vec<String>,
    rows: Vec<bool>,
    outcomes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Review {
    id: String,
    initial: Vec<bool>,
    reference: Plan,
    alternatives: Vec<Plan>,
    wrong: Plan,
    partial: Plan,
    read_only: Plan,
    after_pass_break: Plan,
    repair: Plan,
    practice_initial: Vec<bool>,
    configuration_path: String,
    configuration_tamper_rows: Vec<bool>,
    neighbor_path: String,
    neighbor_tamper_rows: Vec<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewFile {
    schema_version: u32,
    expected_origin: String,
    lessons: Vec<Review>,
}
fn labs() -> Vec<pack::Scenario> {
    static ALL: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        pack::load_catalog(&[
            include_str!("../scenarios/linux-files-v1.json"),
            include_str!("../scenarios/rpm-packages-v1.json"),
        ])
        .unwrap()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
    })
    .clone()
}
fn reviews() -> Vec<Review> {
    let f: ReviewFile =
        serde_json::from_str(include_str!("fixtures/rpm-packages-review.json")).unwrap();
    assert_eq!(f.schema_version, 1);
    assert!(f.expected_origin.contains("not recorded"));
    f.lessons
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let engine::Goal::RpmPackages { definition } = &s.goal else {
        panic!("RPM Goal")
    };
    definition
}
fn model(a: &Attempt) -> &core::State {
    let engine::State::RpmPackages { simulation } = &a.state else {
        panic!("RPM State")
    };
    simulation
}
fn attempt_at(s: &pack::Scenario, visited: u64) -> Attempt {
    let mut a = Attempt::new(s, visited);
    workspace::prepare(&mut a);
    teaching::prepare(
        &mut a,
        definition(s),
        s.rpm_packages_teaching.as_ref().unwrap(),
        true,
    )
    .unwrap();
    a
}
fn attempt(s: &pack::Scenario) -> Attempt {
    attempt_at(s, 9)
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "Complete bounded value differs"
    )
}
fn rows(a: &Attempt, s: &pack::Scenario) -> Vec<bool> {
    progress::current_rpm_checks(a, s)
        .iter()
        .map(|r| r.0)
        .collect()
}
fn command(a: &mut Attempt, s: &pack::Scenario, text: &str) -> Outcome {
    let engine::State::RpmPackages { simulation } = &mut a.state else {
        panic!()
    };
    let t = a.workspace.as_mut().unwrap().rpm_session.as_mut().unwrap();
    t.insert(text).unwrap();
    t.run(definition(s), simulation).unwrap()
}
fn plan(a: &mut Attempt, s: &pack::Scenario, p: &Plan) {
    assert!(!p.name.is_empty());
    assert_eq!(p.commands.len(), p.outcomes.len());
    for (text, kind) in p.commands.iter().zip(&p.outcomes) {
        let before = model(a).clone();
        let out = command(a, s, text);
        match kind.as_str() {
            "utility" => assert!(
                matches!(out, Outcome::Utility { status: 0, .. }),
                "{}: {text}: {out:?}",
                p.name
            ),
            "unsupported" => {
                assert!(
                    matches!(out, Outcome::SimulatorUnsupported { .. }),
                    "{}: {out:?}",
                    p.name
                );
                exact(model(a), &before);
            }
            _ => panic!("Unknown independently authored outcome kind"),
        }
    }
    assert_eq!(rows(a, s), p.rows, "{} / {}", s.id, p.name);
    assert_eq!(progress::current_solved(a, s), p.rows.iter().all(|v| *v));
}
fn utility(a: &mut Attempt, s: &pack::Scenario, commands: &[String]) {
    for c in commands {
        assert!(
            matches!(command(a, s, c), Outcome::Utility { status: 0, .. }),
            "{c}"
        )
    }
}

#[test]
fn actual_prerequisites_guides_and_introduction_precede_the_separate_tasks() {
    let all = labs();
    assert_eq!(all.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
    for (i, s) in all.iter().enumerate() {
        assert_eq!(
            s.prerequisites,
            vec![if i == 0 {
                "lpic1-file-navigation".to_owned()
            } else {
                IDS[i - 1].to_owned()
            }]
        );
        assert_eq!(s.steps.len(), 3);
        assert_eq!(s.guide.len(), s.steps.len());
        let l = s.rpm_packages_teaching.as_ref().unwrap();
        assert_eq!(l.concepts.len(), 3);
        assert!(
            !l.prerequisite_note.is_empty()
                && l.concepts
                    .iter()
                    .all(|c| !c.title.is_empty() && !c.text.is_empty())
        );
        assert!(l.separate_from_definition(definition(s)));
        for g in &s.guide {
            for c in &g.commands {
                terminal::validate_guidance(&c.command).unwrap();
                assert!(!c.simulator_only)
            }
        }
        for c in l
            .example_commands
            .iter()
            .chain(&l.try_change.commands)
            .chain(&l.repair)
        {
            terminal::validate_guidance(&c.command).unwrap();
        }
    }
    let normal = pack::bundled().unwrap();
    assert_eq!(normal.len(), 176);
    assert!(
        normal[..143]
            .iter()
            .all(|s| !IDS.contains(&s.id.as_str()) && s.rpm_packages_teaching.is_none())
    );
    exact(&&normal[143..147], &all);
    let new: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/shell-command-vectors-v1.json")).unwrap();
    assert_eq!(
        normal[147..150]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-command-options",
            "shell-command-dispatch",
            "shell-command-plan"
        ]
    );
    exact(&&normal[147..150], &new.scenarios);
    let nfs: pack::Pack = serde_json::from_str(include_str!("../scenarios/nfs-v1.json")).unwrap();
    exact(&&normal[150..154], &nfs.scenarios);
}

#[test]
fn initial_noop_readonly_wrong_and_partial_cases_have_independent_exact_vectors() {
    for (s, r) in labs().iter().zip(reviews()) {
        assert_eq!(s.id, r.id);
        let mut a = attempt(s);
        assert_eq!(rows(&a, s), r.initial);
        assert!(!progress::current_solved(&a, s));
        let before = serde_json::to_value(&a).unwrap();
        let initial_rows = rows(&a, s);
        for _ in 0..3 {
            assert_eq!(rows(&a, s), initial_rows);
            assert!(!progress::current_solved(&a, s))
        }
        exact(&a, &before);
        let state = model(&a).clone();
        plan(&mut a, s, &r.read_only);
        exact(model(&a), &state);
        let mut a = attempt(s);
        plan(&mut a, s, &r.wrong);
        assert!(!progress::current_solved(&a, s));
        let mut a = attempt(s);
        plan(&mut a, s, &r.partial);
        assert!(!progress::current_solved(&a, s));
    }
}

#[test]
fn references_and_two_substantive_supported_alternatives_pass_from_fresh_attempts() {
    for (s, r) in labs().iter().zip(reviews()) {
        assert_eq!(r.reference.commands, s.solution);
        assert!(r.alternatives.len() >= 2);
        let mut seen = std::collections::BTreeSet::new();
        seen.insert(r.reference.commands.clone());
        for p in std::iter::once(&r.reference).chain(&r.alternatives) {
            if !std::ptr::eq(p, &r.reference) {
                assert!(
                    seen.insert(p.commands.clone()),
                    "Distinct supported command plans, not appended query badges"
                )
            }
            assert!(p.rows.iter().all(|v| *v));
            let mut a = attempt(s);
            plan(&mut a, s, p);
            let before = serde_json::to_value(&a).unwrap();
            let result = rows(&a, s);
            for _ in 0..3 {
                assert_eq!(rows(&a, s), result);
                assert!(progress::current_solved(&a, s))
            }
            exact(&a, &before);
            // The public Goal command route reaches the same actual State, independently of console history.
            let mut through_goal = s.initial.clone();
            for c in &p.commands {
                engine::execute_for_goal(&mut through_goal, &s.goal, c).unwrap();
            }
            exact(&through_goal, &a.state);
        }
    }
}

#[test]
fn versioned_provides_epoch_and_missing_payload_are_different_real_failures() {
    let all = labs();
    let rev = reviews();
    let mut a = attempt(&all[0]);
    plan(&mut a, &all[0], &rev[0].wrong);
    assert!(model(&a).catalog().archives["lookalike"].provides["ledger-api"].is_none());
    assert!(
        !model(&a)
            .runtime()
            .installed
            .iter()
            .any(|i| i.archive == "lookalike")
    );
    let mut a = attempt(&all[1]);
    plan(&mut a, &all[1], &rev[1].wrong);
    let installed = model(&a)
        .runtime()
        .installed
        .iter()
        .find(|i| i.identity.name == "chart-engine")
        .unwrap();
    assert_eq!(
        model(&a).catalog().archives[&installed.archive]
            .evr
            .as_str(),
        "99.0-1"
    );
    assert_eq!(&rows(&a, &all[1])[..2], &[false, false]);
    let mut a = attempt(&all[2]);
    assert!(
        !model(&a)
            .runtime()
            .files
            .contains_key("/usr/share/archive-client/manual")
    );
    let before = model(&a).clone();
    plan(&mut a, &all[2], &rev[2].read_only);
    exact(model(&a), &before);
    assert!(
        model(&a)
            .runtime()
            .installed
            .iter()
            .any(|i| i.identity.name == "archive-client")
    );
    assert_eq!(&rows(&a, &all[2])[..3], &[true, false, false]);
    let mut a = attempt(&all[3]);
    let before = model(&a).clone();
    plan(&mut a, &all[3], &rev[3].wrong);
    exact(model(&a), &before);
}

#[test]
fn breaking_a_pass_invalidates_current_truth_then_a_real_repair_restores_it() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut a = attempt(s);
        plan(&mut a, s, &r.reference);
        plan(&mut a, s, &r.after_pass_break);
        a.workspace.as_mut().unwrap().checked = true;
        assert!(
            !progress::current_solved(&a, s),
            "A former success/check/history cannot grant current credit"
        );
        plan(&mut a, s, &r.repair);
        assert!(progress::current_solved(&a, s));
    }
}

#[test]
fn preservation_checks_use_actual_configuration_and_neighbor_bytes() {
    for (s, r) in labs().iter().zip(reviews()) {
        let mut solved = attempt(s);
        plan(&mut solved, s, &r.reference);
        for (path, expected) in [
            (&r.configuration_path, &r.configuration_tamper_rows),
            (&r.neighbor_path, &r.neighbor_tamper_rows),
        ] {
            let mut a = solved.clone();
            let mut runtime = model(&a).runtime().clone();
            runtime.files.get_mut(path).unwrap().contents =
                "changed virtual preservation control".into();
            a.state = engine::State::RpmPackages {
                simulation: core::State::new(model(&a).catalog().clone(), runtime).unwrap(),
            };
            assert!(work::admitted(model(&a)));
            assert_eq!(rows(&a, s), *expected);
            assert!(!progress::current_solved(&a, s));
        }
    }
}

#[test]
fn every_taught_practice_insert_run_change_and_repair_is_isolated_from_assignment() {
    for (s, r) in labs().iter().zip(reviews()) {
        let source = s.rpm_packages_teaching.as_ref().unwrap();
        let mut a = attempt(s);
        let before = serde_json::to_value(&a).unwrap();
        let mut l = teaching::Learning::new(source).unwrap();
        assert_eq!(
            l.checks(source).iter().map(|r| r.0).collect::<Vec<_>>(),
            r.practice_initial
        );
        for (n, c) in source.example_commands.iter().enumerate() {
            l.selected_example = n;
            let state = l.simulation.clone();
            l.insert(source, &c.command).unwrap();
            exact(&l.simulation, &state);
            assert!(matches!(
                l.run(source).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        assert!(l.checks(source).iter().all(|r| r.0));
        for c in &source.try_change.commands {
            l.insert(source, &c.command).unwrap();
            assert!(matches!(
                l.run(source).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        assert_eq!(
            l.checks(source).iter().map(|r| r.0).collect::<Vec<_>>(),
            source.try_change.expected
        );
        for c in &source.repair {
            l.insert(source, &c.command).unwrap();
            assert!(matches!(
                l.run(source).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        assert!(l.checks(source).iter().all(|r| r.0));
        exact(&a, &before);
        let mut expected = before;
        expected["workspace"]["rpm_learning"] = serde_json::to_value(&l).unwrap();
        a.workspace.as_mut().unwrap().rpm_learning = Some(l);
        exact(&a, &expected);
        assert_eq!(rows(&a, s), r.initial);
        assert!(!progress::current_solved(&a, s));
        utility(&mut a, s, &s.solution);
        assert!(progress::current_solved(&a, s));
        let l = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
        l.insert(source, "unfinished practice '").unwrap();
        assert!(l.valid_for(source));
        assert!(
            progress::current_solved(&a, s),
            "A valid unfinished practice command cannot revoke correct assignment outcomes"
        );
    }
}

#[test]
fn example_commands_first_task_hardcodes_and_forged_output_cannot_solve_other_contexts() {
    let all = labs();
    let first = &all[0].solution;
    for (n, s) in all.iter().enumerate() {
        let mut a = attempt(s);
        let original = model(&a).clone();
        for c in &s.rpm_packages_teaching.as_ref().unwrap().example_commands {
            assert!(matches!(
                command(&mut a, s, &c.command),
                Outcome::SimulatorUnsupported { .. }
            ));
            exact(model(&a), &original);
        }
        if n > 0 {
            for c in first {
                assert!(matches!(
                    command(&mut a, s, c),
                    Outcome::SimulatorUnsupported { .. }
                ));
                exact(model(&a), &original);
            }
        }
        let w = a.workspace.as_mut().unwrap();
        w.checked = true;
        w.rpm_session
            .as_mut()
            .unwrap()
            .entries
            .push(terminal::Entry {
                command: "rpm -q declared".into(),
                output: Outcome::Utility {
                    stdout: "all checks passed (synthetic negative display only)".into(),
                    stderr: String::new(),
                    status: 0,
                    summary: None,
                },
            });
        assert!(w.valid());
        assert!(!progress::current_solved(&a, s));
        exact(model(&a), &original);
    }
}

#[test]
fn foreign_learning_or_catalog_cannot_pass_and_prepare_refusal_is_atomic() {
    let all = labs();
    for (n, s) in all.iter().enumerate() {
        let other = &all[(n + 1) % all.len()];
        let mut a = attempt(s);
        utility(&mut a, s, &s.solution);
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            teaching::prepare(
                &mut a,
                definition(s),
                other.rpm_packages_teaching.as_ref().unwrap(),
                false
            )
            .is_err()
        );
        exact(&a, &before);
        a.workspace.as_mut().unwrap().rpm_learning =
            Some(teaching::Learning::new(other.rpm_packages_teaching.as_ref().unwrap()).unwrap());
        assert!(!progress::current_solved(&a, s));
        let before = serde_json::to_value(&a).unwrap();
        assert!(
            teaching::prepare(
                &mut a,
                definition(s),
                s.rpm_packages_teaching.as_ref().unwrap(),
                false
            )
            .is_err()
        );
        exact(&a, &before);
        let mut a = attempt(s);
        a.state = other.initial.clone();
        let before = serde_json::to_value(&a).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, &s.solution[0]).is_err());
        exact(&a, &before);
    }
}

#[test]
fn both_raw_drafts_full_histories_backup_resume_and_selected_reset_keep_other_work_exact() {
    let all = labs();
    let catalog = pack::bundled().unwrap();
    assert_eq!(catalog.len(), 176);
    for reviewed in &all {
        let registered = catalog.iter().find(|s| s.id == reviewed.id).unwrap();
        exact(registered, reviewed);
    }
    let old = &catalog[0];
    let old_attempt = Attempt::new(old, 3);
    let mut p = Progress {
        schema_version: 1,
        clock: 77,
        ..Default::default()
    };
    p.attempts.insert(old.id.clone(), old_attempt.clone());
    for s in &all {
        let mut a = attempt(s);
        utility(&mut a, s, &s.solution);
        let source = s.rpm_packages_teaching.as_ref().unwrap();
        let w = a.workspace.as_mut().unwrap();
        w.rpm_session
            .as_mut()
            .unwrap()
            .insert("unfinished assignment ; literal '")
            .unwrap();
        w.rpm_session.as_mut().unwrap().selected_path = definition(s)
            .initial
            .catalog()
            .allowed_paths
            .first()
            .cloned();
        let l = w.rpm_learning.as_mut().unwrap();
        for c in &source.example_commands {
            l.insert(source, &c.command).unwrap();
            assert!(matches!(
                l.run(source).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        l.view = View::Example;
        l.selected_example = source.example_commands.len() - 1;
        l.checked = true;
        l.insert(source, "different unfinished practice '").unwrap();
        l.terminal.selected_path = l.simulation.catalog().allowed_paths.first().cloned();
        assert!(progress::current_solved(&a, s));
        let before = serde_json::to_value(&a).unwrap();
        teaching::prepare(&mut a, definition(s), source, false).unwrap();
        exact(&a, &before);
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &catalog).unwrap();
    exact(&loaded, &p);
    exact(&loaded.attempts, &p.attempts);
    let raw = progress::read_bounded(&path, 512 * 1024).unwrap();
    let mut projected: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    projected
        .as_object_mut()
        .unwrap()
        .remove("migration_receipts");
    exact(&p, &projected);
    let out = dir.path().join("backup.json");
    backup::export(&out, &loaded, &catalog).unwrap();
    assert!(backup::inspect(&out, &catalog, &AtomicBool::new(true), &AtomicUsize::new(0)).is_err());
    assert_eq!(progress::read_bounded(&path, 512 * 1024).unwrap(), raw);
    let candidate = backup::inspect(
        &out,
        &catalog,
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    exact(&candidate.progress.attempts, &p.attempts);
    let restored = backup::restore(&path, &candidate, &catalog).unwrap();
    assert_eq!(std::fs::read(restored.safety_copy.unwrap()).unwrap(), raw);
    let mut resumed = progress::load(&path, &catalog).unwrap();
    exact(&resumed.attempts, &p.attempts);
    assert_eq!(resumed.favorites, p.favorites);
    assert_eq!(resumed.clock, p.clock);
    let selected = &all[0];
    let previous = serde_json::to_value(&resumed.attempts).unwrap();
    let visited = resumed.attempts[&selected.id].visited;
    resumed
        .attempts
        .insert(selected.id.clone(), attempt_at(selected, visited));
    for (id, a) in previous.as_object().unwrap() {
        if id != &selected.id {
            exact(&resumed.attempts[id], a)
        }
    }
    assert_eq!(resumed.favorites, p.favorites);
    assert_eq!(resumed.clock, p.clock);
    exact(&resumed.attempts[&old.id], &old_attempt);
    assert!(!progress::current_solved(
        &resumed.attempts[&selected.id],
        selected
    ));
    let reset = resumed.attempts[&selected.id].workspace.as_ref().unwrap();
    assert!(reset.rpm_session.as_ref().unwrap().command.is_empty());
    assert!(reset.rpm_session.as_ref().unwrap().entries.is_empty());
    assert!(
        reset
            .rpm_learning
            .as_ref()
            .unwrap()
            .terminal
            .entries
            .is_empty()
    );
    assert_eq!(
        reset.rpm_learning.as_ref().unwrap().view,
        View::Introduction
    );
}
