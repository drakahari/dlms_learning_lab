//! Bounded mailbox-list parsing for the disclosed plain-ASCII Sieve address subset.
//! A display name is never an address. No filesystem, DNS or delivery operations.
use crate::sieve_lex::{Fault, Kind, fail};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mailbox {
    pub local: String,
    pub domain: String,
}
impl Mailbox {
    pub fn all(&self) -> String {
        format!("{}@{}", self.local, self.domain)
    }
}

/// Parse modern dot-atom mailboxes, optional display phrases and comma lists.
/// Quoted local parts, groups, domain literals and obsolete routes are explicitly
/// unsupported. Invalid dot-atom entries yield no address rather than a substring.
pub fn mailboxes(value: &str) -> Result<Vec<Mailbox>, Fault> {
    if value.len() > 1024 {
        return fail(Kind::Limit, 0, "An address field exceeds 1024 bytes.");
    }
    if !value.is_ascii() {
        return fail(
            Kind::Unsupported,
            0,
            "Address tests support plain ASCII fields only.",
        );
    }
    let mut cleaned = String::new();
    let mut quote = false;
    let mut escape = false;
    let mut comment = 0usize;
    for ch in value.chars() {
        if escape {
            if comment == 0 {
                cleaned.push(ch);
            }
            escape = false;
            continue;
        }
        if ch == '\\' && (quote || comment > 0) {
            if comment == 0 {
                cleaned.push(ch);
            }
            escape = true;
            continue;
        }
        if comment > 0 {
            match ch {
                '(' => {
                    comment += 1;
                    if comment > 16 {
                        return fail(Kind::Limit, 0, "Address comments exceed 16 nesting levels.");
                    }
                }
                ')' => comment -= 1,
                _ => {}
            }
            continue;
        }
        if ch == '"' {
            quote = !quote;
            cleaned.push(ch);
        } else if ch == '(' && !quote {
            comment = 1;
            cleaned.push(' ');
        } else if ch == ')' && !quote {
            return Ok(Vec::new());
        } else {
            cleaned.push(ch);
        }
    }
    if quote || comment != 0 || escape {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    let mut start = 0;
    let mut angle = false;
    quote = false;
    escape = false;
    for (at, ch) in cleaned.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && quote {
            escape = true;
            continue;
        }
        if ch == '"' {
            quote = !quote;
            continue;
        }
        if !quote && matches!(ch, ':' | ';' | '[' | ']') {
            return fail(
                Kind::Unsupported,
                at,
                "Address groups, obsolete routes and domain literals are outside this simulator's address subset. Use a supplied plain mailbox sample.",
            );
        }
        if !quote && ch == '<' {
            if angle {
                return Ok(Vec::new());
            }
            angle = true;
        }
        if !quote && ch == '>' {
            if !angle {
                return Ok(Vec::new());
            }
            angle = false;
        }
        if ch == ',' && !quote && !angle {
            items.push(&cleaned[start..at]);
            start = at + 1;
        }
    }
    if angle {
        return Ok(Vec::new());
    }
    items.push(&cleaned[start..]);
    if items.len() > 32 {
        return fail(
            Kind::Limit,
            0,
            "An address field may contain at most 32 mailbox entries.",
        );
    }
    let mut out = Vec::new();
    for item in items {
        if let Some(address) = mailbox(item)? {
            out.push(address);
        }
    }
    Ok(out)
}

fn mailbox(item: &str) -> Result<Option<Mailbox>, Fault> {
    let item = item.trim_matches([' ', '\t']);
    let mut quote = false;
    let mut escape = false;
    let mut open = None;
    let mut close = None;
    for (at, ch) in item.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && quote {
            escape = true;
            continue;
        }
        if ch == '"' {
            quote = !quote;
            continue;
        }
        if !quote {
            if ch == '<' {
                if open.is_some() || close.is_some() {
                    return Ok(None);
                }
                open = Some(at);
            }
            if ch == '>' {
                if open.is_none() || close.is_some() {
                    return Ok(None);
                }
                close = Some(at);
            }
        }
    }
    let raw = match (open, close) {
        (Some(a), Some(b)) if b > a && item[b + 1..].trim().is_empty() => {
            // Validate the phrase instead of accepting an unquoted impersonating @.
            let mut quoted = false;
            let mut escaped = false;
            for ch in item[..a].chars() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if quoted && ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    quoted = !quoted;
                } else if !quoted
                    && !(ch.is_ascii_alphanumeric()
                        || b"!#$%&'*+-/=?^_`{|}~. \t".contains(&(ch as u8)))
                {
                    return Ok(None);
                }
            }
            item[a + 1..b].trim_matches([' ', '\t'])
        }
        (None, None) => item,
        _ => return Ok(None),
    };
    let mut separator = None;
    quote = false;
    escape = false;
    for (at, ch) in raw.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && quote {
            escape = true;
        } else if ch == '"' {
            quote = !quote;
        } else if ch == '@' && !quote && separator.replace(at).is_some() {
            return Ok(None);
        }
    }
    let Some(at) = separator else {
        return Ok(None);
    };
    let (local, domain) = (&raw[..at], &raw[at + 1..]);
    if local.contains('"') {
        return fail(
            Kind::Unsupported,
            0,
            "Quoted mailbox local parts are outside this simulator's address subset; quoted display names are supported.",
        );
    }
    // CFWS can surround each component. Do not join internal words: removing
    // their separation could invent a different address from malformed input.
    let local = local.trim_matches([' ', '\t']);
    let domain = domain.trim_matches([' ', '\t']);
    for component in [local, domain] {
        if component
            .split('.')
            .any(|part| part != part.trim_matches([' ', '\t']))
        {
            return fail(
                Kind::Unsupported,
                0,
                "Whitespace or comments inside a dotted address component use obsolete syntax outside this subset. Spaces around @ are supported.",
            );
        }
    }
    if !dot_atom(local) || !dot_atom(domain) {
        return Ok(None);
    }
    Ok(Some(Mailbox {
        local: local.into(),
        domain: domain.into(),
    }))
}
fn dot_atom(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-/=?^_`{|}~".contains(&b))
        })
}

/// Restrict address parsing to fields whose defined structure contains mailboxes.
pub fn address_field(name: &str) -> bool {
    [
        "From",
        "To",
        "Cc",
        "Bcc",
        "Sender",
        "Reply-To",
        "Resent-From",
        "Resent-To",
        "Resent-Cc",
        "Resent-Bcc",
        "Resent-Sender",
    ]
    .iter()
    .any(|allowed| name.eq_ignore_ascii_case(allowed))
}
