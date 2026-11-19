use dlms_learning_lab::{
    script_teaching::View,
    source_install::{Configuration, Facts, command},
    source_install_workspace::{
        self as work, Argument, Case, Command, Definition, Destination, Draft, Path, Session, Task,
    },
};
fn task(staged: bool, inherited: bool, example: bool) -> Task {
    let mut facts: Facts =
        serde_json::from_str(include_str!("fixtures/source-install-project-v1.json")).unwrap();
    let required = Configuration {
        prefix: if example { "/opt/demo" } else { "/usr/local" }.into(),
        greeting: if example {
            "Greetings,"
        } else if inherited {
            "Good day,"
        } else {
            "Welcome,"
        }
        .into(),
        uppercase: example || inherited,
    };
    if example {
        facts.program.name = "welcome".into();
        facts.program.version = "2.0".into();
        facts.archive = "/sources/welcome-2.0.tar".into();
        facts.archive_root = "welcome-2.0".into();
    }
    let mut cases = Vec::new();
    for (label, arg, stdout) in [
        (
            "first name",
            "Mira",
            format!("{} Mira\n", required.greeting),
        ),
        (
            "spaced name",
            "Lee Tran",
            format!("{} Lee Tran\n", required.greeting),
        ),
        (
            "compiled prefix",
            "--prefix",
            format!("{}\n", required.prefix),
        ),
        (
            "package version",
            "--version",
            format!("{} {}\n", facts.program.name, facts.program.version),
        ),
    ] {
        let stdout = if required.uppercase && !arg.starts_with('-') {
            stdout.to_ascii_uppercase()
        } else {
            stdout
        };
        cases.push(Case {
            label: label.into(),
            args: vec![Argument(arg.into())],
            stdout,
            stderr: String::new(),
            status: 0,
        });
    }
    let setup = vec![
        format!("tar -xf {} -C /usr/src", facts.archive),
        format!("cd /usr/src/{}", facts.archive_root),
    ];
    let configure = format!(
        "./configure --prefix={} --with-greeting='{}' {}",
        required.prefix,
        required.greeting,
        if required.uppercase {
            "--enable-uppercase"
        } else {
            "--disable-uppercase"
        }
    );
    let mut starter = Vec::new();
    let mut reference = Vec::new();
    if inherited {
        starter.extend(setup.iter().cloned().map(Command));
        starter.extend(
            [
                "./configure --prefix=/usr/local --with-greeting='Old notice,'".into(),
                "make check".into(),
                "make install DESTDIR=/review".into(),
            ]
            .map(Command),
        );
    } else {
        reference.extend(setup.into_iter().map(Command));
    }
    reference.extend([configure, "make check".into()].map(Command));
    if staged {
        reference.push(Command("make DESTDIR=/review install".into()));
    }
    Task {
        id: format!(
            "{}-{}",
            if example { "worked" } else { "assignment" },
            if inherited {
                "repair"
            } else if staged {
                "stage"
            } else {
                "build"
            }
        ),
        facts,
        directories: vec![Path("/usr/src".into()), Path("/review".into())],
        initial_directory: "/sources".into(),
        starter,
        required,
        destination: if staged {
            Destination::Staged {
                directory: "/review".into(),
            }
        } else {
            Destination::Build
        },
        require_self_check: true,
        cases,
        reference,
    }
}
fn definition() -> Definition {
    Definition {
        initial: task(true, true, false),
        example: task(true, false, true),
    }
}
fn run(draft: &mut Draft, task: &Task, command: &str) {
    draft.input = command.into();
    let out = draft.run(task).unwrap().unwrap();
    assert_eq!(out.status, 0, "{command}: {out:?}");
}
fn solve(draft: &mut Draft, task: &Task) {
    for command in &task.reference {
        run(draft, task, &command.0);
    }
    assert!(draft.check(task).unwrap().iter().all(|r| r.correct));
}
#[test]
fn separate_build_stage_and_repair_methods_reach_behavior_cases_without_string_grading() {
    for task in [
        task(false, false, false),
        task(true, false, false),
        task(true, true, false),
    ] {
        task.validate().unwrap();
        let mut draft = Draft::new(&task);
        assert!(!draft.check(&task).unwrap().iter().all(|r| r.correct));
        solve(&mut draft, &task);
        draft.validate(&task).unwrap();
        let frame = task.replay(&draft.history).unwrap();
        let results = task.grade(&frame);
        assert_eq!(results.iter().filter(|r| r.actual.is_some()).count(), 4);
    }
}
#[test]
fn legitimate_alternate_option_spelling_target_order_and_make_directory_pass() {
    let t = task(true, false, false);
    let mut draft = Draft::new(&t);
    for command in [
        "tar --extract --file /sources/greet-1.4.tar --directory /usr/src",
        "cd /usr/src/greet-1.4",
        "./configure --prefix /usr/local --with-greeting Welcome,",
        "cd /sources",
        "make -C /usr/src/greet-1.4 all",
        "make -C /usr/src/greet-1.4 check",
        "make install -C /usr/src/greet-1.4 DESTDIR=/review",
    ] {
        run(&mut draft, &t, command);
    }
    assert!(draft.check(&t).unwrap().iter().all(|r| r.correct));
    let history = draft.history.clone();
    run(&mut draft, &t, "pwd");
    assert!(draft.checked);
    run(&mut draft, &t, "cd /");
    assert!(draft.checked);
    assert!(draft.history.starts_with(&history));
}
#[test]
fn guided_example_isolated_and_each_reset_changes_only_its_selected_draft() {
    let def = definition();
    def.validate().unwrap();
    let mut s = Session::new(&def);
    solve(&mut s.practice, &def.example);
    assert!(!s.solved(&def));
    let practiced = s.practice.clone();
    solve(&mut s.assignment, &def.initial);
    assert!(s.solved(&def));
    s.reset_assignment(&def);
    assert!(!s.solved(&def));
    assert_eq!(s.practice, practiced);
    let unfinished = s.assignment.clone();
    s.reset_practice(&def);
    assert_eq!(s.assignment, unfinished);
    assert!(!s.practice.checked);
    assert_eq!(s.assignment.history, def.initial.starter);
}
#[test]
fn reconfigure_or_rebuild_does_not_repair_staging_until_install_and_current_checks() {
    let t = task(true, true, false);
    let mut draft = Draft::new(&t);
    run(&mut draft, &t, &t.reference[0].0);
    let partial = draft.check(&t).unwrap();
    assert!(
        partial
            .iter()
            .any(|r| r.label == "Configured project" && r.correct)
    );
    assert!(
        partial
            .iter()
            .any(|r| r.label == "Current package self-check" && !r.correct)
    );
    run(&mut draft, &t, "make check");
    let partial = draft.check(&t).unwrap();
    assert!(
        partial
            .iter()
            .any(|r| r.label == "Current package self-check" && r.correct)
    );
    assert!(
        partial
            .iter()
            .any(|r| r.label.starts_with("Program case") && !r.correct)
    );
    assert!(!draft.checked);
    run(&mut draft, &t, "make install DESTDIR=/review");
    assert!(draft.check(&t).unwrap().iter().all(|r| r.correct));
}
#[test]
fn state_breakage_clears_success_but_passing_individual_cases_remain_honest() {
    let t = task(true, false, false);
    let mut draft = Draft::new(&t);
    solve(&mut draft, &t);
    run(
        &mut draft,
        &t,
        "./configure --prefix=/opt/wrong --with-greeting=Wrong",
    );
    assert!(!draft.checked);
    let results = draft.check(&t).unwrap();
    assert!(!results.iter().all(|r| r.correct));
    assert!(
        results
            .iter()
            .filter(|r| r.label.starts_with("Program case"))
            .all(|r| r.correct),
        "The old staged copy still has correct behavior; configuration/self-check policy fails separately."
    );
    assert!(
        results
            .iter()
            .any(|r| !r.correct && r.label == "Configured project")
    );
    assert!(!draft.checked);
}
#[test]
fn an_unstaged_install_cannot_hide_live_file_damage_behind_passing_program_cases() {
    let t = task(true, false, false);
    let mut draft = Draft::new(&t);
    solve(&mut draft, &t);
    run(&mut draft, &t, "make install");
    let results = draft.check(&t).unwrap();
    assert!(
        results
            .iter()
            .filter(|r| r.label.starts_with("Program case"))
            .all(|r| r.correct)
    );
    assert!(
        results
            .iter()
            .any(|r| r.label.starts_with("Preserve") && !r.correct)
    );
    assert!(!draft.checked);
}
#[test]
fn resume_recomputes_actual_output_and_keeps_both_unfinished_inputs_and_views() {
    let d = definition();
    let mut s = Session::new(&d);
    s.view = View::Example;
    run(&mut s.assignment, &d.initial, &d.initial.reference[0].0);
    s.assignment.input = "make --unfinished".into();
    s.practice.input = "tar -tf ".into();
    let bytes = serde_json::to_vec(&s).unwrap();
    let restored: Session = serde_json::from_slice(&bytes).unwrap();
    restored.validate(&d).unwrap();
    assert_eq!(restored, s);
    assert_eq!(
        restored.assignment.last_output(&d.initial).unwrap(),
        s.assignment.last_output(&d.initial).unwrap()
    );
    assert!(!restored.solved(&d));
}
#[test]
fn syntax_unsupported_and_program_exit_status_are_distinct_retryable_results() {
    let t = task(false, false, false);
    let mut draft = Draft::new(&t);
    for (command, kind) in [
        (
            "echo 'unfinished",
            dlms_learning_lab::boot_entry::ErrorKind::Syntax,
        ),
        (
            "bash file.sh",
            dlms_learning_lab::boot_entry::ErrorKind::Unsupported,
        ),
    ] {
        let prior = draft.history.clone();
        draft.input = command.into();
        let err = draft.run(&t).unwrap().unwrap_err();
        assert_eq!(err.kind, kind);
        assert_eq!(draft.history, prior);
        draft.validate(&t).unwrap();
    }
    solve(&mut draft, &t);
    draft.input = "./greet --unsupported".into();
    let out = draft.run(&t).unwrap().unwrap();
    assert_eq!(out.status, 2);
    assert!(!out.stderr.is_empty());
    assert!(draft.checked);
}
#[test]
fn forged_output_success_missing_history_or_erased_inherited_work_are_rejected() {
    let d = definition();
    let mut s = Session::new(&d);
    let mut forged = s.clone();
    forged.assignment.checked = true;
    assert!(forged.validate(&d).is_err());
    let mut forged = s.clone();
    forged.assignment.history.clear();
    assert!(forged.validate(&d).is_err());
    run(&mut s.assignment, &d.initial, "pwd");
    let mut forged = s.clone();
    forged.assignment.last_run.as_mut().unwrap().command = Command("cat INSTALL".into());
    assert!(forged.validate(&d).is_err());
    let mut json = serde_json::to_value(&s).unwrap();
    json["assignment"]["stdout"] = serde_json::json!("pretend success");
    assert!(serde_json::from_value::<Session>(json).is_err());
    let mut json = serde_json::to_value(&s).unwrap();
    json["assignment"]["input"] = serde_json::json!("x".repeat(1025));
    assert!(serde_json::from_value::<Session>(json).is_err());
    let mut json = serde_json::to_value(&s).unwrap();
    json["version"] = serde_json::json!(2);
    let invalid: Session = serde_json::from_value(json).unwrap();
    assert!(invalid.validate(&d).is_err());
}
#[test]
fn exhausted_history_does_not_record_a_fictitious_execution_or_freeze() {
    let t = task(false, false, false);
    let mut draft = Draft::new(&t);
    for _ in 0..work::MAX_HISTORY {
        run(&mut draft, &t, "pwd");
    }
    let prior = draft.history.clone();
    draft.input = "pwd".into();
    let err = draft.run(&t).unwrap().unwrap_err();
    assert_eq!(err.kind, dlms_learning_lab::boot_entry::ErrorKind::Limit);
    assert_eq!(draft.history, prior);
    draft.validate(&t).unwrap();
    assert_eq!(draft.last_output(&t).unwrap().unwrap().unwrap_err(), err);
}
#[test]
fn invalid_or_already_solved_definitions_and_copied_example_values_do_not_pass() {
    let t = task(true, false, false);
    let mut invalid = t.clone();
    invalid.cases[0].stdout = "canned answer".into();
    assert!(invalid.validate().is_err());
    let mut invalid = t.clone();
    invalid.cases[1] = invalid.cases[0].clone();
    assert!(invalid.validate().is_err());
    let mut invalid = t.clone();
    invalid.starter = invalid.reference.clone();
    assert!(invalid.validate().is_err());
    let d = definition();
    let mut s = Session::new(&d);
    s.assignment.history.extend(d.example.reference.clone());
    assert!(s.validate(&d).is_err());
    assert!(!s.solved(&d));
    let mut frame = t.initial().unwrap();
    let mut cwd = frame.cwd.clone();
    let prior = frame.world.clone();
    assert!(
        command::run(
            &mut frame.world,
            &t.facts,
            &mut cwd,
            "cat /proc/self/environ"
        )
        .is_err()
    );
    assert_eq!(frame.world, prior);
}
