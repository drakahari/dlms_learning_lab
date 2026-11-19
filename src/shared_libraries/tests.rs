//! Metadata resolution and isolation regressions.
use super::*;
fn abi() -> Abi {
    Abi {
        class: Class::Elf64,
        machine: Machine::X86_64,
        byte_order: ByteOrder::Little,
    }
}
fn object(id: &str, path: &str, soname: &str, needed: &[&str]) -> Object {
    Object {
        identity: id.into(),
        path: path.into(),
        abi: abi(),
        soname: soname.into(),
        needed: needed.iter().map(|v| (*v).into()).collect(),
    }
}
fn fixture() -> State {
    State::new(
        Catalog {
            version: 1,
            directories: ["/lib64", "/usr/lib64", "/opt/cedar/lib", "/opt/amber/lib"]
                .map(str::to_owned)
                .into(),
            trusted: ["/lib64", "/usr/lib64"].map(str::to_owned).into(),
            objects: vec![
                object("base", "/lib64/libbase.so.1", "libbase.so.1", &[]),
                object(
                    "cedar",
                    "/opt/cedar/lib/libreport.so.1.4",
                    "libreport.so.1",
                    &["libbase.so.1"],
                ),
                object(
                    "amber",
                    "/opt/amber/lib/libreport.so.1.9",
                    "libreport.so.1",
                    &["libbase.so.1"],
                ),
            ],
            programs: vec![
                Program {
                    path: "/usr/bin/report".into(),
                    abi: abi(),
                    needed: vec!["libreport.so.1".into()],
                },
                Program {
                    path: "/usr/bin/viewer".into(),
                    abi: abi(),
                    needed: vec!["libbase.so.1".into()],
                },
            ],
            aliases: vec![
                Alias {
                    path: "/opt/cedar/lib/libreport.so.1".into(),
                    object: "/opt/cedar/lib/libreport.so.1.4".into(),
                },
                Alias {
                    path: "/opt/amber/lib/libreport.so.1".into(),
                    object: "/opt/amber/lib/libreport.so.1.9".into(),
                },
            ],
        },
        "",
    )
    .unwrap()
}
fn selected(s: &State, mode: Environment) -> String {
    s.resolve("/usr/bin/report", mode).unwrap().bindings[0]
        .selected
        .as_ref()
        .unwrap()
        .identity
        .clone()
}
#[test]
fn missing_environment_override_and_clean_launch_are_distinct() {
    let mut s = fixture();
    let catalog = s.catalog.clone();
    assert!(
        !s.resolve("/usr/bin/report", Environment::Configured)
            .unwrap()
            .complete()
    );
    assert!(
        s.resolve("/usr/bin/viewer", Environment::Clean)
            .unwrap()
            .complete()
    );
    s.set_library_path(Some("/opt/cedar/lib")).unwrap();
    let r = s
        .resolve("/usr/bin/report", Environment::Configured)
        .unwrap();
    assert!(r.complete());
    assert_eq!(r.bindings.len(), 2);
    assert_eq!(
        r.bindings[0].selected.as_ref().unwrap().path,
        "/opt/cedar/lib/libreport.so.1"
    );
    assert_eq!(
        r.bindings[0].selected.as_ref().unwrap().object,
        "/opt/cedar/lib/libreport.so.1.4"
    );
    assert!(
        !s.resolve("/usr/bin/report", Environment::Clean)
            .unwrap()
            .complete()
    );
    assert!(r.report().unwrap().contains("no ELF code"));
    assert_eq!(s.catalog, catalog);
}
#[test]
fn configuration_edit_does_not_implicitly_rebuild_or_replace_cache() {
    let mut s = fixture();
    s.rebuild_cache().unwrap();
    let old = s.cache.clone();
    s.save_configuration("/opt/cedar/lib\n").unwrap();
    assert_eq!(s.cache, old);
    assert!(!s.cache_matches_configuration().unwrap());
    assert!(
        !s.resolve("/usr/bin/report", Environment::Clean)
            .unwrap()
            .complete()
    );
    s.rebuild_cache().unwrap();
    assert!(s.cache_matches_configuration().unwrap());
    assert_eq!(selected(&s, Environment::Clean), "cedar");
    s.save_configuration("/opt/amber/lib\n").unwrap();
    assert_eq!(selected(&s, Environment::Clean), "cedar");
    s.rebuild_cache().unwrap();
    assert_eq!(selected(&s, Environment::Clean), "amber");
}
#[test]
fn environment_precedes_cache_and_trusted_defaults_and_safe_variants_pass() {
    let mut s = fixture();
    s.save_configuration("/opt/cedar/lib\n").unwrap();
    s.rebuild_cache().unwrap();
    s.set_library_path(Some("/opt/amber/lib;/opt/cedar/lib"))
        .unwrap();
    assert_eq!(selected(&s, Environment::Configured), "amber");
    assert_eq!(selected(&s, Environment::Clean), "cedar");
    s.set_library_path(Some("/opt/cedar/lib:/opt/amber/lib"))
        .unwrap();
    assert_eq!(selected(&s, Environment::Configured), "cedar");
    s.set_library_path(None).unwrap();
    assert_eq!(selected(&s, Environment::Configured), "cedar");
    let before = s.resolve("/usr/bin/report", Environment::Clean).unwrap();
    s.save_configuration("# cache paths\n/opt/cedar/lib # site\n/lib64\n/opt/cedar/lib\n")
        .unwrap();
    s.rebuild_cache().unwrap();
    assert_eq!(
        s.resolve("/usr/bin/report", Environment::Clean).unwrap(),
        before
    );
}
#[test]
fn soname_cache_key_is_not_a_universal_runtime_abi_equality_gate() {
    let mut s = fixture();
    s.catalog.objects[1].path = "/opt/cedar/lib/libreport.so.2".into();
    s.catalog.aliases[0].object = s.catalog.objects[1].path.clone();
    s.catalog.programs[0].needed = vec!["libreport.so.2".into()];
    s.validate().unwrap();
    s.set_library_path(Some("/opt/cedar/lib")).unwrap();
    assert!(
        s.resolve("/usr/bin/report", Environment::Configured)
            .unwrap()
            .complete()
    );
    s.save_configuration("/opt/cedar/lib").unwrap();
    s.rebuild_cache().unwrap();
    assert!(
        s.cache
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .any(|e| e.name == "libreport.so.1")
    );
    assert!(
        !s.resolve("/usr/bin/report", Environment::Clean)
            .unwrap()
            .complete()
    );
}
#[test]
fn unsupported_syntax_paths_and_ambiguous_cache_refuse_atomically() {
    let mut s = fixture();
    s.save_configuration("/opt/cedar/lib").unwrap();
    s.rebuild_cache().unwrap();
    for text in [
        "include /etc/ld.so.conf.d/*.conf",
        "../lib",
        "/home/personal",
        "$ORIGIN/lib",
        "/opt/cedar/lib\0",
    ] {
        let before = s.to_json().unwrap();
        assert!(s.save_configuration(text).is_err());
        assert_eq!(s.to_json().unwrap(), before);
    }
    for text in [
        "",
        ":/opt/cedar/lib",
        "/opt/cedar/lib:",
        ".",
        "$LIB",
        "/home/personal",
    ] {
        let before = s.to_json().unwrap();
        assert!(s.set_library_path(Some(text)).is_err());
        assert_eq!(s.to_json().unwrap(), before);
    }
    s.save_configuration("/opt/cedar/lib\n/opt/amber/lib")
        .unwrap();
    let before = s.to_json().unwrap();
    assert_eq!(s.rebuild_cache().unwrap_err().kind, ErrorKind::Unsupported);
    assert_eq!(s.to_json().unwrap(), before);
}
#[test]
fn architecture_metadata_is_explicit_and_unverified_mismatch_fallback_is_honest() {
    let mut s = fixture();
    s.catalog.objects[1].abi = Abi {
        class: Class::Elf32,
        machine: Machine::X86,
        byte_order: ByteOrder::Little,
    };
    s.validate().unwrap();
    s.set_library_path(Some("/opt/cedar/lib")).unwrap();
    let before = s.to_json().unwrap();
    assert_eq!(
        s.resolve("/usr/bin/report", Environment::Configured)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(s.to_json().unwrap(), before);
    s.catalog.programs[0].abi.machine = Machine::X86;
    assert!(s.validate().is_err());
}
#[test]
fn complete_dependency_closure_handles_cycles_without_unbounded_work() {
    let mut s = fixture();
    s.catalog.objects[0].needed = vec!["libreport.so.1".into()];
    s.set_library_path(Some("/opt/cedar/lib")).unwrap();
    let r = s
        .resolve("/usr/bin/report", Environment::Configured)
        .unwrap();
    assert!(r.complete());
    assert_eq!(r.bindings.len(), 3);
    let mut missing = fixture();
    missing.catalog.objects[1].needed = vec!["libabsent.so.1".into()];
    missing.set_library_path(Some("/opt/cedar/lib")).unwrap();
    let r = missing
        .resolve("/usr/bin/report", Environment::Configured)
        .unwrap();
    assert!(!r.complete());
    assert!(
        r.bindings
            .iter()
            .any(|b| b.needed == "libabsent.so.1" && b.selected.is_none())
    );
}
#[test]
fn depth_and_cache_operation_limits_leave_existing_state_unchanged() {
    let mut s = fixture();
    for _ in 0..MAX_REBUILDS {
        s.rebuild_cache().unwrap();
    }
    let before = s.to_json().unwrap();
    assert_eq!(s.rebuild_cache().unwrap_err().kind, ErrorKind::Limit);
    assert_eq!(s.to_json().unwrap(), before);
    let objects = (0..18)
        .map(|i| {
            object(
                &format!("node{i}"),
                &format!("/lib64/libnode{i}.so.1"),
                &format!("libnode{i}.so.1"),
                &[],
            )
        })
        .collect::<Vec<_>>();
    let mut chain = Catalog {
        version: 1,
        directories: vec!["/lib64".into()],
        trusted: vec!["/lib64".into()],
        objects,
        programs: vec![Program {
            path: "/usr/bin/chain".into(),
            abi: abi(),
            needed: vec!["libnode0.so.1".into()],
        }],
        aliases: vec![],
    };
    for i in 0..17 {
        chain.objects[i].needed = vec![format!("libnode{}.so.1", i + 1)];
    }
    let s = State::new(chain, "").unwrap();
    let before = s.to_json().unwrap();
    assert_eq!(
        s.resolve("/usr/bin/chain", Environment::Clean)
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(s.to_json().unwrap(), before);
}
#[test]
fn saved_work_recomputes_cache_and_requires_the_original_source() {
    let source = fixture();
    let mut s = source.clone();
    s.save_configuration("/opt/cedar/lib").unwrap();
    s.rebuild_cache().unwrap();
    let raw = s.to_json().unwrap();
    assert_eq!(State::from_json(&raw, &source).unwrap(), s);
    let mut v: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    v["catalog"]["objects"][0]["identity"] = "invented".into();
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap(), &source).is_err());
    let mut v: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    v["cache"]["entries"][0]["object"] = "/opt/cedar/lib/libreport.so.1.4".into();
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap(), &source).is_err());
    let mut v: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    v["catalog"]["programs"][0]["rpath"] = "/opt/cedar/lib".into();
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap(), &source).is_err());
}
#[test]
fn collection_byte_output_and_version_bounds_are_explicit() {
    let source = fixture();
    assert!(State::from_json(&vec![b' '; MAX_STATE + 1], &source).is_err());
    let mut v = serde_json::to_value(&source).unwrap();
    v["version"] = 9.into();
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap(), &source).is_err());
    let mut v = serde_json::to_value(&source).unwrap();
    let row = v["catalog"]["programs"][0].clone();
    v["catalog"]["programs"] = serde_json::Value::Array(vec![row; MAX_PROGRAMS + 1]);
    assert!(State::from_json(&serde_json::to_vec(&v).unwrap(), &source).is_err());
    let selected = Selected {
        path: format!("/{}", "x".repeat(255)),
        object: "/lib64/libx.so.1".into(),
        identity: "x".into(),
        via: Via::Cache,
    };
    let r = Resolution {
        program: "/usr/bin/test".into(),
        bindings: vec![
            Binding {
                requester: format!("/{}", "p".repeat(255)),
                needed: "x".repeat(128),
                selected: Some(selected)
            };
            MAX_VISITS
        ],
    };
    assert_eq!(r.report().unwrap_err().kind, ErrorKind::Limit);
}

#[test]
fn previously_loaded_soname_wins_over_a_new_environment_search() {
    let mut s = fixture();
    s.catalog.objects[1].path = "/opt/cedar/lib/libreport.so.2".into();
    s.catalog.aliases[0].object = s.catalog.objects[1].path.clone();
    s.catalog.programs[0].needed = vec!["libreport.so.2".into(), "libreport.so.1".into()];
    s.validate().unwrap();
    s.set_library_path(Some("/opt/amber/lib:/opt/cedar/lib"))
        .unwrap();
    let before = s.to_json().unwrap();
    let r = s
        .resolve("/usr/bin/report", Environment::Configured)
        .unwrap();
    assert!(r.complete());
    assert_eq!(r.bindings[0].selected.as_ref().unwrap().identity, "cedar");
    assert_eq!(r.bindings[1].selected.as_ref().unwrap().identity, "cedar");
    assert_eq!(r.bindings[1].selected.as_ref().unwrap().via, Via::Loaded);
    assert_eq!(
        r.bindings[1].selected.as_ref().unwrap().path,
        "/opt/cedar/lib/libreport.so.2"
    );
    assert_eq!(
        r.bindings.len(),
        3,
        "One instance of the transitive base need"
    );
    assert_eq!(s.to_json().unwrap(), before);
    // Independent fresh resolution does not inherit the prior link map.
    s.catalog.programs[0].needed = vec!["libreport.so.1".into()];
    assert_eq!(selected(&s, Environment::Configured), "amber");
}
#[test]
fn transitive_request_reuses_an_earlier_soname_without_loading_a_second_object() {
    let mut s = fixture();
    s.catalog.objects[1].path = "/opt/cedar/lib/libreport.so.2".into();
    s.catalog.aliases[0].object = s.catalog.objects[1].path.clone();
    s.catalog.objects[0].needed = vec!["libreport.so.1".into()];
    s.catalog.programs[0].needed = vec!["libreport.so.2".into()];
    s.set_library_path(Some("/opt/amber/lib:/opt/cedar/lib"))
        .unwrap();
    let r = s
        .resolve("/usr/bin/report", Environment::Configured)
        .unwrap();
    assert!(r.complete());
    assert_eq!(r.bindings.len(), 3);
    assert_eq!(r.bindings[2].requester, "/lib64/libbase.so.1");
    assert_eq!(r.bindings[2].selected.as_ref().unwrap().identity, "cedar");
    assert_eq!(r.bindings[2].selected.as_ref().unwrap().via, Via::Loaded);
}

#[test]
fn distinct_runtime_names_with_a_duplicate_soname_refuse_without_a_partial_result() {
    let mut s = fixture();
    s.catalog.objects[1].path = "/opt/cedar/lib/libreport.so.2".into();
    s.catalog.aliases[0].object = s.catalog.objects[1].path.clone();
    s.catalog.objects[2].path = "/opt/amber/lib/libreport.so.3".into();
    s.catalog.aliases[1].object = s.catalog.objects[2].path.clone();
    s.catalog.programs[0].needed = vec!["libreport.so.2".into(), "libreport.so.3".into()];
    s.set_library_path(Some("/opt/cedar/lib:/opt/amber/lib"))
        .unwrap();
    let before = s.to_json().unwrap();
    assert_eq!(
        s.resolve("/usr/bin/report", Environment::Configured)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(
        s.to_json().unwrap(),
        before,
        "Refusal publishes no resolution and mutates no state"
    );
}
#[test]
fn loaded_soname_reuse_precedes_a_later_wrong_abi_directory_candidate() {
    let mut s = fixture();
    s.catalog.objects[1].path = "/opt/cedar/lib/libreport.so.2".into();
    s.catalog.aliases[0].object = s.catalog.objects[1].path.clone();
    s.catalog.objects[2].abi = Abi {
        class: Class::Elf32,
        machine: Machine::X86,
        byte_order: ByteOrder::Little,
    };
    s.catalog.programs[0].needed = vec!["libreport.so.2".into(), "libreport.so.1".into()];
    s.set_library_path(Some("/opt/amber/lib:/opt/cedar/lib"))
        .unwrap();
    let r = s
        .resolve("/usr/bin/report", Environment::Configured)
        .unwrap();
    assert!(r.complete());
    assert_eq!(r.bindings[1].selected.as_ref().unwrap().via, Via::Loaded);
    assert_eq!(r.bindings[1].selected.as_ref().unwrap().identity, "cedar");
    s.catalog.programs[0].needed = vec!["libreport.so.1".into()];
    assert_eq!(
        s.resolve("/usr/bin/report", Environment::Configured)
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
}
