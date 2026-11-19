//! Finite vsftpd local-identity configuration. No daemon or host configuration.
//! See docs/FTP_IDENTITY_SUBSET.md for the source revision and exclusions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Kind,
    pub line: usize,
    pub detail: String,
}
pub const MAX_CONFIG: usize = 16 * 1024;
pub const MAX_LINES: usize = 128;
pub fn canonical(path: &str) -> bool {
    path == "/"
        || (path.starts_with('/')
            && path.len() <= 512
            && !path.ends_with('/')
            && path[1..].split('/').all(|p| {
                !p.is_empty()
                    && p != "."
                    && p != ".."
                    && p.bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            }))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub local_enable: bool,
    pub anonymous_enable: bool,
    pub write_enable: bool,
    pub chroot_local_user: bool,
    pub chroot_list_enable: bool,
    pub chroot_list_file: String,
    pub userlist_enable: bool,
    pub userlist_deny: bool,
    pub userlist_file: String,
    pub download_enable: bool,
    pub dirlist_enable: bool,
    pub local_umask: u16,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            local_enable: false,
            anonymous_enable: true,
            write_enable: false,
            chroot_local_user: false,
            chroot_list_enable: false,
            chroot_list_file: "/etc/vsftpd.chroot_list".into(),
            userlist_enable: false,
            userlist_deny: true,
            userlist_file: "/etc/vsftpd.user_list".into(),
            download_enable: true,
            dirlist_enable: true,
            local_umask: 0o077,
        }
    }
}
fn err(kind: Kind, line: usize, detail: impl Into<String>) -> Diagnostic {
    Diagnostic {
        kind,
        line,
        detail: detail.into(),
    }
}
fn boolean(value: &str, line: usize) -> Result<bool, Diagnostic> {
    match value.to_ascii_uppercase().as_str() {
        "YES" | "TRUE" | "1" => Ok(true),
        "NO" | "FALSE" | "0" => Ok(false),
        _ => Err(err(Kind::Syntax, line, "Use YES or NO for this setting.")),
    }
}
pub fn parse(text: &str) -> Result<Configuration, Diagnostic> {
    if text.len() > MAX_CONFIG || text.lines().count() > MAX_LINES {
        return Err(err(
            Kind::Limit,
            0,
            "Configuration exceeds 16 KiB or 128 lines.",
        ));
    }
    if text
        .bytes()
        .any(|c| !c.is_ascii() || (c.is_ascii_control() && !b"\n\r\t".contains(&c)))
    {
        return Err(err(
            Kind::Unsupported,
            0,
            "Use plain ASCII configuration text without control characters.",
        ));
    }
    let mut c = Configuration::default();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        // The pinned parser trims settings/values, but comments start in column1.
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let (key, value) = raw.split_once('=').ok_or_else(|| {
            err(
                Kind::Syntax,
                line,
                "Use option=value; put comments on a separate line beginning with #.",
            )
        })?;
        let (key, value) = (key.trim(), value.trim());
        match key {
            "local_enable" => c.local_enable = boolean(value, line)?,
            "anonymous_enable" => c.anonymous_enable = boolean(value, line)?,
            "write_enable" => c.write_enable = boolean(value, line)?,
            "chroot_local_user" => c.chroot_local_user = boolean(value, line)?,
            "chroot_list_enable" => c.chroot_list_enable = boolean(value, line)?,
            "userlist_enable" => c.userlist_enable = boolean(value, line)?,
            "userlist_deny" => c.userlist_deny = boolean(value, line)?,
            "download_enable" => c.download_enable = boolean(value, line)?,
            "dirlist_enable" => c.dirlist_enable = boolean(value, line)?,
            "chroot_list_file" | "userlist_file" => {
                if !canonical(value) { return Err(err(Kind::Unsupported, line, "Use a canonical absolute virtual path; substitutions and empty paths are outside this subset.")); }
                if key == "chroot_list_file" { c.chroot_list_file = value.into(); }
                else { c.userlist_file = value.into(); }
            }
            "local_umask" => {
                // Real parseconf treats an initial0 as octal and other values as decimal.
                if value.is_empty() || value.len() > 4 || !value.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(err(Kind::Syntax, line, "Use a permission mask such as 022 (octal)."));
                }
                let radix = if value.starts_with('0') { 8 } else { 10 };
                c.local_umask = u16::from_str_radix(value, radix).ok().filter(|n| *n <= 0o777)
                    .ok_or_else(|| err(Kind::Unsupported, line, "Only ordinary permission masks 000–0777 are supported."))?;
            }
            _ => return Err(err(Kind::Unsupported, line, format!("{key} is outside this simulator's disclosed FTP subset; it has not been applied."))),
        }
    }
    Ok(c)
}
