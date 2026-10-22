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
#[test]
fn plausible_repairs_are_partial_and_current_success_cannot_survive_changed_source_or_results() {
    for s in pack::bundled()
        .unwrap()
        .iter()
        .filter(|s| s.unit == "shell-file-batches")
    {
        let source = &s.solution[0];
        let wrong = match s.id.as_str() {
            "shell-file-manifest" => source.replace("for file in *.log", "for file in '*.log'"),
            "shell-collect-errors" => source.replace("exit 0\n", ""),
            _ => source.replace(
                "publish *.log",
                "printf '' > \"$destination\"\npublish *.log",
            ),
        };
        let results: Vec<_> = cases(s)
            .iter()
            .map(|c| c.accepts(&shell::run(&wrong, c)))
            .collect();
        assert!(
            results.contains(&true) && results.contains(&false),
            "{} {results:?}",
            s.id
        );
        let mut state = State::Script {
            source: source.clone(),
            run: Some(shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases(s).iter().map(|c| shell::run(source, c)).collect(),
            }),
        };
        assert!(engine::solved(&state, &s.goal));
        if let State::Script {
            run: Some(record), ..
        } = &mut state
        {
            record.results[0]
                .files
                .insert("/work/unwanted.txt".into(), "damage".into());
        }
        assert!(!engine::solved(&state, &s.goal));
        if let State::Script { source, .. } = &mut state {
            source.push_str("\nprintf broken");
        }
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn each_declared_example_experiment_has_the_taught_exact_effect_and_no_assignment_credit() {
    let labs = pack::bundled().unwrap();
    for s in labs.iter().filter(|s| s.unit == "shell-file-batches") {
        let t = s.teaching.as_ref().unwrap();
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        let original = a.state.clone();
        let (from, to) = match s.id.as_str() {
            "shell-file-manifest" => ("*.note", "*.missing"),
            "shell-collect-errors" => ("TODO", "LATER"),
            _ => ("target=practice.txt", "target=blue.note"),
        };
        let changed = t.example.replacen(from, to, 1);
        let r = shell::run(&changed, &t.fixture);
        assert_eq!(r.error, None, "{}", s.id);
        let mut after = t.fixture.files.clone();
        let (stderr, status) = match s.id.as_str() {
            "shell-file-manifest" => {
                after.insert("/work/list.txt".into(), String::new());
                ("", 0)
            }
            "shell-collect-errors" => {
                after.insert("/work/flags.txt".into(), String::new());
                ("", 0)
            }
            _ => ("Choose a separate list file\n", 2),
        };
        assert_eq!(
            (r.stdout.as_str(), r.stderr.as_str(), r.status),
            ("", stderr, status),
            "{}",
            s.id
        );
        assert_eq!(r.files, after, "{}", s.id);
        let l = a
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        l.example_source = changed;
        l.example_result = Some(r);
        l.view = View::Example;
        assert_eq!(a.state, original);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn capstone_protects_late_collisions_and_freezes_new_log_destination_out_of_the_sources() {
    let labs = pack::bundled().unwrap();
    let s = labs
        .iter()
        .find(|s| s.id == "shell-file-batch-capstone")
        .unwrap();
    let collision = cases(s)
        .iter()
        .find(|c| c.name == "Later selected source is the destination")
        .unwrap();
    let result = shell::run(&s.solution[0], collision);
    assert!(collision.accepts(&result));
    assert_eq!(result.files, collision.files);
    let source = s.solution[0].replace(
        "publish *.log",
        "printf '' > \"$destination\"\npublish *.log",
    );
    let damaged = shell::run(&source, collision);
    assert_eq!(damaged.error, None);
    assert_eq!(damaged.status, 2);
    assert_ne!(damaged.files, collision.files);
    assert!(!collision.accepts(&damaged));
    let empty_name = cases(s)
        .iter()
        .find(|c| c.name == "Empty destination")
        .unwrap();
    let omitted_guard = s.solution[0].replace(" || [ -z \"$1\" ]", "");
    let bad = shell::run(&omitted_guard, empty_name);
    assert!(!empty_name.accepts(&bad));
    assert!(bad.error.is_some() || bad.stderr != empty_name.stderr);
    let fresh = cases(s)
        .iter()
        .find(|c| c.name == "New report also ends in .log")
        .unwrap();
    assert!(fresh.accepts(&shell::run(&s.solution[0], fresh)));
    assert!(!fresh.accepts(&shell::run(&source, fresh)));
    // Valid empty selection still creates a meaningful report with a zero count.
    let empty = cases(s)
        .iter()
        .find(|c| c.name == "No visible logs")
        .unwrap();
    assert_eq!(
        shell::run(&s.solution[0], empty).files["/work/audit.txt"],
        "Error audit\nFiles: 0\n"
    );
}
#[test]
fn drafts_examples_and_prior_catalog_progress_resume_without_selected_reset_cross_talk() {
    let labs = pack::bundled().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    // Existing catalog and every draft are retained exactly as authored, no invented mastery.
    for s in labs.iter().take(55) {
        p.attempts.insert(s.id.clone(), Attempt::new(s, 1));
    }
    let old = serde_json::to_value(&p).unwrap();
    for s in labs.iter().filter(|s| s.unit == "shell-file-batches") {
        let mut a = Attempt::new(s, 2);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, s.teaching.as_ref().unwrap(), true);
        if let State::Script { source, .. } = &mut a.state {
            source.push_str("\n# unfinished personal assignment draft");
        }
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
    progress::save(&path, &p).unwrap();
    let mut q = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&q).unwrap()
    );
    let selected = labs.iter().find(|s| s.id == "shell-file-manifest").unwrap();
    q.attempts
        .insert(selected.id.clone(), Attempt::new(selected, 3));
    progress::save(&path, &q).unwrap();
    for s in labs.iter().take(55) {
        assert_eq!(
            serde_json::to_value(&q.attempts[&s.id]).unwrap(),
            old["attempts"][&s.id]
        );
    }
    let other = &q.attempts["shell-file-batch-capstone"];
    let State::Script { source, .. } = &other.state else {
        panic!()
    };
    assert!(source.ends_with("# unfinished personal assignment draft"));
    assert!(
        other
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source
            .ends_with("# separate practice draft")
    );
}
#[test]
fn authored_methods_handle_a_larger_batch_without_hardcoded_names() {
    let labs = pack::bundled().unwrap();
    let s = labs
        .iter()
        .find(|s| s.id == "shell-file-batch-capstone")
        .unwrap();
    let mut c = cases(s)[0].clone();
    c.files.clear();
    c.after.clear();
    let mut report = "Error audit\n".to_string();
    for n in 0..24 {
        let name = format!("source {n:02}.log");
        c.files.insert(
            format!("/work/{name}"),
            "OK\nERROR repeated\nERROR repeated\n".into(),
        );
        report.push_str(&format!("File: {name}\nERROR repeated\nERROR repeated\n"));
    }
    report.push_str("Files: 24\n");
    c.args = vec!["new.log".into()];
    c.after = c.files.clone();
    c.after.insert("/work/new.log".into(), report);
    assert!(c.valid());
    let r = shell::run(&s.solution[0], &c);
    assert!(c.accepts(&r), "{r:?}");
}
