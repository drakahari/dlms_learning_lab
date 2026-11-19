use dlms_learning_lab::{
    engine::Node,
    filesystem::Tree,
    ssh_identity::PublicKey,
    ssh_key_access::{self as access, Authorization, Finding},
};
use std::collections::BTreeMap;
const PATH: &str = "/home/alex/.ssh/authorized_keys";
fn key() -> PublicKey {
    PublicKey::parse(
        "ssh-ed25519",
        "AAAAC3NzaC1lZDI1NTE5AAAAINdamAGCsQq31Uv+08lkBzoO4XLz2qYjJa8CGmj3B1Ea",
    )
    .unwrap()
}
fn tree() -> Tree {
    let mut nodes = BTreeMap::new();
    for (p, owner, group, mode, directory) in [
        ("/", "root", "root", 0o755, true),
        ("/home", "root", "root", 0o755, true),
        ("/home/alex", "alex", "staff", 0o750, true),
        ("/home/alex/.ssh", "alex", "staff", 0o700, true),
        (PATH, "alex", "staff", 0o600, false),
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
    Tree {
        nodes,
        files: BTreeMap::from([(PATH.into(), key().line())]),
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
    }
}
fn check(t: &Tree, strict: bool) -> Result<Authorization, String> {
    access::check(t, "alex", &[".ssh/authorized_keys".into()], key(), strict)
}
#[test]
fn accepts_realistic_alternate_modes_and_root_owned_group_readable_key_files() {
    let mut t = tree();
    assert!(t.valid());
    assert!(matches!(
        check(&t, true).unwrap(),
        Authorization::Allowed { .. }
    ));
    for mode in [0o400, 0o600, 0o640, 0o644] {
        t.nodes.get_mut(PATH).unwrap().mode = mode;
        assert!(
            matches!(check(&t, true).unwrap(), Authorization::Allowed { .. }),
            "{mode:o}"
        );
    }
    let n = t.nodes.get_mut(PATH).unwrap();
    n.owner = "root".into();
    n.group = "staff".into();
    n.mode = 0o640;
    t.nodes.get_mut("/home/alex/.ssh").unwrap().mode = 0o755;
    assert!(matches!(
        check(&t, true).unwrap(),
        Authorization::Allowed { .. }
    ));
}
#[test]
fn diagnoses_file_owner_file_mode_home_and_directory_traversal_separately() {
    let mut t = tree();
    t.nodes.get_mut(PATH).unwrap().mode = 0o666;
    assert!(
        matches!(check(&t,true).unwrap(),Authorization::Denied{findings}if findings==vec![Finding::UnsafeMode(PATH.into())])
    );
    assert!(matches!(
        check(&t, false).unwrap(),
        Authorization::Allowed { .. }
    ));
    let mut t = tree();
    let n = t.nodes.get_mut(PATH).unwrap();
    n.owner = "blair".into();
    n.mode = 0o644;
    assert!(
        matches!(check(&t,true).unwrap(),Authorization::Denied{findings}if findings==vec![Finding::UnsafeOwner(PATH.into())])
    );
    let mut t = tree();
    t.nodes.get_mut("/home/alex").unwrap().mode = 0o770;
    assert!(
        matches!(check(&t,true).unwrap(),Authorization::Denied{findings}if findings==vec![Finding::UnsafeMode("/home/alex".into())])
    );
    let mut t = tree();
    t.nodes.get_mut("/home/alex/.ssh").unwrap().mode = 0o600;
    assert!(
        matches!(check(&t,true).unwrap(),Authorization::Denied{findings}if findings==vec![Finding::Unreadable(PATH.into())])
    );
}
#[test]
fn wrong_public_key_and_empty_locations_do_not_become_authentication_success() {
    let mut t = tree();
    let mut wrong = key();
    wrong.bytes[0] ^= 1;
    t.files.insert(
        PATH.into(),
        format!("{} alex approved comment", wrong.line()),
    );
    assert!(
        matches!(check(&t,true).unwrap(),Authorization::Denied{findings}if findings==vec![Finding::NoMatchingKey(PATH.into())])
    );
    assert!(
        matches!(access::check(&t,"alex",&[],key(),true).unwrap(),Authorization::Denied{findings}if findings.is_empty())
    );
}
#[test]
fn checks_all_supported_locations_but_does_not_silently_evaluate_unsupported_key_options() {
    let mut t = tree();
    let second = "/home/alex/.ssh/authorized_keys2";
    t.nodes.insert(second.into(), t.nodes[PATH].clone());
    t.files.insert(second.into(), key().line());
    t.files.insert(PATH.into(), String::new());
    let paths = vec![
        ".ssh/authorized_keys".into(),
        ".ssh/authorized_keys2".into(),
    ];
    assert!(
        matches!(access::check(&t,"alex",&paths,key(),true).unwrap(),Authorization::Allowed{path}if path==second)
    );
    t.files.insert(
        PATH.into(),
        format!("command=\"echo allowed\" {}", key().line()),
    );
    assert!(
        access::check(&t, "alex", &paths, key(), true)
            .unwrap_err()
            .contains("cannot evaluate this key file")
    );
}
#[test]
fn decisions_are_fresh_readonly_and_cannot_follow_paths_outside_the_subset() {
    let mut t = tree();
    let before = serde_json::to_vec(&t).unwrap();
    assert!(matches!(
        check(&t, true).unwrap(),
        Authorization::Allowed { .. }
    ));
    assert_eq!(serde_json::to_vec(&t).unwrap(), before);
    t.nodes.get_mut(PATH).unwrap().mode = 0o000;
    assert!(matches!(
        check(&t, false).unwrap(),
        Authorization::Denied { .. }
    ));
    for name in ["root", "unknown", "../alex"] {
        assert!(access::check(&t, name, &[".ssh/authorized_keys".into()], key(), true).is_err());
    }
    for path in [
        "../authorized_keys",
        "/etc/passwd",
        ".ssh/../authorized_keys",
    ] {
        assert!(access::check(&t, "alex", &[path.into()], key(), true).is_err());
    }
}
