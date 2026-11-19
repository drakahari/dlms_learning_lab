use super::*;
fn cfg(body: &str) -> Config {
    parse(&format!(
        "server {{ listen 8080; location / {{ return 404; }} {body} }}"
    ))
    .unwrap()
}
fn endpoint() -> Endpoint {
    Endpoint {
        address: Ipv4Addr::new(192, 0, 2, 20),
        port: 9000,
    }
}
fn forwarded(config: &Config, uri: &str) -> (Endpoint, String) {
    match config.route(8080, uri).unwrap() {
        Route::Proxy { endpoint, path, .. } => (endpoint, path),
        other => panic!("Expected proxy: {other:?}"),
    }
}
#[test]
fn literal_uri_replaces_prefix_and_missing_uri_preserves_it() {
    let strip = cfg("location /reports/ { proxy_pass http://192.0.2.20:9000/; }");
    assert_eq!(
        forwarded(&strip, "/reports/team/a.csv"),
        (endpoint(), "/team/a.csv".into())
    );
    let keep = cfg("location /reports/ { proxy_pass http://192.0.2.20:9000; }");
    assert_eq!(
        forwarded(&keep, "/reports/team/a.csv"),
        (endpoint(), "/reports/team/a.csv".into())
    );
    let replace = cfg("location /reports/ { proxy_pass http://192.0.2.20:9000/internal/; }");
    assert_eq!(
        forwarded(&replace, "/reports/a.csv"),
        (endpoint(), "/internal/a.csv".into())
    );
}
#[test]
fn preserve_literal_concatenation_instead_of_silently_normalizing() {
    let c = cfg("location /reports { proxy_pass http://192.0.2.20:9000/; }");
    assert_eq!(forwarded(&c, "/reports/a"), (endpoint(), "//a".into()));
    assert_eq!(
        forwarded(&c, "/reports-other"),
        (endpoint(), "/-other".into())
    );
}
#[test]
fn exact_wins_then_longest_prefix_independent_of_order() {
    let a = cfg(
        "location /a { return 403; } location /a/b { return 503; } location = /a/b { return 200 \"ready now\"; }",
    );
    assert!(
        matches!(a.route(8080,"/a/b").unwrap(),Route::Return{status:200,body:Some(ref b),..}if b=="ready now")
    );
    assert!(matches!(
        a.route(8080, "/a/b/c").unwrap(),
        Route::Return { status: 503, .. }
    ));
    assert!(matches!(
        a.route(8080, "/aardvark").unwrap(),
        Route::Return { status: 403, .. }
    ));
    assert!(matches!(
        a.route(8080, "/z").unwrap(),
        Route::Return { status: 404, .. }
    ));
    let b = cfg(
        "location = /a/b { return 200 'ready now'; } location /a/b { return 503; } location /a { return 403; }",
    );
    assert_eq!(difference(&a, &b, 8080).unwrap(), None);
}
#[test]
fn slash_redirect_and_explicit_exact_override() {
    let c = cfg("location /app/ { proxy_pass http://192.0.2.20:9000/; }");
    assert_eq!(
        c.route(8080, "/app").unwrap(),
        Route::Redirect {
            location: "/app/".into(),
            path: "/app/".into()
        }
    );
    assert!(matches!(
        c.route(8080, "/apps").unwrap(),
        Route::Return { status: 404, .. }
    ));
    let c = cfg(
        "location /app/ { proxy_pass http://192.0.2.20:9000/; } location = /app { return 403; }",
    );
    assert!(matches!(
        c.route(8080, "/app").unwrap(),
        Route::Return { status: 403, .. }
    ));
    assert_eq!(parse("server { location / { return 404; } location /app { return 403; } location /app/ { proxy_pass http://192.0.2.20/; } }").unwrap_err().kind,FaultKind::Unsupported);
}
#[test]
fn explicit_listener_and_optional_name_are_not_fake_host_selection() {
    let c =
        parse("server { listen 80; server_name lesson.test; location / { return 200 'hello'; } }")
            .unwrap();
    assert_eq!(
        parse("server { location / { return 404; } }")
            .unwrap_err()
            .kind,
        FaultKind::Unsupported
    );
    assert_eq!(c.listen, 80);
    assert_eq!(c.server_name.as_deref(), Some("lesson.test"));
    assert!(matches!(c.route(8080, "/").unwrap(), Route::NotListening));
    assert!(matches!(
        c.route(80, "/").unwrap(),
        Route::Return { status: 200, .. }
    ));
    assert_eq!(c.route(0, "/").unwrap_err().kind, FaultKind::Invalid);
}
#[test]
fn comments_and_simple_quotes_are_lexed_not_executed() {
    let c=parse("# fixed fixture\nserver{listen 80;location /{return 200 \"hello # world; {}\";} # comment\n}").unwrap();
    assert_eq!(
        c.route(80, "/").unwrap(),
        Route::Return {
            location: "/".into(),
            status: 200,
            body: Some("hello # world; {}".into())
        }
    );
    for s in [
        "server { location / { return 200 '$host'; } }",
        "server { location / { return 200 \"a\\nb\"; } }",
        "server { location / { return 200 'x'y; } }",
    ] {
        assert_eq!(parse(s).unwrap_err().kind, FaultKind::Unsupported);
    }
}
#[test]
fn syntax_and_unsupported_constructs_have_distinct_honest_failures() {
    for s in [
        "",
        "server",
        "server {",
        "server { location / { return 404 } }",
        "server { location / { return 200 'unfinished; } }",
        "server { location / { return 404; } location / { return 403; } }",
    ] {
        assert_eq!(parse(s).unwrap_err().kind, FaultKind::Syntax, "{s}");
    }
    for s in [
        "http { server {} }",
        "server { include x; }",
        "server { root /srv/www; }",
        "server { listen 80 ssl; }",
        "server { location / { return 301 /new; } }",
        "server { location / { proxy_pass https://192.0.2.20/; } }",
        "server { location / { proxy_pass http://backend/; } }",
        "server { location / { proxy_pass http://[::1]/; } }",
    ] {
        assert_eq!(parse(s).unwrap_err().kind, FaultKind::Unsupported, "{s}");
    }
    assert_eq!(
        parse("server { location = / { return 404; } }")
            .unwrap_err()
            .kind,
        FaultKind::Unsupported
    );
}
#[test]
fn request_paths_and_config_paths_do_not_approximate_normalization() {
    let c = cfg("");
    for uri in [
        "/a/../b",
        "/./x",
        "/a//b",
        "/a?x=1",
        "/%2e%2e/private",
        "/a\\b",
        "/café",
    ] {
        assert_eq!(
            c.route(8080, uri).unwrap_err().kind,
            FaultKind::Unsupported,
            "{uri}"
        );
    }
    for uri in ["", "relative", &format!("/{}", "a".repeat(128))] {
        assert_eq!(c.route(8080, uri).unwrap_err().kind, FaultKind::Invalid);
    }
    assert!(c.route(8080, "/.well-known/x_2~.txt").is_ok());
}
#[test]
fn limits_reject_before_unbounded_evaluation() {
    assert_eq!(
        parse(&" ".repeat(MAX_CONFIG + 1)).unwrap_err().kind,
        FaultKind::Limit
    );
    let text = format!(
        "server {{ location / {{ return 200 '{}'; }} }}",
        "x".repeat(MAX_TEXT + 1)
    );
    assert_eq!(parse(&text).unwrap_err().kind, FaultKind::Limit);
    let text = format!(
        "server {{ location / {{ return 404; }} {} }}",
        (0..32)
            .map(|i| format!("location /{i} {{ return 404; }}"))
            .collect::<String>()
    );
    assert_eq!(parse(&text).unwrap_err().kind, FaultKind::Limit);
    assert_eq!(
        tokens(&";".repeat(MAX_TOKENS + 1)).unwrap_err().kind,
        FaultKind::Limit
    );
    for port in ["0", "65536", "-1", "80x"] {
        assert!(
            parse(&format!(
                "server {{ listen {port}; location / {{ return 404; }} }}"
            ))
            .is_err()
        );
    }
}
#[test]
fn whole_path_partition_catches_hardcoded_visible_examples() {
    let expected = cfg("location /reports/ { proxy_pass http://192.0.2.20:9000/; }");
    let fake = cfg(
        "location = /reports { return 404; } location = /reports/a.csv { proxy_pass http://192.0.2.20:9000/a.csv; } location = /reports/b.csv { proxy_pass http://192.0.2.20:9000/b.csv; }",
    );
    assert_eq!(
        forwarded(&fake, "/reports/a.csv"),
        forwarded(&expected, "/reports/a.csv")
    );
    let d = difference(&fake, &expected, 8080).unwrap().unwrap();
    assert!(!same_route(&d.actual, &d.expected));
    assert!(difference(&expected, &expected, 8080).unwrap().is_none());
}
#[test]
fn whole_path_partition_accepts_equivalent_refinements_but_detects_hidden_leaks() {
    let expected = cfg("location /reports/ { proxy_pass http://192.0.2.20:9000/; }");
    let alternative = cfg(
        "location ^~ /reports/ { proxy_pass http://192.0.2.20:9000/; } location /reports/team/ { proxy_pass http://192.0.2.20:9000/team/; } location = /reports/team { proxy_pass http://192.0.2.20:9000/team; } location = /reports/a { proxy_pass http://192.0.2.20:9000/a; }",
    );
    let added_redirect = cfg(
        "location /reports/ { proxy_pass http://192.0.2.20:9000/; } location /reports/team/ { proxy_pass http://192.0.2.20:9000/team/; }",
    );
    let redirect_difference = super::difference(&added_redirect, &expected, 8080)
        .unwrap()
        .unwrap();
    assert_eq!(redirect_difference.path, "/reports/team");
    assert!(matches!(redirect_difference.actual, Route::Redirect { .. }));
    assert_eq!(difference(&alternative, &expected, 8080).unwrap(), None);
    let leak = cfg(
        "location /reports/ { proxy_pass http://192.0.2.20:9000/; } location /secret-branch/ { proxy_pass http://192.0.2.20:9000/; }",
    );
    assert!(difference(&leak, &expected, 8080).unwrap().is_some());
    assert!(difference(&expected, &leak, 8080).unwrap().is_some());
    assert!(difference(&expected, &alternative, 80).unwrap().is_none());
}
#[test]
fn partition_matches_exhaustive_small_path_language_and_maximum_boundary() {
    let configs = [
        cfg(""),
        cfg("location /a { return 403; }"),
        cfg("location = /a { return 503; }"),
        cfg("location /a/ { proxy_pass http://192.0.2.20:9000/; }"),
        cfg("location /a/ { proxy_pass http://192.0.2.20:9000; }"),
    ];
    let mut paths = vec!["/".to_owned()];
    let mut frontier = paths.clone();
    for _ in 0..5 {
        let mut next = Vec::new();
        for base in frontier {
            for ch in ['a', 'b', '/', '.'] {
                let p = format!("{base}{ch}");
                if path(&p).is_ok() {
                    paths.push(p.clone());
                }
                next.push(p);
            }
        }
        frontier = next;
    }
    for a in &configs {
        for b in &configs {
            let found = difference(a, b, 8080).unwrap();
            let exhaustive = paths
                .iter()
                .any(|p| !same_route(&a.route(8080, p).unwrap(), &b.route(8080, p).unwrap()));
            assert_eq!(found.is_some(), exhaustive);
            if let Some(d) = found {
                assert!(!same_route(&d.actual, &d.expected));
            }
        }
    }
    let long = format!("/{}", "a".repeat(MAX_PATH - 1));
    let different = cfg(&format!("location = {long} {{ return 403; }}"));
    assert!(difference(&different, &cfg(""), 8080).unwrap().is_some());
}
#[test]
fn save_validate_and_apply_are_separate_atomic_virtual_operations() {
    let old = "server { listen 8080; location / { return 404; } }";
    let new = "server { listen 8080; location / { return 200 'ready'; } }";
    let mut w = World::new(old).unwrap();
    assert!(w.save(new).unwrap());
    assert_eq!(w.saved(), new);
    assert_eq!(w.applied(), old);
    w.validate_saved().unwrap();
    assert_eq!(w.applied(), old);
    assert!(w.apply_saved().unwrap());
    assert_eq!(w.applied(), new);
    assert!(!w.apply_saved().unwrap());
    assert!(!w.save(new).unwrap());
    let broken = "server { location / { return 404 } }";
    assert!(w.save(broken).unwrap());
    let before = w.clone();
    assert_eq!(w.validate_saved().unwrap_err().kind, FaultKind::Syntax);
    assert_eq!(w.apply_saved().unwrap_err().kind, FaultKind::Syntax);
    assert_eq!(w, before);
    assert!(matches!(
        w.effective_route(8080, "/").unwrap(),
        Route::Return { status: 200, .. }
    ));
}
#[test]
fn imported_world_cannot_supply_parsed_or_cached_authority() {
    let mut w = World::new("server { listen 80; location / { return 404; } }").unwrap();
    w.save("server { broken").unwrap();
    let bytes = serde_json::to_vec(&w).unwrap();
    let restored: World = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, w);
    assert!(restored.validate_saved().is_err());
    let mut value = serde_json::to_value(&w).unwrap();
    value["cached_success"] = serde_json::json!(true);
    assert!(serde_json::from_value::<World>(value).is_err());
    let mut value = serde_json::to_value(&w).unwrap();
    value["applied"] = serde_json::json!("server { broken");
    let forged: World = serde_json::from_value(value).unwrap();
    assert!(forged.validate().is_err());
    let mut value = serde_json::to_value(&w).unwrap();
    value["saved"] = serde_json::json!("x".repeat(32769));
    assert!(serde_json::from_value::<World>(value).is_err());
}
#[test]
fn unspecified_error_page_is_not_a_hidden_exact_text_requirement() {
    let required = cfg("");
    let alternative =
        parse("server { listen 8080; location / { return 404 'Page unavailable'; } }").unwrap();
    assert_eq!(difference(&alternative, &required, 8080).unwrap(), None);
    assert!(difference(&required, &alternative, 8080).unwrap().is_some());
    let wrong_status =
        parse("server { listen 8080; location / { return 403 'Page unavailable'; } }").unwrap();
    assert!(
        difference(&wrong_status, &required, 8080)
            .unwrap()
            .is_some()
    );
}
