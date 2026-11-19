use dlms_learning_lab::{
    boot_entry as boot, initramfs as ram, kernel_build as kernel, kernel_stage::Store,
};
fn store() -> Store {
    let mut store = Store::default();
    for (id, release, entry) in [
        ("old-source", "6.6.60", "fallback.conf"),
        ("new-source", "6.6.61", "candidate.conf"),
    ] {
        let config = kernel::Config {
            modules: true,
            initrd: true,
            nvme: kernel::Feature::Module,
            ext4: kernel::Feature::Module,
            localversion: String::new(),
        };
        let mut a = kernel::Artifacts::default();
        a.build(id, release, &config, kernel::Target::All).unwrap();
        let path = store.image("/", a.image.as_ref().unwrap()).unwrap();
        store.modules("/", a.modules.as_ref().unwrap()).unwrap();
        store.assemble_initramfs("/", &path).unwrap();
        store.entry("/",entry,&format!("title Supplied candidate\nlinux /vmlinuz-{release}\ninitrd /initramfs-{release}.img\noptions root=UUID=11111111-2222-3333-4444-555555555555 ro\n")).unwrap();
    }
    store
}
fn root() -> boot::Root {
    boot::Root {
        uuid: "11111111-2222-3333-4444-555555555555".into(),
        label: "Supplied ext4 root".into(),
        required_drivers: ["nvme_core", "nvme", "jbd2", "mbcache", "ext4"]
            .map(str::to_owned)
            .to_vec(),
        init_present: true,
    }
}
#[test]
fn literal_config_assignment_append_and_comments_have_real_string_semantics() {
    assert_eq!(
        ram::Config::parse("# Supplied generic base\n").unwrap(),
        ram::Config::default()
    );
    let a = ram::Config::parse(
        "omit_drivers=\" nvme \"\nomit_drivers+=\" ext4 \" # append with spaces\n",
    )
    .unwrap();
    let b = ram::Config::parse("omit_drivers='nvme ext4'\n").unwrap();
    assert_eq!(a, b);
    assert_eq!(a.omitted.len(), 2);
    assert!(
        ram::Config::parse("omit_drivers=nvme\nomit_drivers=\"\"\n")
            .unwrap()
            .omitted
            .is_empty()
    );
    assert_eq!(
        ram::Config::parse("omit_drivers=nv\nomit_drivers+=me\n")
            .unwrap()
            .omitted,
        ["nvme".into()].into_iter().collect()
    );
    let e = ram::Config::parse("omit_drivers=nvme\nomit_drivers+=ext4\n").unwrap_err();
    assert!(e.contains("nvmeext4") && e.contains("spaces"));
    assert!(
        ram::Config::parse("omit_drivers= # empty\n")
            .unwrap()
            .omitted
            .is_empty()
    );
}
#[test]
fn unsupported_configuration_never_runs_or_silently_ignores_a_setting() {
    for source in [
        "hostonly=yes",
        "omit_drivers=$(id)",
        "omit_drivers=\"$HOME\"",
        "omit_drivers=\"nvme\"; touch /host",
        "omit_drivers=\"nvme\"#not-a-separated-comment",
        "omit_drivers =nvme",
        "omit_drivers='nvme",
        "omit_drivers=nvme ext4",
        "omit_drivers=\"nvme\\next4\"",
        "omit_drivers=\"nvme\"\r\n",
        "omit_drivers=unknown",
        "rm -rf /",
        "omit_drivers+=\"nvmé\"",
    ] {
        assert!(ram::Config::parse(source).is_err(), "{source}");
    }
    assert!(ram::Config::parse(&"#".repeat(4097)).is_err());
    assert!(ram::Config::parse(&"#\n".repeat(33)).is_err());
}
#[test]
fn supported_positional_and_flag_forms_produce_identical_typed_commands() {
    assert_eq!(
        ram::parse("dracut --kver 6.6.61 /boot/training.img -f").unwrap(),
        ram::parse("dracut --force /boot/training.img 6.6.61").unwrap()
    );
    assert_eq!(
        ram::parse("lsinitrd /boot/training.img").unwrap(),
        ram::Command::List {
            path: "/boot/training.img".into()
        }
    );
    for command in ["dracut --help", "lsinitrd -h"] {
        assert_eq!(ram::parse(command).unwrap(), ram::Command::Help);
    }
    assert!(
        ram::parse("dracut -k 6.6.61 /boot/training.img")
            .unwrap_err()
            .contains("unlike dkms")
    );
    assert!(
        ram::parse("lsinitrd -m /boot/training.img")
            .unwrap_err()
            .contains("feature modules")
    );
}
#[test]
fn generation_respects_existing_output_and_force_and_inspection_is_read_only() {
    let mut s = store();
    let original = s.clone();
    let c = ram::Config::default();
    assert!(
        ram::execute(
            "/",
            &c,
            &mut s,
            "dracut --kver 6.6.61 /boot/initramfs-6.6.61.img"
        )
        .unwrap_err()
        .contains("already exists")
    );
    assert_eq!(s, original);
    ram::execute("/", &c, &mut s, "dracut --kver 6.6.61 /boot/other.img").unwrap();
    assert_eq!(s.initramfs.len(), 3);
    let before = s.clone();
    let text = ram::execute("/", &c, &mut s, "lsinitrd /boot/other.img").unwrap();
    assert!(text.contains("Kernel: 6.6.61"));
    assert!(text.contains("nvme-core.ko") && text.contains("fs/ext4/ext4.ko"));
    assert!(text.contains("not full archive metadata"));
    assert_eq!(s, before);
    ram::execute(
        "/",
        &c,
        &mut s,
        "dracut -f /boot/initramfs-6.6.61.img 6.6.61",
    )
    .unwrap();
    assert_eq!(s.initramfs[1], original.initramfs[1]);
}
#[test]
fn every_driver_omission_combination_changes_actual_boot_consequences() {
    let names = ["nvme_core", "nvme", "ext4", "jbd2", "mbcache"];
    for mask in 0..32 {
        let mut s = store();
        let original = s.clone();
        let c = ram::Config {
            omitted: names
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, n)| (*n).into())
                .collect(),
        };
        let output = ram::execute(
            "/",
            &c,
            &mut s,
            "dracut --force --kver 6.6.61 /boot/initramfs-6.6.61.img",
        )
        .unwrap();
        assert!(output.contains("generation alone does not prove"));
        assert_eq!(s.initramfs[1].image.included.len(), 5 - c.omitted.len());
        assert_eq!(
            s.test_entry("/", "/boot/loader/entries/candidate.conf", root())
                .unwrap()
                .passed(),
            mask == 0,
            "mask {mask}"
        );
        assert_eq!(s.initramfs[0], original.initramfs[0]);
        assert!(
            s.test_entry("/", "/boot/loader/entries/fallback.conf", root())
                .unwrap()
                .passed()
        );
        ram::execute(
            "/",
            &ram::Config::default(),
            &mut s,
            "dracut -f --kver 6.6.61 /boot/initramfs-6.6.61.img",
        )
        .unwrap();
        assert!(
            s.test_entry("/", "/boot/loader/entries/candidate.conf", root())
                .unwrap()
                .passed()
        );
    }
}
#[test]
fn wrong_kernel_can_generate_a_valid_file_that_the_selected_entry_cannot_use() {
    let mut s = store();
    ram::execute(
        "/",
        &ram::Config::default(),
        &mut s,
        "dracut -f --kver 6.6.60 /boot/initramfs-6.6.61.img",
    )
    .unwrap();
    assert_eq!(s.initramfs[1].image.image_build.release, "6.6.60");
    assert!(
        s.test_entry("/", "/boot/loader/entries/candidate.conf", root())
            .is_err()
    );
    ram::execute(
        "/",
        &ram::Config::default(),
        &mut s,
        "dracut -f --kver 6.6.61 /boot/initramfs-6.6.61.img",
    )
    .unwrap();
    assert!(
        s.test_entry("/", "/boot/loader/entries/candidate.conf", root())
            .unwrap()
            .passed()
    );
}
#[test]
fn unsupported_defaults_escapes_and_paths_never_mutate_virtual_state() {
    let mut s = store();
    let original = s.clone();
    for command in [
        "dracut",
        "dracut --force",
        "dracut /boot/image.img",
        "dracut --kver 6.6.61 /boot/../host.img",
        "dracut --kver 6.6.61 /boot/subdir/image.img",
        "dracut --kver 6.6.61 /home/host.img",
        "dracut --kver 6.6.61 /boot/vmlinuz-6.6.61 -f",
        "dracut --kver 6.6.61 /boot/a.img; sh",
        "dracut --kver $(id) /boot/a.img",
        "dracut --kver 6.6.61 '/boot/a.img'",
        "dracut --kver 6.6.61 --kver 6.6.60 /boot/a.img",
        "dracut --hostonly --kver 6.6.61 /boot/a.img",
        "lsinitrd --unpack /boot/initramfs-6.6.61.img",
        "lsinitrd /etc/shadow",
        "lsinitrd\n/boot/initramfs-6.6.61.img",
        "sh script.sh",
    ] {
        assert!(
            ram::execute("/", &ram::Config::default(), &mut s, command).is_err(),
            "{command}"
        );
        assert_eq!(s, original, "{command}");
    }
    assert!(ram::parse(&"x".repeat(2049)).is_err());
}
#[test]
fn missing_or_ambiguous_supplied_inputs_and_invalid_imports_are_honest() {
    let mut s = store();
    s.modules.retain(|m| !m.path.ends_with("6.6.61"));
    let before = s.clone();
    assert!(
        ram::execute(
            "/",
            &ram::Config::default(),
            &mut s,
            "dracut --kver 6.6.61 /boot/a.img"
        )
        .unwrap_err()
        .contains("subset requires")
    );
    assert_eq!(s, before);
    let mut s = store();
    let mut duplicate = s.images[1].clone();
    duplicate.path = "/boot/duplicate".into();
    s.images.push(duplicate);
    let before = s.clone();
    assert!(
        ram::execute(
            "/",
            &ram::Config::default(),
            &mut s,
            "dracut --kver 6.6.61 /boot/a.img"
        )
        .unwrap_err()
        .contains("uniquely")
    );
    assert_eq!(s, before);
    for text in [
        r#"{"omitted":["unknown"]}"#,
        r#"{"omitted":["nvme","nvme"]}"#,
        r#"{"omitted":[],"host":"/home"}"#,
    ] {
        assert!(serde_json::from_str::<ram::Config>(text).is_err());
    }
    let too_many = serde_json::json!({"omitted":vec!["nvme";6]});
    assert!(serde_json::from_value::<ram::Config>(too_many).is_err());
    let c = ram::Config::parse("omit_drivers='nvme ext4'").unwrap();
    let copy: ram::Config = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(c, copy);
}
