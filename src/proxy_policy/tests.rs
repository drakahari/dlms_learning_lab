use super::*;
fn request(ip: &str, method: &str, port: u16) -> Request {
    Request {
        client: ip.parse().unwrap(),
        method: method.into(),
        destination_port: port,
    }
}
fn action(text: &str, ip: &str, method: &str, port: u16) -> Action {
    Policy::parse(text)
        .unwrap()
        .evaluate(&request(ip, method, port))
        .unwrap()
        .action
}
#[test]
fn source_masks_host_bits_and_handles_octet_boundary_zero_and_host_prefixes() {
    let p = "acl staff src 10.48.23.87/20\nhttp_access allow staff\nhttp_access deny all\n";
    for ip in ["10.48.16.0", "10.48.23.87", "10.48.31.255"] {
        assert_eq!(action(p, ip, "GET", 80), Action::Allow);
    }
    for ip in ["10.48.15.255", "10.48.32.0", "10.49.23.87"] {
        assert_eq!(action(p, ip, "GET", 80), Action::Deny);
    }
    assert_eq!(
        action(
            "acl internet src 0.0.0.0/0\nhttp_access allow internet",
            "203.0.113.1",
            "GET",
            80
        ),
        Action::Allow
    );
    assert_eq!(
        action(
            "acl one src 192.0.2.3\nhttp_access allow one",
            "192.0.2.4",
            "GET",
            80
        ),
        Action::Deny
    );
}
#[test]
fn named_values_are_or_rule_terms_and_order_first_match() {
    let p = "acl staff src 10.1.0.0/24\nacl staff src 10.2.0.0/24\nacl web port 80 443\nhttp_access allow staff web\nhttp_access deny all";
    assert_eq!(action(p, "10.2.0.1", "GET", 443), Action::Allow);
    assert_eq!(action(p, "10.3.0.1", "GET", 443), Action::Deny);
    assert_eq!(action(p, "10.1.0.1", "GET", 25), Action::Deny);
    assert_eq!(
        action(
            &format!("http_access allow all\n{p}"),
            "198.51.100.3",
            "GET",
            25
        ),
        Action::Allow
    );
    let result = Policy::parse(p)
        .unwrap()
        .evaluate(&request("10.3.0.1", "GET", 443))
        .unwrap();
    assert_eq!(result.matched_line, Some(5));
    assert_eq!(result.rules.len(), 2);
    assert_eq!(
        result.rules[0]
            .terms
            .iter()
            .map(|t| t.matched)
            .collect::<Vec<_>>(),
        [false, true]
    );
}
#[test]
fn negation_and_connect_port_guards_require_correct_order() {
    let p = "acl secure port 443\nacl web port 80 443\nacl tunnel method CONNECT\nhttp_access deny !web\nhttp_access deny tunnel !secure\nhttp_access allow all";
    assert_eq!(action(p, "192.0.2.1", "CONNECT", 443), Action::Allow);
    for (m, port) in [("CONNECT", 80), ("GET", 25)] {
        assert_eq!(action(p, "192.0.2.1", m, port), Action::Deny);
    }
    assert_eq!(action(p, "192.0.2.1", "GET", 80), Action::Allow);
}
#[test]
fn no_rule_and_opposite_last_fallback_are_not_approximated_as_default_deny() {
    assert_eq!(action("", "192.0.2.1", "GET", 80), Action::Deny);
    let p = "acl staff src 10.0.0.0/8\nhttp_access deny staff";
    let d = Policy::parse(p)
        .unwrap()
        .evaluate(&request("192.0.2.1", "GET", 80))
        .unwrap();
    assert_eq!(d.action, Action::Allow);
    assert_eq!(d.matched_line, None);
    assert!(d.explanation.contains("opposite"));
    assert_eq!(
        action(
            "acl staff src 10.0.0.0/8\nhttp_access allow staff",
            "192.0.2.1",
            "GET",
            80
        ),
        Action::Deny
    );
}
#[test]
fn range_ends_are_inclusive_and_method_tokens_case_sensitive() {
    let p =
        "acl ports port 8000-8010\nacl methods method GET HEAD\nhttp_access allow ports methods";
    for port in [8000, 8005, 8010] {
        assert_eq!(action(p, "192.0.2.1", "HEAD", port), Action::Allow);
    }
    for port in [7999, 8011] {
        assert_eq!(action(p, "192.0.2.1", "HEAD", port), Action::Deny);
    }
    assert_eq!(action(p, "192.0.2.1", "get", 8000), Action::Deny);
}
#[test]
fn malformed_unknown_and_unsupported_rules_cannot_silently_apply() {
    for p in [
        "acl staff src 10.0.0.0/33",
        "acl staff src 10.0.0.0/24\nacl staff port 80",
        "http_access allow missing",
        "http_access allow !!all",
        "http_access allow",
        "acl all src 0.0.0.0/0",
        "acl ports port 9000-8000",
        "acl p src 256.1.2.3",
        "acl p src 1.2.3.4/2/3",
    ] {
        assert_eq!(Policy::parse(p).unwrap_err().kind, FaultKind::Syntax, "{p}");
    }
    for p in [
        "include /etc/passwd",
        "acl auth proxy_auth REQUIRED",
        "acl names dstdomain .example.test",
        "acl p src \"/etc/passwd\"",
        "http_access allow all # later",
    ] {
        assert_eq!(
            Policy::parse(p).unwrap_err().kind,
            FaultKind::Unsupported,
            "{p}"
        );
    }
}
#[test]
fn bounded_config_and_requests_reject_before_rule_work() {
    assert_eq!(
        Policy::parse(&"a".repeat(MAX_CONFIG + 1)).unwrap_err().kind,
        FaultKind::Limit
    );
    assert_eq!(
        Policy::parse(&"\n".repeat(MAX_LINES + 1)).unwrap_err().kind,
        FaultKind::Limit
    );
    let p = Policy::parse("http_access allow all").unwrap();
    for r in [
        request("192.0.2.1", "GET", 0),
        request("192.0.2.1", "GET /etc/passwd", 80),
        request("192.0.2.1", &"A".repeat(21), 80),
    ] {
        assert_eq!(p.evaluate(&r).unwrap_err().kind, FaultKind::Invalid);
    }
}

#[test]
fn many_acls_values_rules_and_terms_are_bounded_independently() {
    let many_acls = (0..33)
        .map(|i| format!("acl a{i} src 192.0.2.1\n"))
        .collect::<String>();
    assert_eq!(
        Policy::parse(&many_acls).unwrap_err().kind,
        FaultKind::Limit
    );
    let values = format!("acl a src {}", ["192.0.2.1"; 257].join(" "));
    // The line-size bound may reject even earlier; both must be Limit.
    assert_eq!(Policy::parse(&values).unwrap_err().kind, FaultKind::Limit);
    assert_eq!(
        Policy::parse(&"http_access allow all\n".repeat(65))
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    let terms = format!("http_access allow {}", ["all"; 17].join(" "));
    assert_eq!(Policy::parse(&terms).unwrap_err().kind, FaultKind::Limit);
}
#[test]
fn comments_crlf_and_repeated_values_are_supported_without_deduplicating_semantics() {
    let p=Policy::parse("# separate note\r\nacl staff src 192.0.2.7 192.0.2.7\r\nhttp_access allow staff\r\nhttp_access deny all\r\n").unwrap();
    assert_eq!(
        p.evaluate(&request("192.0.2.7", "GET", 80)).unwrap().action,
        Action::Allow
    );
    assert_eq!(
        p.evaluate(&request("192.0.2.8", "GET", 80)).unwrap().action,
        Action::Deny
    );
    let bad =
        serde_json::json!({"client":"192.0.2.1","method":"A".repeat(21),"destination_port":80});
    assert!(serde_json::from_value::<Request>(bad).is_err());
}
#[test]
fn saved_and_applied_are_distinct_and_activation_failures_are_atomic() {
    let mut w = World::new("http_access deny all").unwrap();
    let r = request("192.0.2.7", "GET", 80);
    assert!(w.save("http_access allow all").unwrap());
    assert_eq!(w.effective_decision(&r).unwrap().action, Action::Deny);
    assert_eq!(w.saved_decision(&r).unwrap().action, Action::Allow);
    assert!(w.apply_saved().unwrap());
    assert_eq!(w.effective_decision(&r).unwrap().action, Action::Allow);
    assert!(w.save("include /etc/passwd").unwrap());
    let before = w.clone();
    assert_eq!(w.apply_saved().unwrap_err().kind, FaultKind::Unsupported);
    assert_eq!(w, before);
    assert_eq!(w.effective_decision(&r).unwrap().action, Action::Allow);
    assert!(w.saved_decision(&r).is_err());
    let bytes = serde_json::to_string(&w).unwrap();
    let restored: World = serde_json::from_str(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, w);
}
#[test]
fn failed_saves_and_exhausted_history_cannot_partially_mutate() {
    let mut w = World::new("http_access deny all").unwrap();
    let before = w.clone();
    assert!(w.save(&"x".repeat(MAX_CONFIG + 1)).is_err());
    assert_eq!(w, before);
    assert!(w.save("bad\0value").is_err());
    assert_eq!(w, before);
    let same = w.saved().to_owned();
    assert!(!w.save(&same).unwrap());
    assert_eq!(w, before);
    assert!(!w.apply_saved().unwrap());
    assert_eq!(w, before);
    let value = serde_json::json!({"saved":"http_access deny all","applied":"http_access deny all","generation":10_000});
    let mut w: World = serde_json::from_value(value).unwrap();
    let before = w.clone();
    assert!(w.save("http_access allow all").is_err());
    assert_eq!(w, before);
}
#[test]
fn malformed_effective_serialized_state_is_rejected_not_silently_repaired() {
    let value = serde_json::json!({"saved":"http_access deny all","applied":"include /etc/passwd","generation":0});
    let w: World = serde_json::from_value(value).unwrap();
    assert!(w.validate().is_err());
    assert!(
        w.effective_decision(&request("192.0.2.1", "GET", 80))
            .is_err()
    );
    let value = serde_json::json!({"saved":"x".repeat(MAX_CONFIG+1),"applied":"http_access deny all","generation":0});
    assert!(serde_json::from_value::<World>(value).is_err());
}
#[test]
fn exact_partition_comparison_accepts_equivalent_rules_and_rejects_hardcoded_fixtures() {
    let reference=Policy::parse("acl staff src 192.0.2.0/24\nacl web port 80 443\nhttp_access allow staff web\nhttp_access deny all").unwrap();
    let equivalent=Policy::parse("acl staff src 192.0.2.0/25 192.0.2.128/25\nacl web port 80 443\nhttp_access deny !staff\nhttp_access deny !web\nhttp_access allow all").unwrap();
    assert_eq!(equivalent.compare(&reference).unwrap(), None);
    let hardcoded=Policy::parse("acl staff src 192.0.2.3 192.0.2.8 192.0.2.254\nacl web port 80 443\nhttp_access allow staff web\nhttp_access deny all").unwrap();
    let missing = hardcoded.compare(&reference).unwrap().unwrap();
    assert_eq!(
        (missing.actual, missing.required),
        (Action::Deny, Action::Allow)
    );
    assert_eq!(missing.request.client, Ipv4Addr::new(192, 0, 2, 1));
    for wrong in [
        "http_access allow all",
        "http_access deny all",
        "acl staff src 192.0.2.0/23\nhttp_access allow staff",
    ] {
        assert!(
            Policy::parse(wrong)
                .unwrap()
                .compare(&reference)
                .unwrap()
                .is_some()
        );
    }
}
#[test]
fn partition_includes_upper_ipv4_and_port_edges_and_unknown_methods() {
    let all = Policy::parse("http_access allow all").unwrap();
    for wrong in [
        "acl top src 255.255.255.255/32\nhttp_access deny top\nhttp_access allow all",
        "acl top port 65535\nhttp_access deny top\nhttp_access allow all",
        "acl known method GET POST HEAD\nhttp_access allow known\nhttp_access deny all",
    ] {
        let p = Policy::parse(wrong).unwrap();
        let d = p.compare(&all).unwrap().unwrap();
        assert_eq!(d.actual, Action::Deny);
        assert_eq!(d.required, Action::Allow);
        assert_eq!(p.evaluate(&d.request).unwrap().action, d.actual);
    }
    let empty = Policy::parse("").unwrap();
    assert_eq!(
        empty
            .compare(&Policy::parse("http_access deny all").unwrap())
            .unwrap(),
        None
    );
    let port_zero =
        Policy::parse("acl never port 0\nhttp_access allow never\nhttp_access deny all").unwrap();
    assert_eq!(port_zero.compare(&empty).unwrap(), None);
}
#[test]
fn excessive_symbolic_regions_are_refused_instead_of_sampled_or_assumed_safe() {
    let text = (1..=80)
        .map(|i| {
            format!(
                "acl addresses src 10.0.0.{}\nacl ports port {}\n",
                i * 2,
                1000 + i * 2
            )
        })
        .collect::<String>()
        + "http_access allow addresses ports\nhttp_access deny all";
    let p = Policy::parse(&text).unwrap();
    assert_eq!(
        p.compare(&Policy::parse("http_access deny all").unwrap())
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
}

#[test]
fn predicate_work_bound_accounts_for_repeated_rule_terms_and_acl_values() {
    let addresses = (1..=32)
        .map(|i| format!("10.0.0.{}", i * 2))
        .collect::<Vec<_>>()
        .join(" ");
    let rule = format!("http_access allow {}\n", ["a"; 16].join(" "));
    let p = Policy::parse(&format!("acl a src {addresses}\n{}", rule.repeat(64))).unwrap();
    let e = p
        .compare(&Policy::parse("http_access deny all").unwrap())
        .unwrap_err();
    assert_eq!(e.kind, FaultKind::Limit);
    assert!(e.message.contains("work budget"));
}
#[test]
fn diagnostic_prefers_a_wide_interval_witness_but_keeps_singleton_failures() {
    let required=Policy::parse("acl staff src 10.44.16.0/20\nacl web port 80\nacl reads method GET\nhttp_access allow staff web reads\nhttp_access deny all").unwrap();
    let narrow=Policy::parse("acl staff src 10.44.16.1 10.44.31.254\nacl web port 80\nacl reads method GET\nhttp_access allow staff web reads\nhttp_access deny all").unwrap();
    let witness = narrow.compare(&required).unwrap().unwrap();
    assert_eq!(witness.request.client, Ipv4Addr::new(10, 44, 16, 3));
    assert_eq!(
        (witness.actual, witness.required),
        (Action::Deny, Action::Allow)
    );
    let all = Policy::parse("http_access allow all").unwrap();
    let one =
        Policy::parse("acl one src 255.255.255.255\nhttp_access deny one\nhttp_access allow all")
            .unwrap();
    let witness = one.compare(&all).unwrap().unwrap();
    assert_eq!(witness.request.client, Ipv4Addr::new(255, 255, 255, 255));
}
