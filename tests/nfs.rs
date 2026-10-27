//! Prospective pure-model checks. These invoke no host NFS service or utility.
use dlms_learning_lab::nfs::*;
use serde_json::json;

fn inode(path: &str, uid: u32, gid: u32, mode: u16) -> Inode {
    Inode {
        path: path.into(),
        uid,
        gid,
        mode,
    }
}
fn supplied() -> State {
    let profiles = vec![
        Profile {
            id: "writer".into(),
            label: "writer".into(),
            uid: 2001,
            gid: 3001,
            groups: vec![],
        },
        Profile {
            id: "reader".into(),
            label: "reader".into(),
            uid: 2002,
            gid: 3001,
            groups: vec![],
        },
        Profile {
            id: "root".into(),
            label: "root".into(),
            uid: 0,
            gid: 0,
            groups: vec![],
        },
        Profile {
            id: "anonymous".into(),
            label: "anonymous".into(),
            uid: 65534,
            gid: 65534,
            groups: vec![],
        },
        Profile {
            id: "wrong".into(),
            label: "writer".into(),
            uid: 5001,
            gid: 4001,
            groups: vec![],
        },
        Profile {
            id: "primary-zero".into(),
            label: "primary zero".into(),
            uid: 2002,
            gid: 0,
            groups: vec![],
        },
        Profile {
            id: "supplementary-zero".into(),
            label: "supplementary zero".into(),
            uid: 2002,
            gid: 4001,
            groups: vec![0],
        },
        Profile {
            id: "root-retains-group".into(),
            label: "root retains group".into(),
            uid: 0,
            gid: 0,
            groups: vec![3001],
        },
    ];
    let clients = [
        ("old", "192.0.2.20"),
        ("new", "192.0.2.21"),
        ("outside", "198.51.100.30"),
        ("archive", "198.51.100.31"),
    ]
    .into_iter()
    .map(|(id, address)| Client {
        id: id.into(),
        address: address.parse().unwrap(),
        mountpoints: ["work", "reference", "archive", "reports"]
            .into_iter()
            .map(|p| format!("/mnt/{p}"))
            .collect(),
        profiles: profiles.iter().map(|p| p.id.clone()).collect(),
    })
    .collect();
    let roots = vec![
        Root {
            server: "files".into(),
            path: "/srv/work".into(),
            directories: vec![inode("/srv/work", 9001, 3001, 0o755)],
            files: vec![
                inode("/srv/work/result.txt", 2001, 3001, 0o660),
                inode("/srv/work/anon.txt", 65534, 65534, 0o600),
            ],
        },
        Root {
            server: "files".into(),
            path: "/srv/reference".into(),
            directories: vec![inode("/srv/reference", 9001, 3001, 0o755)],
            files: vec![inode("/srv/reference/notice.txt", 9001, 3001, 0o664)],
        },
        Root {
            server: "files".into(),
            path: "/srv/archive".into(),
            directories: vec![inode("/srv/archive", 9001, 3001, 0o755)],
            files: vec![inode("/srv/archive/archive.txt", 9001, 3001, 0o664)],
        },
        Root {
            server: "files".into(),
            path: "/srv/reports".into(),
            directories: vec![inode("/srv/reports", 9001, 3001, 0o755)],
            files: vec![inode("/srv/reports/report.txt", 9001, 3001, 0o664)],
        },
        Root {
            server: "files".into(),
            path: "/srv/mapped".into(),
            directories: vec![inode("/srv/mapped", 9001, 65534, 0o710)],
            files: vec![inode("/srv/mapped/secret.txt", 9002, 65534, 0o660)],
        },
    ];
    let contents = roots
        .iter()
        .flat_map(|r| {
            r.files.iter().map(|f| Contents {
                server: r.server.clone(),
                path: f.path.clone(),
                text: if f.path.ends_with("result.txt") {
                    String::new()
                } else {
                    format!("original:{}", f.path)
                },
            })
        })
        .collect();
    let runtime = Runtime {
        servers: vec![ServerLayers {
            server: "files".into(),
            draft: String::new(),
            saved: String::new(),
            applied: vec![],
        }],
        clients: ["old", "new", "outside", "archive"]
            .into_iter()
            .map(|client| ClientLayers {
                client: client.into(),
                draft: String::new(),
                saved: String::new(),
            })
            .collect(),
        mounts: vec![],
        contents,
    };
    State::new(
        Inventory {
            fixture: "nfs-proof".into(),
            servers: vec![Server {
                id: "files".into(),
                address: "192.0.2.10".parse().unwrap(),
            }],
            clients,
            profiles,
            roots,
        },
        runtime,
    )
    .unwrap()
}
fn run(s: &mut State, a: Action) -> Decision {
    s.apply(&a).unwrap()
}
fn server_text(s: &mut State, text: &str) {
    run(
        s,
        Action::SetExportsDraft {
            server: "files".into(),
            text: text.into(),
        },
    );
    run(
        s,
        Action::SaveExportsDraft {
            server: "files".into(),
        },
    );
}
fn policy(s: &mut State, text: &str) {
    server_text(s, text);
    run(
        s,
        Action::ApplySavedExports {
            server: "files".into(),
        },
    );
}
fn client_text(s: &mut State, client: &str, text: &str) {
    run(
        s,
        Action::SetFstabDraft {
            client: client.into(),
            text: text.into(),
        },
    );
    run(
        s,
        Action::SaveFstabDraft {
            client: client.into(),
        },
    );
}
fn spec(root: &str, target: &str, access: Access) -> MountSpec {
    MountSpec {
        server: "files".into(),
        root: root.into(),
        mountpoint: target.into(),
        access,
    }
}
fn mount(s: &mut State, client: &str, root: &str, target: &str, access: Access) -> Decision {
    run(
        s,
        Action::MountExplicit {
            client: client.into(),
            specification: spec(root, target, access),
        },
    )
}
fn read(s: &mut State, client: &str, target: &str, profile: &str, file: &str) -> Decision {
    run(
        s,
        Action::ReadExisting {
            client: client.into(),
            mountpoint: target.into(),
            profile: profile.into(),
            relative: file.into(),
        },
    )
}
fn write(
    s: &mut State,
    client: &str,
    target: &str,
    profile: &str,
    file: &str,
    text: &str,
) -> Decision {
    run(
        s,
        Action::WriteExisting {
            client: client.into(),
            mountpoint: target.into(),
            profile: profile.into(),
            relative: file.into(),
            text: text.into(),
        },
    )
}
fn denied_at(d: &Decision, stage: DenialStage) {
    assert!(
        matches!(d,Decision::Denied{stage:s,..} if *s==stage),
        "{d:?}"
    )
}
fn text<'a>(s: &'a State, path: &str) -> &'a str {
    &s.runtime()
        .contents
        .iter()
        .find(|v| v.path == path)
        .unwrap()
        .text
}
fn row(root: &str, target: &str, mode: &str) -> String {
    format!("192.0.2.10:{root} {target} nfs vers=3,proto=tcp,sec=sys,{mode} 0 0\n")
}
const ARCHIVE: &str = "/srv/archive 198.51.100.31(ro)\n";

#[test]
fn reference_export_two_distinct_policies_and_saved_only_negative() {
    for reference in [
        "/srv/reference 192.0.2.0/24(ro)\n",
        "/srv/reference 192.0.2.20(ro) 192.0.2.21(ro)\n",
    ] {
        let mut s = supplied();
        policy(&mut s, ARCHIVE);
        mount(
            &mut s,
            "archive",
            "/srv/archive",
            "/mnt/archive",
            Access::Ro,
        );
        let preserved = s.runtime().mounts.clone();
        let archive = text(&s, "/srv/archive/archive.txt").to_owned();
        server_text(&mut s, &format!("{ARCHIVE}{reference}"));
        denied_at(
            &mount(
                &mut s,
                "old",
                "/srv/reference",
                "/mnt/reference",
                Access::Ro,
            ),
            DenialStage::ExportAdmission,
        );
        run(
            &mut s,
            Action::ApplySavedExports {
                server: "files".into(),
            },
        );
        assert_eq!(
            mount(
                &mut s,
                "old",
                "/srv/reference",
                "/mnt/reference",
                Access::Ro
            ),
            Decision::Mounted {}
        );
        assert!(matches!(
            read(&mut s, "old", "/mnt/reference", "reader", "notice.txt"),
            Decision::AllowedRead { .. }
        ));
        denied_at(
            &mount(
                &mut s,
                "outside",
                "/srv/reference",
                "/mnt/reference",
                Access::Ro,
            ),
            DenialStage::ExportAdmission,
        );
        assert_eq!(s.runtime().mounts[0], preserved[0]);
        assert_eq!(text(&s, "/srv/archive/archive.txt"), archive);
        // A compatible CIDR/host spelling change preserves actual mount bindings.
        policy(
            &mut s,
            &format!("{ARCHIVE}/srv/reference 192.0.2.20(ro) 192.0.2.21(ro)\n"),
        );
        assert_eq!(s.runtime().mounts.len(), 2);
    }
}

#[test]
fn write_diagnosis_two_repairs_require_actual_bytes_and_squash_is_mapping() {
    for saved_mount in [true, false] {
        let mut s = supplied();
        policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
        mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Ro);
        let before = s.clone();
        denied_at(
            &write(
                &mut s,
                "old",
                "/mnt/work",
                "writer",
                "result.txt",
                "published",
            ),
            DenialStage::ClientReadOnly,
        );
        assert_eq!(s, before);
        if saved_mount {
            client_text(&mut s, "old", &row("/srv/work", "/mnt/work", "rw"));
        }
        run(
            &mut s,
            Action::Unmount {
                client: "old".into(),
                mountpoint: "/mnt/work".into(),
            },
        );
        if saved_mount {
            run(
                &mut s,
                Action::MountSaved {
                    client: "old".into(),
                    mountpoint: "/mnt/work".into(),
                },
            );
        } else {
            mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
            client_text(&mut s, "old", &row("/srv/work", "/mnt/work", "rw"));
        }
        assert_eq!(text(&s, "/srv/work/result.txt"), "");
        denied_at(
            &write(&mut s, "old", "/mnt/work", "root", "result.txt", "bad"),
            DenialStage::FilePermission,
        );
        assert!(matches!(
            write(
                &mut s,
                "old",
                "/mnt/work",
                "root",
                "anon.txt",
                "anonymous owner works"
            ),
            Decision::AllowedWrite {
                mapping: Mapping {
                    uid: 65534,
                    gid: 65534,
                    ..
                },
                class: Class::Owner
            }
        ));
        denied_at(
            &write(&mut s, "old", "/mnt/work", "wrong", "result.txt", "bad"),
            DenialStage::FilePermission,
        );
        assert!(matches!(
            write(
                &mut s,
                "old",
                "/mnt/work",
                "writer",
                "result.txt",
                "published"
            ),
            Decision::AllowedWrite { .. }
        ));
        assert_eq!(text(&s, "/srv/work/result.txt"), "published");
    }
}

#[test]
fn handover_requires_explicit_unmount_and_preserves_unrelated_mount() {
    for save_new_first in [true, false] {
        let mut s = supplied();
        policy(&mut s, &format!("{ARCHIVE}/srv/work 192.0.2.20(rw)\n"));
        client_text(&mut s, "old", &row("/srv/work", "/mnt/work", "rw"));
        mount(
            &mut s,
            "archive",
            "/srv/archive",
            "/mnt/archive",
            Access::Ro,
        );
        mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
        let archive = s.runtime().mounts[0].clone();
        let bytes = text(&s, "/srv/archive/archive.txt").to_owned();
        if save_new_first {
            client_text(&mut s, "new", &row("/srv/work", "/mnt/work", "rw"));
        }
        server_text(&mut s, &format!("{ARCHIVE}/srv/work 192.0.2.21(rw)\n"));
        let before = s.clone();
        let e = s
            .apply(&Action::ApplySavedExports {
                server: "files".into(),
            })
            .unwrap_err();
        assert_eq!(e.kind, FaultKind::SimulatorUnsupported);
        assert!(e.message.contains("ActiveMountConflict") && e.message.contains("old:/mnt/work"));
        assert_eq!(s, before);
        run(
            &mut s,
            Action::Unmount {
                client: "old".into(),
                mountpoint: "/mnt/work".into(),
            },
        );
        client_text(&mut s, "old", "");
        run(
            &mut s,
            Action::ApplySavedExports {
                server: "files".into(),
            },
        );
        if save_new_first {
            run(
                &mut s,
                Action::MountSaved {
                    client: "new".into(),
                    mountpoint: "/mnt/work".into(),
                },
            );
        } else {
            mount(&mut s, "new", "/srv/work", "/mnt/work", Access::Rw);
            client_text(&mut s, "new", &row("/srv/work", "/mnt/work", "rw"));
        }
        denied_at(
            &mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw),
            DenialStage::ExportAdmission,
        );
        assert_eq!(s.runtime().mounts[0], archive);
        assert_eq!(text(&s, "/srv/archive/archive.txt"), bytes);
        assert!(s.runtime().mounts.iter().any(|m| m.client == "new"));
        assert!(!s.runtime().mounts.iter().any(|m| m.client == "old"));
    }
}

#[test]
fn two_share_capstone_alternates_publish_preserve_and_keep_modes_distinct() {
    for (selector, saved_first) in [("192.0.2.20", true), ("192.0.2.20/32", false)] {
        let mut s = supplied();
        policy(
            &mut s,
            &format!("{ARCHIVE}/srv/work {selector}(rw)\n/srv/reports 192.0.2.21(ro)\n"),
        );
        mount(
            &mut s,
            "archive",
            "/srv/archive",
            "/mnt/archive",
            Access::Ro,
        );
        let archive = s.runtime().mounts[0].clone();
        let report = text(&s, "/srv/reports/report.txt").to_owned();
        let archive_bytes = text(&s, "/srv/archive/archive.txt").to_owned();
        if saved_first {
            client_text(&mut s, "old", &row("/srv/work", "/mnt/work", "rw"));
            client_text(&mut s, "new", &row("/srv/reports", "/mnt/reports", "ro"));
            run(
                &mut s,
                Action::MountSaved {
                    client: "old".into(),
                    mountpoint: "/mnt/work".into(),
                },
            );
            run(
                &mut s,
                Action::MountSaved {
                    client: "new".into(),
                    mountpoint: "/mnt/reports".into(),
                },
            );
        } else {
            mount(&mut s, "new", "/srv/reports", "/mnt/reports", Access::Ro);
            mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
            client_text(&mut s, "new", &row("/srv/reports", "/mnt/reports", "ro"));
            client_text(&mut s, "old", &row("/srv/work", "/mnt/work", "rw"));
        }
        assert!(matches!(
            write(
                &mut s,
                "old",
                "/mnt/work",
                "writer",
                "result.txt",
                "handover complete"
            ),
            Decision::AllowedWrite { .. }
        ));
        assert!(
            matches!(read(&mut s,"new","/mnt/reports","reader","report.txt"),Decision::AllowedRead{text,..} if text==report)
        );
        denied_at(
            &write(&mut s, "new", "/mnt/reports", "reader", "report.txt", "bad"),
            DenialStage::ClientReadOnly,
        );
        denied_at(
            &write(&mut s, "old", "/mnt/work", "root", "result.txt", "bad"),
            DenialStage::FilePermission,
        );
        denied_at(
            &mount(&mut s, "outside", "/srv/work", "/mnt/work", Access::Rw),
            DenialStage::ExportAdmission,
        );
        assert_eq!(text(&s, "/srv/work/result.txt"), "handover complete");
        assert_eq!(text(&s, "/srv/reports/report.txt"), report);
        assert_eq!(text(&s, "/srv/archive/archive.txt"), archive_bytes);
        assert_eq!(s.runtime().mounts[0], archive);
    }
}

#[test]
fn defaults_cidr_boundaries_and_closed_parser_are_not_substring_answers() {
    let s = supplied();
    let i = s.inventory();
    for (selector, address, yes) in [
        ("0.0.0.0/0", "203.0.113.9", true),
        ("192.0.2.20/32", "192.0.2.20", true),
        ("192.0.2.20", "192.0.2.21", false),
        ("192.0.2.0/24", "192.0.3.0", false),
    ] {
        let parsed = parse_exports(
            i,
            "files",
            &format!("# note\r\n/srv/work {selector} # trailing\r\n"),
        )
        .unwrap();
        assert_eq!(parsed[0].clients[0].options, Options::default());
        assert_eq!(
            parsed[0].clients[0]
                .selector
                .contains(address.parse().unwrap()),
            yes
        );
    }
    for selector in [
        "192.0.2.20/24",
        "192.0.2.0/33",
        "192.000.2.20",
        "192.0.2.0/024",
        "::1",
        "host.example",
        "192.0.2.0/255.255.255.0",
    ] {
        assert!(
            parse_exports(i, "files", &format!("/srv/work {selector}\n")).is_err(),
            "{selector}"
        );
    }
    for source in [
        "/srv/work 192.0.2.0/24 192.0.2.20",
        "/srv/work 192.0.2.20 192.0.2.20/32",
        "/srv/work 192.0.2.20(rw,ro)",
        "/srv/work 192.0.2.20(sec=krb5)",
        "/srv/work 192.0.2.20 (rw)",
        "/srv/work 192.0.2.20(anonuid=0)",
    ] {
        assert!(parse_exports(i, "files", source).is_err(), "{source}");
    }
    let a=parse_exports(i,"files","/srv/work 192.0.2.20(rw,sync,root_squash,anonuid=2100,anongid=3100,sec=sys) 192.0.2.21(ro)\n").unwrap();
    assert_eq!(a[0].clients[0].options.anon_uid, 2100);
}

#[test]
fn full_saved_documents_parse_before_mount_and_save_never_repairs_or_applies() {
    let mut s = supplied();
    let malformed = "not a valid export\r\n";
    server_text(&mut s, malformed);
    assert_eq!(s.runtime().servers[0].saved, malformed);
    assert!(s.runtime().servers[0].applied.is_empty());
    let before = s.clone();
    assert!(
        s.apply(&Action::ApplySavedExports {
            server: "files".into()
        })
        .is_err()
    );
    assert_eq!(s, before);
    policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
    client_text(
        &mut s,
        "old",
        &format!(
            "{}broken trailing row\n",
            row("/srv/work", "/mnt/work", "rw")
        ),
    );
    let before = s.clone();
    assert!(
        s.apply(&Action::MountSaved {
            client: "old".into(),
            mountpoint: "/mnt/work".into()
        })
        .is_err()
    );
    assert_eq!(s, before);
    let valid = "192.0.2.10:/srv/work /mnt/work nfs rw,sec=sys,proto=tcp,vers=3 0 0\r\n";
    assert_eq!(
        parse_fstab(s.inventory(), "old", valid).unwrap()[0].access,
        Access::Rw
    );
    for bad in [
        valid.replace("vers=3", "vers=4"),
        valid.replace("proto=tcp", "proto=udp"),
        valid.replace("sec=sys", "sec=krb5"),
        valid.replace("192.0.2.10", "files.example"),
        valid.replace(" 0 0", " 0 1"),
        format!("{valid}{valid}"),
    ] {
        assert!(parse_fstab(s.inventory(), "old", &bad).is_err());
    }
}

#[test]
fn mapped_primary_and_supplementary_zero_allow_directory_search_without_raw_id_dac() {
    let mut s = supplied();
    policy(&mut s, "/srv/mapped 192.0.2.20(rw)\n");
    mount(&mut s, "old", "/srv/mapped", "/mnt/work", Access::Rw);
    for profile in ["primary-zero", "supplementary-zero"] {
        let d = write(
            &mut s,
            "old",
            "/mnt/work",
            profile,
            "secret.txt",
            "mapped group permits this",
        );
        assert!(
            matches!(
                d,
                Decision::AllowedWrite {
                    class: Class::Group,
                    ..
                }
            ),
            "{d:?}"
        );
        let Decision::AllowedRead { mapping, .. } =
            read(&mut s, "old", "/mnt/work", profile, "secret.txt")
        else {
            panic!("mapped group read")
        };
        assert_eq!(mapping.uid, 2002);
        assert!(mapping.gid == 65534 || mapping.groups.contains(&65534));
    }
    denied_at(
        &read(&mut s, "old", "/mnt/work", "wrong", "secret.txt"),
        DenialStage::DirectorySearch,
    );
}

#[test]
fn zero_uid_mapping_keeps_nonzero_supplementary_groups_and_custom_anonymous_ids() {
    let mut s = supplied();
    policy(
        &mut s,
        "/srv/work 192.0.2.20(rw,anonuid=2200,anongid=3300)\n",
    );
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    let d = write(
        &mut s,
        "old",
        "/mnt/work",
        "root-retains-group",
        "result.txt",
        "retained group",
    );
    assert!(
        matches!(d,Decision::AllowedWrite{mapping:Mapping{uid:2200,gid:3300,groups},class:Class::Group} if groups==vec![3001])
    );
    denied_at(
        &write(
            &mut s,
            "old",
            "/mnt/work",
            "root",
            "result.txt",
            "no implicit groups",
        ),
        DenialStage::FilePermission,
    );
    let mut inv = s.inventory().clone();
    inv.profiles
        .iter_mut()
        .find(|p| p.id == "root")
        .unwrap()
        .groups = vec![0, 3300];
    let mut s = State::new(inv, s.runtime().clone()).unwrap();
    let d = read(&mut s, "old", "/mnt/work", "root", "result.txt");
    assert!(
        matches!(d,Decision::Denied{mapping:Some(Mapping{groups,..}),..} if groups==vec![3300])
    );
}

#[test]
fn permission_classes_never_fall_through_and_names_are_not_identity() {
    let mut base = supplied();
    let mut inv = base.inventory().clone();
    inv.roots
        .iter_mut()
        .find(|r| r.path == "/srv/work")
        .unwrap()
        .files[0]
        .mode = 0o604;
    base = State::new(inv, base.runtime().clone()).unwrap();
    policy(&mut base, "/srv/work 192.0.2.20(rw)\n");
    mount(&mut base, "old", "/srv/work", "/mnt/work", Access::Rw);
    denied_at(
        &read(&mut base, "old", "/mnt/work", "reader", "result.txt"),
        DenialStage::FilePermission,
    ); // group0 wins over other4
    denied_at(
        &write(&mut base, "old", "/mnt/work", "wrong", "result.txt", "bad"),
        DenialStage::FilePermission,
    ); // same label as writer is irrelevant
    let mut inv = base.inventory().clone();
    let root = inv
        .roots
        .iter_mut()
        .find(|r| r.path == "/srv/work")
        .unwrap();
    root.directories[0].uid = 2001;
    root.directories[0].mode = 0o677;
    let mut base = State::new(inv, base.runtime().clone()).unwrap();
    let d = read(&mut base, "old", "/mnt/work", "writer", "result.txt");
    assert!(matches!(
        d,
        Decision::Denied {
            stage: DenialStage::DirectorySearch,
            class: Some(Class::Owner),
            ..
        }
    ));
}

#[test]
fn directory_search_not_read_or_parent_write_is_required_for_existing_file() {
    let base = supplied();
    let mut inv = base.inventory().clone();
    inv.roots
        .iter_mut()
        .find(|r| r.path == "/srv/work")
        .unwrap()
        .directories[0]
        .mode = 0o111;
    let mut s = State::new(inv, base.runtime().clone()).unwrap();
    policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    assert!(matches!(
        write(
            &mut s,
            "old",
            "/mnt/work",
            "writer",
            "result.txt",
            "works without directory r/w"
        ),
        Decision::AllowedWrite { .. }
    ));
    assert!(matches!(
        read(&mut s, "old", "/mnt/work", "writer", "result.txt"),
        Decision::AllowedRead { .. }
    ));
    let mut inv = s.inventory().clone();
    inv.roots
        .iter_mut()
        .find(|r| r.path == "/srv/work")
        .unwrap()
        .directories[0]
        .mode = 0o666;
    let mut s = State::new(inv, s.runtime().clone()).unwrap();
    denied_at(
        &read(&mut s, "old", "/mnt/work", "writer", "result.txt"),
        DenialStage::DirectorySearch,
    );
}

#[test]
fn both_crossed_mount_modes_succeed_then_writes_fail_at_exact_gate_atomically() {
    for (export, client, stage) in [
        ("ro", Access::Rw, DenialStage::ExportReadOnly),
        ("rw", Access::Ro, DenialStage::ClientReadOnly),
    ] {
        let mut s = supplied();
        policy(&mut s, &format!("/srv/work 192.0.2.20({export})\n"));
        assert_eq!(
            mount(&mut s, "old", "/srv/work", "/mnt/work", client),
            Decision::Mounted {}
        );
        let before = s.clone();
        denied_at(
            &write(&mut s, "old", "/mnt/work", "writer", "result.txt", "bad"),
            stage,
        );
        assert_eq!(s, before);
        assert!(matches!(
            read(&mut s, "old", "/mnt/work", "writer", "result.txt"),
            Decision::AllowedRead { .. }
        ));
    }
}

#[test]
fn unsquashed_uid_zero_is_unsupported_not_denied_and_nonroot_group_zero_is_ordinary() {
    let mut s = supplied();
    policy(&mut s, "/srv/work 192.0.2.20(rw,no_root_squash)\n");
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    let before = s.clone();
    let e = s
        .apply(&Action::WriteExisting {
            client: "old".into(),
            mountpoint: "/mnt/work".into(),
            profile: "root".into(),
            relative: "result.txt".into(),
            text: "bad".into(),
        })
        .unwrap_err();
    assert_eq!(e.kind, FaultKind::SimulatorUnsupported);
    assert!(e.message.contains("UnsquashedRootPrivileges"));
    assert_eq!(s, before);
    denied_at(
        &write(
            &mut s,
            "old",
            "/mnt/work",
            "primary-zero",
            "result.txt",
            "ordinary mismatch",
        ),
        DenialStage::FilePermission,
    );
}

fn one_trial(base: TrialBase, profile: &str, expected: Expected) -> Evaluation {
    Evaluation {
        current: vec![],
        fresh: vec![Trial {
            base,
            steps: vec![
                TrialStep {
                    action: Action::MountExplicit {
                        client: "old".into(),
                        specification: spec("/srv/work", "/mnt/work", Access::Rw),
                    },
                    expected: Expected::Mounted {},
                },
                TrialStep {
                    action: Action::WriteExisting {
                        client: "old".into(),
                        mountpoint: "/mnt/work".into(),
                        profile: profile.into(),
                        relative: "result.txt".into(),
                        text: "scratch only".into(),
                    },
                    expected,
                },
            ],
        }],
    }
}

#[test]
fn fresh_current_and_saved_observations_are_independent_and_checks_do_not_install_results() {
    let mut s = supplied();
    policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
    client_text(&mut s, "old", &row("/srv/work", "/mnt/work", "rw"));
    let before = s.clone();
    let mut p = one_trial(TrialBase::Applied {}, "writer", Expected::Write {});
    p.current.push(Request::MountPresent {
        client: "old".into(),
        mountpoint: "/mnt/work".into(),
    });
    let report = s.evaluate_batch(&p).unwrap();
    assert!(report.fresh[0].passed);
    assert!(matches!(
        report.current[0],
        Observation::Decision {
            decision: Decision::Denied {
                stage: DenialStage::MountAbsent,
                ..
            }
        }
    ));
    assert_eq!(s, before);
    assert_eq!(text(&s, "/srv/work/result.txt"), "");
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    write(
        &mut s,
        "old",
        "/mnt/work",
        "writer",
        "result.txt",
        "live published",
    );
    server_text(&mut s, "/srv/work 192.0.2.20(ro)\n");
    let before = s.clone();
    let applied = s
        .evaluate_batch(&one_trial(
            TrialBase::Applied {},
            "writer",
            Expected::Write {},
        ))
        .unwrap();
    assert!(applied.fresh[0].passed);
    let saved = s
        .evaluate_batch(&one_trial(
            TrialBase::Saved {
                servers: vec!["files".into()],
                client: "old".into(),
            },
            "writer",
            Expected::Write {},
        ))
        .unwrap();
    assert!(!saved.fresh[0].passed);
    assert!(matches!(
        saved.fresh[0].steps[1].observation,
        Observation::Decision {
            decision: Decision::Denied {
                stage: DenialStage::ExportReadOnly,
                ..
            }
        }
    ));
    assert_eq!(s, before);
    assert_eq!(text(&s, "/srv/work/result.txt"), "live published");
}

#[test]
fn unsupported_context_malformed_and_limit_never_satisfy_expected_denial() {
    let mut s = supplied();
    policy(&mut s, "/srv/work 192.0.2.20(rw,no_root_squash)\n");
    let p = one_trial(
        TrialBase::Applied {},
        "root",
        Expected::Denied {
            stage: DenialStage::FilePermission,
        },
    );
    let r = s.evaluate_batch(&p).unwrap();
    assert!(!r.fresh[0].passed);
    assert!(matches!(
        r.fresh[0].steps[1].observation,
        Observation::Fault {
            fault: Fault {
                kind: FaultKind::SimulatorUnsupported,
                ..
            }
        }
    ));
    let p = one_trial(
        TrialBase::Applied {},
        "foreign-profile",
        Expected::Denied {
            stage: DenialStage::FilePermission,
        },
    );
    assert!(!s.evaluate_batch(&p).unwrap().fresh[0].passed);
    server_text(&mut s, "invalid saved source\n");
    let p = one_trial(
        TrialBase::Saved {
            servers: vec!["files".into()],
            client: "old".into(),
        },
        "writer",
        Expected::Denied {
            stage: DenialStage::FilePermission,
        },
    );
    let r = s.evaluate_batch(&p).unwrap();
    assert!(!r.fresh[0].passed && r.fresh[0].initialization_fault.is_some());
    let mut p = one_trial(
        TrialBase::Applied {},
        "writer",
        Expected::Denied {
            stage: DenialStage::FilePermission,
        },
    );
    if let Action::WriteExisting { text, .. } = &mut p.fresh[0].steps[1].action {
        *text = "x".repeat(MAX_FILE + 1)
    }
    assert_eq!(s.evaluate_batch(&p).unwrap_err().kind, FaultKind::Limit);
}

#[test]
fn strict_state_and_action_admission_round_trip_and_context_use_full_inventory() {
    let s = supplied();
    let encoded = serde_json::to_string(&s).unwrap();
    assert_eq!(State::from_json(&encoded).unwrap(), s);
    assert_eq!(serde_json::from_str::<State>(&encoded).unwrap(), s);
    for (field, value) in [("schema_version", json!(2)), ("unknown", json!(true))] {
        let mut v = serde_json::to_value(&s).unwrap();
        v[field] = value;
        assert!(State::from_json(&v.to_string()).is_err());
    }
    assert!(State::from_json(&format!("{{\"schema_version\":1,{}", &encoded[1..])).is_err());
    assert_eq!(
        State::from_json(&" ".repeat(MAX_STATE + 1))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let mut v = serde_json::to_value(&s).unwrap();
    v["runtime"]["contents"][0]["text"] = json!("bad\u{0}text");
    assert!(State::from_json(&v.to_string()).is_err());
    let mut inv = s.inventory().clone();
    inv.profiles[0].uid += 1;
    let changed = State::new(inv, s.runtime().clone()).unwrap();
    assert!(!s.same_context(&changed));
    assert_eq!(s.inventory().fixture, changed.inventory().fixture);
    for raw in [
        "{\"kind\":\"save_exports_draft\",\"server\":\"files\",\"extra\":1}",
        "{\"kind\":\"save_exports_draft\",\"server\":\"files\",\"server\":\"files\"}",
        "{\"kind\":\"future_operation\"}",
    ] {
        assert!(Action::from_json(raw).is_err());
    }
    let a = Action::SetExportsDraft {
        server: "files".into(),
        text: "# malformed retained\r\n".into(),
    };
    assert_eq!(
        Action::from_json(&serde_json::to_string(&a).unwrap()).unwrap(),
        a
    );
}

#[test]
fn stale_binding_missing_file_and_changed_metadata_admission_are_closed() {
    let mut s = supplied();
    policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    let mut r = s.runtime().clone();
    r.mounts[0].binding.access = Access::Ro;
    assert_eq!(
        State::new(s.inventory().clone(), r).unwrap_err().kind,
        FaultKind::Context
    );
    let mut r = s.runtime().clone();
    r.servers[0].applied[0].clients[0].options.access = Access::Ro;
    assert_eq!(
        State::new(s.inventory().clone(), r).unwrap_err().kind,
        FaultKind::Context
    );
    let mut r = s.runtime().clone();
    r.contents.pop();
    assert_eq!(
        State::new(s.inventory().clone(), r).unwrap_err().kind,
        FaultKind::Context
    );
    let mut r = s.runtime().clone();
    r.contents[0] = r.contents[1].clone();
    assert_eq!(
        State::new(s.inventory().clone(), r).unwrap_err().kind,
        FaultKind::Context
    );
    let mut inv = s.inventory().clone();
    inv.roots[0].files[0].mode = 0o440;
    assert_eq!(
        State::new(inv, s.runtime().clone()).unwrap_err().kind,
        FaultKind::SimulatorUnsupported
    );
    let mut inv = s.inventory().clone();
    inv.roots[0].files[0].mode = 0o760;
    assert_eq!(
        State::new(inv, s.runtime().clone()).unwrap_err().kind,
        FaultKind::SimulatorUnsupported
    );
    let mut inv = s.inventory().clone();
    inv.roots[0].files[0].path = "/srv/work/missing-parent/file.txt".into();
    assert_eq!(
        State::new(inv, s.runtime().clone()).unwrap_err().kind,
        FaultKind::Context
    );
    let mut inv = s.inventory().clone();
    let mut r = s.runtime().clone();
    for n in 0..26 {
        let path = format!("/srv/work/extra{n}.txt");
        inv.roots[0].files.push(inode(&path, 2001, 3001, 0o600));
        r.contents.push(Contents {
            server: "files".into(),
            path,
            text: "x".repeat(MAX_FILE),
        });
    }
    assert_eq!(State::new(inv, r).unwrap_err().kind, FaultKind::Limit);
    let encoded = serde_json::to_string(&s).unwrap();
    let resumed = State::from_json(&encoded).unwrap();
    assert_eq!(resumed, s);
    assert_eq!(serde_json::to_string(&resumed).unwrap(), encoded);
}

#[test]
fn collection_n_plus_one_is_rejected_before_its_typed_body_and_direct_bounds_match() {
    let s = supplied();
    let mut v = serde_json::to_value(&s).unwrap();
    let first = v["inventory"]["servers"][0].clone();
    v["inventory"]["servers"] = json!([first.clone(),first,{"nested-poison":[]}]);
    let e = serde_json::from_value::<State>(v).unwrap_err().to_string();
    assert!(e.contains("collection limit before typed body"), "{e}");
    let mut p = one_trial(TrialBase::Applied {}, "writer", Expected::Write {});
    let first = p.fresh[0].steps[0].clone();
    p.fresh[0].steps = vec![first; 17];
    assert_eq!(s.evaluate_batch(&p).unwrap_err().kind, FaultKind::Limit);
    let mut v = serde_json::to_value(one_trial(
        TrialBase::Applied {},
        "writer",
        Expected::Write {},
    ))
    .unwrap();
    let a = v["fresh"][0]["steps"][0].clone();
    v["fresh"][0]["steps"] = json!([a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a.clone(),a,{"poison":null}]);
    let e = serde_json::from_value::<Evaluation>(v)
        .unwrap_err()
        .to_string();
    assert!(e.contains("collection limit before typed body"), "{e}");
    let mut inv = s.inventory().clone();
    inv.profiles[0].groups = vec![1; 17];
    assert!(State::new(inv, s.runtime().clone()).is_err());
}

#[test]
fn canonical_paths_unknown_files_and_all_failed_writes_leave_state_exact() {
    let mut s = supplied();
    policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    for rel in [
        "../result.txt",
        "/result.txt",
        "result.txt/",
        "a//b",
        "./result.txt",
        "result%2etxt",
        "result.txt;id",
        "missing.txt",
        "a\\b",
    ] {
        let before = s.clone();
        assert!(
            s.apply(&Action::WriteExisting {
                client: "old".into(),
                mountpoint: "/mnt/work".into(),
                profile: "writer".into(),
                relative: rel.into(),
                text: "bad".into()
            })
            .is_err(),
            "{rel}"
        );
        assert_eq!(s, before);
    }
    let before = s.clone();
    assert!(
        s.apply(&Action::WriteExisting {
            client: "old".into(),
            mountpoint: "/mnt/work".into(),
            profile: "writer".into(),
            relative: "result.txt".into(),
            text: "\0".into()
        })
        .is_err()
    );
    assert_eq!(s, before);
    assert!(matches!(
        write(&mut s, "old", "/mnt/work", "writer", "result.txt", ""),
        Decision::AllowedWrite { .. }
    ));
}

#[test]
fn report_joint_read_limit_and_instruction_counts_are_closed() {
    let mut s = supplied();
    policy(&mut s, "/srv/work 192.0.2.20(rw)\n");
    mount(&mut s, "old", "/srv/work", "/mnt/work", Access::Rw);
    write(
        &mut s,
        "old",
        "/mnt/work",
        "writer",
        "result.txt",
        &"\u{1}".repeat(MAX_FILE),
    );
    let r = Request::Read {
        client: "old".into(),
        mountpoint: "/mnt/work".into(),
        profile: "writer".into(),
        relative: "result.txt".into(),
    };
    let p = Evaluation {
        current: vec![r.clone(); 32],
        fresh: vec![],
    };
    let before = s.clone();
    assert_eq!(s.evaluate_batch(&p).unwrap_err().kind, FaultKind::Limit);
    assert_eq!(s, before);
    let p = Evaluation {
        current: vec![r; 1],
        fresh: vec![],
    };
    let report = s.evaluate_batch(&p).unwrap();
    assert!(serde_json::to_vec(&report).unwrap().len() <= MAX_RETURNED);
    let r = Request::MountPresent {
        client: "old".into(),
        mountpoint: "/mnt/work".into(),
    };
    let step = TrialStep {
        action: Action::MountExplicit {
            client: "old".into(),
            specification: spec("/srv/work", "/mnt/work", Access::Rw),
        },
        expected: Expected::Mounted {},
    };
    let p = Evaluation {
        current: vec![r.clone(); 33],
        fresh: vec![],
    };
    assert_eq!(s.evaluate_batch(&p).unwrap_err().kind, FaultKind::Limit);
    let p = Evaluation {
        current: vec![r; 32],
        fresh: vec![
            Trial {
                base: TrialBase::Applied {},
                steps: vec![step; 16]
            };
            4
        ],
    };
    let before = s.clone();
    let _ = s.evaluate_batch(&p).unwrap();
    assert_eq!(s, before); // 32 + 4*(1+16) = 100, bounded below 128.
}

#[test]
fn proposed_escaped_state_frontier_refuses_set_save_and_write_without_mutation() {
    let document = ("\t".repeat(511) + "\n").repeat(16);
    assert_eq!(document.len(), MAX_DOCUMENT);
    // Each next document has the same escaped size. Stop at the first genuinely refused candidate.
    fn frontier(seed: &State, document: &str, exclude_old_saved: bool) -> State {
        let mut r = seed.runtime().clone();
        let mut last = seed.clone();
        for n in 0..10 {
            if exclude_old_saved && n == 1 {
                continue;
            }
            let field = if n < 8 {
                if n % 2 == 0 {
                    &mut r.clients[n / 2].draft
                } else {
                    &mut r.clients[n / 2].saved
                }
            } else if n == 8 {
                &mut r.servers[0].draft
            } else {
                &mut r.servers[0].saved
            };
            if field.as_str() == document {
                continue;
            }
            *field = document.into();
            match State::new(seed.inventory().clone(), r.clone()) {
                Ok(next) => last = next,
                Err(e) => {
                    assert_eq!(e.kind, FaultKind::Limit);
                    return last;
                }
            }
        }
        panic!("frontier fixture must really exceed the joint cap")
    }
    let seed = supplied();
    let mut r = seed.runtime().clone();
    for c in &mut r.contents {
        c.text = "\u{1}".repeat(MAX_FILE)
    }
    r.clients[0].draft = document.clone();
    let seed = State::new(seed.inventory().clone(), r).unwrap();
    let mut last = frontier(&seed, &document, true);
    let before = last.clone();
    assert_eq!(
        last.apply(&Action::SaveFstabDraft {
            client: "old".into()
        })
        .unwrap_err()
        .kind,
        FaultKind::Limit
    );
    assert_eq!(last, before);
    assert_eq!(
        last.apply(&Action::SetFstabDraft {
            client: "old".into(),
            text: document.clone()
        }),
        Ok(Decision::Changed { changed: false })
    );
    assert_eq!(
        last.apply(&Action::SetFstabDraft {
            client: "old".into(),
            text: "\0".into()
        })
        .unwrap_err()
        .kind,
        FaultKind::Limit
    );
    assert_eq!(last, before);
    // The equivalent empty saved field also proves the exact Set-before-copy overflow guard.
    let mut r = last.runtime().clone();
    r.clients[0].draft.clear();
    r.clients[0].saved = document.clone();
    let mut swap = State::new(last.inventory().clone(), r).unwrap();
    let before = swap.clone();
    assert_eq!(
        swap.apply(&Action::SetFstabDraft {
            client: "old".into(),
            text: document.clone()
        })
        .unwrap_err()
        .kind,
        FaultKind::Limit
    );
    assert_eq!(swap, before);
    let mut seed = supplied();
    policy(&mut seed, "/srv/work 192.0.2.20(rw)\n");
    mount(&mut seed, "old", "/srv/work", "/mnt/work", Access::Rw);
    let mut r = seed.runtime().clone();
    for c in &mut r.contents {
        c.text = if c.path == "/srv/work/result.txt" {
            "x".repeat(MAX_FILE)
        } else {
            "\u{1}".repeat(MAX_FILE)
        }
    }
    let seed = State::new(seed.inventory().clone(), r).unwrap();
    let mut last = frontier(&seed, &document, false);
    let before = last.clone();
    assert_eq!(
        last.apply(&Action::WriteExisting {
            client: "old".into(),
            mountpoint: "/mnt/work".into(),
            profile: "writer".into(),
            relative: "result.txt".into(),
            text: "\u{1}".repeat(MAX_FILE)
        })
        .unwrap_err()
        .kind,
        FaultKind::Limit
    );
    assert_eq!(last, before);
    assert_eq!(
        State::from_json(&serde_json::to_string(&last).unwrap()).unwrap(),
        last
    );
}

#[test]
fn fieldless_tagged_variants_keep_wire_kinds_and_reject_ignored_data() {
    fn strict<T>(value: T, kind: &str)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let raw = format!("{{\"kind\":\"{kind}\"}}");
        assert_eq!(serde_json::to_string(&value).unwrap(), raw);
        assert_eq!(serde_json::from_str::<T>(&raw).unwrap(), value);
        for bad in [
            format!("{{\"kind\":\"{kind}\",\"unexpected\":true}}"),
            format!("{{\"unexpected\":{{\"nested\":[]}},\"kind\":\"{kind}\"}}"),
            format!("{{\"kind\":\"{kind}\",\"kind\":\"{kind}\"}}"),
            format!("{{\"kind\":\"{kind}\",\"kind\":\"future\"}}"),
        ] {
            assert!(serde_json::from_str::<T>(&bad).is_err(), "{bad}");
        }
    }
    strict(TrialBase::Applied {}, "applied");
    strict(Expected::Mounted {}, "mounted");
    strict(Expected::Unmounted {}, "unmounted");
    strict(Expected::Write {}, "write");
    strict(Decision::Mounted {}, "mounted");
    strict(Decision::Unmounted {}, "unmounted");

    let plan = one_trial(TrialBase::Applied {}, "writer", Expected::Write {});
    let raw = serde_json::to_string(&plan).unwrap();
    assert_eq!(Evaluation::from_json(&raw).unwrap(), plan);
    for kind in ["applied", "mounted", "write"] {
        let original = format!("{{\"kind\":\"{kind}\"}}");
        for replacement in [
            format!("{{\"kind\":\"{kind}\",\"ignored\":true}}"),
            format!("{{\"kind\":\"{kind}\",\"kind\":\"{kind}\"}}"),
            format!("{{\"kind\":\"{kind}\",\"kind\":\"future\"}}"),
        ] {
            let bad = raw.replacen(&original, &replacement, 1);
            assert_ne!(bad, raw);
            assert!(Evaluation::from_json(&bad).is_err(), "nested {kind}");
        }
    }
}

#[test]
fn unsquashed_root_mode_gates_are_defined_but_permission_paths_stay_unsupported() {
    for (export, client, denied) in [
        ("ro", Access::Ro, Some(DenialStage::ClientReadOnly)),
        ("ro", Access::Rw, Some(DenialStage::ExportReadOnly)),
        ("rw", Access::Ro, Some(DenialStage::ClientReadOnly)),
        ("rw", Access::Rw, None),
    ] {
        let mut s = supplied();
        policy(
            &mut s,
            &format!("/srv/work 192.0.2.20({export},no_root_squash)\n"),
        );
        assert_eq!(
            mount(&mut s, "old", "/srv/work", "/mnt/work", client),
            Decision::Mounted {}
        );
        let before = s.clone();
        let action = Action::WriteExisting {
            client: "old".into(),
            mountpoint: "/mnt/work".into(),
            profile: "root".into(),
            relative: "result.txt".into(),
            text: "must not publish".into(),
        };
        let result = s.apply(&action);
        if let Some(stage) = denied {
            denied_at(&result.unwrap(), stage);
        } else {
            let e = result.unwrap_err();
            assert_eq!(e.kind, FaultKind::SimulatorUnsupported);
            assert!(e.message.contains("UnsquashedRootPrivileges"));
        }
        assert_eq!(s, before);
        let read = Action::ReadExisting {
            client: "old".into(),
            mountpoint: "/mnt/work".into(),
            profile: "root".into(),
            relative: "result.txt".into(),
        };
        assert_eq!(
            s.apply(&read).unwrap_err().kind,
            FaultKind::SimulatorUnsupported
        );
        assert_eq!(s, before);
        let plan = Evaluation {
            current: vec![Request::WritePermission {
                client: "old".into(),
                mountpoint: "/mnt/work".into(),
                profile: "root".into(),
                relative: "result.txt".into(),
            }],
            fresh: vec![Trial {
                base: TrialBase::Applied {},
                steps: vec![
                    TrialStep {
                        action: Action::MountExplicit {
                            client: "old".into(),
                            specification: spec("/srv/work", "/mnt/work", client),
                        },
                        expected: Expected::Mounted {},
                    },
                    TrialStep {
                        action,
                        expected: Expected::Denied {
                            stage: denied.unwrap_or(DenialStage::FilePermission),
                        },
                    },
                ],
            }],
        };
        let mut plan = plan;
        plan.current.push(Request::Read {
            client: "old".into(),
            mountpoint: "/mnt/work".into(),
            profile: "root".into(),
            relative: "result.txt".into(),
        });
        plan.fresh.push(Trial {
            base: TrialBase::Applied {},
            steps: vec![
                TrialStep {
                    action: Action::MountExplicit {
                        client: "old".into(),
                        specification: spec("/srv/work", "/mnt/work", client),
                    },
                    expected: Expected::Mounted {},
                },
                TrialStep {
                    action: read,
                    expected: Expected::Denied {
                        stage: DenialStage::FilePermission,
                    },
                },
            ],
        });
        let report = s.evaluate_batch(&plan).unwrap();
        assert_eq!(report.fresh[0].passed, denied.is_some());
        assert!(!report.fresh[1].passed && !report.fresh[1].steps[1].passed);
        for observation in [&report.current[1], &report.fresh[1].steps[1].observation] {
            assert!(matches!(
                observation,
                Observation::Fault {
                    fault: Fault {
                        kind: FaultKind::SimulatorUnsupported,
                        ..
                    }
                }
            ));
        }
        match denied {
            Some(stage) => {
                let Observation::Decision { decision } = &report.current[0] else {
                    panic!("defined mode denial required")
                };
                denied_at(decision, stage);
            }
            None => {
                for observation in [&report.current[0], &report.fresh[0].steps[1].observation] {
                    assert!(matches!(
                        observation,
                        Observation::Fault {
                            fault: Fault {
                                kind: FaultKind::SimulatorUnsupported,
                                ..
                            }
                        }
                    ));
                }
                assert!(!report.fresh[0].steps[1].passed);
            }
        }
        assert_eq!(s, before);
    }
}
