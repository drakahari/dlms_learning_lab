use super::{
    Artifact, BootFile, Client, Diagnostic, Inventory, MAX_CONFIG, relative_name,
    trace_configuration,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinuxEntry {
    pub name: String,
    pub kernel: String,
    pub initrd: String,
}

impl LinuxEntry {
    /// A single automatic Linux entry with one initial RAM disk. Parse the
    /// documented subset without interpreting command lines or includes.
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() > MAX_CONFIG
            || source.lines().count() > 32
            || !source
                .bytes()
                .all(|byte| matches!(byte, b'\t' | b'\n' | b'\r' | b' '..=b'~'))
        {
            return Err("Use a plain-text configuration within 4 KiB and 32 lines.".into());
        }
        let mut default = None;
        let mut label = None;
        let mut kernel = None;
        let mut initrd = None;
        for (index, line) in source.lines().enumerate() {
            let line = line.trim_matches([' ', '\t']);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let error = |message| format!("Configuration line {}: {message}", index + 1);
            if line.contains('\r') {
                return Err(error("Use LF or CRLF line endings."));
            }
            let mut words = line.split_ascii_whitespace();
            let directive = words
                .next()
                .expect("A non-comment line contains a directive");
            let value = words.next().ok_or_else(|| error("A value is required."))?;
            if words.next().is_some() || !relative_name(value) {
                return Err(error(
                    "Use one plain name without spaces or path traversal.",
                ));
            }
            let field = match directive.to_ascii_uppercase().as_str() {
                "DEFAULT" if !value.contains('/') => &mut default,
                "LABEL" if !value.contains('/') => &mut label,
                "LINUX" if label.is_some() => &mut kernel,
                "INITRD" if label.is_some() => &mut initrd,
                _ => {
                    return Err(error(
                        "Use DEFAULT and one LABEL with its LINUX and INITRD paths.",
                    ));
                }
            };
            if field.replace(value).is_some() {
                return Err(error(
                    "This exercise supports one setting of each directive.",
                ));
            }
        }
        let name = label.ok_or("Add the single LABEL for this exercise.")?;
        if default != Some(name) {
            return Err("DEFAULT must name the supplied configuration’s LABEL.".into());
        }
        Ok(Self {
            name: name.into(),
            kernel: kernel
                .ok_or("Add LINUX with the kernel's relative boot-file path.")?
                .into(),
            initrd: initrd
                .ok_or("Add INITRD with one relative boot-file path.")?
                .into(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinuxPair {
    pub kernel: BootFile,
    pub initramfs: BootFile,
    pub release: String,
}

pub fn resolve_linux_pair(
    inventory: &Inventory,
    program: &BootFile,
    client: &Client,
) -> Result<LinuxPair, Diagnostic> {
    let trace = trace_configuration(inventory, program, client)?;
    let file = trace.selected.ok_or(Diagnostic::MissingConfiguration)?;
    let source = match inventory.get(&file) {
        Some(Artifact::Configuration { source }) => source,
        _ => return Err(Diagnostic::WrongConfigurationType(file)),
    };
    let entry = LinuxEntry::parse(source)
        .map_err(|detail| Diagnostic::InvalidConfiguration { file, detail })?;
    let kernel = program
        .from_program_directory(&entry.kernel)
        .ok_or(Diagnostic::InvalidPath)?;
    let initramfs = program
        .from_program_directory(&entry.initrd)
        .ok_or(Diagnostic::InvalidPath)?;
    let Some(Artifact::Kernel { release: target }) = inventory.get(&kernel) else {
        return Err(Diagnostic::WrongLinuxFileType(kernel));
    };
    let Some(Artifact::Initramfs { release: early }) = inventory.get(&initramfs) else {
        return Err(Diagnostic::WrongLinuxFileType(initramfs));
    };
    if target != early {
        return Err(Diagnostic::ReleaseMismatch {
            kernel: target.clone(),
            initramfs: early.clone(),
        });
    }
    Ok(LinuxPair {
        kernel,
        initramfs,
        release: target.clone(),
    })
}
