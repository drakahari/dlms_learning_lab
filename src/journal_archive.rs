//! Read-only queries over an authored recovery journal. No host journal is opened.
use crate::{cron, cron_workspace as bounded};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_BOOTS: usize = 8;
const MAX_RECORDS: usize = 32;
pub const MAX_COMMAND: usize = 384;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(deserialize_with = "cron::text::<_,20>")]
    pub timestamp: String,
    #[serde(deserialize_with = "cron::text::<_,71>")]
    pub unit: String,
    pub priority: u8,
    #[serde(deserialize_with = "cron::text::<_,256>")]
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Boot {
    #[serde(deserialize_with = "cron::text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Record,MAX_RECORDS>")]
    pub records: Vec<Record>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    #[serde(deserialize_with = "cron::text::<_,96>")]
    pub directory: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Boot,MAX_BOOTS>")]
    pub boots: Vec<Boot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection<'a> {
    ListBoots,
    Messages {
        boot: &'a str,
        unit: &'a str,
        priority: u8,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Query<'a> {
    directory: &'a str,
    pub selection: Selection<'a>,
}

pub enum Observation<'a> {
    Boots(&'a [Boot]),
    Messages {
        boot: &'a Boot,
        records: Vec<&'a Record>,
    },
}

fn identifier(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn unit_name(value: &str) -> bool {
    value.strip_suffix(".service").is_some_and(|name| {
        !name.is_empty()
            && name.len() <= 63
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-@".contains(&b))
    })
}

fn utc_timestamp(value: &str) -> bool {
    // Authored fixtures use ISO 8601 UTC. No general date parsing is exposed.
    let bytes = value.as_bytes();
    if bytes.len() != 20 {
        return false;
    }
    let format_matches = bytes.iter().enumerate().all(|(index, byte)| match index {
        4 | 7 => *byte == b'-',
        10 => *byte == b'T',
        13 | 16 => *byte == b':',
        19 => *byte == b'Z',
        _ => byte.is_ascii_digit(),
    });
    if !format_matches {
        return false;
    }
    let number = |start, end| value[start..end].parse::<u16>().unwrap();
    let minute = cron::Moment {
        year: number(0, 4),
        month: number(5, 7) as u8,
        day: number(8, 10) as u8,
        hour: number(11, 13) as u8,
        minute: number(14, 16) as u8,
    };
    minute.validate().is_ok() && number(17, 19) < 60
}

impl Archive {
    pub fn validate(&self) -> Result<(), String> {
        let safe_path = self.directory.starts_with("/recovery/")
            && self.directory.len() <= 96
            && self.directory.split('/').skip(1).all(|part| {
                !matches!(part, "" | "." | "..")
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            });
        if !safe_path || !(1..=MAX_BOOTS).contains(&self.boots.len()) {
            return Err("Provide a virtual recovery directory with one to eight boots.".into());
        }
        let mut ids = BTreeSet::new();
        let mut previous = "";
        for boot in &self.boots {
            if !identifier(&boot.id)
                || boot.id != boot.id.to_ascii_lowercase()
                || !ids.insert(boot.id.as_str())
                || !(1..=MAX_RECORDS).contains(&boot.records.len())
            {
                return Err("Each boot needs a unique ID and one to 32 journal entries.".into());
            }
            for record in &boot.records {
                if !utc_timestamp(&record.timestamp)
                    || record.timestamp.as_str() < previous
                    || !unit_name(&record.unit)
                    || record.priority > 7
                    || record.message.is_empty()
                    || record.message.len() > 256
                    || record.message.chars().any(char::is_control)
                {
                    return Err(
                        "Journal entries need ordered UTC timestamps, service names, priorities 0–7 and one-line messages."
                            .into(),
                    );
                }
                previous = &record.timestamp;
            }
        }
        Ok(())
    }

    pub fn inspect<'a>(&'a self, query: &Query<'_>) -> Result<Observation<'a>, String> {
        if query.directory != self.directory {
            return Err(format!(
                "No copied journal is supplied at that path. The recovery copy is at {}.",
                self.directory
            ));
        }
        let Selection::Messages {
            boot,
            unit,
            priority,
        } = query.selection
        else {
            return Ok(Observation::Boots(&self.boots));
        };
        let Some(boot) = self
            .boots
            .iter()
            .find(|entry| entry.id.eq_ignore_ascii_case(boot))
        else {
            return Err(
                "That boot ID is absent from the copied journal. List its boots first.".into(),
            );
        };
        let records = boot
            .records
            .iter()
            .filter(|record| record.unit == unit && record.priority <= priority)
            .collect();
        Ok(Observation::Messages { boot, records })
    }
}

impl Query<'_> {
    pub fn parse(command: &str) -> Result<Query<'_>, String> {
        if command.len() > MAX_COMMAND
            || !command
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b" _./@=\t-".contains(&b))
        {
            return Err(
                "Enter one literal journalctl command using the supplied path and identifiers."
                    .into(),
            );
        }
        let mut words = command.split_ascii_whitespace();
        if words.next() != Some("journalctl") {
            return Err(
                "This console supports read-only journalctl queries against the recovery copy."
                    .into(),
            );
        }
        let mut directory = None;
        let mut boot = None;
        let mut unit = None;
        let mut priority = None;
        let mut output = None;
        let mut list = false;
        let mut pager = false;
        while let Some(word) = words.next() {
            match word {
                "--list-boots" if !list => list = true,
                "--no-pager" if !pager => pager = true,
                _ => {
                    let (flag, attached) = match word.split_once('=') {
                        Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
                        Some(_) => {
                            return Err(
                                "Separate short flags from their values with a space.".into()
                            );
                        }
                        None => (word, None),
                    };
                    if flag == "--boot" && attached.is_none() {
                        return Err("Use --boot=ID or -b ID to select a copied boot.".into());
                    }
                    let target = match flag {
                        "-D" | "--directory" => &mut directory,
                        "-b" | "--boot" => &mut boot,
                        "-u" | "--unit" => &mut unit,
                        "-p" | "--priority" => &mut priority,
                        "-o" | "--output" => &mut output,
                        _ => {
                            return Err(format!(
                                "Unsupported option: {flag}. See the command help."
                            ));
                        }
                    };
                    if target.is_some() {
                        return Err(format!("Use {flag} once in this exercise."));
                    }
                    *target = attached.or_else(|| words.next());
                    if target.is_none_or(|value| value.is_empty() || value.starts_with('-')) {
                        return Err(format!("Supply an explicit value after {flag}."));
                    }
                }
            }
        }
        let directory = directory.ok_or(
            "Use -D or --directory to open the copied journal instead of the live journal.",
        )?;
        let selection = if list {
            if [boot, unit, priority, output].iter().any(Option::is_some) {
                return Err("List the copied boots separately before filtering messages.".into());
            }
            Selection::ListBoots
        } else {
            let boot = boot
                .filter(|value| identifier(value))
                .ok_or("Use -b with the full 32-character boot ID from the copied boot list.")?;
            let unit = unit
                .filter(|value| unit_name(value))
                .ok_or("Use -u with one exact service name ending in .service.")?;
            let priority = priority
                .and_then(|value| {
                    crate::linux_logging::SEVERITIES
                        .iter()
                        .position(|name| *name == value)
                        .map(|level| level as u8)
                        .or_else(|| match value.as_bytes() {
                            [digit @ b'0'..=b'7'] => Some(digit - b'0'),
                            _ => None,
                        })
                })
                .ok_or("Use -p with one priority name or a number from 0 to 7.")?;
            if output != Some("cat") {
                return Err("Use -o cat to show just the matching messages.".into());
            }
            Selection::Messages {
                boot,
                unit,
                priority,
            }
        };
        Ok(Query {
            directory,
            selection,
        })
    }
}
