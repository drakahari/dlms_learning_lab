//! Pure virtual path references only; no native authoring/curriculum admission.
use dlms_learning_lab::lvm::{self, device_path as path};
use serde_json::Value;
fn supplied() -> lvm::State {
    let v: Value =
        serde_json::from_str(include_str!("../scenarios/lvm-rename-lessons-v1.json")).unwrap();
    let mut s: lvm::State =
        serde_json::from_value(v["scenarios"][0]["goal"]["definition"]["initial"].clone()).unwrap();
    let names = s.groups.keys().cloned().collect::<Vec<_>>();
    for g in names {
        s.execute(&format!("vgchange -ay {g}")).unwrap();
    }
    s.validate().unwrap();
    s
}
fn record(s: &lvm::State) -> (String, String, String, String) {
    s.groups
        .iter()
        .find_map(|(g, group)| {
            group.volumes.iter().find_map(|(v, lv)| {
                lv.filesystem.as_ref().and_then(|f| {
                    f.files
                        .iter()
                        .next()
                        .map(|(p, data)| (g.clone(), v.clone(), p.clone(), data.contents.clone()))
                })
            })
        })
        .unwrap()
}
#[test]
fn observed_hyphen_convention_roundtrips_without_claiming_it_is_a_production_abi() {
    for (g, v, node) in [
        ("pool", "report", "pool-report"),
        ("data-pool", "report-set", "data--pool-report--set"),
        ("data-", "report", "data---report"),
        ("a--b", "x-y", "a----b-x--y"),
        ("A_+.1", "X_+.2", "A_+.1-X_+.2"),
    ] {
        let alias = path::modeled_mapper(g, v).unwrap();
        assert_eq!(alias, format!("/dev/mapper/{node}"));
        let a = path::parse(&alias).unwrap();
        assert_eq!(a.form, path::Form::Mapper);
        assert_eq!((&*a.group, &*a.volume), (g, v));
        let canonical = path::canonical(g, v).unwrap();
        assert_eq!(canonical, format!("/dev/{g}/{v}"));
        let b = path::parse(&canonical).unwrap();
        assert_eq!(b.form, path::Form::Canonical);
        assert_eq!((a.group, a.volume), (b.group, b.volume));
    }
}
#[test]
fn many_name_pairs_have_distinct_encoded_aliases_and_do_not_split_literal_hyphens() {
    let names = [
        "a", "A", "a-b", "a--b", "a---", "a.b", "a_b", "a+b", "1", "1-a", "a-", "z---q",
    ];
    let mut seen = std::collections::BTreeSet::new();
    for g in names {
        for v in names {
            let encoded = path::modeled_mapper(g, v).unwrap();
            assert!(seen.insert(encoded.clone()));
            let parsed = path::parse(&encoded).unwrap();
            assert_eq!((parsed.group.as_str(), parsed.volume.as_str()), (g, v));
        }
    }
    assert_eq!(seen.len(), 144);
}
#[test]
fn malformed_layers_traversal_control_and_unsupported_device_forms_are_honest_errors() {
    for p in [
        "",
        "vg/lv",
        "/etc/passwd",
        "/dev/dm-0",
        "/dev/g",
        "/dev/g/l/v",
        "/dev/g/../l",
        "/dev//g/l",
        "/dev/g/%2fl",
        "/dev/g/l;sh",
        "/dev/g/l\n",
        "/dev/g/l\0",
        " /dev/g/l",
        "/dev/g/l ",
        "/dev/mapper/",
        "/dev/mapper/vg",
        "/dev/mapper/-lv",
        "/dev/mapper/vg-",
        "/dev/mapper/vg-lv-layer",
        "/dev/mapper/vg-lv\n",
        "/dev/mapper/../lv",
        "/dev/mapper/vg-snapshot",
        "/dev/mapper/vg-lv_tmeta",
        "/dev/mapper/vg-历史",
    ] {
        assert!(path::parse(p).is_err(), "{p:?}");
    }
    assert_eq!(
        path::parse("/dev/mapper/g-l-layer").unwrap_err().kind,
        lvm::ErrorKind::Unsupported
    );
    assert!(path::parse(&format!("/dev/g/{}", "x".repeat(257))).is_err());
    assert!(path::canonical("mapper", "lv").is_err());
    assert!(path::modeled_mapper("mapper", "lv").is_err());
    for (g, v) in [
        ("-g", "lv"),
        ("g", "-lv"),
        (".", "lv"),
        ("g", "snapshot"),
        ("g", "pvmove"),
        ("g", "data_mlog"),
    ] {
        assert!(path::canonical(g, v).is_err());
    }
}
#[test]
fn both_forms_resolve_the_same_real_virtual_volume_and_read_its_supplied_data() {
    let s = supplied();
    let before = s.clone();
    let (g, v, file, contents) = record(&s);
    let canonical = path::canonical(&g, &v).unwrap();
    let alias = path::modeled_mapper(&g, &v).unwrap();
    let a = path::resolve(&s, &canonical).unwrap();
    let b = path::resolve(&s, &alias).unwrap();
    assert!(std::ptr::eq(a.volume, b.volume));
    assert!(std::ptr::eq(a.group, b.group));
    assert_eq!(path::read(&s, &canonical, &file).unwrap(), contents);
    assert_eq!(path::read(&s, &alias, &file).unwrap(), contents);
    assert_eq!(s, before);
    assert!(path::resolve(&s, "/dev/not_supplied/no_volume").is_err());
    assert!(path::read(&s, &canonical, "/etc/passwd").is_err());
    assert!(path::read(&s, &canonical, "/../../etc/passwd").is_err());
    assert_eq!(s, before);
}
#[test]
fn actual_in_process_renames_invalidate_old_references_without_copying_original_storage() {
    let mut s = supplied();
    let (g, v, file, contents) = record(&s);
    let id = s.identity.as_ref().unwrap().volumes[&format!("{g}/{v}")];
    let volume = s.groups[&g].volumes[&v].clone();
    let old = path::modeled_mapper(&g, &v).unwrap();
    let old_canonical = path::canonical(&g, &v).unwrap();
    s.execute(&format!("vgrename {g} renamed-pool")).unwrap();
    s.execute(&format!("lvrename renamed-pool {v} renamed-report"))
        .unwrap();
    assert!(path::resolve(&s, &old).is_err());
    assert!(path::resolve(&s, &old_canonical).is_err());
    let current = path::canonical("renamed-pool", "renamed-report").unwrap();
    let alias = path::modeled_mapper("renamed-pool", "renamed-report").unwrap();
    assert_eq!(path::read(&s, &current, &file).unwrap(), contents);
    assert_eq!(path::read(&s, &alias, &file).unwrap(), contents);
    assert_eq!(s.groups["renamed-pool"].volumes["renamed-report"], volume);
    assert_eq!(
        s.identity.as_ref().unwrap().volumes["renamed-pool/renamed-report"],
        id
    );
}
#[test]
fn resolution_does_not_fake_application_availability_or_a_filesystem_for_raw_storage() {
    let mut s = supplied();
    let (g, v, file, _) = record(&s);
    let p = path::canonical(&g, &v).unwrap();
    // Supplied per-LV inactive fixture: a marked-open peer in this group
    // correctly prevents group deactivation. This is not a new lvchange command.
    s.activation
        .as_mut()
        .unwrap()
        .active
        .insert(format!("{g}/{v}"), false);
    s.validate().unwrap();
    assert!(path::resolve(&s, &p).is_ok());
    assert!(path::read(&s, &p, &file).is_err());
    let raw = s.groups.iter().find_map(|(g, group)| {
        group
            .volumes
            .iter()
            .find(|(_, v)| v.filesystem.is_none())
            .map(|(v, _)| path::canonical(g, v).unwrap())
    });
    if let Some(p) = raw {
        assert!(path::resolve(&s, &p).is_ok());
        assert!(path::read(&s, &p, &file).is_err());
    } else {
        let group = s.groups.keys().next().unwrap().clone();
        s.execute(&format!("vgchange -ay {group}")).unwrap();
        s.execute(&format!("lvcreate -l 1 -n rawpath {group}"))
            .unwrap();
        let p = path::canonical(&group, "rawpath").unwrap();
        assert!(path::resolve(&s, &p).is_ok());
        assert!(path::read(&s, &p, &file).is_err());
    }
}
#[test]
fn wire_commands_and_authoring_do_not_implicitly_gain_a_path_profile() {
    let s = supplied();
    let value = serde_json::to_value(&s).unwrap();
    assert!(value.get("device_paths").is_none());
    let mut injected = value;
    injected["device_paths"] = serde_json::json!({"callers":{}});
    assert!(lvm::State::from_json(&serde_json::to_vec(&injected).unwrap()).is_err());
    let mut current = s.clone();
    assert!(
        current
            .execute("dmsetup splitname data--pool-report--set")
            .is_err()
    );
    assert_eq!(current, s);
}
