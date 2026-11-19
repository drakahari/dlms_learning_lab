//! Registry-aware filters over validated, in-memory custom-schema entries.
//! Uses the existing bounded filter grammar. No command or server execution.
use super::*;
use crate::ldap_query::{Node, Truth};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter(crate::ldap_query::Filter);
impl Filter {
    pub fn parse(text: &str) -> Result<Self> {
        crate::ldap_query::Filter::parse(text)
            .map(Self)
            .map_err(|e| {
                let (category, message) = match e {
                    crate::ldap_query::Error::Limit(s) => (Category::Limit, s),
                    crate::ldap_query::Error::Unsupported(s) => (Category::Unsupported, s),
                    crate::ldap_query::Error::Syntax(s) => (Category::Syntax, s),
                    crate::ldap_query::Error::Fixture(s) => (Category::Violation, s),
                };
                Diagnostic {
                    category,
                    subject: Some("filter".into()),
                    line: 0,
                    column: 0,
                    message,
                }
            })
    }
    /// Bounded read-only literals for preparing behavioral verification inputs.
    /// The evaluator still executes the actual parsed filter; this is not a
    /// comparison against reference code or an accepted-command whitelist.
    pub(crate) fn equality_values(&self) -> Vec<&str> {
        fn collect<'a>(n: &'a Node, out: &mut Vec<&'a str>) {
            match n {
                Node::Equal(_, value) => out.push(value),
                Node::Present(_) => (),
                Node::And(nodes) | Node::Or(nodes) => {
                    for n in nodes {
                        collect(n, out);
                    }
                }
                Node::Not(n) => collect(n, out),
            }
        }
        let mut values = Vec::new();
        collect(self.0.root(), &mut values);
        values
    }
    // Only Directory exposes evaluation, preserving the entry/registry context.
    // This is search truth, not an LDAP Compare operation result code.
    fn evaluate(&self, registry: &Registry, entry: &Entry) -> Truth {
        evaluate(self.0.root(), registry, entry)
    }
}
impl Directory {
    /// Bounded evaluation over at most32actual stored entries. This is not a
    /// search request: no base/scope, projection, access policy or wire result.
    pub fn evaluate_filter(&self, filter: &Filter) -> Vec<(&Entry, Truth)> {
        self.entries()
            .iter()
            .map(|entry| (entry, filter.evaluate(self.registry(), entry)))
            .collect()
    }
}
fn evaluate(node: &Node, registry: &Registry, entry: &Entry) -> Truth {
    use Truth::*;
    match node {
        Node::Present(name) if name == "objectclass" => {
            if entry.classes().is_empty() {
                False
            } else {
                True
            }
        }
        Node::Equal(name, value) if name == "objectclass" => {
            let oid = if numeric_oid(value) {
                value.as_str()
            } else if let Some(class) = registry.class(value) {
                &class.definition.oid
            } else {
                return Undefined;
            };
            if entry.classes().contains(oid) {
                True
            } else {
                False
            }
        }
        Node::Present(name) => {
            let Some(target) = registry.attribute(name) else {
                return Undefined;
            };
            if entry.fields().keys().any(|oid| {
                registry
                    .attribute(oid)
                    .is_some_and(|a| a.ancestors.contains(&target.definition.oid))
            }) {
                True
            } else {
                False
            }
        }
        Node::Equal(name, value) => {
            let Some(target) = registry.attribute(name) else {
                return Undefined;
            };
            let Some(rule) = target.equality else {
                return Undefined;
            };
            // Invalid/unsupported assertion values must remain Undefined even
            // when the entry does not carry this attribute (especially NOT).
            if target.validate_value(value).is_err() {
                return Undefined;
            }
            let key = rule.key(value);
            if entry.fields().iter().any(|(oid, values)| {
                registry
                    .attribute(oid)
                    .is_some_and(|a| a.ancestors.contains(&target.definition.oid))
                    && values.iter().any(|v| rule.key(v) == key)
            }) {
                True
            } else {
                False
            }
        }
        Node::Not(child) => match evaluate(child, registry, entry) {
            True => False,
            False => True,
            Undefined => Undefined,
        },
        Node::And(children) => {
            let mut unknown = false;
            for child in children {
                match evaluate(child, registry, entry) {
                    False => return False,
                    Undefined => unknown = true,
                    True => (),
                }
            }
            if unknown { Undefined } else { True }
        }
        Node::Or(children) => {
            let mut unknown = false;
            for child in children {
                match evaluate(child, registry, entry) {
                    True => return True,
                    Undefined => unknown = true,
                    False => (),
                }
            }
            if unknown { Undefined } else { False }
        }
    }
}
