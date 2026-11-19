use dlms_learning_lab::{
    fs_integrity::{Inode, Volume},
    fs_integrity_terminal::{self as t, Command, DiagnosticKind, Disk, System},
};
use std::collections::BTreeMap;
fn volume() -> Volume {
    Volume {
        version: 1,
        block_count: 32,
        inode_count: 32,
        files: BTreeMap::from([(
            12,
            Inode {
                content: "retained report\n".into(),
                blocks: vec![8],
                links: 1,
            },
        )]),
        entries: BTreeMap::from([("report.txt".into(), 12)]),
        allocated_blocks: (0..9).collect(),
        allocated_inodes: (1..=12).collect(),
        free_blocks: 23,
        free_inodes: 20,
        mounted: true,
    }
}
fn system() -> System {
    System {
        version: 1,
        disks: vec![
            Disk {
                device: "/dev/vdb1".into(),
                mountpoint: "/mnt/data".into(),
                volume: volume(),
            },
            Disk {
                device: "/dev/vdc1".into(),
                mountpoint: "/mnt/notes".into(),
                volume: volume(),
            },
        ],
    }
}
#[test]
fn literal_parser_accepts_supported_equivalent_flags_and_refuses_conflicting_policies() {
    for line in [
        "e2fsck -fn /dev/vdb1",
        "e2fsck -n -f /dev/vdb1",
        "fsck.ext4 -nf /dev/vdb1",
        "e2fsck -ffnn /dev/vdb1",
    ] {
        assert_eq!(
            t::parse(line).unwrap(),
            Command::Check {
                device: "/dev/vdb1".into(),
                mode: dlms_learning_lab::fs_integrity::Mode::ReadOnly
            }
        );
    }
    for line in [
        "e2fsck -fny /dev/vdb1",
        "e2fsck -fp -n /dev/vdb1",
        "e2fsck -fn",
    ] {
        assert_eq!(t::parse(line).unwrap_err().kind, DiagnosticKind::Syntax);
    }
    for line in [
        "e2fsck -n /dev/vdb1",
        "e2fsck -f /dev/vdb1",
        "e2fsck -fc /dev/vdb1",
        "fsck -t ext4 -fn /dev/vdb1",
    ] {
        assert_eq!(
            t::parse(line).unwrap_err().kind,
            DiagnosticKind::Unsupported
        );
    }
}
#[test]
fn information_queries_report_actual_stored_state_and_never_repair_it() {
    let mut s = system();
    s.disks[0].volume.free_blocks = 2;
    let original = s.clone();
    let out = s.execute("findmnt").unwrap();
    assert!(out.stdout.contains("/mnt/data /dev/vdb1 ext4 rw"));
    assert!(out.stdout.contains("/mnt/notes /dev/vdc1 ext4 rw"));
    assert_eq!(s.execute("findmnt /not-supplied").unwrap().status, 1);
    let out = s.execute("dumpe2fs -h /dev/vdb1").unwrap();
    assert!(out.stdout.contains("Free blocks (stored): 2"));
    assert!(out.stdout.contains("Mounted"));
    let out = s.execute("ls -li /mnt/data").unwrap();
    assert!(out.stdout.contains("12 -rw-r--r-- 1 root root 16"));
    assert!(out.stdout.contains("11 drwx------"));
    assert_eq!(
        s.execute("cat /mnt/data/report.txt").unwrap().stdout,
        "retained report\n"
    );
    assert_eq!(s, original);
}
#[test]
fn actual_offline_repair_verifies_corrected_exit_one_and_preserves_the_other_volume() {
    let mut s = system();
    s.disks[0].volume.allocated_blocks.remove(&8);
    s.disks[0].volume.free_blocks = 30;
    let original = s.clone();
    let e = s.execute("e2fsck -fn /dev/vdb1").unwrap_err();
    assert_eq!((e.kind, e.status), (DiagnosticKind::Policy, 8));
    assert_eq!(s, original);
    assert!(s.execute("umount /mnt/data").unwrap().changed);
    let before = s.clone();
    let r = s.execute("e2fsck -fn /dev/vdb1").unwrap();
    assert_eq!((r.status, r.changed), (4, false));
    assert!(r.stdout.contains("used blocks marked free [8]"));
    assert_eq!(s, before);
    let r = s.execute("fsck.ext4 -p -f /dev/vdb1").unwrap();
    assert_eq!((r.status, r.changed), (1, true));
    assert!(r.stdout.contains("errors corrected"));
    let r = s.execute("e2fsck -nf /dev/vdb1").unwrap();
    assert_eq!((r.status, r.changed), (0, false));
    s.execute("mount /dev/vdb1 /mnt/data").unwrap();
    assert_eq!(
        s.execute("cat /mnt/data/report.txt").unwrap().stdout,
        "retained report\n"
    );
    assert_eq!(s.disks[1], original.disks[1]);
    s.disks[0].volume.free_blocks = 0;
    assert!(!s.disks[0].volume.findings().unwrap().is_empty());
    assert!(!s.execute("findmnt /mnt/data").unwrap().changed);
    assert!(!s.disks[0].volume.findings().unwrap().is_empty());
}
#[test]
fn orphan_recovery_requires_a_decision_and_reads_retained_bytes_through_its_new_name() {
    let mut s = system();
    let v = &mut s.disks[0].volume;
    v.mounted = false;
    v.files.insert(
        14,
        Inode {
            content: "recovered handover\n".into(),
            blocks: vec![10],
            links: 1,
        },
    );
    let original = s.clone();
    let r = s.execute("e2fsck -fp /dev/vdb1").unwrap();
    assert_eq!((r.status, r.changed), (4, false));
    assert!(r.stdout.contains("recovery decision"));
    assert_eq!(s, original);
    let r = s.execute("e2fsck -fy /dev/vdb1").unwrap();
    assert_eq!((r.status, r.changed), (1, true));
    assert_eq!(s.execute("e2fsck -fn /dev/vdb1").unwrap().status, 0);
    s.execute("mount /dev/vdb1 /mnt/data").unwrap();
    assert!(
        s.execute("ls -li /mnt/data/lost+found")
            .unwrap()
            .stdout
            .contains("#14")
    );
    assert_eq!(
        s.execute("cat /mnt/data/lost+found/#14").unwrap().stdout,
        "recovered handover\n"
    );
    assert_eq!(s.disks[1], original.disks[1]);
}
#[test]
fn unsupported_input_and_host_escape_attempts_never_change_virtual_state() {
    let mut s = system();
    let original = s.clone();
    for input in [
        "cat /etc/passwd",
        "cat /mnt/data/../../secret",
        "cat /mnt/dataevil/report.txt",
        "mount /dev/vdb1 /home/user",
        "mount /dev/sda1 /mnt/data",
        "umount /",
        "e2fsck -fy /dev/sda1",
        "sh -c touch",
        "e2fsck -fy /dev/vdb1; touch file",
        "cat /mnt/data/$(id)",
        "cat /mnt/data/*",
        "cat /mnt/data/report.txt\nmount /dev/vdb1 /mnt/data",
        "e2fsck -fy /dev/vdb1 | cat",
        "touch /mnt/data/report.txt",
        "mkfs.ext4 /dev/vdb1",
    ] {
        assert!(s.execute(input).is_err(), "{input}");
        assert_eq!(s, original, "{input}");
    }
    assert_eq!(
        s.execute(&"x".repeat(t::MAX_COMMAND + 1)).unwrap_err().kind,
        DiagnosticKind::Limit
    );
    assert_eq!(s, original);
    s.execute("umount /mnt/data").unwrap();
    assert!(
        s.execute("cat /mnt/data/report.txt")
            .unwrap_err()
            .message
            .contains("unmounted")
    );
}
#[test]
fn bounded_console_schema_rejects_unknown_fields_and_oversized_or_ambiguous_devices() {
    let s = system();
    let bytes = serde_json::to_vec(&s).unwrap();
    assert_eq!(System::decode(&bytes).unwrap(), s);
    let raw = String::from_utf8(bytes).unwrap();
    let bad = raw.replace("\"version\":1", "\"version\":1,\"execute_host\":true");
    assert!(System::decode(bad.as_bytes()).is_err());
    let mut v = serde_json::to_value(&s).unwrap();
    let extra = v["disks"][0].clone();
    v["disks"].as_array_mut().unwrap().push(extra);
    assert!(System::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut bad = s.clone();
    bad.disks[1].device = bad.disks[0].device.clone();
    assert!(bad.validate().is_err());
    let mut bad = s.clone();
    bad.disks[1].mountpoint = "/mnt/../secret".into();
    assert!(bad.validate().is_err());
    assert!(System::decode(&vec![b' '; t::MAX_SYSTEM + 1]).is_err());
    for command in [
        "help",
        "findmnt",
        "ls -li /mnt/data",
        "dumpe2fs -h /dev/vdb1",
        "cat /mnt/data/report.txt",
    ] {
        assert!(s.clone().execute(command).unwrap().stdout.len() <= t::MAX_OUTPUT);
    }
}
