//! Finite local-recipient mail expansion. No host file or MTA is opened.
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_TEXT: usize = 4096;
const MAX_ALIASES: usize = 16;
const MAX_TARGETS: usize = 8;
const MAX_STEPS: usize = 128;
pub type Aliases = BTreeMap<String, Vec<String>>;

pub fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        && value.as_bytes()[0].is_ascii_alphabetic()
}

fn text_shape(text: &str) -> Result<(), String> {
    if text.len() > MAX_TEXT || text.lines().count() > 32 || text.contains('\0') {
        return Err("Keep the simulated mail file within 4 KiB and 32 lines.".into());
    }
    Ok(())
}

fn recipients(text: &str) -> Result<Vec<String>, String> {
    let values = text
        .split(',')
        .map(str::trim_ascii)
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if values.is_empty() || values.len() > MAX_TARGETS || values.iter().any(|value| !name(value)) {
        return Err(
            "Use one to eight comma-separated local recipient names. Domains, quotes, paths, programs and include files are outside this exercise."
                .into(),
        );
    }
    Ok(values)
}

pub fn parse_aliases(text: &str) -> Result<Aliases, String> {
    text_shape(text)?;
    let mut aliases = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim_ascii().is_empty() || line.trim_ascii_start().starts_with('#') {
            continue;
        }
        let result = (|| {
            if line.starts_with(char::is_whitespace) {
                return Err("Indented continuation lines are outside this exercise.".to_owned());
            }
            let (key, value) = line
                .split_once(':')
                .ok_or("Use alias: recipient, recipient for each entry.")?;
            let key = key.trim_ascii().to_ascii_lowercase();
            if !name(&key) {
                return Err("Use a simple local alias name before the colon.".into());
            }
            let targets = recipients(value)?;
            if aliases.contains_key(&key) {
                return Err("Duplicate alias definitions are outside this exercise.".into());
            }
            if aliases.len() == MAX_ALIASES {
                return Err("Use at most 16 aliases in the simulated file.".into());
            }
            aliases.insert(key, targets);
            Ok(())
        })();
        result.map_err(|error| format!("Line {}: {error}", index + 1))?;
    }
    Ok(aliases)
}

pub fn parse_forward(text: &str) -> Result<Vec<String>, String> {
    text_shape(text)?;
    let mut targets = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim_ascii().is_empty() || line.trim_ascii_start().starts_with('#') {
            continue;
        }
        targets.extend(recipients(line).map_err(|error| format!("Line {}: {error}", index + 1))?);
        if targets.len() > MAX_TARGETS {
            return Err("Use at most eight recipients in the simulated .forward file.".into());
        }
    }
    Ok(targets)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directory {
    pub users: BTreeSet<String>,
    pub aliases: Aliases,
    pub forwards: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Delivery {
    pub mailboxes: BTreeSet<String>,
    pub failures: BTreeSet<String>,
}

impl Directory {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.users.len())
            || self
                .users
                .iter()
                .any(|user| !name(user) || *user != user.to_ascii_lowercase())
            || self.aliases.len() > MAX_ALIASES
            || self.forwards.len() > self.users.len()
        {
            return Err("Supply one to eight local usernames and a bounded mail policy.".into());
        }
        if self.aliases.iter().any(|(alias, targets)| {
            !name(alias)
                || *alias != alias.to_ascii_lowercase()
                || self.users.contains(alias)
                || targets.is_empty()
                || targets.len() > MAX_TARGETS
                || targets
                    .iter()
                    .any(|target| !name(target) || *target != target.to_ascii_lowercase())
        }) {
            return Err(
                "Each shared alias needs one to eight local recipients and a name different from the supplied usernames. User-name alias collisions are outside this exercise."
                    .into(),
            );
        }
        if self.forwards.iter().any(|(user, targets)| {
            !self.users.contains(user)
                || targets.len() > MAX_TARGETS
                || targets
                    .iter()
                    .any(|target| !name(target) || *target != target.to_ascii_lowercase())
        }) {
            return Err("Forwarding must belong to a supplied user and contain at most eight local targets.".into());
        }
        Ok(())
    }

    pub fn deliver(&self, recipient: &str) -> Result<Delivery, String> {
        self.validate()?;
        if !name(recipient) {
            return Err("Submit a simple local recipient name in this exercise.".into());
        }
        let recipient = recipient.to_ascii_lowercase();
        let mut delivery = Delivery::default();
        let mut path = Vec::new();
        let mut remaining = MAX_STEPS;
        self.expand(&recipient, &mut path, &mut remaining, &mut delivery);
        Ok(delivery)
    }

    fn expand(
        &self,
        recipient: &str,
        path: &mut Vec<String>,
        remaining: &mut usize,
        delivery: &mut Delivery,
    ) {
        if *remaining == 0 {
            delivery
                .failures
                .insert("Expansion exceeded the simulation’s 128-step limit.".into());
            return;
        }
        *remaining -= 1;
        if path.iter().any(|item| item == recipient) {
            delivery
                .failures
                .insert(format!("Recipient loop reaches {recipient} again."));
            return;
        }
        path.push(recipient.to_owned());
        if let Some(targets) = self.aliases.get(recipient) {
            for target in targets {
                self.expand(target, path, remaining, delivery);
            }
        } else if self.users.contains(recipient) {
            match self.forwards.get(recipient) {
                Some(targets) if !targets.is_empty() => {
                    for target in targets {
                        if target == recipient {
                            // The documented own-name .forward case delivers a local copy.
                            delivery.mailboxes.insert(recipient.to_owned());
                        } else {
                            self.expand(target, path, remaining, delivery);
                        }
                    }
                }
                _ => {
                    delivery.mailboxes.insert(recipient.to_owned());
                }
            }
        } else {
            delivery.failures.insert(format!(
                "No supplied mailbox or alias exists for {recipient}."
            ));
        }
        path.pop();
    }
}
