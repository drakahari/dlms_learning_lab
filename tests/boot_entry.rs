use dlms_learning_lab::boot_entry::{self as b, ErrorKind, Initramfs, Inventory, Kernel, Root};
const ROOT: &str = "12345678-9abc-def0-1234-56789abcdef0";
fn inventory() -> Inventory {
    Inventory {
        kernels: vec![Kernel {
            path: "/systems/6.8/linux".into(),
            release: "6.8".into(),
            builtin_drivers: vec!["ext4".into()],
        }],
        initramfs: vec![
            Initramfs {
                path: "/systems/6.8/initrd".into(),
                release: "6.8".into(),
                drivers: vec!["virtio_blk".into()],
                uuid_lookup: true,
            },
            Initramfs {
                path: "/systems/6.7/initrd".into(),
                release: "6.7".into(),
                drivers: vec!["virtio_blk".into()],
                uuid_lookup: true,
            },
        ],
        roots: vec![Root {
            uuid: ROOT.into(),
            label: "system root".into(),
            required_drivers: vec!["virtio_blk".into(), "ext4".into()],
            init_present: true,
        }],
    }
}
fn source() -> String {
    format!(
        "title Example system\nversion 6.8\nlinux /systems/6.8/linux\ninitrd /systems/6.8/initrd\noptions root=UUID={ROOT} ro quiet\n"
    )
}
#[test]
fn parsed_handoff_uses_image_and_filesystem_facts_not_titles_or_expected_strings() {
    let mut i = inventory();
    let s = source();
    let r = b::test(&i, &s).unwrap();
    assert!(r.passed());
    assert_eq!(r.stages.len(), 4);
    assert_eq!(r.root.as_deref(), Some(ROOT));
    assert!(r.read_only);
    assert_eq!(r.target.as_deref(), Some("multi-user.target"));
    let alternate = s
        .replace("title Example system", "title Different arbitrary title")
        .replace("version 6.8", "version 999")
        .replace(" ro quiet", "\noptions quiet ro")
        .replace(ROOT, &ROOT.to_uppercase());
    assert!(b::test(&i, &alternate).unwrap().passed());
    // Bootable does not mean an assignment's desired root; callers must check identity.
    i.roots.push(Root {
        uuid: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
        label: "second system".into(),
        required_drivers: vec!["ext4".into()],
        init_present: true,
    });
    let different = b::test(&i, &s.replace(ROOT, &i.roots[1].uuid)).unwrap();
    assert!(different.passed());
    assert_ne!(different.root, r.root);
}
#[test]
fn stages_diagnose_missing_kernel_image_initramfs_release_root_drivers_and_init() {
    let i = inventory();
    let s = source();
    for (bad, stage, word) in [
        (
            s.replace("linux /systems/6.8/linux", "linux /systems/missing/linux"),
            "Kernel image",
            "no kernel",
        ),
        (
            s.replace("initrd /systems/6.8/initrd", "initrd /missing"),
            "Early userspace",
            "No supplied initramfs",
        ),
        (
            s.replace("initrd /systems/6.8/initrd", "initrd /systems/6.7/initrd"),
            "Early userspace",
            "6.7",
        ),
        (
            s.replace(ROOT, "00000000-0000-0000-0000-000000000000"),
            "Root filesystem",
            "matches no supplied",
        ),
    ] {
        let r = b::test(&i, &bad).unwrap();
        assert!(!r.passed());
        let last = r.stages.last().unwrap();
        assert_eq!(last.title, stage);
        assert!(!last.passed);
        assert!(last.detail.contains(word), "{}", last.detail);
    }
    let mut missing = i.clone();
    missing.initramfs[0].drivers.clear();
    let r = b::test(&missing, &s).unwrap();
    assert!(!r.passed());
    assert!(r.stages.last().unwrap().detail.contains("virtio_blk"));
    missing.roots[0].required_drivers = vec!["ext4".into()];
    missing.roots[0].init_present = false;
    let r = b::test(&missing, &s).unwrap();
    assert!(!r.passed());
    assert_eq!(r.stages.last().unwrap().title, "Init handoff");
    let r = b::test(&i, &s.replace("initrd /systems/6.8/initrd\n", "")).unwrap();
    assert!(!r.passed());
    assert!(r.stages.last().unwrap().detail.contains("bare kernel"));
}
#[test]
fn repeated_initrd_and_options_combine_and_kernel_read_mode_obeys_order() {
    let mut i = inventory();
    i.initramfs.push(Initramfs {
        path: "/systems/6.8/storage".into(),
        release: "6.8".into(),
        drivers: vec!["virtio_blk".into()],
        uuid_lookup: false,
    });
    i.initramfs[0].drivers.clear();
    let s = source()
        .replace(
            "initrd /systems/6.8/initrd",
            "initrd /systems/6.8/initrd\ninitrd /systems/6.8/storage",
        )
        .replace(
            " ro quiet",
            " rw\noptions quiet ro systemd.unit=rescue.target",
        );
    let e = b::Entry::parse(&s).unwrap();
    assert_eq!(
        e.initrd,
        vec!["/systems/6.8/initrd", "/systems/6.8/storage"]
    );
    let r = b::test(&i, &s).unwrap();
    assert!(r.passed());
    assert!(r.read_only);
    assert_eq!(r.target.as_deref(), Some("rescue.target"));
    let r = b::test(&i, &s.replace("quiet ro", "quiet rw")).unwrap();
    assert!(!r.read_only);
}
#[test]
fn syntax_and_unsupported_diagnostics_never_fake_execution() {
    let i = inventory();
    for (s, kind) in [
        ("linux=/x", ErrorKind::Syntax),
        ("linux /x\noptions", ErrorKind::Syntax),
        ("title Hi\n", ErrorKind::Syntax),
        ("linux /x\nlinux /y", ErrorKind::Unsupported),
        ("linux /../etc/passwd", ErrorKind::Unsupported),
        (
            "linux https://example.invalid/kernel",
            ErrorKind::Unsupported,
        ),
        ("linux /x\nexec reboot", ErrorKind::Unsupported),
        ("linux /x\noptions root=/dev/vda2", ErrorKind::Unsupported),
        ("linux /x\noptions root=UUID=bad", ErrorKind::Syntax),
        (
            "linux /x\noptions $(touch personal-file)",
            ErrorKind::Unsupported,
        ),
    ] {
        assert_eq!(b::test(&i, s).unwrap_err().kind, kind, "{s}");
    }
    let repeated = format!("{}options root=UUID={ROOT}\n", source());
    assert_eq!(
        b::test(&i, &repeated).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    let trailing = source().replace(" ro quiet", " ro # inline comments are literal");
    assert_eq!(
        b::test(&i, &trailing).unwrap_err().kind,
        ErrorKind::Unsupported
    );
    let title = source().replace("Example system", "$(unexecuted literal text)");
    assert!(b::test(&i, &title).unwrap().passed());
}
#[test]
fn bounded_parser_inventory_and_deserialization_reject_oversize_and_ambiguous_facts() {
    for s in [
        "x".repeat(b::MAX_SOURCE + 1),
        "#\n".repeat(b::MAX_LINES + 1),
    ] {
        assert_eq!(b::Entry::parse(&s).unwrap_err().kind, ErrorKind::Limit);
    }
    let mut i = inventory();
    i.kernels.push(i.kernels[0].clone());
    assert!(i.validate().is_err());
    let mut i = inventory();
    i.roots.push(i.roots[0].clone());
    assert!(i.validate().is_err());
    for path in [
        "/etc/../passwd",
        "/systems//linux",
        "/systems/./linux",
        "/systems/linux/",
        "relative",
        "/a\0b",
    ] {
        assert!(!b::path(path));
    }
    let mut json = serde_json::to_value(inventory()).unwrap();
    json["kernels"] = serde_json::Value::Array(vec![serde_json::Value::Null; 9]);
    assert!(serde_json::from_value::<Inventory>(json).is_err());
    let mut json = serde_json::to_value(inventory()).unwrap();
    json["kernels"][0]["path"] = serde_json::json!("x".repeat(129));
    assert!(serde_json::from_value::<Inventory>(json).is_err());
    let mut i = inventory();
    i.initramfs[0].uuid_lookup = false;
    assert!(!b::test(&i, &source()).unwrap().passed());
}
