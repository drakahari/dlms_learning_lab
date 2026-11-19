//! Named lookup and record reconciliation: actual VM grading and isolated drafts.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["shell-map-lookup", "shell-map-reconcile"];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::ccna_pilot().unwrap())
}
fn labs() -> impl Iterator<Item = &'static pack::Scenario> {
    catalog().iter().filter(|s| IDS.contains(&s.id.as_str()))
}
fn cases(s: &pack::Scenario) -> &[Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn record(s: &pack::Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn flags(s: &pack::Scenario, source: &str) -> Vec<bool> {
    cases(s)
        .iter()
        .zip(record(s, source).results)
        .map(|(c, r)| c.accepts(&r))
        .collect()
}
#[test]
fn primary_alternate_and_separate_worked_examples_satisfy_all_behavioral_cases() {
    let refs: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/shell-associative-lessons-reference.json"
    ))
    .unwrap();
    let mut count = 0;
    for s in labs() {
        let l = s.teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert_eq!(cases(s).len(), 12);
        let alt = refs["solutions"][&s.id]["alternate"].as_str().unwrap();
        assert_ne!(alt, s.solution[0]);
        for source in [s.solution[0].as_str(), alt] {
            for c in cases(s) {
                assert!(c.valid(), "{}", c.name);
                let r = shell::run(source, c);
                assert!(c.accepts(&r), "{} / {}: {r:?}", s.id, c.name);
                count += 1;
            }
        }
        let r = shell::run(&l.example, &l.fixture);
        assert!(l.fixture.accepts(&r), "{} practice: {r:?}", s.id);
        count += 1;
        let State::Script { source, .. } = &s.initial else {
            panic!()
        };
        assert!(
            flags(s, source).iter().any(|p| !*p),
            "{} starter must require work",
            s.id
        );
    }
    assert_eq!(count, 50, "24primary+24alternate+2separate examples");
}
#[test]
fn hardcoded_first_case_and_file_damage_cannot_earn_whole_assignment_credit() {
    fn quote(s: &str) -> String {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
    for s in labs() {
        let c = &cases(s)[0];
        let source = format!(
            "printf '%s' {}\nprintf '%s' {} >&2\nexit {}\n",
            quote(&c.stdout),
            quote(&c.stderr),
            c.status
        );
        let result = flags(s, &source);
        assert!(result[0]);
        assert!(result.iter().skip(1).any(|p| !*p));
        let file = cases(s)[0].files.keys().next().unwrap();
        let damage = format!("{}\nprintf changed > {}\n", s.solution[0], quote(file));
        assert!(flags(s, &damage).iter().any(|p| !*p));
    }
}
#[test]
fn plausible_mistakes_produce_partial_results_and_specific_diagnostics() {
    for s in labs() {
        let correct = &s.solution[0];
        let broken = if s.id == IDS[0] {
            correct.replace(
                "printf '%s: %s\\n' \"$request\" \"$found\"",
                "printf '%s: %s\\n' \"$request\" $found",
            )
        } else {
            correct.replace(" || [ -n \"$line\" ]", "")
        };
        assert_ne!(&broken, correct);
        let result = flags(s, &broken);
        assert!(
            result.iter().any(|p| *p),
            "{} should retain correct cases",
            s.id
        );
        assert!(
            result.iter().any(|p| !*p),
            "{} must diagnose changed behavior",
            s.id
        );
        let wrong = shell::run("printf wrong", &cases(s)[0]);
        assert!(wrong.error.is_none());
        assert!(!cases(s)[0].accepts(&wrong));
        let syntax = shell::run("value='unfinished", &cases(s)[0]);
        assert!(
            syntax
                .error
                .as_ref()
                .is_some_and(|m| m.contains("Syntax error")),
            "{syntax:?}"
        );
        let unsupported = shell::run("declare -a unsupported", &cases(s)[0]);
        assert!(
            unsupported
                .error
                .as_ref()
                .is_some_and(|m| m.contains("Unsupported")),
            "{unsupported:?}"
        );
    }
    let s = labs().find(|s| s.id == IDS[0]).unwrap();
    let c = cases(s)
        .iter()
        .find(|c| c.name.contains("Guard blank"))
        .unwrap();
    let r = shell::run(&s.solution[0].replace("[ -z \"$request\" ]", "false"), c);
    assert!(
        r.error
            .as_ref()
            .is_some_and(|m| m.contains("cannot be empty")),
        "{r:?}"
    );
}
#[test]
fn malformed_rows_empty_values_and_unique_counts_are_actually_assessed() {
    let s = labs().find(|s| s.id == IDS[1]).unwrap();
    let correct = &s.solution[0];
    let bads = [
        correct.replace("*'|'*'|'*", "never-two-bars"),
        correct.replace(
            "latest[\"$key\"]=\"$value\"",
            "if [ -n \"$value\" ]; then latest[\"$key\"]=\"$value\"; fi",
        ),
        correct.replace("latest[\"$key\"]=\"$value\"", "latest[\"$key\"]=constant"),
        correct.replace("\"${#latest[@]}\"", "0"),
    ];
    let empty = cases(s)
        .iter()
        .find(|c| c.name == "An empty value is still a stored entry")
        .unwrap();
    let empty_skipped = shell::run(&bads[1], empty);
    assert!(empty_skipped.error.is_none(), "{empty_skipped:?}");
    assert!(!empty.accepts(&empty_skipped));
    for bad in bads {
        assert_ne!(&bad, correct);
        assert!(
            flags(s, &bad).iter().any(|p| !*p),
            "A wrong policy must fail: {bad}"
        );
    }
}
#[test]
fn guided_changes_are_runnable_and_distinguish_output_mistakes_from_empty_key_errors() {
    for s in labs() {
        let l = s.teaching.as_ref().unwrap();
        let changed = if s.id == IDS[0] {
            l.example.replace("\"$DESK_VALUE_A\"", "temporary")
        } else {
            l.example.replace(" || [ -n \"$line\" ]", "")
        };
        assert_ne!(changed, l.example);
        let r = shell::run(&changed, &l.fixture);
        assert!(r.error.is_none(), "{}: {r:?}", s.id);
        assert!(!l.fixture.accepts(&r));
    }
    let s = labs().find(|s| s.id == IDS[0]).unwrap();
    let l = s.teaching.as_ref().unwrap();
    let r = shell::run(
        &l.example.replace("[ -z \"$request\" ]", "false"),
        &l.fixture,
    );
    assert!(
        r.error
            .as_ref()
            .is_some_and(|m| m.contains("cannot be empty")),
        "{r:?}"
    );
    let s = labs().find(|s| s.id == IDS[1]).unwrap();
    let l = s.teaching.as_ref().unwrap();
    let r = shell::run(
        &l.example
            .replace("catalog[\"$key\"]=\"$value\"", "catalog[\"$key\"]=constant"),
        &l.fixture,
    );
    assert!(r.error.is_none());
    assert!(r.stdout.contains("Entries: 2"));
    assert!(!l.fixture.accepts(&r));
}
#[test]
fn all_277_prior_objects_and_83_units_are_exact_and_prerequisites_precede_new_unit() {
    let old = pack::accepted277().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (277, 290));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..277]).unwrap()
    );
    let before = course::accepted277(&old).unwrap();
    let units = course::ccna_pilot(now).unwrap();
    assert_eq!((before.len(), units.len()), (83, 93));
    let filtered: Vec<_> = units
        .iter()
        .filter(|u| {
            !["shell-associative-maps", "linux-shared-library-resolution", "linux-file-locations", "shell-managed-configuration", "linux-listener-audit", "linux-command-search", "linux-account-defaults", "linux-command-location-diagnosis", "linux-indexed-file-discovery", "linux-application-support-discovery"].contains(&u.id.as_str())
        })
        .collect();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(filtered).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "shell-associative-maps")
        .unwrap();
    assert_eq!(units[at].labs, IDS);
    for pre in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == pre).unwrap() < at);
    }
}
#[test]
fn examples_drafts_credit_edit_invalidation_save_resume_and_selected_reset_are_isolated() {
    let mut p = Progress {
        schema_version: 1,
        clock: 4,
        ..Default::default()
    };
    for s in labs() {
        let mut a = Attempt::new(s, 4);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        p.attempts.insert(s.id.clone(), a);
    }
    p.favorites.insert(IDS[1].into());
    let untouched = serde_json::to_value(&p.attempts[IDS[1]]).unwrap();
    let s = labs().find(|s| s.id == IDS[0]).unwrap();
    let l = s.teaching.as_ref().unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let initial = a.state.clone();
    let learn = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    learn.example_source.push_str("# separate practice draft\n");
    learn.example_result = Some(shell::run(&learn.example_source, &l.fixture));
    assert_eq!(a.state, initial);
    assert!(!progress::current_solved(a, s));
    a.state = State::Script {
        source: s.solution[0].clone(),
        run: Some(record(s, &s.solution[0])),
    };
    assert!(progress::current_solved(a, s));
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("printf wrong\n");
    assert!(!progress::current_solved(a, s));
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&q.attempts[IDS[1]]).unwrap(),
        untouched
    );
    let a = q.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, s));
    *a = Attempt::new(s, a.visited);
    workspace::prepare(a);
    script_teaching::prepare(a, l, true);
    assert_eq!(a.state, s.initial);
    assert!(!progress::current_solved(a, s));
    assert_eq!(
        serde_json::to_value(&q.attempts[IDS[1]]).unwrap(),
        untouched
    );
    assert!(q.favorites.contains(IDS[1]));
}
