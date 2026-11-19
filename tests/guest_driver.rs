#[path = "support/guest_driver_fixture.rs"]
mod fixture;
use dlms_learning_lab::{guest_driver::*, rpm_packages as rpm};
use fixture::*;

#[test]
fn installed_payload_lookup_index_and_loaded_driver_are_three_separate_states() {
    let f = facts();
    let mut r = runtime(&f);
    assert!(!r.present(&f).unwrap().contains("lab_sensor"));
    assert_eq!(r.run(&f, "modprobe lab_sensor").unwrap().status, 1);
    let before = r.modules.clone();
    install(&mut r, &f);
    assert_eq!(
        r.modules, before,
        "No post-install script, hidden depmod or loading"
    );
    assert!(r.present(&f).unwrap().contains("lab_sensor"));
    assert_eq!(r.run(&f, "modprobe lab_sensor").unwrap().status, 1);
    let path = &f.modules.modules["lab_sensor"].path;
    assert_eq!(r.run(&f, &format!("modinfo {path}")).unwrap().status, 0);
    assert_eq!(r.run(&f, "depmod -a").unwrap().status, 0);
    assert_eq!(r.run(&f, "modprobe lab_sensor").unwrap().status, 0);
    assert!(
        r.run(&f, "lspci -k")
            .unwrap()
            .stdout
            .contains("Kernel driver in use: lab_sensor")
    );
}

#[test]
fn loaded_state_is_insufficient_when_managed_object_has_gone_missing() {
    let f = facts();
    let mut r = runtime(&f);
    ready(&mut r, &f);
    r.remove_object(&f, "lab_sensor").unwrap();
    assert_eq!(r.run(&f, "modprobe lab_sensor").unwrap().status, 0);
    let before = r.clone();
    let trial = r.fresh_request(&f, "lab_sensor").unwrap();
    assert_eq!(trial.status, 1);
    assert!(trial.stderr.contains("No such file"));
    assert_eq!(r, before);
    // Rebuilding while missing drops the stale lookup; reinstall is not depmod.
    r.run(&f, "depmod -a").unwrap();
    let modules = r.modules.clone();
    r.package(
        &f,
        &rpm::Action::ReinstallExact {
            archive: "support-current".into(),
        },
    )
    .unwrap();
    assert_eq!(r.modules, modules);
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 1);
    r.run(&f, "depmod -a").unwrap();
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 0);
    assert!(r.modules.loaded.contains_key("audit_link"));
}

#[test]
fn wrong_kernel_package_refuses_without_mutation_and_matching_upgrade_repairs_an_incomplete_image()
{
    let f = facts();
    let mut r = runtime(&f);
    let before = r.clone();
    assert!(
        r.package(
            &f,
            &rpm::Action::InstallSet {
                archives: vec!["support-old".into()]
            }
        )
        .is_err()
    );
    assert_eq!(r, before);
    // Explicit supplied broken image, not an invented successful transaction.
    let old = &f.packages.archives["support-old"];
    r.packages.installed.push(rpm::Installed {
        identity: old.identity.clone(),
        archive: "support-old".into(),
    });
    r.packages
        .files
        .extend(old.files.iter().map(|(p, f)| (p.clone(), f.file.clone())));
    r.validate(&f).unwrap();
    let object = &f.modules.modules["old_sensor"].path;
    assert!(
        r.run(&f, &format!("insmod {object}"))
            .unwrap()
            .stderr
            .contains("Invalid module format")
    );
    r.package(
        &f,
        &rpm::Action::UpgradeSet {
            archives: vec!["support-current".into()],
        },
    )
    .unwrap();
    assert!(!r.present(&f).unwrap().contains("old_sensor"));
    assert!(r.present(&f).unwrap().contains("lab_sensor"));
    assert!(!r.modules.loaded.contains_key("lab_sensor"));
    r.run(&f, "depmod -a").unwrap();
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 0);
}

#[test]
fn a_working_package_for_a_different_adapter_does_not_bind_the_requested_device() {
    let f = facts();
    let mut r = runtime(&f);
    r.package(
        &f,
        &rpm::Action::InstallSet {
            archives: vec!["support-other-device".into()],
        },
    )
    .unwrap();
    r.run(&f, "depmod -a").unwrap();
    assert_eq!(r.run(&f, "modprobe emulated_net").unwrap().status, 0);
    let observed = r.run(&f, "lspci -k").unwrap();
    assert!(r.modules.loaded.contains_key("emulated_net"));
    assert!(!observed.stdout.contains("Kernel driver in use: lab_sensor"));
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 1);
    install(&mut r, &f);
    r.run(&f, "depmod -a").unwrap();
    for n in ["sensor_bus", "lab_sensor"] {
        assert_eq!(
            r.run(&f, &format!("insmod {}", f.modules.modules[n].path))
                .unwrap()
                .status,
            0
        );
    }
    assert!(
        r.run(&f, "lspci -k")
            .unwrap()
            .stdout
            .contains("Kernel driver in use: lab_sensor")
    );
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 0);
    assert!(r.modules.loaded.contains_key("audit_link"));
}

#[test]
fn erasing_package_retains_running_driver_but_future_request_fails_and_reinstall_paths_are_equivalent()
 {
    let f = facts();
    let mut r = runtime(&f);
    ready(&mut r, &f);
    let loaded = r.modules.loaded.clone();
    r.package(
        &f,
        &rpm::Action::EraseSet {
            packages: vec![f.packages.archives["support-current"].identity.clone()],
        },
    )
    .unwrap();
    assert_eq!(r.modules.loaded, loaded);
    assert_eq!(r.run(&f, "modprobe lab_sensor").unwrap().status, 0);
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 1);
    let mut alternative = r.clone();
    install(&mut r, &f);
    alternative
        .package(
            &f,
            &rpm::Action::UpgradeSet {
                archives: vec!["support-current".into()],
            },
        )
        .unwrap();
    assert_eq!(r, alternative);
    assert_eq!(r.fresh_request(&f, "lab_sensor").unwrap().status, 0);
    let json = serde_json::to_vec(&r).unwrap();
    let restored: Runtime = serde_json::from_slice(&json).unwrap();
    restored.validate(&f).unwrap();
    assert_eq!(restored, r);
}

#[test]
fn malformed_source_changed_objects_and_shell_escape_are_refused_honestly() {
    let f = facts();
    let r = runtime(&f);
    let mut bad = f.clone();
    bad.objects[1] = bad.objects[0].clone();
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.packages
        .archives
        .get_mut("support-current")
        .unwrap()
        .identity
        .arch = rpm::Architecture::Noarch;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.packages.allowed_paths.push("/etc/passwd".into());
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.objects[0].contents = "\0binary".into();
    assert!(bad.validate().is_err());
    let mut raw = serde_json::to_value(&f).unwrap();
    raw["packages"]["archives"]["support-current"]["postinstall"] = serde_json::json!("sh script");
    assert!(serde_json::from_value::<Facts>(raw).is_err());
    let mut bad = r.clone();
    let path = f.modules.modules["bus_core"].path.clone();
    bad.packages.files.get_mut(&path).unwrap().contents = "unrecognized object".into();
    assert!(bad.validate(&f).unwrap_err().contains("outside"));
    for input in [
        "rpm -i unknown.rpm",
        "sh -c anything",
        "insmod /../../etc/passwd",
        "modprobe -f lab_sensor",
    ] {
        let mut actual = r.clone();
        assert!(actual.run(&f, input).is_err(), "{input}");
        assert_eq!(actual, r);
    }
    assert!(r.fresh_request(&f, "lab_sensor; touch /outside").is_err());
    let mut raw = serde_json::to_value(&f).unwrap();
    raw["objects"] = serde_json::json!(vec![f.objects[0].clone(); 17]);
    assert!(serde_json::from_value::<Facts>(raw).is_err());
}

fn install(r: &mut Runtime, f: &Facts) {
    r.package(
        f,
        &rpm::Action::InstallSet {
            archives: vec!["support-current".into()],
        },
    )
    .unwrap();
}
fn ready(r: &mut Runtime, f: &Facts) {
    install(r, f);
    assert_eq!(r.run(f, "depmod -a").unwrap().status, 0);
    assert_eq!(r.run(f, "modprobe lab_sensor").unwrap().status, 0);
}
