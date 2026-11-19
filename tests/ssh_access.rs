use dlms_learning_lab::{
    engine::Node,
    filesystem::Tree,
    ssh_access::{Outcome, Request, Scope, World},
    ssh_identity::PublicKey,
};
use std::collections::BTreeMap;
fn key() -> PublicKey {
    PublicKey::parse(
        "ssh-ed25519",
        "AAAAC3NzaC1lZDI1NTE5AAAAINdamAGCsQq31Uv+08lkBzoO4XLz2qYjJa8CGmj3B1Ea",
    )
    .unwrap()
}
fn setup() -> (Scope, World, Request) {
    let mut nodes = BTreeMap::new();
    for (p, owner, group, mode, directory) in [
        ("/", "root", "root", 0o755, true),
        ("/home", "root", "root", 0o755, true),
        ("/home/alex", "alex", "staff", 0o750, true),
        ("/home/alex/.ssh", "alex", "staff", 0o700, true),
        (
            "/home/alex/.ssh/authorized_keys",
            "alex",
            "staff",
            0o600,
            false,
        ),
    ] {
        nodes.insert(
            p.into(),
            Node {
                owner: owner.into(),
                group: group.into(),
                mode,
                directory,
            },
        );
    }
    let tree = Tree {
        nodes,
        files: BTreeMap::from([("/home/alex/.ssh/authorized_keys".into(), key().line())]),
        users: BTreeMap::from([
            ("alex".into(), vec!["staff".into()]),
            ("blair".into(), vec!["guests".into()]),
            ("root".into(), vec!["root".into()]),
        ]),
        user: "alex".into(),
        home: "/home/alex".into(),
        cwd: "/home/alex".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    let mut other = key();
    other.bytes[1] ^= 1;
    let s = Scope {
        host: "server.lab.test".into(),
        host_key: other,
        identities: BTreeMap::from([("admin-key".into(), key()), ("other-key".into(), other)]),
        initial_tree: tree.clone(),
        editable_metadata: vec![
            "/home/alex".into(),
            "/home/alex/.ssh".into(),
            "/home/alex/.ssh/authorized_keys".into(),
        ],
        editable_text: vec!["/home/alex/.ssh/authorized_keys".into()],
    };
    let config="PermitRootLogin no\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nStrictModes yes\nAllowUsers alex\nAuthorizedKeysFile .ssh/authorized_keys\n".to_owned();
    let w = World {
        version: 1,
        tree,
        known_hosts: format!("server.lab.test {}", other.line()),
        saved_config: config.clone(),
        active_config: Some(config),
    };
    (
        s,
        w,
        Request {
            host: "server.lab.test".into(),
            user: "alex".into(),
            identity: "admin-key".into(),
        },
    )
}
#[test]
fn source_inventory_and_mutable_state_produce_fresh_outcomes_for_multiple_identities() {
    let (s, mut w, r) = setup();
    assert!(s.evaluate(&w, &r).unwrap().permitted());
    let before = w.clone();
    let mut wrong = r.clone();
    wrong.identity = "other-key".into();
    assert!(matches!(
        s.evaluate(&w, &wrong).unwrap(),
        Outcome::KeyRejected { .. }
    ));
    assert_eq!(w, before);
    w.tree
        .nodes
        .get_mut("/home/alex/.ssh/authorized_keys")
        .unwrap()
        .mode = 0o666;
    assert!(!s.evaluate(&w, &r).unwrap().permitted());
}
#[test]
fn server_trust_and_user_authorization_are_distinct_and_no_unknown_server_is_contacted() {
    let (s, mut w, r) = setup();
    w.known_hosts.clear();
    assert_eq!(s.evaluate(&w, &r).unwrap(), Outcome::UnknownHost);
    w.known_hosts = format!("server.lab.test {}", key().line());
    assert_eq!(s.evaluate(&w, &r).unwrap(), Outcome::ChangedHostKey);
    let mut foreign = r;
    foreign.host = "outside.example".into();
    assert!(
        s.evaluate(&w, &foreign)
            .unwrap_err()
            .contains("No other server")
    );
}
#[test]
fn saved_policy_requires_reload_and_a_failed_reload_keeps_the_previous_active_copy() {
    let (s, mut w, r) = setup();
    w.saved_config = "PubkeyAuthentication no".into();
    assert!(!w.published());
    assert!(s.evaluate(&w, &r).unwrap().permitted());
    w.reload().unwrap();
    assert!(w.published());
    assert_eq!(s.evaluate(&w, &r).unwrap(), Outcome::MethodDisabled);
    let active = w.active_config.clone();
    w.saved_config = "Match unsupported".into();
    assert!(w.reload().is_err());
    assert_eq!(w.active_config, active);
    assert!(!w.published());
}
#[test]
fn equivalent_supported_configuration_does_not_require_exact_comments_order_or_duplicate_names() {
    let (_s, mut w, _r) = setup();
    w.saved_config="AllowUsers alex alex\nPermitRootLogin no # same\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nStrictModes yes\nAuthorizedKeysFile .ssh/authorized_keys\n".into();
    assert!(w.published());
    w.saved_config.push_str("PubkeyAuthentication no\n");
    assert!(!w.published());
}
#[test]
fn corrupt_scope_or_forged_user_inventory_cannot_turn_failure_into_permitted_access() {
    let (s, mut w, r) = setup();
    w.tree.users.get_mut("alex").unwrap().push("root".into());
    assert!(s.evaluate(&w, &r).is_err());
    let (mut s, w, r) = setup();
    s.editable_metadata.push("/".into());
    assert!(s.evaluate(&w, &r).is_err());
    let (s, mut w, r) = setup();
    w.active_config = None;
    assert_eq!(s.evaluate(&w, &r).unwrap(), Outcome::NoActivePolicy);
    let mut root = r;
    root.user = "root".into();
    w.active_config = Some("PermitRootLogin yes".into());
    assert!(
        s.evaluate(&w, &root).is_err(),
        "Unsupported root path checking must not count as a denied authentication case"
    );
}

#[test]
fn refusal_at_the_wrong_stage_or_unsupported_behavior_never_earns_a_denial_check() {
    use dlms_learning_lab::ssh_access::Expected;
    let (s, mut w, mut r) = setup();
    r.identity = "other-key".into();
    assert!(s.check_expected(&w, &r, Expected::KeyDenies).0);
    w.known_hosts.clear();
    let (pass, feedback) = s.check_expected(&w, &r, Expected::KeyDenies);
    assert!(!pass);
    assert!(feedback.contains("Needs attention"));
    assert!(feedback.contains("not yet trusted") || feedback.contains("No saved public key"));
    assert!(s.check_expected(&w, &r, Expected::UnknownHost).0);
    let (s, mut w, mut r) = setup();
    r.user = "root".into();
    w.active_config = Some("PermitRootLogin yes".into());
    let (pass, feedback) = s.check_expected(&w, &r, Expected::PolicyDenies);
    assert!(!pass);
    assert!(feedback.starts_with("Not evaluated:"));
}
#[test]
fn imported_active_policy_and_edit_lists_have_early_size_bounds() {
    let (s, w, _) = setup();
    let mut value = serde_json::to_value(w).unwrap();
    value["active_config"] = serde_json::Value::String("x".repeat(16385));
    assert!(serde_json::from_value::<World>(value).is_err());
    let mut value = serde_json::to_value(s).unwrap();
    value["editable_metadata"] = serde_json::json!(vec!["/home/alex"; 33]);
    assert!(serde_json::from_value::<Scope>(value).is_err());
}
