//! Bounded read-only LDAP naming/search semantics. No transport, credentials or host I/O.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_JSON: usize = 512 * 1024;
pub const MAX_ENTRIES: usize = 128;
pub const MAX_FILTER: usize = 4096;
pub const MAX_NODES: usize = 128;
pub const MAX_DEPTH: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Syntax(String),
    Unsupported(String),
    Limit(String),
    Fixture(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (kind, text) = match self {
            Self::Syntax(s) => ("Syntax", s),
            Self::Unsupported(s) => ("Unsupported", s),
            Self::Limit(s) => ("Limit", s),
            Self::Fixture(s) => ("Directory", s),
        };
        write!(f, "{kind}: {text}")
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn printable(s: &str) -> bool {
    s.bytes().all(|b| (32..=126).contains(&b))
}
fn descriptor(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
fn known(a: &str) -> bool {
    matches!(
        a,
        "cn" | "sn" | "ou" | "uid" | "dc" | "mail" | "description" | "objectclass"
    )
}
fn class(s: &str) -> bool {
    matches!(
        s.to_ascii_lowercase().as_str(),
        "top"
            | "domain"
            | "organizationalunit"
            | "person"
            | "organizationalperson"
            | "inetorgperson"
    )
}
// Restricted printable-ASCII equality preparation for the declared text matching rules.
fn prepared(s: &str) -> String {
    s.split(' ')
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}
fn value_valid(a: &str, v: &str) -> bool {
    !v.is_empty() && v.len() <= 256 && printable(v) && (a != "objectclass" || class(v))
}
fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
fn escaped(bytes: &[u8], pos: &mut usize, dn: bool) -> Result<u8> {
    *pos += 1;
    let Some(&first) = bytes.get(*pos) else {
        return Err(Error::Syntax("Finish the backslash escape.".into()));
    };
    if let (Some(h), Some(l)) = (hex(first), bytes.get(*pos + 1).and_then(|b| hex(*b))) {
        *pos += 2;
        let b = h * 16 + l;
        if !(32..=126).contains(&b) {
            return Err(Error::Unsupported(
                "Only printable ASCII escaped values are supported.".into(),
            ));
        }
        return Ok(b);
    }
    if dn && b" ,+\"\\<>;=#".contains(&first) {
        *pos += 1;
        return Ok(first);
    }
    Err(Error::Syntax(
        if dn {
            "Use a DN special-character escape or two hexadecimal digits."
        } else {
            "Filter escapes require two hexadecimal digits, such as \\2a."
        }
        .into(),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Rdn {
    attribute: String,
    value: String,
}
/// Private parsed components prevent callers from constructing unvalidated DNs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dn {
    text: String,
    components: Vec<Rdn>,
}
impl Dn {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > 1024 {
            return Err(Error::Limit("A DN may contain at most 1024 bytes.".into()));
        }
        if text.is_empty() {
            return Err(Error::Unsupported(
                "Empty DN/root DSE searches are outside this subset.".into(),
            ));
        }
        if !printable(text) {
            return Err(Error::Unsupported(
                "DNs use printable ASCII in this simulator.".into(),
            ));
        }
        let b = text.as_bytes();
        let mut pos = 0;
        let mut components = Vec::new();
        while pos < b.len() {
            if components.len() >= 16 {
                return Err(Error::Limit(
                    "A DN may contain at most 16 relative names.".into(),
                ));
            }
            let start = pos;
            while pos < b.len() && b[pos] != b'=' {
                pos += 1;
            }
            if pos == b.len() {
                return Err(Error::Syntax(
                    "Each relative name needs attribute=value.".into(),
                ));
            }
            let attr = &text[start..pos];
            if !descriptor(attr) {
                return Err(Error::Unsupported(
                    "Use unspaced cn, ou, uid or dc DN attribute names.".into(),
                ));
            }
            let attribute = attr.to_ascii_lowercase();
            if !matches!(attribute.as_str(), "cn" | "ou" | "uid" | "dc") {
                return Err(Error::Unsupported(
                    "Supported DN attributes are cn, ou, uid and dc.".into(),
                ));
            }
            pos += 1;
            let mut value = Vec::new();
            let mut last_unescaped_space = false;
            while pos < b.len() && b[pos] != b',' {
                let c = b[pos];
                if c == b'\\' {
                    value.push(escaped(b, &mut pos, true)?);
                    last_unescaped_space = false;
                } else {
                    if c == b'+' {
                        return Err(Error::Unsupported(
                            "Multi-valued relative names are not supported.".into(),
                        ));
                    }
                    if b"\"<>;".contains(&c) || value.is_empty() && matches!(c, b' ' | b'#') {
                        return Err(Error::Syntax(
                            "Escape leading space/# and DN special characters.".into(),
                        ));
                    }
                    value.push(c);
                    last_unescaped_space = c == b' ';
                    pos += 1;
                }
                if value.len() > 256 {
                    return Err(Error::Limit(
                        "Each DN value is limited to 256 bytes.".into(),
                    ));
                }
            }
            if value.is_empty() || last_unescaped_space {
                return Err(Error::Syntax(
                    "Use a nonempty DN value and escape a trailing space.".into(),
                ));
            }
            // Input and decoded bytes are explicitly printable ASCII.
            let value = String::from_utf8(value)
                .map_err(|_| Error::Unsupported("ASCII DN values only.".into()))?;
            if prepared(&value).is_empty() {
                return Err(Error::Unsupported(
                    "Space-only naming values are outside this subset.".into(),
                ));
            }
            components.push(Rdn {
                attribute,
                value: prepared(&value),
            });
            if pos < b.len() {
                pos += 1;
                if pos == b.len() {
                    return Err(Error::Syntax("A DN cannot end with a comma.".into()));
                }
            }
        }
        Ok(Self {
            text: text.into(),
            components,
        })
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn equivalent(&self, other: &Self) -> bool {
        self.components == other.components
    }
    /// Decode the original first naming value for the additive write model.
    /// Equality still uses prepared components; source spelling is retained here.
    pub(crate) fn naming_attribute(&self) -> Attribute {
        let bytes = self.text.as_bytes();
        let mut pos = self.text.find('=').expect("validated DN") + 1;
        let mut value = Vec::new();
        while pos < bytes.len() && bytes[pos] != b',' {
            if bytes[pos] == b'\\' {
                value.push(escaped(bytes, &mut pos, true).expect("validated DN escape"));
            } else {
                value.push(bytes[pos]);
                pos += 1;
            }
        }
        Attribute {
            name: self.components[0].attribute.clone(),
            values: vec![String::from_utf8(value).expect("validated printable ASCII")],
        }
    }
    pub fn within(&self, base: &Self, scope: Scope) -> bool {
        let n = self.components.len();
        let m = base.components.len();
        n >= m
            && self.components[n - m..] == base.components[..]
            && match scope {
                Scope::Base => n == m,
                Scope::One => n == m + 1,
                Scope::Sub => true,
            }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Base,
    One,
    Sub,
}
impl Scope {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "base" => Ok(Self::Base),
            "one" => Ok(Self::One),
            "sub" => Ok(Self::Sub),
            _ => Err(Error::Unsupported(
                "Search scope must be base, one or sub.".into(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Undefined,
}
impl Truth {
    fn negate(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Undefined => Self::Undefined,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Node {
    Equal(String, String),
    Present(String),
    And(Vec<Node>),
    Or(Vec<Node>),
    Not(Box<Node>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter(Node);
impl Filter {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > MAX_FILTER {
            return Err(Error::Limit(
                "A filter may contain at most 4096 bytes.".into(),
            ));
        }
        if !printable(text) {
            return Err(Error::Unsupported(
                "Filters use printable ASCII in this simulator.".into(),
            ));
        }
        let mut p = Parser {
            b: text.as_bytes(),
            pos: 0,
            nodes: 0,
        };
        let node = p.node(1)?;
        if p.pos != p.b.len() {
            return Err(Error::Syntax(
                "Use one complete parenthesized filter without trailing text.".into(),
            ));
        }
        Ok(Self(node))
    }
    pub fn evaluate(&self, entry: &Entry) -> Truth {
        eval(&self.0, entry)
    }
    pub(crate) fn root(&self) -> &Node {
        &self.0
    }
}
struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
    nodes: usize,
}
impl Parser<'_> {
    fn node(&mut self, depth: usize) -> Result<Node> {
        self.nodes += 1;
        if depth > MAX_DEPTH || self.nodes > MAX_NODES {
            return Err(Error::Limit(
                "Filter exceeds 16 levels or 128 expressions.".into(),
            ));
        }
        if self.b.get(self.pos) != Some(&b'(') {
            return Err(Error::Syntax("Start each filter with (.".into()));
        }
        self.pos += 1;
        let kind = self
            .b
            .get(self.pos)
            .copied()
            .ok_or_else(|| Error::Syntax("Finish the filter.".into()))?;
        let node = match kind {
            b'&' | b'|' => {
                self.pos += 1;
                let mut children = Vec::new();
                while self.b.get(self.pos) == Some(&b'(') {
                    children.push(self.node(depth + 1)?);
                }
                if children.is_empty() {
                    return Err(Error::Syntax(
                        "AND/OR needs at least one child filter.".into(),
                    ));
                }
                if kind == b'&' {
                    Node::And(children)
                } else {
                    Node::Or(children)
                }
            }
            b'!' => {
                self.pos += 1;
                Node::Not(Box::new(self.node(depth + 1)?))
            }
            _ => {
                let start = self.pos;
                while let Some(c) = self.b.get(self.pos) {
                    if matches!(c, b'=' | b'>' | b'<' | b'~' | b':' | b')' | b'(') {
                        break;
                    }
                    self.pos += 1;
                }
                if self.b.get(self.pos) != Some(&b'=') {
                    return Err(Error::Unsupported("This subset supports equality/presence, AND, OR and NOT; not ordering, approximate or extensible filters.".into()));
                }
                let a = std::str::from_utf8(&self.b[start..self.pos]).unwrap_or_default();
                if !descriptor(a) {
                    return Err(Error::Unsupported(
                        "Use an attribute descriptor without options or OID aliases.".into(),
                    ));
                }
                let a = a.to_ascii_lowercase();
                self.pos += 1;
                let mut value = Vec::new();
                let mut stars = 0;
                while let Some(&c) = self.b.get(self.pos) {
                    if c == b')' {
                        break;
                    }
                    if c == b'(' {
                        return Err(Error::Syntax(
                            "Escape literal parentheses in assertion values.".into(),
                        ));
                    }
                    if c == b'\\' {
                        value.push(escaped(self.b, &mut self.pos, false)?);
                    } else {
                        if c == b'*' {
                            stars += 1;
                        }
                        value.push(c);
                        self.pos += 1;
                    }
                    if value.len() > 256 {
                        return Err(Error::Limit("An assertion is limited to 256 bytes.".into()));
                    }
                }
                if stars == 1 && value == b"*" {
                    Node::Present(a)
                } else if stars > 0 {
                    return Err(Error::Unsupported(
                        "Substring filters are not implemented yet; \\2a means a literal asterisk."
                            .into(),
                    ));
                } else {
                    if value.is_empty() {
                        return Err(Error::Unsupported(
                            "Empty equality assertions are outside this first subset.".into(),
                        ));
                    }
                    Node::Equal(
                        a,
                        String::from_utf8(value)
                            .map_err(|_| Error::Unsupported("ASCII filter values only.".into()))?,
                    )
                }
            }
        };
        if self.b.get(self.pos) != Some(&b')') {
            return Err(Error::Syntax(
                "Close each filter once; NOT takes exactly one filter.".into(),
            ));
        }
        self.pos += 1;
        Ok(node)
    }
}
fn eval(node: &Node, e: &Entry) -> Truth {
    use Truth::*;
    match node {
        Node::Present(a) => {
            if !known(a) {
                Undefined
            } else if e.attributes.iter().any(|x| x.name == *a) {
                True
            } else {
                False
            }
        }
        Node::Equal(a, v) => {
            if !known(a) || a == "objectclass" && !class(v) {
                return Undefined;
            }
            let target = prepared(v);
            if e.attributes
                .iter()
                .any(|x| x.name == *a && x.values.iter().any(|s| prepared(s) == target))
            {
                True
            } else {
                False
            }
        }
        Node::Not(n) => eval(n, e).negate(),
        Node::And(children) => {
            let mut undefined = false;
            for c in children {
                match eval(c, e) {
                    False => return False,
                    Undefined => undefined = true,
                    True => (),
                }
            }
            if undefined { Undefined } else { True }
        }
        Node::Or(children) => {
            let mut undefined = false;
            for c in children {
                match eval(c, e) {
                    True => return True,
                    Undefined => undefined = true,
                    False => (),
                }
            }
            if undefined { Undefined } else { False }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    #[serde(deserialize_with = "attribute_values")]
    pub values: Vec<String>,
}
fn attribute_values<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Value(#[serde(deserialize_with = "crate::cron::text::<_,256>")] String);
    crate::cron_workspace::sequence::<D, Value, 16>(d).map(|v| v.into_iter().map(|x| x.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntrySource {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub dn: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Attribute,16>")]
    pub attributes: Vec<Attribute>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub suffix: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,EntrySource,128>")]
    pub entries: Vec<EntrySource>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    dn: Dn,
    attributes: Vec<Attribute>,
}
impl Entry {
    pub fn dn(&self) -> &Dn {
        &self.dn
    }
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directory {
    suffix: Dn,
    entries: Vec<Entry>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchResult<'a> {
    NoSuchObject,
    Entries(Vec<&'a Entry>),
}
impl Directory {
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_JSON {
            return Err(Error::Limit("Directory fixture exceeds 512 KiB.".into()));
        }
        let s: Source = serde_json::from_str(raw).map_err(|e| {
            Error::Fixture(format!("Malformed or unsupported bounded fixture: {e}"))
        })?;
        Self::new(&s)
    }
    pub fn new(source: &Source) -> Result<Self> {
        if source.version != 1 || source.entries.is_empty() || source.entries.len() > MAX_ENTRIES {
            return Err(Error::Fixture("Supply version 1 and 1–128 entries.".into()));
        }
        // Check typed shape/string bounds before serialization or cloning any body.
        if source.suffix.len() > 1024 {
            return Err(Error::Limit("DN exceeds 1024 bytes.".into()));
        }
        for entry in &source.entries {
            if entry.dn.len() > 1024 || entry.attributes.len() > 16 {
                return Err(Error::Limit(
                    "Directory entry exceeds its naming/attribute bounds.".into(),
                ));
            }
            for attr in &entry.attributes {
                if attr.name.len() > 32
                    || attr.values.len() > 16
                    || attr.values.iter().any(|v| v.len() > 256)
                {
                    return Err(Error::Limit("Attribute exceeds name/value bounds.".into()));
                }
            }
        }
        // Count encoded bytes without materializing an arbitrarily large JSON copy.
        struct Counter(usize);
        impl std::io::Write for Counter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if bytes.len() > MAX_JSON - self.0 {
                    return Err(std::io::Error::other("fixture limit"));
                }
                self.0 += bytes.len();
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        serde_json::to_writer(Counter(0), source)
            .map_err(|_| Error::Limit("Directory fixture exceeds 512 KiB.".into()))?;
        let suffix = Dn::parse(&source.suffix)?;
        let mut entries: Vec<Entry> = Vec::new();
        for src in &source.entries {
            let dn = Dn::parse(&src.dn)?;
            if !dn.within(&suffix, Scope::Sub) {
                return Err(Error::Fixture(
                    "Every entry must lie within the supplied naming context.".into(),
                ));
            }
            if entries.iter().any(|e| e.dn.equivalent(&dn)) {
                return Err(Error::Fixture(
                    "Two entries name the same DN under declared matching rules.".into(),
                ));
            }
            if src.attributes.is_empty() || src.attributes.len() > 16 {
                return Err(Error::Fixture("Each entry needs 1–16 attributes.".into()));
            }
            let mut attributes = Vec::new();
            let mut names = BTreeSet::new();
            for attr in &src.attributes {
                let name = attr.name.to_ascii_lowercase();
                if !descriptor(&attr.name)
                    || !known(&name)
                    || !names.insert(name.clone())
                    || attr.values.is_empty()
                    || attr.values.len() > 16
                {
                    return Err(Error::Fixture(
                        "Use distinct supported attributes with 1–16 values each.".into(),
                    ));
                }
                let mut seen = BTreeSet::new();
                for v in &attr.values {
                    if !value_valid(&name, v) || !seen.insert(prepared(v)) {
                        return Err(Error::Fixture("Attribute values must be distinct supported printable ASCII values of at most 256 bytes.".into()));
                    }
                }
                attributes.push(Attribute {
                    name,
                    values: attr.values.clone(),
                });
            }
            if !names.contains("objectclass") {
                return Err(Error::Fixture(
                    "Each supplied entry needs objectClass.".into(),
                ));
            }
            let first = &dn.components[0];
            if !attributes.iter().any(|a| {
                a.name == first.attribute && a.values.iter().any(|v| prepared(v) == first.value)
            }) {
                return Err(Error::Fixture("The naming attribute's actual value must match the entry's first relative name.".into()));
            }
            entries.push(Entry { dn, attributes });
        }
        if !entries.iter().any(|e| e.dn.equivalent(&suffix)) {
            return Err(Error::Fixture(
                "Include the naming-context root entry.".into(),
            ));
        }
        for e in &entries {
            if !e.dn.equivalent(&suffix)
                && !entries
                    .iter()
                    .any(|parent| parent.dn.components == e.dn.components[1..])
            {
                return Err(Error::Fixture(
                    "Include every parent within the supplied naming context.".into(),
                ));
            }
        }
        Ok(Self { suffix, entries })
    }
    pub fn suffix(&self) -> &Dn {
        &self.suffix
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn search(&self, base: &Dn, scope: Scope, filter: &Filter) -> SearchResult<'_> {
        if !self.entries.iter().any(|e| e.dn.equivalent(base)) {
            return SearchResult::NoSuchObject;
        }
        SearchResult::Entries(
            self.entries
                .iter()
                .filter(|e| e.dn.within(base, scope) && filter.evaluate(e) == Truth::True)
                .collect(),
        )
    }
}
