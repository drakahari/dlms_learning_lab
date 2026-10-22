use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, View},
    shell, workspace,
};

#[test]
fn count_tasks_grade_partial_behavior_and_invalidate_edited_success() {
    let labs = pack::bundled().unwrap();
    for s in labs.iter().filter(|s| s.unit == "shell-counts") {
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        let src = &s.solution[0];
        let partial = match s.id.as_str() {
            "shell-number-records" => src.replace(" || [ -n \"$record\" ]", ""),
            "shell-order-total" => src.replace("units + quantity", "quantity"),
            "shell-pack-items" => src.replace("items % capacity", "capacity"),
            _ => src
                .replace("    count=0\n", "")
                .replace("total=0\n", "total=0\ncount=0\n"),
        };
        let passed: Vec<_> = cases
            .iter()
            .map(|c| c.accepts(&shell::run(&partial, c)))
            .collect();
        assert!(
            passed.contains(&true) && passed.contains(&false),
            "{} {passed:?}",
            s.id
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
fn every_counting_experiment_has_an_exact_educational_outcome() {
    let labs = pack::bundled().unwrap();
    for (id, from, to, stdout, stderr, status) in [
        (
            "shell-number-records",
            "visits + 1",
            "visits + 2",
            "Visit 2: Lobby\nVisit 4: Workshop\n",
            "",
            0,
        ),
        (
            "shell-order-total",
            "boxes * price + fee",
            "boxes * (price + fee)",
            "Charge (cents): 30\n",
            "",
            0,
        ),
        (
            "shell-pack-items",
            "tray=5",
            "tray=0",
            "",
            "Choose a nonzero tray size\n",
            2,
        ),
        (
            "shell-count-capstone",
            "total + count",
            "total + 1",
            "morning.txt: 2\nevening.txt: 1\nCombined: 2\n",
            "",
            0,
        ),
    ] {
        let s = labs.iter().find(|s| s.id == id).unwrap();
        let l = s.teaching.as_ref().unwrap();
        let r = shell::run(&l.example.replace(from, to), &l.fixture);
        assert_eq!(r.error, None, "{id}");
        assert_eq!(
            (r.stdout.as_str(), r.stderr.as_str(), r.status),
            (stdout, stderr, status),
            "{id}"
        );
        assert_eq!(r.files, l.fixture.files);
    }
}

#[test]
fn zero_capacity_is_an_intentional_rejection_not_an_arithmetic_crash() {
    let labs = pack::bundled().unwrap();
    let s = labs.iter().find(|s| s.id == "shell-pack-items").unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let c = cases.iter().find(|c| c.name == "Zero capacity").unwrap();
    let r = shell::run(&s.solution[0], c);
    assert!(c.accepts(&r));
    let r = shell::run("items=$1\ncapacity=$2\nfull=$((items / capacity))", c);
    assert!(
        r.error
            .as_ref()
            .is_some_and(|e| e.contains("division by zero"))
    );
    assert!(!c.accepts(&r));
    let wrong = s.solution[0].replace("if [ \"$#\" -ne 2 ]; then", "if false; then");
    let c = cases.iter().find(|c| c.name == "Extra argument").unwrap();
    assert!(!c.accepts(&shell::run(&wrong, c)));
}

#[test]
fn capstone_preflights_before_even_creating_an_empty_report_and_handles_declared_maximum() {
    let labs = pack::bundled().unwrap();
    let s = labs
        .iter()
        .find(|s| s.id == "shell-count-capstone")
        .unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let c = cases
        .iter()
        .find(|c| c.name == "Failure must not create report")
        .unwrap();
    let wrong = format!("printf '' >> \"$1\"\n{}", s.solution[0]);
    let r = shell::run(&wrong, c);
    assert_eq!(r.error, None);
    assert_eq!(
        (&r.stdout, &r.stderr, r.status),
        (&c.stdout, &c.stderr, c.status)
    );
    assert!(!c.accepts(&r));
    let mut c = cases[0].clone();
    c.args = vec!["largest count.txt".into()];
    c.files.clear();
    c.after.clear();
    let mut report = "Record counts\n".to_string();
    for n in 0..8 {
        let name = format!("input {n}.txt");
        c.args.push(name.clone());
        // 16 records: 15 terminated records plus the final unterminated one.
        c.files
            .insert(format!("/work/{name}"), "record\n".repeat(15) + "final");
        report.push_str(&format!("{name}: 16\n"));
    }
    report.push_str("Total: 128\n");
    c.after = c.files.clone();
    c.after.insert("/work/largest count.txt".into(), report);
    c.stdout = "Saved: largest count.txt\n".into();
    assert!(c.valid());
    assert!(
        c.accepts(&shell::run(&s.solution[0], &c)),
        "{:?}",
        shell::run(&s.solution[0], &c)
    );
    // Renaming the files in the request cannot be replaced with a known sample path.
    let s = labs
        .iter()
        .find(|s| s.id == "shell-number-records")
        .unwrap();
    let Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let wrong = s.solution[0].replace("\"$1\"", "queue.txt");
    assert!(cases[0].accepts(&shell::run(&wrong, &cases[0])));
    assert!(cases.iter().any(|c| !c.accepts(&shell::run(&wrong, c))));
}

#[test]
fn independent_counting_drafts_resume_and_selected_reset_preserves_all_other_attempts() {
    let labs = pack::bundled().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let old = &labs[9];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    for s in labs.iter().filter(|s| s.unit == "shell-counts") {
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let before = a.state.clone();
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
        assert_eq!(a.state, before);
        assert!(!progress::current_solved(&a, s));
        if let State::Script { source, .. } = &mut a.state {
            source.push_str("\n# assignment investigation");
        }
        p.attempts.insert(s.id.clone(), a);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_vec(&restored).unwrap(),
        serde_json::to_vec(&p).unwrap()
    );
    let selected = labs.iter().find(|s| s.id == "shell-order-total").unwrap();
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
