#![allow(dead_code)]
#[path = "samba_share_fixture.rs"]
mod world_fixture;
use dlms_learning_lab::{
    filesystem::{Access, Requirement},
    samba_share::Credential,
    samba_share_workspace::*,
};
pub fn task() -> Task {
    let reference = world_fixture::world();
    let mut initial = reference.clone();
    initial.accounts.retain(|a| a.user != "blair");
    initial
        .accounts
        .iter_mut()
        .find(|a| a.user == "alex")
        .unwrap()
        .enabled = false;
    initial.files.nodes.get_mut("/srv/team").unwrap().mode = 0o700;
    let clients = vec![
        Client {
            id: "editor".into(),
            label: "Alex: supplied valid credential".into(),
            credential: world_fixture::valid("alex"),
        },
        Client {
            id: "reviewer".into(),
            label: "Blair: supplied valid credential".into(),
            credential: world_fixture::valid("blair"),
        },
        Client {
            id: "anonymous".into(),
            label: "Anonymous client".into(),
            credential: Credential::Anonymous,
        },
    ];
    let read = |id: &str, text: &str| Case {
        id: id.into(),
        label: format!("Editor reads {id}"),
        inputs: vec![Input {
            path: "/srv/team/draft.txt".into(),
            text: text.into(),
        }],
        request: Request {
            client: "editor".into(),
            share: "team".into(),
            operation: Operation::Read {
                path: "draft.txt".into(),
            },
        },
        expected: Expected::Read {
            path: "/srv/team/draft.txt".into(),
            text: text.into(),
        },
    };
    let cases = vec![
        read("read-first", "first sample\n"),
        read("read-second", "another sample\n"),
        Case {
            id: "writer".into(),
            label: "Editor replaces the shared draft".into(),
            inputs: vec![],
            request: Request {
                client: "editor".into(),
                share: "team".into(),
                operation: Operation::Replace {
                    path: "draft.txt".into(),
                    text: "replacement\n".into(),
                },
            },
            expected: Expected::Replaced {
                path: "/srv/team/draft.txt".into(),
                text: "replacement\n".into(),
            },
        },
        Case {
            id: "reviewer-read".into(),
            label: "Reviewer reads the draft".into(),
            inputs: vec![],
            request: Request {
                client: "reviewer".into(),
                share: "team".into(),
                operation: Operation::Read {
                    path: "draft.txt".into(),
                },
            },
            expected: Expected::Read {
                path: "/srv/team/draft.txt".into(),
                text: "draft\n".into(),
            },
        },
        Case {
            id: "guest-denied".into(),
            label: "Reject anonymous team access".into(),
            inputs: vec![],
            request: Request {
                client: "anonymous".into(),
                share: "team".into(),
                operation: Operation::Read {
                    path: "draft.txt".into(),
                },
            },
            expected: Expected::Denied {
                layers: vec![Denial::Authentication, Denial::SharePolicy],
            },
        },
        Case {
            id: "private-denied".into(),
            label: "Private file stays unreadable to editor".into(),
            inputs: vec![],
            request: Request {
                client: "editor".into(),
                share: "team".into(),
                operation: Operation::Read {
                    path: "private.txt".into(),
                },
            },
            expected: Expected::Denied {
                layers: vec![Denial::UnixAccess],
            },
        },
    ];
    Task {
        id: "identity-access".into(),
        initial,
        reference,
        shares: vec![
            Name { id: "team".into() },
            Name {
                id: "public".into(),
            },
        ],
        editable_accounts: vec![Name { id: "alex".into() }, Name { id: "blair".into() }],
        policy: vec![],
        accounts: vec![
            AccountGoal {
                user: "alex".into(),
                enabled: true,
            },
            AccountGoal {
                user: "blair".into(),
                enabled: true,
            },
        ],
        clients,
        filesystem: vec![
            Requirement::Contents {
                path: "/srv/team/private.txt".into(),
                text: "private fixture\n".into(),
            },
            Requirement::Contents {
                path: "/public/notice.txt".into(),
                text: "notice\n".into(),
            },
            Requirement::Access {
                path: "/srv/team/draft.txt".into(),
                user: "casey".into(),
                operation: Access::ReadFile,
                allowed: false,
            },
        ],
        trial: cases[0].request.clone(),
        cases,
    }
}
pub fn definition() -> Definition {
    let assignment = task();
    let mut example = assignment.clone();
    example.id = "guide-identity-access".into();
    example.initial.server = "guide-server".into();
    example.reference.server = "guide-server".into();
    Definition {
        schema_version: 1,
        version: 1,
        id: "share-unit-fixture".into(),
        assignment,
        example,
    }
}
