//! Bounded literal invocations and virtual-only execution; no OS or network input.
//! A capability alone is not a delivered lesson or utility objective coverage.
use crate::{
    ldap_client as client,
    ldap_query::{Dn, Error},
};
const MAX_WORDS: usize = 32;
mod deletion;
mod execution;
pub use deletion::{DeleteInput, DeleteInvocation, parse_delete, run_delete};
pub use execution::{Failure, Inputs, MAX_FILES, MAX_INPUT_BYTES, Report, run};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Editor,
    VirtualFile(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invocation {
    add_default: bool,
    uri: String,
    bind: Dn,
    input: Input,
}
impl Invocation {
    pub fn add_default(&self) -> bool {
        self.add_default
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn bind(&self) -> &Dn {
        &self.bind
    }
    pub fn input(&self) -> &Input {
        &self.input
    }
}
fn syntax(s: &str) -> Error {
    Error::Syntax(s.into())
}
fn unsupported(s: &str) -> Error {
    Error::Unsupported(s.into())
}
fn once(seen: &mut bool) -> Result<(), Error> {
    if *seen {
        return Err(syntax("Write each option only once."));
    }
    *seen = true;
    Ok(())
}
fn virtual_name(s: &str) -> Result<String, Error> {
    if s.len() > 128 {
        return Err(Error::Limit(
            "Virtual input identifier exceeds128bytes.".into(),
        ));
    }
    if !s.starts_with("/work/")
        || s.split('/').skip(1).any(|c| {
            c.is_empty()
                || c == "."
                || c == ".."
                || c.len() > 64
                || !c
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
    {
        return Err(unsupported(
            "Select a supplied /work/ virtual input identifier. This parser never opens host files.",
        ));
    }
    Ok(s.into())
}
fn write_words(raw: &str) -> Result<Vec<String>, Error> {
    if raw.len() > client::MAX_COMMAND {
        return Err(Error::Limit("Directory command exceeds4096bytes.".into()));
    }
    let args = client::words(raw).map_err(|e| match e {
        // Preserve the shared read-only lexer's existing diagnostics for search,
        // while naming this parser's context correctly for future write tasks.
        Error::Unsupported(message) => Error::Unsupported(
            message
                .replace("literal search-command", "literal directory-command")
                .replace("ldapsearch", "directory utility"),
        ),
        Error::Limit(message) => {
            Error::Limit(message.replace("64 command arguments", "32 command words"))
        }
        other => other,
    })?;
    if args.len() > MAX_WORDS {
        return Err(Error::Limit(
            "At most32literal command words are supported.".into(),
        ));
    }
    Ok(args)
}
pub fn parse(raw: &str) -> Result<Invocation, Error> {
    let args = write_words(raw)?;
    let mut add_default = match args.first().map(String::as_str) {
        Some("ldapadd") => true,
        Some("ldapmodify") => false,
        None => return Err(syntax("Enter a supported directory utility command.")),
        _ => {
            return Err(unsupported(
                "Supported command names are ldapadd and ldapmodify. This input was not run.",
            ));
        }
    };
    let (mut simple, mut prompt, mut seen_add) = (false, false, false);
    let (mut uri, mut bind, mut file) = (None, None, None);
    let mut at = 1;
    while at < args.len() {
        match args[at].as_str() {
            "-x" => {
                once(&mut simple)?;
                at += 1;
            }
            "-W" => {
                once(&mut prompt)?;
                at += 1;
            }
            "-a" => {
                once(&mut seen_add)?;
                add_default = true;
                at += 1;
            }
            flag @ ("-H" | "-D" | "-f") => {
                let value = args
                    .get(at + 1)
                    .ok_or_else(|| syntax("This option needs a following value."))?;
                match flag {
                    "-H" => {
                        if uri.is_some() {
                            return Err(syntax("Write -H only once."));
                        }
                        uri = Some(client::uri_key(value)?);
                    }
                    "-D" => {
                        if bind.is_some() {
                            return Err(syntax("Write -D only once."));
                        }
                        bind = Some(Dn::parse(value)?);
                    }
                    _ => {
                        if file.is_some() {
                            return Err(syntax("Write -f only once."));
                        }
                        file = Some(virtual_name(value)?);
                    }
                }
                at += 2;
            }
            "--" if at + 1 == args.len() => {
                at += 1;
            }
            _ => {
                return Err(unsupported(
                    "Supported separate options are -x -W -H URI -D DN and optional -a/-f VIRTUAL_INPUT. Other options, positional operands and shell syntax are not implemented.",
                ));
            }
        }
    }
    if !simple || !prompt {
        return Err(unsupported(
            "This subset requires -x and -W; SASL and password command/file arguments are not implemented.",
        ));
    }
    Ok(Invocation{add_default,uri:uri.ok_or_else(||unsupported("Supply -H for a provided virtual directory; host/default discovery is not supported."))?,bind:bind.ok_or_else(||unsupported("Supply -D for a provided training identity. Naming it does not authenticate it; anonymous writes are not supported."))?,input:file.map_or(Input::Editor,Input::VirtualFile)})
}

mod password;
pub use password::{MissingPrompt, PasswordFailure, PasswordInvocation, PasswordPrompts, PasswordReport, parse_password, run_password};
