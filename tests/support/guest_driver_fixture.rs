//! Pure test data only; these are not authored lessons or actual RPM/ELF files.
#[path = "kernel_module_fixture.rs"]
mod primitive;
use dlms_learning_lab::{guest_driver::*, kernel_modules as modules, rpm_packages as rpm};
use std::collections::BTreeMap;

pub fn facts() -> Facts {
    let mut c = primitive::catalogue();
    let mut extra = c.modules["audit_link"].clone();
    extra.path = format!(
        "/lib/modules/{}/kernel/drivers/emulated_net.ko",
        c.kernel_release
    );
    extra.description = "A different supplied virtual adapter's driver".into();
    c.modules.insert("emulated_net".into(), extra);
    let objects: Vec<_> = c
        .modules
        .iter()
        .map(|(n, m)| Object {
            module: n.clone(),
            contents: format!("INERT descriptor {n} for {}", m.kernel_release),
        })
        .collect();
    fn archive(
        c: &modules::Catalogue,
        objects: &[Object],
        name: &str,
        evr: &str,
        names: &[&str],
        kernel: Option<&str>,
    ) -> rpm::Archive {
        rpm::Archive {
            identity: rpm::Identity {
                name: name.into(),
                arch: rpm::Architecture::X86_64,
            },
            evr: rpm::Evr::parse(evr).unwrap(),
            requires: kernel
                .map(|k| rpm::Requirement {
                    capability: "kernel-api".into(),
                    constraint: Some(rpm::Constraint {
                        relation: rpm::Relation::Eq,
                        evr: rpm::Evr::parse(k).unwrap(),
                    }),
                })
                .into_iter()
                .collect(),
            provides: if kernel.is_none() {
                BTreeMap::from([(
                    "kernel-api".into(),
                    Some(rpm::Evr::parse("6.8.0-1").unwrap()),
                )])
            } else {
                BTreeMap::new()
            },
            files: names
                .iter()
                .map(|n| {
                    (
                        c.modules[*n].path.clone(),
                        rpm::Payload {
                            kind: rpm::FileKind::Ordinary,
                            file: rpm::File {
                                contents: objects
                                    .iter()
                                    .find(|o| o.module == *n)
                                    .unwrap()
                                    .contents
                                    .clone(),
                                mode: 0o644,
                                owner: "root".into(),
                                group: "root".into(),
                                mtime: 10,
                            },
                        },
                    )
                })
                .collect(),
        }
    }
    let archives = BTreeMap::from([
        (
            "kernel-core".into(),
            archive(
                &c,
                &objects,
                "kernel-core",
                "6.8.0-1",
                &["bus_core", "sensor_bus", "audit_link"],
                None,
            ),
        ),
        (
            "support-current".into(),
            archive(
                &c,
                &objects,
                "guest-support",
                "2.0-1",
                &["lab_sensor"],
                Some("6.8.0-1"),
            ),
        ),
        (
            "support-old".into(),
            archive(
                &c,
                &objects,
                "guest-support",
                "1.0-1",
                &["old_sensor"],
                Some("6.7.0-1"),
            ),
        ),
        (
            "support-other-device".into(),
            archive(
                &c,
                &objects,
                "emulated-support",
                "1.0-1",
                &["emulated_net"],
                Some("6.8.0-1"),
            ),
        ),
    ]);
    let packages = rpm::Catalog {
        fixture: "guest-driver-fixture".into(),
        target: rpm::Architecture::X86_64,
        published: rpm::SnapshotId::new("local").unwrap(),
        snapshots: BTreeMap::from([(
            "local".into(),
            rpm::Snapshot {
                archives: archives.keys().cloned().collect(),
            },
        )]),
        allowed_paths: c.modules.values().map(|m| m.path.clone()).collect(),
        archives,
    };
    let facts = Facts {
        modules: c,
        packages,
        objects,
    };
    facts.validate().unwrap();
    facts
}
pub fn runtime(f: &Facts) -> Runtime {
    let mut packages = rpm::State::new(
        f.packages.clone(),
        rpm::Runtime {
            installed: vec![],
            available: f.packages.archives.keys().cloned().collect(),
            cached: None,
            files: BTreeMap::new(),
        },
    )
    .unwrap();
    packages
        .apply(&rpm::Action::InstallSet {
            archives: vec!["kernel-core".into()],
        })
        .unwrap();
    let mut modules = primitive::runtime(&f.modules);
    modules.index = ["bus_core", "sensor_bus", "audit_link"]
        .map(str::to_owned)
        .into_iter()
        .collect();
    primitive::ok(&mut modules, &f.modules, "modprobe audit_link");
    let r = Runtime {
        packages: packages.runtime().clone(),
        modules,
    };
    r.validate(f).unwrap();
    r
}
