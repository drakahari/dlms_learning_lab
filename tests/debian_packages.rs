//! Authored logical-model checks. No host dpkg/APT/RPM invocation or actual archive.
use dlms_learning_lab::debian_packages::*;
use serde_json::{Value, json};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};
const DECLARED: &str = include_str!("fixtures/debian-packages-declared.json");
fn fixture() -> State {
    serde_json::from_value(serde_json::from_str::<Value>(DECLARED).unwrap()["state"].clone())
        .unwrap()
}
fn changed(s: &State, edit: impl FnOnce(&mut Catalog, &mut Runtime)) -> State {
    let (mut c, mut r) = (s.catalog().clone(), s.runtime().clone());
    edit(&mut c, &mut r);
    State::new(c, r).unwrap()
}
fn install(n: &str) -> Action {
    Action::Install {
        packages: vec![Request {
            name: n.into(),
            version: None,
        }],
    }
}
fn reject(s: &mut State, a: Action, expected: ErrorCode) {
    let before = serde_json::to_vec(s).unwrap();
    assert_eq!(s.plan(&a).unwrap_err().code, expected);
    assert_eq!(s.apply(&a).unwrap_err().code, expected);
    assert_eq!(serde_json::to_vec(s).unwrap(), before);
}
#[test]
fn policy_authored_version_vectors_and_semantic_equivalence_keep_raw_identity() {
    let declared: Value = serde_json::from_str(DECLARED).unwrap();
    assert!(
        declared["provenance"]["kind"]
            .as_str()
            .unwrap()
            .contains("NOT dpkg")
    );
    for row in declared["version_comparisons"].as_array().unwrap() {
        let (a, b) = (
            Version::parse(row[0].as_str().unwrap()).unwrap(),
            Version::parse(row[1].as_str().unwrap()).unwrap(),
        );
        let expected = match row[2].as_str().unwrap() {
            "Less" => Ordering::Less,
            "Equal" => Ordering::Equal,
            "Greater" => Ordering::Greater,
            _ => panic!(),
        };
        assert_eq!(a.compare(&b), expected, "{row}");
        assert_eq!(b.compare(&a), expected.reverse());
        assert_eq!(a.as_str(), row[0].as_str().unwrap());
    }
    let (a, b) = (Version::parse("1").unwrap(), Version::parse("1-0").unwrap());
    assert_ne!(a, b);
    assert_eq!(a.compare(&b), Ordering::Equal);
    let versions = [
        "0", "0~a", "1.0~~", "1.0~", "1.0", "1.0a", "1.0+", "1.0.0", "2:0",
    ];
    for a in versions {
        for b in versions {
            for c in versions {
                let (a, b, c) = (
                    Version::parse(a).unwrap(),
                    Version::parse(b).unwrap(),
                    Version::parse(c).unwrap(),
                );
                if a.compare(&b).is_le() && b.compare(&c).is_le() {
                    assert!(a.compare(&c).is_le());
                }
            }
        }
    }
}
#[test]
fn unsupported_versions_and_decimal_runs_fail_honestly_without_semver_or_overflow() {
    for value in [
        "",
        "v1",
        "1:",
        "1:",
        "1.0-",
        "1_0",
        "1.0/2",
        "1.0:3",
        "1.0-1-",
        "4294967296:1",
        "1.é",
        "1\n",
    ] {
        assert!(Version::parse(value).is_err(), "{value:?}");
    }
    assert!(Version::parse(&"1".repeat(129)).is_err());
    assert!(
        Version::parse(&"1".repeat(128))
            .unwrap()
            .compare(&Version::parse(&"9".repeat(127)).unwrap())
            .is_gt()
    );
    assert!(Version::parse("4294967295:1").is_ok());
}
#[test]
fn installed_database_owner_local_archive_and_cached_metadata_are_different_queries() {
    let s = fixture();
    let before = serde_json::to_vec(&s).unwrap();
    let owner = s.recorded_owner("/usr/bin/museum-report").unwrap().unwrap();
    assert_eq!(owner.package, "museum-report");
    assert!(owner.present);
    assert_eq!(
        s.payload_issues("museum-report").unwrap(),
        vec![IntegrityIssue {
            path: "/usr/bin/museum-report".into(),
            kind: IntegrityKind::Changed
        }]
    );
    assert_eq!(
        s.archive("report-1").unwrap().files["/usr/bin/museum-report"].contents,
        "report one\n"
    );
    assert!(s.archive("report-2").is_err());
    assert_eq!(
        s.cached_candidate("museum-report")
            .unwrap()
            .unwrap()
            .version
            .as_str(),
        "1.0-1"
    );
    assert!(s.recorded_owner("/work/note.txt").unwrap().is_none());
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
    let missing = changed(&s, |_, r| {
        r.files.remove("/usr/bin/museum-report");
    });
    assert!(
        !missing
            .recorded_owner("/usr/bin/museum-report")
            .unwrap()
            .unwrap()
            .present
    );
    assert_eq!(
        missing.payload_issues("museum-report").unwrap()[0].kind,
        IntegrityKind::Missing
    );
}
#[test]
fn reinstall_repairs_only_ordinary_payload_preserving_config_peers_and_unmanaged_text() {
    let mut s = fixture();
    let original = s.clone();
    let report = s
        .apply(&Action::Reinstall {
            name: "museum-report".into(),
        })
        .unwrap();
    assert_eq!(report.changed_files, vec!["/usr/bin/museum-report"]);
    assert!(report.packages.is_empty());
    let expected = changed(&original, |_, r| {
        r.files
            .insert("/usr/bin/museum-report".into(), "report one\n".into());
    });
    assert_eq!(s, expected);
    assert!(s.payload_issues("museum-report").unwrap().is_empty());
    let before = s.clone();
    s.apply(&Action::Reinstall {
        name: "museum-report".into(),
    })
    .unwrap();
    assert_eq!(s, before);
}
#[test]
fn install_of_same_version_is_not_a_shortcut_for_reinstall_or_database_integrity() {
    let mut s = fixture();
    let old = s.clone();
    s.apply(&install("museum-report")).unwrap();
    assert_eq!(s, old);
    assert!(!s.payload_issues("museum-report").unwrap().is_empty());
}
#[test]
fn refresh_changes_cached_index_only_and_install_resolves_transitive_and_dependencies() {
    let s = fixture();
    let mut empty = State::empty(s.catalog().clone()).unwrap();
    reject(&mut empty, install("museum-report"), ErrorCode::Unavailable);
    let old = empty.runtime().clone();
    empty.apply(&Action::RefreshIndex {}).unwrap();
    assert_eq!(empty.runtime().files, old.files);
    assert_eq!(empty.runtime().records, old.records);
    assert_eq!(empty.runtime().cached.as_deref(), Some("current"));
    let p = empty.plan(&install("museum-report")).unwrap();
    assert_eq!(p.packages.len(), 2);
    assert!(empty.runtime().records.is_empty());
    empty.apply(&install("museum-report")).unwrap();
    assert_eq!(empty.runtime().records["museum-report"].archive, "report-2");
    assert_eq!(empty.runtime().records["museum-lib"].archive, "lib-2");
    assert!(empty.dependency_issues().unwrap().is_empty());
    assert_eq!(
        empty.runtime().local_archives,
        BTreeSet::from(["report-2".into(), "lib-2".into()])
    );
}
#[test]
fn explicit_dependency_order_is_an_equivalent_safe_action_sequence() {
    let c = fixture().catalog().clone();
    let mut a = State::empty(c.clone()).unwrap();
    let mut b = State::empty(c).unwrap();
    for s in [&mut a, &mut b] {
        s.apply(&Action::RefreshIndex {}).unwrap();
    }
    a.apply(&install("museum-report")).unwrap();
    b.apply(&install("museum-lib")).unwrap();
    b.apply(&install("museum-report")).unwrap();
    assert_eq!(a, b);
    let mut combined = State::empty(a.catalog().clone()).unwrap();
    combined.apply(&Action::RefreshIndex {}).unwrap();
    combined
        .apply(&Action::Install {
            packages: vec![
                Request {
                    name: "museum-report".into(),
                    version: Some(Version::parse("2.0-01").unwrap()),
                },
                Request {
                    name: "museum-lib".into(),
                    version: None,
                },
            ],
        })
        .unwrap();
    assert_eq!(a, combined);
}
#[test]
fn cache_can_be_stale_or_queryable_offline_and_local_archives_have_separate_availability() {
    let s = fixture();
    let mut offline = changed(&s, |c, _| c.repository_reachable = false);
    assert_eq!(
        offline
            .cached_candidate("museum-report")
            .unwrap()
            .unwrap()
            .archive,
        "report-1"
    );
    reject(
        &mut offline,
        Action::RefreshIndex {},
        ErrorCode::Unavailable,
    );
    offline
        .apply(&Action::Reinstall {
            name: "museum-report".into(),
        })
        .unwrap();
    let mut missing = changed(&offline, |_, r| {
        r.local_archives.remove("report-1");
    });
    reject(
        &mut missing,
        Action::Reinstall {
            name: "museum-report".into(),
        },
        ErrorCode::Unavailable,
    );
    let mut all = State::empty(s.catalog().clone()).unwrap();
    all.apply(&Action::RefreshIndex {}).unwrap();
    all.apply(&install("museum-common")).unwrap();
    assert_eq!(
        all.catalog().archives["common-1"].architecture,
        Architecture::All
    );
    reject(&mut all, install("museum-foreign"), ErrorCode::Architecture);
}
#[test]
fn unmet_dependencies_are_importable_findings_and_repair_uses_current_applied_state() {
    let base = fixture();
    let mut broken = changed(&base, |_, r| {
        r.records.remove("museum-lib");
        r.files.remove("/usr/lib/museum/library.txt");
    });
    assert_eq!(broken.dependency_issues().unwrap().len(), 2);
    for (relation, accepts) in [
        (Relation::Lt, false),
        (Relation::Le, true),
        (Relation::Eq, true),
        (Relation::Ge, true),
        (Relation::Gt, false),
    ] {
        let tested = changed(&base, |c, _| {
            c.archives.get_mut("viewer-1").unwrap().depends[0].constraint = Some(Constraint {
                relation,
                version: Version::parse("1.0-1").unwrap(),
            });
        });
        assert_eq!(tested.dependency_issues().unwrap().is_empty(), accepts);
    }
    let before = broken.plan(&install("museum-report")).unwrap();
    assert_eq!(before.packages.len(), 1);
    broken.apply(&install("museum-report")).unwrap();
    assert!(broken.dependency_issues().unwrap().is_empty());
    assert!(!broken.payload_issues("museum-report").unwrap().is_empty());
    let mut guarded = changed(&base, |c, _| {
        c.archives.get_mut("viewer-1").unwrap().depends[0].constraint = Some(Constraint {
            relation: Relation::Lt,
            version: Version::parse("2").unwrap(),
        });
    });
    guarded.apply(&Action::RefreshIndex {}).unwrap();
    reject(
        &mut guarded,
        install("museum-report"),
        ErrorCode::Dependency,
    );
}
#[test]
fn low_level_exact_remove_refuses_even_an_already_broken_remaining_dependent() {
    let base = fixture();
    let mut s = base.clone();
    reject(
        &mut s,
        Action::Remove {
            name: "museum-lib".into(),
        },
        ErrorCode::Dependency,
    );
    let mut s = changed(&base, |c, _| {
        c.archives.get_mut("viewer-1").unwrap().depends[0].constraint = Some(Constraint {
            relation: Relation::Ge,
            version: Version::parse("9").unwrap(),
        });
    });
    assert_eq!(s.dependency_issues().unwrap().len(), 1);
    reject(
        &mut s,
        Action::Purge {
            name: "museum-lib".into(),
        },
        ErrorCode::Dependency,
    );
}
#[test]
fn remove_retains_configuration_record_purge_only_remaining_managed_configs() {
    let mut s = fixture();
    let before = s.clone();
    s.apply(&Action::Remove {
        name: "museum-retired".into(),
    })
    .unwrap();
    assert_eq!(
        s.runtime().records["museum-retired"].status,
        Status::ConfigFiles
    );
    assert!(!s.runtime().files.contains_key("/usr/bin/museum-retired"));
    assert_eq!(
        s.runtime().files["/etc/museum/retired.conf"],
        before.runtime().files["/etc/museum/retired.conf"]
    );
    let owner = s
        .recorded_owner("/etc/museum/retired.conf")
        .unwrap()
        .unwrap();
    assert_eq!(owner.status, Status::ConfigFiles);
    assert!(
        s.recorded_owner("/usr/bin/museum-retired")
            .unwrap()
            .is_none()
    );
    s = changed(&s, |_, r| {
        r.files.insert(
            "/usr/bin/museum-retired".into(),
            "new unmanaged note\n".into(),
        );
    });
    let before = s.clone();
    s.apply(&Action::Purge {
        name: "museum-retired".into(),
    })
    .unwrap();
    assert_eq!(
        s.runtime().files["/usr/bin/museum-retired"],
        "new unmanaged note\n"
    );
    assert_eq!(
        s.runtime().files["/work/note.txt"],
        before.runtime().files["/work/note.txt"]
    );
    assert!(!s.runtime().files.contains_key("/etc/museum/retired.conf"));
    assert!(!s.runtime().records.contains_key("museum-retired"));
}
#[test]
fn config_only_does_not_satisfy_depends_and_does_not_claim_ordinary_files() {
    let base = fixture();
    let mut s = changed(&base, |c, r| {
        c.archives.get_mut("viewer-1").unwrap().depends = vec![Dependency {
            name: "museum-report".into(),
            constraint: None,
        }];
        r.records.get_mut("museum-report").unwrap().status = Status::ConfigFiles;
        r.files.remove("/usr/bin/museum-report");
    });
    assert_eq!(s.dependency_issues().unwrap().len(), 1);
    assert!(
        s.recorded_owner("/usr/bin/museum-report")
            .unwrap()
            .is_none()
    );
    assert_eq!(
        s.recorded_owner("/etc/museum/report.conf")
            .unwrap()
            .unwrap()
            .status,
        Status::ConfigFiles
    );
    reject(
        &mut s,
        Action::Reinstall {
            name: "museum-report".into(),
        },
        ErrorCode::Unavailable,
    );
    s.apply(&install("museum-report")).unwrap();
    assert_eq!(
        s.runtime().records["museum-report"].status,
        Status::Installed
    );
    assert!(s.dependency_issues().unwrap().is_empty());
    assert_eq!(
        s.runtime().files["/etc/museum/report.conf"],
        "style=local\n"
    );
}
#[test]
fn conffile_update_rules_preserve_local_edits_deletions_and_reject_unmodeled_choice() {
    let base = fixture();
    let mut s = base.clone();
    s.apply(&Action::RefreshIndex {}).unwrap();
    reject(&mut s, install("museum-report"), ErrorCode::Conffile);
    let mut clean = changed(&base, |_, r| {
        r.files
            .insert("/etc/museum/report.conf".into(), "style=plain\n".into());
    });
    clean.apply(&Action::RefreshIndex {}).unwrap();
    clean.apply(&install("museum-report")).unwrap();
    assert_eq!(
        clean.runtime().files["/etc/museum/report.conf"],
        "style=bright\n"
    );
    let mut deleted = changed(&base, |_, r| {
        r.files.remove("/etc/museum/report.conf");
    });
    deleted
        .apply(&Action::Reinstall {
            name: "museum-report".into(),
        })
        .unwrap();
    assert!(
        !deleted
            .runtime()
            .files
            .contains_key("/etc/museum/report.conf")
    );
    let mut unchanged = changed(&base, |c, _| {
        c.archives
            .get_mut("report-2")
            .unwrap()
            .files
            .get_mut("/etc/museum/report.conf")
            .unwrap()
            .contents = "style=plain\n".into();
    });
    unchanged.apply(&Action::RefreshIndex {}).unwrap();
    unchanged.apply(&install("museum-report")).unwrap();
    assert_eq!(
        unchanged.runtime().files["/etc/museum/report.conf"],
        "style=local\n"
    );
    let mut pathchange = changed(&base, |c, _| {
        c.archives
            .get_mut("report-2")
            .unwrap()
            .files
            .remove("/etc/museum/report.conf");
    });
    pathchange.apply(&Action::RefreshIndex {}).unwrap();
    reject(
        &mut pathchange,
        install("museum-report"),
        ErrorCode::Unsupported,
    );
}
#[test]
fn collisions_architecture_version_and_nonpresent_names_never_publish_partial_results() {
    let c = fixture().catalog().clone();
    let mut s = State::empty(c).unwrap();
    s.apply(&Action::RefreshIndex {}).unwrap();
    s = changed(&s, |_, r| {
        r.files.insert(
            "/usr/bin/museum-report".into(),
            "unowned supplied text\n".into(),
        );
    });
    reject(&mut s, install("museum-report"), ErrorCode::Unsupported);
    reject(&mut s, install("museum-absent"), ErrorCode::Unavailable);
    let mut collision_catalog = fixture().catalog().clone();
    collision_catalog
        .archives
        .get_mut("common-1")
        .unwrap()
        .files = BTreeMap::from([(
        "/usr/bin/museum-report".into(),
        Payload {
            contents: "colliding package".into(),
            conffile: false,
        },
    )]);
    let mut collision = State::empty(collision_catalog).unwrap();
    collision.apply(&Action::RefreshIndex {}).unwrap();
    reject(
        &mut collision,
        Action::Install {
            packages: vec![
                Request {
                    name: "museum-report".into(),
                    version: None,
                },
                Request {
                    name: "museum-common".into(),
                    version: None,
                },
            ],
        },
        ErrorCode::Collision,
    );
    reject(
        &mut s,
        Action::Install {
            packages: vec![Request {
                name: "museum-report".into(),
                version: Some(Version::parse("99").unwrap()),
            }],
        },
        ErrorCode::Unavailable,
    );
    let mut s = fixture();
    s.apply(&Action::Reinstall {
        name: "museum-report".into(),
    })
    .unwrap();
    s.apply(&Action::RefreshIndex {}).unwrap();
    s = changed(&s, |_, r| {
        r.files
            .insert("/etc/museum/report.conf".into(), "style=plain\n".into());
    });
    s.apply(&install("museum-report")).unwrap();
    s = changed(&s, |_, r| r.cached = Some("old".into()));
    reject(&mut s, install("museum-report"), ErrorCode::Unsupported);
}
#[test]
fn plans_are_read_only_explanations_and_cannot_authorize_a_later_changed_state() {
    let mut s = fixture();
    let old = s.clone();
    let action = Action::Reinstall {
        name: "museum-report".into(),
    };
    let plan = s.plan(&action).unwrap();
    assert_eq!(s, old);
    s = changed(&s, |_, r| {
        r.local_archives.remove("report-1");
    });
    s = changed(&s, |c, _| c.repository_reachable = false);
    reject(&mut s, action, ErrorCode::Unavailable);
    assert_eq!(plan.changed_files, vec!["/usr/bin/museum-report"]);
}
#[test]
fn full_context_pin_json_unknown_duplicate_keys_and_shape_errors_fail_closed() {
    let s = fixture();
    let bytes = serde_json::to_vec(&s).unwrap();
    assert_eq!(State::decode(&bytes).unwrap(), s);
    let mut changed = s.catalog().clone();
    changed.repository_reachable = false;
    assert!(s.validate_context(&changed).is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["surprise"] = json!(true);
    assert!(State::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    let duplicate = String::from_utf8(bytes.clone()).unwrap().replacen(
        "\"cached\":\"old\"",
        "\"cached\":\"old\",\"cached\":\"old\"",
        1,
    );
    assert!(State::decode(duplicate.as_bytes()).is_err());
    let duplicate = String::from_utf8(bytes).unwrap().replacen(
        "\"/work/note.txt\":\"keep this note\\n\"",
        "\"/work/note.txt\":\"x\",\"/work/note.txt\":\"y\"",
        1,
    );
    assert!(State::decode(duplicate.as_bytes()).is_err());
    let mut c = s.catalog().clone();
    c.archives.get_mut("report-2").unwrap().version = Version::parse("1.0-01").unwrap();
    assert!(c.validate().is_err());
    let mut r = s.runtime().clone();
    r.records.get_mut("museum-report").unwrap().archive = "foreign-1".into();
    assert!(State::new(s.catalog().clone(), r).is_err());
    let mut r = s.runtime().clone();
    r.records.get_mut("museum-lib").unwrap().status = Status::ConfigFiles;
    assert!(State::new(s.catalog().clone(), r).is_err());
}
#[test]
fn duplicate_ownership_and_paths_outside_finite_universe_are_structural_errors_not_repairs() {
    let s = fixture();
    let mut c = s.catalog().clone();
    c.archives.get_mut("viewer-1").unwrap().files.insert(
        "/usr/bin/museum-report".into(),
        Payload {
            contents: "peer collision".into(),
            conffile: false,
        },
    );
    assert!(State::new(c, s.runtime().clone()).is_err());
    let mut r = s.runtime().clone();
    r.files.insert("/personal/not-supplied".into(), "x".into());
    assert!(State::new(s.catalog().clone(), r).is_err());
    for p in ["relative", "/a//b", "/a/../b", "/a/", "/a\n"] {
        let mut r = s.runtime().clone();
        r.files.insert(p.into(), "x".into());
        assert!(State::new(s.catalog().clone(), r).is_err());
    }
}
#[test]
fn direct_and_streamed_action_counts_scalars_and_aggregate_limits_reject_before_publication() {
    let mut s = fixture();
    reject(
        &mut s,
        Action::Install {
            packages: vec![
                Request {
                    name: "museum-report".into(),
                    version: None
                };
                9
            ],
        },
        ErrorCode::Limit,
    );
    let bad = json!({"kind":"Install","packages":(0..9).map(|_|json!({"name":"museum-report","version":null})).collect::<Vec<_>>()});
    assert!(Action::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert!(Action::decode(br#"{"kind":"Install","packages":[]}"#).is_err());
    assert!(Action::decode(br#"{"kind":"RefreshIndex","command":"host tool"}"#).is_err());
    assert!(Action::decode(br#"{"kind":"RefreshIndex"}"#).is_ok());
    assert!(
        Action::decode(br#"{"kind":"RefreshIndex","kind":"Purge","name":"museum-report"}"#)
            .is_err()
    );
    for mut action in [
        json!({"kind":"RefreshIndex"}),
        json!({"kind":"Install","packages":[{"name":"museum-report","version":null}]}),
        json!({"kind":"Reinstall","name":"museum-report"}),
        json!({"kind":"Remove","name":"museum-report"}),
        json!({"kind":"Purge","name":"museum-report"}),
    ] {
        assert!(Action::decode(&serde_json::to_vec(&action).unwrap()).is_ok());
        action["unexpected"] = json!("unsupported input");
        assert!(Action::decode(&serde_json::to_vec(&action).unwrap()).is_err());
    }
    assert!(Action::decode(&vec![b' '; 16_385]).is_err());
    assert!(State::decode(&vec![b' '; MAX_JSON + 1]).is_err());
    let mut c = s.catalog().clone();
    let item = c.archives["report-1"].clone();
    for n in 0..MAX_ARCHIVES + 1 {
        c.archives.insert(format!("extra-{n}"), item.clone());
    }
    assert!(c.validate().is_err());
    let mut c = s.catalog().clone();
    c.archives.get_mut("report-1").unwrap().depends = vec![
        Dependency {
            name: "museum-lib".into(),
            constraint: None
        };
        9
    ];
    assert!(c.validate().is_err());
    let mut c = s.catalog().clone();
    c.archives
        .get_mut("report-1")
        .unwrap()
        .files
        .get_mut("/usr/bin/museum-report")
        .unwrap()
        .contents = "é".repeat(1025);
    assert!(c.validate().is_err());
    let mut c = s.catalog().clone();
    c.publications
        .get_mut("old")
        .unwrap()
        .candidates
        .insert("museum-report".into(), "x".repeat(65));
    assert!(c.validate().is_err());
}
#[test]
fn maximum_escaped_admitted_state_roundtrips_compact_and_pretty_without_truncation() {
    let mut archives = BTreeMap::new();
    let mut candidates = BTreeMap::new();
    let mut records = BTreeMap::new();
    let mut files = BTreeMap::new();
    let mut row = 0usize;
    for n in 0..24 {
        for v in 1..=3 {
            let key = format!("archive-{n}-{v}");
            let package = format!("package-{n}");
            let mut payload = BTreeMap::new();
            let count = if n * 3 + v - 1 < 40 { 4 } else { 3 };
            for _ in 0..count {
                let p = format!("/virtual/file-{row}");
                row += 1;
                payload.insert(
                    p.clone(),
                    Payload {
                        contents: "\u{1}".repeat(1024),
                        conffile: false,
                    },
                );
                files.insert(p, "\u{2}".repeat(1023));
            }
            archives.insert(
                key.clone(),
                Archive {
                    name: package.clone(),
                    version: Version::parse(&format!("{v}.0")).unwrap(),
                    architecture: Architecture::All,
                    depends: vec![],
                    files: payload,
                },
            );
            if v == 1 {
                candidates.insert(package.clone(), key.clone());
                records.insert(
                    package,
                    PackageRecord {
                        archive: key,
                        status: Status::Installed,
                    },
                );
            }
        }
    }
    assert_eq!(row, 256);
    let mut extra_paths = BTreeSet::new();
    for n in 0..32 {
        let p = format!("/extra/file-{n}");
        extra_paths.insert(p.clone());
        files.insert(p, "\u{3}".repeat(8));
    }
    let c = Catalog {
        architecture: Architecture::Amd64,
        repository_reachable: true,
        published: "current".into(),
        archives,
        publications: BTreeMap::from([("current".into(), Index { candidates })]),
        extra_paths,
    };
    let r = Runtime {
        cached: Some("current".into()),
        local_archives: c.archives.keys().cloned().collect(),
        records,
        files,
    };
    let s = State::new(c, r).unwrap();
    assert_eq!(s.runtime().files.len(), MAX_FILES);
    for b in [
        serde_json::to_vec(&s).unwrap(),
        serde_json::to_vec_pretty(&s).unwrap(),
    ] {
        assert!(b.len() <= MAX_JSON);
        let decoded = State::decode(&b).unwrap();
        assert_eq!(decoded, s);
        assert_eq!(
            serde_json::to_vec(&decoded).unwrap(),
            serde_json::to_vec(&s).unwrap()
        );
    }
    let mut bad = s.runtime().clone();
    bad.files.get_mut("/extra/file-0").unwrap().push('x');
    assert_eq!(
        State::new(s.catalog().clone(), bad).unwrap_err().code,
        ErrorCode::Limit
    );
}
