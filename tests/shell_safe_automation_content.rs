//! Behavior grading for three distinct reliable-text tasks; no host shell.
use dlms_learning_lab::{
    backup,
    engine::{Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt},
    script_teaching::{self as teaching, View},
    shell::{self, Record},
    workspace,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
fn labs() -> Vec<Scenario> {
    static ALL: std::sync::OnceLock<Vec<Scenario>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let text = include_str!("../scenarios/shell-safe-automation-v1.json");
        let proposed: pack::Pack = serde_json::from_str(text).unwrap();
        let ids: std::collections::BTreeSet<_> =
            proposed.scenarios.iter().map(|s| s.id.clone()).collect();
        // The new unit refers to real prior lessons. Validate it with the full
        // current context, retaining the existing per-document loading ceiling.
        let prior: Vec<_> = pack::bundled()
            .unwrap()
            .into_iter()
            .filter(|s| !ids.contains(&s.id))
            .collect();
        let mut chunks: Vec<String> = prior
            .chunks(8)
            .map(|group| {
                serde_json::to_string(&pack::Pack {
                    schema_version: 1,
                    scenarios: group.to_vec(),
                })
                .unwrap()
            })
            .collect();
        chunks.push(text.into());
        let refs: Vec<_> = chunks.iter().map(String::as_str).collect();
        pack::load_catalog(&refs)
            .unwrap()
            .into_iter()
            .filter(|s| ids.contains(&s.id))
            .collect()
    })
    .clone()
}
fn cases(s: &Scenario) -> &[shell::Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn reference(s: &Scenario) -> &str {
    &s.solution[0]
}
fn run_all(s: &Scenario, source: &str) -> Record {
    Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.into(),
        results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
    }
}
fn accepts(s: &Scenario, source: &str) -> Vec<bool> {
    let record = run_all(s, source);
    cases(s)
        .iter()
        .zip(&record.results)
        .map(|(c, r)| c.accepts(r))
        .collect()
}
fn full(p: &progress::Progress) -> serde_json::Value {
    let mut v = serde_json::to_value(p).unwrap();
    v["migration_receipts"] = serde_json::to_value(&p.migration_receipts).unwrap();
    v
}
#[test]
fn all32_assignment_cases_three_examples_and_real_prerequisites_are_reachable() {
    let proposed = labs();
    let current = pack::bundled().unwrap();
    assert_eq!(proposed.len(), 3);
    assert_eq!(
        proposed.iter().map(|s| cases(s).len()).collect::<Vec<_>>(),
        vec![10, 10, 12]
    );
    for s in &proposed {
        for id in &s.prerequisites {
            assert!(
                current.iter().chain(&proposed).any(|p| &p.id == id),
                "{} missing {id}",
                s.id
            );
        }
        assert_eq!(s.track, "Bash/Shell");
        assert_eq!(s.unit, "shell-safe-automation");
        let record = run_all(s, reference(s));
        for (c, r) in cases(s).iter().zip(&record.results) {
            assert!(
                c.accepts(r),
                "{}/{}: {:?}; expected {:?} actual {:?}",
                s.id,
                c.name,
                r.error,
                c.stdout,
                r.stdout
            );
        }
        let lesson = s.teaching.as_ref().unwrap();
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&lesson.example, &lesson.fixture))
        );
        assert_eq!(lesson.concepts.len(), 3);
        assert!(lesson.inputs.contains("arguments"));
        assert!(s.guide[0].why.contains("reset simulated files"));
    }
    // These guarantees are supplied data, not untested validation demands.
    for c in cases(&proposed[1]) {
        if c.args.len() == 1 {
            assert!(!["errors.txt", "keep.txt"].contains(&c.args[0].as_str()));
        }
    }
}
#[test]
fn genuinely_different_supported_methods_pass_without_exact_source_matching() {
    let all = labs();
    let alternative_lines = reference(&all[0])
        .replace("IFS= read", "IFS='' read")
        .replace("number", "position");
    assert!(accepts(&all[0], &alternative_lines).into_iter().all(|r| r));
    let separate_producer = r#"if test "$#" -ne 1; then
 printf 'Usage: errors SOURCE\n' >&2
 exit 2
fi
matches=$(grep -F ERROR "$1")
status=$?
if test "$status" -gt 1; then
 printf 'Report unavailable\n' >&2
 exit 2
fi
if test -n "$matches"; then
 printf '%s\n' "$matches" | sort | uniq > errors.txt
else
 printf '' > errors.txt
fi
printf 'Report updated\n'
"#;
    assert!(
        accepts(&all[1], separate_producer).into_iter().all(|r| r),
        "check the producer separately instead of pipefail"
    );
    let validation = reference(&all[2]).split("report=$(").next().unwrap();
    let incremental = format!(
        "{validation}printf '' > \"$destination\"\ncount=0\nwhile IFS= read -r line || test -n \"$line\"; do\n printf 'Entry: <%s>\\n' \"$line\" >> \"$destination\"\n count=$((count + 1))\ndone < \"$source\"\nprintf 'Total: %s\\n' \"$count\" >> \"$destination\"\nprintf 'Published report\\n'\n"
    );
    assert!(
        accepts(&all[2], &incremental).into_iter().all(|r| r),
        "validated incremental publication is also within the stated task"
    );
}
#[test]
fn plausible_wrong_loops_status_count_and_whitespace_shortcuts_fail() {
    let all = labs();
    for source in [
        reference(&all[0]).replace("IFS= read", "read"),
        reference(&all[0]).replace(" || test -n \"$line\"", ""),
        reference(&all[0]).replace("-ne 1", "-lt 1"),
    ] {
        let rows = accepts(&all[0], &source);
        assert!(
            rows.iter().any(|r| *r) && rows.iter().any(|r| !*r),
            "useful partial results, never complete"
        );
    }
    for source in [
        reference(&all[1]).replace("set -o pipefail", "set +o pipefail"),
        reference(&all[1]).replace("-ne 1", "-lt 1"),
        reference(&all[1]).replace("-gt 1", "-gt 0"),
    ] {
        assert!(accepts(&all[1], &source).iter().any(|r| !*r));
    }
    for source in [
        reference(&all[2]).replace("-ne 2", "-lt 2"),
        reference(&all[2]).replace("IFS= read", "read"),
        reference(&all[2]).replace("|keep.txt", ""),
        reference(&all[2]).replace(" || test -n \"$line\"", ""),
    ] {
        assert!(accepts(&all[2], &source).iter().any(|r| !*r));
    }
}
#[test]
fn hardcoded_first_cases_partial_credit_and_success_invalidation_are_honest() {
    let all = labs();
    let hardcoded = [
        "printf '1: [Router]\\n2: [Switch]\\n'",
        "printf 'ERROR power\\nERROR rack\\n' > errors.txt; printf 'Report updated\\n'",
        "printf 'Entry: <Door>\\nEntry: <Lift>\\nTotal: 2\\n' > report.txt; printf 'Published report\\n'",
    ];
    for (s, hardcoded) in all.iter().zip(hardcoded) {
        let rows = accepts(s, hardcoded);
        assert!(rows[0], "counterexample passes first sample");
        assert!(
            rows.iter().any(|r| !*r),
            "numeric/text variants are test cases, not duplicate lab credit"
        );
        let mut a = Attempt::new(s, 1);
        let source = reference(s).to_owned();
        a.state = State::Script {
            run: Some(run_all(s, &source)),
            source,
        };
        assert!(progress::current_solved(&a, s));
        let State::Script { source, .. } = &mut a.state else {
            unreachable!()
        };
        source.push_str("\nprintf wrong\n");
        assert!(
            !progress::current_solved(&a, s),
            "edits clear current success without deleting historical work"
        );
    }
}
#[test]
fn syntax_unsupported_wrong_output_and_correct_results_have_distinct_feedback() {
    let s = labs().remove(0);
    let c = &cases(&s)[0];
    let syntax = shell::run("printf '", c);
    let unsupported = shell::run("set -e", c);
    let wrong = shell::run("printf wrong", c);
    let correct = shell::run(reference(&s), c);
    assert_eq!(
        teaching::diagnostic(syntax.error.as_deref().unwrap()),
        teaching::Diagnostic::Syntax
    );
    assert_eq!(
        teaching::diagnostic(unsupported.error.as_deref().unwrap()),
        teaching::Diagnostic::Unsupported
    );
    assert!(wrong.error.is_none() && !c.accepts(&wrong));
    assert!(c.accepts(&correct));
}
#[test]
fn practice_assignment_drafts_and_real_backup_resume_reset_preserve_old_progress() {
    let proposed = labs();
    let mut all = pack::bundled().unwrap();
    all.extend(proposed.iter().cloned());
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v8-synthetic-writer-progress.json"),
        &all,
    )
    .unwrap();
    let originals = full(&p);
    for s in &proposed {
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let lesson = s.teaching.as_ref().unwrap();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learning.view = View::Example;
        learning.example_source = lesson.example.clone();
        learning.example_result = Some(shell::run(&learning.example_source, &lesson.fixture));
        assert!(
            !progress::current_solved(&a, s),
            "practice never earns assignment credit"
        );
        let source = reference(s).to_owned();
        a.state = State::Script {
            run: Some(run_all(s, &source)),
            source,
        };
        assert!(progress::current_solved(&a, s));
        let State::Script { source, .. } = &mut a.state else {
            unreachable!()
        };
        source.push_str("\n# unfinished assignment\nprintf '");
        a.workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap()
            .example_source
            .push_str("\n# separate unfinished practice\nprintf '");
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let before = full(&p);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.json");
    progress::save(&path, &p).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert!(full(&loaded)["attempts"] == before["attempts"]);
    for (id, value) in originals["attempts"].as_object().unwrap() {
        assert!(
            full(&loaded)["attempts"][id] == *value,
            "all preexisting attempt fields survive"
        );
    }
    let export = dir.path().join("backup.json");
    backup::export(&export, &loaded, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    backup::restore(&path, &candidate, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    for s in &proposed {
        let a = &restored.attempts[&s.id];
        assert!(!progress::current_solved(a, s));
        let State::Script { source, .. } = &a.state else {
            unreachable!()
        };
        assert!(source.contains("unfinished assignment"));
        assert!(
            a.workspace
                .as_ref()
                .unwrap()
                .script_learning
                .as_ref()
                .unwrap()
                .example_source
                .contains("separate unfinished practice")
        );
    }
    let other = serde_json::to_value(&restored.attempts[&proposed[1].id]).unwrap();
    restored
        .attempts
        .insert(proposed[0].id.clone(), Attempt::new(&proposed[0], 2));
    assert!(serde_json::to_value(&restored.attempts[&proposed[1].id]).unwrap() == other);
    assert_eq!(restored.favorites, loaded.favorites);
}
