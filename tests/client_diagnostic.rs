use dlms_learning_lab::{client_diagnostic as d, client_names as n, resolver_search as s};
use std::collections::BTreeMap;
fn context() -> n::Context {
    n::Context {
        files_available: true,
        dns: n::Dns {
            status: n::Status::Success,
            records: BTreeMap::from([
                ("app.north.test".into(), "192.0.2.80".parse().unwrap()),
                ("app.south.test".into(), "192.0.2.81".parse().unwrap()),
                ("remote.north.test".into(), "192.0.2.82".parse().unwrap()),
            ]),
        },
    }
}
fn resolve(policy: &str, hosts: &str, settings: &str, q: &str) -> d::Resolution {
    d::resolve(
        &n::Switch::parse(policy).unwrap(),
        &n::Hosts::parse(hosts).unwrap(),
        &context(),
        &s::Settings::parse(settings, "client.north.test").unwrap(),
        q,
    )
    .unwrap()
}
#[test]
fn files_sees_original_name_once_dns_search_is_inside_its_backend() {
    // Looking up each DNS candidate through the whole NSS chain would incorrectly
    // select the hosts-only app.north.test record instead of actual DNS .80.
    let r = resolve(
        "hosts: files dns",
        "192.0.2.40 app.north.test app-local",
        "search north.test south.test",
        "app",
    );
    assert_eq!(r.address, Some("192.0.2.80".parse().unwrap()));
    assert_eq!(r.steps.len(), 2);
    assert_eq!(r.steps[0].status, n::Status::NotFound);
    assert_eq!(r.dns_attempts.len(), 1);
    assert_eq!(r.canonical.as_deref(), Some("app.north.test"));
    let local = resolve(
        "hosts: files dns",
        "192.0.2.40 App.North.Test app-local",
        "search north.test",
        "APP-LOCAL",
    );
    assert_eq!(local.canonical.as_deref(), Some("App.North.Test"));
    assert_eq!(local.address, Some("192.0.2.40".parse().unwrap()));
    assert!(local.dns_attempts.is_empty());
}
#[test]
fn source_order_search_order_and_absolute_names_have_different_effects() {
    let hosts = "192.0.2.40 app";
    let local = resolve(
        "hosts: files dns",
        hosts,
        "search south.test north.test",
        "app",
    );
    assert_eq!(local.answered_by, Some(n::Service::Files));
    let remote = resolve(
        "hosts: dns files",
        hosts,
        "search south.test north.test",
        "app",
    );
    assert_eq!(remote.address, Some("192.0.2.81".parse().unwrap()));
    let absolute = resolve(
        "hosts: files dns",
        hosts,
        "search north.test",
        "remote.north.test.",
    );
    assert_eq!(absolute.address, Some("192.0.2.82".parse().unwrap()));
    assert_eq!(absolute.dns_attempts.len(), 1);
    let absent = resolve("hosts: files dns", hosts, "search north.test", "app.");
    assert_eq!(absent.address, None);
    assert_eq!(absent.dns_attempts[0].name, "app");
}
#[test]
fn explicit_return_and_missing_answers_are_not_success() {
    let r = resolve(
        "hosts: files [NOTFOUND=return] dns",
        "",
        "search north.test",
        "app",
    );
    assert_eq!(r.status, n::Status::NotFound);
    assert_eq!(r.address, None);
    assert_eq!(r.steps.len(), 1);
    assert!(r.dns_attempts.is_empty());
    let r = resolve(
        "hosts: dns files",
        "192.0.2.50 rescue",
        "search north.test",
        "rescue",
    );
    assert_eq!(r.answered_by, Some(n::Service::Files));
    assert_eq!(r.dns_attempts.len(), 2);
}
#[test]
fn irrelevant_dns_limits_and_unavailability_do_not_block_a_local_answer() {
    let mut c = context();
    c.dns.status = n::Status::Unavailable;
    c.dns.records.clear();
    let switch = n::Switch::parse("hosts: files dns").unwrap();
    let hosts = n::Hosts::parse("192.0.2.40 rescue").unwrap();
    let settings = s::Settings::parse("search north.test", "client.north.test").unwrap();
    assert!(
        d::resolve(&switch, &hosts, &c, &settings, "rescue")
            .unwrap()
            .address
            .is_some()
    );
    assert_eq!(
        d::resolve(&switch, &hosts, &c, &settings, "other")
            .unwrap_err()
            .kind,
        n::Kind::Unsupported
    );
    let long = [
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(60),
    ]
    .join(".");
    let hosts = n::Hosts::parse(&format!("192.0.2.41 {long}")).unwrap();
    assert!(
        d::resolve(&switch, &hosts, &context(), &settings, &long)
            .unwrap()
            .address
            .is_some()
    );
    let dns_first = n::Switch::parse("hosts: dns files").unwrap();
    assert_eq!(
        d::resolve(&dns_first, &hosts, &context(), &settings, &long)
            .unwrap_err()
            .kind,
        n::Kind::Limit
    );
}
#[test]
fn getent_formats_actual_canonical_provenance_and_missing_exit_status() {
    let hosts = n::Hosts::parse("192.0.2.40 Canon.North.Test alias").unwrap();
    let switch = n::Switch::parse("hosts: files dns").unwrap();
    let settings = s::Settings::parse("search north.test", "client.north.test").unwrap();
    let local = d::getent(
        &switch,
        &hosts,
        &context(),
        &settings,
        "getent ahostsv4 ALIAS",
    )
    .unwrap();
    assert_eq!(local.status, 0);
    assert_eq!(
        local.stdout,
        "192.0.2.40      STREAM Canon.North.Test\n192.0.2.40      DGRAM  \n192.0.2.40      RAW    \n"
    );
    let dns = d::getent(
        &switch,
        &hosts,
        &context(),
        &settings,
        "getent ahostsv4 remote",
    )
    .unwrap();
    assert!(dns.stdout.contains("remote.north.test"));
    let missing = d::getent(
        &switch,
        &hosts,
        &context(),
        &settings,
        "getent ahostsv4 missing",
    )
    .unwrap();
    assert_eq!((missing.status, missing.stdout.as_str()), (2, ""));
    assert_eq!(missing.resolution.unwrap().status, n::Status::NotFound);
    let numeric = d::getent(
        &switch,
        &hosts,
        &context(),
        &settings,
        "getent ahostsv4 198.51.100.1",
    )
    .unwrap();
    assert_eq!(numeric.status, 0);
    assert!(numeric.resolution.is_none());
    assert!(numeric.stdout.contains("198.51.100.1"));
}
#[test]
fn parser_honestly_refuses_unmodeled_syntax_and_never_interprets_a_shell() {
    for input in [
        "getent hosts app",
        "getent ahosts app",
        "getent ahostsv4",
        "getent -s files ahostsv4 app",
        "getent ahostsv4 app remote",
        "getent ahostsv4 127.1",
        "getent ahostsv4 010.0.0.1",
        "getent ahostsv4 0x7f000001",
        "getent ahostsv4 0x7f.1",
    ] {
        assert_eq!(
            d::parse_getent(input).unwrap_err().kind,
            n::Kind::Unsupported,
            "{input}"
        );
    }
    for input in [
        "getent ahostsv4 $(id)",
        "getent ahostsv4 ../../etc/passwd",
        "getent ahostsv4 app;id",
        "getent ahostsv4 ::1",
        "getent ahostsv4 a..b",
        "getent ahostsv4 a\0",
    ] {
        assert!(d::parse_getent(input).is_err(), "{input}");
    }
    assert!(d::parse_getent(&"x".repeat(d::MAX_COMMAND + 1)).is_err());
    // An ordinary DNS label beginning in 0x is not a legacy numeric literal.
    assert!(matches!(
        d::parse_getent("getent ahostsv4 0x-tools.test").unwrap(),
        d::Query::Name(_)
    ));
    assert!(matches!(
        d::parse_getent("\tgetent\tahostsv4\tAPP. ").unwrap(),
        d::Query::Name(_)
    ));
}
