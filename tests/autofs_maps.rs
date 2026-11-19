//! Pure local-file map parsing; never invokes AutoFS, a shell, files or devices.
use dlms_learning_lab::autofs::maps::{self, ErrorKind, Root, Style};
use std::collections::BTreeSet;
fn files() -> Vec<String> {
    vec![
        "/etc/auto.teams".into(),
        "/etc/auto.direct".into(),
        "/etc/auto.other".into(),
    ]
}
fn points() -> BTreeSet<String> {
    [
        "/srv/teams/report",
        "/srv/teams/audit",
        "/opt/reports",
        "/data/audit",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
fn root(direct: bool) -> Root {
    Root {
        point: if direct { "/-" } else { "/srv/teams" }.into(),
        file: if direct {
            "/etc/auto.direct"
        } else {
            "/etc/auto.teams"
        }
        .into(),
        timeout: 30,
    }
}
#[test]
fn master_modes_explicit_timeout_file_prefix_and_whitespace_parse_by_meaning() {
    let expected = vec![root(false)];
    for text in [
        "/srv/teams /etc/auto.teams --timeout=30\n",
        "# supplied map\n\n /srv/teams\tfile:/etc/auto.teams\t--timeout=00030\r\n",
    ] {
        assert_eq!(maps::master(text, &files()).unwrap(), expected);
    }
    let direct = maps::master("/- /etc/auto.direct --timeout=60", &files()).unwrap();
    assert_eq!(direct[0].style(), Style::Direct);
    assert_eq!(direct[0].timeout, 60);
    assert!(
        maps::master("# no managed directories yet\n", &files())
            .unwrap()
            .is_empty()
    );
}
#[test]
fn indirect_key_combines_with_master_directory_and_local_colon_is_removed() {
    let rows = maps::entries(
        "report -fstype=ext4,rw :/dev/ledger/reports\naudit -ro,fstype=ext4 :/dev/ledger/audit\n",
        &root(false),
        &points(),
    )
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].target, "/srv/teams/report");
    assert_eq!(rows[0].key, "report");
    assert_eq!(rows[0].device, "/dev/ledger/reports");
    assert!(!rows[0].read_only);
    assert!(rows[1].read_only);
}
#[test]
fn direct_keys_are_full_paths_without_adding_the_master_marker() {
    let rows=maps::entries("/opt/reports -fstype=ext4 :/dev/ledger/reports\n/data/audit -fstype=ext4,ro :/dev/ledger/audit\n",&root(true),&points()).unwrap();
    assert_eq!(rows[0].target, "/opt/reports");
    assert_eq!(rows[1].target, "/data/audit");
    assert_eq!(rows[0].key, rows[0].target);
}
#[test]
fn equivalent_supported_option_orders_preserve_access_and_filesystem_source() {
    let mut results = Vec::new();
    for opt in [
        "-fstype=ext4,ro",
        "-ro,fstype=ext4",
        "-defaults,ro,fstype=ext4",
    ] {
        results.push(
            maps::entries(
                &format!("report {opt} :/dev/ledger/reports"),
                &root(false),
                &points(),
            )
            .unwrap(),
        );
    }
    assert!(results.windows(2).all(|v| v[0] == v[1]));
    assert!(
        !maps::entries(
            "report -defaults,fstype=ext4 :/dev/ledger/reports",
            &root(false),
            &points()
        )
        .unwrap()[0]
            .read_only
    );
}
#[test]
fn mode_confusion_and_missing_colon_report_specific_mistakes() {
    let e = maps::entries(
        "/srv/teams/report -fstype=ext4 :/dev/ledger/reports",
        &root(false),
        &points(),
    )
    .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Syntax);
    assert!(e.message.contains("one directory name"));
    let e = maps::entries(
        "report -fstype=ext4 :/dev/ledger/reports",
        &root(true),
        &points(),
    )
    .unwrap_err();
    assert!(e.message.contains("full absolute mountpoint"));
    let e = maps::entries(
        "report -fstype=ext4 /dev/ledger/reports",
        &root(false),
        &points(),
    )
    .unwrap_err();
    assert!(e.message.contains("colon"));
    let e = maps::entries(
        "report fstype=ext4 :/dev/ledger/reports",
        &root(false),
        &points(),
    )
    .unwrap_err();
    assert!(e.message.contains("leading dash"));
}
#[test]
fn unknown_sources_are_not_faked_as_existing_mounted_filesystems() {
    let rows = maps::entries(
        "report -fstype=ext4 :/dev/unknown/volume",
        &root(false),
        &points(),
    )
    .unwrap();
    assert_eq!(rows[0].device, "/dev/unknown/volume");
    // Parsing only: existence/availability/readability belongs to the next pure
    // model, not a command-to-success assertion or host /dev lookup.
    let e = maps::entries(
        "wrong -fstype=ext4 :/dev/ledger/reports",
        &root(false),
        &points(),
    )
    .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert!(e.message.contains("master directory and key"));
}
#[test]
fn programs_network_maps_includes_substitution_and_multimounts_are_explicitly_unsupported() {
    for text in [
        "+auto.master",
        "/srv/teams program:/etc/auto.teams --timeout=30",
        "/srv/teams ldap:example --timeout=30",
        "/srv/teams auto.teams --timeout=30",
        "/srv/teams /etc/auto.teams --timeout=30 -browse",
    ] {
        assert_eq!(
            maps::master(text, &files()).unwrap_err().kind,
            ErrorKind::Unsupported,
            "{text}"
        );
    }
    for text in [
        "* -fstype=ext4 :/dev/ledger/reports",
        "report -fstype=ext4 :/dev/$USER/reports",
        "report -fstype=ext4 :/dev/ledger/&",
        "report -fstype=nfs host:/export",
        "report host:/export",
        "report -fstype=ext4 :/dev/a :/dev/b",
        "report -fstype=ext4 :/dev/ledger/reports # note",
        "report -fstype=ext4 :/dev/ledger/reports;id",
        "report -fstype=ext4 :/dev/$(id)/reports",
    ] {
        assert_eq!(
            maps::entries(text, &root(false), &points())
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported,
            "{text}"
        );
    }
}
#[test]
fn duplicate_and_nested_ambiguity_is_rejected_with_an_honest_subset_diagnostic() {
    let e = maps::master(
        "/srv/teams /etc/auto.teams --timeout=30\n/srv/teams /etc/auto.other --timeout=30",
        &files(),
    )
    .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert!(e.message.contains("Real AutoFS ignores later"));
    for text in [
        "/srv/teams /etc/auto.teams --timeout=30\n/srv/teams/sub /etc/auto.other --timeout=30",
        "/srv/teams /etc/auto.teams --timeout=30\n/- /etc/auto.teams --timeout=30",
    ] {
        assert_eq!(
            maps::master(text, &files()).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    let e = maps::entries(
        "report -fstype=ext4 :/dev/a\nreport -fstype=ext4 :/dev/b",
        &root(false),
        &points(),
    )
    .unwrap_err();
    assert_eq!(e.line, Some(2));
    assert_eq!(e.kind, ErrorKind::Unsupported);
    assert_eq!(
        maps::master(
            "/- /etc/auto.direct --timeout=30\n/- /etc/auto.other --timeout=30",
            &files()
        )
        .unwrap()
        .len(),
        2
    );
}
#[test]
fn no_implicit_daemon_or_nfs_defaults_are_guessed() {
    for text in [
        "/srv/teams /etc/auto.teams",
        "/srv/teams /etc/auto.teams --timeout=0",
        "/srv/teams /etc/auto.teams --timeout=3601",
        "/srv/teams /etc/auto.teams --timeout=-1",
    ] {
        assert!(maps::master(text, &files()).is_err(), "{text}");
    }
    for opt in [
        "-ro",
        "-fstype=xfs",
        "-fstype=ext4,rw,ro",
        "-fstype=ext4,fstype=ext4",
        "-fstype=ext4,unknown",
    ] {
        assert_eq!(
            maps::entries(
                &format!("report {opt} :/dev/ledger/reports"),
                &root(false),
                &points()
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported
        );
    }
}
#[test]
fn bounds_and_line_numbers_are_checked_before_derived_allocations() {
    let f = files();
    assert_eq!(
        maps::master(&"x".repeat(4097), &f).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        maps::master(&"\n".repeat(65), &f).unwrap_err().kind,
        ErrorKind::Limit
    );
    assert_eq!(
        maps::master(
            "# comment\n\n/srv/teams /etc/auto.teams --timeout=wrong",
            &f
        )
        .unwrap_err()
        .line,
        Some(3)
    );
    for value in ["\0", "café", "\u{1b}"] {
        assert_eq!(
            maps::master(value, &f).unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    assert_eq!(
        maps::master("", &vec!["/etc/auto.one".into(); 4])
            .unwrap_err()
            .kind,
        ErrorKind::Context
    );
    let mut many = BTreeSet::new();
    for n in 0..5 {
        many.insert(format!("/point{n}"));
    }
    assert_eq!(
        maps::entries("", &root(true), &many).unwrap_err().kind,
        ErrorKind::Context
    );
    let p = (0..4)
        .map(|n| format!("/point{n}"))
        .collect::<BTreeSet<_>>();
    let mut map = String::new();
    for n in 0..9 {
        map += &format!("/point{} -fstype=ext4 :/dev/item{n}\n", n % 4);
    }
    // Duplicate keys fail before the entry limit; neither is silently accepted.
    assert!(maps::entries(&map, &root(true), &p).is_err());
}
#[test]
fn traversal_control_characters_unknown_files_and_reserved_keys_never_open_paths() {
    for name in [
        "/etc/../auto.teams",
        "/etc//auto.teams",
        "/etc/auto.master",
        "relative",
        "/etc/auto.teams/",
    ] {
        assert_eq!(
            maps::master("", &[name.into()]).unwrap_err().kind,
            ErrorKind::Context
        );
    }
    for key in ["..", ".", "../report", "team/report"] {
        assert!(
            maps::entries(
                &format!("{key} -fstype=ext4 :/dev/ledger/reports"),
                &root(false),
                &points()
            )
            .is_err()
        );
    }
    for src in [
        ":/dev/../etc/passwd",
        ":/dev//a",
        ":/etc/passwd",
        ":UUID=123",
        ":/dev/a/",
    ] {
        assert!(
            maps::entries(
                &format!("report -fstype=ext4 {src}"),
                &root(false),
                &points()
            )
            .is_err()
        );
    }
}
