use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, View},
    shell, workspace,
};
fn cases(s: &pack::Scenario) -> &[shell::Case] {
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    cases
}
fn state(source: &str, s: &pack::Scenario) -> State {
    State::Script {
        source: source.into(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: source.into(),
            results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
        }),
    }
}
#[test]
fn array_assignments_diagnose_partial_repairs_and_invalidate_changed_source_or_files() {
    let reviews: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shell-content-review.json")).unwrap();
    for s in pack::bundled()
        .unwrap()
        .iter()
        .filter(|s| s.unit == "shell-arrays")
    {
        let wrong = if s.id == "shell-array-intake" {
            s.solution[0].replace("\"${queue[1]}\"", "\"${queue[0]}\"")
        } else {
            reviews
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == s.id)
                .unwrap()["incorrect"]
                .as_str()
                .unwrap()
                .into()
        };
        let passes: Vec<_> = cases(s)
            .iter()
            .map(|c| c.accepts(&shell::run(&wrong, c)))
            .collect();
        assert!(
            passes.contains(&true) && passes.contains(&false),
            "{} {passes:?}",
            s.id
        );
        assert!(!engine::solved(&state(&wrong, s), &s.goal));
        let mut correct = state(&s.solution[0], s);
        assert!(engine::solved(&correct, &s.goal));
        if let State::Script { run: Some(r), .. } = &mut correct {
            r.results[0]
                .files
                .insert("/work/unrequested.txt".into(), "damage".into());
        }
        assert!(!engine::solved(&correct, &s.goal));
        correct = state(&s.solution[0], s);
        if let State::Script { source, .. } = &mut correct {
            source.push_str("\nprintf broken");
        }
        assert!(!engine::solved(&correct, &s.goal));
    }
}
#[test]
fn all_suggested_practice_changes_are_runnable_exact_and_ungraded() {
    for s in pack::bundled()
        .unwrap()
        .iter()
        .filter(|s| s.unit == "shell-arrays")
    {
        let t = s.teaching.as_ref().unwrap();
        let (from, to, expected) = match s.id.as_str() {
            "shell-array-intake" => (
                "books=('Field guide' '' '*.txt')",
                "books=()",
                "Books: 0\nFirst: <>\n",
            ),
            "shell-array-priority" => (
                "stops='Workshop'",
                "stops=('Workshop')",
                "Original: <Depot>\nOriginal: <East gate>\nOriginal: <North gate>\nUpdated: <Workshop>\n",
            ),
            "shell-array-package" => ("*.csv", "*.json", "Tables: 0\n"),
            _ => ("\"$1\" = preview", "\"$1\" = save", "Checklist saved\n"),
        };
        let source = t.example.replacen(from, to, 1);
        assert_ne!(source, t.example);
        let r = shell::run(&source, &t.fixture);
        assert_eq!(r.error, None, "{}", s.id);
        assert_eq!(r.status, 0);
        assert_eq!(r.stderr, "");
        assert_eq!(r.stdout, expected, "{}", s.id);
        let mut files = t.fixture.files.clone();
        if s.id == "shell-array-handover" {
            files.insert(
                "/work/checklist.txt".into(),
                "Checklist: 2\nCheck: Cable check\nCheck: Map update\n".into(),
            );
        }
        assert_eq!(r.files, files);
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, t, true);
        let original = a.state.clone();
        let learning = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        learning.example_source = source;
        learning.example_result = Some(r);
        learning.view = View::Example;
        assert_eq!(a.state, original);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn prior_seventy_attempts_and_new_example_assignment_drafts_resume_and_reset_independently() {
    let labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs.iter().take(70) {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let original = serde_json::to_vec(&p).unwrap();
    assert_eq!(p.attempts.len(), 70);
    for s in labs.iter().filter(|s| s.unit == "shell-arrays") {
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        a.state = state(&s.solution[0], s);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source.push_str("\n# separate practice draft");
        l.view = View::Assignment;
        p.attempts.insert(s.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut loaded = progress::decode(&bytes, &labs).unwrap();
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), bytes);
    for s in labs.iter().filter(|s| s.unit == "shell-arrays") {
        assert!(progress::current_solved(&loaded.attempts[&s.id], s));
    }
    let selected = labs
        .iter()
        .find(|s| s.id == "shell-array-priority")
        .unwrap();
    loaded
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 3));
    assert!(!progress::current_solved(
        &loaded.attempts[&selected.id],
        selected
    ));
    assert_eq!(
        serde_json::to_vec(&loaded.attempts["shell-array-intake"]).unwrap(),
        serde_json::to_vec(&p.attempts["shell-array-intake"]).unwrap()
    );
    loaded
        .attempts
        .retain(|id, _| !id.starts_with("shell-array-"));
    assert_eq!(serde_json::to_vec(&loaded).unwrap(), original);
}
#[test]
fn handover_rejects_empty_names_before_any_output_and_preserves_late_failure_data() {
    let labs = pack::bundled().unwrap();
    let s = labs
        .iter()
        .find(|s| s.id == "shell-array-handover")
        .unwrap();
    let source = &s.solution[0];
    // These requirements must belong to grading itself, not only a reference test.
    for omitted in [
        "if [ -z \"$destination\" ]; then printf 'Cannot publish handover\\n' >&2; exit 2; fi\n",
        "[ -z \"$file\" ] || ",
        "[ \"$file\" = \"$destination\" ]",
    ] {
        let wrong = source.replacen(
            omitted,
            if omitted.starts_with("[ \"$file\"") {
                "false"
            } else {
                ""
            },
            1,
        );
        assert_ne!(&wrong, source);
        assert!(
            !engine::solved(&state(&wrong, s), &s.goal),
            "omitted {omitted}"
        );
    }
    for args in [
        vec!["save", "", "west.cfg"],
        vec!["preview", "handover.txt", ""],
    ] {
        let mut c = cases(s)[0].clone();
        c.args = args.into_iter().map(str::to_string).collect();
        let r = shell::run(source, &c);
        assert_eq!(r.error, None);
        assert_eq!(r.status, 2);
        assert_eq!(r.stderr, "Cannot publish handover\n");
        assert_eq!(r.stdout, "");
        assert_eq!(r.files, c.files);
    }
    for c in cases(s).iter().filter(|c| c.status == 2) {
        let r = shell::run(source, c);
        assert!(c.accepts(&r));
        assert_eq!(r.files, c.files);
        assert_eq!(r.stdout, "");
    }
}

#[test]
fn bounded_case_increase_keeps_records_and_loaded_packs_in_agreement() {
    let mut labs = pack::bundled().unwrap();
    let s = labs
        .iter_mut()
        .find(|s| s.id == "shell-array-handover")
        .unwrap();
    let mut record = match state(&s.solution[0], s) {
        State::Script { run: Some(r), .. } => r,
        _ => unreachable!(),
    };
    assert!(record.valid());
    s.prerequisites.clear();
    s.unit.clear();
    if let Goal::Script { cases } = &mut s.goal {
        while cases.len() < shell::MAX_CASES {
            cases.push(cases[0].clone());
        }
    }
    while record.results.len() < shell::MAX_CASES {
        record.results.push(record.results[0].clone());
    }
    assert!(record.valid());
    let data = serde_json::json!({"schema_version":1,"scenarios":[s.clone()]});
    assert!(pack::load(&serde_json::to_string(&data).unwrap()).is_ok());
    if let Goal::Script { cases } = &mut s.goal {
        cases.push(cases[0].clone());
    }
    record.results.push(record.results[0].clone());
    assert!(!record.valid());
    let data = serde_json::json!({"schema_version":1,"scenarios":[s]});
    assert!(
        pack::load(&serde_json::to_string(&data).unwrap())
            .err()
            .unwrap()
            .contains("invalid script cases")
    );
}
#[test]
fn input_lists_stay_distinct_and_array_wrong_quoting_cannot_earn_credit() {
    let labs = pack::bundled().unwrap();
    let s = labs.iter().find(|s| s.id == "shell-array-intake").unwrap();
    for c in cases(s) {
        let r = shell::run(&s.solution[0], c);
        assert!(c.accepts(&r));
    }
    let wrong = s.solution[0].replace("queue=(\"$@\")", "queue=(\"$1\" \"$2\")");
    assert!(!engine::solved(&state(&wrong, s), &s.goal));
    let s = labs.iter().find(|s| s.id == "shell-array-package").unwrap();
    let c = cases(s)
        .iter()
        .find(|c| c.name == "Configuration only")
        .unwrap();
    let r = shell::run(&s.solution[0], c);
    assert!(r.stdout.ends_with("Documents: 0\n"));
    assert!(!r.stdout.contains("*.txt"));
}
