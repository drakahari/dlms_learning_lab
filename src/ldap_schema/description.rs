use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Left,
    Right,
    Dollar,
    Word(String),
    Quoted(String),
}
#[derive(Clone, Debug)]
struct Spanned {
    token: Token,
    at: usize,
    end: usize,
}
struct Parser<'a> {
    raw: &'a str,
    tokens: Vec<Spanned>,
    pos: usize,
}
impl<'a> Parser<'a> {
    fn error(&self, category: Category, message: &'static str) -> Diagnostic {
        self.at_error(
            self.tokens.get(self.pos).map_or(self.raw.len(), |t| t.at),
            category,
            message,
        )
    }
    fn at_error(&self, at: usize, category: Category, message: &'static str) -> Diagnostic {
        let prefix = &self.raw[..at.min(self.raw.len())];
        Diagnostic {
            category,
            subject: None,
            line: prefix.bytes().filter(|b| *b == b'\n').count() + 1,
            column: prefix
                .rsplit_once('\n')
                .map_or(prefix.len(), |(_, last)| last.len())
                + 1,
            message: message.into(),
        }
    }
    fn new(raw: &'a str) -> Result<Self> {
        let mut p = Self {
            raw,
            tokens: Vec::new(),
            pos: 0,
        };
        if raw.len() > MAX_DESCRIPTION {
            return Err(p.error(
                Category::Limit,
                "A schema description allows at most 2048 bytes.",
            ));
        }
        let bytes = raw.as_bytes();
        let mut at = 0;
        while at < bytes.len() {
            let start = at;
            let b = bytes[at];
            if b == b'\r' {
                if bytes.get(at + 1) != Some(&b'\n') {
                    return Err(p.at_error(
                        at,
                        Category::Unsupported,
                        "Use LF or CRLF; bare carriage returns are unsupported.",
                    ));
                }
                at += 2;
                continue;
            }
            if matches!(b, b' ' | b'\t' | b'\n') {
                at += 1;
                continue;
            }
            if !(32..=126).contains(&b) {
                return Err(p.at_error(at,Category::Unsupported,"This description subset uses printable ASCII; non-ASCII and controls are unsupported."));
            }
            let token = match b {
                b'(' => {
                    at += 1;
                    Token::Left
                }
                b')' => {
                    at += 1;
                    Token::Right
                }
                b'$' => {
                    at += 1;
                    Token::Dollar
                }
                b'\'' => {
                    at += 1;
                    let mut value = String::new();
                    let mut closed = false;
                    while at < bytes.len() {
                        let b = bytes[at];
                        if b == b'\'' {
                            at += 1;
                            closed = true;
                            break;
                        }
                        if b == b'\\' {
                            let digits = bytes.get(at + 1..at + 3).ok_or_else(|| {
                                p.at_error(
                                    at,
                                    Category::Syntax,
                                    "A quoted escape needs exactly two hexadecimal digits.",
                                )
                            })?;
                            let hex = |b: u8| match b {
                                b'0'..=b'9' => Some(b - b'0'),
                                b'a'..=b'f' => Some(b - b'a' + 10),
                                b'A'..=b'F' => Some(b - b'A' + 10),
                                _ => None,
                            };
                            let decoded=hex(digits[0]).zip(hex(digits[1])).map(|(a,b)|a*16+b).ok_or_else(||p.at_error(at,Category::Syntax,"Use a two-digit hexadecimal escape inside a quoted string."))?;
                            if !(32..=126).contains(&decoded) {
                                return Err(p.at_error(
                                    at,
                                    Category::Unsupported,
                                    "Decoded quoted values must remain printable ASCII.",
                                ));
                            }
                            value.push(char::from(decoded));
                            at += 3;
                        } else {
                            if !(32..=126).contains(&b) {
                                return Err(p.at_error(at,Category::Unsupported,"Quoted strings cannot contain physical controls or non-ASCII in this subset."));
                            }
                            value.push(char::from(b));
                            at += 1;
                        }
                        if value.len() > 256 {
                            return Err(p.at_error(
                                start,
                                Category::Limit,
                                "A quoted description allows at most 256 decoded bytes.",
                            ));
                        }
                    }
                    if !closed {
                        return Err(p.at_error(
                            start,
                            Category::Syntax,
                            "Close the single-quoted string.",
                        ));
                    }
                    Token::Quoted(value)
                }
                _ => {
                    while at < bytes.len()
                        && !matches!(
                            bytes[at],
                            b' ' | b'\t' | b'\r' | b'\n' | b'(' | b')' | b'$' | b'\''
                        )
                    {
                        if !(32..=126).contains(&bytes[at]) {
                            return Err(p.at_error(
                                at,
                                Category::Unsupported,
                                "Use printable ASCII schema tokens.",
                            ));
                        }
                        at += 1;
                    }
                    if at - start > 128 {
                        return Err(p.at_error(
                            start,
                            Category::Limit,
                            "A schema token allows at most 128 bytes.",
                        ));
                    }
                    Token::Word(raw[start..at].into())
                }
            };
            if p.tokens.len() >= 256 {
                return Err(p.at_error(
                    start,
                    Category::Limit,
                    "A description allows at most 256 tokens.",
                ));
            }
            p.tokens.push(Spanned {
                token,
                at: start,
                end: at,
            });
        }
        Ok(p)
    }
    fn separator(&self) -> Result<()> {
        if self.pos > 0
            && self
                .tokens
                .get(self.pos)
                .is_some_and(|t| t.at == self.tokens[self.pos - 1].end)
        {
            return Err(self.error(
                Category::Syntax,
                "Separate schema fields and their values with whitespace.",
            ));
        }
        Ok(())
    }
    fn keyword(&mut self) -> Result<String> {
        self.separator()?;
        self.word().map(|s| s.to_ascii_uppercase())
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }
    fn punctuation(&mut self, expected: Token) -> Result<()> {
        if self.peek() != Some(&expected) {
            return Err(self.error(
                Category::Syntax,
                "Check the parentheses and list separators in this description.",
            ));
        }
        self.pos += 1;
        Ok(())
    }
    fn word(&mut self) -> Result<String> {
        let Some(Token::Word(value)) = self.peek() else {
            return Err(self.error(
                Category::Syntax,
                "An unquoted name or numeric OID is required here.",
            ));
        };
        let result = value.clone();
        self.pos += 1;
        Ok(result)
    }
    fn reference(&mut self) -> Result<String> {
        let value = self.word()?;
        if !numeric_oid(&value) && !descriptor(&value) {
            return Err(self.error(
                Category::Syntax,
                "Use a descriptor or a canonical dotted numeric OID.",
            ));
        }
        Ok(value.to_ascii_lowercase())
    }
    fn quoted(&mut self) -> Result<String> {
        let Some(Token::Quoted(value)) = self.peek() else {
            return Err(self.error(Category::Syntax, "Use a single-quoted string here."));
        };
        let value = value.clone();
        self.pos += 1;
        Ok(value)
    }
    fn names(&mut self) -> Result<Vec<String>> {
        self.separator()?;
        let list = self.peek() == Some(&Token::Left);
        if list {
            self.pos += 1;
        }
        if list && self.peek() == Some(&Token::Right) {
            return Err(self.error(
                Category::Unsupported,
                "Empty NAME lists are outside this subset; omit NAME for an OID-only definition.",
            ));
        }
        let mut names: Vec<String> = Vec::new();
        loop {
            if !names.is_empty() {
                self.separator()?;
            }
            let token = self
                .tokens
                .get(self.pos)
                .ok_or_else(|| self.error(Category::Syntax, "A NAME alias is missing."))?;
            if self.raw[token.at..token.end].contains('\\') {
                return Err(self.error(
                    Category::Syntax,
                    "Short NAME aliases use plain descriptor characters, without quoted escapes.",
                ));
            }
            let name = self.quoted()?;
            if !descriptor(&name) {
                return Err(self.error(Category::Syntax,"A short name starts with a letter and uses letters, digits or hyphens, up to 32 characters."));
            }
            if names.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                return Err(self.error(
                    Category::Conflict,
                    "Name aliases must be distinct, ignoring letter case.",
                ));
            }
            if names.len() >= 4 {
                return Err(self.error(
                    Category::Limit,
                    "At most four short-name aliases per description.",
                ));
            }
            names.push(name);
            if !list {
                break;
            }
            if self.peek() == Some(&Token::Right) {
                self.pos += 1;
                break;
            }
        }
        Ok(names)
    }
    fn references(&mut self) -> Result<Vec<String>> {
        if self.peek() != Some(&Token::Left) {
            return Ok(vec![self.reference()?]);
        }
        self.pos += 1;
        let mut result = Vec::new();
        loop {
            if result.len() >= 16 {
                return Err(self.error(Category::Limit, "At most sixteen names in a schema list."));
            }
            result.push(self.reference()?);
            if self.peek() == Some(&Token::Right) {
                self.pos += 1;
                break;
            }
            self.punctuation(Token::Dollar)?;
        }
        Ok(result)
    }
    fn start(&mut self) -> Result<String> {
        self.punctuation(Token::Left)?;
        let oid = self.word()?;
        if !numeric_oid(&oid) {
            return Err(self.error(Category::Syntax,"Start with a canonical dotted numeric OID; names cannot replace the definition's OID."));
        }
        Ok(oid)
    }
    fn finish(&mut self) -> Result<()> {
        self.punctuation(Token::Right)?;
        if self.peek().is_some() {
            return Err(self.error(
                Category::Syntax,
                "Each value must contain exactly one complete schema description.",
            ));
        }
        Ok(())
    }
    fn syntax(&mut self) -> Result<SyntaxReference> {
        let raw = self.word()?;
        let (oid, minimum_size_hint) = if let Some((oid, size)) = raw.split_once('{') {
            let size = size.strip_suffix('}').ok_or_else(|| {
                self.error(Category::Syntax, "Close the syntax size hint with }.")
            })?;
            if size.is_empty()
                || size.len() > 1 && size.starts_with('0')
                || !size.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(self.error(
                    Category::Syntax,
                    "A syntax size hint is a canonical nonnegative whole number.",
                ));
            }
            let size = size.parse::<usize>().map_err(|_| {
                self.error(
                    Category::Limit,
                    "The syntax size hint exceeds this model's value capacity.",
                )
            })?;
            if size > MAX_VALUE {
                return Err(self.error(Category::Limit,"This model stores at most 256 value bytes, so it cannot promise a larger minimum syntax capability."));
            }
            (oid, Some(size))
        } else {
            (raw.as_str(), None)
        };
        let kind=match oid {
            "1.3.6.1.4.1.1466.115.121.1.15"=>Syntax::DirectoryString,
            "1.3.6.1.4.1.1466.115.121.1.26"=>Syntax::Ia5String,
            "1.3.6.1.4.1.1466.115.121.1.27"=>Syntax::Integer,
            _=>return Err(self.error(if numeric_oid(oid){Category::Unsupported}else{Category::Syntax},"Supported syntax OIDs are Directory String, IA5 String and Integer; no other syntax is approximated.")),
        };
        Ok(SyntaxReference {
            kind,
            minimum_size_hint,
        })
    }
    fn order(&self, last: &mut u8, next: u8) -> Result<()> {
        if next <= *last {
            return Err(self.error(
                Category::Syntax,
                "Use each schema field once, in the documented LDAP description order.",
            ));
        }
        *last = next;
        Ok(())
    }
}
pub fn attribute_type(raw: &str) -> Result<AttributeType> {
    let mut p = Parser::new(raw)?;
    let oid = p.start()?;
    let mut result = AttributeType {
        oid,
        names: vec![],
        description: None,
        superior: None,
        equality: None,
        syntax: None,
        single_value: false,
    };
    let mut last = 0;
    while p.peek().is_some() && p.peek() != Some(&Token::Right) {
        let key = p.keyword()?;
        if matches!(
            key.as_str(),
            "NAME" | "DESC" | "SUP" | "EQUALITY" | "SYNTAX" | "MUST" | "MAY"
        ) {
            p.separator()?;
        }
        match key.as_str() {
            "NAME"=>{p.order(&mut last,1)?;result.names=p.names()?;},
            "DESC"=>{p.order(&mut last,2)?;result.description=Some(p.quoted()?);},
            "SUP"=>{p.order(&mut last,3)?;result.superior=Some(p.reference()?);},
            "EQUALITY"=>{p.order(&mut last,4)?;result.equality=Some(p.reference()?);},
            "SYNTAX"=>{p.order(&mut last,5)?;result.syntax=Some(p.syntax()?);},
            "SINGLE-VALUE"=>{p.order(&mut last,6)?;result.single_value=true;},
            _=>return Err(p.error(Category::Unsupported,"Supported attribute fields are NAME, DESC, SUP, EQUALITY, SYNTAX and SINGLE-VALUE. Obsolete, ordering, substring, usage, collective, operational and extension fields are unsupported.")),
        }
    }
    p.finish()?;
    if result.superior.is_none() && result.syntax.is_none() {
        return Err(p.error(
            Category::Syntax,
            "An attribute type needs a SYNTAX or an inherited SUP type.",
        ));
    }
    Ok(result)
}
pub fn object_class(raw: &str) -> Result<ObjectClass> {
    let mut p = Parser::new(raw)?;
    let oid = p.start()?;
    let mut result = ObjectClass {
        oid,
        names: vec![],
        description: None,
        superiors: vec![],
        kind: ClassKind::Structural,
        must: vec![],
        may: vec![],
    };
    let mut last = 0;
    while p.peek().is_some() && p.peek() != Some(&Token::Right) {
        let key = p.keyword()?;
        if matches!(
            key.as_str(),
            "NAME" | "DESC" | "SUP" | "EQUALITY" | "SYNTAX" | "MUST" | "MAY"
        ) {
            p.separator()?;
        }
        match key.as_str() {
            "NAME"=>{p.order(&mut last,1)?;result.names=p.names()?;},
            "DESC"=>{p.order(&mut last,2)?;result.description=Some(p.quoted()?);},
            "SUP"=>{p.order(&mut last,3)?;result.superiors=p.references()?;},
            "ABSTRACT"|"STRUCTURAL"=>{p.order(&mut last,4)?;result.kind=if key=="ABSTRACT"{ClassKind::Abstract}else{ClassKind::Structural};},
            "MUST"=>{p.order(&mut last,5)?;result.must=p.references()?;},
            "MAY"=>{p.order(&mut last,6)?;result.may=p.references()?;},
            _=>return Err(p.error(Category::Unsupported,"This class subset supports NAME, DESC, SUP, ABSTRACT/STRUCTURAL, MUST and MAY. AUXILIARY, obsolete, content rules and extensions remain unsupported.")),
        }
    }
    p.finish()?;
    Ok(result)
}
