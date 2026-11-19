//! Literal, one-command Open-iSCSI subset for the virtual initiator.
use super::{Key, Portal, Result, Startup, valid_target};

pub const INPUT_LIMIT: usize = 1024;

pub const HELP: &str = "iscsiadm -m discoverydb -t sendtargets -p PORTAL --discover
iscsiadm -m node
iscsiadm -m node -T TARGET -p PORTAL --op show
iscsiadm -m node -T TARGET -p PORTAL --login
iscsiadm -m node -T TARGET -p PORTAL --logout
iscsiadm -m node -T TARGET -p PORTAL --op update -n node.startup -v automatic
iscsiadm -m node --loginall=automatic
iscsiadm -m session
iscsiadm -m session -P 3
lsblk -o NAME,WWN

Use a supplied target name and IPv4 address:port. The port defaults to 3260.
Startup accepts manual or automatic. Only a single simulated interface is present.
The console does not interpret shell quotes, pipes, scripts or substitutions.
All discovery, login, records and device identities are simulated.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Discover(Portal),
    Nodes,
    Show(Key),
    Login(Key),
    Logout(Key),
    SetStartup(Key, Startup),
    LoginAutomatic,
    Sessions { devices: bool },
    DeviceIds,
}

#[derive(Default)]
struct Options<'a> {
    mode: Option<&'a str>,
    kind: Option<&'a str>,
    target: Option<&'a str>,
    portal: Option<&'a str>,
    operation: Option<&'a str>,
    name: Option<&'a str>,
    value: Option<&'a str>,
    print: Option<&'a str>,
    loginall: Option<&'a str>,
    discover: bool,
    login: bool,
    logout: bool,
}

pub fn parse(input: &str) -> Result<Command> {
    if input.len() > INPUT_LIMIT
        || !input
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b" :-_.=,\t".contains(&b))
    {
        return Err(
            "Enter one literal command within 1,024 bytes. This console does not run shell syntax."
                .into(),
        );
    }

    let words: Vec<_> = input.split_ascii_whitespace().collect();
    if matches!(
        words.as_slice(),
        ["lsblk", "-o" | "--output", "NAME,WWN"] | ["lsblk", "--output=NAME,WWN"]
    ) {
        return Ok(Command::DeviceIds);
    }
    let ["iscsiadm", args @ ..] = words.as_slice() else {
        return Err(
            "Use one of the listed iscsiadm commands. Other utilities are outside this exercise."
                .into(),
        );
    };
    if matches!(args, ["--help"] | ["-h"]) {
        return Ok(Command::Help);
    }
    if args.len() > 16 {
        return Err("Use one listed iscsiadm operation at a time.".into());
    }

    let opts = parse_options(args)?;
    to_command(opts)
}

fn parse_options<'a>(args: &[&'a str]) -> Result<Options<'a>> {
    let mut o = Options::default();
    let mut words = args.iter().copied();
    while let Some(word) = words.next() {
        let (flag, inline) = match word.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
            _ => (word, None),
        };
        let slot = match flag {
            "-m" | "--mode" => &mut o.mode,
            "-t" | "--type" => &mut o.kind,
            "-T" | "--targetname" => &mut o.target,
            "-p" | "--portal" => &mut o.portal,
            "-o" | "--op" => &mut o.operation,
            "-n" | "--name" => &mut o.name,
            "-v" | "--value" => &mut o.value,
            "-P" | "--print" => &mut o.print,
            "-L" | "--loginall" => &mut o.loginall,
            "-D" | "--discover" => {
                set_switch(&mut o.discover, flag, inline)?;
                continue;
            }
            "-l" | "--login" => {
                set_switch(&mut o.login, flag, inline)?;
                continue;
            }
            "-u" | "--logout" => {
                set_switch(&mut o.logout, flag, inline)?;
                continue;
            }
            _ => {
                return Err(format!(
                    "Unsupported iscsiadm option {flag}. See Supported commands."
                ));
            }
        };
        let value = inline
            .or_else(|| words.next())
            .filter(|value| !value.is_empty() && !value.starts_with('-'))
            .ok_or_else(|| format!("{flag} needs a value."))?;
        if slot.replace(value).is_some() {
            return Err(format!("{flag} was supplied more than once."));
        }
    }
    Ok(o)
}

fn set_switch(slot: &mut bool, flag: &str, value: Option<&str>) -> Result<()> {
    if *slot || value.is_some() {
        return Err(format!(
            "{flag} is a switch. Supply it once without a value."
        ));
    }
    *slot = true;
    Ok(())
}

fn to_command(o: Options<'_>) -> Result<Command> {
    match o {
        Options { mode: Some("discoverydb"), kind: Some("sendtargets"), portal: Some(portal),
            target: None, operation: None, name: None, value: None, print: None, loginall: None,
            discover: true, login: false, logout: false } => Ok(Command::Discover(portal.parse()?)),
        Options { mode: Some("session"), kind: None, target: None, portal: None,
            operation: None, name: None, value: None, print: print @ (None | Some("0" | "3")), loginall: None,
            discover: false, login: false, logout: false } => Ok(Command::Sessions { devices: print == Some("3") }),
        Options { mode: Some("node"), kind: None, print: None, discover: false, .. } => node(o),
        _ => Err("This operation is outside the supported discoverydb, node and session commands. See Supported commands.".into()),
    }
}

fn node(o: Options<'_>) -> Result<Command> {
    match (o.target, o.portal, o.operation, o.name, o.value, o.loginall, o.login, o.logout) {
        (None, None, None, None, None, None, false, false) => Ok(Command::Nodes),
        (None, None, None, None, None, Some("automatic"), false, false) => Ok(Command::LoginAutomatic),
        (Some(target), Some(portal), op, name, value, None, login, logout) => {
            if !valid_target(target) {
                return Err("Use the supplied iSCSI target name beginning with iqn.".into());
            }
            let key = Key { target: target.into(), portal: portal.parse()? };
            match (op, name, value, login, logout) {
                (Some("show"), None, None, false, false) => Ok(Command::Show(key)),
                (None, None, None, true, false) => Ok(Command::Login(key)),
                (None, None, None, false, true) => Ok(Command::Logout(key)),
                (Some("update"), Some("node.startup"), Some(startup), false, false) => {
                    Ok(Command::SetStartup(key, startup.parse()?))
                }
                _ => Err("Select one node operation: show, login, logout or update node.startup.".into()),
            }
        }
        _ => Err("Select the supplied target (-T) and portal (-p) for node changes, or use --loginall=automatic.".into()),
    }
}
