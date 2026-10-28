//! Four deliberately bounded, read-only parameter operators. No execution,
//! assignments, capture, quote reparsing or lookup of host state occurs here.
use super::{name, readable_variable};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target<'a> {
    Named(&'a str),
    Positional(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operator {
    Default,
    Alternate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Expansion<'a> {
    pub target: Target<'a>,
    word: &'a str,
    operator: Operator,
    test_null: bool,
}
impl Expansion<'_> {
    /// Presence is distinct from an existing empty scalar/argument. Returned
    /// text stays borrowed until the ordinary VM append/split/glob path owns it.
    pub(super) fn select<'a>(&'a self, value: Option<&'a str>) -> &'a str {
        let present = value.is_some_and(|v| !self.test_null || !v.is_empty());
        match self.operator {
            Operator::Default if !present => self.word,
            Operator::Default => value.unwrap(),
            Operator::Alternate if present => self.word,
            Operator::Alternate => "",
        }
    }
}

/// None preserves all prior plain/array/unsupported-operator dispatch. Only the
/// four declared forms enter this grammar; their WORD is literal source text.
pub(super) fn parse(key: &str) -> Result<Option<Expansion<'_>>, String> {
    let end = if key.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        key.bytes().take_while(u8::is_ascii_digit).count()
    } else {
        key.bytes()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
            .count()
    };
    let (base, suffix) = key.split_at(end);
    let (test_null, tail) = suffix
        .strip_prefix(':')
        .map_or((false, suffix), |tail| (true, tail));
    let (operator, word) = if let Some(word) = tail.strip_prefix('-') {
        (Operator::Default, word)
    } else if let Some(word) = tail.strip_prefix('+') {
        (Operator::Alternate, word)
    } else {
        return Ok(None);
    };
    let target = if name(base) {
        readable_variable(base)?;
        Target::Named(base)
    } else if base.len() == 1 && matches!(base.as_bytes()[0], b'1'..=b'9') {
        Target::Positional(usize::from(base.as_bytes()[0] - b'1'))
    } else {
        return Err("Unsupported default/alternate parameter target; use an ordinary scalar name or $1 through $9".into());
    };
    if word.starts_with('~')
        || word.contains(['$', '\'', '"', '\\', '`', '{', '}', '[', ']', '\r', '\0'])
    {
        return Err("Unsupported default/alternate WORD: this subset accepts literal text only. Put quotes around the complete expansion; nested expansion, inner quotes, escapes and bracket patterns are not supported".into());
    }
    Ok(Some(Expansion {
        target,
        word,
        operator,
        test_null,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn truth_table_distinguishes_absent_empty_and_nonempty_without_mutation() {
        for (form, expected) in [
            ("value-backup", ["backup", "", "kept"]),
            ("value:-backup", ["backup", "backup", "kept"]),
            ("value+label", ["", "label", "label"]),
            ("value:+label", ["", "", "label"]),
        ] {
            let expression = parse(form).unwrap().unwrap();
            for (value, expected) in [None, Some(""), Some("kept")].into_iter().zip(expected) {
                assert_eq!(expression.select(value), expected);
            }
        }
    }
    #[test]
    fn literal_words_are_borrowed_and_plain_or_other_operators_are_not_reinterpreted() {
        for text in [
            "name",
            "?",
            "#",
            "array[0]",
            "#array[@]",
            "name:=value",
            "name?error",
            "name%tail",
        ] {
            assert_eq!(parse(text).unwrap(), None, "{text}");
        }
        for text in [
            "name-",
            "name:+",
            "name:-two words",
            "name:---operand",
            "1+éclair",
        ] {
            assert!(parse(text).unwrap().is_some(), "{text}");
        }
        for text in [
            "0-word",
            "12-word",
            "-word",
            "IFS-word",
            "UID-word",
            "name:-$other",
            "name:-'quote'",
            "name:-\"quote\"",
            "name:-a\\ b",
            "name:-~",
            "name:-[abc]",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
    }
}
