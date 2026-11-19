use dlms_learning_lab::{boot_entry as boot, kernel_build as core, kernel_stage as stage};

const PREFIX: &str = "/review";
const UUID: &str = "11111111-2222-3333-4444-555555555555";

fn root() -> boot::Root {
    boot::Root {
        uuid: UUID.into(),
        label: "Supplied NVMe/ext4 root".into(),
        required_drivers: ["nvme_core", "nvme", "jbd2", "mbcache", "ext4"]
            .map(str::to_owned)
            .to_vec(),
        init_present: true,
    }
}
fn config(suffix: &str, modules: bool) -> core::Config {
    core::Config {
        modules,
        initrd: true,
        nvme: if modules {
            core::Feature::Module
        } else {
            core::Feature::BuiltIn
        },
        ext4: if modules {
            core::Feature::Module
        } else {
            core::Feature::BuiltIn
        },
        localversion: suffix.into(),
    }
}
fn build(source: &str, base: &str, c: &core::Config) -> core::Artifacts {
    let mut a = core::Artifacts::default();
    a.build(source, base, c, core::Target::All).unwrap();
    a
}
fn text(release: &str, initrd: bool) -> String {
    format!(
        "title Supplied review image\nlinux /vmlinuz-{release}\n{}options root=UUID={UUID} ro\n",
        if initrd {
            format!("initrd /initramfs-{release}.img\n")
        } else {
            String::new()
        }
    )
}
fn package(
    s: &mut stage::Store,
    prefix: &str,
    source: &str,
    base: &str,
    c: &core::Config,
    name: &str,
) -> String {
    let a = build(source, base, c);
    let image = a.image.as_ref().unwrap();
    let path = s.image(prefix, image).unwrap();
    if let Some(modules) = &a.modules {
        s.modules(prefix, modules).unwrap();
    }
    s.assemble_initramfs(prefix, &path).unwrap();
    s.entry(prefix, name, &text(&image.release, true)).unwrap()
}

#[test]
fn actual_shared_boot_checks_accept_modular_and_builtin_userspace_alternatives() {
    for modules in [true, false] {
        let mut s = stage::Store::default();
        let entry = package(
            &mut s,
            PREFIX,
            "source-a",
            "6.6.50",
            &config("-training", modules),
            "candidate.conf",
        );
        let report = s.test_entry(PREFIX, &entry, root()).unwrap();
        assert!(report.passed(), "{modules}: {:?}", report.stages);
        assert_eq!(s.modules.len(), usize::from(modules));
        assert_eq!(s.initramfs.len(), 1);
    }
}

#[test]
fn missing_staging_steps_and_wrong_root_fail_without_mutating_valid_files() {
    let mut s = stage::Store::default();
    let a = build("source-a", "6.6.50", &config("-training", true));
    let image = a.image.as_ref().unwrap();
    let path = s.image(PREFIX, image).unwrap();
    let before = s.clone();
    let error = s.assemble_initramfs(PREFIX, &path).unwrap_err();
    assert!(error.contains("Missing installed module tree /review/lib/modules/6.6.50-training"));
    assert!(error.contains("Install the matching built modules"));
    assert_eq!(s, before);
    let entry = s
        .entry(PREFIX, "candidate.conf", &text(&image.release, true))
        .unwrap();
    assert!(!s.test_entry(PREFIX, &entry, root()).unwrap().passed());
    s.modules(PREFIX, a.modules.as_ref().unwrap()).unwrap();
    s.assemble_initramfs(PREFIX, &path).unwrap();
    assert!(s.test_entry(PREFIX, &entry, root()).unwrap().passed());
    let wrong = text(&image.release, true).replace(UUID, "99999999-2222-3333-4444-555555555555");
    s.entry(PREFIX, "candidate.conf", &wrong).unwrap();
    assert!(!s.test_entry(PREFIX, &entry, root()).unwrap().passed());
    s.entry(
        PREFIX,
        "candidate.conf",
        &wrong.replace("Supplied review image", "Correct answer"),
    )
    .unwrap();
    assert!(!s.test_entry(PREFIX, &entry, root()).unwrap().passed());
}

#[test]
fn builtin_drivers_do_not_supply_uuid_lookup_or_external_loading_support() {
    let mut s = stage::Store::default();
    let mut c = config("-builtin", false);
    c.initrd = false;
    let a = build("source-a", "6.6.50", &c);
    assert!(a.readiness().passed);
    let image = a.image.as_ref().unwrap();
    let path = s.image(PREFIX, image).unwrap();
    let entry = s
        .entry(PREFIX, "candidate.conf", &text(&image.release, false))
        .unwrap();
    let report = s.test_entry(PREFIX, &entry, root()).unwrap();
    assert!(!report.passed());
    assert!(
        report
            .stages
            .iter()
            .any(|r| r.detail.contains("UUID lookup"))
    );
    s.assemble_initramfs(PREFIX, &path).unwrap();
    s.entry(PREFIX, "candidate.conf", &text(&image.release, true))
        .unwrap();
    assert!(
        s.test_entry(PREFIX, &entry, root())
            .unwrap_err()
            .contains("loading disabled")
    );
}

#[test]
fn staged_module_identity_is_more_than_a_matching_release_label() {
    let mut s = stage::Store::default();
    let c = config("-same-label", true);
    let a = build("source-a", "6.6.50", &c);
    let b = build("source-b", "6.6.50", &c);
    let path = s.image(PREFIX, a.image.as_ref().unwrap()).unwrap();
    s.modules(PREFIX, b.modules.as_ref().unwrap()).unwrap();
    let before = s.clone();
    assert!(
        s.assemble_initramfs(PREFIX, &path)
            .unwrap_err()
            .contains("different source/configuration")
    );
    assert_eq!(s, before);
    s.modules(PREFIX, a.modules.as_ref().unwrap()).unwrap();
    s.assemble_initramfs(PREFIX, &path).unwrap();
    let entry = s
        .entry(
            PREFIX,
            "candidate.conf",
            &text(&a.image.as_ref().unwrap().release, true),
        )
        .unwrap();
    assert!(s.test_entry(PREFIX, &entry, root()).unwrap().passed());
    let mut other = stage::Store::default();
    package(&mut other, "/other", "source-b", "6.6.50", &c, "other.conf");
    s.initramfs[0].image = other.initramfs[0].image.clone();
    s.validate().unwrap();
    assert!(
        s.test_entry(PREFIX, &entry, root())
            .unwrap_err()
            .contains("different supplied source/configuration")
    );
}

#[test]
fn versioned_staging_preserves_an_independent_working_fallback() {
    let mut s = stage::Store::default();
    let old = package(
        &mut s,
        PREFIX,
        "old-source",
        "6.6.49",
        &config("-working", true),
        "fallback.conf",
    );
    let initial = s.clone();
    let next = package(
        &mut s,
        PREFIX,
        "new-source",
        "6.6.50",
        &config("-review", true),
        "candidate.conf",
    );
    assert_eq!(s.images[..initial.images.len()], initial.images);
    assert_eq!(s.modules[..initial.modules.len()], initial.modules);
    assert_eq!(s.initramfs[..initial.initramfs.len()], initial.initramfs);
    assert_eq!(s.entries[..initial.entries.len()], initial.entries);
    assert!(s.test_entry(PREFIX, &old, root()).unwrap().passed());
    assert!(s.test_entry(PREFIX, &next, root()).unwrap().passed());
    s.entry(PREFIX,"candidate.conf","title Candidate only\nlinux /missing\noptions root=UUID=11111111-2222-3333-4444-555555555555 ro\n").unwrap();
    assert!(!s.test_entry(PREFIX, &next, root()).unwrap().passed());
    assert!(s.test_entry(PREFIX, &old, root()).unwrap().passed());
}

#[test]
fn path_traversal_capacity_and_malformed_namespace_changes_fail_atomically() {
    let mut s = stage::Store::default();
    let a = build("source-a", "6.6.50", &config("-review", false));
    let before = s.clone();
    for prefix in ["../host", "/review/../host", "/review//host"] {
        assert!(s.image(prefix, a.image.as_ref().unwrap()).is_err());
        assert_eq!(s, before);
    }
    for name in ["../../host.conf", "/host.conf", "candidate"] {
        assert!(s.entry(PREFIX, name, "title sample\n").is_err());
        assert_eq!(s, before);
    }
    assert!(s.entry(PREFIX, "candidate.conf", "\0").is_err());
    assert_eq!(s, before);
    for n in 0..8 {
        let a = build("source-a", "6.6.50", &config(&format!("-v{n}"), false));
        s.image(PREFIX, a.image.as_ref().unwrap()).unwrap();
    }
    let before = s.clone();
    assert!(s.image(PREFIX, a.image.as_ref().unwrap()).is_err());
    assert_eq!(s, before);
    let raw = serde_json::to_value(&s).unwrap();
    let mut excess = raw.clone();
    excess["images"]
        .as_array_mut()
        .unwrap()
        .push(raw["images"][0].clone());
    assert!(serde_json::from_value::<stage::Store>(excess).is_err());
    let mut duplicate = s.clone();
    duplicate.images[1].path = duplicate.images[0].path.clone();
    assert!(duplicate.validate().is_err());
    let mut ancestor = stage::Store::default();
    ancestor.images.push(stage::ImageFile {
        path: "/review/boot".into(),
        build: a.image.unwrap(),
    });
    let before = ancestor.clone();
    assert!(
        ancestor
            .entry(PREFIX, "candidate.conf", "title sample\n")
            .is_err()
    );
    assert_eq!(ancestor, before);
}

#[test]
fn partial_staging_and_bad_entry_text_round_trip_without_inventing_boot_success() {
    let mut s = stage::Store::default();
    let a = build("source-a", "6.6.50", &config("-training", true));
    s.image(PREFIX, a.image.as_ref().unwrap()).unwrap();
    let entry = s
        .entry(
            PREFIX,
            "candidate.conf",
            "title Still editing\nlinux /missing\n",
        )
        .unwrap();
    let raw = serde_json::to_vec(&s).unwrap();
    let restored: stage::Store = serde_json::from_slice(&raw).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, s);
    assert!(restored.test_entry(PREFIX, &entry, root()).is_err());
}
