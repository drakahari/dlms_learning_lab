use dlms_learning_lab::{
    debian_contents as contents, debian_packages as core,
    engine::{Goal, State},
    pack,
};
fn sample() -> (
    dlms_learning_lab::debian_packages_workspace::Definition,
    core::State,
) {
    let s = pack::load(include_str!(
        "fixtures/debian-packages-workspace-preview.json"
    ))
    .unwrap()
    .remove(0);
    let Goal::DebianPackages { definition } = s.goal else {
        panic!()
    };
    let State::DebianPackages { simulation } = s.initial else {
        panic!()
    };
    (*definition, simulation)
}
fn parsed(s: &str) -> contents::Query {
    contents::parse(&s.split_whitespace().map(str::to_owned).collect::<Vec<_>>()).unwrap()
}
#[test]
fn repository_search_differs_from_installed_ownership_and_never_mutates() {
    let (_, s) = sample();
    let before = serde_json::to_vec(&s).unwrap();
    let r = contents::query(&s, &parsed("search museum-report")).unwrap();
    assert_eq!(r.status, 0);
    assert!(r.text.contains("museum-report: usr/bin/museum-report\n"));
    let r = contents::query(&s, &parsed("-F search /usr/bin/museum-report")).unwrap();
    assert_eq!(r.text, "museum-report: usr/bin/museum-report\n");
    assert_eq!(serde_json::to_vec(&s).unwrap(), before);
}
#[test]
fn literal_substring_exact_and_package_listing_are_distinct() {
    let (_, s) = sample();
    assert_eq!(
        contents::query(&s, &parsed("-F search museum-report"))
            .unwrap()
            .status,
        1
    );
    let r = contents::query(&s, &parsed("list museum-report")).unwrap();
    assert!(r.text.contains("etc/museum/report.conf"));
    assert!(r.text.contains("usr/bin/museum-report"));
    assert_eq!(
        r,
        contents::query(&s, &parsed("--fixed-string show museum-report")).unwrap()
    );
    assert_eq!(
        contents::query(&s, &parsed("find museum-report")).unwrap(),
        contents::query(&s, &parsed("search museum-report")).unwrap()
    );
}
#[test]
fn no_match_empty_cache_and_offline_cache_have_honest_statuses() {
    let (_, s) = sample();
    let mut r = s.runtime().clone();
    r.cached = None;
    let empty = core::State::new(s.catalog().clone(), r).unwrap();
    assert_eq!(
        contents::query(&empty, &parsed("search museum-report"))
            .unwrap()
            .status,
        3
    );
    assert_eq!(
        contents::query(&s, &parsed("search definitely-missing"))
            .unwrap()
            .status,
        1
    );
    let mut c = s.catalog().clone();
    c.repository_reachable = false;
    let offline = core::State::new(c, s.runtime().clone()).unwrap();
    assert_eq!(
        contents::query(&offline, &parsed("list museum-report")).unwrap(),
        contents::query(&s, &parsed("list museum-report")).unwrap()
    );
}
#[test]
fn unsupported_selectors_patterns_and_typed_escapes_refuse() {
    for text in [
        "search *report*",
        "-x search report",
        "-F -F search report",
        "search ../report",
        "search /",
        "search //usr/bin/report",
        "list museum-*",
        "-a arm64 search report",
        "update",
        "search report extra",
        "search report\0",
    ] {
        assert!(
            contents::parse(
                &text
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            )
            .is_err(),
            "{text}"
        );
    }
    let (_, s) = sample();
    for q in [
        contents::Query::Search {
            pattern: "../secret".into(),
            exact: false,
        },
        contents::Query::Search {
            pattern: "x".repeat(161),
            exact: false,
        },
        contents::Query::List {
            package: "*".into(),
        },
    ] {
        assert!(contents::query(&s, &q).is_err());
    }
}
#[test]
fn cache_refresh_changes_metadata_without_installing_and_other_architectures_stay_out() {
    let (_, mut s) = sample();
    let before = s.runtime().records.clone();
    let old = contents::query(&s, &parsed("search museum")).unwrap();
    assert!(!old.text.contains("museum-foreign"));
    s.apply(&core::Action::RefreshIndex {}).unwrap();
    assert_eq!(s.runtime().records, before);
    let r = contents::query(&s, &parsed("list museum-common")).unwrap();
    assert_eq!(r.status, 0);
    assert!(r.text.contains("museum-common"));
}
#[test]
fn output_bound_is_checked_without_partial_output_or_state_change() {
    let (_, s) = sample();
    let mut c = s.catalog().clone();
    c.archives.clear();
    c.publications.clear();
    c.published = "large".into();
    let mut index = std::collections::BTreeMap::new();
    for i in 0..8 {
        let name = format!("provider-{i}-{}", "n".repeat(50));
        let key = format!("artifact-{i}");
        let mut files = std::collections::BTreeMap::new();
        for j in 0..32 {
            files.insert(
                format!("/usr/share/{i}/{j:02}/{}", "x".repeat(140)),
                core::Payload {
                    contents: "x".into(),
                    conffile: false,
                },
            );
        }
        c.archives.insert(
            key.clone(),
            core::Archive {
                name: name.clone(),
                version: core::Version::parse("1.0-1").unwrap(),
                architecture: core::Architecture::Amd64,
                depends: vec![],
                conflicts: vec![],
                files,
            },
        );
        index.insert(name, key);
    }
    c.publications
        .insert("large".into(), core::Index { candidates: index });
    let mut large = core::State::empty(c).unwrap();
    large.apply(&core::Action::RefreshIndex {}).unwrap();
    let before = serde_json::to_vec(&large).unwrap();
    let e = contents::query(&large, &parsed("search usr/share/")).unwrap_err();
    assert_eq!(e.code, contents::Code::Limit);
    assert_eq!(serde_json::to_vec(&large).unwrap(), before);
    let narrow = contents::query(&large, &parsed("-F search usr/share/0/00/")).unwrap();
    assert_eq!(narrow.status, 1);
}

#[test]
fn leading_slash_is_a_component_boundary_and_fixed_search_is_complete_path() {
    let (_, s) = sample();
    let mut c = s.catalog().clone();
    let r = s.runtime().clone();
    c.archives.get_mut("common-1").unwrap().files = std::collections::BTreeMap::from([
        (
            "/usr/bin/probe".into(),
            core::Payload {
                contents: "a".into(),
                conffile: false,
            },
        ),
        (
            "/opt/usr/bin/probe".into(),
            core::Payload {
                contents: "b".into(),
                conffile: false,
            },
        ),
        (
            "/xusr/bin/probe".into(),
            core::Payload {
                contents: "c".into(),
                conffile: false,
            },
        ),
    ]);
    let s = core::State::new(c, r).unwrap();
    assert_eq!(
        contents::query(&s, &parsed("search usr/bin/probe"))
            .unwrap()
            .rows
            .len(),
        3
    );
    assert_eq!(
        contents::query(&s, &parsed("search /usr/bin/probe"))
            .unwrap()
            .rows
            .len(),
        2
    );
    let exact = contents::query(&s, &parsed("-F search /usr/bin/probe")).unwrap();
    assert_eq!(exact.rows.len(), 1);
    assert_eq!(exact.rows[0].path, "usr/bin/probe");
    assert_eq!(
        exact,
        contents::query(&s, &parsed("search -F usr/bin/probe")).unwrap()
    );
}
