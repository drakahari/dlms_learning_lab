//! Bounded virtual metadata only; never invokes host LVM/devices.
use dlms_learning_lab::lvm::{self, DeviceRole, ErrorKind, MIB, provision::Profile};
fn old() -> lvm::State {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/lvm-growth-v1.json")).unwrap();
    lvm::State::from_json(
        &serde_json::to_vec(&p["scenarios"][0]["goal"]["definition"]["initial"]).unwrap(),
    )
    .unwrap()
}
fn fixture() -> lvm::State {
    let mut s = old();
    s.provisioning = Some(Profile::Lifecycle);
    for p in ["/dev/reta", "/dev/retb"] {
        s.devices.insert(
            p.into(),
            lvm::Device {
                pv_bytes: None,
                bytes: 33 * MIB,
                role: DeviceRole::Blank,
                group: String::new(),
            },
        );
    }
    s.execute("vgcreate retired /dev/reta /dev/retb").unwrap();
    s.execute("lvcreate -l 2 -n scratch retired").unwrap();
    s.execute("lvcreate -l 1 -n buffer retired").unwrap();
    s
}
fn refused(s: &mut lvm::State, c: &str) -> lvm::Error {
    let before = s.clone();
    let e = s.execute(c).expect_err(c);
    assert_eq!(*s, before, "{c}");
    e
}
#[test]
fn remove_in_dependency_order_distinguishes_lv_group_and_pv_labels() {
    let mut s = fixture();
    let protected = s.groups["records"].clone();
    assert_eq!(s.free_extents("retired").unwrap(), 13);
    s.execute("lvremove -y retired/scratch").unwrap();
    assert_eq!(s.free_extents("retired").unwrap(), 15);
    assert_eq!(
        refused(&mut s, "vgremove retired").kind,
        ErrorKind::Unsupported
    );
    s.execute("lvremove --yes /dev/retired/buffer").unwrap();
    assert_eq!(s.free_extents("retired").unwrap(), 16);
    s.execute("vgremove retired").unwrap();
    assert!(!s.groups.contains_key("retired"));
    for p in ["/dev/reta", "/dev/retb"] {
        assert_eq!(s.devices[p].role, DeviceRole::PhysicalVolume);
        assert!(s.devices[p].group.is_empty());
    }
    assert!(s.execute("pvs").unwrap().contains("/dev/reta"));
    s.execute("pvremove /dev/reta /dev/retb").unwrap();
    for p in ["/dev/reta", "/dev/retb"] {
        assert_eq!(s.devices[p].role, DeviceRole::Blank);
        assert_eq!(s.devices[p].bytes, 33 * MIB);
    }
    assert_eq!(s.groups["records"], protected);
    assert_eq!(s.devices["/dev/recordsa"], old().devices["/dev/recordsa"]);
    s.validate().unwrap();
}
#[test]
fn valid_removal_orders_paths_and_yes_spellings_are_equivalent() {
    let mut results = Vec::new();
    for (a, b, vg, pv) in [
        (
            "retired/scratch",
            "retired/buffer",
            "vgremove retired",
            "pvremove /dev/reta /dev/retb",
        ),
        (
            "/dev/retired/buffer",
            "/dev/retired/scratch",
            "vgremove --yes retired",
            "pvremove /dev/retb /dev/reta",
        ),
        (
            "retired/buffer",
            "retired/scratch",
            "vgremove -y retired",
            "pvremove /dev/reta /dev/retb",
        ),
    ] {
        let mut s = fixture();
        for c in [
            format!("lvremove -y {a}"),
            format!("lvremove -y {b}"),
            vg.into(),
            pv.into(),
        ] {
            s.execute(&c).unwrap();
        }
        results.push(s);
    }
    assert!(results.windows(2).all(|v| v[0] == v[1]));
}
#[test]
fn grouped_foreign_blank_and_missing_pv_refuse_atomically() {
    let mut s = fixture();
    for c in [
        "pvremove /dev/reta",
        "pvremove /dev/recordsforeign",
        "pvremove /dev/missing",
        "vgremove records",
        "lvremove -y records/reports",
    ] {
        refused(&mut s, c);
    }
    s.execute("lvremove -y retired/scratch").unwrap();
    s.execute("lvremove -y retired/buffer").unwrap();
    s.execute("vgremove retired").unwrap();
    for c in [
        "pvremove /dev/reta /dev/recordsa",
        "pvremove /dev/reta /dev/recordsforeign",
        "pvremove /dev/reta /dev/missing",
        "pvremove /dev/reta /dev/reta",
    ] {
        refused(&mut s, c);
    }
    s.execute("pvremove /dev/reta").unwrap();
    refused(&mut s, "pvremove /dev/retb /dev/reta");
    assert_eq!(s.devices["/dev/retb"].role, DeviceRole::PhysicalVolume);
}
#[test]
fn empty_existing_group_can_be_removed_but_all_groups_need_not_disappear() {
    let mut s = fixture();
    s.execute("lvremove -y retired/scratch").unwrap();
    s.execute("lvremove -y retired/buffer").unwrap();
    let text = s.execute("vgremove -y retired").unwrap();
    assert!(text.contains("remain labeled"));
    refused(&mut s, "vgremove retired");
    assert!(s.groups.contains_key("records"));
    let text = s.execute("pvremove /dev/retb").unwrap();
    assert!(text.contains("not secure erasure"));
    assert_eq!(s.devices["/dev/reta"].role, DeviceRole::PhysicalVolume);
}
#[test]
fn removed_label_can_be_prepared_again_without_inventing_a_device_or_filesystem() {
    let mut s = fixture();
    let inventory = s.devices.keys().cloned().collect::<Vec<_>>();
    s.execute("lvremove -y retired/scratch").unwrap();
    s.execute("lvremove -y retired/buffer").unwrap();
    s.execute("vgremove retired").unwrap();
    s.execute("pvremove /dev/reta /dev/retb").unwrap();
    s.execute("pvcreate /dev/reta").unwrap();
    assert_eq!(s.devices["/dev/reta"].role, DeviceRole::PhysicalVolume);
    s.execute("vgcreate recovery /dev/reta").unwrap();
    s.execute("lvcreate -n newraw -L 4M recovery").unwrap();
    assert!(s.volume("recovery/newraw").unwrap().1.filesystem.is_none());
    assert_eq!(s.devices.keys().cloned().collect::<Vec<_>>(), inventory);
    assert_eq!(
        lvm::State::from_json(&serde_json::to_vec(&s).unwrap()).unwrap(),
        s
    );
}
#[test]
fn force_aliases_paths_shell_and_resource_escapes_are_honest_refusals() {
    let mut s = fixture();
    for c in [
        "vgremove -f retired",
        "vgremove -ff retired",
        "vgremove retired records",
        "vgremove --yes=1 retired",
        "vgremove /dev/retired",
        "vgremove ../retired",
        "pvremove -ff /dev/reta",
        "pvremove --yes /dev/reta",
        "pvremove /dev/../reta",
        "pvremove",
        "vgremove",
        "vgremove retired; id",
        "pvremove $(id)",
        "pvremove /dev/reta | cat",
        "pvremove /dev/reta\nvgremove records",
    ] {
        refused(&mut s, c);
    }
    assert_eq!(refused(&mut s, &"x".repeat(2049)).kind, ErrorKind::Limit);
}
#[test]
fn absent_and_linear_profiles_preserve_refusals_and_serialized_identity() {
    let mut source = old();
    assert!(
        serde_json::to_value(&source)
            .unwrap()
            .get("provisioning")
            .is_none()
    );
    for profile in [None, Some(Profile::Linear)] {
        source.provisioning = profile;
        for c in [
            "vgremove records",
            "pvremove /dev/recordsa",
            "vgremove -f records",
        ] {
            let e = refused(&mut source, c);
            assert_eq!(e.kind, ErrorKind::Unsupported);
            assert_eq!(e.message, source.command_help());
        }
    }
    let mut s = fixture();
    let e = refused(&mut s, "vgreduce retired /dev/reta");
    assert_eq!(e.message, lvm::retirement::HELP);
    assert!(s.command_help().contains("only empty") || s.command_help().contains("EMPTY"));
}
