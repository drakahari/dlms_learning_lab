#[path = "support/kernel_module_fixture.rs"]
mod fixture;
use dlms_learning_lab::kernel_modules::*;
use fixture::*;

#[test]
fn device_candidate_is_not_an_active_driver_and_auto_loading_closes_dependencies() {
    let c = catalogue();
    let mut r = runtime(&c);
    c.validate().unwrap();
    let out = ok(&mut r, &c, "lspci -k");
    assert!(out.contains("Kernel modules: lab_sensor"));
    assert!(!out.contains("driver in use"));
    let initial = r.clone();
    let path = &c.modules["lab_sensor"].path;
    let out = r.execute(&c, &format!("insmod {path}")).unwrap();
    assert_eq!(out.status, 1);
    assert!(r.loaded.is_empty());
    assert!(ok(&mut r, &c, "dmesg").contains("unresolved dependency"));
    assert_eq!(r.index, initial.index);
    ok(&mut r, &c, "modprobe lab-sensor queues=4");
    assert_eq!(r.loaded.len(), 3);
    assert_eq!(
        ok(&mut r, &c, "cat /sys/module/lab_sensor/parameters/queues"),
        "4\n"
    );
    assert!(ok(&mut r, &c, "lspci -k").contains("Kernel driver in use: lab_sensor"));
    let out = ok(&mut r, &c, "lsmod");
    assert!(out.starts_with("Module"));
    assert!(out.contains("sensor_bus"));
    assert!(out.contains("1 lab_sensor"));
    let loaded = r.loaded.clone();
    ok(&mut r, &c, "modprobe lab_sensor queues=6");
    assert_eq!(
        r.loaded, loaded,
        "Loading an existing module must not apply new parameters"
    );
    assert_eq!(r.execute(&c, &format!("insmod {path}")).unwrap().status, 1);
}

#[test]
fn manual_dependencies_and_automatic_loading_produce_equivalent_effective_state() {
    let c = catalogue();
    for n in [1, 3, 8] {
        let mut manual = runtime(&c);
        let mut automatic = runtime(&c);
        for name in ["bus_core", "sensor_bus", "lab_sensor"] {
            let extra = if name == "lab_sensor" {
                format!(" queues={n}")
            } else {
                String::new()
            };
            ok(
                &mut manual,
                &c,
                &format!("insmod {}{extra}", c.modules[name].path),
            );
        }
        ok(
            &mut automatic,
            &c,
            &format!("modprobe lab_sensor queues={n}"),
        );
        assert_eq!(manual.loaded, automatic.loaded);
    }
}

#[test]
fn removal_checks_users_preserves_other_consumers_and_distinguishes_rmmod() {
    let c = catalogue();
    let mut r = runtime(&c);
    ok(&mut r, &c, "modprobe lab_sensor");
    ok(&mut r, &c, "modprobe audit_link");
    let loaded = r.loaded.clone();
    let fail = r.execute(&c, "rmmod bus_core").unwrap();
    assert_eq!(fail.status, 1);
    assert!(fail.stderr.contains("in use"));
    assert_eq!(r.loaded, loaded);
    ok(&mut r, &c, "modprobe -r lab_sensor");
    assert!(!r.loaded.contains_key("sensor_bus"));
    assert!(r.loaded.contains_key("bus_core"));
    assert!(r.loaded.contains_key("audit_link"));
    ok(&mut r, &c, "modprobe -r lab_sensor");
    assert_eq!(r.execute(&c, "rmmod lab_sensor").unwrap().status, 1);
    ok(&mut r, &c, "rmmod audit_link");
    assert!(
        r.loaded.contains_key("bus_core"),
        "rmmod removes only the named module"
    );
    ok(&mut r, &c, "modprobe -r bus_core");
    assert!(r.loaded.is_empty());
    let mut busy = c.clone();
    busy.modules.get_mut("bus_core").unwrap().external_users = 1;
    ok(&mut r, &busy, "modprobe lab_sensor");
    ok(&mut r, &busy, "modprobe -r lab_sensor");
    assert!(r.loaded.contains_key("bus_core"));
    assert_eq!(r.execute(&busy, "modprobe -r bus_core").unwrap().status, 1);
}

#[test]
fn saved_options_are_not_effective_until_reload_and_invalid_options_remain_diagnosable() {
    let c = catalogue();
    let mut r = runtime(&c);
    ok(&mut r, &c, "modprobe lab_sensor");
    r.config = "# Future insertions\noptions lab-sensor queues=4\n".into();
    ok(&mut r, &c, "modprobe lab_sensor");
    assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 1);
    assert_eq!(ok(&mut r, &c, &format!("cat {CONFIG_PATH}")), r.config);
    ok(&mut r, &c, "modprobe -r lab_sensor");
    ok(&mut r, &c, "modprobe lab_sensor");
    assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 4);
    ok(&mut r, &c, "modprobe -r lab_sensor");
    r.config = "options lab_sensor queues=99\n".into();
    let out = r.execute(&c, "modprobe lab_sensor").unwrap();
    assert_eq!(out.status, 1);
    assert!(!r.loaded.contains_key("lab_sensor"));
    assert!(
        r.loaded.contains_key("sensor_bus"),
        "Already inserted dependencies are observable, not falsely rolled back"
    );
    assert!(ok(&mut r, &c, "dmesg").contains("invalid parameter queues"));
    assert!(ok(&mut r, &c, "modinfo lab_sensor").contains("1..8"));
    // insmod uses explicit object/parameters; it does not read modprobe.d.
    ok(
        &mut r,
        &c,
        &format!("insmod {} queues=6", c.modules["lab_sensor"].path),
    );
    assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 6);
}

#[test]
fn index_rebuild_is_separate_from_object_metadata_and_release_validation() {
    let c = catalogue();
    let mut r = runtime(&c);
    r.index.remove("lab_sensor");
    assert_eq!(r.execute(&c, "modprobe lab_sensor").unwrap().status, 1);
    assert!(r.loaded.is_empty());
    assert_eq!(r.execute(&c, "modinfo lab_sensor").unwrap().status, 1);
    assert!(
        ok(
            &mut r,
            &c,
            &format!("modinfo {}", c.modules["lab_sensor"].path)
        )
        .contains("sensor_bus")
    );
    ok(&mut r, &c, "depmod -a");
    assert!(!r.index.contains("old_sensor"));
    ok(&mut r, &c, "modprobe lab_sensor");
    let loaded = r.loaded.clone();
    assert_eq!(
        r.execute(&c, &format!("insmod {}", c.modules["old_sensor"].path))
            .unwrap()
            .status,
        1
    );
    assert_eq!(r.loaded, loaded);
    assert!(ok(&mut r, &c, "dmesg").contains("version magic"));
    assert_eq!(ok(&mut r, &c, "uname -r"), "6.8.0-lab\n");
}

#[test]
fn unsupported_shell_force_paths_and_configuration_never_execute_or_silently_succeed() {
    let c = catalogue();
    let mut r = runtime(&c);
    for cmd in [
        "modprobe --force lab_sensor",
        "modprobe lab_sensor; uname -r",
        "modprobe $(touch secret)",
        "rmmod -f bus_core",
        "modprobe /lib/modules/6.8.0-lab/kernel/drivers/lab_sensor.ko",
        "cat /etc/passwd",
        "cat /sys/module/../../etc/passwd",
        "insmod /tmp/module.ko",
        "lsmod | cat",
        "modprobe lab_sensor queues=1 queues=2",
        "modprobe lab_sensor queues=abc",
    ] {
        let before = r.clone();
        assert!(r.execute(&c, cmd).is_err(), "{cmd}");
        assert_eq!(r, before, "Unsupported syntax must be nonmutating: {cmd}");
    }
    for text in [
        "install lab_sensor /bin/sh\n",
        "blacklist lab_sensor\n",
        "alias x lab_sensor\n",
        "options lab_sensor queues=1 queues=2\n",
        "options lab_sensor queues=1\noptions lab_sensor queues=2\n",
        "options lab_sensor queues=4 \\\n queues=2\n",
    ] {
        r.config = text.into();
        let before = r.clone();
        assert!(r.execute(&c, "modprobe lab_sensor").is_err());
        assert_eq!(r, before);
    }
}

#[test]
fn source_and_runtime_validation_reject_cycles_forged_loads_and_oversized_data() {
    let c = catalogue();
    let r = runtime(&c);
    let mut bad = c.clone();
    bad.modules.get_mut("bus_core").unwrap().dependencies = vec!["lab_sensor".into()];
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.modules.get_mut("bus_core").unwrap().dependencies = vec!["not_supplied".into()];
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.modules.get_mut("lab_sensor").unwrap().path = "/lib/modules/6.8.0-lab/../escape.ko".into();
    assert!(bad.validate().is_err());
    let mut bad = r.clone();
    bad.loaded.insert(
        "lab_sensor".into(),
        Loaded {
            parameters: [("queues".into(), 4)].into(),
        },
    );
    assert!(bad.validate(&c).is_err());
    let mut bad = r.clone();
    bad.index.insert("invented".into());
    assert!(bad.validate(&c).is_err());
    let mut bad = r.clone();
    bad.config = "x".repeat(MAX_CONFIG + 1);
    assert!(bad.validate(&c).is_err());
    let mut r = r;
    let before = r.clone();
    assert!(r.execute(&c, &"x".repeat(MAX_COMMAND + 1)).is_err());
    assert_eq!(r, before);
    for _ in 0..40 {
        r.execute(&c, &format!("insmod {}", c.modules["lab_sensor"].path))
            .unwrap();
    }
    assert_eq!(r.diagnostics.len(), 32);
    r.validate(&c).unwrap();
    let encoded = serde_json::to_string(&r).unwrap();
    assert_eq!(serde_json::from_str::<Runtime>(&encoded).unwrap(), r);
    assert!(
        serde_json::from_str::<Runtime>(&encoded.replace("\"index\":", "\"host\":true,\"index\":"))
            .is_err()
    );
}

#[test]
fn unknown_options_are_disclosed_as_unsupported_and_integer_boundaries_do_not_fake_linux_rules() {
    let c = catalogue();
    let mut r = runtime(&c);
    for cmd in [
        "modprobe lab_sensor typo=4",
        "insmod /lib/modules/6.8.0-lab/kernel/drivers/lab_sensor.ko typo=4",
    ] {
        let before = r.clone();
        let e = r.execute(&c, cmd).unwrap_err();
        assert!(e.contains("Unsupported parameter") && e.contains("Real kernels may ignore"));
        assert_eq!(r, before);
    }
    r.config = "options lab_sensor typo=4\n".into();
    let before = r.clone();
    assert!(
        r.execute(&c, "modprobe lab_sensor")
            .unwrap_err()
            .contains("Unsupported parameter")
    );
    assert_eq!(r, before);
    r.config.clear();
    for (value, expected) in [
        ("04", Some(4)),
        ("010", Some(8)),
        ("08", None),
        ("0x4", None),
        ("-04", Some(-4)),
        ("9223372036854775808", None),
    ] {
        assert_eq!(integer(value), expected, "value {value}");
    }
    ok(&mut r, &c, "modprobe lab_sensor queues=010");
    assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 8);
    ok(&mut r, &c, "modprobe -r lab_sensor");
    let out = r.execute(&c, "modprobe lab_sensor queues=08").unwrap();
    assert_eq!(out.status, 1);
    assert!(!r.loaded.contains_key("lab_sensor"));
    for value in ["0", "9", "-1", "9223372036854775808"] {
        let out = r
            .execute(&c, &format!("modprobe lab_sensor queues={value}"))
            .unwrap();
        assert_eq!(out.status, 1);
        assert!(!r.loaded.contains_key("lab_sensor"));
    }
    r.config = "options lab_sensor queues=4\n".into();
    let before = r.clone();
    assert!(r.execute(&c, "modprobe lab_sensor queues=3").is_err());
    assert_eq!(
        r, before,
        "Unsupported duplicate sources must not partially load dependencies"
    );
}
