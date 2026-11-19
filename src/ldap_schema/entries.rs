//! Actual schema-governed entries, kept separate from the fixed legacy contact
//! directory. Rejected requests never mutate the accepted entry collection.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_ENTRIES: usize = 32;
pub const MAX_ENTRY_BYTES: usize = 16 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub name: String,
    #[serde(deserialize_with = "value_list")]
    pub values: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntrySource {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub dn: String,
    #[serde(deserialize_with = "class_list")]
    pub classes: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Field,32>")]
    pub fields: Vec<Field>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    dn: crate::ldap_query::Dn,
    structural: String,
    classes: BTreeSet<String>,
    fields: BTreeMap<String, Vec<String>>,
}
impl Entry {
    pub fn dn(&self) -> &crate::ldap_query::Dn {
        &self.dn
    }
    pub fn structural_class(&self) -> &str {
        &self.structural
    }
    pub fn classes(&self) -> &BTreeSet<String> {
        &self.classes
    }
    pub fn fields(&self) -> &BTreeMap<String, Vec<String>> {
        &self.fields
    }
}
fn failure(category: Category, subject: Option<&str>, text: &'static str) -> Diagnostic {
    let mut error = super::registry::issue(category, text);
    error.subject = subject.map(str::to_owned);
    error
}
fn bound(source: &EntrySource) -> Result<()> {
    if source.dn.len() > 1024 || source.classes.len() > 16 || source.fields.len() > 32 {
        return Err(failure(
            Category::Limit,
            None,
            "An entry allows a 1024-byte DN, 16 class references and 32 fields.",
        ));
    }
    let mut bytes = source.dn.len();
    for name in &source.classes {
        if name.len() > 128 {
            return Err(failure(
                Category::Limit,
                None,
                "A class reference allows 128 bytes.",
            ));
        }
        bytes += name.len();
    }
    for f in &source.fields {
        if f.name.len() > 128 || f.values.len() > 16 || f.values.iter().any(|v| v.len() > MAX_VALUE)
        {
            return Err(failure(
                Category::Limit,
                None,
                "Each field allows a 128-byte name and 16 values of at most 256 bytes.",
            ));
        }
        bytes += f.name.len() + f.values.iter().map(String::len).sum::<usize>();
    }
    if bytes > MAX_ENTRY_BYTES {
        return Err(failure(
            Category::Limit,
            None,
            "The combined entry data exceeds the 16 KiB simulation limit.",
        ));
    }
    Ok(())
}
impl EntrySource {
    pub fn validate_shape(&self) -> Result<()> {
        bound(self)
    }
}
impl Registry {
    pub fn validate_entry(&self, source: &EntrySource) -> Result<Entry> {
        bound(source)?;
        let dn=crate::ldap_query::Dn::parse(&source.dn).map_err(|e| {
            let category=match e {crate::ldap_query::Error::Limit(_)=>Category::Limit,crate::ldap_query::Error::Unsupported(_)=>Category::Unsupported,_=>Category::Syntax};
            failure(category,Some("dn"),"Use a supported printable-ASCII, single-valued cn/ou/uid/dc distinguished name; check escapes and separators.")
        })?;
        if source.classes.is_empty() {
            return Err(failure(
                Category::Violation,
                Some("objectClass"),
                "An entry needs a structural object class, not only abstract classes.",
            ));
        }
        let mut classes = BTreeSet::new();
        let mut provided = BTreeSet::new();
        let mut structural: Option<&ResolvedClass> = None;
        for name in &source.classes {
            if !numeric_oid(name) && !super::descriptor(name) {
                return Err(failure(
                    Category::Syntax,
                    Some("objectClass"),
                    "Use a short class name or canonical numeric OID.",
                ));
            }
            let c = self.class(name).ok_or_else(|| {
                failure(
                    Category::Violation,
                    Some(name),
                    "This object class is not registered in the supplied schema.",
                )
            })?;
            if !provided.insert(c.definition.oid.clone()) {
                return Err(failure(
                    Category::Violation,
                    Some(name),
                    "The same object class was supplied twice through a name, alias or OID.",
                ));
            }
            classes.extend(c.ancestors.iter().cloned());
            if c.definition.kind == ClassKind::Structural {
                if let Some(old) = structural {
                    if c.ancestors.contains(&old.definition.oid) {
                        structural = Some(c);
                    } else if !old.ancestors.contains(&c.definition.oid) {
                        return Err(failure(
                            Category::Violation,
                            Some(name),
                            "These structural classes are unrelated. An entry must follow one structural inheritance chain.",
                        ));
                    }
                } else {
                    structural = Some(c);
                }
            }
        }
        let structural=structural.ok_or_else(||failure(Category::Violation,Some("objectClass"),"Abstract classes describe shared rules; add a supported structural class for the actual entry."))?;
        if provided
            .iter()
            .any(|oid| !structural.ancestors.contains(oid))
        {
            return Err(failure(
                Category::Violation,
                Some("objectClass"),
                "An abstract class must be inherited by this entry's structural class.",
            ));
        }
        let mut fields = BTreeMap::new();
        let mut satisfied = BTreeSet::new();
        for f in &source.fields {
            if f.name.contains(';') {
                return Err(failure(
                    Category::Unsupported,
                    Some(&f.name),
                    "Attribute options, language tags and binary transfer forms are outside this entry model.",
                ));
            }
            if !numeric_oid(&f.name) && !super::descriptor(&f.name) {
                return Err(failure(
                    Category::Syntax,
                    Some("attribute"),
                    "Use a short attribute name or canonical numeric OID.",
                ));
            }
            if f.name.eq_ignore_ascii_case("objectclass") || f.name == "2.5.4.0" {
                return Err(failure(
                    Category::Violation,
                    Some("objectClass"),
                    "Supply object classes in the dedicated class list, not a second ordinary field.",
                ));
            }
            let a = self.attribute(&f.name).ok_or_else(|| {
                failure(
                    Category::Violation,
                    Some(&f.name),
                    "This attribute type is not registered in the supplied schema.",
                )
            })?;
            if fields.contains_key(&a.definition.oid) {
                return Err(failure(
                    Category::Violation,
                    Some(&f.name),
                    "This attribute was repeated through a name, alias or numeric OID. Combine its values in one field.",
                ));
            }
            if !a
                .ancestors
                .iter()
                .any(|oid| structural.allowed.contains(oid))
            {
                return Err(failure(
                    Category::Violation,
                    Some(&f.name),
                    "The structural class does not allow this attribute through MUST or MAY.",
                ));
            }
            if f.values.is_empty() {
                return Err(failure(
                    Category::Violation,
                    Some(&f.name),
                    "A present attribute needs at least one value; omit an absent optional field.",
                ));
            }
            if a.single_value && f.values.len() != 1 {
                return Err(failure(
                    Category::Violation,
                    Some(&f.name),
                    "SINGLE-VALUE permits one value for this attribute.",
                ));
            }
            if a.equality.is_none() {
                return Err(failure(
                    Category::Unsupported,
                    Some(&f.name),
                    "Entry updates without an equality rule are outside this model; no byte-equality rule is invented.",
                ));
            }
            let mut keys = BTreeSet::new();
            for v in &f.values {
                a.validate_value(v).map_err(|mut e| {
                    e.subject = Some(f.name.clone());
                    e
                })?;
                if !keys.insert(a.equality.expect("checked equality").key(v)) {
                    return Err(failure(
                        Category::Violation,
                        Some(&f.name),
                        "Two values are equal under this attribute's equality rule; LDAP attribute values must be distinct.",
                    ));
                }
            }
            satisfied.extend(a.ancestors.iter().cloned());
            fields.insert(a.definition.oid.clone(), f.values.clone());
        }
        for oid in &structural.required {
            if !satisfied.contains(oid) {
                let a = self.attribute(oid).expect("registered required attribute");
                let label = a.definition.names.first().unwrap_or(oid);
                return Err(failure(
                    Category::Violation,
                    Some(label),
                    "A MUST attribute is missing. Include a nonempty field of this type or an allowed subtype.",
                ));
            }
        }
        let naming = dn.naming_attribute();
        let a = self.attribute(&naming.name).ok_or_else(|| {
            failure(
                Category::Unsupported,
                Some(&naming.name),
                "This naming attribute is not supported by the supplied schema registry.",
            )
        })?;
        let mut named = false;
        for (oid, values) in &fields {
            let field = self.attribute(oid).expect("registered entry attribute");
            if field.ancestors.contains(&a.definition.oid) {
                for value in values {
                    if a.equivalent(value, &naming.values[0])? {
                        named = true;
                    }
                }
            }
        }
        if !named {
            return Err(failure(
                Category::Violation,
                Some(&naming.name),
                "The entry must contain its naming value. Its DN and stored naming attribute currently disagree.",
            ));
        }
        Ok(Entry {
            dn,
            structural: structural.definition.oid.clone(),
            classes,
            fields,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directory {
    registry: Registry,
    entries: Vec<Entry>,
    pub(super) modules: Vec<super::Module>,
}
impl Directory {
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            entries: Vec::new(),
            modules: Vec::new(),
        }
    }
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn source(&self) -> Vec<EntrySource> {
        self.entries
            .iter()
            .map(|entry| EntrySource {
                dn: entry.dn.text().into(),
                classes: entry
                    .classes
                    .iter()
                    .map(|oid| {
                        self.registry
                            .class(oid)
                            .and_then(|c| c.definition.names.first())
                            .unwrap_or(oid)
                            .clone()
                    })
                    .collect(),
                fields: entry
                    .fields
                    .iter()
                    .map(|(oid, values)| Field {
                        name: self
                            .registry
                            .attribute(oid)
                            .and_then(|a| a.definition.names.first())
                            .unwrap_or(oid)
                            .clone(),
                        values: values.clone(),
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn register(&mut self, attributes: &[String], classes: &[String]) -> Result<()> {
        self.registry.register(attributes, classes)
    }
    pub fn add(&mut self, source: &EntrySource) -> Result<()> {
        if self.entries.len() >= MAX_ENTRIES {
            return Err(failure(
                Category::Limit,
                None,
                "The simulated directory allows 32 entries.",
            ));
        }
        let entry = self.registry.validate_entry(source)?;
        if self.entries.iter().any(|old| old.dn.equivalent(&entry.dn)) {
            return Err(failure(
                Category::Conflict,
                Some("dn"),
                "This entry already exists; Add cannot replace it.",
            ));
        }
        self.entries.push(entry);
        Ok(())
    }
    pub fn replace(&mut self, source: &EntrySource) -> Result<()> {
        let next = self.registry.validate_entry(source)?;
        let old = self
            .entries
            .iter_mut()
            .find(|e| e.dn.equivalent(&next.dn))
            .ok_or_else(|| {
                failure(
                    Category::Conflict,
                    Some("dn"),
                    "This entry does not exist; replacement cannot create it.",
                )
            })?;
        if old.structural != next.structural {
            return Err(failure(
                Category::Violation,
                Some("objectClass"),
                "An existing entry's structural object class cannot be changed by this update.",
            ));
        }
        *old = next;
        Ok(())
    }
}

fn value_list<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,256>")] String);
    crate::cron_workspace::sequence::<_, Text, 16>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
fn class_list<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,128>")] String);
    crate::cron_workspace::sequence::<_, Text, 16>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
