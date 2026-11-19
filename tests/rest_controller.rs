use dlms_learning_lab::rest_controller as api;
#[path = "support/rest_controller_fixture.rs"]
mod fixture;
use fixture::{fixture, request};

fn execute(
    spec: &api::Spec,
    store: &mut api::Store,
    method: &str,
    path: &str,
    body: &str,
) -> api::Response {
    api::send(spec, store, &request(method, path, body)).unwrap()
}

#[test]
fn get_observes_resources_without_changing_actual_network_or_state() {
    let (spec, mut store) = fixture();
    let before = store.clone();
    let fabric = spec.project(&store).unwrap();
    assert!(fabric.visibility("staff-a", "staff-b").unwrap().mutual());
    assert!(fabric.visibility("staff-a", "guest-a").unwrap().isolated());
    for path in [
        "/profiles",
        "/profiles/staff",
        "/bindings",
        "/bindings/desk-1",
    ] {
        let mut read = request("GET", path, "");
        read.authorization = "bEaReR lab-reader".into();
        let result = api::send(&spec, &mut store, &read).unwrap();
        assert_eq!(result.status, 200);
        assert!(!result.changed);
        assert_eq!(result.headers["Content-Type"], "application/json");
        assert!(serde_json::from_str::<serde_json::Value>(&result.body).is_ok());
        assert_eq!(store, before);
    }
    assert_eq!(spec.project(&store).unwrap(), fabric);
}

#[test]
fn patch_and_put_are_equivalent_when_the_full_representation_is_preserved() {
    let (spec, mut patch) = fixture();
    let mut put = patch.clone();
    assert_eq!(
        execute(
            &spec,
            &mut patch,
            "PATCH",
            "/profiles/staff",
            r#"{"vlan":20}"#
        )
        .status,
        204
    );
    assert_eq!(
        execute(
            &spec,
            &mut put,
            "PUT",
            "/profiles/staff",
            r#"{"description":"Staff workgroup","vlan":20,"up":true}"#
        )
        .status,
        204
    );
    assert_eq!(patch, put);
    let fabric = spec.project(&patch).unwrap();
    assert!(fabric.visibility("staff-a", "guest-a").unwrap().mutual());
    assert_eq!(
        patch.profiles["staff"].description.as_deref(),
        Some("Staff workgroup")
    );
    // An equivalent repeat succeeds but truthfully reports no state change.
    let again = execute(
        &spec,
        &mut patch,
        "PATCH",
        "/profiles/staff",
        r#"{"vlan":20}"#,
    );
    assert_eq!(again.status, 204);
    assert!(!again.changed);
    assert!(again.body.is_empty());
    assert!(again.headers.is_empty());
}

#[test]
fn create_rebind_and_retire_affect_real_ports_and_protect_referenced_profiles() {
    let (spec, mut store) = fixture();
    let created = execute(
        &spec,
        &mut store,
        "POST",
        "/profiles",
        r#"{"name":"replacement","up":true,"vlan":30}"#,
    );
    assert_eq!(created.status, 201);
    assert!(created.changed);
    assert_eq!(created.headers["Location"], "/profiles/replacement");
    assert!(
        spec.project(&store)
            .unwrap()
            .visibility("guest-a", "guest-b")
            .unwrap()
            .mutual()
    );
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "PATCH",
            "/bindings/desk-3",
            r#"{"profile":"replacement"}"#
        )
        .status,
        204
    );
    assert!(
        !spec
            .project(&store)
            .unwrap()
            .visibility("guest-a", "guest-b")
            .unwrap()
            .mutual()
    );
    assert_eq!(
        execute(&spec, &mut store, "DELETE", "/profiles/visitors", "").status,
        409
    );
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "PUT",
            "/bindings/desk-4",
            r#"{"profile":"replacement"}"#
        )
        .status,
        204
    );
    let final_fabric = spec.project(&store).unwrap();
    assert!(
        final_fabric
            .visibility("guest-a", "guest-b")
            .unwrap()
            .mutual()
    );
    assert!(
        final_fabric
            .visibility("staff-a", "guest-a")
            .unwrap()
            .isolated()
    );
    let deleted = execute(&spec, &mut store, "DELETE", "/profiles/visitors", "");
    assert_eq!(deleted.status, 204);
    assert!(deleted.changed);
    assert_eq!(spec.project(&store).unwrap(), final_fabric);
    assert_eq!(
        execute(&spec, &mut store, "DELETE", "/profiles/visitors", "").status,
        404
    );
}

#[test]
fn put_creates_at_named_path_post_conflicts_and_collections_are_not_editable_lists() {
    let (spec, mut store) = fixture();
    let response = execute(
        &spec,
        &mut store,
        "PUT",
        "/profiles/new-profile",
        r#"{"up":true,"vlan":30}"#,
    );
    assert_eq!(response.status, 201);
    assert_eq!(response.headers["Location"], "/profiles/new-profile");
    let before = store.clone();
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "POST",
            "/profiles",
            r#"{"name":"new-profile","up":false,"vlan":10}"#
        )
        .status,
        409
    );
    for (method, path, allow) in [
        ("PUT", "/profiles", "GET, POST"),
        ("DELETE", "/bindings/desk-1", "GET, PUT, PATCH"),
        ("POST", "/bindings", "GET"),
    ] {
        let r = execute(&spec, &mut store, method, path, "");
        assert_eq!(r.status, 405);
        assert_eq!(r.headers["Allow"], allow);
    }
    assert_eq!(store, before);
}

#[test]
fn missing_invalid_and_readonly_identities_are_distinct_and_never_modify_state() {
    let (spec, mut store) = fixture();
    let before = store.clone();
    for identity in [
        "",
        "Basic lab-operator",
        "Bearer bad",
        "Bearer Lab-operator",
        "Bearer  lab-operator",
        "Bearer lab-operator extra",
    ] {
        let mut req = request("PATCH", "/profiles/staff", r#"{"vlan":20}"#);
        req.authorization = identity.into();
        let r = api::send(&spec, &mut store, &req).unwrap();
        assert_eq!(r.status, 401);
        assert!(r.headers.contains_key("WWW-Authenticate"));
    }
    let mut req = request("PATCH", "/profiles/staff", r#"{"vlan":20}"#);
    req.authorization = "Bearer lab-reader".into();
    let forbidden = api::send(&spec, &mut store, &req).unwrap();
    assert_eq!(forbidden.status, 403);
    assert!(!forbidden.headers.contains_key("WWW-Authenticate"));
    assert_eq!(store, before);
}

#[test]
fn encoding_syntax_schema_and_projection_failures_are_specific_and_atomic() {
    let (spec, mut store) = fixture();
    let before = store.clone();
    let mut wrong_media = request("PATCH", "/profiles/staff", r#"{"vlan":20}"#);
    wrong_media.content_type = "application/json".into();
    assert_eq!(
        api::send(&spec, &mut store, &wrong_media).unwrap().status,
        415
    );
    for body in [
        r#"{vlan:20}"#,
        r#"{"vlan":10,"vlan":20}"#,
        r#"{"vlan":20} {}"#,
        r#"{"nested":{"up":true,"up":false}}"#,
    ] {
        let r = execute(&spec, &mut store, "PATCH", "/profiles/staff", body);
        assert_eq!(r.status, 400, "{body}: {}", r.body);
        assert!(r.body.contains("JSON"));
        assert_eq!(store, before);
    }
    for body in [
        r#"{"vlan":"20"}"#,
        r#"{"vlan":20.0}"#,
        r#"{"vlan":2e1}"#,
        r#"{"vlan":0}"#,
        r#"{"vlan":4095}"#,
        r#"{"up":"true"}"#,
        r#"{"vlan":30,"up":null}"#,
        r#"{"command":"touch /host"}"#,
        r#"{"vlan":99}"#,
        r#"{"description":"bad\ntext"}"#,
        r#"{"up":{"nested":true}}"#,
        r#"[true,20]"#,
        "null",
    ] {
        let r = execute(&spec, &mut store, "PATCH", "/profiles/staff", body);
        assert_eq!(r.status, 422, "{body}: {}", r.body);
        assert!(r.body.contains("No resource changed"));
        assert_eq!(store, before);
    }
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "PATCH",
            "/bindings/desk-1",
            r#"{"profile":"missing"}"#
        )
        .status,
        422
    );
    assert_eq!(store, before);
}

#[test]
fn patch_null_deletes_optional_fields_but_required_fields_remain_required() {
    let (spec, mut store) = fixture();
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "PATCH",
            "/profiles/staff",
            r#"{"description":null}"#
        )
        .status,
        204
    );
    assert!(store.profiles["staff"].description.is_none());
    let read = execute(&spec, &mut store, "GET", "/profiles/staff", "");
    assert!(
        !serde_json::from_str::<serde_json::Value>(&read.body)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("description")
    );
    let before = store.clone();
    for (method, body) in [
        ("PATCH", r#"{"vlan":null}"#),
        ("PUT", r#"{"vlan":20}"#),
        ("PUT", r#"{"up":true,"vlan":20,"description":null}"#),
    ] {
        assert_eq!(
            execute(&spec, &mut store, method, "/profiles/staff", body).status,
            422
        );
        assert_eq!(store, before);
    }
    // PUT replaces rather than retaining omitted optional fields.
    execute(
        &spec,
        &mut store,
        "PATCH",
        "/profiles/staff",
        r#"{"description":"temporary"}"#,
    );
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "PUT",
            "/profiles/staff",
            r#"{"up":true,"vlan":10}"#
        )
        .status,
        204
    );
    assert!(store.profiles["staff"].description.is_none());
}

#[test]
fn unsupported_or_escape_shaped_requests_are_explicit_and_never_touch_a_filesystem() {
    let (spec, mut store) = fixture();
    let before = store.clone();
    for path in [
        "../../host",
        "file:///etc/passwd",
        "https://host/profiles",
        "/profiles/../../host",
        "/profiles/%2e%2e",
        "/profiles/staff?x=1",
    ] {
        assert_eq!(
            execute(&spec, &mut store, "GET", path, "").status,
            400,
            "{path}"
        );
    }
    assert_eq!(
        execute(&spec, &mut store, "GET", "/unknown", "").status,
        404
    );
    assert_eq!(
        execute(&spec, &mut store, "GET", "/profiles/missing", "").status,
        404
    );
    for method in ["HEAD", "OPTIONS", "get", "CONNECT"] {
        assert_eq!(
            execute(&spec, &mut store, method, "/profiles", "").status,
            501
        );
    }
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "GET",
            "/profiles",
            r#"{"shell":"anything"}"#
        )
        .status,
        400
    );
    let mut req = request("PATCH", "/profiles/staff", "");
    req.authorization = "Bearer lab-operator\nX: injection".into();
    assert!(api::send(&spec, &mut store, &req).is_err());
    assert_eq!(store, before);
}

#[test]
fn resource_json_and_replay_inputs_have_finite_bounds_and_duplicates_refuse() {
    let (spec, mut store) = fixture();
    let before = store.clone();
    let nested = format!("{}0{}", "[".repeat(10), "]".repeat(10));
    assert_eq!(
        execute(&spec, &mut store, "PATCH", "/profiles/staff", &nested).status,
        400
    );
    let many = format!("[{}]", vec!["0"; 129].join(","));
    assert_eq!(
        execute(&spec, &mut store, "PATCH", "/profiles/staff", &many).status,
        400
    );
    let oversize = request("PATCH", "/profiles/staff", &" ".repeat(api::MAX_BODY + 1));
    assert!(api::send(&spec, &mut store, &oversize).is_err());
    for i in 0..9 {
        assert_eq!(
            execute(
                &spec,
                &mut store,
                "POST",
                "/profiles",
                &format!(r#"{{"name":"extra-{i}","up":true,"vlan":30}}"#)
            )
            .status,
            201
        );
    }
    let limit = store.clone();
    assert_eq!(store.profiles.len(), 12);
    assert_eq!(
        execute(
            &spec,
            &mut store,
            "POST",
            "/profiles",
            r#"{"name":"overflow","up":true,"vlan":30}"#
        )
        .status,
        409
    );
    assert_eq!(store, limit);
    let mut invalid = before.clone();
    invalid.bindings.remove("desk-1");
    assert!(spec.project(&invalid).is_err());
    let serialized = serde_json::to_string(&before).unwrap();
    let duplicate = serialized.replacen(
        "\"staff\":",
        "\"staff\":{\"up\":true,\"vlan\":10},\"staff\":",
        1,
    );
    assert!(serde_json::from_str::<api::Store>(&duplicate).is_err());
    assert_eq!(
        serde_json::from_str::<api::Store>(&serialized).unwrap(),
        before
    );
}
