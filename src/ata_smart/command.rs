//! Explicit ATA SMART practice commands. Device selectors are virtual keys.
use super::valid_selector;

pub const INPUT_LIMIT: usize = 1024;
pub const HELP: &str = "\
smartctl -i DEVICE
smartctl -H DEVICE
smartctl -c DEVICE
smartctl -l selftest DEVICE
smartctl -t short DEVICE

Use a device shown in this exercise. The long forms --info, --health and
--capabilities, --log=selftest and --test=short select the same operations.
The practice console accepts one literal command and no shell operators.
Use the separate virtual time control to advance an in-progress self-test.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Info,
    Health,
    Capabilities,
    SelfTestLog,
    StartShort,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Device { name: String, operation: Operation },
}

fn operation(words: &[&str]) -> Option<Operation> {
    match words {
        ["-i" | "--info"] => Some(Operation::Info),
        ["-H" | "--health"] => Some(Operation::Health),
        ["-c" | "--capabilities"] => Some(Operation::Capabilities),
        ["-l" | "--log", "selftest"] | ["--log=selftest"] => Some(Operation::SelfTestLog),
        ["-t" | "--test", "short"] | ["--test=short"] => Some(Operation::StartShort),
        _ => None,
    }
}

pub fn parse(input: &str) -> Result<Command, String> {
    if input.len() > INPUT_LIMIT
        || !input
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/_.-= \t".contains(&byte))
    {
        return Err("Enter one literal smartctl command within 1,024 bytes. The console does not evaluate shell expressions.".into());
    }
    let words: Vec<_> = input.split_ascii_whitespace().collect();
    if matches!(words.as_slice(), ["smartctl", "--help" | "-h"]) {
        return Ok(Command::Help);
    }
    let ["smartctl", args @ ..] = words.as_slice() else {
        return Err("Use a supported smartctl command. See Supported commands.".into());
    };
    let (name, options) = args
        .split_last()
        .ok_or("Add one supported operation and a supplied device.")?;
    if !valid_selector(name) {
        return Err(
            "Use a virtual /dev/sdX name or a supplied /dev/disk/by-id/ata-… selector.".into(),
        );
    }
    let operation =
        operation(options).ok_or("Use one supported inspection or short-test option at a time.")?;
    Ok(Command::Device {
        name: (*name).into(),
        operation,
    })
}
