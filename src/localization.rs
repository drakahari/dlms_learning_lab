//! Bounded locale precedence and sample formatting capability. No authored
//! curriculum, host locale or shell evaluation is implied.
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
pub const MAX_CONFIG: usize = 2048;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    C,
    Cutf8,
    EnUsUtf8,
    DeDeUtf8,
}
impl Locale {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "C" | "POSIX" => Ok(Self::C),
            "C.UTF-8" | "C.utf8" => Ok(Self::Cutf8),
            "en_US.UTF-8" | "en_US.utf8" => Ok(Self::EnUsUtf8),
            "de_DE.UTF-8" | "de_DE.utf8" => Ok(Self::DeDeUtf8),
            _ => Err(format!(
                "Locale {value:?} is outside this supplied subset. Inspect the available locales."
            )),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Cutf8 => "C.UTF-8",
            Self::EnUsUtf8 => "en_US.UTF-8",
            Self::DeDeUtf8 => "de_DE.UTF-8",
        }
    }
    pub fn charmap(self) -> &'static str {
        if self == Self::C {
            "ANSI_X3.4-1968"
        } else {
            "UTF-8"
        }
    }
    pub fn decimal(self) -> char {
        if self == Self::DeDeUtf8 { ',' } else { '.' }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Ctype,
    Numeric,
    Time,
    Collate,
    Monetary,
    Messages,
}
impl Category {
    pub const ALL: [Self; 6] = [
        Self::Ctype,
        Self::Numeric,
        Self::Time,
        Self::Collate,
        Self::Monetary,
        Self::Messages,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Ctype => "LC_CTYPE",
            Self::Numeric => "LC_NUMERIC",
            Self::Time => "LC_TIME",
            Self::Collate => "LC_COLLATE",
            Self::Monetary => "LC_MONETARY",
            Self::Messages => "LC_MESSAGES",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "category",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Variable {
    Lang,
    All,
    Category(Category),
}
impl Variable {
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "LANG" => Ok(Self::Lang),
            "LC_ALL" => Ok(Self::All),
            _ => Category::ALL
                .into_iter()
                .find(|c| c.name() == name)
                .map(Self::Category)
                .ok_or_else(|| {
                    format!("{name} is outside the six-category locale subset; no setting changed.")
                }),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Lang => "LANG",
            Self::All => "LC_ALL",
            Self::Category(c) => c.name(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub variable: Variable,
    pub value: Option<Locale>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Environment(
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Binding,8>")] pub Vec<Binding>,
);
impl Environment {
    pub fn validate(&self, available: &[Locale]) -> Result<()> {
        if self.0.len() > 8 {
            return Err("At most eight locale settings are supported.".into());
        }
        for (n, b) in self.0.iter().enumerate() {
            if self.0[..n].iter().any(|old| old.variable == b.variable) {
                return Err("Duplicate locale setting in saved state.".into());
            }
            if b.value.is_some_and(|v| !available.contains(&v)) {
                return Err(format!(
                    "{} selects an unavailable locale.",
                    b.variable.name()
                ));
            }
        }
        Ok(())
    }
    pub fn set(&mut self, variable: Variable, value: Option<Locale>) {
        if let Some(b) = self.0.iter_mut().find(|b| b.variable == variable) {
            b.value = value;
        } else {
            self.0.push(Binding { variable, value });
        }
    }
    pub fn unset(&mut self, variable: Variable) {
        self.0.retain(|b| b.variable != variable);
    }
    pub fn get(&self, variable: Variable) -> Option<Locale> {
        self.0
            .iter()
            .find(|b| b.variable == variable)
            .and_then(|b| b.value)
    }
    pub fn effective(&self, category: Category) -> Locale {
        self.get(Variable::All)
            .or_else(|| self.get(Variable::Category(category)))
            .or_else(|| self.get(Variable::Lang))
            .unwrap_or(Locale::C)
    }
}
/// Literal systemd-file subset; no shell interpolation, execution or host read.
pub fn configuration(text: &str, available: &[Locale]) -> Result<Environment> {
    if text.len() > MAX_CONFIG || text.contains('\0') {
        return Err(
            "Locale settings exceed the supported file size or contain a zero byte.".into(),
        );
    }
    if available.is_empty()
        || available.len() > 4
        || !available.contains(&Locale::C)
        || available
            .iter()
            .enumerate()
            .any(|(n, l)| available[..n].contains(l))
    {
        return Err("Supply distinct available locales including C.".into());
    }
    let mut result = Environment::default();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fail = |why: &str| format!("Line {}: {why}", number + 1);
        let (name, raw) = line
            .split_once('=')
            .ok_or_else(|| fail("Use NAME=value, not a shell command."))?;
        let variable = Variable::parse(name).map_err(|e| fail(&e))?;
        if variable == Variable::All {
            return Err(fail(
                "LC_ALL is a process override; it is not allowed in /etc/locale.conf. Choose LANG or a category.",
            ));
        }
        if result.0.iter().any(|b| b.variable == variable) {
            return Err(fail(
                "This simulator's file subset requires one assignment per setting; remove the duplicate.",
            ));
        }
        let raw = raw.trim();
        let raw = if raw.starts_with('#') { "" } else { raw };
        let value = if raw.starts_with(['\'', '"']) {
            let quote = raw.chars().next().unwrap();
            let tail = &raw[1..];
            let end = tail
                .find(quote)
                .ok_or_else(|| fail("Close the quoted value."))?;
            if !tail[end + 1..].trim().is_empty() && !tail[end + 1..].trim_start().starts_with('#')
            {
                return Err(fail(
                    "Only a literal value and optional comment are supported.",
                ));
            }
            &tail[..end]
        } else {
            raw.split_once(" #").map_or(raw, |(s, _)| s.trim_end())
        };
        let value = if value.is_empty() {
            None
        } else {
            Some(Locale::parse(value).map_err(|e| fail(&e))?)
        };
        result.set(variable, value);
    }
    result.validate(available)?;
    Ok(result)
}
/// Original demonstration application's fixed-point display, not a fake utility.
pub fn report_number(cents: i64, environment: &Environment) -> Result<String> {
    if cents.unsigned_abs() > 100_000_000 {
        return Err("Sample amount exceeds the finite report range.".into());
    }
    let amount = cents.unsigned_abs();
    Ok(format!(
        "{}{}{}{:02}",
        if cents < 0 { "-" } else { "" },
        amount / 100,
        environment.effective(Category::Numeric).decimal(),
        amount % 100
    ))
}

pub mod encoding;
#[cfg(test)]
mod tests;

pub mod virtual_files;

pub mod command;

pub mod sample_report;

/// UTF-8 text where valid; otherwise a lossless escaped byte view. This is a
/// display representation, never a conversion of the saved virtual document.
pub fn readable(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        text.chars()
            .map(|c| {
                if c.is_control() && !matches!(c, '\n' | '\t') {
                    format!("\\u{{{:x}}}", c as u32)
                } else {
                    c.to_string()
                }
            })
            .collect()
    } else {
        bytes
            .iter()
            .map(|b| match *b {
                b'\n' => "\n".into(),
                b'\t' => "\t".into(),
                32..=126 => char::from(*b).to_string(),
                _ => format!("\\x{b:02x}"),
            })
            .collect()
    }
}
