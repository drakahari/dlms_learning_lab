//! Authored package-policy vectors. No host dpkg/APT or archive execution.
use dlms_learning_lab::{
    debian_packages as p, debian_packages_terminal as terminal, debian_packages_workspace as work,
};
use std::collections::{BTreeMap, BTreeSet};
fn archive(name: &str, version: &str) -> p::Archive {
    p::Archive {
        name: name.into(),
        version: p::Version::parse(version).unwrap(),
        architecture: p::Architecture::Amd64,
        depends: vec![],
        conflicts: vec![],
        files: BTreeMap::from([(
            format!("/usr/bin/{name}"),
            p::Payload {
                contents: format!("{name} {version}\n"),
                conffile: false,
            },
        )]),
    }
}
fn relation(constraint: Option<p::Constraint>) -> p::Dependency {
    p::Dependency {
        name: "legacy-agent".into(),
        constraint,
    }
}
fn before_two() -> Option<p::Constraint> {
    Some(p::Constraint {
        relation: p::Relation::Lt,
        version: p::Version::parse("2.0").unwrap(),
    })
}
fn sample(constraint: Option<p::Constraint>) -> p::State {
    let mut old = archive("legacy-agent", "1.0");
    old.files.insert(
        "/etc/legacy.conf".into(),
        p::Payload {
            contents: "mode=steady\n".into(),
            conffile: true,
        },
    );
    let mut next = archive("legacy-agent", "2.0");
    next.files.insert(
        "/etc/legacy.conf".into(),
        old.files["/etc/legacy.conf"].clone(),
    );
    let mut new = archive("new-app", "1.0");
    new.conflicts.push(relation(constraint));
    let keeper = archive("keeper", "1.0");
    let archives = BTreeMap::from([
        ("legacy-1".into(), old),
        ("legacy-2".into(), next),
        ("new-1".into(), new),
        ("keeper-1".into(), keeper),
    ]);
    let files = archives["legacy-1"]
        .files
        .iter()
        .chain(archives["keeper-1"].files.iter())
        .map(|(n, p)| (n.clone(), p.contents.clone()))
        .collect();
    p::State::new(
        p::Catalog {
            architecture: p::Architecture::Amd64,
            repository_reachable: true,
            published: "current".into(),
            archives,
            publications: BTreeMap::from([(
                "current".into(),
                p::Index {
                    candidates: BTreeMap::from([
                        ("legacy-agent".into(), "legacy-2".into()),
                        ("new-app".into(), "new-1".into()),
                        ("keeper".into(), "keeper-1".into()),
                    ]),
                },
            )]),
            extra_paths: BTreeSet::new(),
        },
        p::Runtime {
            cached: Some("current".into()),
            local_archives: BTreeSet::from(["legacy-1".into(), "keeper-1".into()]),
            records: BTreeMap::from([
                (
                    "legacy-agent".into(),
                    p::PackageRecord {
                        archive: "legacy-1".into(),
                        status: p::Status::Installed,
                    },
                ),
                (
                    "keeper".into(),
                    p::PackageRecord {
                        archive: "keeper-1".into(),
                        status: p::Status::Installed,
                    },
                ),
            ]),
            files,
        },
    )
    .unwrap()
}
fn changed(s: &p::State, edit: impl FnOnce(&mut p::Catalog, &mut p::Runtime)) -> p::State {
    let (mut c, mut r) = (s.catalog().clone(), s.runtime().clone());
    edit(&mut c, &mut r);
    p::State::new(c, r).unwrap()
}
fn install(names: &[&str]) -> p::Action {
    p::Action::Install {
        packages: names
            .iter()
            .map(|n| p::Request {
                name: (*n).into(),
                version: None,
            })
            .collect(),
    }
}
fn reject(s: &mut p::State, action: p::Action) {
    let before = serde_json::to_vec(s).unwrap();
    assert_eq!(s.plan(&action).unwrap_err().code, p::ErrorCode::Conflict);
    let error = s.apply(&action).unwrap_err();
    assert_eq!(error.code, p::ErrorCode::Conflict);
    assert!(error.message.contains("conflicts with installed"));
    assert_eq!(serde_json::to_vec(s).unwrap(), before);
}
#[test]
fn explicit_conflict_is_atomic_even_with_disjoint_files_and_both_install_orders() {
    let mut s = sample(None);
    reject(&mut s, install(&["new-app"]));
    assert_eq!(
        s.conflict_issues().unwrap().len(),
        0,
        "an uninstalled archive does not conflict yet"
    );
    let reverse = changed(&s, |c, _| {
        c.archives.get_mut("new-1").unwrap().conflicts.clear();
        c.archives
            .get_mut("legacy-1")
            .unwrap()
            .conflicts
            .push(p::Dependency {
                name: "new-app".into(),
                constraint: None,
            });
    });
    reject(&mut reverse.clone(), install(&["new-app"]));
    let empty = changed(&s, |_, r| {
        r.records.remove("legacy-agent");
        r.files.retain(|n, _| !n.contains("legacy"));
    });
    let mut x = empty;
    reject(&mut x, install(&["new-app", "legacy-agent"]));
}
#[test]
fn config_only_retirement_preserves_custom_text_and_is_not_a_conflict() {
    let mut s = changed(&sample(None), |_, r| {
        r.files
            .insert("/etc/legacy.conf".into(), "mode=site\n".into());
    });
    s.apply(&p::Action::Remove {
        name: "legacy-agent".into(),
    })
    .unwrap();
    s.apply(&install(&["new-app"])).unwrap();
    assert_eq!(
        s.runtime().records["legacy-agent"].status,
        p::Status::ConfigFiles
    );
    assert_eq!(s.runtime().files["/etc/legacy.conf"], "mode=site\n");
    assert!(s.conflict_issues().unwrap().is_empty());
    assert!(s.payload_issues("new-app").unwrap().is_empty());
}
#[test]
fn version_constraints_use_installed_not_cached_versions_and_accept_equivalent_repairs() {
    let source = changed(&sample(before_two()), |_, r| {
        r.files
            .insert("/etc/legacy.conf".into(), "mode=site\n".into());
    });
    reject(&mut source.clone(), install(&["new-app"]));
    let mut separate = source.clone();
    separate.apply(&install(&["legacy-agent"])).unwrap();
    separate.apply(&install(&["new-app"])).unwrap();
    let mut joint = source;
    joint.apply(&install(&["new-app", "legacy-agent"])).unwrap();
    assert_eq!(joint, separate);
    assert_eq!(joint.runtime().files["/etc/legacy.conf"], "mode=site\n");
    assert!(joint.conflict_issues().unwrap().is_empty());
}
#[test]
fn version_boundary_vectors_are_semantic_not_literal_or_semver() {
    for (version, conflicts) in [
        ("1.9", true),
        ("2.0~rc1", true),
        ("2.0", false),
        ("2.0-0", false),
        ("2.0-1", false),
        ("1:1.0", false),
    ] {
        let mut s = changed(&sample(before_two()), |c, _| {
            c.archives.remove("legacy-2");
            c.archives.get_mut("legacy-1").unwrap().version = p::Version::parse(version).unwrap();
            c.publications
                .get_mut("current")
                .unwrap()
                .candidates
                .insert("legacy-agent".into(), "legacy-1".into());
        });
        let result = s.apply(&install(&["new-app"]));
        assert_eq!(result.is_err(), conflicts, "{version}");
        if let Err(e) = result {
            assert_eq!(e.code, p::ErrorCode::Conflict);
        }
    }
    for (op, expected) in [
        (p::Relation::Lt, false),
        (p::Relation::Le, true),
        (p::Relation::Eq, true),
        (p::Relation::Ge, true),
        (p::Relation::Gt, false),
    ] {
        let constraint = Some(p::Constraint {
            relation: op,
            version: p::Version::parse("1.0-0").unwrap(),
        });
        let mut s = sample(constraint);
        assert_eq!(s.apply(&install(&["new-app"])).is_err(), expected);
    }
}
#[test]
fn transitive_selected_conflicts_and_reinstall_cannot_bypass_the_guard() {
    let s = changed(&sample(None), |c, _| {
        let mut helper = archive("helper", "1.0");
        helper.conflicts.push(relation(None));
        c.archives.get_mut("new-1").unwrap().conflicts.clear();
        c.archives
            .get_mut("new-1")
            .unwrap()
            .depends
            .push(p::Dependency {
                name: "helper".into(),
                constraint: None,
            });
        c.archives.insert("helper-1".into(), helper);
        c.publications
            .get_mut("current")
            .unwrap()
            .candidates
            .insert("helper".into(), "helper-1".into());
    });
    reject(&mut s.clone(), install(&["new-app"]));
    let broken = changed(&sample(None), |c, r| {
        r.records.insert(
            "new-app".into(),
            p::PackageRecord {
                archive: "new-1".into(),
                status: p::Status::Installed,
            },
        );
        for (path, payload) in &c.archives["new-1"].files {
            r.files.insert(path.clone(), payload.contents.clone());
        }
    });
    assert_eq!(broken.conflict_issues().unwrap().len(), 1);
    reject(
        &mut broken.clone(),
        p::Action::Reinstall {
            name: "new-app".into(),
        },
    );
    let mut repair = broken;
    repair
        .apply(&p::Action::Remove {
            name: "legacy-agent".into(),
        })
        .unwrap();
    assert!(repair.conflict_issues().unwrap().is_empty());
}
#[test]
fn self_name_exception_and_an_absent_counterpart_do_not_forbid_installation() {
    let mut s = changed(&sample(None), |c, _| {
        c.archives.get_mut("new-1").unwrap().conflicts = vec![
            p::Dependency {
                name: "new-app".into(),
                constraint: None,
            },
            p::Dependency {
                name: "not-supplied".into(),
                constraint: None,
            },
        ];
    });
    s.apply(&install(&["new-app"])).unwrap();
    assert!(s.conflict_issues().unwrap().is_empty());
}
#[test]
fn old_serialized_objects_stay_exact_and_new_lists_are_bounded_and_validated() {
    let old: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/debian-packages-declared.json")).unwrap();
    let parsed: p::State = serde_json::from_value(old["state"].clone()).unwrap();
    // The authored JSON is input, not a serialized State: its BTreeSet of local
    // archives was already sorted by the preexisting writer. Compare that exact
    // canonical wire, preserving every other field and omitting empty Conflicts.
    let mut expected = old["state"].clone();
    expected["runtime"]["local_archives"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    assert_eq!(serde_json::to_value(parsed).unwrap(), expected);
    let base = serde_json::to_value(sample(None)).unwrap();
    let mut oversized = base.clone();
    oversized["catalog"]["archives"]["new-1"]["conflicts"] =
        serde_json::json!(vec![relation(None); 9]);
    assert!(serde_json::from_value::<p::State>(oversized).is_err());
    let mut invalid = base.clone();
    invalid["catalog"]["archives"]["new-1"]["conflicts"][0]["name"] =
        serde_json::json!("../outside");
    assert!(serde_json::from_value::<p::State>(invalid).is_err());
    let mut unknown = base;
    unknown["catalog"]["archives"]["new-1"]["replaces"] = serde_json::json!(["legacy-agent"]);
    assert!(serde_json::from_value::<p::State>(unknown).is_err());
}
#[test]
fn metadata_output_is_derived_read_only_and_failures_explain_the_conflict() {
    let mut s = sample(before_two());
    let d = work::Definition {
        initial: s.clone(),
        requirements: vec![
            work::Requirement::Installed {
                package: "new-app".into(),
                constraint: None,
            },
            work::Requirement::ConflictsSatisfied {},
        ],
    };
    d.validate().unwrap();
    let before = serde_json::to_vec(&s).unwrap();
    let out = terminal::execute(&d, &mut s, "apt-cache show new-app").unwrap();
    assert_eq!(out.status, 0);
    assert_eq!(
        out.stdout,
        "Package: new-app\nVersion: 1.0\nArchitecture: amd64\nConflicts: legacy-agent (<< 2.0)\n\n"
    );
    assert!(!out.changed);
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
    let error = terminal::execute(&d, &mut s, "apt-get -y install new-app").unwrap_err();
    assert_eq!(error.status, 100);
    assert_eq!(error.kind, terminal::DiagnosticKind::Model);
    assert!(error.message.contains("legacy-agent version 1.0"));
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
    for unsupported in [
        "apt-cache show new-app extra",
        "apt-cache --important show new-app",
        "apt-cache show new-app; anything",
    ] {
        assert!(terminal::execute(&d, &mut s, unsupported).is_err());
        assert_eq!(serde_json::to_vec(&s).unwrap(), before);
    }
}
