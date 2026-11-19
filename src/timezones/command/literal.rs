//! Project-owned finite literal-word parser; other interpreters are unchanged.
use crate::boot_entry as boot;
type Result<T> = std::result::Result<T, boot::Error>;
fn error(kind: boot::ErrorKind, message: impl Into<String>) -> boot::Error {
    boot::Error {
        kind,
        message: message.into(),
    }
}
pub(super) fn words(raw: &str) -> Result<Vec<String>> {
    if raw.len() > 1024 {
        return Err(error(
            boot::ErrorKind::Limit,
            "Keep one time-zone command within 1024 bytes.",
        ));
    }
    if raw.bytes().any(|b| !(32..=126).contains(&b) && b != b'\t') {
        return Err(error(
            boot::ErrorKind::Unsupported,
            "Use one ordinary ASCII command line; multiline commands are outside this subset.",
        ));
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else if q == '"' && c == '\\' {
                let n = chars
                    .next()
                    .ok_or_else(|| error(boot::ErrorKind::Syntax, "Finish the quoted escape."))?;
                if matches!(n, '$' | '`' | '"' | '\\') {
                    word.push(n);
                } else {
                    word.push('\\');
                    word.push(n);
                }
            } else if q == '"' && matches!(c, '$' | '`') {
                return Err(error(
                    boot::ErrorKind::Unsupported,
                    "Shell expansion is outside the literal time-zone subset.",
                ));
            } else {
                word.push(c);
            }
        } else {
            match c {
                '\'' | '"' => {
                    quote = Some(c);
                    started = true;
                }
                '\\' => {
                    word.push(chars.next().ok_or_else(|| {
                        error(boot::ErrorKind::Syntax, "Finish the command escape.")
                    })?);
                    started = true;
                }
                ' ' | '\t' => {
                    if started {
                        words.push(std::mem::take(&mut word));
                        started = false;
                    }
                }
                '#' if !started => {
                    return Err(error(
                        boot::ErrorKind::Unsupported,
                        "Comments are outside this literal time-zone subset.",
                    ));
                }
                '$' | '`' | '~' | ';' | '&' | '|' | '<' | '>' | '(' | ')' | '*' | '?' | '['
                | ']' | '{' | '}' => {
                    return Err(error(
                        boot::ErrorKind::Unsupported,
                        "Use literal supplied paths; expansions, globbing, operators and comments are outside this command subset.",
                    ));
                }
                _ => {
                    word.push(c);
                    started = true;
                }
            }
        }
        if words.len() > 24 {
            return Err(error(
                boot::ErrorKind::Limit,
                "Use at most 24 command words.",
            ));
        }
    }
    if quote.is_some() {
        return Err(error(
            boot::ErrorKind::Syntax,
            "Close the quoted command word.",
        ));
    }
    if started {
        words.push(word);
    }
    if words.len() > 24 {
        return Err(error(
            boot::ErrorKind::Limit,
            "Use at most 24 command words.",
        ));
    }
    Ok(words)
}
