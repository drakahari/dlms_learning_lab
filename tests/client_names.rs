use dlms_learning_lab::client_names as n;
use std::{collections::BTreeMap, net::Ipv4Addr};
fn ip(s: &str) -> Ipv4Addr {
    s.parse().unwrap()
}
fn context(status: n::Status) -> n::Context {
    n::Context {
        files_available: true,
        dns: n::Dns {
            status,
            records: if status == n::Status::Success {
                BTreeMap::from([
                    ("portal.lab.test".into(), ip("192.0.2.90")),
                    ("remote.lab.test".into(), ip("192.0.2.91")),
                ])
            } else {
                BTreeMap::new()
            },
        },
    }
}
fn run(policy: &str, hosts: &str, c: &n::Context, q: &str) -> n::Lookup {
    n::lookup(
        &n::Switch::parse(policy).unwrap(),
        &n::Hosts::parse(hosts).unwrap(),
        c,
        q,
    )
    .unwrap()
}
#[test]
fn source_order_and_default_return_are_actual_lookups_not_expected_strings() {
    let hosts = "192.0.2.50 portal.lab.test portal\n";
    let c = context(n::Status::Success);
    let a = run("hosts: files dns", hosts, &c, "Portal.LaB.TeSt");
    assert_eq!(a.address, Some(ip("192.0.2.50")));
    assert_eq!(a.steps.len(), 1);
    let a = run("hosts: dns files", hosts, &c, "portal.lab.test");
    assert_eq!(a.address, Some(ip("192.0.2.90")));
    assert_eq!(a.steps.len(), 1);
    assert_eq!(
        run("hosts: files dns", hosts, &c, "PORTAL").address,
        Some(ip("192.0.2.50"))
    );
}
#[test]
fn notfound_falls_through_unless_that_source_explicitly_returns() {
    let c = context(n::Status::Success);
    let a = run("hosts: files dns", "", &c, "remote.lab.test");
    assert_eq!(a.address, Some(ip("192.0.2.91")));
    assert_eq!(a.steps[0].status, n::Status::NotFound);
    assert_eq!(a.steps[0].action, n::Action::Continue);
    let a = run(
        "hosts: files [NOTFOUND=return] dns",
        "",
        &c,
        "remote.lab.test",
    );
    assert_eq!(a.address, None);
    assert_eq!(a.status, n::Status::NotFound);
    assert_eq!(a.steps.len(), 1);
}
#[test]
fn unavailable_and_retrying_are_distinct_and_do_not_silently_succeed() {
    for status in [n::Status::Unavailable, n::Status::TryAgain] {
        let c = context(status);
        let a = run(
            "hosts: dns files",
            "192.0.2.60 remote.lab.test",
            &c,
            "remote.lab.test",
        );
        assert_eq!(a.steps[0].status, status);
        assert_eq!(a.address, Some(ip("192.0.2.60")));
        let rule = if status == n::Status::Unavailable {
            "UNAVAIL=return"
        } else {
            "TRYAGAIN=return"
        };
        let a = run(
            &format!("hosts: dns [{rule}] files"),
            "192.0.2.60 remote.lab.test",
            &c,
            "remote.lab.test",
        );
        assert_eq!(a.address, None);
        assert_eq!(a.status, status);
        assert_eq!(a.steps.len(), 1);
    }
}
#[test]
fn success_miss_and_unavailable_cannot_be_confused() {
    let c = context(n::Status::Success);
    let a = run("hosts: files dns", "", &c, "unknown.lab.test");
    assert_eq!(a.address, None);
    assert_eq!(a.status, n::Status::NotFound);
    assert_eq!(a.steps.len(), 2);
    let mut c = c;
    c.files_available = false;
    let a = run(
        "hosts: files [UNAVAIL=return] dns",
        "192.0.2.55 portal.lab.test",
        &c,
        "portal.lab.test",
    );
    assert_eq!(a.status, n::Status::Unavailable);
    assert_eq!(a.address, None);
    assert_eq!(a.steps.len(), 1);
}
#[test]
fn equivalent_comments_spacing_explicit_defaults_and_keyword_case_work() {
    let c = context(n::Status::Success);
    let a = run(
        "# only a hosts excerpt\nhosts : files[notfound=CONTINUE SUCCESS=RETURN] dns # source order\n",
        "# sample\n192.0.2.50\tportal.lab.test\tportal # aliases\n",
        &c,
        "remote.lab.test",
    );
    assert_eq!(a.address, Some(ip("192.0.2.91")));
    assert_eq!(a.steps.len(), 2);
}
#[test]
fn unsupported_service_actions_full_database_and_ambiguous_rows_refuse() {
    for raw in [
        "hosts: resolve files",
        "passwd: files",
        "hosts: files dns files",
        "hosts: files [!UNAVAIL=return] dns",
        "hosts: files [SUCCESS=continue] dns",
        "hosts: files [SUCCESS=merge] dns",
        "hosts: files [BANNED=return] dns",
        "hosts: files\nhosts: dns",
    ] {
        assert_eq!(
            n::Switch::parse(raw).unwrap_err().kind,
            n::Kind::Unsupported,
            "{raw}"
        );
    }
    for raw in [
        "192.0.2.1 app\n192.0.2.2 APP",
        "::1 localhost",
        "127.1 localhost",
    ] {
        assert_eq!(
            n::Hosts::parse(raw).unwrap_err().kind,
            n::Kind::Unsupported,
            "{raw}"
        );
    }
}
#[test]
fn malformed_and_control_input_never_claim_an_answer() {
    for raw in [
        "",
        "hosts:",
        "hosts files dns",
        "hosts: [NOTFOUND=return] files",
        "hosts: files [NOTFOUND=return",
        "hosts: files []",
    ] {
        assert_eq!(
            n::Switch::parse(raw).unwrap_err().kind,
            n::Kind::Invalid,
            "{raw}"
        );
    }
    for raw in [
        "192.0.2.1",
        "192.0.2.1 a..b",
        "192.0.2.1 ../../etc/passwd",
        "192.0.2.1 a;id",
    ] {
        assert_eq!(n::Hosts::parse(raw).unwrap_err().kind, n::Kind::Invalid);
    }
    for raw in ["hosts: files\0", "hosts: files\u{1b}[2J"] {
        assert_eq!(n::Switch::parse(raw).unwrap_err().kind, n::Kind::Invalid);
    }
}
#[test]
fn admission_limits_are_applied_before_parsing_and_expansion() {
    assert_eq!(
        n::Hosts::parse(&"#".repeat(n::MAX_CONFIG + 1))
            .unwrap_err()
            .kind,
        n::Kind::Limit
    );
    assert_eq!(
        n::Switch::parse(&"\n".repeat(129)).unwrap_err().kind,
        n::Kind::Limit
    );
    let rows = (0..65)
        .map(|i| format!("192.0.2.1 h{i}\n"))
        .collect::<String>();
    assert_eq!(n::Hosts::parse(&rows).unwrap_err().kind, n::Kind::Limit);
    let row = "192.0.2.1 a b c d e f g h i";
    assert_eq!(n::Hosts::parse(row).unwrap_err().kind, n::Kind::Limit);
    let c = context(n::Status::Success);
    for q in [
        "192.0.2.1",
        "$(id)",
        "../hosts",
        "portal.lab.test.",
        "a/b",
        "",
    ] {
        assert!(
            n::lookup(
                &n::Switch::parse("hosts: files dns").unwrap(),
                &n::Hosts::parse("").unwrap(),
                &c,
                q
            )
            .is_err(),
            "{q}"
        );
    }
}
#[test]
fn bounded_dns_inputs_reject_duplicate_keys_case_foreign_and_inconsistent_answers() {
    for raw in [
        r#"{"files_available":true,"dns":{"status":"Success","records":{"a.lab.test":"192.0.2.1","a.lab.test":"192.0.2.2"}}}"#,
        r#"{"files_available":true,"dns":{"status":"Success","records":{"A.lab.test":"192.0.2.1"}}}"#,
        r#"{"files_available":true,"dns":{"status":"Success","records":{"../private":"192.0.2.1"}}}"#,
    ] {
        assert!(serde_json::from_str::<n::Context>(raw).is_err());
    }
    let mut c = context(n::Status::Unavailable);
    c.dns.records.insert("app.lab.test".into(), ip("192.0.2.1"));
    assert!(c.validate().is_err());
    let map = (0..65)
        .map(|i| format!("\"h{i}.lab.test\":\"192.0.2.1\""))
        .collect::<Vec<_>>()
        .join(",");
    let raw = format!(
        "{{\"files_available\":true,\"dns\":{{\"status\":\"Success\",\"records\":{{{map}}}}}}}"
    );
    assert!(serde_json::from_str::<n::Context>(&raw).is_err());
}

#[test]
fn malformed_large_tokens_have_bounded_useful_diagnostics() {
    let error = n::Switch::parse(&format!("hosts: {}", "X".repeat(12_000))).unwrap_err();
    assert_eq!(error.kind, n::Kind::Unsupported);
    assert!(error.message.len() < 256);
    assert!(error.message.contains("only files and dns"));
    let error = n::Hosts::parse(&format!("192.0.2.1 {}", "字".repeat(3_000))).unwrap_err();
    assert_eq!(error.kind, n::Kind::Invalid);
    assert!(error.message.len() < 400);
    assert!(error.message.contains("ordinary labels"));
}

#[test]
fn malformed_ipv4_feedback_differs_from_a_valid_but_unsupported_address_family() {
    for address in ["192.0.2.999", "192.0.x.1", "192.0.2."] {
        let e = n::Hosts::parse(&format!("{address} portal")).unwrap_err();
        assert_eq!(e.kind, n::Kind::Invalid);
        assert!(e.message.contains("four decimal octets"));
    }
    let e = n::Hosts::parse("2001:db8::1 portal").unwrap_err();
    assert_eq!(e.kind, n::Kind::Unsupported);
    assert!(e.message.contains("IPv6"));
}
