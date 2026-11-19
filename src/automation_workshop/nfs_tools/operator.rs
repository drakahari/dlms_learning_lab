//! Deliberately small operator commands over the existing virtual NFS records.
use crate::nfs::{Access, Action, Rule, State};
use std::{fmt::Write, net::Ipv4Addr};

const MAX_COMMAND: usize = 256;
const MAX_OUTPUT: usize = 24 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    VerboseExports,
    ReexportSaved,
    ListRemoteExports(Ipv4Addr),
}

pub fn parse(input: &str) -> Result<Command, String> {
    if input.len() > MAX_COMMAND
        || !input
            .bytes()
            .all(|byte| byte == b'\t' || byte == b' ' || byte.is_ascii_graphic())
    {
        return Err("Use one short command with literal arguments.".into());
    }
    let words: Vec<_> = input.split_ascii_whitespace().collect();
    match words.as_slice() {
        ["exportfs", "-v"] => Ok(Command::VerboseExports),
        ["exportfs", "-r" | "-ra" | "-ar"] | ["exportfs", "-r", "-a"] | ["exportfs", "-a", "-r"] => Ok(Command::ReexportSaved),
        ["showmount", "-e" | "--exports", address] => address.parse().map(Command::ListRemoteExports)
            .map_err(|_| "Use the supplied IPv4 server address after showmount -e.".into()),
        ["exportfs", ..] => Err("Supported exportfs options in this exercise are -v (list applied options) and -r or -ra (apply the saved virtual exports).".into()),
        ["showmount", ..] => Err("Use showmount -e followed by one supplied IPv4 address.".into()),
        _ => Err("Use the supported virtual exportfs or showmount command.".into()),
    }
}

fn selector(rule: &Rule) -> String {
    if rule.selector.length == 32 {
        rule.selector.network.to_string()
    } else {
        format!("{}/{}", rule.selector.network, rule.selector.length)
    }
}

/// `local` names the supplied server context for operator commands. Remote
/// listing only selects an exact inventory address. It deliberately does not
/// imply that this listener accepts a mount or grants file access.
pub fn execute(state: &mut State, local: &str, input: &str) -> Result<String, String> {
    let command = parse(input)?;
    let inventory = state.inventory();
    let server = match command {
        Command::ListRemoteExports(ip) => {
            inventory.servers.iter().find(|record| record.address == ip)
        }
        _ => inventory.servers.iter().find(|record| record.id == local),
    }
    .ok_or(
        "Choose a supplied virtual NFS server. This terminal does not contact other addresses.",
    )?;
    if command == Command::ReexportSaved {
        state
            .apply(&Action::ApplySavedExports {
                server: server.id.clone(),
            })
            .map_err(|error| {
                if let Some(affected) = error.message
                    .strip_prefix("ActiveMountConflict: explicitly unmount ")
                    .and_then(|text| text.strip_suffix(" before incompatible Apply"))
                {
                    return format!("A current mount depends on an export that would change ({affected}). Unmount the affected client or preserve its rule, then re-export the saved file.");
                }
                error.message
            })?;
        return Ok(String::new());
    }
    let applied = &state
        .runtime()
        .servers
        .iter()
        .find(|row| row.server == server.id)
        .ok_or("Missing applied export table.")?
        .applied;
    let mut output = String::new();
    match command {
        Command::VerboseExports => {
            for export in applied {
                for rule in &export.clients {
                    let rw = match rule.options.access {
                        Access::Ro => "ro",
                        Access::Rw => "rw",
                    };
                    let squash = if rule.options.root_squash {
                        "root_squash"
                    } else {
                        "no_root_squash"
                    };
                    writeln!(
                        &mut output,
                        "{}\t{}(sync,sec=sys,{rw},{squash},anonuid={},anongid={})",
                        export.root,
                        selector(rule),
                        rule.options.anon_uid,
                        rule.options.anon_gid
                    )
                    .expect("String write");
                }
            }
        }
        Command::ListRemoteExports(ip) => {
            writeln!(&mut output, "Export list for {ip}:").expect("String write");
            for export in applied {
                let clients = export
                    .clients
                    .iter()
                    .map(selector)
                    .collect::<Vec<_>>()
                    .join(",");
                writeln!(&mut output, "{} {clients}", export.root).expect("String write");
            }
        }
        Command::ReexportSaved => unreachable!("Re-export already returned"),
    }
    if output.len() > MAX_OUTPUT {
        return Err("The virtual export listing exceeded its output limit.".into());
    }
    Ok(output)
}
