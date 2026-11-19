//! Atomic schema registration with an explicit, small built-in vocabulary.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_DEFINITIONS: usize = 64;
pub const MAX_BATCH: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matching {
    CaseIgnore,
    CaseExact,
    CaseIgnoreIa5,
    CaseExactIa5,
    Integer,
}
impl Matching {
    fn parse(raw: &str) -> Result<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "caseignorematch" | "2.5.13.2" => Ok(Self::CaseIgnore),
            "caseexactmatch" | "2.5.13.5" => Ok(Self::CaseExact),
            "caseignoreia5match" | "1.3.6.1.4.1.1466.109.114.2" => Ok(Self::CaseIgnoreIa5),
            "caseexactia5match" | "1.3.6.1.4.1.1466.109.114.1" => Ok(Self::CaseExactIa5),
            "integermatch" | "2.5.13.14" => Ok(Self::Integer),
            _ => Err(issue(
                Category::Unsupported,
                "This registry supports caseIgnore/ExactMatch, caseIgnore/ExactIA5Match and integerMatch only.",
            )),
        }
    }
    fn compatible(self, syntax: Syntax) -> bool {
        matches!(
            (self, syntax),
            (Self::CaseIgnore | Self::CaseExact, Syntax::DirectoryString)
                | (Self::CaseIgnoreIa5 | Self::CaseExactIa5, Syntax::Ia5String)
                | (Self::Integer, Syntax::Integer)
        )
    }
    pub(crate) fn key(self, value: &str) -> String {
        if self == Self::Integer {
            return value.into();
        }
        let spaced = value
            .split(' ')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if matches!(self, Self::CaseIgnore | Self::CaseIgnoreIa5) {
            spaced.to_ascii_lowercase()
        } else {
            spaced
        }
    }
}
pub(super) fn issue(category: Category, message: &'static str) -> Diagnostic {
    Diagnostic {
        category,
        subject: None,
        line: 0,
        column: 0,
        message: message.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedAttribute {
    pub definition: AttributeType,
    pub syntax: Syntax,
    pub equality: Option<Matching>,
    pub single_value: bool,
    pub(crate) ancestors: BTreeSet<String>,
}
impl ResolvedAttribute {
    pub fn validate_value(&self, value: &str) -> Result<()> {
        if value.len() > MAX_VALUE {
            return Err(issue(
                Category::Limit,
                "A simulated attribute value allows at most 256 bytes.",
            ));
        }
        if !value.bytes().all(|b| (32..=126).contains(&b)) {
            return Err(issue(
                Category::Unsupported,
                "Only printable ASCII values are modeled; Unicode and controls are not approximated.",
            ));
        }
        match self.syntax {
            Syntax::DirectoryString if value.is_empty() => Err(issue(
                Category::Violation,
                "Directory String values must contain at least one character.",
            )),
            Syntax::Integer => {
                let digits = value.strip_prefix('-').unwrap_or(value);
                if digits.is_empty()
                    || !digits.bytes().all(|b| b.is_ascii_digit())
                    || digits.len() > 1 && digits.starts_with('0')
                    || value == "-0"
                {
                    Err(issue(
                        Category::Violation,
                        "Use canonical decimal Integer values: no plus sign, leading zeros, spaces or negative zero.",
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    pub fn equivalent(&self, a: &str, b: &str) -> Result<bool> {
        self.validate_value(a)?;
        self.validate_value(b)?;
        let Some(rule) = self.equality else {
            return Err(issue(
                Category::Unsupported,
                "This attribute has no equality rule; value comparison is undefined in this model.",
            ));
        };
        Ok(rule.key(a) == rule.key(b))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedClass {
    pub definition: ObjectClass,
    pub required: BTreeSet<String>,
    pub allowed: BTreeSet<String>,
    pub(crate) ancestors: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registry {
    attributes: BTreeMap<String, ResolvedAttribute>,
    attribute_names: BTreeMap<String, String>,
    classes: BTreeMap<String, ResolvedClass>,
    class_names: BTreeMap<String, String>,
}
impl Registry {
    /// Minimal teaching base, authored from definitions in the standards; this
    /// is not a copy or import of a complete OpenLDAP core.schema. objectClass
    /// is represented by the entry's dedicated class list, not a generic field.
    pub fn teaching_base() -> Self {
        let mut r = Self {
            attributes: BTreeMap::new(),
            attribute_names: BTreeMap::new(),
            classes: BTreeMap::new(),
            class_names: BTreeMap::new(),
        };
        let attributes = [
            "(2.5.4.41 NAME 'name' EQUALITY caseIgnoreMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
            "(2.5.4.3 NAME 'cn' SUP name)",
            "(2.5.4.4 NAME 'sn' SUP name)",
            "(2.5.4.13 NAME 'description' EQUALITY caseIgnoreMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
            "(0.9.2342.19200300.100.1.1 NAME 'uid' EQUALITY caseIgnoreMatch SYNTAX 1.3.6.1.4.1.1466.115.121.1.15)",
        ];
        for a in attributes {
            r.install_attribute(attribute_type(a).expect("fixed base attribute"))
                .expect("fixed base registry");
        }
        r.install_class(
            object_class("(2.5.6.0 NAME 'top' ABSTRACT)").expect("fixed base top"),
            true,
        )
        .expect("fixed base top");
        r.install_class(
            object_class(
                "(2.5.6.6 NAME 'person' SUP top STRUCTURAL MUST (cn $ sn) MAY description)",
            )
            .expect("fixed base person"),
            false,
        )
        .expect("fixed base person");
        r
    }
    pub fn attribute(&self, name: &str) -> Option<&ResolvedAttribute> {
        self.attributes.get(name).or_else(|| {
            self.attribute_names
                .get(&name.to_ascii_lowercase())
                .and_then(|oid| self.attributes.get(oid))
        })
    }
    pub fn class(&self, name: &str) -> Option<&ResolvedClass> {
        self.classes.get(name).or_else(|| {
            self.class_names
                .get(&name.to_ascii_lowercase())
                .and_then(|oid| self.classes.get(oid))
        })
    }
    pub fn counts(&self) -> (usize, usize) {
        (self.attributes.len(), self.classes.len())
    }
    /// One atomic registration batch. Dependencies must precede their users;
    /// forward references are explicitly unsupported, not silently unresolved.
    pub fn register(&mut self, attributes: &[String], classes: &[String]) -> Result<()> {
        if attributes.len() + classes.len() > MAX_BATCH
            || self.attributes.len() + self.classes.len() + attributes.len() + classes.len()
                > MAX_DEFINITIONS
        {
            return Err(issue(
                Category::Limit,
                "Register at most 32 definitions per batch and 64 total definitions, including the teaching base.",
            ));
        }
        let attrs = attributes
            .iter()
            .map(|s| attribute_type(s))
            .collect::<Result<Vec<_>>>()?;
        let objects = classes
            .iter()
            .map(|s| object_class(s))
            .collect::<Result<Vec<_>>>()?;
        let mut next = self.clone();
        for a in attrs {
            next.install_attribute(a)?;
        }
        for c in objects {
            next.install_class(c, false)?;
        }
        *self = next;
        Ok(())
    }
    fn new_oid(&self, oid: &str) -> Result<()> {
        if self.attributes.contains_key(oid)
            || self.classes.contains_key(oid)
            || oid == "2.5.4.0"
            || Matching::parse(oid).is_ok()
            || [Syntax::DirectoryString, Syntax::Ia5String, Syntax::Integer]
                .iter()
                .any(|s| s.oid() == oid)
        {
            Err(issue(
                Category::Conflict,
                "This OID is already registered; existing schema definitions cannot be replaced.",
            ))
        } else {
            Ok(())
        }
    }
    fn install_attribute(&mut self, a: AttributeType) -> Result<()> {
        self.new_oid(&a.oid)?;
        if a.names.iter().any(|n| {
            n.eq_ignore_ascii_case("objectClass")
                || self.attribute_names.contains_key(&n.to_ascii_lowercase())
        }) {
            return Err(issue(
                Category::Conflict,
                "An attribute name or alias is already registered.",
            ));
        }
        let parent = if let Some(name) = &a.superior {
            if name == &a.oid || a.names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
                return Err(issue(
                    Category::Conflict,
                    "An attribute cannot inherit from itself.",
                ));
            }
            let p=self.attribute(name).ok_or_else(||issue(Category::Unsupported,"The SUP attribute is not registered. Register supported dependencies before their users; forward references are unsupported."))?;
            if p.single_value {
                return Err(issue(
                    Category::Unsupported,
                    "Subtyping a SINGLE-VALUE attribute is outside this bounded registry; use a direct definition instead.",
                ));
            }
            Some(p)
        } else {
            None
        };
        let syntax = a
            .syntax
            .as_ref()
            .map(|s| s.kind)
            .or_else(|| parent.map(|p| p.syntax))
            .ok_or_else(|| {
                issue(
                    Category::Violation,
                    "An attribute needs a declared or inherited syntax.",
                )
            })?;
        if parent.is_some_and(|p| p.syntax != syntax) {
            return Err(issue(
                Category::Unsupported,
                "A subtype cannot change syntax in this registry; syntax refinements are not modeled.",
            ));
        }
        let equality = a
            .equality
            .as_deref()
            .map(Matching::parse)
            .transpose()?
            .or_else(|| parent.and_then(|p| p.equality));
        if equality.is_some_and(|r| !r.compatible(syntax)) {
            return Err(issue(
                Category::Violation,
                "The equality rule's assertion syntax does not match this attribute's syntax.",
            ));
        }
        if parent.is_some_and(|p| p.equality.is_some() && p.equality != equality) {
            return Err(issue(
                Category::Unsupported,
                "Overriding an inherited equality rule is not modeled; keep the inherited rule.",
            ));
        }
        let mut ancestors = parent.map(|p| p.ancestors.clone()).unwrap_or_default();
        ancestors.insert(a.oid.clone());
        let resolved = ResolvedAttribute {
            syntax,
            equality,
            single_value: a.single_value,
            ancestors,
            definition: a.clone(),
        };
        for n in &a.names {
            self.attribute_names
                .insert(n.to_ascii_lowercase(), a.oid.clone());
        }
        self.attributes.insert(a.oid.clone(), resolved);
        Ok(())
    }
    fn install_class(&mut self, c: ObjectClass, base_top: bool) -> Result<()> {
        self.new_oid(&c.oid)?;
        if c.names
            .iter()
            .any(|n| self.class_names.contains_key(&n.to_ascii_lowercase()))
        {
            return Err(issue(
                Category::Conflict,
                "An object-class name or alias is already registered.",
            ));
        }
        let mut ancestors = BTreeSet::new();
        let mut required = BTreeSet::new();
        let mut allowed = BTreeSet::new();
        let mut structural: Vec<&ResolvedClass> = Vec::new();
        for name in &c.superiors {
            if name == &c.oid || c.names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
                return Err(issue(
                    Category::Conflict,
                    "An object class cannot inherit from itself.",
                ));
            }
            let p=self.class(name).ok_or_else(||issue(Category::Unsupported,"The SUP class is not registered. Register supported dependencies first; forward references are unsupported."))?;
            if c.kind == ClassKind::Abstract && p.definition.kind == ClassKind::Structural {
                return Err(issue(
                    Category::Violation,
                    "An ABSTRACT class cannot inherit from a STRUCTURAL class.",
                ));
            }
            if p.definition.kind == ClassKind::Structural {
                structural.push(p);
            }
            ancestors.extend(p.ancestors.iter().cloned());
            required.extend(p.required.iter().cloned());
            allowed.extend(p.allowed.iter().cloned());
        }
        for a in &structural {
            for b in &structural {
                if !a.ancestors.contains(&b.definition.oid)
                    && !b.ancestors.contains(&a.definition.oid)
                {
                    return Err(issue(
                        Category::Violation,
                        "Structural superclasses must belong to one inheritance chain.",
                    ));
                }
            }
        }
        if !base_top && !ancestors.contains("2.5.6.0") {
            return Err(issue(
                Category::Unsupported,
                "This class registry requires derivation from top; unrelated abstract hierarchies are outside its subset.",
            ));
        }
        for (names, target) in [(&c.must, &mut required), (&c.may, &mut allowed)] {
            for name in names {
                let a=self.attribute(name).ok_or_else(||issue(Category::Unsupported,"A MUST/MAY attribute is not registered. Define supported attributes before their class."))?;
                target.insert(a.definition.oid.clone());
            }
        }
        allowed.extend(required.iter().cloned());
        ancestors.insert(c.oid.clone());
        for n in &c.names {
            self.class_names
                .insert(n.to_ascii_lowercase(), c.oid.clone());
        }
        self.classes.insert(
            c.oid.clone(),
            ResolvedClass {
                definition: c,
                required,
                allowed,
                ancestors,
            },
        );
        Ok(())
    }
}
