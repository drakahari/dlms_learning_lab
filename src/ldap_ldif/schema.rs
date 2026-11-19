//! Schema Add records, opt-in and separate from ordinary data and olcAccess.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Add {
    pub name: String,
    pub attribute_types: Vec<String>,
    pub object_classes: Vec<String>,
}
fn schema_name(raw: &str) -> Result<String> {
    let (head, parent) = raw.split_once(',').ok_or_else(|| {
        error(
            Category::Syntax,
            1,
            "Use cn=your-name,cn=schema,cn=config for a new schema entry.",
        )
    })?;
    let (attribute, name) = head.split_once('=').ok_or_else(|| {
        error(
            Category::Syntax,
            1,
            "The schema entry needs a cn naming value.",
        )
    })?;
    if !attribute.eq_ignore_ascii_case("cn")
        || !parent.eq_ignore_ascii_case("cn=schema,cn=config")
        || !descriptor(name)
    {
        return Err(error(
            Category::Unsupported,
            1,
            "This Add subset creates an unindexed cn=name below cn=schema,cn=config. Names use a letter followed by letters, digits or hyphens, up to 32 characters; other branches, indexes and escapes are unsupported.",
        ));
    }
    Ok(name.into())
}
fn ordered(values: Vec<(usize, String)>) -> Result<Vec<String>> {
    let tagged = values.first().is_some_and(|(_, v)| v.starts_with('{'));
    let mut result = Vec::new();
    for (position, (line, value)) in values.into_iter().enumerate() {
        if value.starts_with('{') != tagged {
            return Err(error(
                Category::Unsupported,
                line,
                "Use either all unindexed values, or a contiguous indexed list beginning at {0}.",
            ));
        }
        if tagged {
            let (index, description) = value[1..].split_once('}').ok_or_else(|| {
                error(
                    Category::Syntax,
                    line,
                    "Close an ordered value's index with }.",
                )
            })?;
            if index != position.to_string() {
                return Err(error(
                    Category::Unsupported,
                    line,
                    "Indexed schema values must be in contiguous {0}, {1}, ... order in this bounded Add subset.",
                ));
            }
            result.push(description.into());
        } else {
            result.push(value);
        }
    }
    Ok(result)
}
fn record(lines: &[Line]) -> Result<Add> {
    let first = &lines[0];
    let (key, dn) = field_with_value_limit(first, 2048)?;
    if key != "dn" {
        return Err(error(
            Category::Syntax,
            first.number,
            "Start the schema Add record with its dn field.",
        ));
    }
    let name = schema_name(&dn).map_err(|mut e| {
        e.line = first.number;
        e
    })?;
    let change = lines.get(1).ok_or_else(|| {
        error(
            Category::Syntax,
            first.number,
            "Include changetype: add after the schema dn.",
        )
    })?;
    if field(change)? != ("changetype".into(), "add".into()) {
        return Err(error(
            Category::Unsupported,
            change.number,
            "Schema definitions are added as a new olcSchemaConfig entry using explicit changetype: add; Modify/Delete are unsupported.",
        ));
    }
    let mut cn = false;
    let mut object = false;
    let mut attrs = Vec::new();
    let mut classes = Vec::new();
    for line in &lines[2..] {
        let (key, value) = field_with_value_limit(line, 2048)?;
        match key.as_str() {
            "cn" if !cn && value.eq_ignore_ascii_case(&name) => {
                cn = true;
            }
            "objectclass" if !object && value.eq_ignore_ascii_case("olcSchemaConfig") => {
                object = true;
            }
            "olcattributetypes" => attrs.push((line.number, value)),
            "olcobjectclasses" => classes.push((line.number, value)),
            "cn" | "objectclass" => {
                return Err(error(
                    Category::Syntax,
                    line.number,
                    "Use one matching cn value and one objectClass: olcSchemaConfig value.",
                ));
            }
            _ => {
                return Err(error(
                    Category::Unsupported,
                    line.number,
                    "Supported schema Add fields are cn, objectClass: olcSchemaConfig, olcAttributeTypes and olcObjectClasses only.",
                ));
            }
        }
        if attrs.len() + classes.len() > 32 {
            return Err(error(
                Category::Limit,
                line.number,
                "A schema Add permits at most 32 combined attribute/class definitions.",
            ));
        }
    }
    if !cn || !object || attrs.is_empty() && classes.is_empty() {
        return Err(error(
            Category::Syntax,
            first.number,
            "Include cn, objectClass: olcSchemaConfig, and at least one attribute or object-class definition.",
        ));
    }
    Ok(Add {
        name,
        attribute_types: ordered(attrs)?,
        object_classes: ordered(classes)?,
    })
}
pub fn parse(raw: &str) -> Result<Vec<Add>> {
    let lines = unfold(raw)?;
    let header = lines
        .iter()
        .position(|l| !l.text.is_empty())
        .ok_or_else(|| {
            error(
                Category::Syntax,
                1,
                "Start with version: 1 and a schema Add record.",
            )
        })?;
    if field(&lines[header])? != ("version".into(), "1".into()) {
        return Err(error(
            Category::Unsupported,
            lines[header].number,
            "Schema LDIF requires explicit version: 1.",
        ));
    }
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in &lines[header + 1..] {
        if line.text.is_empty() {
            if !current.is_empty() {
                if records.len() >= 8 {
                    return Err(error(
                        Category::Limit,
                        line.number,
                        "A document permits at most eight schema Add records.",
                    ));
                }
                records.push(record(&current)?);
                current.clear();
            }
        } else {
            current.push(line.clone());
        }
    }
    if !current.is_empty() {
        if records.len() >= 8 {
            return Err(error(
                Category::Limit,
                current[0].number,
                "A document permits at most eight schema Add records.",
            ));
        }
        records.push(record(&current)?);
    }
    if records.is_empty() {
        return Err(error(
            Category::Syntax,
            1,
            "Include a schema Add record after version: 1.",
        ));
    }
    Ok(records)
}
