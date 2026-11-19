//! Exercise-specific handover format, NOT Linux-required syntax.
//! Closed labels avoid pretending to interpret arbitrary natural language.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bulletin {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub service: String,
    pub starts: u32,
    pub ends: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub contact: String,
}
fn clock(value: &str) -> Result<u32, String> {
    let b = value.as_bytes();
    if b.len() != 5 || b[2] != b':' || [b[0], b[1], b[3], b[4]].iter().any(|x| !x.is_ascii_digit())
    {
        return Err("Write a time as HH:MM using the supplied recording’s 24-hour clock.".into());
    }
    let h = u32::from(b[0] - b'0') * 10 + u32::from(b[1] - b'0');
    let m = u32::from(b[3] - b'0') * 10 + u32::from(b[4] - b'0');
    if h > 23 || m > 59 {
        return Err("Use hours 00–23 and minutes 00–59.".into());
    }
    Ok(h * 60 + m)
}
impl Bulletin {
    pub fn validate(&self) -> Result<(), String> {
        if self.service.is_empty()
            || self.service.len() > 64
            || self.service.trim() != self.service
            || !self
                .service
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b" ._-".contains(&b))
            || self.contact.is_empty()
            || self.contact.len() > 48
            || !self
                .contact
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            || self.starts >= self.ends
            || self.ends >= 1440
        {
            return Err("Use a named service/contact and a same-day window whose end follows its start. Other bulletin shapes are outside this exercise.".into());
        }
        Ok(())
    }
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > 4096
            || raw.lines().count() > 8
            || raw
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        {
            return Err(
                "Use a short plain-text bulletin with no terminal control sequences.".into(),
            );
        }
        let mut values = BTreeMap::new();
        for line in raw.lines().filter(|line| !line.trim().is_empty()) {
            let (key,value)=line.split_once(':').ok_or("Use one labelled Service, Starts, Ends or Contact line; this is the exercise's bulletin format, not a Linux file grammar.")?;
            let key = key.trim().to_ascii_lowercase();
            if !matches!(key.as_str(), "service" | "starts" | "ends" | "contact") {
                return Err(format!(
                    "Unknown bulletin field {key}. This check does not interpret extra prose; use the four supplied labels."
                ));
            }
            if values.insert(key, value.trim()).is_some() {
                return Err(
                    "A repeated label can contradict the notice. Keep each bulletin label once."
                        .into(),
                );
            }
        }
        for key in ["service", "starts", "ends", "contact"] {
            if !values.contains_key(key) {
                return Err(format!(
                    "The bulletin is missing {key}; add that labelled line before checking."
                ));
            }
        }
        let value = Self {
            service: values["service"].to_ascii_lowercase(),
            starts: clock(values["starts"])?,
            ends: clock(values["ends"])?,
            contact: values["contact"].to_ascii_lowercase(),
        };
        value.validate()?;
        Ok(value)
    }
    pub fn render(&self) -> String {
        format!(
            "Service: {}\nStarts: {:02}:{:02}\nEnds: {:02}:{:02}\nContact: {}\n",
            self.service,
            self.starts / 60,
            self.starts % 60,
            self.ends / 60,
            self.ends % 60,
            self.contact
        )
    }
    pub fn differences(&self, expected: &Self) -> Vec<String> {
        let mut result = vec![];
        if !self.service.eq_ignore_ascii_case(&expected.service) {
            result
                .push("The notice names a different service from the maintenance request.".into());
        }
        if self.starts != expected.starts {
            result.push("The notice gives a different start time. Compare it with the maintenance request and pending restart.".into());
        }
        if self.ends != expected.ends {
            result.push("The notice's end time does not match the announced window.".into());
        }
        if !self.contact.eq_ignore_ascii_case(&expected.contact) {
            result.push("The contact is not the supplied duty contact for this work.".into());
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reordered_and_case_varied_labels_are_equivalent_without_keyword_guessing() {
        let a = Bulletin::parse("Service: archive\nStarts: 22:00\nEnds: 22:20\nContact: duty\n")
            .unwrap();
        let b = Bulletin::parse(" contact : DUTY\nENDS:22:20\nstarts: 22:00\nservice: ARCHIVE\n")
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(Bulletin::parse(&a.render()).unwrap(), a);
        assert!(
            Bulletin::parse("No archive maintenance at22:00; contact duty before22:20").is_err()
        );
        assert!(Bulletin::parse(&(a.render() + "This maintenance is cancelled\n")).is_err());
        assert!(Bulletin::parse(&(a.render() + "Starts: 21:00\n")).is_err());
    }
    #[test]
    fn malformed_missing_reversed_and_control_sequence_notices_fail_specifically() {
        let valid = "Service: archive\nStarts: 22:00\nEnds: 22:20\nContact: duty\n";
        for wrong in [
            valid.replace("22:00", "2:00"),
            valid.replace("22:00", "24:00"),
            valid.replace("22:20", "21:00"),
            valid.replace("Contact: duty\n", ""),
            valid.replace("archive", "\u{1b}[2J"),
        ] {
            assert!(Bulletin::parse(&wrong).is_err());
        }
    }
}
