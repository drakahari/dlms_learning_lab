//! Literal Contents queries over coherent supplied Debian metadata; no host I/O.
use crate::debian_packages::State;
use serde::Serialize;
use std::collections::BTreeSet;

pub const MAX_PATTERN: usize = 160;
pub const MAX_OUTPUT: usize = 32_768;
pub const MAX_ROWS: usize = 768;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Unsupported,
    Limit,
    Context,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: Code,
    pub message: &'static str,
}
type Result<T> = std::result::Result<T, Error>;
fn fail<T>(code: Code, message: &'static str) -> Result<T> {
    Err(Error { code, message })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    Search { pattern: String, exact: bool },
    List { package: String },
}
fn ordinary(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= MAX_PATTERN
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._+-".contains(&b))
        && !s.split('/').any(|p| p == "." || p == "..")
}
fn package(s: &str) -> bool {
    s.len() >= 2
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"+.-".contains(&b))
}
/// Arguments after apt-file, already admitted by the app's literal tokenizer.
pub fn parse(words: &[String]) -> Result<Query> {
    if words.is_empty() || words.len() > 4 {
        return fail(
            Code::Unsupported,
            "Use apt-file search PATTERN, apt-file -F search PATH, or apt-file list PACKAGE",
        );
    }
    let mut fixed = false;
    let mut args = Vec::with_capacity(2);
    for word in words {
        if matches!(word.as_str(), "-F" | "--fixed-string") {
            if fixed {
                return fail(Code::Unsupported, "Specify the fixed-string option once");
            }
            fixed = true;
        } else if word.starts_with('-') {
            return fail(
                Code::Unsupported,
                "Only literal search/list and -F/--fixed-string are supported; no regular expressions, filters or update wrapper",
            );
        } else {
            args.push(word.as_str());
        }
    }
    match args.as_slice() {
        ["search" | "find", pattern] if ordinary(pattern) => {
            if *pattern == "/" || pattern.starts_with("//") {
                return fail(
                    Code::Unsupported,
                    "Use a literal nonempty file name or path; no repeated leading slash",
                );
            }
            Ok(Query::Search {
                pattern: (*pattern).into(),
                exact: fixed,
            })
        }
        ["list" | "show", name] if package(name) => Ok(Query::List {
            package: (*name).into(),
        }),
        _ => fail(
            Code::Unsupported,
            "Use a literal file name/path or one exact package name. Globs, regular expressions, controls and traversal are outside this subset",
        ),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Row {
    pub package: String,
    pub path: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub rows: Vec<Row>,
    pub text: String,
    pub status: u8,
}
/// Read only. Each admitted publication supplies matching Packages/Contents data.
/// This cannot represent an absent/separately stale Contents target.
pub fn query(state: &State, query: &Query) -> Result<Output> {
    state.validate().map_err(|_| Error {
        code: Code::Context,
        message: "Invalid supplied Debian metadata state",
    })?;
    // Public typed queries have the same bounds as parsed arguments.
    match query {
        Query::Search { pattern, .. }
            if !ordinary(pattern) || pattern == "/" || pattern.starts_with("//") =>
        {
            return fail(
                Code::Unsupported,
                "Use the bounded literal Contents path subset",
            );
        }
        Query::List { package: name } if !package(name) => {
            return fail(Code::Unsupported, "Use one exact package name");
        }
        _ => {}
    }
    let Some(index) = state.runtime().cached.as_ref() else {
        return Ok(Output {
            rows: vec![],
            text: String::new(),
            status: 3,
        });
    };
    let mut rows = BTreeSet::new();
    let mut bytes = 0usize;
    for archive in state.catalog().publications[index].candidates.values() {
        let a = &state.catalog().archives[archive];
        // Architecture is supplied by this native-architecture metadata snapshot.
        if !matches!(a.architecture, crate::debian_packages::Architecture::All)
            && a.architecture != state.catalog().architecture
        {
            continue;
        }
        for path in a.files.keys() {
            let p = path
                .strip_prefix('/')
                .expect("validated absolute manifest path");
            let found = match query {
                Query::Search { pattern, exact } => {
                    if *exact {
                        p == pattern.strip_prefix('/').unwrap_or(pattern)
                    } else if let Some(root_relative) = pattern.strip_prefix('/') {
                        p.starts_with(root_relative) || p.contains(pattern)
                    } else {
                        p.contains(pattern)
                    }
                }
                Query::List { package } => a.name == *package,
            };
            if found && rows.insert((a.name.clone(), p.to_owned())) {
                bytes = bytes.checked_add(a.name.len() + p.len() + 3).ok_or(Error {
                    code: Code::Limit,
                    message: "Contents output exceeds its resource bound",
                })?;
                if rows.len() > MAX_ROWS || bytes > MAX_OUTPUT {
                    return fail(
                        Code::Limit,
                        "Contents result is too large; search for a more specific file or path",
                    );
                }
            }
        }
    }
    let mut text = String::with_capacity(bytes);
    let rows: Vec<_> = rows
        .into_iter()
        .map(|(package, path)| {
            text.push_str(&package);
            text.push_str(": ");
            text.push_str(&path);
            text.push('\n');
            Row { package, path }
        })
        .collect();
    let status = if rows.is_empty() { 1 } else { 0 };
    Ok(Output { rows, text, status })
}
