//! Literal EFI rescue commands. The parser never invokes a shell or firmware tool.
use super::{Facts, World, fat};
use crate::{bios_loader, partition_fs as disk};
use std::collections::BTreeMap;
use std::fmt::Write;

pub const HELP: &str = "Supplied EFI rescue subset: grub-install --target=x86_64-efi --efi-directory=MOUNT --boot-directory=PATH --bootloader-id=ID --no-nvram; grub-install --target=x86_64-efi --efi-directory=MOUNT --boot-directory=PATH --removable [--no-nvram] on supplied removable media; efibootmgr [-v]; efibootmgr --create --disk DISK --part NUMBER --label LABEL --loader PATH; efibootmgr --bootorder XXXX,YYYY; lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS; blkid [PARTITION]; findmnt -T SUPPLIED_DIRECTORY; cat SUPPLIED_BOOT_DIRECTORY/grub/grub.cfg. Use one literal command at a time. Quote EFI paths containing backslashes. Test boot chain is a native action.";

const INSTALL_OPTIONS: &[OptionSpec] = &[
    ("--target", None, true),
    ("--efi-directory", None, true),
    ("--boot-directory", None, true),
    ("--bootloader-id", None, true),
    ("--no-nvram", None, false),
    ("--removable", None, false),
];
const FIRMWARE_OPTIONS: &[OptionSpec] = &[
    ("--create", Some("-c"), false),
    ("--disk", Some("-d"), true),
    ("--part", Some("-p"), true),
    ("--label", Some("-L"), true),
    ("--loader", Some("-l"), true),
    ("--bootorder", Some("-o"), true),
];
type OptionSpec = (&'static str, Option<&'static str>, bool);
type Options<'a> = BTreeMap<&'static str, Option<&'a str>>;

fn options<'a>(words: &'a [String], supported: &[OptionSpec]) -> Result<Options<'a>, String> {
    let mut result = BTreeMap::new();
    let mut words = words.iter();
    while let Some(word) = words.next() {
        let (name, inline) = word
            .split_once('=')
            .map_or((word.as_str(), None), |(name, value)| (name, Some(value)));
        let Some(&(canonical, _, takes_value)) = supported
            .iter()
            .find(|&&(long, short, _)| name == long || short == Some(name))
        else {
            return Err(format!(
                "Option or argument {name:?} is outside this EFI subset."
            ));
        };
        let value = if takes_value {
            let value = inline
                .or_else(|| words.next().map(String::as_str))
                .filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| format!("Supply a value for {name}."))?;
            Some(value)
        } else {
            if inline.is_some() {
                return Err(format!("{name} takes no value."));
            }
            None
        };
        if result.insert(canonical, value).is_some() {
            return Err(format!("Supply {canonical} only once."));
        }
    }
    Ok(result)
}

fn required<'a>(options: &Options<'a>, name: &str) -> Result<&'a str, String> {
    options
        .get(name)
        .copied()
        .flatten()
        .ok_or_else(|| format!("Supply {name} for this EFI operation."))
}

fn partition_number(value: &str) -> Result<u8, String> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Use a decimal partition number.".into());
    }
    value
        .parse::<u8>()
        .ok()
        .filter(|n| (1..=16).contains(n))
        .ok_or_else(|| "Use a supplied partition number between 1 and 16.".into())
}

fn order(value: &str) -> Result<Vec<u16>, String> {
    value
        .split(',')
        .map(|number| {
            if number.is_empty()
                || number.len() > 4
                || !number.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(
                    "Use comma-separated hexadecimal boot numbers, such as 0002,0007.".into(),
                );
            }
            u16::from_str_radix(number, 16).map_err(|_| "Invalid hexadecimal boot number.".into())
        })
        .collect()
}

impl World {
    pub fn execute(&mut self, facts: &Facts, raw: &str) -> Result<String, String> {
        self.bound(facts)?;
        let words = bios_loader::words(raw).map_err(|e| e.to_string())?;
        let (program, args) = words.split_first().ok_or("Type one supported command.")?;
        match (program.as_str(), args) {
            ("grub-install", args) => self.install_command(facts, args),
            ("efibootmgr", []) => Ok(self.firmware_report(facts, false)),
            ("efibootmgr", [flag]) if matches!(flag.as_str(), "-v" | "--verbose") => {
                Ok(self.firmware_report(facts, true))
            }
            ("efibootmgr", args) => self.firmware_command(facts, args),
            ("lsblk", [flag, fields]) if flag == "-o" && fields == "NAME,TYPE,FSTYPE,MOUNTPOINTS" => {
                Ok(block_report(facts))
            }
            ("blkid", []) => facts.geometry.report_blkid(None).map(|out| out.stdout).map_err(|e| e.to_string()),
            ("blkid", [path]) => facts.geometry.report_blkid(Some(path)).map(|out| out.stdout).map_err(|e| e.to_string()),
            ("findmnt", [flag, path]) if flag == "-T" => mount_report(facts, path),
            ("cat", [path]) => {
                let directory = path.strip_suffix("/grub/grub.cfg")
                    .ok_or("Only supplied boot-directory grub/grub.cfg files can be read here.")?;
                let configuration = facts.directory(directory)?.configuration.as_ref()
                    .ok_or("There is no supplied grub.cfg in this boot directory.")?;
                Ok(configuration.menu.clone())
            }
            _ => Err("Command or arguments are outside the supplied EFI rescue subset. Open Supported commands for the available syntax.".into()),
        }
    }

    fn install_command(&mut self, facts: &Facts, args: &[String]) -> Result<String, String> {
        let args = options(args, INSTALL_OPTIONS)?;
        let target = required(&args, "--target")?;
        if target == "x86_64-efi" && args.contains_key("--removable") {
            if args.contains_key("--bootloader-id") {
                return Err("The removable installation uses the fixed default EFI path. Omit --bootloader-id.".into());
            }
            let path = self.install_removable(
                facts,
                required(&args, "--efi-directory")?,
                required(&args, "--boot-directory")?,
            )?;
            return Ok(format!(
                "Installed {} for x86_64-efi removable startup.\nFirmware entries unchanged.\n",
                path.as_str()
            ));
        }
        if target != "x86_64-efi" || !args.contains_key("--no-nvram") {
            return Err("Use the supplied x86_64-efi target and --no-nvram. Register the boot option separately.".into());
        }
        let path = self.install(
            facts,
            required(&args, "--efi-directory")?,
            required(&args, "--boot-directory")?,
            required(&args, "--bootloader-id")?,
        )?;
        Ok(format!(
            "Installing for x86_64-efi platform.\nInstalled {}.\nFirmware entries unchanged (--no-nvram).\n",
            path.as_str()
        ))
    }

    fn firmware_command(&mut self, facts: &Facts, args: &[String]) -> Result<String, String> {
        let args = options(args, FIRMWARE_OPTIONS)?;
        if args.len() == 1 && args.contains_key("--bootorder") {
            self.reorder(facts, &order(required(&args, "--bootorder")?)?)?;
        } else if args.contains_key("--create") && !args.contains_key("--bootorder") {
            let spelling = required(&args, "--loader")?;
            if !spelling.starts_with('\\') {
                return Err("Use a quoted firmware path with backslash separators.".into());
            }
            let loader = fat::Path::parse(spelling)?;
            self.register(
                facts,
                required(&args, "--disk")?,
                partition_number(required(&args, "--part")?)?,
                required(&args, "--label")?,
                loader,
            )?;
        } else {
            return Err("Use efibootmgr --create with its four required values, or set --bootorder separately.".into());
        }
        Ok(self.firmware_report(facts, true))
    }

    fn firmware_report(&self, facts: &Facts, verbose: bool) -> String {
        let values = self
            .boot_order
            .iter()
            .map(|number| format!("{number:04X}"))
            .collect::<Vec<_>>();
        let mut output = format!("BootOrder: {}\n", values.join(","));
        let mut entries = self.options.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.number);
        for entry in entries {
            let active = if entry.active { "*" } else { " " };
            let _ = write!(output, "Boot{:04X}{active} {}", entry.number, entry.label);
            if verbose {
                let esp = facts.esp(&entry.partition_guid).expect("validated option");
                let partition = facts.geometry.disks[&esp.disk]
                    .partitions()
                    .find(|partition| partition.number == esp.number)
                    .expect("validated ESP");
                let _ = write!(
                    output,
                    "\tHD({},GPT,{},0x{:x},0x{:x})/File({})",
                    esp.number,
                    esp.partition_guid,
                    partition.start,
                    partition.end - partition.start + 1,
                    entry.loader.firmware_spelling()
                );
            }
            output.push('\n');
        }
        output
    }
}

fn block_report(facts: &Facts) -> String {
    let mut output = String::from("NAME TYPE FSTYPE MOUNTPOINTS\n");
    for (path, device) in &facts.geometry.disks {
        let _ = writeln!(output, "{} disk", path.trim_start_matches("/dev/"));
        for part in device.partitions() {
            let filesystem = match part.content {
                disk::Content::Fat32 { .. } => "vfat",
                disk::Content::Ext4 { .. } => "ext4",
                disk::Content::Swap { .. } => "swap",
                _ => "",
            };
            let mountpoints = facts
                .esps
                .iter()
                .filter(|esp| esp.disk == *path && esp.number == part.number)
                .filter_map(|esp| esp.mount.as_ref().map(|mount| mount.path.as_str()))
                .chain(
                    facts
                        .boot_directories
                        .iter()
                        .filter(|directory| {
                            directory.disk == *path && directory.partition == part.number
                        })
                        .map(|directory| directory.path.as_str()),
                )
                .collect::<Vec<_>>();
            let _ = writeln!(
                output,
                "  {}{} part {filesystem} {}",
                path.trim_start_matches("/dev/"),
                part.number,
                mountpoints.join(",")
            );
        }
    }
    output
}

fn mount_report(facts: &Facts, path: &str) -> Result<String, String> {
    let header = "TARGET SOURCE FSTYPE OPTIONS";
    for esp in &facts.esps {
        if let Some(mount) = &esp.mount
            && mount.path == path
        {
            let mode = if mount.writable && facts.geometry.disks[&esp.disk].writable {
                "rw"
            } else {
                "ro"
            };
            return Ok(format!(
                "{header}\n{path} {}{} vfat {mode}\n",
                esp.disk, esp.number
            ));
        }
    }
    let directory = facts.directory(path)?;
    let mode = if directory.writable && facts.geometry.disks[&directory.disk].writable {
        "rw"
    } else {
        "ro"
    };
    Ok(format!(
        "{header}\n{path} {}{} ext4 {mode}\n",
        directory.disk, directory.partition
    ))
}
