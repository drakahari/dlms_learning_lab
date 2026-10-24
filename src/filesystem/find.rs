//! Read-only, bounded traversal of the existing virtual Tree. No host I/O.
use super::{Tree, parent};

const OUTPUT: usize = 32_768;
const RESERVED: usize = 384;
const DIAGNOSTICS: usize = 16;
const DIAGNOSTIC_BYTES: usize = 512;
const WORK: usize = 1_048_576;
const LABEL: usize = 2048;
const PENDING: usize = 512 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindDiagnosticKind {
    PermissionDenied,
    UnsupportedTraversal,
    OutputLimit,
    WorkLimit,
    LabelLimit,
}

/// A short retained diagnostic; it contains no file contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindDiagnostic {
    pub kind: FindDiagnosticKind,
    message: String,
}
impl FindDiagnostic {
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Ephemeral query data, never a saved-state or completion credential.
#[derive(Debug)]
pub struct FindReport {
    paths: Vec<String>,
    diagnostics: Vec<FindDiagnostic>,
    suppressed: usize,
    bytes: usize,
    stopped: bool,
}
impl FindReport {
    fn new() -> Self {
        Self {
            paths: Vec::new(),
            diagnostics: Vec::new(),
            suppressed: 0,
            bytes: 0,
            stopped: false,
        }
    }
    pub fn paths(&self) -> &[String] {
        &self.paths
    }
    pub fn diagnostics(&self) -> &[FindDiagnostic] {
        &self.diagnostics
    }
    pub fn suppressed_diagnostics(&self) -> usize {
        self.suppressed
    }
    pub fn complete(&self) -> bool {
        self.diagnostics.is_empty() && self.suppressed == 0
    }
    fn limit(&mut self, kind: FindDiagnosticKind, message: &str) {
        self.stopped = true;
        // One reserved short diagnostic fits even when normal output is full.
        if self.diagnostics.len() < DIAGNOSTICS {
            let message = small(message, 128);
            self.bytes += message.len() + 1;
            self.diagnostics.push(FindDiagnostic { kind, message });
        } else {
            self.suppressed += 1;
        }
    }
    fn diagnostic(&mut self, kind: FindDiagnosticKind, message: &str) {
        if self.diagnostics.len() >= DIAGNOSTICS {
            self.suppressed += 1;
            return;
        }
        let message = small(message, DIAGNOSTIC_BYTES);
        if self.bytes + message.len() + 1 > OUTPUT - RESERVED {
            self.limit(
                FindDiagnosticKind::OutputLimit,
                "Search output exceeds32KiB; use a narrower start, pattern or depth.",
            );
            return;
        }
        self.bytes += message.len() + 1;
        self.diagnostics.push(FindDiagnostic { kind, message });
    }
    fn add_path(&mut self, path: &str) {
        if self.bytes + path.len() + 1 > OUTPUT - RESERVED {
            self.limit(
                FindDiagnosticKind::OutputLimit,
                "Search output exceeds32KiB; use a narrower start, pattern or depth.",
            );
        } else {
            self.bytes += path.len() + 1;
            self.paths.push(path.into());
        }
    }
    /// Existing terminal has one text result/error channel, not POSIX streams.
    pub(super) fn terminal(&self) -> Result<String, String> {
        let mut out = String::with_capacity(self.bytes + 192);
        if !self.complete() {
            out.push_str("Search incomplete. Partial matches:\n");
        }
        for path in &self.paths {
            out.push_str(path);
            out.push('\n');
        }
        if !self.complete() {
            out.push_str("Diagnostics:\n");
            for diagnostic in &self.diagnostics {
                out.push_str(diagnostic.message());
                out.push('\n');
            }
            if self.suppressed != 0 {
                out.push_str(&format!(
                    "{} additional traversal diagnostics omitted.\n",
                    self.suppressed
                ));
            }
        }
        debug_assert!(out.len() <= OUTPUT);
        if self.complete() { Ok(out) } else { Err(out) }
    }
}

fn small(text: &str, limit: usize) -> String {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    // Collect into a fresh allocation: do not retain a large original capacity.
    text[..end].to_owned()
}

enum PatternByte {
    Literal(u8),
    One,
    Many,
}
struct Pattern(Vec<PatternByte>);
impl Pattern {
    fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 255 || text.chars().any(char::is_control) {
            return Err(
                "find -name pattern must be at most255bytes without control characters.".into(),
            );
        }
        let mut bytes = text.bytes();
        let mut pattern = Vec::new();
        while let Some(byte) = bytes.next() {
            pattern.push(match byte {
                b'*' => PatternByte::Many,
                b'?' => PatternByte::One,
                b'\\' => PatternByte::Literal(
                    bytes
                        .next()
                        .ok_or("Unsupported find pattern: trailing backslash.")?,
                ),
                b'[' | b']' => return Err(
                    "Unsupported find bracket pattern; escape brackets to match them literally."
                        .into(),
                ),
                byte => PatternByte::Literal(byte),
            });
        }
        Ok(Self(pattern))
    }
    fn matches(&self, name: &[u8], work: &mut usize) -> Result<bool, ()> {
        let mut previous = vec![false; name.len() + 1];
        let mut current = vec![false; name.len() + 1];
        previous[0] = true;
        for token in &self.0 {
            current.fill(false);
            for j in 0..=name.len() {
                spend(work)?;
                current[j] = match token {
                    PatternByte::Many => previous[j] || (j > 0 && current[j - 1]),
                    PatternByte::One => j > 0 && previous[j - 1],
                    PatternByte::Literal(byte) => j > 0 && previous[j - 1] && name[j - 1] == *byte,
                };
            }
            std::mem::swap(&mut previous, &mut current);
        }
        Ok(previous[name.len()])
    }
}
fn spend(work: &mut usize) -> Result<(), ()> {
    if *work >= WORK {
        return Err(());
    }
    *work += 1;
    Ok(())
}
enum Test {
    Name(Pattern),
    Directory(bool),
}
struct Query<'a> {
    start: &'a str,
    depth: u32,
    tests: Vec<Test>,
}
impl<'a> Query<'a> {
    fn parse(args: &'a [String]) -> Result<Self, String> {
        // Validate direct callers before cloning/parsing their data.
        if args.len() > 38
            || args
                .iter()
                .try_fold(5usize, |n, a| n.checked_add(a.len() + 1))
                .is_none_or(|n| n > 4096)
        {
            return Err("find command exceeds4096bytes or its bounded operand count.".into());
        }
        let Some(start) = args.first().map(String::as_str) else {
            return Err("find requires exactly one explicit start path.".into());
        };
        if start.is_empty() || start.starts_with('-') || matches!(start, "!" | "(") {
            return Err("find requires one start path; use ./ before a name beginning with '-' or a reserved operand.".into());
        }
        let mut i = 1;
        let mut depth = u32::MAX;
        if args.get(i).is_some_and(|s| s == "-maxdepth") {
            let n = args
                .get(i + 1)
                .ok_or("find -maxdepth needs a nonnegative decimal depth.")?;
            if n.is_empty() || n.len() > 10 || !n.bytes().all(|b| b.is_ascii_digit()) {
                return Err(
                    "find -maxdepth supports a decimal value0..4294967295 (at most10digits)."
                        .into(),
                );
            }
            depth = n.parse().map_err(|_| "find -maxdepth exceeds4294967295.")?;
            i += 2;
        }
        let mut tests = Vec::new();
        while i < args.len() {
            if args[i] == "-print" && i + 1 == args.len() {
                i += 1;
                break;
            }
            if tests.len() == 16 {
                return Err("find supports at most16 name/type predicates.".into());
            }
            let arg = args
                .get(i + 1)
                .ok_or("Incomplete or unsupported find predicate.")?;
            tests.push(match args[i].as_str() {
                "-name" => Test::Name(Pattern::parse(arg)?),
                "-type" if arg == "f" || arg == "d" => Test::Directory(arg == "d"),
                _ => return Err("Unsupported find syntax. Use one start, optional leading -maxdepth, implicit-AND -name/-type f|d tests, and optional final -print.".into()),
            });
            i += 2;
        }
        debug_assert_eq!(i, args.len());
        Ok(Self {
            start,
            depth,
            tests,
        })
    }
}

impl Tree {
    /// A bounded read-only query. Arguments omit the command name itself.
    pub fn find(&self, arguments: &[String]) -> Result<FindReport, String> {
        if !self.valid() {
            return Err("Invalid virtual filesystem state; search was not run.".into());
        }
        let query = Query::parse(arguments)?;
        let start = self
            .resolve(query.start, false)
            .map_err(|error| small(&String::from(error), DIAGNOSTIC_BYTES))?;
        let mut report = FindReport::new();
        let mut work = 0;
        let mut pending = vec![(start.as_str(), query.start.to_owned(), 0u32)];
        let mut pending_bytes = query.start.len();
        while let Some((key, label, depth)) = pending.pop() {
            if report.stopped {
                break;
            }
            pending_bytes -= label.len();
            if spend(&mut work).is_err() {
                report.limit(
                    FindDiagnosticKind::WorkLimit,
                    "Search effort limit reached; use a narrower query.",
                );
                break;
            }
            let node = &self.nodes[key];
            let basename = if depth == 0 {
                let trimmed = query.start.trim_end_matches('/');
                if trimmed.is_empty() {
                    "/"
                } else {
                    trimmed.rsplit('/').next().unwrap()
                }
            } else {
                key.rsplit('/').next().unwrap()
            };
            let mut selected = true;
            for test in &query.tests {
                let matches = match test {
                    Test::Directory(directory) => node.directory == *directory,
                    Test::Name(pattern) => match pattern.matches(basename.as_bytes(), &mut work) {
                        Ok(matches) => matches,
                        Err(()) => {
                            report.limit(
                                FindDiagnosticKind::WorkLimit,
                                "Search effort limit reached; use a narrower query.",
                            );
                            break;
                        }
                    },
                };
                if !matches {
                    selected = false;
                    break;
                }
            }
            if report.stopped {
                break;
            }
            if selected {
                report.add_path(&label);
            }
            if report.stopped {
                break;
            }
            if !node.directory || depth >= query.depth {
                continue;
            }
            if !self.allows(&self.user, key, 4) {
                report.diagnostic(
                    FindDiagnosticKind::PermissionDenied,
                    &format!("Permission denied while reading directory: {label}"),
                );
                continue;
            }
            if !self.allows(&self.user, key, 1) {
                report.diagnostic(FindDiagnosticKind::UnsupportedTraversal, &format!("Unsupported descent into readable, nonsearchable directory: {label}. This model does not emulate readdir-only type/metadata behavior."));
                continue;
            }
            for (child, _) in self.nodes.iter().rev() {
                if spend(&mut work).is_err() {
                    report.limit(
                        FindDiagnosticKind::WorkLimit,
                        "Search effort limit reached; use a narrower query.",
                    );
                    break;
                }
                if child == "/" || parent(child) != key {
                    continue;
                }
                let name = child.rsplit('/').next().unwrap();
                let separator = if label.ends_with('/') { "" } else { "/" };
                let bytes = label.len() + separator.len() + name.len();
                if bytes > LABEL || pending_bytes + bytes > PENDING {
                    report.limit(
                        FindDiagnosticKind::LabelLimit,
                        "Search display-path limit reached; use a shorter or narrower start.",
                    );
                    break;
                }
                pending_bytes += bytes;
                pending.push((
                    child.as_str(),
                    format!("{label}{separator}{name}"),
                    depth + 1,
                ));
            }
            if report.stopped {
                break;
            }
        }
        Ok(report)
    }
}
