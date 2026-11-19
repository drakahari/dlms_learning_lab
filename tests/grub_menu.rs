use dlms_learning_lab::{
    boot_entry::{self as b, ErrorKind},
    grub_menu::{self as g, Menu},
};
const UUID: &str = "12345678-9abc-def0-1234-56789abcdef0";
fn inventory() -> b::Inventory {
    b::Inventory {
        kernels: vec![b::Kernel {
            path: "/images/7.0/linux".into(),
            release: "7.0".into(),
            builtin_drivers: vec!["ext4".into()],
        }],
        initramfs: vec![b::Initramfs {
            path: "/images/7.0/initrd".into(),
            release: "7.0".into(),
            drivers: vec!["virtio_blk".into()],
            uuid_lookup: true,
        }],
        roots: vec![b::Root {
            uuid: UUID.into(),
            label: "system".into(),
            required_drivers: vec!["virtio_blk".into(), "ext4".into()],
            init_present: true,
        }],
    }
}
fn item(title: &str, id: &str, mode: &str, target: &str) -> String {
    format!(
        "menuentry '{title}' --id={id} {{\n linux /images/7.0/linux root=UUID={UUID} {mode} systemd.unit={target}\n initrd /images/7.0/initrd\n}}\n"
    )
}
fn menu(selection: &str) -> String {
    format!(
        "set default={selection}\n{}{}",
        item("Ordinary Linux", "normal", "ro", "multi-user.target"),
        item("Recovery Linux", "recovery", "ro", "rescue.target")
    )
}
#[test]
fn default_selects_actual_entry_by_position_id_or_quoted_title() {
    for (selection, id, target) in [
        ("0", "normal", "multi-user.target"),
        ("normal", "normal", "multi-user.target"),
        ("'Ordinary Linux'", "normal", "multi-user.target"),
        ("1", "recovery", "rescue.target"),
        ("recovery", "recovery", "rescue.target"),
        ("\"Recovery Linux\"", "recovery", "rescue.target"),
    ] {
        let src = menu(selection);
        let m = Menu::parse(&src).unwrap();
        assert_eq!(m.selected().unwrap().id, id);
        let r = g::test(&inventory(), &src).unwrap();
        assert!(r.passed());
        assert_eq!(r.target.as_deref(), Some(target));
        assert_eq!(r.stages[0].title, "Menu selection");
    }
}
#[test]
fn quotes_comments_and_joins_are_literal_not_shell_expansion() {
    let source = menu("'Ordinary Linux'")
        .replace("linux /images", "lin\\\nux /images")
        .replace(" root=UUID=", " \\\n root=UUID=")
        .replace("--id=normal", "--id \"normal\"");
    let m = Menu::parse(&("# a harmless comment\n".to_owned() + &source)).unwrap();
    assert_eq!(m.selected().unwrap().id, "normal");
    assert!(g::test(&inventory(), &source).unwrap().passed());
    let literal = menu("normal").replace("'Ordinary Linux'", "'Ordinary $Linux'");
    assert_eq!(
        Menu::parse(&literal).unwrap().items[0].title,
        "Ordinary $Linux"
    );
    for bad in [
        menu("$saved_entry"),
        menu("\"$saved_entry\""),
        menu("normal").replace("linux /images", "echo substituted\n linux /images"),
        menu("normal") + "source /etc/passwd\n",
        menu("normal") + "set root=hd0\n",
    ] {
        assert_eq!(Menu::parse(&bad).unwrap_err().kind, ErrorKind::Unsupported);
    }
    let unsupported_root = Menu::parse(&(menu("normal") + "set root=hd0\n")).unwrap_err();
    assert!(unsupported_root.message.contains("file-reading root"));
}
#[test]
fn missing_selections_and_ambiguous_menus_are_honest_unsupported_not_success() {
    for src in [
        menu("9"),
        menu("absent"),
        menu("saved"),
        menu("normal").replace("--id=recovery", "--id=normal"),
        menu("normal").replace("'Recovery Linux'", "'Ordinary Linux'"),
        menu("normal").replace("'Recovery Linux'", "'normal'"),
    ] {
        assert_eq!(Menu::parse(&src).unwrap_err().kind, ErrorKind::Unsupported);
    }
}
#[test]
fn syntax_order_limits_and_traversal_do_not_become_success() {
    let good = menu("normal");
    for src in [
        good.trim_end_matches(['\n', '}']).into(),
        good.replace(
            "linux /images/7.0/linux",
            "initrd /images/7.0/initrd\n linux /images/7.0/linux",
        ),
        good.replace("root=UUID=", "root=UUID=broken"),
        good.replace("'Ordinary Linux'", "'Unclosed Linux"),
    ] {
        assert!(Menu::parse(&src).is_err());
    }
    for src in [
        "#".repeat(4097),
        "\n".repeat(49),
        good.replace("/images/7.0/linux", "/images/../secret"),
        good.replace(
            "initrd /images/7.0/initrd",
            "initrd /images/7.0/initrd\n initrd /images/7.0/initrd",
        ),
        good.replace("--id=normal {", "--id=normal{"),
        good.replace("ro systemd.unit", "ro; systemd.unit"),
    ] {
        assert!(Menu::parse(&src).is_err());
    }
    assert!(
        Menu::parse(
            &(good.clone()
                + &item("Third", "third", "ro", "multi-user.target")
                + &item("Fourth", "fourth", "ro", "multi-user.target")
                + &item("Fifth", "fifth", "ro", "multi-user.target"))
        )
        .is_err()
    );
    assert!(Menu::parse(&(good + "\0")).is_err());
}
#[test]
fn bootable_wrong_target_is_distinct_from_broken_image_chain() {
    let i = inventory();
    let normal = g::test(&i, &menu("normal")).unwrap();
    let recovery = g::test(&i, &menu("recovery")).unwrap();
    assert!(normal.passed() && recovery.passed());
    assert_ne!(normal.target, recovery.target);
    let broken = menu("normal").replace("linux /images/7.0/linux", "linux /missing/linux");
    let r = g::test(&i, &broken).unwrap();
    assert!(!r.passed());
    assert!(r.stages[0].passed);
    assert!(!r.stages[1].passed);
    assert!(r.root.is_none());
}
#[test]
fn preservation_compares_parsed_configuration_accepting_equivalent_quotes_and_option_order() {
    let original = Menu::parse(&menu("normal")).unwrap();
    let alternate = menu("recovery")
        .replace("--id=normal", "--id 'normal'")
        .replace(
            "ro systemd.unit=multi-user.target",
            "systemd.unit=multi-user.target ro",
        );
    let changed = Menu::parse(&alternate).unwrap();
    assert!(changed.equivalent_item(&original, "normal"));
    let changed = Menu::parse(&alternate.replace(
        "systemd.unit=multi-user.target ro",
        "systemd.unit=multi-user.target rw",
    ))
    .unwrap();
    assert!(!changed.equivalent_item(&original, "normal"));
    let changed = Menu::parse(&alternate.replace("'Ordinary Linux'", "'Misleading name'")).unwrap();
    assert!(!changed.equivalent_item(&original, "normal"));
}
