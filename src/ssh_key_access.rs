//! Plain authorized-key/path checks using the existing virtual Unix tree.
//! Nonroot supplied users only; not credential/signature verification or a session.
use crate::{
    filesystem::{Access, Tree},
    ssh_identity::{AuthorizedKeys, PublicKey},
    ssh_policy,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    Missing(String),
    Unreadable(String),
    UnsafeOwner(String),
    UnsafeMode(String),
    NoMatchingKey(String),
}
impl Finding {
    pub fn explanation(&self) -> String {
        match self {
            Self::Missing(p) => {
                format!("{p}: no ordinary authorized-key file exists at this supplied location.")
            }
            Self::Unreadable(p) => format!(
                "{p}: the target user cannot read the key file through its parent directories. Check read and directory search permissions."
            ),
            Self::UnsafeOwner(p) => {
                format!("{p}: StrictModes requires ownership by the target user or root.")
            }
            Self::UnsafeMode(p) => format!(
                "{p}: StrictModes rejects group- or other-writable key paths. Other modes can also be valid when they meet the required access."
            ),
            Self::NoMatchingKey(p) => format!(
                "{p}: none of its plain public keys matches the supplied client identity. Comments do not authorize a key."
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authorization {
    Allowed { path: String },
    Denied { findings: Vec<Finding> },
}
/// The caller must independently check server trust, method policy, service state
/// and possession of the source-supplied client identity. `Allowed` means only
/// that this supported key file/path slice permits that supplied public identity.
pub fn check(
    tree: &Tree,
    user: &str,
    files: &[String],
    key: PublicKey,
    strict: bool,
) -> Result<Authorization, String> {
    if !tree.valid() {
        return Err("Invalid bounded virtual file tree; no key authorization was inferred.".into());
    }
    if user == "root" || !ssh_policy::user(user) || !tree.users.contains_key(user) {
        return Err("Unsupported target for this path checker: use a supplied nonroot account. Root login and other authentication stages must not be inferred here.".into());
    }
    if files.len() > 4
        || files
            .iter()
            .any(|p| !matches!(p.as_str(), ".ssh/authorized_keys" | ".ssh/authorized_keys2"))
    {
        return Err("Unsupported authorized-key path list; only the two standard relative files are modeled.".into());
    }
    let home = format!("/home/{user}");
    let mut findings = vec![];
    let mut allowed = None;
    for relative in files {
        let path = format!("{home}/{relative}");
        let Some(node) = tree.nodes.get(&path).filter(|n| !n.directory) else {
            findings.push(Finding::Missing(path));
            continue;
        };
        if !tree.access(user, &path, Access::ReadFile) {
            findings.push(Finding::Unreadable(path));
            continue;
        }
        if strict {
            let mut bad = None;
            for p in [
                path.as_str(),
                format!("{home}/.ssh").as_str(),
                home.as_str(),
            ] {
                let n = tree
                    .nodes
                    .get(p)
                    .ok_or("Virtual path is incomplete; no authorization was inferred.")?;
                if n.owner != user && n.owner != "root" {
                    bad = Some(Finding::UnsafeOwner(p.into()));
                    break;
                }
                if n.mode & 0o022 != 0 {
                    bad = Some(Finding::UnsafeMode(p.into()));
                    break;
                }
            }
            if let Some(bad) = bad {
                findings.push(bad);
                continue;
            }
        }
        let _ = node; // Directory/type/readability were checked before parsing its bytes.
        let text = tree
            .files
            .get(&path)
            .ok_or("Virtual key file lacks contents.")?;
        let parsed=AuthorizedKeys::parse(text).map_err(|e|format!("{path}: {e} This activity cannot evaluate this key file; use the documented plain public-key format."))?;
        if parsed.contains(key) {
            allowed.get_or_insert(path);
        } else {
            findings.push(Finding::NoMatchingKey(path));
        }
    }
    Ok(allowed.map_or(Authorization::Denied { findings }, |path| {
        Authorization::Allowed { path }
    }))
}
