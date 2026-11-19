//! Read the limited InputClass syntax used by the virtual Xorg keyboards.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const LIMIT: usize = 4096;
const MAX_CLASSES: usize = 8;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Us,
    De,
    Fr,
}

impl Layout {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Us => "us",
            Self::De => "de",
            Self::Fr => "fr",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "us" => Ok(Self::Us),
            "de" => Ok(Self::De),
            "fr" => Ok(Self::Fr),
            _ => Err("Use one of the supplied layouts: us, de or fr.".into()),
        }
    }

    /// A few named physical keys with no modifier. Not a complete keyboard map.
    pub const fn letters(self) -> [(char, char); 4] {
        match self {
            Self::Us => [('Q', 'q'), ('A', 'a'), ('Y', 'y'), ('Z', 'z')],
            Self::De => [('Q', 'q'), ('A', 'a'), ('Y', 'z'), ('Z', 'y')],
            Self::Fr => [('Q', 'a'), ('A', 'q'), ('Y', 'y'), ('Z', 'w')],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub product: String,
    pub default: Layout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub identifier: String,
    product: Option<String>,
    keyboard_only: Option<bool>,
    layout: Layout,
}

#[derive(Default)]
struct OpenRule {
    identifier: Option<String>,
    product: Option<String>,
    keyboard_only: Option<bool>,
    layout: Option<Layout>,
}

impl OpenRule {
    fn finish(self) -> Result<Rule> {
        Ok(Rule {
            identifier: self
                .identifier
                .ok_or("Add an Identifier inside this InputClass.")?,
            product: self.product,
            keyboard_only: self.keyboard_only,
            layout: self
                .layout
                .ok_or("Add an XkbLayout option to this InputClass.")?,
        })
    }

    fn directive(&mut self, verb: &str, line: &str) -> Result<()> {
        let (value, remainder) = quoted(line)?;
        match verb.to_ascii_lowercase().as_str() {
            "identifier" => {
                empty(remainder)?;
                field(&mut self.identifier, bounded_label(value)?)
            }
            "matchproduct" => {
                empty(remainder)?;
                field(&mut self.product, bounded_label(value)?)
            }
            "matchiskeyboard" => {
                empty(remainder)?;
                let boolean = match value.to_ascii_lowercase().as_str() {
                    "yes" | "on" | "true" | "1" => true,
                    "no" | "off" | "false" | "0" => false,
                    _ => return Err("Use a quoted boolean value for MatchIsKeyboard.".into()),
                };
                field(&mut self.keyboard_only, boolean)
            }
            "option" if value.eq_ignore_ascii_case("XkbLayout") => {
                let (layout, rest) = quoted(remainder)?;
                empty(rest)?;
                field(&mut self.layout, Layout::parse(layout)?)
            }
            _ => Err("This InputClass exercise supports Identifier, MatchProduct, MatchIsKeyboard and Option \"XkbLayout\".".into()),
        }
    }
}

fn quoted(line: &str) -> Result<(&str, &str)> {
    let rest = line
        .trim_start()
        .strip_prefix('"')
        .ok_or("Put each value in double quotes.")?;
    let (value, remainder) = rest.split_once('"').ok_or("Close the quoted value.")?;
    if value.contains('\\') {
        return Err("Use a literal quoted value without an escape.".into());
    }
    Ok((value, remainder.trim_start()))
}

fn bounded_label(value: &str) -> Result<String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b" -_.".contains(&b))
    {
        return Err("Use a short literal name or product substring without alternatives.".into());
    }
    Ok(value.into())
}

fn empty(line: &str) -> Result<()> {
    if !line.is_empty() && !line.starts_with('#') {
        return Err("Unexpected text after the supported directive.".into());
    }
    Ok(())
}

fn field<T>(slot: &mut Option<T>, value: T) -> Result<()> {
    if slot.is_some() {
        return Err("Use each supported setting once per InputClass in this exercise.".into());
    }
    *slot = Some(value);
    Ok(())
}

pub fn parse(text: &str) -> Result<Vec<Rule>> {
    if text.len() > LIMIT
        || text.lines().count() > 80
        || !text.is_ascii()
        || text
            .lines()
            .any(|s| s.chars().any(|c| c.is_control() && c != '\t'))
    {
        return Err("Keep the virtual fragment within 4 KiB and 80 plain-text lines.".into());
    }
    let mut result = Vec::new();
    let mut open = None::<OpenRule>;
    let mut ids = BTreeSet::new();
    for (n, line) in text.lines().enumerate() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        let split = text
            .find(|c: char| c.is_ascii_whitespace() || c == '#')
            .unwrap_or(text.len());
        let (verb, rest) = text.split_at(split);
        let error = |err: String| format!("Line {}. {err}", n + 1);
        if verb.eq_ignore_ascii_case("Section") {
            let (name, rest) = quoted(rest).map_err(error)?;
            empty(rest).map_err(error)?;
            if !name.eq_ignore_ascii_case("InputClass")
                || open.is_some()
                || result.len() == MAX_CLASSES
            {
                return Err(error(
                    "Enter up to eight unnested InputClass sections.".into(),
                ));
            }
            open = Some(OpenRule::default());
        } else if verb.eq_ignore_ascii_case("EndSection") {
            empty(rest.trim_start()).map_err(error)?;
            let rule = open
                .take()
                .ok_or_else(|| error("No InputClass section is open.".into()))?
                .finish()
                .map_err(error)?;
            if !ids.insert(rule.identifier.clone()) {
                return Err(error("Give each InputClass a distinct Identifier.".into()));
            }
            result.push(rule);
        } else {
            open.as_mut()
                .ok_or_else(|| error("Begin an InputClass section first.".into()))?
                .directive(verb, rest)
                .map_err(error)?;
        }
    }
    if open.is_some() {
        return Err("Close the final InputClass section with EndSection.".into());
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effective<'a> {
    pub layout: Layout,
    pub matched: Vec<&'a str>,
}

/// All supplied devices here are keyboards. Each matching rule can override
/// earlier layout settings, in order. MatchProduct uses a literal substring.
pub fn effective<'a>(device: &Device, rules: &'a [Rule]) -> Effective<'a> {
    let mut state = Effective {
        layout: device.default,
        matched: Vec::new(),
    };
    for rule in rules {
        if rule.keyboard_only != Some(false)
            && rule
                .product
                .as_ref()
                .is_none_or(|match_product| device.product.contains(match_product))
        {
            state.layout = rule.layout;
            state.matched.push(&rule.identifier);
        }
    }
    state
}
