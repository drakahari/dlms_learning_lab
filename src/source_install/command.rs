//! Literal adapter over the supplied in-memory project, not a shell execution path.
use super::{Configuration, Facts, File, Output, Target, World, directory};
use crate::boot_entry::{Error, ErrorKind};
type Result<T> = std::result::Result<T, Error>;
fn fail(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn state(message: String) -> Error {
    fail(ErrorKind::Invalid, message)
}
fn syntax(message: &str) -> Error {
    fail(ErrorKind::Syntax, message)
}
fn unsupported(message: &str) -> Error {
    fail(ErrorKind::Unsupported, message)
}

pub const HELP: &str = "Supplied source-project subset: pwd; cd PATH; mkdir [-p] PATH; ls [PATH]; cat FILE; tar -tf ARCHIVE; tar -xf ARCHIVE -C DIRECTORY (optional -v); ./configure [--prefix=PATH] [--with-greeting=TEXT] [--enable-uppercase|--disable-uppercase]; make [all|check|install|clean] [DESTDIR=PATH] (optional -C DIRECTORY); explicit virtual program paths with NAME, --version or --prefix. Option/value words and supported literal quotes are accepted. Tar long --list/--extract/--file/--directory aliases work. This is an uncompressed structured archive and a supplied fixed build graph, not general tar bytes, C, configure shell or Makefile recipe execution. install depends on all; clean leaves installed copies. DESTDIR prepends staging paths without changing the compiled prefix; omit it and the virtual live prefix changes. No host files, subprocesses, compression, links, arbitrary source editing, shell operators/expansion, environment assignments, PATH lookup, multiple make targets or unknown tool options. All supported paths stay in memory.";

/// Resolve dot components only against this virtual directory; never a host path.
pub fn resolve(cwd: &str, raw: &str) -> Result<String> {
    if !directory(cwd) || raw.is_empty() || raw.len() > 128 {
        return Err(syntax("Use a short virtual path."));
    }
    let combined = if raw.starts_with('/') {
        raw.to_string()
    } else {
        format!("{}/{raw}", cwd.trim_end_matches('/'))
    };
    let mut parts = Vec::new();
    for part in combined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let path = format!("/{}", parts.join("/"));
    if !directory(&path) {
        return Err(syntax(
            "Unsupported virtual path spelling; use ordinary directory/file names.",
        ));
    }
    Ok(path)
}
fn project(world: &World, cwd: &str) -> Result<()> {
    if world.source_root.as_deref() != Some(cwd) {
        return Err(fail(
            ErrorKind::Invalid,
            "Change to the unpacked supplied source directory first, or use make -C DIRECTORY.",
        ));
    }
    Ok(())
}
fn listing(world: &World, cwd: &str, raw: &str) -> Result<Output> {
    let path = resolve(cwd, raw)?;
    if world.files.contains_key(&path) {
        return Ok(Output::success(format!(
            "{}\n",
            path.rsplit('/').next().unwrap()
        )));
    }
    if !world.directories.contains(&path) {
        return Err(fail(
            ErrorKind::Invalid,
            "That directory is not in this virtual workspace.",
        ));
    }
    let prefix = if path == "/" {
        "/".into()
    } else {
        format!("{path}/")
    };
    let names = world
        .files
        .keys()
        .chain(world.directories.iter())
        .filter_map(|p| p.strip_prefix(&prefix))
        .filter(|s| !s.is_empty() && !s.contains('/'))
        .collect::<std::collections::BTreeSet<_>>();
    Ok(Output::success(
        names.into_iter().map(|n| format!("{n}\n")).collect(),
    ))
}
fn configure(world: &mut World, facts: &Facts, cwd: &str, args: &[String]) -> Result<Output> {
    project(world, cwd)?;
    if args == ["--help"] {
        return Ok(Output::success(format!(
            "Supplied project options:\n--prefix=PATH (default {})\n--with-greeting=TEXT (default {})\n--enable-uppercase | --disable-uppercase (default disabled)\n",
            facts.default_prefix, facts.program.default_greeting
        )));
    }
    let mut config = Configuration {
        prefix: facts.default_prefix.clone(),
        greeting: facts.program.default_greeting.clone(),
        uppercase: false,
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let (key, inline) = arg
            .split_once('=')
            .map_or((arg.as_str(), None), |(k, v)| (k, Some(v)));
        if !seen.insert(if key == "--disable-uppercase" {
            "--enable-uppercase"
        } else {
            key
        }) {
            return Err(syntax(
                "Supply each configure option once; conflicting/duplicate values are outside this subset.",
            ));
        }
        match key {
            "--prefix" | "--with-greeting" => {
                let value = if let Some(value) = inline {
                    value
                } else {
                    i += 1;
                    args.get(i)
                        .ok_or_else(|| syntax("That configure option needs a value."))?
                };
                if key == "--prefix" {
                    config.prefix = value.into();
                } else {
                    config.greeting = value.into();
                }
            }
            "--enable-uppercase" | "--disable-uppercase" if inline.is_none() => {
                config.uppercase = key == "--enable-uppercase"
            }
            _ => {
                return Err(unsupported(
                    "That configure option is not supported by this supplied project. Use ./configure --help.",
                ));
            }
        }
        i += 1;
    }
    world.configure(facts, config).map_err(state)?;
    Ok(Output::success("Configured the supplied virtual project. Inspect config.status; existing installed copies are unchanged.\n".into()))
}
fn make(world: &mut World, facts: &Facts, cwd: &str, args: &[String]) -> Result<Output> {
    let mut target = None;
    let mut dest = None;
    let mut selected = cwd.to_string();
    let mut changed_directory = false;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-C" {
            if changed_directory {
                return Err(syntax("Use one make -C directory."));
            }
            i += 1;
            selected = resolve(
                cwd,
                args.get(i)
                    .ok_or_else(|| syntax("make -C needs a directory."))?,
            )?;
            changed_directory = true;
        } else if let Some(value) = arg.strip_prefix("DESTDIR=") {
            if dest.replace(value.to_string()).is_some() {
                return Err(syntax("Use one DESTDIR value."));
            }
        } else {
            let next = match arg.as_str() {
                "all" => Target::All,
                "check" => Target::Check,
                "install" => Target::Install,
                "clean" => Target::Clean,
                _ => {
                    return Err(unsupported(
                        "This supplied project supports one of all, check, install or clean, optional -C and DESTDIR for install; no arbitrary targets, options or variables.",
                    ));
                }
            };
            if target.replace(next).is_some() {
                return Err(unsupported("Run one make target at a time in this subset."));
            }
        }
        i += 1;
    }
    project(world, &selected)?;
    world
        .make(facts, target.unwrap_or(Target::All), dest.as_deref())
        .map_err(state)
}
fn tar(world: &mut World, facts: &Facts, cwd: &str, args: &[String]) -> Result<Output> {
    let mut extract = None;
    let mut archive = None;
    let mut destination = None;
    let mut verbose = false;
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--list" | "-t" => {
                if extract.replace(false).is_some() {
                    return Err(syntax("Choose either tar list or extract once."));
                }
            }
            "--extract" | "-x" => {
                if extract.replace(true).is_some() {
                    return Err(syntax("Choose either tar list or extract once."));
                }
            }
            "-v" | "--verbose" => {
                if verbose {
                    return Err(syntax("Use verbose once."));
                }
                verbose = true;
            }
            "-f" | "--file" | "-tf" | "-xf" | "-xvf" | "-xv" => {
                if matches!(arg, "-tf") && extract.replace(false).is_some() {
                    return Err(syntax("Duplicate tar operation."));
                }
                if matches!(arg, "-xf" | "-xvf" | "-xv") && extract.replace(true).is_some() {
                    return Err(syntax("Duplicate tar operation."));
                }
                if matches!(arg, "-xvf" | "-xv") {
                    verbose = true;
                }
                if arg != "-xv" {
                    i += 1;
                    let value = args
                        .get(i)
                        .ok_or_else(|| syntax("tar -f needs the supplied archive path."))?;
                    if archive.replace(resolve(cwd, value)?).is_some() {
                        return Err(syntax("Use one archive file."));
                    }
                }
            }
            "-C" | "--directory" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| syntax("tar -C needs an existing virtual directory."))?;
                if destination.replace(resolve(cwd, value)?).is_some() {
                    return Err(syntax("Use one extraction directory."));
                }
            }
            _ if arg.starts_with("--file=") => {
                if archive.replace(resolve(cwd, &arg[7..])?).is_some() {
                    return Err(syntax("Use one archive file."));
                }
            }
            _ if arg.starts_with("--directory=") => {
                if destination.replace(resolve(cwd, &arg[12..])?).is_some() {
                    return Err(syntax("Use one extraction directory."));
                }
            }
            _ => {
                return Err(unsupported(
                    "Use the supplied uncompressed archive with tar list/extract, file and optional extraction directory. Compression, arbitrary members and other tar options are outside this subset.",
                ));
            }
        }
        i += 1;
    }
    let extract = extract.ok_or_else(|| syntax("Choose tar -t to list or -x to extract."))?;
    if archive.as_deref() != Some(facts.archive.as_str())
        || world.files.get(&facts.archive) != Some(&File::Archive)
    {
        return Err(fail(
            ErrorKind::Invalid,
            "That is not the available supplied virtual source archive.",
        ));
    }
    if !extract && (destination.is_some() || verbose) {
        return Err(unsupported(
            "List supports tar -tf ARCHIVE; directory and verbose listing are outside this subset.",
        ));
    }
    let mut members = format!("{}/\n", facts.archive_root);
    for (member, entry) in facts.source.entries.iter().filter(|(m, _)| !m.is_empty()) {
        members.push_str(&format!(
            "{}/{}{}\n",
            facts.archive_root,
            member,
            if entry.node.directory { "/" } else { "" }
        ));
    }
    if extract {
        let directory = destination.unwrap_or_else(|| cwd.into());
        if !world.directories.contains(&directory) {
            return Err(fail(
                ErrorKind::Invalid,
                "The extraction directory must already exist in the supplied virtual workspace.",
            ));
        }
        world.extract(facts, &directory).map_err(state)?;
        Ok(Output::success(if verbose {
            members
        } else {
            String::new()
        }))
    } else {
        Ok(Output::success(members))
    }
}
/// Transactional adapter: malformed/unsupported/failed commands never commit a
/// partial virtual transition. This is a simulator property, not real-tool safety.
pub fn run(world: &mut World, facts: &Facts, cwd: &mut String, raw: &str) -> Result<Output> {
    facts.validate().map_err(state)?;
    if !world.directories.contains(cwd.as_str()) {
        return Err(fail(
            ErrorKind::Invalid,
            "Current virtual directory is unavailable.",
        ));
    }
    let args = crate::bios_loader::words(raw)?;
    if args.len() > 24 {
        return Err(fail(
            ErrorKind::Limit,
            "Use at most24 literal words per command.",
        ));
    }
    if args.is_empty() {
        return Err(syntax("Type one command from Supported commands."));
    }
    let mut next = world.clone();
    let mut next_cwd = cwd.clone();
    let output = match args[0].as_str() {
        "pwd" if args.len() == 1 => Output::success(format!("{cwd}\n")),
        "cd" if args.len() == 2 => {
            let path = resolve(cwd, &args[1])?;
            if !world.directories.contains(&path) {
                return Err(fail(
                    ErrorKind::Invalid,
                    "That directory does not exist in the virtual workspace.",
                ));
            }
            next_cwd = path;
            Output::success(String::new())
        }
        "mkdir" if args.len() == 2 && !args[1].starts_with('-') => {
            next.mkdir(&resolve(cwd, &args[1])?, false).map_err(state)?;
            Output::success(String::new())
        }
        "mkdir" if args.len() == 3 && args[1] == "-p" => {
            next.mkdir(&resolve(cwd, &args[2])?, true).map_err(state)?;
            Output::success(String::new())
        }
        "ls" if args.len() <= 2 && !args.get(1).is_some_and(|a| a.starts_with('-')) => {
            listing(world, cwd, args.get(1).map_or(".", String::as_str))?
        }
        "cat" if args.len() >= 2 && args.len() <= 5 => {
            if args[1..].iter().any(|a| a.starts_with('-')) {
                return Err(unsupported(
                    "cat options are outside this subset; use plain cat FILE.",
                ));
            }
            let mut text = String::new();
            for arg in &args[1..] {
                match world.files.get(&resolve(cwd, arg)?) {
                    Some(File::Text { text: value, .. }) => text.push_str(value),
                    Some(_) => {
                        return Err(unsupported(
                            "cat supports the supplied text files only; use tar -tf for the structured archive or run a virtual program by its path.",
                        ));
                    }
                    None => {
                        return Err(fail(
                            ErrorKind::Invalid,
                            "That text file is not present in the virtual workspace.",
                        ));
                    }
                }
            }
            Output::success(text)
        }
        "./configure" => configure(&mut next, facts, cwd, &args[1..])?,
        "make" => make(&mut next, facts, cwd, &args[1..])?,
        "tar" => tar(&mut next, facts, cwd, &args[1..])?,
        command if command.contains('/') => {
            let path = resolve(cwd, command)?;
            next.run(&path, &args[1..]).map_err(state)?
        }
        _ => {
            return Err(unsupported(
                "Unsupported command or arguments in this source-project activity. Use Supported commands. Invoke its program by an explicit build/staging path; PATH lookup is outside this subset.",
            ));
        }
    };
    if output.stdout.len().saturating_add(output.stderr.len()) > 32_768 {
        return Err(fail(
            ErrorKind::Limit,
            "Command output exceeds32 KiB; select fewer or smaller text files.",
        ));
    }
    *world = next;
    *cwd = next_cwd;
    Ok(output)
}
/// Directories supplied by the exercise, never created on the host.
pub fn prepare_directories(world: &mut World, paths: &[&str]) -> Result<()> {
    if paths.len() > 16 {
        return Err(fail(
            ErrorKind::Limit,
            "Supply at most16 initial virtual directories.",
        ));
    }
    let mut next = world.clone();
    for path in paths {
        next.mkdir(path, true).map_err(state)?;
    }
    *world = next;
    Ok(())
}
