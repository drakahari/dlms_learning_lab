//! Documentation-derived prospective checks. No host RPM/DNF invocations.
use dlms_learning_lab::rpm_packages::*;
use std::{cmp::Ordering, collections::BTreeMap};
fn evr(s: &str) -> Evr {
    Evr::parse(s).unwrap()
}
fn id(name: &str, arch: Architecture) -> Identity {
    Identity {
        name: name.into(),
        arch,
    }
}
fn file(s: &str) -> File {
    File {
        contents: s.into(),
        mode: 0o644,
        owner: "root".into(),
        group: "root".into(),
        mtime: 7,
    }
}
fn archive(name: &str, version: &str, path: &str) -> Archive {
    Archive {
        identity: id(name, Architecture::X86_64),
        evr: evr(version),
        requires: Vec::new(),
        provides: BTreeMap::new(),
        files: BTreeMap::from([(
            path.into(),
            Payload {
                file: file(name),
                kind: FileKind::Ordinary,
            },
        )]),
    }
}
fn catalog() -> Catalog {
    let mut consumer = archive("viewer", "1.0-1", "/usr/share/viewer.txt");
    consumer.requires = vec![Requirement {
        capability: "codec-api".into(),
        constraint: Some(Constraint {
            relation: Relation::Ge,
            evr: evr("1.0-1"),
        }),
    }];
    let mut fast = archive("codec-fast", "1.0-1", "/usr/share/fast.txt");
    fast.provides.insert("codec-api".into(), Some(evr("1.2-1")));
    let mut safe = archive("codec-safe", "7.0-1", "/usr/share/safe.txt");
    safe.provides.insert("codec-api".into(), Some(evr("1.1-2")));
    let mut unversioned = archive("codec-label", "9.0-1", "/usr/share/label.txt");
    unversioned.provides.insert("codec-api".into(), None);
    let mut report = archive("report", "1.0-1", "/usr/share/report.txt");
    report.files.insert(
        "/etc/report.conf".into(),
        Payload {
            file: file("quiet=true\n"),
            kind: FileKind::ConfigNoreplace,
        },
    );
    let mut newer = report.clone();
    newer.evr = evr("1:0.1^git1-1");
    newer
        .files
        .get_mut("/usr/share/report.txt")
        .unwrap()
        .file
        .contents = "new report".into();
    newer
        .files
        .get_mut("/etc/report.conf")
        .unwrap()
        .file
        .contents = "quiet=false\n".into();
    let mut noarch = report.clone();
    noarch.identity.arch = Architecture::Noarch;
    noarch.evr = evr("99-1");
    let archives = BTreeMap::from([
        ("viewer".into(), consumer),
        ("fast".into(), fast),
        ("safe".into(), safe),
        ("label".into(), unversioned),
        ("report-old".into(), report),
        ("report-new".into(), newer),
        ("report-noarch".into(), noarch),
    ]);
    let mut paths = archives
        .values()
        .flat_map(|a| a.files.keys().cloned())
        .collect::<Vec<_>>();
    paths.push("/notes.txt".into());
    paths.sort();
    paths.dedup();
    Catalog {
        fixture: "rpm-controlled-world".into(),
        target: Architecture::X86_64,
        published: SnapshotId::new("current").unwrap(),
        archives,
        snapshots: BTreeMap::from([
            (
                "stale".into(),
                Snapshot {
                    archives: vec!["label".into()],
                },
            ),
            (
                "current".into(),
                Snapshot {
                    archives: vec![
                        "fast".into(),
                        "safe".into(),
                        "viewer".into(),
                        "report-new".into(),
                    ],
                },
            ),
        ]),
        allowed_paths: paths,
    }
}
fn empty(c: Catalog) -> State {
    let available = c.archives.keys().cloned().collect();
    State::new(
        c,
        Runtime {
            installed: Vec::new(),
            available,
            cached: Some(SnapshotId::new("stale").unwrap()),
            files: BTreeMap::from([("/notes.txt".into(), file("keep this unrelated note"))]),
        },
    )
    .unwrap()
}
fn initial() -> State {
    empty(catalog())
}
fn install(s: &mut State, ids: &[&str]) {
    s.apply(&Action::InstallSet {
        archives: ids.iter().map(|s| (*s).into()).collect(),
    })
    .unwrap();
}
fn exact(a: &impl serde::Serialize, b: &impl serde::Serialize) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "Complete value mismatch"
    );
}
fn query() -> Requirement {
    Requirement {
        capability: "codec-api".into(),
        constraint: Some(Constraint {
            relation: Relation::Ge,
            evr: evr("1.0-1"),
        }),
    }
}
#[test]
fn independently_authored_evr_vectors_and_order_laws() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/rpm-package-vectors.json")).unwrap();
    let mut values = Vec::new();
    for row in v["cases"].as_array().unwrap() {
        let a = evr(row[0].as_str().unwrap());
        let b = evr(row[1].as_str().unwrap());
        let expected = match row[2].as_i64().unwrap() {
            -1 => Ordering::Less,
            0 => Ordering::Equal,
            1 => Ordering::Greater,
            _ => panic!(),
        };
        assert_eq!(a.compare(&b), expected);
        values.extend([a, b]);
    }
    for a in &values {
        for b in &values {
            assert_eq!(a.compare(b), b.compare(a).reverse());
            for c in &values {
                if a.compare(b) != Ordering::Greater && b.compare(c) != Ordering::Greater {
                    assert_ne!(a.compare(c), Ordering::Greater);
                }
            }
        }
    }
    assert_ne!(evr("1.0-1"), evr("1+0-1")); // Text identity is not ordering equivalence.
}
#[test]
fn evr_rejects_unmodeled_or_malformed_syntax() {
    for s in [
        "",
        "1",
        "1-",
        "-1",
        "1-1-1",
        "a:1-1",
        "4294967296:1-1",
        "1:1:2-1",
        "1.0/1-1",
        "1-..",
        "1-β",
    ] {
        assert!(Evr::parse(s).is_err(), "{s}");
    }
    assert_eq!(Evr::parse(&"1".repeat(129)).unwrap_err().code, Code::Limit);
}
#[test]
fn provider_names_and_sources_are_separate_and_queries_do_not_mutate() {
    let mut s = initial();
    let before = s.clone();
    assert!(s.installed_providers(&query()).unwrap().is_empty());
    assert!(s.available_providers(&query()).unwrap().is_empty());
    assert_eq!(s.local_archive("fast").unwrap().identity.name, "codec-fast");
    exact(&s, &before);
    install(&mut s, &["fast", "viewer"]);
    assert_eq!(
        s.installed_providers(&query()).unwrap()[0].identity.name,
        "codec-fast"
    );
    assert!(s.available_providers(&query()).unwrap().is_empty());
    let mut runtime = s.runtime().clone();
    runtime.cached = Some(SnapshotId::new("current").unwrap());
    let cached = State::new(s.catalog().clone(), runtime).unwrap();
    assert_eq!(cached.available_providers(&query()).unwrap().len(), 2);
    assert_eq!(cached.runtime().installed, s.runtime().installed);
}
#[test]
fn explicit_alternative_provider_and_reverse_set_order_both_work() {
    for ids in [["fast", "viewer"], ["viewer", "safe"]] {
        let mut s = initial();
        install(&mut s, &ids);
        assert!(s.dependency_issues().unwrap().is_empty());
        assert!(
            s.file_issues(&id("viewer", Architecture::X86_64))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            s.runtime().files["/notes.txt"],
            file("keep this unrelated note")
        );
    }
}
#[test]
fn unversioned_provide_cannot_satisfy_versioned_requirement() {
    let mut s = initial();
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::InstallSet {
            archives: vec!["label".into(), "viewer".into()]
        })
        .unwrap_err()
        .code,
        Code::UnsupportedSubset
    );
    exact(&s, &before);
    install(&mut s, &["label"]);
    let q = Requirement {
        capability: "codec-api".into(),
        constraint: None,
    };
    assert_eq!(s.installed_providers(&q).unwrap().len(), 1);
    assert!(s.installed_providers(&query()).unwrap().is_empty());
}
#[test]
fn installation_is_not_upgrade_or_implicit_download() {
    let mut s = initial();
    install(&mut s, &["report-old"]);
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::InstallSet {
            archives: vec!["report-new".into()]
        })
        .unwrap_err()
        .code,
        Code::AlreadyInstalled
    );
    exact(&s, &before);
    let mut runtime = s.runtime().clone();
    runtime.available.retain(|a| a != "report-new");
    let mut unavailable = State::new(s.catalog().clone(), runtime).unwrap();
    assert_eq!(
        unavailable
            .apply(&Action::UpgradeSet {
                archives: vec!["report-new".into()]
            })
            .unwrap_err()
            .code,
        Code::Unavailable
    );
    s.apply(&Action::UpgradeSet {
        archives: vec!["report-new".into()],
    })
    .unwrap();
    assert_eq!(
        s.installed(&id("report", Architecture::X86_64))
            .unwrap()
            .unwrap()
            .evr,
        evr("1:0.1^git1-1")
    );
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::UpgradeSet {
            archives: vec!["report-old".into()]
        })
        .unwrap_err()
        .code,
        Code::UnsupportedSubset
    );
    exact(&s, &before);
}
#[test]
fn ownership_is_not_intact_payload_and_exact_reinstall_repairs() {
    let mut s = initial();
    install(&mut s, &["report-old"]);
    let mut r = s.runtime().clone();
    r.files.get_mut("/usr/share/report.txt").unwrap().contents = "damaged bytes".into();
    r.files.get_mut("/usr/share/report.txt").unwrap().mode = 0o600;
    let mut broken = State::new(s.catalog().clone(), r).unwrap();
    assert_eq!(
        broken.recorded_owners("/usr/share/report.txt").unwrap(),
        vec![id("report", Architecture::X86_64)]
    );
    let issues = broken
        .file_issues(&id("report", Architecture::X86_64))
        .unwrap();
    assert_eq!(
        issues[0].differences,
        vec![Difference::Contents, Difference::Mode]
    );
    broken
        .apply(&Action::ReinstallExact {
            archive: "report-old".into(),
        })
        .unwrap();
    assert!(
        broken
            .file_issues(&id("report", Architecture::X86_64))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn missing_payload_remains_recorded_and_verification_is_not_signature_trust() {
    let mut s = initial();
    install(&mut s, &["fast"]);
    let mut r = s.runtime().clone();
    r.files.remove("/usr/share/fast.txt");
    let broken = State::new(s.catalog().clone(), r).unwrap();
    assert_eq!(
        broken.recorded_owners("/usr/share/fast.txt").unwrap().len(),
        1
    );
    assert_eq!(
        broken
            .file_issues(&id("codec-fast", Architecture::X86_64))
            .unwrap()[0]
            .differences,
        vec![Difference::Missing]
    );
}
#[test]
fn modified_configuration_and_manifest_changes_are_honestly_unsupported() {
    let mut s = initial();
    install(&mut s, &["report-old"]);
    let mut r = s.runtime().clone();
    r.files.get_mut("/etc/report.conf").unwrap().contents = "custom=true\n".into();
    let mut modified = State::new(s.catalog().clone(), r).unwrap();
    for action in [
        Action::UpgradeSet {
            archives: vec!["report-new".into()],
        },
        Action::ReinstallExact {
            archive: "report-old".into(),
        },
        Action::EraseSet {
            packages: vec![id("report", Architecture::X86_64)],
        },
    ] {
        let before = modified.clone();
        assert_eq!(
            modified.apply(&action).unwrap_err().code,
            Code::UnsupportedSubset
        );
        exact(&modified, &before);
    }
    let mut c = s.catalog().clone();
    c.archives
        .get_mut("report-new")
        .unwrap()
        .files
        .remove("/etc/report.conf");
    let mut different = State::new(c, s.runtime().clone()).unwrap();
    assert_eq!(
        different
            .apply(&Action::UpgradeSet {
                archives: vec!["report-new".into()]
            })
            .unwrap_err()
            .code,
        Code::UnsupportedSubset
    );
}
#[test]
fn erase_set_never_strands_dependents_and_preserves_neighbors() {
    let mut s = initial();
    install(&mut s, &["fast", "viewer", "report-old"]);
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::EraseSet {
            packages: vec![id("codec-fast", Architecture::X86_64)]
        })
        .unwrap_err()
        .code,
        Code::UnsupportedSubset
    );
    exact(&s, &before);
    s.apply(&Action::EraseSet {
        packages: vec![
            id("codec-fast", Architecture::X86_64),
            id("viewer", Architecture::X86_64),
        ],
    })
    .unwrap();
    assert!(
        s.installed(&id("report", Architecture::X86_64))
            .unwrap()
            .is_some()
    );
    assert_eq!(
        s.runtime().files["/notes.txt"],
        file("keep this unrelated note")
    );
    s.apply(&Action::EraseSet {
        packages: vec![id("report", Architecture::X86_64)],
    })
    .unwrap();
    assert!(!s.runtime().files.contains_key("/etc/report.conf"));
}
#[test]
fn unrelated_broken_database_is_admitted_but_logical_planner_restriction_is_explicit() {
    let s = initial();
    let mut r = s.runtime().clone();
    r.installed.push(Installed {
        identity: id("viewer", Architecture::X86_64),
        archive: "viewer".into(),
    });
    let mut broken = State::new(s.catalog().clone(), r).unwrap();
    assert_eq!(broken.dependency_issues().unwrap().len(), 1);
    let before = broken.clone();
    let err = broken
        .apply(&Action::InstallSet {
            archives: vec!["report-old".into()],
        })
        .unwrap_err();
    assert_eq!(err.code, Code::UnsupportedSubset);
    assert!(err.message.contains("not a universal RPM"));
    exact(&broken, &before);
    install(&mut broken, &["safe"]);
    assert!(broken.dependency_issues().unwrap().is_empty());
}
#[test]
fn noarch_and_x86_64_are_catalog_choices_not_multilib_or_auto_replacement() {
    let mut s = initial();
    install(&mut s, &["report-old"]);
    for action in [
        Action::InstallSet {
            archives: vec!["report-noarch".into()],
        },
        Action::UpgradeSet {
            archives: vec!["report-noarch".into()],
        },
    ] {
        let before = s.clone();
        assert_eq!(s.apply(&action).unwrap_err().code, Code::UnsupportedSubset);
        exact(&s, &before);
    }
    let mut r = s.runtime().clone();
    r.installed.push(Installed {
        identity: id("report", Architecture::Noarch),
        archive: "report-noarch".into(),
    });
    assert_eq!(
        State::new(s.catalog().clone(), r).unwrap_err().code,
        Code::UnsupportedSubset
    );
}
#[test]
fn immutable_catalog_and_ambiguous_equivalent_evr_are_rejected() {
    let s = initial();
    let mut c = s.catalog().clone();
    c.fixture = "another-world".into();
    assert_eq!(s.validate_context(&c).unwrap_err().code, Code::Context);
    let mut c = s.catalog().clone();
    let mut a = c.archives["fast"].clone();
    a.evr = evr("1+0-1");
    c.archives.insert("equivalent".into(), a);
    assert!(c.validate().is_err());
}
#[test]
fn ownership_conflict_and_unmanaged_adoption_are_atomic() {
    let mut s = initial();
    let mut c = s.catalog().clone();
    let conflicting = c.archives["fast"].files.clone();
    c.archives.get_mut("safe").unwrap().files = conflicting;
    s = State::new(c, s.runtime().clone()).unwrap();
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::InstallSet {
            archives: vec!["fast".into(), "safe".into()]
        })
        .unwrap_err()
        .code,
        Code::Ownership
    );
    exact(&s, &before);
    let mut r = s.runtime().clone();
    r.files
        .insert("/usr/share/fast.txt".into(), file("unmanaged"));
    let mut s = State::new(s.catalog().clone(), r).unwrap();
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::InstallSet {
            archives: vec!["fast".into()]
        })
        .unwrap_err()
        .code,
        Code::UnsupportedSubset
    );
    exact(&s, &before);
}
// Independent serialized-tree accounting, not the implementation's schema walk.
fn tree(v: &serde_json::Value) -> (usize, usize, usize) {
    match v {
        serde_json::Value::String(s) => (s.len(), 1, 1),
        serde_json::Value::Array(a) => a.iter().fold((0, 1, 1), |(s, n, d), v| {
            let (x, y, z) = tree(v);
            (s + x, n + y, d.max(z + 1))
        }),
        serde_json::Value::Object(a) => a.iter().fold((0, 1, 1), |(s, n, d), (k, v)| {
            let (x, y, z) = tree(v);
            (s + k.len() + x, n + 1 + y, d.max(z + 1))
        }),
        _ => (0, 1, 1),
    }
}
#[test]
fn every_wire_string_and_token_is_charged_including_repeated_sources() {
    let s = initial();
    let v = serde_json::to_value(&s).unwrap();
    let (strings, tokens, depth) = tree(&v);
    assert_eq!(s.budget().unwrap(), Budget { strings, tokens });
    assert!(depth <= 16);
    assert!(serde_json::to_vec(&s).unwrap().len() <= 6 * strings + 26 * tokens);
    let mut s = s;
    install(&mut s, &["report-old"]);
    let v = serde_json::to_value(&s).unwrap();
    let (a, b, _) = tree(&v);
    assert_eq!(
        s.budget().unwrap(),
        Budget {
            strings: a,
            tokens: b
        }
    );
}
#[test]
fn worst_escaped_combined_payload_fits_and_next_individually_legal_file_exceeds() {
    let mut c = catalog();
    c.archives.clear();
    c.snapshots = BTreeMap::from([(
        "current".into(),
        Snapshot {
            archives: vec!["large".into()],
        },
    )]);
    c.allowed_paths = Vec::new();
    let mut a = archive("large", "1-1", "/data/0");
    a.files.clear();
    for n in 0..15 {
        let p = format!("/data/{n}");
        c.allowed_paths.push(p.clone());
        a.files.insert(
            p,
            Payload {
                file: file(&"\0".repeat(4096)),
                kind: FileKind::Ordinary,
            },
        );
    }
    c.archives.insert("large".into(), a);
    let mut s = State::new(
        c.clone(),
        Runtime {
            installed: Vec::new(),
            available: vec!["large".into()],
            cached: None,
            files: BTreeMap::new(),
        },
    )
    .unwrap();
    install(&mut s, &["large"]);
    let compact = serde_json::to_vec(&s).unwrap();
    let pretty = serde_json::to_vec_pretty(&s).unwrap();
    let b = s.budget().unwrap();
    assert!(b.strings <= MAX_STRINGS && b.strings > 120000);
    assert!(compact.len() < MAX_COMPACT && pretty.len() < MAX_RAW_STATE);
    exact(&State::decode(&pretty).unwrap(), &s);
    c.allowed_paths.push("/data/15".into());
    c.archives.get_mut("large").unwrap().files.insert(
        "/data/15".into(),
        Payload {
            file: file(&"\0".repeat(4096)),
            kind: FileKind::Ordinary,
        },
    );
    let mut s = State::new(
        c,
        Runtime {
            installed: Vec::new(),
            available: vec!["large".into()],
            cached: None,
            files: BTreeMap::new(),
        },
    )
    .unwrap();
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::InstallSet {
            archives: vec!["large".into()]
        })
        .unwrap_err()
        .code,
        Code::Limit
    );
    exact(&s, &before);
}
#[test]
fn borrowed_scalar_and_count_limits_reject_direct_values_without_cloning_plans() {
    let s = initial();
    let mut c = s.catalog().clone();
    c.archives
        .get_mut("fast")
        .unwrap()
        .files
        .get_mut("/usr/share/fast.txt")
        .unwrap()
        .file
        .contents = "x".repeat(4097);
    assert_eq!(c.validate().unwrap_err().code, Code::Limit);
    let mut r = s.runtime().clone();
    r.installed = vec![
        Installed {
            identity: id("viewer", Architecture::X86_64),
            archive: "viewer".into()
        };
        25
    ];
    assert_eq!(
        State::new(s.catalog().clone(), r).unwrap_err().code,
        Code::Limit
    );
    let mut s = s;
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::InstallSet {
            archives: vec!["x".repeat(1_000_000)]
        })
        .unwrap_err()
        .code,
        Code::Limit
    );
    exact(&s, &before);
    assert_eq!(
        s.installed(&id(&"x".repeat(1_000_000), Architecture::X86_64))
            .unwrap_err()
            .code,
        Code::Limit
    );
}
#[test]
fn strict_schema_duplicate_keys_and_raw_caps_fail_closed() {
    let s = initial();
    let raw = serde_json::to_vec(&s).unwrap();
    exact(&State::decode(&raw).unwrap(), &s);
    for action in [
        Action::InstallSet {
            archives: vec!["fast".into()],
        },
        Action::UpgradeSet {
            archives: vec!["fast".into()],
        },
        Action::ReinstallExact {
            archive: "fast".into(),
        },
        Action::EraseSet {
            packages: vec![id("codec-fast", Architecture::X86_64)],
        },
    ] {
        let mut v = serde_json::to_value(action).unwrap();
        v.as_object_mut()
            .unwrap()
            .insert("unknown".into(), true.into());
        assert!(Action::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    assert!(
        Action::decode(br#"{"kind":"InstallSet","kind":"EraseSet","archives":["fast"]}"#).is_err()
    );
    let mut v = serde_json::to_value(&s).unwrap();
    v["schema_version"] = 2.into();
    assert!(State::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    assert_eq!(
        State::decode(&vec![b' '; MAX_RAW_STATE + 1])
            .unwrap_err()
            .code,
        Code::Limit
    );
    assert_eq!(
        Action::decode(&vec![b' '; MAX_ACTION + 1])
            .unwrap_err()
            .code,
        Code::Limit
    );
    let raw = serde_json::to_string(&s).unwrap();
    let duplicate = raw.replacen(
        "\"fixture\":\"rpm-controlled-world\"",
        "\"fixture\":\"rpm-controlled-world\",\"fixture\":\"different\"",
        1,
    );
    assert_ne!(duplicate, raw);
    assert!(State::decode(duplicate.as_bytes()).is_err());
}

fn many_archives(files: usize) -> Catalog {
    let mut c = catalog();
    c.archives.clear();
    c.snapshots.clear();
    c.allowed_paths = (0..files).map(|n| format!("/v/{n}")).collect();
    for n in 0..24 {
        for release in 0..3 {
            let mut a = archive(&format!("p{n:02}"), &format!("1-{release}"), "/v/0");
            a.files = (0..files)
                .map(|n| {
                    (
                        format!("/v/{n}"),
                        Payload {
                            file: file(""),
                            kind: FileKind::Ordinary,
                        },
                    )
                })
                .collect();
            c.archives.insert(format!("p{n:02}-v{release}"), a);
        }
    }
    c.snapshots.insert(
        "current".into(),
        Snapshot {
            archives: c.archives.keys().cloned().collect(),
        },
    );
    c
}
#[test]
fn combined_token_bound_is_independent_of_text_bytes() {
    // Independent static wire math:72 archives, each extra file adds1152
    // value/key tokens plus one allowed-path token. Six files leave19 tokens
    // below the cap; seven cross it while all strings remain well below128KiB.
    let c = many_archives(6);
    let r = Runtime {
        installed: Vec::new(),
        available: Vec::new(),
        cached: None,
        files: BTreeMap::new(),
    };
    let s = State::new(c, r.clone()).unwrap();
    let (strings, tokens, depth) = tree(&serde_json::to_value(&s).unwrap());
    assert_eq!((strings, tokens, depth), (28549, 8173, 8));
    assert!(
        tokens > MAX_TOKENS - 1500 && tokens <= MAX_TOKENS && strings < MAX_STRINGS && depth <= 16,
        "Valid token-only control: strings={strings}, tokens={tokens}, depth={depth}"
    );
    assert_eq!(s.budget().unwrap(), Budget { strings, tokens });
    let c = many_archives(7);
    let v = serde_json::json!({"schema_version":1,"catalog":c,"runtime":r});
    let (strings, tokens, depth) = tree(&v);
    assert_eq!((strings, tokens, depth), (32513, 9326, 8));
    assert!(
        tokens > MAX_TOKENS && strings < MAX_STRINGS,
        "Token limit independently exceeded: strings={strings}, tokens={tokens}, depth={depth}"
    );
    assert!(State::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn query_identifiers_readonly_plan_and_config_errors_leave_exact_callers() {
    let mut s = initial();
    let before = s.clone();
    let a = Action::InstallSet {
        archives: vec!["viewer".into(), "safe".into()],
    };
    let plan = s.plan(&a).unwrap();
    assert_eq!(plan.changes.len(), 2);
    exact(&s, &before);
    s.apply(&a).unwrap();
    assert_eq!(
        s.installed_providers(&Requirement {
            capability: "viewer".into(),
            constraint: Some(Constraint {
                relation: Relation::Eq,
                evr: evr("1.0-1")
            })
        })
        .unwrap()
        .len(),
        1
    );
    assert!(s.recorded_owners("../outside").is_err());
    let mut noarch = initial();
    install(&mut noarch, &["report-noarch"]);
    assert!(
        noarch
            .installed(&id("report", Architecture::Noarch))
            .unwrap()
            .is_some()
    );
}
#[test]
fn reinstall_does_not_skip_unrelated_logical_database_health_guard() {
    let mut s = initial();
    install(&mut s, &["report-old"]);
    let mut r = s.runtime().clone();
    r.installed.push(Installed {
        identity: id("viewer", Architecture::X86_64),
        archive: "viewer".into(),
    });
    let mut broken = State::new(s.catalog().clone(), r).unwrap();
    let before = broken.clone();
    let error = broken
        .apply(&Action::ReinstallExact {
            archive: "report-old".into(),
        })
        .unwrap_err();
    assert_eq!(error.code, Code::UnsupportedSubset);
    assert!(error.message.contains("whole-database"));
    exact(&broken, &before);
}
#[test]
fn strict_typed_map_count_stops_before_overflow_value_and_unknown_arch_rejects() {
    let c = many_archives(0);
    let mut v = serde_json::to_value(&c).unwrap();
    v["archives"].as_object_mut().unwrap().insert(
        "zz-overflow".into(),
        serde_json::json!({"deliberately_wrong_body":true}),
    );
    let error = serde_json::from_value::<Catalog>(v)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("count"),
        "Reject73rd map value before parsing its malformed body"
    );
    let mut v = serde_json::to_value(initial()).unwrap();
    v["catalog"]["target"] = "aarch64".into();
    assert!(State::decode(&serde_json::to_vec(&v).unwrap()).is_err());
}

#[test]
fn maximum_legal_scalar_lengths_and_numeric_metadata_match_the_wire_budget() {
    let name = format!("p{}", "a".repeat(31));
    let archive_id = format!("a{}", "b".repeat(31));
    let p = format!("/p/{}", "x".repeat(157));
    let version = format!("4294967295:1{}-1", "a".repeat(114));
    assert_eq!(version.len(), 128);
    assert_eq!(p.len(), 160);
    let mut a = archive(&name, &version, &p);
    let f = &mut a.files.get_mut(&p).unwrap().file;
    f.owner = "a".repeat(32);
    f.group = "g".repeat(32);
    f.mtime = u64::MAX;
    f.mode = 0o777;
    let c = Catalog {
        fixture: format!("f{}", "x".repeat(31)),
        target: Architecture::X86_64,
        published: SnapshotId::new("current").unwrap(),
        archives: BTreeMap::from([(archive_id.clone(), a)]),
        snapshots: BTreeMap::from([(
            "current".into(),
            Snapshot {
                archives: vec![archive_id.clone()],
            },
        )]),
        allowed_paths: vec![p],
    };
    let mut s = State::new(
        c,
        Runtime {
            installed: Vec::new(),
            available: vec![archive_id.clone()],
            cached: None,
            files: BTreeMap::new(),
        },
    )
    .unwrap();
    s.apply(&Action::InstallSet {
        archives: vec![archive_id],
    })
    .unwrap();
    let v = serde_json::to_value(&s).unwrap();
    let (strings, tokens, depth) = tree(&v);
    assert_eq!(s.budget().unwrap(), Budget { strings, tokens });
    assert!(depth <= 16);
    assert!(serde_json::to_vec(&s).unwrap().len() <= 6 * strings + 26 * tokens);
    let before = s.clone();
    assert_eq!(
        s.apply(&Action::EraseSet {
            packages: vec![id("a", Architecture::X86_64); 9]
        })
        .unwrap_err()
        .code,
        Code::Limit
    );
    exact(&s, &before);
}
