//! Focused optional-media foundation gates; no curriculum integration yet.
use dlms_learning_lab::{removable_mount as m, storage as s};
use std::collections::BTreeMap;
const DATA: &str = "00000000-0000-4000-8000-000000000001";
const MEDIA: &str = "00000000-0000-4000-8000-000000000002";
fn env(present: &[&str]) -> m::Environment {
    m::Environment {
        inventory: s::Inventory::new(
            BTreeMap::from([
                (
                    "data".into(),
                    s::Volume {
                        uuid: DATA.into(),
                        filesystem: s::Filesystem::Xfs,
                    },
                ),
                (
                    "media".into(),
                    s::Volume {
                        uuid: MEDIA.into(),
                        filesystem: s::Filesystem::Ext4,
                    },
                ),
            ]),
            BTreeMap::from([
                ("/dev/vda1".into(), "data".into()),
                ("/dev/vdb1".into(), "media".into()),
            ]),
            vec!["/project".into(), "/media/audit".into()],
        )
        .unwrap(),
        present: present.iter().map(|id| (*id).into()).collect(),
    }
}
fn text() -> String {
    format!(
        "# Required project; optional audit media\nUUID={DATA} /project xfs rw,auto 0 2\nUUID={MEDIA} /media/audit ext4 ro,auto,nofail 0 0\n"
    )
}
#[test]
fn independent_dependency_policy_is_not_confused_with_mount_outcome() {
    let p = m::Policy::parse(&text()).unwrap();
    assert_eq!(p.entries().len(), 2);
    assert_eq!(p.entries()[0].dependency(), m::Dependency::Required);
    assert!(p.entries()[0].ordered_before_local_fs());
    assert_eq!(p.entries()[1].dependency(), m::Dependency::Wanted);
    assert!(!p.entries()[1].ordered_before_local_fs());
    let all = p.startup(&env(&["data", "media"])).unwrap();
    assert!(all.required_attempts_satisfied());
    assert!(all.rows.iter().all(|r| r.outcome == m::Outcome::Mounted));
    assert_eq!(
        all.state.live()["/media/audit"],
        s::Mount {
            volume: "media".into(),
            read_only: true
        }
    );
    assert!(!all.state.live()["/project"].read_only);
}
#[test]
fn absent_optional_media_does_not_hide_absent_required_storage() {
    let p = m::Policy::parse(&text()).unwrap();
    let optional = p.startup(&env(&["data"])).unwrap();
    assert!(optional.required_attempts_satisfied());
    assert!(
        matches!(&optional.rows[1].outcome,m::Outcome::Failed(e)if e.kind==s::ErrorKind::UnknownSource)
    );
    assert!(!optional.state.live().contains_key("/media/audit"));
    let required = p.startup(&env(&["media"])).unwrap();
    assert!(!required.required_attempts_satisfied());
    assert!(
        matches!(&required.rows[0].outcome,m::Outcome::Failed(e)if e.kind==s::ErrorKind::UnknownSource)
    );
    assert!(required.state.live().contains_key("/media/audit"));
    let absent = p.startup(&env(&[])).unwrap();
    assert!(!absent.required_attempts_satisfied());
    assert_eq!(absent.state.live().len(), 0);
}
#[test]
fn nofail_never_erases_a_wrong_filesystem_or_turns_failure_into_a_mount() {
    let raw = text().replace("/media/audit ext4", "/media/audit xfs");
    let p = m::Policy::parse(&raw).unwrap();
    let r = p.startup(&env(&["data", "media"])).unwrap();
    assert!(r.required_attempts_satisfied());
    assert!(
        matches!(&r.rows[1].outcome,m::Outcome::Failed(e)if e.kind==s::ErrorKind::TypeMismatch)
    );
    assert!(!r.state.live().contains_key("/media/audit"));
    let raw = text().replace(MEDIA, "00000000-0000-4000-8000-000000000099");
    let r = m::Policy::parse(&raw)
        .unwrap()
        .startup(&env(&["data", "media"]))
        .unwrap();
    assert!(r.required_attempts_satisfied());
    assert!(!r.state.live().contains_key("/media/audit"));
    // Actual mounted identity must be assessed by dependent assignments; target readiness alone is inadequate.
}
#[test]
fn manual_requests_work_for_noauto_and_fail_honestly_for_absence_even_with_nofail() {
    let p = m::Policy::parse(&text().replace("ro,auto,nofail", "ro,noauto,nofail")).unwrap();
    let e = env(&["data", "media"]);
    let mut r = p.startup(&e).unwrap();
    assert_eq!(r.rows[1].dependency, m::Dependency::NotEnrolled);
    assert_eq!(r.rows[1].outcome, m::Outcome::Manual);
    assert!(r.required_attempts_satisfied());
    assert!(!r.state.live().contains_key("/media/audit"));
    assert_eq!(
        p.manual_mount(&e, &mut r.state, "/media/audit").unwrap(),
        s::MountOutcome::Mounted
    );
    assert!(r.state.live()["/media/audit"].read_only);
    r.state.umount("/media/audit").unwrap();
    assert!(!r.state.live().contains_key("/media/audit"));
    let e = env(&["data"]);
    let mut r = p.startup(&e).unwrap();
    let before = r.state.clone();
    assert_eq!(
        p.manual_mount(&e, &mut r.state, "/media/audit")
            .unwrap_err()
            .kind,
        s::ErrorKind::UnknownSource
    );
    assert_eq!(r.state, before);
}
#[test]
fn supported_option_order_and_comments_match_existing_core_without_changing_it() {
    let p = m::Policy::parse(&text().replace("ro,auto,nofail", "rw,noauto,ro,auto,nofail,nofail"))
        .unwrap();
    assert!(p.entries()[1].mount.read_only);
    assert!(p.entries()[1].mount.automatic);
    assert!(p.entries()[1].optional);
    assert_eq!(
        s::parse(&text()).unwrap_err().kind,
        s::ErrorKind::Unsupported
    );
    let without = text().replace(",nofail", "");
    let original = s::parse(&without).unwrap();
    let adapter = m::Policy::parse(&without).unwrap();
    assert_eq!(
        adapter
            .entries()
            .iter()
            .map(|r| r.mount.clone())
            .collect::<Vec<_>>(),
        original
    );
}
#[test]
fn wrong_or_unsupported_configuration_is_not_ignored_by_nofail() {
    for raw in [
        text().replace("ro,auto,nofail", "ro,user,nofail"),
        text().replace("ro,auto,nofail", "ro,x-systemd.automount,nofail"),
        text().replace("ro,auto,nofail", "ro,nodev,nofail"),
        text().replace("ro,auto,nofail", "ro,unknown,nofail"),
    ] {
        assert_eq!(
            m::Policy::parse(&raw).unwrap_err().kind,
            s::ErrorKind::Unsupported
        );
    }
    for raw in [
        text().replace("/media/audit", "/media/../audit"),
        text().replace("0 2", "0"),
        text().replace("/media/audit", "/project"),
        text().replace("nofail", "nofail\u{b}"),
    ] {
        assert!(m::Policy::parse(&raw).is_err(), "{raw:?}");
    }
}
#[test]
fn literal_virtual_sources_and_presence_maps_cannot_access_other_contexts() {
    let p = m::Policy::parse(&text()).unwrap();
    let e = env(&["data", "media"]);
    let mut r = p.startup(&e).unwrap();
    let before = r.state.clone();
    assert!(
        p.manual_mount(&e, &mut r.state, "/home/owner/private")
            .is_err()
    );
    assert_eq!(r.state, before);
    let mut bad = e.clone();
    bad.present.insert("foreign".into());
    assert!(bad.validate().is_err());
    assert!(bad.blkid().is_err());
    assert!(p.startup(&bad).is_err());
    let mut aliases = e.inventory.devices().clone();
    let a = aliases["/dev/vda1"].clone();
    let b = aliases["/dev/vdb1"].clone();
    aliases.insert("/dev/vda1".into(), b);
    aliases.insert("/dev/vdb1".into(), a);
    let changed = m::Environment {
        inventory: s::Inventory::new(
            e.inventory.volumes().clone(),
            aliases,
            e.inventory.mountpoints().iter().cloned().collect(),
        )
        .unwrap(),
        present: e.present.clone(),
    };
    assert!(
        p.manual_mount(&changed, &mut r.state, "/media/audit")
            .is_err()
    );
    assert_eq!(r.state, before);
    let next = p.startup(&changed).unwrap();
    assert!(next.rows.iter().all(|r| r.outcome == m::Outcome::Mounted));
    assert_eq!(next.state.live()["/project"].volume, "data");
}
#[test]
fn visible_inventory_lists_only_supplied_present_media() {
    let e = env(&["data"]);
    let listing = e.blkid().unwrap();
    assert!(listing.contains(DATA));
    assert!(!listing.contains(MEDIA));
    assert!(listing.contains("/dev/vda1"));
    assert!(!listing.contains("/dev/vdb1"));
    assert_eq!(env(&[]).blkid().unwrap(), "");
}
#[test]
fn input_limits_apply_before_normalization_and_never_truncate_a_field() {
    assert_eq!(
        m::Policy::parse(&" ".repeat(s::MAX_CONFIG_BYTES + 1))
            .unwrap_err()
            .kind,
        s::ErrorKind::Limit
    );
    assert_eq!(
        m::Policy::parse(&"\n".repeat(129)).unwrap_err().kind,
        s::ErrorKind::Limit
    );
    let options = std::iter::repeat_n("nofail", 17)
        .collect::<Vec<_>>()
        .join(",");
    let raw = text().replace("ro,auto,nofail", &options);
    assert_eq!(
        m::Policy::parse(&raw).unwrap_err().kind,
        s::ErrorKind::Limit
    );
    let row = format!("UUID={MEDIA} /media/audit ext4 nofail 0 123");
    let prefix = format!("#{}\n", "x".repeat(s::MAX_CONFIG_BYTES - row.len() - 2));
    let raw = prefix + &row;
    assert_eq!(raw.len(), s::MAX_CONFIG_BYTES);
    assert_eq!(
        m::Policy::parse(&raw).unwrap_err().kind,
        s::ErrorKind::Limit
    );
    let valid = m::Policy::parse(&text()).unwrap();
    assert_eq!(valid.entries()[1].mount.pass, 0);
}
#[test]
fn serialized_presence_is_bounded_and_rejects_duplicate_or_foreign_identities() {
    let original = env(&["data", "media"]);
    let raw = serde_json::to_string(&original).unwrap();
    let copy: m::Environment = serde_json::from_str(&raw).unwrap();
    assert_eq!(copy, original);
    copy.validate().unwrap();
    let mut value = serde_json::to_value(&original).unwrap();
    value["present"] = serde_json::json!(["data", "data"]);
    assert!(serde_json::from_value::<m::Environment>(value).is_err());
    let mut value = serde_json::to_value(&original).unwrap();
    value["present"] = serde_json::json!(["a", "b", "c", "d", "e", "f", "g", "h", "i"]);
    assert!(serde_json::from_value::<m::Environment>(value).is_err());
    let mut value = serde_json::to_value(&original).unwrap();
    value["present"] = serde_json::json!(["x".repeat(49)]);
    assert!(serde_json::from_value::<m::Environment>(value).is_err());
    let mut value = serde_json::to_value(&original).unwrap();
    value["foreign"] = true.into();
    assert!(serde_json::from_value::<m::Environment>(value).is_err());
}
