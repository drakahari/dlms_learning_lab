use dlms_learning_lab::{
    ftp_identity::Credential,
    ftp_identity_task::{Case, Denial, Expected, Input, Operation, Step, Task},
};
use crate::fixture;
fn case(id: &str, user: &str, steps: Vec<Step>) -> Case {
    Case {
        id: id.into(),
        label: id.replace('-', " "),
        credential: Credential::Valid { user: user.into() },
        denied_login: None,
        confined: true,
        inputs: vec![],
        steps,
    }
}
fn step(operation: Operation, expected: Expected) -> Step {
    Step {
        operation,
        expected,
    }
}
pub fn task() -> Task {
    let mut cases = vec![];
    for (id, payload) in [
        ("download-first", "first supplied input\n"),
        ("download-second", "second distinct input\n"),
    ] {
        let mut c = case(
            id,
            "alex",
            vec![
                step(Operation::Pwd, Expected::Directory { path: "/".into() }),
                step(
                    Operation::Get {
                        path: "readme".into(),
                    },
                    Expected::Download {
                        text: payload.into(),
                    },
                ),
            ],
        );
        c.inputs = vec![Input {
            path: "/srv/alex/readme".into(),
            text: payload.into(),
        }];
        cases.push(c);
    }
    for (id, payload) in [
        ("upload-first", "first upload\n"),
        ("upload-second", "second upload\n"),
    ] {
        cases.push(case(
            id,
            "alex",
            vec![
                step(
                    Operation::Cd {
                        path: "inbox".into(),
                    },
                    Expected::Directory {
                        path: "/inbox".into(),
                    },
                ),
                step(
                    Operation::Put {
                        path: "report".into(),
                        text: payload.into(),
                    },
                    Expected::Uploaded {
                        mode: None,
                        server_path: "/srv/alex/inbox/report".into(),
                        text: payload.into(),
                    },
                ),
                step(
                    Operation::Get {
                        path: "report".into(),
                    },
                    Expected::Download {
                        text: payload.into(),
                    },
                ),
            ],
        ));
    }
    cases.push(case(
        "outside",
        "alex",
        vec![step(
            Operation::Get {
                path: "/etc/secret".into(),
            },
            Expected::Denied {
                layer: Denial::Missing,
            },
        )],
    ));
    cases.push(case(
        "private",
        "alex",
        vec![step(
            Operation::Get {
                path: "private".into(),
            },
            Expected::Denied {
                layer: Denial::Permission,
            },
        )],
    ));
    cases.push(case(
        "second-identity",
        "blair",
        vec![step(
            Operation::Get {
                path: "readme".into(),
            },
            Expected::Download {
                text: "hello Blair\n".into(),
            },
        )],
    ));
    for (id, credential) in [
        (
            "unknown",
            Credential::Valid {
                user: "unknown".into(),
            },
        ),
        (
            "wrong-password",
            Credential::Invalid {
                user: "alex".into(),
            },
        ),
        ("anonymous", Credential::Anonymous),
    ] {
        cases.push(Case {
            id: id.into(),
            label: id.into(),
            credential,
            denied_login: Some(Denial::Authentication),
            confined: false,
            inputs: vec![],
            steps: vec![],
        });
    }
    Task {
        id: "identity-transfer-task".into(),
        initial: fixture::world(),
        editable_files: vec![
            "/etc/vsftpd.chroot_list".into(),
            "/etc/vsftpd.user_list".into(),
        ],
        cases,
    }
}
