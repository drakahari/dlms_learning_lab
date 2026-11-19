use dlms_learning_lab::{
    engine::Node,
    file_backup::{Entry, Tree},
    source_install::{self as model, Configuration, Facts, File, Program, Target, TextFile, World},
};
use std::collections::BTreeMap;

fn facts() -> Facts {
    let source = Tree {
        clock: 1,
        entries: [
            ("", None),
            (
                "INSTALL",
                Some(
                    "Configure a prefix, build/check, then install with an optional staging root.",
                ),
            ),
            (
                "configure",
                Some("Supplied simulated configure entry point; no shell runs."),
            ),
            (
                "Makefile.in",
                Some("Supplied fixed graph: check -> all; install -> all."),
            ),
            (
                "source.txt",
                Some("Supplied greeting-program source identity; no general compiler emulation."),
            ),
        ]
        .into_iter()
        .map(|(path, text)| {
            (
                path.into(),
                Entry {
                    node: Node {
                        directory: text.is_none(),
                        mode: if text.is_none() || path == "configure" {
                            0o755
                        } else {
                            0o644
                        },
                        owner: "builder".into(),
                        group: "builder".into(),
                    },
                    text: text.map(str::to_string),
                    modified: 1,
                    changed: 1,
                },
            )
        })
        .collect::<BTreeMap<_, _>>(),
    };
    let mut source = source;
    source.entries.insert(
        "README".into(),
        Entry {
            node: Node {
                directory: false,
                mode: 0o644,
                owner: "builder".into(),
                group: "builder".into(),
            },
            text: Some(
                "Supplied greeting program: invoke with NAME, --version or --prefix.\n".into(),
            ),
            modified: 1,
            changed: 1,
        },
    );
    Facts {
        version: 1,
        archive: "/sources/greet-1.4.tar".into(),
        archive_root: "greet-1.4".into(),
        source,
        program: Program {
            name: "greet".into(),
            version: "1.4".into(),
            default_greeting: "Hello,".into(),
        },
        default_prefix: "/usr/local".into(),
        manual: "Supplied greeting program: invoke with NAME, --version or --prefix.\n".into(),
        live_files: vec![TextFile {
            path: "/usr/local/bin/greet".into(),
            text: "Keep this installed legacy package unchanged.\n".into(),
            mode: 0o755,
        }],
    }
}
fn config(prefix: &str, greeting: &str, uppercase: bool) -> Configuration {
    Configuration {
        prefix: prefix.into(),
        greeting: greeting.into(),
        uppercase,
    }
}
fn configured(f: &Facts) -> World {
    let mut w = World::new(f).unwrap();
    w.extract(f, "/usr/src").unwrap();
    w.configure(f, config("/usr/local", "Welcome,", false))
        .unwrap();
    w
}
#[test]
fn supplied_source_must_be_unpacked_and_configured_before_a_real_artifact_exists() {
    let f = facts();
    f.validate().unwrap();
    let mut w = World::new(&f).unwrap();
    assert!(w.make(&f, Target::All, None).is_err());
    assert!(
        w.configure(&f, config("/opt/team", "Hello,", false))
            .is_err()
    );
    w.extract(&f, "/usr/src").unwrap();
    assert!(w.files.contains_key("/usr/src/greet-1.4/INSTALL"));
    assert!(w.make(&f, Target::All, None).is_err());
    w.configure(&f, config("/opt/team", "Hello,", false))
        .unwrap();
    assert!(w.files.contains_key("/usr/src/greet-1.4/Makefile"));
    assert!(!w.is_build_current(&f));
    w.make(&f, Target::All, None).unwrap();
    assert!(w.is_build_current(&f));
    assert!(!w.is_check_current(&f));
    let p = w.binary_member(&f).unwrap();
    for (arg, expected) in [
        ("Ada", "Hello, Ada\n"),
        ("Rio", "Hello, Rio\n"),
        ("--version", "greet 1.4\n"),
        ("--prefix", "/opt/team\n"),
    ] {
        let out = w.run(&p, &[arg.into()]).unwrap();
        assert_eq!((out.stdout.as_str(), out.status), (expected, 0));
        assert!(out.stderr.is_empty());
    }
    let bad = w.run(&p, &["--unsupported".into()]).unwrap();
    assert_eq!(bad.status, 2);
    assert!(bad.stdout.is_empty());
    assert!(bad.stderr.starts_with("Usage:"));
}
#[test]
fn check_and_install_follow_the_supplied_dependency_instead_of_requiring_a_ritual_command_order() {
    let f = facts();
    let mut w = configured(&f);
    let out = w.make(&f, Target::Check, None).unwrap();
    assert!(out.stdout.contains("2 cases"));
    assert!(w.is_check_current(&f));
    w.make(&f, Target::Clean, None).unwrap();
    assert!(!w.is_build_current(&f));
    assert!(!w.is_check_current(&f));
    let install = w.make(&f, Target::Install, Some("/stage/review")).unwrap();
    assert!(install.stdout.contains("Building greet"));
    assert!(w.is_build_current(&f));
    assert!(!w.is_check_current(&f));
    let staged = "/stage/review/usr/local/bin/greet";
    assert_eq!(
        w.run(staged, &["Rio".into()]).unwrap().stdout,
        "Welcome, Rio\n"
    );
    assert_eq!(
        w.run(staged, &["--prefix".into()]).unwrap().stdout,
        "/usr/local\n"
    );
    assert_eq!(w.files[staged].mode(), 0o755);
    assert_eq!(
        w.files["/stage/review/usr/local/share/doc/greet/README"].mode(),
        0o644
    );
    assert!(
        matches!(&w.files["/usr/local/bin/greet"], File::Text {text,..} if text.starts_with("Keep"))
    );
}
#[test]
fn reconfiguration_cannot_silently_refresh_staged_program_or_old_check_results() {
    let f = facts();
    let mut w = configured(&f);
    w.make(&f, Target::Check, None).unwrap();
    w.make(&f, Target::Install, Some("/stage")).unwrap();
    w.configure(&f, config("/usr/local", "Thanks,", true))
        .unwrap();
    assert!(!w.is_build_current(&f));
    assert!(!w.is_check_current(&f));
    assert_eq!(
        w.run("/stage/usr/local/bin/greet", &["Ada".into()])
            .unwrap()
            .stdout,
        "Welcome, Ada\n"
    );
    w.make(&f, Target::Check, None).unwrap();
    assert!(w.is_check_current(&f));
    assert_eq!(
        w.run("/usr/src/greet-1.4/greet", &["Rio".into()])
            .unwrap()
            .stdout,
        "THANKS, RIO\n"
    );
    assert_eq!(
        w.run("/stage/usr/local/bin/greet", &["Rio".into()])
            .unwrap()
            .stdout,
        "Welcome, Rio\n"
    );
    w.make(&f, Target::Install, Some("/stage")).unwrap();
    assert_eq!(
        w.run("/stage/usr/local/bin/greet", &["Rio".into()])
            .unwrap()
            .stdout,
        "THANKS, RIO\n"
    );
}
#[test]
fn clean_removes_only_build_products_and_leaves_staged_copies_unchanged() {
    let f = facts();
    let mut w = configured(&f);
    w.make(&f, Target::Install, Some("/stage")).unwrap();
    let installed = w.files["/stage/usr/local/bin/greet"].clone();
    w.make(&f, Target::Clean, None).unwrap();
    assert_eq!(w.files["/stage/usr/local/bin/greet"], installed);
    assert!(!w.files.contains_key("/usr/src/greet-1.4/greet"));
    assert!(w.configuration.is_some());
}
#[test]
fn missing_destdir_changes_virtual_live_files_and_is_not_falsely_presented_as_protected() {
    let f = facts();
    let mut w = configured(&f);
    w.make(&f, Target::Install, None).unwrap();
    assert!(matches!(
        w.files["/usr/local/bin/greet"],
        File::Program { .. }
    ));
    assert_eq!(
        w.run("/usr/local/bin/greet", &["Ada".into()])
            .unwrap()
            .stdout,
        "Welcome, Ada\n"
    );
    // There is no host-path read/write in the model; this is its owned map only.
}
#[test]
fn prefix_and_staging_roots_handle_absolute_paths_and_root_correctly() {
    let f = facts();
    for (prefix, stage, expected) in [
        ("/opt/team", Some("/review"), "/review/opt/team/bin/greet"),
        ("/", Some("/review"), "/review/bin/greet"),
        ("/opt/team", Some("/"), "/opt/team/bin/greet"),
        ("/", None, "/bin/greet"),
    ] {
        let c = config(prefix, "Hello,", false);
        assert_eq!(World::installed_paths(&f, &c, stage).unwrap().0, expected);
    }
}
#[test]
fn invalid_paths_collisions_and_excessive_inputs_are_refused_atomically() {
    let f = facts();
    let mut w = configured(&f);
    for bad in ["relative", "/a/../b", "/a//b", "/a/./b", "/a/;touch"] {
        let prior = w.clone();
        assert!(w.make(&f, Target::Install, Some(bad)).is_err());
        assert_eq!(w, prior);
        assert!(config(bad, "Hello,", false).validate().is_err());
    }
    for bad in ["", "line\nsecond", "\u{1b}[31m", "λ"] {
        let prior = w.clone();
        assert!(w.configure(&f, config("/opt/team", bad, false)).is_err());
        assert_eq!(w, prior);
    }
    let prior = w.clone();
    assert!(w.make(&f, Target::All, Some("/stage")).is_err());
    assert_eq!(w, prior);
    let mut collision = f.clone();
    collision.live_files.extend([
        TextFile {
            path: "/usr/src".into(),
            text: "not a directory".into(),
            mode: 0o644,
        },
        TextFile {
            path: "/usr/src-other".into(),
            text: "lexical sibling".into(),
            mode: 0o644,
        },
    ]);
    let mut w = World::new(&collision).unwrap();
    let prior = w.clone();
    assert!(w.extract(&collision, "/usr/src").is_err());
    assert_eq!(w, prior);
    assert!(model::join("/a", "../escape").is_err());
}
#[test]
fn malformed_source_and_closed_versioned_data_fail_before_execution() {
    let f = facts();
    let mut bad = f.clone();
    bad.version = 2;
    assert!(World::new(&bad).is_err());
    let mut bad = f.clone();
    let entry = bad.source.entries.remove("INSTALL").unwrap();
    bad.source.entries.insert("../INSTALL".into(), entry);
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.program.name = "configure".into();
    assert!(bad.validate().is_err());
    let mut json = serde_json::to_value(&f).unwrap();
    json["host_command"] = serde_json::json!("anything");
    assert!(serde_json::from_value::<Facts>(json).is_err());
    let mut json = serde_json::to_value(&f).unwrap();
    json["program"]["default_greeting"] = serde_json::json!("x".repeat(65));
    assert!(serde_json::from_value::<Facts>(json).is_err());
}

#[test]
fn empty_directories_and_directory_file_collisions_remain_real_virtual_state() {
    let mut f = facts();
    let mut entry = f.source.entries[""].clone();
    entry.node.mode = 0o755;
    f.source.entries.insert("empty".into(), entry);
    let mut w = World::new(&f).unwrap();
    w.extract(&f, "/usr/src").unwrap();
    assert!(w.directories.contains("/usr/src/greet-1.4/empty"));
    w.mkdir("/stage/usr/local/bin/greet", true).unwrap();
    w.configure(&f, config("/usr/local", "Hello,", false))
        .unwrap();
    let prior = w.clone();
    assert!(w.make(&f, Target::Install, Some("/stage")).is_err());
    assert_eq!(w, prior);
    let prior = w.clone();
    assert!(w.mkdir("/usr/local/bin/greet/subdir", true).is_err());
    assert_eq!(w, prior);
}
#[test]
fn command_parser_accepts_equivalent_real_spellings_and_preserves_staging_identity() {
    use model::command;
    let f = facts();
    let mut w = World::new(&f).unwrap();
    command::prepare_directories(&mut w, &["/usr/src", "/stage"]).unwrap();
    let mut cwd: String = "/sources".into();
    let list = command::run(&mut w, &f, &mut cwd, "tar --list --file=greet-1.4.tar").unwrap();
    assert!(list.stdout.contains("greet-1.4/INSTALL"));
    for cmd in [
        "tar -xvf greet-1.4.tar -C /usr/src",
        "cd /usr/src/greet-1.4",
        "./configure --prefix /opt/tools --with-greeting 'Good day,' --enable-uppercase",
        "make check",
        "make DESTDIR=/stage install",
    ] {
        command::run(&mut w, &f, &mut cwd, cmd).unwrap();
    }
    assert!(w.is_check_current(&f));
    let out = command::run(
        &mut w,
        &f,
        &mut cwd,
        "/stage/opt/tools/bin/greet 'Mira Lee'",
    )
    .unwrap();
    assert_eq!(out.stdout, "GOOD DAY, MIRA LEE\n");
    assert_eq!(
        command::run(&mut w, &f, &mut cwd, "/stage/opt/tools/bin/greet --prefix")
            .unwrap()
            .stdout,
        "/opt/tools\n"
    );
    command::run(&mut w, &f, &mut cwd, "cd ..").unwrap();
    assert_eq!(cwd, "/usr/src");
    command::run(&mut w, &f, &mut cwd, "make -C /usr/src/greet-1.4 clean").unwrap();
    assert_eq!(cwd, "/usr/src");
    assert_eq!(
        command::run(&mut w, &f, &mut cwd, "/stage/opt/tools/bin/greet Rio")
            .unwrap()
            .stdout,
        "GOOD DAY, RIO\n"
    );
}
#[test]
fn unsupported_syntax_and_failed_commands_are_honest_and_transactional() {
    use model::command;
    let f = facts();
    let mut w = World::new(&f).unwrap();
    command::prepare_directories(&mut w, &["/usr/src"]).unwrap();
    let mut cwd: String = "/sources".into();
    for cmd in [
        "tar -ft greet-1.4.tar",
        "tar -xzf greet-1.4.tar",
        "tar -tf /etc/passwd",
        "tar -xf greet-1.4.tar -C /missing",
        "make install",
        "mkdir -p",
        "mkdir -x",
        "cd /etc",
        "cat /etc/passwd",
        "sh configure",
        "echo ok > /host-file",
        "make; pwd",
        "$(pwd)",
        "mkdir -p /usr/local/bin/greet/subdir",
    ] {
        let prior = w.clone();
        let before = cwd.clone();
        assert!(command::run(&mut w, &f, &mut cwd, cmd).is_err(), "{cmd}");
        assert_eq!(w, prior, "{cmd}");
        assert_eq!(cwd, before);
    }
    command::run(&mut w, &f, &mut cwd, "tar -xf greet-1.4.tar -C /usr/src").unwrap();
    command::run(&mut w, &f, &mut cwd, "cd /usr/src/greet-1.4").unwrap();
    for cmd in [
        "./configure --prefix=/a --prefix=/b",
        "./configure --enable-uppercase --disable-uppercase",
        "./configure --enable-arbitrary",
        "make all install",
        "make CFLAGS=anything",
        "cat -n INSTALL",
    ] {
        let prior = w.clone();
        assert!(command::run(&mut w, &f, &mut cwd, cmd).is_err(), "{cmd}");
        assert_eq!(w, prior, "{cmd}");
    }
    assert_eq!(
        command::resolve("/sources", "../../../../etc/passwd").unwrap(),
        "/etc/passwd"
    );
    // Normalized paths still address only the finite virtual map, not the host.
}

#[test]
fn documentation_install_uses_actual_source_bytes_and_invalid_source_modes_are_rejected() {
    let f = facts();
    let mut w = configured(&f);
    w.files.insert(
        "/usr/src/greet-1.4/README".into(),
        File::Text {
            text: "changed source documentation\n".into(),
            mode: 0o644,
        },
    );
    w.make(&f, Target::Install, Some("/stage")).unwrap();
    assert!(
        matches!(&w.files["/stage/usr/local/share/doc/greet/README"],File::Text{text,mode:0o644}if text=="changed source documentation\n")
    );
    let mut bad = f.clone();
    bad.source.entries.get_mut("configure").unwrap().node.mode = 0o644;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.source.entries.get_mut("README").unwrap().text =
        Some("different from approved manual".into());
    assert!(bad.validate().is_err());
}
