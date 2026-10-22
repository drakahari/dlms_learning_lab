use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, View},
    shell, workspace,
};
fn record(source: &str, cases: &[shell::Case]) -> State {
    State::Script {
        source: source.into(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.into(),
            results: cases.iter().map(|c| shell::run(source, c)).collect(),
        }),
    }
}
#[test]
fn scope_repairs_distinguish_partial_work_and_retain_only_current_matching_results() {
    let labs = pack::bundled().unwrap();
    for id in [
        "shell-local-heading",
        "shell-scope-review",
        "shell-scope-capstone",
    ] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        let solution = &s.solution[0];
        let partial = match id {
            "shell-local-heading" => solution.replace("local REPORT_TITLE=", "REPORT_TITLE="),
            "shell-scope-review" => solution.replace("matches \"$2\"", "matches \"$2\" || exit 2"),
            _ => solution.replacen("> \"$destination\"", ">> \"$destination\"", 1),
        };
        let outcomes: Vec<_> = cases
            .iter()
            .map(|c| c.accepts(&shell::run(&partial, c)))
            .collect();
        assert!(
            outcomes.contains(&true) && outcomes.contains(&false),
            "{id}: {outcomes:?}"
        );
        assert!(!engine::solved(&record(&partial, cases), &s.goal));
        let mut state = record(solution, cases);
        assert!(engine::solved(&state, &s.goal));
        if let State::Script { source, .. } = &mut state {
            source.push_str("\nprintf broken");
        }
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn scope_example_assignment_drafts_old_attempts_and_selected_reset_roundtrip_independently() {
    let labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let old = &labs[9];
    let mut old_attempt = Attempt::new(old, 1);
    let Goal::Script { cases } = &old.goal else {
        panic!()
    };
    old_attempt.state = record(&old.solution[0], cases);
    p.attempts.insert(old.id.clone(), old_attempt.clone());
    for s in labs.iter().filter(|s| s.unit == "shell-scope") {
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let assignment = a.state.clone();
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.view = View::Example;
        l.example_source
            .push_str("\nprintf 'trying another value\\n'");
        l.example_result = Some(shell::run(
            &l.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        assert_eq!(a.state, assignment);
        assert!(!progress::current_solved(&a, s));
        if let State::Script { source, .. } = &mut a.state {
            source.push_str("\n# unfinished repair");
        }
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
    let selected = labs.iter().find(|s| s.id == "shell-scope-review").unwrap();
    loaded
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 3));
    assert_eq!(
        serde_json::to_vec(&loaded.attempts[&old.id]).unwrap(),
        serde_json::to_vec(&old_attempt).unwrap()
    );
    for s in labs
        .iter()
        .filter(|s| s.unit == "shell-scope" && s.id != selected.id)
    {
        assert_eq!(
            serde_json::to_vec(&loaded.attempts[&s.id]).unwrap(),
            serde_json::to_vec(&p.attempts[&s.id]).unwrap()
        );
    }
}
#[test]
fn review_capstone_rejects_early_redirection_even_when_printed_output_looks_right() {
    let labs = pack::bundled().unwrap();
    let s = labs
        .iter()
        .find(|s| s.id == "shell-scope-capstone")
        .unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let wrong = s.solution[0].replace(
        "for path in \"$@\"; do",
        "printf '' >> \"$destination\"\nfor path in \"$@\"; do",
    );
    let c = cases
        .iter()
        .find(|c| c.name == "Missing input must not create output")
        .unwrap();
    let r = shell::run(&wrong, c);
    assert_eq!(r.error, None);
    assert_eq!(
        (&r.stdout, &r.stderr, r.status),
        (&c.stdout, &c.stderr, c.status)
    );
    assert!(!c.accepts(&r));
}

#[test]
fn repeated_input_sections_are_not_silently_deduplicated() {
    let labs = pack::bundled().unwrap();
    let s = labs
        .iter()
        .find(|s| s.id == "shell-scope-capstone")
        .unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let c = cases
        .iter()
        .find(|c| c.name == "Repeated quiet source keeps both headings")
        .unwrap();
    let wrong=s.solution[0].replace(
        "    printf 'File: %s\\n' \"$path\" >> \"$destination\"\n    grep -F \"$needle\" \"$path\" >> \"$destination\"",
        "    if [ \"$last\" != \"$path\" ]; then\n        printf 'File: %s\\n' \"$path\" >> \"$destination\"\n        grep -F \"$needle\" \"$path\" >> \"$destination\"\n    fi\n    last=$path"
    );
    assert_ne!(wrong, s.solution[0]);
    let r = shell::run(&wrong, c);
    assert_eq!(r.error, None);
    assert!(!c.accepts(&r));
}
