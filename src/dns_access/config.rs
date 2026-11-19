//! Proposed strict named.conf excerpt parser. No includes, files, sockets or host calls.
use super::acl::Acl;
use crate::dns;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_CONFIG: usize = 8192;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZonePolicy {
    #[serde(deserialize_with = "crate::cron::text::<_,254>")]
    pub origin: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub file: String,
    pub transfer: Option<Acl>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub query: Acl,
    pub recursion: bool,
    pub recursive: Acl,
    pub cache: Acl,
    pub transfer: Acl,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,ZonePolicy,4>")]
    pub zones: Vec<ZonePolicy>,
}
impl Policy {
    pub fn validate(&self, known: &BTreeMap<String, String>) -> Result<(), String> {
        if known.is_empty() || known.len() > 4 || self.zones.len() != known.len() {
            return Err("Declare each supplied primary zone exactly once (at most four).".into());
        }
        for acl in [&self.query, &self.recursive, &self.cache, &self.transfer] {
            acl.validate()?;
        }
        let mut seen = BTreeSet::new();
        for zone in &self.zones {
            if dns::canonical_name(&zone.origin).map_err(|e| e.message)? != zone.origin
                || !seen.insert(zone.origin.clone())
                || known.get(&zone.origin) != Some(&zone.file)
            {
                return Err("Zone names and virtual file names must match the supplied zones; includes or foreign files are unsupported.".into());
            }
            if let Some(acl) = &zone.transfer {
                acl.validate()?;
            }
        }
        Ok(())
    }
    pub fn transfer_for(&self, origin: &str) -> Result<&Acl, String> {
        let name = dns::canonical_name(origin).map_err(|e| e.message)?;
        let zone = self
            .zones
            .iter()
            .find(|z| z.origin == name)
            .ok_or("No supplied zone has that name.")?;
        Ok(zone.transfer.as_ref().unwrap_or(&self.transfer))
    }
    pub fn parse(text: &str, known: &BTreeMap<String, String>) -> Result<Self, String> {
        let mut p = Parser {
            tokens: tokens(text)?,
            at: 0,
        };
        let mut options = None;
        let mut zones = vec![];
        while let Some(t) = p.peek() {
            match t {
                Token::Word("options") => {
                    p.at+=1;
                    if options.is_some(){ return Err("Declare one options block, not two.".into()); }
                    options=Some(p.options()?);
                }
                Token::Word("zone") => {
                    p.at+=1;
                    if zones.len()==4{return Err("At most four supplied zones are supported.".into());}
                    zones.push(p.zone()?);
                }
                _=>return Err("Unsupported configuration block. Use one explicit options block and the supplied primary zones; no includes, named ACLs, views or keys.".into()),
            }
        }
        let (query, recursion, recursive, cache, transfer) =
            options.ok_or("Add the explicit options block before applying configuration.")?;
        zones.sort_by(|a, b| a.origin.cmp(&b.origin));
        let result = Self {
            query,
            recursion,
            recursive,
            cache,
            transfer,
            zones,
        };
        result.validate(known)?;
        Ok(result)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'a> {
    Word(&'a str),
    Quoted(&'a str),
    Open,
    Close,
    Semi,
}
fn tokens(text: &str) -> Result<Vec<Token<'_>>, String> {
    if text.len() > MAX_CONFIG {
        return Err("Configuration exceeds 8 KiB.".into());
    }
    if !text.is_ascii()
        || text
            .bytes()
            .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'))
    {
        return Err(
            "Use printable ASCII, tabs and line endings in this configuration subset.".into(),
        );
    }
    let b = text.as_bytes();
    let mut at = 0;
    let mut out = vec![];
    while at < b.len() {
        if b[at].is_ascii_whitespace() {
            at += 1;
            continue;
        }
        if b[at] == b'#' || b[at..].starts_with(b"//") {
            while at < b.len() && b[at] != b'\n' {
                at += 1;
            }
            continue;
        }
        if b[at..].starts_with(b"/*") {
            at += 2;
            let end = text[at..]
                .find("*/")
                .ok_or("Unclosed /* comment; add */.")?;
            at += end + 2;
            continue;
        }
        let token = match b[at] {
            b'{' => {
                at += 1;
                Token::Open
            }
            b'}' => {
                at += 1;
                Token::Close
            }
            b';' => {
                at += 1;
                Token::Semi
            }
            b'"' => {
                at += 1;
                let start = at;
                while at < b.len() && b[at] != b'"' {
                    if b[at] == b'\\' || b[at].is_ascii_control() {
                        return Err("Quoted names must be single-line literal ASCII without escapes in this subset.".into());
                    }
                    at += 1;
                }
                if at == b.len() {
                    return Err("Unclosed quoted name; add a closing double quote.".into());
                }
                let value = &text[start..at];
                at += 1;
                Token::Quoted(value)
            }
            _ => {
                let start = at;
                while at < b.len()
                    && !b[at].is_ascii_whitespace()
                    && !b"{};\"#".contains(&b[at])
                    && !b[at..].starts_with(b"//")
                    && !b[at..].starts_with(b"/*")
                {
                    at += 1;
                }
                if start == at {
                    return Err("Unsupported character in configuration.".into());
                }
                Token::Word(&text[start..at])
            }
        };
        if out.len() == 2048 {
            return Err("Configuration exceeds the 2048-token limit.".into());
        }
        out.push(token);
    }
    Ok(out)
}
struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    at: usize,
}
impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.at).copied()
    }
    fn take(&mut self) -> Result<Token<'a>, String> {
        let value = self
            .peek()
            .ok_or("Configuration ends before the current statement is complete.")?;
        self.at += 1;
        Ok(value)
    }
    fn expect(&mut self, expected: Token<'a>) -> Result<(), String> {
        if self.take()? != expected {
            return Err(format!(
                "Expected {expected:?}; check braces and semicolons."
            ));
        }
        Ok(())
    }
    fn word(&mut self) -> Result<&'a str, String> {
        match self.take()? {
            Token::Word(s) => Ok(s),
            _ => Err("Expected a configuration keyword or address.".into()),
        }
    }
    fn quoted(&mut self) -> Result<&'a str, String> {
        match self.take()? {
            Token::Quoted(s) => Ok(s),
            _ => Err("Put the complete supplied zone/file name in double quotes.".into()),
        }
    }
    fn acl(&mut self) -> Result<Acl, String> {
        self.expect(Token::Open)?;
        let mut nets = vec![];
        let mut keyword = None;
        while self.peek() != Some(Token::Close) {
            let word = self.word()?;
            if matches!(word, "any" | "none") {
                if keyword.is_some() || !nets.is_empty() {
                    return Err("Use any or none alone; mixing keyword and network entries is unsupported here.".into());
                }
                keyword = Some(word);
            } else {
                if keyword.is_some() {
                    return Err("Use any or none alone in this subset.".into());
                }
                if nets.len() == 32 {
                    return Err("Address-match lists support at most 32 entries.".into());
                }
                nets.push(super::acl::Network::parse(word)?);
            }
            self.expect(Token::Semi)?;
        }
        self.expect(Token::Close)?;
        self.expect(Token::Semi)?;
        Ok(match keyword {
            Some("any") => Acl::Any,
            Some("none") => Acl::None,
            _ => Acl::Networks { networks: nets },
        })
    }
    fn options(&mut self) -> Result<(Acl, bool, Acl, Acl, Acl), String> {
        self.expect(Token::Open)?;
        let (mut query, mut recursion, mut recursive, mut cache, mut transfer) =
            (None, None, None, None, None);
        let mut seen = BTreeSet::new();
        while self.peek() != Some(Token::Close) {
            let key = self.word()?;
            if !seen.insert(key) {
                return Err(format!(
                    "The {key} setting is repeated; keep one explicit value."
                ));
            }
            match key {
                "allow-query" => query = Some(self.acl()?),
                "allow-recursion" => recursive = Some(self.acl()?),
                "allow-query-cache" => cache = Some(self.acl()?),
                "allow-transfer" => transfer = Some(self.acl()?),
                "recursion" => {
                    recursion = Some(match self.word()? {
                        "yes" => true,
                        "no" => false,
                        _ => {
                            return Err(
                                "Use recursion yes; or recursion no; in this subset.".into()
                            );
                        }
                    });
                    self.expect(Token::Semi)?;
                }
                _ => {
                    return Err(format!(
                        "Unsupported options setting {key}. This activity supports only explicit recursion and query/cache/transfer access lists."
                    ));
                }
            }
        }
        self.expect(Token::Close)?;
        self.expect(Token::Semi)?;
        Ok((
            query.ok_or("Add explicit allow-query; defaults are not inferred.")?,
            recursion.ok_or("Add explicit recursion yes/no; defaults are not inferred.")?,
            recursive.ok_or("Add explicit allow-recursion; defaults are not inferred.")?,
            cache.ok_or("Add explicit allow-query-cache; defaults are not inferred.")?,
            transfer.ok_or("Add explicit allow-transfer; defaults are not inferred.")?,
        ))
    }
    fn zone(&mut self) -> Result<ZonePolicy, String> {
        let origin = dns::canonical_name(self.quoted()?).map_err(|e| e.message)?;
        self.expect(Token::Open)?;
        let (mut primary, mut file, mut transfer) = (false, None, None);
        let mut seen = BTreeSet::new();
        while self.peek() != Some(Token::Close) {
            let key = self.word()?;
            if !seen.insert(key) {
                return Err(format!("Repeated zone setting {key}."));
            }
            match key {
                "type" => {
                    if self.word()? != "primary" {
                        return Err(
                            "Only supplied primary zone declarations are supported in this editor."
                                .into(),
                        );
                    }
                    primary = true;
                    self.expect(Token::Semi)?;
                }
                "file" => {
                    file = Some(self.quoted()?.to_owned());
                    self.expect(Token::Semi)?;
                }
                "allow-transfer" => transfer = Some(self.acl()?),
                _ => {
                    return Err(format!(
                        "Unsupported zone setting {key}. Use type primary, supplied file and optional allow-transfer."
                    ));
                }
            }
        }
        self.expect(Token::Close)?;
        self.expect(Token::Semi)?;
        if !primary {
            return Err("Declare type primary for each supplied zone.".into());
        }
        Ok(ZonePolicy {
            origin,
            file: file.ok_or("Declare the supplied virtual zone file.")?,
            transfer,
        })
    }
}

mod transfer_fragment;
pub use transfer_fragment::parse_transfer_fragment;
