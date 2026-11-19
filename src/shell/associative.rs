//! Bounded named maps. Source is parsed; expanded keys remain literal data.
//! No host access, arithmetic subscript evaluation, enumeration or Bash import.
use super::*;

const MAX_ENTRIES: usize = 128;
const MAX_KEY: usize = 128;
// Conservative node/key/value ownership charge, in addition to text bytes.
pub(super) const ENTRY_BYTES: usize = 128;

#[derive(Clone, Debug)]
pub(super) enum Key {
    Literal(String),
    Variable(String),
}
fn key(text: &str, quote: Quote) -> Result<Key, String> {
    let variable = if quote != Quote::Single {
        text.strip_prefix("${")
            .and_then(|s| s.strip_suffix('}'))
            .or_else(|| text.strip_prefix('$'))
    } else {
        None
    };
    if let Some(variable) = variable {
        if !name(variable) {
            return Err(
                "Unsupported map key expansion; use one ordinary scalar variable, such as $key"
                    .into(),
            );
        }
        readable_variable(variable)?;
        return Ok(Key::Variable(variable.into()));
    }
    if quote != Quote::Single && text.contains(['$', '`', '\\', '"', '\'']) {
        return Err(
            "Unsupported map key expression; use a literal key or one quoted scalar variable"
                .into(),
        );
    }
    if quote == Quote::Bare
        && !text
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    {
        return Err("Unsupported bare map key; quote text containing spaces or punctuation".into());
    }
    checked_key(text)?;
    Ok(Key::Literal(text.into()))
}
fn checked_key(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("Map keys cannot be empty. Check the supplied key before storing or looking up a value.".into());
    }
    if matches!(text, "@" | "*") {
        return Err(
            "Unsupported special map key @ or *; map enumeration is outside this subset".into(),
        );
    }
    if text.len() > MAX_KEY {
        return Err("Map key limit exceeded (128 UTF-8 bytes)".into());
    }
    Ok(())
}
impl Key {
    fn resolve<'a>(&'a self, vm: &'a Vm) -> Result<&'a str, String> {
        let text = match self {
            Self::Literal(text) => text,
            Self::Variable(name) => match vm.env.get(name) {
                Some(Value::Scalar(text)) => text,
                None => "",
                _ => {
                    return Err(
                        "Unsupported array-valued map key variable; use an ordinary scalar".into(),
                    );
                }
            },
        };
        checked_key(text)?;
        Ok(text)
    }
}

pub(super) struct Assignment {
    pub base: String,
    pub key: Key,
    pub value: Word,
}
/// Recognize only a whole assignment, retaining quote provenance in both sides.
pub(super) fn looks_like_assignment(word: &Word) -> bool {
    word.0
        .first()
        .filter(|p| p.quote == Quote::Bare)
        .and_then(Part::text)
        .and_then(|s| s.split_once('['))
        .is_some_and(|(base, _)| name(base))
        && word
            .0
            .iter()
            .any(|p| p.quote == Quote::Bare && p.text().is_some_and(|s| s.contains("]=")))
}
pub(super) fn assignment(word: &Word) -> Result<Option<Assignment>, String> {
    let Some(first) = word
        .0
        .first()
        .filter(|p| p.quote == Quote::Bare)
        .and_then(Part::text)
    else {
        return Ok(None);
    };
    let Some((base, rest)) = first.split_once('[').filter(|(base, _)| name(base)) else {
        return Ok(None);
    };
    // A normal bracket-containing argument remains the legacy filename grammar.
    if !word
        .0
        .iter()
        .any(|p| p.quote == Quote::Bare && p.text().is_some_and(|t| t.contains("]=")))
    {
        return Ok(None);
    }
    variable_target(base)?;
    let mut key_parts = vec![];
    for (i, part) in word.0.iter().enumerate() {
        let text = part.text().ok_or("Unsupported command substitution in a map assignment; calculate the value separately first")?;
        let text = if i == 0 { rest } else { text };
        if part.quote == Quote::Bare
            && let Some((end, value)) = text.split_once("]=")
        {
            if !end.is_empty() {
                key_parts.push((end, Quote::Bare));
            }
            if key_parts.len() != 1 {
                return Err(
                    "Unsupported composed map key; use one literal or one scalar variable".into(),
                );
            }
            let (text, quote) = key_parts[0];
            let key = key(text, quote)?;
            let mut values = vec![Part {
                value: PartValue::Text(value.into()),
                quote: Quote::Bare,
            }];
            for part in &word.0[i + 1..] {
                if matches!(part.value, PartValue::Substitution(_)) {
                    return Err("Unsupported command substitution in a map assignment; calculate the value separately first".into());
                }
                values.push(part.clone());
            }
            return Ok(Some(Assignment {
                base: base.into(),
                key,
                value: Word(values),
            }));
        }
        if !text.is_empty() {
            key_parts.push((text, part.quote));
        }
    }
    Err("Unsupported map assignment; use name[key]=value".into())
}

impl Vm {
    pub(super) fn declare_map(&mut self, command: &Command) -> Result<(), String> {
        if command.words.len() != 3
            || !command.redirects.is_empty()
            || command.words[1].literal() != Some("-A")
        {
            return Err("Unsupported declare form; this subset supports only literal declare -A NAME with no redirection".into());
        }
        let base = command.words[2]
            .literal()
            .filter(|s| name(s))
            .ok_or("Unsupported computed map name; write a literal variable name")?;
        variable_target(base)?;
        if !self.calls.is_empty() {
            return Err("Unsupported function-local associative declaration; declare the map outside functions".into());
        }
        if self.xtrace || self.environment_active {
            return Err(
                "Unsupported associative maps with xtrace or export/environment observation".into(),
            );
        }
        match self.env.get(base) {
            Some(Value::Associative(_)) => Ok(()),
            Some(_) => Err(
                "Unsupported scalar/indexed conversion to a map; choose a fresh variable name"
                    .into(),
            ),
            None => self.store_value(base.into(), Value::Associative(BTreeMap::new())),
        }
    }
    pub(super) fn map_key(&self, base: &str, key: &Key) -> Result<String, String> {
        if !matches!(self.env.get(base), Some(Value::Associative(_))) {
            return Err("Unsupported element assignment on an undeclared or indexed array; first use declare -A NAME for a named map".into());
        }
        if self.xtrace || self.environment_active {
            return Err(
                "Unsupported associative maps with xtrace or export/environment observation".into(),
            );
        }
        let text = key.resolve(self)?;
        self.charge_expansion(text.len())?;
        self.reserve_memory(text.len())?;
        Ok(text.into())
    }
    pub(super) fn assign_map(
        &mut self,
        base: &str,
        key: String,
        value: String,
    ) -> Result<(), String> {
        checked_key(&key)?;
        let Some(Value::Associative(map)) = self.env.get(base) else {
            return Err("Unsupported map assignment without declare -A".into());
        };
        if !map.contains_key(&key) && map.len() >= MAX_ENTRIES {
            return Err("Map entry limit exceeded (128)".into());
        }
        let bytes = map
            .iter()
            .filter(|(k, _)| *k != &key)
            .map(|(k, v)| k.len() + v.len() + ENTRY_BYTES)
            .sum::<usize>()
            + key.len()
            + value.len()
            + ENTRY_BYTES;
        self.preflight_value(base, bytes)?;
        let Some(Value::Associative(map)) = self.env.get_mut(base) else {
            unreachable!()
        };
        map.insert(key, value);
        Ok(())
    }
    pub(super) fn map_reference<'a>(&'a self, expression: &str) -> Result<Option<&'a str>, String> {
        let Some((base, subscript)) = expression.split_once('[') else {
            return Ok(None);
        };
        let Some(Value::Associative(map)) = self.env.get(base) else {
            return Ok(None);
        };
        let subscript = subscript
            .strip_suffix(']')
            .ok_or("Unsupported map reference; close the bracket")?;
        let (subscript, quote) = if let Some(s) = subscript
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
        {
            (s, Quote::Double)
        } else if let Some(s) = subscript
            .strip_prefix('\'')
            .and_then(|s| s.strip_suffix('\''))
        {
            (s, Quote::Single)
        } else {
            (subscript, Quote::Bare)
        };
        let _scratch = self.scratch(expression.len() * 2)?;
        let key = key(subscript, quote)?;
        let key = key.resolve(self)?;
        self.charge_expansion(key.len())?;
        Ok(Some(map.get(key).map_or("", String::as_str)))
    }
    pub(super) fn map_base(&self, expression: &str) -> bool {
        expression
            .split_once('[')
            .is_some_and(|(base, _)| matches!(self.env.get(base), Some(Value::Associative(_))))
    }
}
