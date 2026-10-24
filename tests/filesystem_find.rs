use dlms_learning_lab::{
    engine::Node,
    filesystem::{FindDiagnosticKind, Tree},
};
use std::collections::BTreeMap;

fn node(directory: bool, mode: u16) -> Node {
    Node {
        directory,
        mode,
        owner: "learner".into(),
        group: "staff".into(),
    }
}
fn tree() -> Tree {
    Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(true, 0o755)),
            ("/work".into(), node(true, 0o755)),
        ]),
        files: BTreeMap::new(),
        users: BTreeMap::from([("learner".into(), vec!["staff".into()])]),
        user: "learner".into(),
        home: "/work".into(),
        cwd: "/work".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
    }
}
fn add(t: &mut Tree, path: &str, directory: bool, mode: u16) {
    t.nodes.insert(path.into(), node(directory, mode));
    if !directory {
        t.files.insert(
            path.into(),
            "Not a search result: contents stay unread.\n".into(),
        );
    }
}
fn args(text: &[&str]) -> Vec<String> {
    text.iter().map(|s| (*s).into()).collect()
}

#[test]
fn find_preorder_hidden_type_and_conjunction_are_actual_tree_queries() {
    let mut t = tree();
    for (path, dir) in [
        ("/work/z.log", false),
        ("/work/a", true),
        ("/work/a/report.log", false),
        ("/work/.hidden.log", false),
        ("/work/a/note.txt", false),
    ] {
        add(&mut t, path, dir, 0o755);
    }
    let before = t.clone();
    assert_eq!(
        t.execute("find .").unwrap(),
        ".\n./.hidden.log\n./a\n./a/note.txt\n./a/report.log\n./z.log\n"
    );
    assert_eq!(
        t.execute("find . -type f -name '*.log' -print").unwrap(),
        "./.hidden.log\n./a/report.log\n./z.log\n"
    );
    assert_eq!(
        t.execute("find . -name '*.log' -type f -name 'report*'")
            .unwrap(),
        "./a/report.log\n"
    );
    assert_eq!(t.execute("find . -type f -type d").unwrap(), "");
    assert_eq!(t.execute("find . -name ''").unwrap(), "");
    assert_eq!(t, before);
}

#[test]
fn find_preserves_entered_start_spelling_and_starting_basename() {
    let mut t = tree();
    add(&mut t, "/work/a", true, 0o755);
    add(&mut t, "/work/a/x", false, 0o600);
    for (command, expected) in [
        ("find ./a/", "./a/\n./a/x\n"),
        ("find /work//a///", "/work//a///\n/work//a///x\n"),
        ("find a/.. -maxdepth 0 -name '..'", "a/..\n"),
        ("find . -maxdepth 0 -name '.'", ".\n"),
        ("find / -maxdepth 0 -name '/'", "/\n"),
        ("find a/x", "a/x\n"),
    ] {
        assert_eq!(t.execute(command).unwrap(), expected, "{command}");
    }
    assert!(t.execute("find a/x/").is_err());
}

#[test]
fn depth_limits_skip_descent_checks_but_predicate_failure_does_not_prune() {
    let mut t = tree();
    add(&mut t, "/work/locked", true, 0o000);
    add(&mut t, "/work/locked/secret.log", false, 0o000);
    add(&mut t, "/work/open", true, 0o755);
    add(&mut t, "/work/open/report.log", false, 0o000);
    assert_eq!(t.execute("find locked -maxdepth 0").unwrap(), "locked\n");
    assert_eq!(t.execute("find locked -maxdepth 0 -type f").unwrap(), "");
    assert_eq!(
        t.execute("find . -maxdepth 1 -type d").unwrap(),
        ".\n./locked\n./open\n"
    );
    assert_eq!(t.execute("find open -type f").unwrap(), "open/report.log\n");
    assert_eq!(
        t.execute("find open -maxdepth 4294967295 -type f").unwrap(),
        "open/report.log\n"
    );
    assert_eq!(
        t.execute("find open -maxdepth 0000000000").unwrap(),
        "open\n"
    );
}

#[test]
fn denied_read_and_unsupported_search_report_healthy_partial_paths_honestly() {
    let mut t = tree();
    for (path, mode) in [
        ("/work/a-denied", 0o100),
        ("/work/b-readable", 0o400),
        ("/work/c-good", 0o755),
    ] {
        add(&mut t, path, true, mode);
        add(&mut t, &format!("{path}/item.log"), false, 0o000);
    }
    let before = t.clone();
    let report = t.find(&args(&[".", "-type", "f"])).unwrap();
    assert!(!report.complete());
    assert_eq!(report.paths(), &["./c-good/item.log"]);
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report.diagnostics()[0].kind,
        FindDiagnosticKind::PermissionDenied
    );
    assert_eq!(
        report.diagnostics()[1].kind,
        FindDiagnosticKind::UnsupportedTraversal
    );
    let error = t.execute("find . -type f").unwrap_err();
    assert!(error.contains("Search incomplete") && error.contains("./c-good/item.log"));
    assert!(!error.contains("Not a search result"));
    assert!(!error.contains("./a-denied/item.log") && !error.contains("./b-readable/item.log"));
    assert_eq!(t, before);
}

#[test]
fn name_and_type_never_require_file_read_or_use_search_as_read() {
    let mut t = tree();
    add(&mut t, "/work/unreadable", false, 0o000);
    assert!(t.execute("cat unreadable").is_err());
    assert_eq!(
        t.execute("find unreadable -type f").unwrap(),
        "unreadable\n"
    );
    add(&mut t, "/work/no-read", true, 0o100);
    add(&mut t, "/work/no-read/item", false, 0o000);
    assert_eq!(
        t.execute("find no-read/item -type f").unwrap(),
        "no-read/item\n"
    );
    assert!(
        t.execute("find no-read -type f")
            .unwrap_err()
            .contains("Permission denied")
    );
    t.nodes.get_mut("/work").unwrap().mode = 0o000;
    // Absolute traversal cannot normalize away a denied component.
    assert!(t.execute("find /work/../work/unreadable").is_err());
    t.cwd = "/work/no-read".into();
    // The already-open cwd does not require searching its inaccessible parent.
    assert_eq!(t.execute("find item -type f").unwrap(), "item\n");
}

#[test]
fn quotes_protect_the_shell_pattern_then_find_matches_c_bytes_and_escapes() {
    let mut t = tree();
    for name in ["*", "?", "[x]", "\\", ".hidden", "two words 100%", "é", "a"] {
        add(&mut t, &format!("/work/{name}"), false, 0o000);
    }
    for (command, expected) in [
        (r"find . -type f -name '\*'", "./*\n"),
        (r"find . -type f -name '\?'", "./?\n"),
        (r"find . -type f -name '\[x\]'", "./[x]\n"),
        (r"find . -type f -name '\\'", "./\\\n"),
        ("find . -name 'two words 100%'", "./two words 100%\n"),
        ("find . -name 'é'", "./é\n"),
        ("find . -type f -name '??'", "./é\n"),
    ] {
        assert_eq!(t.execute(command).unwrap(), expected, "{command}");
    }
    assert!(!t.execute("find . -type f -name '?'").unwrap().contains("é"));
    assert!(
        t.execute("find . -type f -name '*'")
            .unwrap()
            .contains("./.hidden\n")
    );
    let before = t.clone();
    assert!(
        t.execute("find . -name *")
            .unwrap_err()
            .contains("Filename expansion is unsupported")
    );
    assert_eq!(t, before);
}

#[test]
fn unsupported_or_malformed_grammar_fails_before_any_partial_traversal() {
    let mut t = tree();
    add(&mut t, "/work/denied", true, 0o000);
    let before = t.clone();
    for command in [
        "find",
        "find -- .",
        "find . /",
        "find . -name",
        "find . -type",
        "find . -type l",
        "find . -type f,d",
        "find . -maxdepth",
        "find . -maxdepth -1",
        "find . -maxdepth +1",
        "find . -maxdepth 4294967296",
        "find . -maxdepth 00000000000",
        "find . -maxdepth 1 -maxdepth 2",
        "find . -type f -maxdepth 1",
        "find . -print -type f",
        "find . -print -print",
        "find . -name '[a]'",
        "find . -name ']'",
        r"find . -name '\'",
        "find . -exec 'whoami' '{}' ';'",
        "find . -delete",
        "find . -o -print",
        "find . -a -print",
        "find . '!' -type f",
        "find . '(' -type f ')'",
        "find . -print0",
        "find . -size 1",
        "find . -mtime 0",
        "find . -prune",
        "find -L .",
        "find . -name x | cat x",
    ] {
        let error = t.execute(command).unwrap_err();
        assert!(
            !error.contains("Partial matches"),
            "syntax must be validated before traversal: {command}: {error}"
        );
        assert_eq!(t, before, "{command}");
    }
    add(&mut t, "/work/-name", false, 0o600);
    assert_eq!(t.execute("find ./-name").unwrap(), "./-name\n");
}

#[test]
fn direct_call_limits_and_invalid_tree_are_checked_before_search_work() {
    let t = tree();
    for a in [
        vec!["x".repeat(4097)],
        vec![".".into(); 39],
        args(&[".", "-name", &"a".repeat(256)]),
    ] {
        assert!(t.find(&a).is_err());
    }
    let mut a = args(&["."]);
    for _ in 0..17 {
        a.extend(args(&["-type", "d"]));
    }
    assert!(t.find(&a).unwrap_err().contains("at most16"));
    let mut invalid = t.clone();
    invalid.nodes.remove("/");
    assert!(
        invalid
            .find(&args(&["."]))
            .unwrap_err()
            .contains("Invalid virtual filesystem")
    );
}

#[test]
fn output_limit_is_incomplete_without_mutation_or_silent_truncation() {
    let mut t = tree();
    for n in 0..200 {
        add(
            &mut t,
            &format!("/work/{}{:03}", "x".repeat(230), n),
            false,
            0o600,
        );
    }
    assert!(t.valid());
    let before = t.clone();
    let report = t.find(&args(&[".", "-type", "f"])).unwrap();
    assert!(!report.complete());
    assert!(report.paths().len() < 200 && !report.paths().is_empty());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.kind == FindDiagnosticKind::OutputLimit)
    );
    let error = t.execute("find . -type f").unwrap_err();
    assert!(error.len() <= 32_768 && error.contains("Search incomplete"));
    assert_eq!(t, before);
    assert_eq!(t.execute("find . -maxdepth 0").unwrap(), ".\n");
}

#[test]
fn effort_ceiling_reports_partial_search_and_leaves_every_field_exact() {
    let mut t = tree();
    for n in 0..40 {
        add(
            &mut t,
            &format!("/work/{}{:03}", "a".repeat(197), n),
            false,
            0o600,
        );
    }
    let before = serde_json::to_vec(&t).unwrap();
    let report = t
        .find(&args(&[".", "-type", "f", "-name", &"?".repeat(200)]))
        .unwrap();
    assert!(!report.complete());
    assert!(!report.paths().is_empty() && report.paths().len() < 40);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.kind == FindDiagnosticKind::WorkLimit)
    );
    assert_eq!(serde_json::to_vec(&t).unwrap(), before);
}

#[test]
fn diagnostic_count_and_utf8_byte_caps_keep_long_failures_small_and_state_exact() {
    let mut t = tree();
    let mut path = "/work".to_string();
    for _ in 0..3 {
        path.push('/');
        path.push_str(&"é".repeat(100));
        add(&mut t, &path, true, 0o755);
    }
    for n in 0..25 {
        add(&mut t, &format!("{path}/d{n:02}"), true, 0o000);
    }
    assert!(t.valid());
    let before = t.clone();
    let report = t.find(&args(&[&path, "-type", "f"])).unwrap();
    assert!(!report.complete());
    assert_eq!(report.diagnostics().len(), 16);
    assert_eq!(report.suppressed_diagnostics(), 9);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.message().len() <= 512)
    );
    let error = t.execute(&format!("find '{path}' -type f")).unwrap_err();
    assert!(error.len() <= 32_768 && error.contains("9 additional"));
    assert_eq!(t, before);
}

// Independent recursive oracle, deliberately unlike production's iterative DP.
fn oracle(pattern: &[u8], subject: &[u8]) -> bool {
    match pattern.first().copied() {
        None => subject.is_empty(),
        Some(b'*') => (0..=subject.len()).any(|n| oracle(&pattern[1..], &subject[n..])),
        Some(b'?') => !subject.is_empty() && oracle(&pattern[1..], &subject[1..]),
        Some(b'\\') => pattern.get(1).is_some_and(|literal| {
            subject.first() == Some(literal) && oracle(&pattern[2..], &subject[1..])
        }),
        Some(literal) => subject.first() == Some(&literal) && oracle(&pattern[1..], &subject[1..]),
    }
}
fn words(alphabet: &[char], depth: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    for _ in 0..depth {
        let layer: Vec<_> = out
            .iter()
            .filter(|s| s.len() == out.iter().map(String::len).max().unwrap())
            .flat_map(|s| alphabet.iter().map(move |c| format!("{s}{c}")))
            .collect();
        out.extend(layer);
    }
    out
}
#[test]
fn short_exhaustive_simple_patterns_agree_with_independent_recursive_matching() {
    let mut t = tree();
    let mut names = words(&['a', 'b'], 4);
    names.retain(|s| !s.is_empty());
    names.extend(["*", "?", "[", "]", "\\", ".a", "é"].map(String::from));
    for name in &names {
        add(&mut t, &format!("/work/{name}"), false, 0o000);
    }
    let mut patterns = words(&['a', 'b', '*', '?'], 4);
    patterns.extend([r"\*", r"\?", r"\[", r"\]", r"\\", "??"].map(String::from));
    for pattern in patterns {
        let report = t
            .find(&args(&[".", "-type", "f", "-name", &pattern]))
            .unwrap();
        assert!(report.complete());
        let mut expected: Vec<_> = names
            .iter()
            .filter(|name| oracle(pattern.as_bytes(), name.as_bytes()))
            .map(|name| format!("./{name}"))
            .collect();
        expected.sort();
        assert_eq!(report.paths(), expected, "pattern={pattern:?}");
    }
}
