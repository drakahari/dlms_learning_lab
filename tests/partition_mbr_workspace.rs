//! Pure typed outcomes; native workflows have separate focused evidence.
use core::mbr::{Kind, Profile};
use dlms_learning_lab::{partition_fs as core, partition_fs_workspace as work};
use serde_json::{Value, json};
use work::mbr::{self as goal, Plan, Target};
const MIB: u64 = 1024 * 1024;
fn old() -> work::Definition {
    let p: Value =
        serde_json::from_str(include_str!("../scenarios/disk-preparation-v1.json")).unwrap();
    serde_json::from_value(p["scenarios"][0]["goal"]["definition"].clone()).unwrap()
}
fn initial() -> core::State {
    let mut s = old().initial;
    s.mbr_profile = Some(Profile::Dos512);
    let retained = s.disks.remove("/dev/vda").unwrap();
    let blank = s.disks.remove("/dev/vdb").unwrap();
    s.disks.insert("/dev/vda".into(), blank);
    s.disks.insert("/dev/vdb".into(), retained);
    s.validate().unwrap();
    s
}
fn target(purpose: &str, role: Kind, min: u64, max: u64, swap: bool, label: &str) -> Target {
    Target {
        purpose: purpose.into(),
        eligible_disks: vec!["/dev/vda".into()],
        role,
        supplied_slot: None,
        minimum_bytes: min * MIB,
        maximum_bytes: max * MIB,
        aligned_start: true,
        content: if swap {
            work::DesiredContent::Swap {
                label: Some(label.into()),
            }
        } else {
            work::DesiredContent::Ext4 {
                label: Some(label.into()),
            }
        },
    }
}
fn plan() -> Plan {
    Plan {
        targets: vec![
            target("reports", Kind::Primary, 8, 12, false, "reports"),
            target("paging", Kind::Primary, 2, 4, true, "scratch"),
        ],
    }
}
fn run(s: &mut core::State, commands: &[&str]) {
    for c in commands {
        s.execute(c).unwrap();
    }
}
fn passes(initial: &core::State, s: &core::State, p: &Plan) -> bool {
    let r = goal::outcomes(initial, s, p);
    !r.is_empty() && r.iter().all(|r| r.0)
}
#[test]
fn alternate_allocation_order_sizes_positions_and_supported_format_forms_pass() {
    let initial = initial();
    let p = plan();
    p.validate(&initial).unwrap();
    let variants = [
        vec![
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart primary ext4 2048s 18431s",
            "parted -s /dev/vda unit s mkpart primary linux-swap 20480s 24575s",
            "mkfs.ext4 -L reports /dev/vda1",
            "mkswap -L scratch /dev/vda2",
        ],
        vec![
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart primary linux-swap 4096s 12287s",
            "parted -s /dev/vda unit s mkpart primary ext4 16384s 40959s",
            "mkswap -L scratch /dev/vda1",
            "mke2fs -t ext4 -b 4096 -L reports /dev/vda2",
        ],
    ];
    for commands in variants {
        let mut s = initial.clone();
        run(&mut s, &commands);
        assert!(
            passes(&initial, &s, &p),
            "{:?}",
            goal::outcomes(&initial, &s, &p)
        );
        assert_eq!(s.disks["/dev/vdb"], initial.disks["/dev/vdb"]);
    }
}
#[test]
fn partial_unformatted_wrong_hint_and_wrong_role_have_actual_specific_feedback() {
    let initial = initial();
    let p = plan();
    let mut s = initial.clone();
    run(
        &mut s,
        &[
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart primary ext4 2048s 18431s",
            "parted -s /dev/vda unit s mkpart primary linux-swap 20480s 24575s",
        ],
    );
    let rows = goal::outcomes(&initial, &s, &p);
    assert!(!rows[0].0 && !rows[1].0);
    assert!(rows[0].1.contains("unformatted"));
    assert!(rows[0].1.contains("8 MiB–12 MiB"), "{}", rows[0].1);
    assert!(
        rows[0].1.contains("8 MiB"),
        "Observed capacity must use the same units as the task"
    );
    let mut exact_sectors = p.clone();
    exact_sectors.targets[0].minimum_bytes += 512;
    let rows = goal::outcomes(&initial, &s, &exact_sectors);
    assert!(
        rows[0].1.contains("8389120 bytes"),
        "Do not round non-MiB sector boundaries: {}",
        rows[0].1
    );
    s.execute("mkfs.ext4 -L reports /dev/vda1").unwrap();
    let rows = goal::outcomes(&initial, &s, &p);
    assert!(rows[0].0 && !rows[1].0);
    assert!(!passes(&initial, &s, &p));
    s.execute("mkswap -L scratch /dev/vda2").unwrap();
    assert!(passes(&initial, &s, &p));
    let mut wrong = s.clone();
    wrong
        .disks
        .get_mut("/dev/vda")
        .unwrap()
        .mbr
        .as_mut()
        .unwrap()
        .entries[1]
        .partition
        .type_hint = core::TypeHint::LinuxData;
    let rows = goal::outcomes(&initial, &wrong, &p);
    assert!(!rows[1].0);
    assert!(rows[1].1.contains("Linux data"));
    let mut logical = p.clone();
    logical.targets[0].role = Kind::Logical;
    let rows = goal::outcomes(&initial, &s, &logical);
    assert!(!rows[0].0);
    assert!(rows[0].1.contains("logical"));
    assert!(rows[0].1.contains("primary"), "{}", rows[0].1);
}
#[test]
fn logical_goals_require_a_real_container_and_original_allocations_remain_protected() {
    let initial = initial();
    let mut p = plan();
    for t in &mut p.targets {
        t.role = Kind::Logical;
    }
    let mut s = initial.clone();
    run(
        &mut s,
        &[
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart extended 2048s 65535s",
            "parted -s /dev/vda unit s mkpart logical ext4 4096s 20479s",
            "parted -s /dev/vda unit s mkpart logical linux-swap 24576s 28671s",
            "mkfs.ext4 -L reports /dev/vda5",
            "mkswap -L scratch /dev/vda6",
        ],
    );
    assert!(
        passes(&initial, &s, &p),
        "{:?}",
        goal::outcomes(&initial, &s, &p)
    );
    let mut changed = s.clone();
    changed
        .disks
        .get_mut("/dev/vdb")
        .unwrap()
        .table
        .as_mut()
        .unwrap()
        .partitions[0]
        .name = "changed".into();
    assert!(!work::same_problem(&initial, &changed));
    assert!(!passes(&initial, &changed, &p));
    let mut changed = s.clone();
    changed
        .disks
        .get_mut("/dev/vda")
        .unwrap()
        .mbr
        .as_mut()
        .unwrap()
        .entries[1]
        .partition
        .start += 1;
    // Keep the formatted payload's length/capacity valid; isolate alignment.
    changed
        .disks
        .get_mut("/dev/vda")
        .unwrap()
        .mbr
        .as_mut()
        .unwrap()
        .entries[1]
        .partition
        .end += 1;
    changed.validate().unwrap();
    assert!(!passes(&initial, &changed, &p), "Misaligned payload passed");
    assert_eq!(core::State::from_json(&s.to_json().unwrap()).unwrap(), s);
}
#[test]
fn one_payload_cannot_satisfy_two_requirements_and_matching_reassigns_broad_goals() {
    let initial = initial();
    let p = Plan {
        targets: vec![
            target("broad", Kind::Primary, 8, 12, false, "copy"),
            target("narrow", Kind::Primary, 8, 8, false, "copy"),
        ],
    };
    let mut s = initial.clone();
    run(
        &mut s,
        &[
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart primary ext4 2048s 18431s",
            "mkfs.ext4 -L copy /dev/vda1",
        ],
    );
    let rows = goal::outcomes(&initial, &s, &p);
    assert_eq!(rows[..2].iter().filter(|r| r.0).count(), 1);
    assert!(!passes(&initial, &s, &p));
    run(
        &mut s,
        &[
            "parted -s /dev/vda unit s mkpart primary ext4 20480s 45055s",
            "mkfs.ext4 -L copy /dev/vda2",
        ],
    );
    assert!(
        passes(&initial, &s, &p),
        "{:?}",
        goal::outcomes(&initial, &s, &p)
    );
    assert!(goal::outcomes(&initial, &s, &p)[0].1.contains("/dev/vda2"));
    assert!(goal::outcomes(&initial, &s, &p)[1].1.contains("/dev/vda1"));
}
#[test]
fn later_extra_work_clears_success_and_forged_files_or_profile_changes_do_not_pass() {
    let initial = initial();
    let p = plan();
    let mut s = initial.clone();
    run(
        &mut s,
        &[
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart primary ext4 2048s 18431s",
            "parted -s /dev/vda unit s mkpart primary linux-swap 20480s 24575s",
            "mkfs.ext4 -L reports /dev/vda1",
            "mkswap -L scratch /dev/vda2",
        ],
    );
    assert!(passes(&initial, &s, &p));
    let good = s.clone();
    s.execute("parted -s /dev/vda unit s mkpart primary ext4 28672s 32767s")
        .unwrap();
    let rows = goal::outcomes(&initial, &s, &p);
    assert!(rows[0].0 && rows[1].0);
    assert!(!rows.last().unwrap().0);
    assert!(!passes(&initial, &s, &p));
    let mut forged = good.clone();
    if let core::Content::Ext4 { files, .. } = &mut forged
        .disks
        .get_mut("/dev/vda")
        .unwrap()
        .mbr
        .as_mut()
        .unwrap()
        .entries[0]
        .partition
        .content
    {
        files.insert(
            "/invented.txt".into(),
            core::File {
                text: "not supplied or produced".into(),
            },
        );
    }
    forged.validate().unwrap();
    assert!(!work::same_problem(&initial, &forged));
    assert!(!passes(&initial, &forged, &p));
    let mut changed = good;
    changed.disks.get_mut("/dev/vda").unwrap().optimal_alignment = 4096;
    changed.validate().unwrap();
    assert!(!passes(&initial, &changed, &p));
    s.reset_from(&initial).unwrap();
    assert!(!passes(&initial, &s, &p));
    assert_eq!(s, initial);
}
#[test]
fn invalid_typed_goals_wire_bounds_and_unreachable_source_reference_refuse() {
    let initial = initial();
    let p = plan();
    let mut invalid = Vec::new();
    let mut q = p.clone();
    q.targets[0].role = Kind::Extended;
    invalid.push(q);
    let mut q = p.clone();
    q.targets[0].minimum_bytes = 512;
    invalid.push(q);
    let mut q = p.clone();
    q.targets[0].maximum_bytes = 7 * MIB;
    invalid.push(q);
    let mut q = p.clone();
    q.targets[0].eligible_disks = vec!["/dev/../vda".into()];
    invalid.push(q);
    let mut q = p.clone();
    q.targets[0].content = work::DesiredContent::Blank;
    invalid.push(q);
    let mut q = p.clone();
    q.targets[0].content = work::DesiredContent::Ext4 { label: None };
    invalid.push(q);
    let mut q = p.clone();
    q.targets[1].purpose = q.targets[0].purpose.clone();
    invalid.push(q);
    for q in invalid {
        assert!(q.validate(&initial).is_err());
        assert!(!passes(&initial, &initial, &q));
    }
    assert!(p.validate(&old().initial).is_err());
    let root = work::Plan {
        targets: vec![],
        mbr: Some(p.clone()),
        layout: None,
    };
    root.validate(&initial).unwrap();
    let raw = serde_json::to_string(&root).unwrap();
    let round: work::Plan = serde_json::from_str(&raw).unwrap();
    assert_eq!(root, round);
    let mut mixed = root.clone();
    mixed.targets = old().plan.targets;
    assert!(mixed.validate(&initial).is_err());
    let mut d = old();
    d.initial = initial;
    d.plan = root;
    // A compiled native path does not make an unreachable GPT reference a valid
    // MBR assignment: authoring admission must prove the actual worked approach.
    assert!(
        d.validate().is_err(),
        "Unreachable source reference was admitted"
    );
    let mut v = serde_json::to_value(&p).unwrap();
    let item = v["targets"][0].clone();
    v["targets"] = Value::Array(vec![item; 9]);
    assert!(serde_json::from_value::<Plan>(v).is_err());
    let mut v = serde_json::to_value(&p).unwrap();
    v["targets"][0]["host_path"] = json!("/dev/sda");
    assert!(serde_json::from_value::<Plan>(v).is_err());
    assert!(!serde_json::to_string(&old().plan).unwrap().contains("mbr"));
}

#[test]
fn supplied_blank_payload_cannot_be_replaced_by_a_new_matching_filesystem_elsewhere() {
    let mut initial = initial();
    run(
        &mut initial,
        &[
            "parted -s /dev/vda mklabel msdos",
            "parted -s /dev/vda unit s mkpart extended 2048s 65535s",
            "parted -s /dev/vda unit s mkpart logical ext4 4096s 12287s",
        ],
    );
    let mut t = target("supplied_reports", Kind::Logical, 4, 4, false, "reports");
    t.supplied_slot = Some(5);
    let p = Plan { targets: vec![t] };
    p.validate(&initial).unwrap();
    let mut wrong = initial.clone();
    run(
        &mut wrong,
        &[
            "parted -s /dev/vda unit s mkpart logical ext4 16384s 24575s",
            "mkfs.ext4 -L reports /dev/vda6",
        ],
    );
    assert!(!passes(&initial, &wrong, &p));
    assert!(
        goal::outcomes(&initial, &wrong, &p)[0]
            .1
            .contains("supplied /dev/vda5")
    );
    let mut flexible = p.clone();
    flexible.targets[0].supplied_slot = None;
    assert!(
        passes(&initial, &wrong, &flexible),
        "The optional identity is what distinguishes these tasks"
    );
    let mut correct = initial.clone();
    correct.execute("mkfs.ext4 -L reports /dev/vda5").unwrap();
    assert!(passes(&initial, &correct, &p));
    let mut invalid = p.clone();
    invalid.targets[0].supplied_slot = Some(6);
    assert!(invalid.validate(&initial).is_err());
    let mut invalid = p.clone();
    invalid.targets[0].content = work::DesiredContent::Swap {
        label: Some("scratch".into()),
    };
    assert!(
        invalid.validate(&initial).is_err(),
        "An immutable existing hint cannot be silently changed"
    );
    let mut read_only = initial;
    read_only.disks.get_mut("/dev/vda").unwrap().writable = false;
    assert!(p.validate(&read_only).is_err());
}

#[test]
fn safe_intermediate_allocation_is_not_mislabeled_as_unrequested_but_completion_is_strict() {
    let mut initial = initial();
    initial.disks.get_mut("/dev/vdc").unwrap().writable = true;
    let mut p = plan();
    for t in &mut p.targets {
        t.eligible_disks.push("/dev/vdc".into());
    }
    let mut s = initial.clone();
    for command in [
        "parted -s /dev/vda mklabel msdos",
        "parted -s /dev/vda unit s mkpart primary ext4 2048s 18431s",
        "mkfs.ext4 -L reports /dev/vda1",
        "parted -s /dev/vda unit s mkpart primary linux-swap 20480s 24575s",
    ] {
        s.execute(command).unwrap();
        let rows = goal::outcomes(&initial, &s, &p);
        assert!(!passes(&initial, &s, &p));
        assert!(
            rows.last().unwrap().0,
            "Safe requested intermediate work: {rows:?}"
        );
    }
    s.execute("mkswap -L scratch /dev/vda2").unwrap();
    assert!(passes(&initial, &s, &p));
    s.execute("parted -s /dev/vdc mklabel msdos").unwrap();
    assert!(
        !passes(&initial, &s, &p),
        "Unused media may not change after all requested work is present"
    );
    let mut p = plan();
    for t in &mut p.targets {
        t.role = Kind::Logical;
    }
    let mut s = initial.clone();
    s.execute("parted -s /dev/vda mklabel msdos").unwrap();
    s.execute("parted -s /dev/vda unit s mkpart extended 2048s 65535s")
        .unwrap();
    let rows = goal::outcomes(&initial, &s, &p);
    assert!(
        !rows[0].0 && !rows[1].0 && rows.last().unwrap().0,
        "Requested container is safe intermediate work: {rows:?}"
    );
}
