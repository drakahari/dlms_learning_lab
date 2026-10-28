//! A closed portable command seam for the scheduling exercise only.
//! Existing shell programs and evaluator revisions are unchanged.
use super::*;

pub(crate) fn environment_name(key: &str) -> Result<(), String> {
    variable_target(key)?;
    if matches!(key, "OPTIND" | "OPTERR" | "OPTARG") {
        return Err("Unsupported scheduled special variable; use a named text variable".into());
    }
    Ok(())
}

/// Cronie's command preprocessing happens before shell quoting. This first
/// subset deliberately refuses the unescaped-percent stdin extension.
pub fn prepare(source: &str) -> Result<String, String> {
    if source.is_empty() || source.len() > 2048 || source.contains(['\0', '\r', '\n']) {
        return Err("Scheduling commands need one nonempty line, at most 2 KiB".into());
    }
    let mut command = String::with_capacity(source.len());
    let mut escaped = false;
    for c in source.chars() {
        if escaped {
            if c == '%' {
                command.pop();
            }
            command.push(c);
            escaped = false;
        } else {
            if c == '%' {
                return Err("Unsupported cron percent input: escape a literal percent as \\%; stdin splitting is not included in this lesson".into());
            }
            command.push(c);
            escaped = c == '\\';
        }
    }
    let program = compile(&command)?;
    let [Code::Pipe(commands)] = &program.code[program.main.clone()] else {
        return Err("Unsupported scheduled shell syntax: use one simple printf, echo or cat command with optional redirection".into());
    };
    let [simple] = &commands[..] else {
        return Err("Unsupported scheduled pipeline: this lesson runs one simple command".into());
    };
    let Some(head) = simple.words.first() else {
        return Err("A scheduled entry needs a command".into());
    };
    let mut name = String::new();
    for part in &head.0 {
        let Some(text) = part.text() else {
            return Err("Unsupported expanded scheduled command name".into());
        };
        if part.quote != Quote::Single && text.contains('$') {
            return Err("Unsupported expanded scheduled command name".into());
        }
        name.push_str(text);
    }
    if !matches!(name.as_str(), "printf" | "echo" | "cat") {
        return Err(
            "Unsupported scheduled command: this lesson supports printf, echo and cat only".into(),
        );
    }
    for word in simple
        .words
        .iter()
        .chain(simple.redirects.iter().filter_map(|(_, w)| w.as_ref()))
    {
        scalar_word(word)?;
    }
    Ok(command)
}

fn scalar_word(word: &Word) -> Result<(), String> {
    for part in &word.0 {
        let Some(text) = part.text() else {
            return Err("Unsupported scheduled command substitution".into());
        };
        if part.quote == Quote::Single {
            continue;
        }
        if part.quote == Quote::Bare && text.contains(['*', '?', '[', ']']) {
            return Err("Unsupported scheduled glob: use an explicit virtual /work file".into());
        }
        if part.quote == Quote::Bare && text.contains('$') {
            return Err("Unsupported unquoted scheduled expansion: put variable values inside double quotes, such as \"$NAME\"".into());
        }
        let mut rest = text;
        while let Some(i) = rest.find('$') {
            rest = &rest[i + 1..];
            if let Some(parameter) = rest.strip_prefix('{') {
                let Some((key, tail)) = parameter.split_once('}') else {
                    return Err("Close the variable reference with a matching brace".into());
                };
                if !super::name(key) || readable_variable(key).is_err() {
                    return Err("Unsupported scheduled parameter: use $NAME or ${NAME}; arrays, arithmetic and parameter operators are not included".into());
                }
                rest = tail;
            } else {
                let len = rest
                    .bytes()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
                    .count();
                if len == 0
                    || !super::name(&rest[..len])
                    || readable_variable(&rest[..len]).is_err()
                {
                    return Err("Unsupported scheduled expansion: use a named text variable".into());
                }
                rest = &rest[len..];
            }
        }
    }
    Ok(())
}

/// The caller supplies a validated virtual case. No host execution is possible.
pub fn start(source: &str, case: &Case) -> Result<Vm, String> {
    let command = prepare(source)?;
    Vm::new(&command, case)
}
