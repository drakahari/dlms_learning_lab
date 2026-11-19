use dlms_learning_lab::rpm_packages as core;
use std::collections::BTreeMap;
pub fn identity(name: &str) -> core::Identity {
    core::Identity {
        name: name.into(),
        arch: core::Architecture::Noarch,
    }
}
fn file(s: &str) -> core::File {
    core::File {
        contents: s.into(),
        mode: 0o644,
        owner: "root".into(),
        group: "root".into(),
        mtime: 7,
    }
}
pub fn state() -> core::State {
    let mut site = core::Archive {
        identity: identity("site"),
        evr: core::Evr::parse("1.0-1").unwrap(),
        requires: vec![],
        provides: BTreeMap::new(),
        files: BTreeMap::from([
            (
                "/etc/site.conf".into(),
                core::Payload {
                    file: file("route=default\n"),
                    kind: core::FileKind::ConfigNoreplace,
                },
            ),
            (
                "/usr/share/site.txt".into(),
                core::Payload {
                    file: file("site help\n"),
                    kind: core::FileKind::Ordinary,
                },
            ),
        ]),
    };
    site.requires.push(core::Requirement {
        capability: "renderer-api".into(),
        constraint: None,
    });
    let mut newer = site.clone();
    newer.evr = core::Evr::parse("2.0-1").unwrap();
    for p in newer.files.values_mut() {
        p.file.mode = 0o600;
        p.file.owner = "future".into();
        p.file.mtime = 700;
    }
    let app = core::Archive {
        identity: identity("renderer"),
        evr: core::Evr::parse("1.0-1").unwrap(),
        requires: vec![],
        provides: BTreeMap::from([("renderer-api".into(), None)]),
        files: BTreeMap::from([(
            "/usr/share/renderer.txt".into(),
            core::Payload {
                file: file("original renderer\n"),
                kind: core::FileKind::Ordinary,
            },
        )]),
    };
    let mut files = BTreeMap::from([("/notes.txt".into(), file("retain unrelated note\n"))]);
    for a in [&site, &app] {
        for (p, v) in &a.files {
            files.insert(p.clone(), v.file.clone());
        }
    }
    let config = files.get_mut("/etc/site.conf").unwrap();
    config.contents = "route=local\n".into();
    config.mode = 0o640;
    config.owner = "alex".into();
    config.group = "ops".into();
    config.mtime = 99;
    let binary = files.get_mut("/usr/share/renderer.txt").unwrap();
    binary.contents = "damaged renderer\n".into();
    binary.mode = 0o600;
    binary.owner = "alex".into();
    binary.group = "ops".into();
    binary.mtime = 100;
    let catalog = core::Catalog {
        fixture: "metadata-repair-proof".into(),
        target: core::Architecture::X86_64,
        published: core::SnapshotId::new("current").unwrap(),
        archives: BTreeMap::from([
            ("site-v1".into(), site),
            ("site-v2".into(), newer),
            ("renderer-v1".into(), app),
        ]),
        snapshots: BTreeMap::from([(
            "current".into(),
            core::Snapshot {
                archives: vec!["site-v2".into(), "renderer-v1".into()],
            },
        )]),
        allowed_paths: files.keys().cloned().collect(),
    };
    let runtime = core::Runtime {
        installed: vec![
            core::Installed {
                identity: identity("site"),
                archive: "site-v1".into(),
            },
            core::Installed {
                identity: identity("renderer"),
                archive: "renderer-v1".into(),
            },
        ],
        available: vec!["site-v1".into(), "site-v2".into(), "renderer-v1".into()],
        cached: Some(core::SnapshotId::new("current").unwrap()),
        files,
    };
    core::State::new(catalog, runtime).unwrap()
}
