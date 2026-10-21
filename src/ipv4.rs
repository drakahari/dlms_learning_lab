//! Pure IPv4 teaching mathematics and saved lesson metadata; no host I/O.
use crate::engine::{self, State};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Section {
    #[default]
    Original,
    Network,
    Broadcast,
    Usable,
}
impl Section {
    pub const ALL: [Self; 4] = [Self::Original, Self::Network, Self::Broadcast, Self::Usable];
    pub fn index(self) -> usize {
        match self {
            Self::Original => 0,
            Self::Network => 1,
            Self::Broadcast => 2,
            Self::Usable => 3,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Original => "Original IP",
            Self::Network => "Network address",
            Self::Broadcast => "Broadcast address",
            Self::Usable => "Usable range",
        }
    }
    pub fn result_index(self) -> Option<usize> {
        match self {
            Self::Original => None,
            Self::Network => Some(0),
            Self::Broadcast => Some(1),
            Self::Usable => Some(2),
        }
    }
    pub fn legacy_step(self) -> usize {
        match self {
            Self::Original => 0,
            Self::Network | Self::Broadcast => 1,
            Self::Usable => 2,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exercise {
    pub ip: Ipv4Addr,
    pub prefix: u8,
    pub answers: [String; 4],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub section: Section,
    pub original_section: Section,
    pub variant_section: Section,
    pub submitted: [bool; 3],
    pub revealed: [bool; 3],
    pub sandbox_host: u32,
    pub variant: Option<Exercise>,
    pub use_variant: bool,
    pub sequence: u64,
    pub variant_submitted: [bool; 3],
    pub variant_revealed: [bool; 3],
}
impl Lesson {
    pub fn from_legacy(ip: Ipv4Addr, prefix: u8, step: usize) -> Self {
        let section = match step {
            0 => Section::Original,
            1 => Section::Network,
            _ => Section::Usable,
        };
        Self {
            section,
            original_section: section,
            variant_section: Section::Original,
            submitted: [false; 3],
            revealed: [false; 3],
            sandbox_host: u32::from(ip) & !mask(prefix).unwrap_or(0),
            variant: None,
            use_variant: false,
            sequence: 0,
            variant_submitted: [false; 3],
            variant_revealed: [false; 3],
        }
    }
    pub fn gates(&self) -> ([bool; 3], [bool; 3]) {
        if self.use_variant {
            (self.variant_submitted, self.variant_revealed)
        } else {
            (self.submitted, self.revealed)
        }
    }
    pub fn submit(&mut self, e: &Exercise) {
        let (_, revealed) = self.gates();
        let submitted = [
            !e.answers[0].trim().is_empty(),
            !e.answers[1].trim().is_empty(),
            !e.answers[2].trim().is_empty() && !e.answers[3].trim().is_empty(),
        ];
        if self.use_variant {
            for (i, v) in submitted.iter().enumerate() {
                self.variant_submitted[i] |= *v;
            }
            self.variant_revealed = revealed;
        } else {
            for (i, v) in submitted.iter().enumerate() {
                self.submitted[i] |= *v;
            }
        }
    }
    pub fn can_reveal(&self, section: Section, e: &Exercise) -> bool {
        section
            .result_index()
            .is_some_and(|i| self.gates().0[i] || e.solved())
    }
    pub fn reveal(&mut self, section: Section, e: &Exercise) -> bool {
        if !self.can_reveal(section, e) {
            return false;
        }
        let i = section.result_index().unwrap();
        if self.use_variant {
            self.variant_revealed[i] = true;
            self.variant_submitted[i] = true;
        } else {
            self.revealed[i] = true;
            self.submitted[i] = true;
        }
        true
    }
    pub fn worked_example_reviewed(&self) -> bool {
        self.revealed.iter().all(|v| *v)
    }
    pub fn fresh(&mut self) -> bool {
        if !self.worked_example_reviewed() {
            return false;
        }
        if !self.use_variant {
            self.original_section = self.section;
        }
        self.sequence = self.sequence.saturating_add(1);
        let e = generated(self.sequence);
        self.sandbox_host = u32::from(e.ip) & !mask(e.prefix).unwrap();
        self.variant = Some(e);
        self.use_variant = true;
        self.variant_submitted = [false; 3];
        self.variant_revealed = [false; 3];
        self.section = Section::Original;
        self.variant_section = Section::Original;
        true
    }
}
impl Exercise {
    pub fn from_state(state: &State) -> Option<Self> {
        if let State::Subnet {
            ip,
            prefix,
            answers,
        } = state
        {
            Some(Self {
                ip: *ip,
                prefix: *prefix,
                answers: answers.clone(),
            })
        } else {
            None
        }
    }
    pub fn expected(&self) -> Option<[Ipv4Addr; 4]> {
        if self.prefix > 30 {
            return None;
        }
        let (n, b) = engine::bounds(self.ip, self.prefix)?;
        Some([n, b, n + 1, b - 1].map(Ipv4Addr::from))
    }
    pub fn solved(&self) -> bool {
        self.expected().is_some_and(|want| {
            self.answers
                .iter()
                .zip(want)
                .all(|(a, w)| a.trim().parse::<Ipv4Addr>() == Ok(w))
        })
    }
    pub fn capacity(&self) -> Option<u64> {
        (self.prefix <= 30).then(|| 1_u64 << (32 - self.prefix))
    }
}
pub fn mask(prefix: u8) -> Option<u32> {
    if prefix > 30 {
        None
    } else {
        Some(if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        })
    }
}
pub fn sandbox_address(e: &Exercise, host: u32) -> Option<Ipv4Addr> {
    let m = mask(e.prefix)?;
    Some(Ipv4Addr::from((u32::from(e.ip) & m) | (host & !m)))
}
pub fn toggle_host(e: &Exercise, host: u32, position: u8) -> Option<u32> {
    if position >= 32 || position < e.prefix {
        return None;
    }
    Some(host ^ (1_u32 << (31 - position)))
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Diagnosis {
    Blank,
    Malformed,
    Correct,
    HostInsteadOfNetwork,
    WrongBoundary,
    ReservedHost,
    OutsideSubnet,
    WrongHostEnd,
}
pub fn diagnose(e: &Exercise, field: usize) -> Diagnosis {
    if field >= 4 {
        return Diagnosis::Malformed;
    }
    let answer = e.answers[field].trim();
    if answer.is_empty() {
        return Diagnosis::Blank;
    }
    let Ok(ip) = answer.parse::<Ipv4Addr>() else {
        return Diagnosis::Malformed;
    };
    let Some(w) = e.expected() else {
        return Diagnosis::Malformed;
    };
    if ip == w[field] {
        return Diagnosis::Correct;
    }
    let v = u32::from(ip);
    let n = u32::from(w[0]);
    let b = u32::from(w[1]);
    if field == 0 && v > n && v < b {
        return Diagnosis::HostInsteadOfNetwork;
    }
    if field >= 2 && (v == n || v == b) {
        return Diagnosis::ReservedHost;
    }
    if field >= 2 && (v < n || v > b) {
        return Diagnosis::OutsideSubnet;
    }
    if field < 2 {
        Diagnosis::WrongBoundary
    } else {
        Diagnosis::WrongHostEnd
    }
}
pub fn feedback(e: &Exercise, field: usize) -> (Diagnosis, String) {
    let d = diagnose(e, field);
    let text=match d{
        Diagnosis::Blank=>"No prediction yet. Enter an IPv4 address; a blank is different from a wrong answer.".into(),
        Diagnosis::Malformed=>"This is not an IPv4 address. Use four decimal numbers from 0 to 255, separated by dots, without a /prefix.".into(),
        Diagnosis::Correct=>"Correct. This address meets the current objective; you can still change it and retry.".into(),
        Diagnosis::HostInsteadOfNetwork=>"You entered a host inside this subnet, not its network address. Keep every network bit fixed and make every host bit 0. The original interface IP is usually a host, not the subnet’s first boundary.".into(),
        Diagnosis::WrongBoundary=>{let h=32-e.prefix;let partial=e.prefix%8;let block=if partial==0{256}else{1_u32<<(8-partial)};format!("This is the wrong block boundary. A /{} block has 2^{h} = {} addresses. In the first octet containing host bits, boundaries are multiples of {block}; later host octets also change. Keep the fixed network bits and set all host bits {}. Don’t round each octet independently.",e.prefix,e.capacity().unwrap_or(0),if field==0{"to 0"}else{"to 1"})},
        Diagnosis::ReservedHost=>"That is a reserved boundary, not a usable host in this conventional subnet. The first host is one after the network; the last is one before broadcast.".into(),
        Diagnosis::OutsideSubnet=>"That address lies outside this subnet. Keep the network bits from the original IP; usable hosts must remain between its two boundaries.".into(),
        Diagnosis::WrongHostEnd=>"This is not the requested end of the usable range. First usable is network + 1; last usable is broadcast - 1, including any carry between octets.".into(),
    };
    (d, text)
}
pub fn worked(e: &Exercise, section: Section) -> String {
    let Some(w) = e.expected() else {
        return "This exercise excludes /31 and /32.".into();
    };
    let m = Ipv4Addr::from(mask(e.prefix).unwrap());
    let h = 32 - e.prefix;
    match section {
        Section::Original => format!(
            "{} is the interface IP; /{} is the prefix length. The subnet mask {m} has {} network-selection 1 bits followed by {h} host-selection 0 bits. The mask is not a second interface address.",
            e.ip, e.prefix, e.prefix
        ),
        Section::Network => format!(
            "Keep the first {} bits from {}. Turn all {h} host bits to 0. Bitwise IP AND mask ({m}) gives {}. All-zero host bits name this whole subnet; this boundary is reserved in our conventional broadcast-subnet exercise.",
            e.prefix, e.ip, w[0]
        ),
        Section::Broadcast => format!(
            "Keep exactly the same {} network bits. Turn all {h} host bits to 1. The last address in the block is {}. All-one host bits identify broadcast to the subnet; they are not an ordinary host address here.",
            e.prefix, w[1]
        ),
        Section::Usable => format!(
            "The network {} and broadcast {} are reserved. Add one to the network for {}; subtract one from broadcast for {}. This leaves {} usable addresses. Addresses are 32-bit values. This range can span multiple octets; do not assume only the last decimal number varies.",
            w[0],
            w[1],
            w[2],
            w[3],
            e.capacity().unwrap() - 2
        ),
    }
}
/// Deterministic local numerical variants, never extra scenario/catalog entries.
pub fn generated(sequence: u64) -> Exercise {
    const PREFIXES: [u8; 12] = [26, 23, 20, 30, 25, 16, 19, 24, 22, 8, 17, 27];
    let prefix = PREFIXES[sequence.saturating_sub(1) as usize % PREFIXES.len()];
    let x = sequence.wrapping_mul(0x9e3779b97f4a7c15).rotate_left(17);
    let raw = 0x0a00_0000 | ((x as u32) & 0x00ff_ffff);
    let m = mask(prefix).unwrap();
    let usable = (!m as u64) - 1;
    let offset = 1 + (x % usable) as u32;
    Exercise {
        ip: Ipv4Addr::from((raw & m) + offset),
        prefix,
        answers: std::array::from_fn(|_| String::new()),
    }
}
pub fn validate_lesson(l: &Lesson, original: &Exercise) -> bool {
    let active = if l.use_variant {
        let Some(v) = &l.variant else {
            return false;
        };
        v
    } else {
        original
    };
    original.prefix <= 30
        && active.prefix <= 30
        && l.sandbox_host <= !mask(active.prefix).unwrap_or(u32::MAX)
        && l.revealed.iter().zip(l.submitted).all(|(r, s)| !*r || s)
        && l.variant_revealed
            .iter()
            .zip(l.variant_submitted)
            .all(|(r, s)| !*r || s)
        && l.variant.as_ref().is_none_or(|v| {
            l.worked_example_reviewed() && v.prefix <= 30 && v.answers.iter().all(|a| a.len() <= 64)
        })
}
