//! Minimal, opt-in virtual client settings for the protected password workspace.
//! Never discovers ldap.conf, environment variables, system trust or host files.
use crate::{ldap_client, ldap_utility};
use std::fmt;

pub const MAX_LINES: usize = 64;
pub const MAX_LINE: usize = 1200;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Limit,
    Control { line: usize },
    MissingValue { line: usize },
    Duplicate { line: usize },
    Unsupported { line: usize, reason: &'static str },
    MissingCaInput,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit=>write!(f,"Client settings allow at most 8 KiB, {MAX_LINES} lines and {MAX_LINE} bytes per line."),
            Self::Control{line}=>write!(f,"Line {line}: use ordinary text with LF or CRLF lines, not control characters."),
            Self::MissingValue{line}=>write!(f,"Line {line}: put a setting name and its value on the same line."),
            Self::Duplicate{line}=>write!(f,"Line {line}: this subset supports each setting only once; remove the duplicate."),
            Self::Unsupported{line,reason}=>write!(f,"Line {line}: unsupported in this protected client subset. {reason}"),
            Self::MissingCaInput=>f.write_str("Add TLS_CACERT with a supplied /work/ CA input. This simulator does not discover host/system trust files."),
        }
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    ca_input: String,
}
impl Settings {
    pub fn ca_input(&self) -> &str {
        &self.ca_input
    }
    pub fn parse(raw: &str) -> Result<Self, Error> {
        if raw.len() > ldap_client::MAX_CONFIG
            || raw.lines().count() > MAX_LINES
            || raw.lines().any(|l| l.len() > MAX_LINE)
        {
            return Err(Error::Limit);
        }
        let (mut ca, mut certificate_level) = (None, false);
        for (index, line) in raw.lines().enumerate() {
            let number = index + 1;
            if line.chars().any(|c| c.is_control() && c != '\t') {
                return Err(Error::Control { line: number });
            }
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some(split) = line.find(char::is_whitespace) else {
                return Err(Error::MissingValue { line: number });
            };
            let (name, value) = (&line[..split], line[split..].trim());
            if value.is_empty() {
                return Err(Error::MissingValue { line: number });
            }
            match name.to_ascii_uppercase().as_str() {
                "TLS_CACERT" => {
                    if ca.is_some() {
                        return Err(Error::Duplicate { line: number });
                    }
                    // This validates only an in-memory identifier. It opens nothing.
                    ldap_utility::Inputs::new("",&[(value,"")]).map_err(|_|Error::Unsupported{line:number,reason:"Choose one supplied /work/ CA identifier, without quotes, inline comments, expansions, URLs or traversal."})?;
                    ca = Some(value.to_string());
                }
                "TLS_REQCERT" => {
                    if certificate_level {
                        return Err(Error::Duplicate { line: number });
                    }
                    certificate_level = true;
                    if !value.eq_ignore_ascii_case("demand") && !value.eq_ignore_ascii_case("hard")
                    {
                        return Err(Error::Unsupported {
                            line: number,
                            reason: "Use demand or hard, or omit this setting for its demand default. Other real-client levels are outside this verified-peer subset; they are not syntax errors in Linux.",
                        });
                    }
                }
                _ => {
                    return Err(Error::Unsupported {
                        line: number,
                        reason: "Only TLS_CACERT and TLS_REQCERT are modeled here. The command supplies -H and -D; no other client settings, system files or environment overrides are read.",
                    });
                }
            }
        }
        Ok(Self {
            ca_input: ca.ok_or(Error::MissingCaInput)?,
        })
    }
}
