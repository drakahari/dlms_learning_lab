//! Pure supplied virtual filesystems only. No host mounts, devices or processes.
use dlms_learning_lab::{
    lvm::{
        self,
        mounts::{self, Context, Filesystem, Mounted, Mountpoint},
    },
    storage,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
const UUID_A: &str = "12345678-0000-4000-8000-000000000001";
const UUID_B: &str = "12345678-0000-4000-8000-000000000002";
fn world() -> lvm::State {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let s: lvm::State =
        serde_json::from_value(p["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap();
    s.validate().unwrap();
    s
}
fn context(s: &lvm::State) -> Context {
    let ids = &s.identity.as_ref().unwrap().volumes;
    Context {
        filesystems: vec![
            Filesystem {
                original_volume: ids["team/draft"],
                uuid: UUID_A.into(),
            },
            Filesystem {
                original_volume: ids["team/reference"],
                uuid: UUID_B.into(),
            },
        ],
        mountpoints: vec![
            Mountpoint("/srv/report".into()),
            Mountpoint("/srv/audit".into()),
        ],
    }
}
fn policy(source: &str) -> String {
    format!("{source} /srv/report ext4 defaults 0 2\n")
}
fn fixture() -> (lvm::State, Context, Mounted) {
    let s = world();
    let c = context(&s);
    let m = Mounted::new(&c, &s, policy("/dev/team/draft"), BTreeMap::new()).unwrap();
    (s, c, m)
}
fn refuse(m: &mut Mounted, c: &Context, s: &mut lvm::State, cmd: &str) -> String {
    let before = (m.clone(), s.clone());
    let e = m.execute_lvm(c, s, cmd).expect_err(cmd);
    assert_eq!((m.clone(), s.clone()), before, "{cmd}");
    e
}
#[test]
fn saved_policy_live_mount_and_actual_contents_are_distinct() {
    let (s, c, mut m) = fixture();
    assert!(
        m.read_path(&c, &s, "/srv/report/keep.txt")
            .unwrap_err()
            .contains("No filesystem")
    );
    assert_eq!(
        m.mount(&c, &s, "/srv/report").unwrap(),
        storage::MountOutcome::Mounted
    );
    assert_eq!(
        m.read_path(&c, &s, "/srv/report/keep.txt").unwrap(),
        s.read_file("team/draft", "/keep.txt").unwrap()
    );
    m.save(&c, &s, policy("/dev/team/reference")).unwrap();
    assert!(
        m.read_path(&c, &s, "/srv/report/keep.txt")
            .unwrap()
            .starts_with("team/draft:")
    );
    let before = m.clone();
    assert!(m.mount(&c, &s, "/srv/report").is_err());
    assert_eq!(m, before);
    m.umount(&c, &s, "/srv/report").unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    assert!(
        m.read_path(&c, &s, "/srv/report/keep.txt")
            .unwrap()
            .starts_with("team/reference:")
    );
}
#[test]
fn canonical_name_and_explicit_filesystem_uuid_are_alternate_sources() {
    let (s, c, base) = fixture();
    let mut results = Vec::new();
    for source in ["/dev/team/draft".to_owned(), format!("UUID={UUID_A}")] {
        let mut m = base.clone();
        m.save(&c, &s, policy(&source)).unwrap();
        assert!(m.storage.validate_configuration().is_empty());
        m.mount(&c, &s, "/srv/report").unwrap();
        results.push((
            m.storage.live().clone(),
            m.read_path(&c, &s, "/srv/report/keep.txt")
                .unwrap()
                .to_owned(),
        ));
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0].0["/srv/report"].volume, mounts::key(4));
}
#[test]
fn inactive_mount_refuses_then_actual_activation_allows_mounting() {
    let (mut s, c, mut m) = fixture();
    s.activation.as_mut().unwrap().open.clear();
    m.execute_lvm(&c, &mut s, "vgchange -an team").unwrap();
    let before = m.clone();
    assert!(
        m.mount(&c, &s, "/srv/report")
            .unwrap_err()
            .contains("inactive")
    );
    assert_eq!(m, before);
    m.execute_lvm(&c, &mut s, "vgchange -ay team").unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    assert!(refuse(&mut m, &c, &mut s, "vgchange -an team").contains("unmount"));
    m.umount(&c, &s, "/srv/report").unwrap();
    m.execute_lvm(&c, &mut s, "vgchange -an team").unwrap();
    assert!(!lvm::activation::is_active(&s, "team/draft").unwrap());
}
#[test]
fn current_mount_keeps_original_files_through_both_name_changes() {
    let (mut s, c, mut m) = fixture();
    m.mount(&c, &s, "/srv/report").unwrap();
    let actual = m
        .read_path(&c, &s, "/srv/report/keep.txt")
        .unwrap()
        .to_owned();
    for cmd in [
        "vgrename team production",
        "lvrename production draft records",
    ] {
        m.execute_lvm(&c, &mut s, cmd).unwrap();
        m.validate(&c, &s).unwrap();
        assert_eq!(m.read_path(&c, &s, "/srv/report/keep.txt").unwrap(), actual);
    }
    assert!(!m.storage.validate_configuration().is_empty());
    assert!(
        m.storage
            .inventory()
            .devices()
            .contains_key("/dev/production/records")
    );
    m.umount(&c, &s, "/srv/report").unwrap();
    let before = m.clone();
    assert!(m.mount(&c, &s, "/srv/report").is_err());
    assert_eq!(m, before);
    m.save(&c, &s, policy("/dev/production/records")).unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    assert_eq!(m.read_path(&c, &s, "/srv/report/keep.txt").unwrap(), actual);
}
#[test]
fn uuid_policy_remains_valid_after_rename_and_unmount_remount() {
    let (mut s, c, mut m) = fixture();
    m.save(&c, &s, policy(&format!("UUID={UUID_A}"))).unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    let text = m.storage.configuration().to_owned();
    m.execute_lvm(&c, &mut s, "lvrename team draft records")
        .unwrap();
    m.execute_lvm(&c, &mut s, "vgrename team production")
        .unwrap();
    assert_eq!(m.storage.configuration(), text);
    assert!(m.storage.validate_configuration().is_empty());
    m.umount(&c, &s, "/srv/report").unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    assert!(
        m.read_path(&c, &s, "/srv/report/keep.txt")
            .unwrap()
            .starts_with("team/draft:")
    );
}
#[test]
fn wrong_type_unknown_source_and_duplicate_mounts_do_not_change_live_state() {
    let (s, c, mut m) = fixture();
    for text in [
        "/dev/team/draft /srv/report xfs defaults 0 2\n".to_owned(),
        policy("/dev/team/absent"),
        policy("UUID=12345678-0000-4000-8000-999999999999"),
        policy("/dev/mapper/team-draft"),
    ] {
        m.save(&c, &s, text).unwrap();
        assert!(!m.storage.validate_configuration().is_empty());
        let before = m.clone();
        assert!(m.mount(&c, &s, "/srv/report").is_err());
        assert_eq!(m, before);
    }
    m.save(
        &c,
        &s,
        format!(
            "{} /dev/team/draft /srv/audit ext4 defaults 0 2\n",
            policy("/dev/team/draft")
        ),
    )
    .unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    let before = m.clone();
    assert!(m.mount(&c, &s, "/srv/audit").is_err());
    assert_eq!(m, before);
}
#[test]
fn malformed_saved_work_roundtrips_and_is_repairable_without_credit_or_execution() {
    let (s, c, mut m) = fixture();
    m.save(&c, &s, "unfinished UUID=\n".into()).unwrap();
    assert!(!m.storage.validate_configuration().is_empty());
    let bytes = serde_json::to_vec(&m).unwrap();
    let mut restored: Mounted = serde_json::from_slice(&bytes).unwrap();
    restored.validate(&c, &s).unwrap();
    assert_eq!(restored, m);
    restored.save(&c, &s, policy("/dev/team/draft")).unwrap();
    restored.mount(&c, &s, "/srv/report").unwrap();
    assert!(restored.storage.validate_configuration().is_empty());
    assert!(fixture().2.storage.live().is_empty());
}
#[test]
fn path_traversal_unknown_paths_and_underlying_directory_are_not_faked() {
    let (s, c, mut m) = fixture();
    m.mount(&c, &s, "/srv/report").unwrap();
    for path in [
        "/etc/passwd",
        "/srv/report/../audit/keep.txt",
        "/srv/report//keep.txt",
        "/srv/report/./keep.txt",
        "/srv/report/keep.txt\n",
        "/srv/reporter/keep.txt",
        "srv/report/keep.txt",
        "/srv/report",
        "/srv/report/missing",
        "/srv/audit/keep.txt",
    ] {
        assert!(m.read_path(&c, &s, path).is_err(), "{path}");
    }
    let mut changed = s.clone();
    changed
        .groups
        .get_mut("team")
        .unwrap()
        .volumes
        .get_mut("draft")
        .unwrap()
        .filesystem
        .as_mut()
        .unwrap()
        .files
        .get_mut("/keep.txt")
        .unwrap()
        .contents = "changed virtual contents\n".into();
    assert_eq!(
        m.read_path(&c, &changed, "/srv/report/keep.txt").unwrap(),
        "changed virtual contents\n"
    );
}
#[test]
fn context_rejects_raw_missing_duplicate_and_unsupported_profiles() {
    let (s, c, _) = fixture();
    let mut bad = c.clone();
    bad.filesystems[0].original_volume = 3;
    assert!(bad.validate(&s).unwrap_err().contains("raw"));
    bad = c.clone();
    bad.filesystems[0].original_volume = 99;
    assert!(bad.validate(&s).is_err());
    bad = c.clone();
    bad.filesystems[1].original_volume = bad.filesystems[0].original_volume;
    assert!(bad.validate(&s).is_err());
    bad = c.clone();
    bad.filesystems[1].uuid = bad.filesystems[0].uuid.clone();
    assert!(bad.validate(&s).is_err());
    bad = c.clone();
    bad.mountpoints[1] = bad.mountpoints[0].clone();
    assert!(bad.validate(&s).is_err());
    bad = c.clone();
    bad.mountpoints[1] = Mountpoint("/srv/report/nested".into());
    assert!(bad.validate(&s).is_err());
    bad = c.clone();
    bad.filesystems.clear();
    assert!(bad.validate(&s).is_err());
    let mut absent = s.clone();
    absent.identity = None;
    assert!(c.validate(&absent).is_err());
    absent = s.clone();
    absent.activation = None;
    assert!(c.validate(&absent).is_err());
}
#[test]
fn bounded_inputs_invalid_versions_and_incompatible_current_world_are_rejected() {
    let (s, c, mut m) = fixture();
    let before = m.clone();
    for text in ["a".repeat(4097), "\n".repeat(65), "\0".into()] {
        assert!(m.save(&c, &s, text).is_err());
        assert_eq!(m, before);
    }
    let mut bad = m.clone();
    bad.version = 2;
    assert!(bad.validate(&c, &s).is_err());
    let mut renamed = s.clone();
    renamed.execute("vgrename team different").unwrap();
    assert!(m.validate(&c, &renamed).is_err());
    let mut v = serde_json::to_value(&c).unwrap();
    v["filesystems"][0]["uuid"] = json!("a".repeat(37));
    assert!(serde_json::from_value::<Context>(v).is_err());
    v = serde_json::to_value(&c).unwrap();
    v["extra"] = json!(true);
    assert!(serde_json::from_value::<Context>(v).is_err());
    v = serde_json::to_value(&c).unwrap();
    v["filesystems"] = json!(vec![v["filesystems"][0].clone(); 5]);
    assert!(serde_json::from_value::<Context>(v).is_err());
}
#[test]
fn read_only_persists_and_actual_inspection_commands_do_not_change_policy_or_live_data() {
    let (mut s, c, mut m) = fixture();
    m.save(&c, &s, "/dev/team/draft /srv/report ext4 ro 0 2\n".into())
        .unwrap();
    m.mount(&c, &s, "/srv/report").unwrap();
    assert!(m.storage.live()["/srv/report"].read_only);
    let before = (s.clone(), m.clone());
    for cmd in ["pvs", "vgs", "lvs"] {
        assert!(!m.execute_lvm(&c, &mut s, cmd).unwrap().is_empty());
        assert_eq!((s.clone(), m.clone()), before);
    }
}
#[test]
fn unsupported_commands_and_escapes_are_atomic_and_old_lvm_stays_unexpanded() {
    let (mut s, c, mut m) = fixture();
    for cmd in [
        "mount /srv/report",
        "umount /srv/report",
        "findmnt",
        "sh -c true",
        "vgchange -ay team; touch /tmp/x",
        "vgchange -ay team\nlvs",
        "lvremove -y team/draft",
        "lvcreate -l 1 -n other team",
        "lvextend -l +1 /dev/team/draft",
        "vgrename team $(id)",
        &"x".repeat(2049),
    ] {
        refuse(&mut m, &c, &mut s, cmd);
    }
    for cmd in ["mount /srv/report", "umount /srv/report", "findmnt"] {
        let before = s.clone();
        assert!(s.execute(cmd).is_err());
        assert_eq!(s, before);
    }
}
