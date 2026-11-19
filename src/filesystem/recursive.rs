//! Opt-in recursive ordinary-directory foundation over the existing Tree.
//! No catalog/goal hook yet. Old Tree::execute keeps refusing recursion.
//! Transactional simulation only: real cp/rm can leave partial effects on error.
use super::{Access, MAX_DATA, MAX_ENTRIES, PathError, Tree, canonical, child, parent};

pub const HELP: &str = "Opt-in virtual directory console: cp -R|-r|--recursive [--] SOURCE_DIR DEST; rm -r|-R|--recursive [--] DIR. Exactly one ordinary directory, no trailing slash/final dot operands, no merge or overwrite; an existing destination directory adds the source basename. Ordinary pwd/cd/ls/cat/find, mkdir/touch/cp/mv/rm/rmdir, chmod/chgrp/umask/id and show retain their original literal subset. No shell expansion, pipelines, links/devices, -a/-p, combined flags, interactive prompts or host access. Removal is noninteractive. A refused recursive action commits no virtual changes; real cp/rm may leave partial work. This model is not a real-system safety guarantee.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Permission,
    Limit,
    Rejected,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn err(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn resolved(error: PathError) -> Error {
    let kind = if matches!(&error, PathError::Permission(_)) {
        ErrorKind::Permission
    } else {
        ErrorKind::Invalid
    };
    err(kind, String::from(error))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Copy { source: String, destination: String },
    Remove { path: String },
}
/// A finite literal command grammar, not a shell. Unknown recursion flags fail.
pub fn parse(input: &str) -> Result<Option<Operation>, Error> {
    if input.len() > 4096 {
        return Err(err(ErrorKind::Limit, "Command exceeds 4,096 bytes."));
    }
    let words = crate::shell::literal_command(input).map_err(|e| err(ErrorKind::Unsupported, e))?;
    let Some(command) = words.first().map(String::as_str) else {
        return Ok(None);
    };
    if !matches!(command, "cp" | "rm") {
        return Ok(None);
    }
    let args = &words[1..];
    if !args
        .first()
        .is_some_and(|s| matches!(s.as_str(), "-R" | "-r" | "--recursive"))
    {
        return Ok(None);
    }
    let mut args = &args[1..];
    let ended = args.first().is_some_and(|s| s == "--");
    if ended {
        args = &args[1..];
    }
    let wanted = if command == "cp" { 2 } else { 1 };
    if args.len() != wanted || (!ended && args.iter().any(|s| s.starts_with('-'))) {
        return Err(err(
            ErrorKind::Unsupported,
            "Recursive subset: cp -R [--] SOURCE_DIR DEST, or rm -r [--] DIR. Use one source; no combined flags, merge/overwrite or shell expansion.",
        ));
    }
    // Dot/dotdot *within* paths are resolved normally. Final dot components have
    // special cp naming rules outside this deliberately small first subset.
    if args
        .iter()
        .any(|a| a.ends_with('/') || a.rsplit('/').next().is_some_and(|v| v == "." || v == ".."))
    {
        return Err(err(
            ErrorKind::Unsupported,
            "For this recursive subset, name the directory without a trailing slash or final dot/dotdot component. Intermediate dot/dotdot components still resolve inside the virtual tree.",
        ));
    }
    Ok(Some(if command == "cp" {
        Operation::Copy {
            source: args[0].clone(),
            destination: args[1].clone(),
        }
    } else {
        Operation::Remove {
            path: args[0].clone(),
        }
    }))
}
fn preflight(tree: &Tree) -> Result<(), Error> {
    if !tree.valid() || tree.command_location.is_some() {
        return Err(err(
            ErrorKind::Invalid,
            "Recursive work needs a valid ordinary virtual tree without another activity's operational context.",
        ));
    }
    Ok(())
}
fn subtree<'a>(tree: &'a Tree, root: &str) -> Vec<&'a str> {
    let prefix = format!("{root}/");
    tree.nodes
        .keys()
        .filter(|p| p.as_str() == root || p.starts_with(&prefix))
        .map(String::as_str)
        .collect()
}
fn accessible(tree: &Tree, path: &str, op: Access) -> Result<(), Error> {
    if tree.access(&tree.user, path, op) {
        Ok(())
    } else {
        Err(err(
            ErrorKind::Permission,
            format!(
                "Permission denied: {path}. Check directory read/search access and the permissions needed to change names. This simulator applies no recursive mutation on a refused operation; real commands can leave partial work."
            ),
        ))
    }
}
fn writable_parent(tree: &Tree, path: &str) -> Result<(), Error> {
    tree.permitted(parent(path), 3)
        .map_err(|e| err(ErrorKind::Permission, e))
}
fn bounded_clock(tree: &Tree, added: usize) -> Result<(), Error> {
    if tree
        .clock
        .checked_add(added as u64)
        .is_none_or(|n| n > 1_000_000)
    {
        return Err(err(
            ErrorKind::Limit,
            "Virtual action counter is full; no recursive change applied.",
        ));
    }
    Ok(())
}
fn generated_path(path: &str) -> bool {
    canonical(path)
        && path.split('/').filter(|p| !p.is_empty()).count() <= 64
        && path.split('/').all(|p| p.len() <= 255)
}
/// Applies one typed operation entirely in memory, committing only a valid tree.
pub fn apply(tree: &mut Tree, operation: &Operation) -> Result<(), Error> {
    preflight(tree)?;
    let operands: Vec<&str> = match operation {
        Operation::Copy {
            source,
            destination,
        } => vec![source, destination],
        Operation::Remove { path } => vec![path],
    };
    if operands
        .iter()
        .any(|a| a.ends_with('/') || a.rsplit('/').next().is_some_and(|v| v == "." || v == ".."))
    {
        return Err(err(
            ErrorKind::Unsupported,
            "This recursive subset excludes trailing slashes and final dot/dotdot operands; no change applied.",
        ));
    }
    match operation {
        Operation::Copy {
            source,
            destination,
        } => {
            if source.len() > 1024 || destination.len() > 1024 {
                return Err(err(
                    ErrorKind::Limit,
                    "Virtual operand path exceeds 1,024 bytes.",
                ));
            }
            let source = tree.resolve(source, false).map_err(resolved)?;
            if source == "/" || !tree.nodes[&source].directory {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Recursive copy here needs a supplied ordinary directory other than the virtual root.",
                ));
            }
            let mut target = tree.resolve(destination, true).map_err(resolved)?;
            if tree.nodes.get(&target).is_some_and(|n| n.directory) {
                target = child(&target, source.rsplit('/').next().expect("nonroot source"));
            }
            if target == source || target.starts_with(&format!("{source}/")) {
                return Err(err(
                    ErrorKind::Invalid,
                    "A directory cannot be copied into itself or its own descendant.",
                ));
            }
            if tree.nodes.contains_key(&target) {
                return Err(err(
                    ErrorKind::Unsupported,
                    "The destination subtree already exists. This recursive subset does not merge or overwrite trees; choose an unused destination name.",
                ));
            }
            writable_parent(tree, &target)?;
            let entries = subtree(tree, &source);
            if tree
                .nodes
                .len()
                .checked_add(entries.len())
                .is_none_or(|n| n > MAX_ENTRIES)
            {
                return Err(err(
                    ErrorKind::Limit,
                    "Copy would exceed 256 virtual entries; no change applied.",
                ));
            }
            let mut new_bytes = 0usize;
            let mut files = 0usize;
            for path in &entries {
                let node = &tree.nodes[*path];
                accessible(
                    tree,
                    path,
                    if node.directory {
                        Access::ListNames
                    } else {
                        Access::ReadFile
                    },
                )?;
                if node.directory {
                    accessible(tree, path, Access::SearchDirectory)?;
                }
                let dest = format!("{target}{}", &path[source.len()..]);
                if !generated_path(&dest) {
                    return Err(err(
                        ErrorKind::Limit,
                        "A copied path exceeds the virtual path/component/depth limits.",
                    ));
                }
                if !node.directory {
                    files += 1;
                    new_bytes = new_bytes
                        .checked_add(tree.files[*path].len())
                        .ok_or_else(|| err(ErrorKind::Limit, "Copy data size overflow."))?;
                }
            }
            if tree
                .files
                .values()
                .map(String::len)
                .sum::<usize>()
                .checked_add(new_bytes)
                .is_none_or(|n| n > MAX_DATA)
            {
                return Err(err(
                    ErrorKind::Limit,
                    "Copy would exceed 256 KiB of virtual content; no change applied.",
                ));
            }
            bounded_clock(tree, entries.len() * 2 + files)?;
            let mut candidate = tree.clone();
            // New directories temporarily permit their owner to populate them;
            // final modes use source bits filtered by the caller's umask.
            let mut final_modes = Vec::new();
            for path in entries {
                let node = &tree.nodes[path];
                let dest = format!("{target}{}", &path[source.len()..]);
                let mode = node.mode & !tree.umask;
                candidate
                    .new_node(
                        &dest,
                        node.directory,
                        if node.directory { mode | 0o700 } else { mode },
                    )
                    .map_err(|e| err(ErrorKind::Rejected, e))?;
                if node.directory {
                    final_modes.push((dest, mode));
                } else {
                    candidate
                        .files
                        .insert(dest.clone(), tree.files[path].clone());
                    candidate.changed(&dest);
                }
            }
            for (path, mode) in final_modes {
                candidate
                    .nodes
                    .get_mut(&path)
                    .expect("created directory")
                    .mode = mode;
            }
            if !candidate.valid() {
                return Err(err(
                    ErrorKind::Limit,
                    "Copied tree exceeds supported state limits; previous work retained.",
                ));
            }
            *tree = candidate;
        }
        Operation::Remove { path } => {
            if path.len() > 1024 {
                return Err(err(
                    ErrorKind::Limit,
                    "Virtual operand path exceeds 1,024 bytes.",
                ));
            }
            let path = tree.resolve(path, false).map_err(resolved)?;
            if path == "/" || !tree.nodes[&path].directory {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Recursive removal here needs an ordinary directory other than the virtual root.",
                ));
            }
            if [&tree.cwd, &tree.home]
                .iter()
                .any(|p| p.as_str() == path || p.starts_with(&format!("{path}/")))
            {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Move out of the directory before removing it. This persistent activity does not remove a current or home directory.",
                ));
            }
            let entries = subtree(tree, &path);
            bounded_clock(tree, entries.len())?;
            for p in &entries {
                if tree.nodes[*p].directory {
                    accessible(tree, p, Access::ListNames)?;
                    accessible(tree, p, Access::SearchDirectory)?;
                }
                writable_parent(tree, p)?;
            }
            let mut candidate = tree.clone();
            for p in entries.into_iter().rev() {
                candidate.nodes.remove(p);
                candidate.files.remove(p);
                candidate.modified.remove(p);
                candidate.changed(parent(p));
            }
            if !candidate.valid() {
                return Err(err(
                    ErrorKind::Limit,
                    "Removal would leave an unsupported state; previous work retained.",
                ));
            }
            *tree = candidate;
        }
    }
    Ok(())
}
/// Only a future caller-held task may opt into this adapter. No lab is registered.
pub fn execute(tree: &mut Tree, input: &str) -> Result<String, Error> {
    preflight(tree)?;
    if let Some(operation) = parse(input)? {
        apply(tree, &operation)?;
        Ok(String::new())
    } else {
        if crate::shell::literal_command(input)
            .map_err(|e| err(ErrorKind::Unsupported, e))?
            .as_slice()
            == ["help"]
        {
            return Ok(HELP.into());
        }
        tree.execute(input).map_err(|e| err(ErrorKind::Rejected, e))
    }
}
