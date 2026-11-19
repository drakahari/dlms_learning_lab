use dlms_learning_lab::dkms::{self as d, Facts, Kernel, Source, State};

fn facts() -> Facts {
    Facts {
        kernels: ["6.6.49", "6.6.50"]
            .map(|release| Kernel {
                release: release.into(),
            })
            .to_vec(),
        running: 0,
        architecture: "x86_64".into(),
        sources: [
            ("survey", "1.0", "survey"),
            ("survey", "2.0", "survey"),
            ("sensor", "1.0", "sensor"),
        ]
        .map(|(name, version, object)| Source {
            name: name.into(),
            version: version.into(),
            object: object.into(),
        })
        .to_vec(),
    }
}
fn run(s: &mut State, command: &str) -> String {
    d::execute(&facts(), s, command).unwrap()
}

#[test]
fn manual_steps_and_install_chaining_grade_the_same_actual_state() {
    let mut manual = State::default();
    run(&mut manual, "dkms add survey/1.0");
    assert_eq!(run(&mut manual, "dkms status"), "survey/1.0: added");
    assert!(manual.records.is_empty());
    run(&mut manual, "dkms build survey/1.0 -k 6.6.50");
    assert_eq!(
        run(&mut manual, "dkms status -k 6.6.50"),
        "survey/1.0, 6.6.50, x86_64: built"
    );
    assert!(manual.virtual_files(&facts()).unwrap().is_empty());
    run(&mut manual, "dkms install survey/1.0 -k 6.6.50");
    let mut direct = State::default();
    run(&mut direct, "dkms install -k 6.6.50 -v 1.0 -m survey");
    assert_eq!(direct, manual);
    assert_eq!(
        direct.virtual_files(&facts()).unwrap(),
        ["/lib/modules/6.6.50/updates/dkms/survey.ko"]
    );
    assert!(direct.installed(&facts(), 0, 1));
    assert!(!direct.installed(&facts(), 0, 0));
}

#[test]
fn default_kernel_is_supplied_running_selection_not_host_or_newest() {
    let mut s = State::default();
    run(&mut s, "dkms install survey/1.0");
    assert!(s.installed(&facts(), 0, 0));
    assert!(!s.installed(&facts(), 0, 1));
    let mut f = facts();
    f.running = 1;
    let mut other = State::default();
    d::execute(&f, &mut other, "dkms install survey/1.0").unwrap();
    assert!(other.installed(&f, 0, 1));
    assert!(!other.installed(&f, 0, 0));
}

#[test]
fn build_auto_registers_and_repeated_build_install_leave_state_unchanged() {
    let mut s = State::default();
    run(&mut s, "dkms build survey/1.0");
    assert_eq!(s.registered, [0]);
    let built = s.clone();
    assert!(run(&mut s, "dkms build survey/1.0").contains("Already built"));
    assert_eq!(s, built);
    run(&mut s, "dkms install survey/1.0");
    let installed = s.clone();
    assert!(run(&mut s, "dkms install survey/1.0").contains("Already installed"));
    assert_eq!(s, installed);
    run(&mut s, "dkms build survey/1.0");
    assert_eq!(s, installed);
    assert!(
        d::execute(&facts(), &mut s, "dkms add survey/1.0")
            .unwrap_err()
            .contains("already registered")
    );
    assert_eq!(s, installed);
}

#[test]
fn targeted_uninstall_retains_build_and_registration_and_other_kernel() {
    let mut s = State::default();
    run(&mut s, "dkms install survey/1.0");
    run(&mut s, "dkms install survey/1.0 -k 6.6.50");
    run(&mut s, "dkms uninstall survey/1.0 -k 6.6.50");
    assert!(s.installed(&facts(), 0, 0));
    assert!(!s.installed(&facts(), 0, 1));
    assert_eq!(s.records.len(), 2);
    assert_eq!(
        run(&mut s, "dkms status survey/1.0 -k 6.6.50"),
        "survey/1.0, 6.6.50, x86_64: built"
    );
    run(&mut s, "dkms install survey/1.0 -k 6.6.50");
    assert!(s.installed(&facts(), 0, 1));
}

#[test]
fn version_handover_requires_explicit_uninstall_without_guessing_sanity_checks() {
    let mut s = State::default();
    run(&mut s, "dkms install survey/1.0");
    run(&mut s, "dkms install survey/1.0 -k 6.6.50");
    let before = s.clone();
    let e = d::execute(&facts(), &mut s, "dkms install survey/2.0 -k 6.6.50").unwrap_err();
    assert!(e.contains("outside this subset"));
    assert_eq!(s, before);
    run(&mut s, "dkms uninstall survey/1.0 -k 6.6.50");
    run(&mut s, "dkms install survey/2.0 -k 6.6.50");
    assert!(s.installed(&facts(), 0, 0));
    assert!(s.installed(&facts(), 1, 1));
    assert!(!s.installed(&facts(), 0, 1));
    assert_eq!(s.virtual_files(&facts()).unwrap().len(), 2);
    assert!(run(&mut s, "dkms status -m survey -v 1.0").contains("6.6.50, x86_64: built"));
}

#[test]
fn status_filters_are_read_only_and_do_not_imply_module_loading() {
    let mut s = State::default();
    run(&mut s, "dkms add sensor/1.0");
    run(&mut s, "dkms install survey/1.0");
    let before = s.clone();
    assert_eq!(run(&mut s, "dkms status -m sensor"), "sensor/1.0: added");
    assert_eq!(run(&mut s, "dkms status -m absent"), "");
    assert_eq!(run(&mut s, "dkms status -k 6.6.50"), "");
    assert_eq!(s, before);
    assert!(run(&mut s, "dkms --help").contains("not loaded"));
    assert_eq!(s, before);
}

#[test]
fn unsafe_unsupported_and_malformed_commands_do_not_change_state() {
    let mut s = State::default();
    run(&mut s, "dkms install survey/1.0");
    let before = s.clone();
    for input in [
        "dkms install survey/1.0; sh",
        "dkms install $(id)/1.0",
        "dkms install 'survey/1.0'",
        "dkms status | cat",
        "dkms add /host/path",
        "dkms install ../1.0",
        "dkms install survey/1.0 -k ../../host",
        "dkms status\ndkms install survey/2.0",
        "dkms\tstatus",
        "dkms\u{a0}status",
        "dkms install survey/1.0 --force",
        "dkms autoinstall",
        "dkms remove survey/1.0 --all",
        "dkms build survey/1.0 -k 6.6.49 -k 6.6.50",
        "dkms status -v 1.0",
        "dkms status -m",
        "dkms status -m --all",
        "dkms status -m survey -m sensor",
        "dkms add survey/1.0 -k 6.6.50",
        "dkms install survey/1.0 -k 6.6.50/x86_64",
        "sh script.sh",
        "dkms install survey/1.0 extra",
    ] {
        assert!(d::execute(&facts(), &mut s, input).is_err(), "{input}");
        assert_eq!(s, before, "{input}");
    }
    assert!(d::execute(&facts(), &mut s, &"x".repeat(d::MAX_INPUT + 1)).is_err());
    assert_eq!(s, before);
}

#[test]
fn unavailable_fixtures_and_absent_uninstall_are_explicit_not_fake_compiler_failures() {
    let mut s = State::default();
    for input in [
        "dkms build unknown/1.0",
        "dkms install survey/1.0 -k 0.0.0",
        "dkms uninstall survey/1.0",
    ] {
        let e = d::execute(&facts(), &mut s, input).unwrap_err();
        assert!(e.contains("supplied") || e.contains("outside this subset"));
        assert_eq!(s, State::default());
    }
}

#[test]
fn malformed_imports_and_ambiguous_inventory_are_rejected() {
    let f = facts();
    let mut s = State::default();
    run(&mut s, "dkms install survey/1.0");
    let raw = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<State>(&raw).unwrap(), s);
    for broken in [
        State {
            registered: vec![0, 0],
            records: vec![],
        },
        State {
            registered: vec![99],
            records: vec![],
        },
        State {
            registered: vec![0],
            records: vec![d::Record {
                source: 99,
                kernel: 0,
                installed: true,
            }],
        },
        State {
            registered: vec![0],
            records: vec![d::Record {
                source: 0,
                kernel: 9,
                installed: true,
            }],
        },
        State {
            registered: vec![0, 1],
            records: vec![
                d::Record {
                    source: 0,
                    kernel: 0,
                    installed: true,
                },
                d::Record {
                    source: 1,
                    kernel: 0,
                    installed: true,
                },
            ],
        },
    ] {
        assert!(broken.validate(&f).is_err());
        assert!(!broken.installed(&f, 0, 0));
    }
    let mut invalid = f.clone();
    invalid.kernels.push(invalid.kernels[0].clone());
    assert!(invalid.validate().is_err());
    let mut invalid = f.clone();
    invalid.sources[2].object = "survey".into();
    assert!(invalid.validate().is_err());
    let mut invalid = f;
    invalid.running = 9;
    assert!(invalid.validate().is_err());
    assert!(
        serde_json::from_str::<State>(r#"{"registered":[],"records":[],"host_path":"/home"}"#)
            .is_err()
    );
    let too_many = serde_json::json!({"registered":vec![0;9],"records":[]});
    assert!(serde_json::from_value::<State>(too_many).is_err());
}
