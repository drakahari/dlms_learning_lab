//! In-process FTP identity/root/file policy. No protocol socket, daemon or host I/O.
use crate::{
    engine::Node,
    filesystem::{Access, Tree},
    ftp_identity_config as config,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_DATA: usize = 32 * 1024;
pub const MAX_TRANSFER: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Invalid,
    Syntax,
    Unsupported,
    Limit,
    Service,
    Authentication,
    Policy,
    Permission,
    Missing,
    Stale,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: Kind,
    pub detail: String,
}
fn fault(kind: Kind, detail: impl Into<String>) -> Fault {
    Fault {
        kind,
        detail: detail.into(),
    }
}
fn from_config(e: config::Diagnostic) -> Fault {
    fault(
        match e.kind {
            config::Kind::Syntax => Kind::Syntax,
            config::Kind::Unsupported => Kind::Unsupported,
            config::Kind::Limit => Kind::Limit,
        },
        format!("Line {}: {}", e.line, e.detail),
    )
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        && s != "root"
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub user: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub home: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub server: String,
    pub files: Tree,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Account,16>")]
    pub accounts: Vec<Account>,
    /// Anonymous login uses the supplied ftp account, never a real host account.
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub anonymous_home: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub saved: String,
    #[serde(deserialize_with = "crate::apache_static_workspace::option_text::<_,16384>")]
    pub published: Option<String>,
    pub generation: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Credential {
    Valid {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        user: String,
    },
    Invalid {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        user: String,
    },
    Anonymous,
}
/// Sessions cannot be deserialized or manufactured from learner input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    server: String,
    generation: u32,
    user: String,
    root: String,
    cwd: String,
    anonymous: bool,
    policy: config::Configuration,
}
impl Session {
    pub fn user(&self) -> &str {
        &self.user
    }
    pub fn pwd(&self) -> &str {
        &self.cwd
    }
    pub fn confined(&self) -> bool {
        self.root != "/"
    }
}
fn bounded(text: &str) -> bool {
    text.len() <= config::MAX_CONFIG && !text.contains('\0')
}
impl World {
    pub fn validate(&self) -> Result<(), Fault> {
        if self.version != 1
            || !name(&self.server)
            || !(1..=1_000_000).contains(&self.generation)
            || !self.files.valid()
            || !self.files.nodes.get("/etc").is_some_and(|n| n.directory)
            || self.files.nodes.contains_key("/etc/vsftpd.conf")
            || self.files.files.values().map(String::len).sum::<usize>() > MAX_DATA
            || !bounded(&self.saved)
            || self.accounts.len() > 16
            || !config::canonical(&self.anonymous_home)
            || !self
                .files
                .nodes
                .get(&self.anonymous_home)
                .is_some_and(|n| n.directory)
            || !self.files.users.contains_key("ftp")
        {
            return Err(fault(Kind::Invalid,"Use a bounded version1 virtual FTP server, ordinary supplied identities, home directories and at most32KiB of file data."));
        }
        let mut names = BTreeSet::new();
        if self.accounts.iter().any(|a| {
            !name(&a.user)
                || a.user == "ftp"
                || !names.insert(&a.user)
                || !self.files.users.contains_key(&a.user)
                || !config::canonical(&a.home)
                || !self.files.nodes.get(&a.home).is_some_and(|n| n.directory)
        }) {
            return Err(fault(Kind::Invalid,"Each supplied local account needs a distinct non-root Unix identity and an existing canonical home directory."));
        }
        if let Some(text) = &self.published {
            config::parse(text).map_err(from_config)?;
        }
        Ok(())
    }
    pub fn save(&mut self, text: String) -> Result<(), Fault> {
        if !bounded(&text) {
            return Err(fault(
                Kind::Limit,
                "The saved configuration is limited to16KiB without NUL characters.",
            ));
        }
        self.saved = text;
        Ok(())
    }
    /// A failed restart stops the old service; it never claims the old daemon survived.
    pub fn restart(&mut self) -> Result<(), Fault> {
        self.validate()?;
        if self.generation == 1_000_000 {
            return Err(fault(
                Kind::Limit,
                "Restart limit reached; reset this attempt.",
            ));
        }
        self.generation += 1;
        self.published = None;
        config::parse(&self.saved).map_err(from_config)?;
        self.published = Some(self.saved.clone());
        Ok(())
    }
    pub fn stop(&mut self) -> Result<(), Fault> {
        self.validate()?;
        if self.generation == 1_000_000 {
            return Err(fault(
                Kind::Limit,
                "Session generation limit reached; reset this attempt.",
            ));
        }
        self.generation += 1;
        self.published = None;
        Ok(())
    }
    fn listed(&self, path: &str, user: &str) -> Result<bool, Fault> {
        let text = self.files.files.get(path).ok_or_else(|| {
            fault(
                Kind::Policy,
                format!("Cannot read the configured virtual account list {path}."),
            )
        })?;
        // Exact complete lines, not trimmed substrings or group expansion.
        Ok(text.split('\n').any(|line| line == user))
    }
    pub fn login(&self, proof: &Credential) -> Result<Session, Fault> {
        self.validate()?;
        let policy = config::parse(self.published.as_deref().ok_or_else(|| {
            fault(
                Kind::Service,
                "The simulated FTP service is stopped. Check configuration and restart it.",
            )
        })?)
        .map_err(from_config)?;
        let (user,home,anonymous)=match proof {
            Credential::Invalid { .. }=>return Err(fault(Kind::Authentication,"The supplied credentials are invalid; changing file permissions cannot repair authentication.")),
            Credential::Anonymous=> {
                if !policy.anonymous_enable {return Err(fault(Kind::Authentication,"Anonymous login is disabled."));}
                ("ftp".to_string(),self.anonymous_home.clone(),true)
            }
            Credential::Valid { user }=> {
                if !policy.local_enable {return Err(fault(Kind::Authentication,"Local account login is disabled (local_enable)."));}
                let account=self.accounts.iter().find(|a|&a.user==user && a.enabled).ok_or_else(||fault(Kind::Authentication,"No enabled supplied local FTP account matches this identity."))?;
                (user.clone(),account.home.clone(),false)
            }
        };
        let login_name = if anonymous { "anonymous" } else { &user };
        if policy.userlist_enable
            && self.listed(&policy.userlist_file, login_name)? == policy.userlist_deny
        {
            return Err(fault(Kind::Authentication,"The account list denies this login. With userlist_deny=NO, only listed users may log in."));
        }
        let mut jailed = anonymous || policy.chroot_local_user;
        if !anonymous
            && policy.chroot_list_enable
            && self.listed(&policy.chroot_list_file, &user)?
        {
            jailed = !jailed;
        }
        let session = Session {
            server: self.server.clone(),
            generation: self.generation,
            user,
            root: if jailed { home.clone() } else { "/".into() },
            cwd: if jailed { "/".into() } else { home },
            anonymous,
            policy,
        };
        // Before entering a local-user jail, vsftpd changes directory with the
        // target effective IDs. Outside ancestors therefore matter at login;
        // they are excluded only from subsequent in-jail access checks.
        if !anonymous
            && !self.files.access(
                &session.user,
                &home_for_login(&session),
                Access::SearchDirectory,
            )
        {
            return Err(fault(Kind::Permission, "Cannot enter this account's home directory before confinement; check search (x) permission on the home and its outside parent directories."));
        }
        let view = self.view(&session);
        if !view.access(&session.user, &session.cwd, Access::SearchDirectory) {
            return Err(fault(
                Kind::Permission,
                "The login directory needs search (x) permission for this Unix identity.",
            ));
        }
        if jailed && view.access(&session.user, "/", Access::CreateEntry) {
            return Err(fault(Kind::Policy,"The chroot root is writable by this identity. Keep the jail root non-writable and provide a writable child directory for uploads."));
        }
        Ok(session)
    }
    fn current(&self, s: &Session) -> Result<(), Fault> {
        self.validate()?;
        if s.server != self.server || s.generation != self.generation || self.published.is_none() {
            return Err(fault(
                Kind::Stale,
                "This connection ended when the service stopped or restarted. Log in again.",
            ));
        }
        Ok(())
    }
    fn view(&self, s: &Session) -> Tree {
        if s.root == "/" {
            return self.files.clone();
        }
        let map = |p: &str| {
            if p == s.root {
                Some("/".to_string())
            } else {
                p.strip_prefix(&(s.root.clone() + "/"))
                    .map(|v| format!("/{v}"))
            }
        };
        Tree {
            nodes: self
                .files
                .nodes
                .iter()
                .filter_map(|(p, n)| map(p).map(|q| (q, n.clone())))
                .collect(),
            files: self
                .files
                .files
                .iter()
                .filter_map(|(p, t)| map(p).map(|q| (q, t.clone())))
                .collect(),
            users: self.files.users.clone(),
            user: s.user.clone(),
            home: "/".into(),
            cwd: s.cwd.clone(),
            previous: None,
            umask: s.policy.local_umask,
            clock: 0,
            modified: BTreeMap::new(),
            command_location: None,
        }
    }
    fn server_path(s: &Session, p: &str) -> String {
        if s.root == "/" {
            p.into()
        } else if p == "/" {
            s.root.clone()
        } else {
            format!("{}{p}", s.root)
        }
    }
    /// Walk components, checking search before processing '..'; lexical collapse alone
    /// would incorrectly bypass an unreadable or missing intermediate directory.
    fn resolve(
        &self,
        s: &Session,
        path: &str,
        missing_leaf: bool,
    ) -> Result<(Tree, String), Fault> {
        self.current(s)?;
        if path.is_empty()
            || path.len() > 512
            || !path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        {
            return Err(fault(Kind::Unsupported,"Use a virtual path of letters, digits, /, dot, dash or underscore; shell expansion and links are outside this subset."));
        }
        let view = self.view(s);
        let mut parts: Vec<&str> = if path.starts_with('/') {
            vec![]
        } else {
            s.cwd.split('/').filter(|x| !x.is_empty()).collect()
        };
        let steps: Vec<_> = path.split('/').filter(|x| !x.is_empty()).collect();
        if steps.len() > 64 {
            return Err(fault(Kind::Limit, "Path exceeds64 components."));
        }
        for (i, p) in steps.iter().enumerate() {
            let here = format!("/{}", parts.join("/"));
            if !view.access(&s.user, &here, Access::SearchDirectory) {
                return Err(fault(
                    Kind::Permission,
                    format!("Cannot search directory {here}; check directory x permission."),
                ));
            }
            match *p {
                "." => {}
                ".." => {
                    parts.pop();
                }
                _ => parts.push(p),
            }
            let q = format!("/{}", parts.join("/"));
            if !view.nodes.contains_key(&q)
                && !(missing_leaf && i + 1 == steps.len() && !path.ends_with('/'))
            {
                return Err(fault(
                    Kind::Missing,
                    format!("No virtual entry {q} in this connection's root."),
                ));
            }
        }
        let result = format!("/{}", parts.join("/"));
        if path.ends_with('/') && !view.nodes.get(&result).is_some_and(|n| n.directory) {
            return Err(fault(
                Kind::Missing,
                "A trailing slash requires a directory.",
            ));
        }
        Ok((view, result))
    }
    pub fn cd(&self, s: &mut Session, path: &str) -> Result<(), Fault> {
        let (view, p) = self.resolve(s, path, false)?;
        if !view.access(&s.user, &p, Access::SearchDirectory) {
            return Err(fault(
                Kind::Permission,
                "The target must be a searchable directory (x permission).",
            ));
        }
        s.cwd = p;
        Ok(())
    }
    pub fn list(&self, s: &Session, path: &str) -> Result<Vec<String>, Fault> {
        let (view, p) = self.resolve(s, path, false)?;
        if !s.policy.dirlist_enable {
            return Err(fault(
                Kind::Policy,
                "Directory listing is disabled (dirlist_enable).",
            ));
        }
        if !view.access(&s.user, &p, Access::ListNames)
            || !view.access(&s.user, &p, Access::SearchDirectory)
        {
            return Err(fault(
                Kind::Permission,
                "Listing this directory needs read and search (r+x) permissions.",
            ));
        }
        let prefix = if p == "/" {
            "/".into()
        } else {
            format!("{p}/")
        };
        Ok(view
            .nodes
            .keys()
            .filter_map(|q| q.strip_prefix(&prefix))
            .filter(|q| !q.is_empty() && !q.contains('/') && !q.starts_with('.'))
            .map(str::to_string)
            .collect())
    }
    pub fn get(&self, s: &Session, path: &str) -> Result<String, Fault> {
        let (view, p) = self.resolve(s, path, false)?;
        if !s.policy.download_enable {
            return Err(fault(
                Kind::Policy,
                "Downloads are disabled (download_enable).",
            ));
        }
        if !view.access(&s.user, &p, Access::ReadFile) {
            return Err(fault(Kind::Permission,"The file must be readable by this Unix identity, with searchable parent directories."));
        }
        if s.anonymous && view.nodes[&p].mode & 0o004 == 0 {
            return Err(fault(
                Kind::Permission,
                "Anonymous downloads require world-readable files in this subset.",
            ));
        }
        let text = &view.files[&p];
        if text.len() > MAX_TRANSFER {
            return Err(fault(
                Kind::Limit,
                "One simulated transfer is limited to4KiB.",
            ));
        }
        Ok(text.clone())
    }
    pub fn put(&mut self, s: &Session, path: &str, text: &str) -> Result<(), Fault> {
        let (view, p) = self.resolve(s, path, true)?;
        if !s.policy.write_enable || s.anonymous {
            return Err(fault(
                Kind::Policy,
                "Local uploads need write_enable=YES; anonymous uploads are outside this subset.",
            ));
        }
        if text.len() > MAX_TRANSFER || text.contains('\0') {
            return Err(fault(
                Kind::Limit,
                "Upload plain text of at most4KiB without NUL characters.",
            ));
        }
        let server = Self::server_path(s, &p);
        let old = self.files.files.get(&server).map_or(0, String::len);
        if self.files.files.values().map(String::len).sum::<usize>() - old + text.len() > MAX_DATA {
            return Err(fault(Kind::Limit, "Virtual file data exceeds32KiB."));
        }
        if view.nodes.contains_key(&p) {
            if !view.access(&s.user, &p, Access::WriteFile) {
                return Err(fault(Kind::Permission,"Overwriting needs write permission on the existing file; directory write permission alone is insufficient."));
            }
        } else {
            let parent = p
                .rsplit_once('/')
                .map(|(a, _)| if a.is_empty() { "/" } else { a })
                .unwrap_or("/");
            if !view.access(&s.user, parent, Access::CreateEntry) {
                return Err(fault(
                    Kind::Permission,
                    "Creating a file needs write and search (w+x) permission on its directory.",
                ));
            }
            if self.files.nodes.len() >= 256 {
                return Err(fault(
                    Kind::Limit,
                    "The virtual filesystem has reached256entries.",
                ));
            }
            self.files.nodes.insert(
                server.clone(),
                Node {
                    directory: false,
                    mode: 0o666 & !s.policy.local_umask,
                    owner: s.user.clone(),
                    group: self.files.users[&s.user][0].clone(),
                },
            );
        }
        self.files.files.insert(server, text.into());
        Ok(())
    }
}
fn home_for_login(s: &Session) -> String {
    if s.root == "/" {
        s.cwd.clone()
    } else {
        s.root.clone()
    }
}
