use dlms_learning_lab::shell::{self, Case, Vm};
use std::collections::BTreeMap;
fn case(names: &[&str]) -> Case {
    Case {
        name: "Virtual filename matching".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: names
            .iter()
            .map(|n| (format!("/work/{n}"), format!("{n}\n")))
            .collect(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn expect(c: &Case, source: &str, output: &str) {
    let result = shell::run(source, c);
    assert_eq!(result.error, None, "{source}: {:?}", result.error);
    assert_eq!(result.status, 0, "{source}: {}", result.stderr);
    assert_eq!(result.stdout, output, "{source}");
}
fn error(c: &Case, source: &str, reason: &str) {
    let r = shell::run(source, c);
    assert!(
        r.error.as_ref().is_some_and(|s| s
            .to_ascii_lowercase()
            .contains(&reason.to_ascii_lowercase())),
        "{source}: {:?}",
        r.error
    );
    assert_eq!(r.status, 2);
    assert_eq!(r.files, c.files, "failure must precede file mutation");
}
#[test]
fn matches_are_sorted_and_retain_path_style_with_default_no_match_and_hidden_rules() {
    let c = case(&[
        "z.log",
        "a.log",
        ".hidden.log",
        "two words.log",
        "notes.txt",
    ]);
    expect(
        &c,
        "printf '<%s>\\n' *.log",
        "<a.log>\n<two words.log>\n<z.log>\n",
    );
    expect(
        &c,
        "printf '<%s>\\n' /work/*.log",
        "</work/a.log>\n</work/two words.log>\n</work/z.log>\n",
    );
    expect(&c, "printf '<%s>\\n' .*.log", "<.hidden.log>\n");
    expect(&c, "printf '<%s>\\n' no*.csv", "<no*.csv>\n");
    expect(&case(&[]), "printf '<%s>\\n' *", "<*>\n");
}
#[test]
fn quotes_escapes_mixed_parts_splitting_and_argument_forwarding_keep_provenance() {
    let mut c = case(&[
        "a.log",
        "a.txt",
        "x*.log",
        "x?.log",
        "xA.log",
        "two words.log",
    ]);
    expect(
        &c,
        "printf '<%s>\\n' '*.log' \"*.log\" \\*.log",
        "<*.log>\n<*.log>\n<*.log>\n",
    );
    expect(
        &c,
        "printf '<%s>\\n' 'x*'*.log x\\?*.log",
        "<x*.log>\n<x?.log>\n",
    );
    expect(
        &c,
        "prefix=x; printf '<%s>\\n' \"$prefix\"*.log",
        "<x*.log>\n<x?.log>\n<xA.log>\n",
    );
    c.env.insert("patterns".into(), "a.* x?.log".into());
    expect(
        &c,
        "printf '<%s>\\n' $patterns",
        "<a.log>\n<a.txt>\n<x*.log>\n<x?.log>\n<xA.log>\n",
    );
    expect(&c, "printf '<%s>\\n' \"$patterns\"", "<a.* x?.log>\n");
    c.args = vec!["*.log".into(), "two words.log".into(), String::new()];
    expect(
        &c,
        "f() { printf '<%s>\\n' \"$@\"; }; f \"$@\"",
        "<*.log>\n<two words.log>\n<>\n",
    );
}
#[test]
fn assignments_declarations_loops_pipelines_and_redirections_share_correct_contexts() {
    let c = case(&["b.log", "a.log", "one.txt"]);
    expect(
        &c,
        "pattern=*.log; printf '%s\\n' \"$pattern\"; for file in $pattern; do printf '%s\\n' \"$file\"; done",
        "*.log\na.log\nb.log\n",
    );
    expect(
        &c,
        "f() { local pattern=*.log; printf '%s\\n' \"$pattern\"; }; f",
        "*.log\n",
    );
    expect(&c, "printf '%s\\n' *.log | sort -r", "b.log\na.log\n");
    expect(
        &c,
        "read -r row < *.txt; printf '%s\\n' \"$row\"",
        "one.txt\n",
    );
    expect(
        &c,
        "while read -r row; do printf '%s\\n' \"$row\"; done < *.txt",
        "one.txt\n",
    );
    error(&c, "printf overwritten > *.log", "Ambiguous redirection");
    error(
        &c,
        "while false; do :; done < *.log",
        "Ambiguous loop input",
    );
    let r = shell::run("printf 'new\\n' > new.log; printf '%s\\n' *.log", &c);
    assert_eq!(r.error, None);
    assert_eq!(r.stdout, "a.log\nb.log\nnew.log\n");
    assert_eq!(r.files["/work/new.log"], "new\n");
}
#[test]
fn question_mark_counts_c_locale_bytes_and_literal_multibyte_names_remain_intact() {
    let c = case(&["é.txt", "☃.txt", "a.txt", "界.txt"]);
    expect(&c, "printf '%s\\n' ?.txt", "a.txt\n");
    expect(&c, "printf '%s\\n' ??.txt", "é.txt\n");
    expect(&c, "printf '%s\\n' ???.txt", "☃.txt\n界.txt\n");
    expect(&c, "printf '%s\\n' '☃'*.txt", "☃.txt\n");
}
#[test]
fn unsupported_patterns_paths_and_locales_fail_honestly_before_writes() {
    let c = case(&["a.log", "a.txt", "[a].log"]);
    for source in [
        "printf '%s' [ab].log",
        "printf '%s' file[0-9]",
        "printf '%s' **.log",
        "pattern='a\\*'; printf '%s' $pattern",
    ] {
        error(&c, source, "Unsupported");
    }
    for source in [
        "printf '%s' /etc/*",
        "printf '%s' ../*",
        "printf '%s' /work/../*",
        "printf '%s' sub/*",
    ] {
        error(&c, source, "flat virtual");
    }
    for source in [
        "LC_ALL=en_US.UTF-8; printf '%s' *.log",
        "LANG=en_US.UTF-8; printf '%s' *.log",
    ] {
        error(&c, source, "locale");
    }
    expect(&c, "LANG=other; LC_ALL=C; printf '%s\\n' *.txt", "a.txt\n");
    expect(&c, "printf '%s\\n' '[ab].log'", "[ab].log\n");
    expect(&c, "[ x = x ]; printf '%s\\n' \"$?\"", "0\n");
}
// A small independent dynamic-programming matcher provides a non-greedy test oracle.
fn dp(pattern: &[u8], name: &[u8]) -> bool {
    let mut row = vec![false; name.len() + 1];
    row[0] = true;
    for &p in pattern {
        let mut next = vec![false; row.len()];
        if p == b'*' {
            next[0] = row[0];
        }
        for n in 1..row.len() {
            next[n] = match p {
                b'*' => row[n] || next[n - 1],
                b'?' => row[n - 1],
                _ => row[n - 1] && name[n - 1] == p,
            };
        }
        row = next;
    }
    row[name.len()]
}
fn words(alphabet: &[u8], max: usize) -> Vec<String> {
    let mut all = vec![String::new()];
    let mut level = vec![String::new()];
    for _ in 0..max {
        let mut next = vec![];
        for prefix in level {
            for b in alphabet {
                next.push(format!("{prefix}{}", *b as char));
            }
        }
        all.extend(next.clone());
        level = next;
    }
    all
}
#[test]
fn iterative_matching_agrees_with_independent_dp_across_short_pattern_space() {
    let names = words(b"ab", 3);
    let names: Vec<_> = names
        .iter()
        .filter(|s| !s.is_empty())
        .map(String::as_str)
        .collect();
    let c = case(&names);
    for pattern in words(b"ab*?", 4)
        .into_iter()
        .filter(|p| !p.is_empty() && !p.contains("**"))
    {
        let mut matched: Vec<_> = names
            .iter()
            .filter(|n| dp(pattern.as_bytes(), n.as_bytes()))
            .copied()
            .collect();
        matched.sort();
        let expected = if matched.is_empty() {
            format!("<{pattern}>\n")
        } else {
            matched.into_iter().map(|n| format!("<{n}>\n")).collect()
        };
        expect(&c, &format!("printf '<%s>\\n' {pattern}"), &expected);
    }
}
#[test]
fn match_work_and_fields_are_bounded_across_ticks_and_stop_remains_available() {
    let names: Vec<_> = (0..64)
        .map(|i| format!("{}-{i:02}.txt", "a".repeat(200)))
        .collect();
    let c = case(&names.iter().map(String::as_str).collect::<Vec<_>>());
    let source = "while true; do test -n *a*a*a*a*a*a*a*a*a*a*b; done";
    let r = shell::run(source, &c);
    assert!(
        r.error
            .as_ref()
            .is_some_and(|e| e.contains("Expansion work limit")),
        "{:?}",
        r.error
    );
    for slice in [1, 16, 64] {
        let mut vm = Vm::new(source, &c).unwrap();
        for _ in 0..2000 {
            vm.tick(slice);
            if vm.done() {
                break;
            }
        }
        assert!(vm.done());
        assert_eq!(vm.outcome, r);
    }
    let r = shell::run("printf '%s' * * *", &c);
    assert!(
        r.error
            .as_ref()
            .is_some_and(|e| e.contains("Argument limit"))
    );
    let mut vm = Vm::new("while true; do :; done", &c).unwrap();
    vm.tick(1);
    vm.stop();
    assert!(vm.done());
    assert!(vm.outcome.error.as_ref().unwrap().contains("Stopped"));
}

#[test]
fn actual_revision4_bracket_value_credit_survives_current_rejection_and_repair() {
    use dlms_learning_lab::{
        backup,
        engine::{Goal, State},
        pack,
        progress::{self, Attempt, Progress},
        workspace,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    // Verified with a clean 44c1f40 shell-only probe, not invented prior mastery.
    let old: shell::Record = serde_json::from_str(r#"{"engine_revision":4,"source":"pattern='[ab].log'; printf '%s\\n' $pattern","results":[{"stdout":"[ab].log\n","stderr":"","status":0,"files":{"/work/a.log":"a\n"},"error":null}]}"#).unwrap();
    assert!(old.valid());
    let mut c = case(&["a.log"]);
    c.files.insert("/work/a.log".into(), "a\n".into());
    c.after = c.files.clone();
    c.stdout = "[ab].log\n".into();
    assert!(c.accepts(&old.results[0]));
    let mut lab = pack::bundled().unwrap()[9].clone();
    lab.id = "test-filename-upgrade".into();
    lab.teaching = None;
    lab.unit.clear();
    lab.prerequisites.clear();
    lab.goal = Goal::Script {
        cases: vec![c.clone()],
    };
    let mut a = Attempt::new(&lab, 1);
    a.state = State::Script {
        source: old.source.clone(),
        run: Some(old.clone()),
    };
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(lab.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    let original = serde_json::to_vec_pretty(&p).unwrap();
    std::fs::write(&path, &original).unwrap();
    let mut p = progress::load(&path, std::slice::from_ref(&lab)).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let a = p.attempts.get_mut(&lab.id).unwrap();
    workspace::prepare(a);
    assert!(progress::legacy_script_credit(a, &lab));
    assert!(!progress::current_solved(a, &lab));
    let State::Script { source, run } = &mut a.state else {
        panic!()
    };
    let r = shell::run(source, &c);
    assert!(r.error.as_ref().unwrap().contains("Unsupported bracket"));
    *run = Some(shell::Record {
        engine_revision: shell::EVALUATOR_REVISION,
        source: source.clone(),
        results: vec![r],
    });
    assert!(progress::legacy_script_credit(a, &lab));
    assert!(!progress::current_solved(a, &lab));
    progress::save(&path, &p).unwrap();
    let copy = path.with_extension(format!("before-shell-v{}.json", shell::EVALUATOR_REVISION));
    assert_eq!(std::fs::read(&copy).unwrap(), original);
    let export = dir.path().join("export.json");
    backup::export(&export, &p, std::slice::from_ref(&lab)).unwrap();
    let reviewed = backup::inspect(
        &export,
        std::slice::from_ref(&lab),
        &AtomicBool::new(false),
        &AtomicUsize::new(0),
    )
    .unwrap();
    assert!(progress::legacy_script_credit(
        &reviewed.progress.attempts[&lab.id],
        &lab
    ));
    let repair = "pattern='[ab].log'; printf '%s\\n' \"$pattern\"";
    let a = p.attempts.get_mut(&lab.id).unwrap();
    a.state = State::Script {
        source: repair.into(),
        run: Some(shell::Record {
            engine_revision: shell::EVALUATOR_REVISION,
            source: repair.into(),
            results: vec![shell::run(repair, &c)],
        }),
    };
    assert!(progress::current_solved(a, &lab));
    assert!(progress::legacy_script_credit(a, &lab));
    progress::save(&path, &p).unwrap();
    p.attempts.insert(lab.id.clone(), Attempt::new(&lab, 2));
    progress::save(&path, &p).unwrap();
    assert_eq!(std::fs::read(&copy).unwrap(), original);
}
