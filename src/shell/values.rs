//! Runtime variable values and the deliberately bounded indexed-parameter syntax.
use super::{name, readable_variable};
#[derive(Clone, Debug)]
pub(super) enum Value {
    Scalar(String),
    Indexed(Vec<String>),
}
impl Value {
    pub fn scalar(&self) -> &str {
        match self {
            Self::Scalar(s) => s,
            Self::Indexed(v) => v.first().map_or("", String::as_str),
        }
    }
    pub fn elements(&self) -> &[String] {
        match self {
            Self::Scalar(s) => std::slice::from_ref(s),
            Self::Indexed(v) => v,
        }
    }
    pub fn bytes(&self) -> usize {
        match self {
            Self::Scalar(s) => s.len(),
            Self::Indexed(v) => {
                v.iter().map(String::len).sum::<usize>() + v.len() * std::mem::size_of::<String>()
            }
        }
    }
    pub fn assign_scalar(&mut self, value: String) {
        match self {
            Self::Scalar(s) => *s = value,
            Self::Indexed(v) if v.is_empty() => v.push(value),
            Self::Indexed(v) => v[0] = value,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) enum ArrayRef<'a> {
    Element(&'a str, u32),
    All(&'a str),
    Count(&'a str),
}
pub(super) fn array_ref(key: &str) -> Result<Option<ArrayRef<'_>>, String> {
    if !key.contains(['[', ']']) {
        return Ok(None);
    }
    let (count, word) = key.strip_prefix('#').map_or((false, key), |s| (true, s));
    let Some((base, index)) = word.split_once('[') else {
        return Err("Unsupported array parameter syntax".into());
    };
    let index = index
        .strip_suffix(']')
        .ok_or("Unsupported array parameter syntax; close the bracket")?;
    if !name(base) {
        return Err("Unsupported array parameter name".into());
    }
    readable_variable(base)?;
    if index == "@" {
        return Ok(Some(if count {
            ArrayRef::Count(base)
        } else {
            ArrayRef::All(base)
        }));
    }
    if count {
        return Err(
            "Unsupported element-length expansion; use ${#name[@]} for the number of elements"
                .into(),
        );
    }
    if index.is_empty()
        || !index.bytes().all(|b| b.is_ascii_digit())
        || (index.len() > 1 && index.starts_with('0'))
    {
        return Err("Unsupported array index: use a constant nonnegative decimal index without leading zeros; computed, negative, octal and * forms are not supported".into());
    }
    let index = index.parse::<u32>().map_err(
        |_| "Unsupported array index: constant indices are limited to 0 through4294967295",
    )?;
    Ok(Some(ArrayRef::Element(base, index)))
}
/// Exempt supported parameter brackets, not unrelated bracket glob syntax.
pub(super) fn bracket_glob(text: &str) -> Result<bool, String> {
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        if rest[..start].contains(['[', ']']) {
            return Ok(true);
        }
        let Some(end) = rest[start + 2..].find('}') else {
            return Err("Close ${parameter} before running".into());
        };
        let end = start + 2 + end;
        array_ref(&rest[start + 2..end])?;
        rest = &rest[end + 1..];
    }
    Ok(rest.contains(['[', ']']))
}
