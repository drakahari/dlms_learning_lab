use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, View},
    shell, workspace,
};
#[test]
fn new_text_tasks_allow_partial_work_but_require_current_complete_behavior() {
    let labs = pack::bundled().unwrap();
    for id in [
        "shell-cut-inventory",
        "shell-sort-manifest",
        "shell-uniq-counts",
        "shell-text-capstone",
    ] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        let src = &s.solution[0];
        let partial = match id {
            "shell-cut-inventory" => src.replace("-s ", ""),
            "shell-sort-manifest" => src.replace("-u ", ""),
            "shell-uniq-counts" => src.replace("sort | ", ""),
            _ => src.replace("uniq -d", "uniq"),
        };
        let passed: Vec<_> = cases
            .iter()
            .map(|c| c.accepts(&shell::run(&partial, c)))
            .collect();
        assert!(
            passed.contains(&true) && passed.contains(&false),
            "{id}: {passed:?}"
        );
        let mut state = State::Script {
            source: src.clone(),
            run: Some(shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: src.clone(),
                results: cases.iter().map(|c| shell::run(src, c)).collect(),
            }),
        };
        assert!(engine::solved(&state, &s.goal));
        if let State::Script { source, .. } = &mut state {
            source.push_str("\nprintf broken");
        }
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn text_practice_and_assignment_drafts_survive_selected_reset_without_changing_old_credit() {
    let labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let old = &labs[9];
    let mut a = Attempt::new(old, 1);
    if let State::Script { source, .. } = &mut a.state {
        *source = "# earlier unfinished draft".into();
    }
    p.attempts.insert(old.id.clone(), a);
    for s in labs.iter().filter(|s| s.unit == "shell-text") {
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
        l.example_source.push_str("\nprintf 'experiment\\n'");
        l.example_result = Some(shell::run(
            &l.example_source,
            &s.teaching.as_ref().unwrap().fixture,
        ));
        assert_eq!(a.state, assignment);
        assert!(!progress::current_solved(&a, s));
        if let State::Script { source, .. } = &mut a.state {
            source.push_str("\n# assignment investigation");
        }
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    let selected = labs.iter().find(|s| s.id == "shell-uniq-counts").unwrap();
    restored
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 3));
    for (id, a) in &p.attempts {
        if id != &selected.id {
            assert_eq!(
                serde_json::to_vec(&restored.attempts[id]).unwrap(),
                serde_json::to_vec(a).unwrap()
            );
        }
    }
}
#[test]
fn duplicate_audit_rejects_premature_output_creation_and_wrong_argument_counts() {
    let labs = pack::bundled().unwrap();
    let s = labs.iter().find(|s| s.id == "shell-text-capstone").unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let c = cases
        .iter()
        .find(|c| c.name == "Missing source must not create output")
        .unwrap();
    let wrong = format!("printf '' >> \"$2\"\n{}", s.solution[0]);
    let r = shell::run(&wrong, c);
    assert_eq!(r.error, None);
    assert_eq!(
        (&r.stdout, &r.stderr, r.status),
        (&c.stdout, &c.stderr, c.status)
    );
    assert!(!c.accepts(&r));
    for args in [
        vec![],
        vec!["missing.txt".into()],
        vec!["exists.txt".into(), "new.txt".into(), "extra".into()],
    ] {
        let mut c = c.clone();
        c.args = args;
        c.files
            .insert("/work/exists.txt".into(), "a:x\na:y\n".into());
        c.after = c.files.clone();
        let r = shell::run(&s.solution[0], &c);
        assert!(c.accepts(&r), "{r:?}");
    }
}

#[test]
fn changing_supplied_input_filenames_defeats_hardcoded_paths() {
    let labs = pack::bundled().unwrap();
    for (id, first) in [
        ("shell-cut-inventory", "inventory.txt"),
        ("shell-sort-manifest", "queue.txt"),
        ("shell-uniq-counts", "locations.txt"),
    ] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        let wrong = s.solution[0].replace("\"$1\"", &format!("'{first}'"));
        assert!(cases[0].accepts(&shell::run(&wrong, &cases[0])));
        assert!(
            cases.iter().any(|c| !c.accepts(&shell::run(&wrong, c))),
            "{id}"
        );
    }
}
#[test]
fn audit_argument_count_is_graded_separately_from_missing_inputs_and_empty_names() {
    let labs = pack::bundled().unwrap();
    let s = labs.iter().find(|s| s.id == "shell-text-capstone").unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let c = cases
        .iter()
        .find(|c| c.name == "Extra argument preserves existing files")
        .unwrap();
    let wrong = s.solution[0].replace("[ \"$#\" -ne 2 ] || ", "");
    assert_ne!(wrong, s.solution[0]);
    let r = shell::run(&wrong, c);
    assert_eq!(r.error, None);
    assert!(!c.accepts(&r));
    // A valid empty source remains a successful heading-only audit.
    let mut empty = cases[0].clone();
    empty.files.insert("/work/assets.txt".into(), String::new());
    empty.after = empty.files.clone();
    empty
        .after
        .insert("/work/audit.txt".into(), "Duplicate asset IDs\n".into());
    assert!(empty.accepts(&shell::run(&s.solution[0], &empty)));
}
