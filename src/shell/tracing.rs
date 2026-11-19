//! Bounded scalar/simple-command xtrace in the fixed C locale. No extra log,
//! descriptor or host execution; every byte uses the ordinary stderr budget.
use super::*;

fn quoted_size(value: &str) -> Result<usize, String> {
    let size = if value.bytes().any(|b| !(32..127).contains(&b)) {
        value.bytes().try_fold(3usize, |n, b| {
            n.checked_add(match b {
                7..=13 | 27 | b'\\' | b'\'' => 2,
                32..127 => 1,
                _ => 4,
            })
        })
    } else if value == "'" {
        Some(2)
    } else if needs_quotes(value) {
        value
            .len()
            .checked_add(2)
            .and_then(|n| n.checked_add(value.bytes().filter(|b| *b == b'\'').count() * 3))
    } else {
        Some(value.len())
    };
    size.filter(|n| *n <= MAX_OUTPUT).ok_or_else(|| {
        "Trace output limit reached (32 KiB); disable tracing or use smaller values".into()
    })
}
fn needs_quotes(value: &str) -> bool {
    value.is_empty()
        || value.starts_with(['#', '~'])
        || value.contains(":~")
        || value.contains("=~")
        || value
            .bytes()
            .any(|b| b" \t\n!\"'$&;()<>?|*[]^`{}\\".contains(&b))
}
fn append_quoted(out: &mut String, value: &str) {
    if value.bytes().any(|b| !(32..127).contains(&b)) {
        out.push_str("$'");
        for b in value.bytes() {
            match b {
                7 => out.push_str("\\a"),
                8 => out.push_str("\\b"),
                9 => out.push_str("\\t"),
                10 => out.push_str("\\n"),
                11 => out.push_str("\\v"),
                12 => out.push_str("\\f"),
                13 => out.push_str("\\r"),
                27 => out.push_str("\\E"),
                b'\\' => out.push_str("\\\\"),
                b'\'' => out.push_str("\\'"),
                32..127 => out.push(char::from(b)),
                _ => {
                    out.push('\\');
                    out.push(char::from(b'0' + (b >> 6)));
                    out.push(char::from(b'0' + ((b >> 3) & 7)));
                    out.push(char::from(b'0' + (b & 7)));
                }
            }
        }
        out.push('\'');
    } else if value == "'" {
        out.push_str("\\'");
    } else if needs_quotes(value) {
        out.push('\'');
        for c in value.chars() {
            if c == '\'' {
                out.push_str("'\\''");
            } else {
                out.push(c);
            }
        }
        out.push('\'');
    } else {
        out.push_str(value);
    }
}
impl Vm {
    pub(super) fn set_xtrace(&mut self, enabled: bool) -> Result<(), String> {
        if enabled && !self.tracing_checked {
            let supported = self.program.bodies.is_empty()
                && self.program.code.iter().all(|code| match code {
                    Code::Jump(_) | Code::JumpIf(_, _) | Code::Status(_) => true,
                    Code::Pipe(commands) => {
                        commands.len() == 1
                            && commands.iter().all(|c| {
                                c.words.iter().all(Word::assignment)
                                    || !c.words.first().is_some_and(Word::assignment)
                            })
                    }
                    _ => false,
                });
            if !supported {
                return Err("Unsupported xtrace program shape: this tracing subset supports scalar assignments, simple commands, if/else and &&/|| only. Pipelines, loops, case, functions, arrays, command substitution and prefixed commands require broader tracing; ordinary nontracing support is unchanged.".into());
            }
            self.tracing_checked = true;
        }
        self.xtrace = enabled;
        Ok(())
    }
    fn emit_trace(&mut self, text: String) -> Result<(), String> {
        // The temporary string remains owned while its bytes are appended.
        // Trace output goes to the shell descriptor before command redirects.
        self.rebase_pending(0, text.capacity())?;
        let destination = self.base_stderr.clone();
        let result = self.emit(&destination, &text);
        self.release_pending(text.capacity())?;
        result
    }
    pub(super) fn trace_args(&mut self, args: &[String]) -> Result<(), String> {
        if !self.xtrace || args.is_empty() {
            return Ok(());
        }
        self.charge_expansion(args.iter().map(String::len).sum())?;
        let size = args.iter().try_fold(2usize, |n, arg| {
            n.checked_add(quoted_size(arg)? + 1)
                .filter(|n| *n <= MAX_OUTPUT)
                .ok_or_else(|| "Trace output limit reached (32 KiB)".to_string())
        })?;
        self.reserve_memory(size)?;
        let mut line = String::with_capacity(size);
        line.push_str("+ ");
        for (n, arg) in args.iter().enumerate() {
            if n > 0 {
                line.push(' ');
            }
            append_quoted(&mut line, arg);
        }
        line.push('\n');
        self.emit_trace(line)
    }
    pub(super) fn trace_assignment(&mut self, key: &str, value: &str) -> Result<(), String> {
        if !self.xtrace {
            return Ok(());
        }
        self.charge_expansion(key.len() + value.len())?;
        let quoted = if value.is_empty() {
            0
        } else {
            quoted_size(value)?
        };
        let size = key
            .len()
            .checked_add(quoted)
            .and_then(|n| n.checked_add(4))
            .filter(|n| *n <= MAX_OUTPUT)
            .ok_or("Trace output limit reached (32 KiB)")?;
        self.reserve_memory(size)?;
        let mut line = String::with_capacity(size);
        line.push_str("+ ");
        line.push_str(key);
        line.push('=');
        if !value.is_empty() {
            append_quoted(&mut line, value);
        }
        line.push('\n');
        self.emit_trace(line)
    }
}
