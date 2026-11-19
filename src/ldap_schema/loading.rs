//! In-process LDAP-style Add application, never a host schema file or service.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub dn: String,
    pub attributes: Vec<String>,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Load {
    pub applied: Vec<Module>,
    pub error: Option<Diagnostic>,
}
impl Directory {
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }
    /// Syntax preflight of the LDIF envelope, then sequential atomic Add
    /// requests. A later rejected Add does not roll back an earlier success.
    pub fn load(&mut self, raw: &str) -> Load {
        let requests = match crate::ldap_ldif::schema::parse(raw) {
            Ok(v) => v,
            Err(e) => {
                return Load {
                    applied: Vec::new(),
                    error: Some(Diagnostic {
                        category: match e.category {
                            crate::ldap_ldif::Category::Syntax => Category::Syntax,
                            crate::ldap_ldif::Category::Unsupported => Category::Unsupported,
                            crate::ldap_ldif::Category::Limit => Category::Limit,
                        },
                        subject: None,
                        line: e.line,
                        column: 1,
                        message: e.message,
                    }),
                };
            }
        };
        let mut applied = Vec::new();
        for request in requests {
            if self.modules.len() >= 8 {
                return Load {
                    applied,
                    error: Some(registry::issue(
                        Category::Limit,
                        "The simulation supports at most eight added schema entries.",
                    )),
                };
            }
            if self
                .modules
                .iter()
                .any(|m| m.name.eq_ignore_ascii_case(&request.name))
            {
                return Load {
                    applied,
                    error: Some(registry::issue(
                        Category::Conflict,
                        "A schema entry with this cn already exists. Add cannot replace an existing definition.",
                    )),
                };
            }
            if let Err(e) = self.register(&request.attribute_types, &request.object_classes) {
                return Load {
                    applied,
                    error: Some(e),
                };
            }
            let module = Module {
                dn: format!(
                    "cn={{{}}}{},cn=schema,cn=config",
                    self.modules.len() + 1,
                    request.name
                ),
                name: request.name,
                attributes: request.attribute_types,
                classes: request.object_classes,
            };
            self.modules.push(module.clone());
            applied.push(module);
        }
        Load {
            applied,
            error: None,
        }
    }
}
