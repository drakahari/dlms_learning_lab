//! Fixed synthetic worlds; all addresses, files, cache and transfers are in process.
use dlms_learning_lab::{
    dns::{Data, QueryType, Record, Transfer},
    dns_access::{
        config::Policy,
        world::{Facts, Key, Route, TransferReply, World},
    },
};
#[path = "support/dns_access_fixture.rs"]
mod fixture;
use fixture::{config, facts, zone};
fn request(world: &mut World, client: &str, name: &str, recursion: bool) -> Route {
    world
        .query(client.parse().unwrap(), name, QueryType::A, recursion)
        .unwrap()
        .route
}
#[test]
fn authoritative_service_and_private_recursion_are_distinct_for_multiple_clients() {
    let f = facts();
    let text = config(true, "192.0.2.0/24;", "192.0.2.0/24;", "none;", "", "");
    let mut w = World::new(&f, &text).unwrap();
    for client in [
        "192.0.2.0",
        "192.0.2.1",
        "192.0.2.254",
        "192.0.2.255",
        "198.51.100.2",
    ] {
        assert_eq!(
            request(&mut w, client, "www.public.example", true),
            Route::Authoritative
        );
    }
    assert_eq!(
        request(&mut w, "192.0.2.5", "api.external.example", true),
        Route::SuppliedRecursion
    );
    assert_eq!(
        request(&mut w, "192.0.2.6", "api.external.example", false),
        Route::Cached
    );
    for client in ["192.0.1.255", "192.0.3.0", "198.51.100.2"] {
        assert_eq!(
            request(&mut w, client, "api.external.example", true),
            Route::DeniedCache
        );
        assert_eq!(
            request(&mut w, client, "www.public.example", false),
            Route::Authoritative
        );
    }
    assert_eq!(w.zones(), f.zones);
}
#[test]
fn recursion_off_does_not_hide_a_warm_cache_leak_and_saving_is_not_reloading() {
    let f = facts();
    let open = config(false, "any;", "none;", "none;", "", "");
    let mut w = World::new(&f, &open).unwrap();
    assert_eq!(
        request(&mut w, "198.51.100.2", "www.external.example", true),
        Route::Cached
    );
    assert_eq!(
        request(&mut w, "198.51.100.2", "api.external.example", true),
        Route::NoRecursion
    );
    let restricted = config(false, "192.0.2.0/24;", "none;", "none;", "", "");
    w.save(&restricted).unwrap();
    assert_ne!(w.saved(), w.live());
    assert_eq!(
        request(&mut w, "198.51.100.2", "www.external.example", false),
        Route::Cached
    );
    w.reload();
    assert_eq!(w.saved(), w.live());
    assert_eq!(
        request(&mut w, "198.51.100.2", "www.external.example", false),
        Route::DeniedCache
    );
    assert_eq!(
        request(&mut w, "192.0.2.7", "www.external.example", false),
        Route::Cached
    );
    assert_eq!(w.cache_keys(), f.warm_cache);
}
#[test]
fn query_cache_and_recursive_access_fail_for_their_own_reasons_without_false_completion() {
    let f = facts();
    let text = config(true, "any;", "192.0.2.0/24;", "none;", "", "");
    let mut w = World::new(&f, &text).unwrap();
    assert_eq!(
        request(&mut w, "198.51.100.7", "api.external.example", true),
        Route::DeniedRecursion
    );
    assert_eq!(
        request(&mut w, "198.51.100.7", "api.external.example", false),
        Route::NoRecursion
    );
    let text = text.replace("allow-query { any; };", "allow-query { none; };");
    w.save(&text).unwrap();
    w.reload();
    assert_eq!(
        request(&mut w, "192.0.2.1", "www.public.example", true),
        Route::DeniedQuery
    );
    assert_eq!(
        request(&mut w, "192.0.2.1", "www.external.example", true),
        Route::DeniedQuery
    );
}
#[test]
fn effective_transfer_scope_protects_the_other_zone_and_copies_actual_records_by_serial() {
    let f = facts();
    let text = config(
        true,
        "none;",
        "none;",
        "any;",
        "allow-transfer { 198.51.100.2; };",
        "",
    );
    let mut w = World::new(&f, &text).unwrap();
    let before = w.clone();
    assert_eq!(
        w.transfer("public.example", "198.51.100.3".parse().unwrap())
            .unwrap(),
        TransferReply::Refused
    );
    assert_eq!(w, before);
    assert!(
        w.effective_transfer("private.example")
            .unwrap()
            .permits("203.0.113.9".parse().unwrap())
            .unwrap(),
        "The broad inherited default is a real leak, not a solved policy."
    );
    let fixed = config(
        true,
        "none;",
        "none;",
        "none;",
        "allow-transfer { 198.51.100.2; };",
        "",
    );
    w.save(&fixed).unwrap();
    w.reload();
    assert_eq!(
        w.transfer("private.example", "198.51.100.2".parse().unwrap())
            .unwrap(),
        TransferReply::Refused
    );
    assert_eq!(
        w.transfer("public.example", "198.51.100.2".parse().unwrap())
            .unwrap(),
        TransferReply::Secondary {
            decision: Transfer::Updated,
            primary_serial: 10,
            secondary_serial: 10,
            records: 4
        }
    );
    assert_eq!(w.zones()[0].server.secondary, w.zones()[0].server.primary);
    assert_eq!(w.zones()[1], f.zones[1]);
    assert_eq!(
        w.transfer("public.example", "198.51.100.2".parse().unwrap())
            .unwrap(),
        TransferReply::Secondary {
            decision: Transfer::UnchangedEqual,
            primary_serial: 10,
            secondary_serial: 10,
            records: 4
        }
    );
}
#[test]
fn alternative_global_and_zone_placements_produce_equivalent_effective_permissions() {
    let f = facts();
    let a = config(
        true,
        "192.0.2.0/24;",
        "192.0.2.0/24;",
        "none;",
        "allow-transfer { 198.51.100.2; };",
        "",
    );
    let b = config(
        true,
        "192.0.2.0/25; 192.0.2.128/25;",
        "192.0.2.128/25; 192.0.2.0/25;",
        "198.51.100.2;",
        "",
        "allow-transfer { none; };",
    );
    let mut a = World::new(&f, &a).unwrap();
    let mut b = World::new(&f, &b).unwrap();
    for name in ["public.example", "private.example"] {
        assert!(
            a.effective_transfer(name)
                .unwrap()
                .equivalent(b.effective_transfer(name).unwrap())
                .unwrap()
        );
    }
    assert!(a.live().cache.equivalent(&b.live().cache).unwrap());
    for source in ["192.0.2.1", "198.51.100.2", "203.0.113.2"] {
        for zone in ["public.example", "private.example"] {
            assert_eq!(
                a.transfer(zone, source.parse().unwrap()).unwrap(),
                b.transfer(zone, source.parse().unwrap()).unwrap()
            );
        }
    }
    assert_eq!(a.zones(), b.zones());
}
#[test]
fn unknown_names_paths_and_bad_configuration_never_touch_the_host_or_partial_state() {
    let f = facts();
    let text = config(true, "any;", "any;", "none;", "", "");
    let mut w = World::new(&f, &text).unwrap();
    let before = w.clone();
    for bad in [
        "/etc/passwd",
        "$(id)",
        "; cat /etc/passwd",
        "www.unprovided.example",
    ] {
        assert!(
            w.query("192.0.2.1".parse().unwrap(), bad, QueryType::A, true)
                .is_err()
        );
        assert_eq!(w, before);
    }
    for bad in [
        format!("{text}include \"/etc/passwd\";"),
        text.replace("allow-transfer { none; };", "allow-transfer { trusted; };"),
        text.replace("db.public", "../../etc/passwd"),
    ] {
        assert!(w.save(&bad).is_err());
        assert_eq!(w, before);
    }
    assert!(
        w.transfer("foreign.example", "192.0.2.1".parse().unwrap())
            .is_err()
    );
    assert_eq!(w, before);
    assert!(Policy::parse(&";".repeat(2049), &f.known()).is_err());
}
#[test]
fn source_validation_excludes_overlapping_forged_and_unbounded_fixtures() {
    let f = facts();
    assert!(f.validate().is_ok());
    let mut bad = f.clone();
    bad.version = 2;
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.zones.push(bad.zones[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.warm_cache.push(bad.warm_cache[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.warm_cache = vec![Key::new("www.unknown.example", QueryType::A).unwrap()];
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.upstream = vec![zone("public.example.", "192.0.2.1")];
    assert!(bad.validate().is_err());
    let mut bad = f.clone();
    bad.zones[0].server.primary.records.push(Record {
        owner: "alias.public.example.".into(),
        ttl: 300,
        data: Data::Cname {
            target: "www.external.example.".into(),
        },
    });
    assert!(bad.validate().is_err());
    for file in [
        "../../etc/passwd",
        "db\\escape",
        "db\"quote",
        "/etc/../passwd",
    ] {
        let mut bad = f.clone();
        bad.zones[0].file = file.into();
        assert!(bad.validate().is_err(), "{file}");
    }
    let data = serde_json::to_vec(&f).unwrap();
    let imported: Facts = serde_json::from_slice(&data).unwrap();
    assert_eq!(f, imported);
}
#[test]
fn cache_limit_and_clear_are_bounded_and_overflow_publishes_nothing() {
    let f = facts();
    let text = config(true, "any;", "any;", "none;", "", "");
    let mut w = World::new(&f, &text).unwrap();
    for i in 0..31 {
        assert_eq!(
            request(
                &mut w,
                "192.0.2.1",
                &format!("missing-{i}.external.example"),
                true
            ),
            Route::SuppliedRecursion
        );
    }
    assert_eq!(w.cache_keys().len(), 32);
    let before = w.clone();
    assert!(
        w.query(
            "192.0.2.1".parse().unwrap(),
            "api.external.example",
            QueryType::A,
            true
        )
        .unwrap_err()
        .contains("32 questions")
    );
    assert_eq!(w, before);
    w.clear_cache();
    assert!(w.cache_keys().is_empty());
    assert_eq!(
        request(&mut w, "192.0.2.1", "api.external.example", true),
        Route::SuppliedRecursion
    );
}

#[test]
fn effective_service_scopes_accept_equivalent_restrictions_and_case_cache_is_reproducible() {
    let f = facts();
    let a = config(true, "192.0.2.0/24;", "any;", "none;", "", "");
    let b = config(true, "any;", "192.0.2.0/24;", "none;", "", "");
    let mut a = World::new(&f, &a).unwrap();
    let mut b = World::new(&f, &b).unwrap();
    assert_eq!(
        a.effective_recursion().unwrap(),
        b.effective_recursion().unwrap()
    );
    assert_ne!(
        a.effective_cache().unwrap(),
        b.effective_cache().unwrap(),
        "A recursion-only restriction does not protect existing cache data."
    );
    assert_eq!(
        request(&mut b, "198.51.100.1", "www.external.example", false),
        Route::Cached
    );
    assert_eq!(
        request(&mut a, "198.51.100.1", "www.external.example", false),
        Route::DeniedCache
    );
    a.clear_cache();
    assert!(a.cache_keys().is_empty());
    a.reset_cache_to_supplied().unwrap();
    assert_eq!(a.cache_keys(), f.warm_cache);
    assert_eq!(a.zones(), f.zones);
}

#[test]
fn an_allowed_other_recipient_receives_data_without_updating_the_supplied_secondary() {
    let f = facts();
    let text = config(true, "none;", "none;", "any;", "", "");
    let mut w = World::new(&f, &text).unwrap();
    let before = w.clone();
    let reply = w
        .transfer("public.example", "203.0.113.44".parse().unwrap())
        .unwrap();
    let TransferReply::OtherClient { zone } = reply else {
        panic!(
            "The unrelated allowed client must receive its own data, not update another receiver."
        )
    };
    assert_eq!(zone, f.zones[0].server.primary);
    assert_eq!(w, before);
    assert_ne!(w.zones()[0].server.secondary, w.zones()[0].server.primary);
    let reply = w
        .transfer("public.example", f.zones[0].secondary_client)
        .unwrap();
    assert!(matches!(
        reply,
        TransferReply::Secondary {
            decision: Transfer::Updated,
            ..
        }
    ));
    assert_eq!(w.zones()[0].server.secondary, w.zones()[0].server.primary);
}

#[test]
fn supplied_secondary_identity_cannot_be_an_unspecified_or_multicast_receiver() {
    for address in ["0.0.0.0", "224.0.0.1", "255.255.255.255"] {
        let mut bad = facts();
        bad.zones[0].secondary_client = address.parse().unwrap();
        assert!(bad.validate().is_err(), "{address}");
    }
}
