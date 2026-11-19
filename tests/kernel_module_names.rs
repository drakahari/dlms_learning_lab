//! Opt-in exact aliases, not registered lessons or host module operations.
#[path = "support/kernel_module_fixture.rs"]
mod fixture;
use dlms_learning_lab::kernel_modules::*;
use fixture::*;
#[test]
fn exact_saved_names_resolve_to_the_actual_module_dependency_and_parameter_state() {
    let c = catalogue();
    for alias in ["capture_current", "capture-current", "archive_reader"] {
        let mut r = runtime(&c);
        r.config = format!(
            "# name a caller uses\nalias {alias} lab_sensor\noptions lab_sensor queues=4\n"
        );
        let before = r.clone();
        assert!(r.execute(&c, "modprobe lab_sensor").is_err());
        assert_eq!(
            r.execute(&c, &format!("modprobe {alias}")).unwrap().status,
            1
        );
        assert_eq!(r, before, "Existing consoles keep refusing alias files");
        let out = r.execute_named(&c, &format!("modprobe {alias}")).unwrap();
        assert_eq!(out.status, 0, "{}", out.stderr);
        assert_eq!(r.loaded.len(), 3);
        assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 4);
        assert!(!r.loaded.contains_key(alias));
        let output = ok(&mut r, &c, "lsmod");
        assert!(output.contains("lab_sensor") && !output.contains("capture_current"));
        // A saved binding change does not mutate the already loaded instance.
        let loaded = r.loaded.clone();
        r.config = format!("alias {alias} audit_link\noptions lab_sensor queues=4\n");
        assert_eq!(r.loaded, loaded);
        r.execute_named(&c, &format!("modprobe {alias}")).unwrap();
        assert!(r.loaded.contains_key("audit_link"));
        assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 4);
    }
}
#[test]
fn wrong_alias_to_an_already_loaded_provider_does_not_establish_the_required_driver() {
    let c = catalogue();
    let mut r = runtime(&c);
    ok(&mut r, &c, "modprobe audit_link");
    r.config = "alias capture_current audit_link\n".into();
    let loaded = r.loaded.clone();
    assert_eq!(
        r.execute_named(&c, "modprobe capture_current")
            .unwrap()
            .status,
        0
    );
    assert_eq!(r.loaded, loaded);
    assert!(!r.loaded.contains_key("lab_sensor"));
    r.config = "alias capture_current lab_sensor\n".into();
    r.index.remove("lab_sensor");
    let fail = r.execute_named(&c, "modprobe capture_current").unwrap();
    assert_eq!(fail.status, 1);
    assert!(fail.stderr.contains("not found"));
    assert!(!r.loaded.contains_key("lab_sensor"));
    ok(&mut r, &c, "depmod -a");
    assert_eq!(
        r.execute_named(&c, "modprobe capture-current")
            .unwrap()
            .status,
        0
    );
    assert!(r.loaded.contains_key("lab_sensor") && r.loaded.contains_key("audit_link"));
}
#[test]
fn excluded_alias_forms_refuse_before_mutation_instead_of_guessing_priority_or_running_text() {
    let c = catalogue();
    let mut r = runtime(&c);
    for text in [
        "alias x y\nalias y lab_sensor\n",
        "alias x lab_sensor\nalias x audit_link\n",
        "alias x lab_sensor\nalias x lab_sensor\n",
        "alias lab_sensor audit_link\n",
        "alias x* lab_sensor\n",
        "alias x /tmp/evil.ko\n",
        "alias x lab_sensor extra\n",
        "install x /bin/sh\n",
        "alias x lab_sensor\noptions x queues=4\n",
        "alias x x\n",
    ] {
        r.config = text.into();
        let before = r.clone();
        assert!(r.execute_named(&c, "modprobe x").is_err(), "{text}");
        assert_eq!(r, before);
    }
    r.config = (0..9)
        .map(|n| format!("alias name{n} lab_sensor\n"))
        .collect();
    let before = r.clone();
    assert!(r.execute_named(&c, "modprobe name0").is_err());
    assert_eq!(r, before);
    r.config = "alias x lab_sensor\n".into();
    let before = r.clone();
    assert!(
        r.execute_named(&c, "modprobe -r x")
            .unwrap_err()
            .contains("physical")
    );
    assert_eq!(r, before);
    for input in [
        "modprobe x; touch escaped",
        "modprobe $(echo x)",
        "modprobe x | cat",
        "modprobe --force x",
    ] {
        let before = r.clone();
        assert!(r.execute_named(&c, input).is_err());
        assert_eq!(r, before);
    }
}
#[test]
fn aliases_are_bounded_configuration_data_not_new_runtime_authority() {
    let c = catalogue();
    let mut r = runtime(&c);
    r.config = "alias capture-current lab-sensor\noptions lab_sensor queues=04\n".into();
    let cfg = parse_named_config(&r.config, &c).unwrap();
    assert_eq!(cfg.aliases["capture_current"], "lab_sensor");
    assert_eq!(cfg.options["lab_sensor"]["queues"], "04");
    assert!(parse_config(&r.config, &c).is_err());
    r.execute_named(&c, "modprobe capture_current").unwrap();
    let encoded = serde_json::to_string(&r).unwrap();
    let copy: Runtime = serde_json::from_str(&encoded).unwrap();
    assert_eq!(copy, r);
    copy.validate(&c).unwrap();
    let before = r.clone();
    assert!(r.execute_named(&c, &"x".repeat(MAX_COMMAND + 1)).is_err());
    assert_eq!(r, before);
    assert!(
        r.execute_named(&c, "help")
            .unwrap()
            .stdout
            .contains("Only eight")
    );
    assert!(r.execute(&c, "help").unwrap().stdout.contains("No aliases"));
}

#[test]
fn typing_file_directives_in_the_named_terminal_explains_the_editor_action() {
    let c = catalogue();
    let mut r = runtime(&c);
    for input in [
        "alias capture_current lab_sensor",
        "options lab_sensor queues=1",
    ] {
        let before = r.clone();
        let e = r.execute_named(&c, input).unwrap_err();
        assert!(e.contains("configuration line") && e.contains("native editor"));
        assert_eq!(r, before);
    }
}
