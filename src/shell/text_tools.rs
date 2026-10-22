//! Pure, bounded C-locale line utilities. Operates only on supplied text.
const MAX_LINES: usize = 8192;
const MAX_RANGES: usize = 128;
#[derive(Debug)]
pub(super) struct Invocation {
    pub file: Option<String>,
    tool: Tool,
}
#[derive(Debug)]
enum Tool {
    Sort {
        reverse: bool,
        unique: bool,
    },
    Uniq {
        count: bool,
        repeated: bool,
        single: bool,
    },
    Cut {
        delimiter: char,
        ranges: Vec<(usize, usize)>,
        only_delimited: bool,
    },
}
fn unsupported(cmd: &str) -> String {
    format!("Unsupported {cmd} arguments; see the documented simulated text-utility subset")
}
pub(super) fn parse(cmd: &str, args: &[String]) -> Result<Invocation, String> {
    let mut file = None;
    let mut reverse = false;
    let mut unique = false;
    let mut count = false;
    let mut repeated = false;
    let mut single = false;
    let mut delimiter = '\t';
    let mut ranges = None;
    let mut only_delimited = false;
    let mut options = true;
    let mut i = 0;
    while let Some(arg) = args.get(i) {
        if options && arg == "--" {
            options = false;
        } else if options && arg.starts_with('-') && arg != "-" {
            if file.is_some() {
                return Err(unsupported(cmd));
            }
            match cmd {
                "sort" => {
                    for flag in arg[1..].chars() {
                        match flag {
                            'r' => reverse = true,
                            'u' => unique = true,
                            _ => return Err(unsupported(cmd)),
                        }
                    }
                }
                "uniq" => {
                    for flag in arg[1..].chars() {
                        match flag {
                            'c' => count = true,
                            'd' => repeated = true,
                            'u' => single = true,
                            _ => return Err(unsupported(cmd)),
                        }
                    }
                }
                "cut" => {
                    match arg.as_str() {
                        "-s" => only_delimited = true,
                        "-d" => {
                            i += 1;
                            let value = args.get(i).ok_or_else(|| {
                                "cut: -d needs one ASCII delimiter character".to_string()
                            })?;
                            if value.len() != 1
                                || !value.is_ascii()
                                || matches!(value.as_bytes()[0], 0 | b'\n')
                            {
                                return Err("Unsupported cut delimiter; use one non-newline ASCII character".into());
                            }
                            delimiter = value.as_bytes()[0] as char;
                        }
                        "-f" => {
                            if ranges.is_some() {
                                return Err(unsupported(cmd));
                            }
                            i += 1;
                            ranges = Some(parse_ranges(
                                args.get(i)
                                    .ok_or_else(|| "cut: -f needs a field list".to_string())?,
                            )?);
                        }
                        _ => return Err(unsupported(cmd)),
                    }
                }
                _ => return Err(unsupported(cmd)),
            }
        } else {
            if cmd == "uniq"
                && options
                && arg
                    .strip_prefix('+')
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
            {
                return Err("Unsupported traditional uniq +N syntax; use -- before a filename beginning with +digits".into());
            }
            if file.is_some() {
                return Err(unsupported(cmd));
            }
            file = Some(arg.clone());
        }
        i += 1;
    }
    let tool = match cmd {
        "sort" => Tool::Sort { reverse, unique },
        "uniq" if !repeated || !single => Tool::Uniq {
            count,
            repeated,
            single,
        },
        "cut" => Tool::Cut {
            delimiter,
            ranges: ranges.ok_or_else(|| "cut: choose fields with -f LIST".to_string())?,
            only_delimited,
        },
        _ => return Err(unsupported(cmd)),
    };
    Ok(Invocation { file, tool })
}
fn parse_ranges(list: &str) -> Result<Vec<(usize, usize)>, String> {
    let invalid = || "cut: invalid field list; use positive fields such as 1,3-5 or 2-".to_string();
    let number = |s: &str| -> Result<usize, String> {
        if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) {
            return Err(invalid());
        }
        let n = s
            .parse::<usize>()
            .map_err(|_| "Unsupported cut field index; maximum is 1000000".to_string())?;
        if n == 0 {
            return Err("cut: field numbers start at 1, not 0".into());
        }
        if n > 1_000_000 {
            return Err("Unsupported cut field index; maximum is 1000000".into());
        }
        Ok(n)
    };
    let mut ranges = Vec::new();
    for part in list.split(',') {
        if ranges.len() >= MAX_RANGES {
            return Err("cut field-list limit exceeded (128 ranges)".into());
        }
        let range = if let Some((first, last)) = part.split_once('-') {
            if first.is_empty() && last.is_empty() {
                return Err(invalid());
            }
            let first = if first.is_empty() { 1 } else { number(first)? };
            let last = if last.is_empty() {
                usize::MAX
            } else {
                number(last)?
            };
            if first > last {
                return Err("cut: field range ends before it starts".into());
            }
            (first, last)
        } else {
            let n = number(part)?;
            (n, n)
        };
        ranges.push(range);
    }
    ranges.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        if let Some(previous) = merged.last_mut()
            && start <= previous.1.saturating_add(1)
        {
            previous.1 = previous.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    Ok(merged)
}
impl Invocation {
    pub fn execute(&self, text: &str, max_output: usize) -> Result<String, String> {
        let mut lines = Vec::new();
        for line in text.split_terminator('\n') {
            if lines.len() >= MAX_LINES {
                return Err("Text-utility input limit exceeded (8192 lines)".into());
            }
            lines.push(line);
        }
        let mut out = String::new();
        let append = |out: &mut String, value: &str| -> Result<(), String> {
            if out.len().saturating_add(value.len()) > max_output {
                return Err("Text-utility output limit exceeded".into());
            }
            out.push_str(value);
            Ok(())
        };
        match &self.tool {
            Tool::Sort { reverse, unique } => {
                lines.sort_unstable_by(|a, b| {
                    if *reverse {
                        b.as_bytes().cmp(a.as_bytes())
                    } else {
                        a.as_bytes().cmp(b.as_bytes())
                    }
                });
                if *unique {
                    lines.dedup();
                }
                for line in lines {
                    append(&mut out, line)?;
                    append(&mut out, "\n")?;
                }
            }
            Tool::Uniq {
                count,
                repeated,
                single,
            } => {
                let mut i = 0;
                while i < lines.len() {
                    let mut end = i + 1;
                    while end < lines.len() && lines[end] == lines[i] {
                        end += 1;
                    }
                    let size = end - i;
                    if (!*repeated || size > 1) && (!*single || size == 1) {
                        if *count {
                            append(&mut out, &format!("{size:7} "))?;
                        }
                        append(&mut out, lines[i])?;
                        append(&mut out, "\n")?;
                    }
                    i = end;
                }
            }
            Tool::Cut {
                delimiter,
                ranges,
                only_delimited,
            } => {
                for line in lines {
                    if !line.contains(*delimiter) {
                        if !only_delimited {
                            append(&mut out, line)?;
                            append(&mut out, "\n")?;
                        }
                        continue;
                    }
                    let mut wrote = false;
                    let mut range = 0;
                    for (index, field) in line.split(*delimiter).enumerate() {
                        let n = index + 1;
                        while range < ranges.len() && ranges[range].1 < n {
                            range += 1;
                        }
                        if range < ranges.len() && ranges[range].0 <= n {
                            if wrote {
                                append(&mut out, &delimiter.to_string())?;
                            }
                            append(&mut out, field)?;
                            wrote = true;
                        }
                    }
                    append(&mut out, "\n")?;
                }
            }
        }
        Ok(out)
    }
}
