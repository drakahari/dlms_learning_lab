//! Unregistered original search lessons. Every command uses the in-process VM.
use dlms_learning_lab::{
    backup, course,
    engine::{self, Goal, State},
    filesystem::{Access, Requirement},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    shell,
    terminal_teaching::{self, Learning},
    workspace,
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicUsize},
    },
};

const IDS: [&str; 3] = [
    "lpic1-search-release-set",
    "lpic1-search-incomplete-intake",
    "lpic1-search-private-handover",
];
const UNIT: &str = "linux-file-search";
#[derive(Deserialize)]
struct Review {
    id: String,
    reference: Vec<String>,
    alternates: Vec<Vec<String>>,
    partial: Vec<String>,
    wrong: Vec<Wrong>,
    malformed: Vec<String>,
    example: Experiment,
}
#[derive(Deserialize)]
struct Wrong {
    name: String,
    commands: Vec<String>,
    misconception: String,
    failed_rows: Vec<usize>,
}
#[derive(Deserialize)]
struct Experiment {
    experiment: Vec<String>,
    expected_errors: Vec<usize>,
    repair: Vec<String>,
    failed_rows: Vec<usize>,
}
#[derive(Deserialize)]
struct Original {
    id: String,
    version: u32,
    initial: State,
    goal: Goal,
}
fn reviews() -> Vec<Review> {
    serde_json::from_str(include_str!("fixtures/linux-find-content-review.json")).unwrap()
}
fn originals() -> Vec<Original> {
    serde_json::from_str(include_str!("fixtures/linux-find-original-catalog.json")).unwrap()
}
fn catalog() -> &'static Vec<Scenario> {
    static ALL: OnceLock<Vec<Scenario>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut existing = pack::bundled().unwrap();
        existing.retain(|s| !IDS.contains(&s.id.as_str()));
        let normal = serde_json::to_string(&pack::Pack {
            schema_version: 1,
            scenarios: existing,
        })
        .unwrap();
        // Resolve actual prerequisites against the real complete catalog. Do not
        // remove the substantive old-lab prerequisite to load this private pack.
        pack::load_catalog(&[
            normal.as_str(),
            include_str!("../scenarios/linux-find-v1.json"),
        ])
        .unwrap()
    })
}
fn lab(id: &str) -> &'static Scenario {
    catalog().iter().find(|s| s.id == id).unwrap()
}
fn tree(state: &State) -> &dlms_learning_lab::filesystem::Tree {
    let State::Filesystem { tree } = state else {
        panic!("virtual Tree required")
    };
    tree
}
fn requirements(lab: &Scenario) -> &[Requirement] {
    let Goal::Filesystem { requirements } = &lab.goal else {
        panic!("typed file goal")
    };
    requirements
}
fn apply(state: &mut State, lab: &Scenario, commands: &[String]) {
    for command in commands {
        engine::execute_for_goal(state, &lab.goal, command)
            .unwrap_or_else(|error| panic!("{}: {command}: {error}", lab.id));
        assert!(pack::validate_state(state).is_ok());
    }
}
fn passed(lab: &Scenario, commands: &[String]) -> State {
    let mut state = lab.initial.clone();
    apply(&mut state, lab, commands);
    assert!(
        engine::solved(&state, &lab.goal),
        "{}: {:?}",
        lab.id,
        engine::checks(&state, &lab.goal)
    );
    state
}
fn outcomes(learning: &Learning, lab: &Scenario) -> Vec<bool> {
    lab.terminal_teaching
        .as_ref()
        .unwrap()
        .outcomes
        .iter()
        .map(|r| r.check(&learning.tree).0)
        .collect()
}
fn practice_execute(learning: &mut Learning, command: &str, expected_error: bool) {
    let mut control = State::Filesystem {
        tree: learning.tree.clone(),
    };
    let result = engine::execute(&mut control, command);
    assert_eq!(result.is_err(), expected_error, "{command}: {result:?}");
    learning.command = command.into();
    learning.execute();
    assert_eq!(learning.tree, *tree(&control));
    if expected_error {
        assert!(
            learning
                .transcript
                .last()
                .unwrap()
                .contains("Search incomplete")
        );
    }
}

#[test]
fn original113_specs_are_preserved_and_proposed_unit_uses_real_prerequisites_in_order() {
    let normal = pack::bundled().unwrap();
    let old = originals();
    assert_eq!(old.len(), 113);
    let old_ids: BTreeSet<_> = old.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(old_ids.len(), 113);
    for original in &old {
        let current = normal.iter().find(|s| s.id == original.id).unwrap();
        assert_eq!(current.version, original.version, "{}", original.id);
        assert_eq!(
            serde_json::to_value(&current.initial).unwrap(),
            serde_json::to_value(&original.initial).unwrap(),
            "{} initial",
            original.id
        );
        assert_eq!(
            serde_json::to_value(&current.goal).unwrap(),
            serde_json::to_value(&original.goal).unwrap(),
            "{} goal",
            original.id
        );
    }
    for id in IDS {
        assert!(!old_ids.contains(id));
        let s = lab(id);
        assert!(requirements(s).len() <= 20);
    }
    let mut units = course::bundled(&normal).unwrap();
    units.retain(|u| u.id != UNIT);
    let old_order: Vec<_> = units.iter().map(|u| u.id.clone()).collect();
    let proposed: course::Unit =
        serde_json::from_str(include_str!("fixtures/linux-find-unit-proposal.json")).unwrap();
    assert_eq!(proposed.prerequisites, ["linux-file-access"]);
    assert_eq!(proposed.labs, IDS);
    let after = units
        .iter()
        .position(|u| u.id == "linux-file-access")
        .unwrap()
        + 1;
    units.insert(after, proposed);
    let checked = course::load(
        &serde_json::json!({"version":1,"units":units}).to_string(),
        catalog(),
    )
    .unwrap();
    assert_eq!(
        checked
            .iter()
            .filter(|u| u.id != UNIT)
            .map(|u| u.id.clone())
            .collect::<Vec<_>>(),
        old_order
    );
    assert!(
        checked.iter().position(|u| u.id == UNIT).unwrap()
            < checked
                .iter()
                .position(|u| u.id == "linux-processes")
                .unwrap()
    );
    for s in catalog().iter().filter(|s| IDS.contains(&s.id.as_str())) {
        assert!(
            s.prerequisites
                .iter()
                .all(|id| catalog().iter().any(|other| &other.id == id))
        );
    }
}

#[test]
fn references_and_two_independent_alternates_reach_complete_current_outcomes_without_history() {
    for review in reviews() {
        let lab = lab(&review.id);
        assert_eq!(review.reference, lab.solution);
        assert!(review.alternates.len() >= 2);
        let reference = passed(lab, &review.reference);
        for alternate in &review.alternates {
            assert_ne!(alternate, &review.reference);
            let alternative = passed(lab, alternate);
            let mut attempt = Attempt::new(lab, 1);
            attempt.state = alternative;
            assert!(attempt.transcript.is_empty());
            assert!(progress::current_solved(&attempt, lab));
        }
        assert!(
            engine::checks(&reference, &lab.goal)
                .iter()
                .all(|(pass, _)| *pass)
        );
    }
}

#[test]
fn wrong_and_partial_plans_fail_the_specific_promised_rows_with_useful_current_feedback() {
    for review in reviews() {
        let lab = lab(&review.id);
        let mut partial = lab.initial.clone();
        apply(&mut partial, lab, &review.partial);
        assert!(!engine::solved(&partial, &lab.goal));
        assert!(!workspace::issues(&partial, &lab.goal).is_empty());
        for wrong in review.wrong {
            assert!(!wrong.misconception.trim().is_empty());
            let mut state = lab.initial.clone();
            apply(&mut state, lab, &wrong.commands);
            let checks = engine::checks(&state, &lab.goal);
            assert_eq!(checks.len(), requirements(lab).len());
            for row in wrong.failed_rows {
                assert!(
                    !checks[row].0,
                    "{} / {} row{row}: {:?}",
                    lab.id, wrong.name, checks
                );
            }
            assert!(
                !engine::solved(&state, &lab.goal),
                "{} {}",
                lab.id,
                wrong.name
            );
            let feedback = workspace::issues(&state, &lab.goal);
            assert!(!feedback.is_empty() && feedback.iter().all(|s| !s.trim().is_empty()));
        }
    }
}

fn break_row(state: &mut State, requirement: &Requirement) {
    let State::Filesystem { tree: t } = state else {
        panic!()
    };
    match requirement {
        Requirement::DirectoryEntries { path, .. } => {
            let name = format!("{path}/.unrequested-counterexample");
            let mut n = t.nodes[path].clone();
            n.directory = false;
            n.mode = 0o600;
            t.nodes.insert(name.clone(), n);
            t.files.insert(name, "unrequested text\n".into());
        }
        Requirement::Contents { path, .. } => {
            t.files.insert(
                path.clone(),
                "Wrong current text; not an authored receipt.\n".into(),
            );
        }
        Requirement::Mode { path, mode } => t.nodes.get_mut(path).unwrap().mode = mode ^ 0o004,
        Requirement::Ownership { path, group, .. } => {
            t.nodes.get_mut(path).unwrap().group =
                if group == "review" { "staff" } else { "review" }.into()
        }
        Requirement::Access { path, allowed, .. } => {
            if *allowed {
                t.nodes.get_mut(path).unwrap().mode = 0;
            } else {
                for node in t.nodes.values_mut().filter(|n| n.directory) {
                    node.mode = 0o777;
                }
                t.nodes.get_mut(path).unwrap().mode = 0o777;
            }
        }
        Requirement::Mask { mask } => t.umask = mask ^ 0o077,
        Requirement::Directory { path } => {
            t.cwd = if path == "/" {
                t.home.clone()
            } else {
                "/".into()
            }
        }
        Requirement::Entry { path, .. } => {
            t.nodes.remove(path);
            t.files.remove(path);
        }
        Requirement::Absent { path } => {
            let mut n = t.nodes["/work"].clone();
            n.directory = false;
            t.nodes.insert(path.clone(), n);
            t.files.insert(path.clone(), String::new());
        }
    }
}
#[test]
fn every_goal_predicate_has_a_valid_current_counterexample_and_fresh_repair() {
    for id in IDS {
        let lab = lab(id);
        let good = passed(lab, &lab.solution);
        for (index, requirement) in requirements(lab).iter().enumerate() {
            let mut broken = good.clone();
            break_row(&mut broken, requirement);
            assert!(
                pack::validate_state(&broken).is_ok(),
                "{id} row{index} must be a valid counterexample"
            );
            let checks = engine::checks(&broken, &lab.goal);
            assert!(!checks[index].0, "{id} row{index}: {:?}", checks[index]);
            assert!(!engine::solved(&broken, &lab.goal));
            let repaired = good.clone();
            assert!(engine::checks(&repaired, &lab.goal)[index].0);
            assert!(engine::solved(&repaired, &lab.goal));
        }
    }
}

#[test]
fn working_examples_are_separate_fill_only_editable_and_experiments_repair_their_exact_outcomes() {
    for review in reviews() {
        let lab = lab(&review.id);
        let lesson = lab.terminal_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert_ne!(&lesson.fixture, tree(&lab.initial));
        assert_ne!(lesson.fixture.cwd, tree(&lab.initial).cwd);
        let assignment = lab.initial.clone();
        let mut learning = Learning::new(lesson);
        let mutation = lesson
            .example
            .iter()
            .find(|c| c.starts_with("cp ") || c.starts_with("chmod "))
            .unwrap();
        learning.command = mutation.clone();
        assert_eq!(learning.tree, lesson.fixture);
        assert!(learning.transcript.is_empty());
        practice_execute(&mut learning, mutation, false);
        assert_ne!(learning.tree, lesson.fixture);
        learning = Learning::new(lesson);
        for command in &lesson.example {
            practice_execute(&mut learning, command, false);
        }
        assert!(outcomes(&learning, lab).iter().all(|pass| *pass));
        for (index, command) in review.example.experiment.iter().enumerate() {
            practice_execute(
                &mut learning,
                command,
                review.example.expected_errors.contains(&index),
            );
        }
        let changed = outcomes(&learning, lab);
        for row in review.example.failed_rows {
            assert!(!changed[row], "{} experiment row{row}", lab.id);
        }
        for command in &review.example.repair {
            practice_execute(&mut learning, command, false);
        }
        assert!(outcomes(&learning, lab).iter().all(|pass| *pass));
        assert_eq!(lab.initial, assignment);
        learning.command = "touch independent-example-file".into();
        learning.execute();
        assert!(learning.valid());
        assert_ne!(learning.tree, lesson.fixture);
        assert_eq!(lab.initial, assignment);
        let reset = Learning::new(lesson);
        assert_eq!(reset.tree, lesson.fixture);
        assert!(reset.command.is_empty() && reset.transcript.is_empty());
    }
}

#[test]
fn malformed_commands_are_atomic_even_after_success_and_invalid_pack_names_are_rejected() {
    for review in reviews() {
        let lab = lab(&review.id);
        let good = passed(lab, &review.reference);
        for command in review.malformed {
            let mut state = good.clone();
            let error = engine::execute_for_goal(&mut state, &lab.goal, &command).unwrap_err();
            assert!(!error.trim().is_empty());
            assert_eq!(state, good);
            assert!(engine::solved(&state, &lab.goal));
        }
    }
    let mut content: Value =
        serde_json::from_str(include_str!("../scenarios/linux-find-v1.json")).unwrap();
    content["scenarios"][0]["goal"]["requirements"][0]["names"] =
        serde_json::json!(["duplicate", "duplicate"]);
    assert!(
        pack::load_catalog(&[
            serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: pack::bundled().unwrap()
            })
            .unwrap()
            .as_str(),
            content.to_string().as_str()
        ])
        .is_err()
    );
}

#[test]
fn directory_repair_accepts_owner500_and700_but_requires_both_read_and_search_now() {
    let lab = lab(IDS[1]);
    let review = reviews().into_iter().find(|r| r.id == lab.id).unwrap();
    let mut known_path = lab.initial.clone();
    apply(&mut known_path, lab, &review.reference[3..5]);
    assert!(
        !engine::solved(&known_path, &lab.goal),
        "Known path copies do not repair directory read"
    );
    assert!(!engine::checks(&known_path, &lab.goal)[3].0);
    let query: Vec<String> = ["/work/intake", "-type", "f", "-name", "*-receipt.txt"]
        .map(String::from)
        .into();
    assert!(!tree(&known_path).find(&query).unwrap().complete());
    for mode in ["500", "700"] {
        let mut state = known_path.clone();
        engine::execute_for_goal(
            &mut state,
            &lab.goal,
            &format!("chmod {mode} /work/intake/pending"),
        )
        .unwrap();
        assert!(engine::solved(&state, &lab.goal));
        assert!(tree(&state).find(&query).unwrap().complete());
        for user in ["blair", "casey"] {
            assert!(!tree(&state).access(user, "/work/intake/pending", Access::ListNames));
            assert!(!tree(&state).access(user, "/work/intake/pending", Access::SearchDirectory));
        }
    }
    assert!(
        !requirements(lab)
            .iter()
            .any(|r| matches!(r,Requirement::Mode{path,..} if path=="/work/intake/pending"))
    );
}

#[test]
fn exact_shared_inventory_and_expected_public_text_block_both_private_copy_routes() {
    let lab = lab(IDS[2]);
    let good = passed(lab, &lab.solution);
    let mut extra = good.clone();
    engine::execute_for_goal(
        &mut extra,
        &lab.goal,
        "cp /work/releases/internal/private.ready /work/review/.renamed-note",
    )
    .unwrap();
    let checks = engine::checks(&extra, &lab.goal);
    assert!(!checks[0].0 && checks[2].0);
    assert!(checks[0].1.contains("unrequested"));
    let mut overwrite = good.clone();
    engine::execute_for_goal(
        &mut overwrite,
        &lab.goal,
        "cp /work/releases/internal/private.ready /work/review/south.txt",
    )
    .unwrap();
    let checks = engine::checks(&overwrite, &lab.goal);
    assert!(checks[0].0 && !checks[2].0);
    for path in ["/work/review/north.txt", "/work/review/south.txt"] {
        assert!(tree(&good).access("blair", path, Access::ReadFile));
    }
    assert!(!tree(&good).access("blair", "/work/archive/obsolete.txt", Access::ReadFile));
    assert!(!tree(&good).access(
        "casey",
        "/work/releases/internal/private.ready",
        Access::ReadFile
    ));
    let mut local = good.clone();
    engine::execute_for_goal(
        &mut local,
        &lab.goal,
        "cp /work/releases/internal/private.ready /work/local-private-copy",
    )
    .unwrap();
    assert!(
        engine::solved(&local, &lab.goal),
        "Declared review inventory is not a global identical-Tree requirement"
    );
}

fn historical_records(p: &Progress) -> BTreeMap<String, Vec<Value>> {
    let mut out = BTreeMap::new();
    for (id, a) in &p.attempts {
        let mut values = Vec::new();
        let current = if let State::Script { run, .. } = &a.state {
            run.as_ref()
        } else {
            None
        };
        let workspace = a.workspace.as_ref();
        for record in current
            .into_iter()
            .chain(workspace.and_then(|w| w.script_history.as_ref()))
            .chain(
                workspace
                    .into_iter()
                    .flat_map(|w| w.script_history_more.iter()),
            )
        {
            if [6, 7].contains(&record.engine_revision) {
                let value = serde_json::to_value(record).unwrap();
                if !values.contains(&value) {
                    values.push(value);
                }
            }
        }
        if !values.is_empty() {
            out.insert(id.clone(), values);
        }
    }
    out
}
fn assert_historical_records(p: &Progress, wanted: &BTreeMap<String, Vec<Value>>) {
    let actual = historical_records(p);
    for (id, records) in wanted {
        for record in records {
            assert!(
                actual[id].contains(record),
                "{id} genuine earlier record must remain exact"
            );
        }
    }
}
#[test]
fn actual_current8_records_and_genuine6_7_history_survive_drafts_save_export_restore_and_selected_reset()
 {
    let all = catalog();
    let old = originals();
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v7-synthetic-progress-original.json"),
        all,
    )
    .unwrap();
    let genuine = historical_records(&p);
    assert!(
        genuine
            .values()
            .flatten()
            .any(|r| r["engine_revision"] == 6)
    );
    assert!(
        genuine
            .values()
            .flatten()
            .any(|r| r["engine_revision"] == 7)
    );
    let mut actual_current = 0;
    for original in &old {
        let s = all.iter().find(|s| s.id == original.id).unwrap();
        let a = p
            .attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(s, 1));
        if let Goal::Script { cases } = &s.goal {
            workspace::prepare(a);
            let source = s.solution[0].clone();
            let results: Vec<_> = cases.iter().map(|c| shell::run(&source, c)).collect();
            assert!(
                cases.iter().zip(&results).all(|(c, r)| c.accepts(r)),
                "{} currentVM",
                s.id
            );
            a.state = State::Script {
                source: source.clone(),
                run: Some(shell::Record {
                    engine_revision: shell::EVALUATOR_REVISION,
                    source,
                    results,
                }),
            };
            assert!(engine::solved(&a.state, &s.goal));
            actual_current += 1;
        }
    }
    assert_eq!(actual_current, 40);
    assert_historical_records(&p, &genuine);
    for id in IDS {
        let s = lab(id);
        let lesson = s.terminal_teaching.as_ref().unwrap();
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        terminal_teaching::prepare(&mut a, lesson, true);
        apply(&mut a.state, s, &s.solution);
        a.workspace.as_mut().unwrap().command = "cp 'unfinished assignment".into();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap();
        for command in &lesson.example {
            learning.command = command.clone();
            learning.execute();
        }
        learning.command = "touch independent-practice-edit".into();
        learning.execute();
        learning.command = "find . -name 'unfinished practice".into();
        learning.view = View::Example;
        assert!(learning.valid());
        assert!(progress::current_solved(&a, s));
        p.attempts.insert(id.into(), a);
        p.favorites.insert(id.into());
    }
    let desired = serde_json::to_value(&p.attempts).unwrap();
    let favorites = p.favorites.clone();
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("progress.json");
    progress::save(&target, &p).unwrap();
    let original_bytes = std::fs::read(&target).unwrap();
    let resumed = progress::load(&target, all).unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), original_bytes);
    assert_eq!(serde_json::to_value(&resumed.attempts).unwrap(), desired);
    assert_eq!(resumed.favorites, favorites);
    assert_historical_records(&resumed, &genuine);
    let export = dir.path().join("export.json");
    backup::export(&export, &resumed, all).unwrap();
    let candidate =
        backup::inspect(&export, all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert!(candidate.rechecked >= actual_current);
    // Inspect deliberately refreshes retained old example results. Compute this
    // one documented projection using the current VM; every other field remains
    // exactly as exported, including all genuine earlier main-run histories.
    let mut inspected = desired.clone();
    for (id, attempt) in &resumed.attempts {
        if let Some(learning) = attempt
            .workspace
            .as_ref()
            .and_then(|w| w.script_learning.as_ref())
            && learning.example_result.is_some()
        {
            let scenario = all.iter().find(|s| s.id == *id).unwrap();
            let fixture = &scenario.teaching.as_ref().unwrap().fixture;
            let result = shell::run(&learning.example_source, fixture);
            inspected[id]["workspace"]["script_learning"]["example_result"] =
                serde_json::to_value(result).unwrap();
            inspected[id]["workspace"]["script_learning"]["example_revision"] =
                serde_json::json!(shell::EVALUATOR_REVISION);
        }
    }
    assert!(
        serde_json::to_value(&candidate.progress.attempts).unwrap() == inspected,
        "Inspection changes only explicitly refreshed example results"
    );
    assert_historical_records(&candidate.progress, &genuine);
    let old_attempts: BTreeMap<_, _> = old
        .iter()
        .map(|s| (s.id.clone(), inspected[&s.id].clone()))
        .collect();
    let safety = backup::restore(&target, &candidate, all)
        .unwrap()
        .safety_copy
        .unwrap();
    assert_eq!(std::fs::read(safety).unwrap(), original_bytes);
    let mut restored = progress::load(&target, all).unwrap();
    assert!(
        serde_json::to_value(&restored.attempts).unwrap() == inspected,
        "Confirmed restore must exactly retain the inspected projection"
    );
    assert_historical_records(&restored, &genuine);
    let mut invalid = progress::decode(&serde_json::to_vec(&restored).unwrap(), all).unwrap();
    invalid
        .attempts
        .get_mut(IDS[0])
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .command = "x".repeat(4097);
    let safe_bytes = std::fs::read(&target).unwrap();
    assert!(progress::save(&target, &invalid).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), safe_bytes);
    assert!(progress::decode(&serde_json::to_vec(&invalid).unwrap(), all).is_err());
    let reset_lab = lab(IDS[1]);
    let mut reset = Attempt::new(reset_lab, 3);
    workspace::prepare(&mut reset);
    terminal_teaching::prepare(
        &mut reset,
        reset_lab.terminal_teaching.as_ref().unwrap(),
        true,
    );
    restored.attempts.insert(IDS[1].into(), reset);
    assert!(!progress::current_solved(
        &restored.attempts[IDS[1]],
        reset_lab
    ));
    assert_eq!(
        serde_json::to_value(&restored.attempts[IDS[0]]).unwrap(),
        desired[IDS[0]]
    );
    assert_eq!(
        serde_json::to_value(&restored.attempts[IDS[2]]).unwrap(),
        desired[IDS[2]]
    );
    for (id, value) in old_attempts {
        assert_eq!(
            serde_json::to_value(&restored.attempts[&id]).unwrap(),
            value
        );
    }
    assert_eq!(restored.favorites, favorites);
    assert_historical_records(&restored, &genuine);
    progress::save(&target, &restored).unwrap();
    assert_eq!(
        serde_json::to_value(&progress::load(&target, all).unwrap().attempts).unwrap(),
        serde_json::to_value(&restored.attempts).unwrap()
    );
}

#[test]
fn taught_commands_run_in_their_declared_context_and_introductions_explain_required_choices() {
    for id in IDS {
        let lab = lab(id);
        let mut state = lab.initial.clone();
        for step in &lab.guide {
            for example in &step.commands {
                engine::execute_for_goal(&mut state, &lab.goal, &example.command).unwrap();
                assert!(!example.explanation.trim().is_empty());
            }
        }
        let lesson = lab.terminal_teaching.as_ref().unwrap();
        assert!(
            lesson.example.len() <= 8 && lesson.concepts.len() <= 6 && lesson.outcomes.len() <= 20
        );
        let intro = lesson
            .concepts
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !intro.contains("typed virtual Tree")
                && !intro.contains("Check example button")
                && !intro.contains("BIND")
        );
        assert!(!lab.objective.trim().is_empty() && lab.hints.len() >= 3);
        if id == IDS[0] {
            for word in ["quotes", "leading dot", "depth0", "AND", "incomplete"] {
                assert!(intro.contains(word), "missing prerequisite concept {word}");
            }
        }
    }
}
