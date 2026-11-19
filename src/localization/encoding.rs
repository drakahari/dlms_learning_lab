//! Bounded literal character conversion; no host codec, gconv, files or process.
use serde::{Deserialize, Serialize};
pub const MAX_INPUT: usize = 16 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    Ascii,
    Latin1,
    Utf8,
}
impl Encoding {
    pub fn parse(name: &str) -> std::result::Result<Self, String> {
        match name.to_ascii_uppercase().as_str() {
            "ASCII"|"US-ASCII"|"ANSI_X3.4-1968"=>Ok(Self::Ascii),
            "ISO-8859-1"|"ISO8859-1"|"LATIN1"=>Ok(Self::Latin1),
            "UTF-8"|"UTF8"=>Ok(Self::Utf8),
            _=>Err("Supported encodings: ASCII, ISO-8859-1 and UTF-8. No transliteration, discard-invalid flags or other codecs are simulated.".into()),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    InvalidInput,
    IncompleteInput,
    Unrepresentable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub position: usize,
    pub kind: FailureKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conversion {
    pub bytes: Vec<u8>,
    pub failure: Option<Failure>,
}
/// Keep the prefix actually convertible before the first failure. Callers must
/// expose the nonzero result and must not grade partial output as a valid file.
pub fn convert(
    input: &[u8],
    from: Encoding,
    to: Encoding,
) -> std::result::Result<Conversion, String> {
    if input.len() > MAX_INPUT {
        return Err("Input exceeds the16KiB conversion limit; no conversion started.".into());
    }
    let (characters, decode_failure): (Vec<(usize, char)>, Option<Failure>) = match from {
        Encoding::Latin1 => (
            input
                .iter()
                .enumerate()
                .map(|(n, b)| (n, char::from(*b)))
                .collect(),
            None,
        ),
        Encoding::Ascii => {
            let end = input.iter().position(|b| *b > 127).unwrap_or(input.len());
            (
                input[..end]
                    .iter()
                    .enumerate()
                    .map(|(n, b)| (n, char::from(*b)))
                    .collect(),
                (end < input.len()).then_some(Failure {
                    position: end,
                    kind: FailureKind::InvalidInput,
                }),
            )
        }
        Encoding::Utf8 => match std::str::from_utf8(input) {
            Ok(text) => (text.char_indices().collect(), None),
            Err(error) => {
                let end = error.valid_up_to();
                let text = std::str::from_utf8(&input[..end])
                    .map_err(|_| "Internal UTF-8 prefix invariant failed.")?;
                (
                    text.char_indices().collect(),
                    Some(Failure {
                        position: end,
                        kind: if error.error_len().is_some() {
                            FailureKind::InvalidInput
                        } else {
                            FailureKind::IncompleteInput
                        },
                    }),
                )
            }
        },
    };
    let mut bytes = Vec::with_capacity(input.len().saturating_mul(2));
    for (position, c) in characters {
        match to {
            Encoding::Ascii if c as u32 <= 127 => bytes.push(c as u8),
            Encoding::Latin1 if c as u32 <= 255 => bytes.push(c as u8),
            Encoding::Utf8 => {
                let mut buffer = [0; 4];
                bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            }
            _ => {
                return Ok(Conversion {
                    bytes,
                    failure: Some(Failure {
                        position,
                        kind: FailureKind::Unrepresentable,
                    }),
                });
            }
        }
    }
    Ok(Conversion {
        bytes,
        failure: decode_failure,
    })
}
