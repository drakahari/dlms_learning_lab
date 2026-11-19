//! In-memory directory requests with a deliberately small writable schema.
//! A source-held grant is an already-established simulator fact, never a bind.
use crate::{
    ldap_ldif::{Change, Document, Edit, EditKind},
    ldap_query::{self as q, Attribute, Dn, EntrySource, Scope, Source},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    NoSuchAttribute = 16,
    AttributeOrValueExists = 20,
    NoSuchObject = 32,
    InsufficientAccessRights = 50,
    ObjectClassViolation = 65,
    NotAllowedOnNonLeaf = 66,
    EntryAlreadyExists = 68,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported(String),
    Limit(String),
    Invalid(String),
    Result { code: Code, explanation: String },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(s) => write!(f, "Unsupported: {s}"),
            Self::Limit(s) => write!(f, "Limit: {s}"),
            Self::Invalid(s) => write!(f, "Invalid request: {s}"),
            Self::Result { code, explanation } => {
                write!(f, "{code:?} ({}): {explanation}", *code as u8)
            }
        }
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn fail(code: Code, s: &str) -> Error {
    Error::Result {
        code,
        explanation: s.into(),
    }
}
fn invalid(s: &str) -> Error {
    Error::Invalid(s.into())
}
fn unsupported(s: &str) -> Error {
    Error::Unsupported(s.into())
}
fn bounded(s: &str) -> Error {
    Error::Limit(s.into())
}
fn prepared(s: &str) -> String {
    s.split(' ')
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}
fn values<'a>(e: &'a EntrySource, name: &str) -> &'a [String] {
    e.attributes
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(name))
        .map_or(&[], |a| a.values.as_slice())
}
fn has_class(e: &EntrySource, name: &str) -> bool {
    values(e, "objectclass")
        .iter()
        .any(|v| v.eq_ignore_ascii_case(name))
}
fn attribute_shape(a: &Attribute, allow_empty: bool) -> Result<()> {
    if a.name.len() > 32 || a.values.len() > 16 || a.values.iter().any(|v| v.len() > 256) {
        return Err(bounded(
            "An attribute allows32name bytes,16values and256bytes per value.",
        ));
    }
    if a.name.is_empty()
        || !a.name.as_bytes()[0].is_ascii_alphabetic()
        || !a
            .name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        || !allow_empty && a.values.is_empty()
        || a.values
            .iter()
            .any(|v| v.is_empty() || !v.bytes().all(|c| (32..=126).contains(&c)))
    {
        return Err(invalid(
            "Use supported attribute names and nonempty printable ASCII values.",
        ));
    }
    for (i, v) in a.values.iter().enumerate() {
        if a.values[..i].iter().any(|old| prepared(old) == prepared(v)) {
            return Err(invalid(
                "The request repeats an equivalent attribute value.",
            ));
        }
    }
    Ok(())
}
fn entry_shape(e: &EntrySource) -> Result<()> {
    if e.dn.len() > 1024 || e.attributes.len() > 16 {
        return Err(bounded("Entry exceeds DN/attribute bounds."));
    }
    Dn::parse(&e.dn).map_err(|e| invalid(&e.to_string()))?;
    if e.attributes.is_empty() {
        return Err(invalid("An entry needs attributes."));
    }
    for (i, a) in e.attributes.iter().enumerate() {
        attribute_shape(a, false)?;
        if e.attributes[..i]
            .iter()
            .any(|old| old.name.eq_ignore_ascii_case(&a.name))
        {
            return Err(invalid(
                "Collect repeated attribute lines into one value list.",
            ));
        }
    }
    Ok(())
}
// This is a restricted controlling schema, not an LDAP schema parser. SUP values
// are expanded only in the new write context; old read-only fixtures are untouched.
fn schema(e: &EntrySource) -> Result<EntrySource> {
    entry_shape(e)?;
    let raw = values(e, "objectclass");
    if raw.is_empty() {
        return Err(fail(
            Code::ObjectClassViolation,
            "The entry needs objectClass.",
        ));
    }
    let mut classes: Vec<&str> = Vec::new();
    let mut structural = None;
    for v in raw {
        let (family, chain): (&str, &[&str]) = match v.to_ascii_lowercase().as_str() {
            "top" => ("top", &["top"]),
            "domain" => ("domain", &["top", "domain"]),
            "organizationalunit" => ("unit", &["top", "organizationalunit"]),
            "person" => ("person", &["top", "person"]),
            "organizationalperson" => ("person", &["top", "person", "organizationalperson"]),
            "inetorgperson" => (
                "person",
                &["top", "person", "organizationalperson", "inetorgperson"],
            ),
            _ => {
                return Err(unsupported(
                    "Only domain, organizationalUnit and the person/organizationalPerson/inetOrgPerson chain are modeled.",
                ));
            }
        };
        if family != "top" {
            if structural.is_some_and(|old| old != family) {
                return Err(fail(
                    Code::ObjectClassViolation,
                    "Choose one compatible structural object-class chain.",
                ));
            }
            structural = Some(family);
        }
        for c in chain {
            if !classes.contains(c) {
                classes.push(c);
            }
        }
    }
    let Some(family) = structural else {
        return Err(fail(
            Code::ObjectClassViolation,
            "top alone is not a structural class.",
        ));
    };
    let required: &[&str] = match family {
        "domain" => &["dc"],
        "unit" => &["ou"],
        _ => &["cn", "sn"],
    };
    for name in required {
        if values(e, name).is_empty() {
            return Err(fail(
                Code::ObjectClassViolation,
                "A required naming or person attribute is missing (domain:dc; unit:ou; person:cn and sn).",
            ));
        }
    }
    for a in &e.attributes {
        let name = a.name.to_ascii_lowercase();
        let allowed = name == "objectclass"
            || name == "description"
            || match family {
                "domain" => name == "dc",
                "unit" => name == "ou",
                _ => {
                    matches!(name.as_str(), "cn" | "sn")
                        || classes.contains(&"inetorgperson")
                            && matches!(name.as_str(), "uid" | "mail")
                }
            };
        if !allowed {
            return Err(fail(
                Code::ObjectClassViolation,
                "An attribute is not allowed by this entry's supported object-class chain.",
            ));
        }
        if name == "dc" {
            if a.values.len() != 1 {
                return Err(fail(
                    Code::ObjectClassViolation,
                    "dc is single-valued in this schema.",
                ));
            }
            let label = a.values[0].as_bytes();
            if label.len() > 63
                || !label[0].is_ascii_alphanumeric()
                || !label[label.len() - 1].is_ascii_alphanumeric()
                || !label
                    .iter()
                    .all(|c| c.is_ascii_alphanumeric() || *c == b'-')
            {
                return Err(invalid(
                    "dc needs a1–63character domain label with letters/digits and internal hyphens.",
                ));
            }
        }
    }
    let mut out = e.clone();
    for a in &mut out.attributes {
        a.name.make_ascii_lowercase();
        if a.name == "objectclass" {
            a.values = classes.iter().map(|s| s.to_string()).collect();
        }
    }
    Ok(out)
}

/// Source-held delegated rights. The learner cannot create grants through LDIF.
/// This is neither an authentication exchange nor OpenLDAP ACL syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    principal: Dn,
    subtree: Dn,
    add: bool,
    delete: bool,
    attributes: Vec<String>,
}
impl Grant {
    pub fn new(
        principal: Dn,
        subtree: Dn,
        add: bool,
        delete: bool,
        attributes: &[&str],
    ) -> Result<Self> {
        if attributes.len() > 2 {
            return Err(bounded("A grant has at most two writable attributes."));
        }
        let mut attrs = Vec::new();
        for a in attributes {
            if !matches!(*a, "mail" | "description") || attrs.iter().any(|old| old == a) {
                return Err(unsupported(
                    "Grants name distinct mail/description permissions only.",
                ));
            }
            attrs.push(a.to_string());
        }
        Ok(Self {
            principal,
            subtree,
            add,
            delete,
            attributes: attrs,
        })
    }
    pub fn principal(&self) -> &Dn {
        &self.principal
    }
    pub fn subtree(&self) -> &Dn {
        &self.subtree
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Added,
    Deleted,
    Modified,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub dn: String,
    pub operation: Operation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub results: Vec<Result<Applied>>,
    pub unattempted: usize,
}
impl Batch {
    pub fn succeeded(&self) -> bool {
        self.unattempted == 0
            && !self.results.is_empty()
            && self.results.iter().all(std::result::Result::is_ok)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directory {
    source: Source,
    read: q::Directory,
}
impl Directory {
    pub fn new(source: &Source) -> Result<Self> {
        // Reuse the read-only model's typed/aggregate guards BEFORE cloning.
        q::Directory::new(source).map_err(|e| invalid(&e.to_string()))?;
        let entries = source
            .entries
            .iter()
            .map(schema)
            .collect::<Result<Vec<_>>>()?;
        let source = Source {
            version: 1,
            suffix: source.suffix.clone(),
            entries,
        };
        let read = q::Directory::new(&source).map_err(|e| invalid(&e.to_string()))?;
        Ok(Self { source, read })
    }
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub fn read(&self) -> &q::Directory {
        &self.read
    }
    fn index(&self, dn: &Dn) -> Option<usize> {
        self.read
            .entries()
            .iter()
            .position(|e| e.dn().equivalent(dn))
    }
    fn authorize(&self, grant: &Grant, dn: &Dn, change: &Change) -> Result<()> {
        let valid_identity = self
            .index(&grant.principal)
            .is_some_and(|i| has_class(&self.source.entries[i], "person"));
        let valid_base = self.index(&grant.subtree).is_some();
        let allowed = match change {
            Change::Add(_) => grant.add,
            Change::Delete(_) => grant.delete,
            Change::Modify { edits, .. } => edits.iter().all(|e| {
                grant
                    .attributes
                    .iter()
                    .any(|a| a.eq_ignore_ascii_case(&e.attribute.name))
            }),
        };
        if !valid_identity || !valid_base || !dn.within(&grant.subtree, Scope::Sub) || !allowed {
            return Err(fail(
                Code::InsufficientAccessRights,
                "The supplied identity has no delegated permission for this request and subtree.",
            ));
        }
        Ok(())
    }
    pub fn apply(&mut self, change: &Change, grant: &Grant) -> Result<Applied> {
        let dn = match change {
            Change::Add(e) => {
                entry_shape(e)?;
                Dn::parse(&e.dn).map_err(|e| invalid(&e.to_string()))?
            }
            Change::Delete(dn) => dn.clone(),
            Change::Modify { dn, edits } => {
                if edits.is_empty() || edits.len() > 16 {
                    return Err(bounded("A Modify request needs1–16ordered edits."));
                }
                for edit in edits {
                    attribute_shape(&edit.attribute, edit.kind != EditKind::Add)?;
                    if !matches!(
                        edit.attribute.name.to_ascii_lowercase().as_str(),
                        "mail" | "description"
                    ) {
                        return Err(unsupported(
                            "Only mail and description modifications are supported; naming/class/password changes are not.",
                        ));
                    }
                }
                dn.clone()
            }
        };
        self.authorize(grant, &dn, change)?;
        if dn.equivalent(self.read.suffix()) {
            return Err(unsupported(
                "Naming-context root management is outside this workspace.",
            ));
        }
        let index = self.index(&dn);
        let mut staged = self.source.clone();
        let operation = match change {
            Change::Add(e) => {
                if index.is_some() {
                    return Err(fail(
                        Code::EntryAlreadyExists,
                        "This distinguished name already identifies an entry; use Modify for its fields.",
                    ));
                }
                if self.read.entries().len() >= q::MAX_ENTRIES {
                    return Err(bounded("Directory is at its128entry limit."));
                }
                if !self
                    .read
                    .entries()
                    .iter()
                    .any(|p| dn.within(p.dn(), Scope::One))
                {
                    return Err(fail(
                        Code::NoSuchObject,
                        "The new entry's immediate parent does not exist in the supplied directory.",
                    ));
                }
                // RFC4511 Add combines RDN values with the supplied attributes.
                // The read-only fixture constructor remains deliberately strict.
                let mut e = e.clone();
                let naming = dn.naming_attribute();
                if let Some(a) = e
                    .attributes
                    .iter_mut()
                    .find(|a| a.name.eq_ignore_ascii_case(&naming.name))
                {
                    let value = &naming.values[0];
                    if !a.values.iter().any(|v| prepared(v) == prepared(value)) {
                        if a.values.len() >= 16 {
                            return Err(bounded(
                                "Naming value exceeds the attribute's16value limit.",
                            ));
                        }
                        a.values.push(value.clone());
                    }
                } else {
                    if e.attributes.len() >= 16 {
                        return Err(bounded(
                            "Naming attribute exceeds the entry's16attribute limit.",
                        ));
                    }
                    e.attributes.push(naming);
                }
                let e = schema(&e)?;
                if !has_class(&e, "person") {
                    return Err(unsupported(
                        "Adding containers is outside this person-entry workspace.",
                    ));
                }
                staged.entries.push(e);
                Operation::Added
            }
            Change::Delete(_) => {
                let i = index.ok_or_else(|| {
                    fail(Code::NoSuchObject, "No entry has this distinguished name.")
                })?;
                if !has_class(&staged.entries[i], "person") {
                    return Err(unsupported(
                        "Deleting containers is outside this person-entry workspace.",
                    ));
                }
                if self
                    .read
                    .entries()
                    .iter()
                    .any(|e| !e.dn().equivalent(&dn) && e.dn().within(&dn, Scope::Sub))
                {
                    return Err(fail(
                        Code::NotAllowedOnNonLeaf,
                        "The entry has children; this operation cannot recursively remove a branch.",
                    ));
                }
                staged.entries.remove(i);
                Operation::Deleted
            }
            Change::Modify { edits, .. } => {
                let i = index.ok_or_else(|| {
                    fail(Code::NoSuchObject, "No entry has this distinguished name.")
                })?;
                if !has_class(&staged.entries[i], "person") {
                    return Err(unsupported(
                        "Only person-entry mail/description changes are modeled.",
                    ));
                }
                for edit in edits {
                    modify(&mut staged.entries[i], edit)?;
                }
                staged.entries[i] = schema(&staged.entries[i])?;
                Operation::Modified
            }
        };
        // Validate the whole resulting state before committing any request.
        let read = q::Directory::new(&staged).map_err(|e| invalid(&e.to_string()))?;
        self.source = staged;
        self.read = read;
        Ok(Applied {
            dn: dn.text().into(),
            operation,
        })
    }
    /// Stop on the first failed request. Prior successful requests stay committed.
    pub fn apply_document(&mut self, document: &Document, grant: &Grant) -> Result<Batch> {
        let len = match document {
            Document::Content(v) => v.len(),
            Document::Changes(v) => v.len(),
        };
        if len == 0 || len > crate::ldap_ldif::MAX_RECORDS {
            return Err(bounded("A document needs1–16records."));
        }
        let mut results = Vec::new();
        for i in 0..len {
            let result = match document {
                Document::Content(v) => {
                    // Bound a hand-constructed entry before creating its request.
                    match entry_shape(&v[i]) {
                        Ok(()) => self.apply(&Change::Add(v[i].clone()), grant),
                        Err(e) => Err(e),
                    }
                }
                Document::Changes(v) => self.apply(&v[i], grant),
            };
            let failed = result.is_err();
            results.push(result);
            if failed {
                break;
            }
        }
        Ok(Batch {
            unattempted: len - results.len(),
            results,
        })
    }
}
fn modify(entry: &mut EntrySource, edit: &Edit) -> Result<()> {
    let name = edit.attribute.name.to_ascii_lowercase();
    let index = entry.attributes.iter().position(|a| a.name == name);
    let input = &edit.attribute.values;
    match edit.kind {
        EditKind::Add => {
            if let Some(i) = index {
                let old = &mut entry.attributes[i].values;
                if input
                    .iter()
                    .any(|v| old.iter().any(|s| prepared(s) == prepared(v)))
                {
                    return Err(fail(
                        Code::AttributeOrValueExists,
                        "An equivalent value already exists. Add only a new value, or use Replace to change the set.",
                    ));
                }
                if old.len() + input.len() > 16 {
                    return Err(bounded("An attribute may have at most16values."));
                }
                old.extend(input.iter().cloned());
            } else {
                if entry.attributes.len() >= 16 {
                    return Err(bounded("An entry may have at most16attributes."));
                }
                entry.attributes.push(Attribute {
                    name,
                    values: input.clone(),
                });
            }
        }
        EditKind::Delete => {
            let i=index.ok_or_else(||fail(Code::NoSuchAttribute,"The attribute is absent. Delete needs an existing attribute/value; empty Replace can remove one optionally."))?;
            if input.is_empty() {
                entry.attributes.remove(i);
            } else {
                let old = &mut entry.attributes[i].values;
                if input
                    .iter()
                    .any(|v| !old.iter().any(|s| prepared(s) == prepared(v)))
                {
                    return Err(fail(
                        Code::NoSuchAttribute,
                        "A requested value is absent; no part of this Modify request has been applied.",
                    ));
                }
                old.retain(|s| !input.iter().any(|v| prepared(s) == prepared(v)));
                if old.is_empty() {
                    entry.attributes.remove(i);
                }
            }
        }
        EditKind::Replace => {
            if input.is_empty() {
                if let Some(i) = index {
                    entry.attributes.remove(i);
                }
            } else if let Some(i) = index {
                entry.attributes[i].values = input.clone();
            } else {
                if entry.attributes.len() >= 16 {
                    return Err(bounded("An entry may have at most16attributes."));
                }
                entry.attributes.push(Attribute {
                    name,
                    values: input.clone(),
                });
            }
        }
    }
    Ok(())
}
