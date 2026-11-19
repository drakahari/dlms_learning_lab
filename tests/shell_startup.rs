use dlms_learning_lab::shell::{
    self,
    startup::{File, Files, Invocation, Run, select},
};
use std::collections::BTreeMap;
fn files(rows: &[(&str, &str)]) -> Files {
    Files(
        rows.iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    File {
                        text: v.to_string(),
                    },
                )
            })
            .collect(),
    )
}
fn input() -> shell::Case {
    shell::Case {
        name: "Disposable virtual startup sample".into(),
        args: vec![],
        stdin: String::new(),
        env: BTreeMap::new(),
        files: BTreeMap::from([("/work/keep.txt".into(), "Keep these bytes.\n".into())]),
        stdout: String::new(),
        stderr: String::new(),
        status: 0,
        after: BTreeMap::new(),
    }
}
fn finish(run: &mut Run) {
    for _ in 0..1000 {
        run.tick();
        if run.done() {
            return;
        }
    }
    panic!("Startup exceeded its ordinary bounded phases")
}
fn paths(mode: &Invocation, f: &Files) -> Vec<String> {
    select(mode, f)
        .unwrap()
        .into_iter()
        .map(|s| s.path)
        .collect()
}

#[test]
fn login_uses_system_then_first_present_profile_even_when_empty_and_rc_is_separate() {
    let mut f = files(&[
        ("/etc/profile", ":"),
        ("~/.bash_profile", ""),
        ("~/.bash_login", "printf wrong"),
        ("~/.profile", "printf wrong"),
        ("~/.bashrc", "printf rc"),
    ]);
    let login = Invocation::InteractiveLogin { noprofile: false };
    assert_eq!(paths(&login, &f), ["/etc/profile", "~/.bash_profile"]);
    f.0.remove("~/.bash_profile");
    assert_eq!(paths(&login, &f), ["/etc/profile", "~/.bash_login"]);
    f.0.remove("~/.bash_login");
    assert_eq!(paths(&login, &f), ["/etc/profile", "~/.profile"]);
    assert!(paths(&Invocation::InteractiveLogin { noprofile: true }, &f).is_empty());
    assert_eq!(
        paths(&Invocation::InteractiveNonLogin { norc: false }, &f),
        ["~/.bashrc"]
    );
    assert!(paths(&Invocation::InteractiveNonLogin { norc: true }, &f).is_empty());
    assert!(paths(&Invocation::NonInteractive { bash_env: None }, &f).is_empty());
}
#[test]
fn actual_variables_export_attributes_status_and_output_carry_across_distinct_files() {
    let f = files(&[
        (
            "/etc/profile",
            "BASE='system default'\nexport BASE\nexport PENDING\nfalse\n",
        ),
        (
            "~/.bash_profile",
            "SEEN=$?\nexport MODE=\"$SOURCE\"\nPRIVATE=\"$SOURCE\"\nPENDING=ready\n",
        ),
        ("~/.profile", "printf unwanted"),
    ]);
    for value in ["oak", "a name with spaces", "literal $HOME * [x] ; data"] {
        let mut c = input();
        c.env.insert("SOURCE".into(), value.into());
        let mut r = Run::new(
            &Invocation::InteractiveLogin { noprofile: false },
            &f,
            &c,
            "printf '%s|%s|%s\\n' \"$SEEN\" \"$BASE\" \"$MODE\"",
        )
        .unwrap();
        assert!(r.outcome().is_none() && r.environment().is_err());
        finish(&mut r);
        let o = r.outcome().unwrap();
        assert!(o.error.is_none(), "{o:?}");
        assert_eq!(o.stdout, format!("1|system default|{value}\n"));
        assert_eq!(o.files, c.files);
        let actual = r.environment().unwrap();
        assert_eq!(actual.child["MODE"], value);
        assert_eq!(actual.child["PENDING"], "ready");
        assert!(!actual.child.contains_key("PRIVATE"));
        assert_eq!(actual.parent["PRIVATE"].value.as_deref(), Some(value));
        assert!(!actual.parent["SEEN"].exported);
    }
}
#[test]
fn empty_and_comment_only_files_return_zero_while_explicit_exit_stops_the_whole_session() {
    for empty in ["", "# no commands here\n", "\n\n"] {
        let f = files(&[
            ("/etc/profile", "false"),
            ("~/.bash_profile", empty),
            ("~/.profile", "printf never"),
        ]);
        let mut r = Run::new(
            &Invocation::InteractiveLogin { noprofile: false },
            &f,
            &input(),
            "printf '%s\\n' \"$?\"",
        )
        .unwrap();
        finish(&mut r);
        assert_eq!(r.outcome().unwrap().stdout, "0\n");
        assert_eq!(r.outcome().unwrap().status, 0);
    }
    let f = files(&[
        ("/etc/profile", "printf '%s' system; exit 7"),
        ("~/.bash_profile", "printf never"),
    ]);
    let mut r = Run::new(
        &Invocation::InteractiveLogin { noprofile: false },
        &f,
        &input(),
        "printf never",
    )
    .unwrap();
    finish(&mut r);
    assert_eq!(r.outcome().unwrap().stdout, "system");
    assert_eq!(r.outcome().unwrap().status, 7);
    assert!(r.outcome().unwrap().error.is_none());
}
#[test]
fn noninteractive_reads_only_literal_supplied_bash_env_without_path_search() {
    let f = files(&[
        ("/work/startup.env", "export JOB=\"$SOURCE\""),
        ("~/.bashrc", "printf unwanted"),
        ("~/.profile", "printf unwanted"),
    ]);
    let mut c = input();
    c.env.insert("SOURCE".into(), "quiet job".into());
    let mut r = Run::new(
        &Invocation::NonInteractive {
            bash_env: Some("/work/startup.env".into()),
        },
        &f,
        &c,
        "printf '%s\\n' \"$JOB\"",
    )
    .unwrap();
    assert_eq!(r.selected().len(), 1);
    finish(&mut r);
    assert_eq!(r.outcome().unwrap().stdout, "quiet job\n");
    assert_eq!(r.environment().unwrap().child["JOB"], "quiet job");
    for path in [
        "startup.env",
        "$HOME/.profile",
        "$(id)",
        "/etc/shadow",
        "../outside",
    ] {
        assert!(
            Run::new(
                &Invocation::NonInteractive {
                    bash_env: Some(path.into())
                },
                &f,
                &c,
                ":"
            )
            .is_err()
        );
        assert!(
            select(&Invocation::InteractiveNonLogin { norc: false }, &f).is_ok(),
            "An unused BASH_ENV must not be expanded"
        );
    }
}
#[test]
fn separately_parsed_sources_refuse_cross_file_quotes_unsupported_shapes_and_control_changes() {
    let bad = files(&[
        ("/etc/profile", "printf '%s' before"),
        ("~/.bash_profile", "printf '%s' 'unfinished"),
    ]);
    assert!(
        Run::new(
            &Invocation::InteractiveLogin { noprofile: false },
            &bad,
            &input(),
            "'after'"
        )
        .err()
        .unwrap()
        .contains("~/.bash_profile")
    );
    for source in [
        "while true; do :; done",
        "cat /etc/shadow",
        "source ~/.profile",
        "eval \"$SOURCE\"",
        "export HOME=/outside",
        "HOME=/outside",
        "export BASH_ENV=/outside",
        "export -n HOME",
        "export -n 'HOME'",
        "export -n \"HOME\"",
        "export -n H'OM'E",
        "printf -v HOME bad",
        "echo -e bad",
        "printf \"$SOURCE\"",
        "f() { :; }; f",
        "printf '%s' \"$(printf x)\"",
        "printf x > /work/keep.txt",
    ] {
        let f = files(&[("~/.bashrc", source)]);
        let error = Run::new(
            &Invocation::InteractiveNonLogin { norc: false },
            &f,
            &input(),
            ":",
        )
        .err()
        .unwrap();
        assert!(error.contains("Unsupported"), "{source}: {error}");
    }
}
#[test]
fn schema_limits_duplicates_traversal_and_unknown_fields_fail_without_execution() {
    let f = files(&[("~/.bashrc", "VALUE=one")]);
    let raw = serde_json::to_string(&f).unwrap();
    assert_eq!(Files::decode(&raw).unwrap(), f);
    for bad in [
        raw.replace("VALUE=one", &"x".repeat(8193)),
        raw.replace("~/.bashrc", "../outside"),
        raw.replace("{\"text\":", "{\"host\":true,\"text\":"),
        "{\"~/.bashrc\":{\"text\":\"one\"},\"~/.bashrc\":{\"text\":\"two\"}}".into(),
    ] {
        assert!(Files::decode(&bad).is_err());
    }
    assert!(Files::decode(&" ".repeat(65537)).is_err());
    assert!(
        serde_json::from_str::<Invocation>("{\"kind\":\"NonInteractive\",\"host\":true}").is_err()
    );
    let mut f = Files::default();
    for path in shell::startup::PATHS {
        f.0.insert(
            path.into(),
            File {
                text: "#".repeat(4096),
            },
        );
    }
    assert!(f.validate().is_err());
}
#[test]
fn stop_and_global_output_budget_survive_file_boundaries() {
    let f = files(&[
        ("/etc/profile", "printf '%s' \"$PAYLOAD\""),
        ("~/.bash_profile", "printf '%s' \"$PAYLOAD\""),
    ]);
    let mut c = input();
    c.env.insert("PAYLOAD".into(), "x".repeat(16000));
    let mut r = Run::new(
        &Invocation::InteractiveLogin { noprofile: false },
        &f,
        &c,
        "printf '%s' \"$PAYLOAD\"",
    )
    .unwrap();
    finish(&mut r);
    let o = r.outcome().unwrap();
    assert!(
        o.error
            .as_ref()
            .is_some_and(|e| e.to_lowercase().contains("limit")),
        "{o:?}"
    );
    assert!(o.stdout.len() <= 32768);
    assert!(r.environment().is_err());
    let mut stopped = Run::new(
        &Invocation::InteractiveLogin { noprofile: false },
        &f,
        &c,
        ":",
    )
    .unwrap();
    stopped.tick();
    stopped.stop();
    for _ in 0..100 {
        stopped.tick()
    }
    assert!(!stopped.done());
    assert!(stopped.outcome().is_none() && stopped.environment().is_err());
    assert_eq!(c.files, input().files);
}

#[test]
fn virtual_home_is_supplied_without_host_reads_and_cannot_redirect_startup_selection() {
    let f = files(&[("~/.bashrc", "LOCAL=ready")]);
    let mut c = input();
    let mut run = Run::new(
        &Invocation::InteractiveNonLogin { norc: false },
        &f,
        &c,
        "printf '%s\\n' \"$HOME\"",
    )
    .unwrap();
    finish(&mut run);
    assert_eq!(run.outcome().unwrap().stdout, "/home/learner\n");
    assert_eq!(run.environment().unwrap().child["HOME"], "/home/learner");
    c.env.insert("HOME".into(), "/home/someone-else".into());
    assert!(
        Run::new(
            &Invocation::InteractiveNonLogin { norc: false },
            &f,
            &c,
            ":"
        )
        .err()
        .unwrap()
        .contains("Unsupported virtual HOME")
    );
}

#[test]
fn quoted_ordinary_export_names_remain_valid_while_supplied_controls_are_protected() {
    let f = files(&[(
        "~/.bashrc",
        "MODE=ready\nexport 'MODE'\nexport -n \"MODE\"\nexport M'OD'E\n",
    )]);
    let mut r = Run::new(
        &Invocation::InteractiveNonLogin { norc: false },
        &f,
        &input(),
        "printf '%s\\n' \"$MODE\"",
    )
    .unwrap();
    finish(&mut r);
    assert_eq!(r.outcome().unwrap().stdout, "ready\n");
    assert_eq!(r.environment().unwrap().child["MODE"], "ready");
}

#[test]
fn printed_lookalike_and_zero_exit_cannot_claim_the_supplied_command_finished() {
    let f = files(&[(
        "~/.bashrc",
        "APP=ready\nexport APP\nprintf '%s\\n' report\nexit 0",
    )]);
    let mut run = Run::new(
        &Invocation::InteractiveNonLogin { norc: false },
        &f,
        &input(),
        "printf '%s\\n' report",
    )
    .unwrap();
    assert!(!run.session_command_finished());
    finish(&mut run);
    assert_eq!(run.outcome().unwrap().stdout, "report\n");
    assert_eq!(run.outcome().unwrap().status, 0);
    assert!(!run.session_command_finished());
    let fixed = files(&[("~/.bashrc", "APP=ready\nexport APP")]);
    let mut run = Run::new(
        &Invocation::InteractiveNonLogin { norc: false },
        &fixed,
        &input(),
        "printf '%s\\n' report",
    )
    .unwrap();
    finish(&mut run);
    assert!(run.session_command_finished());
}
