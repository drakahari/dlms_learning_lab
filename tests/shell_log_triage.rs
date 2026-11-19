//! Fixed recipes evaluated only by the bounded in-process simulator, not a host shell.
use dlms_learning_lab::shell::{self, Case, Vm};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    source: String,
    alternate: String,
    cases: Vec<Case>,
    example_source: String,
    example_case: Case,
}
fn fixture() -> Fixture {
    let f: Fixture =
        serde_json::from_str(include_str!("fixtures/shell-log-triage-recipes-v1.json")).unwrap();
    assert_eq!(f.schema_version, 1);
    assert_eq!(f.cases.len(), 12);
    assert!(f.cases.iter().all(Case::valid));
    assert!(f.example_case.valid());
    f
}
fn flags(f: &Fixture, s: &str) -> Vec<bool> {
    f.cases
        .iter()
        .map(|c| c.accepts(&shell::run(s, c)))
        .collect()
}
#[test]
fn log_triage_map_and_anchored_lookup_both_match_complete_behavior_and_boundaries() {
    let f = fixture();
    assert_eq!(shell::EVALUATOR_REVISION, 19);
    assert_ne!(f.source, f.alternate);
    let boundaries: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/shell-log-triage-boundaries-v1.json")).unwrap();
    assert_eq!(boundaries.len(), 12);
    for s in [&f.source, &f.alternate] {
        for c in f.cases.iter().chain(&boundaries) {
            assert!(c.valid());
            let r = shell::run(s, c);
            assert!(c.accepts(&r), "{}: actual{r:?} expected{c:?}", c.name);
        }
    }
    let r = shell::run(&f.example_source, &f.example_case);
    assert!(f.example_case.accepts(&r), "{r:?}");
    assert_ne!(f.example_case.files, f.cases[0].files);
}
#[test]
fn log_triage_wrong_supported_repairs_remain_partial_and_cannot_hide_failures() {
    let f = fixture();
    let duplicate = "    if test -n \"${owners[$device]}\"; then\n        printf 'Duplicate device: %s\\n' \"$device\" >&2\n        exit 2\n    fi\n";
    assert!(f.source.contains(duplicate));
    let methods = [
        f.source.replace(" || test -n \"$entry\"", ""),
        f.source.replace(" || test -n \"$event\"", ""),
        f.source.replace("while IFS= read -r", "while read -r"),
        f.source.replace(duplicate, ""),
        f.source
            .replace("owner=\"${owners[$device]}\"", "owner='Network team'"),
        f.source
            .replace("if test \"$skipped\" -gt 0; then exit 1; fi", ""),
    ];
    for s in &methods {
        assert_ne!(s, &f.source);
        let results: Vec<_> = f.cases.iter().map(|c| shell::run(s, c)).collect();
        assert!(
            results.iter().all(|r| r.error.is_none()),
            "plausible wrong supported recipe must execute: {results:?}"
        );
        let pass = flags(&f, s);
        assert!(
            pass.iter().any(|p| *p) && pass.iter().any(|p| !*p),
            "partial incorrect method: {pass:?}"
        );
    }
    let out = shell::run(&methods[5], &f.cases[3]);
    assert_eq!(out.stdout, f.cases[3].stdout);
    assert_eq!(out.stderr, f.cases[3].stderr);
    assert_eq!(out.status, 0);
    assert_eq!(f.cases[3].status, 1);
    assert!(
        !f.cases[3].accepts(&out),
        "right report and diagnostics cannot mask wrong overall status"
    );
    let unquoted = f
        .source
        .replace("\"$owner\" \"$message\"", "$owner $message");
    assert_ne!(unquoted, f.source);
    let pass = flags(&f, &unquoted);
    assert!(pass.iter().any(|p| *p) && pass.iter().any(|p| !*p));
    let plain = shell::run(&unquoted, &f.cases[0]);
    assert!(plain.error.is_none());
    assert!(!f.cases[0].accepts(&plain));
    let brackets = shell::run(&unquoted, &f.cases[1]);
    assert!(
        brackets
            .error
            .as_ref()
            .is_some_and(|m| m.contains("Unsupported bracket filename pattern"))
    );
    assert!(!f.cases[1].accepts(&brackets));
}
#[test]
fn log_triage_first_sample_hardcoding_and_file_damage_never_complete_the_roster() {
    let f = fixture();
    let c = &f.cases[0];
    let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
    let hardcoded = format!(
        "printf '%s' {}\nprintf '%s' {} >&2\nexit {}\n",
        quote(&c.stdout),
        quote(&c.stderr),
        c.status
    );
    let pass = flags(&f, &hardcoded);
    assert!(pass[0]);
    assert!(pass.iter().skip(1).any(|p| !*p));
    let damaged = f.source.replace(
        "declare -A owners",
        "printf 'changed\\n' > keep.txt\ndeclare -A owners",
    );
    assert!(flags(&f, &damaged).iter().any(|p| !*p));
}
#[test]
fn log_triage_ticks_stop_and_unsupported_escape_boundaries_remain_bounded() {
    let f = fixture();
    let c = &f.cases[0];
    let mut vm = Vm::new(&f.source, c).unwrap();
    while !vm.done() {
        vm.tick(7);
    }
    assert!(c.accepts(&vm.outcome));
    let mut vm = Vm::new("while true; do :; done\n", c).unwrap();
    vm.tick(16);
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    for s in [
        "if true; then\n",
        "awk '{print}' owners.txt\n",
        "cat /etc/passwd\n",
        "cat ../outside\n",
        "while true; do printf 'more\\n'; done\n",
    ] {
        let out = shell::run(s, c);
        assert!(out.error.is_some(), "{s}: {out:?}");
        assert!(!c.accepts(&out));
    }
}

#[test]
fn log_triage_unadmitted64_record_proposal_fails_honestly_at_the_unchanged_work_bound() {
    let f = fixture();
    let c: Case =
        serde_json::from_str(include_str!("fixtures/shell-log-triage-limit-v1.json")).unwrap();
    let r = shell::run(&f.source, &c);
    assert!(
        r.error
            .as_ref()
            .is_some_and(|m| m.contains("Expansion work limit"))
    );
    assert!(!c.accepts(&r));
    assert_eq!(r.files, c.files);
}
