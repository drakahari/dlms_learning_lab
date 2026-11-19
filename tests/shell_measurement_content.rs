//! Three authored lessons, validated with their real prerequisite closure before registration.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};
const IDS: [&str; 3] = [
    "shell-counter-rates",
    "shell-cpu-share",
    "shell-measurement-recovery",
];
fn drafted() -> Vec<pack::Scenario> {
    let p: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v15.json")).unwrap();
    p.scenarios
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
}
fn baseline() -> &'static Vec<pack::Scenario> {
    static B: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    B.get_or_init(|| pack::accepted396().unwrap())
}
fn authored() -> &'static Vec<pack::Scenario> {
    static S: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        let mut by_id: BTreeMap<_, _> = baseline()
            .iter()
            .chain(drafted().iter())
            .map(|s| (s.id.clone(), s.clone()))
            .collect();
        let mut pending: Vec<_> = IDS.iter().map(|s| s.to_string()).collect();
        let mut keep = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if keep.insert(id.clone()) {
                let s = by_id.get(&id).unwrap();
                pending.extend(s.prerequisites.iter().cloned());
            }
        }
        let texts: Vec<_> = keep
            .iter()
            .map(|id| {
                serde_json::to_string(&pack::Pack {
                    schema_version: 1,
                    scenarios: vec![by_id.remove(id).unwrap()],
                })
                .unwrap()
            })
            .collect();
        let borrowed: Vec<_> = texts.iter().map(String::as_str).collect();
        let all = pack::load_catalog(&borrowed).unwrap();
        IDS.iter()
            .map(|id| all.iter().find(|s| s.id == *id).unwrap().clone())
            .collect()
    })
}
fn cases(s: &pack::Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn state(s: &pack::Scenario, text: &str) -> State {
    State::Script {
        source: text.into(),
        run: Some(Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: text.into(),
            results: cases(s).iter().map(|c| shell::run(text, c)).collect(),
        }),
    }
}
#[test]
fn every_source_and_example_validates_with_its_actual_prerequisites_and_taught_solution() {
    let labs = authored();
    assert_eq!(labs.len(), 3);
    for s in labs {
        assert_eq!(s.track, "Bash/Shell");
        assert_eq!(s.unit, "shell-measurement-reports");
        assert_eq!(cases(s).len(), 12);
        let teaching = s.teaching.as_ref().unwrap();
        teaching.validate().unwrap();
        let actual = shell::run(&teaching.example, &teaching.fixture);
        assert!(teaching.fixture.accepts(&actual));
        assert!(engine::solved(&state(s, &s.solution[0]), &s.goal));
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(
            teaching
                .concepts
                .iter()
                .all(|c| !c.text.contains("local scalar"))
        );
    }
}
#[test]
fn independent_starters_retain_meaningful_work_and_partial_repair_never_awards_success() {
    for s in authored() {
        let State::Script { source, .. } = &s.initial else {
            panic!()
        };
        assert_ne!(source, &s.solution[0]);
        let initial = state(s, source);
        assert!(!engine::solved(&initial, &s.goal));
        if s.id == IDS[2] {
            let State::Script { run: Some(run), .. } = initial else {
                panic!()
            };
            let flags: Vec<_> = run
                .results
                .iter()
                .zip(cases(s))
                .map(|(r, c)| c.accepts(r))
                .collect();
            assert!(flags.iter().any(|p| *p) && flags.iter().any(|p| !*p));
            assert!(run.results.iter().all(|r| r.error.is_none()));
        }
    }
}
#[test]
fn edits_invalidate_a_successful_run_and_expected_nonzero_cases_can_pass() {
    for s in authored() {
        let mut good = state(s, &s.solution[0]);
        assert!(engine::solved(&good, &s.goal));
        let State::Script {
            source,
            run: Some(run),
        } = &mut good
        else {
            panic!()
        };
        assert!(
            run.results
                .iter()
                .zip(cases(s))
                .filter(|(_, c)| c.status != 0)
                .all(|(r, c)| c.accepts(r))
        );
        source.push_str("# edited after checking\n");
        assert!(!engine::solved(&good, &s.goal));
    }
}
#[test]
fn practice_assignment_favorites_and_other_attempts_survive_resume_and_selected_reset() {
    let mut catalog = baseline().clone();
    catalog.extend(authored().clone());
    let other = &catalog[0];
    let mut peer = Attempt::new(other, 1);
    workspace::prepare(&mut peer);
    let old = serde_json::to_value(&peer).unwrap();
    for s in authored() {
        let mut p = Progress {
            schema_version: 1,
            clock: 3,
            ..Default::default()
        };
        p.attempts.insert(other.id.clone(), peer.clone());
        p.favorites.extend([other.id.clone(), s.id.clone()]);
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        let lesson = s.teaching.as_ref().unwrap();
        script_teaching::prepare(&mut a, lesson, true);
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learning
            .example_source
            .push_str("# retained practice draft\n");
        learning.example_result = Some(shell::run(&learning.example_source, &lesson.fixture));
        assert!(
            lesson
                .fixture
                .accepts(learning.example_result.as_ref().unwrap())
        );
        assert_eq!(a.state, s.initial);
        assert!(!progress::current_solved(&a, s));
        a.state = state(s, &s.solution[0]);
        assert!(progress::current_solved(&a, s));
        let State::Script { source, .. } = &mut a.state else {
            panic!()
        };
        source.push_str("# retained assignment draft\n");
        assert!(!progress::current_solved(&a, s));
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, &catalog).unwrap();
        let mut loaded = progress::load(&path, &catalog).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
            old
        );
        loaded.clock = 4;
        let a = loaded.attempts.get_mut(&s.id).unwrap();
        *a = Attempt::new(s, 4);
        workspace::prepare(a);
        script_teaching::prepare(a, lesson, true);
        assert_eq!(a.state, s.initial);
        assert_eq!(
            a.workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_source,
            lesson.example
        );
        assert_eq!(
            serde_json::to_value(&loaded.attempts[&other.id]).unwrap(),
            old
        );
        assert!(loaded.favorites.contains(&s.id));
    }
}
#[test]
fn invalid_case_expectations_and_broken_teaching_examples_are_rejected() {
    for s in drafted() {
        let mut bad = s.clone();
        bad.teaching
            .as_mut()
            .unwrap()
            .fixture
            .stdout
            .push_str("incorrect");
        assert!(bad.teaching.as_ref().unwrap().validate().is_err());
        let mut cases = cases(&s).to_vec();
        cases[0].stdout.push_str("wrong expectation\n");
        let mut candidate = s.clone();
        candidate.goal = Goal::Script { cases };
        assert!(!engine::solved(
            &state(&candidate, &s.solution[0]),
            &candidate.goal
        ));
    }
}

#[test]
fn registered_extension_retains_every_prior_scenario_unit_and_progress_requirement() {
    let old = baseline();
    let current = pack::accepted399().unwrap();
    assert_eq!((old.len(), current.len()), (396, 399));
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(&current[..396]).unwrap()
    );
    let before = dlms_learning_lab::course::accepted396(old).unwrap();
    let units = dlms_learning_lab::course::accepted399(&current).unwrap();
    assert_eq!((before.len(), units.len()), (152, 153));
    let kept: Vec<_> = units
        .iter()
        .filter(|u| u.id != "shell-measurement-reports")
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(kept).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "shell-measurement-reports")
        .unwrap();
    assert_eq!(units[at - 1].id, "shell-log-correlation");
    for id in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == id).unwrap() < at);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| current.iter().filter(|s| s.track == t).count()),
        [93, 133, 93, 80]
    );
}
