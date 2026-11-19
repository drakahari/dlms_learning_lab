//! Deterministic EFI fixtures. No firmware or host disk calls.
use disk::fat::{Path, Volume};
use dlms_learning_lab::{
    bios_loader as bios, boot_entry as boot, efi_loader as efi, partition_fs as disk,
};
use std::collections::BTreeMap;

const FIRST_ESP: &str = "2e4dbb07-bc66-456e-bbf6-ff102b78a8c6";
const ARCHIVE_ESP: &str = "b6de55de-df93-4625-9f70-3f34c307a76d";
const ROOT: &str = "7240d4ce-f757-4103-be48-d903f6e7e9c9";

fn facts() -> efi::Facts {
    let supplied_disk = |root_uuid: &str, serial: &str| disk::Disk {
        sectors: 1_048_576,
        sector_bytes: 512,
        optimal_alignment: 2048,
        writable: true,
        in_use: true,
        occupied_without_table: false,
        table: Some(disk::Gpt {
            partitions: vec![
                disk::Partition {
                    number: 1,
                    name: "esp".into(),
                    start: 2048,
                    end: 264_191,
                    type_hint: disk::TypeHint::EfiSystem,
                    content: disk::Content::Fat32 {
                        volume: Volume {
                            serial: serial.into(),
                            label: "BOOT".into(),
                            sectors_per_cluster: 1,
                            data_clusters: 65_536,
                        },
                    },
                },
                disk::Partition {
                    number: 2,
                    name: "system".into(),
                    start: 264_192,
                    end: 526_335,
                    type_hint: disk::TypeHint::LinuxData,
                    content: disk::Content::Ext4 {
                        label: "system".into(),
                        uuid: root_uuid.into(),
                        blocks: 32_768,
                        files: BTreeMap::new(),
                    },
                },
            ],
        }),
        mbr: None,
    };
    efi::Facts {
        geometry: disk::State {
            version: 1,
            mbr_profile: None,
            next_uuid: 1,
            disks: BTreeMap::from([
                (
                    "/dev/vda".into(),
                    supplied_disk("849b78c9-fc96-4d39-8e8a-ecac0c0d2f6a", "57A1-CF00"),
                ),
                ("/dev/vdb".into(), supplied_disk(ROOT, "57A1-CF01")),
            ]),
        },
        esps: vec![
            efi::Esp {
                disk: "/dev/vda".into(),
                number: 1,
                partition_guid: FIRST_ESP.into(),
                removable: false,
                mount: Some(efi::Mount {
                    path: "/rescue/esp".into(),
                    writable: true,
                }),
            },
            efi::Esp {
                disk: "/dev/vdb".into(),
                number: 1,
                partition_guid: ARCHIVE_ESP.into(),
                removable: false,
                mount: None,
            },
        ],
        boot_directories: vec![
            bios::Directory {
                path: "/target/boot".into(),
                disk: "/dev/vdb".into(),
                partition: 2,
                writable: true,
                configuration: Some(bios::Configuration {
                    menu: format!(
                        "set default=system\nmenuentry 'Operations' --id=system {{\nlinux /linux root=UUID={ROOT} ro\ninitrd /initrd\n}}\n"
                    ),
                    inventory: boot::Inventory {
                        kernels: vec![boot::Kernel {
                            path: "/linux".into(),
                            release: "7.0".into(),
                            builtin_drivers: vec!["ext4".into()],
                        }],
                        initramfs: vec![boot::Initramfs {
                            path: "/initrd".into(),
                            release: "7.0".into(),
                            drivers: vec!["virtio_blk".into()],
                            uuid_lookup: true,
                        }],
                        roots: vec![boot::Root {
                            uuid: ROOT.into(),
                            label: "Operations".into(),
                            required_drivers: vec!["ext4".into(), "virtio_blk".into()],
                            init_present: true,
                        }],
                    },
                }),
            },
            bios::Directory {
                path: "/boot".into(),
                disk: "/dev/vda".into(),
                partition: 2,
                writable: true,
                configuration: None,
            },
        ],
        grub_image_bytes: 256 * 1024,
        firmware_writable: true,
    }
}
fn world() -> efi::World {
    let archive = Path::parse("/EFI/ARCHIVE/start.efi").unwrap();
    efi::World {
        version: 1,
        files: vec![efi::File {
            partition_guid: ARCHIVE_ESP.into(),
            path: archive.clone(),
            bytes: 4096,
            kind: efi::FileKind::OtherApplication,
        }],
        options: vec![efi::OptionEntry {
            number: 7,
            active: true,
            label: "Archive".into(),
            partition_guid: ARCHIVE_ESP.into(),
            loader: archive,
        }],
        boot_order: vec![7],
    }
}
#[test]
fn install_register_and_firmware_order_are_separate_and_cross_disk_boot_is_valid() {
    let f = facts();
    let mut w = world();
    let preserved = w.clone();
    let loader = w
        .install(&f, "/rescue/esp", "/target/boot", "Operations")
        .unwrap();
    assert_eq!(w.options, preserved.options);
    assert_eq!(w.boot_order, [7]);
    assert_eq!(w.trace(&f).unwrap().started, Some(7));
    assert_eq!(w.files[0], preserved.files[0]);
    assert_eq!(loader.as_str(), "/EFI/OPERATIONS/GRUBX64.EFI");
    let new = w.register(&f, "/dev/vda", 1, "Operations", loader).unwrap();
    assert_eq!(w.boot_order, [new, 7]);
    let trace = w.trace(&f).unwrap();
    assert_eq!(trace.started, Some(new));
    let report = trace.handoff.unwrap();
    assert!(report.passed());
    assert_eq!(report.requested.root, ROOT);
    assert_eq!(w.options[0], preserved.options[0]);
    w.reorder(&f, &[7, new]).unwrap();
    assert_eq!(w.trace(&f).unwrap().started, Some(7));
}
#[test]
fn a_missing_application_falls_through_but_a_started_grub_without_menu_does_not() {
    let f = facts();
    let mut w = world();
    let path = Path::parse("\\EFI\\RECOVERY\\grubx64.efi").unwrap();
    let first = w
        .register(&f, "/dev/vda", 1, "Recovery", path.clone())
        .unwrap();
    let trace = w.trace(&f).unwrap();
    assert_eq!(trace.started, Some(7));
    assert!(trace.attempts[0].outcome.contains("missing"));
    assert_eq!(
        w.install(&f, "/rescue/esp", "/boot", "recovery").unwrap(),
        path
    );
    let trace = w.trace(&f).unwrap();
    assert_eq!(trace.started, Some(first));
    assert!(trace.handoff.is_none());
    assert_eq!(trace.attempts.len(), 1);
    assert!(trace.outcome.contains("no supplied menu"));
}
#[test]
fn efi_role_fat32_format_and_partition_guid_are_independent_validated_facts() {
    let mut f = facts();
    let guid = f.esps[0].partition_guid.clone();
    assert_ne!(guid, "57A1-CF00");
    f.validate().unwrap();
    f.esps[1].partition_guid = guid;
    assert!(f.validate().unwrap_err().contains("unique"));
    let mut f = facts();
    let partition = &mut f
        .geometry
        .disks
        .get_mut("/dev/vda")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0];
    partition.type_hint = disk::TypeHint::LinuxData;
    assert!(f.validate().unwrap_err().contains("System Partition type"));
    let mut f = facts();
    let partition = &mut f
        .geometry
        .disks
        .get_mut("/dev/vda")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0];
    partition.content = disk::Content::Foreign {
        description: "FAT32".into(),
    };
    assert!(f.validate().unwrap_err().contains("modeled FAT32"));
}
#[test]
fn invalid_writes_leave_all_state_and_protected_payloads_intact() {
    let mut f = facts();
    let mut w = world();
    let old = w.clone();
    assert!(w.install(&f, "/dev/vda1", "/target/boot", "lab").is_err());
    assert!(w.install(&f, "/rescue/esp", "/etc", "lab").is_err());
    assert!(
        w.install(&f, "/rescue/esp", "/target/boot", "../backup")
            .is_err()
    );
    assert!(w.reorder(&f, &[7, 7]).is_err());
    assert!(w.reorder(&f, &[4096]).is_err());
    assert_eq!(w, old);
    f.firmware_writable = false;
    let path = w.install(&f, "/rescue/esp", "/target/boot", "lab").unwrap();
    let installed = w.clone();
    assert!(w.register(&f, "/dev/vda", 1, "Lab", path).is_err());
    assert_eq!(w, installed);
    f.esps[0].mount.as_mut().unwrap().writable = false;
    assert!(w.install(&f, "/rescue/esp", "/target/boot", "lab").is_err());
    assert_eq!(w, installed);
    assert_eq!(
        w.trace(&f).unwrap().started,
        Some(7),
        "firmware reads do not need a writable mount"
    );
}
#[test]
fn case_folded_reinstallation_updates_one_record_and_capacity_limits_refuse_atomically() {
    let mut f = facts();
    f.grub_image_bytes = 4 * 1024 * 1024;
    let mut w = world();
    for n in 0..7 {
        w.install(&f, "/rescue/esp", "/target/boot", &format!("image{n}"))
            .unwrap();
    }
    let before = w.clone();
    assert!(
        w.install(&f, "/rescue/esp", "/target/boot", "image8")
            .unwrap_err()
            .contains("lacks room")
    );
    assert_eq!(w, before);
    w.install(&f, "/rescue/esp", "/boot", "IMAGE0").unwrap();
    assert_eq!(w.files.len(), before.files.len());
    assert_eq!(w.files[0], before.files[0]);
    assert_ne!(w.files[1], before.files[1]);
    let restored: efi::World = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    restored.bound(&f).unwrap();
    assert_eq!(restored, w);
}

#[test]
fn literal_console_commands_support_valid_alternatives_and_derive_reports_from_state() {
    let f = facts();
    let mut first = world();
    let mut second = world();
    let install = "grub-install --target=x86_64-efi --efi-directory=/rescue/esp --boot-directory=/target/boot --bootloader-id=Operations --no-nvram";
    let alternate_install = "grub-install --no-nvram --bootloader-id operations --boot-directory '/target/boot' --efi-directory '/rescue/esp' --target x86_64-efi";
    assert!(
        first
            .execute(&f, install)
            .unwrap()
            .contains("Firmware entries unchanged")
    );
    second.execute(&f, alternate_install).unwrap();
    assert_eq!(first, second);
    let register = r"efibootmgr --create --disk /dev/vda --part 1 --label Operations --loader '\EFI\Operations\grubx64.efi'";
    let alternate_register =
        r"efibootmgr -c -l '\efi\operations\GRUBX64.EFI' -L Operations -p 1 -d /dev/vda";
    first.execute(&f, register).unwrap();
    second.execute(&f, alternate_register).unwrap();
    assert_eq!(first, second);
    let report = first.execute(&f, "efibootmgr -v").unwrap();
    assert!(report.contains("BootOrder: 0000,0007"));
    assert!(report.contains(FIRST_ESP));
    assert!(report.contains(r"\EFI\OPERATIONS\GRUBX64.EFI"));
    assert!(
        first
            .execute(&f, "findmnt -T /rescue/esp")
            .unwrap()
            .contains("/dev/vda1 vfat rw")
    );
    assert!(
        first
            .execute(&f, "blkid /dev/vda1")
            .unwrap()
            .contains("57A1-CF00")
    );
    assert!(
        first
            .execute(&f, "cat /target/boot/grub/grub.cfg")
            .unwrap()
            .contains(ROOT)
    );
    assert!(
        first
            .execute(&f, "lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS")
            .unwrap()
            .contains("vda1 part vfat /rescue/esp")
    );
    assert_eq!(first, second, "inventory commands must not change state");
    first.execute(&f, "efibootmgr --bootorder=7,0000").unwrap();
    assert_eq!(first.trace(&f).unwrap().started, Some(7));
}

#[test]
fn unsupported_and_malformed_commands_preserve_the_machine() {
    let f = facts();
    let mut w = world();
    let original = w.clone();
    for bad in [
        "grub-install --target=x86_64-efi --efi-directory=/rescue/esp --boot-directory=/target/boot --bootloader-id=Operations",
        "grub-install --removable",
        "efibootmgr --bootnext 7",
        "efibootmgr --bootorder 0007,0007",
        "efibootmgr -o 0007,",
        "efibootmgr --bootorder=10000",
        "efibootmgr -c --create",
        "cat /etc/passwd",
        "findmnt -T /home",
        "lsblk | cat",
        "echo $(id)",
        r"efibootmgr -c -d /dev/vda -p 1 -L Broken -l '\EFI\..\bad.efi'",
    ] {
        assert!(w.execute(&f, bad).is_err(), "{bad}");
        assert_eq!(w, original, "{bad}");
    }
}

fn exercise() -> dlms_learning_lab::efi_loader_workspace::Definition {
    use dlms_learning_lab::efi_loader_workspace as work;
    let mut supplied = facts();
    supplied.esps[1].mount = Some(efi::Mount {
        path: "/rescue/archive-esp".into(),
        writable: true,
    });
    let assignment = work::Task {
        id: "test-efi-restore".into(),
        instructions: "Restore the intended boot path and retain the archive as a fallback.".into(),
        facts: supplied,
        initial: world(),
        reference: vec![
            "grub-install --target=x86_64-efi --efi-directory=/rescue/esp --boot-directory=/target/boot --bootloader-id=Operations --no-nvram".into(),
            r"efibootmgr --create --disk /dev/vda --part 1 --label Operations --loader '\EFI\Operations\grubx64.efi'".into(),
            work::CHECK_ACTION.into(),
        ],
        required_esp: FIRST_ESP.into(),
        required_root: ROOT.into(),
        required_release: "7.0".into(),
        read_only: true,
        target: "multi-user.target".into(),
        archive_entry: 7,
        media: None,
    };
    let mut example = assignment.clone();
    let workshop_root = "17d6ee5a-ff10-4f51-804b-891789d552af";
    let workshop_esp = "479577b7-2886-490a-8243-7f530046c16b";
    example.id = "workshop-efi-restore".into();
    example.required_root = workshop_root.into();
    example.required_esp = workshop_esp.into();
    example.facts.esps[0].partition_guid = workshop_esp.into();
    example.facts.esps[0].mount.as_mut().unwrap().path = "/workshop/esp".into();
    let disk::Content::Ext4 { uuid, .. } = &mut example
        .facts
        .geometry
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[1]
        .content
    else {
        panic!()
    };
    *uuid = workshop_root.into();
    let boot = &mut example.facts.boot_directories[0];
    boot.path = "/workshop/boot".into();
    let configuration = boot.configuration.as_mut().unwrap();
    configuration.menu = configuration.menu.replace(ROOT, workshop_root);
    configuration.inventory.roots[0].uuid = workshop_root.into();
    example.reference[0] = "grub-install --target=x86_64-efi --efi-directory=/workshop/esp --boot-directory=/workshop/boot --bootloader-id=Workshop --no-nvram".into();
    example.reference[1] =
        r"efibootmgr -c -d /dev/vda -p 1 -L Workshop -l '\EFI\Workshop\grubx64.efi'".into();
    work::Definition {
        version: 1,
        initial: assignment,
        example,
    }
}

#[test]
fn independent_drafts_require_real_installation_registration_test_and_fallback() {
    use dlms_learning_lab::efi_loader_workspace as work;
    let definition = exercise();
    let mut session = work::Session::new(&definition).unwrap();
    for command in &definition.example.reference[..2] {
        session.practice.command = command.clone();
        session.practice.run(&definition.example).unwrap();
    }
    session.practice.test(&definition.example).unwrap();
    assert!(
        definition
            .example
            .checks(&session.practice)
            .iter()
            .all(|row| row.0)
    );
    assert_eq!(session.assignment.current, definition.initial.initial);
    assert!(definition.checks(&session).iter().any(|row| !row.0));

    session
        .execute_reference(&definition, &definition.initial.reference[0])
        .unwrap();
    session
        .execute_reference(&definition, work::CHECK_ACTION)
        .unwrap();
    assert!(
        definition
            .checks(&session)
            .iter()
            .any(|row| !row.0 && row.1.contains("no active BootOrder entry"))
    );
    session
        .execute_reference(
            &definition,
            r"efibootmgr -c -d /dev/vda -p 1 -L 'Recovered Linux' -l '\efi\operations\GRUBX64.EFI'",
        )
        .unwrap();
    assert!(session.assignment.checked.is_none());
    session
        .execute_reference(&definition, work::CHECK_ACTION)
        .unwrap();
    assert!(definition.checks(&session).iter().all(|row| row.0));

    session
        .execute_reference(&definition, "efibootmgr -o 0")
        .unwrap();
    session
        .execute_reference(&definition, work::CHECK_ACTION)
        .unwrap();
    assert!(
        definition
            .checks(&session)
            .iter()
            .any(|row| !row.0 && row.1.contains("Keep Boot0007"))
    );
    session
        .execute_reference(&definition, "efibootmgr --bootorder=0000,0007")
        .unwrap();
    session
        .execute_reference(&definition, work::CHECK_ACTION)
        .unwrap();
    assert!(definition.checks(&session).iter().all(|row| row.0));

    let wrong = definition.initial.reference[0].replace("/rescue/esp", "/rescue/archive-esp");
    session.execute_reference(&definition, &wrong).unwrap();
    session
        .execute_reference(&definition, work::CHECK_ACTION)
        .unwrap();
    assert!(
        definition
            .checks(&session)
            .iter()
            .any(|row| !row.0 && row.1.contains("every file on the archive"))
    );
}

#[test]
fn saved_observations_are_bound_to_the_current_draft_and_reachable_teaching() {
    use dlms_learning_lab::efi_loader_workspace as work;
    let definition = exercise();
    let mut session = work::Session::new(&definition).unwrap();
    for command in &definition.initial.reference {
        session.execute_reference(&definition, command).unwrap();
    }
    session.assignment.command = "grub-install --efi-directory='unfinished".into();
    let encoded = serde_json::to_string(&session).unwrap();
    let decoded: work::Session = serde_json::from_str(&encoded).unwrap();
    decoded.bound(&definition).unwrap();
    assert_eq!(decoded, session);
    assert!(definition.checks(&decoded).iter().all(|row| row.0));
    session.assignment.current.boot_order.reverse();
    assert!(session.bound(&definition).is_err());

    let mut unreachable = definition.clone();
    unreachable.initial.reference[1] =
        unreachable.initial.reference[1].replace("grubx64.efi", "missing.efi");
    assert!(unreachable.validate().is_err());
    let mut trivial = definition.clone();
    trivial.initial.initial = decoded.assignment.current;
    assert!(trivial.validate().is_err());
    let mut same_workshop = definition.clone();
    same_workshop.example.required_root = same_workshop.initial.required_root.clone();
    assert!(same_workshop.validate().is_err());
}

#[test]
fn selected_removable_media_needs_default_path_but_no_new_firmware_entry() {
    let mut f = facts();
    f.esps[0].removable = true;
    let mut w = world();
    let original_options = w.options.clone();
    let original_order = w.boot_order.clone();
    w.install(&f, "/rescue/esp", "/target/boot", "named")
        .unwrap();
    let named = w.trace_removable(&f, FIRST_ESP, None).unwrap();
    assert!(!named.application_started && named.outcome.contains("default x64"));
    let default = w
        .install_removable(&f, "/rescue/esp", "/target/boot")
        .unwrap();
    assert_eq!(default.as_str(), "/EFI/BOOT/BOOTX64.EFI");
    assert_eq!(w.options, original_options);
    assert_eq!(w.boot_order, original_order);
    assert!(w.trace_removable(&f, FIRST_ESP, None).unwrap().passed());
    assert_eq!(w.trace(&f).unwrap().started, Some(7));

    let unavailable_directory = w.trace_removable(&f, FIRST_ESP, Some("/dev/vdb")).unwrap();
    assert!(unavailable_directory.application_started);
    assert!(unavailable_directory.outcome.contains("/target/boot"));
    assert!(!unavailable_directory.passed());
    assert!(w.trace_removable(&f, ARCHIVE_ESP, None).is_err());
    assert!(w.trace_removable(&f, FIRST_ESP, Some("/dev/vdz")).is_err());
    let unavailable_media = w.trace_removable(&f, FIRST_ESP, Some("/dev/vda")).unwrap();
    assert!(!unavailable_media.application_started);
    assert!(unavailable_media.outcome.contains("device is unavailable"));
}

#[test]
fn unavailable_root_fails_after_available_loader_menu_and_images() {
    let mut f = facts();
    f.esps[0].removable = true;
    f.boot_directories[0].disk = "/dev/vda".into();
    let mut w = world();
    w.install_removable(&f, "/rescue/esp", "/target/boot")
        .unwrap();
    assert!(w.trace_removable(&f, FIRST_ESP, None).unwrap().passed());
    let before = w.clone();
    let unavailable = w.trace_removable(&f, FIRST_ESP, Some("/dev/vdb")).unwrap();
    assert!(unavailable.application_started);
    assert!(!unavailable.passed());
    let handoff = unavailable.handoff.unwrap();
    assert_eq!(handoff.requested.root, ROOT);
    assert_eq!(handoff.release.as_deref(), Some("7.0"));
    assert!(handoff.stages.iter().any(|stage| {
        !stage.passed && stage.title == "Root filesystem" && stage.detail.contains("unavailable")
    }));
    assert_eq!(
        w, before,
        "Testing device absence must leave the repair state intact."
    );

    let local_root = "849b78c9-fc96-4d39-8e8a-ecac0c0d2f6a";
    let config = f.boot_directories[0].configuration.as_mut().unwrap();
    config.menu = config.menu.replace(ROOT, local_root);
    config.inventory.roots[0].uuid = local_root.into();
    assert!(
        w.trace_removable(&f, FIRST_ESP, Some("/dev/vdb"))
            .unwrap()
            .passed()
    );
    assert_eq!(w.trace(&f).unwrap().started, Some(7));
}
