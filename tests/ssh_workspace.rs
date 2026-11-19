use dlms_learning_lab::{
    engine::Node,
    filesystem::Tree,
    ssh_access::{Expected, Scope, World},
    ssh_identity::PublicKey,
    ssh_workspace::{
        Attempt, Definition, Document, Event, Fixture, PolicyRule, Requirement, Session,
    },
};
use std::collections::BTreeMap;
fn key() -> PublicKey {
    PublicKey::parse(
        "ssh-ed25519",
        "AAAAC3NzaC1lZDI1NTE5AAAAINdamAGCsQq31Uv+08lkBzoO4XLz2qYjJa8CGmj3B1Ea",
    )
    .unwrap()
}
fn fixture(user: &str, host: &str, mut host_key: PublicKey) -> Fixture {
    let home = format!("/home/{user}");
    let ssh = format!("{home}/.ssh");
    let path = format!("{ssh}/authorized_keys");
    host_key.bytes[0] ^= 1;
    let mut wrong = key();
    wrong.bytes[0] ^= 2;
    let mut nodes = BTreeMap::new();
    for (p, owner, group, mode, directory) in [
        ("/".into(), "root", "root", 0o755, true),
        ("/home".into(), "root", "root", 0o755, true),
        (home.clone(), user, "staff", 0o750, true),
        (ssh.clone(), user, "staff", 0o700, true),
        (path.clone(), user, "staff", 0o666, false),
    ] {
        nodes.insert(
            p,
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
        files: BTreeMap::from([(path.clone(), wrong.line())]),
        users: BTreeMap::from([
            (user.into(), vec!["staff".into()]),
            ("root".into(), vec!["root".into()]),
        ]),
        user: user.into(),
        home: home.clone(),
        cwd: home.clone(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    let scope = Scope {
        host: host.into(),
        host_key,
        identities: BTreeMap::from([("admin-key".into(), key()), ("retired-key".into(), wrong)]),
        initial_tree: tree.clone(),
        editable_metadata: vec![home.clone(), ssh.clone(), path.clone()],
        editable_text: vec![path.clone()],
    };
    let config="StrictModes yes\nPermitRootLogin no\nPasswordAuthentication yes\nAuthorizedKeysFile .ssh/authorized_keys\n".to_owned();
    let world = World {
        version: 1,
        tree,
        known_hosts: format!("backup.lab.test {}", wrong.line()),
        saved_config: config.clone(),
        active_config: Some(config),
    };
    Fixture {
        scope,
        world,
        documents: vec![
            Document::KnownHosts,
            Document::ServerConfig,
            Document::KeyFile { path: path.clone() },
            Document::Attributes { path: home },
            Document::Attributes { path: ssh },
        ],
        requirements: vec![
            Requirement::Access {
                label: "Administrator identity works".into(),
                host: host.into(),
                user: user.into(),
                identity: "admin-key".into(),
                expected: Expected::Permitted,
            },
            Requirement::Access {
                label: "Retired identity is refused".into(),
                host: host.into(),
                user: user.into(),
                identity: "retired-key".into(),
                expected: Expected::KeyDenies,
            },
            Requirement::Published {},
            Requirement::Policy {
                rule: PolicyRule::StrictModes,
            },
            Requirement::Policy {
                rule: PolicyRule::OnlyPublicKey,
            },
            Requirement::KeySet {
                path,
                required: vec!["admin-key".into()],
                forbidden: vec!["retired-key".into()],
            },
            Requirement::PreserveTrust {
                host: "backup.lab.test".into(),
            },
        ],
    }
}
fn solution(f: &Fixture) -> Vec<Event> {
    let path = f.scope.editable_text[0].clone();
    let config = format!(
        "# equivalent supported policy\nAllowUsers {}\nAuthenticationMethods publickey\nPermitRootLogin no\nStrictModes yes\nAuthorizedKeysFile .ssh/authorized_keys\n",
        f.world.tree.user
    );
    vec![
        Event::Text {
            document: Document::KnownHosts,
            text: format!(
                "{}\n{} {}\n",
                f.world.known_hosts,
                f.scope.host,
                f.scope.host_key.line()
            ),
        },
        Event::Save {
            document: Document::KnownHosts,
        },
        Event::Text {
            document: Document::KeyFile { path: path.clone() },
            text: format!("{} admin supplied identity\n", key().line()),
        },
        Event::Metadata {
            path: path.clone(),
            owner: f.world.tree.user.clone(),
            group: "staff".into(),
            mode: "0644".into(),
        },
        Event::Save {
            document: Document::KeyFile { path },
        },
        Event::Text {
            document: Document::ServerConfig,
            text: config,
        },
        Event::Save {
            document: Document::ServerConfig,
        },
        Event::ApplyPolicy {},
        Event::Check {},
    ]
}
fn definition() -> Definition {
    let mut example_key = key();
    example_key.bytes[1] ^= 4;
    let example = fixture("sam", "practice.lab.test", example_key);
    let example_solution = solution(&example);
    Definition {
        id: "test-ssh-assignment".into(),
        objective: "Repair the supplied access conditions without weakening policy.".into(),
        initial: fixture("alex", "assignment.lab.test", key()),
        example,
        example_solution,
    }
}
#[test]
fn complete_worked_and_independent_repairs_use_real_model_state_with_alternate_modes() {
    let d = definition();
    d.validate().unwrap();
    let mut session = Session::new(&d);
    assert!(!session.assignment.solved(&d.initial));
    for e in solution(&d.initial) {
        session.apply(&d, &e, false).unwrap();
    }
    assert!(session.assignment.solved(&d.initial));
    assert!(!session.practice.solved(&d.example));
    let mut invalid = d.clone();
    invalid.example.scope.host = d.initial.scope.host.clone();
    assert!(invalid.validate().is_err());
}
#[test]
fn fill_only_populates_separate_example_drafts_never_applies_checks_or_assignment_credit() {
    let d = definition();
    let mut s = Session::new(&d);
    let assignment = s.assignment.clone();
    let practice = s.practice.world.clone();
    s.fill_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.practice.world, practice);
    assert!(!s.practice.checked);
    assert!(!s.practice.drafts.matches(&practice));
    for e in &d.example_solution {
        if matches!(
            e,
            Event::Save { .. } | Event::ApplyPolicy {} | Event::Check {}
        ) {
            s.apply(&d, e, true).unwrap();
        }
    }
    assert!(s.practice.solved(&d.example));
    assert!(!s.assignment.solved(&d.initial));
}
#[test]
fn draft_edits_clear_success_and_failed_atomic_saves_leave_the_old_world_intact() {
    let d = definition();
    let mut a = Attempt::new(&d.initial);
    for e in solution(&d.initial) {
        a.apply(&d.initial, &e).unwrap();
    }
    assert!(a.solved(&d.initial));
    a.apply(
        &d.initial,
        &Event::Text {
            document: Document::KnownHosts,
            text: "bad known hosts".into(),
        },
    )
    .unwrap();
    assert!(!a.solved(&d.initial));
    let before = a.world.clone();
    assert!(
        a.apply(
            &d.initial,
            &Event::Save {
                document: Document::KnownHosts
            }
        )
        .is_err()
    );
    assert_eq!(a.world, before);
    a.apply(&d.initial, &Event::Check {}).unwrap();
    assert!(
        !a.solved(&d.initial),
        "Old healthy world cannot mask a broken unfinished draft"
    );
}
#[test]
fn partial_edits_wrong_identity_and_saved_only_policy_do_not_pass() {
    let d = definition();
    let mut a = Attempt::new(&d.initial);
    let actions = solution(&d.initial);
    for e in &actions[..7] {
        a.apply(&d.initial, e).unwrap();
    }
    a.apply(&d.initial, &Event::Check {}).unwrap();
    assert!(!a.solved(&d.initial));
    a.apply(&d.initial, &Event::ApplyPolicy {}).unwrap();
    a.apply(&d.initial, &Event::Check {}).unwrap();
    assert!(a.solved(&d.initial));
    let path = d.initial.scope.editable_text[0].clone();
    a.apply(
        &d.initial,
        &Event::Text {
            document: Document::KeyFile { path: path.clone() },
            text: d.initial.scope.identities["retired-key"].line(),
        },
    )
    .unwrap();
    a.apply(
        &d.initial,
        &Event::Save {
            document: Document::KeyFile { path },
        },
    )
    .unwrap();
    a.apply(&d.initial, &Event::Check {}).unwrap();
    assert!(!a.solved(&d.initial));
}
#[test]
fn session_resume_keeps_malformed_drafts_and_reset_example_leaves_assignment_exact() {
    let d = definition();
    let mut s = Session::new(&d);
    s.apply(
        &d,
        &Event::Text {
            document: Document::ServerConfig,
            text: "unfinished independent config".into(),
        },
        false,
    )
    .unwrap();
    s.fill_example(&d).unwrap();
    let raw = serde_json::to_vec(&s).unwrap();
    let mut resumed: Session = serde_json::from_slice(&raw).unwrap();
    assert_eq!(resumed, s);
    assert!(resumed.valid_for(&d));
    let assignment = resumed.assignment.clone();
    resumed.reset_example(&d);
    assert_eq!(resumed.assignment, assignment);
    assert_eq!(resumed.practice, Attempt::new(&d.example));
}
#[test]
fn invalid_documents_source_references_and_unreachable_examples_are_rejected() {
    let d = definition();
    let mut bad = d.clone();
    bad.example_solution
        .retain(|e| !matches!(e, Event::ApplyPolicy {}));
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.documents.push(Document::KeyFile {
        path: "/etc/passwd".into(),
    });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.requirements.push(Requirement::KeySet {
        path: d.initial.scope.editable_text[0].clone(),
        required: vec!["not-supplied".into()],
        forbidden: vec![],
    });
    assert!(bad.validate().is_err());
    let mut s = Session::new(&d);
    assert!(
        s.apply(
            &d,
            &Event::Metadata {
                path: "/".into(),
                owner: "root".into(),
                group: "root".into(),
                mode: "777".into()
            },
            false
        )
        .is_err()
    );
}

#[test]
fn failed_session_save_keeps_specific_feedback_and_does_not_mutate_applied_data() {
    let d = definition();
    let mut s = Session::new(&d);
    s.apply(
        &d,
        &Event::Text {
            document: Document::KnownHosts,
            text: "missing fields".into(),
        },
        false,
    )
    .unwrap();
    let before = s.assignment.world.clone();
    assert!(
        s.apply(
            &d,
            &Event::Save {
                document: Document::KnownHosts
            },
            false
        )
        .is_err()
    );
    assert_eq!(s.assignment.world, before);
    assert!(s.assignment.notice.as_ref().unwrap().contains("Line 1"));
    assert!(!s.assignment.solved(&d.initial));
}
#[test]
fn positive_only_duplicate_or_empty_requirements_cannot_masquerade_as_a_complete_task() {
    let d = definition();
    let mut bad = d.clone();
    bad.initial.requirements.retain(|r| {
        !matches!(
            r,
            Requirement::Access {
                expected: Expected::KeyDenies,
                ..
            }
        )
    });
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial
        .requirements
        .push(bad.initial.requirements[0].clone());
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    let row = bad
        .initial
        .requirements
        .iter_mut()
        .find(|r| matches!(r, Requirement::KeySet { .. }))
        .unwrap();
    let Requirement::KeySet {
        required,
        forbidden,
        ..
    } = row
    else {
        unreachable!()
    };
    required.clear();
    forbidden.clear();
    assert!(
        bad.validate()
            .unwrap_err()
            .contains("Invalid required/forbidden")
    );
}
#[test]
fn malformed_identity_requirement_fails_without_panicking_or_claiming_a_grade() {
    let d = definition();
    let r = Requirement::KeySet {
        path: d.initial.scope.editable_text[0].clone(),
        required: vec!["not-provided".into()],
        forbidden: vec![],
    };
    let (pass, message) = r.check(&d.initial, &d.initial.world);
    assert!(!pass);
    assert!(message.contains("unknown supplied identity"));
}

#[test]
fn source_binding_prevents_resuming_a_similar_but_different_assignment() {
    let d = definition();
    let s = Session::new(&d);
    let mut foreign = d.clone();
    foreign.id = "other-ssh-assignment".into();
    assert!(s.bound(&foreign).is_err());
    assert!(!foreign.checks(&s, false).iter().all(|r| r.0));
}
