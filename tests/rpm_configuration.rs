//! New opt-in content decisions and unchanged legacy refusals; no host RPM execution.
use dlms_learning_lab::{rpm_configuration as cfg, rpm_packages as rpm};
#[path = "support/rpm_metadata_fixture.rs"]
mod f;
#[path = "support/rpm_configuration_fixture.rs"]
mod shared;
use shared::{PATH, fixture};
fn upgrade() -> rpm::Action {
    rpm::Action::UpgradeSet {
        archives: vec!["site-v2".into()],
    }
}
#[test]
fn noreplace_preserves_local_and_stages_new_without_changing_neighbors() {
    let before = fixture(
        rpm::FileKind::ConfigNoreplace,
        Some("route=local\n"),
        "route=vendor-v2\n",
    );
    let snapshot = before.clone();
    let result = cfg::transition(&before, &upgrade()).unwrap();
    let after = &result.state;
    assert_eq!(before, snapshot);
    assert_eq!(after.catalog(), before.catalog());
    assert_eq!(after.runtime().files[PATH], before.runtime().files[PATH]);
    assert_eq!(
        after.runtime().files[&format!("{PATH}.rpmnew")],
        before.catalog().archives["site-v2"].files[PATH].file
    );
    assert_eq!(
        result.configurations[0].disposition,
        cfg::Disposition::StageIncoming
    );
    assert_eq!(
        after
            .runtime()
            .installed
            .iter()
            .find(|i| i.identity.name == "site")
            .unwrap()
            .archive,
        "site-v2"
    );
    for (path, file) in &before.runtime().files {
        if path != PATH
            && !before.catalog().archives["site-v1"]
                .files
                .contains_key(path)
        {
            assert_eq!(&after.runtime().files[path], file);
        }
    }
    assert_eq!(after.runtime().available, before.runtime().available);
    assert_eq!(after.runtime().cached, before.runtime().cached);
}
#[test]
fn ordinary_config_preserves_old_local_as_rpmsave_and_installs_incoming() {
    let before = fixture(
        rpm::FileKind::Config,
        Some("route=site\n"),
        "route=new-vendor\n",
    );
    let r = cfg::transition(&before, &upgrade()).unwrap();
    assert_eq!(r.configurations[0].disposition, cfg::Disposition::SaveLocal);
    assert_eq!(
        r.state.runtime().files[&format!("{PATH}.rpmsave")],
        before.runtime().files[PATH]
    );
    assert_eq!(
        r.state.runtime().files[PATH],
        before.catalog().archives["site-v2"].files[PATH].file
    );
}
#[test]
fn unchanged_incoming_default_keeps_site_contents_without_any_sidecar() {
    for kind in [rpm::FileKind::Config, rpm::FileKind::ConfigNoreplace] {
        let before = fixture(kind, Some("route=site\n"), "route=default\n");
        let r = cfg::transition(&before, &upgrade()).unwrap();
        assert_eq!(r.configurations[0].disposition, cfg::Disposition::KeepLocal);
        assert_eq!(r.state.runtime().files[PATH], before.runtime().files[PATH]);
        assert!(r.configurations[0].sidecar.is_none());
    }
}
#[test]
fn unmodified_already_incoming_and_missing_files_install_without_sidecars() {
    for local in [Some("route=default\n"), Some("route=new\n"), None] {
        let before = fixture(rpm::FileKind::ConfigNoreplace, local, "route=new\n");
        let r = cfg::transition(&before, &upgrade()).unwrap();
        assert_eq!(
            r.state.runtime().files[PATH],
            before.catalog().archives["site-v2"].files[PATH].file
        );
        assert_eq!(
            r.configurations[0].disposition,
            cfg::Disposition::InstallIncoming
        );
        assert!(r.configurations[0].sidecar.is_none());
    }
}
#[test]
fn erase_preserves_only_locally_changed_configuration_and_drops_ownership() {
    for local in [Some("route=local\n"), Some("route=default\n"), None] {
        let before = fixture(rpm::FileKind::ConfigNoreplace, local, "route=new\n");
        let r = cfg::transition(
            &before,
            &rpm::Action::EraseSet {
                packages: vec![f::identity("site")],
            },
        )
        .unwrap();
        assert!(!r.state.runtime().files.contains_key(PATH));
        assert!(
            !r.state
                .runtime()
                .installed
                .iter()
                .any(|p| p.identity.name == "site")
        );
        if local == Some("route=local\n") {
            assert_eq!(
                r.state.runtime().files[&format!("{PATH}.rpmsave")],
                before.runtime().files[PATH]
            );
            assert_eq!(r.configurations[0].disposition, cfg::Disposition::SaveLocal);
        } else {
            assert!(r.configurations[0].sidecar.is_none());
        }
    }
}
#[test]
fn default_rpm_entry_still_refuses_modified_and_missing_configuration() {
    for local in [Some("route=local\n"), None] {
        let mut before = fixture(rpm::FileKind::ConfigNoreplace, local, "route=new\n");
        let snapshot = before.clone();
        assert_eq!(
            before.apply(&upgrade()).unwrap_err().code,
            rpm::Code::UnsupportedSubset
        );
        assert_eq!(before, snapshot);
        assert!(cfg::transition(&before, &upgrade()).is_ok());
    }
}
#[test]
fn metadata_flag_changes_and_existing_sidecars_refuse_atomically() {
    let before = fixture(
        rpm::FileKind::ConfigNoreplace,
        Some("route=local\n"),
        "route=new\n",
    );
    for fault in 0..5 {
        let mut c = before.catalog().clone();
        let mut r = before.runtime().clone();
        match fault {
            0 => r.files.get_mut(PATH).unwrap().mode = 0o600,
            1 => {
                c.archives
                    .get_mut("site-v2")
                    .unwrap()
                    .files
                    .get_mut(PATH)
                    .unwrap()
                    .kind = rpm::FileKind::Config
            }
            2 => {
                r.files
                    .insert(format!("{PATH}.rpmnew"), r.files[PATH].clone());
            }
            3 => c.allowed_paths.retain(|p| p != &format!("{PATH}.rpmnew")),
            4 => {
                c.archives
                    .get_mut("site-v2")
                    .unwrap()
                    .files
                    .get_mut(PATH)
                    .unwrap()
                    .file
                    .owner = "other".into()
            }
            _ => unreachable!(),
        }
        let bad = rpm::State::new(c, r).unwrap();
        let snapshot = bad.clone();
        let e = cfg::transition(&bad, &upgrade()).unwrap_err();
        assert_eq!(e.code, rpm::Code::UnsupportedSubset);
        assert_eq!(bad, snapshot);
    }
}
#[test]
fn dependency_failure_and_unavailable_archives_do_not_publish_normalized_defaults() {
    let before = fixture(
        rpm::FileKind::ConfigNoreplace,
        Some("route=local\n"),
        "route=new\n",
    );
    let mut c = before.catalog().clone();
    c.archives
        .get_mut("site-v2")
        .unwrap()
        .requires
        .push(rpm::Requirement {
            capability: "missing-api".into(),
            constraint: None,
        });
    let bad = rpm::State::new(c, before.runtime().clone()).unwrap();
    let snap = bad.clone();
    assert!(cfg::transition(&bad, &upgrade()).is_err());
    assert_eq!(bad, snap);
    let mut r = before.runtime().clone();
    r.available.retain(|a| a != "site-v2");
    let bad = rpm::State::new(before.catalog().clone(), r).unwrap();
    assert_eq!(
        cfg::transition(&bad, &upgrade()).unwrap_err().code,
        rpm::Code::Unavailable
    );
    assert_eq!(bad.runtime().files[PATH].contents, "route=local\n");
}
#[test]
fn action_bounds_and_roundtrip_do_not_enable_any_legacy_feature() {
    let before = fixture(
        rpm::FileKind::ConfigNoreplace,
        Some("route=local\n"),
        "route=new\n",
    );
    for action in [
        rpm::Action::UpgradeSet {
            archives: vec!["x".repeat(5000)],
        },
        rpm::Action::UpgradeSet {
            archives: vec!["site-v2".into(); 9],
        },
    ] {
        assert!(cfg::transition(&before, &action).is_err());
    }
    let after = cfg::transition(&before, &upgrade()).unwrap().state;
    assert_eq!(
        rpm::State::decode(&serde_json::to_vec(&after).unwrap()).unwrap(),
        after
    );
    let mut poisoned = serde_json::to_value(&before).unwrap();
    poisoned["allow_configuration_updates"] = true.into();
    assert!(serde_json::from_value::<rpm::State>(poisoned).is_err());
}
