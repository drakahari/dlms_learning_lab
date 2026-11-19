//! Clean supplied RAID1 metadata, saved policy and active bindings. No host I/O.
//! Underlying replicas are the existing bounded RAID1 model; they never mutate.
use crate::{cron_workspace as bounded, raid1};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CONFIG: &str = "/etc/mdadm.conf";
pub const HELP: &str = "Supplied clean two-member RAID1 only. Edit /etc/mdadm.conf and Save file. Commands: mdadm --examine DEVICE (or -E); mdadm --detail ARRAY (or -D); mdadm --assemble ARRAY --uuid=UUID DEVICE DEVICE (or -A, members may be reversed); mdadm --assemble --scan ARRAY; mdadm --stop ARRAY (or -S); cat /etc/mdadm.conf; cat /proc/mdstat; help. Only provided literal paths, one command per run. No unqualified scan/discovery, force, creation, degraded/dirty assembly, host files/devices/processes or shell syntax. Saving config does not change active arrays.";
pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text(#[serde(deserialize_with = "crate::cron::text::<_,4096>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,64>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uuid(#[serde(deserialize_with = "crate::cron::text::<_,64>")] pub String);
impl Uuid {
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty()
            || s.len() > 64
            || !s
                .bytes()
                .all(|c| c.is_ascii_hexdigit() || b":-.".contains(&c))
        {
            return Err("An array UUID needs 32 hexadecimal digits, optionally separated by colon, dash or dot".into());
        }
        let value: String = s
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .map(|c| c.to_ascii_lowercase())
            .collect();
        if value.len() != 32 {
            return Err("An array UUID needs exactly 32 hexadecimal digits; compare the complete identity, not its final group".into());
        }
        Ok(Self(value))
    }
    pub fn display(&self) -> String {
        self.0
            .as_bytes()
            .chunks(8)
            .map(|s| std::str::from_utf8(s).unwrap_or("?"))
            .collect::<Vec<_>>()
            .join(":")
    }
    fn valid(&self) -> bool {
        Self::parse(&self.0).is_ok_and(|v| v == *self)
    }
}
pub fn text(s: &str) -> Result<()> {
    if s.len() > 4096
        || s.lines().count() > 64
        || !s.is_ascii()
        || s.bytes()
            .any(|b| b == 127 || (b < 32 && !b"\n\t".contains(&b)))
    {
        return Err("This virtual configuration accepts up to 4096 ASCII bytes and 64 lines; the previous draft is retained".into());
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub array: Name,
    pub uuid: Uuid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub target: Name,
    pub uuid: Uuid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    /// Retained clean replicas, not a declaration that arrays are currently active.
    pub replicas: raid1::State,
    #[serde(deserialize_with = "bounded::sequence::<_,Identity,2>")]
    pub identities: Vec<Identity>,
}
impl Context {
    pub fn validate(&self) -> Result<()> {
        self.replicas.validate().map_err(|e| e.to_string())?;
        if !bounded::within(self, 128 * 1024) || self.identities.len() != self.replicas.arrays.len()
        {
            return Err("Assembly context needs one identity for each bounded clean mirror".into());
        }
        let mut uuids = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut devices = BTreeSet::new();
        for id in &self.identities {
            let a = self
                .replicas
                .arrays
                .get(&id.array.0)
                .ok_or("Identity refers to an unknown retained array")?;
            if !id.uuid.valid()
                || !uuids.insert(&id.uuid.0)
                || !names.insert(&id.array.0)
                || self.replicas.health(&id.array.0).ok() != Some(raid1::Health::Healthy)
                || a.members.len() != 2
                || a.component_sectors % 2 != 0
            {
                return Err("Only distinct, clean, fully synchronized two-member RAID1 identities with whole KiB sizes are supplied".into());
            }
            for m in &a.members {
                if m.role != raid1::Role::Synced || !devices.insert(&m.device) {
                    return Err("Only complete synchronized member pairs are supported".into());
                }
            }
        }
        if devices.len() != self.replicas.devices.len() {
            return Err("All supplied devices must belong to a clean original mirror".into());
        }
        Ok(())
    }
    pub fn identity(&self, u: &Uuid) -> Result<&Identity> {
        self.identities.iter().find(|i| i.uuid == *u).ok_or_else(|| "No supplied metadata has that array UUID; compare mdadm --examine on both original members".into())
    }
    pub fn target(&self, t: &str) -> bool {
        self.replicas.arrays.contains_key(t)
    }
    pub fn components(&self, u: &Uuid) -> Result<Vec<&str>> {
        let id = self.identity(u)?;
        Ok(self.replicas.arrays[&id.array.0]
            .members
            .iter()
            .map(|m| m.device.as_str())
            .collect())
    }
    pub fn validate_bindings(&self, live: &[Binding]) -> Result<()> {
        if live.len() > 2 {
            return Err("At most two supplied arrays can be active".into());
        }
        let mut seen = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for b in live {
            if !self.target(&b.target.0)
                || !b.uuid.valid()
                || self.identity(&b.uuid).is_err()
                || !seen.insert(&b.uuid.0)
                || !targets.insert(&b.target.0)
            {
                return Err("Active bindings must use distinct provided targets and original complete identities".into());
            }
        }
        Ok(())
    }
    pub fn read(&self, live: &[Binding], target: &str, file: &str) -> Result<String> {
        self.validate()?;
        self.validate_bindings(live)?;
        let b = live
            .iter()
            .find(|b| b.target.0 == target)
            .ok_or("Array is not active; assembly connects existing data, it does not create it")?;
        let id = self.identity(&b.uuid)?;
        self.replicas
            .read_file(&id.array.0, file)
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub devices: Vec<String>,
    pub arrays: Vec<Binding>,
}
pub fn policy(c: &Context, s: &str) -> Result<Policy> {
    text(s)?;
    let mut devices = vec![];
    let mut arrays = vec![];
    let mut device_line = false;
    for (n, line) in s.lines().enumerate() {
        let words = line
            .split_ascii_whitespace()
            .take_while(|w| !w.starts_with('#'))
            .collect::<Vec<_>>();
        if words.is_empty() {
            continue;
        }
        let fail = |s: &str| format!("Configuration line {}: {s}", n + 1);
        if line.starts_with([' ', '\t']) {
            return Err(fail(
                "continuation lines are outside this subset; put one supported DEVICE or ARRAY entry on each line",
            ));
        }
        if words
            .iter()
            .any(|w| w.bytes().any(|b| b"'\"\\*?[];|&$`(){}<>".contains(&b)))
        {
            return Err(fail(
                "unsupported quoting, expansion, discovery or shell syntax; use literal supplied paths",
            ));
        }
        let key = words[0].to_ascii_uppercase();
        match key.as_str() {
            "DEVICE" | "DEV" => {
                device_line = true;
                if words.len() < 2 {
                    return Err(fail("DEVICE needs at least one provided component path"));
                }
                for w in &words[1..] {
                    if !c.replicas.devices.contains_key(*w) {
                        return Err(fail(
                            "unsupported DEVICE path; choose supplied literal components only",
                        ));
                    }
                    if !devices.iter().any(|p| p == *w) {
                        devices.push((*w).into());
                    }
                }
            }
            "ARRAY" | "ARR" => {
                if words.len() != 3
                    || !c.target(words[1])
                    || !words[2].starts_with("UUID=") && !words[2].starts_with("uuid=")
                {
                    return Err(fail(
                        "use ARRAY provided-/dev/mdN UUID=complete-identity; other identities/options are outside this subset",
                    ));
                }
                let uuid = Uuid::parse(&words[2][5..]).map_err(|e| fail(&e))?;
                if arrays
                    .iter()
                    .any(|b: &Binding| b.target.0 == words[1] || b.uuid == uuid)
                {
                    return Err(fail(
                        "duplicate target or UUID is ambiguous in this closed activity",
                    ));
                }
                arrays.push(Binding {
                    target: Name(words[1].into()),
                    uuid,
                });
                if arrays.len() > 2 {
                    return Err(fail("at most two provided ARRAY rows"));
                }
            }
            _ => {
                return Err(fail(
                    "unsupported configuration keyword; this activity models DEVICE and UUID-based ARRAY rows only",
                ));
            }
        }
    }
    if !device_line {
        devices = c.replicas.devices.keys().cloned().collect();
    }
    Ok(Policy { devices, arrays })
}
impl Policy {
    pub fn selection(&self, c: &Context, target: &str) -> Result<Uuid> {
        let b = self.arrays.iter().find(|b| b.target.0 == target).ok_or("No explicit ARRAY row for this target. Real mdadm may discover arrays without ARRAY records; discovery is outside this targeted activity, not modeled as universal failure")?;
        if c.components(&b.uuid)?
            .iter()
            .any(|d| !self.devices.iter().any(|v| v == d))
        {
            return Err("Saved DEVICE policy omits an original member. Include both matching components; one disk is not a complete clean pair in this activity".into());
        }
        Ok(b.uuid.clone())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub version: u32,
    pub saved: Text,
    #[serde(deserialize_with = "bounded::sequence::<_,Binding,2>")]
    pub live: Vec<Binding>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub status: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub output: String,
}
impl Outcome {
    pub fn display(&self) -> String {
        if self.status == 0 && !self.output.is_empty() {
            self.output.clone()
        } else {
            format!(
                "[{} · exit status {}]\n{}",
                if self.status == 0 {
                    "Succeeded"
                } else {
                    "Failed"
                },
                self.status,
                self.output
            )
        }
    }
}
enum Command<'a> {
    Help,
    Config,
    Status,
    Examine(&'a str),
    Detail(&'a str),
    Stop(&'a str),
    Scan(&'a str),
    Assemble(&'a str, Uuid, [&'a str; 2]),
}
fn command<'a>(c: &Context, input: &'a str) -> Result<Command<'a>> {
    if input.len() > 2048
        || !input.is_ascii()
        || input.bytes().any(|b| b.is_ascii_control() && b != b'\t')
    {
        return Err(
            "Use one supported command of at most 2048 ASCII bytes; no pasted script ran".into(),
        );
    }
    let w = input.split_ascii_whitespace().collect::<Vec<_>>();
    match w.as_slice() {
        ["help"] => Ok(Command::Help),
        ["cat", CONFIG] => Ok(Command::Config),
        ["cat", "/proc/mdstat"] => Ok(Command::Status),
        ["mdadm", "--examine" | "-E", d] if c.replicas.devices.contains_key(*d) => {
            Ok(Command::Examine(d))
        }
        ["mdadm", "--detail" | "-D", t] if c.target(t) => Ok(Command::Detail(t)),
        ["mdadm", "--stop" | "-S", t] if c.target(t) => Ok(Command::Stop(t)),
        ["mdadm", "--assemble" | "-A", "--scan", t] if c.target(t) => Ok(Command::Scan(t)),
        ["mdadm", "--assemble" | "-A", t, u, d1, d2]
            if c.target(t)
                && u.starts_with("--uuid=")
                && c.replicas.devices.contains_key(*d1)
                && c.replicas.devices.contains_key(*d2) =>
        {
            Ok(Command::Assemble(t, Uuid::parse(&u[7..])?, [d1, d2]))
        }
        _ => Err(format!("Unsupported command or path. Nothing ran. {HELP}")),
    }
}
pub fn command_shape(c: &Context, text: &str) -> Result<()> {
    command(c, text).map(|_| ())
}
impl Runtime {
    pub fn validate(&self, c: &Context) -> Result<()> {
        c.validate()?;
        if self.version != 1 {
            return Err("Unsupported saved assembly state version".into());
        }
        text(&self.saved.0)?;
        c.validate_bindings(&self.live)
    }
    pub fn save(&mut self, c: &Context, text_value: String) -> Result<Outcome> {
        self.validate(c)?;
        text(&text_value)?;
        self.saved = Text(text_value);
        Ok(Outcome { status:0, output:"Saved virtual configuration. Active arrays and original component data are unchanged.".into() })
    }
    fn assemble(
        &mut self,
        c: &Context,
        target: &str,
        uuid: Uuid,
        devices: Option<[&str; 2]>,
    ) -> Result<String> {
        let pair = c.components(&uuid)?;
        if let Some(d) = devices
            && (d[0] == d[1] || !pair.contains(&d[0]) || !pair.contains(&d[1]))
        {
            return Err("Components do not form the two distinct synchronized members of that UUID. Compare both --examine results; do not mix different arrays".into());
        }
        if self.live.iter().any(|b| b.target.0 == target) {
            return Err("Target is already active. Inspect its UUID and data; a saved edit does not replace an active array".into());
        }
        if self.live.iter().any(|b| b.uuid == uuid) {
            return Err("Those component devices are already active in another supplied array; do not attach them twice".into());
        }
        self.live.push(Binding {
            target: Name(target.into()),
            uuid,
        });
        self.live.sort_by(|a, b| a.target.0.cmp(&b.target.0));
        Ok(format!("mdadm: {target} has been started with 2 drives."))
    }
    pub fn execute(&mut self, c: &Context, input: &str) -> Result<Outcome> {
        self.validate(c)?;
        let cmd = command(c, input)?;
        let mut next = self.clone();
        let result = match cmd {
            Command::Help => Ok(HELP.into()), Command::Config => Ok(self.saved.0.clone()),
            Command::Status => {
                let mut text = "Personalities : [raid1]\n".to_string();
                for b in &self.live {
                    let id=c.identity(&b.uuid)?; let a=&c.replicas.arrays[&id.array.0];
                    text.push_str(&format!("{} : active raid1 {}\n      {} blocks super 1.2 [2/2] [UU]\n\n", b.target.0.trim_start_matches("/dev/"),a.members.iter().map(|m|format!("{}[{}]",m.device.trim_start_matches("/dev/"),m.slot.unwrap())).collect::<Vec<_>>().join(" "),a.component_sectors/2));
                }
                text.push_str("unused devices: <none>\n"); Ok(text)
            }
            Command::Examine(d) => {
                let id = c.identities.iter().find(|id| c.replicas.arrays[&id.array.0].members.iter().any(|m| m.device == d)).unwrap();
                let a=&c.replicas.arrays[&id.array.0]; let m=a.members.iter().find(|m|m.device==d).unwrap();
                Ok(format!("{d}:\n        Version : 1.2\n     Array UUID : {}\n     Raid Level : raid1\n   Raid Devices : 2\n          State : clean\n    Device Role : Active device {}\n    Array State : AA ('A' == active, '.' == missing)\n",id.uuid.display(),m.slot.unwrap()))
            }
            Command::Detail(t) => match self.live.iter().find(|b| b.target.0==t) {
                None => Err("Array is not active. Inspect its existing component metadata, then assemble it; do not create new metadata".into()),
                Some(b) => { let id=c.identity(&b.uuid)?; let mut output=c.replicas.detail(&id.array.0).map_err(|e| e.to_string())?.output; output=output.replacen(&id.array.0,t,1); output.push_str(&format!("\n          UUID : {}\n",b.uuid.display())); Ok(output) }
            },
            Command::Stop(t) => match next.live.iter().position(|b| b.target.0==t) {
                None=>Err("That array is already stopped; existing metadata and data remain available for assembly".into()),
                Some(n)=> { next.live.remove(n); Ok(format!("mdadm: stopped {t}")) }
            },
            Command::Scan(t) => policy(c,&self.saved.0).and_then(|p|p.selection(c,t)).and_then(|u|next.assemble(c,t,u,None)),
            Command::Assemble(t,u,d) => next.assemble(c,t,u,Some(d)),
        };
        match result {
            Ok(output) => {
                next.validate(c)?;
                *self = next;
                Ok(Outcome { status: 0, output })
            }
            Err(e) => Ok(Outcome {
                status: 1,
                output: e,
            }),
        }
    }
}
