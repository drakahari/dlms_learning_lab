//! Pure in-process fixtures; never host mdadm, devices or personal files.
use dlms_learning_lab::{raid_assembly as c, raid_assembly_workspace as w};
#[path = "support/raid_assembly.rs"]
mod f;
#[test]
fn context_requires_distinct_clean_complete_replica_identities() {
    let a = f::activity(false);
    a.validate().unwrap();
    let mut b = a.clone();
    b.context.identities[1].uuid = b.context.identities[0].uuid.clone();
    assert!(b.validate().is_err());
    let mut b = a.clone();
    b.context
        .replicas
        .arrays
        .get_mut("/dev/md0")
        .unwrap()
        .members
        .pop();
    assert!(b.validate().is_err());
}
#[test]
fn uuid_equivalence_and_malformed_values() {
    let u = c::Uuid::parse("abcdef0123456789abcdef0123456789").unwrap();
    assert_eq!(
        u,
        c::Uuid::parse("ABCDEF01:23456789:ABCDEF01:23456789").unwrap()
    );
    for bad in [
        "",
        "1234",
        "g0000000000000000000000000000000",
        "$(touch)",
        "é",
    ] {
        assert!(c::Uuid::parse(bad).is_err());
    }
}
#[test]
fn parser_supports_literal_devices_defaults_case_comments_and_three_letter_keywords() {
    let a = f::activity(false);
    let expected = f::correct(&a);
    let p = c::policy(&a.context, &expected).unwrap();
    assert_eq!(p.selection(&a.context, "/dev/md0").unwrap(), f::uuid(1));
    let alt = format!(
        "# sample\narr /dev/md0 UUID={} # intended identity\nArRaY /dev/md1 uuid={}\n",
        f::uuid(1).display(),
        f::uuid(2).display()
    );
    let p = c::policy(&a.context, &alt).unwrap();
    assert_eq!(p.selection(&a.context, "/dev/md0").unwrap(), f::uuid(1));
    assert_eq!(p.devices.len(), 4);
}
#[test]
fn unsupported_configuration_never_approximates_discovery_or_shell() {
    let a = f::activity(false);
    for s in [
        "DEVICE /dev/vd*",
        "HOMEHOST <system>",
        "ARRAY /dev/md0 name=host:0",
        " ARRAY /dev/md0 UUID=1234",
        "DEVICE '/dev/vda1'",
        "DEVICE /dev/vda1; echo bad",
        "DEVICE partitions",
        "ARRAY /dev/md0 UUID=1234",
    ] {
        assert!(c::policy(&a.context, s).is_err(), "{s}");
    }
}
#[test]
fn missing_array_policy_discloses_real_discovery_instead_of_false_general_rule() {
    let a = f::activity(false);
    let p = c::policy(&a.context, "").unwrap();
    let e = p.selection(&a.context, "/dev/md0").unwrap_err();
    assert!(e.contains("Real mdadm may discover"));
}
#[test]
fn duplicate_policy_and_missing_component_fail_specifically() {
    let a = f::activity(false);
    let s = format!(
        "ARRAY /dev/md0 UUID={}\nARRAY /dev/md1 UUID={}\n",
        f::uuid(1).display(),
        f::uuid(1).display()
    );
    assert!(c::policy(&a.context, &s).unwrap_err().contains("duplicate"));
    let s = format!(
        "DEVICE /dev/vda1\nARRAY /dev/md0 UUID={}\n",
        f::uuid(1).display()
    );
    assert!(
        c::policy(&a.context, &s)
            .unwrap()
            .selection(&a.context, "/dev/md0")
            .unwrap_err()
            .contains("omits")
    );
}
#[test]
fn saved_configuration_does_not_assemble_or_overwrite_existing_data() {
    let a = f::activity(false);
    let mut r = a.initial.clone();
    let old = a.context.clone();
    let live = r.live.clone();
    r.save(&a.context, f::correct(&a)).unwrap();
    assert_eq!(r.live, live);
    assert_eq!(a.context, old);
    assert!(a.context.read(&r.live, "/dev/md0", "/records.txt").is_err());
}
#[test]
fn targeted_scan_assembles_both_original_members_and_status_is_derived() {
    let a = f::activity(false);
    let mut r = a.initial.clone();
    r.save(&a.context, f::correct(&a)).unwrap();
    assert_eq!(
        r.execute(&a.context, "mdadm --assemble --scan /dev/md0")
            .unwrap()
            .status,
        0
    );
    let s = r.execute(&a.context, "cat /proc/mdstat").unwrap();
    assert!(s.output.contains("md0 : active raid1 vda1[0] vdb1[1]"));
    assert!(s.output.contains("[2/2] [UU]"));
    assert!(
        a.context
            .read(&r.live, "/dev/md0", "/records.txt")
            .unwrap()
            .contains("Original dataset 0-0")
    );
    assert_eq!(
        a.context.read(&r.live, "/dev/md1", "/records.txt").unwrap(),
        a.context
            .replicas
            .read_file("/dev/md1", "/records.txt")
            .unwrap()
    );
}
#[test]
fn explicit_reversed_member_order_works_without_consulting_broken_saved_policy() {
    let a = f::activity(false);
    let mut r = a.initial.clone();
    let cmd = format!(
        "mdadm -A /dev/md0 --uuid={} /dev/vdb1 /dev/vda1",
        f::uuid(1).display()
    );
    assert_eq!(r.execute(&a.context, &cmd).unwrap().status, 0);
    assert!(a.checks(&r).iter().any(|v| !v.0));
    assert!(
        a.checks(&r)
            .iter()
            .filter(|r| r.1.contains("Active array"))
            .all(|r| r.0)
    );
}
#[test]
fn mixed_duplicate_busy_and_wrong_uuid_members_fail_atomically() {
    let a = f::activity(false);
    for (u, d1, d2) in [
        (1, "/dev/vda1", "/dev/vdc1"),
        (1, "/dev/vda1", "/dev/vda1"),
        (99, "/dev/vda1", "/dev/vdb1"),
        (2, "/dev/vdc1", "/dev/vdd1"),
    ] {
        let mut r = a.initial.clone();
        let old = r.clone();
        let o = r
            .execute(
                &a.context,
                &format!(
                    "mdadm --assemble /dev/md0 --uuid={} {d1} {d2}",
                    f::uuid(u).display()
                ),
            )
            .unwrap();
        assert_eq!(o.status, 1);
        assert_eq!(r, old);
    }
}
#[test]
fn unsupported_commands_never_run_or_mutate() {
    let a = f::activity(false);
    for input in [
        "mdadm --assemble --scan",
        "mdadm --create /dev/md0",
        "sh -c echo",
        "cat /etc/passwd",
        "mdadm --stop /dev/md0; reboot",
        "cat /proc/mdstat\ncat /etc/mdadm.conf",
        "mdadm --assemble --force /dev/md0",
        "cat /../proc/mdstat",
    ] {
        let mut r = a.initial.clone();
        assert!(r.execute(&a.context, input).is_err(), "{input}");
        assert_eq!(r, a.initial);
    }
}
#[test]
fn stop_retains_replica_metadata_and_data_and_removes_active_io() {
    let a = f::activity(false);
    let mut r = a.initial.clone();
    let original = a.context.clone();
    assert_eq!(
        r.execute(&a.context, "mdadm -S /dev/md1").unwrap().status,
        0
    );
    assert!(a.context.read(&r.live, "/dev/md1", "/records.txt").is_err());
    assert_eq!(a.context, original);
    assert_eq!(
        r.execute(&a.context, "mdadm --detail /dev/md1")
            .unwrap()
            .status,
        1
    );
    assert!(
        !r.execute(&a.context, "cat /proc/mdstat")
            .unwrap()
            .output
            .contains("md1 :")
    );
}
#[test]
fn bounds_and_closed_decoders_reject_overflow_duplicate_fields_and_unknowns() {
    let a = f::activity(false);
    assert!(c::text(&"x".repeat(4097)).is_err());
    assert!(c::text(&"\n".repeat(65)).is_err());
    let raw = serde_json::to_string(&a.initial).unwrap();
    assert!(
        serde_json::from_str::<c::Runtime>(&raw.replacen("{", "{\"unknown\":true,", 1)).is_err()
    );
    assert!(serde_json::from_str::<c::Runtime>(&raw.replacen("{", "{\"version\":1,", 1)).is_err());
    let mut r = a.initial.clone();
    r.live.push(r.live[0].clone());
    assert!(r.validate(&a.context).is_err());
}
#[test]
fn no_claim_that_simulated_data_view_is_a_real_mounted_filesystem() {
    let a = f::activity(false);
    let mut r = a.initial.clone();
    assert!(r.execute(&a.context, "cat /dev/md1/records.txt").is_err());
    assert!(
        a.context
            .read(&r.live, "/dev/md1", "/../records.txt")
            .is_err()
    );
    assert_eq!(
        r.execute(&a.context, "mdadm -E /dev/vda1").unwrap().status,
        0
    );
    assert!(
        r.execute(&a.context, "mdadm -E /dev/vda1")
            .unwrap()
            .output
            .contains("Array UUID")
    );
}
#[test]
fn runtime_roundtrip_keeps_saved_policy_and_active_members_without_auto_execution() {
    let a = f::activity(false);
    let mut r = a.initial.clone();
    r.save(&a.context, f::correct(&a)).unwrap();
    let raw = serde_json::to_string(&r).unwrap();
    let next: c::Runtime = serde_json::from_str(&raw).unwrap();
    next.validate(&a.context).unwrap();
    assert_eq!(r, next);
    assert_eq!(next.live.len(), 1);
    let _: w::Activity = a;
}
