//! Planned tests for the supplied FAT32 layout; no reference command execution.
use dlms_learning_lab::partition_fs::fat::{Path, Volume};
fn volume() -> Volume {
    Volume {
        serial: "57A1-CF00".into(),
        label: "OPERATIONS".into(),
        sectors_per_cluster: 4,
        data_clusters: 65_536,
    }
}
#[test]
fn data_capacity_does_not_include_reserved_and_fat_sectors() {
    let f = volume();
    // ceil(65538*4/512) = 513 sectors per FAT; plus32 reserved.
    let sectors = 262_144 + 32 + 2 * 513;
    f.validate(sectors).unwrap();
    assert_eq!(f.data_bytes(), 262_144 * 512);
    assert_eq!(f.allocation_bytes(1), Some(2048));
    assert_eq!(f.allocation_bytes(2049), Some(4096));
    assert!(f.validate(sectors - 1).is_err());
    let mut too_small = f.clone();
    too_small.data_clusters = 65_524;
    assert!(too_small.validate(sectors).is_err());
    let mut bad = f;
    bad.sectors_per_cluster = 3;
    assert!(bad.validate(sectors).is_err());
}
#[test]
fn case_insensitive_lookup_has_one_canonical_serialization_and_no_escape() {
    let real = Path::parse("\\EFI\\Operations\\grubx64.efi").unwrap();
    assert_eq!(real, Path::parse("/efi/OPERATIONS/GRUBX64.EFI").unwrap());
    assert_eq!(real.as_str(), "/EFI/OPERATIONS/GRUBX64.EFI");
    assert_eq!(real.firmware_spelling(), "\\EFI\\OPERATIONS\\GRUBX64.EFI");
    assert!(real.is_application());
    assert_eq!(
        serde_json::from_str::<Path>(&serde_json::to_string(&real).unwrap()).unwrap(),
        real
    );
    assert!(serde_json::from_str::<Path>("\"/efi/Operations/grubx64.efi\"").is_err());
    for invalid in [
        "EFI/BOOT/BOOTX64.EFI",
        "/etc/passwd",
        "/EFI/../OUT.EFI",
        "/EFI//BAD.EFI",
        "/EFI/BOOT\\BOOTX64.EFI",
        "/EFI/END.",
        "/EFI/BOOT/A\0.EFI",
    ] {
        assert!(Path::parse(invalid).is_err(), "{invalid:?}");
    }
}
