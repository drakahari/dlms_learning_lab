//! Portable one-off job seam. Parsing and evaluation remain the existing VM's.
use super::*;

pub const MAX_SCRIPT: usize = 8192;
pub const MAX_COMMANDS: usize = 32;

fn scalar(word: &Word) -> Result<(), String> {
    for part in &word.0 {
        let Some(text) = part.text() else {
            return Err("Unsupported one-off command substitution".into());
        };
        if part.quote == Quote::Single {
            continue;
        }
        if part.quote == Quote::Bare && text.contains(['*', '?', '[', ']', '$']) {
            return Err("Unsupported one-off glob or unquoted expansion; use an explicit /work file or a double-quoted named variable".into());
        }
        let mut rest = text;
        while let Some(i) = rest.find('$') {
            rest = &rest[i + 1..];
            if let Some(parameter) = rest.strip_prefix('{') {
                let Some((key, tail)) = parameter.split_once('}') else {
                    return Err("Close the one-off variable reference with a brace".into());
                };
                if !name(key) || cron::environment_name(key).is_err() {
                    return Err("Unsupported one-off parameter; use $NAME or ${NAME}".into());
                }
                rest = tail;
            } else {
                let n = rest
                    .bytes()
                    .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                    .count();
                if n == 0 || !name(&rest[..n]) || cron::environment_name(&rest[..n]).is_err() {
                    return Err("Unsupported one-off expansion; use a named scalar variable".into());
                }
                rest = &rest[n..];
            }
        }
    }
    Ok(())
}

fn program(source: &str) -> Result<Arc<Program>, String> {
    if source.is_empty() || source.len() > MAX_SCRIPT || source.contains(['\0', '\r']) {
        return Err("One-off scripts need 1..8192 UTF-8 bytes without NUL or CR".into());
    }
    let program = compile(source)?;
    if !program.bodies.is_empty() || program.main.len() > MAX_COMMANDS {
        return Err(
            "Unsupported one-off script: use at most 32 simple commands without substitutions"
                .into(),
        );
    }
    for instruction in &program.code[program.main.clone()] {
        let Code::Pipe(commands) = instruction else {
            return Err("Unsupported one-off control syntax; this first subset uses simple printf, echo and cat commands".into());
        };
        let [simple] = &commands[..] else {
            return Err("Unsupported one-off pipeline; use separate simple commands".into());
        };
        let Some(head) = simple.words.first() else {
            return Err("Unsupported one-off redirect-only command".into());
        };
        let mut command = String::new();
        for part in &head.0 {
            let Some(text) = part.text() else {
                return Err("Unsupported expanded one-off command name".into());
            };
            if part.quote != Quote::Single && text.contains('$') {
                return Err("Unsupported expanded one-off command name".into());
            }
            command.push_str(text);
        }
        if !matches!(command.as_str(), "printf" | "echo" | "cat") {
            return Err(
                "Unsupported one-off command: use printf, echo or cat; no host command runs".into(),
            );
        }
        for word in &simple.words {
            scalar(word)?;
        }
        for (op, word) in &simple.redirects {
            if !matches!(op.as_str(), "<" | ">" | ">>" | "2>" | "2>>") {
                return Err("Unsupported one-off redirection; use <, >, >>, 2> or 2>>".into());
            }
            scalar(
                word.as_ref()
                    .ok_or("A one-off redirection needs a virtual filename")?,
            )?;
        }
    }
    Ok(program)
}

/// A syntax/shape check only; no commands execute and no source is rewritten.
pub fn validate(source: &str) -> Result<(), String> {
    program(source).map(|_| ())
}

/// Start the same compiled evaluator with caller-supplied virtual inputs only.
pub fn start(source: &str, case: &Case) -> Result<Vm, String> {
    if !case.valid() {
        return Err("Invalid one-off virtual inputs".into());
    }
    Vm::from_program(program(source)?, case)
}
