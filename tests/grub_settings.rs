//! Literal persistent configuration and supplied-template generation tests.
use dlms_learning_lab::{
    boot_entry::ErrorKind,
    grub_settings::{self, Defaults},
};
const ROOT: &str = "31415926-5358-9793-2384-626433832795";
fn templates(root: &str) -> String {
    format!(
        "set default=main\nmenuentry 'Normal Linux' --id=main {{\nlinux /vmlinuz-7.0 root=UUID={root} ro\ninitrd /initramfs-7.0.img\n}}\nmenuentry 'Recovery Linux' --id=recovery {{\nlinux /vmlinuz-7.0 root=UUID={root} ro systemd.unit=rescue.target\ninitrd /initramfs-7.0.img\n}}\n"
    )
}
fn generate(s: &str) -> grub_settings::Generated {
    grub_settings::generate(s, &templates(ROOT), &["main".into()]).unwrap()
}
#[test]
fn settings_literals_defaults_quotes_comments_and_later_assignments_are_explicit() {
    assert_eq!(
        Defaults::parse("# empty defaults\n").unwrap(),
        Defaults::default()
    );
    let s=Defaults::parse("GRUB_DEFAULT=main\nGRUB_DEFAULT='Recovery 'Linux # later value\nGRUB_TIMEOUT=0\nGRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"\n").unwrap();
    assert_eq!(s.selection, "Recovery Linux");
    assert_eq!(s.timeout, 0);
    assert_eq!(s.normal, ["quiet"]);
    assert_eq!(
        Defaults::parse("GRUB_DEFAULT='~'\n").unwrap().selection,
        "~"
    );
    assert_eq!(
        Defaults::parse("GRUB_CMDLINE_LINUX_DEFAULT=quiet#literal\n")
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        Defaults::parse("GRUB_CMDLINE_LINUX_DEFAULT='qu'\"iet\"\n")
            .unwrap()
            .normal,
        ["quiet"]
    );
    for n in [-1, 0, 1, 5, 300] {
        assert_eq!(generate(&format!("GRUB_TIMEOUT={n}\n")).timeout, n);
    }
}
#[test]
fn common_and_normal_only_options_affect_actual_distinct_generated_entries() {
    let both = generate("GRUB_CMDLINE_LINUX=quiet\n");
    assert!(
        both.menu
            .items
            .iter()
            .all(|i| i.entry.options.iter().any(|o| o == "quiet"))
    );
    let normal = generate("GRUB_CMDLINE_LINUX_DEFAULT=quiet\n");
    assert!(
        normal.menu.items[0]
            .entry
            .options
            .iter()
            .any(|o| o == "quiet")
    );
    assert!(
        !normal.menu.items[1]
            .entry
            .options
            .iter()
            .any(|o| o == "quiet")
    );
    let rw = generate("GRUB_CMDLINE_LINUX=rw\nGRUB_CMDLINE_LINUX_DEFAULT=ro\n");
    assert!(
        dlms_learning_lab::boot_entry::Options::parse(&rw.menu.items[0].entry.options)
            .unwrap()
            .read_only
    );
    assert!(
        !dlms_learning_lab::boot_entry::Options::parse(&rw.menu.items[1].entry.options)
            .unwrap()
            .read_only
    );
    assert!(normal.text.contains("set timeout=5\n"));
    assert!(!normal.menu_source.contains("set timeout"));
}
#[test]
fn generation_uses_current_supplied_facts_not_task_answer_strings() {
    for (selector, index) in [
        ("0", 0),
        ("main", 0),
        ("1", 1),
        ("recovery", 1),
        ("'Recovery Linux'", 1),
    ] {
        let g = generate(&format!("GRUB_DEFAULT={selector}\n"));
        assert_eq!(g.menu.selected().unwrap().id, g.menu.items[index].id);
    }
    let other = "11223344-5566-7788-99aa-bbccddeeff00";
    let g =
        grub_settings::generate("GRUB_DEFAULT=1\n", &templates(other), &["main".into()]).unwrap();
    assert!(g.menu_source.contains(other));
    assert!(!g.menu_source.contains(ROOT));
    assert!(
        grub_settings::generate("GRUB_DEFAULT=missing\n", &templates(ROOT), &["main".into()])
            .is_err()
    );
    for normal in [
        vec![],
        vec!["absent".into()],
        vec!["main".into(), "main".into()],
    ] {
        assert!(grub_settings::generate("", &templates(ROOT), &normal).is_err());
    }
}
#[test]
fn unsupported_shell_paths_keys_and_ambiguous_kernel_options_fail_honestly() {
    for source in [
        ". /etc/passwd",
        "source /etc/passwd",
        "export GRUB_DEFAULT=0",
        "GRUB_DEFAULT=$(id)",
        "GRUB_DEFAULT=~",
        "GRUB_DEFAULT=\"$HOME\"",
        "GRUB_DEFAULT=`id`",
        "GRUB_DEFAULT=0; id",
        "GRUB_DEFAULT=0 > /etc/passwd",
        "GRUB_DEFAULT=saved",
        "GRUB_TIMEOUT_STYLE=hidden",
        "GRUB_CMDLINE_LINUX=root=UUID=bad",
        "GRUB_CMDLINE_LINUX=\"quiet splash\"",
        "GRUB_DEFAULT=0 unexpected",
        "GRUB_DEFAULT =0",
        "GRUB_DEFAULT=0\\\nGRUB_TIMEOUT=5",
    ] {
        assert!(Defaults::parse(source).is_err(), "{source:?}");
    }
    assert!(
        grub_settings::generate(
            "GRUB_CMDLINE_LINUX=systemd.unit=multi-user.target\n",
            &templates(ROOT),
            &["main".into()]
        )
        .is_err()
    );
    assert!(Defaults::parse("GRUB_DEFAULT='unfinished").is_err());
}
#[test]
fn input_output_and_numeric_resources_remain_bounded() {
    for s in [
        "GRUB_TIMEOUT=-2",
        "GRUB_TIMEOUT=301",
        "GRUB_TIMEOUT=5.5",
        "GRUB_TIMEOUT=999999999999999999999999",
    ] {
        assert!(Defaults::parse(s).is_err());
    }
    assert_eq!(
        Defaults::parse(&"#\n".repeat(49)).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        Defaults::parse(&format!("#{}", "x".repeat(4096)))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert!(Defaults::parse("GRUB_TIMEOUT=5\r\n").is_err());
    assert!(Defaults::parse("GRUB_DEFAULT=0\0").is_err());
    assert_eq!(
        Defaults::parse(&format!("GRUB_CMDLINE_LINUX='{}'", "quiet ".repeat(17)))
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
}
