//! Conventional IPv4 teaching calculations; no networking or host execution.
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Blocks,
    Boundaries,
    CrossOctet,
    Aligned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Problem {
    pub ip: Ipv4Addr,
    pub prefix: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calculation {
    pub mask: Ipv4Addr,
    pub octet: usize,
    pub mask_octet: u8,
    pub block: u16,
    pub start: u8,
    pub end: u8,
    pub network: Ipv4Addr,
    pub broadcast: Ipv4Addr,
    pub first: Ipv4Addr,
    pub last: Ipv4Addr,
    pub next: Option<Ipv4Addr>,
}
impl Problem {
    pub fn calculate(self) -> Result<Calculation, String> {
        if !(1..=30).contains(&self.prefix) {
            return Err("Use /1 through /30 here. /31 point-to-point links and /32 host routes are special cases; the usual network/broadcast reservations do not apply universally.".into());
        }
        let (n, b) = crate::engine::bounds(self.ip, self.prefix).unwrap();
        let mask = Ipv4Addr::from(u32::MAX << (32 - self.prefix));
        // At an octet boundary choose the last network octet, not an all-zero host octet.
        let octet = usize::from((self.prefix - 1) / 8);
        let m = mask.octets()[octet];
        let block = 256 - u16::from(m);
        let start = (u16::from(self.ip.octets()[octet]) / block * block) as u8;
        Ok(Calculation {
            mask,
            octet,
            mask_octet: m,
            block,
            start,
            end: (u16::from(start) + block - 1) as u8,
            network: n.into(),
            broadcast: b.into(),
            first: (n + 1).into(),
            last: (b - 1).into(),
            next: b.checked_add(1).map(Ipv4Addr::from),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub address: String,
    pub prefix: String,
    pub answers: [String; 6],
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub problem: Problem,
    pub stage: Stage,
    pub view: crate::script_teaching::View,
    pub answers: [String; 6],
    pub checked: Option<[String; 6]>,
    pub example: Snapshot,
    pub example_checked: Option<Snapshot>,
    pub hints: u8,
}
impl Session {
    pub fn new(problem: Problem, stage: Stage) -> Self {
        let p = stage.practice();
        let mut starter: [String; 6] = Default::default();
        let supplied = stage.fields(p).expect("fixed practice problem");
        starter[0] = supplied[0].expected.clone();
        if stage == Stage::Blocks {
            starter[1] = supplied[1].expected.clone();
        }
        Self {
            problem,
            stage,
            view: crate::script_teaching::View::Introduction,
            answers: Default::default(),
            checked: None,
            example: Snapshot {
                address: p.ip.to_string(),
                prefix: p.prefix.to_string(),
                answers: starter,
            },
            example_checked: None,
            hints: 0,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.problem.calculate()?;
        let text = |s: &String| s.len() <= 256 && !s.contains('\0');
        let snap = |s: &Snapshot| text(&s.address) && text(&s.prefix) && s.answers.iter().all(text);
        if self.hints > 3
            || !self.answers.iter().all(text)
            || self.checked.as_ref().is_some_and(|a| !a.iter().all(text))
            || !snap(&self.example)
            || self.example_checked.as_ref().is_some_and(|a| !snap(a))
        {
            return Err("Subnet lesson draft exceeds its supported limits".into());
        }
        Ok(())
    }
    pub fn example_problem(&self) -> Result<Problem, String> {
        let p = Problem {
            ip: self
                .example
                .address
                .trim()
                .parse()
                .map_err(|_| "Enter four decimal numbers, each 0–255, separated by dots.")?,
            prefix: self
                .example
                .prefix
                .trim()
                .parse()
                .map_err(|_| "Enter a prefix number from 1 through 30, without the slash.")?,
        };
        p.calculate()?;
        Ok(p)
    }
}
#[derive(Clone, Debug)]
pub struct Field {
    pub label: &'static str,
    pub expected: String,
    pub numeric: bool,
}
impl Stage {
    pub fn practice(self) -> Problem {
        match self {
            Self::Blocks => Problem {
                ip: Ipv4Addr::new(198, 51, 100, 141),
                prefix: 26,
            },
            Self::Boundaries => Problem {
                ip: Ipv4Addr::new(192, 0, 2, 78),
                prefix: 27,
            },
            Self::CrossOctet => Problem {
                ip: Ipv4Addr::new(172, 22, 46, 200),
                prefix: 20,
            },
            Self::Aligned => Problem {
                ip: Ipv4Addr::new(10, 35, 255, 254),
                prefix: 16,
            },
        }
    }
    pub fn worked(self) -> Problem {
        match self {
            Self::Blocks | Self::Boundaries => Problem {
                ip: Ipv4Addr::new(192, 168, 10, 77),
                prefix: 26,
            },
            Self::CrossOctet => Problem {
                ip: Ipv4Addr::new(172, 16, 70, 9),
                prefix: 20,
            },
            Self::Aligned => Problem {
                ip: Ipv4Addr::new(10, 12, 9, 254),
                prefix: 24,
            },
        }
    }
    pub fn objective(self) -> &'static str {
        match self {
            Self::Blocks => {
                "Use the mask to find the block size, then locate this address inside a block."
            }
            Self::Boundaries => {
                "Find the network, broadcast, usable hosts and next subnet from one block."
            }
            Self::CrossOctet => {
                "Follow a subnet boundary in the third octet and account for the whole fourth octet."
            }
            Self::Aligned => {
                "Find subnet boundaries when the prefix ends exactly between two octets."
            }
        }
    }
    pub fn fields(self, p: Problem) -> Result<Vec<Field>, String> {
        let c = p.calculate()?;
        let number = |label, expected: String| Field {
            label,
            expected,
            numeric: true,
        };
        let address = |label, expected: String| Field {
            label,
            expected,
            numeric: false,
        };
        if self == Self::Blocks {
            return Ok(vec![
                address("Subnet mask", c.mask.to_string()),
                number("Relevant mask octet", c.mask_octet.to_string()),
                number("Block size", c.block.to_string()),
                number("Containing block starts", c.start.to_string()),
                number("Containing block ends", c.end.to_string()),
            ]);
        }
        let mut f = vec![];
        if matches!(self, Self::CrossOctet | Self::Aligned) {
            f.push(number("Relevant octet (1–4)", (c.octet + 1).to_string()));
        }
        f.extend([
            address("Network address", c.network.to_string()),
            address("Broadcast address", c.broadcast.to_string()),
            address("First usable host", c.first.to_string()),
            address("Last usable host", c.last.to_string()),
            address(
                "Next subnet",
                c.next.map_or("none".into(), |a| a.to_string()),
            ),
        ]);
        Ok(f)
    }
}
pub fn checks(stage: Stage, p: Problem, answers: &[String; 6]) -> Vec<(bool, String)> {
    let Ok(fields) = stage.fields(p) else {
        return vec![(
            false,
            "This prefix is outside this conventional-subnet exercise.".into(),
        )];
    };
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let a = answers[i].trim();
            let pass = if f.numeric {
                a.parse::<u64>().ok() == f.expected.parse::<u64>().ok()
            } else if f.expected == "none" {
                a.eq_ignore_ascii_case("none")
            } else {
                a.parse::<Ipv4Addr>().ok() == f.expected.parse::<Ipv4Addr>().ok()
            };
            (
                pass,
                if pass {
                    format!("{} · Correct", f.label)
                } else {
                    feedback(stage, p, f, a)
                },
            )
        })
        .collect()
}
fn feedback(stage: Stage, p: Problem, f: &Field, a: &str) -> String {
    let lead = format!("{}: ", f.label);
    let message = if a.is_empty() {
        "This field is blank. Enter your calculation, then check again."
    } else if f.numeric && a.parse::<u64>().is_err() {
        "Enter a whole number, not an address or a calculation such as 256−192."
    } else if !f.numeric && f.expected != "none" && a.parse::<Ipv4Addr>().is_err() {
        "Use four decimal numbers from 0 to 255, separated by dots."
    } else if f.label == "Subnet mask" && a == p.ip.to_string() {
        "That is the computer’s address. The mask describes which bits identify the network."
    } else if f.label == "Block size" {
        "Subtract the relevant mask value from 256. The mask value itself is not the block size."
    } else if f.label == "Relevant mask octet" {
        "Enter the mask value in that position, not the octet position (1–4) or a number from the computer’s IP address."
    } else if f.label == "Relevant octet (1–4)" {
        "Count eight bits per octet. Choose the octet containing the last network bit; later octets are entirely host bits."
    } else if f.label == "Containing block starts" {
        "A block starts at a multiple of its size. Choose the last boundary at or below the address’s relevant number."
    } else if f.label == "Containing block ends" {
        "The block ends one below the next boundary. Include both endpoints when counting its addresses."
    } else if f.label == "Network address"
        && a == p.ip.to_string()
        && p.calculate().is_ok_and(|c| c.network != p.ip)
    {
        "That is a host inside the block. Set the host bits to zero to identify the network."
    } else if f.label == "Network address" {
        "Check the block boundary. Round the relevant octet down to its boundary and set every later octet to zero."
    } else if f.label == "Broadcast address" {
        "Broadcast is the last address in this block, one before the next subnet. Set all later host octets to 255."
    } else if f.label == "First usable host" {
        "Reserve the network address. The first conventional host is one address after it, including any octet carry."
    } else if f.label == "Last usable host" {
        "Reserve broadcast. The last conventional host is one address before it."
    } else if f.label == "Next subnet" {
        "Advance by one complete subnet, resetting later octets to zero. Enter none only if there is no next subnet inside IPv4."
    } else {
        "Recheck the given address and prefix."
    };
    let _ = stage;
    lead + message
}
