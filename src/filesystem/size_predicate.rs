//! Opt-in ordinary-file byte lengths and rounded units, never disk allocation.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Blocks512,
    Bytes,
    Words2,
    Kibibytes,
    Mebibytes,
    Gibibytes,
}
impl Unit {
    pub fn bytes(self) -> u64 {
        match self {
            Self::Blocks512 => 512,
            Self::Bytes => 1,
            Self::Words2 => 2,
            Self::Kibibytes => 1024,
            Self::Mebibytes => 1_048_576,
            Self::Gibibytes => 1_073_741_824,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compare {
    Equal,
    Less,
    Greater,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Criterion {
    pub compare: Compare,
    pub quantity: u64,
    pub unit: Unit,
}
impl Criterion {
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.is_empty() || raw.len() > 24 {
            return Err("Size needs a bounded nonnegative quantity, an optional + or - comparison and optional b/c/w/k/M/G unit.".into());
        }
        let (compare, body) = if let Some(v) = raw.strip_prefix('+') {
            (Compare::Greater, v)
        } else if let Some(v) = raw.strip_prefix('-') {
            (Compare::Less, v)
        } else {
            (Compare::Equal, raw)
        };
        let (unit, number) = if body.as_bytes().last().is_some_and(u8::is_ascii_digit) {
            (Unit::Blocks512, body)
        } else {
            let last = body
                .as_bytes()
                .last()
                .copied()
                .ok_or("Size comparison needs a quantity.")?;
            let unit=match last {b'b'=>Unit::Blocks512,b'c'=>Unit::Bytes,b'w'=>Unit::Words2,b'k'=>Unit::Kibibytes,b'M'=>Unit::Mebibytes,b'G'=>Unit::Gibibytes,_=>return Err("Unsupported size unit. Use b (512), c (bytes), w (2), k (1024), M or G; lowercase m and uppercase K are not these units.".into())};
            (unit, &body[..body.len() - 1])
        };
        if number.is_empty() || number.len() > 20 || !number.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Size quantity must contain decimal digits only; + means greater than, - means less than.".into());
        }
        let quantity = number
            .parse()
            .map_err(|_| "Size quantity exceeds the supported integer range.")?;
        Ok(Self {
            compare,
            quantity,
            unit,
        })
    }
    pub fn matches(self, bytes: usize) -> bool {
        let rounded = (bytes as u64).div_ceil(self.unit.bytes());
        match self.compare {
            Compare::Equal => rounded == self.quantity,
            Compare::Less => rounded < self.quantity,
            Compare::Greater => rounded > self.quantity,
        }
    }
}

/// Explicit opt-in only. The original Tree terminal still refuses -size.
pub const HELP: &str = "Opt-in file-size queries: find START [-maxdepth N] -type f [-name QUOTED_PATTERN] -size [+|-]N[b|c|w|k|M|G] [-print]. Size uses content byte length rounded up to the chosen unit: default/b=512, c=1, w=2, k=1024, M=1048576, G=1073741824. +N is strictly greater, -N strictly less, no sign exactly N. Require -type f because directory metadata sizes are not modeled. No allocation/sparse-file measurement, timestamps, -exec/-delete, shell expansion or host filesystem. Other commands retain the disclosed recursive-tree subset.";
pub fn execute(tree: &mut super::Tree, input: &str) -> Result<String, String> {
    if input.len() > 4096 {
        return Err("Command exceeds4096bytes.".into());
    }
    let words = crate::shell::literal_command(input)?;
    if words.first().is_some_and(|w| w == "find") {
        return tree.find_sized(&words[1..])?.terminal();
    }
    if words.as_slice() == ["help"] {
        return Ok(format!("{}\n{HELP}", super::recursive::HELP));
    }
    super::recursive::execute(tree, input).map_err(|e| e.to_string())
}
