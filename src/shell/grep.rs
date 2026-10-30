//! Explicit -E only; the historical literal-F builtin remains unchanged.
use super::*;
use crate::ere;
use std::{fmt::Write, sync::Arc};
const SLICE: usize = 64;
const MAX_LINES: usize = 8192;
#[derive(Clone, Copy)]
struct Options {
    invert: bool,
    number: bool,
    quiet: bool,
}
struct Invocation<'a> {
    options: Options,
    pattern: &'a str,
    file: Option<&'a str>,
}
fn usage() -> String {
    "Unsupported grep -E arguments; use -E/-v/-n/-q before one pattern and at most one virtual file, with -- before dash-leading literals".into()
}
pub(super) fn extended(args: &[String]) -> bool {
    // In particular, never reinterpret an old -F pattern beginning with '-E'.
    if args.first().is_some_and(|v| v == "-F") {
        return false;
    }
    for arg in args {
        if arg == "--" || !arg.starts_with('-') || arg == "-" {
            break;
        }
        if arg.as_bytes().contains(&b'E') {
            return true;
        }
    }
    false
}
fn parse(args: &[String]) -> Result<Invocation<'_>, String> {
    let mut options = Options {
        invert: false,
        number: false,
        quiet: false,
    };
    let (mut i, mut extended, mut closed) = (0, false, false);
    while let Some(arg) = args.get(i) {
        if arg == "--" {
            closed = true;
            i += 1;
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            break;
        }
        if arg.starts_with("--") {
            return Err(usage());
        }
        for flag in arg.as_bytes().iter().skip(1) {
            match flag {
                b'E' => extended = true,
                b'v' => options.invert = true,
                b'n' => options.number = true,
                b'q' => options.quiet = true,
                _ => return Err(usage()),
            }
        }
        i += 1;
    }
    let pattern = args.get(i).ok_or_else(usage)?;
    i += 1;
    if !extended || args.len() > i + 1 {
        return Err(usage());
    }
    let file = args.get(i).map(String::as_str);
    if file.is_some_and(|s| !closed && s.starts_with('-') && s != "-") {
        return Err(usage());
    }
    Ok(Invocation {
        options,
        pattern,
        file,
    })
}
fn locale(vm: &Vm) -> Result<(), String> {
    let all = vm.scalar("LC_ALL").filter(|s| !s.is_empty());
    let bad = if let Some(all) = all {
        all != "C" && all != "POSIX"
    } else {
        ["LC_COLLATE", "LC_CTYPE", "LANG"].iter().any(|k| {
            vm.scalar(k)
                .is_some_and(|v| !v.is_empty() && v != "C" && v != "POSIX")
        })
    };
    if bad {
        Err("Unsupported grep -E locale; this simulator uses C/POSIX bytes only; variable assignment is not general environment export".into())
    } else {
        Ok(())
    }
}
enum Stage {
    Find {
        start: usize,
        at: usize,
    },
    Match {
        range: Range<usize>,
        after: usize,
    },
    Select {
        range: Range<usize>,
        after: usize,
        matched: bool,
    },
    Done,
}
pub(super) struct Task {
    cursor: ere::Cursor,
    options: Options,
    work: ere::Budget,
    output: String,
    lines: usize,
    any: bool,
    stage: Stage,
    pub(super) stdin: bool,
}
pub(super) enum Start {
    Pending(Box<Task>),
    Complete(String, String, i32),
}
impl Task {
    pub(super) fn start(vm: &Vm, args: &[String]) -> Result<Start, String> {
        let invocation = parse(args)?;
        locale(vm)?;
        let charge =
            ere::Regex::compile_charge(invocation.pattern.len()).map_err(|e| e.to_string())?;
        vm.charge_regex(charge)?;
        let compile_memory = vm.scratch(ere::Regex::compile_storage_bound())?;
        let regex = match ere::Regex::compile(invocation.pattern) {
            Ok(regex) => regex,
            Err(error) if error.kind == ere::ErrorKind::Syntax => {
                vm.reserve_memory(1024)?;
                let message = format!("grep: {error}\n");
                return Ok(Start::Complete(String::new(), message, 2));
            }
            Err(error) => return Err(error.to_string()),
        };
        drop(compile_memory);
        let file = invocation.file.filter(|s| *s != "-");
        let text = if let Some(file) = file {
            // A leading ./ names an ordinary flat file; unlike bare -, ./-
            // is not stdin. Keep this spelling local to the new -E subset.
            let p = vm.virtual_path(file.strip_prefix("./").unwrap_or(file))?;
            let Some(text) = vm.outcome.files.get(&p.text) else {
                vm.reserve_memory(file.len() + 60)?;
                return Ok(Start::Complete(
                    String::new(),
                    format!("grep: {file}: No such file or directory\n"),
                    2,
                ));
            };
            text.as_str()
        } else {
            &vm.input[vm.input_pos..]
        };
        let output_bytes = if invocation.options.quiet {
            0
        } else {
            MAX_OUTPUT
        };
        let required = regex
            .cursor_storage_bound(text.len())
            .map_err(|e| e.to_string())?
            + std::mem::size_of::<Self>()
            - std::mem::size_of::<ere::Cursor>()
            + output_bytes;
        vm.reserve_memory(required)?;
        let input: Arc<str> = Arc::from(text);
        let cursor = regex.into_cursor(input).map_err(|e| e.to_string())?;
        let mut output = String::new();
        output
            .try_reserve_exact(output_bytes)
            .map_err(|_| "Cannot reserve bounded grep output")?;
        if output.capacity() > output_bytes {
            return Err("Grep output capacity exceeds admitted bound".into());
        }
        let work = ere::Budget::new(MAX_REGEX_WORK.saturating_sub(vm.regex_work.get()))
            .map_err(|e| e.to_string())?;
        let task = Self {
            cursor,
            options: invocation.options,
            work,
            output,
            lines: 0,
            any: false,
            stage: Stage::Find { start: 0, at: 0 },
            stdin: file.is_none(),
        };
        vm.reserve_memory(task.bytes())?;
        Ok(Start::Pending(Box::new(task)))
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() - std::mem::size_of::<ere::Cursor>()
            + self.cursor.retained_bytes()
            + self.output.capacity()
    }
    pub(super) fn advance(&mut self, vm: &Vm) -> Result<bool, String> {
        let before = self.work.spent();
        let result = self.inner(vm);
        vm.charge_regex(self.work.spent() - before)?;
        result
    }
    fn inner(&mut self, vm: &Vm) -> Result<bool, String> {
        let start = self.work.spent();
        while self.work.spent() - start < SLICE {
            let phase = std::mem::replace(&mut self.stage, Stage::Done);
            match phase {
                Stage::Done => return Ok(true),
                Stage::Find { start, at } => {
                    self.work.consume(1).map_err(|e| e.to_string())?;
                    let length = self.cursor.input().len();
                    if at == length && start == at {
                        return Ok(true);
                    }
                    if at == length || self.cursor.input().as_bytes()[at] == b'\n' {
                        self.lines = self
                            .lines
                            .checked_add(1)
                            .filter(|v| *v <= MAX_LINES)
                            .ok_or("Grep line-count limit reached (8192)")?;
                        self.cursor
                            .start_line(start..at)
                            .map_err(|e| e.to_string())?;
                        self.stage = Stage::Match {
                            range: start..at,
                            after: at + usize::from(at < length),
                        };
                    } else {
                        if at - start >= ere::MAX_LINE {
                            return Err("Regex input line exceeds64KiB".into());
                        }
                        self.stage = Stage::Find { start, at: at + 1 };
                    }
                }
                Stage::Match { range, after } => {
                    let remaining = SLICE - (self.work.spent() - start);
                    match self
                        .cursor
                        .advance(remaining, &mut self.work)
                        .map_err(|e| e.to_string())?
                    {
                        Some(matched) => {
                            self.stage = Stage::Select {
                                range,
                                after,
                                matched,
                            }
                        }
                        None => {
                            self.stage = Stage::Match { range, after };
                            return Ok(false);
                        }
                    }
                }
                Stage::Select {
                    range,
                    after,
                    matched,
                } => {
                    self.work.consume(1).map_err(|e| e.to_string())?;
                    if matched != self.options.invert {
                        self.any = true;
                        if self.options.quiet {
                            return Ok(true);
                        }
                        let prefix = if self.options.number {
                            self.lines.ilog10() as usize + 2
                        } else {
                            0
                        };
                        let additional = range.len() + 1 + prefix;
                        if vm
                            .output_bytes()
                            .checked_add(self.output.len() + additional)
                            .is_none_or(|n| n > MAX_OUTPUT)
                        {
                            return Err("Command output exceeds32KiB".into());
                        }
                        if self.output.len() + additional > self.output.capacity() {
                            return Err("Grep output exceeds admitted capacity".into());
                        }
                        if self.options.number {
                            write!(&mut self.output, "{}:", self.lines)
                                .map_err(|_| "Cannot format grep line number")?;
                        }
                        self.output.push_str(&self.cursor.input()[range]);
                        self.output.push('\n');
                    }
                    self.stage = Stage::Find {
                        start: after,
                        at: after,
                    };
                }
            }
        }
        Ok(matches!(self.stage, Stage::Done))
    }
    pub(super) fn finish(self) -> (String, String, i32) {
        (self.output, String::new(), i32::from(!self.any))
    }
}
