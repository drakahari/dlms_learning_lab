use dlms_learning_lab::pxe::{
    Architecture, Artifact, BootFile, Client, Diagnostic, FileRecord, Inventory, LinuxEntry,
    Policy, Request, bootstrap, configuration_candidates, resolve_linux_pair, trace_configuration,
};
use std::net::Ipv4Addr;

const SERVER: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 10);
const OLD_SERVER: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 11);
const BIOS: &str = "rescue/pxelinux.0";
const UEFI: &str = "boot/recovery.efi";
const CLIENT_MAC_KEY: &str = "01-02-00-00-00-00-15";

fn path(name: &str) -> BootFile {
    at(SERVER, name)
}

fn at(server: Ipv4Addr, name: &str) -> BootFile {
    BootFile {
        server,
        name: name.into(),
    }
}

fn file(name: &str, artifact: Artifact) -> FileRecord {
    FileRecord {
        file: path(name),
        artifact,
    }
}

fn client(request: Request, last: u8) -> Client {
    Client {
        address: Ipv4Addr::new(192, 0, 2, last),
        request,
        reachable_servers: [Some(SERVER), Some(OLD_SERVER), None, None],
        ethernet_mac: [0x02, 0x00, 0x00, 0x00, 0x00, last],
        uuid: None,
    }
}

fn pxe(code: u16) -> Request {
    Request::Pxe {
        option_93: Some(code),
    }
}

fn configuration(kernel: &str, initrd: &str) -> Artifact {
    Artifact::Configuration {
        source: format!("DEFAULT normal\nLABEL normal\nLINUX {kernel}\nINITRD {initrd}\n"),
    }
}

fn fixture() -> Inventory {
    Inventory::new(vec![
        file(BIOS, Artifact::BiosPxelinux6),
        file(UEFI, Artifact::X64EfiApplication),
        FileRecord {
            file: at(OLD_SERVER, UEFI),
            artifact: Artifact::BiosPxelinux6,
        },
        file("rescue/ldlinux.c32", Artifact::PxelinuxCore),
        file(
            "rescue/pxelinux.cfg/default",
            configuration("linux/fallback", "init/fallback.img"),
        ),
        file(
            "rescue/pxelinux.cfg/01-02-00-00-00-00-15",
            configuration("linux/current", "init/fallback.img"),
        ),
        file(
            "rescue/linux/fallback",
            Artifact::Kernel {
                release: "6.6.60".into(),
            },
        ),
        file(
            "rescue/init/fallback.img",
            Artifact::Initramfs {
                release: "6.6.60".into(),
            },
        ),
        file(
            "rescue/linux/current",
            Artifact::Kernel {
                release: "6.6.61".into(),
            },
        ),
        file(
            "rescue/init/current.img",
            Artifact::Initramfs {
                release: "6.6.61".into(),
            },
        ),
    ])
    .unwrap()
}

#[test]
fn served_program_metadata_and_server_identity_determine_firmware_compatibility() {
    let inventory = fixture();
    let mut policy = Policy {
        bios: Some(path(BIOS)),
        x64_uefi: Some(at(OLD_SERVER, UEFI)),
        unconditional: Some(path(BIOS)),
    };
    assert_eq!(
        bootstrap(&inventory, &policy, &client(pxe(0), 21))
            .unwrap()
            .unwrap()
            .architecture,
        Architecture::PcBios
    );
    assert_eq!(
        bootstrap(&inventory, &policy, &client(pxe(7), 22)),
        Err(Diagnostic::WrongProgram {
            file: at(OLD_SERVER, UEFI),
            architecture: Architecture::X64Uefi
        })
    );
    assert_eq!(
        bootstrap(&inventory, &policy, &client(Request::OrdinaryDhcp, 23)),
        Err(Diagnostic::UnexpectedOffer(path(BIOS)))
    );
    policy.x64_uefi = Some(path(UEFI));
    policy.unconditional = None;
    assert_eq!(
        bootstrap(&inventory, &policy, &client(pxe(7), 22))
            .unwrap()
            .unwrap()
            .file,
        path(UEFI)
    );
    assert_eq!(
        bootstrap(&inventory, &policy, &client(Request::OrdinaryDhcp, 23)),
        Ok(None)
    );
}

#[test]
fn absent_option_unsupported_architecture_unreachable_server_and_missing_path_differ() {
    let inventory = fixture();
    let mut policy = Policy::default();
    let mut efi = client(pxe(7), 22);
    assert_eq!(bootstrap(&inventory, &policy, &efi), Ok(None));
    assert_eq!(
        bootstrap(
            &inventory,
            &policy,
            &client(Request::Pxe { option_93: None }, 22)
        ),
        Err(Diagnostic::MissingArchitecture)
    );
    assert_eq!(
        bootstrap(&inventory, &policy, &client(pxe(9), 22)),
        Err(Diagnostic::UnsupportedArchitecture(9))
    );
    policy.x64_uefi = Some(path("boot/does-not-exist.efi"));
    let file = policy.x64_uefi.as_ref().unwrap().clone();
    assert_eq!(
        bootstrap(&inventory, &policy, &efi),
        Err(Diagnostic::MissingFile(file))
    );
    efi.reachable_servers[0] = None;
    assert_eq!(
        bootstrap(&inventory, &policy, &efi),
        Err(Diagnostic::UnreachableServer(SERVER))
    );
}

#[test]
fn pxelinux_searches_uuid_mac_then_hex_prefixes_and_default_in_program_directory() {
    let mut source = client(pxe(0), 21);
    source.uuid = Some("aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee".into());
    let candidates = configuration_candidates(&path(BIOS), &source).unwrap();
    let names: Vec<_> = candidates
        .iter()
        .map(|p| p.name.strip_prefix("rescue/pxelinux.cfg/").unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee",
            CLIENT_MAC_KEY,
            "C0000215",
            "C000021",
            "C00002",
            "C0000",
            "C000",
            "C00",
            "C0",
            "C",
            "default",
        ]
    );
    let trace = trace_configuration(&fixture(), &path(BIOS), &source).unwrap();
    assert_eq!(trace.attempted, candidates[..2]);
    assert_eq!(trace.selected, Some(candidates[1].clone()));

    let absent = Inventory::new(vec![file(BIOS, Artifact::BiosPxelinux6)]).unwrap();
    assert_eq!(
        trace_configuration(&absent, &path(BIOS), &source),
        Err(Diagnostic::MissingCore(path("rescue/ldlinux.c32")))
    );
}

#[test]
fn save_selected_boot_file_repair_restores_target_and_preserves_fallback_peer() {
    let mut inventory = fixture();
    let program = path(BIOS);
    let target = client(pxe(0), 21);
    let peer = client(pxe(0), 22);
    let before_peer = resolve_linux_pair(&inventory, &program, &peer).unwrap();
    assert_eq!(before_peer.release, "6.6.60");
    assert_eq!(
        resolve_linux_pair(&inventory, &program, &target),
        Err(Diagnostic::ReleaseMismatch {
            kernel: "6.6.61".into(),
            initramfs: "6.6.60".into()
        })
    );
    let fixed =
        "default repair\r\nLabel repair\r\nLINUX linux/current\r\nINITRD init/current.img\r\n";
    inventory
        .save_configuration(&program, CLIENT_MAC_KEY, fixed.into())
        .unwrap();
    let current = resolve_linux_pair(&inventory, &program, &target).unwrap();
    assert_eq!(current.kernel, path("rescue/linux/current"));
    assert_eq!(current.initramfs, path("rescue/init/current.img"));
    assert_eq!(current.release, "6.6.61");
    assert_eq!(
        resolve_linux_pair(&inventory, &program, &peer).unwrap(),
        before_peer
    );
}

#[test]
fn present_high_priority_wrong_file_does_not_fall_back_to_a_working_default() {
    let mut inventory = fixture();
    let mut source = client(pxe(0), 21);
    let uuid = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
    source.uuid = Some(uuid.into());
    let file = inventory
        .save_configuration(&path(BIOS), uuid, "INCLUDE other.cfg".into())
        .unwrap();
    let trace = trace_configuration(&inventory, &path(BIOS), &source).unwrap();
    assert_eq!(trace.attempted.as_slice(), std::slice::from_ref(&file));
    assert!(matches!(
        resolve_linux_pair(&inventory, &path(BIOS), &source),
        Err(Diagnostic::InvalidConfiguration { file: selected, detail })
            if selected == file && detail.contains("Configuration line 1")
    ));
}

#[test]
fn unsupported_paths_and_syntax_are_refused_without_mutating_the_inventory() {
    let mut inventory = fixture();
    let before = inventory.clone();
    for invalid in ["../ldlinux.c32", "foo/bar", "/default", "xx; echo", "."] {
        assert!(
            inventory
                .save_configuration(&path(BIOS), invalid, "changed".into())
                .is_err()
        );
    }
    assert!(
        inventory
            .save_configuration(&path(UEFI), "default", "changed".into())
            .is_err()
    );
    assert_eq!(inventory, before);
    for unsupported in [
        "DEFAULT linux\nLABEL linux\nLINUX ../escape\nINITRD good",
        "DEFAULT linux\nLABEL other\nLINUX good\nINITRD good",
        "DEFAULT linux\nLABEL linux\nLINUX good\nAPPEND initrd=good",
        "DEFAULT linux\nLABEL linux\nLINUX good; cat\nINITRD good",
    ] {
        assert!(LinuxEntry::parse(unsupported).is_err(), "{unsupported}");
    }
}

#[test]
fn bounded_input_and_duplicate_fixture_path_checks_preserve_typed_file_identity() {
    let earlier = fixture();
    let saved = serde_json::to_value(&earlier).unwrap();
    let roundtrip: Inventory = serde_json::from_value(saved.clone()).unwrap();
    roundtrip.validate().unwrap();
    assert_eq!(roundtrip, earlier);
    let records = earlier.files();
    let mut duplicates = records.to_vec();
    duplicates.push(records[0].clone());
    assert!(Inventory::new(duplicates).is_err());
    let mut excess = saved;
    excess["files"] =
        serde_json::Value::Array(vec![serde_json::to_value(&records[0]).unwrap(); 33]);
    assert!(serde_json::from_value::<Inventory>(excess).is_err());
}

mod saved {
    use super::*;
    use dlms_learning_lab::pxe::workspace::{
        Action, Case, EntryDraft, Mode, PolicyDraft, Requirement, Task, Work,
    };

    fn boot_policy() -> Task {
        let starter = Policy {
            bios: Some(path(BIOS)),
            x64_uefi: Some(at(OLD_SERVER, UEFI)),
            unconditional: Some(path(BIOS)),
        };
        let mut correct = PolicyDraft::from(&starter);
        correct.x64_uefi.server = SERVER.to_string();
        correct.unconditional.enabled = false;
        Task {
            id: "lpic2-pxe-offer-fixture".into(),
            instructions: "Restore compatible firmware offers and preserve ordinary DHCP.".into(),
            mode: Mode::Policy,
            starter_policy: starter,
            inventory: fixture(),
            cases: vec![
                Case {
                    label: "Legacy recovery".into(),
                    client: client(pxe(0), 21),
                    requirement: Requirement::Program {
                        architecture: Architecture::PcBios,
                    },
                },
                Case {
                    label: "UEFI recovery".into(),
                    client: client(pxe(7), 22),
                    requirement: Requirement::Program {
                        architecture: Architecture::X64Uefi,
                    },
                },
                Case {
                    label: "Ordinary client".into(),
                    client: client(Request::OrdinaryDhcp, 23),
                    requirement: Requirement::NoOffer,
                },
            ],
            reference: vec![
                Action::EditPolicy { draft: correct },
                Action::SavePolicy,
                Action::Check,
            ],
        }
    }

    fn config_draft(leaf: &str) -> EntryDraft {
        EntryDraft {
            leaf: leaf.into(),
            source: "DEFAULT repair\nLABEL repair\nLINUX linux/current\nINITRD init/current.img\n"
                .into(),
        }
    }

    fn configuration_task() -> Task {
        let mut target = client(pxe(0), 21);
        target.uuid = Some("aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee".into());
        Task {
            id: "lpic2-pxe-config-fixture".into(),
            instructions: "Repair the client-specific configuration and protect the fallback."
                .into(),
            mode: Mode::Configuration {
                program: path(BIOS),
                starter_leaf: CLIENT_MAC_KEY.into(),
            },
            starter_policy: Policy {
                bios: Some(path(BIOS)),
                ..Policy::default()
            },
            inventory: fixture(),
            cases: vec![
                Case {
                    label: "Maintenance host".into(),
                    client: target,
                    requirement: Requirement::Release {
                        release: "6.6.61".into(),
                    },
                },
                Case {
                    label: "Fallback peer".into(),
                    client: client(pxe(0), 22),
                    requirement: Requirement::PreservePair,
                },
            ],
            reference: vec![
                Action::EditEntry {
                    draft: config_draft(CLIENT_MAC_KEY),
                },
                Action::SaveEntry,
                Action::Check,
            ],
        }
    }

    #[test]
    fn saved_policy_is_required_and_invalid_edit_preserves_active_offer() {
        let task = boot_policy();
        task.validate().unwrap();
        let mut work = task.start().unwrap();
        task.apply(&mut work, &Action::Check).unwrap();
        assert_eq!(
            task.checks(&work)
                .iter()
                .filter(|check| check.passed)
                .count(),
            2
        );
        task.apply(&mut work, &task.reference[0]).unwrap();
        task.apply(&mut work, &Action::Check).unwrap();
        assert!(!task.solved(&work));
        assert!(!task.checks(&work)[0].passed);
        for action in &task.reference[1..] {
            task.apply(&mut work, action).unwrap();
        }
        assert!(task.solved(&work));
        let mut draft = work.policy_draft.clone();
        draft.x64_uefi.server = "not-an-ip".into();
        task.apply(&mut work, &Action::EditPolicy { draft })
            .unwrap();
        assert!(!task.solved(&work));
        let before = work.clone();
        assert!(task.apply(&mut work, &Action::SavePolicy).is_err());
        assert_eq!(work, before);
    }

    #[test]
    fn target_override_and_in_place_repairs_pass_but_changing_default_fails_protection() {
        let task = configuration_task();
        task.validate().unwrap();
        for leaf in [CLIENT_MAC_KEY, "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee"] {
            let mut work = task.start().unwrap();
            for action in [
                Action::EditEntry {
                    draft: config_draft(leaf),
                },
                Action::SaveEntry,
                Action::Check,
            ] {
                task.apply(&mut work, &action).unwrap();
            }
            assert!(task.solved(&work));
            for action in [
                Action::EditEntry {
                    draft: config_draft("default"),
                },
                Action::SaveEntry,
                Action::Check,
            ] {
                task.apply(&mut work, &action).unwrap();
            }
            let checks = task.checks(&work);
            assert!(!task.solved(&work));
            assert!(checks[1].passed && !checks[2].passed);
        }
    }

    #[test]
    fn unfinished_invalid_override_reopens_without_becoming_active_and_reset_is_exact() {
        let task = configuration_task();
        let mut work = task.start().unwrap();
        for action in &task.reference {
            task.apply(&mut work, action).unwrap();
        }
        let mut draft = config_draft("aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee");
        draft.source = "INCLUDE other.cfg".into();
        task.apply(&mut work, &Action::EditEntry { draft }).unwrap();
        assert!(!task.solved(&work));
        let serialized = serde_json::to_string(&work).unwrap();
        let mut read: Work = serde_json::from_str(&serialized).unwrap();
        assert_eq!(work, read);
        assert!(task.checks(&read)[1].passed);
        task.apply(&mut read, &Action::SaveEntry).unwrap();
        task.apply(&mut read, &Action::Check).unwrap();
        let result = task.checks(&read);
        assert!(!result[1].passed);
        assert!(result[1].detail.contains("Configuration line"));
        assert!(result[2].passed);
        task.apply(&mut read, &Action::Reset).unwrap();
        assert_eq!(read, task.start().unwrap());
    }

    #[test]
    fn malformed_task_cases_and_forged_saved_cross_mode_changes_are_refused() {
        let mut task = configuration_task();
        let mut work = task.start().unwrap();
        work.policy.x64_uefi = Some(path(UEFI));
        assert!(task.effective_inventory(&work).is_err());
        task.reference = vec![Action::Check];
        assert!(task.validate().is_err());
        let mut task = boot_policy();
        task.cases.pop();
        assert!(task.validate().is_err());
    }
}
