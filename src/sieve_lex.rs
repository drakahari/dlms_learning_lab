//! Bounded Sieve tokenization with explicit syntax/unsupported/limit diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Kind {
    Syntax,
    Unsupported,
    Limit,
    Runtime,
}
impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Syntax => "Syntax error",
            Self::Unsupported => "Unsupported syntax",
            Self::Limit => "Limit reached",
            Self::Runtime => "Filing error",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: Kind,
    pub offset: usize,
    pub message: String,
}
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "At byte {}: {}", self.offset, self.message)
    }
}
impl std::error::Error for Fault {}
impl Fault {
    pub fn explain(&self, source: &str) -> String {
        let mut at = self.offset.min(source.len());
        while !source.is_char_boundary(at) {
            at -= 1;
        }
        let before = &source[..at];
        let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        format!(
            "{} · line {line}, column {column}: {}",
            self.kind.label(),
            self.message
        )
    }
}
pub(crate) fn fail<T>(kind: Kind, offset: usize, message: &str) -> Result<T, Fault> {
    Err(Fault {
        kind,
        offset,
        message: message.into(),
    })
}
pub const MAX_SOURCE: usize = 16 * 1024;
pub const MAX_TOKENS: usize = 4096;
pub const MAX_STRING: usize = 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Token {
    Name(String),
    Tag(String),
    Text(String),
    Number(u64),
    Open,
    Close,
    ListOpen,
    ListClose,
    TestOpen,
    TestClose,
    Comma,
    Semi,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Located {
    pub token: Token,
    pub offset: usize,
}
pub(crate) fn lex(source: &str) -> Result<Vec<Located>, Fault> {
    if source.len() > MAX_SOURCE {
        return fail(
            Kind::Limit,
            0,
            "The script exceeds the 16 KiB source limit.",
        );
    }
    if let Some(at) = source.find('\0') {
        return fail(Kind::Syntax, at, "A script cannot contain a NUL character.");
    }
    let bytes = source.as_bytes();
    let mut at = 0;
    let mut out = Vec::new();
    while at < bytes.len() {
        let start = at;
        if bytes[at] == b'\r' && bytes.get(at + 1) != Some(&b'\n') {
            return fail(
                Kind::Syntax,
                at,
                "Use LF or CRLF line endings, not a lone carriage return.",
            );
        }
        if matches!(bytes[at], b' ' | b'\t' | b'\r' | b'\n') {
            at += 1;
            continue;
        }
        if bytes[at] == b'#' {
            while at < bytes.len() && !matches!(bytes[at], b'\r' | b'\n') {
                at += 1;
            }
            continue;
        }
        if bytes[at..].starts_with(b"/*") {
            let Some(end) = source[at + 2..].find("*/") else {
                return fail(Kind::Syntax, at, "Close the comment with */.");
            };
            at += end + 4;
            continue;
        }
        let token = match bytes[at] {
            b'{' => {
                at += 1;
                Token::Open
            }
            b'}' => {
                at += 1;
                Token::Close
            }
            b'[' => {
                at += 1;
                Token::ListOpen
            }
            b']' => {
                at += 1;
                Token::ListClose
            }
            b'(' => {
                at += 1;
                Token::TestOpen
            }
            b')' => {
                at += 1;
                Token::TestClose
            }
            b',' => {
                at += 1;
                Token::Comma
            }
            b';' => {
                at += 1;
                Token::Semi
            }
            b'"' => {
                at += 1;
                let mut text = String::new();
                let mut closed = false;
                while at < bytes.len() {
                    if bytes[at] == b'"' {
                        at += 1;
                        closed = true;
                        break;
                    }
                    if bytes[at] == b'\\' {
                        at += 1;
                        if at == bytes.len() {
                            break;
                        }
                    }
                    let c = source[at..].chars().next().expect("valid UTF-8 boundary");
                    if matches!(c, '\r' | '\n') {
                        return fail(
                            Kind::Unsupported,
                            start,
                            "Multiline quoted strings are outside this subset. Keep this quoted value on one line.",
                        );
                    }
                    text.push(c);
                    at += c.len_utf8();
                    if text.len() > MAX_STRING {
                        return fail(Kind::Limit, start, "A string exceeds the 1024-byte limit.");
                    }
                }
                if !closed {
                    return fail(Kind::Syntax, start, "Close the string with a double quote.");
                }
                Token::Text(text)
            }
            b'0'..=b'9' => {
                while at < bytes.len() && bytes[at].is_ascii_digit() {
                    at += 1;
                }
                let Ok(mut n) = source[start..at].parse::<u64>() else {
                    return fail(Kind::Limit, start, "This number is too large.");
                };
                if let Some(suffix) = bytes.get(at).map(u8::to_ascii_lowercase) {
                    let shift = match suffix {
                        b'k' => 10,
                        b'm' => 20,
                        b'g' => 30,
                        _ => 0,
                    };
                    if shift != 0 {
                        at += 1;
                        let Some(scaled) = n.checked_mul(1u64 << shift) else {
                            return fail(Kind::Limit, start, "This number is too large.");
                        };
                        n = scaled;
                    }
                }
                Token::Number(n)
            }
            b':' => {
                at += 1;
                let name = at;
                if !bytes.get(at).is_some_and(u8::is_ascii_alphabetic) {
                    return fail(Kind::Syntax, start, "A tag needs a name after the colon.");
                }
                while at < bytes.len() && (bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_') {
                    at += 1;
                }
                Token::Tag(source[name..at].to_ascii_lowercase())
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while at < bytes.len() && (bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_') {
                    at += 1;
                }
                let name = source[start..at].to_ascii_lowercase();
                if name == "text" && bytes.get(at) == Some(&b':') {
                    return fail(
                        Kind::Unsupported,
                        start,
                        "Multiline text: strings are outside this subset. Use quoted strings.",
                    );
                }
                Token::Name(name)
            }
            b'$' | b'`' | b'|' => {
                return fail(
                    Kind::Unsupported,
                    start,
                    "This is Sieve, not a shell. Variables, subprocesses and pipelines are unavailable.",
                );
            }
            _ => {
                return fail(
                    Kind::Syntax,
                    start,
                    "Unexpected character. Use a Sieve command, quoted string or punctuation.",
                );
            }
        };
        if out.len() == MAX_TOKENS {
            return fail(
                Kind::Limit,
                start,
                "The script exceeds the 4096-token limit.",
            );
        }
        out.push(Located {
            token,
            offset: start,
        });
    }
    Ok(out)
}
