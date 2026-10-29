//! Authored unit checks: state outcomes, independent alternatives and realistic
//! negative work; numerical variants are not counted as separate scenarios.
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    storage_teaching::{self, Learning},
    storage_workspace::{self as model, Draft},
    workspace,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    id: String,
    alternate: Vec<String>,
    practice_repair: Vec<String>,
    negatives: Vec<Wrong>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wrong {
    name: String,
    start: String,
    actions: Vec<String>,
    expected_complete: bool,
    reason: String,
}
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/storage-v1.json")).unwrap()
}
fn plans() -> Vec<Plan> {
    serde_json::from_str(include_str!("fixtures/storage-content-cases.json")).unwrap()
}
fn storage(state: &State) -> &dlms_learning_lab::storage::State {
    let State::PersistentMounts { storage } = state else {
        panic!()
    };
    storage
}
fn storage_mut(state: &mut State) -> &mut dlms_learning_lab::storage::State {
    let State::PersistentMounts { storage } = state else {
        panic!()
    };
    storage
}
fn run(state: &mut State, commands: &[String]) {
    for c in commands {
        engine::execute(state, c).unwrap_or_else(|e| panic!("{c}: {e}"));
    }
}
fn solved(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    run(&mut state, &s.solution);
    state
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    storage_teaching::prepare(&mut a, s.storage_teaching.as_ref().unwrap(), true);
    a
}

#[test]
fn four_distinct_assignments_belong_to_the_normal_twentieth_unit() {
    let labs = labs();
    assert_eq!(labs.len(), 4);
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_boundary::before_len(&all), 176);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"].map(|track| {
            one_off_boundary::loop_registration::lvm_registration::before_scenarios(&all)
                .iter()
                .filter(|s| s.track == track)
                .count()
        }),
        [53, 40, 32, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all.iter().filter(|s| s.track == track).count()),
        [53, 40, 36, 57]
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|track| all[..176].iter().filter(|s| s.track == track).count()),
        [53, 36, 32, 55]
    );
    let units = dlms_learning_lab::course::bundled(&all).unwrap();
    assert_eq!(one_off_boundary::before_len(&units), 46);
    let unit = units.iter().find(|u| u.id == "persistent-mounts").unwrap();
    assert_eq!(
        unit.labs,
        labs.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    assert!(
        labs.iter()
            .all(|s| s.track == "LPIC-2" && s.unit == unit.id)
    );
    // Each task's initial fault differs: identity, saved/live access, automatic
    // selection versus explicit use, and interacting multi-volume handover.
    let initial = labs
        .iter()
        .map(|s| serde_json::to_string(&s.initial).unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(initial.len(), 4);
    assert!(
        storage(&labs[0].initial)
            .validate_configuration()
            .iter()
            .any(|e| e.kind == dlms_learning_lab::storage::ErrorKind::UnknownSource)
    );
    assert!(
        storage(&labs[1].initial)
            .validate_configuration()
            .is_empty()
    );
    assert_eq!(storage(&labs[2].initial).inventory().volumes().len(), 3);
    assert_eq!(storage(&labs[3].initial).inventory().volumes().len(), 4);
}

#[test]
fn longer_practice_durations_search_favorites_and_resume_remain_composable() {
    use dlms_learning_lab::catalog::{self, Filter};
    let labs = labs();
    let mut f = Filter {
        track: "LPIC-2".into(),
        unit: "persistent-mounts".into(),
        max_minutes: 30,
        ..Default::default()
    };
    let selected = labs
        .iter()
        .filter(|s| catalog::matches(s, &f, false, false, false))
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 3);
    assert!(!selected.contains(&"lpic2-mount-handover"));
    f.max_minutes = 0;
    f.search = "maintenance".into();
    assert!(catalog::matches(&labs[2], &f, false, false, false));
    f.search.clear();
    f.favorites = true;
    assert!(!catalog::matches(&labs[2], &f, true, false, false));
    assert!(catalog::matches(&labs[2], &f, true, false, true));
    f.recent = true;
    f.status = "In progress".into();
    assert!(!catalog::matches(&labs[2], &f, false, false, true));
    assert!(catalog::matches(&labs[2], &f, true, false, true));
    assert!(!catalog::matches(&labs[2], &f, true, true, true));
}

#[test]
fn taught_references_independent_alternatives_and_every_negative_grade_resulting_state() {
    let plans = plans();
    let labs = labs();
    assert_eq!(plans.len(), 4);
    let mut negative_count = 0;
    for (s, plan) in labs.iter().zip(&plans) {
        assert_eq!(s.id, plan.id);
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(engine::solved(&solved(s), &s.goal));
        let mut alternative = s.initial.clone();
        run(&mut alternative, &plan.alternate);
        assert!(engine::solved(&alternative, &s.goal), "{} alternate", s.id);
        assert_ne!(s.solution, plan.alternate);
        for wrong in &plan.negatives {
            assert!(!wrong.expected_complete);
            assert!(!wrong.reason.trim().is_empty());
            let mut state = match wrong.start.as_str() {
                "initial" => s.initial.clone(),
                "solved" => solved(s),
                _ => panic!("unknown controlled start"),
            };
            run(&mut state, &wrong.actions);
            assert!(
                !engine::solved(&state, &s.goal),
                "{} {}: {}",
                s.id,
                wrong.name,
                wrong.reason
            );
            assert!(engine::checks(&state, &s.goal).iter().any(|c| !c.0));
            negative_count += 1;
        }
    }
    assert_eq!(negative_count, 33);
}

#[test]
fn every_required_operation_is_available_and_save_only_or_partial_work_never_finishes() {
    for s in labs() {
        let mut state = s.initial.clone();
        let mut has_partial = false;
        for (i, command) in s.solution.iter().enumerate() {
            engine::execute(&mut state, command).unwrap();
            if i + 1 < s.solution.len() {
                assert!(
                    !engine::solved(&state, &s.goal),
                    "{} should still need its live work",
                    s.id
                );
                has_partial |= engine::checks(&state, &s.goal).iter().any(|c| c.0);
            }
        }
        assert!(has_partial);
        assert!(engine::solved(&state, &s.goal));
        let before = state.clone();
        for text in ["help", "blkid", "mount", "cat /etc/fstab"] {
            model::command(storage_mut(&mut state), text).unwrap();
        }
        assert_eq!(state, before);
        assert!(
            model::command(storage_mut(&mut state), "mount -a")
                .unwrap_err()
                .contains("Unsupported")
        );
        assert_eq!(state, before);
    }
}

#[test]
fn separate_examples_teach_supported_commands_and_have_real_experiments_and_repairs() {
    let labs = labs();
    let plans = plans();
    for (s, plan) in labs.iter().zip(&plans) {
        let lesson = s.storage_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(lesson.fixture.inventory().volumes().values().all(|v| {
            !storage(&s.initial)
                .inventory()
                .volumes()
                .values()
                .any(|original| original.uuid == v.uuid)
        }));
        let mut learning = Learning::new(lesson);
        let original = learning.state.clone();
        learning.fill_example(lesson).unwrap();
        assert_eq!(learning.state, original);
        assert!(!learning.checked);
        assert!(learning.draft.save(&mut learning.state));
        for command in &lesson.example_solution {
            model::adapter(&mut learning.state, command).unwrap();
        }
        learning.draft = Draft::new(&learning.state);
        learning.checked = true;
        assert!(learning.checks(lesson).iter().all(|c| c.0));
        for action in &lesson.try_actions {
            model::adapter(&mut learning.state, action).unwrap();
        }
        learning.draft = Draft::new(&learning.state);
        assert!(
            learning.checks(lesson).iter().any(|c| !c.0),
            "{} experiment must expose the stated misconception",
            s.id
        );
        for action in &plan.practice_repair {
            model::adapter(&mut learning.state, action).unwrap();
        }
        learning.draft = Draft::new(&learning.state);
        assert!(
            learning.checks(lesson).iter().all(|c| c.0),
            "{} experiment repair",
            s.id
        );
        let mut a = attempt(s);
        a.workspace.as_mut().unwrap().storage_learning = Some(learning);
        assert_eq!(a.state, s.initial);
        assert!(!progress::current_solved(&a, s));
        for c in &lesson.example_commands {
            assert!(!c.simulator_only);
            assert!(!c.command.starts_with("storage "));
        }
    }
    assert!(
        labs[0]
            .storage_teaching
            .as_ref()
            .unwrap()
            .concepts
            .iter()
            .any(|c| c.text.contains("auto"))
    );
    assert!(
        labs[1]
            .storage_teaching
            .as_ref()
            .unwrap()
            .example_commands
            .iter()
            .any(|c| c.command.starts_with("umount "))
    );
    assert!(
        labs[2]
            .storage_teaching
            .as_ref()
            .unwrap()
            .concepts
            .iter()
            .any(|c| c.text.contains("noauto"))
    );
}

#[test]
fn saved_and_unsaved_damage_clear_current_success_and_selected_reset_preserves_other_work() {
    let mut progress = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let all = pack::bundled().unwrap();
    for s in &all {
        progress.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let untouched = serde_json::to_vec(&progress.attempts[&all[0].id]).unwrap();
    for s in labs() {
        let mut a = attempt(&s);
        a.state = solved(&s);
        a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
        assert!(progress::current_solved(&a, &s));
        a.workspace
            .as_mut()
            .unwrap()
            .storage_draft
            .as_mut()
            .unwrap()
            .text
            .push_str("\n# pending");
        assert!(!progress::current_solved(&a, &s));
        a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
        let target = storage(&a.state).live().keys().next().unwrap().clone();
        storage_mut(&mut a.state).umount(&target).unwrap();
        assert!(!progress::current_solved(&a, &s));
        progress.attempts.insert(s.id.clone(), a);
        progress.favorites.insert(s.id.clone());
        let reset = attempt(&s);
        progress.attempts.insert(s.id.clone(), reset);
        assert_eq!(progress.attempts[&s.id].state, s.initial);
        assert!(progress.favorites.contains(&s.id));
    }
    assert_eq!(
        serde_json::to_vec(&progress.attempts[&all[0].id]).unwrap(),
        untouched
    );
}

#[test]
fn all_prior_attempts_and_new_assignment_example_drafts_resume_without_fabricated_work() {
    let all = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in all.iter().filter(|s| s.unit != "persistent-mounts") {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let old = serde_json::to_value(&p.attempts).unwrap();
    for s in labs() {
        let mut a = attempt(&s);
        a.workspace
            .as_mut()
            .unwrap()
            .storage_draft
            .as_mut()
            .unwrap()
            .text = "incomplete draft".into();
        a.workspace
            .as_mut()
            .unwrap()
            .storage_learning
            .as_mut()
            .unwrap()
            .draft
            .text = "separate example draft".into();
        p.attempts.insert(s.id.clone(), a);
    }
    let encoded = serde_json::to_vec(&p).unwrap();
    let loaded = progress::decode(&encoded, &all).unwrap();
    assert_eq!(one_off_boundary::before_len(&loaded.attempts), 176);
    for s in all.iter().filter(|s| s.unit != "persistent-mounts") {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[&s.id]).unwrap(),
            old[&s.id]
        );
    }
    for s in labs() {
        let a = &loaded.attempts[&s.id];
        assert!(!progress::current_solved(a, &s));
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .storage_draft
                .as_ref()
                .unwrap()
                .text,
            "incomplete draft"
        );
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .storage_learning
                .as_ref()
                .unwrap()
                .draft
                .text,
            "separate example draft"
        );
    }
}
