use dlms_learning_lab::{
    boot_entry as boot, initramfs as core,
    initramfs_workspace::{Action, Kind, Task},
    kernel_build as kernel,
    kernel_stage::Store,
};
pub fn entry(release: &str, image: &str, uuid: &str) -> String {
    format!(
        "title Training candidate\nlinux /vmlinuz-{release}\ninitrd {image}\noptions root=UUID={uuid} ro\n"
    )
}
pub fn task(kind: Kind, practice: bool) -> Task {
    let (old, new, uuid) = if practice {
        ("6.1.120", "6.1.121", "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
    } else {
        ("6.6.60", "6.6.61", "11111111-2222-3333-4444-555555555555")
    };
    let mut initial_store = Store::default();
    for (source, release, file) in [
        ("fallback-source", old, "fallback.conf"),
        ("target-source", new, "candidate.conf"),
    ] {
        let cfg = kernel::Config {
            modules: true,
            initrd: true,
            nvme: kernel::Feature::Module,
            ext4: kernel::Feature::Module,
            localversion: String::new(),
        };
        let mut a = kernel::Artifacts::default();
        a.build(source, release, &cfg, kernel::Target::All).unwrap();
        let path = initial_store.image("/", a.image.as_ref().unwrap()).unwrap();
        initial_store
            .modules("/", a.modules.as_ref().unwrap())
            .unwrap();
        initial_store.assemble_initramfs("/", &path).unwrap();
        initial_store
            .entry(
                "/",
                file,
                &entry(release, &format!("/initramfs-{release}.img"), uuid),
            )
            .unwrap();
    }
    let initial_config = if kind == Kind::Drivers {
        "# Driver choices for the supplied generic profile.\nomit_drivers+=\" nvme \"\n"
    } else {
        "# No driver omissions in this profile.\n"
    }
    .to_owned();
    let generated = if kind == Kind::Drivers { new } else { old };
    core::execute(
        "/",
        &core::Config::parse(&initial_config).unwrap(),
        &mut initial_store,
        &format!("dracut --force --kver {generated} /boot/initramfs-{new}.img"),
    )
    .unwrap();
    let mut t = Task {
        id: format!(
            "initramfs-{}-{}",
            if kind == Kind::Drivers {
                "drivers"
            } else {
                "handover"
            },
            if practice { "example" } else { "assignment" }
        ),
        kind,
        instructions: format!(
            "Save the driver choices and prepare matching early userspace for target {new}. Verify the selected candidate reaches the supplied root; preserve the working {old} fallback."
        ),
        prefix: "/".into(),
        initial_config,
        initial_store,
        root: boot::Root {
            uuid: uuid.into(),
            label: "Supplied NVMe/ext4 root".into(),
            required_drivers: ["nvme_core", "nvme", "jbd2", "mbcache", "ext4"]
                .map(str::to_owned)
                .to_vec(),
            init_present: true,
        },
        wanted_release: new.into(),
        entry_filename: "candidate.conf".into(),
        initial_entry: entry(new, &format!("/initramfs-{new}.img"), uuid),
        preserve_paths: vec![
            format!("/boot/vmlinuz-{old}"),
            format!("/lib/modules/{old}"),
            format!("/boot/initramfs-{old}.img"),
            "/boot/loader/entries/fallback.conf".into(),
        ],
        reference: vec![],
    };
    t.reference = if kind == Kind::Drivers {
        vec![
            Action::ConfigDraft {
                text: "# Keep the supplied root drivers available.\nomit_drivers=\"\"\n".into(),
            },
            Action::SaveConfig,
            Action::Run {
                input: format!("dracut --force --kver {new} /boot/initramfs-{new}.img"),
            },
            Action::Check,
        ]
    } else {
        vec![
            Action::Run {
                input: format!("dracut /boot/repaired.img {new}"),
            },
            Action::EntryDraft {
                filename: "trial.conf".into(),
                text: entry(new, "/repaired.img", uuid),
            },
            Action::SaveEntry,
            Action::Check,
        ]
    };
    t
}
