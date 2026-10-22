//! Explicit-class ordinary mode operations; no I/O or recursive traversal.
pub(super) fn mode(spec: &str, original: u16) -> Result<u16, String> {
    let unsupported = || {
        "Unsupported chmod mode: use octal 000–777 or explicit u/g/o/a clauses with + - = and r/w/x (for example u=rw,g=r,o=)".to_string()
    };
    if spec.is_empty() || spec.len() > 512 {
        return Err(unsupported());
    }
    if spec.bytes().all(|c| (b'0'..=b'7').contains(&c)) {
        return u16::from_str_radix(spec, 8)
            .ok()
            .filter(|m| spec.len() <= 4 && *m <= 0o777)
            .ok_or_else(unsupported);
    }
    let mut result = original;
    let mut clauses = 0;
    for clause in spec.split(',') {
        clauses += 1;
        if clauses > 32 {
            return Err("chmod exceeds32 symbolic clauses".into());
        }
        let split = clause.find(['+', '-', '=']).ok_or_else(unsupported)?;
        let (who, change) = clause.split_at(split);
        if who.is_empty() {
            return Err(unsupported());
        }
        let mut classes = 0;
        for c in who.bytes() {
            classes |= match c {
                b'u' => 0o700,
                b'g' => 0o070,
                b'o' => 0o007,
                b'a' => 0o777,
                _ => return Err(unsupported()),
            };
        }
        let mut bits = 0;
        for c in change[1..].bytes() {
            bits |= match c {
                b'r' => 0o444,
                b'w' => 0o222,
                b'x' => 0o111,
                _ => return Err(unsupported()),
            };
        }
        bits &= classes;
        result = match change.as_bytes()[0] {
            b'+' => result | bits,
            b'-' => result & !bits,
            b'=' => (result & !classes) | bits,
            _ => unreachable!(),
        };
    }
    Ok(result)
}
