//! Fixed metadata-only model proof; no real RPM command/package/host operation.
use dlms_learning_lab::rpm_packages as core;
#[path = "support/rpm_metadata_fixture.rs"]
mod f;
#[test]
fn metadata_restore_uses_installed_record_and_preserves_every_content_cache_and_neighbor() {
    let before = f::state();
    let expected = before.installed(&f::identity("site")).unwrap().unwrap();
    assert_eq!(expected.evr.as_str(), "1.0-1");
    let after = before.restored_metadata(&f::identity("site")).unwrap();
    assert_eq!(after.catalog(), before.catalog());
    assert_eq!(after.runtime().installed, before.runtime().installed);
    assert_eq!(after.runtime().available, before.runtime().available);
    assert_eq!(after.runtime().cached, before.runtime().cached);
    for (path, old) in &before.runtime().files {
        let new = &after.runtime().files[path];
        assert_eq!(new.contents, old.contents);
        if let Some(p) = expected.files.get(path) {
            assert!(core::same_recorded_metadata(new, &p.file));
        } else {
            assert_eq!(new, old);
        }
    }
    assert_eq!(
        after.runtime().files["/etc/site.conf"].contents,
        "route=local\n"
    );
    assert_eq!(
        after.restored_metadata(&f::identity("site")).unwrap(),
        after
    );
    assert_eq!(
        before.dependency_issues().unwrap(),
        after.dependency_issues().unwrap()
    );
    assert_ne!(
        after.runtime().files["/etc/site.conf"],
        before.catalog().archives["site-v2"].files["/etc/site.conf"].file
    );
}
#[test]
fn metadata_restore_is_not_content_repair_or_pristine_config_reinstallation() {
    let before = f::state();
    let after = before.restored_metadata(&f::identity("renderer")).unwrap();
    let issues = after.file_issues(&f::identity("renderer")).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].differences, [core::Difference::Contents]);
    assert_eq!(
        after.runtime().files["/etc/site.conf"],
        before.runtime().files["/etc/site.conf"]
    );
    let mut repaired = before.restored_metadata(&f::identity("site")).unwrap();
    let old = repaired.clone();
    let e = repaired
        .apply(&core::Action::ReinstallExact {
            archive: "site-v1".into(),
        })
        .unwrap_err();
    assert_eq!(e.code, core::Code::UnsupportedSubset);
    assert_eq!(repaired, old);
    let mut app = after.clone();
    app.apply(&core::Action::ReinstallExact {
        archive: "renderer-v1".into(),
    })
    .unwrap();
    assert!(
        app.file_issues(&f::identity("renderer"))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn metadata_restore_refuses_missing_paths_and_uninstalled_identity_without_partial_work() {
    let before = f::state();
    let mut runtime = before.runtime().clone();
    runtime.files.remove("/usr/share/site.txt");
    let missing = core::State::new(before.catalog().clone(), runtime).unwrap();
    let exact = missing.clone();
    let e = missing.restored_metadata(&f::identity("site")).unwrap_err();
    assert_eq!(e.code, core::Code::UnsupportedSubset);
    assert!(e.message.contains("Missing payload"));
    assert_eq!(missing, exact);
    assert_eq!(missing.runtime().files["/etc/site.conf"].mode, 0o640);
    let e = before
        .restored_metadata(&f::identity("absent"))
        .unwrap_err();
    assert_eq!(e.code, core::Code::NotInstalled);
    let wrong = core::Identity {
        name: "site".into(),
        arch: core::Architecture::X86_64,
    };
    assert_eq!(
        before.restored_metadata(&wrong).unwrap_err().code,
        core::Code::NotInstalled
    );
    assert_eq!(before, f::state());
}
#[test]
fn metadata_restore_remains_validated_bounded_and_round_trips_without_new_state_authority() {
    let before = f::state();
    let after = before.restored_metadata(&f::identity("site")).unwrap();
    let raw = serde_json::to_vec(&after).unwrap();
    assert_eq!(core::State::decode(&raw).unwrap(), after);
    let mut poison = serde_json::to_value(&before).unwrap();
    poison["allow_metadata_restore"] = true.into();
    assert!(serde_json::from_value::<core::State>(poison).is_err());
    let mut poison = serde_json::to_value(&before).unwrap();
    poison["runtime"]["files"]["/etc/site.conf"]["mode"] = 4095.into();
    assert!(serde_json::from_value::<core::State>(poison).is_err());
    let mut poison = serde_json::to_value(&before).unwrap();
    poison["runtime"]["files"]["/etc/site.conf"]["owner"] = "x".repeat(33).into();
    assert!(serde_json::from_value::<core::State>(poison).is_err());
    let mut poison = serde_json::to_value(&before).unwrap();
    poison["runtime"]["files"]["/outside"] = poison["runtime"]["files"]["/etc/site.conf"].clone();
    assert!(serde_json::from_value::<core::State>(poison).is_err());
}
