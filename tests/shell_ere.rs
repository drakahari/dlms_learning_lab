//! Prospective capability14 outcomes; no host commands or authored-lab credit.
use dlms_learning_lab::shell::{self, Case, Vm};
use std::collections::BTreeMap;
fn case(input: &str, files: &[(&str, &str)]) -> Case {
    Case {
        name: "Fixed ERE capability".into(),
        args: vec![],
        stdin: input.into(),
        env: BTreeMap::new(),
        files: files
            .iter()
            .map(|(p, v)| (p.to_string(), v.to_string()))
            .collect(),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn run(source: &str, input: &str) -> shell::Outcome {
    shell::run(source, &case(input, &[]))
}
fn expect(source: &str, input: &str, stdout: &str, status: i32) {
    let o = run(source, input);
    assert_eq!(o.error, None, "{source}: {:?}", o.error);
    assert_eq!(o.stdout, stdout);
    assert_eq!(o.stderr, "");
    assert_eq!(o.status, status);
    assert!(o.files.is_empty());
}
#[test]
fn grouped_pattern_and_equivalent_flag_orders_select_real_records() {
    for source in [
        "grep -En '^(ERROR|WARN): [0-9]{2}$'",
        "grep -n -E '^(ERROR|WARN): [0-9]{2}$'",
        "grep -nE -- '^(ERROR|WARN): [0-9]{2}$'",
    ] {
        expect(
            source,
            "INFO: 00\nERROR: 42\nWARN: 03\nERROR: nope\n",
            "2:ERROR: 42\n3:WARN: 03\n",
            0,
        );
    }
}
#[test]
fn inverted_numbered_empty_and_unterminated_records_preserve_lf_semantics() {
    expect(
        "grep -Evn '^keep$'",
        "drop\nkeep\nlast",
        "1:drop\n3:last\n",
        0,
    );
    expect("grep -E '^$'", "\nfilled\n", "\n", 0);
    expect("grep -E ''", "last", "last\n", 0);
    expect("grep -E ''", "", "", 1);
    expect("grep -Ev ''", "anything\n", "", 1);
}
#[test]
fn quiet_status_is_independent_of_empty_stdout_and_stops_after_selected_record() {
    expect("grep -Enq '^yes$'", "no\nyes\n", "", 0);
    expect("grep -Eq '^yes$'", "no\n", "", 1);
    expect("grep -Evq '^yes$'", "no\nyes\n", "", 0);
    expect("grep -Eq '^yes$'", "yes\n", "", 0);
    let input = format!("yes\n{}", "x".repeat(65537));
    expect("grep -Eq '^yes$'", &input, "", 0); // Later matching limitation isn't a prior acquisition error.
}
#[test]
fn c_byte_matching_and_utf8_ranges_do_not_confuse_characters_cr_or_unicode_separators() {
    expect("grep -En '^..$'", "é\na\n", "1:é\n", 0);
    expect("grep -E '^.$'", "é\n", "", 1);
    expect("grep -E '^a$'", "a\r\n", "", 1);
    expect("grep -En '^a.b$'", "a\u{2028}b\na b\n", "2:a b\n", 0);
    expect("grep -En '.'", "\r\né\n", "1:\r\n2:é\n", 0);
}
#[test]
fn single_virtual_file_dash_paths_and_finite_stdin_are_defined_without_host_access() {
    let c = case(
        "stdin\n",
        &[("/work/-records", "-bad\ngood\n"), ("/work/-", "dash\n")],
    );
    let o = shell::run("grep -En -- '^-bad$' -records", &c);
    assert_eq!(
        (o.stdout.as_str(), o.stderr.as_str(), o.status, o.error),
        ("1:-bad\n", "", 0, None)
    );
    assert_eq!(o.files, c.files);
    let o = shell::run("grep -E -- '^dash$' ./-", &c);
    assert_eq!(o.stdout, "dash\n");
    assert_eq!(o.error, None);
    expect("grep -E '^stdin$' -", "stdin\n", "stdin\n", 0);
    expect("grep -Eq yes; cat", "yes\nlater\n", "", 0); // Disclosed complete finite-snapshot consumption.
}
#[test]
fn syntax_error_returns_two_and_script_can_handle_it_but_unsupported_syntax_cannot_pass() {
    let o = run("grep -E '['; printf 'status=%s\\n' \"$?\"", "data\n");
    assert_eq!(o.error, None);
    assert_eq!(o.stdout, "status=2\n");
    assert!(o.stderr.starts_with("grep: Syntax ERE:"));
    assert_eq!(o.status, 0);
    for pattern in [r"(a)\1", "(?=x)", r"\w", "a^b"] {
        let o = run(&format!("grep -E '{pattern}'"), "data\n");
        assert!(o.error.as_ref().is_some_and(|e| e.contains("Unsupported")));
        assert_eq!(o.status, 2);
        assert_eq!(o.stdout, "");
    }
}
#[test]
fn missing_file_and_prior_acquisition_error_are_not_hidden_by_quiet_or_empty_pattern() {
    let c = case("data\n", &[]);
    for source in ["grep -E x missing", "grep -Eq '' missing"] {
        let o = shell::run(source, &c);
        assert_eq!(o.error, None);
        assert_eq!(o.status, 2);
        assert_eq!(o.stdout, "");
        assert_eq!(o.stderr, "grep: missing: No such file or directory\n");
        assert_eq!(o.files, c.files);
    }
}
#[test]
fn unsupported_option_and_pattern_forms_fail_instead_of_guessing() {
    for source in [
        "grep -Ei x",
        "grep -E -e x",
        "grep -E x a b",
        "grep -E x --",
        "grep -E x -dash",
        "grep -E -F x",
    ] {
        let o = run(source, "data\n");
        assert!(o.error.is_some(), "{source}");
        assert_eq!(o.status, 2);
        assert_eq!(o.stdout, "");
    }
}
#[test]
fn redirection_pipeline_substitution_and_status_share_existing_execution_rules() {
    let c = case(
        "unused\n",
        &[("/work/input", "bad\nOK: 42\n"), ("/work/output", "old\n")],
    );
    let o = shell::run("grep -En '^OK: [0-9]+$' < input > output; cat output", &c);
    assert_eq!(o.error, None);
    assert_eq!(o.stdout, "2:OK: 42\n");
    assert_eq!(o.files["/work/output"], "2:OK: 42\n");
    assert_eq!(o.files["/work/input"], c.files["/work/input"]);
    expect(
        "printf '%s\\n' WARN INFO | grep -En '^WARN$'",
        "",
        "1:WARN\n",
        0,
    );
    expect(
        "result=$(printf '%s\\n' WARN INFO | grep -E '^WARN$'); printf '<%s>\\n' \"$result\"",
        "",
        "<WARN>\n",
        0,
    );
    let o = shell::run("grep -E '.' input > input", &c);
    assert_eq!(o.error, None);
    assert_eq!(o.status, 1);
    assert_eq!(o.files["/work/input"], "");
}
#[test]
fn fixed_locale_rejects_unmodeled_settings_and_lc_all_has_precedence() {
    let mut c = case("é\n", &[]);
    c.env.insert("LANG".into(), "en_US.UTF-8".into());
    let o = shell::run("grep -E '^.$'", &c);
    assert!(o.error.as_ref().is_some_and(|e| e.contains("locale")));
    c.env.insert("LC_ALL".into(), "C".into());
    let o = shell::run("grep -E '^.$'", &c);
    assert_eq!(o.error, None);
    assert_eq!(o.status, 1);
    c.env.insert("LC_ALL".into(), "POSIX".into());
    let o = shell::run("grep -E '^..$'", &c);
    assert_eq!(o.error, None);
    assert_eq!(o.stdout, "é\n");
}
#[test]
fn stop_drops_large_pending_match_and_prevents_following_publication() {
    let c = case(
        "",
        &[
            ("/work/input", &"x".repeat(20000)),
            ("/work/report", "protected\n"),
        ],
    );
    let mut vm = Vm::new("grep -E '^z$' input; printf 'wrong\\n' > report", &c).unwrap();
    for _ in 0..8 {
        vm.tick(1);
    }
    assert!(!vm.done());
    vm.stop();
    assert!(vm.done());
    assert!(
        vm.outcome
            .error
            .as_ref()
            .is_some_and(|e| e.contains("Stopped"))
    );
    assert_eq!(vm.outcome.files, c.files);
}
#[test]
fn global_regex_budget_cannot_be_reset_by_repeated_calls_or_command_substitution() {
    for (prefix, body) in [
        ("", "grep -Eq '^yes$' < input"),
        ("", "result=$(grep -Eq '^yes$' < input)"),
        ("", "grep -Eq '^yes$' < input | true"),
        (
            "select_rows() { grep -Eq '^yes$' < input; }; ",
            "select_rows",
        ),
    ] {
        let mut c = case("", &[("/work/input", "yes\n")]);
        c.args = (0..64).map(|n| n.to_string()).collect();
        let o = shell::run(
            &format!("{prefix}for item in \"$@\"; do {body}; done; printf 'should not finish\\n'"),
            &c,
        );
        assert!(
            o.error.as_ref().is_some_and(|e| e.contains("work")),
            "{:?}",
            o.error
        );
        assert_eq!(o.status, 2);
        assert_eq!(o.stdout, "");
        assert_eq!(o.files, c.files);
    }
}
#[test]
fn output_line_and_pattern_bounds_fail_honestly_with_no_false_completion() {
    expect(
        "grep -E '.'",
        &"x".repeat(32767),
        &format!("{}\n", "x".repeat(32767)),
        0,
    );
    for (source, input) in [
        ("grep -E '.'".to_string(), "x".repeat(32768)),
        ("grep -E '.'".to_string(), "x".repeat(65537)),
        (format!("grep -E '{}'", "x".repeat(513)), "x\n".to_string()),
    ] {
        let o = run(&source, &input);
        assert!(o.error.is_some());
        assert_eq!(o.status, 2);
    }
}
#[test]
fn historical_literal_f_behavior_is_preserved_and_new_outputs_remain_state_driven() {
    expect("grep -F 'a.b'", "a.b\naxb\n", "a.b\n", 0);
    let o = run("grep -F '-E'", "-E\n");
    assert!(
        o.error
            .as_ref()
            .is_some_and(|e| e.contains("pattern starting with -"))
    );
    let o = run("grep -E '^secret$'", "public\n");
    assert_eq!(o.error, None);
    assert_eq!(o.stdout, "");
    assert_eq!(o.status, 1);
    expect("grep -E '^secret$'", "secret\n", "secret\n", 0);
}

#[test]
fn fixed_utility_recipes_compare_full_simulated_outcomes_without_claiming_host_observations() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/shell-ere-reference.json")).unwrap();
    assert_eq!(fixture["schema_version"], 1);
    assert!(fixture["host_execution"].is_null() && fixture["source_identity"].is_null());
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 24);
    let mut ids = std::collections::BTreeSet::new();
    for row in rows {
        assert!(ids.insert(row["id"].as_str().unwrap()));
        assert!(row["observed"].is_null());
        let mut c = case(row["stdin"].as_str().unwrap(), &[]);
        for (name, value) in row["files"].as_object().unwrap() {
            assert!(!name.contains('/') && name != "." && name != "..");
            c.files
                .insert(format!("/work/{name}"), value.as_str().unwrap().into());
        }
        let outcome = shell::run(row["vm_source"].as_str().unwrap(), &c);
        assert_eq!(outcome.error, None, "{}: {:?}", row["id"], outcome.error);
        assert_eq!(
            outcome.stdout,
            row["expected"]["stdout"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        assert_eq!(
            outcome.stderr,
            row["expected"]["stderr"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        assert_eq!(
            outcome.status,
            row["expected"]["status"].as_i64().unwrap() as i32,
            "{}",
            row["id"]
        );
        assert_eq!(outcome.files, c.files, "{}", row["id"]);
    }
    assert_eq!(fixture["vm_only"].as_array().unwrap().len(), 6);
}

#[test]
fn flat_dot_prefix_file_operands_keep_stdin_distinct_and_cannot_traverse() {
    let c = case(
        "input\n",
        &[
            ("/work/-", "dash\n"),
            ("/work/records", "alpha\n"),
            ("/work/keep", "protected\n"),
        ],
    );
    for (source, expected) in [
        ("grep -E '^alpha$' ./records; cat", "alpha\ninput\n"),
        ("grep -E -- '^dash$' ./-; cat", "dash\ninput\n"),
        ("grep -E -- '^dash$' /work/-; cat", "dash\ninput\n"),
        ("grep -E '^input$' -; cat", "input\n"),
    ] {
        let o = shell::run(source, &c);
        assert_eq!(
            (
                o.stdout.as_str(),
                o.stderr.as_str(),
                o.status,
                o.error.as_ref()
            ),
            (expected, "", 0, None),
            "{source}"
        );
        assert_eq!(
            o.files, c.files,
            "Reading a file or stdin must preserve all virtual files"
        );
    }
    for operand in [
        "./../records",
        "./dir/records",
        "././records",
        "./",
        "./..",
        "/work/../records",
    ] {
        let o = shell::run(&format!("grep -E alpha {operand}"), &c);
        assert!(
            o.error
                .as_ref()
                .is_some_and(|e| e.contains("Only ordinary virtual paths")),
            "{operand}: {:?}",
            o.error
        );
        assert_eq!(o.status, 2);
        assert_eq!(o.stdout, "");
        assert_eq!(o.files, c.files);
    }
    let old_literal = shell::run("grep -F alpha ./records", &c);
    assert!(
        old_literal.error.is_some(),
        "The historical literal-F path subset is unchanged"
    );
    assert_eq!(old_literal.files, c.files);
}
