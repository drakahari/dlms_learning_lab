//! Bounded PAM auth control flow over supplied module results. No host PAM or I/O.
use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_TEXT: usize = 8192;
pub const MAX_LINES: usize = 128;
pub const MAX_LINE: usize = 256;
pub const MAX_RULES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Code {
    Success,
    AuthError,
    UserUnknown,
    AuthInfoUnavailable,
    Ignore,
    PermissionDenied,
}
impl Code {
    pub fn label(self) -> &'static str {
        match self {
            Self::Success => "PAM_SUCCESS",
            Self::AuthError => "PAM_AUTH_ERR",
            Self::UserUnknown => "PAM_USER_UNKNOWN",
            Self::AuthInfoUnavailable => "PAM_AUTHINFO_UNAVAIL",
            Self::Ignore => "PAM_IGNORE",
            Self::PermissionDenied => "PAM_PERM_DENIED",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Required,
    Requisite,
    Sufficient,
    Optional,
}
impl Control {
    pub fn label(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Requisite => "requisite",
            Self::Sufficient => "sufficient",
            Self::Optional => "optional",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    Unix,
    Permit,
    Deny,
}
impl Module {
    pub fn label(self) -> &'static str {
        match self {
            Self::Unix => "pam_unix.so",
            Self::Permit => "pam_permit.so",
            Self::Deny => "pam_deny.so",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub line: usize,
    pub control: Control,
    pub module: Module,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack {
    rules: Vec<Rule>,
}
impl Stack {
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    /// A supplied synthetic module result, never a password or a verification API.
    pub unix_result: Code,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Contribute,
    KeepFailure,
    RememberFailure,
    Ignore,
    ReturnSuccess,
    ReturnFailure,
}
impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Self::Contribute => "Contributes success; continue",
            Self::KeepFailure => "Success cannot erase the earlier failure; continue",
            Self::RememberFailure => "Remember failure; continue",
            Self::Ignore => "Ignore this result; continue",
            Self::ReturnSuccess => "Return success now",
            Self::ReturnFailure => "Return remembered failure now",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Step {
    pub line: usize,
    pub control: Control,
    pub module: Module,
    pub returned: Code,
    pub action: Action,
    pub remembered_failure: Option<Code>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Outcome {
    pub result: Code,
    pub trace: Vec<Step>,
}
impl Outcome {
    pub fn accepted(&self) -> bool {
        self.result == Code::Success
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: Option<usize>,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self.kind {
            ErrorKind::Syntax => "Invalid configuration",
            ErrorKind::Unsupported => "Unsupported in this simulator",
            ErrorKind::Limit => "Simulator limit",
        };
        write!(f, "{category}")?;
        if let Some(line) = self.line {
            write!(f, " at line {line}")?;
        }
        write!(f, ": {}", self.message)
    }
}
impl std::error::Error for Error {}
fn error(kind: ErrorKind, line: Option<usize>, message: &str) -> Error {
    Error {
        kind,
        line,
        message: message.into(),
    }
}
pub fn parse(text: &str) -> Result<Stack, Error> {
    if text.len() > MAX_TEXT {
        return Err(error(
            ErrorKind::Limit,
            None,
            "Use at most 8KiB of stack text.",
        ));
    }
    let mut rules = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = Some(index + 1);
        if index >= MAX_LINES || raw.len() > MAX_LINE {
            return Err(error(
                ErrorKind::Limit,
                line,
                "Use at most 128 physical lines and 256 bytes per line.",
            ));
        }
        if raw.chars().any(|c| c.is_control() && c != '\t') {
            return Err(error(
                ErrorKind::Syntax,
                line,
                "Unexpected control character.",
            ));
        }
        let rule = raw.split('#').next().unwrap_or("").trim();
        if rule.is_empty() {
            continue;
        }
        if rule.contains('\\') {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Line continuation and escaping are not supported; use one rule per line.",
            ));
        }
        let fields: Vec<_> = rule.split_ascii_whitespace().collect();
        if fields.len() < 3 {
            return Err(error(
                ErrorKind::Syntax,
                line,
                "Use auth CONTROL MODULE, for example auth required pam_unix.so.",
            ));
        }
        if !fields[0].eq_ignore_ascii_case("auth") {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "This activity models auth rules only; other PAM groups and service fields are not supported.",
            ));
        }
        let control = match fields[1].to_ascii_lowercase().as_str() {
            "required" => Control::Required,
            "requisite" => Control::Requisite,
            "sufficient" => Control::Sufficient,
            "optional" => Control::Optional,
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    line,
                    "Use required, requisite, sufficient or optional; bracket actions, include and substack are outside this activity.",
                ));
            }
        };
        let module = match fields[2] {
            "pam_unix.so" => Module::Unix,
            "pam_permit.so" => Module::Permit,
            "pam_deny.so" => Module::Deny,
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    line,
                    "Only the supplied pam_unix.so, pam_permit.so and pam_deny.so outcomes are modeled; no module or path is loaded.",
                ));
            }
        };
        if fields.len() != 3 {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Module arguments are outside the declared subset; no argument is silently ignored.",
            ));
        }
        if rules.len() == MAX_RULES {
            return Err(error(ErrorKind::Limit, line, "Use at most 32 auth rules."));
        }
        rules.push(Rule {
            line: index + 1,
            control,
            module,
        });
    }
    if rules.is_empty() {
        return Err(error(
            ErrorKind::Syntax,
            None,
            "Add at least one supported auth rule.",
        ));
    }
    Ok(Stack { rules })
}
/// Evaluates only an already validated, bounded stack. No callbacks or module execution.
pub fn evaluate(stack: &Stack, facts: Facts) -> Outcome {
    let mut failure = None;
    let mut success = false;
    let mut trace = Vec::with_capacity(stack.rules.len());
    for rule in &stack.rules {
        let returned = match rule.module {
            Module::Unix => facts.unix_result,
            Module::Permit => Code::Success,
            Module::Deny => Code::AuthError,
        };
        let action = match rule.control {
            Control::Required | Control::Requisite => match returned {
                Code::Success => {
                    success = true;
                    Action::Contribute
                }
                Code::Ignore => Action::Ignore,
                code => {
                    failure.get_or_insert(code);
                    if rule.control == Control::Requisite {
                        Action::ReturnFailure
                    } else {
                        Action::RememberFailure
                    }
                }
            },
            Control::Sufficient | Control::Optional => {
                if returned != Code::Success {
                    Action::Ignore
                } else {
                    success = true;
                    if rule.control == Control::Sufficient && failure.is_none() {
                        Action::ReturnSuccess
                    } else {
                        Action::Contribute
                    }
                }
            }
        };
        let action = if action == Action::Contribute && failure.is_some() {
            Action::KeepFailure
        } else {
            action
        };
        trace.push(Step {
            line: rule.line,
            control: rule.control,
            module: rule.module,
            returned,
            action,
            remembered_failure: failure,
        });
        if matches!(action, Action::ReturnFailure | Action::ReturnSuccess) {
            break;
        }
    }
    Outcome {
        result: failure.unwrap_or(if success {
            Code::Success
        } else {
            Code::PermissionDenied
        }),
        trace,
    }
}
