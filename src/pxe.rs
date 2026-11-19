//! Bounded PXE offer and BIOS PXELINUX selection over supplied file metadata.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};

mod config;
pub use config::{LinuxEntry, LinuxPair, resolve_linux_pair};
pub mod session;
pub mod source;
pub mod workspace;

const MAX_FILES: usize = 32;
const MAX_NAME: usize = 127;
const MAX_CONFIG: usize = 4096;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Architecture {
    PcBios,
    X64Uefi,
}

impl Architecture {
    pub fn label(self) -> &'static str {
        match self {
            Self::PcBios => "PC BIOS",
            Self::X64Uefi => "x64 UEFI",
        }
    }

    pub fn from_option_93(value: u16) -> Option<Self> {
        match value {
            0 => Some(Self::PcBios),
            7 => Some(Self::X64Uefi),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Request {
    OrdinaryDhcp,
    Pxe { option_93: Option<u16> },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub address: Ipv4Addr,
    pub request: Request,
    pub reachable_servers: [Option<Ipv4Addr>; 4],
    pub ethernet_mac: [u8; 6],
    #[serde(default, deserialize_with = "uuid")]
    pub uuid: Option<String>,
}

fn uuid<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Bounded(#[serde(deserialize_with = "crate::cron::text::<_,36>")] String);
    Ok(Option::<Bounded>::deserialize(deserializer)?.map(|s| s.0))
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BootFile {
    pub server: Ipv4Addr,
    #[serde(deserialize_with = "crate::cron::text::<_,127>")]
    pub name: String,
}

fn relative_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME
        && name.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

impl BootFile {
    pub fn valid(&self) -> bool {
        !self.server.is_unspecified()
            && !self.server.is_broadcast()
            && !self.server.is_multicast()
            && relative_name(&self.name)
    }

    pub fn from_program_directory(&self, name: &str) -> Option<Self> {
        if !self.valid() || !relative_name(name) {
            return None;
        }
        let path = Self {
            server: self.server,
            name: self.name.rsplit_once('/').map_or_else(
                || name.to_owned(),
                |(directory, _)| format!("{directory}/{name}"),
            ),
        };
        path.valid().then_some(path)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub bios: Option<BootFile>,
    pub x64_uefi: Option<BootFile>,
    pub unconditional: Option<BootFile>,
}

impl Policy {
    pub fn validate(&self) -> Result<(), String> {
        for file in [&self.bios, &self.x64_uefi, &self.unconditional]
            .into_iter()
            .flatten()
        {
            if !file.valid() {
                return Err(
                    "Use a supplied server address and a simple relative boot filename.".into(),
                );
            }
        }
        Ok(())
    }

    pub fn offer(&self, request: Request) -> Result<Option<&BootFile>, Diagnostic> {
        match request {
            Request::OrdinaryDhcp => self.unconditional.as_ref().map_or(Ok(None), |file| {
                Err(Diagnostic::UnexpectedOffer(file.clone()))
            }),
            Request::Pxe { option_93: None } => Err(Diagnostic::MissingArchitecture),
            Request::Pxe {
                option_93: Some(code),
            } => {
                let file = match Architecture::from_option_93(code) {
                    Some(Architecture::PcBios) => &self.bios,
                    Some(Architecture::X64Uefi) => &self.x64_uefi,
                    None => return Err(Diagnostic::UnsupportedArchitecture(code)),
                };
                Ok(file.as_ref().or(self.unconditional.as_ref()))
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Artifact {
    BiosPxelinux6,
    X64EfiApplication,
    PxelinuxCore,
    Configuration {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        source: String,
    },
    Kernel {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        release: String,
    },
    Initramfs {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        release: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileRecord {
    pub file: BootFile,
    pub artifact: Artifact,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,FileRecord,32>")]
    files: Vec<FileRecord>,
}

fn release_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'_'))
}

impl Inventory {
    pub fn new(files: Vec<FileRecord>) -> Result<Self, String> {
        let inventory = Self { files };
        inventory.validate()?;
        Ok(inventory)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.files.len() > MAX_FILES {
            return Err("Keep the supplied boot inventory within 32 files.".into());
        }
        let mut paths = BTreeSet::new();
        for record in &self.files {
            if !record.file.valid() || !paths.insert(&record.file) {
                return Err(
                    "Give each boot artifact a unique canonical server and relative name.".into(),
                );
            }
            match &record.artifact {
                Artifact::Configuration { source } if source.len() > MAX_CONFIG => {
                    return Err("Configuration source exceeds 4 KiB.".into());
                }
                Artifact::Kernel { release } | Artifact::Initramfs { release }
                    if !release_name(release) =>
                {
                    return Err("Use a bounded literal supplied kernel release.".into());
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn files(&self) -> &[FileRecord] {
        &self.files
    }

    pub fn get(&self, file: &BootFile) -> Option<&Artifact> {
        self.files
            .iter()
            .find(|record| &record.file == file)
            .map(|record| &record.artifact)
    }

    pub fn save_configuration(
        &mut self,
        program: &BootFile,
        leaf: &str,
        source: String,
    ) -> Result<BootFile, String> {
        self.validate()?;
        if self.get(program) != Some(&Artifact::BiosPxelinux6)
            || !relative_name(leaf)
            || leaf.contains('/')
            || source.len() > MAX_CONFIG
        {
            return Err("Use a small configuration directly in the supplied BIOS program's pxelinux.cfg directory.".into());
        }
        let file = program
            .from_program_directory(&format!("pxelinux.cfg/{leaf}"))
            .ok_or("The resulting configuration path exceeds the supported relative name.")?;
        let artifact = Artifact::Configuration { source };
        let has_space = self.files.len() < MAX_FILES;
        match self.files.iter_mut().find(|entry| entry.file == file) {
            Some(record) if matches!(record.artifact, Artifact::Configuration { .. }) => {
                record.artifact = artifact;
            }
            Some(_) => return Err("The editor cannot overwrite another kind of boot file.".into()),
            None if has_space => {
                self.files.push(FileRecord {
                    file: file.clone(),
                    artifact,
                });
            }
            None => return Err("The virtual boot inventory has reached its file limit.".into()),
        }
        Ok(file)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Diagnostic {
    UnexpectedOffer(BootFile),
    MissingArchitecture,
    UnsupportedArchitecture(u16),
    InvalidPath,
    UnreachableServer(Ipv4Addr),
    MissingFile(BootFile),
    WrongProgram {
        file: BootFile,
        architecture: Architecture,
    },
    MissingCore(BootFile),
    MissingConfiguration,
    WrongConfigurationType(BootFile),
    InvalidConfiguration {
        file: BootFile,
        detail: String,
    },
    WrongLinuxFileType(BootFile),
    ReleaseMismatch {
        kernel: String,
        initramfs: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bootstrap {
    pub file: BootFile,
    pub architecture: Architecture,
}

pub fn bootstrap(
    inventory: &Inventory,
    policy: &Policy,
    client: &Client,
) -> Result<Option<Bootstrap>, Diagnostic> {
    let Some(file) = policy.offer(client.request)? else {
        return Ok(None);
    };
    let Request::Pxe {
        option_93: Some(code),
    } = client.request
    else {
        return Err(Diagnostic::MissingArchitecture);
    };
    let architecture =
        Architecture::from_option_93(code).ok_or(Diagnostic::UnsupportedArchitecture(code))?;
    if !file.valid() {
        return Err(Diagnostic::InvalidPath);
    }
    if !client.reachable_servers.contains(&Some(file.server)) {
        return Err(Diagnostic::UnreachableServer(file.server));
    }
    let artifact = inventory
        .get(file)
        .ok_or_else(|| Diagnostic::MissingFile(file.clone()))?;
    if !matches!(
        (architecture, artifact),
        (Architecture::PcBios, Artifact::BiosPxelinux6)
            | (Architecture::X64Uefi, Artifact::X64EfiApplication)
    ) {
        return Err(Diagnostic::WrongProgram {
            file: file.clone(),
            architecture,
        });
    }
    Ok(Some(Bootstrap {
        file: file.clone(),
        architecture,
    }))
}

fn canonical_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

pub fn configuration_candidates(
    program: &BootFile,
    client: &Client,
) -> Result<Vec<BootFile>, Diagnostic> {
    let mut keys = Vec::with_capacity(11);
    if let Some(uuid) = &client.uuid {
        if !canonical_uuid(uuid) {
            return Err(Diagnostic::InvalidPath);
        }
        keys.push(uuid.clone());
    }
    let ethernet = client
        .ethernet_mac
        .map(|byte| format!("{byte:02x}"))
        .join("-");
    keys.push(format!("01-{ethernet}"));
    let hex = format!("{:08X}", u32::from(client.address));
    keys.extend((1..=8).rev().map(|length| hex[..length].to_owned()));
    keys.push("default".to_owned());
    keys.into_iter()
        .map(|name| {
            program
                .from_program_directory(&format!("pxelinux.cfg/{name}"))
                .ok_or(Diagnostic::InvalidPath)
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationTrace {
    pub attempted: Vec<BootFile>,
    pub selected: Option<BootFile>,
}

pub fn trace_configuration(
    inventory: &Inventory,
    program: &BootFile,
    client: &Client,
) -> Result<ConfigurationTrace, Diagnostic> {
    if inventory.get(program) != Some(&Artifact::BiosPxelinux6) {
        return Err(Diagnostic::WrongProgram {
            file: program.clone(),
            architecture: Architecture::PcBios,
        });
    }
    let core = program
        .from_program_directory("ldlinux.c32")
        .ok_or(Diagnostic::InvalidPath)?;
    if inventory.get(&core) != Some(&Artifact::PxelinuxCore) {
        return Err(Diagnostic::MissingCore(core));
    }
    let mut attempted = configuration_candidates(program, client)?;
    let selected = attempted
        .iter()
        .position(|path| inventory.get(path).is_some());
    if let Some(index) = selected {
        attempted.truncate(index + 1);
    }
    let selected = selected.map(|index| attempted[index].clone());
    Ok(ConfigurationTrace {
        attempted,
        selected,
    })
}
