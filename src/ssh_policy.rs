//! Deliberately bounded sshd_config policy semantics, not a daemon or full parser.
//! OpenBSD sshd_config(5), checked2026-11-13; no source code imported.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootLogin {
    Yes,
    ProhibitPassword,
    No,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    PublicKey,
    Password,
    KeyboardInteractive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Malformed,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Kind,
    pub line: usize,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            f.write_str(&self.message)
        } else {
            write!(f, "Line {}: {}", self.line, self.message)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub pubkey: bool,
    pub password: bool,
    pub keyboard_interactive: bool,
    pub strict_modes: bool,
    pub root: RootLogin,
    pub allow_users: Option<Vec<String>>,
    pub key_only: bool,
    pub authorized_files: Vec<String>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            pubkey: true,
            password: true,
            keyboard_interactive: true,
            strict_modes: true,
            root: RootLogin::ProhibitPassword,
            allow_users: None,
            key_only: false,
            authorized_files: vec![
                ".ssh/authorized_keys".into(),
                ".ssh/authorized_keys2".into(),
            ],
        }
    }
}
pub fn user(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 32
        && name.as_bytes()[0].is_ascii_lowercase()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'))
}
impl Policy {
    pub fn parse(text: &str) -> Result<Self, Diagnostic> {
        let error = |kind, line, message: &str| Diagnostic {
            kind,
            line,
            message: message.into(),
        };
        if text.len() > 16_384 {
            return Err(error(
                Kind::Limit,
                0,
                "Configuration exceeds the 16,384-byte document limit.",
            ));
        }
        if !text.is_ascii()
            || text
                .bytes()
                .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'))
        {
            return Err(error(
                Kind::Unsupported,
                0,
                "This activity supports ASCII configuration text without embedded control characters.",
            ));
        }
        let mut p = Self::default();
        let mut seen = std::collections::BTreeSet::new();
        let mut lines = 0;
        for (n, line) in text.lines().enumerate() {
            let n = n + 1;
            if line.len() > 1024 {
                return Err(error(
                    Kind::Limit,
                    n,
                    "Configuration line exceeds 1,024 bytes.",
                ));
            }
            let comment = line
                .char_indices()
                .find(|(i, c)| {
                    *c == '#' && (*i == 0 || line.as_bytes()[i - 1].is_ascii_whitespace())
                })
                .map(|r| r.0);
            let line = line[..comment.unwrap_or(line.len())].trim();
            if line.is_empty() {
                continue;
            }
            lines += 1;
            if lines > 128 {
                return Err(error(
                    Kind::Limit,
                    n,
                    "At most 128 configuration directives are supported.",
                ));
            }
            // This subset deliberately rejects quoting/continuations/Include/Match rather
            // than guessing their precedence or evaluating paths.
            if line.contains(['"', '\'', '\\']) {
                return Err(error(
                    Kind::Unsupported,
                    n,
                    "Quoted arguments and continuations are outside this activity's sshd_config subset.",
                ));
            }
            let words: Vec<_> = line.split_whitespace().collect();
            let key = words[0].to_ascii_lowercase();
            let supported = matches!(
                key.as_str(),
                "pubkeyauthentication"
                    | "passwordauthentication"
                    | "kbdinteractiveauthentication"
                    | "strictmodes"
                    | "permitrootlogin"
                    | "allowusers"
                    | "authenticationmethods"
                    | "authorizedkeysfile"
            );
            if !supported {
                return Err(error(
                    Kind::Unsupported,
                    n,
                    "Unsupported directive or syntax. This activity supports PubkeyAuthentication, PasswordAuthentication, KbdInteractiveAuthentication, StrictModes, PermitRootLogin, literal AllowUsers, AuthenticationMethods any/publickey, and the two standard relative AuthorizedKeysFile paths.",
                ));
            }
            if words.len() < 2 {
                return Err(error(
                    Kind::Malformed,
                    n,
                    "This directive needs an argument.",
                ));
            }
            if key == "allowusers" {
                if words[1..].iter().any(|v| !user(v)) {
                    return Err(error(
                        Kind::Unsupported,
                        n,
                        "AllowUsers supports literal lower-case user names only; patterns, groups and USER@HOST are not modeled.",
                    ));
                }
                let names = p.allow_users.get_or_insert_with(Vec::new);
                if names.len() + words.len() - 1 > 32 {
                    return Err(error(
                        Kind::Limit,
                        n,
                        "AllowUsers exceeds 32 literal names.",
                    ));
                }
                names.extend(words[1..].iter().map(|v| (*v).to_owned()));
                continue;
            }
            if key == "authorizedkeysfile" {
                if words.len() > 5 {
                    return Err(error(
                        Kind::Limit,
                        n,
                        "At most four authorized-key file arguments are supported.",
                    ));
                }
                let value = if words[1..] == ["none"] {
                    vec![]
                } else {
                    if words[1..]
                        .iter()
                        .any(|v| !matches!(*v, ".ssh/authorized_keys" | ".ssh/authorized_keys2"))
                    {
                        return Err(error(
                            Kind::Unsupported,
                            n,
                            "Only the two standard relative .ssh/authorized_keys paths or none are supported; no tokens, absolute paths or filesystem access occur.",
                        ));
                    }
                    words[1..].iter().map(|s| (*s).to_owned()).collect()
                };
                if seen.insert(key) {
                    p.authorized_files = value;
                }
                continue;
            }
            if words.len() != 2 {
                return Err(error(
                    Kind::Malformed,
                    n,
                    "Use one argument for this supported directive.",
                ));
            }
            let value = words[1];
            match key.as_str() {
                "permitrootlogin" => {
                    let mode = match value {
                        "yes" => RootLogin::Yes,
                        "no" => RootLogin::No,
                        "prohibit-password" | "without-password" => RootLogin::ProhibitPassword,
                        "forced-commands-only" => {
                            return Err(error(
                                Kind::Unsupported,
                                n,
                                "Forced-command key options are not modeled; this is not an invalid OpenSSH value.",
                            ));
                        }
                        _ => {
                            return Err(error(
                                Kind::Malformed,
                                n,
                                "PermitRootLogin expects yes, no or prohibit-password in this activity.",
                            ));
                        }
                    };
                    if seen.insert(key) {
                        p.root = mode;
                    }
                }
                "authenticationmethods" => {
                    let only = match value {
                        "any" => false,
                        "publickey" => true,
                        _ => {
                            return Err(error(
                                Kind::Unsupported,
                                n,
                                "Only AuthenticationMethods any or publickey is modeled; multi-method authentication sequences are not approximated.",
                            ));
                        }
                    };
                    if seen.insert(key) {
                        p.key_only = only;
                    }
                }
                _ => {
                    let value = match value {
                        "yes" => true,
                        "no" => false,
                        _ => {
                            return Err(error(
                                Kind::Malformed,
                                n,
                                "Use the lower-case argument yes or no; directive names themselves are case-insensitive.",
                            ));
                        }
                    };
                    if seen.insert(key.clone()) {
                        match key.as_str() {
                            "pubkeyauthentication" => p.pubkey = value,
                            "passwordauthentication" => p.password = value,
                            "kbdinteractiveauthentication" => p.keyboard_interactive = value,
                            "strictmodes" => p.strict_modes = value,
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        Ok(p)
    }
    /// Whether this single method is offered to a named user under the supported
    /// policy. This is NOT credential verification or evidence of a completed login.
    pub fn offers(&self, name: &str, method: Method) -> Result<bool, String> {
        if !user(name) {
            return Err("Unsupported user spelling: choose a supplied literal user name.".into());
        }
        if self
            .allow_users
            .as_ref()
            .is_some_and(|users| !users.iter().any(|u| u == name))
        {
            return Ok(false);
        }
        if name == "root"
            && (self.root == RootLogin::No
                || self.root == RootLogin::ProhibitPassword && method != Method::PublicKey)
        {
            return Ok(false);
        }
        if self.key_only && method != Method::PublicKey {
            return Ok(false);
        }
        Ok(match method {
            Method::PublicKey => self.pubkey,
            Method::Password => self.password,
            Method::KeyboardInteractive => self.keyboard_interactive,
        })
    }
}
