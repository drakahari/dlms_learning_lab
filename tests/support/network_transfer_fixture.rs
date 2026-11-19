// Different focused binaries use subsets of this fixture helper.
#![allow(dead_code)]
use dlms_learning_lab::{acl_editor, acl_workspace, network_transfer as work};
use std::collections::BTreeMap;
pub fn fixture(text: &str) -> (work::Context, work::World) {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../../scenarios/transport-lessons-v1.json")).unwrap();
    let network = serde_json::from_value(
        p["scenarios"][0]["goal"]["definition"]["assignment"]["network"].clone(),
    )
    .unwrap();
    let read = work::Access {
        read: vec!["public/".into()],
        write: vec![],
    };
    let writer = work::Access {
        read: vec!["public/".into(), "archive/".into()],
        write: vec!["archive/".into()],
    };
    let c = work::Context {
        version: 1,
        network,
        client: "Client".into(),
        client_port: 49152,
        servers: BTreeMap::from([(
            "Server".into(),
            work::Server {
                tftp: true,
                ftp: true,
                tftp_tid: 50000,
                passive_port: 50001,
                tftp_access: writer.clone(),
                logins: BTreeMap::from([("reader".into(), read), ("archivist".into(), writer)]),
            },
        )]),
    };
    let w = work::World {
        files: BTreeMap::from([
            (
                "Client".into(),
                BTreeMap::from([
                    ("startup.cfg".into(), "CURRENT\n".into()),
                    ("keep.txt".into(), "SAFETY\n".into()),
                ]),
            ),
            (
                "Server".into(),
                BTreeMap::from([
                    ("public/recovery.cfg".into(), text.into()),
                    ("archive/keep.cfg".into(), "AUDIT\n".into()),
                ]),
            ),
        ]),
    };
    w.validate(&c).unwrap();
    (c, w)
}
pub fn request(protocol: work::Protocol, direction: work::Direction) -> work::Request {
    work::Request {
        protocol,
        direction,
        server: "Server".into(),
        local_path: match direction {
            work::Direction::Download => "staged.cfg",
            work::Direction::Upload => "startup.cfg",
        }
        .into(),
        remote_path: match direction {
            work::Direction::Download => "public/recovery.cfg",
            work::Direction::Upload => "archive/site.cfg",
        }
        .into(),
        login: (protocol == work::Protocol::Ftp).then(|| "archivist".into()),
    }
}
pub fn policy(
    c: &work::Context,
    forward: &str,
    back: &str,
) -> dlms_learning_lab::acl_routing::Network {
    let doc = format!(
        "ip access-list extended OUTWARD\n{forward}\n 100 deny ip any any\nexit\nip access-list extended INWARD\n{back}\n 100 deny ip any any\nexit\ninterface office\n ip access-group OUTWARD in\n ip access-group INWARD out\nexit\n"
    );
    acl_workspace::apply_documents(
        &c.network,
        &acl_editor::Drafts {
            schema_version: 1,
            routers: BTreeMap::from([("R1".into(), doc)]),
        },
    )
    .unwrap()
}

use dlms_learning_lab::network_transfer_workspace as state;
pub fn task() -> state::Task {
    let (c, w) = fixture("first configuration\n");
    let mut second = w.clone();
    second.files.get_mut("Server").unwrap().insert(
        "public/recovery.cfg".into(),
        "DIFFERENT second configuration without a final newline".into(),
    );
    let required = vec![
        state::Requirement::Copied {
            source: state::FileRef {
                host: "Server".into(),
                path: "public/recovery.cfg".into(),
            },
            destination: state::FileRef {
                host: "Client".into(),
                path: "staged.cfg".into(),
            },
        },
        state::Requirement::Unchanged {
            file: state::FileRef {
                host: "Client".into(),
                path: "startup.cfg".into(),
            },
        },
        state::Requirement::Ready {
            request: request(work::Protocol::Tftp, work::Direction::Download),
        },
    ];
    let t = state::Task {
        id: "test-transfer-assignment".into(),
        cases: vec![
            state::Case {
                name: "First supplied file".into(),
                context: c.clone(),
                initial: w,
                requirements: required.clone(),
            },
            state::Case {
                name: "Changed source bytes".into(),
                context: c,
                initial: second,
                requirements: required,
            },
        ],
        editable_routers: vec!["R1".into()],
        starter: request(work::Protocol::Tftp, work::Direction::Download),
    };
    t.validate().unwrap();
    t
}
pub fn definition() -> state::Definition {
    let assignment = task();
    // Separate virtual identities/files, without changing any host settings.
    let raw = serde_json::to_string(&assignment)
        .unwrap()
        .replace("\"Client\"", "\"Trainer\"")
        .replace("first configuration", "worked gallery")
        .replace(
            "DIFFERENT second configuration without a final newline",
            "another training input",
        );
    let mut example: state::Task = serde_json::from_str(&raw).unwrap();
    example.id = "transfer-practice".into();
    let worked_example = vec![example.new_draft().unwrap()];
    let d = state::Definition {
        id: "test-transfer-unit".into(),
        version: 1,
        assignment,
        example,
        worked_example,
    };
    d.validate().unwrap();
    d
}
