//! Authoring and progress admission of the two source-fixture content-recovery projects before catalog registration.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use std::sync::OnceLock;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitIndex {
    version: u32,
    units: Vec<course::Unit>,
}
const IDS: [&str; 2] = ["shell-backup-content-set", "shell-restore-validated-set"];
fn catalog() -> &'static Vec<Scenario> {
    static C: OnceLock<Vec<Scenario>> = OnceLock::new();
    C.get_or_init(|| {
        let mut base = pack::accepted292().unwrap();
        assert_eq!(base.len(), 292);
        let extra: Pack = serde_json::from_str(include_str!(
            "fixtures/shell-content-backup-lessons-v1.json"
        ))
        .unwrap();
        assert_eq!(extra.scenarios.len(), 2);
        base.extend(extra.scenarios);
        // Use the real collection loader; one giant catalog is not a single 2MB pack.
        // Keep original prerequisite data and every pack/collection size guard.
        let fragments: Vec<String> = base
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&Pack {
                    schema_version: 1,
                    scenarios: rows.to_vec(),
                })
                .unwrap()
            })
            .collect();
        let refs: Vec<&str> = fragments.iter().map(String::as_str).collect();
        pack::load_catalog(&refs).unwrap()
    })
}
fn labs() -> impl Iterator<Item = &'static Scenario> {
    catalog().iter().filter(|s| IDS.contains(&s.id.as_str()))
}
fn cases(s: &Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn fresh(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
    a
}
fn solve(s: &Scenario) -> Attempt {
    let mut a = fresh(s);
    a.state = State::Script {
        source: s.solution[0].clone(),
        run: Some(Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: s.solution[0].clone(),
            results: cases(s)
                .iter()
                .map(|c| shell::run(&s.solution[0], c))
                .collect(),
        }),
    };
    assert!(progress::current_solved(&a, s));
    a
}
#[test]
fn content_recovery_drafts_are_real_reachable_teaching_with_prerequisites_and_unsolved_starters() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let base = pack::accepted292().unwrap();
    let before = course::accepted292(&base).unwrap();
    let mut after = before.clone();
    let units: UnitIndex =
        serde_json::from_str(include_str!("fixtures/shell-content-backup-units-v1.json")).unwrap();
    assert_eq!(units.version, 1);
    assert_eq!(units.units.len(), 1);
    assert_eq!(units.units[0].labs, IDS);
    let at = after
        .iter()
        .position(|u| u.id == "shell-managed-configuration")
        .unwrap()
        + 1;
    after.splice(at..at, units.units);
    let validated = course::load(
        &serde_json::to_string(&UnitIndex {
            version: 1,
            units: after,
        })
        .unwrap(),
        catalog(),
    )
    .unwrap();
    assert_eq!(validated.len(), 95);
    let retained: Vec<_> = validated
        .iter()
        .filter(|u| u.id != "shell-content-backups")
        .collect();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    for s in labs() {
        assert!(base.iter().all(|old| old.id != s.id));
        for id in &s.prerequisites {
            assert!(catalog().iter().any(|s| s.id == *id));
        }
        assert_eq!(cases(s).len(), 12);
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert!(!progress::current_solved(&fresh(s), s));
        solve(s);
        let State::Script { source, .. } = &s.initial else {
            panic!()
        };
        assert!(cases(s).iter().any(|c| !c.accepts(&shell::run(source, c))));
        assert!(
            lesson
                .concepts
                .iter()
                .any(|c| c.text.contains("Bash-specific"))
        );
        assert!(s.simplifications.contains("80 UTF-8"));
        assert!(s.simplifications.contains("crash"));
    }
}
#[test]
fn content_recovery_examples_and_three_guided_edits_have_real_observable_results() {
    for s in labs() {
        let lesson = s.teaching.as_ref().unwrap();
        let original = s.initial.clone();
        let correct = shell::run(&lesson.example, &lesson.fixture);
        assert!(lesson.fixture.accepts(&correct));
        let wrong = if s.id == IDS[0] {
            lesson.example.replace(
                "cat < \"$name\" > \"saved-$name\"",
                "content=$(cat < \"$name\")\n    printf '%s\\n' \"$content\" > \"saved-$name\"",
            )
        } else {
            lesson.example.replace(" || test -n \"$name\"", "")
        };
        assert_ne!(wrong, lesson.example);
        let out = shell::run(&wrong, &lesson.fixture);
        assert!(out.error.is_none(), "{out:?}");
        assert!(!lesson.fixture.accepts(&out));
        if s.id == IDS[1] {
            let trim = lesson
                .example
                .replace("while IFS= read -r", "while read -r");
            let out = shell::run(&trim, &lesson.fixture);
            assert!(out.error.is_none());
            assert!(!lesson.fixture.accepts(&out));
        }
        assert_eq!(
            serde_json::to_value(&s.initial).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        assert!(!progress::current_solved(&fresh(s), s));
    }
}
#[test]
fn content_recovery_drafts_results_favorites_and_selected_reset_roundtrip_safely() {
    let mut p = Progress {
        schema_version: 1,
        clock: 4,
        ..Default::default()
    };
    let old = &catalog()[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    p.favorites.insert(old.id.clone());
    let old_bytes = serde_json::to_value(&p.attempts[&old.id]).unwrap();
    for s in labs() {
        let mut a = solve(s);
        let learn = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learn
            .example_source
            .push_str("\n# separate unfinished example note\n");
        learn.example_result = None;
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let good = std::fs::read(&path).unwrap();
    let mut restored = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for s in labs() {
        assert!(progress::current_solved(&restored.attempts[&s.id], s));
    }
    let other = serde_json::to_value(&restored.attempts[IDS[1]]).unwrap();
    let selected = labs().next().unwrap();
    restored
        .attempts
        .insert(selected.id.clone(), fresh(selected));
    assert!(!progress::current_solved(
        &restored.attempts[&selected.id],
        selected
    ));
    assert_eq!(
        serde_json::to_value(&restored.attempts[IDS[1]]).unwrap(),
        other
    );
    assert_eq!(
        serde_json::to_value(&restored.attempts[&old.id]).unwrap(),
        old_bytes
    );
    assert_eq!(restored.favorites, p.favorites);
    let State::Script { source, .. } = &mut restored.attempts.get_mut(IDS[1]).unwrap().state else {
        panic!()
    };
    source.push_str("\nprintf 'broken\\n'\n");
    assert!(!progress::current_solved(
        &restored.attempts[IDS[1]],
        labs().find(|s| s.id == IDS[1]).unwrap()
    ));
    assert_eq!(std::fs::read(&path).unwrap(), good);
    // Reopening the valid disk file must still recover the actual saved successful work.
    assert!(progress::current_solved(
        &progress::load(&path, catalog()).unwrap().attempts[IDS[1]],
        labs().find(|s| s.id == IDS[1]).unwrap()
    ));
}

#[test]
fn registered_content_recovery_catalog_preserves_every292_source94_units_and_old_track_counts() {
    let old = pack::accepted292().unwrap();
    let all = pack::accepted294().unwrap();
    assert_eq!((old.len(), all.len()), (292, 294));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&all[..292]).unwrap()
    );
    let before = course::accepted292(&old).unwrap();
    let after = course::accepted294(&all).unwrap();
    assert_eq!((before.len(), after.len()), (94, 95));
    let retained: Vec<_> = after
        .iter()
        .filter(|u| u.id != "shell-content-backups")
        .collect();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = after
        .iter()
        .position(|u| u.id == "shell-content-backups")
        .unwrap();
    assert!(
        after
            .iter()
            .position(|u| u.id == "shell-managed-configuration")
            .unwrap()
            < at
    );
    assert_eq!(after[at].labs, IDS);
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [72, 101, 45, 76]
    );
    for s in labs() {
        let registered = all.iter().find(|other| s.id == other.id).unwrap();
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(registered).unwrap()
        );
    }
}
