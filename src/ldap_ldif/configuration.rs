//! Bounded config-Modify codec, separate from the existing contact/data codec.
//! No resources, paths, server processes or credentials are accessed.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Frontend,
    Config,
    Mdb,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    Global,
    Database { index: i8, backend: Backend },
}
impl Name {
    pub fn parse(raw: &str) -> Result<Self> {
        if raw.len() > 128 {
            return Err(error(
                Category::Limit,
                1,
                "A configuration name allows128bytes.",
            ));
        }
        if raw.eq_ignore_ascii_case("cn=config") {
            return Ok(Self::Global);
        }
        let bad = || {
            error(
                Category::Unsupported,
                1,
                "Use the actual indexed name from the inventory. This subset supports frontend{-1}, config{0} and MDB{1} through MDB{31}; creation names, escapes and other branches are unsupported.",
            )
        };
        let (head, parent) = raw.split_once(',').ok_or_else(bad)?;
        if !parent.eq_ignore_ascii_case("cn=config") {
            return Err(bad());
        }
        let (attribute, value) = head.split_once('=').ok_or_else(bad)?;
        if !attribute.eq_ignore_ascii_case("olcDatabase") {
            return Err(bad());
        }
        let (index, backend) = value
            .strip_prefix('{')
            .and_then(|s| s.split_once('}'))
            .ok_or_else(bad)?;
        let number: i8 = index.parse().map_err(|_| bad())?;
        if number.to_string() != index {
            return Err(bad());
        }
        let backend = match backend.to_ascii_lowercase().as_str() {
            "frontend" if number == -1 => Backend::Frontend,
            "config" if number == 0 => Backend::Config,
            "mdb" if (1..=31).contains(&number) => Backend::Mdb,
            _ => return Err(bad()),
        };
        Ok(Self::Database {
            index: number,
            backend,
        })
    }
    pub fn text(&self) -> String {
        match self {
            Self::Global => "cn=config".into(),
            Self::Database { index, backend } => format!(
                "olcDatabase={{{index}}}{},cn=config",
                match backend {
                    Backend::Frontend => "frontend",
                    Backend::Config => "config",
                    Backend::Mdb => "mdb",
                }
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Modification {
    pub name: Name,
    pub values: Vec<String>,
}
fn field(line: &Line) -> Result<(String, String)> {
    super::field_with_value_limit(line, 1024)
}
fn record(lines: &[Line]) -> Result<Modification> {
    let first = &lines[0];
    let (attribute, value) = field(first)?;
    if attribute != "dn" {
        return Err(error(
            Category::Syntax,
            first.number,
            "Start the configuration record with dn: its full configuration name.",
        ));
    }
    let name = Name::parse(&value).map_err(|mut e| {
        e.line = first.number;
        e
    })?;
    let expected = [("changetype", "modify"), ("replace", "olcaccess")];
    for (at, (attribute, value)) in expected.into_iter().enumerate() {
        let line = lines.get(at + 1).ok_or_else(|| {
            error(
                Category::Syntax,
                first.number,
                "Include changetype: modify and replace: olcAccess after dn.",
            )
        })?;
        let (actual_attribute, actual_value) = field(line)?;
        if actual_attribute != attribute || !actual_value.eq_ignore_ascii_case(value) {
            return Err(error(
                Category::Unsupported,
                line.number,
                "This configuration subset supports one complete replace: olcAccess in each Modify record. Other directives, add/delete and multiple edits are unsupported.",
            ));
        }
    }
    let mut values = Vec::new();
    let mut ended = false;
    for line in &lines[3..] {
        if line.text == "-" && !ended {
            ended = true;
            continue;
        }
        if ended {
            return Err(error(
                Category::Unsupported,
                line.number,
                "Only one olcAccess replacement per configuration request is supported.",
            ));
        }
        let (attribute, value) = field(line)?;
        if attribute != "olcaccess" {
            return Err(error(
                Category::Unsupported,
                line.number,
                "Only olcAccess values belong in this supported replacement; no other setting changed.",
            ));
        }
        if values.len() >= 16 {
            return Err(error(
                Category::Limit,
                line.number,
                "At most16ordered access values per replacement.",
            ));
        }
        values.push(value);
    }
    // A final '-' is optional in the existing LDIF grammar. Empty replacement
    // removes the attribute; the effective frontend/default policy still applies.
    Ok(Modification { name, values })
}
/// Syntax preflight for a bounded complete document. Semantic application is
/// per request, never a claim of whole-document atomicity.
pub fn parse(raw: &str) -> Result<Vec<Modification>> {
    let lines = unfold(raw)?;
    let header = lines
        .iter()
        .position(|l| !l.text.is_empty())
        .ok_or_else(|| {
            error(
                Category::Syntax,
                1,
                "Start with version: 1 and include a configuration Modify record.",
            )
        })?;
    if field(&lines[header])? != ("version".into(), "1".into()) {
        return Err(error(
            Category::Unsupported,
            lines[header].number,
            "This configuration subset requires explicit version: 1.",
        ));
    }
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in lines.into_iter().skip(header + 1) {
        if line.text.is_empty() {
            if !current.is_empty() {
                if records.len() >= MAX_RECORDS {
                    return Err(error(
                        Category::Limit,
                        line.number,
                        "At most16configuration records.",
                    ));
                }
                records.push(record(&current)?);
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        if records.len() >= MAX_RECORDS {
            return Err(error(
                Category::Limit,
                current[0].number,
                "At most16configuration records.",
            ));
        }
        records.push(record(&current)?);
    }
    if records.is_empty() {
        return Err(error(
            Category::Syntax,
            1,
            "Include a configuration Modify record after version: 1.",
        ));
    }
    Ok(records)
}
