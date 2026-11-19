//! Bounded public-identity/known-host parsing, not an SSH or cryptographic engine.
//! No private material, signature verification, hash/fingerprint claim, file I/O,
//! subprocess, socket, key generation or session is implemented here.
use serde::{Deserialize, Serialize};
pub const MAX_TEXT: usize = 16_384;
pub const MAX_ENTRIES: usize = 64;
const MAX_LINE: usize = 1024;
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PREFIX: [u8; 19] = [
    0, 0, 0, 11, b's', b's', b'h', b'-', b'e', b'd', b'2', b'5', b'5', b'1', b'9', 0, 0, 0, 32,
];
/// A correctly shaped public encoding. Curve membership/possession is NOT verified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicKey {
    pub bytes: [u8; 32],
}
impl PublicKey {
    pub fn parse(algorithm: &str, encoded: &str) -> Result<Self, String> {
        if algorithm != "ssh-ed25519" {
            return Err("Unsupported public-key type or options: this activity accepts plain ssh-ed25519 public keys only.".into());
        }
        if encoded.len() != 68 || !encoded.is_ascii() {
            return Err("An ssh-ed25519 public key needs its complete 68-character base64 field; private keys are not accepted.".into());
        }
        let mut wire = [0u8; 51];
        for (chunk, dest) in encoded
            .as_bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(wire.as_chunks_mut::<3>().0.iter_mut())
        {
            let mut value = 0u32;
            for c in chunk {
                let n = ALPHABET.iter().position(|v| v == c).ok_or("Unsupported public-key encoding: use the complete canonical base64 field without padding or shell text.")?;
                value = (value << 6) | n as u32;
            }
            dest.copy_from_slice(&[(value >> 16) as u8, (value >> 8) as u8, value as u8]);
        }
        if wire[..19] != PREFIX {
            return Err("Public-key encoding has the wrong algorithm or length fields; expected an ssh-ed25519 name and 32 public bytes.".into());
        }
        let mut bytes = [0; 32];
        bytes.copy_from_slice(&wire[19..]);
        Ok(Self { bytes })
    }
    pub fn encoded(self) -> String {
        let mut wire = [0u8; 51];
        wire[..19].copy_from_slice(&PREFIX);
        wire[19..].copy_from_slice(&self.bytes);
        let mut result = String::with_capacity(68);
        for c in wire.as_chunks::<3>().0.iter() {
            let value = (u32::from(c[0]) << 16) | (u32::from(c[1]) << 8) | u32::from(c[2]);
            for shift in [18, 12, 6, 0] {
                result.push(ALPHABET[((value >> shift) & 63) as usize] as char);
            }
        }
        result
    }
    pub fn line(self) -> String {
        format!("ssh-ed25519 {}", self.encoded())
    }
}
fn bounded_lines(text: &str) -> Result<Vec<(usize, Vec<&str>)>, String> {
    if text.len() > MAX_TEXT
        || !text.is_ascii()
        || text
            .bytes()
            .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'))
    {
        return Err("Unsupported public-key document: use at most 16384 bytes of ASCII text, without control characters or private-key material.".into());
    }
    let mut rows = vec![];
    for (n, line) in text.lines().enumerate() {
        if line.len() > MAX_LINE {
            return Err(format!(
                "Line {} exceeds this activity's 1024-byte line limit.",
                n + 1
            ));
        }
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if rows.len() == MAX_ENTRIES {
            return Err("Public-key document exceeds 64 entries.".into());
        }
        rows.push((n + 1, line.split_whitespace().collect()));
    }
    Ok(rows)
}
/// Literal lower-case hostname or canonical IPv4 only. No pattern, hash or port syntax.
pub fn target(name: &str) -> Result<&str, String> {
    let unsupported = || {
        "Unsupported host spelling: use a supplied lower-case literal hostname or canonical IPv4 address, without patterns, hashes, ports or a trailing dot.".to_string()
    };
    if name.is_empty() || name.len() > 253 || !name.is_ascii() {
        return Err(unsupported());
    }
    if let Ok(ip) = name.parse::<std::net::Ipv4Addr>() {
        return if ip.to_string() == name {
            Ok(name)
        } else {
            Err(unsupported())
        };
    }
    if name.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        || name.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || !label.as_bytes()[0].is_ascii_lowercase()
                    && !label.as_bytes()[0].is_ascii_digit()
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
    {
        return Err(unsupported());
    }
    Ok(name)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEntry {
    pub names: Vec<String>,
    pub key: PublicKey,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownHosts {
    pub entries: Vec<HostEntry>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust {
    UnknownHost,
    ChangedKey,
    Trusted,
}
impl Trust {
    pub fn explanation(self) -> &'static str {
        match self {
            Self::UnknownHost => {
                "No saved public key identifies this host. Compare the supplied trusted identity before adding an entry; nothing is automatically trusted."
            }
            Self::ChangedKey => {
                "This host has a saved key, but the presented public key differs. Stop and verify the intended server identity; do not bypass the mismatch."
            }
            Self::Trusted => {
                "The presented public key matches a saved entry for this host. This checks supplied identity data, not a real SSH handshake or signature."
            }
        }
    }
}
impl KnownHosts {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut entries = vec![];
        for (n, row) in bounded_lines(text)? {
            if row.len() < 3 {
                return Err(format!(
                    "Line {n}: use HOST ssh-ed25519 PUBLIC_KEY, with an optional comment."
                ));
            }
            let names: Vec<_> = row[0].split(',').collect();
            if names.len() > 8 {
                return Err(format!(
                    "Line {n}: at most eight literal host aliases per entry are supported."
                ));
            }
            for name in &names {
                target(name).map_err(|e| format!("Line {n}: {e}"))?;
            }
            let key = PublicKey::parse(row[1], row[2]).map_err(|e| format!("Line {n}: {e}"))?;
            entries.push(HostEntry {
                names: names.into_iter().map(str::to_owned).collect(),
                key,
            });
        }
        Ok(Self { entries })
    }
    /// Strict, noninteractive supplied-host comparison; never adds or changes trust.
    pub fn check(&self, host: &str, presented: PublicKey) -> Result<Trust, String> {
        target(host)?;
        let mut found = false;
        for e in &self.entries {
            if e.names.iter().any(|n| n == host) {
                found = true;
                if e.key == presented {
                    return Ok(Trust::Trusted);
                }
            }
        }
        Ok(if found {
            Trust::ChangedKey
        } else {
            Trust::UnknownHost
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedKeys {
    pub keys: Vec<PublicKey>,
}
impl AuthorizedKeys {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut keys = vec![];
        for (n, row) in bounded_lines(text)? {
            if row.len() < 2 {
                return Err(format!(
                    "Line {n}: use ssh-ed25519 PUBLIC_KEY, with an optional comment. No private key is needed."
                ));
            }
            keys.push(PublicKey::parse(row[0], row[1]).map_err(|e| format!("Line {n}: {e}"))?);
        }
        Ok(Self { keys })
    }
    /// Membership only. The later access model must separately check possession,
    /// policy, ownership, read/traversal permissions and host trust.
    pub fn contains(&self, key: PublicKey) -> bool {
        self.keys.contains(&key)
    }
}
