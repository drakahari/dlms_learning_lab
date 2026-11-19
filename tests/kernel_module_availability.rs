//! Image-object availability is opt-in, source-owned and entirely in memory.
#[path = "support/kernel_module_fixture.rs"]
mod fixture;
use dlms_learning_lab::kernel_modules::*;
use fixture::*;
use std::collections::BTreeSet;

fn all(c: &Catalogue) -> BTreeSet<String> {
    c.modules.keys().cloned().collect()
}

#[test]
fn legacy_consoles_and_complete_images_have_identical_outcomes_and_saved_state() {
    let c = catalogue();
    let mut old = runtime(&c);
    let mut image = old.clone();
    for command in [
        "uname -r",
        "lspci -k",
        "modinfo lab_sensor",
        "depmod -a",
        "modprobe audit_link",
        "modprobe lab_sensor queues=4",
        "lsmod",
        "cat /sys/module/lab_sensor/parameters/queues",
        "modprobe -r lab_sensor",
        "modprobe lab_sensor",
        "dmesg",
    ] {
        assert_eq!(
            old.execute(&c, command).unwrap(),
            image.execute_available(&c, command, &all(&c)).unwrap(),
            "{command}"
        );
        assert_eq!(old, image);
        assert_eq!(
            serde_json::to_value(&old).unwrap(),
            serde_json::to_value(&image).unwrap()
        );
    }
}

#[test]
fn knowing_a_module_and_index_entry_does_not_create_its_missing_object() {
    let c = catalogue();
    let mut r = runtime(&c);
    let mut present = all(&c);
    present.remove("lab_sensor");
    assert!(r.index.contains("lab_sensor"));
    let pci = r.execute_available(&c, "lspci -k", &present).unwrap();
    assert!(pci.stdout.contains("Teaching sensor controller"));
    assert!(!pci.stdout.contains("Kernel modules:") && !pci.stdout.contains("driver in use"));
    for which in ["lab_sensor", c.modules["lab_sensor"].path.as_str()] {
        let result = r
            .execute_available(&c, &format!("modinfo {which}"), &present)
            .unwrap();
        assert_eq!(result.status, 1);
        assert!(result.stderr.contains("not found"));
    }
    let result = r
        .execute_available(&c, "modprobe lab_sensor", &present)
        .unwrap();
    assert_eq!(result.status, 1);
    assert!(result.stderr.contains("No such file"));
    assert!(!r.loaded.contains_key("lab_sensor"));
    // Dependency loading is deliberately not an invented atomic transaction.
    assert!(r.loaded.contains_key("bus_core") && r.loaded.contains_key("sensor_bus"));
    assert!(r.index.contains("lab_sensor"));
}

#[test]
fn rebuilt_index_contains_only_present_current_kernel_objects() {
    let c = catalogue();
    let mut r = runtime(&c);
    let present = BTreeSet::from(["bus_core".into(), "old_sensor".into()]);
    assert_eq!(
        r.execute_available(&c, "depmod -a", &present)
            .unwrap()
            .status,
        0
    );
    assert_eq!(r.index, BTreeSet::from(["bus_core".into()]));
    let out = r
        .execute_available(&c, "modprobe lab_sensor", &present)
        .unwrap();
    assert_eq!(out.status, 1);
    assert!(out.stderr.contains("not found"));
    assert!(r.loaded.is_empty());
    let old = &c.modules["old_sensor"].path;
    assert_eq!(
        r.execute_available(&c, &format!("modinfo {old}"), &present)
            .unwrap()
            .status,
        0
    );
    let out = r
        .execute_available(&c, &format!("insmod {old}"), &present)
        .unwrap();
    assert_eq!(out.status, 1);
    assert!(out.stderr.contains("Invalid module format"));
    assert!(
        r.execute_available(&c, "dmesg", &present)
            .unwrap()
            .stdout
            .contains("version magic")
    );
}

#[test]
fn loaded_driver_can_outlive_its_file_but_not_survive_unload_and_reload() {
    let c = catalogue();
    let mut r = runtime(&c);
    ok(&mut r, &c, "modprobe audit_link");
    ok(&mut r, &c, "modprobe lab_sensor queues=4");
    let mut present = all(&c);
    present.remove("lab_sensor");
    r.validate(&c).unwrap();
    let loaded = r.loaded.clone();
    assert_eq!(
        r.execute_available(&c, "modprobe lab_sensor", &present)
            .unwrap()
            .status,
        0
    );
    assert_eq!(r.loaded, loaded);
    assert!(
        r.execute_available(&c, "lspci -k", &present)
            .unwrap()
            .stdout
            .contains("Kernel driver in use: lab_sensor")
    );
    let object = &c.modules["lab_sensor"].path;
    assert!(
        r.execute_available(&c, &format!("insmod {object}"), &present)
            .unwrap()
            .stderr
            .contains("No such file")
    );
    assert_eq!(
        r.execute_available(&c, "modprobe -r lab_sensor", &present)
            .unwrap()
            .status,
        0
    );
    assert!(r.loaded.contains_key("audit_link") && r.loaded.contains_key("bus_core"));
    assert_eq!(
        r.execute_available(&c, "modprobe lab_sensor", &present)
            .unwrap()
            .status,
        1
    );
    assert!(!r.loaded.contains_key("lab_sensor"));
    present.insert("lab_sensor".into());
    assert_eq!(
        r.execute_available(&c, "modprobe lab_sensor", &present)
            .unwrap()
            .status,
        0
    );
    assert_eq!(r.loaded["lab_sensor"].parameters["queues"], 1);
}

#[test]
fn absent_dependency_fails_before_its_consumer_and_restoration_allows_real_alternatives() {
    let c = catalogue();
    let mut present = all(&c);
    present.remove("sensor_bus");
    let mut automatic = runtime(&c);
    let failed = automatic
        .execute_available(&c, "modprobe lab_sensor", &present)
        .unwrap();
    assert_eq!(failed.status, 1);
    assert!(failed.stderr.contains("sensor_bus.ko"));
    assert!(automatic.loaded.contains_key("bus_core"));
    assert!(
        !automatic.loaded.contains_key("sensor_bus")
            && !automatic.loaded.contains_key("lab_sensor")
    );
    present.insert("sensor_bus".into());
    assert_eq!(
        automatic
            .execute_available(&c, "modprobe lab_sensor queues=3", &present)
            .unwrap()
            .status,
        0
    );
    let mut manual = runtime(&c);
    for name in ["bus_core", "sensor_bus", "lab_sensor"] {
        let suffix = if name == "lab_sensor" {
            " queues=3"
        } else {
            ""
        };
        assert_eq!(
            manual
                .execute_available(
                    &c,
                    &format!("insmod {}{suffix}", c.modules[name].path),
                    &present
                )
                .unwrap()
                .status,
            0
        );
    }
    assert_eq!(automatic.loaded, manual.loaded);
}

#[test]
fn image_presence_is_not_serialized_runtime_authority_and_unknown_names_refuse() {
    let c = catalogue();
    let mut r = runtime(&c);
    let original = r.clone();
    assert!(
        r.execute_available(&c, "depmod -a", &BTreeSet::from(["foreign_driver".into()]))
            .is_err()
    );
    assert_eq!(r, original);
    let raw = serde_json::to_value(&r).unwrap();
    assert!(raw.get("present").is_none());
    let restored: Runtime = serde_json::from_value(raw).unwrap();
    assert_eq!(restored, original);
    for command in [
        "sh -c anything",
        "insmod /../../etc/passwd",
        "modprobe lab_sensor; touch /outside",
        "modprobe -f lab_sensor",
    ] {
        assert!(
            r.execute_available(&c, command, &all(&c)).is_err(),
            "{command}"
        );
        assert_eq!(r, original);
    }
}
