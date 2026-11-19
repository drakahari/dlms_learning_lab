//! Bounded LDIF parsing only: no evaluation, I/O, commands, credentials or transport.
use crate::ldap_query::{self as query, Attribute, Dn, EntrySource};
pub const MAX_TEXT: usize = 64 * 1024;
pub const MAX_LINES: usize = 1024;
pub const MAX_RECORDS: usize = 16;
pub const MAX_LINE: usize = 2048;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Syntax,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub category: Category,
    pub line: usize,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at line {}: {}",
            self.category, self.line, self.message
        )
    }
}
impl std::error::Error for Diagnostic {}
type Result<T> = std::result::Result<T, Diagnostic>;
fn error(category: Category, line: usize, message: &str) -> Diagnostic {
    Diagnostic {
        category,
        line,
        message: message.into(),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Add,
    Delete,
    Replace,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub kind: EditKind,
    pub attribute: Attribute,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Add(EntrySource),
    Delete(Dn),
    Modify { dn: Dn, edits: Vec<Edit> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Content(Vec<EntrySource>),
    Changes(Vec<Change>),
}
#[derive(Clone, Debug)]
struct Line {
    number: usize,
    text: String,
}
fn descriptor(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
fn unfold(raw: &str) -> Result<Vec<Line>> {
    if raw.len() > MAX_TEXT {
        return Err(error(Category::Limit, 1, "LDIF exceeds64KiB."));
    }
    if raw
        .bytes()
        .any(|b| !(32..=126).contains(&b) && !matches!(b, b'\r' | b'\n'))
    {
        return Err(error(
            Category::Unsupported,
            1,
            "Only printable ASCII data and LF/CRLF are supported.",
        ));
    }
    let mut lines: Vec<Line> = Vec::new();
    for (i, rawline) in raw.split_terminator('\n').enumerate() {
        if i >= MAX_LINES {
            return Err(error(
                Category::Limit,
                i + 1,
                "LDIF exceeds1024physical lines.",
            ));
        }
        let line = rawline.strip_suffix('\r').unwrap_or(rawline);
        if line.contains('\r') {
            return Err(error(
                Category::Syntax,
                i + 1,
                "Use LF or CRLF; a bare carriage return is not supported.",
            ));
        }
        if let Some(rest) = line.strip_prefix(' ') {
            let prev = lines
                .last_mut()
                .filter(|l| !l.text.is_empty())
                .ok_or_else(|| {
                    error(
                        Category::Syntax,
                        i + 1,
                        "A continuation needs a preceding nonempty logical line.",
                    )
                })?;
            if prev.text.len() + rest.len() > MAX_LINE {
                return Err(error(
                    Category::Limit,
                    i + 1,
                    "Unfolded line exceeds2048bytes.",
                ));
            }
            prev.text.push_str(rest);
        } else {
            if line.len() > MAX_LINE {
                return Err(error(
                    Category::Limit,
                    i + 1,
                    "Logical line exceeds2048bytes.",
                ));
            }
            lines.push(Line {
                number: i + 1,
                text: line.into(),
            });
        }
    }
    Ok(lines
        .into_iter()
        .filter(|l| !l.text.starts_with('#'))
        .collect())
}
fn sextet(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
fn decode64(raw: &str, max: usize, line: usize) -> Result<String> {
    let b = raw.as_bytes();
    if b.is_empty() || !b.len().is_multiple_of(4) {
        return Err(error(
            Category::Syntax,
            line,
            "Use complete standard base64 groups and padding.",
        ));
    }
    if b.len() / 4 * 3 > max + 2 {
        return Err(error(
            Category::Limit,
            line,
            "Encoded value exceeds its decoded byte limit.",
        ));
    }
    let mut out = Vec::new();
    for (i, c) in b.as_chunks::<4>().0.iter().enumerate() {
        let last = i + 1 == b.len() / 4;
        let a = sextet(c[0])
            .ok_or_else(|| error(Category::Syntax, line, "Invalid base64 alphabet/padding."))?;
        let d = sextet(c[1])
            .ok_or_else(|| error(Category::Syntax, line, "Invalid base64 alphabet/padding."))?;
        let (n, e, f) = if c[2] == b'=' {
            if !last || c[3] != b'=' || d & 15 != 0 {
                return Err(error(
                    Category::Syntax,
                    line,
                    "Invalid base64 padding or unused bits.",
                ));
            }
            (1, 0, 0)
        } else {
            let e = sextet(c[2])
                .ok_or_else(|| error(Category::Syntax, line, "Invalid base64 alphabet."))?;
            if c[3] == b'=' {
                if !last || e & 3 != 0 {
                    return Err(error(
                        Category::Syntax,
                        line,
                        "Invalid base64 padding or unused bits.",
                    ));
                }
                (2, e, 0)
            } else {
                (
                    3,
                    e,
                    sextet(c[3])
                        .ok_or_else(|| error(Category::Syntax, line, "Invalid base64 alphabet."))?,
                )
            }
        };
        let bytes = [(a << 2) | (d >> 4), (d << 4) | (e >> 2), (e << 6) | f];
        if out.len() + n > max {
            return Err(error(
                Category::Limit,
                line,
                "Decoded value exceeds its byte limit.",
            ));
        }
        if bytes[..n].iter().any(|b| !(32..=126).contains(b)) {
            return Err(error(
                Category::Unsupported,
                line,
                "Decoded binary, control and non-ASCII values are not supported.",
            ));
        }
        out.extend_from_slice(&bytes[..n]);
    }
    String::from_utf8(out)
        .map_err(|_| error(Category::Unsupported, line, "Printable ASCII values only."))
}
fn field(line: &Line) -> Result<(String, String)> {
    field_with_value_limit(line, 256)
}
// Explicit opt-in limit for configuration values; the contact codec keeps its
// original 256-byte value and 1024-byte DN limits.
fn field_with_value_limit(line: &Line, value_max: usize) -> Result<(String, String)> {
    let Some((name, raw)) = line.text.split_once(':') else {
        return Err(error(
            Category::Syntax,
            line.number,
            "Use attribute: value, attribute:: base64, or a modification separator -.",
        ));
    };
    if !descriptor(name) {
        return Err(error(
            Category::Unsupported,
            line.number,
            "Use an attribute descriptor without options, spaces or OID aliases.",
        ));
    }
    let name = name.to_ascii_lowercase();
    if name == "control" {
        return Err(error(
            Category::Unsupported,
            line.number,
            "LDIF controls are not implemented.",
        ));
    }
    let max = if name == "dn" { 1024 } else { value_max };
    let value = if let Some(encoded) = raw.strip_prefix(':') {
        decode64(encoded.trim_start_matches(' '), max, line.number)?
    } else if raw.starts_with('<') {
        return Err(error(
            Category::Unsupported,
            line.number,
            "LDIF resource URLs are not supported; no resource was opened.",
        ));
    } else {
        let plain = raw.trim_start_matches(' ');
        if plain.starts_with([':', '<']) {
            return Err(error(
                Category::Syntax,
                line.number,
                "Use base64 for a literal unsafe initial character; resource URLs are unsupported.",
            ));
        }
        if plain.len() > max {
            return Err(error(
                Category::Limit,
                line.number,
                "Value exceeds its supported byte limit.",
            ));
        }
        plain.into()
    };
    if value.is_empty() {
        return Err(error(
            Category::Unsupported,
            line.number,
            "Empty attribute values are outside this subset. An empty delete/replace list has no value lines.",
        ));
    }
    Ok((name, value))
}
fn dn(line: &Line) -> Result<Dn> {
    let (name, value) = field(line)?;
    if name != "dn" {
        return Err(error(
            Category::Syntax,
            line.number,
            "Start each record with dn: distinguished-name.",
        ));
    }
    Dn::parse(&value).map_err(|e| {
        let category = match e {
            query::Error::Syntax(_) => Category::Syntax,
            query::Error::Limit(_) => Category::Limit,
            _ => Category::Unsupported,
        };
        error(category, line.number, &e.to_string())
    })
}
fn attributes(lines: &[Line]) -> Result<Vec<Attribute>> {
    if lines.is_empty() {
        return Err(error(
            Category::Syntax,
            1,
            "An added entry needs attribute values.",
        ));
    }
    let mut attrs: Vec<Attribute> = Vec::new();
    for line in lines {
        let (name, value) = field(line)?;
        if matches!(name.as_str(), "dn" | "version" | "changetype") {
            return Err(error(
                Category::Syntax,
                line.number,
                "Place dn first and changetype only immediately after it; version belongs at the document start.",
            ));
        }
        if let Some(a) = attrs.iter_mut().find(|a| a.name == name) {
            if a.values.len() >= 16 {
                return Err(error(
                    Category::Limit,
                    line.number,
                    "At most16values per attribute.",
                ));
            }
            a.values.push(value);
        } else {
            if attrs.len() >= 16 {
                return Err(error(
                    Category::Limit,
                    line.number,
                    "At most16attributes per entry.",
                ));
            }
            attrs.push(Attribute {
                name,
                values: vec![value],
            });
        }
    }
    Ok(attrs)
}
fn edits(lines: &[Line]) -> Result<Vec<Edit>> {
    let mut at = 0;
    let mut edits = Vec::new();
    while at < lines.len() {
        if edits.len() >= 16 {
            return Err(error(
                Category::Limit,
                lines[at].number,
                "At most16edits per Modify request.",
            ));
        }
        let head = &lines[at];
        let (op, name) = field(head)?;
        let kind = match op.as_str() {
            "add" => EditKind::Add,
            "delete" => EditKind::Delete,
            "replace" => EditKind::Replace,
            _ => {
                return Err(error(
                    Category::Unsupported,
                    head.number,
                    "Supported modifications: add, delete or replace mail/description.",
                ));
            }
        };
        let name = name.to_ascii_lowercase();
        if !matches!(name.as_str(), "mail" | "description") {
            return Err(error(
                Category::Unsupported,
                head.number,
                "This first Modify subset changes mail or description only; naming/schema/password changes are not implemented.",
            ));
        }
        at += 1;
        let mut values = Vec::new();
        while at < lines.len() && lines[at].text != "-" {
            let (attr, value) = field(&lines[at])?;
            if attr != name {
                return Err(error(
                    Category::Syntax,
                    lines[at].number,
                    "Value lines must name the attribute in the current modification.",
                ));
            }
            if values.len() >= 16 {
                return Err(error(
                    Category::Limit,
                    lines[at].number,
                    "At most16values per edit.",
                ));
            }
            values.push(value);
            at += 1;
        }
        if at == lines.len() {
            return Err(error(
                Category::Syntax,
                head.number,
                "Finish each modification with a line containing only -.",
            ));
        }
        at += 1;
        if kind == EditKind::Add && values.is_empty() {
            return Err(error(
                Category::Syntax,
                head.number,
                "An add modification needs at least one value.",
            ));
        }
        edits.push(Edit {
            kind,
            attribute: Attribute { name, values },
        });
    }
    if edits.is_empty() {
        return Err(error(
            Category::Syntax,
            1,
            "A Modify record needs at least one modification.",
        ));
    }
    Ok(edits)
}
pub fn parse(raw: &str) -> Result<Document> {
    let lines = unfold(raw)?;
    let mut input = lines.iter().filter(|l| !l.text.is_empty());
    let first = input.next().ok_or_else(|| {
        error(
            Category::Syntax,
            1,
            "Start with version: 1 and at least one record.",
        )
    })?;
    let (name, value) = field(first)?;
    if name != "version" || value != "1" {
        return Err(error(
            Category::Unsupported,
            first.number,
            "This LDIF subset requires explicit version: 1.",
        ));
    }
    // Keep original blank record boundaries after the header, with comments ignored.
    let header = lines.iter().position(|l| std::ptr::eq(l, first)).unwrap();
    let mut records: Vec<Vec<Line>> = Vec::new();
    let mut current = Vec::new();
    for line in &lines[header + 1..] {
        if line.text.is_empty() {
            if !current.is_empty() {
                if records.len() >= MAX_RECORDS {
                    return Err(error(
                        Category::Limit,
                        line.number,
                        "At most16LDIF records.",
                    ));
                }
                records.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line.clone());
        }
    }
    if !current.is_empty() {
        if records.len() >= MAX_RECORDS {
            return Err(error(
                Category::Limit,
                current[0].number,
                "At most16LDIF records.",
            ));
        }
        records.push(current);
    }
    if records.is_empty() {
        return Err(error(
            Category::Syntax,
            first.number,
            "Include at least one record after version: 1.",
        ));
    }
    let mut content = Vec::new();
    let mut changes = Vec::new();
    let mut mode = None;
    for record in records {
        let name = dn(&record[0])?;
        let second = record.get(1).map(field).transpose()?;
        let changed = second.as_ref().is_some_and(|(n, _)| n == "changetype");
        if mode.is_some_and(|m| m != changed) {
            return Err(error(
                Category::Syntax,
                record[0].number,
                "Do not mix content records and change records in one document.",
            ));
        }
        mode = Some(changed);
        if changed {
            let kind = second.unwrap().1;
            let change = change_record(&record, name, &kind)?;
            changes.push(change);
        } else {
            content.push(EntrySource {
                dn: name.text().into(),
                attributes: attributes(&record[1..])?,
            });
        }
    }
    Ok(if mode == Some(true) {
        Document::Changes(changes)
    } else {
        Document::Content(content)
    })
}

fn change_record(record: &[Line], name: Dn, kind: &str) -> Result<Change> {
    Ok(match kind {
        "add" => Change::Add(EntrySource {
            dn: name.text().into(),
            attributes: attributes(&record[2..])?,
        }),
        "delete" => {
            if record.len() != 2 {
                return Err(error(
                    Category::Syntax,
                    record[2].number,
                    "A delete record contains only dn and changetype.",
                ));
            }
            Change::Delete(name)
        }
        "modify" => Change::Modify {
            dn: name,
            edits: edits(&record[2..])?,
        },
        _ => {
            return Err(error(
                Category::Unsupported,
                record[1].number,
                "Supported change records: add, delete and restricted modify; rename/move and other types are not implemented.",
            ));
        }
    })
}

mod utility_records;
pub use utility_records::UtilityRecords;

/// Separately selected configuration-change subset; data-DN parsing is unchanged.
pub mod configuration;

pub mod schema;
