//! Prospective source-derived repository checks. No host utilities or fixture filesystem.
use dlms_learning_lab::rpm_packages;
use dlms_learning_lab::rpm_repositories as repository;
use repository::*;
use rpm_packages as rpm;
use std::collections::BTreeMap;

fn snapshot(s: &str) -> rpm::SnapshotId {
    rpm::SnapshotId::new(s).unwrap()
}
fn archive(version: &str, text: &str) -> rpm::Archive {
    rpm::Archive {
        identity: rpm::Identity {
            name: "viewer".into(),
            arch: rpm::Architecture::Noarch,
        },
        evr: rpm::Evr::parse(version).unwrap(),
        requires: vec![],
        provides: BTreeMap::new(),
        files: BTreeMap::from([(
            "/usr/share/viewer.txt".into(),
            rpm::Payload {
                kind: rpm::FileKind::Ordinary,
                file: rpm::File {
                    contents: text.into(),
                    mode: 0o644,
                    owner: "root".into(),
                    group: "root".into(),
                    mtime: 1,
                },
            },
        )]),
    }
}
fn source() -> Source {
    Source {
        schema_version: 1,
        catalog: rpm::Catalog {
            fixture: "repository-core-fixture".into(),
            target: rpm::Architecture::X86_64,
            published: snapshot("current"),
            archives: BTreeMap::from([
                ("viewer-old".into(), archive("1.0-1", "first\n")),
                ("viewer-new".into(), archive("2.0-1", "second\n")),
            ]),
            snapshots: BTreeMap::from([
                (
                    "older".into(),
                    rpm::Snapshot {
                        archives: vec!["viewer-old".into()],
                    },
                ),
                (
                    "current".into(),
                    rpm::Snapshot {
                        archives: vec!["viewer-new".into()],
                    },
                ),
            ]),
            allowed_paths: vec!["/usr/share/viewer.txt".into()],
        },
        endpoints: vec![
            Endpoint {
                id: "updates".into(),
                uri: "file:///virtual-repositories/updates".into(),
                delivery: Delivery::Available,
                published: snapshot("current"),
                history: vec![snapshot("older"), snapshot("current")],
            },
            Endpoint {
                id: "archive".into(),
                uri: "file:///virtual-repositories/archive".into(),
                delivery: Delivery::Available,
                published: snapshot("older"),
                history: vec![snapshot("older")],
            },
        ],
    }
}
fn configuration(enabled: bool) -> Configuration {
    Configuration {
        alias: "updates".into(),
        uri: "file:///virtual-repositories/updates".into(),
        enabled,
    }
}
fn empty(s: &Source, enabled: bool) -> State {
    State::new(s, vec![configuration(enabled)], vec![]).unwrap()
}
fn old(s: &Source, enabled: bool) -> State {
    State::new(
        s,
        vec![configuration(enabled)],
        vec![Cached {
            alias: "updates".into(),
            endpoint: "updates".into(),
            snapshot: snapshot("older"),
        }],
    )
    .unwrap()
}
fn refresh() -> Action {
    Action::Refresh {
        alias: "updates".into(),
    }
}
fn json<T: serde::Serialize>(s: &T) -> serde_json::Value {
    serde_json::to_value(s).unwrap()
}
fn bytes<T: serde::Serialize>(s: &T) -> Vec<u8> {
    serde_json::to_vec(s).unwrap()
}

#[test]
fn missing_cache_is_distinct_from_no_matching_package_and_never_populates_itself() {
    let s = source();
    let state = empty(&s, true);
    let original = state.clone();
    let q = state.query_exact(&s, "viewer").unwrap();
    assert!(q.rows.is_empty());
    assert_eq!(q.omitted, vec![("updates", Omission::MissingCache)]);
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Missing
    );
    assert_eq!(state, original);
}
#[test]
fn refresh_changes_cached_metadata_and_rows_not_immutable_rpm_payload_or_catalog() {
    let s = source();
    let mut state = old(&s, true);
    let immutable = s.clone();
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Older
    );
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().rows[0]
            .archive
            .evr
            .as_str(),
        "1.0-1"
    );
    state.apply(&s, &refresh()).unwrap();
    let q = state.query_exact(&s, "viewer").unwrap();
    assert_eq!(q.rows.len(), 1);
    assert_eq!(q.rows[0].archive_id, "viewer-new");
    assert_eq!(q.rows[0].endpoint, "updates");
    assert_eq!(q.rows[0].snapshot.as_str(), "current");
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Current
    );
    assert_eq!(s, immutable);
    assert_eq!(json(&state)["source"], json(&immutable));
    assert_eq!(
        json(&state)
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["caches", "configurations", "schema_version", "source"]
    );
}
#[test]
fn default_refresh_of_disabled_repository_refuses_atomically_then_enabled_retry_succeeds() {
    let s = source();
    let mut state = empty(&s, false);
    let before = state.clone();
    assert_eq!(
        state.apply(&s, &refresh()).unwrap_err().code,
        Code::Disabled
    );
    assert_eq!(state, before);
    let inspect = state.inspect(&s, "updates").unwrap();
    assert!(!inspect.configuration.enabled);
    assert_eq!(inspect.cache_status, CacheStatus::Missing);
    let q = state.query_exact(&s, "viewer").unwrap();
    assert!(q.rows.is_empty());
    assert_eq!(q.omitted, vec![("updates", Omission::Disabled)]);
    state
        .apply(
            &s,
            &Action::SetEnabled {
                alias: "updates".into(),
                enabled: true,
            },
        )
        .unwrap();
    state.apply(&s, &refresh()).unwrap();
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().rows[0].archive_id,
        "viewer-new"
    );
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Current
    );
}
#[test]
fn enabling_and_repairing_uri_in_either_order_before_refresh_reaches_same_state() {
    let s = source();
    let mut a = empty(&s, false);
    a.apply(
        &s,
        &Action::SetUri {
            alias: "updates".into(),
            uri: "file:///virtual-repositories/wrong".into(),
        },
    )
    .unwrap();
    let mut b = a.clone();
    let enable = Action::SetEnabled {
        alias: "updates".into(),
        enabled: true,
    };
    let repair = Action::SetUri {
        alias: "updates".into(),
        uri: s.endpoints[0].uri.clone(),
    };
    a.apply(&s, &enable).unwrap();
    a.apply(&s, &repair).unwrap();
    b.apply(&s, &repair).unwrap();
    b.apply(&s, &enable).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Missing
    );
    a.apply(&s, &refresh()).unwrap();
    b.apply(&s, &refresh()).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a.query_exact(&s, "viewer").unwrap().rows[0].archive_id,
        "viewer-new"
    );
}
#[test]
fn source_uri_change_retains_old_inspection_but_refuses_wrong_provenance_rows() {
    let s = source();
    let mut state = old(&s, true);
    state
        .apply(
            &s,
            &Action::SetUri {
                alias: "updates".into(),
                uri: s.endpoints[1].uri.clone(),
            },
        )
        .unwrap();
    let inspect = state.inspect(&s, "updates").unwrap();
    assert_eq!(inspect.cache_status, CacheStatus::WrongSource);
    assert_eq!(inspect.cached.unwrap().endpoint, "updates");
    assert_eq!(inspect.endpoint.unwrap().id, "archive");
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().omitted,
        vec![("updates", Omission::WrongSource)]
    );
    state.apply(&s, &refresh()).unwrap();
    let q = state.query_exact(&s, "viewer").unwrap();
    assert_eq!(q.rows[0].endpoint, "archive");
    assert_eq!(q.rows[0].archive_id, "viewer-old");
}
#[test]
fn unknown_uri_can_be_diagnosed_and_repaired_without_mutating_old_cache() {
    let s = source();
    let mut state = old(&s, true);
    state
        .apply(
            &s,
            &Action::SetUri {
                alias: "updates".into(),
                uri: "file:///virtual-repositories/typo".into(),
            },
        )
        .unwrap();
    let before = state.clone();
    assert!(state.inspect(&s, "updates").unwrap().endpoint.is_none());
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().omitted,
        vec![("updates", Omission::UnknownSource)]
    );
    assert_eq!(
        state.apply(&s, &refresh()).unwrap_err().code,
        Code::UnknownSource
    );
    assert_eq!(state, before);
    state
        .apply(
            &s,
            &Action::SetUri {
                alias: "updates".into(),
                uri: s.endpoints[0].uri.clone(),
            },
        )
        .unwrap();
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Older
    );
}
#[test]
fn unavailable_source_does_not_invalidate_usable_cache_and_refresh_failure_is_atomic() {
    let mut s = source();
    s.endpoints[0].delivery = Delivery::Unavailable;
    let mut state = old(&s, true);
    let before = state.clone();
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().rows[0].archive_id,
        "viewer-old"
    );
    assert_eq!(
        state.apply(&s, &refresh()).unwrap_err().code,
        Code::Unavailable
    );
    assert_eq!(state, before);
}
#[test]
fn two_repositories_keep_distinct_alias_snapshot_and_endpoint_provenance() {
    let s = source();
    let mut state = State::new(
        &s,
        vec![
            configuration(true),
            Configuration {
                alias: "historic".into(),
                uri: s.endpoints[1].uri.clone(),
                enabled: true,
            },
        ],
        vec![],
    )
    .unwrap();
    state.apply(&s, &refresh()).unwrap();
    assert_eq!(
        state.inspect(&s, "historic").unwrap().cache_status,
        CacheStatus::Missing
    );
    state
        .apply(
            &s,
            &Action::Refresh {
                alias: "historic".into(),
            },
        )
        .unwrap();
    let q = state.query_exact(&s, "viewer").unwrap();
    assert_eq!(
        q.rows
            .iter()
            .map(|r| (r.alias, r.archive_id))
            .collect::<Vec<_>>(),
        vec![("updates", "viewer-new"), ("historic", "viewer-old")]
    );
    assert!(
        state
            .query_exact(&s, "not-present")
            .unwrap()
            .rows
            .is_empty()
    );
}
#[test]
fn disabling_after_success_removes_current_results_despite_historical_cache() {
    let s = source();
    let mut state = empty(&s, true);
    state.apply(&s, &refresh()).unwrap();
    assert_eq!(state.query_exact(&s, "viewer").unwrap().rows.len(), 1);
    state
        .apply(
            &s,
            &Action::SetEnabled {
                alias: "updates".into(),
                enabled: false,
            },
        )
        .unwrap();
    assert!(state.query_exact(&s, "viewer").unwrap().rows.is_empty());
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Current
    );
}
#[test]
fn context_compares_every_source_field_not_only_id_or_cached_success() {
    let s = source();
    let state = old(&s, true);
    for mode in 0..5 {
        let mut other = s.clone();
        match mode {
            0 => other.catalog.fixture = "foreign".into(),
            1 => other.endpoints[0].delivery = Delivery::Unavailable,
            2 => other.endpoints[0].published = snapshot("older"),
            3 => {
                other
                    .catalog
                    .archives
                    .get_mut("viewer-old")
                    .unwrap()
                    .files
                    .values_mut()
                    .next()
                    .unwrap()
                    .file
                    .contents = "foreign\n".into()
            }
            _ => other.endpoints[0].uri = "file:///virtual-repositories/other".into(),
        }
        assert!(other.validate().is_ok());
        assert_eq!(
            state.validate_context(&other).unwrap_err().code,
            Code::Context
        );
        assert!(state.query_exact(&other, "viewer").is_err());
        let mut changed = state.clone();
        assert!(changed.apply(&other, &refresh()).is_err());
        assert_eq!(changed, state);
    }
}
#[test]
fn unknown_alias_invalid_name_and_oversized_uri_fail_before_any_mutation() {
    let s = source();
    let mut state = old(&s, true);
    let before = state.clone();
    for action in [
        Action::Refresh {
            alias: "not-configured".into(),
        },
        Action::Refresh {
            alias: "../updates".into(),
        },
        Action::SetUri {
            alias: "updates".into(),
            uri: format!("https://{}", "x".repeat(161)),
        },
        Action::SetUri {
            alias: "updates".into(),
            uri: "https://bad\nuri".into(),
        },
    ] {
        assert!(state.apply(&s, &action).is_err());
        assert_eq!(state, before);
    }
    assert!(state.query_exact(&s, "viewer*").is_err());
}
#[test]
fn duplicate_configurations_foreign_caches_and_source_history_are_rejected() {
    let s = source();
    let state = old(&s, true);
    let raw = json(&state);
    for mode in 0..7 {
        let mut value = raw.clone();
        match mode {
            0 => {
                let duplicate = value["configurations"][0].clone();
                value["configurations"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            }
            1 => value["caches"][0]["alias"] = "not-configured".into(),
            2 => value["caches"][0]["endpoint"] = "foreign".into(),
            3 => value["caches"][0]["snapshot"] = "foreign".into(),
            4 => {
                value["source"]["endpoints"][1]["uri"] =
                    value["source"]["endpoints"][0]["uri"].clone()
            }
            5 => value["source"]["endpoints"][1]["published"] = "current".into(),
            _ => {
                let duplicate = value["caches"][0].clone();
                value["caches"].as_array_mut().unwrap().push(duplicate);
            }
        }
        assert!(State::decode(&bytes(&value), &s).is_err());
    }
}
#[test]
fn raw_limits_nested_limits_unknown_fields_and_duplicate_json_fields_fail_closed() {
    let s = source();
    let state = old(&s, true);
    let raw = String::from_utf8(bytes(&state)).unwrap();
    assert!(State::decode(&vec![b' '; MAX_RAW + 1], &s).is_err());
    assert!(Source::decode(&vec![b' '; MAX_RAW + 1]).is_err());
    assert!(State::decode(raw.replacen("{", "{\"unknown\":1,", 1).as_bytes(), &s).is_err());
    assert!(
        State::decode(
            raw.replacen(
                "\"schema_version\":1",
                "\"schema_version\":1,\"schema_version\":1",
                1
            )
            .as_bytes(),
            &s
        )
        .is_err()
    );
    let mut value = json(&state);
    value["configurations"][0]["alias"] = "x".repeat(33).into();
    assert!(State::decode(&bytes(&value), &s).is_err());
    let mut value = json(&state);
    value["configurations"][0]["uri"] = format!("https://{}", "é".repeat(80)).into();
    assert!(State::decode(&bytes(&value), &s).is_err());
    let mut value = json(&state);
    value["configurations"]
        .as_array_mut()
        .unwrap()
        .extend([json(&configuration(true)), json(&configuration(true))]);
    assert!(State::decode(&bytes(&value), &s).is_err());
}
#[test]
fn source_size_uses_escaped_utf8_serialization_and_refuses_before_state_clone() {
    let mut s = source();
    for a in s.catalog.archives.values_mut() {
        a.files.values_mut().next().unwrap().file.contents = "\0".repeat(4096);
    }
    // Two legal 4096-byte RPM contents escape to >48KiB and exceed this source's32KiB cap.
    assert!(s.catalog.validate().is_ok());
    assert_eq!(s.validate().unwrap_err().code, Code::Limit);
    assert!(State::new(&s, vec![configuration(true)], vec![]).is_err());
}
#[test]
fn valid_direct_and_decoded_sources_roundtrip_without_fabricating_cache_or_work() {
    let s = source();
    assert_eq!(Source::decode(&bytes(&s)).unwrap(), s);
    let state = old(&s, false);
    let raw = bytes(&state);
    assert_eq!(State::decode(&raw, &s).unwrap(), state);
    assert_eq!(bytes(&State::decode(&raw, &s).unwrap()), raw);
    assert_eq!(
        State::decode(&raw, &s)
            .unwrap()
            .inspect_all(&s)
            .unwrap()
            .len(),
        1
    );
    let fresh = empty(&s, false);
    assert_ne!(fresh, state);
    assert_eq!(
        fresh.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Missing
    );
}
#[test]
fn source_admission_rejects_unsupported_architecture_foreign_ids_and_invalid_delivery_shape() {
    let s = source();
    for mode in 0..6 {
        let mut bad = s.clone();
        match mode {
            0 => bad.schema_version = 2,
            1 => {
                bad.catalog
                    .archives
                    .get_mut("viewer-old")
                    .unwrap()
                    .identity
                    .arch = rpm::Architecture::X86_64
            }
            2 => bad.endpoints[0].history.push(snapshot("foreign")),
            3 => bad.endpoints[0].history = vec![snapshot("current"), snapshot("current")],
            4 => bad.endpoints[0].id = "../source".into(),
            _ => bad
                .catalog
                .snapshots
                .get_mut("older")
                .unwrap()
                .archives
                .push("foreign".into()),
        }
        assert!(bad.validate().is_err());
        assert!(Source::decode(&bytes(&bad)).is_err());
    }
    let mut value = json(&s);
    value["endpoints"][0]["delivery"] = "trusted-gpg".into();
    assert!(Source::decode(&bytes(&value)).is_err());
}
#[test]
fn direct_deserialized_poison_cannot_use_public_views_or_actions() {
    let s = source();
    let original = old(&s, true);
    let mut value = json(&original);
    value["schema_version"] = 2.into();
    let mut poison: State = serde_json::from_value(value).unwrap();
    assert!(poison.validate().is_err());
    assert!(poison.inspect(&s, "updates").is_err());
    assert!(poison.inspect_all(&s).is_err());
    assert!(poison.query_exact(&s, "viewer").is_err());
    let before = poison.clone();
    assert!(poison.apply(&s, &refresh()).is_err());
    assert_eq!(poison, before);
    let mut invalid_source = s.clone();
    invalid_source.schema_version = 2;
    assert!(original.validate_context(&invalid_source).is_err());
}

#[test]
fn escaped_source_and_full_configuration_cache_bounds_remain_roundtrippable_after_growth() {
    let mut s = source();
    for archive in s.catalog.archives.values_mut() {
        archive.files.values_mut().next().unwrap().file.contents = "\0".repeat(1800);
    }
    assert!(s.validate().is_ok());
    let configurations = vec![
        configuration(true),
        Configuration {
            alias: "a".repeat(32),
            uri: s.endpoints[1].uri.clone(),
            enabled: false,
        },
    ];
    let caches = vec![
        Cached {
            alias: "updates".into(),
            endpoint: "updates".into(),
            snapshot: snapshot("older"),
        },
        Cached {
            alias: "a".repeat(32),
            endpoint: "archive".into(),
            snapshot: snapshot("older"),
        },
    ];
    let mut state = State::new(&s, configurations, caches).unwrap();
    let longest_uri = format!("https://{}", "x".repeat(152));
    assert_eq!(longest_uri.len(), 160);
    state
        .apply(
            &s,
            &Action::SetUri {
                alias: "updates".into(),
                uri: longest_uri,
            },
        )
        .unwrap();
    assert!(bytes(&state).len() < MAX_STATE);
    assert_eq!(State::decode(&bytes(&state), &s).unwrap(), state);
    assert_eq!(state.inspect_all(&s).unwrap().len(), 2);
}

#[test]
fn disabled_default_refresh_preserves_real_older_cache_and_a_later_enabled_retry_updates_it() {
    let s = source();
    let mut state = old(&s, false);
    let before = state.clone();
    let raw = bytes(&state);
    assert_eq!(
        state.inspect(&s, "updates").unwrap().cache_status,
        CacheStatus::Older
    );
    assert_eq!(
        state.apply(&s, &refresh()).unwrap_err().code,
        Code::Disabled
    );
    assert_eq!(state, before);
    assert_eq!(bytes(&state), raw);
    assert_eq!(
        state
            .inspect(&s, "updates")
            .unwrap()
            .cached
            .unwrap()
            .snapshot
            .as_str(),
        "older"
    );
    assert!(state.query_exact(&s, "viewer").unwrap().rows.is_empty());
    state
        .apply(
            &s,
            &Action::SetEnabled {
                alias: "updates".into(),
                enabled: true,
            },
        )
        .unwrap();
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().rows[0].archive_id,
        "viewer-old"
    );
    state.apply(&s, &refresh()).unwrap();
    assert_eq!(
        state.query_exact(&s, "viewer").unwrap().rows[0].archive_id,
        "viewer-new"
    );
}
