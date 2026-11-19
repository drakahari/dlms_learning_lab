//! Separate plain-DN delete interface. Existing LDIF invocations are unchanged.
use super::{
    Failure, Input, Inputs, Report, client, once, syntax, unsupported, virtual_name, write_words,
};
use crate::{
    ldap_identity as identity, ldap_ldif as ldif,
    ldap_query::{Dn, Error},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeleteInput {
    Names(Vec<String>),
    Data(Input),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteInvocation {
    uri: String,
    bind: Dn,
    input: DeleteInput,
}
impl DeleteInvocation {
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn bind(&self) -> &Dn {
        &self.bind
    }
    pub fn input(&self) -> &DeleteInput {
        &self.input
    }
}
/// This API parses only ldapdelete; the established parse() remains LDIF-only.
pub fn parse_delete(raw: &str) -> Result<DeleteInvocation, Error> {
    let args = write_words(raw)?;
    if args.first().map(String::as_str) != Some("ldapdelete") {
        return Err(unsupported(
            "This interface supports ldapdelete with explicit training authentication; no command was run.",
        ));
    }
    let (mut simple, mut prompt, mut operands) = (false, false, false);
    let (mut uri, mut bind, mut file) = (None, None, None);
    let mut names = Vec::new();
    let mut at = 1;
    while at < args.len() {
        match args[at].as_str() {
            "-x" if !operands => {
                once(&mut simple)?;
                at += 1;
            }
            "-W" if !operands => {
                once(&mut prompt)?;
                at += 1;
            }
            flag @ ("-H" | "-D" | "-f") if !operands => {
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
            "--" if !operands => {
                operands = true;
                at += 1;
            }
            value if value.starts_with('-') => {
                return Err(unsupported(
                    "Options must precede DNs. Only -x -W -H -D and optional -f are supported; recursive/continuous/dry-run/control/password options are not implemented.",
                ));
            }
            value => {
                if file.is_some() {
                    return Err(unsupported(
                        "Choose positional DNs or virtual file input, not both, in this subset.",
                    ));
                }
                if names.len() >= ldif::MAX_RECORDS || value.len() > 1024 {
                    return Err(Error::Limit(
                        "Use at most 16 DN arguments, each at most 1024 bytes.".into(),
                    ));
                }
                operands = true;
                names.push(value.to_string());
                at += 1;
            }
        }
    }
    if !simple || !prompt {
        return Err(unsupported(
            "This subset requires -x and -W with a supplied fictional profile; anonymous/SASL/password arguments are not supported.",
        ));
    }
    Ok(DeleteInvocation {
        uri: uri.ok_or_else(|| unsupported("Supply -H for a provided virtual directory."))?,
        bind: bind.ok_or_else(|| {
            unsupported(
                "Supply -D for a provided training identity; naming it does not authenticate it.",
            )
        })?,
        input: if names.is_empty() {
            DeleteInput::Data(file.map_or(Input::Editor, Input::VirtualFile))
        } else {
            DeleteInput::Names(names)
        },
    })
}
fn diagnostic(category: ldif::Category, line: usize, message: &str) -> ldif::Diagnostic {
    ldif::Diagnostic {
        category,
        line,
        message: message.into(),
    }
}
fn name(raw: &str, line: usize) -> Result<Dn, ldif::Diagnostic> {
    if raw.len() > 1024 {
        return Err(diagnostic(
            ldif::Category::Limit,
            line,
            "A DN allows at most 1024 bytes.",
        ));
    }
    if raw.is_empty() || raw.starts_with('#') || raw.bytes().any(|b| !(32..=126).contains(&b)) {
        return Err(diagnostic(
            ldif::Category::Unsupported,
            line,
            "Use one nonempty printable ASCII DN per line. Blank/comment lines, CRLF and controls are outside this DN-list subset; use LF. This is not LDIF input.",
        ));
    }
    Dn::parse(raw).map_err(|e| {
        let category = match e {
            Error::Syntax(_) => ldif::Category::Syntax,
            Error::Unsupported(_) => ldif::Category::Unsupported,
            Error::Limit(_) => ldif::Category::Limit,
            Error::Fixture(_) => ldif::Category::Unsupported,
        };
        diagnostic(category, line, &e.to_string())
    })
}
/// Finite, virtual-only deletion. Later failures retain prior successful requests.
pub fn run_delete(
    command: &str,
    sample_password: &str,
    inputs: &Inputs<'_>,
    realm: &mut identity::Realm,
) -> Report {
    let mut report = Report {
        applied: Vec::new(),
        failure: None,
    };
    let invocation = match parse_delete(command) {
        Ok(i) => i,
        Err(e) => {
            report.failure = Some(Failure::Invocation(e));
            return report;
        }
    };
    let mut lines: Box<dyn Iterator<Item = &str> + '_> = match invocation.input() {
        DeleteInput::Names(names) => Box::new(names.iter().map(String::as_str)),
        DeleteInput::Data(input) => {
            let Some(text) = inputs.select(input) else {
                report.failure = Some(Failure::MissingVirtualInput);
                return report;
            };
            if text.len() > ldif::MAX_TEXT || text.split_terminator('\n').count() > ldif::MAX_LINES
            {
                report.failure = Some(Failure::Record {
                    number: 1,
                    error: diagnostic(
                        ldif::Category::Limit,
                        1,
                        "DN-list input allows at most 64 KiB and 1024 lines; no requests were attempted.",
                    ),
                });
                return report;
            }
            Box::new(text.split_terminator('\n'))
        }
    };
    if invocation.uri() != realm.uri() {
        report.failure = Some(Failure::UnavailableVirtualDirectory);
        return report;
    }
    let session = match realm.authenticate(invocation.bind(), sample_password) {
        Ok(s) => s,
        Err(e) => {
            report.failure = Some(Failure::Authentication(e));
            return report;
        }
    };
    for (index, raw) in lines.by_ref().enumerate() {
        if index >= ldif::MAX_RECORDS {
            report.failure = Some(Failure::Record {
                number: index + 1,
                error: diagnostic(
                    ldif::Category::Limit,
                    index + 1,
                    "At most 16 DN deletions; earlier successful requests remain applied.",
                ),
            });
            break;
        }
        let dn = match name(raw, index + 1) {
            Ok(d) => d,
            Err(e) => {
                report.failure = Some(Failure::Record {
                    number: index + 1,
                    error: e,
                });
                break;
            }
        };
        match realm.apply(&session, &ldif::Change::Delete(dn)) {
            Ok(applied) => report.applied.push(applied),
            Err(error) => {
                report.failure = Some(Failure::Request {
                    number: index + 1,
                    error,
                });
                break;
            }
        }
    }
    if report.failure.is_none() && report.applied.is_empty() {
        report.failure = Some(Failure::Record {
            number: 1,
            error: diagnostic(
                ldif::Category::Syntax,
                1,
                "Enter at least one DN, one per line. Empty delete input is outside this subset.",
            ),
        });
    }
    report
}
