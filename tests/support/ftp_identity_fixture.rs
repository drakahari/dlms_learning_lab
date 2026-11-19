use dlms_learning_lab::{
    engine::Node,
    filesystem::Tree,
    ftp_identity::{Account, World},
};
use std::collections::BTreeMap;
pub fn settings() -> String {
    "anonymous_enable=NO\nlocal_enable=YES\nwrite_enable=YES\nchroot_local_user=YES\nlocal_umask=022\n".into()
}
fn node(directory: bool, mode: u16, owner: &str) -> Node {
    Node {
        directory,
        mode,
        owner: owner.into(),
        group: owner.into(),
    }
}
pub fn world() -> World {
    let mut nodes = BTreeMap::new();
    let mut files = BTreeMap::new();
    for (p, mode, owner) in [
        ("/", 0o755, "admin"),
        ("/srv", 0o755, "admin"),
        ("/srv/alex", 0o755, "admin"),
        ("/srv/alex/inbox", 0o700, "alex"),
        ("/srv/blair", 0o755, "admin"),
        ("/srv/blair/inbox", 0o700, "blair"),
        ("/public", 0o755, "admin"),
        ("/etc", 0o755, "admin"),
    ] {
        nodes.insert(p.into(), node(true, mode, owner));
    }
    for (p, text, mode, owner) in [
        ("/srv/alex/readme", "hello Alex\n", 0o644, "admin"),
        ("/srv/alex/inbox/own", "old\n", 0o600, "alex"),
        ("/srv/alex/private", "private\n", 0o600, "admin"),
        ("/srv/blair/readme", "hello Blair\n", 0o644, "admin"),
        ("/public/readme", "public\n", 0o644, "admin"),
        ("/public/not-world-readable", "restricted\n", 0o640, "ftp"),
        ("/etc/secret", "outside\n", 0o644, "admin"),
        ("/etc/vsftpd.chroot_list", "", 0o600, "admin"),
        ("/etc/vsftpd.user_list", "alex\n", 0o600, "admin"),
    ] {
        nodes.insert(p.into(), node(false, mode, owner));
        files.insert(p.into(), text.into());
    }
    let tree = Tree {
        nodes,
        files,
        users: ["admin", "alex", "blair", "ftp"]
            .into_iter()
            .map(|u| (u.into(), vec![u.into()]))
            .collect(),
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
        version: 1,
        server: "fixture-ftp".into(),
        files: tree,
        accounts: vec![
            Account {
                user: "alex".into(),
                home: "/srv/alex".into(),
                enabled: true,
            },
            Account {
                user: "blair".into(),
                home: "/srv/blair".into(),
                enabled: true,
            },
        ],
        anonymous_home: "/public".into(),
        saved: settings(),
        published: Some(settings()),
        generation: 1,
    }
}
