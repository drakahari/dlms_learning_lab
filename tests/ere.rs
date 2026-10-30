//! Prospective pure foundation checks. No host grep, shell, learner profile or lab credit.
#[path = "../src/ere.rs"]
mod ere;
use ere::{Budget, ErrorKind, Regex};
fn found(pattern: &str, line: &str) -> bool {
    let re = Regex::compile(pattern).unwrap();
    let mut m = re.matcher().unwrap();
    m.matches_line(line, &mut Budget::default()).unwrap()
}
#[test]
fn literals_search_and_branch_anchors_are_distinct() {
    for (p, s, want) in [
        ("ERROR", "before ERROR after", true),
        ("ERROR", "error", false),
        ("^ERROR", "before ERROR", false),
        ("^ERROR", "ERROR later", true),
        ("ERROR$", "before ERROR", true),
        ("ERROR$", "ERROR later", false),
        ("^ERROR$", "ERROR", true),
        ("^ERROR$", "ERROR\r", false),
        ("^$", "", true),
        ("^$", " ", false),
        ("^a|b$", "ax", true),
        ("^a|b$", "xb", true),
        ("^(a|b)$", "ax", false),
        ("^(a|b)$", "b", true),
    ] {
        assert_eq!(found(p, s), want, "{p:?} {s:?}");
    }
}
#[test]
fn escaped_specials_are_literals_not_a_second_syntax_pass() {
    for (p, s) in [
        (r"^a\.b$", "a.b"),
        (r"^\[x\]$", "[x]"),
        (r"^\(a\|b\)$", "(a|b)"),
        (r"^\{2\}\+$", "{2}+"),
        (r"^\^\$\?\*$", "^$?*"),
        (r"^\\$", r"\"),
    ] {
        assert!(found(p, s));
        assert!(!found(p, &format!("{s}x")));
    }
    assert!(found(r"^]$", "]"));
}
#[test]
fn grouping_alternation_and_concatenation_obey_precedence() {
    for (p, yes, no) in [
        ("ab|cd", "cd", "ad"),
        ("a(b|c)d", "acd", "abcd"),
        ("^(WARN|ERROR) [ab]+$", "ERROR abba", "WARN "),
        ("^(ab|a)c$", "ac", "abbc"),
    ] {
        assert!(found(p, yes));
        assert!(!found(p, no));
    }
    for s in ["", "a", "b", "ab", "aab", "abc", "ac", "bc", "zzaczz", "é"] {
        assert_eq!(found("(a|b)c", s), found("ac|bc", s));
        assert_eq!(found("a+", s), found("aa*", s));
        assert_eq!(found("(ab)?", s), found("(ab){0,1}", s));
    }
}
#[test]
fn optional_star_plus_and_finite_open_intervals_have_real_lengths() {
    for (p, s, want) in [
        ("^ab?$", "a", true),
        ("^ab?$", "abb", false),
        ("^ab*$", "a", true),
        ("^ab+$", "a", false),
        ("^ab+$", "abbb", true),
        ("^a{2}$", "aa", true),
        ("^a{2}$", "a", false),
        ("^a{1,3}$", "aaa", true),
        ("^a{1,3}$", "aaaa", false),
        ("^a{2,}$", "aaaa", true),
        ("^a{2,}$", "a", false),
        ("^a{0}$", "", true),
        ("^a{0}$", "a", false),
        ("^a{00,02}$", "aa", true),
    ] {
        assert_eq!(found(p, s), want, "{p} {s}");
    }
    for n in 0..=5 {
        let line = "a".repeat(n);
        assert_eq!(found("^a{2,3}$", &line), (2..=3).contains(&n));
        assert_eq!(found("^a{2,3}$", &line), found("^aaa?$", &line));
    }
}
#[test]
fn bracket_sets_ranges_negation_and_special_first_last_members_work() {
    for (p, s, want) in [
        ("^[a-cx-z]$", "b", true),
        ("^[a-cx-z]$", "d", false),
        ("^[^a-c]$", "z", true),
        ("^[^a-c]$", "b", false),
        ("^[]a]$", "]", true),
        ("^[]a]$", "a", true),
        ("^[-a]$", "-", true),
        ("^[a-]$", "-", true),
        ("^[[]$", "[", true),
        ("^[.]$", ".", true),
        ("^[a^]$", "^", true),
        ("^[^]-]$", "]", false),
        ("^[^]-]$", "-", false),
        ("^[^]-]$", "x", true),
    ] {
        assert_eq!(found(p, s), want, "{p:?} {s:?}");
    }
}
#[test]
fn all_twelve_classes_have_independent_ascii_truth_tables() {
    for name in [
        "alnum", "alpha", "blank", "cntrl", "digit", "graph", "lower", "print", "punct", "space",
        "upper", "xdigit",
    ] {
        let re = Regex::compile(&format!("^[[:{name}:]]$")).unwrap();
        let mut m = re.matcher().unwrap();
        let mut budget = Budget::default();
        for byte in 1u8..=127 {
            if byte == b'\n' {
                continue;
            }
            let line = char::from(byte).to_string();
            let expected = match name {
                "alnum" => {
                    (48..=57).contains(&byte)
                        || (65..=90).contains(&byte)
                        || (97..=122).contains(&byte)
                }
                "alpha" => (65..=90).contains(&byte) || (97..=122).contains(&byte),
                "blank" => [9, 32].contains(&byte),
                "cntrl" => byte < 32 || byte == 127,
                "digit" => (48..=57).contains(&byte),
                "graph" => (33..=126).contains(&byte),
                "lower" => (97..=122).contains(&byte),
                "print" => (32..=126).contains(&byte),
                "punct" => {
                    (33..=47).contains(&byte)
                        || (58..=64).contains(&byte)
                        || (91..=96).contains(&byte)
                        || (123..=126).contains(&byte)
                }
                "space" => [9, 11, 12, 13, 32].contains(&byte),
                "upper" => (65..=90).contains(&byte),
                "xdigit" => {
                    (48..=57).contains(&byte)
                        || (65..=70).contains(&byte)
                        || (97..=102).contains(&byte)
                }
                _ => unreachable!(),
            };
            assert_eq!(
                m.matches_line(&line, &mut budget).unwrap(),
                expected,
                "{name}/{byte}"
            );
        }
    }
}
#[test]
fn c_locale_matches_utf8_bytes_without_slicing_or_claiming_unicode_classes() {
    assert!(!found("^.$", "é"));
    assert!(found("^..$", "é"));
    assert!(found(".", "é"));
    assert!(!found("^[[:alpha:]]+$", "é"));
    assert!(found("[^[:alpha:]]", "é"));
    assert!(!found("^..$", "界"));
    assert!(found("^...$", "界"));
    assert!(found("^...$", "\u{2028}"));
    assert!(found("^[[:space:]]$", "\u{000b}"));
    assert!(found("^[[:space:]]$", "\r"));
    for pattern in ["é", "[é]", "\0", "a\nb"] {
        assert_eq!(
            Regex::compile(pattern).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    let re = Regex::compile(".*").unwrap();
    let mut m = re.matcher().unwrap();
    for line in ["a\n", "\n", "a\0b"] {
        assert_eq!(
            m.matches_line(line, &mut Budget::default())
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
    }
    assert!(m.matches_line("", &mut Budget::default()).unwrap());
}
#[test]
fn bad_and_excluded_patterns_are_diagnosed_not_guessed() {
    for pattern in ["[abc", "(ab", r"abc\", "[z-a]", "a{3,2}", "a{2"] {
        assert_eq!(
            Regex::compile(pattern).unwrap_err().kind,
            ErrorKind::Syntax,
            "{pattern}"
        );
    }
    for pattern in [
        r"(a)\1",
        r"\w+",
        r"\s",
        r"\t",
        r"\q",
        r"\-",
        "(?=a)",
        "(?!a)",
        "(?i)a",
        "(?:a)",
        "a*?",
        "a++",
        "a{1}{2}",
        "a|",
        "|a",
        "()",
        "[[.a.]]",
        "[[=a=]]",
        "[[:word:]]",
        r"[\-]",
        "[a-b-c]",
        "[a-[:digit:]]",
        "[:digit:]",
        "a^b",
        "a$b",
        "^*",
        "a{,2}",
        "a{word}",
        "ab)",
    ] {
        assert_eq!(
            Regex::compile(pattern).unwrap_err().kind,
            ErrorKind::Unsupported,
            "{pattern}"
        );
    }
    assert!(
        Regex::compile("(?=a)")
            .unwrap_err()
            .message
            .contains("Lookaround")
    );
    assert!(
        Regex::compile(r"\1")
            .unwrap_err()
            .message
            .contains("Backreferences")
    );
}
#[test]
fn admission_checks_exact_and_overbound_pattern_nodes_depth_states_and_line_sizes() {
    let exact_pattern = format!("[{}]", "a".repeat(ere::MAX_PATTERN - 2));
    assert!(Regex::compile(&exact_pattern).is_ok());
    assert_eq!(
        Regex::compile(&(exact_pattern + "a")).unwrap_err().kind,
        ErrorKind::Limit
    );
    let exact_nodes = format!("{}?", "a".repeat(128));
    assert!(Regex::compile(&exact_nodes).is_ok());
    assert_eq!(
        Regex::compile(&(exact_nodes + "a")).unwrap_err().kind,
        ErrorKind::Limit
    );
    let exact_depth = format!(
        "{}a{}",
        "(".repeat(ere::MAX_DEPTH),
        ")".repeat(ere::MAX_DEPTH)
    );
    assert!(Regex::compile(&exact_depth).is_ok());
    assert_eq!(
        Regex::compile(&format!("({exact_depth})"))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    let exact_states = format!("{}a{{31}}", "a{32}".repeat(31));
    let re = Regex::compile(&exact_states).unwrap();
    assert_eq!(re.state_count(), ere::MAX_STATES);
    assert!(re.compile_work() <= ere::MAX_COMPILE_VISITS + ere::MAX_PATTERN + ere::MAX_NODES);
    let m = re.matcher().unwrap();
    assert!(re.retained_bytes() + m.scratch_bytes() < 131_072);
    assert_eq!(
        Regex::compile(&"a{32}".repeat(32)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(Regex::compile("a{33}").unwrap_err().kind, ErrorKind::Limit);
    let re = Regex::compile("^x").unwrap();
    let mut m = re.matcher().unwrap();
    assert!(
        m.matches_line(&"x".repeat(ere::MAX_LINE), &mut Budget::default())
            .unwrap()
    );
    assert_eq!(
        m.matches_line(&"x".repeat(ere::MAX_LINE + 1), &mut Budget::default())
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
#[test]
fn nested_nullable_cycles_terminate_with_visited_closures_and_zero_graph_subtrees_fold() {
    for line in ["", "a", "aaaa", "b", "ab"] {
        assert_eq!(found("^((a?)*)*$", line), line.bytes().all(|b| b == b'a'));
    }
    let mut source = "a{0}".to_string();
    for _ in 0..14 {
        source = format!("({source}){{32}}");
    }
    let re = Regex::compile(&source).unwrap();
    assert_eq!(re.state_count(), 1);
    assert!(re.compile_work() < 1000);
    let mut m = re.matcher().unwrap();
    let mut b = Budget::default();
    assert!(
        m.matches_line("discarded repetition bodies", &mut b)
            .unwrap()
    );
    assert!(b.spent() < 100);
    assert_eq!(
        Regex::compile("(a{32}){32}").unwrap_err().kind,
        ErrorKind::Limit
    );
}
#[test]
fn nested_repetitions_do_not_backtrack_and_work_is_linear_in_states_and_input() {
    for pattern in ["^(a+)+$", "^(a|aa)*b$", "^((a?)*)*$"] {
        let re = Regex::compile(pattern).unwrap();
        let mut m = re.matcher().unwrap();
        let line = format!("{}!", "a".repeat(256));
        let mut b = Budget::default();
        assert!(!m.matches_line(&line, &mut b).unwrap());
        assert!(
            b.spent() <= 8 * (line.len() + 1) * (re.state_count() + 1),
            "{pattern}: {}",
            b.spent()
        );
        assert!(b.spent() < 100_000);
    }
}
#[test]
fn one_shared_allowance_includes_validation_and_empty_matches_across_all_lines() {
    let re = Regex::compile("^WARN").unwrap();
    let mut m = re.matcher().unwrap();
    let mut b = Budget::new(80).unwrap();
    assert!(m.matches_line("WARN a", &mut b).unwrap());
    let first = b.spent();
    assert!(first > 0);
    let mut failures = 0;
    for _ in 0..20 {
        match m.matches_line("WARN a", &mut b) {
            Ok(true) => assert!(b.spent() > first),
            Ok(false) => panic!(),
            Err(e) => {
                assert_eq!(e.kind, ErrorKind::Limit);
                failures += 1;
                break;
            }
        }
    }
    assert_eq!(failures, 1);
    assert_eq!(b.remaining(), 0);
    assert_eq!(b.spent(), 80);
    let empty = Regex::compile("").unwrap();
    let mut m = empty.matcher().unwrap();
    assert_eq!(
        m.matches_line("", &mut Budget::new(0).unwrap())
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(m.matches_line("", &mut Budget::default()).unwrap());
    let mut tiny = Budget::new(5).unwrap();
    assert_eq!(
        m.matches_line("sixxxx", &mut tiny).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(tiny.spent(), 5);
    assert_eq!(
        Budget::new(ere::MAX_WORK + 1).unwrap_err().kind,
        ErrorKind::Limit
    );
}
#[test]
fn reused_matcher_scratch_never_grows_and_recovers_after_a_refused_match() {
    let re = Regex::compile("^(a|b){1,4}$").unwrap();
    let mut m = re.matcher().unwrap();
    let bytes = m.scratch_bytes();
    for line in ["a", "b", "abba", "", "abbaa", "é", "abb"] {
        let _ = m.matches_line(line, &mut Budget::default()).unwrap();
        assert_eq!(m.scratch_bytes(), bytes);
    }
    assert!(m.matches_line("a", &mut Budget::new(1).unwrap()).is_err());
    assert!(m.matches_line("abba", &mut Budget::default()).unwrap());
    assert_eq!(m.scratch_bytes(), bytes);
}
#[test]
fn structured_failure_selection_uses_combined_rules_and_preserves_literal_hostile_looking_data() {
    let pattern = r"^(ERROR|CRIT) job=[a-z][a-z0-9-]{0,7} code=[45][0-9]{2}($| )";
    let lines = [
        "INFO job=a code=500",
        "ERROR job=a code=500",
        "CRIT job=a-1 code=404 message=$(do-not-run)",
        "ERROR job=UPPER code=500",
        "ERROR job=bad/path code=500",
        "ERROR job=a code=500x",
        "ERROR job=abcd1234 code=403",
        "ERROR job=abcdefghi code=500",
    ];
    let original = lines;
    let re = Regex::compile(pattern).unwrap();
    let mut m = re.matcher().unwrap();
    let mut b = Budget::default();
    let actual = lines
        .iter()
        .map(|s| m.matches_line(s, &mut b).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [false, true, true, false, false, false, true, false]
    );
    assert_eq!(lines, original);
}
#[test]
fn equivalent_supported_patterns_agree_on_exhaustive_short_binary_inputs() {
    for len in 0..=5 {
        for bits in 0..(1usize << len) {
            let s = (0..len)
                .map(|i| if bits & (1 << i) == 0 { 'a' } else { 'b' })
                .collect::<String>();
            assert_eq!(found("^[ab]+$", &s), !s.is_empty());
            assert_eq!(found("^(a|b)+$", &s), !s.is_empty());
            assert_eq!(found("^a{1,2}b*$", &s), {
                let n = s.bytes().take_while(|b| *b == b'a').count();
                (1..=2).contains(&n) && s[n..].bytes().all(|b| b == b'b')
            });
        }
    }
}

#[test]
fn fixed_reference_recipe_predictions_are_explicit_and_observations_remain_absent() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ere-fixed-reference.json")).unwrap();
    assert_eq!(fixture["schema_version"], 1);
    assert!(fixture["actual_comparison"].is_null());
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 42);
    let mut ids = std::collections::BTreeSet::new();
    let (mut supported, mut excluded) = (0, 0);
    for r in cases {
        assert!(ids.insert(r["id"].as_str().unwrap()));
        assert!(r["vm_observation"].is_null() && r["gnu_observation"].is_null());
        let pattern = r["pattern"].as_str().unwrap();
        let line = r["line"].as_str().unwrap();
        match r["partition"].as_str().unwrap() {
            "supported" => {
                supported += 1;
                assert!(pattern.is_ascii() && !pattern.contains(['\n', '\0']));
                assert!(!line.contains(['\n', '\0']));
                assert_eq!(
                    found(pattern, line),
                    r["expected_match"].as_bool().unwrap(),
                    "{}",
                    r["id"]
                );
            }
            "excluded" => {
                excluded += 1;
                let e = Regex::compile(pattern).unwrap_err();
                assert_eq!(
                    format!("{:?}", e.kind),
                    r["expected_error_kind"].as_str().unwrap()
                );
            }
            _ => panic!("Uncurated reference partition"),
        }
    }
    assert_eq!((supported, excluded), (34, 8));
}

#[test]
fn cooperative_cursor_matches_all_fixed_supported_cases_across_tiny_slices() {
    use std::sync::Arc;
    let fixed: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ere-fixed-reference.json")).unwrap();
    for row in fixed["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["partition"] == "supported")
    {
        let pattern = row["pattern"].as_str().unwrap();
        let line = row["line"].as_str().unwrap();
        let expected = row["expected_match"].as_bool().unwrap();
        for slice in [1, 7, 64] {
            let regex = Regex::compile(pattern).unwrap();
            assert!(Regex::compile_storage_bound() >= regex.retained_bytes());
            assert!(Regex::compile_charge(pattern.len()).unwrap() > 0);
            let admitted = regex.cursor_storage_bound(line.len()).unwrap();
            let mut cursor: ere::Cursor = regex.into_cursor(Arc::<str>::from(line)).unwrap();
            assert_eq!(cursor.input(), line);
            assert!(cursor.retained_bytes() <= admitted);
            cursor.start_line(0..line.len()).unwrap();
            let mut budget = Budget::default();
            let mut done = None;
            for _ in 0..=ere::MAX_WORK {
                let before = budget.spent();
                done = cursor.advance(slice, &mut budget).unwrap();
                assert!(budget.spent() - before <= slice);
                if done.is_some() {
                    break;
                }
            }
            assert_eq!(done, Some(expected), "{} slice{slice}", row["id"]);
        }
    }
}
#[test]
fn cooperative_owned_snapshot_utf8_ranges_and_shared_limits_fail_without_false_matches() {
    use std::sync::Arc;
    let mut original = "é\nok\n".to_string();
    let input: Arc<str> = Arc::from(original.as_str());
    let weak = Arc::downgrade(&input);
    let mut cursor = Regex::compile("^..$")
        .unwrap()
        .into_cursor(input.clone())
        .unwrap();
    original.clear();
    assert_eq!(cursor.input(), "é\nok\n");
    assert!(cursor.start_line(1..2).is_err());
    cursor.start_line(0..2).unwrap();
    assert!(cursor.start_line(3..5).is_err());
    let mut budget = Budget::new(300).unwrap();
    budget.consume(1).unwrap();
    while cursor.advance(1, &mut budget).unwrap().is_none() {}
    let before = budget.spent();
    cursor.start_line(3..5).unwrap();
    while cursor.advance(7, &mut budget).unwrap().is_none() {}
    assert!(budget.spent() > before);
    drop(cursor);
    drop(input);
    assert!(weak.upgrade().is_none());
    for text in ["a\0b", "a\nb"] {
        let mut c = Regex::compile("")
            .unwrap()
            .into_cursor(Arc::<str>::from(text))
            .unwrap();
        c.start_line(0..text.len()).unwrap();
        assert_eq!(
            c.advance(64, &mut Budget::default()).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    let mut c = Regex::compile("((a?)*)*b")
        .unwrap()
        .into_cursor(Arc::<str>::from("aaaaaaaa"))
        .unwrap();
    c.start_line(0..8).unwrap();
    assert_eq!(
        c.advance(64, &mut Budget::new(3).unwrap())
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(Regex::compile_charge(513).is_err());
    assert!(
        Regex::compile(".")
            .unwrap()
            .into_cursor(Arc::<str>::from("x".repeat(262145)))
            .is_err()
    );
}
