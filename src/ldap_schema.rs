//! Bounded, opt-in LDAP schema parsing, atomic registration and entry validation.
//! The legacy fixed-schema engines remain unchanged. No I/O, transport, host
//! service or process is used by this in-memory model.
pub const MAX_DESCRIPTION: usize = 2048;
pub const MAX_VALUE: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Syntax,
    Unsupported,
    Limit,
    Conflict,
    Violation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub category: Category,
    pub subject: Option<String>,
    pub line: usize,
    pub column: usize,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(subject) = &self.subject {
            write!(f, "{subject}: ")?;
        }
        let label = match self.category {
            Category::Syntax => "Syntax error",
            Category::Unsupported => "Unsupported",
            Category::Limit => "Simulation limit",
            Category::Conflict => "Conflicting definition",
            Category::Violation => "Schema rule",
        };
        if self.line > 0 {
            write!(
                f,
                "{label} at line {}, column {}: {}",
                self.line, self.column, self.message
            )
        } else {
            write!(f, "{label}: {}", self.message)
        }
    }
}
impl std::error::Error for Diagnostic {}
pub type Result<T> = std::result::Result<T, Diagnostic>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syntax {
    DirectoryString,
    Ia5String,
    Integer,
}
impl Syntax {
    pub fn oid(self) -> &'static str {
        match self {
            Self::DirectoryString => "1.3.6.1.4.1.1466.115.121.1.15",
            Self::Ia5String => "1.3.6.1.4.1.1466.115.121.1.26",
            Self::Integer => "1.3.6.1.4.1.1466.115.121.1.27",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxReference {
    pub kind: Syntax,
    /// Recommended minimum capability, never a maximum value length. The model
    /// separately bounds actual data to MAX_VALUE for resource isolation.
    pub minimum_size_hint: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeType {
    pub oid: String,
    pub names: Vec<String>,
    pub description: Option<String>,
    pub superior: Option<String>,
    pub equality: Option<String>,
    pub syntax: Option<SyntaxReference>,
    pub single_value: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassKind {
    Abstract,
    Structural,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectClass {
    pub oid: String,
    pub names: Vec<String>,
    pub description: Option<String>,
    pub superiors: Vec<String>,
    pub kind: ClassKind,
    pub must: Vec<String>,
    pub may: Vec<String>,
}
pub fn numeric_oid(s: &str) -> bool {
    if s.len() > 128 {
        return false;
    }
    let mut n = 0;
    for component in s.split('.') {
        if component.is_empty()
            || component.len() > 1 && component.starts_with('0')
            || !component.bytes().all(|b| b.is_ascii_digit())
        {
            return false;
        }
        n += 1;
    }
    (2..=64).contains(&n)
}
fn descriptor(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
mod description;
pub use description::{attribute_type, object_class};

mod registry;
pub use registry::{Matching, Registry, ResolvedAttribute, ResolvedClass};

mod entries;
pub use entries::{Directory, Entry, EntrySource, Field};

mod loading;
pub use loading::{Load, Module};

mod filter;
pub use filter::Filter;
