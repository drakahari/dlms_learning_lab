use dlms_learning_lab::{bios_loader as b, boot_entry as boot};
#[path = "support/bios_fixture.rs"]
mod fixture;
use fixture::{ROOT, facts};
fn install(w: &mut b::World, f: &b::Facts, s: &str) {
    w.execute(f, s).unwrap();
}
#[test]
fn actual_mbr_geometry_and_mounted_directories_control_installation_without_touching_files() {
    let f = facts();
    f.validate().unwrap();
    let before = serde_json::to_value(&f.geometry).unwrap();
    let mut w = b::World::default();
    assert!(!w.test(&f).unwrap().passed());
    install(
        &mut w,
        &f,
        "grub-install --target=i386-pc --boot-directory=/target/boot /dev/vda",
    );
    assert!(!w.current_test());
    let trace = w.test(&f).unwrap();
    assert!(trace.passed());
    assert!(w.current_test());
    assert_eq!(trace.handoff.unwrap().root.as_deref(), Some(ROOT));
    assert_eq!(f.embedding_sectors("/dev/vda").unwrap(), 2047);
    assert_eq!(serde_json::to_value(&f.geometry).unwrap(), before);
    assert!(
        f.geometry.disks["/dev/vda"].in_use,
        "installation must not pretend a mounted target needs partition-tool unmount restrictions"
    );
}
#[test]
fn literal_flag_order_and_quote_alternatives_have_equal_resulting_state() {
    let f = facts();
    let mut expected = None;
    for command in [
        "grub-install /dev/vda --boot-directory /target/boot --target i386-pc",
        "grub-install --boot-directory='/target/boot' /dev/vda",
        "grub-install --target=\"i386-pc\" --boot-directory=/target/boot '/dev/vda'",
    ] {
        let mut w = b::World::default();
        install(&mut w, &f, command);
        assert!(w.test(&f).unwrap().passed());
        if let Some(old) = &expected {
            assert_eq!(old, &w);
        } else {
            expected = Some(w);
        }
    }
}
#[test]
fn successful_install_on_wrong_disk_or_empty_boot_directory_does_not_imply_boot_success() {
    let f = facts();
    let mut w = b::World::default();
    install(
        &mut w,
        &f,
        "grub-install --boot-directory=/target/boot /dev/vdb",
    );
    let t = w.test(&f).unwrap();
    assert!(!t.passed());
    assert_eq!(t.stages.last().unwrap().title, "MBR boot code");
    install(&mut w, &f, "grub-install /dev/vda");
    let t = w.test(&f).unwrap();
    assert!(!t.passed());
    assert!(
        t.stages
            .last()
            .unwrap()
            .detail
            .contains("does not generate")
    );
    install(
        &mut w,
        &f,
        "grub-install --boot-directory=/target/boot /dev/vda",
    );
    assert!(w.test(&f).unwrap().passed());
    install(&mut w, &f, "grub-install /dev/vda");
    assert!(!w.current_test());
    assert!(!w.test(&f).unwrap().passed());
}
#[test]
fn embedding_follows_supplied_core_size_and_first_partition_instead_of_a_canned_success() {
    let mut f = facts();
    f.core_sectors = 2048;
    let mut w = b::World::default();
    let before = w.clone();
    let e = w
        .execute(&f, "grub-install --boot-directory=/target/boot /dev/vda")
        .unwrap_err();
    assert!(e.message.contains("2047") && e.message.contains("2048"));
    assert_eq!(w, before);
    f.core_sectors = 2047;
    install(
        &mut w,
        &f,
        "grub-install --boot-directory=/target/boot /dev/vda",
    );
    assert!(w.test(&f).unwrap().passed());
    f.boot_disk = "/dev/vdb".into();
    let mut other = b::World::default();
    install(
        &mut other,
        &f,
        "grub-install --boot-directory=/target/boot /dev/vdb",
    );
    assert!(
        other.test(&f).unwrap().passed(),
        "a boot directory can live on another supplied disk; do not invent a universal same-disk rule"
    );
}
#[test]
fn readonly_writes_partition_targets_unsupported_flags_and_shell_escapes_fail_atomically() {
    let f = facts();
    let mut w = b::World::default();
    for command in [
        "grub-install /dev/vda1",
        "grub-install --target=x86_64-efi /dev/vda",
        "grub-install --force /dev/vda",
        "grub-install --boot-directory=/target/../boot /dev/vda",
        "grub-install --target=i386-pc --target=i386-pc /dev/vda",
        "grub-install /dev/vda; id",
        "grub-install $(id)",
        "grub-install /dev/vd*",
        "grub-install /dev/vda\n",
        "grub-install --boot-directory='unfinished",
    ] {
        let before = w.clone();
        assert!(w.execute(&f, command).is_err(), "{command}");
        assert_eq!(w, before);
    }
    let mut read = f.clone();
    read.directories[0].writable = false;
    assert!(
        w.execute(&read, "grub-install --boot-directory=/target/boot /dev/vda")
            .is_err()
    );
    assert!(w.records.is_empty());
    assert_eq!(
        b::Install::parse("grub-install --force").unwrap_err().kind,
        boot::ErrorKind::Unsupported
    );
}
#[test]
fn inventory_reads_are_derived_from_facts_and_never_change_boot_records() {
    let f = facts();
    let mut w = b::World::default();
    let before = w.clone();
    let text = w
        .execute(&f, "lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS")
        .unwrap();
    assert!(text.contains("vda1 part ext4 /target/boot"));
    assert!(text.contains("vdb1 part ext4 /boot"));
    assert!(
        w.execute(&f, "findmnt -T /target/boot")
            .unwrap()
            .contains("/target/boot /dev/vda1 ext4 rw")
    );
    assert!(
        w.execute(&f, "cat /target/boot/grub/grub.cfg")
            .unwrap()
            .contains(ROOT)
    );
    assert!(w.execute(&f, "cat /etc/passwd").is_err());
    assert!(w.execute(&f, "cat /boot/grub/grub.cfg").is_err());
    assert_eq!(w, before);
}
#[test]
fn saved_observations_and_supplied_filesystem_identity_are_validated() {
    let f = facts();
    let mut w = b::World::default();
    install(
        &mut w,
        &f,
        "grub-install --boot-directory=/target/boot /dev/vda",
    );
    w.test(&f).unwrap();
    let saved = serde_json::to_string(&w).unwrap();
    let loaded: b::World = serde_json::from_str(&saved).unwrap();
    assert_eq!(w, loaded);
    loaded.bound(&f).unwrap();
    let mut changed = w.clone();
    changed.records[0].disk = "/dev/vdb".into();
    assert!(changed.bound(&f).is_err());
    let mut forged = w.clone();
    forged.checked = None;
    let b::Code::GrubBios { core_sectors, .. } = &mut forged.records[0].code else {
        panic!()
    };
    *core_sectors = 1;
    assert!(forged.bound(&f).is_err());
    let mut ghost = f.clone();
    ghost.directories[0]
        .configuration
        .as_mut()
        .unwrap()
        .inventory
        .roots[0]
        .uuid = "99999999-8888-7777-6666-555555555555".into();
    assert!(ghost.validate().is_err());
    let mut encoded = serde_json::to_value(&w).unwrap();
    encoded["records"] = serde_json::json!([
        w.records[0].clone(),
        w.records[0].clone(),
        w.records[0].clone(),
        w.records[0].clone(),
        w.records[0].clone()
    ]);
    assert!(serde_json::from_value::<b::World>(encoded).is_err());
    assert_eq!(
        b::Install::parse(&"x".repeat(1025)).unwrap_err().kind,
        boot::ErrorKind::Limit
    );
}
