//! Actual parent/child state, not command spelling or look-alike output.
use dlms_learning_lab::{environment_workspace as e, shell};
use std::collections::BTreeMap;
fn input(
    name: &str,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    stdout: String,
) -> shell::Case {
    let files = BTreeMap::from([
        ("/work/keep.txt".into(), "protected".into()),
        ("/work/SITE=wrong.txt".into(), "decoy".into()),
    ]);
    shell::Case {
        name: name.into(),
        args,
        env,
        stdin: String::new(),
        files: files.clone(),
        after: files,
        stdout,
        stderr: String::new(),
        status: 0,
    }
}
fn task(note: bool) -> e::Task {
    let mut cases = Vec::new();
    for (n, (site, mode, team)) in [
        ("Atrium East", "brief", "ops"),
        ("Hill Room", "wide", "east ops"),
        ("*", "", ""),
    ]
    .iter()
    .enumerate()
    {
        let mut env = BTreeMap::from([("TEAM".into(), (*team).into())]);
        let mut parent = BTreeMap::from([
            (
                "SITE".into(),
                shell::EnvironmentVariable {
                    value: Some((*site).into()),
                    exported: true,
                },
            ),
            (
                "MODE".into(),
                shell::EnvironmentVariable {
                    value: Some((*mode).into()),
                    exported: true,
                },
            ),
        ]);
        if note {
            let value = if n == 1 { "" } else { "working * note" };
            env.insert("NOTE".into(), value.into());
            parent.insert(
                "NOTE".into(),
                shell::EnvironmentVariable {
                    value: Some(value.into()),
                    exported: false,
                },
            );
        }
        let name = format!("Sample {}", n + 1);
        let given = input(
            &name,
            vec![(*site).into(), (*mode).into()],
            env.clone(),
            String::new(),
        );
        env.remove("NOTE");
        env.insert("SITE".into(), (*site).into());
        env.insert("MODE".into(), (*mode).into());
        let child = input(
            &name,
            vec![],
            env,
            format!("Team: {team}\nSite: {site}\nMode: {mode}\n"),
        );
        cases.push(e::Case {
            input: given,
            parent,
            child,
        });
    }
    e::Task{id:if note{"remove-child-note"}else{"child-site"}.into(),instructions:"Use supplied arguments for SITE and MODE, preserve TEAM and files, and pass only the required child settings.".into(),starter:"SITE=$1\nMODE=$2\nexport MODE\n".into(),child_source:"printf 'Team: %s\\nSite: %s\\nMode: %s\\n' \"$TEAM\" \"$SITE\" \"$MODE\"\n".into(),cases,path_lookup:None}
}
const REFERENCE: &str = "SITE=$1\nMODE=$2\nexport SITE MODE\n";
fn draft(t: &e::Task, source: &str) -> e::Draft {
    e::Draft {
        source: source.into(),
        result: Some(e::run(t, source).unwrap()),
    }
}
#[test]
fn actual_parent_child_cases_and_equivalent_declarations_pass() {
    let t = task(false);
    t.validate_child_examples().unwrap();
    for source in [
        REFERENCE,
        "export SITE=$1 MODE=$2\n",
        "scratch=$1\nMODE=$2\nSITE=$scratch\nexport MODE\nexport SITE\n",
    ] {
        let d = draft(&t, source);
        assert!(e::checks(&t, &d).iter().all(|(ok, _)| *ok));
        for (c, r) in t.cases.iter().zip(&d.result.unwrap().results) {
            let s = r.bindings.as_ref().unwrap();
            assert_eq!(s.child, c.child.env);
            assert_eq!(r.parent.files, c.input.files);
            assert!(c.child.accepts(r.child.as_ref().unwrap()));
        }
    }
}
#[test]
fn printing_like_a_child_missing_exports_partial_and_hardcoded_values_fail() {
    let t = task(false);
    for source in [
        "SITE=$1\nMODE=$2\n",
        "printf 'Team: %s\\nSite: %s\\nMode: %s\\n' \"$TEAM\" \"$1\" \"$2\"\n",
    ] {
        let d = draft(&t, source);
        assert!(e::checks(&t, &d).iter().all(|(ok, _)| !*ok));
    }
    let partial =
        "SITE=$1\nMODE=$2\nexport MODE\nif test \"$SITE\" = 'Atrium East'; then export SITE; fi\n";
    let d = draft(&t, partial);
    let rows = e::checks(&t, &d);
    assert!(rows[0].0);
    assert!(!rows[1].0 && !rows[2].0);
    assert!(rows[1].1.contains("does not receive"));
    let hardcoded = "SITE='Atrium East'\nMODE=brief\nexport SITE MODE\n";
    let d = draft(&t, hardcoded);
    let rows = e::checks(&t, &d);
    assert!(rows[0].0);
    assert!(!rows[1].0 && !rows[2].0);
    assert!(rows[1].1.contains("supplied inputs"));
}
#[test]
fn removing_visibility_preserves_parent_including_empty_values_and_rejects_extra_exports() {
    let t = task(true);
    t.validate_child_examples().unwrap();
    for source in [
        format!("export -n NOTE\n{REFERENCE}"),
        format!("{REFERENCE}export -n NOTE\n"),
    ] {
        let d = draft(&t, &source);
        assert!(e::checks(&t, &d).iter().all(|(ok, _)| *ok));
        for (c, r) in t.cases.iter().zip(&d.result.unwrap().results) {
            let s = r.bindings.as_ref().unwrap();
            assert_eq!(s.parent["NOTE"].value, c.parent["NOTE"].value);
            assert!(!s.parent["NOTE"].exported);
            assert!(!s.child.contains_key("NOTE"));
        }
    }
    let d = draft(&t, REFERENCE);
    assert!(e::checks(&t, &d).iter().all(|(ok, _)| !*ok));
    let extra = format!("export -n NOTE\n{REFERENCE}export SCRATCH=yes\n");
    assert!(
        e::checks(&t, &draft(&t, &extra))
            .iter()
            .all(|(ok, m)| !*ok && m.contains("Unexpected settings"))
    );
}
#[test]
fn syntax_unsupported_and_wrong_behavior_have_real_failure_feedback() {
    let t = task(false);
    let syntax = draft(&t, "if true; then\n");
    assert!(
        syntax
            .result
            .as_ref()
            .unwrap()
            .results
            .iter()
            .all(|r| r.parent.error.is_some())
    );
    assert!(
        e::checks(&t, &syntax)
            .iter()
            .all(|(ok, m)| !*ok && m.contains("Setup could not run"))
    );
    let unsupported = draft(&t, "export -q SITE\n");
    assert!(
        e::checks(&t, &unsupported)
            .iter()
            .all(|(ok, m)| !*ok && m.contains("Unsupported"))
    );
    let broken = draft(&t, "SITE=wrong\nMODE=$2\nexport SITE MODE\n");
    assert!(
        e::checks(&t, &broken)
            .iter()
            .all(|(ok, m)| !*ok && m.contains("parent value"))
    );
}
#[test]
fn separate_example_edit_invalidation_safe_roundtrip_and_context_validation() {
    let assignment = task(false);
    let mut example = assignment.clone();
    example.id = "worked-child-site".into();
    example.starter = REFERENCE.into();
    let definition = e::Definition {
        id: "environment-session".into(),
        assignment: assignment.clone(),
        example: example.clone(),
    };
    let mut session = e::Session::new(definition.clone());
    let original = session.assignment.clone();
    let r = e::run(&example, REFERENCE).unwrap();
    assert!(session.example.store(r, &example));
    assert_eq!(session.assignment, original);
    assert!(!session.solved());
    session.assignment.edit(REFERENCE.into()).unwrap();
    let r = e::run(&assignment, REFERENCE).unwrap();
    assert!(session.assignment.store(r.clone(), &assignment));
    assert!(session.solved());
    session.validate(&definition).unwrap();
    let raw = serde_json::to_vec(&session).unwrap();
    let mut restored = e::Session::decode(&raw, &definition).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&session).unwrap()
    );
    assert!(!restored.solved());
    assert!(restored.assignment.needs_reverification(&assignment));
    let rechecked = e::run(&assignment, &restored.assignment.source).unwrap();
    assert!(restored.assignment.store(rechecked, &assignment));
    assert!(restored.solved());
    let mut forged = restored.clone();
    forged.assignment.result.as_mut().unwrap().results[0]
        .bindings
        .as_mut()
        .unwrap()
        .child
        .insert("FORGED".into(), "no".into());
    assert!(forged.validate(&definition).is_err());
    session.assignment.edit("SITE=wrong\n".into()).unwrap();
    assert!(!session.solved());
    assert!(session.assignment.result.is_none());
    assert!(!session.assignment.store(r, &assignment));
    let mut wrong = definition.clone();
    wrong.assignment.instructions.push_str("changed");
    assert!(restored.validate(&wrong).is_err());
    let reset = e::Session::new(definition);
    assert!(!reset.solved());
    assert_eq!(reset.assignment, original);
}
#[test]
fn stop_remains_cooperative_in_setup_and_supplied_child_and_never_awards_credit() {
    let t = task(false);
    let mut job = e::Job::new(&t, "while true; do :; done\n").unwrap();
    for _ in 0..20 {
        job.advance(16);
    }
    assert!(!job.done());
    job.stop();
    assert!(job.done());
    assert!(job.record().is_none());
    let mut slow = t.clone();
    slow.child_source = "while true; do :; done\n".into();
    let mut job = e::Job::new(&slow, REFERENCE).unwrap();
    for _ in 0..100 {
        job.advance(16);
    }
    assert!(!job.done());
    job.stop();
    assert!(job.record().is_none());
    assert!(e::Job::new(&t, &"x".repeat(e::MAX_SOURCE + 1)).is_err());
}
#[test]
fn invalid_case_definitions_and_imported_observations_are_rejected() {
    let t = task(false);
    let mut wrong = t.clone();
    wrong.cases[0].child.env.clear();
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.cases[0].child.after.clear();
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.cases.push(wrong.cases[0].clone());
    assert!(wrong.validate().is_err());
    let mut wrong = t.clone();
    wrong.child_source = "printf wrong\n".into();
    assert!(wrong.validate_child_examples().is_err());
}

#[test]
fn cached_source_and_observations_cannot_counterfeit_current_success() {
    let assignment = task(false);
    let mut example = assignment.clone();
    example.id = "worked-cache".into();
    let definition = e::Definition {
        id: "cached-environment".into(),
        assignment: assignment.clone(),
        example,
    };
    let mut session = e::Session::new(definition.clone());
    session.assignment.edit(REFERENCE.into()).unwrap();
    assert!(
        session
            .assignment
            .store(e::run(&assignment, REFERENCE).unwrap(), &assignment)
    );
    assert!(session.solved());
    let bad = "SITE=$1\nMODE=$2\n";
    let mut changed = session.clone();
    changed.assignment.source = bad.into();
    changed.assignment.result.as_mut().unwrap().source = bad.into();
    assert!(!changed.assignment.result.as_ref().unwrap().verified());
    assert!(!changed.solved());
    let raw = serde_json::to_vec(&changed).unwrap();
    let mut restored = e::Session::decode(&raw, &definition).unwrap();
    assert!(!restored.solved());
    assert!(restored.assignment.needs_reverification(&assignment));
    let actual = e::run(&assignment, &restored.assignment.source).unwrap();
    assert!(restored.assignment.store(actual, &assignment));
    assert!(!restored.solved());
    assert!(
        e::checks(&assignment, &restored.assignment)
            .iter()
            .all(|(ok, m)| !*ok && m.contains("does not receive"))
    );
    let mut wrong = draft(&assignment, bad);
    wrong.result.as_mut().unwrap().results = session.assignment.result.unwrap().results;
    assert!(!wrong.result.as_ref().unwrap().verified());
    assert!(e::checks(&assignment, &wrong).iter().all(|(ok, _)| !*ok));
}
#[test]
fn raw_restore_checks_size_before_parse_and_never_imports_execution_verification() {
    let assignment = task(false);
    let mut example = assignment.clone();
    example.id = "worked-import".into();
    let definition = e::Definition {
        id: "import-environment".into(),
        assignment: assignment.clone(),
        example,
    };
    let too_large = vec![b' '; e::MAX_SESSION + 1];
    assert!(
        e::Session::decode(&too_large, &definition)
            .unwrap_err()
            .contains("exceeds")
    );
    let mut session = e::Session::new(definition.clone());
    session.assignment.edit(REFERENCE.into()).unwrap();
    session
        .assignment
        .store(e::run(&assignment, REFERENCE).unwrap(), &assignment);
    let raw = serde_json::to_vec(&session).unwrap();
    assert!(!String::from_utf8_lossy(&raw).contains("verification"));
    let loaded = e::Session::decode(&raw, &definition).unwrap();
    assert!(!loaded.assignment.result.as_ref().unwrap().verified());
    assert!(!loaded.solved());
    let mut malformed = serde_json::to_value(&session).unwrap();
    malformed["assignment"]["result"]["results"][0]["bindings"]["parent"]["SITE"]["exported"] =
        serde_json::json!(1);
    assert!(e::Session::decode(&serde_json::to_vec(&malformed).unwrap(), &definition).is_err());
}
