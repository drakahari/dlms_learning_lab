//! Executes validated requests against virtual data only; no OS input lookup.
use super::{Input, parse, virtual_name};
use crate::{ldap_changes, ldap_identity, ldap_ldif, ldap_query};

pub const MAX_FILES: usize = 8;
pub const MAX_INPUT_BYTES: usize = 128 * 1024;

/// Borrowed editor and explicitly supplied virtual files. Never OS path lookup.
pub struct Inputs<'a> {
    editor: &'a str,
    files: Vec<(&'a str, &'a str)>,
}
impl<'a> Inputs<'a> {
    pub fn new(editor: &'a str, files: &[(&'a str, &'a str)]) -> Result<Self, ldap_query::Error> {
        if files.len() > MAX_FILES || editor.len() > ldap_ldif::MAX_TEXT {
            return Err(ldap_query::Error::Limit(
                "Supply at most 8 virtual files and a 64 KiB editor.".into(),
            ));
        }
        let mut bytes = editor.len();
        for (i, (name, text)) in files.iter().enumerate() {
            virtual_name(name)?;
            if files[..i].iter().any(|(old, _)| old == name) {
                return Err(ldap_query::Error::Syntax(
                    "Each virtual file identifier must be distinct.".into(),
                ));
            }
            if text.len() > ldap_ldif::MAX_TEXT || bytes + text.len() > MAX_INPUT_BYTES {
                return Err(ldap_query::Error::Limit(
                    "Each virtual input allows 64 KiB; combined input allows 128 KiB.".into(),
                ));
            }
            bytes += text.len();
        }
        Ok(Self {
            editor,
            files: files.to_vec(),
        })
    }
    pub(super) fn select(&self, input: &Input) -> Option<&str> {
        match input {
            Input::Editor => Some(self.editor),
            Input::VirtualFile(name) => self
                .files
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, text)| *text),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    Invocation(ldap_query::Error),
    MissingVirtualInput,
    UnavailableVirtualDirectory,
    Authentication(ldap_identity::Error),
    Record {
        number: usize,
        error: ldap_ldif::Diagnostic,
    },
    Request {
        number: usize,
        error: ldap_identity::Error,
    },
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invocation(e) => write!(f, "{e}"),
            Self::MissingVirtualInput => write!(
                f,
                "The selected virtual input is not supplied by this lesson. No host file was opened."
            ),
            Self::UnavailableVirtualDirectory => write!(
                f,
                "That virtual directory is not supplied by this lesson. No network connection was attempted."
            ),
            Self::Authentication(e) => write!(f, "{e}"),
            Self::Record { number, error } => write!(
                f,
                "Record {number}: {error}. Processing stopped; earlier successful changes remain."
            ),
            Self::Request { number, error } => write!(
                f,
                "Record {number}: {error}. Processing stopped; earlier successful changes remain."
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub applied: Vec<ldap_changes::Applied>,
    pub failure: Option<Failure>,
}
impl Report {
    pub fn succeeded(&self) -> bool {
        self.failure.is_none() && !self.applied.is_empty()
    }
    pub fn exit_status(&self) -> u8 {
        match &self.failure {
            None if !self.applied.is_empty() => 0,
            Some(Failure::Authentication(ldap_identity::Error::InvalidCredentials)) => 49,
            Some(Failure::Request {
                error: ldap_identity::Error::Request(ldap_changes::Error::Result { code, .. }),
                ..
            }) => *code as u8,
            Some(Failure::Request {
                error: ldap_identity::Error::ForeignSession,
                ..
            }) => 50,
            _ => 1,
        }
    }
    fn fail(&mut self, failure: Failure) {
        self.failure = Some(failure);
    }
}
/// A finite invocation; the supplied sample password is never added to output.
/// No credential from a previous invocation is inherited, including after failure.
pub fn run(
    command: &str,
    sample_password: &str,
    inputs: &Inputs<'_>,
    realm: &mut ldap_identity::Realm,
) -> Report {
    let mut report = Report {
        applied: Vec::new(),
        failure: None,
    };
    let invocation = match parse(command) {
        Ok(i) => i,
        Err(e) => {
            report.fail(Failure::Invocation(e));
            return report;
        }
    };
    let Some(input) = inputs.select(invocation.input()) else {
        report.fail(Failure::MissingVirtualInput);
        return report;
    };
    if invocation.uri() != realm.uri() {
        report.fail(Failure::UnavailableVirtualDirectory);
        return report;
    }
    let session = match realm.authenticate(invocation.bind(), sample_password) {
        Ok(s) => s,
        Err(e) => {
            report.fail(Failure::Authentication(e));
            return report;
        }
    };
    let records = match ldap_ldif::UtilityRecords::new(input, invocation.add_default()) {
        Ok(r) => r,
        Err(e) => {
            report.fail(Failure::Record {
                number: 1,
                error: e,
            });
            return report;
        }
    };
    for (index, change) in records.enumerate() {
        let change = match change {
            Ok(c) => c,
            Err(e) => {
                report.fail(Failure::Record {
                    number: index + 1,
                    error: e,
                });
                break;
            }
        };
        match realm.apply(&session, &change) {
            Ok(applied) => report.applied.push(applied),
            Err(e) => {
                report.fail(Failure::Request {
                    number: index + 1,
                    error: e,
                });
                break;
            }
        }
    }
    report
}
