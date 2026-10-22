//! Deliberate fixed-width arithmetic subset; no I/O or mutation of shell state.
#[derive(Clone, Debug)]
enum Token {
    Integer(i64),
    Name(String),
    Op(u8),
}
fn unsupported() -> String {
    "Unsupported arithmetic syntax; use integers, ordinary variables, + - * / %, and parentheses"
        .into()
}
fn integer(value: &str) -> Result<i64, String> {
    if value.len() > 4096 {
        return Err("Arithmetic integer value exceeds 4096-byte limit".into());
    }
    let value = value.trim_ascii();
    if value.is_empty() {
        return Ok(0);
    }
    let (negative, digits) = if let Some(s) = value.strip_prefix('-') {
        (true, s)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    let (radix, digits) = if let Some(s) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        (16, s)
    } else if digits.len() > 1 && digits.starts_with('0') {
        (8, &digits[1..])
    } else {
        (10, digits)
    };
    if digits.is_empty()
        || !digits
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && (b as char).is_digit(radix))
    {
        return Err("Unsupported arithmetic variable value or invalid integer; use an integer literal (leading 0 means octal), not an expression stored in a variable".into());
    }
    let magnitude = u64::from_str_radix(digits, radix).map_err(|_| {
        "Unsupported arithmetic integer magnitude; maximum literal is 18446744073709551615"
            .to_string()
    })?;
    let n = magnitude as i64;
    Ok(if negative { n.wrapping_neg() } else { n })
}
fn tokens(text: &str) -> Result<Vec<Token>, String> {
    if text.len() > 4096 {
        return Err("Arithmetic expression exceeds 4096-byte limit".into());
    }
    let bytes = text.as_bytes();
    let mut pos = 0;
    let mut result = Vec::new();
    while pos < bytes.len() {
        let c = bytes[pos];
        if c.is_ascii_whitespace() {
            pos += 1;
            continue;
        }
        if result.len() >= 256 {
            return Err("Arithmetic token limit exceeded (256)".into());
        }
        if c.is_ascii_digit() {
            let start = pos;
            pos += 1;
            while pos < bytes.len()
                && (bytes[pos].is_ascii_alphanumeric() || matches!(bytes[pos], b'_' | b'#' | b'@'))
            {
                pos += 1;
            }
            let value = &text[start..pos];
            if value.contains('#') {
                return Err(unsupported());
            }
            result.push(Token::Integer(integer(value)?));
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let start = pos;
            pos += 1;
            while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }
            result.push(Token::Name(text[start..pos].into()));
        } else if matches!(c, b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')') {
            if matches!(c, b'+' | b'-' | b'*') && bytes.get(pos + 1) == Some(&c) {
                return Err(unsupported());
            }
            result.push(Token::Op(c));
            pos += 1;
        } else {
            return Err(unsupported());
        }
    }
    Ok(result)
}
struct Parser<'a, F> {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
    variable: &'a F,
}
impl<F: Fn(&str) -> Result<String, String>> Parser<'_, F> {
    fn expression(&mut self, min: u8) -> Result<i64, String> {
        self.depth += 1;
        if self.depth > 24 {
            return Err("Arithmetic nesting limit exceeded (24)".into());
        }
        let left = match self.tokens.get(self.pos).cloned() {
            Some(Token::Integer(n)) => {
                self.pos += 1;
                n
            }
            Some(Token::Name(name)) => {
                self.pos += 1;
                integer(&(self.variable)(&name)?)?
            }
            Some(Token::Op(op @ (b'+' | b'-'))) => {
                self.pos += 1;
                let n = self.expression(3)?;
                if op == b'-' { n.wrapping_neg() } else { n }
            }
            Some(Token::Op(b'(')) => {
                self.pos += 1;
                let n = self.expression(0)?;
                if !matches!(self.tokens.get(self.pos), Some(Token::Op(b')'))) {
                    return Err("Arithmetic error: close the grouping parenthesis".into());
                }
                self.pos += 1;
                n
            }
            _ => {
                return Err(
                    "Arithmetic error: expected an integer or variable after the operator".into(),
                );
            }
        };
        let mut left = left;
        while let Some(Token::Op(op)) = self.tokens.get(self.pos) {
            let op = *op;
            let precedence = match op {
                b'+' | b'-' => 1,
                b'*' | b'/' | b'%' => 2,
                _ => break,
            };
            if precedence < min {
                break;
            }
            self.pos += 1;
            let right = self.expression(precedence + 1)?;
            left = match op {
                b'+' => left.wrapping_add(right),
                b'-' => left.wrapping_sub(right),
                b'*' => left.wrapping_mul(right),
                b'/' | b'%' if right == 0 => {
                    return Err(
                        "Arithmetic error: division by zero; check the divisor before calculating"
                            .into(),
                    );
                }
                b'/' | b'%' if left == i64::MIN && right == -1 => {
                    return Err(
                        "Unsupported arithmetic minimum-integer division/remainder by -1".into(),
                    );
                }
                b'/' => left / right,
                b'%' => left % right,
                _ => unreachable!(),
            };
        }
        self.depth -= 1;
        Ok(left)
    }
}
pub(super) fn evaluate(
    text: &str,
    variable: impl Fn(&str) -> Result<String, String>,
) -> Result<i64, String> {
    let tokens = tokens(text)?;
    if tokens.is_empty() {
        return Ok(0);
    }
    let mut parser = Parser {
        tokens,
        pos: 0,
        depth: 0,
        variable: &variable,
    };
    let n = parser.expression(0)?;
    if parser.pos != parser.tokens.len() {
        return Err("Arithmetic error: unexpected operand or unmatched parenthesis".into());
    }
    Ok(n)
}
/// End of a source arithmetic expansion, preserving its text for runtime expansion.
pub(super) fn end(chars: &[char], start: usize) -> Result<usize, String> {
    let mut pos = start + 3;
    let mut depth = 0usize;
    while let Some(c) = chars.get(pos) {
        match c{
            '('=>{depth+=1;if depth>24{return Err("Arithmetic nesting limit exceeded (24)".into());}},
            ')' if depth==0=>{
                if chars.get(pos+1)==Some(&')'){return Ok(pos+2);}
                return Err("Arithmetic error: close expansion with ))".into());
            }
            ')'=>depth-=1,
            '\''|'"'|'`'=>return Err("Unsupported quotes or substitution inside arithmetic; use ordinary integer variables".into()),
            _=>{},
        }
        pos += 1;
    }
    Err("Arithmetic error: close expansion with ))".into())
}
