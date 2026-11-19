//! Authored file-lifecycle projects, bounded VM only.
use dlms_learning_lab::{
    course,
    engine::{Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching,
    shell::{self, Case, Record},
    workspace,
};
use std::{collections::BTreeMap, sync::OnceLock};
const IDS: [&str; 3] = [
    "shell-stage-report",
    "shell-cleanup-preview",
    "shell-recover-publication",
];
fn catalog() -> &'static Vec<pack::Scenario> {
    static C: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    C.get_or_init(|| pack::accepted272().unwrap())
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
fn alternate(s: &pack::Scenario) -> String {
    let x = &s.solution[0];
    match s.id.as_str(){
 "shell-stage-report"=>{let at=x.find("staged=$(mktemp").unwrap();format!("{}cat < \"$source\" > \"$target\"\nprintf 'Published: %s\\n' \"$target\"\n",&x[..at])},
 "shell-cleanup-preview"=>x.replace("if test \"$mode\" = preview; then printf 'Would remove: %s\\n' \"$file\"; else rm -- \"$file\"; printf 'Removed: %s\\n' \"$file\"; fi","case \"$mode\" in preview) printf 'Would remove: %s\\n' \"$file\" ;; apply) rm -- \"$file\"; printf 'Removed: %s\\n' \"$file\" ;; esac"),
 _=>x.replace("staged=$(mktemp recovery.XXXXXX)\n cat < \"$backup\" > \"$staged\"\n if test \"$staged\" = \"$current\"; then :; else mv -- \"$staged\" \"$current\"; fi","cat < \"$backup\" > \"$current\"").replace("saved=$(mktemp snapshot.XXXXXX)\n cat < \"$current\" > \"$saved\"\n if test \"$saved\" = \"$backup\"; then :; else mv -- \"$saved\" \"$backup\"; fi\n staged=$(mktemp rollout.XXXXXX)\n cat < \"$candidate\" > \"$staged\"\n if test \"$staged\" = \"$current\"; then :; else mv -- \"$staged\" \"$current\"; fi","cat < \"$current\" > \"$backup\"\n cat < \"$candidate\" > \"$current\"")
 }
}
#[test]
fn references_distinct_supported_alternatives_examples_and_extra_boundaries_match() {
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    let extra: BTreeMap<String, Vec<Case>> = serde_json::from_str(include_str!(
        "fixtures/shell-file-lifecycle-lesson-boundaries.json"
    ))
    .unwrap();
    let mut graded = 0;
    let mut boundaries = 0;
    for s in labs() {
        let lesson = s.teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        let alt = alternate(s);
        assert_ne!(&alt, &s.solution[0], "{} alternate must differ", s.id);
        assert!(
            !flags(
                s,
                match &s.initial {
                    State::Script { source, .. } => source,
                    _ => panic!(),
                }
            )
            .iter()
            .all(|b| *b)
        );
        for source in [&s.solution[0], &alt] {
            for c in cases(s).iter().chain(&extra[&s.id]) {
                assert!(c.valid(), "{}", c.name);
                let out = shell::run(source, c);
                assert!(
                    c.accepts(&out),
                    "{} / {} actual{out:?} expected{c:?}",
                    s.id,
                    c.name
                );
            }
        }
        assert!(
            lesson
                .fixture
                .accepts(&shell::run(&lesson.example, &lesson.fixture))
        );
        assert!(!s.solution[0].contains("trap "));
        graded += cases(s).len();
        boundaries += extra[&s.id].len();
    }
    assert_eq!(graded, 35);
    assert_eq!(boundaries, 12);
}
#[test]
fn first_case_hardcodes_output_only_and_lost_protected_files_cannot_pass() {
    for s in labs() {
        let c = &cases(s)[0];
        let out_only = format!("printf '%s' '{}'\n", c.stdout.replace('\'', "'\\''"));
        let hardcoded = flags(s, &out_only);
        assert!(hardcoded.iter().any(|b| !*b), "{}", s.id);
        // Reproduce the entire first result, including its file mutations.
        // Passing that one case must not earn the whole assignment.
        let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
        let mut fixed = String::new();
        for path in c.files.keys().filter(|path| !c.after.contains_key(*path)) {
            fixed.push_str(&format!("rm -- {}\n", quote(path)));
        }
        for (path, text) in &c.after {
            if c.files.get(path) != Some(text) {
                fixed.push_str(&format!("printf '%s' {} > {}\n", quote(text), quote(path)));
            }
        }
        fixed.push_str(&format!(
            "printf '%s' {}\nprintf '%s' {} >&2\nexit {}\n",
            quote(&c.stdout),
            quote(&c.stderr),
            c.status
        ));
        let fixed_flags = flags(s, &fixed);
        assert!(
            fixed_flags[0],
            "{} genuine first-case hardcode must actually pass case1: {:?}",
            s.id,
            shell::run(&fixed, c)
        );
        assert!(
            fixed_flags.iter().skip(1).any(|p| !*p),
            "{} hardcoded first result cannot pass every case",
            s.id
        );
        let mut source = s.solution[0].clone();
        source.push_str("\nrm -- notes.txt\n");
        assert!(!flags(s, &source).iter().all(|b| *b), "{}", s.id);
        let State::Script { source, .. } = &s.initial else {
            panic!()
        };
        let starter = flags(s, source);
        assert!(starter.iter().any(|b| !*b));
        if s.id != "shell-recover-publication" {
            assert!(
                starter.iter().any(|b| *b),
                "Useful partial starter {}",
                s.id
            );
        }
    }
}
#[test]
fn plausible_unsafe_edits_fail_specific_state_checks() {
    for s in labs() {
        let correct = &s.solution[0];
        let broken = match s.id.as_str() {
            "shell-stage-report" => correct.replace(
                "cat < \"$source\" > \"$staged\"",
                "printf 'wrong' > \"$staged\"",
            ),
            "shell-cleanup-preview" => correct.replace(
                "printf 'Would remove: %s\\n' \"$file\"",
                "rm -- \"$file\"; printf 'Would remove: %s\\n' \"$file\"",
            ),
            _ => correct.replace(
                "cat < \"$current\" > \"$saved\"",
                "cat < \"$candidate\" > \"$saved\"",
            ),
        };
        assert_ne!(&broken, correct);
        let result = flags(s, &broken);
        assert!(result.iter().any(|b| !*b));
        assert!(
            result.iter().any(|b| *b),
            "Partial case results retained {}",
            s.id
        );
    }
    let s = labs().find(|s| s.id == "shell-cleanup-preview").unwrap();
    let late = cases(s)
        .iter()
        .find(|c| c.name == "Late protected request")
        .unwrap();
    let bad = "shift; for file in \"$@\"; do rm -- \"$file\"; done";
    let out = shell::run(bad, late);
    assert!(!late.accepts(&out));
    assert_ne!(out.files, late.files);
}
#[test]
fn syntax_limitations_and_output_failures_are_distinct() {
    for s in labs() {
        let c = &cases(s)[0];
        let syntax = shell::run("value='unfinished", c);
        assert!(syntax.error.is_some());
        let unsupported = shell::run("rm -rf .", c);
        assert!(
            unsupported
                .error
                .as_ref()
                .is_some_and(|m| m.contains("Unsupported"))
        );
        let wrong = shell::run("printf wrong", c);
        assert!(wrong.error.is_none());
        assert!(!c.accepts(&wrong));
    }
}
#[test]
fn practice_drafts_save_resume_invalidated_success_and_selected_reset_are_separate() {
    let mut p = Progress {
        schema_version: 1,
        clock: 9,
        ..Default::default()
    };
    for (n, s) in labs().enumerate() {
        let mut a = Attempt::new(s, n as u64 + 1);
        workspace::prepare(&mut a);
        script_teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        p.attempts.insert(s.id.clone(), a);
    }
    p.favorites.insert(IDS[2].into());
    let untouched = serde_json::to_value(&p.attempts[IDS[2]]).unwrap();
    let s = labs().next().unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    let original = a.state.clone();
    let lesson = s.teaching.as_ref().unwrap();
    let l = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    l.example_source.push_str("# Separate workshop draft\n");
    l.example_result = Some(shell::run(&l.example_source, &lesson.fixture));
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(a, s));
    let example = labs()
        .next()
        .unwrap()
        .teaching
        .as_ref()
        .unwrap()
        .example
        .clone();
    assert_ne!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source,
        example
    );
    a.state = State::Script {
        source: s.solution[0].clone(),
        run: Some(record(s, &s.solution[0])),
    };
    assert!(progress::current_solved(a, s));
    let State::Script { source, .. } = &mut a.state else {
        panic!()
    };
    source.push_str("\nprintf wrong\n");
    assert!(!progress::current_solved(a, s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, catalog()).unwrap();
    let mut q = progress::load(&path, catalog()).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&q.attempts[IDS[2]]).unwrap(),
        untouched
    );
    let a = q.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, s));
    let visited = a.visited;
    *a = Attempt::new(s, visited);
    workspace::prepare(a);
    script_teaching::prepare(a, lesson, true);
    assert!(!progress::current_solved(a, s));
    assert_eq!(a.state, s.initial);
    assert_eq!(
        serde_json::to_value(&q.attempts[IDS[2]]).unwrap(),
        untouched
    );
    assert!(q.favorites.contains(IDS[2]));
}
#[test]
fn all_previous269_objects_and80_units_are_preserved_and_new_prerequisites_are_earlier() {
    let old = pack::accepted269().unwrap();
    let now = catalog();
    assert_eq!((old.len(), now.len()), (269, 272));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&now[..269]).unwrap()
    );
    let old_units = course::accepted269(&old).unwrap();
    let units = course::accepted272(now).unwrap();
    assert_eq!((old_units.len(), units.len()), (80, 81));
    let filtered: Vec<_> = units
        .iter()
        .filter(|u| u.id != "shell-file-lifecycle")
        .collect();
    assert_eq!(
        serde_json::to_value(old_units).unwrap(),
        serde_json::to_value(filtered).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "shell-file-lifecycle")
        .unwrap();
    for pre in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == pre).unwrap() < at);
    }
    assert_eq!(units[at].labs, IDS);
}

#[test]
fn promised_eof_empty_backup_and_validation_failures_are_in_the_actual_grading_cases() {
    let recover = labs()
        .find(|s| s.id == "shell-recover-publication")
        .unwrap();
    let correct = &recover.solution[0];
    let read_line = "IFS= read -r header < \"$candidate\"";
    let reject_eof=correct.replace(read_line,"if IFS= read -r header < \"$candidate\"; then :; else printf 'Candidate not approved\\n' >&2; exit 2; fi");
    assert_ne!(&reject_eof, correct);
    let eof = cases(recover)
        .iter()
        .find(|c| c.name.contains("header without"))
        .unwrap();
    let actual = shell::run(&reject_eof, eof);
    assert!(actual.error.is_none(), "{actual:?}");
    assert!(!eof.accepts(&actual));
    assert!(!flags(recover, &reject_eof).iter().all(|p| *p));
    let reject_empty=correct.replace("if test -f \"$backup\"; then :;", "if test -f \"$backup\"; then text=$(cat < \"$backup\"); if test -z \"$text\"; then printf 'Missing backup\\n' >&2; exit 2; fi;");
    assert_ne!(&reject_empty, correct);
    let empty = cases(recover)
        .iter()
        .find(|c| c.name.contains("empty backup"))
        .unwrap();
    let actual = shell::run(&reject_empty, empty);
    assert!(actual.error.is_none(), "{actual:?}");
    assert!(!empty.accepts(&actual));
    assert!(!flags(recover, &reject_empty).iter().all(|p| *p));
    let consuming = alternate(recover).replace(
        "cat < \"$backup\" > \"$current\"",
        "mv -- \"$backup\" \"$current\"",
    );
    let restore = cases(recover)
        .iter()
        .find(|c| c.name == "Restore preserves backup")
        .unwrap();
    let out = shell::run(&consuming, restore);
    assert!(out.error.is_none(), "{out:?}");
    assert_eq!((&out.stdout, out.status), (&restore.stdout, 0));
    assert!(!restore.accepts(&out));
    let cleanup = labs().find(|s| s.id == "shell-cleanup-preview").unwrap();
    let late = cases(cleanup)
        .iter()
        .find(|c| c.name == "Late protected request")
        .unwrap();
    let incremental = "mode=$1; shift; for file in \"$@\"; do case \"$file\" in cache-old|'cache spare'|-cache) if test -f \"$file\"; then rm -- \"$file\"; fi ;; *) printf 'Unsafe cleanup request\\n' >&2; exit 2 ;; esac; done";
    let out = shell::run(incremental, late);
    assert!(out.error.is_none(), "{out:?}");
    assert_eq!(
        (&out.stdout, &out.stderr, out.status),
        (&late.stdout, &late.stderr, late.status)
    );
    assert_ne!(out.files, late.files);
    assert!(!late.accepts(&out));
}
