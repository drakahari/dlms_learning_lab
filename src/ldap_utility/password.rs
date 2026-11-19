//! Literal ldappasswd subset over protected fictional credential state.
//! Prompt data is supplied separately, never shell arguments or host input.
use super::{client, once, syntax, unsupported, write_words};
use crate::{
    ldap_identity::{self, protected},
    ldap_protection as protection,
    ldap_query::{Dn, Error},
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordInvocation {
    uri: String,
    bind: Dn,
    target: Option<Dn>,
    request: protection::Request,
    ask_old: bool,
}
impl PasswordInvocation {
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn bind(&self) -> &Dn {
        &self.bind
    }
    pub fn target(&self) -> Option<&Dn> {
        self.target.as_ref()
    }
    pub fn protection_request(&self) -> protection::Request {
        self.request
    }
    pub fn asks_old_password(&self) -> bool {
        self.ask_old
    }
}
/// Only this API recognizes ldappasswd. Existing LDIF/delete parsers stay closed.
pub fn parse_password(raw: &str) -> Result<PasswordInvocation, Error> {
    let args = write_words(raw)?;
    if args.first().map(String::as_str) != Some("ldappasswd") {
        return Err(unsupported(
            "This interface supports only the declared ldappasswd subset.",
        ));
    }
    let (mut simple, mut bind_prompt, mut new_prompt, mut old_prompt, mut operands) =
        (false, false, false, false, false);
    let (mut uri, mut bind, mut target, mut request) = (None, None, None, None);
    let mut at = 1;
    while at < args.len() {
        match args[at].as_str() {
            "-x" if !operands => {
                once(&mut simple)?;
                at += 1;
            }
            "-W" if !operands => {
                once(&mut bind_prompt)?;
                at += 1;
            }
            "-S" if !operands => {
                once(&mut new_prompt)?;
                at += 1;
            }
            "-A" if !operands => {
                once(&mut old_prompt)?;
                at += 1;
            }
            flag @ ("-Z" | "-ZZ") if !operands => {
                if request.is_some() {
                    return Err(syntax("Choose at most one -Z or -ZZ option."));
                }
                request = Some(if flag == "-ZZ" {
                    protection::Request::RequireStartTls
                } else {
                    protection::Request::TryStartTls
                });
                at += 1;
            }
            flag @ ("-H" | "-D") if !operands => {
                let value = args
                    .get(at + 1)
                    .ok_or_else(|| syntax("This option needs a following value."))?;
                if flag == "-H" {
                    if uri.is_some() {
                        return Err(syntax("Write -H only once."));
                    }
                    uri = Some(client::uri_key(value)?);
                } else {
                    if bind.is_some() {
                        return Err(syntax("Write -D only once."));
                    }
                    bind = Some(Dn::parse(value)?);
                }
                at += 2;
            }
            "--" if !operands => {
                operands = true;
                at += 1;
            }
            value if value.starts_with('-') => {
                return Err(unsupported(
                    "Options must precede the optional self DN. Only -x -W -S -A -H -D -Z/-ZZ are supported; password arguments/files, SASL, controls, verbose/dry-run and other options are unsupported.",
                ));
            }
            value => {
                if target.is_some() {
                    return Err(unsupported(
                        "This subset accepts at most one optional target DN.",
                    ));
                }
                target = Some(Dn::parse(value)?);
                operands = true;
                at += 1;
            }
        }
    }
    if !simple || !bind_prompt {
        return Err(unsupported(
            "Use explicit -x and -W with the supplied fictional bind profile; anonymous/SASL/password arguments are unsupported.",
        ));
    }
    if !new_prompt {
        return Err(unsupported(
            "Use -S with a supplied replacement profile. Server-generated passwords are outside this subset.",
        ));
    }
    Ok(PasswordInvocation {
        uri: uri.ok_or_else(|| unsupported("Supply -H for the provided virtual directory."))?,
        bind: bind
            .ok_or_else(|| unsupported("Supply -D for the provided authentication identity."))?,
        target,
        request: request.unwrap_or(protection::Request::Plain),
        ask_old: old_prompt,
    })
}
/// Fictional source-held prompt values. Debug never exposes their contents.
pub struct PasswordPrompts<'a> {
    pub bind: &'a str,
    pub old_target: Option<&'a str>,
    pub replacement: Option<&'a str>,
}
impl fmt::Debug for PasswordPrompts<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PasswordPrompts")
            .field("bind", &"[redacted]")
            .field("old_target", &self.old_target.map(|_| "[redacted]"))
            .field("replacement", &self.replacement.map(|_| "[redacted]"))
            .finish()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingPrompt {
    OldTarget,
    Replacement,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasswordFailure {
    Invocation(Error),
    MissingPrompt(MissingPrompt),
    UnsupportedTransport(protection::PlainReason),
    Operation(protected::Error),
}
impl fmt::Display for PasswordFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invocation(e) => write!(f, "{e}"),
            Self::MissingPrompt(MissingPrompt::OldTarget) => f.write_str(
                "-A needs the separately supplied old-target sample password. Nothing changed.",
            ),
            Self::MissingPrompt(MissingPrompt::Replacement) => f.write_str(
                "-S needs the separately supplied replacement sample password. Nothing changed.",
            ),
            Self::UnsupportedTransport(protection::PlainReason::NotRequested) => f.write_str(
                "This command does not request TLS. Plaintext authentication is outside this protected password subset; use -ZZ. No authentication or password change was simulated.",
            ),
            Self::UnsupportedTransport(protection::PlainReason::StartTlsUnavailable) => f.write_str(
                "Optional -Z would fall back to plaintext because this sample cannot start TLS. That real-client fallback is outside this protected password subset; use -ZZ to require TLS. No authentication or password change was simulated.",
            ),
            Self::Operation(e) => write!(f, "{e}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordReport {
    pub failure: Option<PasswordFailure>,
}
impl PasswordReport {
    pub fn succeeded(&self) -> bool {
        self.failure.is_none()
    }
    pub fn exit_status(&self) -> u8 {
        match &self.failure {
            None => 0,
            Some(PasswordFailure::Operation(protected::Error::Identity(
                ldap_identity::Error::InvalidCredentials,
            ))) => 49,
            Some(_) => 1,
        }
    }
}
/// Performs one bounded simulated invocation. No real prompt, TLS, command,
/// process, file, service or network operation occurs.
pub fn run_password(
    raw: &str,
    prompts: &PasswordPrompts<'_>,
    trust: &protection::Trust,
    sample_time: u64,
    realm: &mut protected::ProtectedRealm,
) -> PasswordReport {
    let mut run = || -> Result<(), PasswordFailure> {
        let command = parse_password(raw).map_err(PasswordFailure::Invocation)?;
        let old = if command.ask_old {
            Some(
                prompts
                    .old_target
                    .ok_or(PasswordFailure::MissingPrompt(MissingPrompt::OldTarget))?,
            )
        } else {
            None
        };
        let new = prompts
            .replacement
            .ok_or(PasswordFailure::MissingPrompt(MissingPrompt::Replacement))?;
        let connection = realm
            .open(&command.uri, command.request, trust, sample_time)
            .map_err(PasswordFailure::Operation)?;
        if let protection::Protection::Plain(reason) = connection.protection() {
            return Err(PasswordFailure::UnsupportedTransport(reason));
        }
        let authenticated = realm
            .authenticate(&connection, &command.bind, prompts.bind)
            .map_err(PasswordFailure::Operation)?;
        realm
            .change_password(&authenticated, command.target.as_ref(), old, Some(new))
            .map_err(PasswordFailure::Operation)
    };
    PasswordReport {
        failure: run().err(),
    }
}
