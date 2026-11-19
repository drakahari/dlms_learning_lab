//! Fixed authoring recipes; not registered lessons. Entirely bounded in-process execution.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    shell::{self, Case, Record, Vm},
};
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Project {
    source: String,
    alternate: String,
    cases: Vec<Case>,
    example_source: String,
    example_case: Case,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    projects: BTreeMap<String, Project>,
}
fn fixture() -> Fixture {
    let f: Fixture = serde_json::from_str(include_str!(
        "fixtures/shell-content-backup-recipes-v1.json"
    ))
    .unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.projects.len(), 2);
    for p in f.projects.values() {
        assert_eq!(p.cases.len(), 12);
        assert!(p.cases.iter().all(Case::valid));
        assert!(p.example_case.valid());
        assert_ne!(p.source, p.alternate);
    }
    f
}
#[test]
fn content_backup_canonical_alternate_and_separate_examples_match_all_behavior_cases() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    for (name, p) in fixture().projects {
        for source in [&p.source, &p.alternate] {
            let extra: BTreeMap<String, Vec<Case>> = serde_json::from_str(include_str!(
                "fixtures/shell-content-backup-boundaries-v1.json"
            ))
            .unwrap();
            for c in p.cases.iter().chain(&extra[&name]) {
                assert!(c.valid());
                let out = shell::run(source, c);
                assert!(
                    c.accepts(&out),
                    "{name}/{}: actual{out:?} expected{c:?}",
                    c.name
                );
            }
        }
        assert_ne!(p.example_source, p.source);
        assert_ne!(p.example_case.files, p.cases[0].files);
        let out = shell::run(&p.example_source, &p.example_case);
        assert!(
            p.example_case.accepts(&out),
            "{name}/practice actual{out:?}"
        );
    }
}
#[test]
fn content_backup_restore_consumes_an_actual_created_set_without_consuming_recovery_data() {
    let f = fixture();
    let save = &f.projects["snapshot"];
    let restore = &f.projects["restore"];
    for source in [&save.source, &save.alternate] {
        let first = shell::run(source, &save.cases[0]);
        assert!(save.cases[0].accepts(&first));
        for restore_source in [&restore.source, &restore.alternate] {
            let mut c = save.cases[0].clone();
            c.name = "Actual saved output restored after damage".into();
            c.args.clear();
            c.files = first.files.clone();
            c.files
                .insert("/work/alpha.txt".into(), "later damage\n".into());
            c.files.remove("/work/beta.txt");
            c.after = first.files.clone();
            c.stdout = "Restored: 2 files\n".into();
            assert!(c.valid());
            let out = shell::run(restore_source, &c);
            assert!(c.accepts(&out), "roundtrip {out:?}");
        }
    }
}
fn flags(p: &Project, source: &str) -> Vec<bool> {
    p.cases
        .iter()
        .map(|c| c.accepts(&shell::run(source, c)))
        .collect()
}
#[test]
fn content_backup_plausible_lossy_read_and_premature_write_methods_fail_without_being_unsupported()
{
    let f = fixture();
    let p = &f.projects["snapshot"];
    let lossy = p.source.replace(
        "cat < \"$name\" > \"copy-$name\"",
        "content=$(cat < \"$name\")\n    printf '%s\\n' \"$content\" > \"copy-$name\"",
    );
    assert_ne!(lossy, p.source);
    let block = "for name in \"${names[@]}\"; do\n    cat < \"$name\" > \"copy-$name\"\ndone\n";
    assert!(p.source.contains(block));
    let early = p.source.replace(block, "").replace(
        "for name in \"${names[@]}\"; do\n    if test -f \"$name\"; then",
        &format!("{block}for name in \"${{names[@]}}\"; do\n    if test -f \"$name\"; then"),
    );
    assert_ne!(early, p.source);
    for source in [lossy, early] {
        let outcomes: Vec<_> = p.cases.iter().map(|c| shell::run(&source, c)).collect();
        assert!(
            outcomes.iter().all(|r| r.error.is_none()),
            "Wrong supported method must run: {outcomes:?}"
        );
        let passed = flags(p, &source);
        assert!(
            passed.iter().any(|b| *b) && passed.iter().any(|b| !*b),
            "Must remain partial: {passed:?}"
        );
    }
    let p = &f.projects["restore"];
    for source in [
        p.source.replace(" || test -n \"$name\"", ""),
        p.source
            .replace("while IFS= read -r name", "while read -r name"),
        p.source.replace(
            "    names=(\"${names[@]}\" \"$name\")",
            "    cat < \"copy-$name\" > \"$name\"\n    names=(\"${names[@]}\" \"$name\")",
        ),
    ] {
        assert_ne!(source, p.source);
        let results: Vec<_> = p.cases.iter().map(|c| shell::run(&source, c)).collect();
        assert!(results.iter().all(|r| r.error.is_none()), "{results:?}");
        let passed = flags(p, &source);
        assert!(
            passed.iter().any(|b| *b) && passed.iter().any(|b| !*b),
            "Must remain partial: {passed:?}"
        );
    }
}
#[test]
fn content_backup_full_first_result_hardcoding_cannot_complete_either_assignment() {
    let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
    for (name, p) in fixture().projects {
        let c = &p.cases[0];
        let mut source = String::new();
        for (path, value) in &c.after {
            if c.files.get(path) != Some(value) {
                source.push_str(&format!("printf '%s' {} > {}\n", quote(value), quote(path)));
            }
        }
        source.push_str(&format!("printf '%s' {}\n", quote(&c.stdout)));
        let checks = flags(&p, &source);
        assert!(
            checks[0],
            "First-result reproduction must really pass: {name}"
        );
        assert!(checks.iter().any(|b| !*b));
    }
}
#[test]
fn content_backup_current_success_requires_current_source_and_stop_or_escape_errors_do_not_pass() {
    for (_, p) in fixture().projects {
        let goal = Goal::Script {
            cases: p.cases.clone(),
        };
        let mut state = State::Script {
            source: p.source.clone(),
            run: Some(Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: p.source.clone(),
                results: p.cases.iter().map(|c| shell::run(&p.source, c)).collect(),
            }),
        };
        assert!(engine::solved(&state, &goal));
        let State::Script { source, .. } = &mut state else {
            panic!()
        };
        source.push_str("\nprintf 'extra output\\n'\n");
        assert!(!engine::solved(&state, &goal));
        for source in [
            "if true; then\n",
            "tar -cf backup.tar file\n",
            "cat /etc/passwd\n",
            "while true; do :; done\n",
        ] {
            let out = shell::run(source, &p.cases[0]);
            assert!(out.error.is_some());
            assert!(!p.cases[0].accepts(&out));
        }
        let mut vm = Vm::new("while true; do :; done\n", &p.cases[0]).unwrap();
        vm.tick(16);
        assert!(!vm.done());
        vm.stop();
        assert!(vm.done());
    }
}

#[test]
fn content_backup_real_roster_rejects_unprotected_recovery_namespace_and_off_by_one_limits() {
    let f = fixture();
    let restore = &f.projects["restore"];
    let unsafe_target = restore.source.replace("|copy-*", "");
    assert_ne!(unsafe_target, restore.source);
    let original = flags(restore, &restore.source);
    assert!(original.iter().all(|b| *b));
    let wrong = flags(restore, &unsafe_target);
    assert!(wrong.iter().any(|b| *b) && wrong.iter().any(|b| !*b));
    assert!(
        !wrong[2],
        "Reserved backup names must be part of actual grading"
    );
    for p in f.projects.values() {
        let source = p.source.replace("-gt 8", "-gt 7");
        assert_ne!(source, p.source);
        let wrong = flags(p, &source);
        assert!(
            wrong.iter().any(|b| *b) && wrong.iter().any(|b| !*b),
            "An eight-file task must include a successful eight-file case"
        );
    }
}
