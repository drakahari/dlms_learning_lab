use super::*;
fn reply(at: u64, tag: &str, body: &str) -> OriginReply {
    OriginReply {
        response_time: at,
        date: at,
        age: 0,
        etag: tag.into(),
        body: body.into(),
    }
}
fn step(at: u64, tag: &str, body: &str) -> Step {
    Step {
        path: "/guide".into(),
        request_time: at,
        request_control: String::new(),
        origin: Some(reply(at, tag, body)),
    }
}
fn run_one(control: &str, steps: &[Step]) -> Vec<Observation> {
    run(steps, &BTreeMap::from([("/guide".into(), control.into())])).unwrap()
}
#[test]
fn real_directive_lexing_accepts_case_whitespace_and_quoted_seconds() {
    let c = Control::parse(
        " PUBLIC, max-age=\"60\",\ts-maxage=120, must-revalidate",
        Side::Response,
    )
    .unwrap();
    assert!(c.public && c.must_revalidate);
    assert_eq!((c.max_age, c.freshness_lifetime()), (Some(60), 120));
    assert!(c.shared_storage_allowed());
    assert_eq!(
        Control::parse("  ", Side::Response).unwrap(),
        Control::default()
    );
}
#[test]
fn syntax_unsupported_and_limits_are_distinct_without_silent_guessing() {
    for s in [
        "max-age",
        "max-age=-1",
        "max-age=sixty",
        "max-age=\"2",
        "max-age =2",
        "s-maxage=1.5",
        "no-cache\n",
    ] {
        assert_eq!(
            Control::parse(s, Side::Response).unwrap_err().kind,
            FaultKind::Syntax,
            "{s}"
        );
    }
    for s in [
        "no-cache=\"set-cookie\"",
        "max-age=1, MAX-AGE=2",
        "immutable",
        "private=\"secret\"",
        "max-age=1,, public",
    ] {
        assert_eq!(
            Control::parse(s, Side::Response).unwrap_err().kind,
            FaultKind::Unsupported,
            "{s}"
        );
    }
    assert_eq!(
        Control::parse(&"a".repeat(1025), Side::Response)
            .unwrap_err()
            .kind,
        FaultKind::Limit
    );
    assert_eq!(
        Control::parse("s-maxage=2", Side::Request)
            .unwrap_err()
            .kind,
        FaultKind::Unsupported
    );
}
#[test]
fn seconds_saturate_rather_than_overflow_or_wrap() {
    let c = Control::parse(&format!("max-age={}", "9".repeat(100)), Side::Response).unwrap();
    assert_eq!(c.max_age, Some(AGE_LIMIT));
    assert_eq!(seconds("2147483648").unwrap(), AGE_LIMIT);
    assert_eq!(seconds("00060").unwrap(), 60);
}
#[test]
fn freshness_and_exact_expiry_are_not_the_same_as_origin_revision() {
    let r = run_one(
        "public, max-age=60",
        &[
            step(0, "a", "old"),
            step(59, "b", "new"),
            step(60, "b", "new"),
        ],
    );
    assert_eq!(
        r.iter().map(|v| v.delivery).collect::<Vec<_>>(),
        vec![Delivery::Downloaded, Delivery::Cached, Delivery::Downloaded]
    );
    assert_eq!(r[1].body.as_deref(), Some("old"));
    assert_eq!(r[1].age, Some(59));
    assert_eq!(r[2].body.as_deref(), Some("new"));
    assert_eq!(r[2].origin_status, Some(200));
}
#[test]
fn shared_age_overrides_private_browser_max_age() {
    let r = run_one(
        "max-age=600, s-maxage=30",
        &[step(0, "a", "v1"), step(30, "b", "v2")],
    );
    assert_eq!(r[1].delivery, Delivery::Downloaded);
    assert_eq!(r[1].body.as_deref(), Some("v2"));
}
#[test]
fn supplied_age_apparent_age_transit_and_residence_all_matter() {
    let mut first = step(100, "a", "v1");
    let o = first.origin.as_mut().unwrap();
    o.response_time = 105;
    o.date = 90;
    o.age = 20;
    let r = run_one(
        "max-age=60",
        &[first, step(115, "b", "v2"), step(140, "b", "v2")],
    );
    assert_eq!(r[0].age, Some(25));
    assert_eq!(r[1].age, Some(35));
    assert_eq!(r[1].delivery, Delivery::Cached);
    assert_eq!(r[2].delivery, Delivery::Downloaded);
    let mut future = step(0, "a", "v1");
    future.origin.as_mut().unwrap().date = 5;
    assert_eq!(run_one("max-age=60", &[future])[0].age, Some(0));
}
#[test]
fn no_cache_stores_but_requires_validation_on_each_reuse() {
    let r = run_one(
        "no-cache, max-age=60",
        &[
            step(0, "a", "same"),
            step(1, "a", "same"),
            step(2, "b", "changed"),
        ],
    );
    assert!(r.iter().all(|v| v.stored));
    assert_eq!(r[1].delivery, Delivery::Validated);
    assert_eq!(r[1].origin_status, Some(304));
    assert_eq!(r[2].delivery, Delivery::Downloaded);
    assert_eq!(r[2].body.as_deref(), Some("changed"));
}
#[test]
fn no_store_and_private_never_store_a_shared_response_even_with_public() {
    for control in [
        "no-store, max-age=60",
        "private, max-age=60",
        "public, private, max-age=60",
    ] {
        let r = run_one(control, &[step(0, "a", "same"), step(1, "a", "same")]);
        assert!(
            r.iter()
                .all(|v| !v.stored && v.delivery == Delivery::Downloaded)
        );
    }
}
#[test]
fn request_no_cache_revalidates_fresh_content_but_request_no_store_can_hit() {
    let mut force = step(1, "a", "same");
    force.request_control = "no-cache".into();
    let mut hit = step(2, "b", "changed");
    hit.request_control = "no-store".into();
    let r = run_one("max-age=60", &[step(0, "a", "same"), force, hit]);
    assert_eq!(r[1].delivery, Delivery::Validated);
    assert_eq!(r[2].delivery, Delivery::Cached);
    assert!(r[2].stored);
    assert_eq!(r[2].body.as_deref(), Some("same"));
    let mut miss = step(0, "a", "same");
    miss.request_control = "no-store".into();
    let r = run_one("max-age=60", &[miss, step(1, "a", "same")]);
    assert!(!r[0].stored);
    assert_eq!(r[1].delivery, Delivery::Downloaded);
}
#[test]
fn request_max_age_can_require_validation_before_normal_expiry() {
    let mut restrictive = step(20, "b", "changed");
    restrictive.request_control = "max-age=10".into();
    let r = run_one("max-age=60", &[step(0, "a", "same"), restrictive]);
    assert_eq!(r[1].delivery, Delivery::Downloaded);
}
#[test]
fn validation_updates_freshness_without_redownloading_the_body() {
    let r = run_one(
        "max-age=10",
        &[
            step(0, "a", "same"),
            step(10, "a", "same"),
            step(19, "b", "changed"),
        ],
    );
    assert_eq!(r[1].delivery, Delivery::Validated);
    assert_eq!(r[1].age, Some(0));
    assert_eq!(r[2].delivery, Delivery::Cached);
    assert_eq!(r[2].body.as_deref(), Some("same"));
    assert_eq!(r[2].age, Some(9));
}
#[test]
fn missing_freshness_uses_no_heuristic_and_offline_never_serves_stale() {
    let mut offline = step(60, "a", "same");
    offline.origin = None;
    let r = run_one(
        "public, max-age=60",
        &[step(0, "a", "same"), offline.clone()],
    );
    assert_eq!(r[1].delivery, Delivery::Unavailable);
    assert!(r[1].body.is_none());
    offline.request_time = 59;
    assert_eq!(
        run_one("max-age=60", &[step(0, "a", "same"), offline.clone()])[1].delivery,
        Delivery::Cached
    );
    assert_eq!(
        run_one("public", &[step(0, "a", "same"), offline])[1].delivery,
        Delivery::Unavailable
    );
}
#[test]
fn malformed_source_facts_are_rejected_before_evaluation_and_runs_are_isolated() {
    let headers = BTreeMap::from([("/guide".into(), "max-age=60".into())]);
    let first = step(0, "a", "same");
    let mut changed = step(1, "a", "different");
    assert_eq!(
        run(&[first.clone(), changed.clone()], &headers)
            .unwrap_err()
            .kind,
        FaultKind::Invalid
    );
    changed.path = "/../personal".into();
    assert!(run(&[changed], &headers).is_err());
    let mut late = first.clone();
    late.origin.as_mut().unwrap().response_time = 3601;
    assert!(run(&[late], &headers).is_err());
    for _ in 0..2 {
        assert_eq!(
            run(std::slice::from_ref(&first), &headers).unwrap()[0].delivery,
            Delivery::Downloaded
        );
    }
    assert_eq!(
        run(&vec![first; 65], &headers).unwrap_err().kind,
        FaultKind::Limit
    );
}

#[test]
fn distinct_paths_do_not_share_a_body_or_a_storage_rule() {
    let mut private = step(1, "a", "personal");
    private.path = "/account".into();
    let mut later = private.clone();
    later.request_time = 3;
    later.origin = Some(reply(3, "b", "different person"));
    let rows = run(
        &[
            step(0, "a", "public"),
            private,
            step(2, "c", "changed public"),
            later,
        ],
        &BTreeMap::from([
            ("/guide".into(), "max-age=60".into()),
            ("/account".into(), "private, max-age=60".into()),
        ]),
    )
    .unwrap();
    assert_eq!(rows[2].delivery, Delivery::Cached);
    assert_eq!(rows[2].body.as_deref(), Some("public"));
    assert_eq!(rows[3].body.as_deref(), Some("different person"));
    assert!(!rows[1].stored && !rows[3].stored);
}

#[test]
fn no_store_on_origin_contact_drops_old_metadata_but_never_discards_the_received_body() {
    let mut force = step(1, "a", "same");
    force.request_control = "no-cache, no-store".into();
    let rows = run_one(
        "max-age=60",
        &[step(0, "a", "same"), force, step(2, "a", "same")],
    );
    assert_eq!(rows[1].delivery, Delivery::Validated);
    assert_eq!(rows[1].body.as_deref(), Some("same"));
    assert!(!rows[1].stored);
    assert_eq!(rows[2].delivery, Delivery::Downloaded);
}

#[test]
fn response_age_overflow_and_residence_never_wrap_to_a_fresh_response() {
    let mut first = step(0, "a", "old");
    first.origin.as_mut().unwrap().age = u64::MAX;
    let rows = run_one("max-age=2147483648", &[first, step(1, "b", "new")]);
    assert_eq!(rows[0].age, Some(AGE_LIMIT));
    assert_eq!(rows[1].delivery, Delivery::Downloaded);
    let rows = run_one(
        "max-age=60",
        &[step(0, "a", "old"), step(u64::MAX, "b", "new")],
    );
    assert_eq!(rows[1].delivery, Delivery::Downloaded);
}

#[test]
fn typed_imports_refuse_unknown_execution_fields_and_overlong_values() {
    let value = serde_json::to_value(step(0, "a", "body")).unwrap();
    let mut bad = value.clone();
    bad["host_command"] = serde_json::json!("anything");
    assert!(serde_json::from_value::<Step>(bad).is_err());
    let mut bad = value.clone();
    bad["origin"]["body"] = serde_json::json!("x".repeat(MAX_BODY + 1));
    assert!(serde_json::from_value::<Step>(bad).is_err());
    let mut bad = value;
    bad["path"] = serde_json::json!("x".repeat(129));
    assert!(serde_json::from_value::<Step>(bad).is_err());
    assert!(
        serde_json::from_str::<Step>(
            "{\"path\":\"/guide\",\"request_time\":-1,\"request_control\":\"\",\"origin\":null}"
        )
        .is_err()
    );
}

#[test]
fn programmatically_constructed_oversize_values_and_backward_clocks_also_fail() {
    let headers = BTreeMap::from([("/guide".into(), "max-age=60".into())]);
    let mut body = step(0, "a", "small");
    body.origin.as_mut().unwrap().body = "x".repeat(MAX_BODY + 1);
    assert!(run(&[body], &headers).is_err());
    let mut bad_tag = step(0, "a", "small");
    bad_tag.origin.as_mut().unwrap().etag = "quoted\"tag".into();
    assert!(run(&[bad_tag], &headers).is_err());
    assert!(run(&[step(2, "a", "small"), step(1, "a", "small")], &headers).is_err());
    let many = (0..17)
        .map(|i| (format!("/doc{i}"), "max-age=60".into()))
        .collect();
    assert_eq!(
        run(&[step(0, "a", "small")], &many).unwrap_err().kind,
        FaultKind::Limit
    );
}
