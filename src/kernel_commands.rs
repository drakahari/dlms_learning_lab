//! One bounded supplied-source make operation; never a shell or Makefile engine.
use crate::{kernel_build as core, kernel_stage, source_install};
use std::fmt;

pub const MAX_INPUT: usize = 2048;
pub const HELP: &str = "This simulated command console supports make (the supplied All target), make all, make bzImage, make modules, make modules_install and make help. INSTALL_MOD_PATH=/virtual/root may appear before or after the single target. Without it modules_install uses the virtual / root. Use the native source picker/configuration editor and staging controls. No host make, shell, source recipes, flags, quotes, expansion, operators or other targets run here. Installing stale supplied module outputs is outside this subset: rebuild them first.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Invalid,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub detail: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.detail)
    }
}
impl std::error::Error for Error {}
fn error(kind: ErrorKind, detail: impl Into<String>) -> Error {
    Error {
        kind,
        detail: detail.into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Build(core::Target),
    InstallModules { prefix: String },
    Help,
}

pub fn parse(input: &str) -> Result<Command, Error> {
    if input.len() > MAX_INPUT {
        return Err(error(
            ErrorKind::Limit,
            "Keep one make operation within 2048 text bytes.",
        ));
    }
    if input.chars().any(char::is_control) {
        return Err(error(
            ErrorKind::Syntax,
            "Enter one command per Run; control characters and pasted command lists are not supported.",
        ));
    }
    let words: Vec<_> = input.split_whitespace().collect();
    if words.is_empty() {
        return Err(error(
            ErrorKind::Syntax,
            "Enter a listed make operation, or use make help.",
        ));
    }
    if words.len() > 8 {
        return Err(error(
            ErrorKind::Limit,
            "Use one make target and at most one supported variable assignment.",
        ));
    }
    if words[0] != "make" {
        return Err(error(
            ErrorKind::Unsupported,
            "This is a supplied-build console, not a Linux shell. Use make help for the implemented operations.",
        ));
    }
    if input.bytes().any(|b| b"'\"\\;$|&><`(){}[]*?#".contains(&b)) {
        return Err(error(
            ErrorKind::Unsupported,
            "Shell quoting, expansion, operators and recipes are not simulated here. Use one listed make operation with literal virtual paths.",
        ));
    }
    let mut prefix = None;
    let mut target = None;
    for word in &words[1..] {
        if let Some(value) = word.strip_prefix("INSTALL_MOD_PATH=") {
            if prefix.is_some() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Repeated make-variable assignments are outside this subset; use one INSTALL_MOD_PATH.",
                ));
            }
            if !source_install::directory(value) {
                return Err(error(
                    ErrorKind::Invalid,
                    "INSTALL_MOD_PATH needs a canonical virtual directory, without .., empty components or a trailing slash.",
                ));
            }
            prefix = Some(value.to_owned());
        } else {
            if word.starts_with('-') || word.contains('=') {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Only INSTALL_MOD_PATH is supported here; make flags and other variables are not implemented.",
                ));
            }
            if target.replace(*word).is_some() {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This console implements one make target per Run; run separate targets in separate commands.",
                ));
            }
        }
    }
    match target.unwrap_or("all") {
        "all" => Ok(Command::Build(core::Target::All)),
        "bzImage" => Ok(Command::Build(core::Target::Image)),
        "modules" => Ok(Command::Build(core::Target::Modules)),
        "modules_install" => Ok(Command::InstallModules {
            prefix: prefix.unwrap_or_else(|| "/".into()),
        }),
        "help" => Ok(Command::Help),
        _ => Err(error(
            ErrorKind::Unsupported,
            "That make target is outside the supplied model. Use make help; no recipe or host command was executed.",
        )),
    }
}

pub struct Context<'a> {
    pub source_id: &'a str,
    pub base_release: &'a str,
    pub saved_config: Option<&'a core::Config>,
}

pub fn execute(
    context: Context<'_>,
    outputs: &mut core::Artifacts,
    store: &mut kernel_stage::Store,
    input: &str,
) -> Result<String, Error> {
    let command = parse(input)?;
    if command == Command::Help {
        return Ok(HELP.into());
    }
    let config = context.saved_config.ok_or_else(|| {
        error(
            ErrorKind::Invalid,
            "Save a supported configuration fragment before running the build operation.",
        )
    })?;
    config
        .validate()
        .map_err(|e| error(ErrorKind::Invalid, e))?;
    outputs
        .validate_shape()
        .map_err(|e| error(ErrorKind::Invalid, e))?;
    store.validate().map_err(|e| error(ErrorKind::Invalid, e))?;
    match command {
        Command::Build(target) => {
            outputs
                .build(context.source_id, context.base_release, config, target)
                .map_err(|e| error(ErrorKind::Invalid, e))?;
            let identity =
                core::Build::from_config(context.source_id, context.base_release, config.clone())
                    .map_err(|e| error(ErrorKind::Invalid, e))?;
            let image = matches!(target, core::Target::Image | core::Target::All);
            let modules =
                config.modules && matches!(target, core::Target::Modules | core::Target::All);
            let mut lines = vec![
                format!("Simulated supplied-source build: {}", context.source_id),
                format!("Kernel release: {}", identity.release),
            ];
            if image {
                lines.push("Image output: arch/x86/boot/bzImage · modeled metadata, not an executable binary".into());
            }
            if modules {
                lines.push(format!(
                    "Selectable module objects built: {}",
                    config.loadable_inventory().len()
                ));
            }
            if image && !modules {
                lines.push("Earlier module outputs and staged files were not changed.".into());
            }
            Ok(lines.join("\n"))
        }
        Command::InstallModules { prefix } => {
            if !config.modules {
                return Err(error(
                    ErrorKind::Invalid,
                    "The saved configuration has modules disabled; the explicit modules_install target is unavailable.",
                ));
            }
            let record=outputs.modules.as_ref().ok_or_else(||error(ErrorKind::Invalid,"No module outputs are available. Run the supported modules or All build first."))?;
            if !record.current_for(context.source_id, context.base_release, config) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "This model does not install stale or foreign build objects. Rebuild modules for the selected source and saved configuration; no staged files were changed.",
                ));
            }
            let destination = store
                .modules(&prefix, record)
                .map_err(|e| error(ErrorKind::Invalid, e))?;
            Ok(format!(
                "Installed supplied module-tree metadata: {destination}\nSelectable module objects: {}\nVirtual placement only; no host files or services were changed.",
                record.config.loadable_inventory().len()
            ))
        }
        Command::Help => Ok(HELP.into()),
    }
}
