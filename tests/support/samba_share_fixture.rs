#![allow(dead_code)]
use dlms_learning_lab::{
    engine::Node,
    filesystem::Tree,
    samba_share::{Account, Credential, World},
};
use std::collections::BTreeMap;
pub fn config() -> String {
    "[global]\nsecurity = user\nserver role = standalone server\nmap to guest = Never\n[team]\npath = /srv/team\nvalid users = @team\nread only = yes\nread list = @reviewers\nwrite list = @writers\n[public]\npath = /public\nguest ok = yes\nread only = yes\n".into()
}
fn node(directory: bool, mode: u16, group: &str) -> Node {
    Node {
        directory,
        mode,
        owner: "admin".into(),
        group: group.into(),
    }
}
pub fn world() -> World {
    let files = Tree {
        nodes: BTreeMap::from([
            ("/".into(), node(true, 0o755, "staff")),
            ("/srv".into(), node(true, 0o755, "staff")),
            ("/etc".into(), node(true, 0o755, "staff")),
            ("/etc/samba".into(), node(true, 0o755, "staff")),
            ("/etc/samba/smb.conf".into(), node(false, 0o600, "staff")),
            ("/srv/team".into(), node(true, 0o770, "team")),
            ("/srv/team/draft.txt".into(), node(false, 0o660, "team")),
            ("/srv/team/private.txt".into(), node(false, 0o600, "staff")),
            ("/public".into(), node(true, 0o755, "staff")),
            ("/public/notice.txt".into(), node(false, 0o644, "staff")),
        ]),
        files: BTreeMap::from([
            ("/etc/samba/smb.conf".into(), config()),
            ("/srv/team/draft.txt".into(), "draft\n".into()),
            ("/srv/team/private.txt".into(), "private fixture\n".into()),
            ("/public/notice.txt".into(), "notice\n".into()),
        ]),
        users: BTreeMap::from([
            ("admin".into(), vec!["staff".into(), "team".into()]),
            ("alex".into(), vec!["team".into(), "writers".into()]),
            ("blair".into(), vec!["team".into(), "reviewers".into()]),
            ("casey".into(), vec!["visitors".into()]),
            ("guest".into(), vec!["visitors".into()]),
        ]),
        user: "admin".into(),
        home: "/".into(),
        cwd: "/".into(),
        previous: None,
        umask: 0o022,
        clock: 0,
        modified: BTreeMap::new(),
        command_location: None,
    };
    World {
        schema_version: 1,
        server: "team-server".into(),
        files,
        accounts: vec![
            Account {
                user: "alex".into(),
                enabled: true,
            },
            Account {
                user: "blair".into(),
                enabled: true,
            },
        ],
        guest: "guest".into(),
        configuration_path: "/etc/samba/smb.conf".into(),
        published: config(),
        generation: 1,
    }
}
pub fn valid(user: &str) -> Credential {
    Credential::Valid { user: user.into() }
}
