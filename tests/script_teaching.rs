use dlms_learning_lab::{
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::{self as teaching, Diagnostic, View},
    shell, workspace,
};
#[test]
fn every_introduction_example_runs_and_its_suggested_edit_changes_behavior() {
    let labs = pack::bundled().unwrap();
    let edits = [
        ("Mina", "Rowan"),
        ("For:", "Recipient:"),
        ("Note:", "Reminder:"),
        ("-z", "-n"),
        ("OPEN", "CLOSED"),
        ("Reading", "Menu:"),
        ("-f", "-z"),
        ("2>", ">"),
        ("OPEN", "CLOSED"),
        (r#" || [ -n "$name" ]"#, ""),
        ("found_problem=yes", "found_problem=''"),
        ("Rowan Reed", "Tess Ford"),
        ("return 1", "exit 1"),
        ("shift\n", "shift 0\n"),
        (r#"if available "$@""#, "if available menu.txt"),
        ("local title=", "title="),
        ("loud", "busy"),
        ("local destination=", "destination="),
        ("1,3", "2,3"),
        ("-u", "-r"),
        ("-c", "-d"),
        ("-d\n", "-u\n"),
        ("visits + 1", "visits + 2"),
        ("boxes * price + fee", "boxes * (price + fee)"),
        ("tray=5", "tray=0"),
        ("total + count", "total + 1"),
        ("*.note", "*.missing"),
        ("TODO", "LATER"),
        ("target=practice.txt", "target=blue.note"),
        ("books=('Field guide' '' '*.txt')", "books=()"),
        ("stops='Workshop'", "stops=('Workshop')"),
        ("*.csv", "*.json"),
        ("\"$1\" = preview", "\"$1\" = save"),
        ("loans.txt", "returns.txt"),
        (r"%s\n\n%s\n\n", r"%s\n\n\n%s\n\n"),
        ("return 4", "return 0"),
        ("note=$(make_note)", "make_note; note='direct call'"),
        ("quick|summary", "'quick*'"),
        (
            r#"send:rush:*) printf '%s\n' "$row" >> 'fast lane.txt'; fast=$((fast+1)) ;;
        send:*|return:*) printf '%s\n' "$row" >> 'normal lane.txt'; normal=$((normal+1)) ;;"#,
            r#"send:*|return:*) printf '%s\n' "$row" >> 'normal lane.txt'; normal=$((normal+1)) ;;
        send:rush:*) printf '%s\n' "$row" >> 'fast lane.txt'; fast=$((fast+1)) ;;"#,
        ),
        ("mode=$1", "mode=save"),
        ("s) short=yes", "s) short=no"),
        ("local prefix=museum", "local prefix=annex"),
        ("apply=no", "apply=yes"),
        ("IFS= read", "read"),
        (r#"grep -F WARN "$1""#, "grep -F WARN missing.log"),
        (
            r#"printf '%s\n' "$report" > "$destination""#,
            r#"printf '%s\n' "$report" > keep.txt"#,
        ),
    ];
    assert_eq!(
        edits.len(),
        labs.iter().filter(|s| s.teaching.is_some()).count()
    );
    for (s, (from, to)) in labs.iter().filter(|s| s.teaching.is_some()).zip(edits) {
        let l = s.teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert_ne!(l.example, s.solution[0]);
        let baseline = shell::run(&l.example, &l.fixture);
        assert!(l.fixture.accepts(&baseline), "{}", s.id);
        assert!(l.example.contains(from));
        let changed = shell::run(&l.example.replacen(from, to, 1), &l.fixture);
        assert!(changed.error.is_none(), "{} {:?}", s.id, changed.error);
        assert_ne!(
            (
                baseline.stdout,
                baseline.stderr,
                baseline.status,
                baseline.files
            ),
            (
                changed.stdout,
                changed.stderr,
                changed.status,
                changed.files
            ),
            "{}",
            s.id
        );
    }
}
#[test]
fn optional_learning_data_preserves_old_attempts_completed_state_and_separate_drafts() {
    let labs = pack::bundled().unwrap();
    let s = &labs[9];
    let l = s.teaching.as_ref().unwrap();
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    let engine::Goal::Script { cases } = &s.goal else {
        panic!()
    };
    let source = s.solution[0].clone();
    a.state = State::Script {
        source: source.clone(),
        run: Some(shell::Record {
            engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
            source: source.clone(),
            results: cases.iter().map(|c| shell::run(&source, c)).collect(),
        }),
    };
    let before = a.state.clone();
    teaching::prepare(&mut a, l, false);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .script_learning
        .as_mut()
        .unwrap();
    learning.view = View::Example;
    learning.example_source.push_str("\nprintf 'changed\\n'");
    learning.example_result = Some(shell::run(&learning.example_source, &l.fixture));
    assert_eq!(a.state, before);
    assert!(engine::solved(&a.state, &s.goal));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let p = progress::load(&path, &labs).unwrap();
    let a = &p.attempts[&s.id];
    assert_eq!(a.state, before);
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .example_source
            .ends_with("printf 'changed\\n'")
    );
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .view,
        View::Example
    );
    // Historical JSON has no optional learning field; deserialization and launch stay safe.
    let mut old = serde_json::to_value(&p).unwrap();
    old["attempts"][&s.id]["workspace"]
        .as_object_mut()
        .unwrap()
        .remove("script_learning");
    let mut old: Progress = serde_json::from_value(old).unwrap();
    let a = old.attempts.get_mut(&s.id).unwrap();
    teaching::prepare(a, l, false);
    assert_eq!(a.state, before);
    assert_eq!(
        a.workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap()
            .view,
        View::Assignment
    );
    let mut fresh = Attempt::new(s, 2);
    workspace::prepare(&mut fresh);
    teaching::prepare(&mut fresh, l, true);
    assert_eq!(
        fresh.workspace.unwrap().script_learning.unwrap().view,
        View::Introduction
    );
}
#[test]
fn feedback_separates_malformed_unsupported_limits_and_output_mistakes() {
    let labs = pack::bundled().unwrap();
    let l = labs[9].teaching.as_ref().unwrap();
    for (source, kind) in [
        ("printf 'unfinished", Diagnostic::Syntax),
        ("if true; then printf ok", Diagnostic::Syntax),
        ("fi", Diagnostic::Syntax),
        ("printf ok >", Diagnostic::Syntax),
        ("case x in", Diagnostic::Syntax),
        ("ls", Diagnostic::Unsupported),
        ("printf '%d' 1", Diagnostic::Unsupported),
        ("while true; do :; done", Diagnostic::Limit),
    ] {
        let r = shell::run(source, &l.fixture);
        let e = r.error.expect(source);
        assert_eq!(teaching::diagnostic(&e), kind, "{source}: {e}");
    }
    for error in [
        "Expansion work limit reached",
        "Expansion memory limit reached",
    ] {
        assert_eq!(teaching::diagnostic(error), Diagnostic::Limit);
        assert!(teaching::diagnostic_advice(error).contains("variable"));
    }
    assert!(teaching::output_advice("hello\n", "hello").contains("final line break"));
    assert!(teaching::output_advice("two  words\n", "two words\n").contains("spacing"));
    let r = shell::run("printf 'wrong\\n'", &l.fixture);
    assert!(r.error.is_none());
    assert!(!l.fixture.accepts(&r));
}
#[test]
fn invalid_teaching_is_rejected_instead_of_shipping_unrunnable_examples() {
    let labs = pack::bundled().unwrap();
    let mut l = labs[9].teaching.clone().unwrap();
    l.example = "printf 'wrong'".into();
    assert!(l.validate().is_err());
    l = labs[9].teaching.clone().unwrap();
    l.concepts.clear();
    assert!(l.validate().is_err());
    let mut learning = teaching::Learning::new(labs[9].teaching.as_ref().unwrap());
    learning.version = 2;
    assert!(!learning.valid());
}
